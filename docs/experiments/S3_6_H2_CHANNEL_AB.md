# S3.6 — HTTP/2 Window × Connections-per-Node A/B

*[한국어 원문](S3_6_H2_CHANNEL_AB.ko.md)*

- Experiment ID: **S3.6**
- Measured: 2026-08-20
- Code: `11cec9b` + the `[transport]` settings added (defaults behave identically to the freeze)
- Status: **complete (20 runs, 4 conditions × 5 rounds)**
- Raw data: [`../../results/h2-channel-ab-20260820/`](../../results/h2-channel-ab-20260820/)
- Predecessor: [`S3_5_TRANSPORT_PROFILE.md`](S3_5_TRANSPORT_PROFILE.md)
- Successor: **S4** — this result sets the direction (§7)

---

## 1. Research Question

> **The −30% loss S3.5 narrowed to the transport path — what within that path
> causes it?**

S3.5 excluded bandwidth (51% per direction), board CPU capacity (63% idle),
CPU0 softirq concentration (RPS A/B −0.2%) and the server/scheduler (three nodes
scaling 3.00× linearly). What remained was the scheduler↔node HTTP/2 transport
path, with three candidates bundled inside it.

| | Sub-candidate |
|---|---|
| ① | **flow control** — does the 64 KB default window turn a 1.2 MB message into stop-and-wait |
| ② | **connection/TCP** — is one connection state machine and socket a serialization point |
| ③ | **protobuf and copies** — is it framing and encode/decode cost |

**The bare fact of "one connection" must not be taken as ②.** HTTP/2 was
designed precisely to multiplex streams over a single connection. So ① and ②
are varied orthogonally.

## 2. Method

A 2×2. Under the 1-node (king) saturation condition (c32, 60 s), **5 runs** per
condition, 20 runs total.

| Test | Connections/node | H2 window | Purpose |
|---|---:|---|---|
| **A** | 1 | default (64 KB) | baseline |
| **B** | 1 | stream 8 MB / conn 64 MB | test ① |
| **C** | 4 | default | test ② |
| **D** | 4 | 8 MB / 64 MB | combined |

- The window is **not** a search for an optimum. It only tests whether a
  64 KB-class default was blocking. It is sized so that one message (1.23 MB)
  fits whole without a WINDOW_UPDATE.
- Flow control is **advertised by the receiver**. The node sets it for the
  request direction (1.23 MB) and the scheduler for the response direction
  (1.218 MB), so **both sides** were configured.
- Condition order rotates each round, so temperature and elapsed time do not
  land on one condition.
- Each run counts the node's actual TCP connections with `ss` and records it — a
  silently ignored setting would turn the A/B into the same condition four times.
- Scripts: [`run-h2-channel-ab.sh`](../../scripts/run-h2-channel-ab.sh),
  [`analyze-h2-channel-ab.py`](../../scripts/analyze-h2-channel-ab.py).

## 3. Results

Mean ± SD of 5 runs. Error rate **0** throughout.

| Condition | TCP measured | throughput | vs A | E2E p50 | E2E p95 | →node | node_queue |
|---|---:|---:|---:|---:|---:|---:|---:|
| **A** 1ch default | 1 | **115.3 ± 0.8** | — | 269.3 | 392.8 | 115.8 | 0.02 |
| **B** 1ch bigwin | 1 | **73.5 ± 0.4** | **−36.3%** | 480.1 | 596.2 | 204.0 | 0.02 |
| **C** 4ch default | 4 | **140.1 ± 0.3** | **+21.5%** | 163.3 | 572.6 | 60.1 | 0.02 |
| **D** 4ch bigwin | 4 | **139.5 ± 1.1** | +21.0% | 172.7 | 558.6 | 64.2 | 0.02 |

SD of 0.3–1.1 is very small. The measured TCP counts of 1/1/4/4 confirm the
intended conditions actually took effect.

**A at 115.3 reproduces the S2/S3 baseline (115.2 ceiling).** That serves as a
regression check that adding `[transport]` did not change existing behaviour.

### Board profile (king, 5-run average per condition)

| Condition | %usr | %sys | %soft | %idle | CPU0 busy | CPU0 %soft | syscall/req |
|---|---:|---:|---:|---:|---:|---:|---:|
| A | 18.0 | 12.2 | 6.3 | **63.4** | 69.5 | 51.0 | 84.9 |
| B | 13.9 | 8.9 | 4.3 | **73.0** | 51.5 | 35.2 | 93.6 |
| C | 27.8 | 18.9 | 9.4 | **43.9** | **81.1** | **74.4** | 82.4 |
| D | 27.9 | 18.8 | 9.4 | 43.9 | 80.6 | 74.1 | 80.5 |

## 4. Interpretation

### 4.1 ② The single-connection-per-node structure is the primary constraint (+21.5%)

Raising connections alone from 1 to 4 gives **115.3 → 140.1 inf/s**, with the
window left at its default. `network_to_node` halved from 115.8 to 60.1 ms and
board idle fell from 63.4% to 43.9% — **more work actually progresses in the
same time.**

Against local direct (161.5 inf/s):

```text
A  115.3  --  recovered 24.8  -->  C  140.1  --  remaining 21.4  -->  local 161.5
              (54% of the 46.2 gap)              (13.3% left)
```

**One line of configuration recovered more than half the gap.**

> ⚠️ **What this experiment showed goes as far as "connection count is a
> constraint".** What within that is the actual serialization point **has not
> been separated.** At least three remain.

>
> | | Remaining internal candidate |
> |---|---|
> | ②-a | TCP per-flow processing (socket, softirq and congestion control are per flow) |
> | ②-b | HTTP/2 multiplexing / locking and serialization in the connection state machine |
> | ②-c | interaction with flow control (streams sharing the per-connection window) |
>
> The +21.5% reproduced from the single change 1ch → 4ch, so **the claim that
> the single-connection structure is a real constraint is quite strong.** But
> naming which of ②-a/b/c requires a connection-count sweep and per-flow
> instrumentation (§7).

### 4.2 ① A 64 MB-class large window is badly harmful on this workload (−36.3%)

Enlarging the window **dropped throughput by 36%**, and reproducibly (SD 0.4).

Latency is why. E2E p50 269 → 480 ms, `network_to_node` 115.8 → 204.0 ms. In a
closed loop at c32, added latency is throughput loss directly (32 / 0.48 s ≈ 67).
Board idle actually **rises** (63.4% → 73.0%) — it is not doing more work, it is
waiting more.

> **Interpretation (hypothesis).** A 64 MB connection window permits all 32
> concurrent requests (39 MB) to be pushed into the socket at once. HTTP/2
> interleaves DATA frames between streams, so all 32 advance together and
> **finish late together**. The 64 KB default window limited in-flight data and
> was effectively acting as backpressure and pacing, which let earlier requests
> finish first.
>
> This is **a hypothesis supported by the latency breakdown and the rise in
> idle, not a settled fact.** Settling it needs direct measurement of in-flight
> bytes and frame interleaving.

At 4 connections the window effect disappears (D 139.5 ≈ C 140.1). That is
consistent with concurrent streams per connection falling from 32 to 8 and
narrowing the interleaving, but this too was not measured directly.

**Practical conclusion: keep the window at its default (64 KB).** At this size,
at least, it is a loss.

> ⚠️ **What this experiment showed goes as far as "a 64 MB-class large window
> badly degraded performance on this workload".** It is not "window tuning has
> no effect". 64 KB → 64 MB is a **1000× extreme A/B**, and it cannot rule out
> an optimum at intermediate values (256 KB / 1 MB / 4 MB). Not the priority
> now, but left open.

### 4.3 The cost — tail latency gets worse

> ⚠️ **[Corrected 2026-08-20 — S3.7b]** This section's tail conclusion **was
> measured at c32, and c32 is this workload's overload region.** S3.7b fixed the
> operating point at c12 (the lowest concurrency delivering 98% of peak), and
> **at that point going from 1 to 2 connections improves the tail as well —
> throughput +18.8% together with p95 −18.8% / p99 −17.8%.** Not a trade-off but
> a strict Pareto improvement.
>
> The measured values below remain valid. What they measure, however, is not
> "which configuration is better" but **"which configuration degrades more
> gracefully under overload"**.
> → [`S3_7_CONNECTION_TUNING.md`](S3_7_CONNECTION_TUNING.md) §4.3

| | A | C |
|---|---:|---:|
| E2E p50 | 269.3 | **163.3** (−39%) |
| E2E p95 | **392.8** | 572.6 (**+46%**) |

Throughput and p50 improve while **p95 gets 46% worse.** The average request got
much faster and some requests got much slower.

**This must not be pinned on round-robin.** Several causes are possible.

| | Candidate |
|---|---|
| a | in-flight imbalance across connections (round-robin does not look at load) |
| b | queue variance inside the HTTP/2 connection |
| c | bursty arrivals at the NPU workers |
| d | transport queueing |
| e | the general growth of tail queueing that comes with higher throughput |

All unverified. This trade-off must not be hidden — the tail is an important
metric for real-time inference, and **it is the next experiment's research
question, not a footnote.** S3.7's connection sweep decides the optimum by
**looking at p95 and p99 alongside throughput** (§7).

### 4.4 The next bottleneck has surfaced — and it explains S3.5b's null

In C and D, **CPU0 busy is 81.1%, of which 74.4% is softirq.** Other cores have
headroom while CPU0 alone approaches saturation. eth0 has one RX queue and RPS
is off.

Why RPS was ineffective in S3.5b becomes clear here — **RPS distributes by flow
hash, and at that time there was only one flow.** Now there are four. So
repeating S3.5b on top of condition C may give a different result (§7).

## 5. Verdicts

| Candidate | Verdict |
|---|---|
| ① flow control | **Enlarging to 64 MB is harmful at −36.3%.** The 64 KB default was functioning as backpressure. Intermediate values are unmeasured, so this is not concluded as "tuning is ineffective" |
| ② connection/TCP | **The single-connection-per-node structure is the primary constraint.** 1 → 4 gives +21.5%, recovering 54% of the gap. Which of ②-a/b/c remains unseparated |
| ③ protobuf and copies | May lie within the remaining 13.3%. Still unseparated |

## 6. Limitations

- **There is no basis for 4 being optimal.** Only 1 and 4 were compared. 2/8/16
  are unmeasured.
- **This is a 1-node result.** At three nodes the server would hold 12
  connections. Updating the S2/S3 numbers requires re-measuring at multiple
  nodes.
- **§4.2's bufferbloat explanation is a hypothesis.** It is consistent with the
  latency breakdown and the rise in idle, but in-flight bytes were not measured
  directly.
- The window was sampled at one point, 8 MB / 64 MB. Intermediate sizes (e.g.
  1–2 MB) are unmeasured, so "bigger is worse" cannot be generalised as a
  monotonic relation.
- **The cause of the p95 degradation is unverified.** There are at least five
  candidates (§4.3) and none was excluded.
- The scheduler and node are restarted for each condition. The effect of
  restarting itself is indirectly excluded by A reproducing the baseline.

## 7. Implications for S4

**io_uring is still not justified.** syscall/req barely moves across the four
conditions (80.5–93.6) while throughput differs twofold (73.5–140.1) — **syscall
count does not explain the current primary bottleneck.**

> This does not mean "io_uring has no effect". **Syscall count being equal and
> CPU time spent on syscalls and copies being small are different questions.**
> What can be said now is a matter of **order** — cheaper bottlenecks remain, so
> io_uring is pushed back.

The roadmap updates to:

```text
S3.5  transport profiling   DONE  narrowed to the transport path
S3.6  H2/channel A/B        DONE  <- the single-connection structure is the primary constraint
        |
S3.7  1. connection sweep (1/2/4/8/16) -> optimal N
      2. retry RPS on top of that N                              <- next
        |
optimized gRPC baseline (re-measure 1N/2N/3N)
        |
analyse the remaining gap
        |
io_uring if needed
```

**The optimum is decided by the throughput–tail-latency trade-off, not by
maximum throughput.** If `4ch = 140 inf/s, p95 573` and `8ch = 148 inf/s,
p95 900`, 8ch cannot be called the better system. Nor is more unconditionally
better — past some point, connection management cost and queueing bend the curve
back down.

Why retrying RPS is especially worthwhile: at one connection there was one flow
and nothing to divide, but now there are several flows. And CPU0 is at busy 81%
/ soft 74%.

- If it rises → the narrative holds: "releasing the single-connection constraint
  exposed a NIC processing bottleneck, and RPS only has an effect once there are
  multiple flows".
- If again nothing changes → CPU0 softirq can be excluded more strongly as
  **merely a correlation and not a throughput limiter**.

Why S3.7 is cheap: connection count is already a setting, and RPS is zero lines
of code. Only after clearing those two does ③ remain in pure form.

> **The S2/S3 numbers are not updated yet.** 140.1 is a single-node optimization
> result, and at three nodes the server would hold 4 × 3 = 12 connections, which
> may surface a new server-side bottleneck. Once S3.7 fixes N, 1N/2N/3N will be
> re-run.

## 8. Reproduction

```bash
bash scripts/run-h2-channel-ab.sh 5     # 20 runs, about 35 minutes
PYTHONIOENCODING=utf-8 python scripts/analyze-h2-channel-ab.py \
    results/h2-channel-ab-20260820/raw/results.csv
```

The script restores the default settings (behaving identically to the freeze) at
the end. The frozen binaries remain as
`npuforge-{scheduler,node}.frozen-01f29a2`.

> The node needs `--features rknn` and `RKNN_SDK_PATH=/usr/include`. Omitting
> them produces a mock-backend binary that fails to start (this did happen once,
> and the harness failed loudly so it was caught immediately).

## 9. Conclusion

**The single gRPC/HTTP2 connection per node was confirmed as the primary factor
limiting throughput.** Raising connections per node to 4 gave
**115.3 → 140.1 inf/s (+21.5%)**, recovering **54% of the 46.2 gap to local
direct through configuration alone.** That came from one connection pool, with
no rewrite of the code architecture. The cost is a 46% worse p95.

Whether, within that structure, it is TCP per-flow processing, H2
multiplexing/locking, or the flow-control interaction **has not been separated**
(§4.1).

> ⚠️ **[Correction]** The "cost is a 46% worse p95" below was measured at c32,
> the overload region. At the c12 operating point, 1 → 2 connections improves
> the tail as well (S3.7b §4.3).

**A 64 MB-class large window was badly harmful on this workload at −36.3%**,
meaning the 64 KB default was functioning as backpressure. The default is kept,
though since this was a 1000× extreme A/B the possibility of an optimum at
intermediate values is left open.

The remaining 13.3% and the newly surfaced CPU0 saturation (busy 81%, soft 74%)
are handled in S3.7. io_uring still lacks a basis — syscall/req is nearly
invariant across conditions while throughput differs twofold.

---

## Figure

![Connections help; enlarging the window hurts](../../results/h2-channel-ab-20260820/figures/fig_h2_window_vs_conns.png)

**`fig_h2_window_vs_conns.png`** — connections help; enlarging the window hurts

Regenerate: `python scripts/make-experiment-figures.py`
