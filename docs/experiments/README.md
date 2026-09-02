# Experiment Ledger

*[한국어 원문](README.ko.md)*

- Last updated: **2026-08-20**
- Subject: the M3 cluster (RK3576 ×3 + Xeon scheduler), YOLOv8n INT8, `want_float=0`
- Fixed conditions: governor `performance`, active cooling (120 mm fan per node),
  round-robin, 8 workers/node, gRPC (tonic + protobuf), closed-loop bench

> The detail of each experiment is in its own report. This document is the one
> page that shows **what was asked and what was ruled out**.
> Terminology is in [`../GLOSSARY.md`](../GLOSSARY.md).

**In one sentence**

> NPUDure's transport work began as a custom-transport implementation, but by
> removing bottlenecks the measurements actually pointed at, **a standard gRPC
> configuration alone improved three-node throughput by 13.3%** — and along the
> way established that **choosing the operating point and validating the
> experiment come before optimizing anything.**

---

## 1. The ledger

| ID | Question | Scale | Key result | Report |
|---|---|---:|---|---|
| **Pilot** | Do three nodes actually run | 3 runs | 336 inf/s, 0 errors, 33.3% even split | `board-worklog` §2.24 |
| **S2** | Does adding nodes scale linearly | **30 runs** | **112.9 / 229.0 / 338.4 inf/s**, speedup **3.00×**, eff 100%, 0 errors | [S2](S2_GRPC_BASELINE.md) |
| **S3** | What is each configuration's real ceiling | **45 runs** | ceiling **115.2 / 232.0 / 341.8**, 3N **2.97×** | [S3](S3_SATURATION.md) |
| **S3.5** | Where does the −30% loss come from | 3 conditions | bandwidth, total CPU and the server ruled out → narrowed to the **transport path** | [S3.5](S3_5_TRANSPORT_PROFILE.md) |
| **S3.5b** | Is CPU0 softirq concentration the cause | 6 runs | **−0.2% (null)** — though with a single flow, open to challenge | [S3.5](S3_5_TRANSPORT_PROFILE.md) §4.3 |
| **S3.6** | Flow control or connections | **20 runs** | enlarging the window **−36.3%**; connections 1→4 **+21.5%** | [S3.6](S3_6_H2_CHANNEL_AB.md) |
| **S3.7a** | How many connections is optimal (fixed c32) | **25 runs** | knee at c4, only the tail degrades past it | [S3.7](S3_7_CONNECTION_TUNING.md) |
| **S3.7b** | What is each configuration's **operating point** | **75 runs** | all three at **c12**. conn2 **dominates conn1 on both axes** | [S3.7](S3_7_CONNECTION_TUNING.md) §4 |
| **S3.7c** | Does RPS help at the operating point | **10 runs** | **−0.8% (null)**, unchanged even with CPU0 %soft 68→56 | [S3.7](S3_7_CONNECTION_TUNING.md) |
| **S3.8** | Does the optimization hurt scale-out | **36 runs** | **135.5 / 263.3 / 387.2**, 3N **2.86× (95.3%)**. Absolute +13.3% but eff 98.9→95.3% | [S3.8](S3_8_OPTIMIZED_SCALEOUT.md) |

| **S3.9a** | Where does 3N's 4.5% efficiency loss arise | **9 runs** | Server resources **all ruled out**. The loss **shows up in the tail** (p50 flat, p99 +36%). TCP retransmits 3.5× — though the micro-mechanism was not isolated | [S3.9a](S3_9A_SCALEOUT_PROFILE.md) |
| **S0-B** | Does the operating point hold under sustained load (active cooling) | **30 runs / 31 min** | **degradation 1.9%**, **zero** clock downgrades. short-run = sustained | [S0](S0_SUSTAINED_LOAD.md) |
| **S0-A** | What happens fanless | **30 runs / 32 min** | **degradation 11.3%**. CPU 2208→**816 MHz** (king), NPU pinned at 950. **The king is 2.4× slower and round-robin still sends it 1/3** | [S0](S0_SUSTAINED_LOAD.md) |

| **S0-C** 1st | Does a load-aware policy recover the thermal-heterogeneity loss | **15 runs** | The policies **collapse throughput by 55–58%**. Cause: **herding on stale heartbeat state** — a scheduler bug, found | [S0-C](S0_C_POLICY_AB.md) |
| **S0-C** 2nd | Re-measure after fixing that bug | **12 runs** | **RR 373.9 / LQ 380.9 / ECT 384.2.** Collapse gone, **p99 −37%**, node latency spread 1.33×→**1.00×** | [S0-C](S0_C_POLICY_AB.md) §8–11 |

| **S0-C** 3rd | Any policy regression under homogeneity (active cooling) | **12 runs** | No regression (LQ −0.0%, ECT −0.3%). Tail improves even when homogeneous. **Neither LQ nor ECT dominates** | [S0-C](S0_C_POLICY_AB.md) §12–15 |
| **S0-C** 4th | LQ vs ECT under strong heterogeneity (2.4×) | **1 run (aborted)** | **Gate missed at 1.10×.** Thermal conditions were identical (86.8 °C) — **what sets heterogeneity is not temperature but the spread in CPU downgrade** | [S0-C](S0_C_POLICY_AB.md) §17–19 |
| **S3.9b** | Do node-side syscalls/copies matter in the residual gap | **4 conditions** | **Not syscalls** — ~1% of transport cost (8% being generous). User time exceeds kernel time (9.37 vs 6.99 ms/req). CPU is 48.9% idle = **not a constraint**. → **S4 io_uring cancelled/shelved** | [S3.9b](S3_9B_NODE_RESIDUAL.md) |
| **S0-D** calibration | Can heterogeneity be produced deterministically | **12 runs** | **Yes.** Cap 2208→600 moves the spread **1.12×→3.93×**. **Cap 816 reproduces S0-A (2.4×) to within 6 ms** | [S0-D](S0_D_CAPACITY_HETERO.md) |
| **S0-D** policy | Does ECT gain as the spread widens | not run | Would test the design rationale for the ECT default directly | [S0-D](S0_D_CAPACITY_HETERO.md) §6 |

**Total measurement runs: 421** (418 bench + 3 profile conditions), **error rate 0**
throughout. Four discarded runs (harness-collision contamination,
`results/policy-ab-20260821-contaminated/`) are excluded.

> This number is **not maintained by hand** — `bash scripts/count-runs.sh` counts
> it. Two documents once each carried their own copy and diverged to 343 vs 420
> (2026-08-21).

### 1.1 Raw-data map

Which experiment every directory under `results/` belongs to. **This table exists
to prevent orphan data** — 87 runs were in fact sitting unclaimed by any document
when this was checked (found 2026-08-21).

| Directory | Runs | Experiment |
|---|---:|---|
| `scaling-20260820` | 3 | pilot measurement (`RESULTS.md` §2.5, **superseded**) |
| `baseline-20260820` | 30 | S2 |
| `saturation-20260820` | 45 | S3 |
| `transport-profile-20260820` | 3 conditions | S3.5 |
| `rps-ab-20260820` | 6 | S3.5b (RPS null) |
| `h2-channel-ab-20260820` | 20 | S3.6 |
| `connection-sweep-20260820` | 25 | S3.7a (fixed c32) |
| `concurrency-sweep-20260820` | 30 | **S3.7b** — conn 2·4 × c24–64 (overload region) |
| `concurrency-sweep-20260820-low` | 30 | **S3.7b** — conn 2·4 × c8–24 (operating region) |
| `concurrency-sweep-20260820-conn1` | 15 | **S3.7b** — conn 1 × c8–24 |
| `s37b-operating-point` | 45 | S3.7b operating-point determination |
| `scaleout-optimized-20260820` | 36 | S3.8 |
| `scaleout-optimized-20260820-1n-only` | 12 | **S3.8** — 1N re-measurement |
| `scaleout-profile-20260821` | 9 | S3.9a |
| `node-residual-20260821` | 4 conditions | S3.9b |
| `sustained-20260821-fan` | 30 | S0-B |
| `sustained-20260821-fanless` | 30 | S0-A |
| `policy-ab-20260821` | 15 | S0-C 1st (herding bug found) |
| `policy-ab-20260821b` | 12 | S0-C 2nd |
| `policy-ab-20260821fan` | 12 | S0-C 3rd |
| `policy-ab-20260821-contaminated` | 4 | S0-C 4th — **discarded** (harness collision) |
| `capacity-calib-20260821` | 12 | S0-D calibration |
| `accuracy` · `thermal-20260811-*` | — | model accuracy · pilot thermal measurement (`RESULTS.md`) |

> ⚠️ `results/policy-ab-20260821-contaminated/` — **concurrent harness
> collision; invalid for performance conclusions.** It is the S0-C 4th attempt;
> only the r1 round-robin run and the 1-second thermal log are usable. The
> incident is written up in §4.11 and [S0-D](S0_D_CAPACITY_HETERO.md) §4. Kept
> as a methodology record.

---

## 2. Bottleneck candidates — exclusions are conditional

**The value of this table is in shrinking the candidate space, not in having
identified what the remaining gap is.** And **a candidate once excluded reopens
when conditions change** — S3.8 is exactly that (§4.7).

| Candidate | Current verdict | Basis |
|---|---|---|
| Link bandwidth | **excluded** | node eth0 51% per direction (S3.5); server 10G **40% per direction** (S3.9a). The "76%" in S3.8 was a full-duplex arithmetic error and is **withdrawn** |
| Board CPU capacity | **excluded** | 8 cores **49–63% idle** (S3.5, S3.7c) |
| Server CPU · NIC · scheduler | **excluded (24-thread host only)** | CPU 42%, busiest core 47.6%, 0 drops, no thread serialization, syscalls/req unchanged (S3.9a). ⚠️ **Reopens on an 8-thread host** — see below |
| **Shared-path congestion (10G→2.5G)** | **new, leading (unverified)** | per-connection TCP retransmit rate **3.5×**, cwnd 176→106–119. Consistent with p50 flat and only the tail rising (S3.9a) |
| Kernel RX distribution (RPS) | **excluded** | throughput unchanged even after taking **12 pp off** CPU0 %soft (S3.7c) |
| HTTP/2 flow control | **counterproductive at the extreme** | **−36.3%** when enlarged to 64 MB. **Mid-range values (256 KB–4 MB) unmeasured** (S3.6) |
| **Connections per node** | **primary constraint** | 1→2 gives **+18.8%** throughput and **−18.8%** on the tail (S3.7b) |
| Remaining cost | **not separated** | protobuf / memcpy / syscall / H2 implementation / userspace scheduling / NPU submission → needs profiling |

> **"Server and scheduler excluded" held only under the baseline (conn1)
> condition.** Optimizing per-node transport raised the load reaching the shared
> path and that exclusion broke. An exclusion verdict has to carry **the
> conditions it was reached under**.

> **The same exclusion broke a second time — this time on hardware conditions
> (2026-08-26).** Swapping the scheduler host from 24 threads (Xeon E5-2630L ×2)
> to 8 threads (Core i7-4790) dropped the baseline from **391 to 360 inf/s
> (−7.5%)**. Server CPU during the measurement was **82.2%** (42% under the old
> host).
>
> The interesting part is that **the application queue is still empty** —
> `scheduler_queue` 0.00 ms · `scheduler_route` 0.01 ms. What S3.9a actually
> excluded was that queue, and that verdict still stands. What narrowed is
> **outside** it: the host's CPU. The measurement setup — **the bench client
> runs on the same host as the scheduler** — amplifies this.
>
> All 421 measurements were taken on the old server and **those values stand as
> recorded.** Reproduction figures on the new server are kept separately in
> `../infrastructure.md` §3.2.1 and `../environment-matrix.md` §10.2. We do not
> compare the two hosts' numbers directly.

---

## 3. Numeric lineage — how per-node throughput moved

The **single-connection** configuration converges on **113–117** across five
independent experiments. Different days, different harnesses, different purposes,
same value.

```text
S2   1N @c8   (30 runs)  112.9 ± 0.5
S3   1N ceiling @c32     115.2
S3.5 cluster  @c32       116.6
S3.6 A(1ch)   @c32       115.3 ± 0.8
S3.7a c1      @c32       115.6 ± 0.7
S3.7b conn1   @c12  ★    114.8 ± 0.7   <- conn1's operating point
─────────────────────────────────────
S3.7b conn2   @c12  ★★   136.4 ± 0.3   <- optimized operating point  (+18.8%)
S3.8  conn2   @c12       135.5 ± 0.4   <- reproduced independently, different harness
local direct (no network)  161.5
─────────────────────────────────────
residual gap at the operating point
                 161.5 − 135.5 = 26.0 inf/s = **16.1% of direct**
                 (against S3.7b's 136.4 it is 25.1 = 15.5%)
```

★ A fair comparison is between operating points found by the same rule (98% of
peak).

---

## 4. Methodology lessons — what will outlast the numbers

### 4.1 Optimize at the operating point, not in the overload region

S3.6 and S3.7a, at **fixed c32**, produced "more connections make the tail 46%
worse". The measurement was correct, but c32 was **overload for all three
configurations**. Re-measured at the operating point (c12), the same change
**improved the tail by 18.8%**. The sign of the conclusion flipped.

A fixed-load comparison can show **overload behaviour** rather than a
configuration effect. So the c32 results are not discarded — they are kept as a
separate result labelled "overload behaviour", just not used as grounds for
operating decisions.

### 4.2 Two measurements agreeing does not make the interpretation right

c32 (+28.2%) and c24 (+27.4%) agreed closely, and we wrote that up as "a
property of 4 connections as such". **Both were in the overload region — we had
seen the same bias twice.** Reproducibility only confirms the bias.

### 4.3 Fix the decision rule before the results, and do not move it to fit them

The operating-point definition is pinned as a constant in code.

> operating concurrency = the **lowest** concurrency delivering at least **98%**
> of peak
> (99% overlaps with the ±1 inf/s run-to-run SD)

In S3.7a, c2 came in at **96.4%** and missed the threshold by 0.6 pp. Lowering
the threshold to 96% would have produced the answer we wanted, and **we did not
lower it.** We recorded "the rule does not decide at this boundary" as the
result instead, and S3.7b settled it with data.

### 4.4 Turn silent failures loud

- **Make the harness stop when it fails.** When a node was built with the mock
  backend and failed to start, the harness died loudly and immediately, and it
  was caught on the spot.
- **Leave evidence that the configuration took effect.** Every run counts the
  actual TCP connections with `ss` and records it. A silently ignored setting
  turns an A/B into the same condition run four times.
- **Verify node count before measuring.** Process existence ≠ receiving traffic.
  S3.8 uses a probe bench to count the **distribution of responding node IDs**
  and skips the configuration when expected ≠ observed.
- **Do not delete the raw data.** A bug that cleared the output directory
  between runs destroyed an earlier run's JSON (throughput survived in the CSV,
  so the conclusion was unharmed). After that, staging directories were split
  out.

### 4.5 A candidate once excluded reopens when conditions change

S2 showed three nodes scaling 3.00× linearly, so **the server and scheduler were
excluded**. That verdict was right — **under conn1**.

Raising per-node connections to 2 increased load on the shared path, and
optimized 3N efficiency fell to **95.3%**. The server is a candidate again.

> **An exclusion verdict has to carry "under what conditions".** When conditions
> change, the exclusion table has to be re-read. Write it once and freeze it,
> and you will not see the new bottleneck your own optimization created.

### 4.6 A throttling verdict needs its conditions too

Same hardware, same operating point, same load — and **cooling alone changes the
conclusion.**

```text
active cooling   degradation  1.9%   0 clock downgrades   NPU 60 °C
fanless          degradation 11.3%   CPU 2208→816 MHz     NPU 88 °C
```

The worklog's "CPU −27% over 300 s" was a correct observation too — **under those
conditions.** S0-A saw the CPU fall to the same 816 MHz and the loss was −11.3%
(the cluster workload has CPU headroom). **Detach the conditions and the number
lies.**

This is the same story as the exclusion verdicts (§4.5). **"There is / is not
throttling" has to be written with its conditions.** Detached, the next person
plans on a false premise — which is exactly what happened here: that −27% put S0
ahead of io_uring this session (the right call, but the basis turned out to
belong to different conditions).

### 4.7 Separate "the policy is bad" from "the implementation is broken" first

S0-C saw load-aware policies drop throughput by **55–58%**. Stopping there with
"load-aware policies do not suit this workload" would have been the wrong
conclusion.

The per-stage breakdown separated it — `scheduler_route` 0.004 ms across all
three (decisions are fast), `node_queue` 0.023 ms (nodes are not backed up), yet
the round trip alone 2.8×. Node CPU was in fact halved (45% → 20%). **They are
not doing more work; they are waiting more.**

The cause was not the policies' judgement quality but **state freshness**. The
`queue_depth` the policies read was refreshed only by heartbeat and not updated
by the dispatch path, so hundreds of requests per second all read the same fixed
snapshot and all chose the same node.

> **When performance looks wrong, ask "is the implementation doing what it was
> meant to" before "this approach is bad".** 55% is not the size of a
> quality difference.

Re-measured after the fix, the policies behaved and **p99 improved by 37%**. Had
we concluded from the first result, the record would say the exact opposite:
"load-aware scheduling does not help".

One more thing — **a decision rule failing to fire is also a result.** In the
2nd round the shift in king's share was 0.5 pp, short of the 3 pp rule. We did
not lower the threshold; we wrote down why it did not fire (the thermal spread
was a weak 1.33×, and least-outstanding is a closed loop regulating concurrent
occupancy rather than counts).

### 4.8 State how percentiles were aggregated

In tables pooling several runs, p95/p99 are the **average of run-level
percentiles**, not pooled percentiles. Run-level averaging dilutes each run's
worst window and so **makes the tail read low**. Valid for comparing conditions;
not to be quoted as "this system's p99" →
[S2 §7.4.1](S2_GRPC_BASELINE.md)

### 4.9 A process without logs tells you nothing afterwards

The jack node died and **the cause could not be established.** It was neither OOM
nor segfault, and there were no logs at all — the `setsid nohup` in the startup
procedure was missing a redirect, so stdout was being thrown away. **Not knowing
why it died is worse than it dying.**

---

### 4.10 Your instrument may be measuring a different quantity

The policy A/B harness recorded SoC temperature every run. Those values read
**78–79 °C**, so "cooler than S0-A (86 °C)" went into several documents, and on
top of it a prescription: "we need continuous heating".

**Wrong.** The harness reads the value *after* the 60-second run finishes, over
three sequential ssh calls. RK3576 cools within seconds once load stops, so that
value is the trough between runs. Re-aggregated from the 1-second thermal
logger, it was **86.8 °C — the same as S0-A.** The thermal conditions had been
identical from the start; what differed was the spread in CPU downgrade.

Two instruments carried the same name (`soc`) and measured different quantities
(max during the run vs. an instantaneous value after it). **That the CSV column
was named `max_soc_c` made it worse** — it was not a maximum.

> Moving a decision threshold and fixing the instrument that measures it are
> different acts. The first fits the rule to the results (violating §4.3); the
> second is what keeping the rule requires. When fixing, **state in the document
> that the reference value is unchanged and only its source moved.**

### 4.11 Do not trust "I stopped it" — verify at the shared resource

A harness was stopped and another was started. In reality **the stop failed and
both were hitting the same three nodes at c36 each.** The baseline came out at
197 inf/s (391 is normal) and the next run had an 82% error rate. **We were one
step from misdiagnosing a cluster failure** — after cleanup, re-measurement gave
391.2 with 0 errors.

Because the surviving harness kept restarting the scheduler with its own
configuration, the "restore to default settings" step was being silently
overwritten seconds later.

**Local observation lied.** In git-bash it did not appear in `ps -ef` and
`pkill -f` could not catch it. Only PowerShell's `Get-CimInstance Win32_Process`
showed it. Process observation is not trustworthy across platforms.

So verification moved **to the shared resource.** `npuforge_assert_cluster_free`
will not start a harness if `npuforge-bench` is running on the server. A bench
running on the server does not lie. (This is §4.4 extended — turn silent
failures loud.)

### 4.12 Harness invariants — §4.4 and §4.11 hardened into rules

These came out of two incidents (§4.11, and the results-path overwrite). New
harnesses obey both.

1. **Verify shared-resource state at the shared resource.**
   Knowing locally that "I stopped it" is not enough. Whether the cluster is
   free is a question **for the cluster** (`npuforge_assert_cluster_free`).
2. **Do not treat the results path as an appendable/overwritable scratch
   directory.** `results/<experiment>-<date>` overwrites itself when run twice
   in a day. This did overwrite S0-C's 1st round (15 runs), which would have
   been lost had it not been under git. Stop if the existing directory is not
   empty.

### 4.13 The six times the instruments lied — the authoritative list

§4.10 and §4.11 cover two of them, but talks and public material cite "six
times". **If you are going to use the number, there has to be a list.** This is
that list.

**Scope: the cluster measurement campaign (2026-08-20 to 08-21).** The four
failures from the single-node era are kept separately in
[`../RESULTS.md`](../RESULTS.md) §6. The two are not counted together.

| # | What lied | How it surfaced | Basis |
|---:|---|---|---|
| 1 | **Post-run temperature sampling.** The `max_soc_c` column was not a maximum but the inter-run cooling trough — ~5 °C below actual | Compared against the 1-second thermal logger | [S0-C §17.5](S0_C_POLICY_AB.md) · §4.10 |
| 2 | **The explanation built on that value.** "The 2nd round only reached 1.33× because it was less hot" — the 2nd round was also 86.8 °C | Re-aggregated the 2nd round's thermal log | [S0-C §18.4](S0_C_POLICY_AB.md) |
| 3 | **13.2% was an overload-region figure.** That percentage came from 140.1 (c32) but got paired with the 135.5 operating point and spread through several documents. The real figure is 16.1% | Computed the denominator directly | §3 · commit `62855bd` |
| 4 | **Harness collision.** A harness believed stopped survived and hit the same cluster as the new one, both at c36. Baseline 197 inf/s (391 is normal), next run 82% error rate | Checked the server's process list | [S0-D §4](S0_D_CAPACITY_HETERO.md) · §4.11 |
| 5 | **Results-path overwrite.** Reusing the same dated path overwrote S0-C's 1st round (15 runs) with 4 lines | `git status` | [S0-D §4.2](S0_D_CAPACITY_HETERO.md) · §4.12 |
| 6 | **The `strace -c` parser read the wrong columns.** `usecs/call` and `calls` were swapped, making the call count come out 100× too small | Compared against expected values from `/proc/PID/io` | [S3.9b §8](S3_9B_NODE_RESIDUAL.md) |

**What the six have in common: every one of them looked like success.** A number
came out, it was plausible, nobody stopped. Four were caught **by comparison
against another measurement** (1, 2, 4, 6); two were caught because **a tool said
so loudly** (3, 5).

> **Three of the six (1, 2, 3) share one root** — an instrument was wrong, an
> explanation was built on it, and that explanation propagated into other
> documents. Instrumentation errors do not stay contained.

## 5. Current settled state

**The measurement lineage closed on 2026-08-21.** S2 through S3.9b and S0-D.

**The two lineages are not mixed.** The transport operating point and the
scheduling policy rest on different evidence.

```text
Transport operating point  -- settled ------------------------------
    2 connections/node @ concurrency 12/node

    1N   135.5 inf/s   p95 120.7 ms
    3N   387.2 inf/s   p95 151.1 ms   scaling 2.86x, eff 95.3%
    31 min sustained (active cooling)  380.3 inf/s  (-1.9%, 0 clock downgrades)

Adaptive policy  -- settled ----------------------------------------
    Default stays `ect`.

    RR is out of the running -- p99 SD 34.7 under heterogeneity
    (adaptive is ~1). Load-aware scheduling improves the tail markedly.

    LQ and ECT: **neither dominates.**
      fanless (heterogeneous)   LQ p99 146.9 / ECT 384.2 inf/s
      active cooling (homogeneous)  both fine. No regression (S0-C 3rd)
```

> **`node_connections` has two "defaults".** Easy to confuse, so it is written
> down.
>
> | | Value | What it is |
> |---|---:|---|
> | Library fallback | **1** | `SchedulerTransportConfig::default()`. For reproducing the baseline — give no configuration and you get the initial measurement condition |
> | Recommended operating value | **2** | `configs/scheduler.example.toml`. The operating point S3.7b established |
>
> Raising the code default to 2 would silently give a different condition to
> anyone trying to reproduce the old baseline. So **the fallback stays 1 and the
> example recommends 2.**

> The 0.9% throughput difference between ECT and LQ is **not used as grounds for
> preferring either.** ECT's basis is that it **absorbed the node latency spread
> to 1.00×** — that is, it reflected heterogeneous capacity as designed.

> **Mind the connection units** — `node_connections` is **per node**.
> 1N → 2 total, 2N → 4 total, 3N → 6 total.

Against the conn1 baseline (114.8, same rule): **+18.8% throughput, −18.8% p95**
— a strict Pareto improvement on the throughput and latency metrics measured. Of
the 46.7 gap to local direct (161.5), **21.6 (46%) was recovered by
configuration alone.**

---

## 6. How the lineage closed

| Step | Result |
|---|---|
| ~~S3.8~~ | Re-verified scale-out at the operating point. **+13.3%, efficiency 98.9→95.3%** |
| ~~S3.9a~~ | Server-side profile. **Server resources all excluded** — the loss shows up in the tail |
| ~~S0-A / S0-B~~ | 30 minutes of sustained load. Fanless −11.3% / active cooling −1.9% |
| ~~Policy validation on real hardware~~ | S0-C. Policies collapse 55–58% → **state-freshness defect (herding) found** |
| ~~Herding fix~~ | `local_in_flight` atomic reservation + RAII guard |
| ~~S0-C 3rd~~ | After the fix the policies adapt. No regression under homogeneity |
| ~~S0-D~~ | Deterministic heterogeneity fixture (clock caps). Produces heterogeneity without relying on heat |
| ~~S3.9b~~ | Remaining node-side cost. **io_uring's reachable share is ≈8%** |
| ~~S4 (io_uring)~~ | **Not adopted.** The measurement argued against it → `01-TECHSPEC.md` §15 |

### 6.1 The S0 result — the operating point is attached to the cooling condition

```text
short-run operating point                   3N 387-389 inf/s
sustained (active cooling)                  3N 380.3      (-1.9%)
sustained (fanless)                         3N 345.4      (-11.3%)
```

Under active cooling there were **zero** clock downgrades and a 58–61 °C plateau
— **the 60-second results from S2 through S3.9a apply unchanged to sustained
operation.**

Remove the fan and it splits. What gets downgraded is **the CPU, not the NPU**
(pinned at 950 vs 2208 → 816 MHz), and by a different amount on each board.

And **the fanless loss is not purely a thermal problem** — the king became 2.4×
slower while round-robin kept sending it 1/3 of the work (S0 §4.3). It is the
product of **thermal spread × a load-blind policy**. That is why the next
experiment became **policy validation** rather than io_uring.

### 6.2 S4's question changed, and the answer was "do not"

```text
at first   How much faster is io_uring than gRPC?
now        How far does a properly configured standard gRPC stack get, and what cost remains behind it?
```

S3.9b answered. Of the gap remaining at the operating point, **io_uring can
reach about 8%**. That is a small recovery against the implementation cost.
**We decided not to implement it and recorded the verdict** — `01-TECHSPEC.md`
§15.

---

## 7. Open — deliberately not closed

The measurement lineage is closed, but the following were not answered. **We do
not write down what we do not know as though we knew it.**

| Item | Status |
|---|---|
| **LQ vs ECT under strong heterogeneity (2.4×)** | Unmeasured. The basis for keeping `ect` as the default is a homogeneous sanity pass. **S0-D has made this question reproducible** |
| **Micro-mechanism of the 3N efficiency loss** | Confirmed as far as "it shows up in the tail" (p50 flat, p99 +36%). The shared-path congestion hypothesis (10G→2.5G) is **unverified** — needs switch counter access |
| **Short-window distribution** | Only 60-second aggregates exist. Needs `bench --dump-samples` |
| **Pooled percentiles** | Needs the same option. Current percentile figures are **run-level averages** |
| **Node exclusion behaviour** | Unverified — even fanless never reached the 90 °C threshold |
| **Grid resolution** | Whether the c12 operating point is the true knee or c10 is undetermined (grid step of 4) |
| **Mid-range H2 window** | 256 KB–4 MB unmeasured. Only the 64 KB ↔ 64 MB extremes were looked at |
| **c8/c16 connection ceilings** | Merely dropped from the S3.7b candidate set; not proven inferior |

> The entries in this table are **not things left unfinished but things decided
> against.** Each carries why it is open.
