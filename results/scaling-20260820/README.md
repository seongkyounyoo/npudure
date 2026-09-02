# S2 multi-node scalability — the first measurement (2026-08-20)

*[한국어 원문](README.ko.md)*

- Measured: 2026-08-20
- Subject: 3 RK3576 nodes (king/queen/jack) + the scheduler (server .9)
- Status: **the first S2 measurement.** Not the formal one — a single run,
  without `--with-inference`
- **Cooling: active (a dedicated fan per node, fitted from the start of
  measurement).** All of today's data is with the fans on
- Raw data: [`raw/`](raw/) (3 bench JSON files)
- Reflected in: `docs/RESULTS.md` §2.5; the story: `docs/board-worklog.md` §2.25

> The first measured answer to this project's central question, **"do three
> 6 TOPS NPUs really make 18 TOPS?"**

---

## 1. The one-line conclusion

**Scaling efficiency ~98% (nearly linear).** Three nodes at **2.93×** a single
node (on a cluster basis). The bottleneck is not scaling but **per-node gRPC
overhead** — the cluster node ceiling of 115 is 27% below the local ceiling of
157.

---

## 2. Measurement conditions

**This measurement's baseline conditions (fixed):**

```text
Cooling      : active - a 120mm 5V USB fan per node (larger than the board)
CPU governor : performance
Network      : 2.5GbE / node, 10GbE server (aggregation)
Transport    : gRPC (tonic + protobuf)
Model        : YOLOv8n INT8, want_float=0
```

| Item | Value |
|---|---|
| Model | YOLOv8n **INT8** (sha256 `dba155d2…`) |
| Output | `want_float=0` (blob v2, dequantization parameters included) |
| CPU governor | `performance` |
| Cooling | **active — a dedicated fan per node** (from the start of measurement) |
| Communication | **gRPC** (tonic + protobuf), via the scheduler |
| Policy | round-robin |
| worker_count | 8 (a dedicated RKNN context per worker) |
| Input | raw RGB 640×640×3 = 1,228,800 byte/request |
| Load tool | `npuforge-bench` (closed-loop), run on server (.9) |
| Duration | 30 s (20 s for the 1-node concurrency sweep) |
| Runs | 1 per condition (no repetition) |
| preflight | passed (alias, hashes, governor, temperature, voltage, NTP) |

The measurement path (all three hops over gRPC):

```text
npuforge-bench --gRPC--> scheduler(.9:50051) --gRPC--> node(.X:51001) --> RKNN
   SchedulerService.Infer        NodeService                    (only here is the NPU)
   (bench and scheduler are on    (the real 2.5G network section)
    the same host, so loopback)
```

Reducing node count was done by stopping processes (jack, then queen), with
cooldown in between.

---

## 3. Results

### 3.1 Equal per-node load (concurrency = 8 × node count)

| Configuration | concurrency | Throughput | Distribution | Error rate | Raw file |
|---|---:|---:|---|---:|---|
| 1 node (king) | 8 | **111.6 inf/s** | king 100% | 0% | `raw/yolov8n-c8-n3.json` |
| 2 nodes (king, queen) | 16 | **228.7 inf/s** | 50 / 50 | 0% | `raw/yolov8n-c16-n3.json` |
| 3 nodes | 24 | **337.7 inf/s** | 33 / 33 / 33 | 0% | `raw/yolov8n-c24-n3.json` |

Round-robin distributed exactly evenly, with 0 failures.

### 3.2 Single-node concurrency sweep (the saturation point)

king alone, 20 s. Concurrency is raised to find the node ceiling.

| concurrency | 8 | 16 | 32 |
|---|---:|---:|---:|
| Throughput | 111.6 | 114.0 | **115.1 (saturated)** |

Even at 4× the concurrency it **saturates at ~115 inf/s.** That is the
single-node ceiling through the scheduler.

### 3.3 Latency (3 nodes at c24)

```text
round trip (client's view)   min 35.2  p50 67.8  p90 92.7  p99 122.5  ms
node-reported inference time p50 22.3  p99 44.6  ms
```

---

## 4. Analysis

### 4.1 Scaling is linear (efficiency ~98%)

```text
against the 1-node saturation of 115
  2 nodes 228.7 = 1.99x   (efficiency 99%)
  3 nodes 337.7 = 2.93x   (efficiency 98%)
```

Data parallelism (`adrs/001`) holds, and **the single scheduler (`adrs/003`) is
not a bottleneck even with three nodes.** Because the nodes are independent of
one another, the scheduler and network do not degrade in proportion to node
count.

### 4.2 The per-node ceiling is cut by 27%

| Measurement method | Node ceiling | Cooling | Difference |
|---|---:|---|---|
| Local `sustained_load_test` (no gRPC) | **161.5** inf/s | active cooling, 8 workers | reference |
| Cluster (through the scheduler's gRPC) | ~115 inf/s | active cooling, 8 workers | **−28.8%** |
| (for reference) local fanless | 157.2 inf/s | fanless, 08-11/12 | |

The round-trip p50 is 69 ms while node-reported inference is 24–28 ms. The
remaining 40 ms+ appears to be overhead from going through gRPC — protobuf
serialization + transferring 1.17 MiB in and out + queueing and routing. The
bench↔scheduler section is loopback and barely contributes, so most of it is the
**scheduler↔node 2.5G gRPC section.**

> ✅ **Cooling unified (2026-08-20).** The local fan baseline re-measured at
> 161.5 inf/s (8 threads, 8 workers, `board-worklog.md` §2.27). With cooling and
> workers matched to the cluster, the overhead is
> **(161.5−115)/161.5 = 28.8%.** The difference from the fanless 157.2 is small
> because 30/60 seconds is before throttling, so cooling has little effect.
> **27% is settled as 28.8%** (on short measurements). Under sustained load
> (300 s) the fan's benefit grows and the overhead could widen further. That
> **94% of the overhead is payload transfer** was decomposed with
> `TimingBreakdown` (§2.26).

---

## 5. A note on the raw files

Three bench JSON files in `raw/`. **The `-n3` in the filenames is inaccurate** —
the bench labels the run_id's node count from the **initial ListNodes (scheduler
registrations)** rather than the nodes active at measurement time. Stopping a
node leaves its registration, so it was stamped as 3.

**The nodes actually measured are established from each JSON's
`summary.per_node` (the active distribution) and `nodes_after` (the temperature
rise).**

| File | run_id | Actual nodes (`per_node`) | Basis |
|---|---|---|---|
| `yolov8n-c8-n3.json` | c8-n3 | **1 node**, king | in nodes_after only king goes 36→49.9 °C |
| `yolov8n-c16-n3.json` | c16-n3 | **2 nodes**, king and queen | 2 entries in per_node |
| `yolov8n-c24-n3.json` | c24-n3 | **3 nodes** | 3 entries in per_node |

→ **Fixed on 2026-08-20.** The bench's run_id and `node_count` now come from
`per_node.len()` (the actually active nodes). These `raw/` files are from
**before** the fix and still say `-n3` — the data is valid and the actual nodes
are as tabulated above.

Each JSON has `verdict.valid = true`, with the closed-loop caveat in `caveats`
(`adrs/028`: absolute latency is not quoted as an SLA; for comparison between
configurations only).

---

## 6. Reproduction

```bash
# after bringing up the 3-node cluster (scheduler + king/queen/jack)
ssh npuforge-server '/root/npuforge/target/release/npuforge-bench \
  --scheduler http://127.0.0.1:50051 --model yolov8n \
  --concurrency 24 --duration 30 --policy round-robin --out /tmp/s2'

# reduce node count by stopping processes (by comm, never -f - adrs/017 pitfall 1)
ssh npuforge-j 'pkill -9 npuforge-node; sleep 3'   # -> 2 nodes
ssh npuforge-q 'pkill -9 npuforge-node; sleep 3'   # -> 1 node
```

---

## 7. What remains for the formal S2

- Repeated runs (to establish variance)
- **A comparison against the fanless (S0-A) condition** — today's baseline is
  active cooling (fans on)
- `preflight --with-inference` (accuracy agreement across the three boards)
- The full concurrency sweep (the ceiling curve at each node count)
- Comparing 2-node combinations (king+queen vs king+jack)
- **Re-measuring the local baseline under the same fan conditions** — essential
  to settle the 27% precisely
- **Decomposing the overhead with `TimingBreakdown`** — is the 27% transfer,
  queueing or serialization
- Reducing node count via the drain RPC (letting in-flight requests through and
  excluding cleanly, `adrs/027`)
