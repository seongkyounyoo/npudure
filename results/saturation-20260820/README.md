# S3 saturation sweep — raw data and aggregation (2026-08-20)

*[한국어 원문](README.ko.md)*

- Frozen commit: `1da69d4` (bench `254d560`, the same code as S2)
- Raw data: [`raw/`](raw/), 45 files (`sat_n{nodes}_c{concurrency}_r{round}.json`)
- Figure: [`figures/fig3`](figures/fig3_saturation_sweep.png)
- **The experiment report (question, interpretation, conclusion):** [`../../docs/experiments/S3_SATURATION.md`](../../docs/experiments/S3_SATURATION.md)

## Ceilings (mean of 3 runs per point)

| Config | Ceiling | @ conc | Speedup | Eff |
|---|---:|---:|---:|---:|
| 1 node | 115.2 inf/s | c32 | 1.00× | 100% |
| 2 node | 232.0 inf/s | c24 | 2.01× | 101% |
| 3 node | **341.8 inf/s** | c32 | **2.97×** | **99%** |

Near-linear by the ceiling measure too. The shape of the curve (unsaturated →
plateau → decline under overload) and the full sweep table are in §3 of the
experiment report.
