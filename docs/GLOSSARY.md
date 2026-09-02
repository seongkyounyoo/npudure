# Glossary

*[한국어 원문](GLOSSARY.ko.md)*

- Last updated: **2026-08-21**
- Scope: every term that actually appears in the S2–S4 experiment lineage. Not
  just definitions — **what value or judgement each is tied to in this project**
  is written alongside.
- Related: [`experiments/README.md`](experiments/README.md) (the experiment
  ledger), [`01-TECHSPEC.md`](01-TECHSPEC.md), [`RESULTS.md`](RESULTS.md)

---

## 1. Experiment ID scheme

| ID | Question | Result summary |
|---|---|---|
| **S0-A** | Does the operating point hold under fanless sustained load | degradation 11.3%, CPU 2208→816 MHz |
| **S0-B** | Sustained load with active cooling | degradation 1.9%, 0 clock downgrades |
| **S0-C** | Do load-aware policies recover the thermal heterogeneity loss | 1st found the herding bug → 2nd and 3rd confirmed recovery → 4th missed the gate |
| **S0-D** | Can heterogeneity be produced deterministically | Yes. Clock caps give spreads of 1.12–3.93× |
| **S2** | Does adding nodes scale linearly | 112.9 / 229.0 / 338.4, 3.00× |
| **S3** | What is each configuration's true ceiling | 115.2 / 232.0 / 341.8 |
| **S3.5** | Where does the −30% loss come from | narrowed to the transport path |
| **S3.5b** | Is CPU0 softirq concentration the cause | null (−0.2%) |
| **S3.6** | Flow control or connections | connections. Enlarging the window backfires |
| **S3.7a** | How many connections is optimal (fixed load) | knee at c4 |
| **S3.7b** | What is each configuration's **operating point** | c12 for all three. conn2 comes out ahead |
| **S3.7c** | Does RPS help at the operating point | null (−0.8%) |
| **S3.8** | Does the optimization hurt scale-out | 387.2 inf/s, eff 95.3% |
| **S3.9a** | Where does the 3N efficiency loss come from | server resources excluded, tail rises |
| **S3.9b** | The remaining node-side cost | syscalls are ~1%. User time exceeds kernel time |
| **S4** | Is io_uring needed | **No — refuted by measurement** (S3.9b) |

> **Naming rule** — integers (S2, S3) are experiments planned from the start.
> Decimals (S3.5, S3.7a) are experiments the measurements newly demanded. It is
> also a record that **the data, not the plan, decided the next experiment.**

---

## 2. Measurement methodology

| Term | Meaning | In this project |
|---|---|---|
| **closed-loop** | a load model holding concurrency fixed, sending the next request only after a response arrives | the bench works this way. Absolute latency must not be quoted as an SLA and is used **only for comparison between configurations** |
| **open-loop** | a model that keeps sending at a set arrival rate regardless of responses | not used |
| **coordinated omission** | in a closed loop, a slowing system produces fewer requests so **latency is under-measured** | flagged as a warning in the bench `--help` |
| **Little's law** | `concurrency = throughput x mean latency` | used in S3.9a to show the efficiency loss **matches the rise in mean latency exactly** |
| **saturation / ceiling** | the limit beyond which more load does not raise throughput | measured per configuration by S3 |
| **operating point** | the load point at which it would actually be run | defined as **the lowest concurrency delivering at least 98% of peak** (a code constant) |
| **concurrency knee** | the number of concurrent requests needed to saturate the device | **c12/node** within the tested range, observed independently of connection count |
| **connection knee** | the optimum for how many connections to split those requests across | c4 at fixed load, **conn2** by operating point |
| **overload region** | past saturation. Throughput is flat and only latency rises | the whole c24–c64 range. **Comparing configurations here inverts the conclusion** |
| **short-run / sustained operating point** | the operating point on a 60-second basis / on a thermal steady-state basis | identical under active cooling (−1.9%), divergent when fanless (−11.3%) |
| **steady-state** | the region where values no longer change with time | S0's definition: **the mean of the last third** |
| **degradation** | `1 − steady / peak` | verdicts: <3% none / 3–10% slight / >10% pronounced |
| **scaling / efficiency** | `tp_N / tp_1` / that divided by N | optimized 3N: 2.86× / 95.3% |
| **rotation** | changing the condition order each repetition to cancel time and temperature drift | applied in every A/B harness |
| **preheat / reheat** | matching the thermal state with load before measuring | on the S0-C re-run, **per policy** |
| **freeze** | not changing code, configuration or model during measurement | binaries preserved as `*.frozen-<commit>` |
| **verdict** | the bench's own judgement of run validity | `valid` + `reasons`. Anomalous runs are not deleted |
| **preflight** | a hard-failing check immediately before measuring | alias↔hostname, hashes, governor, temperature, voltage, **inference accuracy** |
| **probe bench** | a short load thrown before the real measurement to confirm conditions | used for node-count verification — it filtered out 6 configurations in S3.8 |
| **capacity heterogeneity** | the spread in processing capability between nodes | thermally induced (S0-A/C) and **clock-cap induced** (S0-D) are recorded separately. What the scheduler sees is capacity, not its cause |
| **heterogeneity gauge** | the observed quantity used to measure the spread | **round-robin's per-node p50 max/min.** RR does not adapt, so the raw capacity spread shows through directly under even load |
| **utime / stime** | user / kernel CPU time | from `/proc/PID/stat`. Kernel holds syscall entry, the TCP stack and `copy_to_user`; user holds serialization, user-space copies and HTTP/2 framing. **What io_uring reduces is a portion of stime** |
| **one-directional test** | a measurement whose bias direction is known | `strace -c` **inflates** values through ptrace → if the inflated value is small, the real one is conclusively smaller |

### 2.1 Percentile aggregation

| Term | Meaning |
|---|---|
| **nearest-rank** | no interpolation: "sort, then the first value at or past that point". What the bench uses |
| **run-level percentile** | a percentile computed within one run over that run's requests |
| **pooled percentile** | a percentile computed after **combining all runs'** requests |
| **caveat** | every table in this repository is **the average of run-level values**, not pooled. Run-level averaging dilutes each run's worst window and **makes the tail read low**. Valid for comparing conditions, but the absolute values must not be quoted as "this system's p99" |

---

## 3. Performance metrics

| Term | Meaning |
|---|---|
| **inf/s** | inferences per second (throughput) |
| **p50 / p95 / p99 / max** | percentiles of the latency distribution. p50 = median |
| **tail latency** | high-percentile (p95/p99) latency. "How late do some requests get" |
| **tail amplification** | the tail worsening more than throughput improves |
| **balance (pp)** | request distribution deviation between nodes. 0 is perfectly even |
| **error_rate** | failure ratio. **0** across every experiment in this repository |
| **TimingBreakdown** | the 11-stage time breakdown carried in the response (proto `Timing`) |
| `scheduler_queue` | waiting inside the scheduler |
| `scheduler_route` | policy selection time |
| `network_to_node` / `network_to_client` | **the full round trip minus node-internal time**, split in half. A method for avoiding subtraction of absolute clocks on different machines |
| `node_queue` | waiting on the node's worker pool |
| `decode / preprocess / npu_input / inference / postprocess` | node-internal stages. This project uses raw RGB input and raw tensor output, so everything except inference is ~0 |
| `end_to_end` | the total as measured by the scheduler |
| **syscalls/req · ctx switches/req · cycles/req** | the io_uring decision metrics TECHSPEC §15.4 requires |

---

## 4. Network and kernel

| Term | Meaning | In this project |
|---|---|---|
| **full-duplex** | send and receive each having their own bandwidth | **the two directions must not be summed into one link budget.** S3.8 made this mistake and wrote "10G 76%", withdrawn in S3.9a (the real figure is 40% per direction) |
| **goodput** | actual payload throughput excluding headers | what iperf3 measures. Board link measured at 2.34 Gbps |
| **MTU** | the maximum payload in one frame. 1500 here | |
| **RSS** (Receive Side Scaling) | the NIC spreading packets across cores via **hardware multi-queue** | the server NIC has 24 RX queues. The boards have **one** |
| **RPS** (Receive Packet Steering) | the kernel spreading **in software** by flow hash | attempted on the boards, null both times. **With one flow there is nothing to divide** |
| **softirq / NET_RX** | the lightweight context in which the kernel does post-interrupt work. Network receive processing runs here | board CPU0 %soft 51.5% → because of the single queue |
| **IRQ affinity** | which core takes an interrupt | on the boards the NIC IRQ is pinned to CPU0 |
| **cwnd / ssthresh** | TCP congestion window / slow-start threshold | at 3N, cwnd is suppressed from 176 to 106–119 |
| **retransmission** | segments resent due to loss or delay | per-connection retransmit rate 0.055% → **0.19%** (3.5×) |
| **incast / speed mismatch** | buffering and loss at the switch egress when traffic funnels from a fast link (10G) to a slow one (2.5G) | the **leading (unverified) hypothesis** for the 3N efficiency loss |
| **bufferbloat** | excessive buffering inflating latency | the interpretive hypothesis for the −36.3% from enlarging the window to 64 MB |
| **/proc/stat · /proc/net/dev · /proc/interrupts · /proc/softirqs** | counters the kernel exposes | the server has no sysstat, so figures were computed directly from their deltas |
| **ss -tin** | per-socket TCP state (rtt, cwnd, retrans, bytes_sent) | observing connection count and congestion state |

---

## 5. HTTP/2 and gRPC

| Term | Meaning | In this project |
|---|---|---|
| **HTTP/2 multiplexing** | carrying several streams concurrently over one TCP connection | which is why "one connection" alone does not establish a bottleneck |
| **stream** | one logical request/response pair inside a connection | 1 request = 1 stream |
| **flow control window** | the size a receiver advertises as "I can take this much". There are separate **per-stream** and **per-connection** windows | h2 defaults to 65,535 bytes. This project's messages are 1.2 MB |
| **WINDOW_UPDATE** | the frame that reopens the window | with a small window this round trip repeats and becomes stop-and-wait |
| **DATA frame** | the frame carrying the actual payload | the 1.218 MB response is split into roughly 14.4 KB pieces (84.4 write syscalls per request) |
| **head-of-line blocking** | everything behind being blocked by what is in front | when multiplexed streams contend for the same connection resources |
| **tonic** | the Rust gRPC implementation (built on hyper + h2) | v0.12.3 |
| **h2 / hyper** | the HTTP/2 protocol / HTTP library | h2 0.4.15, hyper 1.11.0 |
| **prost** | the Protocol Buffers code generator | `.proto` → Rust types |
| **protoc** | the protobuf compiler | a build prerequisite. Absent on the Windows development PC, so builds happen on the server and boards |
| **`node_connections`** | gRPC connections **per node** (a setting this project added) | 1N→2, 2N→4, 3N→6 total. **Not a cluster-wide sum** |

---

## 6. Scheduling

| Term | Meaning | In this project |
|---|---|---|
| **round-robin (RR)** | assigning in order without looking at state | the baseline. Structurally even, but **it sends the same amount to a slowed node** |
| **least-queue / LOR** (least-outstanding-requests) | choosing the node with the fewest outstanding requests | it does not know service **speed**. Under a simultaneous burst, even distribution is correct behaviour |
| **ECT** (Estimated Completion Time) | estimating the completion time as `(outstanding+1) x EWMA_inference + EWMA_network + penalties` | the only policy able to reflect service-speed differences. But **the EWMAs have to be populated** for it to work |
| **EWMA** | exponentially weighted moving average | tracking inference time and network round trip |
| **herding (herd behaviour)** | several decision-makers looking at **the same stale information** and making the same choice simultaneously | the cause in S0-C. Throughput collapsed 55–58% |
| **stale state / state freshness** | how out of date the state information is | heartbeat 1 s vs dispatch ~3 ms → **hundreds of times apart** |
| **control-loop sampling problem** | control failing because the feedback period is longer than the system's rate of change | the general form of herding. Not a policy tuning problem |
| **reservation** | marking load as occupied at the moment of selection | handled in one critical section by `select_and_reserve()` |
| **RAII guard** | a pattern where a value cleans up automatically on leaving scope | `Reservation`'s `Drop` decrements it — closing **every path**: success, error, timeout, cancellation and retry |
| **`local_in_flight`** | the count of requests the scheduler has sent but that have not finished (updated immediately) | the policy's **primary signal**. Not added to the heartbeat value (that would count the same request twice) |
| **`health.in_flight` / `queue_depth`** | observed values the node carries in its heartbeat (up to 1 s stale) | used only for health verdicts and tie-breaking |
| **busy_queue_depth / degraded / disable temperature** | thresholds for classifying node state | 8 / 80 °C / 90 °C |
| **drain** | the state of sending no new requests and emptying the queue | an operator-specified state |

---

## 7. Hardware and thermal

| Term | Meaning | In this project |
|---|---|---|
| **RK3576** | a Rockchip SoC. 4×Cortex-A72 + 4×Cortex-A53, 2-core NPU | the three node boards |
| **big.LITTLE** | a configuration mixing high-performance and low-power cores | A72 2208 MHz (policy4), A53 2016 MHz (policy0) |
| **cpufreq governor** | the CPU frequency policy | fixed to `performance` (holds the maximum clock). The alternative is `ondemand` |
| **devfreq** | frequency management for non-CPU devices (NPU, GPU, DDR) | NPU 300–950 MHz |
| **thermal zone** | a temperature sensor the kernel exposes | six: soc / bigcore / little-core / ddr / **npu** / gpu |
| **thermal throttling** | lowering the clock because of temperature | **the NPU never dropped once.** What drops is the CPU |
| **thermal steady-state** | the temperature plateau where heat generation and dissipation balance | 58–61 °C under active cooling (within 5 min), 86–88 °C fanless |
| **thermal heterogeneity** | identical board models diverging in performance because their thermal conditions differ | fanless: king 816 / jack 1200 / queen 1416 MHz |
| **boot_id** | an identifier that changes on every boot | detects a board reset mid-run → that measurement is void |
| **input voltage monitoring** | early warning of insufficient adapter capacity | preflight fails below 5.00 V |
| **2.5GbE / 10GbE** | link speeds | boards 2.5G, server 10G. **The speed mismatch is §4's incast hypothesis** |

---

## 8. Model and NPU runtime

| Term | Meaning | In this project |
|---|---|---|
| **RKNN** | the Rockchip NPU runtime | `librknnrt.so` 2.3.0 |
| **RKNPU driver** | the kernel driver | v0.9.8 |
| **YOLOv8n** | the object detection model | input 640×640×3 |
| **INT8 quantization** | weights and activations as 8-bit integers | +17.3% throughput against FP16 |
| **`want_float`** | whether to receive the output dequantized to float | **0** (integers as-is). Output size a quarter, throughput +17.3% |
| **blob v2** | our own serialization format holding several tensors | a 36-byte header per tensor carrying `scale` and `zero_point` |
| **payload size** | | request **1,228,800 B**, response **1,218,000 B** (2,446,800 B per inference combined) |
| **postprocess (DFL + NMS)** | decoding detections and removing duplicates | **currently not done on the node.** Raw tensors are sent as-is → a 1.2 MB response. Doing it on the node would shrink it to a few KB (an unimplemented idea) |
| **warmup** | preheating to exclude the first inference's initialisation cost | excluded from aggregation |
| **worker_count** | the node's number of concurrent inference workers | 8. **The workers are not independent** — local direct with 8 workers reaches 161.5 inf/s |

---

## 9. Software stack

| Term | Meaning |
|---|---|
| **tokio** | the Rust async runtime. The node uses multi_thread (workers = 8 cores) |
| **`spawn_blocking`** | the tokio API that moves blocking work to a separate thread pool. RKNN FFI calls run here |
| **async worker vs blocking pool** | network and protobuf on 8 async workers, inference on the blocking pool — **sharing the same 8 cores** |
| **`parking_lot`** | a faster Mutex/RwLock implementation |
| **`Arc<AtomicU32>`** | an atomic counter shared between threads |
| **`Bytes`** | a reference-counted byte buffer (shared without copying) |
| **`to_vec()`** | a call that creates a copy. One of the candidates for the remaining gap |
| **feature flag** | compile-time feature selection. The node needs `--features rknn` (without it the Mock backend gets built) |
| **`RKNN_SDK_PATH`** | the location of `rknn_api.h` at build time |

---

## 10. Diagnostic tools

| Tool | Purpose | Note |
|---|---|---|
| **iperf3** | measuring link bandwidth | board→server 2.34 Gbps |
| **mpstat** | per-core CPU breakdown (%usr/%sys/%soft/%idle) | on the boards only |
| **pidstat** | per-process and per-thread CPU | on the boards only |
| **ethtool** | link speed, NIC statistics, offload settings | on both |
| **ss** | socket state | connection count and TCP internal state |
| **perf** | PMU-based profiling | **on neither.** cycles/req is an approximation |
| **`/proc` deltas** | aggregating CPU, network and syscalls without sysstat | used for the server profile |
| **thermal-logger.sh** | a 1-second sampler of board temperature, frequency and voltage | |

---

## 11. This project's components

| Name | Role |
|---|---|
| `npuforge-scheduler` | the central scheduler. Distributes client requests to nodes (x86_64, on the server) |
| `npuforge-node` | the node agent. Performs NPU inference (aarch64, on the three boards) |
| `npuforge-bench` | load generation, aggregation and run-validity judgement |
| `npuforge-proto` | the single source for `.proto` |
| `npuforge-rknn` | the RKNN backend |
| `npuforge-mock-backend` | for development and testing without hardware |
| `npuforge-common` | types, error codes, configuration and the backend interface |
| **king / queen / jack** | the names of the three node boards (SSH aliases `npuforge-k/q/j`) |
| **server** | the scheduler and bench host (`npuforge-server`) |

### 11.1 Error codes

| Code | Meaning |
|---|---|
| `NPF-0000` | success |
| `NPF-1002` | payload size exceeded |
| `NPF-1303` | node overloaded (queue full) |
| `NodeUnavailable` | transmission failure → reflected in the health counters |
| `NoAvailableNode` | no node able to handle it |

---

## 12. Experimental rules fixed by this project

Values fixed before measuring and **not changed to fit the results.**

| Rule | Value | Basis |
|---|---|---|
| operating concurrency | the lowest concurrency delivering at least **98%** of peak | 99% overlaps the run-to-run SD (±1 inf/s) |
| steady-state | the mean of the last **third** | |
| degradation verdict | <3% / 3–10% / >10% | |
| Selected operating point | the lowest p95 among those within **97%** of maximum throughput | an **engineering heuristic**, not a statistical optimum |
| policy shift verdict | a distribution shift of **3 pp** or more = a shift; throughput of **2%** or more = recovery | |
| strong heterogeneity gate | RR per-node p50 max/min **≥ 2.0×** | between S0-A's 2.4× and S0-C 2nd's 1.33× (S0-C §17.2) |
| LQ vs ECT decision bands | throughput **2%**, p99 **5%** | at n=4, anything smaller is unusable (S0-C §17.3) |
| incumbent tie-break | if the band is not cleared, **keep the existing default** | unseating an incumbent requires positive grounds |

---

## 13. Phrases that came out of the methodology lessons

| Phrase | Meaning |
|---|---|
| **"exclusions are conditional"** | a bottleneck candidate once excluded reopens when conditions change. A verdict has to carry **under what conditions** |
| **"Optimize at the operating point, not in the overload region"** | comparing configurations in the overload region shows overload behaviour rather than a configuration effect |
| **"turn silent failures loud"** | the harness simply stops when a condition is not met. Node-count verification, configuration-injection verification, evidence of the TCP connection count |
| **"a process being up ≠ receiving traffic"** | node count is confirmed from the probe bench's **distribution of responding node IDs** |
| **"two measurements agreeing does not mean the interpretation is right"** | if both share a bias, reproducibility only confirms the bias |
| **"when performance looks wrong, ask first whether the implementation is doing what it was meant to"** | 55% is not the size of a quality difference |
| **"do not multiply two quantities"** | a throughput-loss % and a share-of-latency % are different axes |
| **"a cost, not a constraint"** | reducing usage (CPU-ms/req) of an unsaturated resource does not raise throughput. The heart of the S4 verdict |
| **"your instrument may be measuring a different quantity"** | when the output differs from expectation, **suspect the instrument first.** Moving a threshold and fixing an instrument are different acts |
| **"do not trust 'I stopped it' — verify at the shared resource"** | local process observation lies depending on the platform. Whether the cluster is free is a question **for the cluster** |
| **"hand-maintained derived numbers diverge"** | run totals and percentages are counted by scripts, with the source recorded |
