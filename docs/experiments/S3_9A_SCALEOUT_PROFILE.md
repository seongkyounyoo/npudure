# S3.9a — Scale-out Efficiency Loss Profiling

*[한국어 원문](S3_9A_SCALEOUT_PROFILE.ko.md)*

- Experiment ID: **S3.9a**
- Measured: 2026-08-21
- Code: `e1ad9ed` + `[transport] node_connections = 2`
- Status: **complete (9 runs, 3 configurations × 3, node-count verification passed 9/9)**
- Raw data: [`../../results/scaleout-profile-20260821/`](../../results/scaleout-profile-20260821/)
- Predecessor: [`S3_8_OPTIMIZED_SCALEOUT.md`](S3_8_OPTIMIZED_SCALEOUT.md)

---

## 1. Research Question

> **Where in the shared path does the ~4.5% efficiency that optimized 3N loses
> actually go?**

The scope does not widen. **No new sweep is mixed in** — the operating points
S3.8 already established are used as they are: 1N@c12 / 2N@c24 / 3N@c36. All
three run 2 connections per node and c12 per node. **The only thing that
changes is node count.**

## 2. Method

- Profile the server and king **simultaneously**. "Did the server grow or did
  the node shrink" can only be told apart side by side.
- The server has no sysstat (no mpstat, pidstat, sar or perf). It was **not
  installed**; the figures were computed directly from 24-core `/proc/stat`
  deltas, to avoid changing the environment mid-campaign.
- Collected: per-core busy and softirq, NIC RX/TX and drops, IRQ/softirq
  distribution, **per-thread scheduler CPU**, `ss -tin` (rtt, cwnd, retrans),
  per-node distribution.
- Scripts: [`run-scaleout-profile.sh`](../../scripts/run-scaleout-profile.sh),
  [`server-profile-collect.sh`](../../scripts/server-profile-collect.sh),
  [`analyze-scaleout-profile.py`](../../scripts/analyze-scaleout-profile.py).

## 3. ⚠️ First, S3.8's leading candidate is withdrawn

S3.8 named **"the server 10G link has climbed to 76%"** as the leading candidate
for the efficiency drop. **That calculation was wrong.**

```text
what was written   387.2 inf/s x 2,446,800 byte x 8 = 7.58 Gbps -> "76% of 10G"
```

**10GbE is full-duplex.** Requests (TX) and responses (RX) each use their own
10 Gbps. The two must not be summed into one link budget.

| | TX (request) | RX (response) | vs 10G per direction |
|---|---:|---:|---:|
| 1N | 1.34 Gbps | 1.33 Gbps | 13.4% |
| 2N | 2.61 | 2.59 | 26.1% |
| **3N** | **3.84** | **3.80** | **38.4%** |

The measurement (`/proc/net/dev`) agrees: at 3N, RX 3.997 / TX 4.048 Gbps —
**40.5% per direction**. Not 76% but **40%**. **The server 10G link is not the
bottleneck.**

> Earlier in the session full-duplex was reasoned about correctly for the
> boards' 2.5GbE ([S3.5 §4.1](S3_5_TRANSPORT_PROFILE.md)), and then the same
> mistake was made on the server. The same trap gets stepped in again when the
> axis changes.

## 4. Results

Error rate 0, distribution deviation 0.0 pp, node-count verification passed 9/9.

### 4.1 The efficiency loss equals the rise in mean latency, exactly

| N | conc | throughput | **mean** | p50 | p95 | p99 | Efficiency | mean increase |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 12 | 136.2 | 87.63 | **85.9** | 119.7 | 137.9 | **100.0%** | +0.0% |
| 2 | 24 | 265.6 | 89.79 | **86.0** | 136.6 | 169.0 | **97.5%** | **+2.5%** |
| 3 | 36 | 390.3 | 91.55 | **85.9** | 147.4 | 187.6 | **95.5%** | **+4.5%** |

In a closed loop, throughput = concurrency / mean latency, so this agreement is
an identity. What matters is **where that mean grew**.

> **p50 is completely flat — 85.9 / 86.0 / 85.9 ms.**
> The work of handling one request is independent of node count.
>
> **What pulled the mean up is entirely the tail.**
> p95 +23% (119.7 → 147.4), p99 **+36%** (137.9 → 187.6).

The stage breakdown says the same (p50, ms):

| | e2e | inference | →node | →client | payload sum | non-inference |
|---|---:|---:|---:|---:|---:|---:|
| 1N | 79.07 | 32.77 | 22.46 | 22.46 | 44.91 | 46.30 |
| 2N | 77.92 | 30.32 | 22.91 | 22.91 | 45.82 | 47.60 |
| 3N | 76.16 | 29.33 | 22.33 | 22.33 | 44.66 | 46.84 |

**No stage grows with node count.** Scheduler queue and routing are ~0
(0.000–0.004 ms), and node queue is fixed at 0.022 ms.

### 4.2 No server resource is anywhere near saturation

| N | busy cores | busiest core | softirq cores | RX Gbps | TX Gbps | 10G per dir | drop | schedCPU | sysc/req |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 2.81 | 18.4% | 0.51 | 1.390 | 1.408 | 14.1% | **0** | 1.57 | 164.7 |
| 2 | 6.03 | 29.6% | 1.04 | 2.712 | 2.749 | 27.5% | **0** | 3.32 | 164.5 |
| 3 | **10.12** / 24 | **47.6%** | 1.67 | 3.997 | 4.048 | **40.5%** | **0** | 5.62 | 164.2 |

- **CPU**: 10.12 of 24 cores in use (42%). Even the busiest core is at 47.6%.
- **Link**: 40.5% per direction. **0 drops.**
- **No thread serialization**: the top 5 threads at 3N sit at 37/35/34/31/30% —
  evenly spread. There is **no** serialization point where one thread spikes.
- **syscalls/req unchanged**: 164.7 / 164.5 / 164.2.

> As expected the server has 24 RX queues (the boards have one), so RSS spreads
> the load. The "single queue + CPU0 concentration" problem that bit on the
> boards does not exist on the server.

### 4.3 But two things grow with node count

**(a) Server CPU per request is +26%**

```text
1N  2.81 cores / 136.2 inf/s = 20.6 ms.core/req
2N  6.03 / 265.5             = 22.7
3N 10.12 / 390.3             = 25.9      (+26%)
```

Not saturation (42%), but **superlinear** — 3× the nodes for 3.60× the CPU.

**(b) TCP retransmit rate is 3.5× per connection**

Retransmitted-byte ratio per connection, from the raw `ss -tin`:

| | bytes_sent | bytes_retrans | retransmit rate | cwnd | ssthresh |
|---|---:|---:|---:|---:|---:|
| 1N | 3.05 GB | 1.67 MB | **0.055%** | **176** | 138 |
| 3N | 2.95 GB | 5.57 MB | **0.189%** | 118 | 103 |
| 3N | 2.94 GB | 5.93 MB | 0.201% | 119 | 66 |
| 3N | 2.95 GB | 5.33 MB | 0.181% | 106 | 59 |

**Bytes sent per connection are comparable (≈3 GB) while only the retransmit
rate is 3.5×.** And the congestion window is suppressed — cwnd 176 → 106–119,
ssthresh 138 → 59–103.

## 5. Interpretation

Summarised:

```text
efficiency loss  =  rise in mean latency  =  entirely a rise in the tail
                    (p50 completely flat)

server resources    no saturation (CPU 42%, link 40%/direction, 0 drops, threads even)
server per-req cost +26% (superlinear but not saturated)
TCP retransmit rate 3.5x per connection, cwnd suppressed
```

**Leading hypothesis (unverified): congestion on the shared path.** The server
sends out of one 10G port and the boards receive on three 2.5G ports. The
totals are far below the link ceiling, but the **10G → 2.5G speed mismatch**
creates buffering at the switch egress. As nodes are added, the total traffic
crossing the switch fabric rises and TCP responds with retransmits and a reduced
cwnd. Retransmits barely touch the median and **push only the tail up** — which
is exactly the shape of the observed signal (p50 flat, p95/p99 rising).

**But this is consistency, not proof.** The switch-side counters were not read,
and whether retransmits are the cause or another symptom of the same cause was
not separated.

## 6. Candidate status, updated

| Candidate | As of S3.8 | **After S3.9a** |
|---|---|---|
| Server 10G link | leading candidate (76%) | **withdrawn** — arithmetic error. 40% per direction |
| Server CPU saturation | candidate | **excluded** — 42%, busiest core 47.6% (**on the 24-thread host**; see §7) |
| Scheduler serialization | candidate | **excluded** — thread CPU is even |
| Server NIC drops | candidate | **excluded** — 0 drops |
| **Shared-path congestion (10G→2.5G)** | — | **new, leading** — retransmits 3.5×, cwnd suppressed |
| Superlinear server per-req CPU | — | **open** — +26%, but not saturated |

## 7. Limitations

- **The congestion hypothesis is unverified.** Switch counters (per-port drops,
  pause frames, buffers) were not read.
- `ss` retransmit and cwnd figures are **cumulative since the connection was
  established**, not values over a controlled window, so they were used only for
  **ratio comparison** rather than absolute comparison.
- A node-side profile (king) was collected as well, but this document covers
  only the server axis.
- **"Server CPU excluded" is a verdict about that host (added 2026-08-26).**
  Swapping the scheduler host from 24 threads to 8 took server CPU from
  **42% to 82.2%** under the same load and dropped the baseline from
  **391 to 360 inf/s**. The measurements and conclusions in this document hold
  for the 24-thread host and **stand as recorded.** When reproducing on another
  host, watch server CPU utilisation alongside.
  → `../infrastructure.md` §3.2.1
- 60-second measurements — before the throttling region.
- Percentiles are run-level averages (S2 §7.4.1).

## 8. Reproduction

```bash
bash scripts/run-scaleout-profile.sh 3     # 9 runs, about 15 minutes
PYTHONIOENCODING=utf-8 python scripts/analyze-scaleout-profile.py \
    results/scaleout-profile-20260821
```

## 9. Conclusion

**3N's 4.5% efficiency loss is not caused by server resource saturation.** CPU
42%, link 40% per direction, 0 drops, no thread serialization, syscalls/req
unchanged. The "10G at 76%" S3.8 pointed at is withdrawn as **an arithmetic
error that ignored full-duplex**.

The loss is **entirely a rise in the tail**. p50 is completely flat at 85.9 ms
while p95 rises +23% and p99 +36%; the mean rises with them and closed-loop
throughput is cut accordingly. No stage in the breakdown grows.

The accompanying signal is a **3.5× per-connection TCP retransmit rate and a
reduced cwnd**. With the server on one 10G port and the boards on three 2.5G
ports, **shared-path congestion from the speed mismatch** is the leading
hypothesis — but it is **unverified**.

→ Next is **S0 (sustained load)**. Verifying the congestion hypothesis needs
switch counter access and separate preparation. Before that, the question is
**whether the current operating point still holds over 30 minutes** — every
result so far covers a 60-second window.

---

## Figure

![p50 flat (+0%), p95 +23%, p99 +36% - the loss is entirely in the tail](../../results/scaleout-profile-20260821/figures/fig_efficiency_loss_is_tail.png)

**`fig_efficiency_loss_is_tail.png`** — p50 flat (+0%), p95 +23%, p99 +36%; the
loss is entirely in the tail

Regenerate: `python scripts/make-experiment-figures.py`
