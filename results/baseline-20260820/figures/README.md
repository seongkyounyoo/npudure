# Figures — gRPC baseline, 30 runs

*[한국어 원문](README.ko.md)*

Generated with `scripts/make-figures.py` (data from the frozen commit
`254d560`). Colours come from the dataviz skill's validated default palette
(king=blue / queen=orange / jack=aqua).

| File | Content |
|---|---|
| `fig1_throughput_vs_node.png` | Throughput vs node (measured ±SD + ideal) — near-linear |
| `fig2_scaling_efficiency.png` | Scaling efficiency (~100%) |
| `fig4_latency_percentiles.png` | Latency p50/p95/p99 vs node (stable) |
| `fig5_per_node_distribution.png` | Round-robin evenness (33.3%, 0.00 pp) |
| `fig7_timing_breakdown.png` | TimingBreakdown — non-inference overhead is dominated by payload transfer (94%) |
| `fig8_local_vs_cluster.png` | Local 161.5 vs cluster 112.9 (−30.1%) |

Interactive version: `../dashboard.html` (open in a browser; hover, dark toggle,
tables). Its title still carries the old project name (NPUForge) — it is a
measurement-time artifact and is frozen as generated.
Regenerate: `python scripts/make-figures.py`
