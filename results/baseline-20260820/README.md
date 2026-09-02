# gRPC baseline, 30 repetitions — reproduction confirmed (2026-08-20)

*[한국어 원문](README.ko.md)*

- Measured: 2026-08-20
- Subject: 3 RK3576 nodes + the scheduler (server .9), gRPC
- **Frozen commit:** `254d560` (bench timing extension). No code or
  configuration changes during measurement
- Raw data: [`raw/`](raw/) — 30 bench JSON files (`n{nodes}_r{round}.json`)
- Summary: `docs/RESULTS.md` §2.5; the story: `docs/board-worklog.md` §2.28

> **From "337.7 came out once" to "3-node near-linear scaling confirmed across
> 30 repeated experiments."**
>
> This document is **the raw data, aggregation and figures.** The experiment
> report (research question, interpretation, conclusion) is
> [`docs/experiments/S2_GRPC_BASELINE.md`](../../docs/experiments/S2_GRPC_BASELINE.md).

---

## 1. Measurement design

- **10 runs of 60 s** each at 1N/2N/3N, concurrency = 8 × node count
  (c8/c16/c24)
- **Condition order rotates** (R1 1-2-3, R2 2-3-1, R3 3-1-2, …) so drift in time
  and temperature does not land on one condition
- Reducing node count means stopping the process, with cooldown between runs
- Fixed conditions: INT8, want_float=0, governor=performance, **active
  cooling**, round-robin, 8 workers

## 2. Integrity (the basis for trusting the measurement)

| Check | Result |
|---|---|
| Run count | 30 / 30 |
| Active-node determination | **30/30 correct** (n1=1, n2=2, n3=3; the bug fix confirmed) |
| Invalid runs | 0 |
| Error rate | **0.00%** (every run) |
| Retries | 0 |
| Load balance deviation | **0.00 pp** (round-robin perfectly even) |

## 3. The target table — repeated measurement results

| Nodes | Throughput Mean ± SD | Speedup | Efficiency | p50 ms | p99 ms | Error | Balance |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | **112.9 ± 0.5** inf/s | 1.00× | 100% | 68.0 | 116.3 | 0% | 0.00 pp |
| 2 | **229.0 ± 0.9** inf/s | 2.03× | 101% | 67.0 | 118.6 | 0% | 0.00 pp |
| 3 | **338.4 ± 1.1** inf/s | 3.00× | 100% | 67.6 | 123.9 | 0% | 0.00 pp |

- **Speedup is against the 1-node c8 reference.** Near-linear (3.00×).
- Against single-node saturation (~115), 3N = 338.4/115 = **2.94× (98%)**.
- **SD of 0.5–1.1 is extremely small** — throughput barely moved across 30 runs.
  The first measurement's 337.7 reproduced as 338.4 ± 1.1.

## 4. TimingBreakdown (30-run average of p50)

| Stage | 1N | 3N |
|---|---:|---:|
| scheduler_queue | 0.00 | 0.00 |
| scheduler_route | 0.00 | 0.00 |
| **network_to_node** | 17.72 | 17.11 |
| node_queue | 0.02 | 0.02 |
| **inference (NPU)** | 24.70 | 22.49 |
| **network_to_client** | 17.72 | 17.11 |
| **end_to_end** | 61.54 | 58.83 |

**The first measurement reproduced (3N):**

```text
non-inference overhead = end_to_end - inference = 58.83 - 22.49 = 36.34 ms
payload transfer       = network_to_node + network_to_client = 34.21 ms
  = 94% of the non-inference overhead
  = 58% of E2E
```

- `scheduler_queue` and `scheduler_route` are **~0** regardless of node count —
  a single scheduler is not a bottleneck even with three nodes (`adrs/003`
  re-confirmed).
- 1N's and 3N's `network_to_node` (17.72 vs 17.11) are nearly identical — a
  single request's transfer time is independent of node count.

> ⛔ **Do not multiply the two quantities.** Throughput loss (28.8%, §5) and the
> latency breakdown (94%) are different axes. The correct wording is in §5.

## 5. Per-node overhead (cooling and workers unified)

| Mode | Cooling | Worker | Throughput |
|---|---|---:|---:|
| Local direct RKNN (no gRPC) | Active Cooling | 8 | 161.5 inf/s |
| Cluster gRPC (single node) | Active Cooling | 8 | 112.9 inf/s |

**Throughput loss = (161.5 − 112.9) / 161.5 = 30.1%** (against the 30-run 1N
mean). The local fan baseline is in `board-worklog.md` §2.27.

> The sentence to use in a talk (no multiplying): **going through the cluster,
> single-node throughput was about 30% lower than local (throughput), and
> separately, a latency breakdown observed 94% of non-inference latency in the
> payload-transfer path.**

## 6. Conclusions

1. **3-node near-linear scaling confirmed across 30 repetitions** —
   338.4 ± 1.1 inf/s, speedup 3.00× (against 1N) / 2.94× (against saturation),
   error 0%, balance 0 pp.
2. **The per-node overhead is payload transfer** (94% of the overhead). Not
   serialization, not the scheduler queue, not the node queue. That is what
   io_uring, zero-copy, JPEG and postprocessing would aim at.

**The gRPC baseline is frozen here.** Next: the saturation sweep → (freeze
maintained) → compare io_uring under **identical conditions**.

## 7. Reproduction

```bash
bash scripts/run-grpc-baseline30.sh      # 30 runs, saved to server:/tmp/baseline30
# the local fan baseline: after stopping the king node
ssh npuforge-k 'cd ~/npuforge-rknn-test; ./sustained_load_test yolov8n-int8.rknn 60 8'
```
