# NPUDure measurement results — first pass

*[한국어 원문](RESULTS.ko.md)*

- Compiled: **2026-08-14** (the single-node lineage)
- Period covered: 2026-08-07 to 2026-08-12
- Raw data: `results/`, `benchmarks/`
- The discussion: `discuss.md` (chronological); work history:
  `board-worklog.md`

> ## ⚠️ This document is **the first pass over the single-node lineage** (updated 2026-08-21)
>
> On 2026-08-20 and 21 **the cluster measurement lineage ran through to
> completion** (S2 · S3 · S3.5–3.9b · S0-A–D, 421 runs, error rate 0). Those
> results are not here but in
> **[`experiments/README.md`](experiments/README.md)**. Start there for
> multi-node figures.
>
> §2.5 below is **a pilot measurement from before that lineage** and has been
> **superseded by later measurements.** The single-node results in §1–§4 and the
> failure list in §5–§6 remain valid.

> This document collects **results only**. Why those conclusions were reached is
> in `discuss.md`; what happened is in `board-worklog.md`.
>
> **Every number carries its measurement conditions.** We learned this time that
> a number without conditions is useless three months later.

---

# 1. One page

A distributed inference cluster was built from three RK3576 boards (6 TOPS NPU).
This document covers **single-node characteristics and the software.** The
multi-node lineage ran separately on 2026-08-20 and 21 and is complete —
**three nodes at 387.2 inf/s (operating point, 2.86× / 95.3%)**,
[`experiments/README.md`](experiments/README.md).

The three most important numbers so far.

| Item | Value | Meaning |
|---|---|---|
| Per-node throughput (120 s) | **84.3 inf/s** (FP16) / **157.2 inf/s** (INT8) | on `want_float=0` |
| Per-node **steady state** (300 s) | **59.7 inf/s** (FP16) | **−27%** against the start. CPU throttling |
| Two application optimizations | **+0.1%, −1.8%** (`core_mask`, zero-copy) | there is almost nothing left to squeeze inside the node |
| INT8 quantization | **1.86×** | the measure that landed hardest |
| `want_float=0` | **+17.3%** (INT8) / **+15.7%** (FP16) | output a quarter too. Network and throughput move together |

And the most important **non-numeric** result.

> **Four measurements were wrong, and all four "looked like success."**
> Half this project's output is that list of failures. See §6.

---

# 2. Settled figures

## 2.1 Hardware

| Item | Value | Source |
|---|---|---|
| SoC | Rockchip RK3576 | measured |
| CPU | 4× Cortex-A72 @2208 MHz + 4× A53 @2016 MHz | `cpufreq` |
| NPU | 2 cores, 950 MHz, 6 TOPS (nominal) | `/sys/kernel/debug/rknpu` |
| RAM | 4GB LPDDR4X | measured |
| Power | 5V DC. **A 4A adapter is required** | §6.2 |
| Network | 2.5GbE × 2 (r8125) | measured |
| RKNN Runtime | 2.3.0 (`c949ad889d@2024-11-07`) | `strings librknnrt.so` |
| RKNPU driver | v0.9.8 | `/sys/kernel/debug/rknpu/version` |

The three nodes' kernel (6.1.141), `librknnrt.so` hash, driver version and model
hash all match. `preflight-check.sh` confirms this before every measurement.

## 2.2 Inference performance

**Conditions: `king`, 8 threads, 120 s sustained, CPU governor `performance`,
fanless, a dedicated RKNN context per thread.**

| Model | Throughput | Mean latency | Model size |
|---|---:|---:|---:|
| YOLOv8n FP16 | **84.3 inf/s** | 94.5 ms | 9.65 MB |
| YOLOv8n INT8 | **157.2 inf/s** | 50.8 ms | 6.46 MB |
| Ratio | **1.86×** | −46% | −33% |

Related figures.

| Item | Value | Conditions |
|---|---|---|
| Optimal `worker_count` | **8** | +27% over 4. Has not bent yet at 8 |
| Contribution of the NPU's second core | **1.51×** (not 2×) | single core 48.2 → two cores 73.0 inf/s |
| Kernel ioctls per inference | **76** | identical for FP16 and INT8 |
| CPU governor effect | +7% | `ondemand` → `performance`. **A 120-second measurement.** Unverified under sustained load |
| `want_float=0` effect | **INT8 +17.3% / FP16 +15.7%** | output becomes a quarter too |

> ⚠️ **Throughput figures in documents from before 2026-08-11 are on
> `ondemand`** (FP16 79.0 / INT8 146.2). Do not compare directly.

## 2.3 Thermal characteristics (pilot, not S0)

**Conditions: 3 boards concurrently, 8 threads, 900 s, fanless, no desk fan,
governor `ondemand`.** The plateau is from 300 s after load to the end.

| Board | NPU mean | NPU peak | Throughput |
|---|---:|---:|---:|
| king | 73.0 °C | 75.8 °C | 80.5 inf/s |
| queen | 67.5 °C | 70.2 °C | 77.7 inf/s |
| jack | 72.6 °C | 74.8 °C | 77.8 inf/s |

- **Node-to-node spread 5.6 °C**
- **No NPU throttling** — all 928 samples at 950 MHz
- ⚠️ **But the CPU is downgraded.** That verdict looked only at the NPU clock.
  The CPU clocks in the same log show A72 2208 → 816 MHz and A53 2016 → 600 MHz.
  `discuss.md` §12
- Never exceeded 90 °C. Did not reach the current thresholds
  (`degraded 80` / `disable 90`)
- Idle temperature 35–40 °C, minimum input voltage 5.046 V

**Sustained 8-thread load is possible fanless.** It completes without errors.
But **throughput is not sustained** — over 300 seconds it goes 81.6 → 59.7 inf/s
(−27%). Because it is the CPU, not the NPU, being downgraded by heat.
`discuss.md` §12

The formal S0 (30 min × 2 conditions) is still to come. This is a 15-minute
measurement for checking node-to-node spread.

## 2.4 Accuracy

**Conditions: real board (`king`), one COCO val2017 image, preprocessing done in
one place so both see the same input bytes.**

| Comparison | box cosine | Detection cells | Class agreement |
|---|---|---|---|
| FP16 vs ONNX | 0.99999 | 10/10 | 100% |
| INT8 vs FP16 | 0.997 | 10/10 | 100% |

For INT8 the top detection's cell moves by one and its score is −5.5%. **The
detection set and classes are identical.** Getting 1.86× for that price is worth
it.

The RKNN simulator cannot infer with a built `.rknn` (after `load_rknn`,
`init_runtime` refuses). Verification has to use the same file that gets
deployed, so it was measured on a real board.

## 2.5 Multi-node scalability (2026-08-20, pilot) — **superseded**

> **This section has been superseded by later measurements.** It is a
> single-run pilot; the formal results are below. When quoting values, use the
> individual experiment documents rather than this section.
>
> | This section (pilot) | Formal | Document |
> |---|---|---|
> | 3 nodes 337.7 inf/s | **338.4** (30 runs) / ceiling **341.8** | [S2](experiments/S2_GRPC_BASELINE.md) · [S3](experiments/S3_SATURATION.md) |
> | scaling efficiency ~98% | **98.9%** (baseline) / **95.3%** (operating point) | [S2](experiments/S2_GRPC_BASELINE.md) · [S3.8](experiments/S3_8_OPTIMIZED_SCALEOUT.md) |
> | node ceiling 115 | operating point **135.5** (2 connections/node) | [S3.7](experiments/S3_7_CONNECTION_TUNING.md) |
> | −27% against local 157 | **−16.1%** against local direct **161.5** | [S3.9b](experiments/S3_9B_NODE_RESIDUAL.md) |
>
> The cooling-condition mismatch flagged by the ⚠️ below is also resolved —
> since S3.5 the local direct reference has been unified as **161.5 with active
> cooling and 8 workers.**

**Conditions: gRPC via the scheduler (server .9), INT8, want_float=0,
governor=performance, active cooling (a dedicated fan per node), round-robin,
30 s (20 s for the 1-node sweep), a single run, preflight passed.** Not the
formal S2 — no repeated runs, no fanless comparison, no `--with-inference`. Raw
data `results/scaling-20260820/`.

Equal per-node load (concurrency = 8 × node count):

| Configuration | Throughput | Distribution | Error rate |
|---|---:|---|---:|
| 1 node | 111.6 inf/s | king 100% | 0% |
| 2 nodes | 228.7 inf/s | 50 / 50 | 0% |
| 3 nodes | **337.7 inf/s** | 33 / 33 / 33 | 0% |

1-node concurrency sweep (the saturation point): c8 111.6 → c16 114.0 → c32
**115.1 (saturated)**.

**Scaling efficiency ~98% (nearly linear).** Against the 1-node saturation of
115, three nodes at 337.7 is **2.93×**. Data parallelism (`adrs/001`) holds and
the scheduler is not a bottleneck even with three nodes.

**But the cluster node ceiling of 115 < the local sustained 157 (−27%).** The
round-trip p50 is 69 ms while the node reports inference at 24–28 ms — 40 ms+
appears to be overhead from going through the scheduler's gRPC (serialization +
transferring 1.17 MiB in and out + queueing and routing).

> ⚠️ **The 27% cannot be attributed purely to gRPC overhead.** The reference of
> 157 is **fanless** sustained (08-11/12) while the cluster's 115 is **active
> cooling** (today), so the cooling conditions differ. It gets settled after
> re-measuring the local baseline under the same fan conditions.

> **The first answer to the central question, "do three 6 TOPS units make
> 18 TOPS": 2.93× (98%) on a cluster basis.** The bottleneck is not scaling but
> per-node overhead. The source of that 27% gets broken down next with the
> `TimingBreakdown` stages. `board-worklog.md` §2.25.

Raw data and detailed report: `results/scaling-20260820/`.

> ✅ **Reproduced across 30 runs (2026-08-20).** 1/2/3 nodes = 112.9±0.5 /
> 229.0±0.9 / **338.4±1.1 inf/s**, speedup 3.00×, error 0%, balance 0 pp. Tiny
> SD. Promoted from "a value that came out once" to "a repeatedly confirmed
> result". Experiment report: `docs/experiments/S2_GRPC_BASELINE.md`, raw data:
> `results/baseline-20260820/`.
>
> ✅ **S3 saturation (2026-08-20).** Each configuration's ceiling =
> **115 / 232 / 342 inf/s** (1/2/3 node), 3N = 2.97× (99%). Near-linear by the
> ceiling measure too. `docs/experiments/S3_SATURATION.md`.
>
> ✅ **S3.5 transport profiling (2026-08-20).** Settled what the −30% loss above
> actually is. **Not bandwidth** (51% of the link used per direction), **not
> board CPU capacity** (63% idle), **not kernel softirq concentration** (RPS A/B
> −0.2%), **not the server or scheduler** (three nodes scaling 3.00×). What
> remains is the **scheduler↔node HTTP/2 transport path** — the bench uses 32
> connections, one per worker, while the scheduler funnels to one connection per
> node, and the h2 windows are all at the default (64 KB). The node yields
> 161.5 inf/s local direct on the same board while managing only 116 in the
> cluster, with `node_queue` ≈ 0 showing headroom. Which of ①flow control
> ②connection/TCP ③protobuf and copies it is within that path **gets separated
> by the S3.6 A/B.** That result fixes S4 as either `gRPC optimized` or
> `io_uring`.
> `docs/experiments/S3_5_TRANSPORT_PROFILE.md`.
>
> ✅ **S3.6 H2/channel A/B (2026-08-20, 20 runs).** **Confirmed the single
> gRPC/HTTP2 connection per node as the primary factor limiting throughput.**
> Changing connections 1 → 4 alone gave **115.3 → 140.1 inf/s (+21.5%)**,
> recovering **54% of the 46.2 gap to local direct through configuration alone**
> (a connection pool, with no architectural change). But which of TCP per-flow /
> H2 multiplexing and locking / flow control interaction it is within that
> structure is **still unseparated**.
> Meanwhile **a 64 MB-class large window was badly harmful on this workload at
> −36.3%** — meaning the 64 KB default was functioning as backpressure (the
> default is kept). But 64 KB→64 MB is a 1000× extreme A/B, so it is **not
> concluded as "window tuning is ineffective"** (intermediate values unmeasured).
> The cost is **p95 46% worse** (393 → 573 ms) — five candidate causes, all
> unverified.
> CPU0 saturation surfaced as a new bottleneck (busy 81%, soft 74%), and with
> four flows there is now room to revisit S3.5b's RPS null.
> **io_uring is pushed back in the ordering** — syscall/req is nearly invariant
> at 80–94 across conditions while throughput differs twofold, 73.5–140.1. (The
> syscall *count* being equal and the syscall/copy *CPU time* being small are
> different questions, so this does not mean it has no effect.)
> **The S2 and S3 numbers are not updated yet** — they are 1-node results, and
> at three nodes the server would hold 12 connections. 1N/2N/3N get re-measured
> after S3.7 fixes N.
> `docs/experiments/S3_6_H2_CHANNEL_AB.md`.
>
> ✅ **S3.7 + S3.8 optimized gRPC (2026-08-20, 146 runs total).** The operating
> point was settled at **2 connections per node @ concurrency 12** and scale-out
> re-verified. The operating-point definition is **the lowest concurrency
> delivering at least 98% of peak.**
>
> | | baseline (conn1) | optimized (conn2/node) | gain |
> |---|---:|---:|---:|
> | 1N | 115.2 | **135.5** | +17.6% |
> | 2N | 232.0 | **263.3** | +13.5% |
> | 3N | 341.8 | **387.2** | +13.3% |
> | scaling | 2.97× (98.9%) | **2.86× (95.3%)** | |
>
> Zero errors, 0.0 pp distribution deviation. But **scaling efficiency slipped
> slightly** — the per-node gain shrinks from +17.6% (1N) to +13.3% (3N), so the
> single-node optimization is not fully preserved at multiple nodes. The server
> 10G link rising from 67% to **76%** is the leading candidate but is
> unconfirmed → next is a **server-side profile**.
> `docs/experiments/S3_7_CONNECTION_TUNING.md`, `S3_8_OPTIMIZED_SCALEOUT.md`.

---

# 3. Software status

| Crate | Status | Note |
|---|---|---|
| `npuforge-common` | ✅ | types, error codes (16 kinds), configuration, backend interface |
| `npuforge-proto` | ✅ | gRPC definitions. `SchedulerService` / `NodeService` |
| `npuforge-mock-backend` | ✅ | deterministic seed. Development without hardware |
| `npuforge-rknn` | ✅ | context pool, multiple outputs, real-hardware verification |
| `npuforge-node` | ✅ | worker pool, gRPC server, registration and heartbeat |
| `npuforge-scheduler` | ✅ | three policies, registry, retries |
| `npuforge-bench` | ✅ | load, aggregation, run-validity judgement |

**209 tests, clippy `-D warnings`, fmt clean.** (as of 2026-08-14)

## 3.1 Verified behaviour

The local Mock 3-node integration test
(`crates/npuforge-scheduler/tests/mock_cluster.rs`). It runs over real gRPC, so
the transport path is the same as on real hardware.

| Item | Result |
|---|---|
| Requests spread across 3 nodes | ✅ round-robin |
| Bypass when 1 node dies | ✅ 6/6 succeeded |
| All nodes dead | ✅ `NPF-1302` + the list of nodes attempted |
| Timing breakdown | ✅ both node and scheduler sections |
| Avoiding a slow node | ✅ least-queue |

Also confirmed with four real processes (scheduler + 3 nodes). **Killing the
scheduler and bringing it back has all three nodes re-register by themselves
within about 1.3 seconds.**

The six real-hardware RKNN integration tests
(`crates/npuforge-rknn/tests/real_device.rs`) pass too — returning 9 outputs,
determinism, uncontaminated results under 4-thread concurrent inference, and so
on.

## 3.2 Not implemented

- Prometheus metrics
- The REST management API and dashboard
- JPEG decoding (currently raw RGB8/BGR8 only)
- Postprocessing (NMS). The node returns raw tensors as-is

---

# 4. Inverted conclusions

**Things whose conclusion changed on re-measurement.** Publishing the first
conclusion would have meant saying something wrong.

## 4.1 "king runs 19 °C hotter" → did not reproduce

Under 08-10 sustained load, `king` alone hit NPU 91.3 °C, 19 °C above the other
two (70.2/72.1). It was judged a physical placement problem and raised to top
priority.

Re-measured under controlled conditions on 08-11, the **spread was 5.6 °C.**

The cause was not placement but **a difference in load profile.**

| | 08-10 | 08-11 |
|---|---|---|
| Tool | `thread_safety_test` | `sustained_load_test` |
| Load | sequential sweep 1→8 threads | fixed at 8 threads |
| Start | king led by 6 minutes | simultaneous |

`thread_safety_test` runs single- and two-thread baselines before reaching the
target thread count. `king` had been heating for far longer by the time the
other two entered 8 threads.

The decisive evidence: **`queen`'s peak temperature was an identical 70.2 °C in
both measurements.** Only `king` moved.

→ **Do not compare temperatures between different load profiles.**

## 4.2 "78 inf/s is a driver characteristic" → scope narrowed

Having confirmed that roughly 80 kernel ioctls per inference get serialized, the
node ceiling of 78 inf/s was defined as a driver characteristic. It also fit
with all three application optimizations being meaningless.

Then INT8 came in at 1.85×. The ioctl counts were checked.

```
strace -c -f -e trace=ioctl, 1 thread, 20 s
  FP16  315 inferences  15.7 inf/s  24,079 ioctls  76.4 per inference
  INT8  718 inferences  35.8 inf/s  54,707 ioctls  76.2 per inference
```

**The count is the same and throughput is 2.28×.** What sets the ceiling is not
the ioctl count but **how long one inference holds the serialized section.**

The CPU governor experiment (+7%) reinforces this. That time includes not only
NPU execution but **the CPU work either side of it.**

→ "It cannot be exceeded by application optimization" holds. But **quantization
  is a model change, not an application optimization.**

## 4.3 "RKNN is thread-safe" → true, but the sequence is not atomic

`environment-matrix.md` §3.1 had concluded that "RKNN Runtime 2.3.0 is
thread-safe". Sharing one context would simplify the implementation.

But that verification **counted only API return codes and never compared output
contents.**

One inference is three calls.

```
rknn_inputs_set  ->  rknn_run  ->  rknn_outputs_get
```

Even with each call thread-safe, **the sequence is not atomic.**

| Configuration | API errors | **Result mismatches** |
|---|---:|---:|
| Shared context | 0 | **200 / 200 (100%)** |
| Per-thread dedicated context | 0 | 0 / 200 (0%) |

**A shared context produces 100% wrong answers with no errors.**

The nature of this defect is especially bad.

- No exception and no error code
- Never reproduces single-threaded
- **The throughput metric actually looks better** (in §3.1, shared 34.8 >
  dedicated 33.2 — it was producing wrong answers faster)
- The detections come from another frame, so they look plausible to the eye

Left alone, it would have reached a public talk **with all throughput valid and
only the detections quietly wrong.**

→ `RknnContext::infer` takes `&mut self`. **The compiler blocks concurrent
  calls.** Writing a rule in a comment and blocking it with a type are different
  things.

---

# 5. Verified but ineffective

The basis for there being almost nothing left to squeeze inside the node.

| Attempt | Result | Verdict |
|---|---:|---|
| Manual core assignment via `core_mask` | **+0.1%** | not used. `CORE_AUTO`'s distribution is already even |
| Zero-copy buffer reuse | **−1.8%** | not used. Hypothesis refuted |

`core_mask` was +9% at 4 threads but **vanishes to +0.1% at 8.** `CORE_0_1` is
actually a loss at −11.5% at 8 threads.

> **`want_float=0` was originally in this table.** The 08-10 measurement put it
> at FP16 +5.4%, close to "no effect". But that measurement was mostly a
> single-thread condition, and re-measuring on 08-12 at 8 threads for 120 s gave
> **INT8 +17.3% / FP16 +15.7%.** The time output conversion holds the serialized
> section grows with the number of concurrent threads.
> → moved to §2.2. **It was not ineffective; the measurement's conditions were
> inadequate.** `discuss.md` §12

---

# 6. The list of measurement failures

**The highest reuse-value result this project has.** All of them are failures
that "looked like success".

## 6.1 Not checking what a metric counts (4 times)

| # | What | Actually |
|---|---|---|
| 1 | Reading `RKNN_QUERY_PERF_RUN.run_duration` as NPU occupancy time | a value that included queue wait |
| 2 | Sampling NPU load at 0.2 s intervals with `delayms=3000` still set | it was reading a 3-second average |
| 3 | Judging thread-safety by API return codes alone | output contents were never compared |
| 4 | Judging throttling **by NPU clock alone** | the CPU was bending from A72 2208 to 816 MHz |

Number 1 was self-discovered through a contradiction ("5.03 cores in use" on a
2-core NPU). Number 2 was pointed out by a ChatGPT review. Number 3 was found by
suspicion while implementing the backend. Number 4 had the CPU clocks **already
recorded in the same log** and simply did not use them in the verdict — it was
found only when the FP16 re-measurement came out at 66.9 rather than 84.3
(§2.3).

**What they share: reading a metric's name and assuming its meaning.**

## 6.2 Not noticing a changed premise

| What | Result |
|---|---|
| A stale `king` IP in the documents (`.22`, actually `.12`) | misdiagnosed as "the node is dead"; scanned the whole subnet |
| Comparing two measurements with different load profiles | a 19 °C gap was misread (§4.1) |
| Misdiagnosing the cause of board resets three times | shared PSU → bootloader → 12V input. Actually **insufficient adapter current** |

`~/.ssh/config` had had the correct IP all along. **Only the IP pinned into the
documents was stale.** → Boards are reached by alias, not by IP.

## 6.3 Remote execution where failure looks like success

Found while building `preflight-check.sh`. The check **was silently not
working.**

**`pgrep -f` counts itself.** The ssh wrapper's command line contains the
pattern string, so it matches. The bracket trick (`[s]ustained`) is also
neutralised once a form without brackets appears on the same command line.

| Situation | Actual | pgrep reports |
|---|---|---|
| Load running | 1 | **0 (missed)** |
| No load | 0 | **2 (its own shell)** |

**`cd DIR && setsid nohup ... &` does not come up.** The `&` applies to the
whole list, and if ssh disconnects immediately the subshell dies before reaching
`setsid`. **Exit code 0, empty stderr.** Without checking, you measure "the
temperature with no load" for fifteen minutes.

**A heredoc inside ssh nested with sudo does not create the file.** Encountered
while deploying a systemd unit. This too gave exit code 0.

→ All of them hardened into functions in `scripts/lib/remote.sh`.

## 6.4 Judging "could not read" as "identical"

`/sys/kernel/debug/rknpu/version` is readable only by root. All three nodes
returned an empty value and it **passed on the grounds that the values matched.**
A variant of 6.1.

→ Empty values and placeholders such as `unknown` are treated as failures.

## 6.5 Tooling

To stop the same mistakes recurring, the checks were put into tools.

| Tool | What it blocks |
|---|---|
| `preflight-check.sh` | alias↔hostname, hash matching, governor, temperature, voltage, residual load, **inference accuracy** |
| `npuforge-bench` | warmup exclusion, reboot detection via `boot_id`, insufficient-sample verdicts, excluding failures from throughput |
| `run-thermal-comparison.sh` | simultaneous start, hash verification, confirming load actually started |
| `scripts/lib/remote.sh` | the remote execution pitfalls |

**Accuracy is checked before performance** — `preflight --with-inference` checks
that the three boards give the same answer to the same input. A configuration
that produces wrong answers fast must not win a benchmark.

---

# 7. Reproduction

## 7.1 Without hardware

```bash
cargo test --workspace          # 209 tests
cargo clippy --workspace --all-targets -- -D warnings
```

The Mock 3-node cluster also runs without hardware.

```bash
cargo build --release -p npuforge-scheduler -p npuforge-node -p npuforge-bench
./target/release/npuforge-scheduler --config configs/scheduler.example.toml &
for i in 1 2 3; do ./target/release/npuforge-node --config configs/mock/node-0$i.toml & done
./target/release/npuforge-bench --scheduler http://127.0.0.1:50051 \
  --model yolov8n --concurrency 6 --duration 15
```

## 7.2 On real hardware

Prerequisites: the `npuforge-k/q/j` aliases in `~/.ssh/config`, and
`NPUFORGE_SUDO_PASS` set.

```bash
bash scripts/preflight-check.sh --with-inference   # no measuring until it passes
bash scripts/run-thermal-comparison.sh 900 8       # thermal comparison
```

## 7.3 Model conversion

```bash
python tools/model-converter/fetch_calibration.py --out datasets/coco-calib --count 200
docker run --rm -v "$PWD/models:/work/models" -v "$PWD/datasets:/work/datasets" \
  npuforge-converter:2.3.1 python3 /work/tools/convert_yolov8n.py \
  --onnx models/yolov8n.onnx --out models/yolov8n-int8.rknn \
  --dataset datasets/coco-calib --calib-limit 200
```

> **The model is converted once and the same file deployed to all three nodes.**
> INT8 conversion is not byte-reproducible — converting three times from the
> same input gave a different hash each time (1.8% of bytes differing). But
> **the inference results are completely identical** (all 9 tensors at cosine
> 1.000000). The difference is in serialization, not in computation.
>
> `model.toml`'s `sha256` guarantees **deployment integrity**, not identity of
> the conversion recipe.

---

# 8. What comes next

## 8.1 Blocked — 10G aggregation is needed

One raw RGB input is `640 × 640 × 3 = 1,228,800 byte`.

```text
INT8  1,228,800 x 157.2 x 8 = 1.545 Gbps / node   ->  3 nodes 4.636 Gbps
FP16  1,228,800 x  84.3 x 8 = 0.829 Gbps / node   ->  3 nodes 2.486 Gbps
```

> **Corrected 2026-08-12.** Earlier documents had 1.43 / 4.3 Gbps, because
> converting MiB/s to Gbps used the binary prefix (÷1024). **Network speeds are
> decimal.** The correct values are above.

### The output is larger

The above is **input (TX) only.** The node does not postprocess and returns nine
raw tensors, and with `want_float=1` the output is **3.96× the input.**

```text
input                       1,228,800 byte
output (want_float=1, f32)  4,872,000 byte
output (want_float=0, int8) 1,218,000 byte
```

The scheduler-side link load at three-node saturation:

| Configuration | Model | 3-node TX | 3-node RX | Fits in 10G? |
|---|---|---:|---:|---|
| `want_float=1` (old default) | INT8 | 4.64 Gbps | **18.38 Gbps** | **no** |
| `want_float=1` (old default) | FP16 | 2.49 Gbps | **9.86 Gbps** | barely |
| **`want_float=0` (current default)** | INT8 | 4.64 Gbps | 4.60 Gbps | yes |
| **`want_float=0` (current default)** | FP16 | 2.49 Gbps | 2.46 Gbps | yes |

**Had `want_float=1` remained, even 10G could not have carried three INT8
nodes.**

→ One of two things was needed before M3.
  **(A)** switch to `want_float=0` — the output becomes a quarter
  **(B)** postprocess (NMS) on the node — the response shrinks to a few KB, but
  it is unimplemented

  `want_float=0`, filed in §5 as "an optional optimization, deferred", was
  **promoted to a precondition for M3.** **The grounds for the promotion were
  not throughput but RX bandwidth.**

> ✅ **(A) completed 2026-08-12.** The node configuration `[worker] want_float`
> default changed to `false`, and the blob was bumped to **v2** to carry
> `qnt_type`, `scale` and `zero_point` per tensor. Without those, sending int8
> leaves the receiver unable to interpret it. Dequantization was confirmed on a
> real board to match float32 (9 tensors, **maximum error 9.5e-7** — the limit
> of float32 precision).
> Throughput rose alongside — **INT8 +17.3% / FP16 +15.7%** (§2.2,
> `discuss.md` §12).
>
> Additionally, `sustained_load_test` had hardcoded `want_float=0` from the
> beginning. So §2.2's 157.2 / 84.3 were **already on `want_float=0`**, and this
> change **brought the Rust backend in line with the measurement conditions.**

### Summary

- It is the **aggregation link, not the worker links (2.5G), that fills up
  first**
- **10G** is needed on the scheduler side
- **A measure to reduce output size** has to accompany it

**Calculating the input and not looking at the output was this section's
original error.** The failure list in §6 already has three of the same type.

## 8.2 Possible without the switch

- Prometheus metrics
- Configuring an NTP server on `dealer`
- The formal S0 thermal characterisation (30 min × fanless/cooled, 2 conditions)
- `ondemand` vs `performance` over 300 s — §2.2's +7% is a 120-second value
- INT8's thermal behaviour (does less computation mean less heat)

---

# 9. Document guide

| Document | Content |
|---|---|
| `../adrs/` | **Why it was decided that way.** 28 decisions, by topic |
| `TODO.md` | **What to do now.** The resumption procedure is at the top |
| `RESULTS.md` | This document. A collection of results |
| `discuss.md` | The discussion and reasoning. 11 chronological sections |
| `board-worklog.md` | Work history. Failed hypotheses are preserved |
| `environment-matrix.md` | Settled environment values |
| `infrastructure.md` | A snapshot of the current setup |
| `00-PRD.md` – `03-*.md` | Requirements and design specifications |

When numbers disagree between documents, **`environment-matrix.md` is the
authority** (the document authority order in `00-PRD.md` §0).
