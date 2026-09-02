# FAQ

Answers to the questions this project actually gets asked. Every answer states
the number, the conditions it was measured under, and where the raw data is.
Where something is unmeasured, it says so instead of estimating.

> **NPUDure is an open-source Edge NPU Cluster runtime for scaling distributed
> AI inference across low-cost NPUs over standard Ethernet.**

**The measurement base for everything below**

| | |
|---|---|
| Nodes | 3 × NanoPi R76S — Rockchip RK3576, 6 TOPS NPU each |
| Network | 2.5 GbE per node |
| Scheduler host | Dell PowerEdge R620, dual Xeon E5-2630L, 24 threads, 10 GbE |
| Workload | YOLOv8n INT8, 640×640×3 RGB in, raw tensor blobs out |
| Transport | gRPC over HTTP/2 (tonic), plaintext |
| Runs | **421 valid runs, zero inference errors** |
| Scale tested | **1, 2 and 3 nodes. 4+ is not measured.** |

Ledger and raw-data map: [`docs/experiments/README.md`](experiments/README.md).

---

## 1. What is an Edge NPU Cluster?

**Several low-cost edge NPU boards, each running a full copy of the model,
fronted by a scheduler that spreads independent inference requests across them
over ordinary Ethernet.**

It is data parallelism at the request level. Each board does whole inferences;
nothing is split across boards. The cluster raises how many requests per second
you can serve. It does not make the boards behave as one larger accelerator.

The contrast worth drawing is with model parallelism — layer-wise partitioning
or LLM tensor parallelism — where one request is split across devices and the
interconnect sits inside the model's critical path. That is a different problem
with different interconnect requirements, and it is not what this is.

| | |
|---|---|
| Source | [`README.md`](../README.md) § What this is, and isn't |

---

## 2. Can multiple NPUs be combined for inference?

**For throughput, yes, and it works well. For a single request, no.**

Independent requests distribute cleanly: three nodes served 3.00× the requests
one node did, with even distribution and no errors. But any one request is
still handled start to finish by one NPU, so the time that request takes is
unchanged — see [question 4](#4-does-npudure-reduce-single-request-latency).

"Combining NPUs" is two different asks that get one name. This project answers
the throughput one and is explicit about not answering the other.

| | |
|---|---|
| Measured | 112.9 → 338.4 inf/s going from 1 to 3 nodes |
| Conditions | YOLOv8n INT8, concurrency 8/node, active cooling, 30 runs |
| Source | [S2](experiments/S2_GRPC_BASELINE.md) · raw: [`results/baseline-20260820/`](../results/baseline-20260820/) |

---

## 3. Does 3 × 6 TOPS equal 18 TOPS?

**No. But on this configuration the throughput scaling was 3.00×, which is
closer to the naive sum than we expected.**

Two separate things get conflated here.

**TOPS do not add.** 18 TOPS is a datasheet arithmetic that assumes zero
distribution cost and perfectly divisible work. Neither holds.

**Throughput did scale near-linearly anyway** — but from a baseline that has
already paid for being a cluster. One board doing local direct inference
reaches **161.5 inf/s**. That same board serving through the cluster's gRPC
path reaches **112.9 inf/s** — a **30.1% throughput reduction** before any
scale-out happens. Three nodes then multiply that reduced baseline by 3.00×.

So the honest phrasing is: near-linear scaling on top of a real per-node
distribution cost, not 18 TOPS.

| | |
|---|---|
| Measured | 112.9 ± 0.5 / 229.0 ± 0.9 / 338.4 ± 1.1 inf/s at 1 / 2 / 3 nodes → **3.00×** |
| | Local direct 161.5 inf/s vs cluster single-node 112.9 inf/s → **−30.1%** |
| Conditions | Concurrency 8 per node, active cooling, 30 runs, error rate 0 |
| Source | [S2](experiments/S2_GRPC_BASELINE.md) · raw: [`results/baseline-20260820/`](../results/baseline-20260820/) |

---

## 4. Does NPUDure reduce single-request latency?

**No, and it cannot. It raises throughput and adds latency to each request.**

A request goes to exactly one node. Adding nodes gives you more requests in
flight at once; it does nothing for the one in front of you. The distribution
path — serialization, a 1.2 MB payload each way, HTTP/2 framing, the network —
is added to every request. The 1.2 MB round trip alone costs roughly **8.2 ms**
on a 2.5 GbE link.

If single-request latency is what you need, a cluster is the wrong tool.

| | |
|---|---|
| Conditions | 640×640×3 RGB request = 1,228,800 B; response blobs ≈ 1,218,000 B |
| Source | [`README.md`](../README.md) § What this is, and isn't · [ADR-008](../adrs/008-grpc-tonic-protobuf.md) |

---

## 5. How well does RK3576 scale?

**3.00× at three nodes — about 98.9% scaling efficiency — on the untuned
baseline.**

Tuning the transport raised absolute throughput and *lowered* efficiency at the
same time. Connection tuning took three-node throughput from 341.8 to
**387.2 inf/s (+13.3%)**, while scaling efficiency fell from **98.9% to 95.3%**
(3.00× → 2.86×).

Both numbers are true and they move in opposite directions. Quoting either one
alone misrepresents the result, so this repository always reports both with the
operating point attached.

| | |
|---|---|
| Measured | Baseline 338.4 inf/s, 3.00×, eff 98.9% |
| | Tuned 387.2 inf/s, 2.86×, eff 95.3% (+13.3% throughput) |
| Conditions | 3 nodes, YOLOv8n INT8, concurrency 12/node at the operating point |
| Source | [S2](experiments/S2_GRPC_BASELINE.md) · [S3.8](experiments/S3_8_OPTIMIZED_SCALEOUT.md) · raw: [`results/scaleout-optimized-20260820/`](../results/scaleout-optimized-20260820/) |

---

## 6. Why gRPC instead of RDMA?

**RDMA was never a candidate, and the measurements argue that a lower-overhead
transport would not have bought much here.**

Being straight about this: [ADR-008](../adrs/008-grpc-tonic-protobuf.md)
compared three options — REST+JSON, gRPC, and a hand-rolled binary protocol.
RDMA was not among them and was not benchmarked, so this is not a measured
comparison. gRPC won on binary payloads without base64 inflation, schema
enforcement, and code generation.

What *is* measured bears on the question indirectly. Profiling the transport
for a planned io_uring port found that transport CPU is a cost, not a
constraint — the boards were **48.9% idle under load with no core saturated**.
Cutting transport overhead reduces consumption of a resource that was never the
limit. See [question 9](#9-why-wasnt-io_uring-implemented).

The design goal was also explicit: ordinary Ethernet and a standard transport,
so the result transfers to hardware people already own. Whether RDMA-class
interconnect changes the picture on this workload is **unmeasured**.

| | |
|---|---|
| Measured | Board CPU 48.9% idle under load, no core saturated |
| Source | [ADR-008](../adrs/008-grpc-tonic-protobuf.md) · [S3.9b](experiments/S3_9B_NODE_RESIDUAL.md) |

---

## 7. What limits scale-out efficiency?

**The tail. Median latency does not move at all; p95 and p99 do.**

Going from one node to three at the tuned operating point:

```text
p50   flat        (+0%)
p95   119.7 → 147.4 ms   (+23%)
p99   137.9 → 187.6 ms   (+36%)
```

The mean rises with the tail, and under closed-loop load a higher mean is
directly less throughput. Ideal three-node scaling from the 135.5 inf/s
operating point would be 406.5 inf/s; the measured value was 387.2 —
**19.3 inf/s short**, and that shortfall is entirely tail-shaped.

**Separately, there is a per-node gap we did not explain.** Local direct
inference reaches 161.5 inf/s against 135.5 at the operating point — a
**16.1% residual**. It looks like path latency rather than CPU cost, but we did
not pin it down, and `perf` is unavailable on these boards (vendor kernel), so
there is no symbol-level profile.

| | |
|---|---|
| Measured | p50 +0%, p95 +23%, p99 +36% going 1 → 3 nodes |
| | Residual per-node gap 161.5 → 135.5 inf/s = 16.1%, unexplained |
| Conditions | Tuned transport, operating point, 3–4 runs per configuration |
| Source | [S3.9a](experiments/S3_9A_SCALEOUT_PROFILE.md) · [S3.9b](experiments/S3_9B_NODE_RESIDUAL.md) |

---

## 8. Why did load-aware scheduling perform worse?

**It wasn't the policy. Our scheduler was herding on stale heartbeat state — a
bug in our own default configuration.**

Load-aware policies *collapsed* throughput by **55–58%** against plain round
robin. The policies were deciding from heartbeat data that had already gone out
of date, so every scheduler instance picked the same "idle" node at the same
moment and piled onto it.

After switching the decision input to a locally-tracked in-flight counter,
adaptive scheduling **cut p99 latency by 37%** and evened the per-node latency
spread from **1.33× to 1.00×**.

The lesson generalises past this codebase: a load-aware policy is only as good
as the freshness of what it reads. It took a policy A/B to find this — the
symptom looked like "load-aware scheduling doesn't work here".

| | |
|---|---|
| Measured | Stale state: −55 to −58% throughput vs round robin |
| | After fix: p99 −37%, node spread 1.33× → 1.00× |
| Conditions | 3 nodes, deliberate heterogeneity fixture |
| Source | [S0-C](experiments/S0_C_POLICY_AB.md) · raw: [`results/policy-ab-20260821/`](../results/policy-ab-20260821/) |

---

## 9. Why wasn't io_uring implemented?

**We profiled it, found the reachable gain was about 8% of transport cost, and
did not build it.**

The plan was: profile CPU, measure syscall and copy cost, then implement
io_uring. We did the first two and the numbers ended the third.

```text
transport cost          16.35 CPU-ms per request
  user   9.37 ms (57%)  serialization, user-space copy, HTTP/2 framing
  kernel 6.99 ms (43%)  syscall entry, TCP stack, copy_to_user

network syscalls        ~165 per request
syscall entry           ~0.17 ms = 1.0% of transport cost
board CPU under load    48.9% idle, no core saturated
```

Even granting that io_uring eliminates the 1.2 MB copy in both directions, the
total reachable slice is about **8%** of transport cost. And recovering it buys
nothing, because **CPU here is a cost, not a constraint** — reducing
consumption of an unsaturated resource does not raise throughput.

This is a conditional exclusion, not a permanent verdict. If the workload
becomes CPU-bound on the boards, it reopens.

| | |
|---|---|
| Measured | Reachable slice ≈ 8% of a 16.35 CPU-ms/request transport cost |
| Conditions | RK3576 boards under load, 48.9% CPU idle, `/proc/PID/stat` split |
| Source | [S3.9b](experiments/S3_9B_NODE_RESIDUAL.md) · raw: [`results/node-residual-20260821/`](../results/node-residual-20260821/) |

---

## 10. Can NPUDure scale beyond three nodes?

**We do not know. Four or more nodes was never measured, and we are not going
to extrapolate 3.00× into a claim about node four.**

Three nodes is what the hardware budget covered. Everything in this repository
is bounded by that.

Two measured facts do point at where the next wall probably is, and neither is
the NPU:

- **The scheduler host is a real bottleneck.** Thread count on it matters more
  than clock speed. When the host was swapped, a faster-per-core 8-thread
  desktop produced **7.5% less** throughput than the 24-thread server, because
  the load generator shares that CPU with the scheduler. Sixteen threads or
  more is what we would recommend.
- **Traffic converges at the scheduler.** Three nodes at 2.5 GbE each converge
  on one host; 2.5 GbE there is not enough, which is why the scheduler sits on
  10 GbE.

So the honest expectation is that scale-out runs into scheduler host capacity
and its network before it runs into the boards — but **that is reasoning, not a
measurement**, and it is exactly the kind of claim this repository otherwise
refuses to make.

If you run this on four or more nodes, we would like to see the data.

| | |
|---|---|
| Measured | 8-thread desktop scheduler host: −7.5% throughput vs 24-thread server |
| Unmeasured | 4+ nodes, at any configuration |
| Source | [`docs/02-HARDWARE-SETUP.md`](02-HARDWARE-SETUP.md) §3.3.4 · [`docs/infrastructure.md`](infrastructure.md) §3.2.1 |

---

## Reading the numbers in this file

Four caveats apply to everything above, and they are the same ones in the
README limitations:

- **Three nodes only.** Whether any conclusion holds at four or more is
  unmeasured.
- **Most configurations are 3–4 runs.** Percentile differences have small SD
  and are usable. **Throughput differences under 1% were never used to rank
  anything.**
- **Percentiles are run-level averages, not pooled.** This dilutes each run's
  worst window, so tail numbers read low. Valid for comparing conditions,
  invalid as "the p99 of this system".
- **No authentication, no TLS.** Scoped for a trusted private network. A
  boundary, not a defect.
