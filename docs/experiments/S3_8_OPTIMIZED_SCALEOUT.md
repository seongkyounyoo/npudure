# S3.8 — Optimized gRPC Scale-out

*[한국어 원문](S3_8_OPTIMIZED_SCALEOUT.ko.md)*

- Experiment ID: **S3.8**
- Measured: 2026-08-20
- Code: `0af696d` + `[transport] node_connections = 2`
- Status: **complete (36 runs, node-count verification passed on 9/9 configurations)**
- Raw data: [`../../results/scaleout-optimized-20260820/`](../../results/scaleout-optimized-20260820/)
- Predecessor: [`S3_7_CONNECTION_TUNING.md`](S3_7_CONNECTION_TUNING.md)

---

## 1. Research Question

> **Does the per-node operating point S3.7 found (2 connections @ c12) hurt
> scale-out?**

S3.7 is a **single-node** result. At three nodes the scheduler holds 2 × 3 = 6
connections and carries three times the traffic. A new bottleneck may appear on
the server side.

## 2. Method

- **Sweep concurrency again at each node count and find each one's own
  operating point.** Comparing at a fixed concurrency shows overload behaviour
  rather than a configuration effect — which is exactly what happened in S3.7
  §4.3.
- Connections are **2 per node** — 1N→2, 2N→4, 3N→6 total.
- 1N: c8/12/16/20 · 2N: c16/24/32/40 · 3N: c24/36/48/60, **3 runs** each, 60 s.
- Both the node-count order and the concurrency order rotate per repetition.
- The operating-point definition matches S3.7 — **the lowest concurrency
  delivering at least 98% of peak**.

### 2.1 Node-count verification before measuring — it actually fired

For each configuration a short probe bench counts **the distribution of
responding node IDs**, and the configuration is skipped when
`expected ≠ observed`.

**On the first attempt all six 2N and 3N configurations were caught.**

```text
!! node count mismatch - expected=2 observed=1 (king). skipping this configuration
!! node count mismatch - expected=3 observed=1 (king). skipping this configuration
```

The cause was that the `npuforge-node.s36` build — the one that reads the
`[transport]` settings — had only been deployed **to king** (every experiment
after S3.6 was single-node, so it never surfaced). The startup logic is
`pgrep || run`, so a missing file fails silently.

> **Without this check we would have measured 1N three times and recorded it as
> "2N and 3N".** The result would have been 2N 136, 3N 136 — the exact opposite
> conclusion, "scale-out has completely collapsed". **A process being up ≠ it
> receiving traffic.**

After deploying to all three boards (hash `73227f64…` identical) and re-running
— **9/9 configurations passed verification**.

## 3. Results

Error rate **0** throughout; inter-node distribution deviation **0.0 pp**
throughout. Latency is the run-to-run average of run-level percentiles (not
pooled, S2 §7.4.1).

### 3.1 Curves by node count

| conc | 1N tp | 1N p95 | | conc | 2N tp | 2N p95 | | conc | 3N tp | 3N p95 |
|---:|---:|---:|---|---:|---:|---:|---|---:|---:|---:|
| 8 | 120.2 | 94.3 | | 16 | 239.9 | 95.5 | | 24 | 354.3 | 98.3 |
| **12** | **135.5** | **120.7** | | **24** | **263.3** | **140.7** | | **36** | **387.2** | **151.1** |
| 16 | 137.4 | 175.8 | | 32 | 262.5 | 237.0 | | 48 | 385.0 | 288.7 |
| 20 | 134.9 | 245.0 | | 40 | 260.6 | 349.7 | | 60 | 385.8 | 407.3 |

All three configurations **show saturation within the swept range** (both sides
of the peak are lower). Per-node concurrency is 12 in all three — S3.7b's knee
holds at multiple nodes as well.

### 3.2 Operating points compared

| Nodes | Tot.conn | Op.conc | Throughput | p95 | p99 | Scaling | Efficiency |
|---:|---:|---:|---:|---:|---:|---:|---:|
| **1** | 2 | 12 | **135.5** | 120.7 | 141.0 | 1.00× | 100.0% |
| **2** | 4 | 24 | **263.3** | 140.7 | 172.7 | **1.94×** | **97.1%** |
| **3** | 6 | 36 | **387.2** | 151.1 | 201.6 | **2.86×** | **95.3%** |

### 3.3 Against the baseline

| | baseline (S3 ceiling, conn1) | optimized (S3.8, conn2/node) | gain | per-node |
|---|---:|---:|---:|---:|
| 1N | 115.2 | **135.5** | **+17.6%** | 135.5 |
| 2N | 232.0 | **263.3** | **+13.5%** | 131.7 |
| 3N | 341.8 | **387.2** | **+13.3%** | 129.1 |
| scaling | 2.97× (eff 98.9%) | **2.86× (eff 95.3%)** | | |

## 4. Interpretation

### 4.1 Absolute throughput went up — 3 nodes +13.3%

**387.2 inf/s.** Obtained from a single line of connection configuration, with
0 errors and 0.0 pp distribution deviation — scale-out itself is healthy.

### 4.2 But scaling efficiency slipped (98.9% → 95.3%)

**We are not dressing this up.** The per-node gain shrinks as nodes are added.

```text
1N  +17.6%   (115.2 -> 135.5)
2N  +13.5%   (232.0 -> 263.3)
3N  +13.3%   (341.8 -> 387.2)
```

Per-node throughput falls monotonically: **135.5 → 131.7 → 129.1**. The
single-node optimization is **not fully preserved** at multiple nodes.

Against an ideal 3N (135.5 × 3 = 406.5), the measured 387.2 is **19.3 inf/s
short**.

### 4.3 Leading candidate: the server side

> ⚠️ **[Withdrawn 2026-08-21 — S3.9a]** The "10G 76%" below is an **arithmetic
> error**. **10GbE is full-duplex**, so requests (TX) and responses (RX) each
> get their own 10 Gbps, and the two were summed into one link budget. The
> measured figure is **40.5% per direction** (S3.9a §3).
>
> S3.9a excluded every server resource — CPU 42%, 40% per link direction, 0
> drops, no thread serialization. **The loss is entirely a rise in the tail**,
> with p50 flat.
> → [S3.9a](S3_9A_SCALEOUT_PROFILE.md)

~~Scheduler↔node traffic per inference is 2,446,800 bytes. At the 3N operating point~~

| | baseline 3N | optimized 3N |
|---|---:|---:|
| Throughput | 341.8 | **387.2** |
| ~~Server NIC load~~ | ~~6.69 Gbps~~ | ~~7.58 Gbps~~ |
| ~~Against the 10G link~~ | ~~67%~~ | ~~76%~~ |

**Those two rows are withdrawn.** Summing both directions cannot be applied to
a full-duplex link.

### 4.4 Latency rises with node count

Even though per-node load is 12 in all three cases, p95 at the operating point
rises **120.7 → 140.7 → 151.1 ms**. Since the per-node conditions are identical,
that increase can be attributed to the **scheduler fan-out path** — though which
stage it comes from was not decomposed.

## 5. Limitations

- **The cause of the efficiency drop is unidentified** (§4.3). Whether it is the
  server NIC, CPU or scheduler fan-out was not separated. A **server-side
  profile** in the manner of S3.5 is needed.
- These are 60-second measurements, sitting **before throttling appears**, so
  sustained load (S0) may differ.
- The concurrency grid is coarse (steps of 4 at 1N, 12 at 3N). Use it for
  comparison between configurations rather than for the absolute operating-point
  value.
- Percentiles are run-level averages (S2 §7.4.1).
- Only one 2-node combination was examined: king+queen.

## 6. Reproduction

```bash
bash scripts/run-scaleout-optimized.sh 3     # 36 runs, about 50 minutes
PYTHONIOENCODING=utf-8 python scripts/analyze-scaleout.py \
    results/scaleout-optimized-20260820/raw/results.csv
```

> `npuforge-node.s36` (the build with `[transport]` support) must be deployed on
> all three boards. Without it the node-count check skips that configuration —
> stopping loudly instead of silently producing a wrong result.

## 7. Conclusion

**The per-node operating point (2 connections @ c12) holds up to three nodes and
lifts absolute throughput from 341.8 to 387.2 inf/s (+13.3%).** Zero errors,
even distribution.

That said, **scaling efficiency slipped from 98.9% to 95.3%**, and the per-node
gain shrinks from +17.6% (1N) to +13.3% (3N) — meaning the single-node
optimization is not fully preserved at multiple nodes. ~~The server 10G link at
76%~~ — **withdrawn in S3.9a** (full-duplex arithmetic error; the real figure is
40% per direction). S3.9a excluded every server resource and confirmed the loss
is **a rise in the tail**.

→ Next is a **server-side profile**. The node side was done in S3.5; this time
the server has become a candidate. Until that result is in, the remaining gap
must not be attributed solely to node-side costs (protobuf, copies, syscalls).

---

## Figure

![Absolute values rise at every scale while efficiency goes 98.9% -> 95.3%](../../results/scaleout-optimized-20260820/figures/fig_scaleout_optimized.png)

**`fig_scaleout_optimized.png`** — absolute values rise at every scale while
efficiency goes 98.9% → 95.3%

Regenerate: `python scripts/make-experiment-figures.py`
