# S0-D — Capacity Heterogeneity (deterministic)

*[한국어 원문](S0_D_CAPACITY_HETERO.ko.md)*

- Experiment ID: **S0-D**
- Started: 2026-08-21
- Status: **stage 1 calibration complete** (12 runs, 0 errors). Stage 2 policy A/B outstanding
- Predecessors: [`S0_C_POLICY_AB.md`](S0_C_POLICY_AB.md) §18–19 · [`S0_SUSTAINED_LOAD.md`](S0_SUSTAINED_LOAD.md)

---

## 1. Research Question

> **Does ECT gain over LQ as the capacity spread between nodes widens?**

S0-C closed the question as far as "adaptive beats RR when there is a spread"
(§11). What remains is **which of LQ and ECT should be the default**, and across
two conditions (heterogeneous 1.33× / homogeneous) neither dominated.

The design rationale for making ECT the default is that **it reflects service
rate in its score**. If so, **ECT should gain as the spread widens.** This
experiment tests that hypothesis directly. It is a stronger question than "which
is better at 2.4×".

## 2. Why manipulate the clock rather than heat

The S0-C 4th attempt tried to reproduce strong heterogeneity through continuous
fanless heating and failed (§18).

```text
all three experiments: soc 86.8 / 85.9 / 86.8 C - thermal conditions identical
what differed was only the board-to-board spread in CPU downgrade

  clock spread    1.14x -> 1.50x -> 1.79x
  latency spread  1.10x -> 1.33x -> 2.40x     (S0-C 4th / S0-C 2nd / S0-A)
```

**Thermal conditions are necessary for heterogeneity but not sufficient.** The
sufficient condition is the downgrade differing per board, and thermal control
targets temperature, not the spread. If all three boards come down together at
the same temperature no heterogeneity appears, and that divergence is a product
of silicon, airflow and position — **it cannot be summoned by cooling.**

So we take hold of **the handle thermal control itself uses.** The downgrade is
implemented by lowering `scaling_max_freq` (king was observed at `1008000`
during measurement). We use the same file.

| | Thermally induced (S0-A/C) | **Clock cap (S0-D)** |
|---|---|---|
| Reproducibility | depends on silicon luck | **deterministic** |
| Thermal disturbance | mixes into the policy comparison | **fan ON — removed as a variable** |
| Specifying the spread | not possible | **set to 1.3× / 1.8× / 2.4×** |

The cost: since it is not thermally induced, **it cannot be called thermal
heterogeneity.** It is capacity heterogeneity. The causal claim that the policies
work under thermally induced heterogeneity was already closed in S0-C's 2nd
round, and the present question is **how the policies respond to the size of the
spread**, so the substitution is legitimate.

## 3. Method — stage 1 calibration

`scripts/run-capacity-calibration.sh`

- **Fan ON**, boards idle at 42–47 °C. Thermal control has no reason to go below
  the cap.
- Policy **fixed at round-robin** — it does not adapt, so raw capacity spread
  shows through directly under an even load. S0-A's 2.4× uses the same
  definition.
- king's CPU cap on a ladder: **2208 / 1608 / 1200 / 1008 / 816 / 600 MHz**
  (on both `policy0` and `policy4`, each clamped to its group ceiling)
- Per cap: c36 · 60 s × 2. Three nodes, 2 connections/node — the operating point
  as-is.
- Each run reads `scaling_cur_freq` back to confirm **the cap holds under load**.
  An EXIT trap makes sure an interruption does not leave king downgraded.

**Why S0-A's clocks are not simply replicated**: the thermal logger records only
`cpu4`, so the little-core values are unknown. Rather than matching clocks, it
is more honest to **match the observed quantity (per-node p50 spread)
directly**. Calibration gives the cap → spread mapping.

## 4. Incident record — the first attempt was discarded to a harness collision (2026-08-21)

The first calibration attempt produced an **uncapped baseline of 197.4 inf/s**
(391.2 is normal). Node spread 2.73×, and the next run had an **82.4% error
rate**. It looked like "only king and jack are slow" and **we were one step from
misdiagnosing a cluster failure.**

The cause was not a failure. **The policy A/B harness had not died.**

```text
believed   the policy A/B harness was stopped via TaskStop
actually   only the wrapper shell died; the child bash kept running
result     two harnesses hitting the same 3 nodes at c36 each (72 combined)
```

The surviving harness kept **restarting the scheduler with its own
configuration** (`scheduler-s0c.toml`). So the "restore to default settings"
step was being overwritten seconds later, invisibly.

**Observation lied.** In git-bash it did not appear in `ps -ef` and `pkill -f`
could not catch it. Only PowerShell's `Get-CimInstance Win32_Process` showed it.

```powershell
Get-CimInstance Win32_Process -Filter "Name='bash.exe'" |
  ? { $_.CommandLine -match 'scripts/run-' } | select ProcessId,CommandLine
```

After cleanup and re-measurement: **391.2 inf/s · p50 86.2 · 0 errors · spread
1.02×.** The cluster had been fine all along.

### 4.1 Preventing recurrence

`npuforge_assert_cluster_free` (`scripts/lib/remote.sh`) was added and wired into
the start of the policy A/B and capacity calibration harnesses. **If
`npuforge-bench` is running on the server, it does not start — it stops loudly.**

The point is verifying **at the shared resource** rather than checking local
processes — local observation lies depending on the platform, but a bench
running on the server does not.

### 4.2 Contaminated data

- **First calibration attempt — discarded.** Re-measured under the same
  conditions (§5).
- **The 4th policy A/B in progress — renamed to
  `results/policy-ab-20260821-contaminated/` and kept.** It is not deleted
  because the incident itself is a methodology record (README §4.11). That
  directory's `README.md` states which parts are valid and which are not —
  **only the r1 round-robin run and the 1-second thermal log are valid** — and
  S0-C §18's gate verdict rests on just those two, so it is unaffected.

> ⚠️ The 4th harness **reused the same dated path and overwrote S0-C's 1st-round
> data (15 runs).** It was restored with `git checkout`. The harness output path
> is `results/policy-ab-<date>`, so running twice in a day overwrites — it must
> use `NPUFORGE_SUFFIX` or stop when the directory already exists.

## 5. Results — calibration (12 runs, error rate 0)

- Raw data: [`../../results/capacity-calib-20260821/`](../../results/capacity-calib-20260821/)
- Fan ON, boards at 48–55 °C. **Thermal control never intervened** — the
  `scaling_cur_freq` read back each run always matched the specified cap.

| king cap (MHz) | throughput | king p50 | jack p50 | queen p50 | **spread** |
|---:|---:|---:|---:|---:|---:|
| 2208 (uncapped) | 388.1 | 83.8 | 86.6 | 89.4 | **1.12×** |
| 1608 | 382.9 | 96.3 | 83.3 | 81.6 | **1.18×** |
| 1200 | 379.6 | 103.6 | 83.3 | 77.7 | **1.33×** |
| 1008 | 369.0 | 127.7 | 72.4 | 72.7 | **1.79×** |
| **816** | **359.6** | **149.8** | **67.9** | **66.3** | **2.26×** |
| 600 | 318.4 | 213.5 | 54.4 | 54.5 | **3.93×** |

Spread reproducibility: the 2 runs at each cap landed within ±0.05 of each other
(e.g. 816 → 2.30 / 2.22).

### 5.1 A cap of 816 reproduces S0-A almost exactly

| | king p50 | jack p50 | queen p50 | spread | throughput |
|---|---:|---:|---:|---:|---:|
| **S0-A** (thermal, fanless 86 °C) | 156.9 | 64.7 | 66.0 | **2.4×** | 345.4 |
| **cap 816** (clock, fan ON 50 °C) | 150.9 | 67.3 | 65.6 | **2.30×** | 359.8 |

All three node latencies **overlap within 6 ms.** It also fits the fact that
king's **CPU minimum in S0-A was 816 MHz** — we specified directly the floor the
thermal downgrade had pushed it to.

> **Strong heterogeneity can be produced deterministically.** The condition that
> clears S0-C §17.2's gate (2.0×) is now reproducible, with no 30-minute preheat
> and no silicon luck required.

### 5.2 Side observation — under RR a slow node idles the fast ones

As the cap comes down, **king gets slower while jack and queen actually get
faster.**

```text
king  cap 2208  83.8ms  ->  cap 600  213.5ms   (2.5x slower)
jack  cap 2208  86.6ms  ->  cap 600   54.4ms   (1.6x faster)
queen cap 2208  89.4ms  ->  cap 600   54.5ms   (1.6x faster)
```

At fixed c36 with RR, the client's 36 slots are split evenly across three nodes.
When king slows, **more slots are tied up waiting on king**, so fewer requests
are in flight on jack and queen and those two run underloaded. A p50 of 54 ms
means they are idling.

So the −18% throughput loss at cap 600 (388.1 → 318.4) is not king's capacity
loss alone; it includes **the share RR fails to use from the two idling nodes.**
The ceiling on what an adaptive policy can recover is right there. It is the same
phenomenon S0-A observed — "king is 2.4× slower and requests are still exactly
1/3" — except that this time **the spread can be specified and its size dialled.**

## 6. Stage 2 policy A/B — **future work (not being done now)**

Calibration produced the mapping, so it can be run at any time. But **it is not
the priority right now** — which of ECT and LQ wins does not change NPUDure's
central conclusion (§7). The main line is S3.9b.

The design when it is run:

```bash
# caps 1200 / 1008 / 816 / 600  =  spreads 1.33 / 1.79 / 2.26 / 3.93x
# 3 policies x 4 spreads x 3 runs; fan ON so no preheat - about 40 minutes
```

- The decision bands are taken unchanged from S0-C §17.3 (throughput 2%, p99 5%).
- Hypothesis: **ECT's throughput advantage grows as the spread widens.** If it
  does not, ECT's design rationale (reflecting service rate) is refuted by
  measurement.
- Because the spread is treated as **a continuous variable**, the conclusion is
  stronger than "which one at 2.4×" — it asks whether the advantage increases
  monotonically with the spread.

## 7. Where this lineage currently stands

Combining the calibration result with the policy lineage as a whole:

1. **RR is vulnerable to heterogeneity.** It keeps sending 1/3 to a slow node,
   and under heterogeneity even the predictability of the tail collapses
   (p99 SD 34.7 vs ~1).
2. **Fresh-state adaptive scheduling improves RR's tail markedly.** p99 −37%,
   node latency spread 1.33× → 1.00× (S0-C §9).
3. **LQ and ECT both work.** No regression under either condition.
4. **Whether ECT wins under strong heterogeneity is undetermined.**
5. **But that outcome does not change NPUDure's central conclusion.** The core
   is "load-aware scheduling with state freshness fixed absorbs heterogeneity",
   and that holds with either LQ or ECT. The default stays `ect`.
6. **What S0-D leaves behind is not an answer but a fixture** — apparatus for
   testing that question **reproducibly**, whenever.

---

## Figure

![Cap to spread mapping. 816 MHz reproduces S0-A (2.4x)](../../results/capacity-calib-20260821/figures/fig_capacity_calibration.png)

**`fig_capacity_calibration.png`** — cap → spread mapping; 816 MHz reproduces
S0-A (2.4×)

Regenerate: `python scripts/make-experiment-figures.py`
