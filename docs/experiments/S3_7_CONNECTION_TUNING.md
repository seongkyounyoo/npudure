# S3.7 — Connection Tuning (a: sweep, b: concurrency, c: RPS)

*[한국어 원문](S3_7_CONNECTION_TUNING.ko.md)*

- Experiment ID: **S3.7a · S3.7b · S3.7c** (complete)
- Measured: 2026-08-20
- Code: `4e64bf4` (the `[transport]` settings; defaults behave identically to the freeze)
- Raw data: [`../../results/connection-sweep-20260820/`](../../results/connection-sweep-20260820/)
- Predecessor: [`S3_6_H2_CHANNEL_AB.md`](S3_6_H2_CHANNEL_AB.md)

---

## 0. What this experiment answers

S3.6 compared only 1 → 4 connections and saw +21.5%. **There was no basis for 4
being optimal**, and the throughput gain came with **p95 46% worse.**

So S3.7 frames this not as "find maximum throughput" but as a problem of
**operating point selection**.

```text
S3.7a  connections 1/2/4/8/16 at fixed c32   -> shortlist Pareto candidates   <- done
S3.7b  concurrency sweep for the shortlist   -> establish the real operating point
S3.7c  RPS OFF/ON at that operating point    -> freeze optimized gRPC
```

---

# S3.7a — Fixed-load connection-count A/B

## 1. Method

One node (king), **fixed c32**, 60 s, connections 1/2/4/8/16, **5 runs** per
condition (25 total). The window stays at its default (S3.6's conclusion: a
64 MB-class enlargement is −36.3%). The order reverses each round so temperature
and elapsed time do not land on one condition. Each run counts the node's actual
TCP connections with `ss` and records it.

> **This is not each setting's ceiling.** Load is fixed at c32, which is good for
> comparing the *pure effect* of connection count, but adding connections may
> have moved the saturation concurrency above c32. That is why S3.7b exists
> separately.

## 2. Results

Error rate **0** throughout. All latencies are **the run-to-run average of
run-level percentiles** (not pooled — S2 §7.4.1).

| conn | TCP measured | throughput | vs c1 | p50 | p95 | p99 | max | →node |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **1** | 1 | 115.6 ± 0.7 | — | 268.0 | **392.4** | **452.4** | 597.6 | 114.9 |
| **2** | 2 | **134.4 ± 0.7** | **+16.3%** | 226.7 | **438.2** | **514.9** | 679.7 | 92.5 |
| **4** | 4 | **139.5 ± 0.2** | **+20.7%** | 169.5 | 561.6 | 698.2 | 944.5 | 63.1 |
| **8** | 8 | 139.1 ± 0.6 | +20.4% | 157.4 | 597.0 | 827.0 | 1222.5 | 56.9 |
| **16** | 16 | 136.8 ± 0.7 | +18.4% | 173.4 | 584.2 | 895.1 | 1481.5 | 65.3 |

- Main figure: [`fig_sweep_pareto.png`](../../results/connection-sweep-20260820/figures/fig_sweep_pareto.png)
  (X = p95, Y = throughput, points = connection count)
- Supporting: [`fig_sweep_throughput.png`](../../results/connection-sweep-20260820/figures/fig_sweep_throughput.png),
  [`fig_sweep_latency.png`](../../results/connection-sweep-20260820/figures/fig_sweep_latency.png)

## 3. Interpretation

### 3.1 Connection parallelism has a knee

> ⚠️ **[Corrected — §4]** Everything below was measured at **fixed c32**. All
> three configurations have their operating point at c12, so c32 is the overload
> region, and much of the tail degradation seen here is **overload queueing
> rather than connection count**. The observation that a knee exists holds, but
> that knee is **both a connection knee and entangled with the concurrency
> knee** (§0).

Throughput **flattens at c4** (139.5). c8 at 139.1 is effectively the same and
c16 at 136.8 actually drops. The tail, meanwhile, **degrades monotonically**.

```text
p99   c1 452  ->  c2 515  ->  c4 698  ->  c8 827  ->  c16 895
max   c1 598  ->  c2 680  ->  c4 945  ->  c8 1223 ->  c16 1482
```

**This is as far as the data proves.**

> **Beyond c4, additional connection parallelism does not improve throughput on
> a c32 workload and degrades tail latency.**

⚠️ **"It bends because of connection management cost and queueing" is not an
established cause.** Several possible contributors are mixed together and none
was separated — H2-internal queueing, per-connection in-flight imbalance, TCP
processing, bursty NPU arrivals (the same list as S3.6 §4.3).

Another interesting point is that **the median and the tail move in opposite
directions.** Adding connections makes the average request faster (p50 268 →
157) while some requests get much slower (p99 452 → 895). It is not "more
connections are faster" but that **a throughput–tail trade-off is real.**

### 3.2 The real choice is c2 or c4

> ⚠️ **[Corrected — §4]** This section's trade-off was measured at **c32 (the
> overload region)**. At the c12 operating point, conn4 against conn2 is
> throughput +1.2% for p95 +1.2% — **essentially a draw** (§4.1). The "tail
> 28–39% for +3.8%" below holds only in the overload region.

| | c2 | c4 | what c4 pays |
|---|---:|---:|---|
| throughput | 134.4 | 139.5 | **+3.8%** |
| p95 | 438.2 | 561.6 | **+28.2%** |
| p99 | 514.9 | 698.2 | **+35.6%** |
| max | 679.7 | 944.5 | **+39.0%** |

**c4 gives up 28–39% of the tail for +3.8% throughput.** For real-time inference
that looks like a bad trade.

Against c1 it is clearer — c2 gets **79%** of what c4 gained (+16.3 / +20.7) for
about **a quarter** of the tail cost (p95 +11.7% vs +43.1%).

Recovery of the gap to local direct (161.5):

```text
c1 115.6  - gap 45.9 ->  local 161.5
c2 134.4  recovered 18.8 (41%)
c4 139.5  recovered 23.9 (52%)
```

### 3.3 The heuristic picked c4 — and it was a close thing

The analyser's rule ("lowest p95 among those within 97% of maximum throughput")
picks **c4**, because c2 at **96.4%** missed the threshold **by 0.6 pp**.

> **The threshold is not moved to fit the result.** Lowering 97% to 96% to
> include 96.4% would be post-hoc rationalisation, not a heuristic. The rule
> stays, and **the fact that the rule fails to decide at this boundary is
> recorded as the result.**
>
> This is why §0 pins down that the selected operating point is a deliberate
> engineering heuristic rather than a statistical optimum. It is also why the
> table is published alongside — at a boundary a human has to judge, and the
> basis for that judgement has to be in the table.

## 4. S3.7a conclusion and the next move

- **c8 and c16 are dropped from the S3.7b shortlist.**

  > This is **not** "c8/c16 are inferior at any concurrency". S3.7a is
  > fixed-load at c32 and did not measure their absolute ceilings. The reason is
  > **priority** — the tail cost at c32 is already this large (p99 827 and 895,
  > max 1223 and 1482), so the expected value against further search cost is
  > low. We can come back if needed.

- **c2 and c4 go forward as S3.7b candidates.** At fixed c32 neither wins. If c2
  is not yet saturated it could overtake at a higher concurrency, and c4's tail
  could collapse faster as concurrency rises.

  Characterised as they stand: **c2 = efficiency point, c4 = performance
  point.** Which is the operating point is decided by looking at the ceiling.

## 5. Limitations

- **These are not each setting's ceiling** (note in §1). It is a fixed-c32
  result.
- Percentiles are run-level averages, so they show the tail lower than pooled.
  Valid for comparing conditions, not to be quoted as absolutes (S2 §7.4.1).
- **The cause of the p95/p99 degradation remains unverified.** None of S3.6
  §4.3's five candidates (per-connection in-flight imbalance / H2-internal queue
  variance / bursty NPU arrivals / transport queueing / the general tail growth
  that accompanies higher throughput) was excluded.
- This is a 1-node result. At three nodes the server holds N×3 connections
  (S3.8).

## 6. Reproduction

```bash
bash scripts/run-connection-sweep.sh sweep 5     # 25 runs, about 40 minutes
PYTHONIOENCODING=utf-8 python scripts/analyze-connection-sweep.py \
    results/connection-sweep-20260820/raw/results.csv
python scripts/make-sweep-figures.py \
    results/connection-sweep-20260820/raw/results.csv \
    results/connection-sweep-20260820/figures
```

---

# S3.7b — Concurrency sweep

## 0. The thing to tune was two-dimensional, not one

This is the structure that emerged through S3.7a and b. **There are two knees.**

```text
Concurrency knee   how many requests in flight are needed to saturate the device?
Connection knee    how many connections should those requests be split across?
```

So what needs tuning is not "connection count" alone but a **two-dimensional
operating point of load concurrency × connection parallelism**.

This lands exactly on NPUDure's original question — "why won't it scale?"
**Past saturation, pushing more in does not make the NPU do more work; it only
piles queues up inside the system.** §2 below is that captured by measurement.

## 1. Definition of operating concurrency (an experimental rule)

Pinned to a number. Without it, results like 132.8 / 134.1 / 134.3 turn "where
is the knee" into a human judgement every time.

> **operating concurrency = the lowest concurrency delivering at least 98% of
> peak throughput**

**Why 98%**: the observed run-to-run SD is around ±1 inf/s, so a 99% threshold
would overlap measurement noise. This definition lives as a constant in
`analyze-concurrency-sweep.py`.

## 2. First range (c24–c64) — all of it was overload

Candidates **c2 · c4**, 3 runs each.

| conc | conn2 tp | conn2 p95 | conn2 p99 | conn4 tp | conn4 p95 | conn4 p99 |
|---:|---:|---:|---:|---:|---:|---:|
| **24** | **134.3 ± 1.1** | **306.9** | **357.5** | **139.3 ± 1.2** | **390.9** | **480.2** |
| 32 | 133.7 | 431.5 | 505.7 | 138.3 | 576.5 | 715.0 |
| 40 | 134.2 | 572.1 | 674.4 | 137.7 | 719.5 | 932.9 |
| 48 | 133.8 | 697.6 | 832.0 | 137.6 | 958.6 | 1200.9 |
| 64 | 132.9 | 946.0 | 1132.3 | 137.9 | 1254.4 | 1566.7 |

Error rate 0 throughout.

**Throughput is completely flat across c24–c64** (conn2 ≈ 134, conn4 ≈ 138),
while the tail grows nearly linearly — conn4 @c64 reaches p99 1567 ms, max
2128 ms.

> **This entire range is past saturation.** What the data says:
>
> **Throughput saturation occurs at concurrency ≤ 24. Additional concurrency
> past saturation does not increase throughput and only increases tail latency.**

Textbook queueing. Requests pushed in beyond that go to a queue, not to
computation.

### 2.1 ~~The trade-off is stable across load~~ — **wrong (refuted in §4)**

| | S3.7a @c32 | S3.7b @c24 |
|---|---:|---:|
| throughput | +3.8% | +3.7% |
| p95 | +28.2% | +27.4% |
| p99 | +35.6% | +34.3% |

The two agreed closely, so this was initially written up as "not a coincidence
of one concurrency but **a trade-off that 4 connections create as such**".
**That interpretation was wrong.**

c32 and c24 agreed because **both are in the overload region and we saw the same
phenomenon twice.** Descending to the true operating point (c12) in §4, the p95
penalty vanishes from **+28% to +1.2%**. It was not a property of four
connections but **a property of post-saturation queueing.**

> The lesson: **two measurements agreeing does not mean the interpretation is
> right.** If both are biased in the same direction, reproducibility only
> confirms the bias.

### 2.2 So the sweep direction was wrong

Both candidates peak at **the bottom of the sweep (c24)**. The saturation point
is therefore **below** c24, and the operating point (the lowest concurrency
yielding the ceiling) has not been seen.
→ **Re-sweep downwards over c8/c12/c16/c20/c24.**

## 3. Re-measure the conn1 baseline over the same range

Skip this and the interpretation gets mixed. Placing the two points we have side
by side:

```text
conn1 @c32 ->  115.6 inf/s,  p95 392
conn2 @c24 ->  134.3 inf/s,  p95 307
```

It is tempting to write "2 connections improved **both** throughput and
latency". But **two variables changed at once** — connections 1→2 and
concurrency 32→24. Causality cannot be separated.

The question only stands if each connection count's **operating point is found
by the same rule (§1)** and then compared.

The question narrows to one.

> **Under an identical saturation criterion, how does connection parallelism
> affect throughput and tail latency?**

## 4. Second range (c8–c24) — the result inverts

conn **1 · 2 · 4** on **the same grid** (c8/12/16/20/24), 3 runs each, 45 runs
total. Error rate 0.

| conc | conn1 tp | conn1 p95 | conn2 tp | conn2 p95 | conn4 tp | conn4 p95 |
|---:|---:|---:|---:|---:|---:|---:|
| 8 | 112.1 | 101.3 | 120.4 | 93.9 | 111.0 | 105.0 |
| **12** | **114.8** | **147.6** | **136.4** | **119.8** | **138.1** | **121.2** |
| 16 | 115.1 | 191.5 | 136.2 | 178.7 | 138.8 | 210.0 |
| 20 | 114.9 | 239.2 | 135.1 | 245.4 | 138.5 | 306.1 |
| 24 | 115.9 | 286.8 | 134.0 | 307.7 | 139.1 | 392.3 |

### 4.1 The operating point is c12 for all three

Applying the 98% rule (§1):

| connections | operating conc | throughput | p50 | p95 | p99 | vs peak |
|---:|---:|---:|---:|---:|---:|---:|
| **1** | **12** | 114.8 | 102.1 | 147.6 | 167.2 | 99.1% |
| **2** | **12** | **136.4** | 85.8 | **119.8** | **137.4** | 100.0% |
| **4** | **12** | 138.1 | 83.4 | 121.2 | 145.7 | 99.3% |

**For all three connection counts tested (1, 2, 4), the 98%-criterion operating
concurrency was observed at c12.**

> **Within the tested range, the concurrency knee remained invariant to
> connection parallelism.**

That is evidence the concurrency knee did not move when connection parallelism
changed. It **strongly suggests, but does not prove**, that the two knees are
independent — with a different model, payload size, node count or network it
could move. §0's two-dimensional structure should be read as observed within
this range.

### 4.2 At the operating point there is no trade-off — conn2 dominates conn1

| conn2 vs conn1 @c12 | |
|---|---:|
| throughput | **+18.8%** |
| p50 | **−16.0%** |
| p95 | **−18.8%** |
| p99 | **−17.8%** |

**Throughput rises while latency falls at every percentile.** Not a trade-off but
a **strict Pareto improvement** — on the measured throughput and latency
metrics, that is. It does not mean a whole-system Pareto including CPU, memory
and connection resources.
→ [`fig_sweep_pareto.png`](../../results/s37b-operating-point/figures/fig_sweep_pareto.png)

**conn4 is not absolutely worse.** If maximum throughput is the priority, conn4
is a legitimate choice too (138.1 vs 136.4).

The basis for making conn2 the default operating point is not "conn4 is bad".

| What conn4 gives extra | What conn4 spends extra |
|---|---|
| throughput **+1.2%** — close to measurement variation (SD ±0.3–1.6) | **twice** the connection resources |
| | p99 **+6.0%** |

> **2 connections is the lowest-complexity configuration that captures
> nearly all available throughput.**

conn2 because it takes nearly all of the ceiling with the fewest resources.

### 4.3 So the earlier "tail degradation" was not the connections' fault

S3.6 §4.3 and S3.7a recorded that adding connections worsens the tail (p95 +46%,
+43%). **Those measurements are right and the interpretation was wrong.**

Those experiments all **measured at c32, and c32 is the overload region for all
three configurations** (the operating point is c12). That comparison was
therefore not "which configuration is better" but **"which configuration
degrades more gracefully under overload"**.

```text
seen at c32   1ch -> 4ch :  throughput +21%, p95 +46%   <- overload comparison
seen at c12   1ch -> 2ch :  throughput +19%, p95 -19%   <- operating-point comparison
```

**S3.6's and S3.7a's numbers are not wrong. The question was different.**

| What was asked | Answer |
|---|---|
| **Fixed-c32 comparison** — how does each configuration behave **under overload**? | more connections raise the ceiling slightly but **amplify the tail more** |
| **Operating-point comparison** — which configuration is better **in operation**? | conn2 dominates conn1 on both axes |

So the c32 results are not to be discarded but remain **a separate valid
result** — a result about overload behaviour. They just must not be used as
grounds for operating decisions.

> **Optimize at the operating point, not in the overload region.**

Comparing configurations at a fixed load without defining the operating point
means **seeing overload behaviour rather than a configuration effect, and the
conclusion can invert.** This is the most practical lesson S3.7 leaves.

## 5. S3.7b conclusion

> **Selected operating point: 2 connections @ concurrency 12
> — 136.4 inf/s, p95 119.8 ms, p99 137.4 ms**

Against the conn1 baseline under the same rule (114.8 @c12): **throughput
+18.8%, p95 −18.8%**. Of the 46.7 gap to local direct (161.5), **21.6 (46%) was
recovered by configuration alone**, with the tail improving alongside.

## 6. Limitations

- **Grid resolution.** The knee lies between c8 (88% of peak) and c12, and with
  a step of 4 **we do not know whether c12 is the true knee or c10**. This is
  fine for comparing three configurations on the same grid, but the caveat
  attaches when quoting the operating point as an absolute.
- The analyser flags "saturation unconfirmed (peak at the top of the sweep)" for
  conn1 and conn4. But conn1 is c12 114.8 ± 0.7 vs c24 115.9 ± 0.8, and conn4 is
  c12 138.1 ± 1.6 vs c24 139.1 ± 0.8 — **noise within a flat region**. The
  warning is left in place, conservatively.
- This is a 1-node result. At three nodes the server holds 2×3 = 6 connections
  (S3.8).
- Percentiles are run-level averages (S2 §7.4.1).

# S3.7c — RPS at the selected operating point

Settled operating point: **2 connections @ c12**.

This now becomes an experiment that asks one question with nothing else mixed in.

> **Does RPS improve the selected operating point?**

If a null comes back as in S3.5b, **that is a good result too**. It would
substantially weaken the hypothesis that "RPS was ineffective because there was
only one flow" — if nothing changes with two flows, evidence accumulates that
**IRQ/RX-side distribution is not this workload's bottleneck.**

`rps_cpus` OFF/ON at the settled operating point. In S3.5b there was one flow
and nothing to distribute. Now there are several flows, and under S3.6's 4ch
condition CPU0 was at busy 81% / soft 74%.

**If c2 and c4 tie ambiguously in S3.7b, run the RPS A/B on both.** Ten runs per
condition suffices, so it is cheap, and **the RPS effect may differ between two
flows and four** — which is itself information aimed at ②-a (TCP per-flow
processing).

- If it rises → releasing the single-connection constraint exposed a NIC
  processing bottleneck
- If unchanged → CPU0 softirq is **merely a correlation, not a throughput
  limiter** (a much stronger exclusion than S3.5b alone)

## Result — a null. And this null is much stronger

conn2 @ c12 fixed, `rps_cpus` = `00` (CPU0 only) vs `fe` (cores 1–7), 5 runs
each.

| | throughput | p50 | p95 | p99 | board idle | **CPU0 busy** | **CPU0 %soft** |
|---|---:|---:|---:|---:|---:|---:|---:|
| RPS off | **136.8 ± 0.6** | 85.4 | 119.1 | 137.7 | 49.3% | **78.7%** | **68.0%** |
| RPS on | 135.6 ± 0.4 | 86.4 | 119.7 | 139.3 | 49.1% | **74.6%** | **56.0%** |
| difference | **−0.8%** | +1.2% | +0.5% | +1.2% | — | −4.1 pp | **−12.0 pp** |

Error rate 0. The −0.8% throughput difference is within the SD (±0.4–0.6).

### Why this null is stronger than S3.5b's

In S3.5b there was a counter-argument — **with one flow, RPS had nothing to
distribute.** That counter-argument is blocked here.

1. **There are two flows.** There really is something for RPS to hash apart.
2. **RPS actually worked.** CPU0 %soft came down 12 pp, **68.0% → 56.0%**, and
   CPU0 busy fell from 78.7% to 74.6%. The setting was not ignored.
3. **CPU0 was not idling.** At 78.7% busy it was under real load (comparable to
   81% under S3.6's c32/4ch condition). "There was too little load for RPS to
   act on" does not hold either.

> **At the selected operating point, RPS reduced CPU0 softirq load
> substantially but produced no measurable throughput or tail-latency
> improvement. Therefore, CPU0 receive-side processing was not
> performance-limiting under the tested configuration.**

**Read the scope precisely.** What this says is not "CPU0 softirq is not a
limiter" but **"it is not a limiter at this operating point, in this
configuration"**. Under a different load, model, payload size or node count it
could differ.

Within that scope it is quite strong — the mechanism demonstrably worked and did
not touch the end-to-end limiter. S3.5's (§4.3) nomination of CPU0 as "the next
bottleneck candidate" is excluded **for this configuration.**

## Overall S3.7 conclusion

| Candidate | Verdict |
|---|---|
| Link bandwidth | excluded (51% per direction) — S3.5 |
| Board CPU capacity | excluded (49–63% idle) — S3.5, S3.7c |
| Server and scheduler | **reopened** — excluded at baseline, but optimized 3N eff 95.3% (S3.8) |
| **CPU0 softirq / RPS** | **excluded.** Throughput unchanged after taking 12 pp off — S3.7c |
| H2 flow control window | enlarging to 64 MB is harmful at −36.3% — S3.6 |
| **Connections per node** | **primary constraint.** 1→2 gives +18.8% and improves the tail — S3.7b |
| protobuf, copies, syscalls | **unseparated.** May lie within the remaining 15.5% |

**Selected operating point: 2 connections **per node** @ concurrency 12 —
136.4 inf/s, p95 119.8 ms, p99 137.4 ms**

> **Always state the unit.** `[transport] node_connections` is a **per-node**
> value (`GrpcNodePool` creates N channels per `NodeId`). It is not a
> cluster-wide total.
>
> | Nodes | node_connections | cluster-wide connections |
> |---:|---:|---:|
> | 1 | 2 | 2 |
> | 2 | 2 | 4 |
> | 3 | 2 | 6 |
>
> Fixing "2 connections" cluster-wide in S3.8 would not preserve the per-node
> condition, and at 3N **connection supply itself would become a new
> bottleneck**. That is an entirely different experiment and must not be
> confused with this one.

**15.5%** still remains to local direct at 161.5.

> ⚠️ **The exclusion table shrank the candidate space; it did not identify what
> the remaining 15.5% is.** Several candidates remain.
>
> | Candidates for the remaining gap |
> |---|
> | protobuf serialization |
> | memcpy / buffer ownership (`to_vec()` and the like) |
> | syscall / submission path |
> | HTTP/2 implementation overhead |
> | userspace scheduling (tokio workers ↔ blocking pool contention) |
> | NPU submission / RKNN runtime overhead |
> | other |

**io_uring is now a legitimate candidate. But "the next bottleneck is syscalls
and copies" is not yet established.** So S4's question is framed this way:

```text
no    Does io_uring recover the remaining 15.5%?
yes   Is the syscall / submission path actually a meaningful cost?
```

Confirm syscall and copy cost by profiling first, and go to io_uring only if the
answer is yes. That is TECHSPEC §15.1's order and the same principle held since
S3.5 — **measurement decides implementation.**

Next is **S3.8** — re-verify 1N/2N/3N scale-out at this operating point.
