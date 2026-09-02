# S2 — gRPC Multi-node Scaling Baseline

*[한국어 원문](S2_GRPC_BASELINE.ko.md)*

- Experiment ID: **S2**
- Measured: 2026-08-20
- Frozen commit: `254d560` (no code or configuration changes during measurement)
- Status: **complete · reproduction confirmed (30 runs)**
- Raw data: [`../../results/baseline-20260820/raw/`](../../results/baseline-20260820/raw/) · figures: [`figures/`](../../results/baseline-20260820/figures/) · dashboard: [`dashboard.html`](../../results/baseline-20260820/dashboard.html)

---

## 1. Research Question

> **Does aggregate inference throughput increase approximately linearly as
> identical low-cost NPU nodes are added to an Ethernet-connected edge cluster?**

With low-cost edge NPUs (RK3576, 6 TOPS) tied together over Ethernet, **does
total inference throughput grow close to linearly as nodes are added?** The
question is measured scaling efficiency, not the sum of nominal TOPS.

## 2. Hypothesis

Under the data-parallel design ([`adrs/001`](../../adrs/001-data-parallel-only.md)),
nodes handle different requests independently of one another. Since no
node-to-node communication sits in the inference path, **throughput should be
linear in node count as long as the single central scheduler does not become a
bottleneck.** At the same time, going through the cluster (gRPC + network)
should reduce per-node throughput **by some fixed proportion** against local
direct inference — the overhead.

## 3. System Under Test

| Item | Value |
|---|---|
| Board | NanoPi R76S ×3 (king / queen / jack) |
| SoC / NPU | Rockchip RK3576 / 2-core 6 TOPS |
| Model | YOLOv8n **INT8** (sha256 `dba155d2…`), `want_float=0` |
| Input | raw RGB 640×640×3 = 1,228,800 byte/request |
| Scheduler host | server (.9): Xeon E5-2630L ×2 (24T) / 16 GB / Rocky 9.4 |
| Network | worker 2.5GbE / aggregation 10GbE (NEXI NS-S25G10G-N) |
| Transport | **gRPC** (tonic + protobuf) |
| Topology | client → scheduler(.9) → node, all 3 hops gRPC |

Topology and rationale: [`adrs/014`](../../adrs/014-10g-aggregation-separate-scheduler.md),
[`docs/infrastructure.md`](../infrastructure.md).

## 4. Experimental Controls

Held fixed across every run.

```text
Cooling      : Active cooling - 120mm 5V USB fan per node (from the start)
CPU governor : performance
Policy       : round-robin
Worker count : 8 / node  (a dedicated RKNN context per thread, adrs/007)
Transport    : gRPC
Model        : YOLOv8n INT8, want_float=0
Warmup       : excluded
```

- **Cooling is active (fan ON).** Not fanless — see
  [`docs/board-worklog.md`](../board-worklog.md) §2.24 and §2.27.
- `preflight-check.sh` passed before measuring (alias↔hostname, hashes,
  governor, temperature, voltage, NTP).

## 5. Measurement Method

- Load tool: `npuforge-bench` (**closed-loop**), run on server (.9).
- Equal load per node: **concurrency = 8 × node count** (1N c8 / 2N c16 / 3N c24).
- **10 runs of 60 s** per condition. 30 runs total.
- **Condition order rotates** so that drift in time or temperature does not
  land on one condition:
  ```text
  Round 1: 1N -> 2N -> 3N
  Round 2: 2N -> 3N -> 1N
  Round 3: 3N -> 1N -> 2N   (repeating)
  ```
- Reducing node count means stopping the process; cooldown between runs.
- Script: [`scripts/run-grpc-baseline30.sh`](../../scripts/run-grpc-baseline30.sh).
  Code and configuration frozen for all 30 runs.

> Because the bench is closed-loop, absolute latency is never quoted as an SLA
> — it is used only **for comparison between configurations**
> ([`adrs/028`](../../adrs/028-bench-run-validity.md)).

## 6. Validation / Integrity Checks

All 30 runs checked. **This is the basis for trusting the measurement.**

| Check | Result |
|---|---|
| Run count | 30 / 30 |
| Active-node determination | **30/30 correct** (n1=1, n2=2, n3=3) |
| Invalid runs (verdict) | 0 |
| Error rate (inference) | **0.00%** (every run) |
| Retries | 0 |
| Load-balance deviation | **0.00 pp** |

- Active node is determined from **the nodes that actually served requests**
  (`per_node`), not from registered nodes. A bench fix resolved the problem of
  registrations persisting after a node was stopped (board-worklog §2.28).
- The retry count comes from the `attempts` field in the response protocol —
  the scheduler's actual attempts.

## 7. Results

### 7.1 Throughput

| Nodes | Throughput Mean ± SD |
|---:|---:|
| 1 | **112.9 ± 0.5** inf/s |
| 2 | **229.0 ± 0.9** inf/s |
| 3 | **338.4 ± 1.1** inf/s |

SD of 0.5–1.1 is extremely small — throughput barely moved across 30 runs. The
first measurement of 337.7 reproduced as 338.4 ± 1.1.
→ [fig1](../../results/baseline-20260820/figures/fig1_throughput_vs_node.png)

### 7.2 Speedup

| Reference | 2N | 3N |
|---|---:|---:|
| 1-node c8 (112.9) | 2.03× | **3.00×** |
| single-node saturation (~115) | 1.99× | 2.94× |

### 7.3 Scaling Efficiency

Against the 1-node c8 reference: **100% / 101% / 100%**; against saturation,
3N ≈ **98%**.
→ [fig2](../../results/baseline-20260820/figures/fig2_scaling_efficiency.png)

### 7.4 Latency (round-trip, closed-loop)

**The 30-run average of run-level percentiles** (see the caveat in §7.4.1):

| Nodes | p50 | p95 | p99 |
|---:|---:|---:|---:|
| 1 | 68.0 | 100.8 | 116.3 ms |
| 2 | 67.0 | 100.1 | 118.6 ms |
| 3 | 67.6 | 102.7 | 123.9 ms |

The latency distribution stays nearly flat as nodes are added — scaling does not
degrade latency.

#### 7.4.1 Caveat — these are not pooled percentiles

Percentiles are computed within each run over that run's requests
(nearest-rank, `stats.rs`), and **those run-level values are then averaged**.
This differs from pooling all 30 runs' requests and re-sorting.

```text
what was used   mean( p99(run1), p99(run2), ..., p99(run30) )
what it is not  p99( run1 u run2 u ... u run30 )
```

In general, **run-level averaging makes the tail read lower than pooled** —
each run's worst window is diluted by the average. This is fine for *comparing*
configurations (every condition is treated the same way), but **the absolute
values must not be quoted as "this system's p99".**

Producing pooled percentiles requires the per-request latency source, and the
bench only writes summary percentiles to JSON. Adding a raw dump option is
filed in `TODO.md` §1.2.
→ [fig4](../../results/baseline-20260820/figures/fig4_latency_percentiles.png)

### 7.5 Load Distribution

Round-robin split the three nodes at **exactly 33.3% each** (deviation 0.00 pp).
→ [fig5](../../results/baseline-20260820/figures/fig5_per_node_distribution.png)

## 8. Timing Breakdown

The 11 stages of the response `Timing` (proto), 30-run average of p50 (ms):

| Stage | 1N | 3N |
|---|---:|---:|
| scheduler_queue | 0.00 | 0.00 |
| scheduler_route | 0.00 | 0.00 |
| **network_to_node** (input) | 17.72 | 17.11 |
| node_queue | 0.02 | 0.02 |
| **inference (NPU)** | 24.70 | 22.49 |
| **network_to_client** (output) | 17.72 | 17.11 |
| **end_to_end** | 61.54 | 58.83 |

```text
non-inference overhead = end_to_end - inference = 58.83 - 22.49 = 36.34 ms
payload transfer       = network_to_node + network_to_client = 34.21 ms
```

- `scheduler_queue` and `scheduler_route` are **~0** regardless of node count —
  a single scheduler is not a bottleneck even with three nodes
  ([`adrs/003`](../../adrs/003-central-simple-scheduler.md) confirmed by
  measurement).
- `network_to_node` for 1N and 3N is nearly identical (17.72 vs 17.11) — the
  transfer time of a single request is independent of node count.
- → [fig7](../../results/baseline-20260820/figures/fig7_timing_breakdown.png)

## 9. Local vs Cluster Overhead

| Mode | Cooling | Worker | Throughput |
|---|---|---:|---:|
| Local direct RKNN (no gRPC) | Active Cooling | 8 | 161.5 inf/s |
| Cluster gRPC (single node) | Active Cooling | 8 | 112.9 inf/s |

**Throughput loss = (161.5 − 112.9) / 161.5 = 30.1%.**
The local baseline was re-measured with cooling and worker count matched to the
cluster (board-worklog §2.27).
→ [fig8](../../results/baseline-20260820/figures/fig8_local_vs_cluster.png)

> ⛔ **Do not multiply the two quantities.** Throughput loss (30.1%, a
> throughput figure) and the latency breakdown (94%, a share of latency) are
> different axes. Use the wording in §10.

## 10. Interpretation

**Finding 1 — near-linear scaling (reproduced).**

> Three-node throughput reached **3.00×** the one-node c8 baseline and **~98%**
> of the single-node saturation-derived ideal. All 30 runs completed without
> inference errors or retries, with effectively uniform round-robin distribution.

**Finding 2 — node-level overhead is payload transfer.**

> Local direct inference reached **161.5 inf/s** while single-node cluster
> throughput reached **112.9 inf/s**, a **30.1% throughput reduction**.
> Separately, latency decomposition showed that **94% of non-inference latency
> was observed in the payload-transfer path** — not in serialization, scheduler
> queueing, or node queueing (all ~0).

The two reinforce each other. Scaling is linear because neither the scheduler
nor the network bottlenecks on node count (Finding 1), while the absolute
per-node ceiling is cut by the time it takes to carry the payload over 2.5G
(Finding 2). What optimization should aim at is not compute but **transport**.

## 11. Limitations

- **The measurement window is short (60 s / 30 runs).** CPU throttling shows up
  at −27% over 300 s (board-worklog §2.24), so this result sits **before the
  throttling region**. Sustained-load throughput is settled in a separate
  experiment (S0).
- **Cooling axis.** Active cooling only. There is no fanless (condition A)
  cluster measurement here.
- **Saturation not established.** 1N was seen near ~115 at c8/c16/c32 but c48
  was not measured, and the 2N/3N ceilings were not swept → **S3**.
- **Serialization not measured in isolation.** The proto `Timing` has no field
  for gRPC serialization alone; it currently sits inside the ~2 ms non-inference
  residual. An additional instrumentation point is needed.
- **Closed-loop.** Not absolute latency; for comparison between configurations
  only.
- **A single 2-node combination (king+queen).** Other combinations such as
  king+jack were not measured.

## 12. Reproduction

```bash
# after bringing up the 3-node cluster (scheduler + king/queen/jack)
bash scripts/run-grpc-baseline30.sh        # 30 runs -> server:/tmp/baseline30
# local fan baseline (Finding 2):
ssh npuforge-k 'pkill -9 npuforge-node; sleep 3; cd ~/npuforge-rknn-test; \
  ./sustained_load_test yolov8n-int8.rknn 60 8'
# regenerate figures:
python scripts/make-figures.py
```

Frozen commit: `254d560`. The fixed-condition table is §4.

## 13. Raw Data

- 30 bench JSON files: [`../../results/baseline-20260820/raw/`](../../results/baseline-20260820/raw/)
  (`n{nodes}_r{round}.json`; each carries throughput, latency, node_inference,
  TimingBreakdown, per_node, nodes_before/after (temp, voltage), verdict, run_id)
- Aggregate report: [`../../results/baseline-20260820/README.md`](../../results/baseline-20260820/README.md)
- Figures and dashboard: [`figures/`](../../results/baseline-20260820/figures/), [`dashboard.html`](../../results/baseline-20260820/dashboard.html)

## 14. Conclusion

Across 30 repeated runs the RK3576 3-node NPU cluster showed **near-linear
scaling (338.4 ± 1.1 inf/s, 3.00×, error 0%)**. The TimingBreakdown confirmed
that per-node overhead lies not in compute or the scheduler but in the
**payload-transfer path** (94% of non-inference latency).

→ The gRPC baseline is **frozen**. Next: **S3** (saturation / scaling limit) →
**S4** (io_uring). S4 compares transport cost against this baseline under
**identical conditions**.
