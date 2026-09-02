# Figures — gRPC baseline 30-run

*[English](README.md) — 영문이 정본이다.*

`scripts/make-figures.py` 로 생성 (동결 commit `254d560` 데이터).
색상은 dataviz 스킬의 검증된 default 팔레트(king=blue / queen=orange / jack=aqua).

| 파일 | 내용 |
|---|---|
| `fig1_throughput_vs_node.png` | Throughput vs Node (measured ±SD + ideal) — near-linear |
| `fig2_scaling_efficiency.png` | Scaling efficiency (~100%) |
| `fig4_latency_percentiles.png` | Latency p50/p95/p99 vs node (stable) |
| `fig5_per_node_distribution.png` | round-robin 균등 (33.3%, 0.00 %p) |
| `fig7_timing_breakdown.png` | TimingBreakdown — non-inference overhead is dominated by payload transfer (94%) |
| `fig8_local_vs_cluster.png` | Local 161.5 vs Cluster 112.9 (-30.1%) |

인터랙티브 버전: `../dashboard.html` (브라우저에서 열기, hover·다크토글·테이블).
제목에는 옛 이름(NPUForge)이 그대로 남아 있다 — 측정 시점 생성물이므로
생성된 상태 그대로 동결한다.
재생성: `python scripts/make-figures.py`
