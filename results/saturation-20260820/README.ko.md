# S3 saturation sweep — 원본·집계 (2026-08-20)

*[English](README.md) — 영문이 정본이다.*

- 동결 commit: `1da69d4` (bench `254d560`, S2 와 동일 코드)
- 원본: [`raw/`](raw/) 45건 (`sat_n{노드}_c{concurrency}_r{라운드}.json`)
- 그래프: [`figures/fig3`](figures/fig3_saturation_sweep.png)
- **실험 보고서(질문·해석·결론):** [`../../docs/experiments/S3_SATURATION.md`](../../docs/experiments/S3_SATURATION.md)

## Ceilings (3 runs/point 평균)

| Config | Ceiling | @ conc | Speedup | Eff |
|---|---:|---:|---:|---:|
| 1 node | 115.2 inf/s | c32 | 1.00× | 100% |
| 2 node | 232.0 inf/s | c24 | 2.01× | 101% |
| 3 node | **341.8 inf/s** | c32 | **2.97×** | **99%** |

ceiling 기준으로도 near-linear. 곡선 형태(미포화→plateau→과부하 하락)와
전체 sweep 표는 실험 보고서 §3 참조.
