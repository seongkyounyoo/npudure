# NPUDure Technical Discussion Log

*[한국어 원문](discuss.ko.md)*

This document records technical discussions that influenced experiment design
and interpretation.

All hardware experiments, measurements, implementation changes, and final
technical decisions were carried out and owned by the project maintainer.

Claude and ChatGPT were used as discussion and review tools. Their names are
retained only to preserve which assistant proposed or challenged a particular
interpretation.

**Experiment owner and maintainer: Seongkyoun Yoo**

Raw measurements are in `benchmarks/`, settled facts in `environment-matrix.md`,
work history in `board-worklog.md`.

**Each section carries its writing time (KST) and commit hash.** With several
experiments in one day, a date alone does not give the order, and it becomes
impossible to judge later whether a conclusion came before or after a given
measurement.

---

## Reading order

The discussion is arranged chronologically. New opinions are appended at the
end.

| # | Section | Written (KST) | Discussion source | Gist |
|---|---|---|---|---|
| 1 | The NPU occupancy experiment | 08-10 (time unknown) | Claude review | the first measurement and interpretation |
| 2 | ChatGPT's response | 08-10 (time unknown) | ChatGPT review | softening the claims and demanding re-verification |
| 3 | Claude's re-examination | 08-10 (time unknown) | Claude review | accepting the points and re-measuring |
| 4 | The core_mask distribution experiment | **08-10 17:03** | Claude review | control group added, `worker_count=8` settled |
| 5 | The want_float experiment | **08-10 17:15** | Claude review | output conversion removed, +5.4% |
| 6 | Syscall decomposition | **08-10 17:26** | Claude review | bottleneck settled: driver ioctl serialization |
| 7 | The zero-copy experiment | **08-10 17:44** | Claude review | the hypothesis refuted |
| 8 | INT8 measured | **08-11 16:45** | Claude review | **1.85×**. Refines the conclusions of §6 and §7 |
| 9 | The shared context experiment | **08-11 16:45** | Claude review | "0 errors" is not a correct answer |
| 10 | Bench tool design | **08-11 17:15** | Claude review | building the mistakes into the tool |
| 11 | The CPU governor effect | **08-12 10:16** | Claude review | **+7%**. Every existing figure was on `ondemand` — **a currently valid conclusion** |

Sections 1–3 went into the first commit (`eda93a3`, 08-10 16:29) together and
their per-section times cannot be recovered. From §4 on, the commit time is the
writing time.

**Some of §1's figures were corrected in §3.** For the conclusions alone, read
§3.
**The "78 inf/s ceiling" wording in §6 and §7 had its scope narrowed in §8.**

---

# The NPU occupancy experiment — Claude's interpretation

> ⚠️ **This section's NPU load figure (30%) and some of its conclusions were
> corrected by later re-measurement.** Read "Claude's re-examination" further
> down alongside it. The original is preserved to keep the reasoning process.

- Written: 2026-08-10 (included in the first commit `eda93a3` at 16:29; the
  per-section time is unknown)
- Node measured: `queen` (NanoPi R76S, RK3576)
- Model: `yolov8n-fp16.rknn` (FP16, SHA-256 `459602ea70479c1c...`)
- Tool: `crates/npuforge-rknn/native/npu_occupancy_test.c`

## Context: what was being decided

In the thread-safety trial, **throughput rose 5.55× at 8 threads on a 2-core
NPU.** Two hypotheses for the cause diverged.

| Hypothesis | Content | Optimization direction |
|---|---|---|
| **A** | The NPU submission pipeline was underfed | feed the NPU better (batching, queueing) |
| **B** | A large share of per-call time is CPU work, and threads parallelised it | optimize CPU preprocessing and postprocessing |

The two lead optimization in opposite directions. Misinterpreting it derails all
of M7.

## Measurement results

### By thread count

| Threads | Throughput | NPU Core0 / Core1 | CPU | inputs_set | run | outputs_get |
|---:|---:|---|---:|---:|---:|---:|
| 1 | 17.0 /s | **16% / 0%** | 9.8% | 17.7 ms | 28.4 ms | 12.3 ms |
| 2 | 33.3 | **17% / 15%** | 17.2% | 17.8 | 29.6 | 12.3 |
| 4 | 56.8 | **25% / 24%** | 26.9% | 19.1 | 40.9 | 10.3 |
| 8 | 76.0 | **32% / 30%** | 43.0% | 24.6 | 65.9 | 12.4 |

NPU occupancy was sampled at 0.2-second intervals from kernel debugfs
(`/sys/kernel/debug/rknpu/load`).

### The composition of a single-thread call

```text
total 58.9 ms
  inputs_set     17.7 ms  (30%)   CPU
  run            28.4 ms  (48%)   NPU submission + execution + wait
  outputs_get    12.3 ms  (21%)   CPU (dequantization included, since want_float=1)
  release         0.5 ms  ( 1%)
```

## ⚠️ First, a correction: my metric was wrong

I interpreted `RKNN_QUERY_PERF_RUN`'s `run_duration` as "actual NPU occupancy
time", but it **includes queue wait**. I was misled by the header comment's
phrase `real inference time (us)`.

**Evidence:**

- On a 2-core device my calculation gave `npu_cores_busy = 5.03`. Physically
  impossible
- `run_duration` matches `rknn_run`'s wall time exactly across every condition.
  They are the same value

**The kernel debugfs was the trustworthy source.** Values the RKNN API reports
must not be used without verification.

This is the second hasty interpretation made before this experiment. The first
was "5.55×, so the CPU is the bottleneck", and that was wrong too.

## Conclusion: both hypotheses are only partly right

### Hypothesis A is the main answer

At 1 thread, **Core0 is at 16% and Core1 at 0%** — the NPU is effectively idle.
Raising threads brings both cores into play and throughput goes 17 → 76 inf/s
(4.5×).

**But the NPU never saturates.** It stays around 30% even at 8 threads.

### Yet the CPU is not the bottleneck either

At 8 threads, CPU utilisation is **43%.** About 3.4 of 8 cores are in use. There
is headroom.

### The real ceiling is somewhere else

`run` grew 28.4 → 65.9 ms, **2.3×**, while `inputs_set` (17.7→24.6) and
`outputs_get` (12.3→12.4) barely changed.

```text
NPU occupancy   30%   <- not busy
CPU utilisation 43%   <- not busy
rknn_run        66ms  <- and yet it waits here
```

**Neither saturated while latency alone grows is the classic signature of a
queueing bottleneck.** Serialization is happening somewhere in the NPU
submission path.

Candidates:

| Candidate | Description |
|---|---|
| A lock inside the RKNN runtime | several contexts sharing one submission path |
| Kernel driver serialization | an exclusive section in the ioctl path or IOMMU mapping |
| NPU scheduling policy | `CORE_AUTO` not using the cores fully |

## What this means for the project

### The optimization priorities change

| Optimization | Expected effect | Basis |
|---|---|---|
| CPU preprocessing optimization | improves latency. **Cannot raise the throughput ceiling** | CPU is not the bottleneck (43%) |
| io_uring | **irrelevant** | a problem in a section the network is not involved in |
| **Removing NPU submission path serialization** | **raises the ceiling itself** | this is the real bottleneck |
| Switching to INT8 | may shorten `run` time | FP16 may be inefficient on this NPU |

### It fits the project's thesis

**The NPU is 70% idle.** The reason "6 TOPS × 3 ≠ 18 TOPS" may be neither the
network nor scheduling but **failing to use the NPU fully inside a single node.**

That directly supports this project's problem statement (PRD §2): that the
vendor spec sheet's TOPS does not represent actual throughput.

## What to check next

In priority order.

1. **Explicit `core_mask` distribution** — does specifying `CORE_0`/`CORE_1`
   directly break the 30% wall
   - The thread-safety trial found core separation slightly slower, but that was
     a **2-thread** condition
   - At 8 threads the result may differ
2. **INT8 vs FP16** — a shorter `run` raises the ceiling. After the calibration
   data settles
3. **`want_float=0`** — removing dequantization from `outputs_get`. A latency
   improvement
4. **`rknn_dup_context`** — is duplicating a context better than individual
   `rknn_init`

Number 1 gives an answer fastest.

## Notes on measurement method

What is needed for reproduction.

```bash
# build
gcc -O2 -Wall -Wextra -o npu_occupancy_test npu_occupancy_test.c -lrknnrt -lpthread

# run (model, iterations, threads)
./npu_occupancy_test yolov8n-fp16.rknn 120 8

# NPU occupancy has to be sampled separately (needs sudo)
sudo cat /sys/kernel/debug/rknpu/load
```

**`RKNN_QUERY_PERF_RUN` can be queried without `RKNN_FLAG_COLLECT_PERF_MASK`
and has no overhead.** Only `PERF_DETAIL` requires the flag and lowers the frame
rate.

But as written above, `run_duration` includes queue wait and **must not be used
to judge NPU occupancy.**

---

---

# ChatGPT's response

> For the current conclusions, follow "Claude's re-examination — incorporating
> ChatGPT's points and re-measuring" further down. This section is preserved as
> the review that prompted the re-measurement.

- Written: 2026-08-10 (included in the first commit `eda93a3` at 16:29; the
  per-section time is unknown)
- Subject: `The NPU occupancy experiment — Claude's interpretation`

## Overall

`discuss.md`'s direction is good. In particular, **withdrawing the hasty
interpretation "5.55×, so the CPU is the bottleneck"**, and the correction that
`RKNN_QUERY_PERF_RUN.run_duration` should not be read as actual NPU occupancy
time, are important.

This discussion is worth keeping. It also makes good material later for showing
"how an initial bottleneck hypothesis was corrected by measurement".

That said, a few places would be better with the claims lowered slightly.

## 1. "The CPU is not the bottleneck either" is not yet settled

A total CPU utilisation of 43% is an average. It can hide one thread blocking in
a lock, a driver or an ioctl path, or a single core saturating.

Even with a low average across 8 cores, a bottleneck in a non-parallelisable CPU
path creates a throughput ceiling.

So the current sentence's meaning is more accurately limited to roughly this.

```text
There is headroom by total CPU utilisation.
But single-core saturation, runtime locks and ioctl/off-CPU wait have to be
checked separately.
```

## 2. "io_uring is irrelevant" is also too strong

This experiment looked at an RKNN call bottleneck inside a single node. That
result alone cannot judge `io_uring`'s value on the distributed inference path.

The correct conclusion is:

```text
io_uring is not the direct cause of this single-node RKNN scaling problem.
Whether to optimize the distributed transport is judged after M2/M3's
network_* timing and syscall instrumentation.
```

That is, the cause of the observed 8-thread scaling limit is not network I/O.
But whether `io_uring` is meaningful for NPUDure as a whole has to be measured
separately after the gRPC baseline.

## 3. The NPU load 30% interpretation needs one more verification

Judging `/sys/kernel/debug/rknpu/load` the most trustworthy source is
reasonable. But whether that value means an average over the immediately
preceding sample window, or an accumulated/decaying value inside the driver,
needs confirming.

Core0/Core1 in the 30s while 76 inf/s comes out at 8 threads is possible, but it
is a fairly strong signal. Confirming the following would make the
interpretation more solid.

- The effect of the 0.2-second sampling interval
- The baseline with no load
- The difference between `watch -n` and a direct sampling loop
- Whether the value changes immediately after a read
- Recording NPU devfreq and the load value together

## 4. Keep the ceiling candidates a little wider

"RKNN/NPU submission path serialization" is a strong hypothesis. But it is
better not to narrow the candidates too far.

Candidates to check:

- A lock inside the RKNN runtime
- Kernel driver ioctl serialization
- IOMMU or buffer mapping cost
- DDR or memory bandwidth
- Output conversion or a hidden copy
- Thermal/frequency changes
- `CORE_AUTO` scheduling limits

`inputs_set` at 17–25 ms is larger than expected. It is far more time than a
100KB JPEG network transfer. The actual model input tensor size, copy count,
cache behaviour and whether zero-copy input is possible should be looked at too.

## 5. Suggested next experiments

In priority order.

1. Re-measure `CORE_AUTO` vs explicit `CORE_0`/`CORE_1` distribution at 4 and 8
   threads, not just 2
2. Remove the `outputs_get` cost with `want_float=0` and check the throughput
   change
3. Check the user/kernel hot paths with `perf top`/`perf record`
4. Check ioctl/syscall counts with `strace -c` or `perf stat`
5. Check per-core CPU utilisation and off-CPU time
6. Repeat the same experiment after generating an INT8 model

## Summary opinion

The document's direction is right. But the final conclusions are a little
strong, so summarising them roughly as follows would be safer.

```text
The current experiment showed that the 8-thread throughput increase is not
explained by CPU preprocessing parallelisation alone.
At 1 thread the NPU was not sufficiently used, and with multiple threads the NPU
submission pipeline gets fuller.
But since neither the NPU nor the CPU is saturated even at 8 threads,
serialization or queueing in the RKNN runtime or kernel driver submission path
is likely.
This is a single-node internal bottleneck separate from io_uring, and whether to
optimize the distributed transport is judged separately with TimingBreakdown
after the gRPC baseline.
```

---

---

# Claude's re-examination — incorporating ChatGPT's points and re-measuring

- Written: 2026-08-10 (included in the first commit `eda93a3` at 16:29; the
  per-section time is unknown)
- Subject: the 5 items in `ChatGPT's response` above
- Re-measurement tool: the same (`npu_occupancy_test.c`) + a corrected sampling
  procedure

## Summary

**Three of the five points were confirmed by measurement, and doing so revealed
that the earlier figures were underestimates.**

The biggest is number 3 (re-verifying NPU load). The measurement method itself
was flawed.

## Point 3 confirmed: `delayms = 3000` — a measurement method flaw

There is a `delayms` file under `/sys/kernel/debug/rknpu/`, and its value was
**3000**.

**NPU load is averaged over a 3-second window and I was sampling at 0.2-second
intervals.** The same value was read 15 times over, and the 0% section after the
load ended got mixed into the average.

`delayms` is writable. I **lowered it to 100 ms** and re-measured, sampling only
the steady section after a 4-second warmup.

```bash
sudo sh -c 'echo 100 > /sys/kernel/debug/rknpu/delayms'
```

The no-load baseline confirmed 0%, showing the value does not accumulate or
decay.

### Re-measured results

| Threads | Throughput | Core0 avg/max | Core1 avg/max | CPU (cpu0–3 / cpu4–7) |
|---:|---:|---|---|---|
| 1 | 16.7 /s | 18.9% / 39% | **0.5% / 1%** | 3,1,2,2 / **30,23**,6,7 |
| 2 | 36.0 | 23.4% / 48% | 16.0% / 33% | 3,2,3,3 / 28,36,40,19 |
| 4 | 55.6 | 29.6% / 62% | 26.9% / 54% | 18,8,7,4 / 48,45,46,48 |
| 8 | 75.0 | **38.9% / 86%** | **37.0% / 81%** | 47,43,44,42 / 48,49,42,46 |

**The earlier 30% was an underestimate.** The real average is 38.9%, and the
**instantaneous maximum is 86%.**

## Point 1 confirmed: "the CPU is not the bottleneck either" was inaccurate

The 43% overall average **was hiding per-core imbalance.**

```text
1 thread  cpu4=30%, cpu5=23%      only some big cores in use, the little ones idle
8 threads all cores 42-49%        evenly spread
```

`cpu0–3` are little (A53 2.016 GHz) and `cpu4–7` are big (A72 2.208 GHz). One
big core at 30% on a single thread is not a small load in single-thread terms.

**Corrected wording:**

```text
There is headroom by total CPU utilisation (at most 49% at 8 threads).
Single-core saturation was not observed.
But runtime locks, ioctl serialization and off-CPU wait need separate
instrumentation.
```

`perf record` / `strace -c` / off-CPU analysis have not been done yet.

## Point 2 accepted: "io_uring is irrelevant" overstepped

This experiment looked at **the RKNN call path inside a single node.** It is not
grounds for judging the value of the distributed transport.

**Corrected wording:**

```text
The direct cause of this single-node RKNN scaling limit is not network I/O.
Whether io_uring is meaningful for NPUDure as a whole is judged separately after
the gRPC baseline, with TimingBreakdown and syscall instrumentation.
```

The `io_uring — irrelevant` entry in the previous section's "optimization
priorities" table is replaced by that sentence.

## Point 4 accepted: widen the ceiling candidates

The point that **`inputs_set` at 17–25 ms** is large is well taken. The input
tensor is 640×640×3 = 1,228,800 bytes, and 17 ms works out at about 70 MB/s.
That is slow for a plain memcpy, so a format conversion or multiple copies are
suspected.

Candidates to check (not narrowed):

| Candidate | How to check |
|---|---|
| A lock inside the RKNN runtime | `perf record`, off-CPU analysis |
| Kernel driver ioctl serialization | `strace -c`, ioctl count and duration |
| IOMMU / buffer mapping cost | `perf`, driver traces |
| DDR / memory bandwidth | `inputs_set` throughput against theoretical bandwidth |
| Output conversion / hidden copy | comparison with `want_float=0` |
| Thermal / frequency changes | recording devfreq `cur_freq` alongside (confirmed pinned at 950 MHz this time) |
| `CORE_AUTO` scheduling limits | comparison with explicit `core_mask` distribution |

## The corrected conclusion

```text
The 8-thread throughput increase is not explained by CPU preprocessing
parallelisation alone.

At 1 thread NPU Core1 was effectively unused at 0.5%, and raising threads brought
both cores up evenly to 38-39%.
The instantaneous maximum reaches 86%, close to saturation, but the average is
under 40%.

That is, the NPU cannot be fed continuously and there are intermittent gaps.
The CPU is at or below 49% overall with no single-core saturation.

So serialization or queueing in the RKNN runtime or kernel driver submission path
is likely, but it is not settled until lock/off-CPU instrumentation is done.

This is a single-node internal bottleneck, a separate matter from io_uring.
Whether to optimize the distributed transport is judged separately after the gRPC
baseline.
```

## To reflect in operations

**`delayms` reverts to 3000 on reboot.** Using NPU load as telemetry requires
setting it before measuring.

To add to `preflight-check.sh`:

```bash
sudo sh -c 'echo 100 > /sys/kernel/debug/rknpu/delayms'
# confirm
[ "$(sudo cat /sys/kernel/debug/rknpu/delayms)" = "100" ] || abort
```

And the NPU load sampling rules:

- Sample at intervals of at least `delayms` (preventing duplicate reads)
- Exclude the warmup section and the section right after the end from the average
- **Record the maximum alongside the average** (the average alone misses
  instantaneous saturation)

## Verifications not yet done

The unperformed items among ChatGPT's suggestions.

- [x] `CORE_AUTO` vs explicit `CORE_0`/`CORE_1` distribution (4/8 threads) — see
  §4
- [x] Removing the `outputs_get` cost with `want_float=0` — see §5
- [x] `perf` was impossible due to a kernel version mismatch. Substituted `time`
  + `strace -c` — §6
- [x] ioctl count checked with `strace -c` — 80 per inference, §6
- [x] Off-CPU analysis — 58 ms blocked per call at 8 threads, §6
- [ ] Repeat the same experiment with an INT8 model (after the calibration data
  settles)

## Meta: the same mistake twice

There were three hasty interpretations in this episode.

| # | The wrong judgement | Cause |
|---|---|---|
| 1 | "5.55×, so the CPU is the bottleneck" | did not exclude the alternative hypothesis |
| 2 | "`run_duration` = NPU occupancy time" | trusted an API comment without verification |
| 3 | "NPU load 30%" | did not check the measurement tool's sampling characteristics |

What they share is **reaching a conclusion before confirming what the measured
value means.**

Number 2 was caught by self-contradiction (5.03 on a 2-core device), number 3 by
an external review. Number 1 was caught by measurement.

The lesson: **when using a new metric, first confirm the value's meaning, update
period and boundary conditions.** Values the kernel exposes are often
undocumented, so verify them with a no-load baseline and extreme values.

---

# The core_mask distribution experiment — Claude's interpretation

- Written: **2026-08-10 17:03 KST** (commit `0e6e264`)
- Node measured: `queen`
- Model: `yolov8n-fp16.rknn` (FP16)
- Raw data: `benchmarks/results/2026-08-10-coremask/coremask-queen.txt`
- Conditions: 200 iterations per thread, `delayms=100`, sampled after a
  4-second warmup

## What was being checked

The previous section listed "`CORE_AUTO` not using the cores fully" as a
bottleneck candidate. This verifies it.

**A control group was added.** Until now we had only seen the number, Core1's
38% occupancy, without ever confirming that it translated into actual
throughput. Comparing against `CORE_0_ONLY`, which pins every thread to one
core, settles it.

| mode | Setting | Intent |
|---|---|---|
| 0 | `CORE_AUTO` | the runtime chooses (the current default) |
| 1 | `ALTERNATE` | threads pinned alternately to `CORE_0`/`CORE_1` |
| 2 | `CORE_0_1` | every thread uses both cores together |
| 3 | **`CORE_0_ONLY`** | **all pinned to core 0 — the control group** |

## Results

Throughput (inf/s)

| Threads | AUTO | ALTERNATE | CORE_0_1 | **CORE_0_ONLY** |
|---:|---:|---:|---:|---:|
| 1 | 16.7 | 16.7 | **18.2** | 16.5 |
| 2 | 36.2 | 36.5 | 36.4 | **26.4** |
| 4 | 52.4 | **57.1** | 48.5 | **38.5** |
| 8 | 72.9 | **73.0** | 64.5 | **48.2** |

`run` time (µs) and NPU occupancy (average/maximum %)

| Threads | mode | run | Core0 | Core1 |
|---:|---|---:|---|---|
| 8 | AUTO | 69,046 | 39/85 | 37/78 |
| 8 | ALTERNATE | 66,314 | 38/81 | 38/81 |
| 8 | CORE_0_1 | 83,175 | 38/80 | 29/60 |
| 8 | **CORE_0_ONLY** | **120,608** | 46/96 | **0/1** |

## Finding 1: the second core does contribute

At 8 threads, **one core gives 48.2 and two give 73.0 inf/s, 1.51×.**

Core1's 38% occupancy was not decoration but real throughput. The control group
filled in what the previous section could not confirm.

**But it is 1.51×, not 2×.** Doubling the cores raises throughput by only half.
That means there is a shared resource outside the cores, consistent with the
previous section's "submission path serialization" hypothesis.

`CORE_0_ONLY`'s `run` exploding to 120.6 ms is the same phenomenon. Eight
threads piled onto one core, and the wait accumulates directly.

## Finding 2: explicit distribution brings almost no gain

```text
4 threads   52.4 -> 57.1   +9.0%
8 threads   72.9 -> 73.0   +0.1%
```

It improves only at 4 threads and there is no difference at 8.

And unpacking the 4-thread improvement, most of it is `outputs_get` falling from
13.6 to 10.0 ms. Whether that is a core-distribution effect or measurement noise
is not separated.

**`AUTO`'s distribution is already even** (39%/37% at 8 threads). The runtime
scheduler is doing its job, leaving no room for manual intervention.

## Finding 3: `CORE_0_1` is actually a loss

```text
8 threads   72.9 -> 64.5   -11.5%
```

Opening both cores to every thread makes it slower. Scheduling overhead or cache
invalidation from contexts moving between cores appears to be the cause.

**The exception is 1 thread.** Only there is it a gain, 16.7 → 18.2 (+9%), with
`run` falling from 29.7 to 23.7 ms.

A single request benefits from using both cores together. **It is a card
available for latency-sensitive workloads** and worth considering when handling
deadline requests.

## Conclusion

```toml
[worker]
worker_count = 8      # 27% higher than 4
                      # core_mask is not set (CORE_AUTO)
```

`ALTERNATE`'s 4-thread gain (+9%) vanishes at 8 threads. **Going to 8 threads
beats manual core assignment, and the code is simpler** — the
`rknn_set_core_mask` call becomes unnecessary.

## The bottleneck candidates have narrowed

| Candidate | Verdict |
|---|---|
| ~~`CORE_AUTO` scheduling limits~~ | **excluded** — already evenly distributed, manual intervention pointless |
| ~~The second core unused~~ | **excluded** — a 1.51× contribution confirmed |
| **Serialization on a shared resource outside the cores** | **leading** — 2 cores giving only 1.51× |

The `CORE_AUTO` entry is removed from the previous section §4's candidate list.
What remains:

- A lock inside the RKNN runtime
- Kernel driver ioctl serialization
- IOMMU / buffer mapping cost
- DDR / memory bandwidth
- Output conversion / hidden copy

## Next

`want_float=0` gives an answer fastest. `outputs_get` is a steady 12–15 ms under
every condition; if that is dequantization cost, removing it alone reduces
latency. If it does not, that is information too.

---

# The want_float experiment — Claude's interpretation

- Written: **2026-08-10 17:15 KST** (commit `e0025b4`)
- Node measured: `queen`
- Model: `yolov8n-fp16.rknn` (FP16)
- Raw data: `benchmarks/results/2026-08-10-wantfloat/wantfloat-queen.txt`
- Conditions: 250 iterations per thread, `CORE_AUTO`

## What was being checked

In earlier measurements, `outputs_get` **barely changed** across every
condition, staying at 12–15 ms. It stayed the same even at 8× the threads. Two
interpretations were possible.

| Interpretation | Expected result with `want_float=0` |
|---|---|
| Dequantization CPU cost dominates | `outputs_get` falls sharply, throughput rises |
| Kernel/driver transfer cost dominates | `outputs_get` falls slightly, throughput unchanged |

Setting `rknn_output.want_float` to 0 gives the model's native output as-is.

## Results

| Threads | wf | Throughput | total | inputs_set | run | **outputs_get** | out bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1 | 16.6 | 60,116 | 17,828 | 29,343 | **12,424** | 1,638,400 |
| 1 | **0** | **17.6** | 56,704 | 19,826 | 28,897 | **7,620** | **819,200** |
| 2 | 1 | 36.6 | 54,668 | 17,604 | 26,781 | **10,071** | 1,638,400 |
| 2 | **0** | **36.9** | 54,204 | 19,370 | 27,890 | **6,783** | **819,200** |
| 4 | 1 | 55.9 | 71,440 | 18,361 | 41,228 | **11,614** | 1,638,400 |
| 4 | **0** | **56.9** | 70,274 | 19,491 | 41,564 | **8,965** | **819,200** |
| 8 | 1 | 73.5 | 108,341 | 23,781 | 68,664 | **15,041** | 1,638,400 |
| 8 | **0** | **77.5** | 102,422 | 27,036 | 66,442 | **8,625** | **819,200** |

Times are in µs, averaged per call.

## Finding 1: the output is exactly half

```text
want_float=1   1,638,400 bytes   FP32
want_float=0     819,200 bytes   FP16 (the model's native type)
```

Being an FP16 model, the native output is FP16 (2 bytes) and `want_float=1` was
widening it to FP32 (4 bytes). **It is precision widening, not dequantization.**

With an INT8 model the difference would be 1 byte → 4 bytes, so a fourfold gap,
and the effect is expected to be larger. To be re-measured after the calibration
data settles.

## Finding 2: `outputs_get` does clearly fall

```text
1 thread    12,424 -> 7,620    -39%
2 threads   10,071 -> 6,783    -33%
4 threads   11,614 -> 8,965    -23%
8 threads   15,041 -> 8,625    -43%
```

**6.4 ms saved at 8 threads**, and `outputs_get` stabilises near 8.6 ms as
threads increase. With `want_float=1` it grew to 15 ms.

## Finding 3: and yet the throughput gain is small

```text
1 thread    16.6 -> 17.6   +6.0%
2 threads   36.6 -> 36.9   +0.8%
4 threads   55.9 -> 56.9   +1.8%
8 threads   73.5 -> 77.5   +5.4%
```

**6.4 ms per call was removed and throughput rose 5.4%.** At 8 threads that is a
6% reduction of the 108 ms per call, and the throughput increase is about the
same.

Removing CPU work gains only that much, with no multiplier effect. **Evidence
that the system is not CPU-bound.**

## Finding 4: `inputs_set` rises consistently

An unexpected observation.

```text
1 thread    17,828 -> 19,826   +2,000
2 threads   17,604 -> 19,370   +1,766
4 threads   18,361 -> 19,491   +1,130
8 threads   23,781 -> 27,036   +3,255
```

`want_float` is an output-path setting and the input time rises. There should be
no causal link.

**Presumption: allocator behaviour.** `want_float=1` allocates and frees a
1.6 MB output buffer while `want_float=0` uses 0.8 MB. The block freed by
`rknn_outputs_release` gets reused by the next iteration's `rknn_inputs_set`,
and the differing size may have lowered the reuse rate.

**Needs confirming.** Introducing a buffer pool (`01-TECHSPEC.md` §15.1-4) and
seeing whether this variation disappears would settle it.

## Conclusion

**Adopt `want_float=0`.**

- +5.4% throughput at 8 threads, `outputs_get` −43%
- Output transfer volume halved → node→scheduler network load also halved
- YOLOv8 postprocessing happens on the CPU anyway, so handling FP16 directly is
  fine

The `npuforge-rknn` implementation defaults to `want_float=0`, with the
postprocessing code written to handle the model's native type. The output type is
queried with `RKNN_QUERY_OUTPUT_ATTR` and branched on.

## Bottleneck verdict: the second interpretation is right

Of the two hypotheses above, it is **"kernel/driver transfer cost dominates".**

Even with dequantization fully removed, `outputs_get` leaves 8.6 ms. And the
throughput gain is merely proportional to the time removed, not a multiple.

That is, **the output path passes through the same shared resource.** `run`
remains dominant at 66 ms, and that is the section left untouched.

The candidates narrow further.

| Candidate | Verdict |
|---|---|
| ~~`CORE_AUTO` scheduling limits~~ | excluded (§4) |
| ~~The second core unused~~ | excluded (§4) |
| ~~Output conversion cost~~ | **excluded** — removing it gives +5.4% |
| **A lock inside the RKNN runtime** | leading |
| **Kernel driver ioctl serialization** | leading |
| IOMMU / buffer mapping | possible |
| DDR / memory bandwidth | possible (potentially related to finding 4) |

## Next

Look at the hot path and ioctl count with `perf record` and `strace -c`. So far
we have measured inputs and outputs from outside the black box, but the
remaining candidates require looking inside it.

`rknn_run` being 66 ms with NPU occupancy at 40% means **over 26 ms is consumed
outside the NPU.** Where that time goes is the next question.

---

# Syscall decomposition — the bottleneck settled: driver ioctl serialization

> ⚠️ **This section's "78 inf/s is a driver characteristic" wording had its scope
> narrowed in §8.** Ioctls per inference are the same 76 on INT8, and throughput
> is 1.85×. What sets the ceiling is not the ioctl **count** but the **time** one
> inference holds the serialized section. The conclusion that application
> optimization cannot exceed it stands — quantization is a model change, not an
> application optimization.

- Written: **2026-08-10 17:26 KST** (commit `3656401`)
- Node measured: `queen`
- Raw data: `benchmarks/results/2026-08-10-syscall/`

## What was being checked

`rknn_run` is 66 ms while NPU occupancy is 40%. **Over 26 ms is consumed outside
the NPU.** Find out what that time is.

`perf` could not be used. The board's kernel is BSP 6.1.141 while the Ubuntu
repository's `linux-tools` is for 6.8.0, and the versions do not match. Two
things were used instead.

1. The bash builtin `time` — a user/sys split. No overhead
2. `strace -f -c` — syscall counts and durations

## Measurement 1: blocked time dominates

| Threads | real | user | sys | on-CPU share |
|---:|---:|---:|---:|---:|
| 1 | 14.75 | 6.40 | 1.87 | 56% |
| 2 | 13.88 | 12.94 | 2.88 | 57% |
| 4 | 17.90 | 28.76 | 5.23 | 47% |
| 8 | 25.85 | 74.13 | 9.56 | **40%** |

Per call at 8 threads (99.9 ms total):

```text
37.1 ms   userspace CPU   inside librknnrt (mostly inputs_set)
 4.8 ms   kernel CPU      the ioctl handling itself
58.0 ms   blocked         asleep, waiting
```

**That kernel CPU is only 4.8 ms matters.** It is not burning CPU on ioctl
handling. The threads **sleep and wait.**

And NPU occupancy is 40%. **They are not waiting on NPU computation.**

## Measurement 2: ioctls serialize

`strace -f -c`, 30 iterations per thread.

| | 1 thread | 8 threads | |
|---|---:|---:|---|
| ioctl calls | 2,419 | 19,072 | |
| **ioctls per inference** | **80.6** | **79.5** | identical |
| **Duration per ioctl** | **69 µs** | **374 µs** | **5.4×** |
| futex calls | 3 | 395 | negligible |

Two things are decisive.

**First, ioctls per inference stay at ~80 regardless of thread count.** The
amount of work has not grown.

**Second, per-call latency grows 5.4×.** The same work takes far longer at 8
threads. That means an exclusive section inside the driver.

**Third, futex is only 395 calls.** Userspace lock contention would have given
tens of thousands. **It is not a lock inside librknnrt.**

## Conclusion: kernel driver ioctl serialization

```text
80 ioctls x 374 us ~ 30 ms
```

That explains over half of the 58 ms blocked time. The rest appears to be actual
NPU computation wait and scheduling delay.

The bottleneck candidate list is now settled.

| Candidate | Verdict |
|---|---|
| ~~`CORE_AUTO` scheduling limits~~ | excluded (§4) |
| ~~The second core unused~~ | excluded (§4) |
| ~~Output conversion cost~~ | excluded (§5) |
| ~~A lock inside the RKNN runtime~~ | **excluded** — 395 futex calls |
| **Kernel driver ioctl serialization** | **settled** |
| IOMMU / buffer mapping | likely what the ioctls contain |

## The real problem is that there are 80 ioctls

The serialization itself is the driver's implementation and not something we can
fix. But **the call count can be reduced.**

Eighty ioctls for one inference is excessive. `rknn_inputs_set` /
`rknn_outputs_get` are presumed to allocate, map and free buffers on every call.

RKNN provides a **zero-copy memory API** to avoid this. Confirmed in the
headers.

```c
rknn_create_mem(ctx, size)              /* allocate the buffer once */
rknn_set_io_mem(ctx, mem, attr)         /* bind it to the context */
rknn_destroy_mem(ctx, mem)
RKNN_QUERY_NATIVE_INPUT_ATTR  = 8       /* query the native layout */
RKNN_QUERY_NATIVE_OUTPUT_ATTR = 9
RKNN_FLAG_MEM_ALLOC_OUTSIDE   = 0x10
```

Allocating the buffer once and reusing it could eliminate the per-call mapping
ioctls.

**This is currently the most promising optimization.** And it matches exactly
`01-TECHSPEC.md` §15.1-6's "consider registered buffers or zero-copy". Except
the document assumed that as a network path optimization, whereas **the place it
is actually needed is the NPU input/output path.**

## What this means for the project

This result goes straight into the talk's narrative.

```text
Looking for why 6 TOPS x 3 does not become 18 TOPS,
it was neither the network nor scheduling
but the path for getting data into and out of the NPU inside a single node.

One inference produces 80 kernel ioctls,
and under concurrency their latency grows 5.4x.
The NPU works only 40% of the time and the rest is waiting.
```

It also connects to the io_uring discussion. **Optimizing network I/O does not
touch this section.** The earlier judgement (§3, §5) that the optimization target
has to be chosen from data is confirmed here.

## Next

Measure whether the zero-copy memory API reduces the ioctl count.

```text
now    rknn_inputs_set -> rknn_run -> rknn_outputs_get -> rknn_outputs_release
       80 ioctls per inference

goal   rknn_create_mem (once) -> rknn_set_io_mem (once)
       -> [rknn_run repeated]
       -> rknn_destroy_mem (once)
       expecting fewer ioctls per inference
```

Measured items: ioctls per inference, latency per ioctl, throughput, on-CPU
share.

---

# The zero-copy experiment — the hypothesis refuted

> ⚠️ **This section's "78 inf/s is a driver characteristic" wording had its scope
> narrowed in §8.** Ioctls per inference are the same 76 on INT8, and throughput
> is 1.85×. What sets the ceiling is not the ioctl **count** but the **time** one
> inference holds the serialized section. The conclusion that application
> optimization cannot exceed it stands — quantization is a model change, not an
> application optimization.

- Written: **2026-08-10 17:44 KST** (commit `7a0379b`)
- Node measured: `queen`
- Tool: `crates/npuforge-rknn/native/zerocopy_test.c`
- Raw data: `benchmarks/results/2026-08-10-zerocopy/`

## The hypothesis

§6 confirmed about 80 ioctls per inference with per-call latency growing 5.4× at
8 threads. The serialization is the driver's implementation and cannot be fixed,
but **the call count can be reduced.**

If `rknn_inputs_set` / `rknn_outputs_get` allocate, map and free a buffer on
every call, the zero-copy memory API could reuse the buffer and eliminate the
per-call mapping ioctls.

```c
rknn_create_mem(ctx, size)          /* once */
rknn_set_io_mem(ctx, mem, attr)     /* once */
  -> [ memcpy + rknn_mem_sync + rknn_run repeated ]
rknn_destroy_mem(ctx, mem)          /* once */
```

## Result: the ioctls did not fall

| | ioctls per inference | Latency per ioctl | 8-thread throughput |
|---|---:|---:|---:|
| NORMAL | 79.7 | 54 µs | 78.5 inf/s |
| ZEROCOPY | **89.8** | 56 µs | 77.1 inf/s |

**They went up by 10.** The `rknn_mem_sync` calls I added (1 input + 9 outputs =
10) are themselves ioctls.

CPU usage did not fall either.

| | real | user | sys | Throughput |
|---|---:|---:|---:|---:|
| NORMAL | 15.91 | 44.59 | 5.77 | 78.7 |
| ZEROCOPY | 16.38 | 46.48 | 6.64 | 76.7 |

## The section times shifted dramatically

At 1 thread.

| Section | NORMAL | ZEROCOPY |
|---|---:|---:|
| prepare | 20,208 µs | **1,025 µs** |
| run | 28,152 | **58,560** |
| fetch | 8,233 | **778** |
| **total** | **56,593** | **60,364** |

`prepare` and `fetch` nearly vanished, by close to 95%, but `run` doubled and
**the total actually got worse.**

The work did not disappear; it moved inside `rknn_run`.

## Conclusion: the 80 ioctls are intrinsic to inference submission

They are unrelated to buffer management. `rknn_run` itself exchanges about 80
round trips with the driver.

If YOLOv8n consists of many layers and the driver submits per task, that is a
natural figure. **It is not something the application layer can reduce.**

So the throughput ceiling confirmed in §6 (about 78 inf/s) is **a property of the
driver and hardware** and cannot be worked around.

## ⚠️ A fairness limit in this experiment

**The two paths were not doing the same work.**

```text
native_in_bytes = 2,457,600 = 640 x 640 x 3 x 2   <- FP16
model input (uint8) = 1,228,800
```

The native input is **FP16.** The NORMAL path's `rknn_inputs_set` performs the
uint8 → FP16 conversion and normalisation. That is what `prepare`'s 20 ms
contains.

The ZEROCOPY path did not do that conversion and `memset` dummy data instead.
**The conversion cost was excluded from the measurement.**

That is, a real application would have to do that conversion itself, so
`prepare` at 1 ms is unachievable.

**The conclusion holds regardless.** The ioctl count and CPU usage did not fall,
and the total time did not improve. Adding the conversion only makes it worse.

## What still has value

Zero-copy itself is not a card to discard. It may be valid under the following
condition.

**When the application can already produce the input in the native format.** In
a pipeline doing its own JPEG decoding and resizing, using that output directly
as FP16 removes the intermediate conversion. But the throughput ceiling is still
set by the driver.

**Revisit on an INT8 model.** With a native input of int8, it is the same size as
a uint8 input and the conversion becomes simple. To be re-measured after the
calibration data settles.

## The optimization candidates, summarised

| Candidate | Verdict |
|---|---|
| ~~`CORE_AUTO` scheduling~~ | excluded (§4) |
| ~~The second core unused~~ | excluded (§4) |
| ~~Output conversion~~ | excluded (§5) |
| ~~A lock inside the RKNN runtime~~ | excluded (§6) |
| ~~Reducing ioctls with zero-copy~~ | **excluded (§7)** |
| Kernel driver ioctl serialization | settled. **Cannot be worked around** |
| Switching to INT8 | **unverified. The only big card left** |

## What this means for the project

**A single node's throughput ceiling is about 78 inf/s, and that is a driver
characteristic.**

That application optimization cannot exceed it was confirmed by three
experiments (§4 core_mask, §5 want_float, §7 zero-copy). They were +0.1%, +5.4%
and −1.8% respectively.

This directly affects scaling efficiency measurement. **With the per-node ceiling
fixed, three-node scaling efficiency is determined by scheduling and the network
alone.** That there is almost no room for node-internal optimization actually
simplifies the experimental conditions.

It goes into the talk's narrative too.

```text
Three things were tried to use the NPU better.
Manual core assignment, removing output conversion, zero-copy buffer reuse.
They were +0.1%, +5.4% and -1.8%.

One inference produces 80 kernel ioctls and those serialize,
and the application could not reduce that count.

What the TOPS figure does not tell you is not computational capability
but the cost of the path for getting data into and out of that compute unit.
```

## Next

**The INT8 model** is the only big variable left. Against FP16, the following
change.

- NPU computation time (the actual computation part of `run`)
- Reduced conversion cost if the native input is int8
- A quarter of the output size

It cannot proceed until the calibration data settles. Until then, 78 inf/s per
node is the reference value.

---

# INT8 measured — Claude's interpretation

- Written: **2026-08-11 16:45 KST** (commit `547333c`)
- Node measured: `king`
- Tool: `crates/npuforge-rknn/native/sustained_load_test.c`
- Accuracy raw data: `results/accuracy/README.md`

The end of §7 said "INT8 is the only big variable left". It was measured.

## Results

`king`, `sustained_load_test`, fixed at 8 threads, 120 s.

| Model | Throughput | Mean latency | Model size |
|---|---:|---:|---:|
| FP16 | 79.0 inf/s | 100.9 ms | 9.65 MB |
| **INT8** | **146.2 inf/s** | **54.6 ms** | 6.46 MB |
| Ratio | **1.85×** | −46% | −33% |

> These figures were measured with the CPU governor at `ondemand`.
> Switching to `performance` gives FP16 84.3 / INT8 157.2 inf/s (§11).
> **The 1.86× ratio is unchanged.**

It is an order of magnitude different from the three application optimizations
(+0.1%, +5.4%, −1.8%).

## It conflicts with §6 and §7's conclusions

§6 defined **the node ceiling of 78 inf/s as a driver characteristic**, on the
grounds that "one inference produces about 80 ioctls and those serialize". §7
carried that wording forward.

But if INT8 is 1.85×, the ioctl count is not what sets the ceiling. Confirmed.

```text
strace -c -f -e trace=ioctl, 1 thread, 20 s

              inferences  throughput   mean latency   total ioctls   per inference
FP16          315         15.7 inf/s   63.3 ms        24,079         76.4
INT8          718         35.8 inf/s   27.8 ms        54,707         76.2
```

**Ioctls per inference are effectively the same, 76.4 vs 76.2.** And throughput
is 2.28× (at 1 thread). That is, what sets the ceiling is not the ioctl **count**
but **the time one inference holds the serialized section.**

## The corrected model

The two statements are not contradictory. Combined:

```text
throughput ~ 1 / (time per inference in the serialized section)

  - that serialization occurs           -> §6 is right (ioctls, only 395 futex calls)
  - that latency grows 5.4x
    at 8 threads                        -> §6 is right (evidence of serialization)
  - reducing the size of that time
    raises the ceiling                  -> §8 (INT8 reduces the actual computation)
```

**"Cannot be exceeded at the application layer"** and **"there is no way to
exceed it"** are different things. core_mask, want_float and zero-copy reduced
neither the ioctl count nor the per-call computation. Quantization reduces the
computation.

## Corrected wording

The following sentences in §6 and §7 are narrowed.

| Previously | Corrected |
|---|---|
| "The node ceiling of 78 inf/s is a driver characteristic" | "**On FP16**, the node ceiling is about 78 inf/s, and that value cannot be exceeded by application optimization" |
| "Cannot be exceeded by application optimization" | Stands. But **quantization is a model change, not an application optimization** |

## The talk narrative, updated

One more line attaches to §7's narrative. This version is both more honest and
more useful.

```text
Three things were tried to use the NPU better.
Manual core assignment, removing output conversion, zero-copy buffer reuse.
They were +0.1%, +5.4% and -1.8%.

One inference produces 76 kernel ioctls and those serialize.
The application could not reduce that count.

And then INT8 quantization was 1.85x.
The ioctl count was the same 76.

What had to be reduced was not the number of calls
but the time one call holds on.
```

## The accuracy cost

It is not lossless. A detection-level comparison on a real board
(`results/accuracy/README.md`).

| Comparison | box cosine | Detection cells | Classes |
|---|---|---|---|
| FP16 vs ONNX | 0.99999 | 10/10 | 100% |
| INT8 vs FP16 | 0.997 | 10/10 | 100% |

The top detection's cell moved by one and its score was −5.5%. **The detection
set and classes are identical.** Getting 1.85× for that price is worth it.

### The trap hit during accuracy verification

**Raw tensor cosine similarity misleads on this model.** Even FP16 vs ONNX drops
to 0.16 on some tensors. It is not a quantization problem.

Of YOLOv8n's 9 outputs, tensors 2/5/8 are the sum of 80 class scores. RKNN's
sigmoid does not output exactly 0 but has a floor of 0.001831, so amplified 80×
it creates a 0.1465 offset (matching the measured floor exactly). Most cells are
background, so this offset dominates the cosine. **The same value is added to
every cell, so the ranking does not change and the detections are unaffected.**

The acceptance criterion was changed to the detection level.
`compare_detections.py`.

**INT8 conversion is not byte-reproducible.** Converting three times from the
same ONNX with the same calibration list gave a different hash each time (same
file size, 1.8% of bytes differing). But the inference results are completely
identical (all 9 tensors at cosine 1.000000, error 0.0). The difference is in
serialization and layout, not in numerical computation.

→ The model is converted once and the same file deployed to all three nodes.
  `model.toml`'s `sha256` guarantees deployment integrity, not identity of the
  conversion recipe.

## What to check next

- **INT8 + `want_float=0`.** §5 gave +5.4% on FP16. With INT8 the output is int8,
  so the dequantization cost may be relatively larger and the gain bigger.
- **INT8's thermal behaviour.** Whether less computation means less heat. Compare
  temperatures against FP16 under sustained 8-thread load. Under fanless
  conditions this may matter more.
- **The effect on three-node scaling efficiency.** As the per-node ceiling rises,
  the network becomes a bottleneck relatively sooner.

  > ⚠️ The 1.43 / 4.3 Gbps first written here is **wrong.** Converting MiB/s to
  > Gbps used the binary prefix (÷1024). Network speeds are decimal.
  > The correct values: `1,228,800 × 157.2 × 8 = 1.545 Gbps/node`, 4.636 Gbps
  > across three nodes.
  > FP16 too, at 2.486 Gbps across three nodes, exceeds a single 2.5GbE link.
  > See `RESULTS.md` §8.1.

  **The S2 scalability experiment's design has to be re-examined.**

---

# The shared context experiment — "0 errors" is not "correct"

- Written: **2026-08-11 16:45 KST** (commit `547333c`; the measurement is
  `d228cda` at 16:33)
- Node measured: `king`
- Tool: `crates/npuforge-rknn/native/shared_context_test.c`

While implementing the `npuforge-rknn` backend, a choice was needed between
sharing a context and one per thread. `environment-matrix.md` §3.1 had concluded
that "RKNN Runtime 2.3.0 is thread-safe". Believing that outright, one context
would do.

## Why it was suspected

One inference is three calls.

```text
rknn_inputs_set  ->  rknn_run  ->  rknn_outputs_get
```

**Individual calls being thread-safe and this sequence being atomic are different
things.** A thread cutting in between can mismatch input with output.

And re-reading §3.1's measurement, **it counted only API return codes.** It never
compared output contents. Even with results mixed up, `ok / err` comes back
`40 / 0`.

## The measurement

`native/shared_context_test.c`. Each thread is given a different input; a
reference output is first captured by inferring alone, then the concurrent
results are compared against each thread's own reference.

`king`, FP16, 4 threads × 50.

| Configuration | API errors | **Result mismatches** |
|---|---:|---:|
| Shared context | 0 | **200 / 200 (100%)** |
| Per-thread dedicated context | 0 | 0 / 200 (0%) |

**A shared context produces 100% wrong answers without a single error.**

## What was dangerous

This defect has every one of the following properties.

- It leaves no exception and no error code
- It never reproduces in a single-threaded test
- The throughput metric actually looks better (in §3.1, two threads sharing gave
  34.8 inf/s against 33.2 dedicated — **it was producing wrong answers faster**)
- Even by eye the detections look "plausible" — being results from another frame,
  they are not garbage

Had the benchmarks been run in that state, it would very likely have reached the
talk **with all throughput figures valid and only the detections quietly wrong.**

## What was done

- A correction block was added to `environment-matrix.md` §3.1. The throughput
  figures in the "shared context" row are excluded from performance comparison.
- `RknnContext::infer` takes `&mut self`. **The compiler blocks concurrent
  calls.** Writing a rule in a comment and blocking it with a type are different
  things.
- `ContextPool` creates `worker_count` contexts and occupies them one at a time.
- `supports_concurrent_infer = true` stays but its basis changed. Not "the
  runtime handles it" but **"the backend serializes through a pool"**.

## What this means for the project

The same mistake for the third time. §3's "Meta" records two of them.

```text
1. read RKNN_QUERY_PERF_RUN.run_duration as NPU occupancy time
   -> it included queue wait
2. sampled NPU load at 0.2 s intervals with delayms=3000
   -> it was reading a 3-second average
3. judged thread-safety by API return codes alone
   -> never compared results
4. judged throttling by NPU clock alone  (§12)
   -> the CPU clocks in the same log were falling 63-70%
```

The commonality is clear. **A metric was trusted by its name without checking
what it counts.**

One more item for `preflight-check.sh`.
**Check accuracy before measuring performance.** A configuration that produces
wrong answers fast must be stopped from winning a benchmark.

---

# Bench tool design — building the mistakes into the tool

- Written: **2026-08-11 17:15 KST** (implementation commit `b2cae0d`, 17:12)
- Subject: `crates/npuforge-bench/`
- Verification: an end-to-end Mock 3-node run

## Why this section exists

`npuforge-bench` is not a new measurement result but **a tool.** But its design
rationale comes entirely from the failures in the preceding sections, so it is
recorded here.

Collecting the measurement mistakes so far, they are of three kinds.

```text
A. did not check what a metric counts
   - read run_duration as NPU occupancy time (§3)
   - sampled at 0.2 s with delayms=3000 (§3)
   - judged thread-safety by API return codes alone (§9)

B. compared values without noticing a condition had changed
   - compared two measurements with different load profiles and misread a 19 C gap
     (board-worklog.md §2.19)
   - a stale IP in the docs led to misdiagnosing a node as dead (§2.20)

C. nearly treated invalid data as valid
   - the throughput of a board reset by insufficient adapter capacity (§2.17.2)
```

**Writing "let's be careful" in a comment did not work.** All three happened
while knowing better. So the tool enforces it.

## The rules built into the tool

| Past mistake | What the tool does |
|---|---|
| The first inference's latency spikes | Warmup requests excluded from aggregation |
| A reset board read as "degraded performance" | A change in `boot_id` → run invalid |
| p99 computed from 20 samples | Fewer than 100 successes → invalid |
| — | Failures excluded from throughput and per-node shares |
| — | Conditions (concurrency, seed, policy, node count) carried with the result |
| — | Percentiles are nearest-rank; interpolation forbidden |

### Why failures do not go into throughput

Include them and **throughput is highest when every node is dead.** Failures
return immediately, so requests per second explode. Read this metric as-is in the
S4 failure-handling experiment and the result reads "performance improves during
an outage".

Per-node shares are the same. A failed request's `node_id` is empty, and counting
that makes a dead node look like it "processed a lot".

### Why percentiles are not interpolated

Linear interpolation invents **values never actually observed** when samples are
few. Interpolating p95 over 1–10 gives 9.55, and no request experienced that
latency. Writing "p95 = 9.55 ms" in a presentation makes it a computation, not a
measurement.

It is fixed to nearest-rank and the definition is pinned in the module
documentation.

### Why the invalidity warning prints before the numbers

```text
!!!!!! THIS RUN IS INVALID !!!!!!
  - error rate 100.00% exceeds the 1.00% allowance
  - 0 successful samples is below the minimum of 100
Do not quote the figures below.

requests : 200 (0 succeeded / 200 failed, ...)
```

Show the numbers first and people believe them first. Put the warning below and
the first screenful without scrolling is the numbers, and those numbers get
copied into a table.

Invalid runs are **not deleted.** They have to remain with their reason for the
cause to be traceable, and repeated reboots are themselves a finding.

## One problem caught during implementation

The first approach queried node state via the heartbeat RPC, because the
scheduler had no node listing API.

**But that overwrites the scheduler's node state.** A heartbeat is a call that
records observations, and a bench sending an empty `health` has the scheduler
accept it as a real observation and zero out temperature and queue depth. It
**contaminates the state of the thing being measured, immediately before
measuring it.**

A read-only `ListNodes` RPC was added separately. This too is a variant of type A
(using an API without checking its side effects).

The policy name was also made to prefer the value the scheduler reports. Typing
`--policy round-robin` by hand goes wrong, and **a result labelled with the wrong
policy name ruins the whole S3 policy comparison.**

## What the tool does not guarantee

The load is a closed loop. Concurrency N is fixed and the next request is sent
after the response arrives.

That approach is vulnerable to **coordinated omission.** When the system slows
down the client slows down with it, so the latency distribution comes out
optimistic. A slow request delays the launch time of subsequent requests, and
that delay is not charged to any request's latency.

**Never quote absolute latency as an SLA.** Use it only for comparison between
configurations. That sentence goes into the result file's `caveats` so it is
visible even when the results are read in isolation.

An open model (fixed target RPS) was not used because the node queue is finite.
Raising RPS quickly ends in `NPF-1303` rejections and the latency distribution
cannot be seen. If both models are needed, they are added in M7.

## The Mock 3-node results

```text
requests : 395 (395 succeeded / 0 failed)
throughput: 23.3 inf/s  (17.0 s)
retries: 31

latency (round trip, ms)
  min 23.7  p50 45.1  p90 256.9  p95 302.8  p99 3092.9  max 3214.4

per-node distribution
  mock-01     160   40.5%   p50    30.3 ms  p99  3059.0 ms
  mock-02     157   39.7%   p50   220.8 ms  p99  3178.3 ms
  mock-03      78   19.7%   p50    30.0 ms  p99    75.1 ms
```

**A p99 of 3.09 seconds is not a bug.** Requests hitting the Mock nodes'
`queue_timeout_ms = 3000` were rejected with `NPF-1303`, and the scheduler
retried them on another node and succeeded (31 retries). With concurrency 6 and
`worker_count = 1` on the Mock, the queues build up.

That is, these figures are **evidence the retry path actually works.** On real
hardware `worker_count = 8` gives a different picture.

Exit codes support unattended overnight runs.

```text
0  valid
3  invalid   <- the script uses this to decide whether to re-run
2  argument error
1  execution failure
```

## Next

The M3 real-hardware measurement (1/2/3-node scaling efficiency) starts **only
once the 10G aggregation setup exists.**

As §8 calculated, a single INT8 node demands **1.545 Gbps** (the 1.43 in §8 was
corrected as a binary-prefix error). The current 1GbE management network
**cannot even take one node's worth.** Measuring now would misreport a network
bottleneck as scaling efficiency — repeating a type B mistake exactly.

Until the switch arrives, work proceeds on Prometheus metrics,
`preflight-check.sh` and configuring `dealer` as an NTP server.

---

# The CPU governor effect — every existing figure was on `ondemand`

- Written: **2026-08-12 10:16 KST**
- Node measured: `king` (re-measured under identical conditions)
- Trigger: `preflight-check.sh` blocked `ondemand` as a hard failure, so it was
  changed

## Results

The same tool and conditions (8 threads, 120 s, `king`) with only the governor
changed.

| Model | `ondemand` | `performance` | Change |
|---|---:|---:|---:|
| FP16 | 79.0 inf/s | **84.3 inf/s** | **+6.7%** |
| INT8 | 146.2 inf/s | **157.2 inf/s** | **+7.5%** |

Mean latency fell alongside (FP16 100.9 → 94.5 ms, INT8 54.6 → 50.8 ms).

## Why the CPU governor changes NPU throughput

One inference is not just NPU execution.

```text
set input (CPU) -> NPU execution -> get output and dequantize (CPU)
```

The structure was already confirmed in §3 as the reason 8 threads beat 2:
pipelining occurs, with one thread in its CPU section while another occupies the
NPU. **That CPU section's speed feeds directly into total throughput.**

`ondemand` raises frequency with load, but while waiting on the NPU the CPU looks
lightly used and the frequency comes down. The next request's CPU section then
starts at a slow clock. The observed idle clock wobbling between 1008 and
1800 MHz is this (the maxima are A53 2016 / A72 2208 MHz).

## What this result means

**Every throughput figure up to §8 is on `ondemand`.** They must not be compared
directly with figures from here on. The documents' settled figures were updated
to `performance`.

But **the conclusions do not change.**

| Conclusion | Basis |
|---|---|
| INT8 is 1.85× FP16 | on `performance`, 157.2/84.3 = **1.86×**. Unchanged |
| The three application optimizations are meaningless | the governor is not an application optimization |
| The ceiling is the per-call time in the serialized section | reinforced, if anything, by the CPU section being part of that time |

§8 wrote that "what sets the ceiling is not the ioctl count but the time one call
holds on", and this result shows **that time includes the CPU work either side.**
The ioctl count will be the same 76 regardless of governor.

## Actions taken

`scripts/set-cpu-governor.sh` fixes all three nodes to `performance` and makes it
permanent with a systemd unit.

Survival across reboot was actually confirmed. After rebooting `jack` and its
`boot_id` changing from `6caea6bd` to `83d2981f`, the governor held.

The `cpufrequtils` package was not used. Installing it would make the three
nodes' package lists differ and break environment matching.

## Idle temperature barely rose

| Node | `ondemand` | `performance` |
|---|---:|---:|
| king | 36.1 °C | 37.0 °C |
| queen | 35.2 °C | 36.1 °C |

The clock is always at maximum but idle cores still halt, so heat output does not
rise. **It does not burden the fanless S0 measurement.**

## One remaining pitfall

While writing `set-cpu-governor.sh`, nesting a heredoc and sudo inside ssh meant
**the unit file was never created and yet the exit code was 0.** The script
reported "apply failed", but without a value-confirmation step it would have gone
through as "made permanent".

It was changed to create the unit file locally and transfer it with `scp`. The
same family as the remote execution pitfalls in board-worklog.md §2.21.

---

# The want_float=0 switch and CPU throttling — Claude's interpretation

- Written: **2026-08-12 17:40 KST**
- Node measured: `king`
- Trigger: the network calculation promoted `want_float=0` to an M3 precondition

## 1. want_float=0's throughput effect (previously unmeasured)

§5's `+5.4%` was measured on FP16 and, as noted, could not be carried across to
INT8. It was measured. `king`, 8 threads, 120 s.

| Model | `want_float=0` | `want_float=1` | Gain |
|---|---:|---:|---:|
| INT8 | **156.7 inf/s** | 133.6 inf/s | **+17.3%** |
| FP16 | 66.9 inf/s | 57.8 inf/s | **+15.7%** |

Far larger than §5's +5.4%. §5 was a mostly single-thread condition; here, with 8
threads concurrent, output conversion holds the serialized section longer.

**The network and throughput point the same way.** The output becomes a quarter
and throughput rises 15–17%. There is no reason not to use `want_float=0`.

> And only now do I realise that **`sustained_load_test` had hardcoded
> `want_float=0` from the beginning.** So the documents' 157.2 / 84.3 were
> already on `want_float=0`. Only the Rust backend was on `true`, so this switch
> **brought the software in line with the measurement conditions.**

## 2. But something bigger came out — the CPU bends from heat

Re-measuring FP16 gave 66.9 rather than 84.3. INT8 matched at 156.7. FP16 alone
differing was odd, so it was checked.

**The cause was measurement order.** The FP16 measurement was appended after two
INT8 measurements.

| Starting temperature | FP16 throughput |
|---|---:|
| 53.6 °C (after cooling) | **81.6 inf/s** |
| 71.2 °C (mid continuous measurement) | 66.9 inf/s |

**−18%.** So the clocks were observed directly under load. The governor is
`performance`.

```text
        NPU temp   npu_clk   cpu4(A72)   cpu0(A53)
 +15s   86.8 C     950 MHz   2208 MHz    2016 MHz
 +30s   90.4 C     950 MHz   1416 MHz    1200 MHz
 +45s   89.5 C     950 MHz   1008 MHz     816 MHz
 +60s   87.8 C     950 MHz    816 MHz     600 MHz
+120s   87.8 C     950 MHz    816 MHz     600 MHz
```

**The NPU clock never drops from 950 MHz. The CPU falls 63–70%.**

Over a 300-second sustained measurement, throughput converges like this.

```text
 +10s  81.6 inf/s   <- start
+120s  63.6
+300s  59.7         <- steady state
mean   71.3 inf/s
```

**−27% against the start.**

## 3. What this overturns

### 3.1 "No throttling" was wrong

`RESULTS.md` §2.3 and `environment-matrix.md` §9.0 say this.

> No throttling — all 928 samples at NPU 950 MHz, never dropping once

**Only the NPU clock was looked at.** The CPU clocks were recorded in the same
log and were not used in the verdict. Despite §11 already confirming that one
inference is `set input (CPU) → NPU → get output (CPU)` and that the CPU sections
feed directly into throughput, the throttling verdict was made on the NPU alone.

**This is the fourth mistake of the same type.** Added to §3's "Meta" list.

```text
1. read run_duration as NPU occupancy time     -> queue wait included
2. sampled NPU load with delayms=3000          -> a 3-second average
3. judged thread-safety by API return codes    -> results never compared
4. judged throttling by NPU clock alone        -> the CPU was bending
```

### 3.2 The CPU governor conclusion narrows in scope

§11's **+7%** is a 120-second measurement. That window is before the CPU is fully
downgraded. **It cannot be asserted that `performance` is favourable under
sustained load.**

`performance` holds the maximum clock even at idle, so it has less thermal
headroom at the start of load. It may heat up faster and be downgraded earlier.

**Not measured.** `ondemand` and `performance` have to be compared under
identical 300-second conditions. Until then §11's +7% is read only as **"a gain
in short measurements".**

### 3.3 The peak vs sustained gap becomes one of this project's central figures

Until now it was recorded as "peak vs sustained, about 10%". This measurement is
**−27% over 300 seconds.** And it pins the cause on **CPU thermal throttling**
rather than the NPU.

> The TOPS a vendor publishes is instantaneous performance. What actually
> collapses first at the fanless edge — **it was the CPU handling either side,
> not the NPU.**

That is a far better narrative for the talk. Running S0 properly makes it a
settled figure.

## 4. Actions

**Done**

- `want_float` exposed as node configuration (`[worker] want_float`) with the
  default changed to `false`. The Rust backend now matches the measurement tool's
  conditions
- The blob format bumped to **v2**, carrying `qnt_type`, `zero_point` and `scale`
  per tensor. Without those, sending int8 leaves the receiver unable to interpret
  it
- Dequantization confirmed on a real board to match float32 (9 tensors,
  **maximum error 9.5e-7** — the limit of float32 precision)

**To do**

- Compare `ondemand` vs `performance` under identical 300-second conditions
- Run S0 for 30 minutes to settle steady-state throughput and the downgrade
  timing
- Fix `run-thermal-comparison.sh` to **include CPU clock** in the thermal verdict
- Correct the "no throttling" wording in `RESULTS.md` §2.3 and
  `environment-matrix.md` §9.0 (included in this commit)
