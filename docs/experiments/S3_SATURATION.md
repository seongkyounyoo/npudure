# S3 — Per-configuration Saturation

*[한국어 원문](S3_SATURATION.ko.md)*

- Experiment ID: **S3**
- Measured: 2026-08-20
- Frozen commit: `1da69d4` (bench code `254d560`, same as S2). No changes during measurement
- Status: **complete (45 runs)**
- Raw data: [`../../results/saturation-20260820/raw/`](../../results/saturation-20260820/raw/) · figure: [`figures/fig3`](../../results/saturation-20260820/figures/fig3_saturation_sweep.png)
- Predecessor: [`S2_GRPC_BASELINE.md`](S2_GRPC_BASELINE.md)

---

## 1. Research Question

> **What is the maximum sustainable throughput (ceiling) of each cluster
> configuration, and at what concurrency is it reached?**

**This is a different question from S2.** S2 looked at linearity under
*identical per-node load* (c = 8×N). S3 explores each configuration's **true
ceiling** (1/2/3 node) by raising concurrency. The two experiments are not
mixed.

## 2. Method

- Concurrency sweep, past per-node load and up to saturation:
  ```text
  1 node : c4, c8, c16, c32, c48
  2 node : c8, c16, c24, c32, c48
  3 node : c12, c24, c32, c48, c64
  ```
- **3 runs of 30 s** per point. Condition order rotates. 45 runs total.
- Fixed conditions match S2 (INT8, want_float=0, performance, active cooling,
  round-robin, 8 workers, gRPC). The freeze holds.
- Script: [`scripts/run-saturation-sweep.sh`](../../scripts/run-saturation-sweep.sh).

## 3. Results — Saturation Curves

Mean of 3 runs (inf/s); every SD is ≤ 2.2:

| concurrency | 1 node | 2 node | 3 node |
|---:|---:|---:|---:|
| c4 | 84.0 | | |
| c8 | 112.6 | 168.3 | |
| c12 | | | 252.2 |
| c16 | 113.8 | 228.1 | |
| c24 | | **232.0** | 339.4 |
| c32 | **115.2** | 230.2 | **341.8** |
| c48 | 114.1 | 230.3 | 339.2 |
| c64 | | | 335.9 |

**Ceilings:**

| Config | Ceiling | @ concurrency | per-node concurrency |
|---|---:|---:|---:|
| 1 node | **115.2** inf/s | c32 | 32 |
| 2 node | **232.0** inf/s | c24 | 12 |
| 3 node | **341.8** inf/s | c32 | ~11 |

→ [Figure 3](../../results/saturation-20260820/figures/fig3_saturation_sweep.png)

## 4. Interpretation

**Finding — near-linear even at the ceiling.**

| Config | Ceiling | Speedup (vs 1-node ceiling) | Efficiency |
|---|---:|---:|---:|
| 1 node | 115.2 | 1.00× | 100% |
| 2 node | 232.0 | 2.01× | 101% |
| 3 node | 341.8 | **2.97×** | **99%** |

S2 showed linearity under identical load; S3 shows it at maximum throughput.
**Near-linear scaling is confirmed independently from two angles.** The 3-node
ceiling of 341.8 inf/s is 2.97× the 1-node ceiling.

Three regions of the curve:

- **Low concurrency (unsaturated):** throughput is held down by round-trip
  latency (≈68 ms, S2 §7.4). Being closed-loop, too few in-flight requests
  leave the pipeline empty (1N c4 = 84, 3N c12 = 252).
- **Plateau (saturated):** maximum at roughly 10–16 concurrent per node. Once
  the pipeline keeps the 8 workers fed, raising it further adds nothing.
- **Overload (slight decline):** beyond that, only queueing grows and throughput
  dips slightly (3N c32 341.8 → c64 335.9). Errors remain 0 — the scheduler and
  node queues absorb it.

## 5. Limitations

- Same as S2: short measurement window (30 s, before throttling), active cooling
  only, closed-loop, one 2-node combination (king+queen).
- **The duration differs from S2 (30 vs 60 s).** The ceiling values
  (115/232/342) are close to S2's c8/c16/c24 (112.9/229.0/338.4) but the
  conditions are not identical. Saturation is about the shape of the curve and
  where the ceiling sits; for absolute values, S2 takes precedence.
- The decline past the ceiling is a closed-loop queueing effect — it may look
  different under an open model
  ([`adrs/028`](../../adrs/028-bench-run-validity.md)).

## 6. Reproduction

```bash
bash scripts/run-saturation-sweep.sh    # 45 runs -> server:/tmp/sat30
python scripts/make-figures.py          # regenerate Figure 3
```
Frozen commit `1da69d4`.

## 7. Raw Data & Conclusion

- 45 raw files: [`../../results/saturation-20260820/raw/`](../../results/saturation-20260820/raw/)
  (`sat_n{nodes}_c{concurrency}_r{round}.json`)

**Conclusion.** The throughput ceiling of each configuration is
**115 / 232 / 342 inf/s** at 1/2/3 nodes, and 3 nodes reach **2.97× (99%)** of
the 1-node ceiling — **near-linear by the ceiling measure too**. Saturation
occurs at roughly 10–16 concurrent per node. This re-confirms S2's linear-scaling
conclusion from the maximum-throughput perspective.

→ Next: **S4 (io_uring)** — compare, under conditions identical to this
baseline, how much it reduces the cost of the payload-transfer path (S2 §8: 94%
of non-inference latency).
