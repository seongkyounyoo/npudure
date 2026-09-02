# NPUDure Hardware Setup Guide

*[한국어 원문](02-HARDWARE-SETUP.ko.md)*

- Document: `02-HARDWARE-SETUP.md`
- Project: NPUDure
- Document version: v0.2
- Target release: NPUDure v0.1
- Target talk: FOSS for All Conference, November 2026
- Written: 2026-08-05
- Last modified: 2026-08-06
- Status: Draft
- Related documents:
  - `00-PRD.md`
  - `01-TECHSPEC.md`
  - `03-DEVELOPMENT-REQUIREMENTS.md`
  - `environment-matrix.md`

This document is normative for physical setup, network, power, cooling and
experimental conditions. Where values in those areas differ from another
document, this one wins.

---

# 1. Recommended configuration

All three NanoPi R76S are configured as identical NPU Workers.

The central scheduler, dashboard and benchmark client run on **a separate
server.** **It has to have a PCIe slot** — a 10G NIC goes in it (§3.3.2).

```text
              +----------------------------+
              | Benchmark / Scheduler      |
              | Server (PCIe slot required)|
              |                            |
              | . NPUDure Scheduler        |
              | . Benchmark Client         |
              | . Dashboard                |
              | . Prometheus               |
              +-------------+--------------+
                            | 10GbE (SFP+ DAC)   <- aggregation
                  +---------v----------+
                  | 2.5G / 10G Switch  |
                  +----+-----+-----+---+
                       |2.5G |2.5G |2.5G
               +-------v+ +--v----+ +v------+
               | KING   | | QUEEN | | JACK  |
               | Worker | | Worker| | Worker|
               | 6 TOPS | | 6 TOPS| | 6 TOPS|
               +--------+ +-------+ +-------+
```

**Worker links are 2.5G and only aggregation is 10G.** At most 1.545 Gbps per
node means 2.5G suffices, and the scheduler end where the three converge becomes
the bottleneck. The basis is §3.3.2.

Core principles:

```text
3 NanoPi = identical Workers
a separate Linux PC = Scheduler
2.5GbE = the inference network
identical OS, kernel, RKNN, model
independent power
identical cooling
all benchmarks recorded centrally
```

---

# 2. Roles by machine

| Machine | Role | Note |
|---|---|---|
| Linux PC | Scheduler | receiving requests and selecting nodes |
| Linux PC | Benchmark Client | generating load and storing results |
| Linux PC | Dashboard | live throughput and failure display |
| Linux PC | Metrics Server | Prometheus and optionally Grafana |
| KING | NPU Worker | RKNN inference |
| QUEEN | NPU Worker | RKNN inference |
| JACK | NPU Worker | RKNN inference |

## 2.1 Why all three NanoPi are Workers

Running the scheduler and an NPU worker together on one NanoPi causes the
following.

- CPU and network load rise on that node alone.
- The three nodes' experimental conditions differ.
- The 1-node, 2-node and 3-node comparison can be distorted.
- Separating a scheduler bottleneck from an NPU bottleneck becomes hard.
- It becomes hard to describe it as a like-for-like comparison in a talk.

So official benchmarks keep the three as completely symmetric workers.

For simple development or a portable demo the scheduler may run on KING, but not
for official performance figures.

---

# 3. Network configuration

## 3.1 Recommended topology

A star topology centred on a **2.5G/10G switch.** Workers are 2.5G and only the
scheduler uplink is 10G (§3.3.2).

Required equipment:

- **One 2.5G/10G switch** — ≥ 4 × 2.5G ports + an SFP+ uplink
- **One scheduler server** — a PCIe slot is required
- **One 10G NIC (PCIe, SFP+)** — e.g. Intel X520
- **One SFP+ DAC cable** — server ↔ switch
- Three CAT5e cables — switch ↔ nodes (Cat5e suffices for 2.5G)
- Three NanoPi R76S

```text
Linux PC ---+
KING ------+
QUEEN ------+-- 2.5GbE Switch
JACK ------+
```

## 3.2 IP address plan

An example of the dedicated NPUDure inference network:

```text
Network     : 10.20.0.0/24
Scheduler   : 10.20.0.10
KING        : 10.20.0.21
QUEEN       : 10.20.0.22
JACK        : 10.20.0.23
```

Hostnames:

```text
npuforge-scheduler
npuforge-king
npuforge-queen
npuforge-jack
```

An `/etc/hosts` example:

```text
10.20.0.10  npuforge-scheduler
10.20.0.21  npuforge-king
10.20.0.22  npuforge-queen
10.20.0.23  npuforge-jack
```

## 3.3 Separating the management network (required)

The NanoPi R76S has **two 2.5GbE ports.** Separating the management network is
therefore possible at no extra cost, and is treated as the default rather than
an option.

```text
port 1 -> the existing office/home network   = management network
           SSH, apt, binary deployment, log collection

port 2 -> the dedicated switch               = inference network
           inference traffic only. Nothing else
```

The management network is separated not for convenience but to **prevent
measurement contamination.** SSH sessions, `apt` downloads and log transfers
mixed into the inference network produce unexplained spikes in the network
latency figures.

Port names differ by board and kernel, so always check.

```bash
ip -br a
for n in /sys/class/net/e*; do
  echo "$(basename $n): speed=$(cat $n/speed 2>/dev/null) mac=$(cat $n/address)"
done
```

Which port is used for the inference network **has to be the same on all three
nodes.** It is recorded per node in `environment-matrix.md` §8.

## 3.3.1 Staged build-out

Development proceeds before the inference-network switch arrives.

| Stage | Management network | Inference network | Possible work |
|---|---|---|---|
| Current | the existing 1G hub | none | board setup, RKNN validation, single node |
| Interim | the existing 1G hub | shared 1G hub | all of M2–M5 (gRPC, 3 nodes, failure recovery) |
| Final | the existing 1G hub | dedicated 2.5GbE switch | official benchmarks |

**Everything up to M5 can be developed on the interim stage.** Link speed does
not affect functional correctness; 2.5GbE is needed only for official
performance figures.

On JPEG input, 1GbE has sufficient bandwidth (see the rationale calculation in
§3.1). Only the raw RGB input scenario saturates 1GbE first.

## 3.3.2 The scheduler host's link speed — **10G required** (revised 2026-08-12)

The scheduler host is where all three nodes' traffic converges. Calculating once
measured throughput was available confirmed that **2.5GbE is insufficient.**

One raw RGB input is `640 × 640 × 3 = 1,228,800 byte`.

```text
                    per node         3-node total
INT8  157.2 inf/s   1.545 Gbps       4.636 Gbps
FP16   84.3 inf/s   0.829 Gbps       2.486 Gbps
```

**Even FP16's three-node total of 2.486 Gbps exceeds a single 2.5GbE link
(effectively about 2.35 Gbps).** INT8 exceeds it by nearly double at 4.636 Gbps.

That is, **it is the aggregation link, not the worker links, that fills up
first.**

### The revised topology

```text
        Benchmark / Scheduler Server
                    |
                  10GbE          <- aggregation. This is the point
                    |
            2.5G / 10G Switch
              |-- 2.5G -- king
              |-- 2.5G -- queen
              \-- 2.5G -- jack
```

- **The worker links stay at 2.5G.** At most 1.545 Gbps per node, which suffices
- **Only the aggregation link** is raised to 10G
- The switch is a model with 2.5G ports plus a 10G (SFP+) uplink

### What is needed

| Item | Specification | Note |
|---|---|---|
| Scheduler host | **a server with a PCIe slot** | a laptop cannot take a 10G card |
| ┗ CPU | **16 threads or more recommended** | measured 2026-08-26. §3.3.4 |
| 10G NIC | PCIe, SFP+ (e.g. Intel X520) | |
| DAC cable | SFP+ Direct Attach | cheaper than optics and suited to short runs |
| Switch | ≥ 4 × 2.5G ports + an SFP+ uplink | |

> **The previous version (obtain a 2.5GbE NIC) is discarded.** That calculation
> predated per-node measured throughput. The principle of not using values
> measured without the required equipment as official figures stands.

## 3.3.3 Measured network values to record before M3

These are **measurements**, not calculations. Taken from interface counters on
`dealer` (or the new scheduler server).

| Condition | What to record |
|---|---|
| A single request | scheduler TX bytes / RX bytes |
| 1-node saturation | TX Gbps / RX Gbps |
| 3-node saturation | total TX / RX Gbps |

An example method:

```bash
# interface counter snapshot -> load -> snapshot again
IF=enp3s0
read t0 r0 <<< "$(awk -v i=$IF '$1==i":"{print $10, $2}' /proc/net/dev)"
# ... load ...
read t1 r1 <<< "$(awk -v i=$IF '$1==i":"{print $10, $2}' /proc/net/dev)"
echo "TX $(( (t1-t0)*8/1000000 )) Mb   RX $(( (r1-r0)*8/1000000 )) Mb"
```

If these values differ substantially from the calculation (1.545 Gbps/node of
input), **the calculation's premise is wrong.** S2 does not proceed before
establishing which is right.

---

## 3.3.4 The scheduler host's CPU — **thread count decides throughput** (2026-08-26)

§3.3.2 covered **bandwidth only.** Actually swapping the host showed that
**CPU narrowed first.**

| | Old server | New server |
|---|---|---|
| Host | Dell PowerEdge R620 | ASUS H81M-K desktop |
| CPU | Xeon E5-2630L ×2 · **24 threads** · 2.0–2.5 GHz | Core i7-4790 · **8 threads** · 3.6–4.0 GHz |
| 10G NIC | **the same Intel X550T card** (moved across) | |
| Server CPU during measurement | 42% | **82.2%** |
| **Throughput** | **~391 inf/s** | **~360 inf/s** (−7.5%) |
| Error rate | 0 | 0 |

**Single-thread performance is clearly better on the new server and throughput
still fell.** The scheduler workload is governed not by single-request latency
but by **concurrent-stream throughput.**

### Why CPU narrows first

```text
scheduler          45.3%  ~ 3.6 cores
other (bench+kernel) 36.9%  ~ 2.9 cores
────────────────────────────────
total              82.2%  (of 8 threads)
```

**The bench client runs on the same host as the scheduler.** One CPU divides
its time between sending to and receiving from three nodes and generating the
load. On 24 threads the same work was 42%.

The application queue is empty under both conditions (`scheduler_queue` 0.00 ms
· `scheduler_route` 0.01 ms). What narrowed is not the queue but **the host's
CPU.**

### If you are following along

| | |
|---|---|
| **Recommended** | 16 threads or more |
| Minimum | 8 threads also **works correctly with 0 errors.** Only the throughput baseline differs |
| How to check | watch server CPU utilisation during measurement. Above 80% and the host is the constraint |
| Alternative | run the bench on another host. But that host also has to be on 10G |

**If you use lower specifications, lay a new baseline on that host and do not
compare directly against another host's values.**

> The PCIe generation was not a bottleneck. The same X550T sat at PCIe 3.0 x4
> (~32 Gbps per direction) on the R620 and 2.0 x4 (~16 Gbps) on the new server,
> but real three-node use is ~4.6 Gbps per direction, so both have ample
> headroom.

Basis: `infrastructure.md` §3.2.1 · `hosts/` ·
`../results/baseline-20260826-althost/`

### ⚠️ The output direction is larger — with `want_float=1`, even 10G was insufficient

The calculation above is **the input (TX) direction only.** Calculating the
output inverts the conclusion.

The node does not postprocess and **returns all nine raw tensors as they are.**

```text
input                       1,228,800 byte
output (want_float=1, f32)  4,872,000 byte   <- 3.96x the input
output (want_float=0, int8) 1,218,000 byte   <- 0.99x the input
```

The load on the scheduler link at three-node saturation:

| Configuration | Model | 3-node TX | 3-node RX | Fits in 10G? |
|---|---|---:|---:|---|
| `want_float=1` (old default) | INT8 | 4.64 Gbps | **18.38 Gbps** | **no** |
| `want_float=1` (old default) | FP16 | 2.49 Gbps | **9.86 Gbps** | barely (no headroom) |
| **`want_float=0` (current default)** | INT8 | 4.64 Gbps | 4.60 Gbps | yes |
| **`want_float=0` (current default)** | FP16 | 2.49 Gbps | 2.46 Gbps | yes |

**Had `want_float=1` remained, even 10G could not have carried three INT8
nodes.**

### So one of two things was needed before M3

**(A) Switch to `want_float=0`** — ✅ **completed 2026-08-12**

Receiving the output in its native dtype **cuts RX to a quarter.** In exchange
the receiver has to dequantize, so the blob was bumped to **v2** and carries
`qnt_type`, `scale` and `zero_point` per tensor. The node configuration's
`[worker] want_float` defaults to `false`.

> Dequantization was confirmed on a real board to match float32 (9 tensors,
> **maximum error 9.5e-7** — the limit of float32 precision).
>
> **Throughput rose alongside — INT8 +17.3% / FP16 +15.7%** (`king`, 8 threads,
> 120 s). The +5.4% in `discuss.md` §5 came out small because it was a mostly
> single-threaded FP16 figure. The grounds for the promotion were **RX
> bandwidth**, not throughput, but the two metrics pointed the same way.
> `discuss.md` §12

**(B) Postprocess (NMS) on the node as well** — unimplemented

Returning only detections shrinks the response to a few KB and effectively
removes RX. It is the correct final form, but the implementation remains.

Either way **the input TX of 4.64 Gbps is unchanged**, so 10G aggregation is
still needed.

### Measure it anyway

All of the above is calculation. Actual TX/RX are measured and recorded before
M3 starts (§3.3.3). **Calculating the input and not looking at the output is
what caused this error.** Trusting a calculation and moving on repeats the same
mistake.

---

## 3.4 Initial network restrictions

The following are not used initially.

- Wi-Fi
- Daisy-chaining nodes
- Docker overlay networks
- Kubernetes networking
- Complex VLANs
- Jumbo frames
- Multi-subnet routing

The default MTU is unified at 1500 on every device.

Jumbo frames are compared in a separate experiment after the baseline
performance is secured.

---

# 4. Storage configuration

## 4.1 Priority

1. **eMMC** (32GB or 64GB onboard)
2. High-endurance microSD
3. Ordinary microSD only for early development

An NPU worker does not store large data for long, so 32GB of eMMC is sufficient
for basic operation.

```text
/opt/npuforge/
├── bin/
├── config/
├── models/
└── logs/
```

Benchmark datasets, raw results, figures and presentation material are stored on
the scheduler PC.

## 4.2 NVMe is not used

The NanoPi R76S's M.2 slot is **SDIO-based and intended for a Wi-Fi module.** An
NVMe SSD cannot be fitted.

So the following are excluded from v0.1's scope.

- Storing large video files on the nodes
- Node file I/O performance comparison experiments
- Keeping several models locally on a node
- Long-term log retention on the nodes

Benchmark datasets, raw results and logs are all stored on the scheduler host.
In a simple three-node inference setup this constraint is not a problem.

Using the M.2 slot for Wi-Fi is also not done in v0.1 (Wi-Fi is excluded in
§3.4).

---

# 5. Operating system configuration

## 5.1 Recommended OS

The same headless Debian or Ubuntu Server image is installed on all three nodes.

A general Linux development environment suits better than a router-oriented
distribution.

Items that must be unified:

```text
same OS image
same kernel version
same NPU driver
same RKNN Runtime
same Rust binaries
same model files
same CPU governor
same cooling conditions
```

## 5.2 Base packages

```bash
sudo apt update

sudo apt install -y \
    build-essential \
    pkg-config \
    cmake \
    git \
    curl \
    chrony \
    iperf3 \
    ethtool \
    jq \
    htop \
    sysstat \
    linux-perf
```

The `linux-perf` package name can differ by distribution.

## 5.3 Rust binary deployment

Rather than building separately on each node, the following is recommended.

1. Produce the ARM64 binaries on a build PC
2. Deploy the same binaries to all three nodes
3. Verify the SHA-256 hashes
4. Run them as systemd services

This reduces build environment differences and raises reproducibility.

---

# 6. RKNN configuration

## 6.1 Separating conversion from execution

```text
development PC
  ONNX/PyTorch
      | RKNN-Toolkit2
  model.rknn
      | deploy
KING / QUEEN / JACK
      | RKNN Runtime
  NPU inference
```

Model conversion happens on the development PC, and the NanoPi run only the
converted RKNN model.

## 6.2 Items that must match across the three nodes

```text
RKNN Runtime version
RKNPU kernel driver version
model.rknn SHA-256
preprocessing configuration
postprocessing code
input resolution
quantization scheme
NPU core settings
```

An example of checking the model hash:

```bash
sha256sum /opt/npuforge/models/yolov8n/model.rknn
```

The result has to be identical on all three nodes.

## 6.3 Model directory example

```text
/opt/npuforge/models/
└── yolov8n/
    ├── model.rknn
    ├── model.toml
    └── labels.txt
```

---

# 7. Node configuration

Differences between nodes are limited to the following three.

```text
Node ID
IP address
Hostname
```

Everything else — configuration, models, binaries and runtime versions — has to
be the same.

## 7.1 KING

```toml
[node]
id = "king"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.21:51001"
scheduler_address = "http://10.20.0.10:50051"

[backend]
type = "rknn"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]

[worker]
worker_count = 1
max_queue_depth = 32
```

## 7.2 QUEEN

```toml
[node]
id = "queen"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.22:51001"
scheduler_address = "http://10.20.0.10:50051"

[backend]
type = "rknn"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]

[worker]
worker_count = 1
max_queue_depth = 32
```

## 7.3 JACK

```toml
[node]
id = "jack"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.23:51001"
scheduler_address = "http://10.20.0.10:50051"

[backend]
type = "rknn"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]

[worker]
worker_count = 1
max_queue_depth = 32
```

---

# 8. Power configuration

## 8.1 Input method: 12V DC (not USB-C PD)

The draft assumed USB-C PD and was **wrong.** The NanoPi R76S uses a **12V DC
input.**

Measured from the kernel log on 2026-08-10:

```text
vcc12v_dcin: 12000 mV, enabled          <- the main power input
vcc_sys: supplied by vcc12v_dcin
vbus5v0_typec: 5000 mV, disabled        <- Type-C is a 5V output, not an input
power_supply: simple-vin
PMIC: rk806
```

The Type-C port is for data and 5V VBUS **output**, not a power input path.

So the power measurement plan changes too. What is needed is a **12V DC line
power meter**, not a USB-C power meter (§14.2).

## 8.2 Recommended method

An independent 12V DC adapter per board.

```text
12V Adapter 1 -> KING
12V Adapter 2 -> QUEEN
12V Adapter 3 -> JACK
```

Recommended conditions:

- **12V, 2A (24W) or more**
- Same manufacturer, same model
- Same cable length
- Even with all three on one power strip, the adapters stay separate

### ⚠️ Insufficient current capacity resets the board under heavy load

Measurements on 2026-08-10 found different stability limits per node.

| Node | Stable limit | Symptom |
|---|---|---|
| `queen` | completed 8 threads | normal |
| `king` | **only up to 4 threads** | hard reset at 5 threads or more |
| `jack` | undetermined | one reset observed |

Since all three boards are the same model with the same software, **a difference
in power supply capability** is the likely cause. Details in
`board-worklog.md` §2.17.

Using 8 CPU cores and 2 NPU cores at maximum simultaneously raises instantaneous
current substantially. If the adapter's capacity is short, voltage drops and the
PMIC resets. The characteristic of this case is that nothing is left in the
kernel log.

**Different stability limits per node break the experimental premise of "three
identical machines".** This has to be resolved before measuring scaling
efficiency.

## 8.2 Power measurement

Including energy efficiency in a paper or talk requires per-node power
measurement.

Recommended method:

- Three USB-C power meters
- Or repeated measurement one machine at a time under identical conditions
- Idle power separated from inference load power
- Switch and scheduler power recorded separately

Measured metrics:

```text
Idle Watt
Peak Watt
Average Watt
Requests per Watt-hour
FPS per Watt
```

---

# 9. Cooling configuration

## 9.1 Two cooling conditions are measured (decided 2026-08-10)

**Fanless and active cooling are each measured.**

```text
condition A  fanless        as shipped. Throttling occurs
condition B  active cooling 3 identical fans. Throttling suppressed
```

### Rationale

The draft was to measure fanless only, but the sustained load test on
2026-08-10 observed the following.

| Condition | 8-thread throughput |
|---|---:|
| Burst load (20 repetitions) | 77.3 inf/s |
| Sustained load (3,000 repetitions) | 69.7 inf/s ⚠️ on `ondemand`. Current values are in `RESULTS.md` §2.2 |

**About a 10% drop.** And `king` exceeded `disable_temperature_c` (90 °C) at NPU
91.3 °C.

That is, the cooling condition directly affects both throughput and node
availability. Measuring only one condition leaves the following unanswerable.

- Fanless only → you do not know "how much better does cooling make it"
- Cooled only → you do not know "how much do you get in a real edge deployment"

**Measuring both conditions makes "the effect of cooling on scaling efficiency"
a result.** That is a figure absent from vendor spec sheets, and it matches this
project's identity of settling things by measurement.

### Condition A: fanless

Used exactly as shipped. Thermal throttling is not something to remove but
**something to measure.**

### Condition B: active cooling

**Three fans of the same model** are mounted identically on the three nodes.

- Same manufacturer, same model, same speed
- Same distance and angle
- Fan power consumption recorded separately (to be separated out in power
  efficiency calculations)

**Actual installation (2026-08-20):** three 120 mm-class 5V USB fans, one placed
over each node's board — **the fan is larger than the board (NanoPi R76S).**
Labelled K/Q/J, powered from a USB hub. The board sits directly under the fan
grille and takes airflow across its whole top surface.

> ⚠️ **All measurements on 2026-08-20 (the pilot and S2) were taken under this
> condition B (active cooling).** They were initially mis-recorded as
> "fanless (S0-A)" and corrected. With fans this large throttling is effectively
> suppressed, so the fanless (condition A) sustained figure of 157 must not be
> used as the node-ceiling comparison reference for this condition — see the 27%
> caveat in `results/scaling-20260820/README.md` §4.2.
> **Measuring condition A against condition B over the same gRPC path is §9.1's
> purpose**, and there is still no cluster measurement on the condition A
> (fanless) side.

### ⛔ A desk fan is not used

A desk fan was used for cooling during diagnosis on 2026-08-10. **It was valid
for diagnosis but cannot be used as a measurement condition.**

- The airflow does not reach the three boards evenly
- "The fan was angled like this" cannot be reproduced
- It does not satisfy condition B's requirements (identical fans, identical
  conditions)

### Applying to both conditions

- The same case, or no case at all
- Placed in the same orientation and spacing
- The same ambient temperature
- At least 10 cm between boards (so adjacent boards' heat does not affect each
  other)

```text
[KING]  <-10cm->  [QUEEN]  <-10cm->  [JACK]
        same ambient temperature, same orientation, same cooling
```

Ambient temperature is recorded for every experiment. It varies with the season
and indoor air conditioning, and without it results from different days cannot
be compared.

### ⚠️ Uniform placement has to come first

In the 2026-08-10 measurement, under identical load, **`king` was 19 °C hotter
than the other two** (NPU 91.3 vs 70.2 / 72.1 °C).

Turning on a fan converged all three to 56–62 °C, confirming **an airflow
problem rather than a defective unit.**

**Whichever of the two conditions is being measured, no valid data comes out
until the placement is made uniform.** Node-to-node temperature spread directly
contaminates scaling-efficiency measurement. Details in `board-worklog.md`
§2.19.

## 9.2 Temperature thresholds are a protection mechanism, not a measurement tool

The scheduler's `degraded_temperature_c` (80 °C) and `disable_temperature_c`
(90 °C) exist for **hardware protection.**

Leaving those values as-is in a fanless environment causes the following.

```text
300 s of sustained load -> all three nodes exceed 90 C -> all excluded from scheduling
-> NPF-1201 NO_AVAILABLE_NODE -> the benchmark stops
```

What gets measured then is not hardware performance but **the scheduler's
temperature policy.**

So the order is:

1. Perform **S0 thermal characterisation** (`01-TECHSPEC.md` §20.2) first to
   establish the steady-state temperature.
2. Set the thresholds on that basis. They have to be comfortably above the
   steady-state temperature.
3. Record the settled thresholds in `environment-matrix.md` §10.
4. Every official benchmark thereafter uses the same thresholds.

If a node really does get excluded on temperature, that is recorded as a result
too. But **it is reported separately from scaling-efficiency measurement.**
Mixed together, neither cause can be explained.

## 9.3 Benchmark temperature conditions

```text
starting temperature: within +5 C of the idle temperature established in S0
Warmup: 30 s
Measurement: 300 s
Repetitions: 5
Cooldown between repetitions: at most 180 s, or until the starting temperature is reached, whichever comes first
```

Being fanless, cooling is slow, so cooldown has **a cap.** When the cap is hit,
that fact and the actual starting temperature are recorded with the result.
Waiting indefinitely would break the 16-hour total budget (§20.4).

## 9.4 Required recorded items

The following are stored with the results, per node.

```text
ambient temperature
starting temperature
peak temperature
steady-state temperature
throttling onset (seconds)
CPU frequency changes
NPU frequency changes (where queryable)
whether and how often temperature caused scheduling exclusion
```

Under different temperature conditions, one node's thermal throttling appears as
a scheduler or network problem. That is why temperature recording is not
optional in this project.

---

# 10. Time synchronisation

The scheduler and all three nodes use `chrony`.

```bash
sudo systemctl enable --now chrony

chronyc tracking
chronyc sources
```

## 10.1 Timing measurement principles

Monotonic clock values from different machines are never compared directly.

Scheduler:

- End-to-end latency
- Scheduler queue time
- Routing time
- Node RPC round-trip time

Node:

- Local queue time
- Decode time
- Preprocess time
- NPU input preparation time
- Inference time
- Postprocess time

The node includes each stage's duration in its response.

NTP or chrony is used for ordering events in the structured logs.

---

# 11. Running the processes

## 11.1 NanoPi Worker

Each NanoPi runs only `npuforge-node`.

```text
systemd
└── npuforge-node.service
```

An example service:

```ini
[Unit]
Description=NPUDure Node Agent
After=network-online.target
Wants=network-online.target

[Service]
User=npuforge
Group=npuforge
ExecStart=/opt/npuforge/bin/npuforge-node \
    --config /etc/npuforge/node.toml
Restart=always
RestartSec=2
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

## 11.2 Scheduler PC

The scheduler PC runs the following processes.

```text
npuforge-scheduler
npuforge-dashboard
Prometheus
npuforge-bench
```

Grafana can optionally be added.

---

# 12. Benchmark configuration

## 12.1 One node

```text
active: KING
inactive: QUEEN, JACK
```

## 12.2 Two nodes

```text
active: KING, QUEEN
inactive: JACK
```

## 12.3 Three nodes

```text
active: KING, QUEEN, JACK
```

Official experiments use the scheduler's drain or disable functions rather than
killing processes or cutting power.

This keeps the network, power, temperature and equipment placement conditions
intact.

## 12.4 Default load conditions

```text
Concurrency: 1, 4, 16, 64
Warmup: 30 s
Measurement: 300 s
Cooldown: 60 s, or until the starting temperature is reached
Repetitions: 5
```

The per-scenario axes and the total measurement time budget follow
`01-TECHSPEC.md` §20.2 and §20.4.

## 12.5 Measured metrics

- Requests/sec
- FPS
- p50 latency
- p95 latency
- p99 latency
- Error rate
- Retry rate
- CPU utilisation
- Memory utilisation
- NPU utilisation
- Network usage
- Node temperature
- Power consumption
- Scaling factor
- Scaling efficiency

---

# 13. Physical setup for the talk demo

```text
+--------------+
| laptop       | <- Dashboard
+------+-------+
       |
+------v-------+
| 2.5G Switch  |
+-+----+----+--+
  |    |    |
+-v-++-v-++-v-+
|01 ||02 ||03 |
+---++---++---+
```

Each node carries a numbered label.

External status LEDs can optionally be used.

- Green: Healthy
- Yellow: Busy or Degraded
- Red: Unreachable
- Blue: Recovering

## 13.1 The failure demo

During the talk, disconnect QUEEN's network cable rather than pulling its power.

```text
3-node processing
-> QUEEN's network disconnected
-> health check fails
-> automatic exclusion
-> service continues on 2 nodes
-> cable reconnected
-> Recovering
-> automatic re-admission
```

Disconnecting the network recovers faster than cutting power and makes the demo
run more reliably.

## 13.2 Fallbacks for the talk

- A spare Ethernet cable
- A spare USB-C power adapter
- A recording of the same demo
- Mock backend mode
- Pre-generated benchmark results
- A configuration that works without internet

---

# 14. Recommended BOM

## 14.1 What is on hand (as of 2026-08-06)

| Item | Qty | Status |
|---|---:|---|
| NanoPi R76S | 3 | on hand. RAM specification to be confirmed |
| 1GbE switching hub | 1 | on hand. Used for the management network and the interim inference network |
| CAT6 cable | 1 | on hand |
| Linux PC | 1 | on hand. NIC speed to be confirmed |
| USB-TTL UART adapter | 1 | on hand |

## 14.2 To be obtained

In priority order.

| Item | Qty | Priority | Note |
|---|---:|---|---|
| **Identical-model fans** | **3** | **highest** | §9.1 condition B. Same manufacturer, model and speed. 5V USB fans recommended |
| **2.5G/10G switch** | 1 | **highest** | ≥ 4 × 2.5G ports + an SFP+ uplink. ~~A 2.5GbE-only switch~~ is discarded |
| **Scheduler server** | 1 | **highest** | a PCIe slot is required. `dealer` (a laptop) will not do |
| **10G NIC (PCIe SFP+)** | 1 | **highest** | e.g. Intel X520 |
| **SFP+ DAC cable** | 1 | **highest** | server ↔ switch uplink |
| Ethernet cables | 6–7 | high | 3 management + 4 inference. **Cat5e suffices** |
| USB power meters | 3 | medium | the board takes a 5V input, so USB meters work. Needed for FPS/Watt |
| Spare Ethernet cable | 1 | medium | a fallback for the talk |
| Spare power adapter | 1 | medium | a fallback for the talk. **5V 4A** |

### The power adapters are resolved (2026-08-10)

**Replaced with 5V 4A × 3.** Removed from the purchase list.

The previous adapters could not hold 5V even at no load (4.983V) and the boards
hard-reset under heavy load. After replacement they hold above 5.05V even under
sustained load on all three simultaneously.

**The board input is 5V.** Do not be misled by the `vcc12v_dcin` naming in the
kernel device tree. Measure it at
`/sys/class/power_supply/simple-vin/voltage_now`.

### Fan selection criteria

| Item | Criterion |
|---|---|
| Power | 5V USB recommended (the same voltage as the board, no separate adapter) |
| Quantity | **3, the same model** |
| Speed | fixed speed, or settable identically on all three |
| Noise | it is used in the talk demo, so worth considering |
| Mounting | attached directly to the board, or held at the same distance and angle |

**If speed is adjustable, fix all three fans at the same value.** Different
speeds give the nodes different cooling conditions and break §9.1's premise.

### On cable grades

**There is no need to buy CAT6.**

- 1GbE: Cat5/Cat5e suffices
- 2.5GBASE-T (IEEE 802.3bz): **supported on Cat5e up to 100 m.** The standard
  exists precisely to reuse existing Cat5e wiring.

Use the cables on hand and top up the quantity. What actually needs buying is
four things: **the 2.5G/10G switch, the scheduler server, the 10G NIC and the
SFP+ DAC** (§3.3.2).

### Why cooling equipment is not being bought

Heatsink cases and fans are excluded from the BOM. This follows §9.1's decision
to stay fanless, treating thermal throttling as something to measure rather than
remove.

---

# 15. Initial build order

## Step 1. Unify the hardware

- Confirm the RAM specification of the three boards
- Prepare identical storage
- Install identical heatsinks and fans
- Use identical power adapters

## Step 2. Clone the OS

- Configure one reference node
- Install the OS, kernel, packages and RKNN Runtime
- Clone the image to the other two nodes
- Change only hostname and IP

## Step 3. Verify the network

```bash
ping 10.20.0.21
ping 10.20.0.22
ping 10.20.0.23

iperf3 -s
iperf3 -c <target-ip>
```

Check the per-node link speed:

```bash
ethtool eth0
```

## Step 4. Verify RKNN on a single node

- Run the same model
- Confirm the result for the same input
- Confirm repeated inference stability
- Record the inference time

## Step 5. Verify the three nodes match

- Confirm the model SHA-256
- Confirm the binary SHA-256
- Confirm the runtime version
- Confirm the kernel and NPU driver
- Compare results for the same input

## Step 6. Deploy the NPUDure node

- Create the dedicated user
- Install the binaries
- Deploy the configuration files
- Register with systemd
- Confirm automatic registration with the scheduler

## Step 7. Baseline benchmarks

- 1 node
- 2 nodes
- 3 nodes
- Round Robin
- Record temperature and power

---

# 16. Final configuration baseline

The official NPUDure v0.1 hardware configuration is defined as follows.

```text
Worker Node:
  NanoPi R76S x 3
  SoC     : Rockchip RK3576 (4x A72 @2.2GHz + 4x A53 @1.8GHz)
  NPU     : 6 TOPS
  GPU     : Mali-G52 MC3
  Network : 2.5GbE x 2 (1 management + 1 inference)
  Storage : eMMC (M.2 is SDIO, so no NVMe)
  Cooling : stays fanless. Throttling is something to measure
  Same OS / Kernel / RKNN Runtime / Model / Power Supply

Scheduler:
  a separate Linux PC (a 2.5GbE NIC is required)
  not run on a NanoPi

Network:
  management : the existing network, 1GbE
  inference  : 2.5GbE star topology, 10.20.0.0/24, static IP, MTU 1500

Storage:
  Workers on eMMC
  Benchmark data and results stored on the scheduler
```

## 16.1 The change from RK3588 to RK3576 (2026-08-06)

The draft was written assuming an RK3588-based NanoPi R6C, but the equipment
actually on hand turned out to be the **RK3576-based NanoPi R76S.**

The main differences and their effects:

| Item | RK3588 (the draft's premise) | RK3576 (actual) | Effect |
|---|---|---|---|
| CPU | A76 + A55 | A72 @2.2 + A53 @1.8 | lower preprocessing/decoding performance. Higher chance the bottleneck is not the NPU |
| NPU | 6 TOPS | 6 TOPS | **none.** The talk title stands |
| Network | 2.5G + 1G | **2.5G × 2** | management network separation becomes the default configuration |
| M.2 | NVMe possible | SDIO (Wi-Fi only) | NVMe experiments excluded |
| Cooling | fan assumed | fanless | throttling becomes something to measure |
| RKNN | `target_platform='rk3588'` | `target_platform='rk3576'` | model reconversion needed. `.rknn` files are not portable across platforms |

The weaker CPU is, if anything, more material for this project. Preprocessing
and JPEG decoding are done by the CPU, so if the bottleneck appears as CPU
preprocessing rather than the NPU, that result itself supports this project's
claim that "the TOPS figure does not represent actual throughput".

This configuration satisfies three purposes at once.

- The FOSS for All Conference demo, November 2026
- A reproducible open-source benchmark
- An experimental platform for the doctoral thesis and follow-up research
