# S0 — Sustained Load (condition A fanless / condition B active cooling)

*[한국어 원문](S0_SUSTAINED_LOAD.ko.md)*

- Experiment ID: **S0-A · S0-B**
- Measured: 2026-08-21
- Code: `bb3f7ab` + `[transport] node_connections = 2`
- Status: **both complete** (30 runs × 60 s each ≈ 31 minutes continuous)
- Raw data: [`../../results/sustained-20260821-fan/`](../../results/sustained-20260821-fan/) ·
  [`../../results/sustained-20260821-fanless/`](../../results/sustained-20260821-fanless/)
- Predecessor: [`S3_8_OPTIMIZED_SCALEOUT.md`](S3_8_OPTIMIZED_SCALEOUT.md)

---

## 1. Research Question

> **Does the short-run operating point hold under sustained load? And how much
> does that answer depend on the cooling condition?**

**Every** measurement so far has been 60 seconds or less — the region before
throttling appears.

```text
short-run operating point    based on benchmarks of 60 s or less
sustained operating point    based on thermal steady state
```

## 2. Method

- The operating point as-is: **3 nodes, 2 connections per node, c36** (= c12 per
  node).
- **60-second runs × 30 consecutively**, with **no restart** of nodes or
  scheduler.
- `thermal-logger.sh` on all three boards at **1-second intervals** — four
  temperatures, CPU MHz, NPU MHz, voltage.
- Each run records **the number of responding nodes and peak NPU temperature**.
  If a node is excluded fanless by hitting a threshold (degraded 80 / disable
  90 °C), the throughput drop is a **reduction in node count**, not throttling.
  The two have to be distinguished.
- The decision rule was fixed **before measuring**: `steady = mean of the last
  third`, `degradation = 1 − steady/peak`. <3% none / 3–10% slight / >10%
  pronounced.
- The two conditions **start from similar idle temperatures** (fan 40.7–41.6 °C,
  fanless 38.8–40.7 °C). At idle the fan does little, so this is a fair A/B with
  matched starting points.

## 3. Results

Error rate **0 throughout in both**. **Zero node exclusions** (even fanless
never reached the 90 °C threshold).

| | **B: active cooling** | **A: fanless** |
|---|---:|---:|
| peak | 387.7 | 389.4 |
| **steady (last third)** | **380.3 ± 2.2** | **345.4 ± 3.8** |
| **degradation** | **1.9%** | **11.3%** |
| soc max | 58.2 – 61.0 °C | **85.9 – 86.8 °C** |
| npu max | 59.2 – 61.0 °C | **86.8 – 87.8 °C** |
| **CPU minimum** | **2208 MHz (0 downgrades)** | **816 / 1200 / 1416 MHz** |
| NPU minimum | 950 MHz | **950 MHz (no downgrade)** |
| Node exclusions | 0 | 0 |

Over time:

| t+min | B throughput | A throughput | A vs peak |
|---:|---:|---:|---:|
| 1 | 387.7 | 389.4 | 100.0% |
| 5 | 385.8 | 380.9 | 97.8% |
| 10 | 382.5 | 359.7 | 92.4% |
| 15 | 381.7 | 356.2 | 91.5% |
| 20 | 380.2 | 355.8 | 91.4% |
| 25 | 382.3 | 342.1 | 87.9% |
| 30 | 377.3 | 343.5 | 88.2% |

## 4. Interpretation

### 4.1 Cooling was holding the operating point up

Under active cooling: **1.9%** — "no degradation" by the decision rule. Clock
downgrades number **zero** across all ~1,660 samples per board. Temperature
reaches a 58–61 °C plateau within five minutes, with over 20 °C of headroom to
the threshold.

**Remove the fan and it is 11.3%.** The two operating points diverge.

```text
short-run operating point                       3N 387-389 inf/s
sustained operating point (active cooling)      3N 380.3      (-1.9%)
sustained operating point (fanless)             3N 345.4      (-11.3%)
```

> **"There is / is not throttling" has to be written with its conditions.**
> Same hardware, same operating point, same load — and cooling alone changes the
> conclusion.

### 4.2 The NPU was never downgraded — what was downgraded is the CPU

**The NPU stayed pinned at 950 MHz in both conditions.** Fanless, even with NPU
temperature reaching 87.8 °C, the clock did not drop.

What was downgraded is the CPU. And **it differs per board.**

| Board | CPU minimum | soc max |
|---|---:|---:|
| **king** | **816 MHz** (−63%) | 86.8 °C |
| jack | 1200 MHz (−46%) | 86.8 °C |
| queen | 1416 MHz (−36%) | 85.9 °C |

This is precisely the point the worklog recorded as its fourth mistake —
"judging throttling by NPU clock alone" (discuss §3.1). **This measurement
re-confirms that lesson.**

### 4.3 The real finding — round-robin keeps hitting the downgraded node

Per-node latency over the last 5 fanless runs:

| | p50 | p95 | **share** |
|---|---:|---:|---:|
| jack | 64.7 | 107.0 | **33.3%** |
| **king** | **156.9** | **313.9** | **33.3%** |
| queen | 66.0 | 107.4 | **33.3%** |

**king is 2.4× slower than the other two and still receives exactly one third of
the requests.** Round-robin looks at neither load nor state.

Under active cooling the three nodes sit evenly at 85.2–90.3 ms. They diverge
only when fanless.

And queen and jack actually got **faster** fanless (85–90 → 65–66 ms), because
total throughput fell and with it the per-node load.

> The large latency drop is **a strong signal that queue pressure fell**, but
> concluding "they are idling" would need per-node **CPU idle or outstanding
> queue depth**. This measurement has neither — S0-C records them alongside.

> ⚠️ **Everything to this point is observation; what follows is hypothesis.**
>
> **Confirmed**
> - There is a thermal spread (CPU 816 / 1200 / 1416 MHz)
> - king's service capacity really is lower (p50 2.4×)
> - RR keeps sending 33.3% to the slowed king
>
> **Not yet**
> - "fanless loss = thermal spread × load-blind policy" — the final causal link
>   closes only when changing the policy **actually recovers** the loss.
>
> `least-queue` and `ect` are implemented in the repository but have no
> validation on real hardware. **S0-C closes this link** (§8).
>
> A negative result matters too — if the split stays at 1/3 or performance is
> unchanged with the policies on, that means **the current policies' state
> signal does not detect thermal-induced capacity degradation.**

### 4.4 Relation to the original −27%

| | Original (discuss §12) | This S0-A |
|---|---|---|
| Load | local, 8 threads (CPU saturated) | cluster (CPU headroom) |
| Cooling | fanless | fanless |
| NPU temperature | 90.4 °C | 87.8 °C |
| CPU downgrade | 2208 → **816 MHz** | 2208 → **816 MHz** (king) |
| Result | **−27%** | **−11.3%** |

**The CPU fell to the same 816 MHz.** Yet the loss is less than half. Cluster
operation leaves the board CPU 49–63% idle (S3.5, S3.7c) so it is less affected
by the downgrade, and only one of the three boards fell to the worst case.

→ **−27% was not wrong. The conditions were different.**

## 5. Limitations

- **Load-aware policies not measured** (§4.3). Only round-robin was used. How
  much of the fanless loss `least-queue`/`ect` recover **is a hypothesis, and
  the thing to be tested**.
- This is 31 minutes. Temperature reached a plateau so longer runs are unlikely
  to differ much, but that is an **estimate**.
- Room temperature was not controlled. The two conditions were measured
  back-to-back on the same day.
- 2–4 second gaps between runs (§2).
- One 3-node operating point only. 1N and 2N were not measured.
- Even fanless never reached the 90 °C threshold, so **node-exclusion behaviour
  is unverified.**

## 6. Reproduction

```bash
bash scripts/run-sustained-load.sh 30 fan       # condition B
bash scripts/run-sustained-load.sh 30 fanless   # condition A (fan removed)
PYTHONIOENCODING=utf-8 python scripts/analyze-sustained.py \
    results/sustained-20260821-fanless
```

## 7. Conclusion

**Under active cooling the short-run operating point holds under sustained load**
(degradation **1.9%**, 0 clock downgrades). The 60-second results from S2 through
S3.9a apply unchanged to continuous operation.

**Remove the fan and it widens to 11.3%.** What is downgraded is not the NPU but
the **CPU** (pinned at 950 MHz vs 2208 → 816 MHz), and by different amounts per
board.

The most valuable finding is §4.3 — **king became 2.4× slower and round-robin
still sends it one third.** To RR the three nodes are identical; their actual
service capacity already is not.

**This is the proving ground for adaptive scheduling.** Validating load-aware
policies on real hardware is hereby **promoted from a functional item to a
performance item** → **S0-C**.

That said, "loss = thermal spread × policy" is **still a hypothesis** (see the
note in §4.3). Causality closes only once changing the policy is seen to recover
the loss.

## 8. Next — S0-C (do it before turning the fan back on)

**Under active cooling the three nodes are nearly homogeneous, so policy
differences are likely to vanish.** The current fanless state is the best
condition for validating the policies. Close the causal check before cooling
down.

| Policy | Throughput | p95 | p99 | king share | jack share | queen share |
|---|---:|---:|---:|---:|---:|---:|
| round-robin | 345.4 | ? | ? | 33.3% | 33.3% | 33.3% |
| least-queue | ? | ? | ? | ? | ? | ? |
| ect | ? | ? | ? | ? | ? | ? |

What we want to see is not simply higher throughput. If, for example, ECT shifts
to something like `king 15% / jack 42% / queen 43%` while moving 345 towards
370–380, then this can be said:

> **Thermal heterogeneity reduces node capacity, and state-aware scheduling
> recovers performance by adapting load allocation to heterogeneous service
> rates.**

Design constraints to respect:

- **Heat thoroughly into thermal steady state first**, then compare. If the
  starting temperature differs per policy, the policy effect and thermal drift
  get mixed together.
- Rotate the policy order.
- Beyond throughput, p95 and p99, record **per-node distribution and per-node
  latency**, plus **per-node CPU idle**.

---

## Figure

![31 minutes continuous - active cooling -1.9% vs fanless -11.3%](../../results/sustained-20260821-fanless/figures/fig_sustained_thermal.png)

**`fig_sustained_thermal.png`** — 31 minutes continuous; active cooling −1.9%
vs fanless −11.3%

Regenerate: `python scripts/make-experiment-figures.py`
