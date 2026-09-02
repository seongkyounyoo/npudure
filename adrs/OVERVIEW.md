# NPUDure architecture overview

*[한국어 원문](OVERVIEW.ko.md)*

The document to read before the ADRs. Its purpose is a single pass over **what
the whole system looks like**; the rationale for individual choices is left to
each ADR.

---

## 1. One sentence

A Rust runtime that spreads inference requests across three cheap edge NPU
boards and **measures whether three really becomes three times.**

## 2. What it does and does not do

```text
does                                does not
─────────────────────────────       ─────────────────────────────
spread independent requests         split one model across nodes
choose by node load                 make a single request 3x faster
drop dead nodes, readmit revived    Kubernetes-class orchestration
break the time down by stage        make three NPUs look like one
```

The right column is the **explicit non-goals**. The first two in particular keep
attracting "so why not do that?", so the rationale is written separately in
[ADR-001](001-data-parallel-only.md).

The core of it: **this system cannot make one request fast.** It only makes the
whole get through more when there are many requests.

## 3. Three layers

```text
+---------------------------------------------------------+
|  Client                                                 |
|  benchmark CLI . demo web . API clients calling directly |
+---------------------------+-----------------------------+
                            |  gRPC : Infer(model, image)
                            v
+---------------------------------------------------------+
|  Scheduler   (runs on a separate host, not a board)     |
|                                                         |
|   Node Registry   who is alive                          |
|   Scheduler       who gets this request                 |
|   Retry Manager   another node on failure               |
|   Health Monitor  drop from candidates when heartbeats stop |
+----------+--------------+--------------+----------------+
           |              |              |  gRPC
           v              v              v
    +-----------+  +-----------+  +-----------+
    |  king     |  |  queen    |  |  jack     |
    |  RK3576   |  |  RK3576   |  |  RK3576   |
    |  NPU 6TOPS|  |  NPU 6TOPS|  |  NPU 6TOPS|
    +-----------+  +-----------+  +-----------+
      each node holds the entire same model
```

**The three nodes are completely equivalent.** They use the same binary, the
same model file and the same configuration, differing only in the `id` and
address in the `[node]` section. There is no communication between nodes — they
do not even know the others exist.

The scheduler does not run on a board because loading the scheduler onto one
node contaminates the 1/2/3-node comparison the moment it happens.

## 4. The life of one request

```text
 1. Client ------------> Scheduler      one image (640x640x3 = 1.23 MB)
 2. Scheduler                           shortlist candidate nodes (exclude dead ones)
 3. Scheduler                           run the policy -> select a node
 4. Scheduler ---------> Node           forward to the selected node
 5. Node                                put on the local queue
 6. Node                                a worker picks it up and preprocesses
 7. Node                                NPU inference  <- only here is the NPU; the rest is CPU
 8. Node --------------> Scheduler      9 raw tensors as a single blob
 9. Scheduler ---------> Client         result + per-stage timings
```

The time taken at each arrow and each box is **recorded separately**.

```rust
scheduler_queue_us   scheduler_route_us   network_to_node_us
node_queue_us        decode_us            preprocess_us
npu_input_us         inference_us         postprocess_us
network_to_client_us end_to_end_us
```

This breakdown is close to the project's reason for existing. "Three nodes only
reached 2.4×" is not an answer; **which field it leaks from** is.

> Step 7 is the NPU section and everything else is CPU. And in measurement,
> **what collapses first is the CPU, not the NPU** — throughput falls 27% over
> 300 seconds of sustained load while the NPU clock stays pinned at 950 MHz and
> the CPU is downgraded from A72 2208 to 816 MHz.

### On failure

```text
failure at 4 --> classify cause --> retryable? --> drop that node from candidates
                                                --> retry on another node
                                                --> NPF-1302 if all fail
```

It never throws the request back at the same node. A node that just failed is
likely to fail on the next attempt too.

## 5. Crate map

```text
                    npuforge-common
                    types . error codes . config . InferenceBackend interface
                            | everything references this
        +-------------------+-------------------+
        |                   |                   |
 npuforge-scheduler   npuforge-node      npuforge-bench
 3 policies . registry worker pool . queue  load generation . aggregation
        |                   |
        +---- npuforge-proto +          gRPC definitions (.proto -> tonic)
                            |
                 +----------+----------+
                 |                     |
        npuforge-rknn          npuforge-mock-backend
        the real NPU. all of      a fake backend that runs without hardware
        the unsafe lives here     deterministic seed . latency/error injection
```

| Crate | One line |
|---|---|
| `npuforge-common` | The types and interfaces everyone shares. This is the contract |
| `npuforge-proto` | gRPC service definitions |
| `npuforge-scheduler` | Decides which node to send to, and resends on failure |
| `npuforge-node` | The agent running on a board. Queue + worker pool |
| `npuforge-rknn` | RKNN Runtime FFI. **The `unsafe` containment zone** |
| `npuforge-mock-backend` | An NPU imitation. For running everything without hardware |
| `npuforge-bench` | Applies load, produces statistics and **judges whether this run is valid** |

The two backends implement the same `InferenceBackend` interface. So **without a
single RK3576 board**, `cargo test --workspace` passes and a 3-node cluster runs
locally. This is a design principle, not a convenience feature.

## 6. Physical setup

```text
current (cannot measure)             planned (M3)

 management 1GbE                      Scheduler server
   |-- king                              | 10GbE  <- aggregation
   |-- queen                             |
   |-- jack                        2.5G/10G switch
   \-- dealer (scheduler, laptop)     |-2.5G- king
                                      |-2.5G- queen
 no inference network. switch not bought  \-2.5G- jack
```

**The worker links are fine at 2.5G and only aggregation needs 10G**, because
the three nodes' traffic converges at one point. Measuring now would measure
link saturation rather than NPU scaling efficiency, so M3 has not started.

The current scheduler host, `dealer`, is a laptop with no PCIe slot and cannot
take a 10G NIC. A separate server is needed.

## 7. Where things stand

| | Status |
|---|---|
| Software skeleton | ✅ 209 tests, clippy `-D warnings`, fmt clean |
| Single-node measurement | ✅ INT8 157.2 inf/s / FP16 84.3 inf/s (8 threads, 120 s) |
| Mock 3-node cluster | ✅ connects over real gRPC. Without hardware |
| Real 3-node hardware | ⬜ **halted, waiting on network equipment** |
| Prometheus and dashboard | ⬜ |

What blocks it is equipment, not code. The detailed resumption procedure is at
the top of `docs/TODO.md`.

## 8. Where to read more

| Question | Where |
|---|---|
| **The full list of decisions** | **[README.md](README.md)** |
| Why is the model not split | [ADR-001](001-data-parallel-only.md) |
| Why is there only one scheduler | [ADR-003](003-central-simple-scheduler.md) |
| Why does everything run without hardware | [ADR-004](004-backend-abstraction-mock-first.md) |
| Why an NPU context per thread | [ADR-007](007-per-thread-rknn-context.md) |
| Why INT8 | [ADR-011](011-int8-quantization.md) |
| Why does the node send integers rather than floats | [ADR-012](012-want-float-zero-blob-v2.md) |
| Why no fan | [ADR-013](013-fanless-thermal-as-measurement.md) |
| Why wait instead of measuring now | [ADR-014](014-10g-aggregation-separate-scheduler.md) |
| Why run preflight before measuring | [ADR-015](015-preflight-hard-fail.md) |
| What does what (the full specification) | `docs/01-TECHSPEC.md` |
| What numbers came out | `docs/RESULTS.md` |
| What to do now | `docs/TODO.md` |
| The final authority for values | `docs/environment-matrix.md` |
