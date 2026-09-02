# NPUDure Environment Matrix

*[한국어 원문](environment-matrix.ko.md)*

- Document: `environment-matrix.md`
- Project: NPUDure
- Target release: NPUDure v0.1
- Written: 2026-08-06
- Status: **settled.** Closed through the S0 thermal characterisation (§9). The open list is in `experiments/README.md` §7
- Related documents:
  - `01-TECHSPEC.md` §2.5 reproducibility
  - `03-DEVELOPMENT-REQUIREMENTS.md` §2.1, §9

---

# 1. Purpose

This document is the single source for **pinning NPUDure v0.1's version
combination and hashes.**

The values recorded here cannot be derived from the source code, the
configuration files or the git history. The combination of RKNN Toolkit, Runtime
and kernel driver is a fact given from outside, and when the combination changes
previous benchmark results become incomparable.

Item 1 of `03-DEVELOPMENT-REQUIREMENTS.md` §9's immediate actions is filling in
this document.

**Performance figures recorded before this table is filled in are not used as
official results.**

---

# 2. Settled status

| Item | Status | Value |
|---|---|---|
| Board and SoC | **settled (2026-08-06)** | RK3576, 2-core NPU — §2.1 |
| RKNN version combination | **settled (2026-08-07)** | Runtime 2.3.0 / Driver v0.9.8 / Toolkit2 2.3.0 — §3 |
| Kernel and driver | **settled (2026-08-07)** | 6.1.141, identical across 3 nodes — §4 |
| Reference model hashes | **settled (2026-08-12)** | FP16 `459602ea…` / INT8 `dba155d2…` — §6 |
| Dataset hash | **settled (2026-08-11)** | 200 COCO val2017 images `224b8beb…` — §7 |
| Rust toolchain | **settled (2026-08-12)** | 1.97.1 / edition 2024 / MSRV 1.85 — §8 |
| Node inventory | **settled (2026-08-07)** | serials and MACs — §8.1 |
| RKNN concurrency contract | **settled (2026-08-11)** | context sharing forbidden — the correction in §3.1 |
| CPU governor | **settled (2026-08-12)** | fixed to `performance` and made permanent — §4 |
| Thermal characteristics and temperature thresholds | **settled** — degraded 80 / disable 90 °C | S0 results. §9.2 |
| OS patch level uniformity | **settled (2026-08-12)** | all three nodes on 24.04.4 — §4 |
| SSH host key uniqueness | ⚠️ **unresolved** | queen and jack are identical — §8.1 |

At the point of settling, each row's status changes to `settled (YYYY-MM-DD)`
and the value is filled in.

When a value changes, the previous value is recorded in §11's change history.

---

# 2.1 Board and SoC (settled)

Settled by measurement on all three nodes on 2026-08-07. Collected with
`scripts/collect-node-info.sh`; the raw output is in
`benchmarks/node-info/{k,q,j}.txt`.

| Item | Value | How verified |
|---|---|---|
| Board | FriendlyElec NanoPi R76S | `/proc/device-tree/model` |
| device-tree compatible | `friendlyelec,nanopi-r76s rockchip,rk3576` | `/proc/device-tree/compatible` |
| SoC | **Rockchip RK3576** | as above |
| CPU core count | 8 | `nproc` |
| CPU little cluster maximum | 2,016,000 kHz (2.016 GHz) | `cpufreq/policy0` |
| CPU big cluster maximum | 2,208,000 kHz (2.208 GHz) | `cpufreq/policy4` |
| GPU | Mali-G52 MC3 | product specification |
| NPU | 6 TOPS | product specification |
| **NPU core count** | **2 (Core0, Core1)** | `/sys/kernel/debug/rknpu/load` |
| NPU frequency | 300–950 MHz, default 950 MHz | `devfreq/27700000.npu` |
| NPU IOMMU | enabled | `dmesg` |
| RAM | **4GB LPDDR4X** (3,997,848 kB) | `/proc/meminfo` |
| eMMC | **64GB** (122,142,720 × 512B ≈ 62.5GB) | `/sys/block/mmcblk2/size` |
| rootfs free | 50GB | `df -h /` |
| Network | **2.5GbE × 2** (`eth0`, `eth1`) — driver `r8125`, separate PCIe buses | `ethtool` |
| M.2 | SDIO (Wi-Fi only, no NVMe) | product specification |
| Cooling | fanless | product specification |

**The NPU has 2 cores.** That differs from the RK3588's 3, so RK3588-based
`core_mask` examples cannot be used as-is. It directly affects the
`worker_count` decision (§3.1).

4GB of RAM is enough to run several workers. A 2GB variant would have been a
constraint.

The draft assumed RK3588/NanoPi R6C and was corrected on 2026-08-06. Detailed
effects in `02-HARDWARE-SETUP.md` §16.1.

## 2.2 Thermal sensors

There are 6 thermal zones. A dedicated NPU sensor exists and can be used
directly for §9's thermal characterisation.

| zone | type | idle temperature (2026-08-07) |
|---|---|---|
| 0 | `soc-thermal` | 44.4 – 46.2 °C |
| 1 | `bigcore-thermal` | 45.3 °C |
| 2 | `little-core-thermal` | 45.3 °C |
| 3 | `ddr-thermal` | 44.4 °C |
| 4 | **`npu-thermal`** | 42.5 – 45.3 °C |
| 5 | `gpu-thermal` | 46.2 °C |

The node configuration's `temperature_path` uses `soc-thermal` (zone0) for
scheduling decisions and records `npu-thermal` (zone4) separately.

**It is already 42–46 °C at idle.** On a fanless board the draft document's
"starting temperature at or below 45 °C" condition is marginal even at idle. It
is reset from measurement in §9.2.

---

# 3. The RKNN stack

All three nodes have to be identical.

Measured 2026-08-07. The `librknnrt.so` SHA-256 was confirmed identical on all
three nodes.

| Item | Value | How verified |
|---|---|---|
| Conversion target platform | **`rk3576`** | fixed. Not `rk3588` |
| **RKNN Runtime version** | **2.3.0** (`c949ad889d@2024-11-07T11:35:33`) | `strings librknnrt.so` |
| **RKNPU driver version** | **v0.9.8** | `/sys/kernel/debug/rknpu/version` |
| **NPU core count** | **2** | `/sys/kernel/debug/rknpu/load` |
| `librknnrt.so` path | `/usr/lib/librknnrt.so` | identical on 3 nodes |
| `librknnrt.so` SHA-256 | `73993ed4b440460825f21611731564503cc1d5a0c123746477da6cd574f34885` | identical on 3 nodes |
| Headers | `/usr/include/rknn_api.h` | installed |
| RKNN-Toolkit2 version | **2.3.0** | the `npuforge-converter:2.3.1` Docker image on `dealer`. Matches the Runtime |

**Toolkit2 has to match Runtime 2.3.0.** If the Toolkit version is higher than
the Runtime's, converted models may fail to load. When installing on the
development PC, try `rknn-toolkit2==2.3.0` first.

Since the NPU has 2 cores, core_mask strategy differs from RK3588 (3-core)
examples.

## 3.1 Thread-safety verification results (settled 2026-08-07)

The node architecture depends directly on this result. If concurrent calls are
impossible, a dedicated worker thread and mutex per model are needed; if
possible, `worker_count` can be set above 1.

**Conditions.** `king`, the FP16 model (`yolov8n-fp16.rknn`), 20 iterations per
thread. The tool is
`crates/npuforge-rknn/native/thread_safety_test.c`.

| Configuration | Threads | ok / err | Mean latency | Throughput | vs baseline |
|---|---:|---:|---:|---:|---:|
| Baseline (dedicated context) | 1 | 20 / **0** | 62.62 ms | 16.0 inf/s | 1.00× |
| **Shared context** | 2 | 40 / **0** | 57.28 ms | 34.8 inf/s | 2.18× |
| Dedicated context (`CORE_AUTO`) | 2 | 40 / **0** | 58.77 ms | 33.2 inf/s | 2.08× |
| Dedicated context + core separation | 2 | 40 / **0** | 62.58 ms | 31.9 inf/s | 1.99× |
| Dedicated context | 4 | 80 / **0** | 76.22 ms | **52.3 inf/s** | **3.27×** |

### Conclusions

| Item | Result |
|---|---|
| Concurrent calls on the same context | **possible** (0 errors) |
| Concurrent calls on different contexts | **possible** (0 errors) |
| Serializing with a dedicated worker thread per model | **unnecessary** |
| Explicit `core_mask` separation | **unnecessary** — +0.1% at 8 threads |
| Recommended `worker_count` | **8** (+27% over 4) |
| Actual contribution of the NPU's 2 cores | **1.51×** (single core 48.2 → two cores 73.0 inf/s) |

**RKNN Runtime 2.3.0 is thread-safe.** No errors occurred in any combination.

> ### ⚠️ Correction 2026-08-11: "0 errors" does not mean "the results are right"
>
> The table above **counted only API return codes and never compared output
> contents.** Actually comparing the outputs changes the conclusion.
>
> One inference is three calls.
>
> ```text
> rknn_inputs_set  ->  rknn_run  ->  rknn_outputs_get
> ```
>
> Even with each call thread-safe, **this sequence is not atomic.** Two threads
> overlapping on the same context take each other's results.
>
> Verified with `native/shared_context_test.c`. Each thread was given a
> different input and compared against its standalone result (4 threads × 50,
> `king`).
>
> | Configuration | API errors | **Result mismatches** |
> |---|---:|---:|
> | Shared context | 0 | **200 / 200 (100%)** |
> | Per-thread dedicated context | 0 | 0 / 200 (0%) |
>
> **A shared context produces 100% wrong answers with no errors.**
>
> So `supports_concurrent_infer = true` stays, but its basis is not "the runtime
> handles it" but **"the backend serializes through a context pool"**. See
> `crates/npuforge-rknn/src/context.rs`.
>
> Among the throughput figures in the table above, the "shared context" row
> (2 threads, 34.8 inf/s) is **the speed of a state producing wrong results**
> and is not used for performance comparison.

### Why 4 threads is faster than a 2-core NPU would suggest

One inference is not just NPU execution but **set input → NPU execution → get
output**, and the sections either side are handled by the CPU. With more threads
than cores, one thread can occupy the NPU while another is in its CPU section,
producing a pipelining effect.

**Latency and throughput trade off.**

```text
1 thread : 62.6 ms,  16.0 inf/s   minimum latency
2 threads: 58.8 ms,  33.2 inf/s
4 threads: 76.2 ms,  52.3 inf/s   maximum throughput (within the measured range)
```

**This project targets throughput, so raising thread count is correct.** But
latency increases disadvantage requests carrying a deadline, so it is tuned
alongside `max_queue_depth`.

### Why explicit core separation is not used (settled by re-measurement 2026-08-10)

Four modes including a control group (`CORE_0_ONLY`) were compared at 1/2/4/8
threads. Details in `docs/discuss.md` §4.

| Threads | `CORE_AUTO` | `ALTERNATE` | `CORE_0_1` | `CORE_0_ONLY` |
|---:|---:|---:|---:|---:|
| 1 | 16.7 | 16.7 | **18.2** | 16.5 |
| 4 | 52.4 | **57.1** | 48.5 | 38.5 |
| 8 | **72.9** | 73.0 | 64.5 | 48.2 |

**Conclusion: do not set `core_mask`.**

- `ALTERNATE`'s gain is +9% at 4 threads and **vanishes to +0.1% at 8**
- `CORE_0_1` is actually a loss at −11.5% at 8 threads
- `CORE_AUTO`'s distribution is already even (Core0 39% / Core1 37% at 8 threads)

Going to 8 threads beats manual core assignment, and dropping the
`rknn_set_core_mask` call simplifies the implementation.

**The second core does contribute.** Against the control group it goes 48.2 →
73.0 inf/s, **1.51×**. That it is not 2× suggests serialization on a shared
resource outside the cores.

**Exception: when single-request latency matters.** Only at 1 thread is
`CORE_0_1` favourable at +9% (`run` 29.7 → 23.7 ms). Worth considering for
deadline-carrying requests.

`rknn_api.h`'s `rknn_core_mask` defines up to three cores, but RK3576 has two,
so `CORE_2` cannot be used.

### FP16 baseline performance and its implications

FP16 gives **84.3 inf/s** per node (8 threads, governor `performance`); INT8
gives **157.2 inf/s**.
(The 16–52 inf/s of the initial measurements were at 1–4 threads on `ondemand`.
`RESULTS.md` §2.2)

The network requirement when summed across three nodes. **Input and output are
considered together.**

```text
                                per node      3 nodes
INT8 input (raw RGB 1.23MB)    1.545 Gbps    4.636 Gbps
INT8 output (want_float=1)     6.128 Gbps   18.383 Gbps   <- even 10G is insufficient
INT8 output (want_float=0)     1.532 Gbps    4.596 Gbps
FP16 input                     0.829 Gbps    2.486 Gbps
FP16 output (want_float=1)     3.286 Gbps    9.858 Gbps
```

**It is the aggregation link, not the worker links (2.5G), that fills up
first.** 10G is needed on the scheduler side, with a measure to reduce output on
top of it. **The output reduction was solved by switching to `want_float=0`
(2026-08-12, the default).** What remains is securing 10G aggregation.
`02-HARDWARE-SETUP.md` §3.3.2.

> **The previous version is discarded.** It said "3 nodes at 156 FPS, raw RGB
> 1.5 Gbps, 2.5GbE needed only in S6". That calculation (a) used 52 inf/s from
> 4 threads on `ondemand` and (b) **never looked at the output direction.** Both
> premises changed after measurement.

### Not yet settled

- Throughput had not bent even at 8 threads, so the region past `MAX_THREADS`
  is unexplored
- S0 thermal characterisation (30 min × fanless/cooled, 2 conditions)
- `ondemand` vs `performance` compared under **identical 300-second conditions**
  (§3.1's +7% is a 120-second value covering only the pre-downgrade region)

> **Resolved — `want_float=0`'s effect on INT8 throughput** (2026-08-12).
> This had been left as "§5's +5.4% was measured on FP16 and cannot be carried
> across". Measured at 8 threads for 120 s. **INT8 156.7 vs 133.6 inf/s
> (+17.3%), FP16 66.9 vs 57.8 inf/s (+15.7%).** It exceeds §5 because §5 was a
> mostly single-thread condition — the more concurrent threads there are, the
> longer output conversion holds the serialized section. `discuss.md` §12

### The remaining bottleneck

With NPU at 40% and CPU at 49%, **neither is saturated and yet `rknn_run` wait
alone grows.** Serialization on a shared resource outside the cores is the
presumption, with these candidates:

- A lock inside the RKNN runtime
- Kernel driver ioctl serialization
- IOMMU / buffer mapping cost
- DDR / memory bandwidth
- Output conversion / a hidden copy

`perf record`, `strace -c` and off-CPU analysis are needed. See
`docs/discuss.md`.

---

# 4. Operating system and kernel

| Item | Value | Status |
|---|---|---|
| Distribution | Ubuntu 24.04 LTS (Noble Numbat) | settled |
| **Patch level** | **24.04.4 LTS** | ✅ identical on 3 nodes (confirmed 2026-08-12. king went 24.04.3 → 24.04.4) |
| Kernel version | 6.1.141 (aarch64) | identical on 3 nodes |
| glibc | 2.39 | identical on 3 nodes |
| gcc | 13.3.0-6ubuntu2~24.04.1 | ✅ identical on 3 nodes (confirmed 2026-08-12) |
| Python | 3.12.3 | identical on 3 nodes |
| rustc | **1.97.1 installed on `king` only** | for native node binary builds. Not installed on queen/jack |
| **CPU Governor** | **`performance`** | ✅ fixed 2026-08-12. Made permanent with a systemd unit (+7% throughput) |
| Unapplied package updates | K: 274 / Q: 280 / J: 280 | ⚠️ recommended to unify before measuring. The kernel is held, so it is safe |
| OS image filename | not recorded | not captured when the boards arrived. Record it on reinstallation without fail |
| OS image SHA-256 | not recorded | as above |
| io_uring support | **supported** | `io_uring_setup` confirmed present in `/proc/kallsyms` (2026-08-12) |

## 4.0 Bootloader firmware ⚠️

**The layer responsible for power management (BL31/ATF) and DDR timings.**
Differing versions between nodes give differing stability under heavy load.

Measured 2026-08-10:

| Component | `king` | `queen` | `jack` |
|---|---|---|---|
| DDR init | **v1.09** | v1.13 | v1.13 |
| SPL | **v1.07** | v1.09 | v1.09 |
| **BL31 (ATF)** | **v1.17** | **v1.24** | **v1.24** |
| BL32 | **v1.05** | v1.10 | v1.10 |
| U-Boot | **`44f011c4ba` 2025-07-17** | `c5c053fa55` 2026-07-10 | `c5c053fa55` 2026-07-10 |
| PMIC initialisation | **`ON:0x20 OFF:0x2`** | `ON:0x40 OFF:0x0` | `ON:0x40 OFF:0x0` |

`queen` and `jack` are completely identical and **only `king` is about a year
old.**

### This appears to be the cause of `king`'s heavy-load resets

`king` hard-resets at 5 threads or more (`board-worklog.md` §2.17). BL31 handles
DVFS and voltage regulation on Rockchip, so an old version's voltage table
failing to cope with heavy load produces exactly this symptom. The DDR firmware
difference can also cause instability under memory-heavy multi-threaded
conditions.

The differing PMIC initialisation register is also a consequence of the firmware
difference.

### How to check

```bash
grep -oE 'androidboot\.fwver=[^ ]*' /proc/cmdline
```

`scripts/collect-node-info.sh` collects this value (added 2026-08-10).

### Remedy

**`king`'s bootloader has to be updated to the same version as `queen`/`jack`.**
Re-verify with a 5–8 thread test after updating.

The three nodes' `fwver` strings have to match exactly for the premise of "three
identical machines" to hold. That this item was missing from §4.1's list of
required matches was a documentation omission.

## 4.1 Unresolved mismatches

The three nodes are supposed to be on the "same OS image"
(`02-HARDWARE-SETUP.md` §5.1). The following are currently out of line.

| Item | Detail | Risk |
|---|---|---|
| Ubuntu patch level | only K on 24.04.3 | library version differences can appear as per-node performance variance |
| Pending updates | 279–374 | as above |
| SSH host key | **queen and jack identical** (king is unique after a reinstall) | ⚠️ **unresolved.** Regeneration was missed when cloning the image. queen and jack cannot be told apart cryptographically — a changed IP attaches you to the wrong board without a warning (the §2.20 type) |
| hostname | K and Q both `NanoPi-R76S`, J `localhost.localdomain` | nodes indistinguishable in logs and the dashboard |
| CPU Governor | **`performance`** | fixed and made permanent 2026-08-12. +7% throughput over `ondemand` |

**Caution on kernel upgrades.** Kernel 6.1.141 is the FriendlyElec BSP kernel
and the RKNPU driver v0.9.8 is tied to it. If `apt upgrade` replaces the kernel,
the NPU may stop working. Hold the kernel package when updating.

## 4.2 The scheduler host

Measured 2026-08-07. An old laptop serves as the scheduler / benchmark / model
conversion host.

| Item | Value | Verdict |
|---|---|---|
| hostname | **`dealer`** | set 2026-08-07 (unified with the K/Q/J card naming) |
| Model | Samsung 370E5J / 380E5Q series | |
| **Distribution** | **Rocky Linux 9.7 (Blue Onyx)** | ⚠️ the boards are Ubuntu 24.04 |
| Kernel | 5.14.0-611.13.1.el9_7.x86_64 | |
| glibc | **2.34** | ⚠️ the boards are 2.39 |
| Package manager | **`dnf`** | ⚠️ the boards use `apt` |
| CPU | Intel Core i7-4712MQ @2.30GHz (Haswell, 4C/8T) | sufficient for generating load |
| RAM | **3.5GB** (about 1.8GB available) | ⚠️ the biggest constraint |
| Swap | 3.9GB | eases memory pressure during conversion |
| Disk free | **60GB** (`/`, 16% of 70GB used) | sufficient for the Docker image |
| Architecture | x86_64 | can run RKNN-Toolkit2 |
| NIC | Realtek RTL8111/8168 (`r8169`), **1GbE ceiling** | no 2.5G support |
| Link speed | **1000 Mb/s** | normal |
| Management IP | `192.168.123.14/24` (`enp3s0`) | the same range as the boards |
| MAC | `<redacted-mac>` | |
| USB 3.0 | Bus 004 (`xhci_hcd`, 5000M, 4 ports) | a 2.5G adapter could be added |
| Thunderbolt | none | |
| Docker | **29.2.1**, storage `overlayfs` | the model conversion environment |
| Python (host) | 3.9.23 | irrelevant, since conversion happens inside the container |
| Account | `yoo2` (in `wheel` and `docker`) | groups added 2026-08-07 |
| root SSH | blocked | escalation via `su` |

### ⚠️ The host and the nodes run different distributions

| | Scheduler host | The 3 nodes |
|---|---|---|
| Distribution | Rocky Linux 9.7 | Ubuntu 24.04 |
| glibc | 2.34 | 2.39 |
| Package manager | `dnf` | `apt` |

**Effect 1 — scripts.** `scripts/fix-node-consistency.sh` is `apt`-only. That is
fine since it targets the nodes, but a script applied to the host too has to
branch on package manager.

**Effect 2 — binary deployment.** Fortunately the direction is the safe one.

```text
build host glibc 2.34  ->  run target glibc 2.39   (old -> new, compatible)
```

A binary built against a lower glibc runs on a higher one. The reverse does not
hold.

But **Rust is not currently installed on `dealer`.** The actual build happens
natively on `king` and the artefacts are deployed to the three nodes. Since the
three boards are all on glibc 2.39, that direction is fine.

However, the `npuforge-scheduler` (x86_64) binary has to be built on `dealer`
directly, or in an environment with glibc 2.34 or lower.

**Effect 3 — recording reproducibility.** On open-source publication we cannot
write "developed on Ubuntu". The host's and the nodes' distributions are stated
separately.

### The 100 Mb/s link problem (resolved)

The initial measurement had negotiated `Speed: 100Mb/s`. The port supports
1000baseT, so it was a cable problem, and replacement normalised it to
1000 Mb/s.

Left alone, at 100KB JPEGs the link would have saturated at about 125 FPS, and
we would have measured the cable rather than the NPU. **A procedure for checking
link speed before every experiment is in place.**

```bash
ethtool enp3s0 | grep Speed
```

### Handling the RAM constraint

The scheduler host (3.5GB) has less memory than a node (4GB).
`npuforge-scheduler` + `npuforge-bench` + Prometheus + Dashboard cannot all be
run at maximum load simultaneously.

**Policy:**

| Situation | Configuration |
|---|---|
| Official benchmarks | Scheduler + bench only. Prometheus and Dashboard stopped. Raw data recorded as JSONL |
| Talk demo | Scheduler + Dashboard. Load kept low |
| Development | no restriction |

The raw data is the output and the dashboard is for the demo, so there is no
need to run both at maximum. `npuforge-bench` records host CPU and memory
utilisation while running, so whether the client was the bottleneck can be
determined afterwards.

### The 2.5GbE upgrade decision is deferred

Currently 1GbE. Whether 2.5G is needed is decided **after measuring actual FPS
per node in S0/S1.**

```text
assuming 40 FPS per node -> 3 nodes 120 FPS x 100KB ~ 96 Mbps   -> 1GbE suffices
raw RGB input (S6)       -> 120 FPS x 1.23MB ~ 1.2 Gbps         -> exceeds 1GbE
```

~~A USB 3.0 2.5GbE adapter~~ is not enough. Three-node aggregation needs **10G**
(the table above), and a USB adapter tops out at 2.5G. `dealer` has no PCIe
slot, so **the scheduler host has to be replaced with a server.**
`02-HARDWARE-SETUP.md` §3.3.2.

If a USB NIC is used, state that fact in the results.

---

# 5. Rust and the build toolchain

| Item | Value |
|---|---|
| Rust version | **1.97.1** (installed on `king` only) |
| Edition | **2024** |
| MSRV | **1.85** |
| Cross linker | `aarch64-linux-gnu-gcc` |
| Cross toolchain version | not used — native build on `king` (gcc 13.3.0) |
| protoc version | **libprotoc 3.21.12** (`king`) |

Build artefact hashes differ per release and are recorded in the release notes
rather than here.

---

# 6. The reference model

## 6.1 The ONNX original

| Item | Value |
|---|---|
| Model | YOLOv8n (the RKNN-optimized version) |
| Source | `airockchip/rknn_model_zoo` → `examples/yolov8` |
| Upstream project | `airockchip/ultralytics_yolov8` |
| **License** | **AGPL-3.0** (see `MODEL_LICENSES.md` §2) |
| File | `yolov8n.onnx` |
| Size | 12,650,184 bytes |
| **SHA-256** | `0c8716701f471067932b797eeb67c8e5db47c693c2557c881d7679ec12e21bc5` |
| Export tool | PyTorch 2.0 |
| Input resolution | 640 × 640 RGB |

### ⚠️ Why the standard Ultralytics export is not used

The official original includes DFL and NMS postprocessing in the ONNX graph.
Those operators do not map to the NPU and cause extensive CPU fallback.
**Measuring in that state measures the CPU, not the NPU.**

The Rockchip-optimized version outputs the raw tensors before decoding and
performs postprocessing separately on the CPU.

```text
official original : 1 output (decode and NMS included)
optimized version : 3 output groups
                    [1,64,80,80]  box coordinates
                    [1,80,80,80]  per-class confidence for 80 classes
                    [1,1,80,80]   confidence sum
```

RK3576 is on the officially supported list (RK3562/3566/3568/**3576**/3588/
RV1126B/RV1109/RV1126/RK1808/RK3399PRO).

## 6.2 The converted RKNN

### FP16 (for thread-safety verification, 2026-08-07)

Since the calibration data was not settled, FP16 was converted first. Without
quantization, concurrency verification is unaffected.

| Item | Value |
|---|---|
| File | `yolov8n-fp16.rknn` |
| Size | 9,645,065 bytes |
| **SHA-256** | `459602ea70479c1ce4fdd7419aa81e10e2f795fe6fe87444f3607f25b7054c0f` |
| Quantization | none (FP16) |
| target_platform | `rk3576` |
| Deployed to 3 nodes with matching hashes | confirmed |

### INT8 (the reference model) — **generated and verified (2026-08-12)**

| Item | Value |
|---|---|
| Quantization | INT8 |
| Calibration images | **200** (COCO val2017, seed 20261128) |
| Calibration manifest SHA-256 | `d8d189fc386897dd…` ⚠️ based on absolute paths. The portable value is `224b8bebd5f3a4ce…` |
| RKNN SHA-256 | INT8 `dba155d2088df622…` / FP16 `459602ea70479c1c…` |
| CPU fallback operator list | not investigated. Can be confirmed from the `not support` warnings in the conversion log |

Generated after the calibration dataset was settled (§7).

## 6.3 The conversion environment

| Item | Value |
|---|---|
| Image | `npuforge-converter:2.3.0` (9.61GB) |
| Base | `ubuntu:22.04` |
| Python | 3.10.12 |
| **rknn-toolkit2** | **2.3.0** (matching the boards' Runtime 2.3.0) |
| **onnx** | **1.14.1 (pinning required)** |
| torch | 2.4.0 (to be switched to CPU-only) |
| numpy | 1.26.4 |
| protobuf | 4.25.4 |

### ⚠️ The onnx version has to be pinned

`rknn-toolkit2`'s dependency specification does not constrain the onnx version,
so the latest (1.22.0 at the time) got installed and conversion failed.

```text
AttributeError: module 'onnx' has no attribute 'mapping'
```

`onnx.mapping` was removed in onnx 1.16, and rknn-toolkit2 2.3.0 uses it.
**Pinning to 1.14.1 makes it work** (measured 2026-08-07).

The pin and a verification step are in the Dockerfile.

```dockerfile
RUN python3 -m pip install "onnx==1.14.1" \
    && python3 -c "import onnx; assert hasattr(onnx, 'mapping')"
```

The CPU fallback list feeds directly into scaling-efficiency analysis and must
be recorded. The more operators run on the CPU rather than the NPU, the greater
the node-to-node variance and thermal influence.

---

# 7. The benchmark dataset

| Item | Value |
|---|---|
| Dataset name | **a COCO val2017 subset** |
| Source | `http://images.cocodataset.org/val2017` |
| Redistribution terms | **redistribution forbidden.** The individual images come from Flickr with varying licenses. COCO applies CC-BY 4.0 to the annotations only. Only a manifest goes in the repository |
| Image count | **200** |
| Selection method | sorted, then extracted with a fixed seed (20261128). `tools/model-converter/fetch_calibration.py` |
| Input format | 640×640×3 uint8 NHWC RGB (preprocessing done by `make_reference.py`) |
| Manifest SHA-256 | `224b8bebd5f3a4ce906388d2fab1371ce0b84bf92e352226fb270f2fe3560fec` |

The same data is currently used for both calibration and accuracy verification.
**The benchmark load uses synthetic input generated deterministically by
`npuforge-bench`** (fixed seed). If loading with real images becomes necessary,
a separate set is defined here.

---

# 8. Node inventory

The management information required by `03-DEVELOPMENT-REQUIREMENTS.md` §4.4.

The boards carry physical **K / Q / J** labels. Node IDs and hostnames match
them.

| Item | K | Q | J |
|---|---|---|---|
| Node ID | `king` | `queen` | `jack` |
| hostname | `king` | `queen` | `jack` |
| Previous hostname | `NanoPi-R76S` | `NanoPi-R76S` | `localhost.localdomain` |
| Management IP (current) | `192.168.123.12` | `192.168.123.16` | `192.168.123.33` |
| Management port | `eth1` | `eth1` | `eth1` |
| Management MAC | `<redacted-mac>` | (collected) | (collected) |
| Management link | 1000 Mbps (negotiated to the 1G hub. The port supports 2.5G) | 1000 Mbps | 1000 Mbps |
| Inference IP (planned) | `10.20.0.21` | `10.20.0.22` | `10.20.0.23` |
| Inference port | `eth0` (2.5G, unconnected) | `eth0` (2.5G, unconnected) | `eth0` (2.5G, unconnected) |
| Inference MAC | `<redacted-mac>` | (collected) | (collected) |
| Serial | `aaf2afcf6887055` | `64901d66a690b679` | `5b1e0475e81e50e4` |
| RAM | 4GB | 4GB | 4GB |
| eMMC | 64GB | 64GB | 64GB |
| Power adapter | 5V 4A | 5V 4A | 5V 4A |

The full MAC list is in `benchmarks/node-info/{k,q,j}.txt`.

**`eth0` is `down` on all three nodes.** The second 2.5G port is free and can be
used for the inference network as-is. `eth1` is currently connected to the 1G
hub serving as the management network.

All three nodes stay in their fanless factory state (`02-HARDWARE-SETUP.md`
§9.1).

**Which physical port is used for the inference network has to be identical on
all three nodes.** Mixing ports gives the nodes different network
characteristics and makes comparison meaningless.

The scheduler host is `npuforge-scheduler` / `10.20.0.10`.

## 8.1 Verifying they match

```bash
./scripts/check-versions.sh
./scripts/check-model-hashes.sh
```

The three nodes' output has to be identical, and is checked before every
official benchmark run.

---

# 8.2 Power (settled 2026-08-10)

## Input method

**The input is 5V.** The kernel device tree's `vcc12v_dcin: 12000 mV` is merely
a fixed-regulator declaration, not the actual input voltage. It is a leftover
from Rockchip device trees being copied between boards.

**Always check the measured sensor value.**

```bash
cat /sys/class/power_supply/simple-vin/voltage_now   # microvolts
```

| Item | Value |
|---|---|
| Input voltage | **5V** |
| Sensor path | `/sys/class/power_supply/simple-vin/` |
| Adapter rating | **5V 4A (20W)** × 3, independent per node |

## Before and after the adapter replacement

| State | Idle voltage | Stability under heavy load |
|---|---|---|
| Before | **4.983 V** (below 5V) | hard reset at 3–5 threads |
| After (5V 4A) | **5.27 – 5.31 V** | completes 8 threads |

The previous adapters **could not hold 5V even at no load.** Dropping further
under heavy load past the brownout threshold was the cause of the reboots.

## Voltage under sustained load (3 boards simultaneously, 8 threads)

| Node | Minimum voltage |
|---|---|
| `king` | 5.061 V |
| `queen` | 5.157 V |
| `jack` | 5.124 V |

Even running all three at maximum load simultaneously it does not fall below 5V.
**Power headroom is secured.**

## Recording obligation during benchmarks

Voltage is recorded alongside temperature. A voltage drop is a leading indicator
of performance degradation and resets.

```text
psu_simple-vin_voltage_v    at measurement start / minimum / mean / end
```

`scripts/collect-node-info.sh` collects it, and it is sampled at 1-second
intervals during a benchmark run.

---

# 9. Thermal characteristics (S0 results)

Being a fanless configuration, these values are the premise for every other
experiment. **Measurement completed with S0 (2026-08-21).** The source is
[`experiments/S0_SUSTAINED_LOAD.md`](experiments/S0_SUSTAINED_LOAD.md).

> ⚠️ **This section was originally a per-node Peak/Sustained FPS table.** At
> planning time the picture was measuring boards separately, but S0 was designed
> as **30 minutes of sustained load at the cluster level.** Per node you get
> temperature, clock and latency; FPS comes out as a cluster total. **Cells that
> were not measured do not get filled in** — the table was rewritten to match
> the actual output structure.

### Cluster (3-node total, 30 minutes)

| Item | B: active cooling | A: fanless |
|---|---:|---:|
| peak | 387.7 inf/s | 389.4 inf/s |
| **steady (last third)** | **380.3 ± 2.2** | **345.4 ± 3.8** |
| **degradation** | **1.9%** | **11.3%** |
| soc max | 58.2 – 61.0 °C | **85.9 – 86.8 °C** |
| npu max | 59.2 – 61.0 °C | **86.8 – 87.8 °C** |
| NPU minimum clock | 950 MHz | 950 MHz (**no downgrade**) |
| Node exclusions | 0 | 0 |
| Error rate | 0 | 0 |

### Per node (they diverge under the fanless condition)

| Item | KING | QUEEN | JACK |
|---|---:|---:|---:|
| Idle starting temperature | 38.8 – 41.6 °C (both conditions; little difference between nodes) |||
| npu max (fanless) | 86.8 °C | 85.9 °C | 86.8 °C |
| **CPU minimum clock (fanless)** | **816 MHz (−63%)** | 1416 MHz (−36%) | 1200 MHz (−46%) |
| p50 latency (fanless) | **156.9 ms** | 66.0 ms | 64.7 ms |
| Request share | 33.3% | 33.3% | 33.3% |

**What is downgraded is the CPU, not the NPU.** And by different amounts per
board — king became **2.4× slower than the other two and round-robin still sends
it one third.** That observation led to the S0-C policy A/B.

Under active cooling there are **zero clock downgrades.** That is why the
60-second results from S2 through S3.9a apply unchanged to sustained operation.

> Not measured: **throttling onset (seconds)** and **time to return to idle**
> were not taken. S0 asked about sustained performance after reaching steady
> state, not about the transient region.

## 9.0 The pilot measurement (not S0, 2026-08-11)

Not the formal S0 (30 minutes) but **a 15-minute measurement for checking
node-to-node thermal spread.** It does not fill in the S0 table. See
`board-worklog.md` §2.19.

Conditions: fixed at 8 threads, 900 s, all three boards started simultaneously,
fanless, no desk fan. **The CPU governor was `ondemand` at the time.** Since
2026-08-12 it has been `performance`, so the throughput figures came out about
7% low (discuss.md §11). Temperatures differ by within 1 °C at idle, so the
thermal conclusions are unaffected.
Tools: `scripts/run-thermal-comparison.sh` + `sustained_load_test`.
Plateau: from 300 s after load to the end (about 557 samples per board).

| Item | `king` | `queen` | `jack` |
|---|---|---|---|
| Idle NPU | 37.0 °C | 35.2 °C | 36.1 °C |
| Plateau NPU mean | 73.0 °C | 67.5 °C | 72.6 °C |
| **Peak NPU** | **75.8 °C** | 70.2 °C | 74.8 °C |
| Plateau SoC mean | 71.2 °C | 65.8 °C | 71.6 °C |
| Minimum input voltage | 5.070 V | 5.090 V | 5.046 V |
| NPU clock | pinned at 950 MHz | pinned at 950 MHz | pinned at 950 MHz |
| Sustained throughput | 80.5 inf/s | 77.7 inf/s | 77.8 inf/s |
| Total inferences (900 s) | 72,481 | 69,928 | 70,049 |
| Mean latency | 99.3 ms | 102.9 ms | 102.8 ms |
| Errors | 0 | 0 | 0 |

**Maximum node-to-node spread 5.6 °C. No NPU throttling** — all 928 samples at
950 MHz, with the NPU clock never dropping once.

What can be settled here:

- The current thresholds (`degraded 80` / `disable 90`) **do not fire** under
  this load. At a peak of 75.8 °C they never reach 80 °C, so no node gets
  arbitrarily excluded. But S0 (30 minutes) could go higher, so §9.2 is still
  decided from S0's results.
- Sustained 8-thread load runs to completion fanless with no errors
- ⚠️ **But the CPU is downgraded by heat.** The verdict above looked only at the
  NPU clock. The CPU clocks in the same log show A72 2208 → 816 MHz and A53
  2016 → 600 MHz. Throughput falls 27% over 300 seconds. `discuss.md` §12
- The three boards' throughput varies by within 3.5%. The premise for
  scaling-efficiency measurement holds.

## 9.1 Measurement conditions

| Item | Value | |
|---|---|---|
| Ambient temperature | not measured | no thermometer. Indirectly estimated from an idle NPU of 35–40 °C |
| Date and time | pilot measurement 2026-08-11 10:48 KST | the formal S0 was not run |
| Spacing between boards | not recorded | to be captured by photograph or measurement |
| Orientation | not recorded | as above |
| Case | **none** (bare boards) | |

## 9.2 The settled temperature thresholds

Decided on the basis of S0's results. They have to be comfortably above the
steady-state temperature so that nodes are not arbitrarily excluded during a
benchmark.

| Configuration key | Value | Basis |
|---|---:|---|
| `degraded_temperature_c` | **80.0** | fanless sustained load reaches soc 85.9–86.8 °C. Set below that to catch degradation as a signal |
| `disable_temperature_c` | **90.0** | **0 node exclusions** across all of S0 — even fanless never reached it |
| Cooldown between repetitions (s) | the harness gates on idle temperature | `preflight-check.sh` checks an idle temperature ceiling (50 °C). Judged by state rather than a fixed time |

> **`disable` at 90 °C has never fired.** Even 31 minutes fanless peaked at
> 87.8 °C. That is, the value is **not verified but merely not reached.** The
> node exclusion behaviour itself remains unverified — `experiments/README.md`
> §7.

Once settled, `configs/scheduler.example.toml` and this table are updated
together.

---

# 10. The scheduler host

Hostname `dealer`. It serves as both the model conversion (Docker) host and the
scheduler.

| Item | Value | Note |
|---|---|---|
| Distribution | **Rocky Linux 9.7** (Blue Onyx) | |
| Kernel | **5.14.0-611.13.1.el9_7.x86_64** | |
| CPU | **Intel i7-4712MQ @2.30GHz, 8 cores** | a 2014 laptop CPU |
| RAM | **3GB** | ⚠️ see §10.1 |
| NIC | **`enp3s0` 1000 Mb/s** | ⚠️ no 2.5GbE |
| Rust | **not installed** | node binaries are built on `king` |

Values measured while the scheduler host is on 1GbE are not used as official
figures. It is where the three nodes' traffic converges, so it saturates first.
See `02-HARDWARE-SETUP.md` §3.3.2.

## 10.1 Constraints needing confirmation

**RAM 3GB.** The scheduler holds request payloads in memory and relays them to
the nodes. At 640×640×3 = 1.17 MiB per request, this is not negligible once the
concurrent count grows.

```text
3 nodes x worker_count 8 = 24 in-flight
+ the scheduler queue + gRPC buffers (both request and response)
-> 1.17 MiB x tens = hundreds of MB
```

Arithmetically there is headroom, but **it has to be confirmed by
measurement.** If it falls short, the payload would have to be streamed or
passed by reference, which is a design change. Scheduler RSS is observed before
the S2 measurement.

**A 1GbE NIC and no PCIe slot.** On INT8 a single node demands **1.545 Gbps.**
In its current state it **cannot even take one node's worth.**

Three nodes' input alone is 4.636 Gbps, and the output is 3.96× the input, so on
`want_float=1` RX goes to **18.38 Gbps.** **2.5G is nowhere near enough and 10G
is needed.**

`dealer` is a laptop and cannot take a PCIe 10G card. **A separate server is
needed.** See `02-HARDWARE-SETUP.md` §3.3.2 and `RESULTS.md` §8.1.

---

## 10.2 The current scheduler host (2026-08-26–)

§10 and §10.1 are records from the `dealer` (laptop) era. Those constraints were
resolved by moving to a server, and the server has since been replaced once
more. **This table holds the values to use for reproduction.**

| Item | Value | Note |
|---|---|---|
| hostname | `server` | SSH alias `npuforge-server` |
| Motherboard | ASUS H81M-K (H81) | a spare desktop, dedicated |
| CPU | **Intel Core i7-4790, 4C/8T, 3.6–4.0 GHz** | ⚠️ the old server was Xeon E5-2630L ×2 (24T) |
| RAM | **16GB DDR3-1600 non-ECC** | |
| Disk | ST2000VN004 2TB, root LVM 70GB | |
| Distribution | **Rocky Linux 9.4** (Blue Onyx) | |
| Kernel | **5.14.0-427.13.1.el9_4.x86_64** | same as the old server |
| glibc | **2.34** | satisfies the requirement for running the frozen binaries |
| NIC | **Intel X550T `enp1s0`**, driver `ixgbe` | 10GBASE-T, 10000 Mb/s full measured. **The same card moved from the old server** (it was `enp4s0`) |
| NIC slot | PCIe **2.0 x4** (`LnkSta 5GT/s x4`) | the H81 x16 slot's limit. 16 Gbps per direction — not a bottleneck |
| Time sync | chronyd active, synchronized | enabled 2026-08-26 |

### The baseline on this host

```text
throughput   ~360 inf/s   (3 runs: 360.5 / 362.5 / 357.2)
round-trip p50  ~93 ms
error rate    0
node spread   ~1.07x
server CPU during measurement 82.2% (across 8 threads) - scheduler 45.3% / bench and kernel 36.9%
```

**The old server's baseline was ~391 inf/s.** The cause of the difference
(−7.5%) is CPU headroom on the scheduler host. The evidence and verdict are in
`infrastructure.md` §3.2.1, and **the raw bench JSON is in
`results/baseline-20260826-althost/`.**

> **The 421 measurements were taken on the old server and stand as recorded.**
> They are not retroactively edited. If measurement continues on the new server,
> **its values are not compared directly with the old server's**; a baseline is
> re-laid here and compared relatively. Exactly as the last sentence of this
> document says — change the combination and it cannot be compared directly with
> the previous one.

---

# 11. Change history

| Date | Item | Previous value | New value | Reason |
|---|---|---|---|---|
| 2026-08-06 | — | — | — | document created |
| 2026-08-06 | SoC | RK3588 | RK3576 | the equipment on hand was confirmed to be a NanoPi R76S |
| 2026-08-06 | Board | NanoPi R6C | NanoPi R76S | as above |
| 2026-08-06 | Cooling | add 3 fans | stay fanless | throttling switched to something to measure |
| 2026-08-06 | Network | 2.5G + 1G | 2.5G × 2 | management network separation becomes the default |
| 2026-08-07 | Board/SoC/NPU/RAM/eMMC | unsettled | settled by measurement | collected with `collect-node-info.sh` after SSH access to the 3 nodes |
| 2026-08-07 | Network ports | unsettled | 2.5G × 2 (`r8125`, separate PCIe) | measured with `ethtool` |
| 2026-08-07 | hostname | `NanoPi-R76S` ×2, `localhost.localdomain` | `king` / `queen` / `jack` | resolved the indistinguishable-node problem |
| 2026-08-07 | NPU core count | unsettled | **2** | differs from RK3588 (3 cores) |
| 2026-08-07 | RKNN Runtime | unsettled | **2.3.0** | SHA-256 identical on 3 nodes |
| 2026-08-07 | RKNPU Driver | unsettled | **v0.9.8** | included in the kernel 6.1.141 BSP |
| 2026-08-07 | Node ID | `r76s-01/02/03` | `king` / `queen` / `jack` | matched to the boards' physical labels |
| 2026-08-26 | Scheduler host CPU | Xeon E5-2630L ×2 (24T) | **Core i7-4790 (8T)** | the old server was physically replaced. Moved to a spare desktop |
| 2026-08-26 | Scheduler host NIC name | `enp4s0` | `enp1s0` | **the card is the same** — the one Intel X550T was pulled from the old server and plugged into the new one. Only the name changes, because the slot differs |
| 2026-08-26 | Baseline throughput | ~391 inf/s | **~360 inf/s** | reduced host CPU headroom (24T→8T). §10.2 · `infrastructure.md` §3.2.1 |
| 2026-08-26 | `h2` (the HTTP/2 implementation) | **0.4.15** | **0.4.19** | RUSTSEC-2026-0258 (unbounded queueing of empty DATA frames, Low). ⚠️ **the 421 measurements were performed on 0.4.15** — see below |

> ## ⚠️ `h2` is not an incidental dependency in this project
>
> **It is the transport layer we measured.** S3.6 A/B'd H2 flow control (window
> size) and S3.7 dealt with connections per node. The entire throughput lineage
> over gRPC came out on top of this crate.
>
> The 421 measurements were performed on **`h2` 0.4.15.** On 2026-08-26 a
> security advisory (RUSTSEC-2026-0258) took `Cargo.lock` to 0.4.19. **The
> numbers are not retroactively edited** — those values were obtained on 0.4.15
> and stand as recorded.
>
> Cloning and building the repository now brings in 0.4.19. Throughput on
> reproduction may differ slightly, and **if it does, that is also a result.**
> The frozen binaries (`*.frozen-01f29a2`) were built with 0.4.15 and are kept
> for comparison.
>
> Ignoring the advisory and pinning the lock was an option and was not taken.
> **A public repository carrying a known vulnerability is worse.**

Change the version combination and benchmark results measured with the previous
combination become directly incomparable. When changing, judge whether
re-measurement is needed at the same time.
