# S3.9b — Node-side Residual Cost Profiling

*[한국어 원문](S3_9B_NODE_RESIDUAL.ko.md)*

- Experiment ID: **S3.9b**
- Measured: 2026-08-21
- Code: `62855bd`
- Status: **complete** (4 conditions × 45 s of collection, 0 errors)
- Raw data: [`../../results/node-residual-20260821/`](../../results/node-residual-20260821/)
- Predecessors: [`S3_5_TRANSPORT_PROFILE.md`](S3_5_TRANSPORT_PROFILE.md) ·
  [`S3_9A_SCALEOUT_PROFILE.md`](S3_9A_SCALEOUT_PROFILE.md)

---

## 1. Research Question (narrow)

> **In the residual gap between 161.5 and 135.5, do node-side serialization,
> copy and syscall costs account for a meaningful share?**

**Explaining the whole gap is not the objective.** S3.9a separately surfaced the
scale-out tail/TCP cost, so there is no reason a node-side profile should have
to account for all 26.0 inf/s. Whatever is not explained stays unexplained.

The decision rule was fixed **before measuring**.

| Result | Decision |
|---|---|
| syscall and copy are **large enough** | proceed to S4 io_uring |
| **small** | **cancel/shelve S4** |
| **some other term is large** | record that term only. If it is outside the core scope, dig no further |

## 2. Method

The decisive difference from S3.5 is that **this measures at the operating
point**.

```text
S3.5    c32 . conn1   116.6 inf/s   overload . baseline
S3.9b   c12 . conn2   136.6 inf/s   operating point . optimized
```

Overload-region values are not used for operating decisions (README §4.1). This
repository has already fallen into that trap once — the 13.2% misquotation
incident.

- One node (king only). queen and jack are brought down so round-robin cannot
  split the load, and a probe leaves evidence that **the only responding node ID
  is king**.
- Of the 80 s of load, only **45 s starting at t+20** is collected, excluding
  the ramp and warmup.
- Four conditions: `idle` (instrument floor) / `op` (operating point) /
  `strace` / `local` (direct, 8 threads).

### 2.1 Choice of instrument — there is no perf

The boards have no `perf`, `bpftrace` or `gdb` (kernel 6.1.141, vendor tree).
Symbol-level profiling is impossible. Instead, the **utime/stime split from
`/proc/PID/stat`** is used.

```text
utime  user time   - protobuf serialization, user-space copies, HTTP/2 framing
stime  kernel time - syscall entry, TCP stack, copy_to_user, skb, driver
```

**What io_uring reduces is a portion of stime.** So the whole of stime is
io_uring's absolute upper bound, and what is actually recoverable is smaller
than that.

As a secondary instrument, `strace -c` for 10 s. Because ptrace stops the
process at every syscall, the reported residency is **inflated**, so it is used
**only as an upper bound** — if the inflated figure is small, the real one is
conclusively smaller. It is a test that is valid in one direction only.

## 3. Results

### 3.1 Node CPU per request

| Condition | throughput | utime/req | stime/req | **CPU-ms/req** | user% | kernel% |
|---|---:|---:|---:|---:|---:|---:|
| op (operating point) | 136.6 | 14.50 | 11.09 | **25.59** | 56.7 | 43.3 |
| local direct | 157.9 | 5.14 | 4.10 | **9.23** | 55.6 | 44.4 |
| **transport cost** | | **9.37** | **6.99** | **16.35** | **57.3** | **42.7** |

The operating point of 136.6 agrees with S3.8's 135.5 ± 0.4 and S3.7b's
136.4 ± 0.3 — confirmation that the condition was set up correctly.

> ⚠️ The local figure of 157.9 is the average over the **full 80 s** and so
> includes the ramp. The steady-state rate within the collection window
> (t+20–65) was 162.6. That difference overestimates local's per-request CPU by
> about 3%, which **overestimates rather than underestimates the transport
> cost** — so it does not weaken the conclusion below (that the cost is small).

### 3.2 No core is saturated

```text
op    cpu0  soft=68.3  idle=21.2   <- the only hot core (78.8% busy)
      cpu1-3            idle 61-64
      cpu4-7            idle 42-47
      overall           idle 48.9
local overall           idle 82.5   softirq 0
```

Even the hottest core, cpu0, has 21% left. cpu0's load is mostly **softirq**
(the NIC's single receive queue) — and **S3.5 §4.3 already spread that with RPS
and got a −0.2% null.** cpu0 softirq is not a constraint either.

### 3.3 Syscalls — many calls, small cost

`strace -c` over 10 s (about 1,284 requests):

| syscall | residency | calls | calls/req | |
|---|---:|---:|---:|---|
| futex | 30.07s | 48,565 | 37.8 | thread synchronization **wait** |
| ioctl | 24.72s | 68,924 | 53.7 | RKNN driver (NPU submission) |
| epoll_pwait | 9.78s | 37,157 | 28.9 | event **wait** |
| **recvfrom** | 9.50s | 136,602 | **106.4** | request receive ← io_uring target |
| **writev** | 5.91s | 69,245 | **53.9** | response send ← io_uring target |
| **write** | 0.35s | 5,524 | **4.3** | response send ← io_uring target |

**Network syscall residency is 15.77 s / 80.36 s = 19.6%.** The other 80.4% is
futex (synchronization wait), ioctl (NPU driver) and epoll (event wait) — **none
of which io_uring touches.**

## 4. Verdict — **S4 io_uring cancelled/shelved**

Network syscalls per request come to about **165** (recvfrom 106 + writev 54 +
write 4). Even taking aarch64 syscall entry cost **generously at 1 µs**:

```text
165 calls x 1 us = 0.165 ms/req
0.165 / 16.35    = 1.0% of per-request transport CPU
```

Even **assuming** registered buffers eliminate the 1.2 MB copy in both
directions (roughly 0.6–1.2 ms at RK3576 memory bandwidth), the total is
**1.4 ms/req ≈ 8% of transport cost**.

And recovering all of that 8% **would not raise throughput.** The board CPU is
48.9% idle, no core is saturated, and spreading the hottest core's (cpu0)
softirq with RPS produced a −0.2% null.

> **CPU-ms/req is a cost, not a constraint.** Reducing consumption of an
> unsaturated resource does not raise throughput.

```text
Question   Does io_uring recover the remaining 16.1%?
Answer     No. What it targets (syscall entry) is 1% of transport cost, and 8%
           under the most generous assumptions. And CPU is not the constraint.
```

**S4 is cancelled/shelved.** The io_uring item in TECHSPEC §15 changes status
from "necessity unproven" to **"refuted by measurement"**.

## 5. The third branch of the decision rule — record the large term separately

The question bundled three things — serialization / copy / syscall — and the
answer split them.

| Term | Size | Verdict |
|---|---|---|
| **syscall** | ~1% of transport cost | **small** |
| **serialization / user-space copy** | **9.37 ms/req = 57%** | **large** |

**User time exceeds kernel time** (9.37 vs 6.99). The majority of transport cost
is protobuf serialization, user-space copies and HTTP/2 framing. io_uring does
not touch that side.

But **we stop here**, per the third branch of the pre-registered rule: record
the large term, but as long as CPU is not the constraint there is no guarantee
that reducing this raises throughput either. There is not yet grounds to dig in.

## 6. So what is the 26.0 inf/s gap — out of scope, observation only

Not this experiment's job, but the direction is observable. At fixed
concurrency, throughput = concurrency / latency.

```text
op     c12,  136.6 inf/s  ->  mean latency 87.8 ms
local  8 threads, 157.9   ->  mean latency 50.5 ms   (wrapper measured 50,531 us)
                              difference +37.3 ms
```

Of that, node CPU work is only 16.35 ms; the rest is **waiting**. With a 1.2 MB
request and a 1.2 MB response, on the measured link (2.34 Gbps ≈ 292 MB/s)
**pure transfer time alone is about 4.1 ms per direction, 8.2 ms round trip.**
The scheduler hop and queueing add to that.

> The gap looks like a **path-latency** problem rather than a CPU-cost one. The
> lever for reducing it is not io_uring but **payload size** (ADR-008's raw
> 640×640×3 transfer). That is outside S3.9b's scope, so it is **left as an
> observation only.**

## 7. Limitations

- One run per condition (45 s of collection). The utime/stime deltas are stable
  because they accumulate over 45 s, but there is no run-to-run SD.
- The seconds reported by `strace -c` are **residency including blocking**, not
  CPU time. That is why futex and epoll top the list, and the 19.6% network
  syscall share is only valid within that same scale. The primary basis for the
  verdict is the utime/stime split; strace is secondary.
- The 1 µs syscall entry cost is not measured but a **generous** take on the
  usual aarch64 figure. Measuring it would need a microbenchmark, but since the
  1 µs assumption already yields 1%, the conclusion does not flip.
- local direct uses `sustained_load_test` (a separate binary), so its code path
  is not identical to the node's. It is the reference baseline used consistently
  since S3.5.

## 8. The instrument error caught in this experiment

The regular expression parsing the `strace -c` summary read the **`usecs/call`
and `calls` columns swapped**. The call count came out 100× too small, and we
nearly concluded "strace attached to only one thread → the upper-bound test is
invalid". It was caught by comparing against the expected value (83.4 writes per
request, from `/proc/PID/io`).

> When an instrument's output differs from expectation, **suspect the instrument
> first** (README §4.10). This time it was not the measurement but the parser
> that was wrong.

---

## Figure

![The user/kernel split of transport cost and the share io_uring can reach (about 8%)](../../results/node-residual-20260821/figures/fig_transport_cost_split.png)

**`fig_transport_cost_split.png`** — the user/kernel split of transport cost and
the share io_uring can reach (≈8%)

Regenerate: `python scripts/make-experiment-figures.py`
