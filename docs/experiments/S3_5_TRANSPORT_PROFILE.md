# S3.5 — Transport Cost Profiling

*[한국어 원문](S3_5_TRANSPORT_PROFILE.ko.md)*

- Experiment ID: **S3.5** (+ **S3.5b** RPS A/B)
- Measured: 2026-08-20
- Frozen commit: `01f29a2`. Node, scheduler, model and bench **unchanged**
- Status: **complete**
- Raw data: [`../../results/transport-profile-20260820/raw/`](../../results/transport-profile-20260820/raw/) ·
  [`../../results/rps-ab-20260820/`](../../results/rps-ab-20260820/)
- Predecessors: [`S2_GRPC_BASELINE.md`](S2_GRPC_BASELINE.md), [`S3_SATURATION.md`](S3_SATURATION.md)
- Successor: **S3.6** (H2 / channel A/B — separates the ①②③ this document leaves open, §7)

---

## 1. Research Question

> **What is actually holding the per-node ceiling at ~115 inf/s (−30% against
> ~160 for local direct)?**

S2 established that this loss lies in the **payload-transfer path** (94% of
non-inference latency). But *what* within that path costs was left open. There
are at least four candidates — link bandwidth, board CPU capacity, the kernel
network stack, and the transport layer's structure.

**This question has to close before S4 (io_uring) starts.** io_uring is a tool
for reducing syscall and copy costs. If the bottleneck is not there, a large
implementation buys nothing. In the order `01-TECHSPEC.md` §15.1 lays out
(2. CPU profile → 3. syscall/copy cost → 4. buffer pool → 5. io_uring), steps
2–4 were empty.

Also, the metrics §15.4 requires (syscalls/req, ctx switches/req, cycles/req)
are needed anyway as S4's **before** baseline, and the repository had none of
them (the 30 S2 raw files carry no CPU fields). Building first and measuring
afterwards leaves nothing to attribute an improvement to.

## 2. Method

Three conditions on the same board (king). Cooling, governor and model match S2
and S3.

| Condition | Load | Meaning |
|---|---|---|
| `idle` | none | the instrument's own floor |
| `cluster` | 1-node cluster c32 | the S3 ceiling condition |
| `local` | local direct, 8 threads | the network path removed entirely |

The difference between `cluster` and `local` is what transport costs on the
board.

- 80 s of load, collecting only **45 s from t+20** within it, excluding the ramp
  and warmup.
- Only raw `/proc` data is pulled from the board; the arithmetic happens on the
  development PC, so it can be revisited from another angle later.
- Collected: `mpstat -P ALL` (per core), `pidstat -t` (per thread),
  `/proc/PID/io` (syscr, syscw), `/proc/PID/task/*/status` (ctx switch),
  `/proc/net/dev`, `/proc/interrupts`, `/proc/softirqs`.
- Scripts: [`run-transport-profile.sh`](../../scripts/run-transport-profile.sh),
  [`node-profile-collect.sh`](../../scripts/node-profile-collect.sh),
  [`analyze-transport-profile.py`](../../scripts/analyze-transport-profile.py).

> `perf` is not on the board (kernel 6.1.141 vendor; apt only offers a 6.8
> build). cycles/req is not a PMU value but an **approximation** from per-core
> busy time × fixed clock (A53 2016 / A72 2208 MHz, governor=performance).

## 3. Results

45.1 s collection window, king, fan, performance.

| | idle | cluster | local |
|---|---:|---:|---:|
| throughput (inf/s) | 0 | **116.6** | **159.1** |
| **%idle (all 8 cores)** | 99.9 | **63.1** | 82.9 |
| %usr / %sys / %soft | 0.0 / 0.0 / 0.0 | 18.3 / 12.2 / 6.4 | 9.7 / 7.3 / 0.0 |
| **CPU0 busy** | 0.3 | **69.7** | 21.5 |
| **CPU0 %soft** | 0.0 | **51.5** | 0.0 |
| eth0 RX / TX (Gbps) | 0 | **1.196 / 1.194** | 0 |
| **vs measured link (2.34)** | — | **51.1% / 51.0%** | — |
| RX packets/s | 9 | 112,008 | 8 |
| NET_RX softirq/s | 10 | 10,954 | 8 |

Per-core busy%:

```text
cluster :  c0=70  c1=38  c2=37  c3=37  c4=30  c5=29  c6=27  c7=27
local   :  c0=21  c1=19  c2=19  c3=19  c4=15  c5=15  c6=15  c7=15
```

### Per-request cost (TECHSPEC §15.4 — S4's before baseline)

| | cluster | local | difference |
|---|---:|---:|---:|
| **syscalls/req** | **84.5** | ~0.0 | +84.5 |
| ├ read/req | 0.1 | 0.0 | |
| └ write/req | **84.4** | 0.0 | |
| ctx switch/req (vol) | 157.6 | 221.6 | −64.0 |
| ctx switch/req (nonvol) | 0.7 | 0.1 | |
| Process CPU-ms/req | **22.2** | 9.0 | **+13.2** |
| Whole-board CPU-ms/req | **25.3** | 8.6 | **+16.7** |
| ≈ Mcycles/req | 52.9 | 18.1 | +34.8 |
| RX packets/req | 960.7 | 0 | |

Transport makes each inference cost about **2.9×** the board CPU (8.6 → 25.3 ms).
Write syscalls come to **84.4 per request** — the 1,218,000-byte response is
being pushed out in roughly 14.4 KB pieces (matching the HTTP/2 frame size).

## 4. Ruling out bottleneck candidates one at a time

### 4.1 Link bandwidth — no

2.5GbE is full-duplex, so request and response split the directions.

| | bytes/inference | @116.6 inf/s | vs measured link (2.34 Gbps) |
|---|---:|---:|---:|
| RX (request 640×640×3) | 1,228,800 | 1.196 Gbps | **51.1%** |
| TX (response want_float=0) | 1,218,000 | 1.194 Gbps | **51.0%** |

Half is left in each direction. The `/proc/net/dev` measurement agrees with
ADR-008's payload sizes to within 4.7% (HTTP/2 + TCP/IP headers), so this is an
observation, not a calculation.

Server-side aggregation is not it either. Three nodes scaled 3.00× linearly
(S2 Finding 1), so the shared 10G link and the scheduler are not the bottleneck
at this point. **By the same reasoning the server and scheduler themselves are
excluded** — if the server were what kept one node under 116, three nodes could
not reach 342. The bottleneck is on the node side.

> ⚠️ **[Added 2026-08-20 — S3.8]** This exclusion **holds only under the
> baseline condition (1 connection per node).** Raising it to 2 connections per
> node increased load on the shared path, dropping optimized 3N scaling
> efficiency from **98.9% to 95.3%** and taking the server 10G link from 67% to
> **76%**. **The server and scheduler are candidates again.**
> → [S3.8 §4.3](S3_8_OPTIMIZED_SCALEOUT.md)
>
> An exclusion verdict has to carry **the conditions it was reached under**.

### 4.2 Board CPU capacity — no

All 8 cores are **63.1% idle**. Even the busiest, CPU0, has 30.3% left.

### 4.3 Kernel softirq concentration (CPU0) — **refuted by A/B**

The profile pointed at CPU0 as unusually busy (69.7% busy, of which 51.5%
softirq; the other cores are 27–38%). eth0 has **one RX queue**, its IRQ is
pinned to CPU0, and **RPS is off** (`rps_cpus=00`,
[`nic-topology.txt`](../../results/transport-profile-20260820/raw/nic-topology.txt)).
So all NET_RX softirq work is serialized onto CPU0.

Since this is verifiable with zero lines of code, it was measured first —
**S3.5b**: alternate `rps_cpus` between `00` (CPU0 only) and `fe` (cores 1–7),
3 runs of 60 s each, c32.

| rps_cpus | throughput | CPU0 %soft |
|---|---:|---:|
| `00` (default) | **115.9 ± 0.7** | 50.4 / 50.9 / 51.3 |
| `fe` (cores 1–7) | **115.6 ± 0.9** | 42.4 / 41.9 / 42.0 |
| difference | **−0.3 inf/s (−0.2%)** | |

**No effect.** The softirq work really did move (51% → 42%) and throughput did
not change. CPU0 was not the bottleneck — consistent with it having 30% left at
69.7% busy.

> This null result becomes the basis for §4.4. RPS distributes by **flow hash**.
> With only one flow there is nothing to divide. And there is, in fact, one flow.

### 4.4 The HTTP/2 transport path — **this is what is left**

The actual TCP connections were counted under load.

```text
king  <- scheduler       : 1 connection   192.168.123.3:51001 <- 192.168.123.9:37992
server: bench -> scheduler : 32 connections (c32, one per worker)
```

The code says the same.

- The bench creates **one channel per concurrency worker**
  ([`driver.rs:83-90`](../../crates/npuforge-bench/src/driver.rs)).
- The scheduler caches and reuses **one channel per node**
  ([`node_client.rs:31-79`](../../crates/npuforge-scheduler/src/node_client.rs)).
  That decision trusts HTTP/2 multiplexing, and its rationale — avoiding a
  handshake per request — is itself sound.

The result is that **32 connections on the client side converge to 1 connection
in front of the node.** All 32 concurrent requests flow through HTTP/2 streams
on that single connection. And that connection:

- is framed serially by one h2 connection state machine (a single task),
- has **one 64 KB connection flow-control window shared by 32 streams** — in
  tonic 0.12.3 / h2 0.4.15 the window is set nowhere in the code, so everything
  is at the default (65,535),
- carries one TCP flow, so it cannot be split by RPS or RSS (§4.3).

That said, **these three are still one lump.** HTTP/2 was designed precisely to
multiplex streams over a single connection. The bare fact of "one connection"
does not make it the bottleneck. It has to be split at least three ways.

| Sub-candidate | Content |
|---|---|
| ① flow control | the 64 KB default window turns a 1.2 MB message into stop-and-wait |
| ② connection/TCP path | one h2 connection state machine and socket is a serialization point |
| ③ protobuf and copies | framing, encode/decode, and the `to_vec()` copy cost |

**S3.6 separates these three** (§7). The consistency below supports "the
transport path is suspect" — it does not name which of the three.

Every observation fits the picture.

| Observation | Consistency with the single-connection hypothesis |
|---|---|
| Bandwidth 51%, CPU 63% idle | the ceiling comes from **waiting**, not resources |
| RPS ineffective | one flow, nothing to distribute |
| `node_queue` ≈ 0.02 ms | requests are not waiting for a worker; they **fail to arrive** |
| Local direct, 8 workers, same board = **161.5 inf/s** | cluster is 116. The node has headroom |
| S3 plateau (no gain past c10–16 per node) | adding streams leaves the connection ceiling unchanged |
| Write syscalls 84.4/req (≈14.4 KB) | one connection transmitting serially, frame by frame |

**`node_queue` ≈ 0 together with local direct at 161.5 inf/s** is decisive. Had
the node hit its own ceiling (161.5), worker waiting would pile up under c32
load. Instead `node_queue` is 0.02 ms. It processes what it receives
immediately and has room left. The bottleneck is **in front of** the worker
pool, in the transport layer.

> ⚠️ **Do not use `8 workers / inference_us 24.7 ms ≈ 324 inf/s` as node
> capacity.** Local direct with 8 workers on the same board tops out at
> 161.5 inf/s, so the 8 workers do not run independently — there is already
> contention inside the RKNN runtime and the NPU. The reference for starvation
> is **161.5**. The recoverable gap is 116 → 161.5, about 30% — not 116 → 324.

## 5. Interpretation

The −30% per-node loss (116 → 161.5) is not compute, not bandwidth and not the
kernel stack, but **the scheduler↔node HTTP/2 transport path**. Which of flow
control, the connection, or serialization it is within that path **is separated
in S3.6.**

S2's Finding 2 ("the overhead is in the payload-transfer path") holds. S3.5
changes the character of the cost within that path — **it is not a busy cost but
a waiting cost.** The board is 63% idle and the link 49% empty, and throughput
still does not rise.

## 6. Limitations

- **§4.4 is a failure to refute, not a proof.** The other three were excluded
  and every observation is consistent, but it is settled only by changing the
  connection count or window and seeing throughput rise. That check needs code
  changes and falls outside the freeze.
- Single board (king), single condition (c32, 45 s window). There is no 3-node
  profile.
- cycles/req is an approximation without a PMU (note in §2). For comparison
  between conditions, not as an absolute.
- The `local` condition's tool (`sustained_load_test`) is a different program
  from the node. Its latency definition differs (50.2 ms vs `inference_us`
  24.7 ms), so the two must not be subtracted. Only throughput and CPU occupancy
  were compared.
- S3.5b changed only `rps_cpus`. RSS (multiple RX queues) is impossible on the
  single-queue r8125.
- **Only the last of S3.5b's per-run bench JSON files survived.** The script
  cleared the output directory with `rm -f *.json` between runs and deleted the
  earlier raw files with it (since fixed). Throughput and CPU0 %soft survive for
  all six in `raw/results.csv` and `raw/mpstat_*`, so §4.3's conclusion is
  unaffected.

## 7. Implications for S4

**io_uring does not target this bottleneck.** What io_uring reduces is syscall
entry cost and copies. The board's CPU is 63% idle, so making syscalls cheaper
does not raise the ceiling. This falls squarely under the non-applicability
condition TECHSPEC §15.3 records ("improvement under 5% against implementation
complexity").

In order of measured cost, much cheaper means come first.

**This is not cancelling io_uring.** It inserts a final step to confirm whether
this is a problem that warrants a knife of that size.

```text
S2   scaling baseline      DONE
S3   saturation            DONE
S3.5 transport profiling   DONE  <- this document
S3.6 H2 / channel A/B      next  <- splits the cause three ways
       |
     cause established
       |
S4 |- if H2 tuning is the answer -> gRPC optimized
   \- otherwise                  -> io_uring
```

S3.6 separates §4.4's ① and ② with minimal changes, under conditions identical
to 1-node saturation:

| Test | Connections/node | H2 window | Purpose |
|---|---:|---|---|
| A | 1 | default | baseline (= the current 115) |
| B | 1 | greatly enlarged | **test flow control** |
| C | 4 | default | **test the connection/TCP path** |
| D | 4 | enlarged | combined effect |

The interpretation is clean.

- **B alone rises** → the culprit is HTTP/2 flow control, not connection count
- **C alone rises** → the culprit is the single connection / TCP path
- **Both B and C rise** → both contribute
- **Even D unchanged** → the HTTP/2 hypothesis weakens → back to ③ (protobuf,
  copies, syscalls), **and at that point io_uring has a much stronger case**
  (not bandwidth, not CPU placement, not flow control)
  — though "not the scheduler either" was later **withdrawn** in S3.8 (see the
  note in §4.1 above)

The window is not a search for an optimum but only a test of **whether a
64 KB-class default was blocking**. Set it generously large, in the range of
several MB to tens of MB.

If enlarging the window alone takes 115 to 145–155, S4's conclusion changes —
"gRPC is not slow; **the default HTTP/2 settings did not suit a large-payload
workload**". Recovering a substantial share of the 30% with a few lines of
configuration, before writing a transport of several thousand lines, is the
stronger judgement as systems research.

Other means the measurements support:

| Means | Basis |
|---|---|
| **Shrink the response payload** — postprocess on the node and return only detections (1.218 MB → a few KB) | removes half the wire, protobuf and copy load |

## 8. Reproduction

```bash
bash scripts/run-transport-profile.sh              # three conditions (about 5 min)
bash scripts/run-transport-profile.sh --only local # one condition
PYTHONIOENCODING=utf-8 python scripts/analyze-transport-profile.py

bash scripts/run-rps-ab.sh                         # S3.5b (about 10 min)
```

Frozen commit `01f29a2`. `run-rps-ab.sh` changes `rps_cpus` at runtime only and
restores the original value (`00`) at the end.

## 9. Conclusion

The cause of the ~116 inf/s per-node ceiling lies in the **scheduler↔node
HTTP/2 transport path**. Link bandwidth (51% used per direction), board CPU
capacity (63% idle), kernel softirq concentration (RPS A/B −0.2%) and the
server/scheduler (three nodes scaling 3.00× linearly) are all excluded. The same
board yields 161.5 inf/s local direct while managing only 116 in the cluster,
with `node_queue` ≈ 0 showing headroom to spare.

Which of ①flow control ②connection/TCP ③protobuf and copies it is within the
transport path **has not yet been separated.** → **S3.6** splits it with a
minimal-change A/B, and that result fixes S4 as either `gRPC optimized` or
`io_uring` (§7).
