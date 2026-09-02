# NPUDure Development Requirements

*[한국어 원문](03-DEVELOPMENT-REQUIREMENTS.ko.md)*

- Document: `03-DEVELOPMENT-REQUIREMENTS.md`
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
  - `02-HARDWARE-SETUP.md`
  - `environment-matrix.md`

This document is normative for the development environment, tooling, deployment
automation and licensing. The actual pinned values of the version combination
are recorded in `environment-matrix.md`.

---

# 1. Purpose

This document defines the additional software, development environment,
instrumentation, automation, open-source publication preparation and talk
components needed to develop NPUDure v0.1.

It assumes the three NanoPi R76S on hand plus a separate Linux PC, and
prioritises the following over the hardware itself.

- The RKNN model conversion environment
- Rust and C cross-compilation
- The RKNN C wrapper
- The Mock backend
- Benchmark tooling
- Metrics and profiling
- Automated deployment
- License review
- Stabilising the talk demo

---

# 2. Essential development elements

## 2.1 The RKNN model conversion environment

Models are not converted on the NanoPi.

An ONNX or PyTorch model is converted to RKNN format on the development PC and
deployed identically to all three nodes.

```text
PyTorch / ONNX
      |
RKNN-Toolkit2
      |
model.rknn
      |
KING / QUEEN / JACK
```

Recommended setup:

- An Ubuntu x86_64 development environment
- A Python virtual environment
- RKNN-Toolkit2
- ONNX Runtime
- Model conversion scripts
- Calibration images for quantization
- Conversion result verification scripts
- A Docker-based conversion environment where possible

Recommended directory:

```text
tools/model-converter/
├── requirements.txt
├── convert_yolov8n.py
├── calibration.txt
├── validate_onnx.py
├── validate_rknn.py
└── Dockerfile
```

Items that must be managed:

```text
RKNN-Toolkit2 version
RKNN Runtime version
RKNPU Driver version
Python version
ONNX model SHA-256
RKNN model SHA-256
Calibration dataset hash
Conversion options
Quantization scheme
```

The actual pinned values are recorded in `environment-matrix.md`. They cannot be
derived from the code or the git history, which is the sole reason they are
managed in a separate document.

### The conversion target platform

The equipment on hand is the **RK3576**-based NanoPi R76S.

```python
rknn.config(target_platform='rk3576')
```

A `.rknn` file converted for `rk3588` does not work on RK3576. **They are not
compatible across platforms**, so take care not to use reference examples or
existing models as-is.

Confirm the minimum RKNN-Toolkit2 version supporting RK3576 and record it in
`environment-matrix.md` §3. Below the supported version, conversion does not
work at all.

---

## 2.2 The reference model

v0.1 uses exactly one reference model.

Recommended model:

```text
Model       : YOLOv8n INT8
Input       : 640 x 640 RGB
Purpose     : Object Detection
Dataset     : 100-500 public images
Model Hash  : SHA-256
Dataset Hash: SHA-256 manifest
```

Why YOLOv8n first:

- The results can be shown intuitively on a talk screen.
- The input size is easy to fix.
- Concurrent request handling performance is easy to compare.
- There is relatively more RK3576 and RKNN reference material for it.
- Comparing the accuracy of single-node and 3-node results is easy.

Verification targets:

```text
ONNX results
RKNN simulator or Toolkit results
KING results
QUEEN results
JACK results
```

Detection results for the same input have to match within tolerance.

Support for multiple models is not added before the v0.1 talk.

---

## 2.3 The Rust ARM64 build environment

The scheduler and node agent have different build targets.

```text
Scheduler:
  x86_64-unknown-linux-gnu

Node Agent:
  aarch64-unknown-linux-gnu
```

Example packages on an Ubuntu build PC:

```bash
sudo apt install -y \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    pkg-config \
    cmake \
    protobuf-compiler

rustup target add aarch64-unknown-linux-gnu
```

An example `.cargo/config.toml`:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

Recommended order for early development:

1. A successful native Rust build on the R76S
2. A successful connection to the RKNN C wrapper
3. The same source built for ARM64 (cross-compiled or natively on the board —
   §5.3)
4. The same binary deployed to all three nodes
5. Binary SHA-256 verified

Cautions:

- Check the board OS's glibc version
- Check the location of `librknnrt.so`
- Check the dynamic linker path
- Minimise dependence on `LD_LIBRARY_PATH`
- Remove or unify distribution-specific OpenSSL dependencies
- Prefer dependencies that can be statically linked where possible

---

## 2.4 The RKNN C wrapper

Rust does not call the RKNN C API directly and extensively.

A minimal C wrapper is written and Rust uses only a safe wrapper over it.

```text
Rust Application
      |
Safe Rust Wrapper
      |
Rust FFI Module
      |
Minimal C Wrapper
      |
librknnrt.so
```

Example minimal functions:

```c
npf_rknn_create()
npf_rknn_destroy()
npf_rknn_get_model_info()
npf_rknn_infer()
npf_rknn_release_output()
npf_rknn_get_runtime_version()
```

Implementation principles:

- `unsafe` code is confined inside `npuforge-rknn`
- The RKNN context is managed by RAII
- Input and output buffer lifetimes are managed explicitly
- Raw pointers are not exposed to other crates
- A standalone test program for the C wrapper is maintained
- ~~Verify whether concurrent runtime calls are possible~~ → **done.**
  Individual calls are thread-safe but the sequence is not atomic. A context
  pool is mandatory (`environment-matrix.md` §3.1)
- If it is not thread-safe, use a dedicated worker thread per model
- Convert FFI errors into NPUDure error codes

Required tests:

```text
model loading
one inference
1,000 repeated inferences
input errors
model file errors
output buffer release
context cleanup on process exit
multi-thread calls
```

---

# 3. What performance measurement requires

## 3.1 The benchmark client

Repeating `curl` does not produce data at the level a talk or a paper needs.

`npuforge-bench` has to support the following.

```text
concurrency 1 / 4 / 16 / 64
fixed-duration runs
fixed-request-count runs
excluding the warmup section
cycling through input data
input shuffling
storing raw JSONL
storing a CSV summary
p50 / p95 / p99
error rate
retry rate
per-node request distribution ratio
recording the scheduler policy
```

Example invocation:

```bash
npuforge-bench \
  --model yolov8n \
  --dataset ./datasets/coco-sample \
  --concurrency 16 \
  --duration 300 \
  --scheduler ect \
  --output ./benchmarks/results
```

`--scheduler` takes one of three values: `round-robin`, `least-queue`, `ect`.
They have to be identical to the identifiers defined in `01-TECHSPEC.md` §10.0.

An example of a per-request raw result:

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "node_id": "queen",
  "scheduler_policy": "ect",
  "queue_us": 321,
  "network_us": 842,
  "preprocess_us": 1600,
  "inference_us": 18200,
  "postprocess_us": 900,
  "end_to_end_us": 22451,
  "success": true
}
```

The raw data is kept separately from the summary results.

---

## 3.2 Profiling tools

The following are prepared on each node and the scheduler PC.

```text
perf
pidstat
sar
vmstat
iostat
ethtool
iperf3
strace
bpftrace
FlameGraph
```

Essential initial tools:

```text
perf
pidstat
iperf3
ethtool
```

Items to analyse:

- NPU inference time
- Image decoding time
- Preprocessing time
- Postprocessing time
- Memory allocation
- System call count
- Context switches
- Network bandwidth
- CPU utilisation
- Scheduler CPU bottlenecks
- Temperature and thermal throttling
- Per-model runtime variance
- Per-node throughput variance

Recommended profiling order:

1. Confirm single-node inference time
2. Measure preprocessing and postprocessing separately
3. Scheduler CPU profile
4. Node CPU profile
5. Measure network bandwidth
6. Confirm system calls and context switches
7. Confirm buffer allocation and copy sections
8. Decide whether io_uring is needed

---

## 3.3 Metrics collection

Setup:

```text
npuforge-scheduler -> /metrics
npuforge-node      -> /metrics
Prometheus
NPUDure Dashboard
```

Minimum metrics:

```text
requests_total
requests_in_flight
request_latency_seconds
scheduler_queue_seconds
scheduler_route_seconds
inference_seconds
preprocess_seconds
postprocess_seconds
queue_depth
node_temperature_celsius
node_cpu_percent
node_memory_percent
node_npu_percent
node_network_rx_bytes_total
node_network_tx_bytes_total
request_failures_total
request_retries_total
```

Prometheus is used as the raw time-series collector.

The talk screen prefers NPUDure's own dashboard.

Grafana is optional.

---

## 3.4 Power measurement

Not required early on, but needed before the final benchmarks for the paper and
for industry comparison.

Recommended equipment:

- Three USB-C power meters
- Or a meter allowing repeated per-node measurement
- Scheduler and switch power recorded separately

Measured metrics:

```text
Idle Power
Average Load Power
Peak Power
FPS per Watt
Requests per Watt-hour
Cost per FPS
```

Conditions to record during power measurement:

```text
power adapter model
cable length
ambient temperature
node temperature (at measurement start and end)
input voltage
measurement equipment model
sampling interval
```

Being a fanless configuration there is no fan power item. Instead, **power
consumption varies with temperature.** When thermal throttling engages, the
frequency drops and power falls with it, so the same workload gives different
values depending on when it is measured. Always record the starting and ending
temperatures.

See `02-HARDWARE-SETUP.md` §9.

---

# 4. What development stability requires

## 4.1 The Mock backend

Avoid a structure where development requires three real NanoPi to be powered on.

```text
InferenceBackend
├── RKNN Backend
└── Mock Backend
```

Mock backend configuration items:

```text
base inference time
inference time variance
error rate
response delay
queue limit
node failure
a slow node
temperature rise
timeout
```

Example configuration:

```toml
[backend]
type = "mock"
base_latency_ms = 20
jitter_ms = 5
error_rate = 0.02

[worker]
worker_count = 1
max_queue_depth = 32
```

`max_queue_depth` is a node execution setting independent of backend type, so it
goes under `[worker]`. `[backend]` describes only backend-specific items. The
full schema follows `01-TECHSPEC.md` §16.2.

What to verify with the Mock backend:

- Round Robin
- Least Queue
- Estimated Completion Time
- Excluding a failed node
- Automatic recovery
- Retries
- Deadline
- Queue saturation
- Dashboard
- CI integration tests

---

## 4.2 CI and automated testing

CI items:

```text
cargo fmt --check
cargo clippy
cargo test
Mock 3-node integration test
Protocol code generation
x86_64 build
aarch64 cross build
cargo audit
License check
```

Recommended GitHub Actions workflows:

```text
.github/workflows/
├── ci.yml
├── build-arm64.yml
├── security.yml
└── release.yml
```

Hardware testing approach:

- Early: manual testing
- Mid: one R76S can serve as a self-hosted runner
- Before release: full 3-node testing
- Nightly repeat testing is optional

Before the November talk, ordinary CI combined with manual hardware testing is
sufficient.

---

## 4.3 Deployment automation

Copying binaries to three nodes by hand easily produces version mismatches.

Required scripts:

```text
scripts/
├── build-arm64.sh
├── deploy-all.sh
├── start-all.sh
├── stop-all.sh
├── restart-all.sh
├── status-all.sh
├── collect-logs.sh
├── check-versions.sh
├── check-model-hashes.sh
└── run-benchmark.sh
```

`deploy-all.sh`'s processing order:

```text
1. Build the ARM64 binaries
2. Generate SHA-256
3. Copy the same binaries to all three nodes
4. Copy the configuration files
5. Verify the model hashes
6. systemd restart
7. Verify the agent version
8. Verify the runtime version
9. Verify health status
```

With only three nodes, Bash and SSH suffice initially.

Ansible is considered once the repetitive work grows.

---

## 4.4 Serial console and recovery tools

Prepare for the following during development.

- Boot failure
- Network configuration errors
- Kernel panic
- eMMC corruption
- NPU driver problems
- Inability to connect over SSH

Required equipment:

```text
1 or more 3.3V USB-TTL UART adapters
A USB microSD card reader
A recovery microSD
The reference OS image
A spare Ethernet cable
A spare USB-C power adapter
```

Items to record in the management document:

```text
Node ID
MAC Address
IP Address
Hostname
Serial Number
Storage Type
OS Image Version
Kernel Version
RKNPU Driver Version
RKNN Runtime Version
```

---

# 5. Preparing for open-source publication

## 5.1 License structure

Apache License 2.0 is the preferred option for NPUDure's own source code.

Recommended structure:

```text
NPUDure Source        : Apache-2.0
RKNN Runtime          : not included in the repository
RKNN Toolkit          : installed separately from the official source
RKNN Header/Binary    : redistribution terms to be confirmed
model.rknn            : the original model's license to be confirmed
Sample Dataset        : only redistributable material included
```

Required files:

```text
LICENSE
NOTICE
THIRD_PARTY_NOTICES.md
DEPENDENCIES.md
MODEL_LICENSES.md
```

Cautions:

- Do not arbitrarily include the RKNN SDK binary in the NPUDure repository
- Direct users to install the Runtime from the official source
- Confirm the original model's license
- Confirm the redistribution terms of the converted `.rknn` file
- Confirm whether the dataset images can be redistributed
- List the licenses of third-party Rust and C libraries

---

## 5.2 README and installation documentation

Required README content:

```text
An introduction to NPUDure
The core problem statement
Architecture
Mock 3-node quick start
RK3576 installation
Guidance on installing the RKNN Runtime separately
Model conversion
Running benchmarks
Running the failure demo
Reproducing the results
Known limitations
License
```

An external user has to be able to run the core structure with the Mock backend
even without real RK3576 hardware.

Recommended documents:

```text
docs/
├── quick-start.md
├── rknn-installation.md
├── model-conversion.md
├── benchmark-guide.md
├── deployment-guide.md
├── failure-demo.md
├── performance-analysis.md
└── troubleshooting.md
```

---

# 6. Verifying io_uring and zero-copy

## 6.1 Basic principle

io_uring and zero-copy are not conditions for success.

They are applied only when the following hold.

- The network or system calls are confirmed as an actual bottleneck
- The input payload is large enough
- Concurrent requests are numerous enough
- Comparison against the reference implementation under identical conditions is
  possible
- There is an improvement worth the implementation complexity

## 6.2 Prior hardware verification

Confirm the following on the NanoPi.

```bash
ethtool -l eth0
ethtool -k eth0
ethtool -g eth0
ethtool -n eth0
ethtool --show-priv-flags eth0
```

Items to verify:

```text
RX queue count
Header/data split support
Flow steering support
RSS support
Kernel io_uring capability
NIC driver support
Registered buffer support
```

If the R76S NIC or BSP driver does not support zero-copy RX, that implementation
is excluded.

In that case the following alternative optimizations are performed.

```text
Tokio/gRPC
-> Bytes-based shared buffers
-> a buffer pool
-> input buffer reuse
-> reduced memory reallocation
-> an io_uring general I/O comparison
```

## 6.3 Measured metrics

```text
System Calls per Request
Context Switches per Request
CPU Cycles per Request
Memory Allocations per Request
Memory Copies per Request
Requests per Second
p95 Latency
CPU Utilization
```

Below a 5% improvement it is not adopted as a headline feature for the talk.

The reason it had no effect is also recorded as a valid result for the talk.

---

# 7. Additional elements for the talk

## 7.1 Required

```text
A live dashboard
1 / 2 / 3 node comparison
Switching between Round Robin and ECT
Node status cards
Live FPS
p95 latency
Queue depth
Temperature
Automatic exclusion of a failed node
Automatic node recovery
A recorded backup video
Pre-saved benchmark results
Offline execution
```

## 7.2 Recommended

```text
Node status LEDs
Node number labels
A small stand
The NPUDure logo
A GitHub QR code
A live power readout
Object detection video
```

## 7.3 Fallbacks for the talk

- A spare Ethernet cable
- A spare power adapter
- Benchmark results as CSV
- Result graphs as PNG
- A recording of the same demo
- Mock backend mode
- Running in an environment without internet
- Verifying a full reboot scenario before the talk

---

# 8. Development priorities

| Category | Item | Priority |
|---|---|---:|
| Model | the YOLOv8n INT8 reference model | highest |
| Environment | the RKNN Toolkit conversion environment | highest |
| Build | ARM64 Rust/C cross-compilation | highest |
| Integration | the RKNN C wrapper | highest |
| Testing | the Mock backend | highest |
| Measurement | the benchmark CLI | highest |
| Operations | 3-node deployment scripts | high |
| Instrumentation | Prometheus metrics | high |
| Equipment | the 2.5GbE switch and identical power and cooling | high |
| Publication | licenses and third-party notices | high |
| Measurement | USB-C power meters | medium |
| Optimization | verifying io_uring support | medium |
| Optimization | zero-copy experiments | low–medium |
| Talk | dashboard and backup video | October–November |

---

# 9. Five things to do immediately

```text
1. Pin the RKNN Toolkit / Runtime / Driver versions
2. Successful YOLOv8n INT8 single-node inference
3. Successfully calling the RKNN C wrapper from Rust
4. Implement the Mock 3-node scheduler
5. Deploy the same binary to all three with deploy-all.sh
```

With those five complete, most of the core technical risk is removed.

The order after that:

```text
Scheduler
-> failure recovery
-> benchmarks
-> metrics
-> dashboard
-> profiling
-> consider io_uring
-> consider zero-copy
```

---

# 10. Criteria for development readiness

NPUDure v0.1 is judged ready for main development when the following hold.

- The RKNN version combination is pinned
- The reference model is selected
- Model conversion is reproducible
- Single-NanoPi NPU inference succeeds
- The Rust ARM64 build succeeds
- The RKNN C wrapper works
- The Mock backend works
- The three nodes are networked
- The same binary deploys automatically
- Raw benchmark results can be stored
- Metrics can be collected
- The license structure is organised
- The GitHub repository is ready to publish
- The talk schedule and feature freeze date are defined

---

# 11. Final judgement

Developing NPUDure v0.1 is possible with the three NanoPi R76S on hand plus a
separate Linux PC.

What matters most beyond that is not buying new hardware but completeness in the
following.

```text
reproducibility of model conversion
stability of the Rust-to-RKNN integration
a three-node setup in an identical environment
raw benchmark data
failure recovery
automated deployment
license organisation
stability of the talk demo
```

Zero-copy and io_uring are applied only when an actual bottleneck is confirmed
at the final optimization stage.

NPUDure v0.1's success lies not in the theoretical figure of 18 TOPS but in
demonstrating actual scaling efficiency and the causes of loss, reproducibly.
