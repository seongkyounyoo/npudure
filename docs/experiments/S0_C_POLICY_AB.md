# S0-C — Scheduling Policy A/B (fanless)

*[한국어 원문](S0_C_POLICY_AB.ko.md)*

- Experiment ID: **S0-C**
- Measured: 2026-08-21
- Code: `7281411` + `[transport] node_connections = 2`
- Status: **1st (15 runs, bug found) · 2nd fanless (12 runs) · 3rd active cooling (12 runs)** complete
- Raw data: [`../../results/policy-ab-20260821/`](../../results/policy-ab-20260821/) (1st) ·
  [`../../results/policy-ab-20260821b/`](../../results/policy-ab-20260821b/) (2nd, fanless) ·
  [`../../results/policy-ab-20260821fan/`](../../results/policy-ab-20260821fan/) (3rd, active cooling)
- Predecessor: [`S0_SUSTAINED_LOAD.md`](S0_SUSTAINED_LOAD.md)

---

## 1. Research Question (as originally intended)

> S0-A saw king become 2.4× slower fanless while RR kept sending it one third.
> **Do load-aware policies (`least-queue`/`ect`) recover that loss?**

## 2. 1st Results — the policies collapse

Fanless, after 15 minutes of preheat, 3 nodes / 2 connections per node / c36,
5 runs per policy. Error rate **0 throughout**.

| Policy | Throughput | p50 | p95 | p99 | king% | jack% | queen% |
|---|---:|---:|---:|---:|---:|---:|---:|
| **round-robin** | **379.9 ± 13.5** | 85.5 | 165.4 | 213.8 | 33.3 | 33.3 | 33.3 |
| **least-queue** | **169.8 ± 2.3** | 199.8 | 390.0 | 477.1 | 35.1 | 33.3 | 31.6 |
| **ect** | **158.5 ± 2.1** | 219.6 | 397.9 | 483.0 | 34.8 | 36.3 | 28.9 |

**The load-aware policies drop throughput by 55–58%.** That is the opposite of
the expected direction, and it is not "a bit worse" but **less than half.**

Per-node CPU busy% (`/proc/stat` deltas):

| Policy | king | jack | queen |
|---|---:|---:|---:|
| round-robin | 51.5% | 44.5% | 45.3% |
| least-queue | 22.6% | 21.1% | 20.7% |
| ect | 21.0% | 21.7% | 17.7% |

**The nodes are actually idling.** Throughput is halved and so is CPU usage.

## 3. Cause — herding on stale heartbeat state

The stage breakdown narrows the culprit (p50, ms):

| Policy | scheduler_route | scheduler_queue | node_queue | inference | **end_to_end** |
|---|---:|---:|---:|---:|---:|
| round-robin | **0.004** | 0.000 | 0.023 | 30.27 | **75.8** |
| least-queue | **0.003** | 0.000 | 0.023 | 33.69 | **192.4** |
| ect | **0.004** | 0.000 | 0.023 | 33.97 | **212.5** |

- **Policy selection is not slow** — route is 0.004 ms in all three.
- **Nor is it the node queue** — node_queue is an identical 0.023 ms.
- **Nor inference** — 30–34 ms, comparable.
- What grew is **the whole round trip** (75.8 → 212.5).

So requests are waiting somewhere **before reaching the node**.

### 3.1 The state the policies read is refreshed only by heartbeat

```rust
// registry.rs:110-121  - the snapshot handed to the policy
queue_depth: self.health.queue_depth,
in_flight:   self.health.in_flight,
```

`self.health` is replaced wholesale **only when a heartbeat arrives**
(`on_health_success`: `self.health = health;`).

And **the dispatch path does not update this state at all** — there is **not a
single** reference to `in_flight` or `queue_depth` anywhere in `service.rs`.

The heartbeat period is **1000 ms** at the node, with the scheduler expecting
2000 ms.

### 3.2 So decisions pile up deterministically

```text
RR-level throughput 380 inf/s
about 380 requests are dispatched between two heartbeats (1 second)
all 380 see the same (fixed) snapshot

LeastQueue.choose() = min_by(queue_depth, in_flight, ewma_inference)
with a fixed snapshot this function is deterministic
  -> all 380 pick the same node
  -> that node's queue_depth spikes at the next heartbeat
  -> the following second goes entirely to a different node
```

**Textbook herd behaviour under stale load information.** With only 2 connections
per node, the hundreds of requests piled onto one node back up behind those 2
connections — which is why **the node idles (CPU 20%) while only the round trip
grows (212 ms).** It is also why node_queue is 0: the requests are stacked in the
**transport layer**, not the worker pool.

The aggregate split looking like ~33% is **the average of alternating pile-ups
over 60 seconds**, not an even distribution.

Round-robin does not read state and distributes structurally, so it does not have
this problem.

## 4. Verdict

> **`least-queue` and `ect` are not in a usable state under this load.**
> This is not a policy-quality problem but a **state-freshness design defect.**
> Routing hundreds of requests per second on state refreshed once per second
> produces herding.

This is **not the answer to the question S0-C intended.** Whether the policies
recover the thermal-heterogeneity loss remains **undetermined** — the
implementation was not in a state to test it.

## 5. ⚠️ Two defects in this experiment

### 5.1 The thermal condition was not maintained (fatal)

The RR results differ substantially by round.

| round | RR throughput | king p50 |
|---:|---:|---:|
| 1 | **355.7** | **144.7** ← right after preheat, hot |
| 2 | 385.1 | 88.7 |
| 3 | 387.2 | 89.7 |
| 4 | 386.1 | 86.5 |
| 5 | 385.2 | 89.5 |

**The boards cooled down while the low-load policies (LQ/ECT) ran.** With CPU
busy falling from 45–51% to 17–22%, heat output dropped and the following RR
runs executed cool. From round 2 on, king p50 normalised from 144.7 to 87 —
**the thermal heterogeneity S0-A created had disappeared.**

So this experiment stopped being "a policy comparison under fanless thermal
heterogeneity".

> **Fortunately the policy collapse itself is unrelated to temperature.** LQ/ECT
> held steady at 166–172 / 155–160 across all 5 rounds (hot or cool). Only RR
> tracked temperature, moving 355 → 385. §3's conclusion is unaffected by this
> defect.

### 5.2 Temperature collection failed

The `max_soc_c` column is entirely empty. The awk program inside ssh was wrapped
in **double quotes**, so the remote shell consumed `$1` as a positional argument.

```bash
# wrong - the remote shell substitutes $1 with an empty string
ssh "$h" 'awk "{print $1/1000}" /sys/.../temp'
# right - the awk program goes in single quotes
ssh "$h" "awk '{print \$1/1000}' /sys/.../temp"
```

`thermal-logger.sh` uses single quotes and was fine. Rewriting it inline stepped
into the same trap.

## 6. Next

1. **Fix policy state freshness** — incrementing and decrementing `in_flight` at
   dispatch time (the scheduler knows how many requests it sent) keeps the state
   current without heartbeats. A few dozen lines.
2. **Re-run S0-C after the fix** — only then can the original question be
   answered.
3. To avoid §5.1 on re-run, **the thermal condition has to be re-matched between
   policies** (reheat with RR before each policy, or a separate session per
   policy).

## 7. Conclusion

**Turning on load-aware policies collapsed throughput by 55–58%.** The cause is
not policy quality but **state freshness** — the `queue_depth`/`in_flight` the
policies read is refreshed only by heartbeat (1 s) and never updated by the
dispatch path, so hundreds of requests per second read the same fixed snapshot
and **all choose the same node** (herding).

The nodes idle (CPU 20%) while requests stack in the transport layer
(node_queue 0, round trip 212 ms).

**The original question — do load-aware policies recover the thermal
heterogeneity loss — remains undetermined.** The implementation was not in a
state to test it, and the thermal condition vanished mid-experiment (§5.1). But
**finding this defect first matters more** — without the fix we would have been
one step from the opposite conclusion: "load-aware scheduling does not help".

---

# 2nd (after the fix) — the policies actually adapt

The fix is commit `ece4eba` — `local_in_flight` atomic reservation, a
`Reservation` RAII guard, a `select_and_reserve()` critical section, and
replacing the policies' primary signal. 98 tests pass.

## 8. Changes to method

The 1st round's fatal defect (§5.1) is fixed. **Each policy is preceded by 3
reheat runs of RR** to match the **starting** thermal state.

> What is matched is the **starting** condition. Temperature diverging during a
> policy run is left alone — if an adaptive scheduler reduces king's load and
> king cools as a result, that is itself part of the policy's effect.

It was in fact controlled — every run started at soc **81–82 °C**, with a
within-run maximum of 78.5–79.5 °C.

## 9. Results (12 runs, 4 per policy)

Error rate **0 throughout**.

| Policy | Throughput | p50 | **p95** | **p99** | king% | jack% | queen% |
|---|---:|---:|---:|---:|---:|---:|---:|
| round-robin | 373.9 ± 4.5 | 87.1 | 170.7 | 232.0 | 33.3 | 33.3 | 33.3 |
| least-queue | 380.9 ± 2.1 | 92.2 | **127.1** | **146.9** | 32.9 | 33.3 | 33.7 |
| **ect** | **384.2 ± 0.8** | 90.6 | 130.9 | 156.4 | 32.8 | 33.3 | 33.9 |

**The collapse is gone.** 169.8 / 158.5 → **380.9 / 384.2**.

### 9.1 The tail improves markedly

| | RR | least-queue | ect |
|---|---:|---:|---:|
| p95 | 170.7 | **127.1** (−25.5%) | 130.9 (−23.3%) |
| p99 | 232.0 | **146.9** (−36.7%) | 156.4 (−32.6%) |

### 9.2 Per-node latency levels out

| Policy | king | jack | queen | max/min |
|---|---:|---:|---:|---:|
| round-robin | 103.2 | 85.4 | 77.6 | **1.33×** |
| least-queue | 93.3 | 92.0 | 91.1 | **1.02×** |
| **ect** | 90.4 | 90.7 | 90.5 | **1.00×** |

### 9.3 CPU utilisation levels out too

| Policy | king | jack | queen | spread |
|---|---:|---:|---:|---:|
| round-robin | 53.9% | 47.4% | 43.6% | **10.3 pp** |
| least-queue | 54.4% | 52.5% | 50.4% | 4.0 pp |
| ect | 53.3% | 51.9% | 50.2% | 3.1 pp |

This confirms "idling" through utilisation rather than latency. Under RR, queen
idles at 43.6% while king alone is pushed to 53.9%. With a policy on, the three
converge at 50–54%.

## 10. Verdict — the rule did not fire, and that is recorded as such

The rule fixed before measuring: **a shift of ≥3 pp in king's share counts as a
"shift"**.

```text
least-queue  king 33.3% -> 32.9%   (-0.4 pp)   below the rule
ect          king 33.3% -> 32.8%   (-0.5 pp)   below the rule
```

**The threshold is not lowered.** Instead, look at why it did not fire.

### 10.1 The distribution barely moved while latency and CPU levelled out

These are not contradictory. **Least-outstanding is not a policy that moves
counts but a closed loop regulating concurrent occupancy.** Reducing the number
of requests held **concurrently** on a slow node reduces that node's queue wait
and brings its latency down. Over 60 seconds the cumulative **count** processed
comes back to similar levels as latency levels out.

That is, under this condition **levelling was achieved with only a 0.5 pp
shift.**

### 10.2 This round's thermal heterogeneity was far weaker than S0-A's

| | S0-A | S0-C 2nd |
|---|---|---|
| soc max | 86–88 °C | **78–79 °C** |
| RR per-node p50 | 156.9 / 64.7 / 66.0 | 103.2 / 85.4 / 77.6 |
| spread | **2.4×** | **1.33×** |

The 3 pp threshold was set assuming S0-A's **2.4× spread**. Under a 1.33×
condition the required shift is itself small.

The cause is the harness — scheduler restarts and a probe bench between policies
made the average load lower than S0-A's (30 minutes continuous).

> ⚠️ **Correction (4th round, §18.4).** The "78–79 °C" in the table above is an
> **instrument error**. It reads an instantaneous value after the run ends and
> falls into the inter-run cooling trough; re-aggregated from the 1-second
> thermal logger, **the 2nd round was also 86.8 °C, the same as S0-A.** So this
> paragraph's causal claim — "it was less hot because average load was lower" —
> does not hold. The actual reason the 2nd round stopped at 1.33× is **that the
> CPU downgrade diverged less** (clock spread 1.50× vs S0-A's 1.79×). The
> temperature was identical from the start.

## 11. Conclusion (2nd)

**With state freshness fixed, the load-aware policies work.** The 55–58% collapse
is gone and throughput is +1.9% (LQ) / **+2.7% (ECT)** against RR.

**The biggest gain is the tail** — p99 **−37%** (232.0 → 146.9). Per-node latency
spread goes 1.33× → **1.00×** and CPU utilisation spread 10.3 pp → 3.1 pp.
**The policies really are absorbing the thermal heterogeneity.**

ECT is slightly ahead of LQ on throughput (384.2 vs 380.9) and levels latency
completely (1.00× vs 1.02×). That matches the design of reflecting service rate
in the score. But the difference is a small 0.9%, so **it is too early to declare
ECT superior under this condition.**

### What remains

- **How much is recovered under strong thermal heterogeneity (2.4×) is still
  unmeasured.** This round was 1.33×. Reproducing S0-A's level requires
  continuous heating with no low-load interval between policies.
- The distribution shift was small, so the 3 pp rule did not fire. The rule
  stays; the next experiment needs to look at **short-window (1 s) distribution**
  to see the instantaneous shift (the bench does not yet support a per-request
  dump).

---

# 3rd (active cooling, homogeneous) — a sanity test for the default

## 12. Why it was needed

The 2nd round is a **fanless (thermally heterogeneous)** condition. What was
proved there is precisely "when node performance is heterogeneous, adaptive
scheduling using fresh state beats RR" — not that it is **always best on a normal
homogeneous cluster.**

Under active cooling the three boards run at nearly the same speed. Confirming
that the adaptive policies produce **no regression** under that condition is what
lets a default be chosen.

3N / 2 connections per node / c36 / **active cooling**, 4 runs per policy. soc
47–54 °C.

## 13. Results — no regression, and the tail actually improves

| Policy | Throughput | p50 | **p95** | **p99** | king / jack / queen p50 |
|---|---:|---:|---:|---:|---|
| round-robin | 389.9 ± 1.6 | 86.3 | 146.1 ± 1.8 | 185.6 ± 5.4 | 86.3 / 86.1 / 87.0 (1.01×) |
| **least-queue** | **389.9 ± 2.0** | 89.1 | **129.3 ± 0.9** | **151.0 ± 1.5** | 89.2 / 89.1 / 89.1 (1.00×) |
| ect | 388.6 ± 1.4 | 88.5 | 136.3 ± 0.4 | 163.2 ± 1.9 | 88.2 / 88.5 / 88.8 (1.01×) |

CPU busy is an identical 45% for all three policies. The distribution does not
move either, at 33.2–33.5% — **not moving is correct under a homogeneous
condition.**

- **No throughput regression.** LQ −0.0%, ECT −0.3%, within the decision band
  (±2%).
- **The tail improves even when homogeneous.** p99 185.6 → **151.0** (LQ,
  −18.6%) / 163.2 (ECT, −12.1%).
- p50 rises slightly (86.3 → 89.1 / 88.5, +3%). It trades a little median for a
  lot of tail.

## 14. Placing the two conditions side by side

| | Fanless (heterogeneous) | Active cooling (homogeneous) |
|---|---|---|
| **RR** | 373.9±4.5 · p95 170.7**±19.9** · p99 232.0**±34.7** | 389.9±1.6 · p95 146.1±1.8 · p99 185.6±5.4 |
| **LQ** | 380.9±2.1 · p95 **127.1±0.5** · p99 **146.9±1.0** | 389.9±2.0 · p95 **129.3±0.9** · p99 **151.0±1.5** |
| **ECT** | **384.2±0.8** · p95 130.9±0.0 · p99 156.4±0.5 | 388.6±1.4 · p95 136.3±0.4 · p99 163.2±1.9 |

Three things are visible.

**① RR's tail becomes unstable under heterogeneity.** p95 SD **19.9**, p99 SD
**34.7** — against ~1 for the adaptive policies. The adaptive policies' gain is
not only "a lower tail" but **"a predictable tail"**.

**② LQ has the lowest tail under both conditions.** With SDs of 0.4–1.9 the
difference is real. Against ECT, p99 is −6.1% fanless and −7.5% cooled.

**③ ECT's throughput advantage appears only under heterogeneity.** +0.9%
fanless (384.2 vs 380.9), inverting to −0.3% when cooled.

## 15. Choosing the default

**RR drops out of contention.** It has the worst tail under both conditions, and
under heterogeneity even its predictability collapses.

**Neither LQ nor ECT dominates.**

| | LQ | ECT |
|---|---|---|
| tail (both conditions) | **lower** (p99 −6 to −8%) | |
| throughput (heterogeneous) | | **+0.9%** |
| throughput (homogeneous) | **+0.3%** | |
| design rationale | count-based | **reflects service rate** — potentially favourable as heterogeneity worsens |

> **The repository's current default is `ect`** (`policy.rs` `default()`, pinned
> by a test). Since both policies are regression-free and better than RR, **there
> is sufficient basis to keep the default.** Switching to LQ is a judgement about
> "whether to trade 6–8% of p99 for service-rate awareness", and that decision
> can wait until it is re-measured under strong heterogeneity (2.4×).

## 16. Limitations

- There is still no LQ vs ECT comparison under strong heterogeneity (S0-A's
  2.4×). The 2nd round was 1.33×.
- Three policies × 4 runs each. The p50/p95 differences are trustworthy given the
  small SDs, but the 0.3–0.9% throughput differences are **too weak to use as
  grounds for preferring one.**
- Short-window distribution was not observed (60-second aggregates only).

---

# 4th (strong heterogeneity) — deciding the LQ vs ECT default

## 17. Pre-registration — decision rules (written before the results arrive)

> This section was written **while the measurement was running**, at a point when
> `results/policy-ab-*` was still empty. It is a device for not moving the rules
> to fit the results (the same stance as when §10's 3 pp rule did not fire).

### 17.1 Conditions

| | Value |
|---|---|
| Cooling | fanless (physically off immediately before measuring) |
| Harness | `run-policy-ab.sh 4 25 5` — 4 rounds, 25 min preheat, 5 reheat runs |
| Policies | round-robin / least-queue / ect, order rotating each round |
| Everything else | 3 nodes · 2 connections/node · c36 · 60 s/run — same as the 2nd and 3rd |

**Two harness changes against the 2nd and 3rd rounds.** Both aim to shorten the
"low-load interval between policies" §10.2 identified.

1. `verify_nodes`'s probe goes from `c12` to `c36`. Node-count verification is
   independent of load, so the 10 seconds immediately before measuring are spent
   heating rather than cooling. Combined with the 14 seconds of no load during
   scheduler restart, the **low-load interval falls from 24 s to 14 s**.
2. `LOG_DUR` fixed to account for `REHEAT_RUNS`. With these parameters the old
   formula computed 2,880 s, which would have **killed the thermal logger
   mid-measurement** (the actual run took ~6,400 s).

### 17.2 The gate — was strong heterogeneity reproduced

**If reproduction fails, LQ vs ECT is not judged.** The 2nd round failed to
answer because the condition was not met, so this gate exists to avoid dressing
the same failure up as a "result".

| Metric | Criterion | Basis |
|---|---|---|
| **max/min of per-node p50 in the RR rounds** | **≥ 2.0×** | S0-A 2.4×, 2nd round 1.33×. Draw the line between them |
| soc max | ≥ 85 °C (secondary) | S0-A 86–88 °C, 2nd round 78–79 °C |

**RR is used as the heterogeneity gauge** — it does not adapt, so it shows the
raw capacity spread that surfaces under an even load. S0-A's 2.4× uses the same
definition.

- Below 2.0× → **judgement withheld.** Record it as failing the condition and do
  not touch the default.
- Around 1.33× → the harness changes were insufficient. Revisit the
  continuous-heating design.

### 17.3 LQ vs ECT decision bands

With n=4 per policy, small differences are unusable (§16). **A difference has to
clear the band to count as "winning".**

| Axis | Winning criterion | Basis |
|---|---|---|
| Throughput | relative difference **≥ 2%** | the same value as §13's regression band (±2%) |
| Tail (p99) | relative difference **≥ 5%** | 6–8% observed in the 2nd and 3rd rounds, SD ~1–2. 5% separates them |

Secondary metrics (used for interpretation, not for the verdict): the levelling
ratio of per-node p50 max/min, per-node CPU busy spread, and the size of king's
distribution shift.

### 17.4 Decision matrix

`ect` is **the incumbent** (`policy.rs` `default()`). Unseating an incumbent
requires positive grounds — this tie-break is also a pre-registered rule.

| Throughput (ECT−LQ) | Tail (LQ favoured) | Decision |
|---|---|---|
| ECT ≥ +2% | < 5% | **keep `ect`** — question closed |
| < 2% | LQ ≥ 5% | **switch to `least-queue`** — question closed |
| ECT ≥ +2% | LQ ≥ 5% | **no dominance.** Keep `ect` but state the trade-off and leave the question **open** |
| < 2% | < 5% | **indistinguishable.** Keep `ect`, question **closed** — "the default does not matter" is also an answer |

The last row is the important one. If the two cannot be distinguished even under
strong heterogeneity, then rather than hunting for a harsher condition we
**close this question** and move on to S3.9b.

### 17.5 Found mid-measurement — the soc gate's instrument was wrong

During preheat the harness printed `soc: 81 80 80`, which looked like the gate
failing. In fact **the condition was being reproduced.** From the 1-second
thermal logger at the same moment:

| | Harness output | Thermal logger (last 3 min) | S0-A |
|---|---|---|---|
| king | 81 | max **86.8** · avg 85.8 · min 78.5 | 85.9–86.8 |
| queen | 80 | max **85.9** · avg 85.6 · min 78.5 | 85.0–85.9 |
| jack | 80 | max **86.8** · avg 85.8 · min 80.4 | 85.0–85.9 |
| CPU minimum | — | **1008–1200 MHz** | 816–1800 MHz |

The cause is the sampling moment. The harness reads **after** the 60-second run
finishes, over three sequential ssh calls. RK3576 cools within seconds once load
stops, so the value falls into the inter-run trough (min 78.5–80.4) — matching
the thermal logger's min exactly. The CSV's `max_soc_c` column works the same
way and, **despite its name, is not a maximum.**

> **The gate criterion (85 °C) stays. What changes is only the data source.**
> Moving a threshold to fit results and fixing an instrument that was measuring a
> different quantity are different acts. The secondary soc gate is judged from
> **the within-run maximum from the 1-second thermal logger.** The primary gate
> (RR per-node p50 max/min) comes from the bench JSON and is unaffected.

Side effect: S0-C's 2nd-round "78–79 °C" came from the same instrument.
**Temperature during load in the 2nd round may have been higher** — which puts
§10.2's explanation, attributing the 2nd round's 1.33× to "being less hot", up
for review. The 2nd round's thermal log survives, so it can be checked.

## 18. Results (4th) — **gate not met. LQ vs ECT is not judged.**

- Raw data: [`../../results/policy-ab-20260821-contaminated/`](../../results/policy-ab-20260821-contaminated/)
  — **only the r1 round-robin run and the thermal log are valid.** The rest is
  invalidated by the harness collision (that directory's `README.md`,
  [S0-D](S0_D_CAPACITY_HETERO.md) §4)
- After the 25-minute preheat, only r1 was measured and the run was
  **deliberately stopped** (§18.3).

### 18.1 Primary gate: 1.10× (criterion 2.0×)

```text
r1 round-robin   384.8 inf/s   p50 88.8  p95 140.2  p99 173.9  err 0
   distribution  king 33.3 / jack 33.3 / queen 33.3
   node p50      king 93.3  jack 88.4  queen 85.1   ->  max/min 1.10x
```

Lower even than the 2nd round (1.33×). **Strong heterogeneity was not
reproduced.**

### 18.2 And yet the thermal condition reproduced perfectly

The 1-second thermal logger was aggregated the same way as for S0-A (with
§17.5's instrument fix applied).

| | soc_max | soc_avg | **CPU p50** | CPU min | node p50 spread |
|---|---:|---:|---:|---:|---:|
| **S0-A** king | 86.8 | 84.3 | **1008** | 816 | |
| S0-A queen | 85.9 | 83.5 | **1800** | 1416 | |
| S0-A jack | 86.8 | 83.9 | **1800** | 1200 | **2.4×** |
| **4th** king | 86.8 | 84.3 | **1416** | 1008 | |
| 4th queen | 85.9 | 83.8 | **1608** | 1200 | |
| 4th jack | 86.8 | 84.3 | **1608** | 1008 | **1.10×** |

**The soc values match to the decimal.** What differs is only the CPU clock
distribution.

| | Clock spread (p50 max/min) | Latency spread |
|---|---:|---:|
| S0-A | **1.79×** | **2.4×** |
| 4th | **1.14×** | **1.10×** |

### 18.3 So what was wrong — the premise

The handoff and §10.2 attributed the 2nd round's 1.33× to **"being less hot"**
and prescribed continuous heating. This measurement refutes that premise.

> **Thermal conditions are necessary for heterogeneity but not sufficient.**
> The sufficient condition is **the downgrade differing per board**, and thermal
> control targets temperature, not the spread. If the three boards come down
> **together** at the same temperature, no heterogeneity appears.

S0-A's 1.79× divergence is a product of silicon, airflow and position variance,
and **is not something a cooling condition can summon.** More preheat will not
make them diverge — all three boards were already under thermal control.

Once that was clear, the remaining 11 runs (about 1 hour 20 minutes) bought
nothing beyond raising n to 4 on a negative result, so the run was stopped.
**Better to leave the verdict as not-met and change the design.**

### 18.4 §10.2's "78–79 °C" was an instrument error — **corrected**

The 2nd round's thermal log was re-aggregated
(`results/policy-ab-20260821b`). **The 2nd round was also 86.8 °C.** The
78–79 °C is an artefact of §17.5's instantaneous-value instrument.

| Experiment | soc_max (1 s logger) | Value in the document | CPU p50 | Clock spread | Latency spread |
|---|---|---|---:|---:|---:|
| S0-A | 86.8 / 85.9 / 86.8 | 85.9–86.8 ✓ | 1008 / 1800 / 1800 | **1.79×** | **2.4×** |
| **S0-C 2nd** | **86.8 / 85.9 / 86.8** | **78–79 ✗** | 1200 / 1800 / 1800 | **1.50×** | **1.33×** |
| S0-C 4th | 86.8 / 85.9 / 86.8 | — | 1416 / 1608 / 1608 | **1.14×** | **1.10×** |

**The thermal conditions of all three experiments are identical.** §10.2's
explanation ("the 2nd round was less hot") loses its basis. The real cause is
**that the downgrade diverged less.**

And the three points form a monotone series.

```text
clock spread    1.14x -> 1.50x -> 1.79x
latency spread  1.10x -> 1.33x -> 2.40x
```

What determines the size of the heterogeneity is not temperature but **the clock
spread**. Take hold of the clock directly and the heterogeneity can be placed at
any desired value (§19).

> For reference, the 1st round (`results/policy-ab-20260821`, 2 runs) had CPU p50
> at 2208 MHz on all three boards and an soc average of 77.8 °C — there was no
> downgrade at all. §5.1's diagnosis, that the thermal condition was not
> maintained, stands.

## 19. Next — make heterogeneity deterministic

Instead of waiting for heat to produce a spread, **take hold of the handle
thermal control itself uses.** The downgrade is implemented by pulling
`scaling_max_freq` down (king was observed at `1008000` during measurement), and
we can write the same file.

```text
fan ON (thermally homogeneous, cool)  +  king 1008 MHz / queen 1800 / jack 1800
   = replicating S0-A's CPU p50 profile exactly
```

Advantages:

1. **Reproducible.** No reliance on silicon luck.
2. **Heat leaves the variable set.** With the fan on, the cap holds (thermal
   control has no reason to go lower) and there is no drift. The disturbance to
   the policy comparison disappears.
3. **The spread can be swept.** Sweeping 1.0× / 1.3× / 1.8× / 2.4× yields a far
   stronger conclusion than "which one wins at 2.4×" —
   **"does ECT gain as the spread widens"**. That hypothesis is exactly ECT's
   design rationale for being the default (reflecting service rate), so the sweep
   tests that rationale directly.

The cost: since it is not thermally induced, **it cannot be called thermal
heterogeneity.** It is recorded separately as capacity heterogeneity. The causal
claim that the policies work under thermally induced heterogeneity was already
closed in the 2nd round (§11), and the remaining question is **the policies'
response to the size of the spread**, so the substitution is legitimate.

---

## Figure

![RR's p99 SD explodes only under heterogeneity (+/-34.7 vs +/-1)](../../results/policy-ab-20260821b/figures/fig_policy_tail.png)

**`fig_policy_tail.png`** — RR's p99 SD explodes only under heterogeneity
(±34.7 vs ±1)

Regenerate: `python scripts/make-experiment-figures.py`
