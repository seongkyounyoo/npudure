<a id="index"></a>

# NPUDure 문서 묶음

> **이 파일은 생성물이다. 직접 편집하지 않는다.**
> `docs/` 의 원본 27개를 읽기·인쇄·검토용으로 이어 붙인 것이다.
> 고칠 것이 있으면 원본을 고치고 다시 만든다.
>
> ```bash
> python scripts/build-docs-bundle.py $(git log -1 --format=%cs -- docs/)
> ```
>
> - 생성 기준: **2026-09-02** (`docs/` 최종 커밋일)
> - 파일 간 링크는 문서 내 앵커로 바뀌어 있다
> - `docs/` 밖을 가리키는 링크(`../results/...`)는 그대로다
> - **세션 인수인계 메모(`handoff-*.md`)와 공개 준비 문서(`public/`)는 빠져 있다.**
>   연구 산출물이 아니다
>
> 아키텍처 결정 기록은 별도 묶음이다 — [`adrs/ALL.md`](../adrs/ALL.md)

## 차례

- [NPUDure Product Requirements Document](#00-prd)  ·  `docs/00-PRD.md`
- [NPUDure Technical Specification](#01-techspec)  ·  `docs/01-TECHSPEC.md`
- [NPUDure Hardware Setup Guide](#02-hardware-setup)  ·  `docs/02-HARDWARE-SETUP.md`
- [NPUDure Development Requirements](#03-development-requirements)  ·  `docs/03-DEVELOPMENT-REQUIREMENTS.md`
- [FAQ](#faq)  ·  `docs/FAQ.md`
- [Glossary](#glossary)  ·  `docs/GLOSSARY.md`
- [NPUDure infrastructure status](#infrastructure)  ·  `docs/infrastructure.md`
- [NPUDure Environment Matrix](#environment-matrix)  ·  `docs/environment-matrix.md`
- [Host inventory](#hosts-readme)  ·  `docs/hosts/README.md`
- [호스트 인벤토리 — server](#hosts-server-i7-4790-20260826)  ·  `docs/hosts/server-i7-4790-20260826.md`
- [Host inventory — Dell PowerEdge R620 (the old scheduler server)](#hosts-server-xeon-e5-2630l-20260826)  ·  `docs/hosts/server-xeon-e5-2630l-20260826.md`
- [Experiment Ledger](#experiments-readme)  ·  `docs/experiments/README.md`
- [S0-C — Scheduling Policy A/B (fanless)](#experiments-s0-c-policy-ab)  ·  `docs/experiments/S0_C_POLICY_AB.md`
- [S0-D — Capacity Heterogeneity (deterministic)](#experiments-s0-d-capacity-hetero)  ·  `docs/experiments/S0_D_CAPACITY_HETERO.md`
- [S0 — Sustained Load (condition A fanless / condition B active cooling)](#experiments-s0-sustained-load)  ·  `docs/experiments/S0_SUSTAINED_LOAD.md`
- [S2 — gRPC Multi-node Scaling Baseline](#experiments-s2-grpc-baseline)  ·  `docs/experiments/S2_GRPC_BASELINE.md`
- [S3.5 — Transport Cost Profiling](#experiments-s3-5-transport-profile)  ·  `docs/experiments/S3_5_TRANSPORT_PROFILE.md`
- [S3.6 — HTTP/2 Window × Connections-per-Node A/B](#experiments-s3-6-h2-channel-ab)  ·  `docs/experiments/S3_6_H2_CHANNEL_AB.md`
- [S3.7 — Connection Tuning (a: sweep, b: concurrency, c: RPS)](#experiments-s3-7-connection-tuning)  ·  `docs/experiments/S3_7_CONNECTION_TUNING.md`
- [S3.8 — Optimized gRPC Scale-out](#experiments-s3-8-optimized-scaleout)  ·  `docs/experiments/S3_8_OPTIMIZED_SCALEOUT.md`
- [S3.9a — Scale-out Efficiency Loss Profiling](#experiments-s3-9a-scaleout-profile)  ·  `docs/experiments/S3_9A_SCALEOUT_PROFILE.md`
- [S3.9b — Node-side Residual Cost Profiling](#experiments-s3-9b-node-residual)  ·  `docs/experiments/S3_9B_NODE_RESIDUAL.md`
- [S3 — Per-configuration Saturation](#experiments-s3-saturation)  ·  `docs/experiments/S3_SATURATION.md`
- [NPUDure measurement results — first pass](#results)  ·  `docs/RESULTS.md`
- [NPUDure 기술 논의](#discuss)  ·  `docs/discuss.md`
- [NPUDure 보드 작업 로그](#board-worklog)  ·  `docs/board-worklog.md`
- [NPUDure status](#todo)  ·  `docs/TODO.md`

---

<a id="00-prd"></a>

# NPUDure Product Requirements Document

*[한국어 원문](00-PRD.ko.md)*

- Document: `00-PRD.md`
- Document version: v0.2
- Project: NPUDure
- Project type: an open-source distributed edge NPU inference runtime
- Language: Rust
- Target platform: Linux / RK3576-based edge devices
- Target talk: FOSS for All Conference, November 2026
- Target public release: NPUDure v0.1
- Status: Draft
- Written: 2026-08-05
- Last modified: 2026-08-06 (body)
- 2026-08-27: banner pointing at the final state added only. **The body stays as the planning baseline**
- Related documents:
  - `01-TECHSPEC.md`
  - `02-HARDWARE-SETUP.md`
  - `03-DEVELOPMENT-REQUIREMENTS.md`
  - `environment-matrix.md`

> ## ⚠️ This document is **the Phase 1 planning baseline**
>
> The research questions, schedule and candidate features written here were
> **fixed before measurement began.** Editing a plan after the fact erases "what
> was expected and what missed", so it stays.
>
> **What actually closed and what is still open is not here.**
>
> | What | Where |
> |---|---|
> | Final experiment state | [`experiments/README.md`](#experiments-readme) §5–§7 |
> | The io_uring verdict (**not adopted**) | [`01-TECHSPEC.md`](#01-techspec) §15 |
> | Settled figures | [`RESULTS.md`](#results) · [`experiments/`](experiments) |
>
> In particular, the **io_uring** this document carries as a comparison
> experiment **was decided against.** S3.9b measured the recoverable share at
> ≈8% and it was excluded on that basis.

---

# 0. Document roles and priority

Where documents state different values, the following priority applies.

| Domain | Normative document |
|---|---|
| Goals, non-goals, functional requirements, success criteria | `00-PRD.md` |
| Repository structure, protocol, config schema, scheduling algorithm, error codes | `01-TECHSPEC.md` |
| Physical setup, network, power, cooling, experimental conditions | `02-HARDWARE-SETUP.md` |
| Development environment, tooling, deployment automation, licensing | `03-DEVELOPMENT-REQUIREMENTS.md` |
| Version combinations and hash pinning | `environment-matrix.md` |

This document covers only "why" and "what".

Algorithm formulas, crate names, configuration keys and identifier strings are
not written here; they reference `01-TECHSPEC.md`. Not duplicating the same
content across two documents is the only way to maintain document consistency.

---

# 1. Project overview

NPUDure is a Rust-based open-source runtime for operating several low-cost edge
NPU devices as a single distributed inference resource.

The first implementation connects three 6 TOPS-class RK3576 NPU devices over a
network and measures how far actual inference throughput scales at two and three
devices against one.

This project does not aim simply to add up each device's nominal TOPS. Its
central goal is to quantitatively measure and analyse the losses arising in a
real application environment from network latency, memory copies, request
scheduling, preprocessing and postprocessing, and node failure.

---

# 2. Problem statement

Edge AI devices are generally operated independently. Even with several devices,
their NPU resources are not unified, so a situation arises where some devices
are overloaded while others sit idle.

Also, the TOPS figure a manufacturer provides is a theoretical maximum
throughput of computation, and does not directly represent a real model's
end-to-end inference performance.

For example, connecting three 6 TOPS NPUs may not reach 18 TOPS in practice
because of the following.

- Network data transfer latency
- Input and output buffer copies
- Image decoding and preprocessing cost
- NPU input memory conversion
- CPU execution of unsupported operators
- Imbalanced request distribution
- Per-node temperature and performance variance
- Insufficient concurrent requests
- Request failures from a failed node
- A central scheduler bottleneck

There is a shortage of public software and experimental material that verifies,
in a reproducible form, the scaling efficiency, cost efficiency and failure
tolerance that appear when several low-cost NPUs are actually connected.

---

# 3. Project goals

## 3.1 Core goals

NPUDure v0.1's core goals are as follows.

1. Configure three RK3576-based NPU nodes as one inference cluster.
2. Have a Rust-based central scheduler distribute inference requests to each
   node.
3. Compare actual throughput and latency across single-, 2- and 3-node
   configurations.
4. Implement dynamic request distribution that accounts for node state and
   queues.
5. Automatically exclude a failed node and re-admit it after recovery.
6. Provide a dashboard for checking system performance and state in real time.
7. Publish the installation method, source code, experimental conditions and
   benchmark results.
8. Present a working demo and the experimental results at the FOSS for All
   Conference, November 2026.

## 3.2 Research goals

The aim is to answer the following questions experimentally.

- By how much does the actual throughput of three 6 TOPS NPUs increase against a
  single device?
- What level is the scaling efficiency as node count rises?
- How much difference is there between plain round robin and load-based
  scheduling?
- What proportion of total latency do the network and memory copies account for?
- Is there a meaningful difference between Tokio-based and io_uring-based
  network handling?
- How does the bottleneck shift with input size and concurrent request count?
- How do service throughput and latency change when a node fails?
- Under what conditions is a low-cost multi-NPU configuration favourable against
  a single high-performance edge accelerator?

---

# 4. Non-goals

NPUDure v0.1 does not aim at the following.

- Hardware-level integration making several NPUs appear as one physical NPU
- Reducing a single inference request's latency in proportion to node count
- Splitting one large model layer-wise across several nodes
- Implementing tensor and pipeline parallelism for large language models
- Kubernetes-level general-purpose cluster orchestration
- Supporting every NPU manufacturer and runtime
- Guaranteeing a fully copy-free data path
- Commercial SLAs and security certification
- Wide-area distributed inference over the internet
- Mobile and Windows client support
- Final-product-level user management and billing

v0.1 concentrates on the data-parallel structure of several NPU nodes handling
independent inference requests in parallel.

---

# 5. Target users

## 5.1 Primary users

### Edge AI developers

Users building multi-camera or multi-request inference systems with low-cost ARM
boards and NPUs.

### Embedded Linux developers

Users interested in Linux networking, device drivers, NPU runtimes and
Rust-based systems software.

### AI systems researchers

Researchers experimenting with distributed edge inference performance, latency,
energy efficiency and scalability.

### Industrial AI solution developers

Developers processing multiple video or sensor streams on site — factories,
equipment, access control systems, CCTV.

## 5.2 Secondary users

- Rust developers
- RK3576 board users
- Open-source contributors
- Graduate students and research labs
- Industrial gateway manufacturers
- Edge AI platform vendors

---

# 6. Main usage scenarios

## 6.1 Multi-image inference

A user sends several images to NPUDure and the scheduler distributes the
requests to available NPU nodes.

```text
Client
  -> Scheduler
      -> NPU Node 1
      -> NPU Node 2
      -> NPU Node 3
```

Each node performs inference independently and returns the result to the
scheduler.

## 6.2 Multi-camera analysis

Frames arriving from several cameras are spread across the NPU nodes.

Expected application areas:

- Factory safety monitoring
- Equipment anomaly detection
- Access headcount analysis
- Object detection
- Defect inspection
- Multi-CCTV analysis

## 6.3 Handling a node failure

If a node stops during cluster operation, the scheduler automatically excludes it
from the request targets.

Once the node is healthy again, it is re-admitted to the cluster after a set
number of successful health checks.

## 6.4 Performance comparison

A user can compare the following configurations with the same model and dataset.

- A single node
- 2 nodes
- 3 nodes
- Round robin scheduling
- Load-based scheduling
- Tokio-based network handling
- io_uring-based network handling
- Before and after data copy optimization

---

# 7. Core functional requirements

## FR-01. NPU node registration

Each NPU node has to register its information with the central scheduler at
startup.

Registration information:

- Node ID
- IP address
- Port
- Device type
- NPU type
- NPU core count
- Model list
- Model version
- Runtime version
- Memory capacity
- Software version

## FR-02. Health checks

The scheduler has to check each node's state periodically.

State categories:

- Registering
- Healthy
- Busy
- Degraded
- Unreachable
- Recovering
- Draining
- Disabled

`Draining` and `Disabled` are states an operator switches to explicitly and are
defined in FR-15.

The transition conditions and thresholds are defined in `01-TECHSPEC.md` §9.

Health check information:

- Response time
- Current queue length
- Recent inference success rate
- CPU utilisation
- Memory utilisation
- NPU utilisation
- Device temperature
- Recent errors

## FR-03. The inference request API

A client has to be able to submit inference requests over a network API.

Request information:

- Request ID
- Model ID
- Input data
- Input format
- Priority
- Request deadline
- Tracing information

Response information:

- Request ID
- Result data
- The node that handled it
- Wait time
- Preprocessing time
- Inference time
- Postprocessing time
- Total processing time
- Error code

## FR-04. Round robin scheduling

The default scheduler has to distribute requests sequentially to healthy nodes.

Round robin serves as the comparison baseline for the other scheduling policies.

## FR-05. Load-based scheduling

The scheduler has to be able to distribute requests based on each node's queue
length and recent processing time.

Basic calculation inputs:

- Current queue length
- Requests being processed
- Moving average inference time
- Moving average network time
- Recent error rate
- Node temperature
- Node state

The policy types, score formula, and configuration and CLI identifiers are
defined in `01-TECHSPEC.md` §10.

In this document, "load-based scheduling" is a prose umbrella term for the
non-round-robin policies and is not used as a configuration value or CLI
argument.

## FR-06. Automatic exclusion of a failed node

A node has to be excluded from request distribution when any of the following
holds.

- Consecutive health check failures
- Inference request deadline exceeded
- Error rate above threshold
- Temperature above threshold
- Abnormal runtime termination

## FR-07. Automatic node re-admission

An excluded node that passes consecutive health checks has to transition through
Recovering back to Healthy.

Immediately after recovery, only limited requests are assigned so that stability
can be confirmed.

## FR-08. Retries

A failed inference request has to be retryable a limited number of times on
another healthy node.

Request ID-based duplicate handling has to be managed to prevent problems from
duplicate execution.

## FR-09. Model management

The scheduler has to be able to determine which models each node can run.

In v0.1 the following take priority over automatic model file deployment.

- Model identification
- Version checking
- Model load state checking
- Per-model request routing
- Model mismatch detection

## FR-10. Metrics collection

The system has to collect the following metrics.

- Total requests/sec
- Per-node requests/sec
- Total FPS
- Per-node FPS
- p50 latency
- p95 latency
- p99 latency
- Error rate
- Retry rate
- Per-node queue length
- CPU utilisation
- Memory utilisation
- NPU utilisation
- Network send and receive volume
- Temperature
- Node availability

## FR-11. A live dashboard

A user has to be able to check system state through a web browser.

Main dashboard views:

- Overall cluster state
- Per-node state
- Total throughput
- Per-node throughput
- Latency distribution
- Current queue length
- Failure and recovery events
- System configuration
- Benchmark execution state

## FR-12. A benchmark execution tool

A CLI tool has to be provided for repeating experiments under identical
conditions.

Example CLI options:

```bash
npuforge-bench \
  --model yolov8n \
  --dataset ./dataset \
  --concurrency 16 \
  --duration 300 \
  --scheduler ect
```

The exact argument list and policy identifiers follow `01-TECHSPEC.md` §10.0 and
`03-DEVELOPMENT-REQUIREMENTS.md` §3.1.

Output formats:

- Console summary
- JSON
- CSV

## FR-13. Per-stage latency measurement

Each request has to record the following times separately.

- Client transmission
- Scheduler queue
- Scheduler routing
- Node reception
- Decode
- Preprocess
- NPU input preparation
- NPU inference
- Postprocess
- Result transmission
- End-to-end latency

## FR-14. Event logs

The following events have to be recorded as structured logs.

- Node registration
- Node connection closed
- Health check failure
- Failed node excluded
- Node recovered
- Request retried
- Deadline exceeded
- Model mismatch
- Scheduler policy changed

## FR-15. Node drain and disable

An operator has to be able to exclude a node from request distribution without
physically stopping it.

- **Drain**: stop assigning new requests and wait for in-flight requests to
  complete.
- **Disable**: exclude from candidates immediately.
- **Enable**: return it to the candidates.

The official benchmarks' 1-, 2- and 3-node comparisons are performed with these
functions, not by cutting power or killing processes.

The experimental conditions only hold if power, temperature, network and
equipment placement stay identical when node count changes. The detailed
rationale follows `02-HARDWARE-SETUP.md` §12.3.

The forced termination and network disconnection in the failure experiments
(§11.6) are separate, and exist to verify the automatic detection behaviour.

---

# 8. Non-functional requirements

## NFR-01. Performance

- A 3-node scaling efficiency of 80% or better — total throughput of 2.4× a
  single node — is the first target.
- A 3-node scaling efficiency of 85% or better is the final target.
- Under normal load, the scheduler's own CPU utilisation must not become the
  system's bottleneck.
- Routing overhead arising in the central scheduler targets 5% or less of total
  latency.

Scaling efficiency is calculated as:

```text
scaling efficiency =
3-node total throughput /
(single-node throughput x 3)
```

The figures in this section are **targets, not success conditions.**

Missing a target is not in itself a failure. The criterion for judging success
follows §12.1.

## NFR-02. Reliability

- A single node's failure must not lead to a full service outage.
- A failed node has to be excluded from request distribution within a configured
  time of detection.
- Failed requests have to be retryable on another node.
- Recovery after a node restart has to happen without manual intervention.

## NFR-03. Reproducibility

- All benchmark conditions have to be storable as configuration files.
- The model, dataset, runtime, kernel and board information used has to be
  recorded.
- Experiments have to be repeatable under identical conditions.
- Raw results have to be preserved as JSON or CSV.

## NFR-04. Portability

The core scheduler must not depend directly on a particular NPU runtime.

The NPU runtime is separated behind a backend interface.

```text
InferenceBackend
  |- RKNN Backend
  |- CPU Mock Backend
  \- Future Backend
```

## NFR-05. Security

v0.1 assumes a closed local network environment.

Minimum requirements:

- Request size limits
- Validation of malformed input
- A node registration token
- Restricted management API access
- No sensitive information in the logs

TLS and user authentication are optional features, but become mandatory on
exposure to an external network.

## NFR-06. Open-source quality

- Apply a clear license.
- Provide installation and execution instructions in the README.
- Provide sample configuration files.
- Provide at least one reproducible demo scenario.
- Provide unit tests for the main modules.
- Set up GitHub Actions or an equivalent CI.

---

# 9. Technical composition

## 9.1 Overall structure

```text
+----------------------+
| Benchmark Client     |
| Demo Web Client      |
+----------+-----------+
           |
           v
+----------------------+
| NPUDure Scheduler    |
|                      |
| . API Gateway        |
| . Node Registry      |
| . Health Monitor     |
| . Load Scheduler     |
| . Metrics Collector  |
+------+------+--------+
       |      |
       v      v
+----------+ +----------+ +----------+
| Node 01  | | Node 02  | | Node 03  |
| RK3576   | | RK3576   | | RK3576   |
| RKNN NPU | | RKNN NPU | | RKNN NPU |
+----------+ +----------+ +----------+
```

## 9.2 Main components

### npuforge-scheduler

The central scheduler.

Main responsibilities:

- Receiving API requests
- Node registration and state management
- Request scheduling
- Retrying failed requests
- Metrics collection
- Event recording

### npuforge-node

The inference agent running on each RK3576 board.

Main responsibilities:

- Loading the RKNN model
- Receiving inference requests
- Preprocessing
- NPU inference
- Postprocessing
- Reporting state and metrics

### npuforge-bench

The performance measurement CLI.

Main responsibilities:

- Generating concurrent requests
- Sending test data repeatedly
- Configuring the load pattern
- Storing results
- Computing basic statistics

### npuforge-dashboard

A web UI displaying cluster state and experimental results.

### npuforge-common

Contains the shared data models, error codes and configuration structures.

### npuforge-mock-backend

A software backend for verifying scheduling, failure detection, recovery, the
dashboard and CI integration tests without a real NPU.

Inference time, variance, error rate and queue limits are adjustable through
configuration.

An external user has to be able to run NPUDure's core structure without RK3576
hardware, so it is treated as an essential component rather than an extra.

The full crate list and their exact names are defined in `01-TECHSPEC.md` §4.

---

# 10. Technology choices

## 10.1 Rust

Why Rust:

- Memory safety for long-running systems software
- Implementing asynchronous network servers
- Low runtime overhead
- Implementing structured concurrency
- FFI integration with the C-based RKNN API
- A high degree of fit with Linux open-source projects

## 10.2 Tokio

Tokio is the default network runtime for v0.1.

The Tokio implementation serves as the performance baseline, and is compared
against an io_uring-based implementation if an actual bottleneck is confirmed.

## 10.3 io_uring

io_uring is not a required feature but a comparison experiment.

Conditions for applying it:

- Network or system call overhead is a measurable bottleneck
- The concurrent request count is high enough
- A comparable experiment against the ordinary Tokio implementation can be
  constructed

## 10.4 Zero-copy

Zero-copy is not a marketing term; it is evaluated by whether the actual copy
count falls.

v0.1 analyses the following path.

```text
Network Buffer
-> User Buffer
-> Decode Buffer
-> Preprocess Buffer
-> NPU Input Buffer
```

Each section is checked for copies, and only removable copies are optimized.

Complete zero-copy is not guaranteed.

## 10.5 RKNN

RK3576 NPU execution uses the RKNN Runtime in the initial version.

To minimise RKNN dependence, the inference backend interface is separated into
its own layer.

---

# 11. Experiment design

## 11.1 Basic experimental setup

- Three identical RK3576 boards
- The same OS and kernel
- The same RKNN Runtime
- The same model
- The same input data
- A 2.5GbE wired network
- The same power and cooling conditions

The detailed physical conditions follow `02-HARDWARE-SETUP.md`.

## 11.2 Node count comparison

- 1 node
- 2 nodes
- 3 nodes

Metrics measured:

- requests/sec
- FPS
- p50
- p95
- p99
- Error rate
- CPU utilisation
- Memory utilisation
- Network usage
- Power consumption
- Temperature

## 11.3 Concurrent request count

The default concurrency conditions are four steps at 4× intervals.

- 1
- 4
- 16
- 64

The concurrency axis multiplies with node count, policy and repetition count, so
each additional step raises total measurement time by hours.

Always calculate the total time before adding an axis. The concurrencies applied
per scenario are defined in `01-TECHSPEC.md` §20.2.

## 11.4 Scheduler comparison

- Round Robin
- Least Queue
- Estimated Completion Time

The three policies' score formulas and their configuration and CLI identifiers
are defined in `01-TECHSPEC.md` §10.

## 11.5 Network implementation comparison

- Tokio TCP or gRPC
- Tokio + a buffer pool
- io_uring
- io_uring + applicable copy optimizations

## 11.6 Failure experiments

- Forcibly killing a node process
- Disconnecting the network cable
- Performance degradation from high temperature
- Model loading failure
- Request processing delay
- Node recovery and re-admission

## 11.7 Cost comparison

Includes the following.

- Hardware purchase cost
- Power consumption
- Cost per throughput
- Cost per FPS
- Operational complexity
- Equipment count
- Number of failure points

---

# 12. Success criteria

## 12.1 Required success criteria

Success is defined not as "did a particular number come out" but as **"can it be
measured and explained."**

In a project whose purpose is measurement, setting a result value as a success
condition creates an incentive to choose experimental conditions favourably when
the target does not come out. That collides head-on with reproducibility, this
project's central value.

So the performance targets sit in §8 NFR-01, and whether they were achieved is
reported as a result rather than as a success condition.

NPUDure v0.1 is judged successful when the following hold.

- Three RK3576 NPU nodes connected
- Single-, 2- and 3-node inference working
- 1/2/3-node scaling factor and scaling efficiency measured, with a quantitative
  account of the bottleneck if the target is missed
- Service continuing when a node fails
- Automatic exclusion and recovery of a failed node
- p50, p95 and p99 measured
- The live status dashboard working
- Source published on GitHub
- Installation and execution documentation published
- Reproducible benchmarks published
- Presentation material and demo ready for the FOSS for All Conference

## 12.2 Recommended success criteria

- 3-node scaling efficiency of 85% or better
- A meaningful improvement from load-based scheduling over round robin
- A quantitative account of io_uring's effect
- Analysis of the data copy count and its cost
- Minimised request failure rate during a node failure
- An automated installation script
- Docker- or systemd-based execution support
- A CPU Mock backend provided
- Documentation sufficient for an external contributor to reproduce the work

## 12.2.1 Thermal characterisation

The NanoPi R76S on hand is a fanless board. No active cooling is added and
**thermal throttling is included in what gets measured.**

Measured results:

- The gap between peak FPS and sustained FPS
- The onset of throttling
- Steady-state temperature and the degradation rate

The TOPS a vendor publishes is instantaneous performance, and there is barely any
public material on a fanless edge device's sustained performance. Measuring that
gap connects directly to this project's problem statement (§2), that "the TOPS
figure does not represent actual throughput".

But **thermal characteristics and scaling efficiency are reported separately.**
Mixed together, there is no way to tell whether a throughput drop came from
scheduling or from temperature.

The detailed design follows `01-TECHSPEC.md` §20.2 S0, and the physical
conditions `02-HARDWARE-SETUP.md` §9.

## 12.3 Results that are meaningful even in failure

The following are also counted as valid technical outcomes.

- io_uring producing no meaningful performance improvement
- Zero-copy applying to only a limited scope
- The NPU or preprocessing, rather than the network, being confirmed as the
  primary bottleneck
- Three-node scaling efficiency being lower than expected
- A single high-performance device being more favourable on cost

Even then, presenting the bottleneck's cause and the conditions of application
quantitatively has value as a talk and as a research result.

---

# 13. Demo scenarios for the talk

## Demo 1. Single node versus three nodes

Compare single-node and three-node throughput live, using the same video or
image dataset.

Displayed:

- Single-node FPS
- Total 3-node FPS
- Scaling factor
- Scaling efficiency
- p95 latency

## Demo 2. Live load distribution

Show requests being distributed across the nodes on the dashboard.

Displayed:

- Per-node queue
- Per-node throughput
- Per-node temperature
- Per-node state
- Request routing status

## Demo 3. Node failure

During three-node operation, kill one node's process or cut its network.

Expected behaviour:

1. Health check failure
2. The failed node automatically excluded
3. Requests redistributed to the remaining two nodes
4. Service continues
5. The node restarts
6. Automatic re-admission after confirming health

## Demo 4. Scheduler comparison

Compare throughput and latency between the round robin and ECT policies.

---

# 14. Schedule

## August 2026

- Write the PRD
- Write the technical specification
- Create the project repository
- Organise the development environment
- Verify single RK3576 RKNN inference
- Minimal Rust FFI validation

## September 2026

- Implement the NPU node agent
- Implement the central scheduler
- Connect single, 2 and 3 nodes
- Implement round robin
- Implement the basic benchmark tool
- Secure preliminary results

## October 2026

- Implement the Least Queue and ECT schedulers
- Implement health checks
- Failed node exclusion and recovery
- Request retries
- Metrics collection
- Implement the dashboard
- Settle the Tokio baseline performance
- Decide whether to apply io_uring

## 1–15 November 2026

- The io_uring comparison experiment
- Data copy analysis
- Final benchmarks
- Cost and power analysis
- Code cleanup
- Write the GitHub documentation
- Stabilise the demo

## 16–22 November 2026

- Feature freeze
- Write the presentation material
- Record the demo video
- Rehearse the talk
- Prepare a backup video in case of failure

## 28 November 2026

- The FOSS for All Conference talk
- NPUDure v0.1 published

---

# 15. Main risks and responses

## Risk 1. RKNN Rust FFI instability

Response:

- Write a minimal C wrapper.
- Isolate the unsafe region in a separate module.
- Manage input and output buffer lifetimes explicitly.
- Run unit tests and repeated inference tests.

## Risk 2. Low performance gain at three nodes

Response:

- Measure NPU computation and preprocessing time separately.
- Compare input data sizes.
- Adjust the concurrent request count.
- Measure network and scheduler overhead.
- Use a negative result as bottleneck analysis material.

## Risk 3. io_uring having little effect

Response:

- Keep Tokio as the default implementation.
- Separate io_uring into an experimental branch.
- If there is no effect, write up the conditions of application and the cause as
  a result.

## Risk 4. Zero-copy implementation difficulty

Response:

- Exclude complete zero-copy from the success criteria.
- Prioritise the buffer pool and memory reuse.
- Measure the reduction in copy count itself.

## Risk 5. Insufficient time before the talk

Response:

- Limit the required demo scope.
- Prioritise benchmark reproducibility over feature completeness.
- Forbid feature additions after 15 November.
- Prepare a recorded video against a live demo failure.

## Risk 6. Scope creep

Response:

v0.1 declines the following requests.

- LLM model parallelism
- Kubernetes integration
- Multi-manufacturer NPU support
- A cloud management service
- A user account system
- Automatic model conversion
- General-purpose AI platform features

---

# 16. Open-source publication plan

## Repository structure

Defined in `01-TECHSPEC.md` §4.

## Published artefacts

- The full source code
- Build instructions
- Installation instructions
- Sample configuration
- Guidance on the test data
- Benchmark execution instructions
- Raw benchmark results
- Architecture documentation
- Known limitations
- Presentation material
- The demo video

## License candidates

Under consideration:

- Apache License 2.0
- MIT License

Considering the patent clause and the potential for corporate use, Apache
License 2.0 is the preferred candidate.

---

# 17. Future extensions

Features that can be considered after v0.1:

- A Hailo backend
- A Jetson TensorRT backend
- An OpenVINO backend
- Automatic model deployment
- Multi-model scheduling
- SLA-based scheduling
- Energy-optimizing scheduling
- Direct camera stream input
- Distributed tracing
- A WebAssembly client
- A Kubernetes device plugin
- Wide-area edge node integration
- Per-request LLM distribution
- An adaptive scheduling algorithm for the doctoral thesis

---

# 18. Final product definition

NPUDure v0.1 is not a product that physically combines three 6 TOPS NPUs into a
single 18 TOPS NPU.

NPUDure is an open-source distributed inference runtime that efficiently
distributes independent inference requests across several edge NPUs, measures
actual performance and bottlenecks, and keeps the service running through node
failures.

The project's core value lies not in a high TOPS figure but in the following.

- Verifying actual scaling efficiency
- Quantitative analysis of the bottleneck
- Comparing throughput against cost
- A fault-tolerant structure
- A reproducible open-source experiment
- Integrating Linux, Rust and edge AI technologies

---

# 19. The talk's core message

> Connecting three 6 TOPS NPUs does not automatically make 18 TOPS.
> NPUDure is an open-source project that measures where that difference arises
> and works out the conditions under which it actually scales.

The talk title:

> **Do three 6 TOPS NPUs really make 18 TOPS?**

---

<a id="01-techspec"></a>

# NPUDure Technical Specification

*[한국어 원문](01-TECHSPEC.ko.md)*

> ## ⚠️ This document is **normative in part and a planning baseline in part**
>
> The structure, protocol and configuration schema are current. The **schedule
> and candidate features** (S5, the November io_uring comparison and so on) are
> plans made before measuring, and are left as they were. Editing a plan after
> the fact erases "what was expected and what missed", so it stays.
>
> **What actually closed and what is still open is not here.**
>
> | What | Where |
> |---|---|
> | Final experiment state | [`experiments/README.md`](#experiments-readme) §5–§7 |
> | The io_uring verdict (**not adopted**) | **§15 of this document** |
> | Settled figures | [`RESULTS.md`](#results) · [`experiments/`](experiments) |
>
> In particular, the **io_uring** this document carries as a comparison
> experiment **was decided against.** S3.9b measured the recoverable share at
> ≈8% and it was excluded on that basis.

- Document: `01-TECHSPEC.md`
- Project: NPUDure
- Document version: v0.2
- Target release: NPUDure v0.1
- Target talk: FOSS for All Conference, November 2026
- Written: 2026-08-05
- Last modified: 2026-08-06 (body)
- 2026-08-27: banner pointing at the final state added only. **The body stays as the planning baseline**
- Status: Draft
- Related documents:
  - `00-PRD.md`
  - `02-HARDWARE-SETUP.md`
  - `03-DEVELOPMENT-REQUIREMENTS.md`
  - `environment-matrix.md`

This document is normative for repository structure, protocol, configuration
schema, scheduling algorithm and error codes. Where values in those areas differ
from another document, this one wins. Physical setup and experimental conditions
follow `02-HARDWARE-SETUP.md`.

---

# 1. Purpose

This document defines NPUDure v0.1's implementation structure, component
responsibilities, communication protocol, data models, scheduling, failure
handling, metrics collection, benchmarking method and deployment structure.

NPUDure v0.1 is a Rust-based open-source runtime operating up to three RK3576
6 TOPS NPU nodes as one distributed inference cluster.

The document's goals are:

1. Let a developer begin implementation without further interpretation.
2. Make the functional scope and non-functional requirements technically
   concrete.
3. Define a reference implementation against which performance can be compared.
4. Secure reproducibility for the talk demo and the research benchmarks.
5. Isolate RKNN-dependent code so other NPU backends can be added later.

---

# 2. Design principles

## 2.1 Data parallelism first

NPUDure v0.1 does not split one model across several nodes.

Each node holds the same entire model and handles different inference requests
independently.

```text
Request A -> Node 1
Request B -> Node 2
Request C -> Node 3
```

The goal is not reducing single-request latency but raising total throughput and
tolerating failure.

## 2.2 Measurable optimization

io_uring, buffer pools, memory reuse and zero-copy are not goals by name.

Every optimization has to satisfy the following.

- A reference implementation exists
- It can be compared under identical experimental conditions
- At least one of throughput, latency, CPU utilisation or copy count is measured
- The result is recorded even when the effect is absent or negative

## 2.3 A simple central scheduler

v0.1 uses a central scheduler structure.

Distributed consensus, leader election and multi-scheduler high availability are
not implemented.

```text
Client
   |
   v
Scheduler
   |-- Node 1
   |-- Node 2
   \-- Node 3
```

## 2.4 Backend separation

Direct RKNN Runtime calls are isolated in the `npuforge-rknn` crate.

The scheduler and node agent use only the common `InferenceBackend` interface.

## 2.5 Reproducibility first

Every experiment has to store the following alongside its results.

- Git commit hash
- OS and kernel version
- RKNN Runtime version
- Model identifier and hash
- Input dataset hash
- Node count
- Network configuration
- Scheduling policy
- Concurrency
- Test duration
- Temperature and power conditions
- The raw measurements

---

# 3. Overall architecture

## 3.1 Logical structure

```text
+-----------------------------------------------------------+
| Clients                                                   |
|                                                           |
|  +----------------+  +----------------+  +-------------+  |
|  | Demo Web Client|  | Benchmark CLI  |  | API Client  |  |
|  +-------+--------+  +-------+--------+  +------+------+  |
+----------+-------------------+------------------+---------+
           |                   |                  |
           +-------------------+------------------+
                               |
                               v
+-----------------------------------------------------------+
| NPUDure Scheduler                                         |
|                                                           |
|  API Gateway                                              |
|  Node Registry                                            |
|  Scheduler Engine                                         |
|  Retry Manager                                            |
|  Health Monitor                                           |
|  Metrics Collector                                        |
|  Event Logger                                             |
+--------------+----------------+----------------+----------+
               |                |                |
               v                v                v
      +----------------+ +----------------+ +----------------+
      | NPUDure Node 1 | | NPUDure Node 2 | | NPUDure Node 3 |
      | RK3576 / RKNN  | | RK3576 / RKNN  | | RK3576 / RKNN  |
      +----------------+ +----------------+ +----------------+
```

## 3.2 Physical setup

The reference hardware is:

- Three RK3576-based NanoPi R76S or equivalent boards
- The same OS, kernel and RKNN Runtime installed on every node
- Wired Ethernet
- The central scheduler running on a separate x86 or ARM Linux machine
- The network is **2.5GbE for workers / 10GbE for aggregation** (rationale below)

### Rationale for the network baseline

**Revised 2026-08-12.** The previous version said "3 nodes at 150 FPS × 1.23 MB
≈ 1.5 Gbps, so 2.5GbE suffices". Two things were wrong — (a) 150 FPS assumed a
**total** across three nodes, whereas measurement puts **one** node at
157.2 inf/s on INT8, and (b) **the output direction was never calculated.** The
figures below are recomputed from measurements.

One input payload is `640 × 640 × 3 = 1,228,800 byte`.

```text
                     per node        3-node total
INT8  157.2 inf/s    1.545 Gbps      4.636 Gbps
FP16   84.3 inf/s    0.829 Gbps      2.486 Gbps
```

**Even FP16's three-node total exceeds a single 2.5GbE link (effectively about
2.35 Gbps).**

That is, **it is the aggregation link, not the worker links, that fills up
first.** Each node uses only its own link (at most 1.545 Gbps), but the three
nodes' traffic converges in front of the scheduler.

- **Worker links: 2.5GbE** — sufficient at 1.545 Gbps per node
- **Aggregation link: 10GbE** — this is the real constraint
- The scheduler host needs a **PCIe slot** for a 10G SFP+ NIC

The response direction uses the link too. The node returns raw tensors without
postprocessing, so with `want_float=1` the response is 3.96× the request and
three-node RX reaches 18.38 Gbps. Even 10G is insufficient. That problem was
solved by switching to `want_float=0` (§16.2, `02-HARDWARE-SETUP.md` §3.3.2,
`adrs/012-want-float-zero-blob-v2.md`).

Taking 1GbE as the baseline would mean measuring link saturation rather than NPU
scaling efficiency under large-input conditions. That does not fit this
project's measurement purpose.

But 1GbE is not removed; it is kept as a **comparison condition** for §20.2's S5
and S6. Presenting "the network is the bottleneck" and "it is not" side by side
has value as a bottleneck-analysis result.

Detailed rationale and topology:
`adrs/014-10g-aggregation-separate-scheduler.md`.

### Constraint on scheduler placement

Official benchmarks do **not** run the scheduler on an RK3576 node.

Loading the scheduler onto one node alone gives the three nodes different
experimental conditions and distorts the 1/2/3-node comparison. Detailed
rationale follows `02-HARDWARE-SETUP.md` §2.1.

For simple development and portable demos it may run on one of the nodes, but
values measured that way are not used as official performance figures.

## 3.3 Process composition

### Scheduler Host

- `npuforge-scheduler`
- `npuforge-dashboard`
- Prometheus
- Optionally Grafana

### NPU Node

- `npuforge-node`
- RKNN Runtime
- Model files
- Hardware metrics collector

### Benchmark Host

- `npuforge-bench`
- Test datasets
- Results directory

---

# 4. Repository structure

```text
npuforge/
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── README.md              (English - default)
├── README.ko.md           (Korean)
├── rust-toolchain.toml
├── crates/
│   ├── npuforge-common/
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── model.rs
│   │   │   ├── protocol.rs
│   │   │   ├── telemetry.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── npuforge-proto/
│   │   ├── proto/
│   │   │   └── npuforge.proto
│   │   ├── build.rs
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   ├── npuforge-scheduler/
│   │   ├── src/
│   │   │   ├── api/
│   │   │   ├── health/
│   │   │   ├── registry/
│   │   │   ├── retry/
│   │   │   ├── scheduler/
│   │   │   ├── telemetry/
│   │   │   ├── state.rs
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── npuforge-node/
│   │   ├── src/
│   │   │   ├── backend/
│   │   │   ├── health/
│   │   │   ├── inference/
│   │   │   ├── metrics/
│   │   │   ├── model_manager/
│   │   │   ├── worker/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── npuforge-rknn/
│   │   ├── include/
│   │   ├── native/
│   │   │   ├── rknn_wrapper.c
│   │   │   └── rknn_wrapper.h
│   │   ├── src/
│   │   │   ├── ffi.rs
│   │   │   ├── backend.rs
│   │   │   ├── buffer.rs
│   │   │   ├── error.rs
│   │   │   └── lib.rs
│   │   ├── build.rs
│   │   └── Cargo.toml
│   ├── npuforge-bench/
│   │   ├── src/
│   │   │   ├── load.rs
│   │   │   ├── output.rs
│   │   │   ├── scenario.rs
│   │   │   ├── statistics.rs
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   └── npuforge-mock-backend/
│       ├── src/lib.rs
│       └── Cargo.toml
├── dashboard/
│   ├── src/
│   ├── public/
│   └── package.json
├── configs/
│   ├── scheduler.example.toml
│   ├── node.example.toml
│   └── benchmark.example.toml
├── deploy/
│   ├── systemd/
│   ├── docker/
│   └── scripts/
├── scripts/
├── tools/
│   └── model-converter/
├── examples/
│   ├── image-classification/
│   └── object-detection/
├── benchmarks/
│   ├── scenarios/
│   ├── results/
│   └── analysis/
├── docs/
│   ├── 00-PRD.md
│   ├── 01-TECHSPEC.md
│   ├── 02-HARDWARE-SETUP.md
│   ├── 03-DEVELOPMENT-REQUIREMENTS.md
│   ├── environment-matrix.md
│   ├── quick-start.md
│   ├── rknn-installation.md
│   ├── model-conversion.md
│   ├── benchmark-guide.md
│   ├── deployment-guide.md
│   ├── failure-demo.md
│   ├── performance-analysis.md
│   └── troubleshooting.md
└── .github/
    └── workflows/
```

## 4.1 Distinguishing scripts from deploy

The two directories are divided so their roles do not overlap.

| Directory | Purpose | Examples |
|---|---|---|
| `scripts/` | operational automation run by developers and experimenters | `build-arm64.sh`, `deploy-all.sh`, `run-benchmark.sh`, `check-versions.sh` |
| `deploy/` | artefacts installed onto target machines, and their installers | systemd units, Dockerfiles, `install-node.sh` |

The full list of `scripts/` follows `03-DEVELOPMENT-REQUIREMENTS.md` §4.3.

The composition of `tools/model-converter/` follows
`03-DEVELOPMENT-REQUIREMENTS.md` §2.1.

---

# 5. Technology stack

## 5.1 Rust

- Rust Stable
- Edition 2024 under consideration
- The minimum supported Rust version is pinned after initial project validation
- A workspace-based multi-crate layout

## 5.2 Async runtime

Reference implementation:

- Tokio
- tonic gRPC
- axum for the management API
- tower middleware

Experimental implementation:

- A separate io_uring-based transport, or a specific data transfer path
- `tokio-uring` or a direct wrapper if needed
- The reference and experimental implementations separated by feature flags

## 5.3 Serialization

- Internal RPC: Protocol Buffers
- Configuration: TOML
- Structured logs: JSON
- Benchmark results: JSON Lines and CSV
- Management API: JSON

## 5.4 Observability

- tracing
- tracing-subscriber
- the metrics or prometheus crate
- A Prometheus endpoint
- OpenTelemetry is optional in v0.1

## 5.5 Web dashboard

Choose one of the following.

Preferred:

- React or a simple TypeScript SPA
- Using the scheduler's REST API with WebSocket/SSE

Simplified:

- Rust + askama or minijinja
- An htmx-based UI

Since the talk schedule takes priority, the choice is made on implementation
speed rather than features.

---

# 6. Components in detail

## 6.1 npuforge-common

Provides the shared types and configuration.

Main types:

```rust
pub type NodeId = String;
pub type RequestId = uuid::Uuid;
pub type ModelId = String;
pub type ModelVersion = String;
```

Main modules:

- `config`: the TOML configuration structures
- `error`: the shared error codes
- `model`: model identification and metadata
- `protocol`: shared request/response data models
- `telemetry`: metric types

Dependencies are kept minimal.

## 6.2 npuforge-proto

Contains the gRPC service definitions and generated code.

Services:

- `SchedulerService`
- `NodeService`
- `ControlService`

Protocol changes reserve fields rather than deleting them, for version
compatibility.

## 6.3 npuforge-scheduler

The central control component.

Responsibilities:

- Receiving external inference requests
- Node registration and removal
- Health checks
- Scheduling
- Retries
- Timeouts
- Metric aggregation
- Event emission
- Providing the dashboard API

Internal state is kept in memory in the initial version.

Items needing persistence:

- Benchmark results
- Event logs
- Configuration snapshots

PostgreSQL is not required for v0.1.

## 6.4 npuforge-node

Runs on each NPU device.

Responsibilities:

- Registering with the scheduler
- Model loading
- Handling inference requests
- Preprocessing and postprocessing
- Managing the local work queue
- Reporting metrics
- Reporting status
- Graceful shutdown

The node's internal worker count is configurable, subject to model and RKNN
Runtime constraints.

## 6.5 npuforge-rknn

The crate dedicated to RKNN Runtime integration.

Responsibilities:

- The C API FFI
- A safe Rust wrapper
- Creating and releasing model contexts
- Input and output buffer management
- Invoking inference
- Error conversion
- Querying model metadata

`unsafe` code is confined inside this crate.

## 6.6 npuforge-bench

The load generation and statistics tool.

Responsibilities:

- Fixed concurrency
- Gradual load ramp
- Fixed request count
- Fixed duration tests
- Cycling through input data
- Storing results
- Summary statistics
- Failure rate calculation

---

# 7. APIs and communication protocol

## 7.1 The external inference API

The reference API is gRPC.

### Infer

```protobuf
rpc Infer(InferRequest) returns (InferResponse);
```

Example structures:

```protobuf
message InferRequest {
  string request_id = 1;
  string model_id = 2;
  bytes payload = 3;
  string input_format = 4;
  int32 priority = 5;
  int64 deadline_unix_ms = 6;
  map<string, string> metadata = 7;
}

message InferResponse {
  string request_id = 1;
  string node_id = 2;
  bytes result = 3;
  string result_format = 4;
  Timing timing = 5;
  string error_code = 6;
  string error_message = 7;
}
```

### BatchInfer

Optional in v0.1.

Client-side batching and node-internal batching have to be distinguished.

## 7.2 The node registration API

```protobuf
rpc RegisterNode(RegisterNodeRequest) returns (RegisterNodeResponse);
rpc Heartbeat(HeartbeatRequest) returns (HeartbeatResponse);
rpc DeregisterNode(DeregisterNodeRequest) returns (DeregisterNodeResponse);
```

Information passed at registration:

```protobuf
message NodeDescriptor {
  string node_id = 1;
  string hostname = 2;
  string address = 3;
  string device_type = 4;
  string npu_type = 5;
  uint32 npu_core_count = 6;
  uint64 memory_bytes = 7;
  string agent_version = 8;
  string runtime_version = 9;
  repeated ModelDescriptor models = 10;
}
```

## 7.3 The node inference API

Called by the scheduler on a node.

```protobuf
service NodeService {
  rpc Infer(NodeInferRequest) returns (NodeInferResponse);
  rpc Health(HealthRequest) returns (HealthResponse);
  rpc ListModels(ListModelsRequest) returns (ListModelsResponse);
  rpc Warmup(WarmupRequest) returns (WarmupResponse);
}
```

## 7.4 The management API

Provided over REST.

Examples:

```text
GET  /api/v1/cluster
GET  /api/v1/nodes
GET  /api/v1/nodes/{node_id}
GET  /api/v1/models
GET  /api/v1/metrics/summary
GET  /api/v1/events
POST /api/v1/scheduler/policy
POST /api/v1/nodes/{node_id}/drain
POST /api/v1/nodes/{node_id}/enable
```

## 7.5 The metrics API

```text
GET /metrics
```

Exposed in Prometheus format.

---

# 8. Main data models

## 8.1 NodeRecord

```rust
pub struct NodeRecord {
    pub descriptor: NodeDescriptor,
    pub state: NodeState,
    pub last_heartbeat_at: Instant,
    pub consecutive_health_failures: u32,
    pub consecutive_health_successes: u32,
    pub queue_depth: u32,
    pub in_flight: u32,
    pub ewma_inference_ms: f64,
    pub ewma_network_ms: f64,
    pub error_rate: f64,
    pub temperature_c: Option<f64>,
    pub cpu_percent: Option<f64>,
    pub memory_percent: Option<f64>,
    pub npu_percent: Option<f64>,
}
```

## 8.2 NodeState

```rust
pub enum NodeState {
    Registering,
    Healthy,
    Busy,
    Degraded,
    Unreachable,
    Recovering,
    Draining,
    Disabled,
}
```

## 8.3 InferenceTask

```rust
pub struct InferenceTask {
    pub request_id: RequestId,
    pub model_id: ModelId,
    pub payload: bytes::Bytes,
    pub input_format: InputFormat,
    pub priority: i32,
    pub deadline: Option<Instant>,
    pub created_at: Instant,
    pub attempt: u32,
    pub max_attempts: u32,
}
```

## 8.4 TimingBreakdown

```rust
pub struct TimingBreakdown {
    pub scheduler_queue_us: u64,
    pub scheduler_route_us: u64,
    pub network_to_node_us: u64,
    pub node_queue_us: u64,
    pub decode_us: u64,
    pub preprocess_us: u64,
    pub npu_input_us: u64,
    pub inference_us: u64,
    pub postprocess_us: u64,
    pub network_to_client_us: u64,
    pub end_to_end_us: u64,
}
```

---

# 9. The node state machine

## 9.1 State transitions

```text
Registering
   | registration success
   v
Healthy --------------\
   | high load        | manual drain
   v                  v
Busy                Draining
   | errors           | queue empty
   v                  v
Degraded            Disabled
   | health fail
   v
Unreachable
   | health success
   v
Recovering
   | consecutive success
   \---------------> Healthy
```

## 9.2 Default thresholds

Initial defaults:

- Heartbeat interval: 2 s
- Health timeout: 1 s
- 3 consecutive failures: `Unreachable`
- 3 consecutive successes: `Recovering` to `Healthy`
- Queue length above threshold: `Busy`
- Recent error rate above 10%: `Degraded`
- Temperature at or above 80 °C: `Degraded`
- Temperature at or above 90 °C: excluded from scheduling

Every value has to be configurable.

---

# 10. Scheduling

## 10.0 Policy identifiers

The policy identifiers are fixed at the following three. The configuration file,
CLI arguments, metric labels, logs and dashboard all use **the same strings**.

| Identifier | Policy | Purpose |
|---|---|---|
| `round-robin` | Round Robin | comparison baseline |
| `least-queue` | Least Queue | intermediate comparison |
| `ect` | Estimated Completion Time | recommended default |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchedulingPolicyKind {
    RoundRobin,
    LeastQueue,
    Ect,
}
```

Identifiers are parsed only through the Rust enum and serde; string comparisons
are not scattered through the code.

Notations such as `queue-aware`, `estimated-completion-time` and `queue_aware`
are not used. "Load-based scheduling" and "queue-aware" are used only as prose
umbrella terms for the non-`round-robin` policies, never as identifiers.

## 10.1 The scheduler interface

```rust
pub trait SchedulingPolicy: Send + Sync {
    fn select_node(
        &self,
        task: &InferenceTask,
        candidates: &[NodeSnapshot],
    ) -> Result<NodeId, ScheduleError>;
}
```

## 10.2 Round Robin

The reference implementation.

Conditions:

- Only `Healthy` and `Busy` states are candidates
- Only nodes with the requested model loaded are candidates
- `Draining`, `Disabled` and `Unreachable` are excluded

Advantages:

- Simple to implement
- Provides a comparison baseline

Disadvantages:

- Does not reflect per-node processing speed or queue state

## 10.3 Least Queue

Selects the node with the smallest queue.

On a tie, the following apply in order:

1. Lower in-flight count
2. Lower mean inference time
3. Node ID ordering

## 10.4 Estimated Completion Time

The recommended policy.

Estimated cost:

```text
ECT =
((queue_depth + in_flight + 1) x EWMA_inference_time
 + EWMA_network_time
 + thermal_penalty
 + error_penalty)
/ load_factor
```

The node with the lowest score is selected.

### Why `+ 1`

It counts not only waiting requests but the request being assigned right now.

ECT estimates "when will this request finish", so its own inference time has to
be included. Also, if a node with an empty queue scores 0, the `load_factor`
correction below is neutralised.

### Why `load_factor`

A per-state weight.

| State | load_factor |
|---|---:|
| Healthy | 1.0 |
| Busy | 1.0 |
| Degraded | 0.5 |
| Recovering | 0.25 |
| Otherwise | 0.0 (excluded from candidates) |

A `Recovering` node has an empty queue, so on score alone it always wins. Taking
the full load right after recovery risks failing again from the same cause, so
PRD FR-07's "assign only limited requests" requirement is implemented as a
single score rather than a separate counter.

### Tie-breaking

On equal scores, decided by lexicographic Node ID. If tie-breaking wavers, the
results of repeated experiments under identical conditions differ and
reproducibility breaks.

### A shared candidate filter

All three policies pass through the same candidate filter.

- Must be in an `is_schedulable()` state
- Must hold the requested model in a `Ready` state
- Temperature must be below `disable_temperature_c`

If the filters differed per policy, §20.2's S3 policy comparison would measure
filter differences rather than policy differences.

## 10.5 Priority

v0.1 supports a simple integer priority.

- 0: normal
- 10: high
- −10: low

Wait-time-based aging may be applied to prevent starvation.

## 10.6 Deadline

For requests carrying a deadline, nodes whose estimated completion time exceeds
the deadline may be excluded from the candidates.

If no node can meet the deadline, one of the following is chosen.

- Return `DEADLINE_UNSATISFIABLE` immediately
- Send best-effort to the fastest node

The default policy is best-effort.

---

# 11. Request handling flow

## 11.1 The normal flow

```text
1.  Client sends an Infer request
2.  Scheduler validates the request
3.  Request ID generated or validated
4.  Candidate nodes queried
5.  Scheduling policy executed
6.  Request sent to the selected node
7.  Node enqueues it locally
8.  Preprocessing
9.  RKNN inference
10. Postprocessing
11. Node returns the result
12. Scheduler records metrics
13. Response to the client
```

## 11.2 The failure flow

```text
1. Node call fails or times out
2. Failure cause classified
3. Retryability checked
4. attempt incremented
5. Failed node temporarily excluded from candidates
6. Another node selected
7. Retry
8. Error returned once the maximum is exceeded
```

## 11.3 Duplicate handling

Inference requests have no side effects in principle, so retrying is possible.

The scheduler keeps a short-TTL Request ID cache to detect duplicate
submissions.

v0.1 does not require implementing a result cache as well.

---

# 12. Retries and timeouts

## 12.1 Kinds of timeout

- Client request timeout
- Scheduler queue timeout
- Node RPC timeout
- Node local queue timeout
- Inference timeout

## 12.2 Retryable errors

- Network connection failure
- Node timeout
- Transient runtime errors
- Node overloaded
- Node unavailable

## 12.3 Non-retryable errors

- Invalid input
- Unsupported model
- Unsupported input format
- Model version mismatch
- Payload size exceeded
- Authentication failure

## 12.4 Defaults

- Maximum retries: 1
- Node RPC timeout: configured per model
- Overall request timeout: 5 s
- Retry backoff: a short delay in the 10–100 ms range

Given the real-time inference character, long exponential backoff is not used.

---

# 13. Model management

## 13.1 The model directory

Each node specifies its model directory in the configuration file.

```text
/opt/npuforge/models/
├── yolov8n/
│   ├── model.rknn
│   ├── model.toml
│   └── labels.txt
└── mobilenet_v3/
    ├── model.rknn
    └── model.toml
```

## 13.2 Model metadata

Example:

```toml
id = "yolov8n"
version = "1.0.0"
backend = "rknn"
model_file = "model.rknn"
input_width = 640
input_height = 640
input_channels = 3
input_format = "rgb8"
output_format = "yolo-detections"
sha256 = "..."
```

## 13.3 Model states

```rust
pub enum ModelState {
    Unloaded,
    Loading,
    Ready,
    Failed,
    Draining,
}
```

## 13.4 Warmup

A per-model warmup count is configurable at node startup.

By default 3 are performed before real requests are accepted.

Warmup results are excluded from benchmarks.

---

# 14. The RKNN backend

## 14.1 The InferenceBackend interface

```rust
#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn load_model(
        &self,
        spec: &ModelSpec,
    ) -> Result<Box<dyn LoadedModel>, BackendError>;

    fn backend_name(&self) -> &'static str;
    fn runtime_version(&self) -> Result<String, BackendError>;
}

#[async_trait::async_trait]
pub trait LoadedModel: Send + Sync {
    async fn infer(
        &self,
        input: InferenceInput,
    ) -> Result<InferenceOutput, BackendError>;

    fn model_info(&self) -> &LoadedModelInfo;
}
```

If the RKNN call itself blocks, `spawn_blocking` or a dedicated worker thread is
used.

## 14.2 The C wrapper

A minimal C wrapper keeps Rust from coupling tightly to the RKNN headers.

The wrapper's responsibilities:

- Context creation and release
- Model load
- Input set
- Run
- Output get
- Output release
- Simplifying error codes

## 14.3 Memory lifetimes

Safety rules:

- The RKNN context is managed by RAII
- Output buffer release is guaranteed in Drop
- Raw pointers are not exposed outside the FFI module
- Input buffers live until the inference call returns
- **Confirmed 2026-08-11.** Individual calls in RKNN Runtime 2.3.0 are
  thread-safe.
- **But the `inputs_set → run → outputs_get` sequence is not atomic.** Several
  threads using the same context produce **100% wrong results with 0 API
  errors** (4 threads × 50, 200/200 mismatched). See the correction in
  `environment-matrix.md` §3.1.
- Therefore **as many contexts are created as there are concurrent executions,
  each occupied one at a time.** `npuforge-rknn`'s `ContextPool` handles this,
  and `RknnContext::infer` takes `&mut self` so the compiler blocks concurrent
  calls.

## 14.4 Buffer pool

The recommended v0.1 implementation:

- Buffer pools by input size
- Reusing output structures
- Reusing image decoding buffers
- Minimising per-request Vec reallocation

Benchmark before and after applying the buffer pool.

---

# 15. io_uring and data copy optimization

> ## ⛔ Verdict: **not adopted** (2026-08-21, S3.9b)
>
> Measuring steps 2 and 3 of §15.1 (CPU profile, syscall and copy cost) at the
> operating point, **two of §15.3's non-applicability conditions actually
> fired.**
>
> | §15.3 condition | Measured |
> |---|---|
> | gRPC serialization is the larger bottleneck | **fired.** user 9.37 > kernel 6.99 ms/req — the majority of transport cost is serialization and user-space copies |
> | Improvement under 5% | **fired.** Syscall entry is **1.0%** of transport cost, and 8% under the most generous assumption (eliminating the 1.2 MB copy in both directions) |
>
> And more fundamentally — **board CPU is not the constraint.** It is 48.9% idle
> under load and no core is saturated. Even the hottest, cpu0 (softirq 68.3%),
> gave a **−0.2% null** when spread with RPS (S3.5 §4.3).
> Reducing usage of an unsaturated resource does not raise throughput.
>
> ```text
> Question   Does io_uring recover the remaining 16.1%?
> Answer     No. What it targets is 1% (8% generously), and CPU is not the constraint.
> ```
>
> The status is **"refuted by measurement", not "necessity unproven".**
> Every measurement item in §15.4 has been filled in.
> → [`experiments/S3_9B_NODE_RESIDUAL.md`](#experiments-s3-9b-node-residual)
>
> §15.1–15.4 below remain as **the plan used on the way to that verdict.**
> They reopen if conditions change (if shrinking the payload makes CPU the
> constraint, for instance) — exclusions are conditional
> (`experiments/README.md` §4.5).

## 15.1 Staged application

1. The Tokio/gRPC reference implementation
2. Measure the CPU profile
3. Confirm network syscall and copy cost
4. Apply a buffer pool
5. Experiment with io_uring
6. Consider registered buffers or zero-copy where possible

## 15.2 Candidate scopes

- Benchmark Client → Scheduler
- Scheduler → Node
- Receiving large image payloads
- Reading file-based datasets
- Storing results

## 15.3 Possible non-applicability

io_uring may not be applied under the following conditions.

- NPU inference occupies most of the total time
- The input data is small
- gRPC serialization is the larger bottleneck
- A final copy into the RKNN input buffer is unavoidable
- The improvement is under 5% against the implementation complexity

## 15.4 Measurement items

- syscalls/request
- context switches/request
- CPU cycles/request
- memory copies/request
- requests/sec
- p95 latency
- CPU utilization

---

# 16. Configuration files

## 16.1 Scheduler configuration

`configs/scheduler.example.toml`

```toml
[server]
grpc_listen = "0.0.0.0:50051"
http_listen = "0.0.0.0:8080"
metrics_listen = "0.0.0.0:9090"
max_payload_bytes = 10485760

[scheduler]
policy = "ect"
request_timeout_ms = 5000
node_rpc_timeout_ms = 3000
max_retries = 1

[health]
heartbeat_interval_ms = 2000
health_timeout_ms = 1000
failure_threshold = 3
recovery_threshold = 3

[thresholds]
busy_queue_depth = 8
degraded_error_rate = 0.10
degraded_temperature_c = 80.0
disable_temperature_c = 90.0

[telemetry]
json_log = true
log_level = "info"
prometheus = true
```

## 16.2 Node configuration

`configs/node.example.toml`

```toml
[node]
id = "king"
scheduler_address = "http://10.20.0.10:50051"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.21:51001"

[worker]
worker_count = 8          # measured, settled. environment-matrix.md §3.1
max_queue_depth = 32
queue_timeout_ms = 3000
want_float = false        # whether to dequantize the output. Default false

[backend]
type = "rknn"
runtime_library = "/usr/lib/librknnrt.so"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]
warmup_runs = 3

[telemetry]
json_log = true
log_level = "info"
prometheus = true
heartbeat_interval_ms = 1000
temperature_path = "/sys/class/thermal/thermal_zone0/temp"
```

`worker_count` and `max_queue_depth` go under `[worker]`, not `[node]`.

With `want_float = false` the node returns the model's native dtype as-is (int8
for an INT8 model). The basis for the `false` default is not throughput but
**the network** — with `true` the output is 3.96× the input, and at three-node
saturation the scheduler's RX reaches 18.38 Gbps
(`02-HARDWARE-SETUP.md` §3.3.2). So that the receiver can dequantize, **the
response blob is v2 and carries `qnt_type`, `scale` and `zero_point` per
tensor.** As a side effect throughput rises too, by INT8 +17.3% / FP16 +15.7%.

`[node]` covers only node identity and addressing, with execution concurrency
split into a separate section. Differences between nodes should be limited to
the three values in `[node]` (`id`, `advertise_address`, hostname), and this
separation makes configuration diff verification simple.

The example IPs use `10.20.0.0/24`, the official range in
`02-HARDWARE-SETUP.md` §3.2.

## 16.3 Benchmark configuration

`configs/benchmark.example.toml`

```toml
[target]
scheduler_address = "http://10.20.0.10:50051"
model_id = "yolov8n"

[load]
mode = "fixed-duration"
duration_seconds = 300
concurrency = 16
request_timeout_ms = 5000

[data]
directory = "./datasets/coco-sample"
shuffle = true
repeat = true

[output]
directory = "./benchmarks/results"
formats = ["jsonl", "csv"]
```

---

# 17. Metrics

## 17.1 Scheduler metrics

```text
npuforge_requests_total
npuforge_requests_in_flight
npuforge_requests_failed_total
npuforge_requests_retried_total
npuforge_request_latency_seconds
npuforge_scheduler_queue_seconds
npuforge_scheduler_route_seconds
npuforge_nodes_total
npuforge_nodes_healthy
npuforge_nodes_unreachable
npuforge_node_queue_depth
npuforge_node_in_flight
npuforge_node_error_rate
```

## 17.2 Node metrics

```text
npuforge_node_requests_total
npuforge_node_inference_seconds
npuforge_node_preprocess_seconds
npuforge_node_postprocess_seconds
npuforge_node_temperature_celsius
npuforge_node_cpu_percent
npuforge_node_memory_percent
npuforge_node_npu_percent
npuforge_node_network_rx_bytes_total
npuforge_node_network_tx_bytes_total
```

## 17.3 Labels

Permitted labels:

- node_id
- model_id
- status
- scheduler_policy
- error_code

Request ID is not used as a metric label.

---

# 18. Logs and events

## 18.1 Log format

Structured JSON logs by default.

```json
{
  "timestamp": "2026-09-12T10:00:00.000Z",
  "level": "INFO",
  "component": "scheduler",
  "event": "request_completed",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "node_id": "queen",
  "model_id": "yolov8n",
  "latency_ms": 31.2
}
```

## 18.2 Main events

- scheduler_started
- node_registered
- node_state_changed
- model_loaded
- request_received
- request_scheduled
- request_completed
- request_failed
- request_retried
- node_removed
- node_recovered
- benchmark_started
- benchmark_completed

---

# 19. Dashboard

## 19.1 Required views

### Cluster Overview

- Total node count
- Healthy/Busy/Degraded/Unreachable counts
- Total throughput
- p50/p95/p99
- Error rate
- The current scheduler policy

### Node View

- Node ID
- State
- Queue length
- in-flight
- FPS
- Mean inference time
- Temperature
- CPU/RAM/NPU utilisation
- Recent errors

### Benchmark View

- The current scenario
- Elapsed time
- Target concurrency
- Live throughput
- Latency
- Success/failure counts
- Per-node distribution ratio

### Event Timeline

- Failure detection
- Node exclusion
- Recovery
- Policy changes

## 19.2 Live transport

SSE or WebSocket.

Considering demo stability for the talk, SSE is looked at first.

---

# 20. Benchmark design

## 20.1 The reference model

The v0.1 default model is a lightweight object detection model with stable RKNN
conversion and real-time performance.

Leading candidates:

- YOLOv8n
- MobileNet-family classification models

The final model is selected on the following conditions.

- Runs identically on all three nodes
- Minimal CPU fallback
- Fixed input size
- Results are verifiable
- Licensing and distribution terms are clear

## 20.2 Test scenarios

### Fixing the axes

Measurement combinations grow multiplicatively, so the axes are fixed first.

```text
default concurrency axis: 1, 4, 16, 64   (4 steps at 4x intervals)
default node count axis : 1, 2, 3
reference policy        : ect
comparison point        : concurrency 16, 3 nodes
repetitions             : 5
```

Concurrency uses **4 steps at 4× intervals** rather than 7 steps at 2×. Four
steps suffice for seeing the shape of the scaling curve, and adding steps raises
the total measurement time to an infeasible level (see §20.3).

Policy and implementation comparisons are not repeated across the whole
concurrency axis but performed at **a single point, concurrency 16 with 3
nodes**. That point is where the nodes begin to saturate but the queues do not
run away, and it can be adjusted from preliminary results.

The 5 repetitions are not reduced. Reproducibility is this project's central
output, so if time has to be cut, the number of axes is cut rather than the
repetitions.

### S0. Thermal characterisation (a prerequisite, two conditions)

The NanoPi R76S is a fanless board, so thermal throttling occurs under sustained
load. Every other scenario's thresholds and cooldown times come from this
result, so **it goes first.**

**Two cooling conditions are each measured** (`02-HARDWARE-SETUP.md` §9.1).

| Condition | Cooling | Purpose |
|---|---|---|
| **S0-A** | fanless | sustained performance in a real edge deployment |
| **S0-B** | 3 identical fans | the ceiling without throttling |

The difference between the two results is **"the effect of cooling on scaling
efficiency"**, which is itself material for the talk.

- Node count: 1 (the other two nodes confirm reproducibility)
- Concurrency: fixed at 16
- Duration: **1,800 s** (30 min). Until steady-state temperature is reached
- Temperature thresholds: disabled (`disable_temperature_c` set very high during
  measurement)
- Sampling: 1-second intervals
- **Premise: the three nodes' physical placement is uniform.** In the 2026-08-10
  measurement, placement differences alone produced a 19 °C spread between nodes

Measured items:

```text
temperature curve over time
FPS curve over time
throttling onset (the point where FPS falls 5% or more below steady state)
peak FPS (before throttling)
sustained FPS (steady state)
degradation = 1 - (sustained FPS / peak FPS)
steady-state temperature
time to return to idle
CPU/NPU frequency changes
```

Outputs:

- The settled `degraded_temperature_c` / `disable_temperature_c` for use in all
  later scenarios
- The settled cooldown time
- **The peak vs sustained performance gap** — a figure absent from vendor spec
  sheets, and one of the talk's central materials

Performed on all three nodes to check unit-to-unit variance. If the variance is
large, later per-node comparisons have to account for it.

### S1. Single-node baseline

- Node count: 1
- Concurrency: 1, 4, 16, 64
- Policy: `round-robin`
- Purpose: measure single-node maximum throughput and secure the denominator for
  the scaling factor
- Premise: uses the thresholds and cooldown settled in S0

### S2. Scalability

- Node count: 1, 2, 3
- Concurrency: 1, 4, 16, 64
- Policy: `ect`
- Purpose: measure scale-out efficiency
- Note: 60 runs combined with S1

### S3. Scheduler comparison

- Policies: `round-robin`, `least-queue`, `ect`
- Node count: fixed at 3
- Concurrency: fixed at 16
- Purpose: confirm throughput and p95 differences between policies

### S4. Failure handling

- Three nodes operating normally
- One node force-killed
- Failure detection
- Operating on two nodes
- Node recovery
- Re-admission

### S5. Network implementation comparison

- Implementations: Tokio/gRPC, Tokio/gRPC + buffer pool, an experimental
  io_uring implementation, copy optimization applied
- Node count: fixed at 3
- Concurrency: 16, 64
- Purpose: quantify the effect of optimizing the network path

### S6. Input size comparison

- Inputs: small JPEG, large JPEG, raw RGB
- Node count: fixed at 3
- Concurrency: fixed at 16
- Link: 2.5GbE baseline, with a 1GbE comparison added for the raw RGB condition
- Purpose: confirm how the bottleneck moves with input size, and secure the
  basis for the network baseline (§3.2)

## 20.3 Experiment duration

Each run consists of:

- Warmup: 30 s
- Measurement: 300 s
- Cooldown between repetitions: **at most 180 s**, or until the starting
  temperature is reached, whichever comes first
- Repetitions: 5

```text
1 run ~ 30 + 300 + 180 = 510 s ~ 8.5 min  (worst case at the cooldown cap)
```

On a fanless board 60 seconds of cooldown is insufficient. The cap is 180 s, and
when the cap is hit the actual starting temperature is recorded with the result.
Waiting indefinitely would break the total budget and is not permitted.

The exact cooldown value is settled from S0's measurement of the time to return
to idle.

## 20.4 Total measurement time budget

The total is calculated before the axes are fixed.

| Scenario | Combinations | Runs | Duration |
|---|---|---:|---:|
| **S0-A** (fanless) | 3 nodes × 1,800 s + cooldown | 3 | about 1.8 h |
| **S0-B** (cooled) | 3 nodes × 1,800 s + cooldown | 3 | about 1.8 h |
| S1 + S2 | 3 nodes × 4 concurrencies × 5 repetitions | 60 | about 8.5 h |
| S3 | 3 policies × 5 repetitions | 15 | about 2.1 h |
| S4 | failure scenario × 5 repetitions | 5 | about 0.7 h |
| S5 | 4 implementations × 2 concurrencies × 5 repetitions | 40 | about 5.7 h |
| S6 | 3 inputs × 5 repetitions + 5 for the 1GbE comparison | 20 | about 2.8 h |
| **Total** | | **146** | **about 23.4 h** |

**S1–S6 are performed under one cooling condition only.** Repeating all of it
under both would come to 46 hours and be infeasible.

**The default measurement condition is decided after seeing S0's results.** If
fanless throttling is severe enough to contaminate scaling-efficiency
measurement, the cooled condition becomes the default and fanless is presented
through S0's results alone.

The decision to stay fanless raised cooldown from 60 to 180 seconds and the
total budget from 16 to 22 hours. Assuming unattended overnight runs between
1 and 15 November it is still manageable, but the margin has shrunk. If S0's
actual time to return to idle comes in below 180 seconds, the budget is
recalculated.

For reference, putting concurrency on 7 steps (1/2/4/8/16/32/64) and running S2
and S3 over the full combination would make those two scenarios alone 315 runs
and about 45 hours — infeasible.

### Requirements for unattended execution

Sixteen hours in total cannot be run interactively.

`run-benchmark.sh` has to satisfy the following.

- Take the scenario list from a file and execute in sequence
- Handle cooldown and thermal stabilisation between runs automatically
- Record and continue rather than aborting everything when an individual run
  fails
- Flush the raw results immediately at the end of each run
- Collect the reproducibility metadata (§2.5) automatically
- Be resumable from the point of interruption

## 20.5 Result calculations

### Scaling factor

```text
scale_factor(N) =
throughput(N nodes) / throughput(1 node)
```

### Scaling efficiency

```text
scale_efficiency(N) =
throughput(N nodes) /
(throughput(1 node) x N)
```

### Throughput per cost

```text
cost_efficiency =
throughput / total_hardware_cost
```

### Throughput per energy

```text
energy_efficiency =
requests / watt-hour
```

## 20.6 Statistics

- Mean
- Median
- Standard deviation
- p50
- p95
- p99
- Minimum/maximum
- A 95% confidence interval where possible

---

# 21. Error codes

Examples:

```text
NPF-0000 OK
NPF-1001 INVALID_REQUEST
NPF-1002 PAYLOAD_TOO_LARGE
NPF-1003 UNSUPPORTED_INPUT_FORMAT
NPF-1101 MODEL_NOT_FOUND
NPF-1102 MODEL_VERSION_MISMATCH
NPF-1201 NO_AVAILABLE_NODE
NPF-1202 DEADLINE_UNSATISFIABLE
NPF-1301 NODE_TIMEOUT
NPF-1302 NODE_UNAVAILABLE
NPF-1303 NODE_OVERLOADED
NPF-1401 BACKEND_ERROR
NPF-1402 INFERENCE_FAILED
NPF-1501 INTERNAL_ERROR
```

Error codes are kept stable in the external API.

---

# 22. Security

The v0.1 baseline premise is a trusted local network.

Minimum implementation:

- A maximum payload size limit
- Input format validation
- A node registration token
- A management API token
- No raw images or sensitive data in the logs
- Directory traversal prevention
- A model path allowlist
- The processes run as a dedicated non-root user

Optional implementation:

- mTLS
- TLS
- API keys
- Model signature verification

---

# 23. Deployment

## 23.1 systemd

The default deployment method.

Services:

```text
npuforge-scheduler.service
npuforge-node.service
npuforge-dashboard.service
```

## 23.2 Docker

The scheduler and dashboard can support Docker.

The node defaults to a host installation, because of the RKNN Runtime and device
access.

## 23.3 Installation scripts

```bash
./deploy/scripts/install-scheduler.sh
./deploy/scripts/install-node.sh
./deploy/scripts/install-dashboard.sh
```

The scripts are written to be idempotent.

---

# 24. Test strategy

## 24.1 Unit tests

- Scheduling score calculation
- State transitions
- Retry decisions
- Configuration parsing
- Error conversion
- Statistics calculation

## 24.2 Integration tests

Using the Mock backend.

Scenarios:

- Registering 3 mock nodes
- Differing processing speeds
- Differing error rates
- Node failure
- Recovery
- Timeout
- Retry

## 24.3 Hardware tests

Performed on real RK3576 hardware.

- Model loading
- Repeated inference stability
- One hour of sustained load
- Temperature rise
- Network disconnection
- Forced process termination
- Automatic registration after restart

## 24.4 CI

GitHub Actions:

- fmt
- clippy
- unit tests
- mock integration tests
- build
- dependency audit

RKNN hardware tests are separated onto a self-hosted runner or run manually.

---

# 25. Performance profiling

Candidate tools:

- perf
- flamegraph
- bpftrace
- strace
- sar
- pidstat
- ethtool
- iperf3
- tcpdump
- valgrind massif, used sparingly if needed

Measurement steps:

1. Scheduler CPU flamegraph
2. Node CPU flamegraph
3. Network throughput
4. Syscall frequency
5. Context switches
6. Memory allocation
7. Copy sections
8. NPU inference time
9. Temperature and throttling

---

# 26. Development milestones

## M0. Repository and environment

Completion criteria:

- Rust workspace created
- Basic CI
- Document structure
- License
- Mock backend working

## M1. Single-node inference

Completion criteria:

- RKNN FFI
- Model loading
- Inference on one image
- Repeated inference
- Timing measurement

## M2. Remote inference

Completion criteria:

- Scheduler → Node gRPC
- Single-node remote inference
- Error handling
- Basic metrics

## M3. Multiple nodes

Completion criteria:

- 3 nodes registered
- Round Robin
- 1/2/3-node benchmarks

## M4. Dynamic scheduling

Completion criteria:

- Least Queue
- ECT
- Policy comparison

## M5. Failure recovery

Completion criteria:

- Health checks
- Automatic exclusion
- Request retries
- Automatic re-admission

## M6. Dashboard

Completion criteria:

- Live node state
- Throughput
- Latency
- Failure events

## M7. Optimization experiments

Completion criteria:

- Buffer pool
- Profile results
- The io_uring adoption decision
- Comparison against the reference implementation if adopted

## M8. Talk release

Completion criteria:

- The v0.1 tag
- README
- Installation scripts
- Raw benchmark data
- Presentation material
- A demo video

---

# 27. Schedule

## August 2026

- `00-PRD.md`
- `01-TECHSPEC.md`
- Repository initialisation
- Mock backend
- Minimal Rust-RKNN FFI validation

## September 2026

- Single-node remote inference
- 3 nodes registered
- Round Robin
- The benchmark CLI
- Preliminary results

## October 2026

- Least Queue and ECT scheduling
- Health checks
- Failure exclusion and recovery
- Metrics
- Dashboard
- Baseline performance settled

## 1–15 November 2026

- Profiling
- Buffer optimization
- io_uring experiments
- Final benchmarks
- Documentation

## 16–22 November 2026

- Feature freeze
- Presentation material
- Demo video
- Rehearsal

## 28 November 2026

- The FOSS for All Conference talk
- NPUDure v0.1 published

---

# 28. Scope control

The following are not added before the November talk.

- Kubernetes integration
- Multi-scheduler HA
- PostgreSQL-based state storage
- User accounts
- Billing
- Automatic model conversion
- Automatic model deployment
- LLM tensor parallelism
- Model layer partitioning
- Hailo/Jetson backends
- WAN clusters
- A mobile app
- Complex permission management

Additional requests move to the v0.2 backlog.

---

# 29. Demo composition for the talk

## 29.1 Main screen

- The NPUDure logo and version
- Node 1/2/3 state
- Per-node FPS
- Total FPS
- p95 latency
- Scaling factor
- Scaling efficiency
- Temperature
- Queue length

## 29.2 Demo sequence

1. Run a single node
2. Add a second node
3. Add a third node
4. Confirm the throughput increase
5. Compare Round Robin against ECT
6. Kill Node 2's process
7. Automatic exclusion
8. Service continues on two nodes
9. Restart Node 2
10. Automatic re-admission

## 29.3 Fallbacks

- A recording of the same scenario
- Pre-generated benchmark results
- A local simulation that works without a network
- Mock backend mode

---

# 30. Definition of done

NPUDure v0.1 is considered complete when all of the following hold.

- Three RK3576 NPU nodes operating
- The Rust scheduler operating
- Single/2/3-node benchmarks
- Three-node scaling factor and scaling efficiency measured, with cause analysis
  if the target is missed
- Round Robin, Least Queue and ECT compared
- Automatic node failure detection
- Failed nodes excluded
- Service continues
- Automatic node re-admission
- p50/p95/p99 provided
- Raw results published
- Installation documentation published
- Published on GitHub
- A stable demo suitable for the talk
- Ready for the FOSS for All Conference talk, November 2026

---

# 31. Final technical definition

NPUDure is not a technology for physically combining several edge NPUs.

NPUDure is a Linux/Rust-based open-source distributed inference runtime that
spreads independent inference requests across several NPU nodes, schedules those
requests according to each node's load and state, keeps the service running when
a failure occurs, and measures real performance loss and scaling efficiency
reproducibly.

---

<a id="02-hardware-setup"></a>

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

---

<a id="03-development-requirements"></a>

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

---

<a id="faq"></a>

# FAQ

Answers to the questions this project actually gets asked. Every answer states
the number, the conditions it was measured under, and where the raw data is.
Where something is unmeasured, it says so instead of estimating.

> **NPUDure is an open-source Edge NPU Cluster runtime for scaling distributed
> AI inference across low-cost NPUs over standard Ethernet.**

**The measurement base for everything below**

| | |
|---|---|
| Nodes | 3 × NanoPi R76S — Rockchip RK3576, 6 TOPS NPU each |
| Network | 2.5 GbE per node |
| Scheduler host | Dell PowerEdge R620, dual Xeon E5-2630L, 24 threads, 10 GbE |
| Workload | YOLOv8n INT8, 640×640×3 RGB in, raw tensor blobs out |
| Transport | gRPC over HTTP/2 (tonic), plaintext |
| Runs | **421 valid runs, zero inference errors** |
| Scale tested | **1, 2 and 3 nodes. 4+ is not measured.** |

Ledger and raw-data map: [`docs/experiments/README.md`](#experiments-readme).

---

## 1. What is an Edge NPU Cluster?

**Several low-cost edge NPU boards, each running a full copy of the model,
fronted by a scheduler that spreads independent inference requests across them
over ordinary Ethernet.**

It is data parallelism at the request level. Each board does whole inferences;
nothing is split across boards. The cluster raises how many requests per second
you can serve. It does not make the boards behave as one larger accelerator.

The contrast worth drawing is with model parallelism — layer-wise partitioning
or LLM tensor parallelism — where one request is split across devices and the
interconnect sits inside the model's critical path. That is a different problem
with different interconnect requirements, and it is not what this is.

| | |
|---|---|
| Source | [`README.md`](../README.md) § What this is, and isn't |

---

## 2. Can multiple NPUs be combined for inference?

**For throughput, yes, and it works well. For a single request, no.**

Independent requests distribute cleanly: three nodes served 3.00× the requests
one node did, with even distribution and no errors. But any one request is
still handled start to finish by one NPU, so the time that request takes is
unchanged — see [question 4](#4-does-npudure-reduce-single-request-latency).

"Combining NPUs" is two different asks that get one name. This project answers
the throughput one and is explicit about not answering the other.

| | |
|---|---|
| Measured | 112.9 → 338.4 inf/s going from 1 to 3 nodes |
| Conditions | YOLOv8n INT8, concurrency 8/node, active cooling, 30 runs |
| Source | [S2](#experiments-s2-grpc-baseline) · raw: [`results/baseline-20260820/`](../results/baseline-20260820) |

---

## 3. Does 3 × 6 TOPS equal 18 TOPS?

**No. But on this configuration the throughput scaling was 3.00×, which is
closer to the naive sum than we expected.**

Two separate things get conflated here.

**TOPS do not add.** 18 TOPS is a datasheet arithmetic that assumes zero
distribution cost and perfectly divisible work. Neither holds.

**Throughput did scale near-linearly anyway** — but from a baseline that has
already paid for being a cluster. One board doing local direct inference
reaches **161.5 inf/s**. That same board serving through the cluster's gRPC
path reaches **112.9 inf/s** — a **30.1% throughput reduction** before any
scale-out happens. Three nodes then multiply that reduced baseline by 3.00×.

So the honest phrasing is: near-linear scaling on top of a real per-node
distribution cost, not 18 TOPS.

| | |
|---|---|
| Measured | 112.9 ± 0.5 / 229.0 ± 0.9 / 338.4 ± 1.1 inf/s at 1 / 2 / 3 nodes → **3.00×** |
| | Local direct 161.5 inf/s vs cluster single-node 112.9 inf/s → **−30.1%** |
| Conditions | Concurrency 8 per node, active cooling, 30 runs, error rate 0 |
| Source | [S2](#experiments-s2-grpc-baseline) · raw: [`results/baseline-20260820/`](../results/baseline-20260820) |

---

## 4. Does NPUDure reduce single-request latency?

**No, and it cannot. It raises throughput and adds latency to each request.**

A request goes to exactly one node. Adding nodes gives you more requests in
flight at once; it does nothing for the one in front of you. The distribution
path — serialization, a 1.2 MB payload each way, HTTP/2 framing, the network —
is added to every request. The 1.2 MB round trip alone costs roughly **8.2 ms**
on a 2.5 GbE link.

If single-request latency is what you need, a cluster is the wrong tool.

| | |
|---|---|
| Conditions | 640×640×3 RGB request = 1,228,800 B; response blobs ≈ 1,218,000 B |
| Source | [`README.md`](../README.md) § What this is, and isn't · [ADR-008](../adrs/008-grpc-tonic-protobuf.md) |

---

## 5. How well does RK3576 scale?

**3.00× at three nodes — about 98.9% scaling efficiency — on the untuned
baseline.**

Tuning the transport raised absolute throughput and *lowered* efficiency at the
same time. Connection tuning took three-node throughput from 341.8 to
**387.2 inf/s (+13.3%)**, while scaling efficiency fell from **98.9% to 95.3%**
(3.00× → 2.86×).

Both numbers are true and they move in opposite directions. Quoting either one
alone misrepresents the result, so this repository always reports both with the
operating point attached.

| | |
|---|---|
| Measured | Baseline 338.4 inf/s, 3.00×, eff 98.9% |
| | Tuned 387.2 inf/s, 2.86×, eff 95.3% (+13.3% throughput) |
| Conditions | 3 nodes, YOLOv8n INT8, concurrency 12/node at the operating point |
| Source | [S2](#experiments-s2-grpc-baseline) · [S3.8](#experiments-s3-8-optimized-scaleout) · raw: [`results/scaleout-optimized-20260820/`](../results/scaleout-optimized-20260820) |

---

## 6. Why gRPC instead of RDMA?

**RDMA was never a candidate, and the measurements argue that a lower-overhead
transport would not have bought much here.**

Being straight about this: [ADR-008](../adrs/008-grpc-tonic-protobuf.md)
compared three options — REST+JSON, gRPC, and a hand-rolled binary protocol.
RDMA was not among them and was not benchmarked, so this is not a measured
comparison. gRPC won on binary payloads without base64 inflation, schema
enforcement, and code generation.

What *is* measured bears on the question indirectly. Profiling the transport
for a planned io_uring port found that transport CPU is a cost, not a
constraint — the boards were **48.9% idle under load with no core saturated**.
Cutting transport overhead reduces consumption of a resource that was never the
limit. See [question 9](#9-why-wasnt-io_uring-implemented).

The design goal was also explicit: ordinary Ethernet and a standard transport,
so the result transfers to hardware people already own. Whether RDMA-class
interconnect changes the picture on this workload is **unmeasured**.

| | |
|---|---|
| Measured | Board CPU 48.9% idle under load, no core saturated |
| Source | [ADR-008](../adrs/008-grpc-tonic-protobuf.md) · [S3.9b](#experiments-s3-9b-node-residual) |

---

## 7. What limits scale-out efficiency?

**The tail. Median latency does not move at all; p95 and p99 do.**

Going from one node to three at the tuned operating point:

```text
p50   flat        (+0%)
p95   119.7 → 147.4 ms   (+23%)
p99   137.9 → 187.6 ms   (+36%)
```

The mean rises with the tail, and under closed-loop load a higher mean is
directly less throughput. Ideal three-node scaling from the 135.5 inf/s
operating point would be 406.5 inf/s; the measured value was 387.2 —
**19.3 inf/s short**, and that shortfall is entirely tail-shaped.

**Separately, there is a per-node gap we did not explain.** Local direct
inference reaches 161.5 inf/s against 135.5 at the operating point — a
**16.1% residual**. It looks like path latency rather than CPU cost, but we did
not pin it down, and `perf` is unavailable on these boards (vendor kernel), so
there is no symbol-level profile.

| | |
|---|---|
| Measured | p50 +0%, p95 +23%, p99 +36% going 1 → 3 nodes |
| | Residual per-node gap 161.5 → 135.5 inf/s = 16.1%, unexplained |
| Conditions | Tuned transport, operating point, 3–4 runs per configuration |
| Source | [S3.9a](#experiments-s3-9a-scaleout-profile) · [S3.9b](#experiments-s3-9b-node-residual) |

---

## 8. Why did load-aware scheduling perform worse?

**It wasn't the policy. Our scheduler was herding on stale heartbeat state — a
bug in our own default configuration.**

Load-aware policies *collapsed* throughput by **55–58%** against plain round
robin. The policies were deciding from heartbeat data that had already gone out
of date, so every scheduler instance picked the same "idle" node at the same
moment and piled onto it.

After switching the decision input to a locally-tracked in-flight counter,
adaptive scheduling **cut p99 latency by 37%** and evened the per-node latency
spread from **1.33× to 1.00×**.

The lesson generalises past this codebase: a load-aware policy is only as good
as the freshness of what it reads. It took a policy A/B to find this — the
symptom looked like "load-aware scheduling doesn't work here".

| | |
|---|---|
| Measured | Stale state: −55 to −58% throughput vs round robin |
| | After fix: p99 −37%, node spread 1.33× → 1.00× |
| Conditions | 3 nodes, deliberate heterogeneity fixture |
| Source | [S0-C](#experiments-s0-c-policy-ab) · raw: [`results/policy-ab-20260821/`](../results/policy-ab-20260821) |

---

## 9. Why wasn't io_uring implemented?

**We profiled it, found the reachable gain was about 8% of transport cost, and
did not build it.**

The plan was: profile CPU, measure syscall and copy cost, then implement
io_uring. We did the first two and the numbers ended the third.

```text
transport cost          16.35 CPU-ms per request
  user   9.37 ms (57%)  serialization, user-space copy, HTTP/2 framing
  kernel 6.99 ms (43%)  syscall entry, TCP stack, copy_to_user

network syscalls        ~165 per request
syscall entry           ~0.17 ms = 1.0% of transport cost
board CPU under load    48.9% idle, no core saturated
```

Even granting that io_uring eliminates the 1.2 MB copy in both directions, the
total reachable slice is about **8%** of transport cost. And recovering it buys
nothing, because **CPU here is a cost, not a constraint** — reducing
consumption of an unsaturated resource does not raise throughput.

This is a conditional exclusion, not a permanent verdict. If the workload
becomes CPU-bound on the boards, it reopens.

| | |
|---|---|
| Measured | Reachable slice ≈ 8% of a 16.35 CPU-ms/request transport cost |
| Conditions | RK3576 boards under load, 48.9% CPU idle, `/proc/PID/stat` split |
| Source | [S3.9b](#experiments-s3-9b-node-residual) · raw: [`results/node-residual-20260821/`](../results/node-residual-20260821) |

---

## 10. Can NPUDure scale beyond three nodes?

**We do not know. Four or more nodes was never measured, and we are not going
to extrapolate 3.00× into a claim about node four.**

Three nodes is what the hardware budget covered. Everything in this repository
is bounded by that.

Two measured facts do point at where the next wall probably is, and neither is
the NPU:

- **The scheduler host is a real bottleneck.** Thread count on it matters more
  than clock speed. When the host was swapped, a faster-per-core 8-thread
  desktop produced **7.5% less** throughput than the 24-thread server, because
  the load generator shares that CPU with the scheduler. Sixteen threads or
  more is what we would recommend.
- **Traffic converges at the scheduler.** Three nodes at 2.5 GbE each converge
  on one host; 2.5 GbE there is not enough, which is why the scheduler sits on
  10 GbE.

So the honest expectation is that scale-out runs into scheduler host capacity
and its network before it runs into the boards — but **that is reasoning, not a
measurement**, and it is exactly the kind of claim this repository otherwise
refuses to make.

If you run this on four or more nodes, we would like to see the data.

| | |
|---|---|
| Measured | 8-thread desktop scheduler host: −7.5% throughput vs 24-thread server |
| Unmeasured | 4+ nodes, at any configuration |
| Source | [`docs/02-HARDWARE-SETUP.md`](#02-hardware-setup) §3.3.4 · [`docs/infrastructure.md`](#infrastructure) §3.2.1 |

---

## Reading the numbers in this file

Four caveats apply to everything above, and they are the same ones in the
README limitations:

- **Three nodes only.** Whether any conclusion holds at four or more is
  unmeasured.
- **Most configurations are 3–4 runs.** Percentile differences have small SD
  and are usable. **Throughput differences under 1% were never used to rank
  anything.**
- **Percentiles are run-level averages, not pooled.** This dilutes each run's
  worst window, so tail numbers read low. Valid for comparing conditions,
  invalid as "the p99 of this system".
- **No authentication, no TLS.** Scoped for a trusted private network. A
  boundary, not a defect.

---

<a id="glossary"></a>

# Glossary

*[한국어 원문](GLOSSARY.ko.md)*

- Last updated: **2026-08-21**
- Scope: every term that actually appears in the S2–S4 experiment lineage. Not
  just definitions — **what value or judgement each is tied to in this project**
  is written alongside.
- Related: [`experiments/README.md`](#experiments-readme) (the experiment
  ledger), [`01-TECHSPEC.md`](#01-techspec), [`RESULTS.md`](#results)

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

---

<a id="infrastructure"></a>

# NPUDure infrastructure status

*[한국어 원문](infrastructure.ko.md)*

- Document: `infrastructure.md`
- Last updated: 2026-08-20
- Related: `board-worklog.md` (chronological work history),
  `environment-matrix.md` (version pinning)

This document is **a snapshot of the current state.** How that state was reached
is in `board-worklog.md`.

> **The 2026-08-20 rework.** Introducing the 2.5G/10G switch and the 10G server
> cleared every blocker for M3, and the IPs and roles changed substantially. The
> previous arrangement (the dealer laptop + a 1G management network) is retired.
> The story is in `board-worklog.md` §2.23.

---

# 1. Equipment

```text
                    +----------------------------------+
                    | server   192.168.123.9           |
                    | Rocky Linux 9.4 / x86_64         |
                    | Core i7-4790 (4C/8T) / 16GB      |
                    |                                  |
                    | . Scheduler (planned)            |
                    | . Benchmark Client (planned)     |
                    +----------------+-----------------+
                                     | 10GbE (enp1s0)   <- aggregation
                                     |
                    +----------------v-----------------+
                    | NEXI NS-S25G10G-N                |
                    | 2.5G x4 + 10G x2 (all RJ45)      |
                    +-+----+------+------+------+------+
              10G ----+    |2.5G  |2.5G  |2.5G  +--2.5G-- internet (ipTIME)
          dev PC (laptop)  |      |      |
          (1G NIC, unused) |      |      |
                    +------v+ +---v---+ +v------+
                    | king  | | queen | | jack  |
                    |  .3   | |  .5   | |  .4   |
                    | 6 TOPS| |6 TOPS | |6 TOPS |
                    +-------+ +-------+ +-------+
                       Ubuntu 24.04 / RK3576 / aarch64
                       eth0 2.5G each, static
```

| Host | IP | Role | OS | Arch | Switch port |
|---|---|---|---|---|---|
| `server` | 192.168.123.9 | Scheduler / Bench | Rocky Linux 9.4 | x86_64 | **10G (6)** |
| `king` | 192.168.123.3 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (2) |
| `jack` | 192.168.123.4 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (4) |
| `queen` | 192.168.123.5 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (3) |
| dev PC | 192.168.123.26 | writing code / remote operation | Windows | x86_64 | 10G (5) — **1G NIC** |
| internet | — | ipTIME upstream | — | — | 2.5G (1) |

> **Static IPs completed (2026-08-20).** The rework changed the board IPs
> wholesale (`.12/.16/.33` → `.3/.4/.5`), leaving stale SSH aliases unable to
> find the nodes. Rather than router DHCP reservations, **the current IP is
> pinned as NetworkManager static on each host** (host configuration is better
> for measurement reproducibility). **All four are `ipv4.method=manual`**
> (§2.3). `adrs/019-ssh-alias-not-ip.md`.

**`dealer` (the old scheduler, laptop .14) has been removed.** No response. Its
roles (scheduler and bench) moved to `server`. The model conversion Docker also
lived on dealer, so the conversion environment is due for rebuilding (§6).
Though the model is already converted, so it is not needed immediately.

The dev PC is plugged into the switch's 10G port (5) but **its NIC is 1G
(currently negotiating 100 Mb/s) and cannot do 10G.** The bench client runs on
`server`, not on the dev PC.

---

# 2. Access

## 2.1 SSH aliases

Registered in the dev PC's `~/.ssh/config`. **The IP lives only here**
(`adrs/019-ssh-alias-not-ip.md`).

```text
npuforge-k        -> pi@192.168.123.3     (king)
npuforge-q        -> pi@192.168.123.5     (queen)
npuforge-j        -> pi@192.168.123.4     (jack)
npuforge-server   -> root@192.168.123.9   (server)
```

All connect without a password using the `~/.ssh/id_ed25519_npuforge` key.

> This key is for automation only and has no passphrase. Do not expose it in a
> public repository or on an untrusted network.

## 2.2 Privilege escalation

| Host | Account | sudo | Note |
|---|---|---|---|
| king / queen / jack | `pi` | passed via `NPUFORGE_SUDO_PASS` | `printf '%s\n' "$NPUFORGE_SUDO_PASS" \| sudo -S -p "" <cmd>` |
| server | `root` | not needed (root directly) | the automation key is in root's `authorized_keys` |

`sudo -S` consumes stdin's first line as the password. File contents cannot be
piped in, so writing a file goes through a temporary file.

### 2.2.1 Board credentials are the vendor defaults — a deliberate choice

The board accounts and sudo password are **the OS image's vendor defaults,
unchanged.** Writing that down openly was judged better than hiding it.

| | |
|---|---|
| Premise | The boards are on the private `192.168.123.0/24` range, behind NAT. No inbound forwarding |
| Keeping the defaults | Vendor defaults are **already public information.** At the very least this tells nobody anything new |
| Why not change them | Using a custom value **creates a secret that did not exist.** If that value leaks anywhere — a document, a history, a photograph — it exposes a password **pattern**, and that is information that spreads beyond this lab |

> **This judgement changes when the conditions do.** The moment port 22 is
> forwarded externally or the boards sit on a non-isolated network, the defaults
> become a problem immediately. Like an exclusion, this decision carries **under
> what conditions.**

```bash
S() { printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" "$@"; }
cat > /tmp/f.new <<'H'
...
H
S cp /tmp/f.new /etc/target       # printf "text" | S tee ... does not work
```

Remote execution pitfalls (background startup, process counting) are in
`adrs/017-remote-exec-pitfalls-library.md`.

## 2.3 Static IPs (host-side static)

To stop DHCP reassignment changing the IPs, **the current IP is pinned as
NetworkManager static on each host.** Host configuration is used rather than
router (ipTIME) DHCP reservations because the settings survive a router change,
which is better for measurement reproducibility. **Since the current IP is
pinned as-is, SSH sessions are not dropped.**

Common parameters: gateway `192.168.123.254`, DNS `210.94.0.73 210.220.163.82`,
prefix `/24`. All managed by NetworkManager (not netplan or networkd).

```bash
# server (root, connection enp1s0) - done
nmcli con mod enp1s0 ipv4.method manual \
  ipv4.addresses 192.168.123.9/24 ipv4.gateway 192.168.123.254 \
  ipv4.dns "210.94.0.73 210.220.163.82"
nmcli con up enp1s0

# boards (pi/sudo, connection 'Wired connection 1', eth0) - done (2026-08-20)
#   king .3 / queen .5 / jack .4. Same IPs, so SSH held; external reachability confirmed
```

> ⚠️ **Beware DHCP pool collisions.** If `.3/.4/.5/.9` fall inside the ipTIME
> DHCP pool, the router can lease those addresses to another device (it does not
> know about host-side statics). Full avoidance means excluding those addresses
> from the pool in the ipTIME UI. The risk is low on a small home LAN but it is
> residual.

## 2.4 The sudo password file

The sudo password for board automation is passed via the dev PC's local
`~/.npuforge/sudo-pass` (chmod 600) or the `NPUFORGE_SUDO_PASS` environment
variable. It is not put in the repository. `preflight-check.sh` and the
deployment scripts read that path.

---

# 3. Software status

## 3.1 Nodes (king / queen / jack)

| Item | Value | Matching across 3 |
|---|---|---|
| SoC | Rockchip RK3576 | ✓ |
| NPU | 2 cores, 300–950 MHz, IOMMU enabled | ✓ |
| RKNN Runtime | 2.3.0 (`librknnrt.so` SHA-256 identical) | ✓ |
| RKNPU Driver | v0.9.8 | ✓ |
| Kernel | 6.1.141 | ✓ |
| glibc | 2.39 | ✓ |
| RAM / eMMC | 4GB / 64GB | ✓ |
| Ubuntu patch level | 24.04.4 | ✓ |
| gcc | 13.3.0-6ubuntu2~24.04.1 | ✓ |
| CPU Governor | **`performance`** | ✓ survives reboot |
| eth0 link | **2.5G (2500 Mb/s)** | ✓ measured 2026-08-20 |
| **SSH host key** | **identical on queen and jack** | ✗ **unresolved** |
| `.rknn` model (FP16) | `459602ea…` deployed to all 3 | ✓ |
| `.rknn` model (INT8) | `dba155d2…` **on `king` only** | ✗ needs deploying |
| Rust toolchain | 1.97.1 | **on `king` only.** For builds |
| C measurement tools | `~/npuforge-rknn-test/` | ✓ hashes identical |

**The SSH host keys are identical on queen and jack.** The two boards cannot be
told apart cryptographically, so a changed IP attaches you to the wrong board
without a warning. Since DHCP means the IPs do change (§1), the risk is
significant. The remediation commands are in `TODO.md` §1.2.

`preflight-check.sh` confirms the above items match before every measurement.

## 3.2 server (192.168.123.9)

| Item | Value |
|---|---|
| OS | Rocky Linux 9.4 (Blue Onyx), kernel 5.14.0-427.13.1.el9_4 |
| Motherboard | ASUS H81M-K (H81 chipset) |
| CPU / RAM | **Core i7-4790 (4C/8T, 3.6–4.0 GHz)** / **16GB DDR3-1600 non-ECC** |
| Disk | ST2000VN004 2TB, root LVM 70GB (65GB free) |
| NIC | `enp1s0` **Intel X550T 10GBASE-T, 10G full measured** (2026-08-26), driver `ixgbe` |
| NIC slot | `PCIEX16_1` (direct to CPU). **Operating at PCIe 2.0 x4** |
| Basis for the slot limit | root port `00:01.0`'s `LnkCap: Speed 5GT/s, Width x16` — **the motherboard's x16 slot is itself capped at PCIe 2.0.** The slot decides, not the card (`LnkCap 8GT/s x4`). Nothing to be done |
| Is it a bottleneck | **No.** PCIe 2.0 x4 = about 16 Gbps per direction. Real use is ~4.6 Gbps per direction across three nodes — 3× headroom |
| Full inventory | new server [`hosts/server-i7-4790-20260826.md`](#hosts-server-i7-4790-20260826) · old server [`hosts/server-xeon-e5-2630l-20260826.md`](#hosts-server-xeon-e5-2630l-20260826) |
| Firewall | firewalld active, zone `public`. gRPC ports need opening (before measuring) |
| Build toolchain | **rust/cargo 1.92, gcc 11.5, protoc 3.14, git** (installed 2026-08-20) |
| Docker | not installed — to be set up if model conversion is needed |

> protoc is not in Rocky 9's default repositories; **the CRB repository**
> (`dnf config-manager --set-enabled crb`) has to be enabled for
> `protobuf-compiler` to appear. tonic-build 0.12 requires the system protoc.

**Two of dealer's (the laptop's) constraints are resolved by this server.**

1. **RAM 3GB → 16GB.** The concern about scheduler RSS (relaying payloads) is
   greatly eased. `environment-matrix.md` §10.1,
   `adrs/003-central-simple-scheduler.md`
2. **1GbE → 10GbE.** Aggregation bandwidth secured. Measured in §4.

**The scheduler (x86_64) is built natively on server.** With MSRV 1.85 < dnf
rust 1.92, it builds on the stable channel. Sources are handed over as a
`git archive` tarball via scp (server cannot reach foxden directly; github is
fine). The node (aarch64) is still built on king. Windows→Linux cross-building
is not used because of linker problems.

## 3.2.1 Server replacement (2026-08-26) — the baseline dropped 7.5%

The old server (Xeon E5-2630L ×2, **24 threads**) was physically replaced and
things moved to a spare desktop (i7-4790, **8 threads**). **Only the scheduler
host changed; the three nodes, switch, model and binaries are unchanged.**

| | Old server (to ~2026-08-24) | New server (2026-08-26–) |
|---|---|---|
| CPU | Xeon E5-2630L ×2 · 24T · **2.0–2.5 GHz** | Core i7-4790 · 8T · 3.6–4.0 GHz |
| RAM | 16GB | 16GB DDR3-1600 |
| NIC | **Intel X550T** `enp4s0` | **the same card** `enp1s0` (PCIe 2.0 x4) |
| **Baseline throughput** | **~391 inf/s** | **~360 inf/s** (3 runs: 360.5 / 362.5 / 357.2) |
| Round-trip p50 | ~86 ms | ~93 ms |
| Node spread | ~1.02× | ~1.07× |
| Error rate | 0 | 0 |

> **The 10G NIC is the same physical card.** There is only one Intel X550T; it
> was pulled from the old server and plugged into the new one. So **the
> hardware of the 10G path is identical across both measurements** — the NIC is
> a controlled variable and what changed is only the host (CPU, motherboard,
> PCIe slot). That narrows the verdict below accordingly.
>
> **Which link that card negotiated on each host was confirmed on 2026-08-26 by
> powering the old server back up** — the slot's capability remains after the
> card is removed.
>
> | | Old server (R620) | New server (H81M-K) |
> |---|---|---|
> | Slot generation | **PCIe 3.0** (`LnkCap 8GT/s`) | PCIe 2.0 (`LnkCap 5GT/s`) |
> | X550T link | 8GT/s × x4 | 5GT/s × x4 |
> | Bandwidth per direction | **about 32 Gbps** | about 16 Gbps |
>
> **The link bandwidth halved. It is still not a bottleneck** — real three-node
> use is ~4.6 Gbps per direction, so even 16 Gbps is 3.5× headroom. This is now
> a measurement rather than an estimate.
> → [`hosts/server-xeon-e5-2630l-20260826.md`](#hosts-server-xeon-e5-2630l-20260826)

### Cause — the scheduler host narrowed to CPU

Server CPU utilisation during measurement is **82.2%** (across 8 threads).

```text
scheduler          45.3%  ~ 3.6 cores
other (bench+kernel) 36.9%  ~ 2.9 cores
────────────────────────────────
total              82.2%
```

**The bench client runs on the same host as the scheduler.** On the old server
the same work was ~27% of 24 threads; on the new server it is 82% of 8.

Where the loss sits supports this. The node side is unchanged (NPU inference p50
28.35 ms, distribution an even 33.3%, temperature 53–57 °C), and the scheduler's
internal queue is empty too, with `scheduler_queue` 0.00 ms and
`scheduler_route` 0.01 ms. All the added time is in the transport sections
(`network_to_node` / `network_to_client`, p50 24.2 ms each) — not an application
queue but **CPU contention on the host.**

> **The PCIe downgrade is not the cause.** `LnkSta 5GT/s x4` is 16 Gbps per
> direction, 3× the headroom over real use (~4.6 Gbps). It is a hardware limit
> arising from H81M-K's x16 slot being PCIe 2.0, it cannot be remedied, and it
> does not need to be.

> **Raw data.** The bench JSON for those 3 runs is in
> [`../results/baseline-20260826-althost/`](../results/baseline-20260826-althost).
> The `-althost` suffix keeps `count-runs.sh` from adding them to the 421 and
> counts them separately.

### Effect on existing measurements — none

**All 421 measurements were taken on the old server and those values stand as
recorded.** The numbers are not retroactively edited. The new server's values
are written here separately as "reproduction figures on a different scheduler
host".

That said, S3.9a's verdict — **the scheduler is not a resource bottleneck** —
**has turned out to have been conditional.** That verdict held on a 24-thread
host. It does not hold at 8.

> Exactly the principle in the experiment ledger §4. **Exclusions are
> conditional.** A candidate once excluded reopens when conditions change. A
> verdict has to carry "under what conditions".

If measurement continues on the new server, **its values are not compared
directly with the old server's.** Where a comparison is needed, a baseline is
re-laid on the new server and relative comparison is done on top of that.

## 3.3 Distribution differences

```text
server  Rocky Linux 9.4   glibc 2.34   dnf   x86_64
nodes   Ubuntu 24.04      glibc 2.39   apt   aarch64
```

The node binary is aarch64, so it is built natively on `king` and deployed to
all three nodes (all three boards are on glibc 2.39). The scheduler is x86_64
and therefore a separate build.

---

# 4. Network

## 4.1 Current (rework completed 2026-08-20)

```text
                server (10G) -+
                              +-- NS-S25G10G-N --+-- king  (2.5G)
       dev PC (10G port/1G NIC)                  +-- queen (2.5G)
                                                 +-- jack  (2.5G)
                                                 \-- internet (2.5G, ipTIME)
```

- **Worker links 2.5G, aggregation (server) 10G.** As ADR-014 designed.
- **The management and inference networks are still not separated.** Everything
  is on the single `192.168.123.0/24` range and the boards' eth1 is unused.
  VLAN/subnet separation to prevent measurement contamination is to be decided
  before M3's main measurements.

## 4.2 Bandwidth measurements (2026-08-20)

| Measurement | Value | Tool | Meaning |
|---|---:|---|---|
| server enp1s0 negotiation | 10000 Mb/s full | ethtool | 10G link confirmed |
| Single king→server | **2.34 Gbps** | iperf3 | the effective 2.5G ceiling |
| **3 nodes concurrently →server** | **1.70 each, 5.11 Gbps total** | nc | **aggregation is not a bottleneck** |

Under concurrent three-node transmission the three streams **stayed even (213
MB/s each)**. Had the server been the bottleneck the total would have been cut
somewhere, and it was not. It comfortably accommodates the INT8 three-node RX
target of **4.60 Gbps** (`RESULTS.md` §8.1).

> The individual 1.70 Gbps being below the link ceiling (2.34) is an nc /
> single-core board CPU limit, not a switch or server limit. Actual M3 traffic
> is gRPC inference traffic, so this figure is used only to verify "does the
> infrastructure absorb 4.6 Gbps aggregate" — and the answer is yes.

## 4.3 Link speed gets checked every time

Faulty cables lowering the negotiated speed has happened repeatedly (the old
dealer at 100 Mb/s, the current dev PC at 100 Mb/s). 10GBASE-T requires Cat6/6a,
and Cat5e silently falls back to 2.5G/5G. Left unchecked you measure the cable
rather than the NPU.

```bash
ssh npuforge-server 'ethtool enp1s0 | grep Speed'
for h in npuforge-k npuforge-q npuforge-j; do
  ssh "$h" 'printf "%s eth0=%s\n" "$(hostname)" "$(cat /sys/class/net/eth0/speed)"'
done
```

---

# 5. Purchases needed

The equipment that was blocking M3 has **all been obtained.**

| Item | Status |
|---|---|
| ~~2.5G/10G switch~~ | ✅ NEXI NS-S25G10G-N (2.5G×4 + 10G×2) |
| ~~server with a PCIe slot~~ | ✅ i7-4790 / 16GB / Rocky 9.4 (replaced 2026-08-26) |
| ~~10G NIC~~ | ✅ Intel X550T `enp1s0` 10GBASE-T |
| ~~10G cable~~ | ✅ 10G full negotiation confirmed (RJ45, not DAC) |

The remaining purchases are for measurement quality and do not block starting M3.

| Item | Qty | Priority | Basis |
|---|---|---|---|
| Identical-model fans | 3 | medium | for the S0-B cooling comparison |
| USB power meters | 3 | low | for computing FPS/Watt |
| Cat6/6a cables (spares) | 2–3 | low | spares for the 10G link. The current link is fine |

Permanent cooling equipment is not on the list. Fanless is kept and thermal
throttling is treated as something to measure
(`adrs/013-fanless-thermal-as-measurement.md`).

---

# 6. Open items

| # | Item | Status | Blocker |
|---|---|---|---|
| 1 | ~~Static IP pinning~~ | ✅ all four manual (2026-08-20) | — |
| 2 | **Duplicate SSH host keys (queen, jack)** | not done | none. Commands in `TODO.md` §1.2 |
| 3 | **Deploy the INT8 model to queen and jack** | not done | none |
| 4 | **Settle the scheduler build/deploy path** | undecided | no Rust on server (§3.2) |
| 5 | **Open the gRPC port in server's firewall** | not done | before measuring. firewalld public zone |
| 6 | Rebuild the model conversion environment | on hold | dealer is gone. Not urgent since the model is already converted |
| 7 | Separate management and inference networks | undecided | before M3's main measurements |
| 8 | Record measured TX/RX (inference traffic) | not measured | after the node software is up |
| 9 | S0 thermal characterisation (30 min × 2 conditions) | not run | needs 3 fans (for S0-B) |

**Per-host MAC / static IP** (the actual MACs confirmed in §1). Use this table if
router reservations are done alongside:

```text
king    22-94-FF-34-46-B1  ->  192.168.123.3
jack    62-CE-3B-B6-E4-41  ->  192.168.123.4
queen   7E-D8-D7-40-45-82  ->  192.168.123.5
server  6C-B3-11-13-2F-38  ->  192.168.123.9
```

Resolved on 2026-08-20: the 2.5G/10G switch, the 10G scheduler server, the 10G
NIC and cable, measured aggregation bandwidth, and dealer's 3GB RAM constraint.
Previously resolved: RKNN thread-safety (context sharing forbidden), model
conversion (FP16 and INT8), calibration (200 COCO images), CPU governor
(`performance`), board placement variance, and OS patch level.

---

<a id="environment-matrix"></a>

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
[`experiments/S0_SUSTAINED_LOAD.md`](#experiments-s0-sustained-load).

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

---

<a id="hosts-readme"></a>

# Host inventory

*[한국어 원문](hosts/README.ko.md)*

The scheduler hosts' hardware specifications, kept exactly as the machine
collected them.

| File | Host | Period |
|---|---|---|
| `server-xeon-e5-2630l-20260826.md` | **Dell PowerEdge R620** / Xeon E5-2630L ×2 | 2026-08-20 – 08-26 (**the 421 measurements**) |
| `server-i7-4790-20260826.md` | Core i7-4790 / ASUS H81M-K | 2026-08-26 – |

## Why this exists

**The old server's (Xeon E5-2630L ×2) specification had not been kept.** It is
the equipment the 421 measurements came from, and the documents recorded only
the CPU, RAM capacity and NIC name — no motherboard, RAM type, disk model or
PCIe information.

It was belatedly collected on 2026-08-26 by powering that server back up. **We
were lucky** — the equipment was still within reach. By then the OS had moved
from 9.4 to 9.8 and the 10G card had been removed. **A belated collection cannot
fully restore the state at the time.**

The boards had `collect-node-info.sh`; the hosts did not.
`server-profile-collect.sh` is a performance profiler (S3.9a), not an inventory
tool.

## Collection

```bash
ssh <host> 'bash -s' < scripts/collect-host-info.sh > docs/hosts/<name>-<date>.md
```

**When changing hosts, run it before deployment.** Serial numbers, asset tags
and UUIDs are not collected — what reproduction needs is the model name and
specification, not a unit identifier.

---

<a id="hosts-server-i7-4790-20260826"></a>

# 호스트 인벤토리 — server

- 수집: 2026-08-26 15:26:33 KST
- 수집기: `scripts/collect-host-info.sh`

## 시스템

| 항목 | 값 |
|---|---|
| hostname | server |
| 메인보드 | ASUSTeK COMPUTER INC. H81M-K |
| BIOS | 1003 (10/24/2014) |
| 배포판 | Rocky Linux 9.4 (Blue Onyx) |
| 커널 | 5.14.0-427.13.1.el9_4.x86_64 |
| 아키텍처 | x86_64 |
| glibc | 2.34 |
| SELinux | Enforcing |

## CPU

| 항목 | 값 |
|---|---|
| CPU(s) | 8 |
| Model name | Intel(R) Core(TM) i7-4790 CPU @ 3.60GHz |
| Thread(s) per core | 2 |
| Core(s) per socket | 4 |
| Socket(s) | 1 |
| CPU(s) scaling MHz | 99% |
| CPU max MHz | 4000.0000 |
| CPU min MHz | 800.0000 |
| L3 cache | 8 MiB (1 instance) |

## 메모리

| 항목 | 값 |
|---|---|
| 총량 | 15 GB |

| 슬롯 | 용량 | 종류 | 속도 | 제조사 |
|---|---|---|---|---|
| ChannelA-DIMM0 | 8 GB | DDR3 | 1600 MT/s | Samsung |
| ChannelB-DIMM0 | 8 GB | DDR3 | 1600 MT/s | Samsung |

## 저장장치

```text
NAME  SIZE MODEL              ROTA
sda   1.8T ST2000VN004-2E4164    1

Filesystem           Size  Used Avail Use% Mounted on
/dev/mapper/rl-root   70G  5.3G   65G   8% /
```

## 네트워크

```text
enp3s0           DOWN           
enp1s0           UP             192.168.123.9/24 fe80::f4c7:56a1:f4a6:5cfd/64 
```

| 인터페이스 | 속도 | 드라이버 | PCI | PCIe 링크 |
|---|---|---|---|---|
| `enp1s0` | 10000Mb/s | ixgbe | 0000:01:00.0 | Speed 5GT/s (downgraded), Width x4 (ok) |
| `enp3s0` | Unknown! | r8169 | 0000:03:00.0 | Speed 2.5GT/s (ok), Width x1 (ok) |

## PCIe 슬롯

| 슬롯 | 규격 | 사용 |
|---|---|---|
| PCIEX16_1 | x16 PCI Express | In Use |
| PCIEX1_1 | x1 PCI Express | Available |
| PCIEX1_2 | x1 PCI Express | In Use |

**루트 포트 능력** — 카드가 느리면 슬롯 탓인지 카드 탓인지 여기서 갈린다.

```text
00:01.0
  LnkCap:	Port #2, Speed 5GT/s, Width x16, ASPM L0s L1, Exit Latency L0s <256ns, L1 <8us
  LnkSta:	Speed 5GT/s (ok), Width x4 (downgraded)
00:1c.0
  LnkCap:	Port #1, Speed 5GT/s, Width x1, ASPM L0s L1, Exit Latency L0s <1us, L1 <4us
  LnkSta:	Speed 2.5GT/s (downgraded), Width x0 (downgraded)
00:1c.2
  LnkCap:	Port #3, Speed 5GT/s, Width x1, ASPM L0s L1, Exit Latency L0s <512ns, L1 <16us
  LnkSta:	Speed 2.5GT/s (downgraded), Width x1 (ok)
```

## PCI 장치

```text
00:00.0 Host bridge: Intel Corporation 4th Gen Core Processor DRAM Controller (rev 06)
00:01.0 PCI bridge: Intel Corporation Xeon E3-1200 v3/4th Gen Core Processor PCI Express x16 Controller (rev 06)
00:02.0 VGA compatible controller: Intel Corporation Xeon E3-1200 v3/4th Gen Core Processor Integrated Graphics Controller (rev 06)
00:14.0 USB controller: Intel Corporation 8 Series/C220 Series Chipset Family USB xHCI (rev 05)
00:16.0 Communication controller: Intel Corporation 8 Series/C220 Series Chipset Family MEI Controller #1 (rev 04)
00:1a.0 USB controller: Intel Corporation 8 Series/C220 Series Chipset Family USB EHCI #2 (rev 05)
00:1b.0 Audio device: Intel Corporation 8 Series/C220 Series Chipset High Definition Audio Controller (rev 05)
00:1c.0 PCI bridge: Intel Corporation 8 Series/C220 Series Chipset Family PCI Express Root Port #1 (rev d5)
00:1c.2 PCI bridge: Intel Corporation 8 Series/C220 Series Chipset Family PCI Express Root Port #3 (rev d5)
00:1d.0 USB controller: Intel Corporation 8 Series/C220 Series Chipset Family USB EHCI #1 (rev 05)
00:1f.0 ISA bridge: Intel Corporation H81 Express LPC Controller (rev 05)
00:1f.2 SATA controller: Intel Corporation 8 Series/C220 Series Chipset Family 6-port SATA Controller 1 [AHCI mode] (rev 05)
00:1f.3 SMBus: Intel Corporation 8 Series/C220 Series Chipset Family SMBus Controller (rev 05)
01:00.0 Ethernet controller: Intel Corporation Ethernet Controller 10G X550T (rev 01)
03:00.0 Ethernet controller: Realtek Semiconductor Co., Ltd. RTL8111/8168/8211/8411 PCI Express Gigabit Ethernet Controller (rev 0c)
```

## 서비스

| 항목 | 값 |
|---|---|
| firewalld | active / enabled |
| 열린 포트(영구) | 8080/tcp 9090/tcp 50051/tcp |
| chronyd | active / enabled |
| 시각 동기 | yes |

> 시리얼·자산번호·UUID 는 수집하지 않는다. 재현에 필요한 것은
> 모델명과 규격이지 개체 식별자가 아니다.

---

<a id="hosts-server-xeon-e5-2630l-20260826"></a>

# Host inventory — Dell PowerEdge R620 (the old scheduler server)

*[한국어 원문](hosts/server-xeon-e5-2630l-20260826.ko.md)*

- Collected: 2026-08-26
- Method: **manually, at the console.** No SSH public key was registered, so
  `scripts/collect-host-info.sh` could not be run remotely. The same items were
  pulled at the console and transcribed.
- Role: **the scheduler host for the 421 measurements** (2026-08-20 – 08-26)

> ## ⚠️ This collection is not the state at measurement time
>
> | | At measurement (as documented) | At collection (2026-08-26) |
> |---|---|---|
> | Distribution | **Rocky 9.4** | Rocky 9.8 |
> | Kernel | **`5.14.0-427.13.1.el9_4`** | `5.14.0-687.41.1.el9_8` |
> | 10G NIC | **Intel X550T fitted** (`enp4s0`) | **absent — the card was pulled and moved to the new server** |
> | IP | 192.168.123.9 (static) | 192.168.123.19 (onboard NIC, DHCP) |
>
> **The measurement conditions are the documented ones.** This table was
> captured belatedly to preserve the hardware specification, and the software
> state changed in the meantime.

---

## System

| Item | Value |
|---|---|
| Manufacturer / model | **Dell Inc. PowerEdge R620** |
| Baseboard | `0VV3F2` |
| BIOS | `2.2.3` (2014-05-20) |
| Distribution (at collection) | Rocky Linux 9.8 (Blue Onyx) |
| Kernel (at collection) | `5.14.0-687.41.1.el9_8.x86_64` |
| glibc | 2.34 |

## CPU

| Item | Value |
|---|---|
| Model | **Intel Xeon E5-2630L @ 2.00GHz** |
| Sockets | **2** |
| Cores per socket | 6 |
| Threads per core | 2 |
| **Total threads** | **24** |
| Maximum clock | 2500 MHz |
| L3 | 30 MiB (15 MiB × 2 instances) |

> An earlier document said `1.8GHz`, which was **wrong.** It is 2.0 GHz base /
> 2.5 GHz turbo.

## Memory

16GB total. **Only 4 of 24 slots are populated.**

| Slot | Capacity | Type | Speed |
|---|---|---|---|
| DIMM_A1 | 4 GiB | DDR3 | 1333 MT/s |
| DIMM_A2 | 4 GiB | DDR3 | 1333 MT/s |
| DIMM_B1 | 4 GiB | DDR3 | 1333 MT/s |
| DIMM_B2 | 4 GiB | DDR3 | 1333 MT/s |

The other 20 (A3–A12, B3–B12) are empty. With 2 DIMMs per socket, **only 2
channels per socket are active** (the E5-2630L supports 4 channels per socket).

## Storage

```text
NAME   SIZE MODEL      ROTA
sda  278.9G PERC H710P    1

/dev/mapper/rl-root   70G  7.3G  63G  11% /
```

A volume behind a `PERC H710P` (Broadcom/LSI MegaRAID SAS 2208) RAID controller.

## Network

**The onboard is an Intel I350 quad-port 1GbE. There is no 10G.**

| Interface | Speed | Driver | PCI |
|---|---|---|---|
| `eno1` | 1000Mb/s Full | `igb` | `01:00.0` |
| `eno2` `eno3` `eno4` | no link | `igb` | `01:00.1–.3` |

> This is **the evidence that the earlier record of `enp4s0` being onboard 10G
> was wrong.** The 10G was an Intel X550T **card**, now pulled and fitted to the
> new server.

## PCIe slots — the slot capability remains even with the card gone

| Slot | Specification | Use |
|---|---|---|
| `PCI1` | PCI Express 3 | Available |
| `PCI2` | PCI Express 3 | Available |

### Root port capability

```text
00:01.0  LnkCap 8GT/s x8    LnkSta 5GT/s x4    <- the onboard I350 (a PCIe 2.0 x4 card)
00:02.0  LnkCap 8GT/s x8    LnkSta x0          <- empty
00:02.2  LnkCap 8GT/s x8    LnkSta 5GT/s x8    <- the PERC H710P (PCIe 2.0 x8)
00:03.0  LnkCap 8GT/s x16   LnkSta x0          <- empty
40:02.0  LnkCap 8GT/s x16   LnkSta x0          <- empty (the second socket's IIO)
```

**All three empty root ports are 8GT/s (PCIe 3.0).** The physical slots `PCI1`
and `PCI2` also report as `PCI Express 3`. So **whichever slot the X550T went
into, it negotiated PCIe 3.0.**

### Which means the link bandwidth halved

| | Old server (R620) | New server (H81M-K) |
|---|---|---|
| Slot generation | **PCIe 3.0** (`LnkCap 8GT/s`) | PCIe 2.0 (`LnkCap 5GT/s`) |
| X550T link | 8GT/s × x4 | 5GT/s × x4 |
| Bandwidth per direction | **about 32 Gbps** | about 16 Gbps |

**Not a bottleneck.** Real three-node use is ~4.6 Gbps per direction, so even
16 Gbps is 3.5× headroom. But this is now **a measured value rather than an
estimate.**

→ The cause of the baseline difference (391 → 360) is still host CPU.
   `../infrastructure.md` §3.2.1

## PCI devices (summary)

```text
01:00.0-.3  Intel I350 Gigabit Network Connection  x4
02:00.0     Broadcom/LSI MegaRAID SAS 2208 (PERC H710P)
0a:00.0     Matrox G200eR2 (BMC-integrated VGA)
07-09:xx    Renesas SH7757 PCIe Switch/Bridge (internal bridges)
```

## What this server was used for

All 421 measurements came from this host. The S2 baseline, S3 saturation, the
S3.5–S3.9b transport lineage, and the S0-A–D thermal and policy lineage.

The replacement and its effects are in `../infrastructure.md` §3.2.1,
`../environment-matrix.md` §10.2 and
`../../results/baseline-20260826-althost/`.

---

<a id="experiments-readme"></a>

# Experiment Ledger

*[한국어 원문](experiments/README.ko.md)*

- Last updated: **2026-08-20**
- Subject: the M3 cluster (RK3576 ×3 + Xeon scheduler), YOLOv8n INT8, `want_float=0`
- Fixed conditions: governor `performance`, active cooling (120 mm fan per node),
  round-robin, 8 workers/node, gRPC (tonic + protobuf), closed-loop bench

> The detail of each experiment is in its own report. This document is the one
> page that shows **what was asked and what was ruled out**.
> Terminology is in [`../GLOSSARY.md`](#glossary).

**In one sentence**

> NPUDure's transport work began as a custom-transport implementation, but by
> removing bottlenecks the measurements actually pointed at, **a standard gRPC
> configuration alone improved three-node throughput by 13.3%** — and along the
> way established that **choosing the operating point and validating the
> experiment come before optimizing anything.**

---

## 1. The ledger

| ID | Question | Scale | Key result | Report |
|---|---|---:|---|---|
| **Pilot** | Do three nodes actually run | 3 runs | 336 inf/s, 0 errors, 33.3% even split | `board-worklog` §2.24 |
| **S2** | Does adding nodes scale linearly | **30 runs** | **112.9 / 229.0 / 338.4 inf/s**, speedup **3.00×**, eff 100%, 0 errors | [S2](#experiments-s2-grpc-baseline) |
| **S3** | What is each configuration's real ceiling | **45 runs** | ceiling **115.2 / 232.0 / 341.8**, 3N **2.97×** | [S3](#experiments-s3-saturation) |
| **S3.5** | Where does the −30% loss come from | 3 conditions | bandwidth, total CPU and the server ruled out → narrowed to the **transport path** | [S3.5](#experiments-s3-5-transport-profile) |
| **S3.5b** | Is CPU0 softirq concentration the cause | 6 runs | **−0.2% (null)** — though with a single flow, open to challenge | [S3.5](#experiments-s3-5-transport-profile) §4.3 |
| **S3.6** | Flow control or connections | **20 runs** | enlarging the window **−36.3%**; connections 1→4 **+21.5%** | [S3.6](#experiments-s3-6-h2-channel-ab) |
| **S3.7a** | How many connections is optimal (fixed c32) | **25 runs** | knee at c4, only the tail degrades past it | [S3.7](#experiments-s3-7-connection-tuning) |
| **S3.7b** | What is each configuration's **operating point** | **75 runs** | all three at **c12**. conn2 **dominates conn1 on both axes** | [S3.7](#experiments-s3-7-connection-tuning) §4 |
| **S3.7c** | Does RPS help at the operating point | **10 runs** | **−0.8% (null)**, unchanged even with CPU0 %soft 68→56 | [S3.7](#experiments-s3-7-connection-tuning) |
| **S3.8** | Does the optimization hurt scale-out | **36 runs** | **135.5 / 263.3 / 387.2**, 3N **2.86× (95.3%)**. Absolute +13.3% but eff 98.9→95.3% | [S3.8](#experiments-s3-8-optimized-scaleout) |

| **S3.9a** | Where does 3N's 4.5% efficiency loss arise | **9 runs** | Server resources **all ruled out**. The loss **shows up in the tail** (p50 flat, p99 +36%). TCP retransmits 3.5× — though the micro-mechanism was not isolated | [S3.9a](#experiments-s3-9a-scaleout-profile) |
| **S0-B** | Does the operating point hold under sustained load (active cooling) | **30 runs / 31 min** | **degradation 1.9%**, **zero** clock downgrades. short-run = sustained | [S0](#experiments-s0-sustained-load) |
| **S0-A** | What happens fanless | **30 runs / 32 min** | **degradation 11.3%**. CPU 2208→**816 MHz** (king), NPU pinned at 950. **The king is 2.4× slower and round-robin still sends it 1/3** | [S0](#experiments-s0-sustained-load) |

| **S0-C** 1st | Does a load-aware policy recover the thermal-heterogeneity loss | **15 runs** | The policies **collapse throughput by 55–58%**. Cause: **herding on stale heartbeat state** — a scheduler bug, found | [S0-C](#experiments-s0-c-policy-ab) |
| **S0-C** 2nd | Re-measure after fixing that bug | **12 runs** | **RR 373.9 / LQ 380.9 / ECT 384.2.** Collapse gone, **p99 −37%**, node latency spread 1.33×→**1.00×** | [S0-C](#experiments-s0-c-policy-ab) §8–11 |

| **S0-C** 3rd | Any policy regression under homogeneity (active cooling) | **12 runs** | No regression (LQ −0.0%, ECT −0.3%). Tail improves even when homogeneous. **Neither LQ nor ECT dominates** | [S0-C](#experiments-s0-c-policy-ab) §12–15 |
| **S0-C** 4th | LQ vs ECT under strong heterogeneity (2.4×) | **1 run (aborted)** | **Gate missed at 1.10×.** Thermal conditions were identical (86.8 °C) — **what sets heterogeneity is not temperature but the spread in CPU downgrade** | [S0-C](#experiments-s0-c-policy-ab) §17–19 |
| **S3.9b** | Do node-side syscalls/copies matter in the residual gap | **4 conditions** | **Not syscalls** — ~1% of transport cost (8% being generous). User time exceeds kernel time (9.37 vs 6.99 ms/req). CPU is 48.9% idle = **not a constraint**. → **S4 io_uring cancelled/shelved** | [S3.9b](#experiments-s3-9b-node-residual) |
| **S0-D** calibration | Can heterogeneity be produced deterministically | **12 runs** | **Yes.** Cap 2208→600 moves the spread **1.12×→3.93×**. **Cap 816 reproduces S0-A (2.4×) to within 6 ms** | [S0-D](#experiments-s0-d-capacity-hetero) |
| **S0-D** policy | Does ECT gain as the spread widens | not run | Would test the design rationale for the ECT default directly | [S0-D](#experiments-s0-d-capacity-hetero) §6 |

**Total measurement runs: 421** (418 bench + 3 profile conditions), **error rate 0**
throughout. Four discarded runs (harness-collision contamination,
`results/policy-ab-20260821-contaminated/`) are excluded.

> This number is **not maintained by hand** — `bash scripts/count-runs.sh` counts
> it. Two documents once each carried their own copy and diverged to 343 vs 420
> (2026-08-21).

### 1.1 Raw-data map

Which experiment every directory under `results/` belongs to. **This table exists
to prevent orphan data** — 87 runs were in fact sitting unclaimed by any document
when this was checked (found 2026-08-21).

| Directory | Runs | Experiment |
|---|---:|---|
| `scaling-20260820` | 3 | pilot measurement (`RESULTS.md` §2.5, **superseded**) |
| `baseline-20260820` | 30 | S2 |
| `saturation-20260820` | 45 | S3 |
| `transport-profile-20260820` | 3 conditions | S3.5 |
| `rps-ab-20260820` | 6 | S3.5b (RPS null) |
| `h2-channel-ab-20260820` | 20 | S3.6 |
| `connection-sweep-20260820` | 25 | S3.7a (fixed c32) |
| `concurrency-sweep-20260820` | 30 | **S3.7b** — conn 2·4 × c24–64 (overload region) |
| `concurrency-sweep-20260820-low` | 30 | **S3.7b** — conn 2·4 × c8–24 (operating region) |
| `concurrency-sweep-20260820-conn1` | 15 | **S3.7b** — conn 1 × c8–24 |
| `s37b-operating-point` | 45 | S3.7b operating-point determination |
| `scaleout-optimized-20260820` | 36 | S3.8 |
| `scaleout-optimized-20260820-1n-only` | 12 | **S3.8** — 1N re-measurement |
| `scaleout-profile-20260821` | 9 | S3.9a |
| `node-residual-20260821` | 4 conditions | S3.9b |
| `sustained-20260821-fan` | 30 | S0-B |
| `sustained-20260821-fanless` | 30 | S0-A |
| `policy-ab-20260821` | 15 | S0-C 1st (herding bug found) |
| `policy-ab-20260821b` | 12 | S0-C 2nd |
| `policy-ab-20260821fan` | 12 | S0-C 3rd |
| `policy-ab-20260821-contaminated` | 4 | S0-C 4th — **discarded** (harness collision) |
| `capacity-calib-20260821` | 12 | S0-D calibration |
| `accuracy` · `thermal-20260811-*` | — | model accuracy · pilot thermal measurement (`RESULTS.md`) |

> ⚠️ `results/policy-ab-20260821-contaminated/` — **concurrent harness
> collision; invalid for performance conclusions.** It is the S0-C 4th attempt;
> only the r1 round-robin run and the 1-second thermal log are usable. The
> incident is written up in §4.11 and [S0-D](#experiments-s0-d-capacity-hetero) §4. Kept
> as a methodology record.

---

## 2. Bottleneck candidates — exclusions are conditional

**The value of this table is in shrinking the candidate space, not in having
identified what the remaining gap is.** And **a candidate once excluded reopens
when conditions change** — S3.8 is exactly that (§4.7).

| Candidate | Current verdict | Basis |
|---|---|---|
| Link bandwidth | **excluded** | node eth0 51% per direction (S3.5); server 10G **40% per direction** (S3.9a). The "76%" in S3.8 was a full-duplex arithmetic error and is **withdrawn** |
| Board CPU capacity | **excluded** | 8 cores **49–63% idle** (S3.5, S3.7c) |
| Server CPU · NIC · scheduler | **excluded (24-thread host only)** | CPU 42%, busiest core 47.6%, 0 drops, no thread serialization, syscalls/req unchanged (S3.9a). ⚠️ **Reopens on an 8-thread host** — see below |
| **Shared-path congestion (10G→2.5G)** | **new, leading (unverified)** | per-connection TCP retransmit rate **3.5×**, cwnd 176→106–119. Consistent with p50 flat and only the tail rising (S3.9a) |
| Kernel RX distribution (RPS) | **excluded** | throughput unchanged even after taking **12 pp off** CPU0 %soft (S3.7c) |
| HTTP/2 flow control | **counterproductive at the extreme** | **−36.3%** when enlarged to 64 MB. **Mid-range values (256 KB–4 MB) unmeasured** (S3.6) |
| **Connections per node** | **primary constraint** | 1→2 gives **+18.8%** throughput and **−18.8%** on the tail (S3.7b) |
| Remaining cost | **not separated** | protobuf / memcpy / syscall / H2 implementation / userspace scheduling / NPU submission → needs profiling |

> **"Server and scheduler excluded" held only under the baseline (conn1)
> condition.** Optimizing per-node transport raised the load reaching the shared
> path and that exclusion broke. An exclusion verdict has to carry **the
> conditions it was reached under**.

> **The same exclusion broke a second time — this time on hardware conditions
> (2026-08-26).** Swapping the scheduler host from 24 threads (Xeon E5-2630L ×2)
> to 8 threads (Core i7-4790) dropped the baseline from **391 to 360 inf/s
> (−7.5%)**. Server CPU during the measurement was **82.2%** (42% under the old
> host).
>
> The interesting part is that **the application queue is still empty** —
> `scheduler_queue` 0.00 ms · `scheduler_route` 0.01 ms. What S3.9a actually
> excluded was that queue, and that verdict still stands. What narrowed is
> **outside** it: the host's CPU. The measurement setup — **the bench client
> runs on the same host as the scheduler** — amplifies this.
>
> All 421 measurements were taken on the old server and **those values stand as
> recorded.** Reproduction figures on the new server are kept separately in
> `../infrastructure.md` §3.2.1 and `../environment-matrix.md` §10.2. We do not
> compare the two hosts' numbers directly.

---

## 3. Numeric lineage — how per-node throughput moved

The **single-connection** configuration converges on **113–117** across five
independent experiments. Different days, different harnesses, different purposes,
same value.

```text
S2   1N @c8   (30 runs)  112.9 ± 0.5
S3   1N ceiling @c32     115.2
S3.5 cluster  @c32       116.6
S3.6 A(1ch)   @c32       115.3 ± 0.8
S3.7a c1      @c32       115.6 ± 0.7
S3.7b conn1   @c12  ★    114.8 ± 0.7   <- conn1's operating point
─────────────────────────────────────
S3.7b conn2   @c12  ★★   136.4 ± 0.3   <- optimized operating point  (+18.8%)
S3.8  conn2   @c12       135.5 ± 0.4   <- reproduced independently, different harness
local direct (no network)  161.5
─────────────────────────────────────
residual gap at the operating point
                 161.5 − 135.5 = 26.0 inf/s = **16.1% of direct**
                 (against S3.7b's 136.4 it is 25.1 = 15.5%)
```

★ A fair comparison is between operating points found by the same rule (98% of
peak).

---

## 4. Methodology lessons — what will outlast the numbers

### 4.1 Optimize at the operating point, not in the overload region

S3.6 and S3.7a, at **fixed c32**, produced "more connections make the tail 46%
worse". The measurement was correct, but c32 was **overload for all three
configurations**. Re-measured at the operating point (c12), the same change
**improved the tail by 18.8%**. The sign of the conclusion flipped.

A fixed-load comparison can show **overload behaviour** rather than a
configuration effect. So the c32 results are not discarded — they are kept as a
separate result labelled "overload behaviour", just not used as grounds for
operating decisions.

### 4.2 Two measurements agreeing does not make the interpretation right

c32 (+28.2%) and c24 (+27.4%) agreed closely, and we wrote that up as "a
property of 4 connections as such". **Both were in the overload region — we had
seen the same bias twice.** Reproducibility only confirms the bias.

### 4.3 Fix the decision rule before the results, and do not move it to fit them

The operating-point definition is pinned as a constant in code.

> operating concurrency = the **lowest** concurrency delivering at least **98%**
> of peak
> (99% overlaps with the ±1 inf/s run-to-run SD)

In S3.7a, c2 came in at **96.4%** and missed the threshold by 0.6 pp. Lowering
the threshold to 96% would have produced the answer we wanted, and **we did not
lower it.** We recorded "the rule does not decide at this boundary" as the
result instead, and S3.7b settled it with data.

### 4.4 Turn silent failures loud

- **Make the harness stop when it fails.** When a node was built with the mock
  backend and failed to start, the harness died loudly and immediately, and it
  was caught on the spot.
- **Leave evidence that the configuration took effect.** Every run counts the
  actual TCP connections with `ss` and records it. A silently ignored setting
  turns an A/B into the same condition run four times.
- **Verify node count before measuring.** Process existence ≠ receiving traffic.
  S3.8 uses a probe bench to count the **distribution of responding node IDs**
  and skips the configuration when expected ≠ observed.
- **Do not delete the raw data.** A bug that cleared the output directory
  between runs destroyed an earlier run's JSON (throughput survived in the CSV,
  so the conclusion was unharmed). After that, staging directories were split
  out.

### 4.5 A candidate once excluded reopens when conditions change

S2 showed three nodes scaling 3.00× linearly, so **the server and scheduler were
excluded**. That verdict was right — **under conn1**.

Raising per-node connections to 2 increased load on the shared path, and
optimized 3N efficiency fell to **95.3%**. The server is a candidate again.

> **An exclusion verdict has to carry "under what conditions".** When conditions
> change, the exclusion table has to be re-read. Write it once and freeze it,
> and you will not see the new bottleneck your own optimization created.

### 4.6 A throttling verdict needs its conditions too

Same hardware, same operating point, same load — and **cooling alone changes the
conclusion.**

```text
active cooling   degradation  1.9%   0 clock downgrades   NPU 60 °C
fanless          degradation 11.3%   CPU 2208→816 MHz     NPU 88 °C
```

The worklog's "CPU −27% over 300 s" was a correct observation too — **under those
conditions.** S0-A saw the CPU fall to the same 816 MHz and the loss was −11.3%
(the cluster workload has CPU headroom). **Detach the conditions and the number
lies.**

This is the same story as the exclusion verdicts (§4.5). **"There is / is not
throttling" has to be written with its conditions.** Detached, the next person
plans on a false premise — which is exactly what happened here: that −27% put S0
ahead of io_uring this session (the right call, but the basis turned out to
belong to different conditions).

### 4.7 Separate "the policy is bad" from "the implementation is broken" first

S0-C saw load-aware policies drop throughput by **55–58%**. Stopping there with
"load-aware policies do not suit this workload" would have been the wrong
conclusion.

The per-stage breakdown separated it — `scheduler_route` 0.004 ms across all
three (decisions are fast), `node_queue` 0.023 ms (nodes are not backed up), yet
the round trip alone 2.8×. Node CPU was in fact halved (45% → 20%). **They are
not doing more work; they are waiting more.**

The cause was not the policies' judgement quality but **state freshness**. The
`queue_depth` the policies read was refreshed only by heartbeat and not updated
by the dispatch path, so hundreds of requests per second all read the same fixed
snapshot and all chose the same node.

> **When performance looks wrong, ask "is the implementation doing what it was
> meant to" before "this approach is bad".** 55% is not the size of a
> quality difference.

Re-measured after the fix, the policies behaved and **p99 improved by 37%**. Had
we concluded from the first result, the record would say the exact opposite:
"load-aware scheduling does not help".

One more thing — **a decision rule failing to fire is also a result.** In the
2nd round the shift in king's share was 0.5 pp, short of the 3 pp rule. We did
not lower the threshold; we wrote down why it did not fire (the thermal spread
was a weak 1.33×, and least-outstanding is a closed loop regulating concurrent
occupancy rather than counts).

### 4.8 State how percentiles were aggregated

In tables pooling several runs, p95/p99 are the **average of run-level
percentiles**, not pooled percentiles. Run-level averaging dilutes each run's
worst window and so **makes the tail read low**. Valid for comparing conditions;
not to be quoted as "this system's p99" →
[S2 §7.4.1](#experiments-s2-grpc-baseline)

### 4.9 A process without logs tells you nothing afterwards

The jack node died and **the cause could not be established.** It was neither OOM
nor segfault, and there were no logs at all — the `setsid nohup` in the startup
procedure was missing a redirect, so stdout was being thrown away. **Not knowing
why it died is worse than it dying.**

---

### 4.10 Your instrument may be measuring a different quantity

The policy A/B harness recorded SoC temperature every run. Those values read
**78–79 °C**, so "cooler than S0-A (86 °C)" went into several documents, and on
top of it a prescription: "we need continuous heating".

**Wrong.** The harness reads the value *after* the 60-second run finishes, over
three sequential ssh calls. RK3576 cools within seconds once load stops, so that
value is the trough between runs. Re-aggregated from the 1-second thermal
logger, it was **86.8 °C — the same as S0-A.** The thermal conditions had been
identical from the start; what differed was the spread in CPU downgrade.

Two instruments carried the same name (`soc`) and measured different quantities
(max during the run vs. an instantaneous value after it). **That the CSV column
was named `max_soc_c` made it worse** — it was not a maximum.

> Moving a decision threshold and fixing the instrument that measures it are
> different acts. The first fits the rule to the results (violating §4.3); the
> second is what keeping the rule requires. When fixing, **state in the document
> that the reference value is unchanged and only its source moved.**

### 4.11 Do not trust "I stopped it" — verify at the shared resource

A harness was stopped and another was started. In reality **the stop failed and
both were hitting the same three nodes at c36 each.** The baseline came out at
197 inf/s (391 is normal) and the next run had an 82% error rate. **We were one
step from misdiagnosing a cluster failure** — after cleanup, re-measurement gave
391.2 with 0 errors.

Because the surviving harness kept restarting the scheduler with its own
configuration, the "restore to default settings" step was being silently
overwritten seconds later.

**Local observation lied.** In git-bash it did not appear in `ps -ef` and
`pkill -f` could not catch it. Only PowerShell's `Get-CimInstance Win32_Process`
showed it. Process observation is not trustworthy across platforms.

So verification moved **to the shared resource.** `npuforge_assert_cluster_free`
will not start a harness if `npuforge-bench` is running on the server. A bench
running on the server does not lie. (This is §4.4 extended — turn silent
failures loud.)

### 4.12 Harness invariants — §4.4 and §4.11 hardened into rules

These came out of two incidents (§4.11, and the results-path overwrite). New
harnesses obey both.

1. **Verify shared-resource state at the shared resource.**
   Knowing locally that "I stopped it" is not enough. Whether the cluster is
   free is a question **for the cluster** (`npuforge_assert_cluster_free`).
2. **Do not treat the results path as an appendable/overwritable scratch
   directory.** `results/<experiment>-<date>` overwrites itself when run twice
   in a day. This did overwrite S0-C's 1st round (15 runs), which would have
   been lost had it not been under git. Stop if the existing directory is not
   empty.

### 4.13 The six times the instruments lied — the authoritative list

§4.10 and §4.11 cover two of them, but talks and public material cite "six
times". **If you are going to use the number, there has to be a list.** This is
that list.

**Scope: the cluster measurement campaign (2026-08-20 to 08-21).** The four
failures from the single-node era are kept separately in
[`../RESULTS.md`](#results) §6. The two are not counted together.

| # | What lied | How it surfaced | Basis |
|---:|---|---|---|
| 1 | **Post-run temperature sampling.** The `max_soc_c` column was not a maximum but the inter-run cooling trough — ~5 °C below actual | Compared against the 1-second thermal logger | [S0-C §17.5](#experiments-s0-c-policy-ab) · §4.10 |
| 2 | **The explanation built on that value.** "The 2nd round only reached 1.33× because it was less hot" — the 2nd round was also 86.8 °C | Re-aggregated the 2nd round's thermal log | [S0-C §18.4](#experiments-s0-c-policy-ab) |
| 3 | **13.2% was an overload-region figure.** That percentage came from 140.1 (c32) but got paired with the 135.5 operating point and spread through several documents. The real figure is 16.1% | Computed the denominator directly | §3 · commit `62855bd` |
| 4 | **Harness collision.** A harness believed stopped survived and hit the same cluster as the new one, both at c36. Baseline 197 inf/s (391 is normal), next run 82% error rate | Checked the server's process list | [S0-D §4](#experiments-s0-d-capacity-hetero) · §4.11 |
| 5 | **Results-path overwrite.** Reusing the same dated path overwrote S0-C's 1st round (15 runs) with 4 lines | `git status` | [S0-D §4.2](#experiments-s0-d-capacity-hetero) · §4.12 |
| 6 | **The `strace -c` parser read the wrong columns.** `usecs/call` and `calls` were swapped, making the call count come out 100× too small | Compared against expected values from `/proc/PID/io` | [S3.9b §8](#experiments-s3-9b-node-residual) |

**What the six have in common: every one of them looked like success.** A number
came out, it was plausible, nobody stopped. Four were caught **by comparison
against another measurement** (1, 2, 4, 6); two were caught because **a tool said
so loudly** (3, 5).

> **Three of the six (1, 2, 3) share one root** — an instrument was wrong, an
> explanation was built on it, and that explanation propagated into other
> documents. Instrumentation errors do not stay contained.

## 5. Current settled state

**The measurement lineage closed on 2026-08-21.** S2 through S3.9b and S0-D.

**The two lineages are not mixed.** The transport operating point and the
scheduling policy rest on different evidence.

```text
Transport operating point  -- settled ------------------------------
    2 connections/node @ concurrency 12/node

    1N   135.5 inf/s   p95 120.7 ms
    3N   387.2 inf/s   p95 151.1 ms   scaling 2.86x, eff 95.3%
    31 min sustained (active cooling)  380.3 inf/s  (-1.9%, 0 clock downgrades)

Adaptive policy  -- settled ----------------------------------------
    Default stays `ect`.

    RR is out of the running -- p99 SD 34.7 under heterogeneity
    (adaptive is ~1). Load-aware scheduling improves the tail markedly.

    LQ and ECT: **neither dominates.**
      fanless (heterogeneous)   LQ p99 146.9 / ECT 384.2 inf/s
      active cooling (homogeneous)  both fine. No regression (S0-C 3rd)
```

> **`node_connections` has two "defaults".** Easy to confuse, so it is written
> down.
>
> | | Value | What it is |
> |---|---:|---|
> | Library fallback | **1** | `SchedulerTransportConfig::default()`. For reproducing the baseline — give no configuration and you get the initial measurement condition |
> | Recommended operating value | **2** | `configs/scheduler.example.toml`. The operating point S3.7b established |
>
> Raising the code default to 2 would silently give a different condition to
> anyone trying to reproduce the old baseline. So **the fallback stays 1 and the
> example recommends 2.**

> The 0.9% throughput difference between ECT and LQ is **not used as grounds for
> preferring either.** ECT's basis is that it **absorbed the node latency spread
> to 1.00×** — that is, it reflected heterogeneous capacity as designed.

> **Mind the connection units** — `node_connections` is **per node**.
> 1N → 2 total, 2N → 4 total, 3N → 6 total.

Against the conn1 baseline (114.8, same rule): **+18.8% throughput, −18.8% p95**
— a strict Pareto improvement on the throughput and latency metrics measured. Of
the 46.7 gap to local direct (161.5), **21.6 (46%) was recovered by
configuration alone.**

---

## 6. How the lineage closed

| Step | Result |
|---|---|
| ~~S3.8~~ | Re-verified scale-out at the operating point. **+13.3%, efficiency 98.9→95.3%** |
| ~~S3.9a~~ | Server-side profile. **Server resources all excluded** — the loss shows up in the tail |
| ~~S0-A / S0-B~~ | 30 minutes of sustained load. Fanless −11.3% / active cooling −1.9% |
| ~~Policy validation on real hardware~~ | S0-C. Policies collapse 55–58% → **state-freshness defect (herding) found** |
| ~~Herding fix~~ | `local_in_flight` atomic reservation + RAII guard |
| ~~S0-C 3rd~~ | After the fix the policies adapt. No regression under homogeneity |
| ~~S0-D~~ | Deterministic heterogeneity fixture (clock caps). Produces heterogeneity without relying on heat |
| ~~S3.9b~~ | Remaining node-side cost. **io_uring's reachable share is ≈8%** |
| ~~S4 (io_uring)~~ | **Not adopted.** The measurement argued against it → `01-TECHSPEC.md` §15 |

### 6.1 The S0 result — the operating point is attached to the cooling condition

```text
short-run operating point                   3N 387-389 inf/s
sustained (active cooling)                  3N 380.3      (-1.9%)
sustained (fanless)                         3N 345.4      (-11.3%)
```

Under active cooling there were **zero** clock downgrades and a 58–61 °C plateau
— **the 60-second results from S2 through S3.9a apply unchanged to sustained
operation.**

Remove the fan and it splits. What gets downgraded is **the CPU, not the NPU**
(pinned at 950 vs 2208 → 816 MHz), and by a different amount on each board.

And **the fanless loss is not purely a thermal problem** — the king became 2.4×
slower while round-robin kept sending it 1/3 of the work (S0 §4.3). It is the
product of **thermal spread × a load-blind policy**. That is why the next
experiment became **policy validation** rather than io_uring.

### 6.2 S4's question changed, and the answer was "do not"

```text
at first   How much faster is io_uring than gRPC?
now        How far does a properly configured standard gRPC stack get, and what cost remains behind it?
```

S3.9b answered. Of the gap remaining at the operating point, **io_uring can
reach about 8%**. That is a small recovery against the implementation cost.
**We decided not to implement it and recorded the verdict** — `01-TECHSPEC.md`
§15.

---

## 7. Open — deliberately not closed

The measurement lineage is closed, but the following were not answered. **We do
not write down what we do not know as though we knew it.**

| Item | Status |
|---|---|
| **LQ vs ECT under strong heterogeneity (2.4×)** | Unmeasured. The basis for keeping `ect` as the default is a homogeneous sanity pass. **S0-D has made this question reproducible** |
| **Micro-mechanism of the 3N efficiency loss** | Confirmed as far as "it shows up in the tail" (p50 flat, p99 +36%). The shared-path congestion hypothesis (10G→2.5G) is **unverified** — needs switch counter access |
| **Short-window distribution** | Only 60-second aggregates exist. Needs `bench --dump-samples` |
| **Pooled percentiles** | Needs the same option. Current percentile figures are **run-level averages** |
| **Node exclusion behaviour** | Unverified — even fanless never reached the 90 °C threshold |
| **Grid resolution** | Whether the c12 operating point is the true knee or c10 is undetermined (grid step of 4) |
| **Mid-range H2 window** | 256 KB–4 MB unmeasured. Only the 64 KB ↔ 64 MB extremes were looked at |
| **c8/c16 connection ceilings** | Merely dropped from the S3.7b candidate set; not proven inferior |

> The entries in this table are **not things left unfinished but things decided
> against.** Each carries why it is open.

---

<a id="experiments-s0-c-policy-ab"></a>

# S0-C — Scheduling Policy A/B (fanless)

*[한국어 원문](experiments/S0_C_POLICY_AB.ko.md)*

- Experiment ID: **S0-C**
- Measured: 2026-08-21
- Code: `7281411` + `[transport] node_connections = 2`
- Status: **1st (15 runs, bug found) · 2nd fanless (12 runs) · 3rd active cooling (12 runs)** complete
- Raw data: [`../../results/policy-ab-20260821/`](../results/policy-ab-20260821) (1st) ·
  [`../../results/policy-ab-20260821b/`](../results/policy-ab-20260821b) (2nd, fanless) ·
  [`../../results/policy-ab-20260821fan/`](../results/policy-ab-20260821fan) (3rd, active cooling)
- Predecessor: [`S0_SUSTAINED_LOAD.md`](#experiments-s0-sustained-load)

---

## 1. Research Question (as originally intended)

> S0-A saw king become 2.4× slower fanless while RR kept sending it one third.
> **Do load-aware policies (`least-queue`/`ect`) recover that loss?**

## 2. 1st Results — the policies collapse

Fanless, after 15 minutes of preheat, 3 nodes / 2 connections per node / c36,
5 runs per policy. Error rate **0 throughout**.

| Policy | Throughput | p50 | p95 | p99 | king% | jack% | queen% |
|---|---:|---:|---:|---:|---:|---:|---:|
| **round-robin** | **379.9 ± 13.5** | 85.5 | 165.4 | 213.8 | 33.3 | 33.3 | 33.3 |
| **least-queue** | **169.8 ± 2.3** | 199.8 | 390.0 | 477.1 | 35.1 | 33.3 | 31.6 |
| **ect** | **158.5 ± 2.1** | 219.6 | 397.9 | 483.0 | 34.8 | 36.3 | 28.9 |

**The load-aware policies drop throughput by 55–58%.** That is the opposite of
the expected direction, and it is not "a bit worse" but **less than half.**

Per-node CPU busy% (`/proc/stat` deltas):

| Policy | king | jack | queen |
|---|---:|---:|---:|
| round-robin | 51.5% | 44.5% | 45.3% |
| least-queue | 22.6% | 21.1% | 20.7% |
| ect | 21.0% | 21.7% | 17.7% |

**The nodes are actually idling.** Throughput is halved and so is CPU usage.

## 3. Cause — herding on stale heartbeat state

The stage breakdown narrows the culprit (p50, ms):

| Policy | scheduler_route | scheduler_queue | node_queue | inference | **end_to_end** |
|---|---:|---:|---:|---:|---:|
| round-robin | **0.004** | 0.000 | 0.023 | 30.27 | **75.8** |
| least-queue | **0.003** | 0.000 | 0.023 | 33.69 | **192.4** |
| ect | **0.004** | 0.000 | 0.023 | 33.97 | **212.5** |

- **Policy selection is not slow** — route is 0.004 ms in all three.
- **Nor is it the node queue** — node_queue is an identical 0.023 ms.
- **Nor inference** — 30–34 ms, comparable.
- What grew is **the whole round trip** (75.8 → 212.5).

So requests are waiting somewhere **before reaching the node**.

### 3.1 The state the policies read is refreshed only by heartbeat

```rust
// registry.rs:110-121  - the snapshot handed to the policy
queue_depth: self.health.queue_depth,
in_flight:   self.health.in_flight,
```

`self.health` is replaced wholesale **only when a heartbeat arrives**
(`on_health_success`: `self.health = health;`).

And **the dispatch path does not update this state at all** — there is **not a
single** reference to `in_flight` or `queue_depth` anywhere in `service.rs`.

The heartbeat period is **1000 ms** at the node, with the scheduler expecting
2000 ms.

### 3.2 So decisions pile up deterministically

```text
RR-level throughput 380 inf/s
about 380 requests are dispatched between two heartbeats (1 second)
all 380 see the same (fixed) snapshot

LeastQueue.choose() = min_by(queue_depth, in_flight, ewma_inference)
with a fixed snapshot this function is deterministic
  -> all 380 pick the same node
  -> that node's queue_depth spikes at the next heartbeat
  -> the following second goes entirely to a different node
```

**Textbook herd behaviour under stale load information.** With only 2 connections
per node, the hundreds of requests piled onto one node back up behind those 2
connections — which is why **the node idles (CPU 20%) while only the round trip
grows (212 ms).** It is also why node_queue is 0: the requests are stacked in the
**transport layer**, not the worker pool.

The aggregate split looking like ~33% is **the average of alternating pile-ups
over 60 seconds**, not an even distribution.

Round-robin does not read state and distributes structurally, so it does not have
this problem.

## 4. Verdict

> **`least-queue` and `ect` are not in a usable state under this load.**
> This is not a policy-quality problem but a **state-freshness design defect.**
> Routing hundreds of requests per second on state refreshed once per second
> produces herding.

This is **not the answer to the question S0-C intended.** Whether the policies
recover the thermal-heterogeneity loss remains **undetermined** — the
implementation was not in a state to test it.

## 5. ⚠️ Two defects in this experiment

### 5.1 The thermal condition was not maintained (fatal)

The RR results differ substantially by round.

| round | RR throughput | king p50 |
|---:|---:|---:|
| 1 | **355.7** | **144.7** ← right after preheat, hot |
| 2 | 385.1 | 88.7 |
| 3 | 387.2 | 89.7 |
| 4 | 386.1 | 86.5 |
| 5 | 385.2 | 89.5 |

**The boards cooled down while the low-load policies (LQ/ECT) ran.** With CPU
busy falling from 45–51% to 17–22%, heat output dropped and the following RR
runs executed cool. From round 2 on, king p50 normalised from 144.7 to 87 —
**the thermal heterogeneity S0-A created had disappeared.**

So this experiment stopped being "a policy comparison under fanless thermal
heterogeneity".

> **Fortunately the policy collapse itself is unrelated to temperature.** LQ/ECT
> held steady at 166–172 / 155–160 across all 5 rounds (hot or cool). Only RR
> tracked temperature, moving 355 → 385. §3's conclusion is unaffected by this
> defect.

### 5.2 Temperature collection failed

The `max_soc_c` column is entirely empty. The awk program inside ssh was wrapped
in **double quotes**, so the remote shell consumed `$1` as a positional argument.

```bash
# wrong - the remote shell substitutes $1 with an empty string
ssh "$h" 'awk "{print $1/1000}" /sys/.../temp'
# right - the awk program goes in single quotes
ssh "$h" "awk '{print \$1/1000}' /sys/.../temp"
```

`thermal-logger.sh` uses single quotes and was fine. Rewriting it inline stepped
into the same trap.

## 6. Next

1. **Fix policy state freshness** — incrementing and decrementing `in_flight` at
   dispatch time (the scheduler knows how many requests it sent) keeps the state
   current without heartbeats. A few dozen lines.
2. **Re-run S0-C after the fix** — only then can the original question be
   answered.
3. To avoid §5.1 on re-run, **the thermal condition has to be re-matched between
   policies** (reheat with RR before each policy, or a separate session per
   policy).

## 7. Conclusion

**Turning on load-aware policies collapsed throughput by 55–58%.** The cause is
not policy quality but **state freshness** — the `queue_depth`/`in_flight` the
policies read is refreshed only by heartbeat (1 s) and never updated by the
dispatch path, so hundreds of requests per second read the same fixed snapshot
and **all choose the same node** (herding).

The nodes idle (CPU 20%) while requests stack in the transport layer
(node_queue 0, round trip 212 ms).

**The original question — do load-aware policies recover the thermal
heterogeneity loss — remains undetermined.** The implementation was not in a
state to test it, and the thermal condition vanished mid-experiment (§5.1). But
**finding this defect first matters more** — without the fix we would have been
one step from the opposite conclusion: "load-aware scheduling does not help".

---

# 2nd (after the fix) — the policies actually adapt

The fix is commit `ece4eba` — `local_in_flight` atomic reservation, a
`Reservation` RAII guard, a `select_and_reserve()` critical section, and
replacing the policies' primary signal. 98 tests pass.

## 8. Changes to method

The 1st round's fatal defect (§5.1) is fixed. **Each policy is preceded by 3
reheat runs of RR** to match the **starting** thermal state.

> What is matched is the **starting** condition. Temperature diverging during a
> policy run is left alone — if an adaptive scheduler reduces king's load and
> king cools as a result, that is itself part of the policy's effect.

It was in fact controlled — every run started at soc **81–82 °C**, with a
within-run maximum of 78.5–79.5 °C.

## 9. Results (12 runs, 4 per policy)

Error rate **0 throughout**.

| Policy | Throughput | p50 | **p95** | **p99** | king% | jack% | queen% |
|---|---:|---:|---:|---:|---:|---:|---:|
| round-robin | 373.9 ± 4.5 | 87.1 | 170.7 | 232.0 | 33.3 | 33.3 | 33.3 |
| least-queue | 380.9 ± 2.1 | 92.2 | **127.1** | **146.9** | 32.9 | 33.3 | 33.7 |
| **ect** | **384.2 ± 0.8** | 90.6 | 130.9 | 156.4 | 32.8 | 33.3 | 33.9 |

**The collapse is gone.** 169.8 / 158.5 → **380.9 / 384.2**.

### 9.1 The tail improves markedly

| | RR | least-queue | ect |
|---|---:|---:|---:|
| p95 | 170.7 | **127.1** (−25.5%) | 130.9 (−23.3%) |
| p99 | 232.0 | **146.9** (−36.7%) | 156.4 (−32.6%) |

### 9.2 Per-node latency levels out

| Policy | king | jack | queen | max/min |
|---|---:|---:|---:|---:|
| round-robin | 103.2 | 85.4 | 77.6 | **1.33×** |
| least-queue | 93.3 | 92.0 | 91.1 | **1.02×** |
| **ect** | 90.4 | 90.7 | 90.5 | **1.00×** |

### 9.3 CPU utilisation levels out too

| Policy | king | jack | queen | spread |
|---|---:|---:|---:|---:|
| round-robin | 53.9% | 47.4% | 43.6% | **10.3 pp** |
| least-queue | 54.4% | 52.5% | 50.4% | 4.0 pp |
| ect | 53.3% | 51.9% | 50.2% | 3.1 pp |

This confirms "idling" through utilisation rather than latency. Under RR, queen
idles at 43.6% while king alone is pushed to 53.9%. With a policy on, the three
converge at 50–54%.

## 10. Verdict — the rule did not fire, and that is recorded as such

The rule fixed before measuring: **a shift of ≥3 pp in king's share counts as a
"shift"**.

```text
least-queue  king 33.3% -> 32.9%   (-0.4 pp)   below the rule
ect          king 33.3% -> 32.8%   (-0.5 pp)   below the rule
```

**The threshold is not lowered.** Instead, look at why it did not fire.

### 10.1 The distribution barely moved while latency and CPU levelled out

These are not contradictory. **Least-outstanding is not a policy that moves
counts but a closed loop regulating concurrent occupancy.** Reducing the number
of requests held **concurrently** on a slow node reduces that node's queue wait
and brings its latency down. Over 60 seconds the cumulative **count** processed
comes back to similar levels as latency levels out.

That is, under this condition **levelling was achieved with only a 0.5 pp
shift.**

### 10.2 This round's thermal heterogeneity was far weaker than S0-A's

| | S0-A | S0-C 2nd |
|---|---|---|
| soc max | 86–88 °C | **78–79 °C** |
| RR per-node p50 | 156.9 / 64.7 / 66.0 | 103.2 / 85.4 / 77.6 |
| spread | **2.4×** | **1.33×** |

The 3 pp threshold was set assuming S0-A's **2.4× spread**. Under a 1.33×
condition the required shift is itself small.

The cause is the harness — scheduler restarts and a probe bench between policies
made the average load lower than S0-A's (30 minutes continuous).

> ⚠️ **Correction (4th round, §18.4).** The "78–79 °C" in the table above is an
> **instrument error**. It reads an instantaneous value after the run ends and
> falls into the inter-run cooling trough; re-aggregated from the 1-second
> thermal logger, **the 2nd round was also 86.8 °C, the same as S0-A.** So this
> paragraph's causal claim — "it was less hot because average load was lower" —
> does not hold. The actual reason the 2nd round stopped at 1.33× is **that the
> CPU downgrade diverged less** (clock spread 1.50× vs S0-A's 1.79×). The
> temperature was identical from the start.

## 11. Conclusion (2nd)

**With state freshness fixed, the load-aware policies work.** The 55–58% collapse
is gone and throughput is +1.9% (LQ) / **+2.7% (ECT)** against RR.

**The biggest gain is the tail** — p99 **−37%** (232.0 → 146.9). Per-node latency
spread goes 1.33× → **1.00×** and CPU utilisation spread 10.3 pp → 3.1 pp.
**The policies really are absorbing the thermal heterogeneity.**

ECT is slightly ahead of LQ on throughput (384.2 vs 380.9) and levels latency
completely (1.00× vs 1.02×). That matches the design of reflecting service rate
in the score. But the difference is a small 0.9%, so **it is too early to declare
ECT superior under this condition.**

### What remains

- **How much is recovered under strong thermal heterogeneity (2.4×) is still
  unmeasured.** This round was 1.33×. Reproducing S0-A's level requires
  continuous heating with no low-load interval between policies.
- The distribution shift was small, so the 3 pp rule did not fire. The rule
  stays; the next experiment needs to look at **short-window (1 s) distribution**
  to see the instantaneous shift (the bench does not yet support a per-request
  dump).

---

# 3rd (active cooling, homogeneous) — a sanity test for the default

## 12. Why it was needed

The 2nd round is a **fanless (thermally heterogeneous)** condition. What was
proved there is precisely "when node performance is heterogeneous, adaptive
scheduling using fresh state beats RR" — not that it is **always best on a normal
homogeneous cluster.**

Under active cooling the three boards run at nearly the same speed. Confirming
that the adaptive policies produce **no regression** under that condition is what
lets a default be chosen.

3N / 2 connections per node / c36 / **active cooling**, 4 runs per policy. soc
47–54 °C.

## 13. Results — no regression, and the tail actually improves

| Policy | Throughput | p50 | **p95** | **p99** | king / jack / queen p50 |
|---|---:|---:|---:|---:|---|
| round-robin | 389.9 ± 1.6 | 86.3 | 146.1 ± 1.8 | 185.6 ± 5.4 | 86.3 / 86.1 / 87.0 (1.01×) |
| **least-queue** | **389.9 ± 2.0** | 89.1 | **129.3 ± 0.9** | **151.0 ± 1.5** | 89.2 / 89.1 / 89.1 (1.00×) |
| ect | 388.6 ± 1.4 | 88.5 | 136.3 ± 0.4 | 163.2 ± 1.9 | 88.2 / 88.5 / 88.8 (1.01×) |

CPU busy is an identical 45% for all three policies. The distribution does not
move either, at 33.2–33.5% — **not moving is correct under a homogeneous
condition.**

- **No throughput regression.** LQ −0.0%, ECT −0.3%, within the decision band
  (±2%).
- **The tail improves even when homogeneous.** p99 185.6 → **151.0** (LQ,
  −18.6%) / 163.2 (ECT, −12.1%).
- p50 rises slightly (86.3 → 89.1 / 88.5, +3%). It trades a little median for a
  lot of tail.

## 14. Placing the two conditions side by side

| | Fanless (heterogeneous) | Active cooling (homogeneous) |
|---|---|---|
| **RR** | 373.9±4.5 · p95 170.7**±19.9** · p99 232.0**±34.7** | 389.9±1.6 · p95 146.1±1.8 · p99 185.6±5.4 |
| **LQ** | 380.9±2.1 · p95 **127.1±0.5** · p99 **146.9±1.0** | 389.9±2.0 · p95 **129.3±0.9** · p99 **151.0±1.5** |
| **ECT** | **384.2±0.8** · p95 130.9±0.0 · p99 156.4±0.5 | 388.6±1.4 · p95 136.3±0.4 · p99 163.2±1.9 |

Three things are visible.

**① RR's tail becomes unstable under heterogeneity.** p95 SD **19.9**, p99 SD
**34.7** — against ~1 for the adaptive policies. The adaptive policies' gain is
not only "a lower tail" but **"a predictable tail"**.

**② LQ has the lowest tail under both conditions.** With SDs of 0.4–1.9 the
difference is real. Against ECT, p99 is −6.1% fanless and −7.5% cooled.

**③ ECT's throughput advantage appears only under heterogeneity.** +0.9%
fanless (384.2 vs 380.9), inverting to −0.3% when cooled.

## 15. Choosing the default

**RR drops out of contention.** It has the worst tail under both conditions, and
under heterogeneity even its predictability collapses.

**Neither LQ nor ECT dominates.**

| | LQ | ECT |
|---|---|---|
| tail (both conditions) | **lower** (p99 −6 to −8%) | |
| throughput (heterogeneous) | | **+0.9%** |
| throughput (homogeneous) | **+0.3%** | |
| design rationale | count-based | **reflects service rate** — potentially favourable as heterogeneity worsens |

> **The repository's current default is `ect`** (`policy.rs` `default()`, pinned
> by a test). Since both policies are regression-free and better than RR, **there
> is sufficient basis to keep the default.** Switching to LQ is a judgement about
> "whether to trade 6–8% of p99 for service-rate awareness", and that decision
> can wait until it is re-measured under strong heterogeneity (2.4×).

## 16. Limitations

- There is still no LQ vs ECT comparison under strong heterogeneity (S0-A's
  2.4×). The 2nd round was 1.33×.
- Three policies × 4 runs each. The p50/p95 differences are trustworthy given the
  small SDs, but the 0.3–0.9% throughput differences are **too weak to use as
  grounds for preferring one.**
- Short-window distribution was not observed (60-second aggregates only).

---

# 4th (strong heterogeneity) — deciding the LQ vs ECT default

## 17. Pre-registration — decision rules (written before the results arrive)

> This section was written **while the measurement was running**, at a point when
> `results/policy-ab-*` was still empty. It is a device for not moving the rules
> to fit the results (the same stance as when §10's 3 pp rule did not fire).

### 17.1 Conditions

| | Value |
|---|---|
| Cooling | fanless (physically off immediately before measuring) |
| Harness | `run-policy-ab.sh 4 25 5` — 4 rounds, 25 min preheat, 5 reheat runs |
| Policies | round-robin / least-queue / ect, order rotating each round |
| Everything else | 3 nodes · 2 connections/node · c36 · 60 s/run — same as the 2nd and 3rd |

**Two harness changes against the 2nd and 3rd rounds.** Both aim to shorten the
"low-load interval between policies" §10.2 identified.

1. `verify_nodes`'s probe goes from `c12` to `c36`. Node-count verification is
   independent of load, so the 10 seconds immediately before measuring are spent
   heating rather than cooling. Combined with the 14 seconds of no load during
   scheduler restart, the **low-load interval falls from 24 s to 14 s**.
2. `LOG_DUR` fixed to account for `REHEAT_RUNS`. With these parameters the old
   formula computed 2,880 s, which would have **killed the thermal logger
   mid-measurement** (the actual run took ~6,400 s).

### 17.2 The gate — was strong heterogeneity reproduced

**If reproduction fails, LQ vs ECT is not judged.** The 2nd round failed to
answer because the condition was not met, so this gate exists to avoid dressing
the same failure up as a "result".

| Metric | Criterion | Basis |
|---|---|---|
| **max/min of per-node p50 in the RR rounds** | **≥ 2.0×** | S0-A 2.4×, 2nd round 1.33×. Draw the line between them |
| soc max | ≥ 85 °C (secondary) | S0-A 86–88 °C, 2nd round 78–79 °C |

**RR is used as the heterogeneity gauge** — it does not adapt, so it shows the
raw capacity spread that surfaces under an even load. S0-A's 2.4× uses the same
definition.

- Below 2.0× → **judgement withheld.** Record it as failing the condition and do
  not touch the default.
- Around 1.33× → the harness changes were insufficient. Revisit the
  continuous-heating design.

### 17.3 LQ vs ECT decision bands

With n=4 per policy, small differences are unusable (§16). **A difference has to
clear the band to count as "winning".**

| Axis | Winning criterion | Basis |
|---|---|---|
| Throughput | relative difference **≥ 2%** | the same value as §13's regression band (±2%) |
| Tail (p99) | relative difference **≥ 5%** | 6–8% observed in the 2nd and 3rd rounds, SD ~1–2. 5% separates them |

Secondary metrics (used for interpretation, not for the verdict): the levelling
ratio of per-node p50 max/min, per-node CPU busy spread, and the size of king's
distribution shift.

### 17.4 Decision matrix

`ect` is **the incumbent** (`policy.rs` `default()`). Unseating an incumbent
requires positive grounds — this tie-break is also a pre-registered rule.

| Throughput (ECT−LQ) | Tail (LQ favoured) | Decision |
|---|---|---|
| ECT ≥ +2% | < 5% | **keep `ect`** — question closed |
| < 2% | LQ ≥ 5% | **switch to `least-queue`** — question closed |
| ECT ≥ +2% | LQ ≥ 5% | **no dominance.** Keep `ect` but state the trade-off and leave the question **open** |
| < 2% | < 5% | **indistinguishable.** Keep `ect`, question **closed** — "the default does not matter" is also an answer |

The last row is the important one. If the two cannot be distinguished even under
strong heterogeneity, then rather than hunting for a harsher condition we
**close this question** and move on to S3.9b.

### 17.5 Found mid-measurement — the soc gate's instrument was wrong

During preheat the harness printed `soc: 81 80 80`, which looked like the gate
failing. In fact **the condition was being reproduced.** From the 1-second
thermal logger at the same moment:

| | Harness output | Thermal logger (last 3 min) | S0-A |
|---|---|---|---|
| king | 81 | max **86.8** · avg 85.8 · min 78.5 | 85.9–86.8 |
| queen | 80 | max **85.9** · avg 85.6 · min 78.5 | 85.0–85.9 |
| jack | 80 | max **86.8** · avg 85.8 · min 80.4 | 85.0–85.9 |
| CPU minimum | — | **1008–1200 MHz** | 816–1800 MHz |

The cause is the sampling moment. The harness reads **after** the 60-second run
finishes, over three sequential ssh calls. RK3576 cools within seconds once load
stops, so the value falls into the inter-run trough (min 78.5–80.4) — matching
the thermal logger's min exactly. The CSV's `max_soc_c` column works the same
way and, **despite its name, is not a maximum.**

> **The gate criterion (85 °C) stays. What changes is only the data source.**
> Moving a threshold to fit results and fixing an instrument that was measuring a
> different quantity are different acts. The secondary soc gate is judged from
> **the within-run maximum from the 1-second thermal logger.** The primary gate
> (RR per-node p50 max/min) comes from the bench JSON and is unaffected.

Side effect: S0-C's 2nd-round "78–79 °C" came from the same instrument.
**Temperature during load in the 2nd round may have been higher** — which puts
§10.2's explanation, attributing the 2nd round's 1.33× to "being less hot", up
for review. The 2nd round's thermal log survives, so it can be checked.

## 18. Results (4th) — **gate not met. LQ vs ECT is not judged.**

- Raw data: [`../../results/policy-ab-20260821-contaminated/`](../results/policy-ab-20260821-contaminated)
  — **only the r1 round-robin run and the thermal log are valid.** The rest is
  invalidated by the harness collision (that directory's `README.md`,
  [S0-D](#experiments-s0-d-capacity-hetero) §4)
- After the 25-minute preheat, only r1 was measured and the run was
  **deliberately stopped** (§18.3).

### 18.1 Primary gate: 1.10× (criterion 2.0×)

```text
r1 round-robin   384.8 inf/s   p50 88.8  p95 140.2  p99 173.9  err 0
   distribution  king 33.3 / jack 33.3 / queen 33.3
   node p50      king 93.3  jack 88.4  queen 85.1   ->  max/min 1.10x
```

Lower even than the 2nd round (1.33×). **Strong heterogeneity was not
reproduced.**

### 18.2 And yet the thermal condition reproduced perfectly

The 1-second thermal logger was aggregated the same way as for S0-A (with
§17.5's instrument fix applied).

| | soc_max | soc_avg | **CPU p50** | CPU min | node p50 spread |
|---|---:|---:|---:|---:|---:|
| **S0-A** king | 86.8 | 84.3 | **1008** | 816 | |
| S0-A queen | 85.9 | 83.5 | **1800** | 1416 | |
| S0-A jack | 86.8 | 83.9 | **1800** | 1200 | **2.4×** |
| **4th** king | 86.8 | 84.3 | **1416** | 1008 | |
| 4th queen | 85.9 | 83.8 | **1608** | 1200 | |
| 4th jack | 86.8 | 84.3 | **1608** | 1008 | **1.10×** |

**The soc values match to the decimal.** What differs is only the CPU clock
distribution.

| | Clock spread (p50 max/min) | Latency spread |
|---|---:|---:|
| S0-A | **1.79×** | **2.4×** |
| 4th | **1.14×** | **1.10×** |

### 18.3 So what was wrong — the premise

The handoff and §10.2 attributed the 2nd round's 1.33× to **"being less hot"**
and prescribed continuous heating. This measurement refutes that premise.

> **Thermal conditions are necessary for heterogeneity but not sufficient.**
> The sufficient condition is **the downgrade differing per board**, and thermal
> control targets temperature, not the spread. If the three boards come down
> **together** at the same temperature, no heterogeneity appears.

S0-A's 1.79× divergence is a product of silicon, airflow and position variance,
and **is not something a cooling condition can summon.** More preheat will not
make them diverge — all three boards were already under thermal control.

Once that was clear, the remaining 11 runs (about 1 hour 20 minutes) bought
nothing beyond raising n to 4 on a negative result, so the run was stopped.
**Better to leave the verdict as not-met and change the design.**

### 18.4 §10.2's "78–79 °C" was an instrument error — **corrected**

The 2nd round's thermal log was re-aggregated
(`results/policy-ab-20260821b`). **The 2nd round was also 86.8 °C.** The
78–79 °C is an artefact of §17.5's instantaneous-value instrument.

| Experiment | soc_max (1 s logger) | Value in the document | CPU p50 | Clock spread | Latency spread |
|---|---|---|---:|---:|---:|
| S0-A | 86.8 / 85.9 / 86.8 | 85.9–86.8 ✓ | 1008 / 1800 / 1800 | **1.79×** | **2.4×** |
| **S0-C 2nd** | **86.8 / 85.9 / 86.8** | **78–79 ✗** | 1200 / 1800 / 1800 | **1.50×** | **1.33×** |
| S0-C 4th | 86.8 / 85.9 / 86.8 | — | 1416 / 1608 / 1608 | **1.14×** | **1.10×** |

**The thermal conditions of all three experiments are identical.** §10.2's
explanation ("the 2nd round was less hot") loses its basis. The real cause is
**that the downgrade diverged less.**

And the three points form a monotone series.

```text
clock spread    1.14x -> 1.50x -> 1.79x
latency spread  1.10x -> 1.33x -> 2.40x
```

What determines the size of the heterogeneity is not temperature but **the clock
spread**. Take hold of the clock directly and the heterogeneity can be placed at
any desired value (§19).

> For reference, the 1st round (`results/policy-ab-20260821`, 2 runs) had CPU p50
> at 2208 MHz on all three boards and an soc average of 77.8 °C — there was no
> downgrade at all. §5.1's diagnosis, that the thermal condition was not
> maintained, stands.

## 19. Next — make heterogeneity deterministic

Instead of waiting for heat to produce a spread, **take hold of the handle
thermal control itself uses.** The downgrade is implemented by pulling
`scaling_max_freq` down (king was observed at `1008000` during measurement), and
we can write the same file.

```text
fan ON (thermally homogeneous, cool)  +  king 1008 MHz / queen 1800 / jack 1800
   = replicating S0-A's CPU p50 profile exactly
```

Advantages:

1. **Reproducible.** No reliance on silicon luck.
2. **Heat leaves the variable set.** With the fan on, the cap holds (thermal
   control has no reason to go lower) and there is no drift. The disturbance to
   the policy comparison disappears.
3. **The spread can be swept.** Sweeping 1.0× / 1.3× / 1.8× / 2.4× yields a far
   stronger conclusion than "which one wins at 2.4×" —
   **"does ECT gain as the spread widens"**. That hypothesis is exactly ECT's
   design rationale for being the default (reflecting service rate), so the sweep
   tests that rationale directly.

The cost: since it is not thermally induced, **it cannot be called thermal
heterogeneity.** It is recorded separately as capacity heterogeneity. The causal
claim that the policies work under thermally induced heterogeneity was already
closed in the 2nd round (§11), and the remaining question is **the policies'
response to the size of the spread**, so the substitution is legitimate.

---

## Figure

![RR's p99 SD explodes only under heterogeneity (+/-34.7 vs +/-1)](../results/policy-ab-20260821b/figures/fig_policy_tail.png)

**`fig_policy_tail.png`** — RR's p99 SD explodes only under heterogeneity
(±34.7 vs ±1)

Regenerate: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s0-d-capacity-hetero"></a>

# S0-D — Capacity Heterogeneity (deterministic)

*[한국어 원문](experiments/S0_D_CAPACITY_HETERO.ko.md)*

- Experiment ID: **S0-D**
- Started: 2026-08-21
- Status: **stage 1 calibration complete** (12 runs, 0 errors). Stage 2 policy A/B outstanding
- Predecessors: [`S0_C_POLICY_AB.md`](#experiments-s0-c-policy-ab) §18–19 · [`S0_SUSTAINED_LOAD.md`](#experiments-s0-sustained-load)

---

## 1. Research Question

> **Does ECT gain over LQ as the capacity spread between nodes widens?**

S0-C closed the question as far as "adaptive beats RR when there is a spread"
(§11). What remains is **which of LQ and ECT should be the default**, and across
two conditions (heterogeneous 1.33× / homogeneous) neither dominated.

The design rationale for making ECT the default is that **it reflects service
rate in its score**. If so, **ECT should gain as the spread widens.** This
experiment tests that hypothesis directly. It is a stronger question than "which
is better at 2.4×".

## 2. Why manipulate the clock rather than heat

The S0-C 4th attempt tried to reproduce strong heterogeneity through continuous
fanless heating and failed (§18).

```text
all three experiments: soc 86.8 / 85.9 / 86.8 C - thermal conditions identical
what differed was only the board-to-board spread in CPU downgrade

  clock spread    1.14x -> 1.50x -> 1.79x
  latency spread  1.10x -> 1.33x -> 2.40x     (S0-C 4th / S0-C 2nd / S0-A)
```

**Thermal conditions are necessary for heterogeneity but not sufficient.** The
sufficient condition is the downgrade differing per board, and thermal control
targets temperature, not the spread. If all three boards come down together at
the same temperature no heterogeneity appears, and that divergence is a product
of silicon, airflow and position — **it cannot be summoned by cooling.**

So we take hold of **the handle thermal control itself uses.** The downgrade is
implemented by lowering `scaling_max_freq` (king was observed at `1008000`
during measurement). We use the same file.

| | Thermally induced (S0-A/C) | **Clock cap (S0-D)** |
|---|---|---|
| Reproducibility | depends on silicon luck | **deterministic** |
| Thermal disturbance | mixes into the policy comparison | **fan ON — removed as a variable** |
| Specifying the spread | not possible | **set to 1.3× / 1.8× / 2.4×** |

The cost: since it is not thermally induced, **it cannot be called thermal
heterogeneity.** It is capacity heterogeneity. The causal claim that the policies
work under thermally induced heterogeneity was already closed in S0-C's 2nd
round, and the present question is **how the policies respond to the size of the
spread**, so the substitution is legitimate.

## 3. Method — stage 1 calibration

`scripts/run-capacity-calibration.sh`

- **Fan ON**, boards idle at 42–47 °C. Thermal control has no reason to go below
  the cap.
- Policy **fixed at round-robin** — it does not adapt, so raw capacity spread
  shows through directly under an even load. S0-A's 2.4× uses the same
  definition.
- king's CPU cap on a ladder: **2208 / 1608 / 1200 / 1008 / 816 / 600 MHz**
  (on both `policy0` and `policy4`, each clamped to its group ceiling)
- Per cap: c36 · 60 s × 2. Three nodes, 2 connections/node — the operating point
  as-is.
- Each run reads `scaling_cur_freq` back to confirm **the cap holds under load**.
  An EXIT trap makes sure an interruption does not leave king downgraded.

**Why S0-A's clocks are not simply replicated**: the thermal logger records only
`cpu4`, so the little-core values are unknown. Rather than matching clocks, it
is more honest to **match the observed quantity (per-node p50 spread)
directly**. Calibration gives the cap → spread mapping.

## 4. Incident record — the first attempt was discarded to a harness collision (2026-08-21)

The first calibration attempt produced an **uncapped baseline of 197.4 inf/s**
(391.2 is normal). Node spread 2.73×, and the next run had an **82.4% error
rate**. It looked like "only king and jack are slow" and **we were one step from
misdiagnosing a cluster failure.**

The cause was not a failure. **The policy A/B harness had not died.**

```text
believed   the policy A/B harness was stopped via TaskStop
actually   only the wrapper shell died; the child bash kept running
result     two harnesses hitting the same 3 nodes at c36 each (72 combined)
```

The surviving harness kept **restarting the scheduler with its own
configuration** (`scheduler-s0c.toml`). So the "restore to default settings"
step was being overwritten seconds later, invisibly.

**Observation lied.** In git-bash it did not appear in `ps -ef` and `pkill -f`
could not catch it. Only PowerShell's `Get-CimInstance Win32_Process` showed it.

```powershell
Get-CimInstance Win32_Process -Filter "Name='bash.exe'" |
  ? { $_.CommandLine -match 'scripts/run-' } | select ProcessId,CommandLine
```

After cleanup and re-measurement: **391.2 inf/s · p50 86.2 · 0 errors · spread
1.02×.** The cluster had been fine all along.

### 4.1 Preventing recurrence

`npuforge_assert_cluster_free` (`scripts/lib/remote.sh`) was added and wired into
the start of the policy A/B and capacity calibration harnesses. **If
`npuforge-bench` is running on the server, it does not start — it stops loudly.**

The point is verifying **at the shared resource** rather than checking local
processes — local observation lies depending on the platform, but a bench
running on the server does not.

### 4.2 Contaminated data

- **First calibration attempt — discarded.** Re-measured under the same
  conditions (§5).
- **The 4th policy A/B in progress — renamed to
  `results/policy-ab-20260821-contaminated/` and kept.** It is not deleted
  because the incident itself is a methodology record (README §4.11). That
  directory's `README.md` states which parts are valid and which are not —
  **only the r1 round-robin run and the 1-second thermal log are valid** — and
  S0-C §18's gate verdict rests on just those two, so it is unaffected.

> ⚠️ The 4th harness **reused the same dated path and overwrote S0-C's 1st-round
> data (15 runs).** It was restored with `git checkout`. The harness output path
> is `results/policy-ab-<date>`, so running twice in a day overwrites — it must
> use `NPUFORGE_SUFFIX` or stop when the directory already exists.

## 5. Results — calibration (12 runs, error rate 0)

- Raw data: [`../../results/capacity-calib-20260821/`](../results/capacity-calib-20260821)
- Fan ON, boards at 48–55 °C. **Thermal control never intervened** — the
  `scaling_cur_freq` read back each run always matched the specified cap.

| king cap (MHz) | throughput | king p50 | jack p50 | queen p50 | **spread** |
|---:|---:|---:|---:|---:|---:|
| 2208 (uncapped) | 388.1 | 83.8 | 86.6 | 89.4 | **1.12×** |
| 1608 | 382.9 | 96.3 | 83.3 | 81.6 | **1.18×** |
| 1200 | 379.6 | 103.6 | 83.3 | 77.7 | **1.33×** |
| 1008 | 369.0 | 127.7 | 72.4 | 72.7 | **1.79×** |
| **816** | **359.6** | **149.8** | **67.9** | **66.3** | **2.26×** |
| 600 | 318.4 | 213.5 | 54.4 | 54.5 | **3.93×** |

Spread reproducibility: the 2 runs at each cap landed within ±0.05 of each other
(e.g. 816 → 2.30 / 2.22).

### 5.1 A cap of 816 reproduces S0-A almost exactly

| | king p50 | jack p50 | queen p50 | spread | throughput |
|---|---:|---:|---:|---:|---:|
| **S0-A** (thermal, fanless 86 °C) | 156.9 | 64.7 | 66.0 | **2.4×** | 345.4 |
| **cap 816** (clock, fan ON 50 °C) | 150.9 | 67.3 | 65.6 | **2.30×** | 359.8 |

All three node latencies **overlap within 6 ms.** It also fits the fact that
king's **CPU minimum in S0-A was 816 MHz** — we specified directly the floor the
thermal downgrade had pushed it to.

> **Strong heterogeneity can be produced deterministically.** The condition that
> clears S0-C §17.2's gate (2.0×) is now reproducible, with no 30-minute preheat
> and no silicon luck required.

### 5.2 Side observation — under RR a slow node idles the fast ones

As the cap comes down, **king gets slower while jack and queen actually get
faster.**

```text
king  cap 2208  83.8ms  ->  cap 600  213.5ms   (2.5x slower)
jack  cap 2208  86.6ms  ->  cap 600   54.4ms   (1.6x faster)
queen cap 2208  89.4ms  ->  cap 600   54.5ms   (1.6x faster)
```

At fixed c36 with RR, the client's 36 slots are split evenly across three nodes.
When king slows, **more slots are tied up waiting on king**, so fewer requests
are in flight on jack and queen and those two run underloaded. A p50 of 54 ms
means they are idling.

So the −18% throughput loss at cap 600 (388.1 → 318.4) is not king's capacity
loss alone; it includes **the share RR fails to use from the two idling nodes.**
The ceiling on what an adaptive policy can recover is right there. It is the same
phenomenon S0-A observed — "king is 2.4× slower and requests are still exactly
1/3" — except that this time **the spread can be specified and its size dialled.**

## 6. Stage 2 policy A/B — **future work (not being done now)**

Calibration produced the mapping, so it can be run at any time. But **it is not
the priority right now** — which of ECT and LQ wins does not change NPUDure's
central conclusion (§7). The main line is S3.9b.

The design when it is run:

```bash
# caps 1200 / 1008 / 816 / 600  =  spreads 1.33 / 1.79 / 2.26 / 3.93x
# 3 policies x 4 spreads x 3 runs; fan ON so no preheat - about 40 minutes
```

- The decision bands are taken unchanged from S0-C §17.3 (throughput 2%, p99 5%).
- Hypothesis: **ECT's throughput advantage grows as the spread widens.** If it
  does not, ECT's design rationale (reflecting service rate) is refuted by
  measurement.
- Because the spread is treated as **a continuous variable**, the conclusion is
  stronger than "which one at 2.4×" — it asks whether the advantage increases
  monotonically with the spread.

## 7. Where this lineage currently stands

Combining the calibration result with the policy lineage as a whole:

1. **RR is vulnerable to heterogeneity.** It keeps sending 1/3 to a slow node,
   and under heterogeneity even the predictability of the tail collapses
   (p99 SD 34.7 vs ~1).
2. **Fresh-state adaptive scheduling improves RR's tail markedly.** p99 −37%,
   node latency spread 1.33× → 1.00× (S0-C §9).
3. **LQ and ECT both work.** No regression under either condition.
4. **Whether ECT wins under strong heterogeneity is undetermined.**
5. **But that outcome does not change NPUDure's central conclusion.** The core
   is "load-aware scheduling with state freshness fixed absorbs heterogeneity",
   and that holds with either LQ or ECT. The default stays `ect`.
6. **What S0-D leaves behind is not an answer but a fixture** — apparatus for
   testing that question **reproducibly**, whenever.

---

## Figure

![Cap to spread mapping. 816 MHz reproduces S0-A (2.4x)](../results/capacity-calib-20260821/figures/fig_capacity_calibration.png)

**`fig_capacity_calibration.png`** — cap → spread mapping; 816 MHz reproduces
S0-A (2.4×)

Regenerate: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s0-sustained-load"></a>

# S0 — Sustained Load (condition A fanless / condition B active cooling)

*[한국어 원문](experiments/S0_SUSTAINED_LOAD.ko.md)*

- Experiment ID: **S0-A · S0-B**
- Measured: 2026-08-21
- Code: `bb3f7ab` + `[transport] node_connections = 2`
- Status: **both complete** (30 runs × 60 s each ≈ 31 minutes continuous)
- Raw data: [`../../results/sustained-20260821-fan/`](../results/sustained-20260821-fan) ·
  [`../../results/sustained-20260821-fanless/`](../results/sustained-20260821-fanless)
- Predecessor: [`S3_8_OPTIMIZED_SCALEOUT.md`](#experiments-s3-8-optimized-scaleout)

---

## 1. Research Question

> **Does the short-run operating point hold under sustained load? And how much
> does that answer depend on the cooling condition?**

**Every** measurement so far has been 60 seconds or less — the region before
throttling appears.

```text
short-run operating point    based on benchmarks of 60 s or less
sustained operating point    based on thermal steady state
```

## 2. Method

- The operating point as-is: **3 nodes, 2 connections per node, c36** (= c12 per
  node).
- **60-second runs × 30 consecutively**, with **no restart** of nodes or
  scheduler.
- `thermal-logger.sh` on all three boards at **1-second intervals** — four
  temperatures, CPU MHz, NPU MHz, voltage.
- Each run records **the number of responding nodes and peak NPU temperature**.
  If a node is excluded fanless by hitting a threshold (degraded 80 / disable
  90 °C), the throughput drop is a **reduction in node count**, not throttling.
  The two have to be distinguished.
- The decision rule was fixed **before measuring**: `steady = mean of the last
  third`, `degradation = 1 − steady/peak`. <3% none / 3–10% slight / >10%
  pronounced.
- The two conditions **start from similar idle temperatures** (fan 40.7–41.6 °C,
  fanless 38.8–40.7 °C). At idle the fan does little, so this is a fair A/B with
  matched starting points.

## 3. Results

Error rate **0 throughout in both**. **Zero node exclusions** (even fanless
never reached the 90 °C threshold).

| | **B: active cooling** | **A: fanless** |
|---|---:|---:|
| peak | 387.7 | 389.4 |
| **steady (last third)** | **380.3 ± 2.2** | **345.4 ± 3.8** |
| **degradation** | **1.9%** | **11.3%** |
| soc max | 58.2 – 61.0 °C | **85.9 – 86.8 °C** |
| npu max | 59.2 – 61.0 °C | **86.8 – 87.8 °C** |
| **CPU minimum** | **2208 MHz (0 downgrades)** | **816 / 1200 / 1416 MHz** |
| NPU minimum | 950 MHz | **950 MHz (no downgrade)** |
| Node exclusions | 0 | 0 |

Over time:

| t+min | B throughput | A throughput | A vs peak |
|---:|---:|---:|---:|
| 1 | 387.7 | 389.4 | 100.0% |
| 5 | 385.8 | 380.9 | 97.8% |
| 10 | 382.5 | 359.7 | 92.4% |
| 15 | 381.7 | 356.2 | 91.5% |
| 20 | 380.2 | 355.8 | 91.4% |
| 25 | 382.3 | 342.1 | 87.9% |
| 30 | 377.3 | 343.5 | 88.2% |

## 4. Interpretation

### 4.1 Cooling was holding the operating point up

Under active cooling: **1.9%** — "no degradation" by the decision rule. Clock
downgrades number **zero** across all ~1,660 samples per board. Temperature
reaches a 58–61 °C plateau within five minutes, with over 20 °C of headroom to
the threshold.

**Remove the fan and it is 11.3%.** The two operating points diverge.

```text
short-run operating point                       3N 387-389 inf/s
sustained operating point (active cooling)      3N 380.3      (-1.9%)
sustained operating point (fanless)             3N 345.4      (-11.3%)
```

> **"There is / is not throttling" has to be written with its conditions.**
> Same hardware, same operating point, same load — and cooling alone changes the
> conclusion.

### 4.2 The NPU was never downgraded — what was downgraded is the CPU

**The NPU stayed pinned at 950 MHz in both conditions.** Fanless, even with NPU
temperature reaching 87.8 °C, the clock did not drop.

What was downgraded is the CPU. And **it differs per board.**

| Board | CPU minimum | soc max |
|---|---:|---:|
| **king** | **816 MHz** (−63%) | 86.8 °C |
| jack | 1200 MHz (−46%) | 86.8 °C |
| queen | 1416 MHz (−36%) | 85.9 °C |

This is precisely the point the worklog recorded as its fourth mistake —
"judging throttling by NPU clock alone" (discuss §3.1). **This measurement
re-confirms that lesson.**

### 4.3 The real finding — round-robin keeps hitting the downgraded node

Per-node latency over the last 5 fanless runs:

| | p50 | p95 | **share** |
|---|---:|---:|---:|
| jack | 64.7 | 107.0 | **33.3%** |
| **king** | **156.9** | **313.9** | **33.3%** |
| queen | 66.0 | 107.4 | **33.3%** |

**king is 2.4× slower than the other two and still receives exactly one third of
the requests.** Round-robin looks at neither load nor state.

Under active cooling the three nodes sit evenly at 85.2–90.3 ms. They diverge
only when fanless.

And queen and jack actually got **faster** fanless (85–90 → 65–66 ms), because
total throughput fell and with it the per-node load.

> The large latency drop is **a strong signal that queue pressure fell**, but
> concluding "they are idling" would need per-node **CPU idle or outstanding
> queue depth**. This measurement has neither — S0-C records them alongside.

> ⚠️ **Everything to this point is observation; what follows is hypothesis.**
>
> **Confirmed**
> - There is a thermal spread (CPU 816 / 1200 / 1416 MHz)
> - king's service capacity really is lower (p50 2.4×)
> - RR keeps sending 33.3% to the slowed king
>
> **Not yet**
> - "fanless loss = thermal spread × load-blind policy" — the final causal link
>   closes only when changing the policy **actually recovers** the loss.
>
> `least-queue` and `ect` are implemented in the repository but have no
> validation on real hardware. **S0-C closes this link** (§8).
>
> A negative result matters too — if the split stays at 1/3 or performance is
> unchanged with the policies on, that means **the current policies' state
> signal does not detect thermal-induced capacity degradation.**

### 4.4 Relation to the original −27%

| | Original (discuss §12) | This S0-A |
|---|---|---|
| Load | local, 8 threads (CPU saturated) | cluster (CPU headroom) |
| Cooling | fanless | fanless |
| NPU temperature | 90.4 °C | 87.8 °C |
| CPU downgrade | 2208 → **816 MHz** | 2208 → **816 MHz** (king) |
| Result | **−27%** | **−11.3%** |

**The CPU fell to the same 816 MHz.** Yet the loss is less than half. Cluster
operation leaves the board CPU 49–63% idle (S3.5, S3.7c) so it is less affected
by the downgrade, and only one of the three boards fell to the worst case.

→ **−27% was not wrong. The conditions were different.**

## 5. Limitations

- **Load-aware policies not measured** (§4.3). Only round-robin was used. How
  much of the fanless loss `least-queue`/`ect` recover **is a hypothesis, and
  the thing to be tested**.
- This is 31 minutes. Temperature reached a plateau so longer runs are unlikely
  to differ much, but that is an **estimate**.
- Room temperature was not controlled. The two conditions were measured
  back-to-back on the same day.
- 2–4 second gaps between runs (§2).
- One 3-node operating point only. 1N and 2N were not measured.
- Even fanless never reached the 90 °C threshold, so **node-exclusion behaviour
  is unverified.**

## 6. Reproduction

```bash
bash scripts/run-sustained-load.sh 30 fan       # condition B
bash scripts/run-sustained-load.sh 30 fanless   # condition A (fan removed)
PYTHONIOENCODING=utf-8 python scripts/analyze-sustained.py \
    results/sustained-20260821-fanless
```

## 7. Conclusion

**Under active cooling the short-run operating point holds under sustained load**
(degradation **1.9%**, 0 clock downgrades). The 60-second results from S2 through
S3.9a apply unchanged to continuous operation.

**Remove the fan and it widens to 11.3%.** What is downgraded is not the NPU but
the **CPU** (pinned at 950 MHz vs 2208 → 816 MHz), and by different amounts per
board.

The most valuable finding is §4.3 — **king became 2.4× slower and round-robin
still sends it one third.** To RR the three nodes are identical; their actual
service capacity already is not.

**This is the proving ground for adaptive scheduling.** Validating load-aware
policies on real hardware is hereby **promoted from a functional item to a
performance item** → **S0-C**.

That said, "loss = thermal spread × policy" is **still a hypothesis** (see the
note in §4.3). Causality closes only once changing the policy is seen to recover
the loss.

## 8. Next — S0-C (do it before turning the fan back on)

**Under active cooling the three nodes are nearly homogeneous, so policy
differences are likely to vanish.** The current fanless state is the best
condition for validating the policies. Close the causal check before cooling
down.

| Policy | Throughput | p95 | p99 | king share | jack share | queen share |
|---|---:|---:|---:|---:|---:|---:|
| round-robin | 345.4 | ? | ? | 33.3% | 33.3% | 33.3% |
| least-queue | ? | ? | ? | ? | ? | ? |
| ect | ? | ? | ? | ? | ? | ? |

What we want to see is not simply higher throughput. If, for example, ECT shifts
to something like `king 15% / jack 42% / queen 43%` while moving 345 towards
370–380, then this can be said:

> **Thermal heterogeneity reduces node capacity, and state-aware scheduling
> recovers performance by adapting load allocation to heterogeneous service
> rates.**

Design constraints to respect:

- **Heat thoroughly into thermal steady state first**, then compare. If the
  starting temperature differs per policy, the policy effect and thermal drift
  get mixed together.
- Rotate the policy order.
- Beyond throughput, p95 and p99, record **per-node distribution and per-node
  latency**, plus **per-node CPU idle**.

---

## Figure

![31 minutes continuous - active cooling -1.9% vs fanless -11.3%](../results/sustained-20260821-fanless/figures/fig_sustained_thermal.png)

**`fig_sustained_thermal.png`** — 31 minutes continuous; active cooling −1.9%
vs fanless −11.3%

Regenerate: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s2-grpc-baseline"></a>

# S2 — gRPC Multi-node Scaling Baseline

*[한국어 원문](experiments/S2_GRPC_BASELINE.ko.md)*

- Experiment ID: **S2**
- Measured: 2026-08-20
- Frozen commit: `254d560` (no code or configuration changes during measurement)
- Status: **complete · reproduction confirmed (30 runs)**
- Raw data: [`../../results/baseline-20260820/raw/`](../results/baseline-20260820/raw) · figures: [`figures/`](../results/baseline-20260820/figures) · dashboard: [`dashboard.html`](../results/baseline-20260820/dashboard.html)

---

## 1. Research Question

> **Does aggregate inference throughput increase approximately linearly as
> identical low-cost NPU nodes are added to an Ethernet-connected edge cluster?**

With low-cost edge NPUs (RK3576, 6 TOPS) tied together over Ethernet, **does
total inference throughput grow close to linearly as nodes are added?** The
question is measured scaling efficiency, not the sum of nominal TOPS.

## 2. Hypothesis

Under the data-parallel design ([`adrs/001`](../adrs/001-data-parallel-only.md)),
nodes handle different requests independently of one another. Since no
node-to-node communication sits in the inference path, **throughput should be
linear in node count as long as the single central scheduler does not become a
bottleneck.** At the same time, going through the cluster (gRPC + network)
should reduce per-node throughput **by some fixed proportion** against local
direct inference — the overhead.

## 3. System Under Test

| Item | Value |
|---|---|
| Board | NanoPi R76S ×3 (king / queen / jack) |
| SoC / NPU | Rockchip RK3576 / 2-core 6 TOPS |
| Model | YOLOv8n **INT8** (sha256 `dba155d2…`), `want_float=0` |
| Input | raw RGB 640×640×3 = 1,228,800 byte/request |
| Scheduler host | server (.9): Xeon E5-2630L ×2 (24T) / 16 GB / Rocky 9.4 |
| Network | worker 2.5GbE / aggregation 10GbE (NEXI NS-S25G10G-N) |
| Transport | **gRPC** (tonic + protobuf) |
| Topology | client → scheduler(.9) → node, all 3 hops gRPC |

Topology and rationale: [`adrs/014`](../adrs/014-10g-aggregation-separate-scheduler.md),
[`docs/infrastructure.md`](#infrastructure).

## 4. Experimental Controls

Held fixed across every run.

```text
Cooling      : Active cooling - 120mm 5V USB fan per node (from the start)
CPU governor : performance
Policy       : round-robin
Worker count : 8 / node  (a dedicated RKNN context per thread, adrs/007)
Transport    : gRPC
Model        : YOLOv8n INT8, want_float=0
Warmup       : excluded
```

- **Cooling is active (fan ON).** Not fanless — see
  [`docs/board-worklog.md`](#board-worklog) §2.24 and §2.27.
- `preflight-check.sh` passed before measuring (alias↔hostname, hashes,
  governor, temperature, voltage, NTP).

## 5. Measurement Method

- Load tool: `npuforge-bench` (**closed-loop**), run on server (.9).
- Equal load per node: **concurrency = 8 × node count** (1N c8 / 2N c16 / 3N c24).
- **10 runs of 60 s** per condition. 30 runs total.
- **Condition order rotates** so that drift in time or temperature does not
  land on one condition:
  ```text
  Round 1: 1N -> 2N -> 3N
  Round 2: 2N -> 3N -> 1N
  Round 3: 3N -> 1N -> 2N   (repeating)
  ```
- Reducing node count means stopping the process; cooldown between runs.
- Script: [`scripts/run-grpc-baseline30.sh`](../scripts/run-grpc-baseline30.sh).
  Code and configuration frozen for all 30 runs.

> Because the bench is closed-loop, absolute latency is never quoted as an SLA
> — it is used only **for comparison between configurations**
> ([`adrs/028`](../adrs/028-bench-run-validity.md)).

## 6. Validation / Integrity Checks

All 30 runs checked. **This is the basis for trusting the measurement.**

| Check | Result |
|---|---|
| Run count | 30 / 30 |
| Active-node determination | **30/30 correct** (n1=1, n2=2, n3=3) |
| Invalid runs (verdict) | 0 |
| Error rate (inference) | **0.00%** (every run) |
| Retries | 0 |
| Load-balance deviation | **0.00 pp** |

- Active node is determined from **the nodes that actually served requests**
  (`per_node`), not from registered nodes. A bench fix resolved the problem of
  registrations persisting after a node was stopped (board-worklog §2.28).
- The retry count comes from the `attempts` field in the response protocol —
  the scheduler's actual attempts.

## 7. Results

### 7.1 Throughput

| Nodes | Throughput Mean ± SD |
|---:|---:|
| 1 | **112.9 ± 0.5** inf/s |
| 2 | **229.0 ± 0.9** inf/s |
| 3 | **338.4 ± 1.1** inf/s |

SD of 0.5–1.1 is extremely small — throughput barely moved across 30 runs. The
first measurement of 337.7 reproduced as 338.4 ± 1.1.
→ [fig1](../results/baseline-20260820/figures/fig1_throughput_vs_node.png)

### 7.2 Speedup

| Reference | 2N | 3N |
|---|---:|---:|
| 1-node c8 (112.9) | 2.03× | **3.00×** |
| single-node saturation (~115) | 1.99× | 2.94× |

### 7.3 Scaling Efficiency

Against the 1-node c8 reference: **100% / 101% / 100%**; against saturation,
3N ≈ **98%**.
→ [fig2](../results/baseline-20260820/figures/fig2_scaling_efficiency.png)

### 7.4 Latency (round-trip, closed-loop)

**The 30-run average of run-level percentiles** (see the caveat in §7.4.1):

| Nodes | p50 | p95 | p99 |
|---:|---:|---:|---:|
| 1 | 68.0 | 100.8 | 116.3 ms |
| 2 | 67.0 | 100.1 | 118.6 ms |
| 3 | 67.6 | 102.7 | 123.9 ms |

The latency distribution stays nearly flat as nodes are added — scaling does not
degrade latency.

#### 7.4.1 Caveat — these are not pooled percentiles

Percentiles are computed within each run over that run's requests
(nearest-rank, `stats.rs`), and **those run-level values are then averaged**.
This differs from pooling all 30 runs' requests and re-sorting.

```text
what was used   mean( p99(run1), p99(run2), ..., p99(run30) )
what it is not  p99( run1 u run2 u ... u run30 )
```

In general, **run-level averaging makes the tail read lower than pooled** —
each run's worst window is diluted by the average. This is fine for *comparing*
configurations (every condition is treated the same way), but **the absolute
values must not be quoted as "this system's p99".**

Producing pooled percentiles requires the per-request latency source, and the
bench only writes summary percentiles to JSON. Adding a raw dump option is
filed in `TODO.md` §1.2.
→ [fig4](../results/baseline-20260820/figures/fig4_latency_percentiles.png)

### 7.5 Load Distribution

Round-robin split the three nodes at **exactly 33.3% each** (deviation 0.00 pp).
→ [fig5](../results/baseline-20260820/figures/fig5_per_node_distribution.png)

## 8. Timing Breakdown

The 11 stages of the response `Timing` (proto), 30-run average of p50 (ms):

| Stage | 1N | 3N |
|---|---:|---:|
| scheduler_queue | 0.00 | 0.00 |
| scheduler_route | 0.00 | 0.00 |
| **network_to_node** (input) | 17.72 | 17.11 |
| node_queue | 0.02 | 0.02 |
| **inference (NPU)** | 24.70 | 22.49 |
| **network_to_client** (output) | 17.72 | 17.11 |
| **end_to_end** | 61.54 | 58.83 |

```text
non-inference overhead = end_to_end - inference = 58.83 - 22.49 = 36.34 ms
payload transfer       = network_to_node + network_to_client = 34.21 ms
```

- `scheduler_queue` and `scheduler_route` are **~0** regardless of node count —
  a single scheduler is not a bottleneck even with three nodes
  ([`adrs/003`](../adrs/003-central-simple-scheduler.md) confirmed by
  measurement).
- `network_to_node` for 1N and 3N is nearly identical (17.72 vs 17.11) — the
  transfer time of a single request is independent of node count.
- → [fig7](../results/baseline-20260820/figures/fig7_timing_breakdown.png)

## 9. Local vs Cluster Overhead

| Mode | Cooling | Worker | Throughput |
|---|---|---:|---:|
| Local direct RKNN (no gRPC) | Active Cooling | 8 | 161.5 inf/s |
| Cluster gRPC (single node) | Active Cooling | 8 | 112.9 inf/s |

**Throughput loss = (161.5 − 112.9) / 161.5 = 30.1%.**
The local baseline was re-measured with cooling and worker count matched to the
cluster (board-worklog §2.27).
→ [fig8](../results/baseline-20260820/figures/fig8_local_vs_cluster.png)

> ⛔ **Do not multiply the two quantities.** Throughput loss (30.1%, a
> throughput figure) and the latency breakdown (94%, a share of latency) are
> different axes. Use the wording in §10.

## 10. Interpretation

**Finding 1 — near-linear scaling (reproduced).**

> Three-node throughput reached **3.00×** the one-node c8 baseline and **~98%**
> of the single-node saturation-derived ideal. All 30 runs completed without
> inference errors or retries, with effectively uniform round-robin distribution.

**Finding 2 — node-level overhead is payload transfer.**

> Local direct inference reached **161.5 inf/s** while single-node cluster
> throughput reached **112.9 inf/s**, a **30.1% throughput reduction**.
> Separately, latency decomposition showed that **94% of non-inference latency
> was observed in the payload-transfer path** — not in serialization, scheduler
> queueing, or node queueing (all ~0).

The two reinforce each other. Scaling is linear because neither the scheduler
nor the network bottlenecks on node count (Finding 1), while the absolute
per-node ceiling is cut by the time it takes to carry the payload over 2.5G
(Finding 2). What optimization should aim at is not compute but **transport**.

## 11. Limitations

- **The measurement window is short (60 s / 30 runs).** CPU throttling shows up
  at −27% over 300 s (board-worklog §2.24), so this result sits **before the
  throttling region**. Sustained-load throughput is settled in a separate
  experiment (S0).
- **Cooling axis.** Active cooling only. There is no fanless (condition A)
  cluster measurement here.
- **Saturation not established.** 1N was seen near ~115 at c8/c16/c32 but c48
  was not measured, and the 2N/3N ceilings were not swept → **S3**.
- **Serialization not measured in isolation.** The proto `Timing` has no field
  for gRPC serialization alone; it currently sits inside the ~2 ms non-inference
  residual. An additional instrumentation point is needed.
- **Closed-loop.** Not absolute latency; for comparison between configurations
  only.
- **A single 2-node combination (king+queen).** Other combinations such as
  king+jack were not measured.

## 12. Reproduction

```bash
# after bringing up the 3-node cluster (scheduler + king/queen/jack)
bash scripts/run-grpc-baseline30.sh        # 30 runs -> server:/tmp/baseline30
# local fan baseline (Finding 2):
ssh npuforge-k 'pkill -9 npuforge-node; sleep 3; cd ~/npuforge-rknn-test; \
  ./sustained_load_test yolov8n-int8.rknn 60 8'
# regenerate figures:
python scripts/make-figures.py
```

Frozen commit: `254d560`. The fixed-condition table is §4.

## 13. Raw Data

- 30 bench JSON files: [`../../results/baseline-20260820/raw/`](../results/baseline-20260820/raw)
  (`n{nodes}_r{round}.json`; each carries throughput, latency, node_inference,
  TimingBreakdown, per_node, nodes_before/after (temp, voltage), verdict, run_id)
- Aggregate report: [`../../results/baseline-20260820/README.md`](../results/baseline-20260820/README.md)
- Figures and dashboard: [`figures/`](../results/baseline-20260820/figures), [`dashboard.html`](../results/baseline-20260820/dashboard.html)

## 14. Conclusion

Across 30 repeated runs the RK3576 3-node NPU cluster showed **near-linear
scaling (338.4 ± 1.1 inf/s, 3.00×, error 0%)**. The TimingBreakdown confirmed
that per-node overhead lies not in compute or the scheduler but in the
**payload-transfer path** (94% of non-inference latency).

→ The gRPC baseline is **frozen**. Next: **S3** (saturation / scaling limit) →
**S4** (io_uring). S4 compares transport cost against this baseline under
**identical conditions**.

---

<a id="experiments-s3-5-transport-profile"></a>

# S3.5 — Transport Cost Profiling

*[한국어 원문](experiments/S3_5_TRANSPORT_PROFILE.ko.md)*

- Experiment ID: **S3.5** (+ **S3.5b** RPS A/B)
- Measured: 2026-08-20
- Frozen commit: `01f29a2`. Node, scheduler, model and bench **unchanged**
- Status: **complete**
- Raw data: [`../../results/transport-profile-20260820/raw/`](../results/transport-profile-20260820/raw) ·
  [`../../results/rps-ab-20260820/`](../results/rps-ab-20260820)
- Predecessors: [`S2_GRPC_BASELINE.md`](#experiments-s2-grpc-baseline), [`S3_SATURATION.md`](#experiments-s3-saturation)
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
- Scripts: [`run-transport-profile.sh`](../scripts/run-transport-profile.sh),
  [`node-profile-collect.sh`](../scripts/node-profile-collect.sh),
  [`analyze-transport-profile.py`](../scripts/analyze-transport-profile.py).

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
> → [S3.8 §4.3](#experiments-s3-8-optimized-scaleout)
>
> An exclusion verdict has to carry **the conditions it was reached under**.

### 4.2 Board CPU capacity — no

All 8 cores are **63.1% idle**. Even the busiest, CPU0, has 30.3% left.

### 4.3 Kernel softirq concentration (CPU0) — **refuted by A/B**

The profile pointed at CPU0 as unusually busy (69.7% busy, of which 51.5%
softirq; the other cores are 27–38%). eth0 has **one RX queue**, its IRQ is
pinned to CPU0, and **RPS is off** (`rps_cpus=00`,
[`nic-topology.txt`](../results/transport-profile-20260820/raw/nic-topology.txt)).
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
  ([`driver.rs:83-90`](../crates/npuforge-bench/src/driver.rs)).
- The scheduler caches and reuses **one channel per node**
  ([`node_client.rs:31-79`](../crates/npuforge-scheduler/src/node_client.rs)).
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

---

<a id="experiments-s3-6-h2-channel-ab"></a>

# S3.6 — HTTP/2 Window × Connections-per-Node A/B

*[한국어 원문](experiments/S3_6_H2_CHANNEL_AB.ko.md)*

- Experiment ID: **S3.6**
- Measured: 2026-08-20
- Code: `11cec9b` + the `[transport]` settings added (defaults behave identically to the freeze)
- Status: **complete (20 runs, 4 conditions × 5 rounds)**
- Raw data: [`../../results/h2-channel-ab-20260820/`](../results/h2-channel-ab-20260820)
- Predecessor: [`S3_5_TRANSPORT_PROFILE.md`](#experiments-s3-5-transport-profile)
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
- Scripts: [`run-h2-channel-ab.sh`](../scripts/run-h2-channel-ab.sh),
  [`analyze-h2-channel-ab.py`](../scripts/analyze-h2-channel-ab.py).

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
> → [`S3_7_CONNECTION_TUNING.md`](#experiments-s3-7-connection-tuning) §4.3

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

![Connections help; enlarging the window hurts](../results/h2-channel-ab-20260820/figures/fig_h2_window_vs_conns.png)

**`fig_h2_window_vs_conns.png`** — connections help; enlarging the window hurts

Regenerate: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s3-7-connection-tuning"></a>

# S3.7 — Connection Tuning (a: sweep, b: concurrency, c: RPS)

*[한국어 원문](experiments/S3_7_CONNECTION_TUNING.ko.md)*

- Experiment ID: **S3.7a · S3.7b · S3.7c** (complete)
- Measured: 2026-08-20
- Code: `4e64bf4` (the `[transport]` settings; defaults behave identically to the freeze)
- Raw data: [`../../results/connection-sweep-20260820/`](../results/connection-sweep-20260820)
- Predecessor: [`S3_6_H2_CHANNEL_AB.md`](#experiments-s3-6-h2-channel-ab)

---

## 0. What this experiment answers

S3.6 compared only 1 → 4 connections and saw +21.5%. **There was no basis for 4
being optimal**, and the throughput gain came with **p95 46% worse.**

So S3.7 frames this not as "find maximum throughput" but as a problem of
**operating point selection**.

```text
S3.7a  connections 1/2/4/8/16 at fixed c32   -> shortlist Pareto candidates   <- done
S3.7b  concurrency sweep for the shortlist   -> establish the real operating point
S3.7c  RPS OFF/ON at that operating point    -> freeze optimized gRPC
```

---

# S3.7a — Fixed-load connection-count A/B

## 1. Method

One node (king), **fixed c32**, 60 s, connections 1/2/4/8/16, **5 runs** per
condition (25 total). The window stays at its default (S3.6's conclusion: a
64 MB-class enlargement is −36.3%). The order reverses each round so temperature
and elapsed time do not land on one condition. Each run counts the node's actual
TCP connections with `ss` and records it.

> **This is not each setting's ceiling.** Load is fixed at c32, which is good for
> comparing the *pure effect* of connection count, but adding connections may
> have moved the saturation concurrency above c32. That is why S3.7b exists
> separately.

## 2. Results

Error rate **0** throughout. All latencies are **the run-to-run average of
run-level percentiles** (not pooled — S2 §7.4.1).

| conn | TCP measured | throughput | vs c1 | p50 | p95 | p99 | max | →node |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **1** | 1 | 115.6 ± 0.7 | — | 268.0 | **392.4** | **452.4** | 597.6 | 114.9 |
| **2** | 2 | **134.4 ± 0.7** | **+16.3%** | 226.7 | **438.2** | **514.9** | 679.7 | 92.5 |
| **4** | 4 | **139.5 ± 0.2** | **+20.7%** | 169.5 | 561.6 | 698.2 | 944.5 | 63.1 |
| **8** | 8 | 139.1 ± 0.6 | +20.4% | 157.4 | 597.0 | 827.0 | 1222.5 | 56.9 |
| **16** | 16 | 136.8 ± 0.7 | +18.4% | 173.4 | 584.2 | 895.1 | 1481.5 | 65.3 |

- Main figure: [`fig_sweep_pareto.png`](../results/connection-sweep-20260820/figures/fig_sweep_pareto.png)
  (X = p95, Y = throughput, points = connection count)
- Supporting: [`fig_sweep_throughput.png`](../results/connection-sweep-20260820/figures/fig_sweep_throughput.png),
  [`fig_sweep_latency.png`](../results/connection-sweep-20260820/figures/fig_sweep_latency.png)

## 3. Interpretation

### 3.1 Connection parallelism has a knee

> ⚠️ **[Corrected — §4]** Everything below was measured at **fixed c32**. All
> three configurations have their operating point at c12, so c32 is the overload
> region, and much of the tail degradation seen here is **overload queueing
> rather than connection count**. The observation that a knee exists holds, but
> that knee is **both a connection knee and entangled with the concurrency
> knee** (§0).

Throughput **flattens at c4** (139.5). c8 at 139.1 is effectively the same and
c16 at 136.8 actually drops. The tail, meanwhile, **degrades monotonically**.

```text
p99   c1 452  ->  c2 515  ->  c4 698  ->  c8 827  ->  c16 895
max   c1 598  ->  c2 680  ->  c4 945  ->  c8 1223 ->  c16 1482
```

**This is as far as the data proves.**

> **Beyond c4, additional connection parallelism does not improve throughput on
> a c32 workload and degrades tail latency.**

⚠️ **"It bends because of connection management cost and queueing" is not an
established cause.** Several possible contributors are mixed together and none
was separated — H2-internal queueing, per-connection in-flight imbalance, TCP
processing, bursty NPU arrivals (the same list as S3.6 §4.3).

Another interesting point is that **the median and the tail move in opposite
directions.** Adding connections makes the average request faster (p50 268 →
157) while some requests get much slower (p99 452 → 895). It is not "more
connections are faster" but that **a throughput–tail trade-off is real.**

### 3.2 The real choice is c2 or c4

> ⚠️ **[Corrected — §4]** This section's trade-off was measured at **c32 (the
> overload region)**. At the c12 operating point, conn4 against conn2 is
> throughput +1.2% for p95 +1.2% — **essentially a draw** (§4.1). The "tail
> 28–39% for +3.8%" below holds only in the overload region.

| | c2 | c4 | what c4 pays |
|---|---:|---:|---|
| throughput | 134.4 | 139.5 | **+3.8%** |
| p95 | 438.2 | 561.6 | **+28.2%** |
| p99 | 514.9 | 698.2 | **+35.6%** |
| max | 679.7 | 944.5 | **+39.0%** |

**c4 gives up 28–39% of the tail for +3.8% throughput.** For real-time inference
that looks like a bad trade.

Against c1 it is clearer — c2 gets **79%** of what c4 gained (+16.3 / +20.7) for
about **a quarter** of the tail cost (p95 +11.7% vs +43.1%).

Recovery of the gap to local direct (161.5):

```text
c1 115.6  - gap 45.9 ->  local 161.5
c2 134.4  recovered 18.8 (41%)
c4 139.5  recovered 23.9 (52%)
```

### 3.3 The heuristic picked c4 — and it was a close thing

The analyser's rule ("lowest p95 among those within 97% of maximum throughput")
picks **c4**, because c2 at **96.4%** missed the threshold **by 0.6 pp**.

> **The threshold is not moved to fit the result.** Lowering 97% to 96% to
> include 96.4% would be post-hoc rationalisation, not a heuristic. The rule
> stays, and **the fact that the rule fails to decide at this boundary is
> recorded as the result.**
>
> This is why §0 pins down that the selected operating point is a deliberate
> engineering heuristic rather than a statistical optimum. It is also why the
> table is published alongside — at a boundary a human has to judge, and the
> basis for that judgement has to be in the table.

## 4. S3.7a conclusion and the next move

- **c8 and c16 are dropped from the S3.7b shortlist.**

  > This is **not** "c8/c16 are inferior at any concurrency". S3.7a is
  > fixed-load at c32 and did not measure their absolute ceilings. The reason is
  > **priority** — the tail cost at c32 is already this large (p99 827 and 895,
  > max 1223 and 1482), so the expected value against further search cost is
  > low. We can come back if needed.

- **c2 and c4 go forward as S3.7b candidates.** At fixed c32 neither wins. If c2
  is not yet saturated it could overtake at a higher concurrency, and c4's tail
  could collapse faster as concurrency rises.

  Characterised as they stand: **c2 = efficiency point, c4 = performance
  point.** Which is the operating point is decided by looking at the ceiling.

## 5. Limitations

- **These are not each setting's ceiling** (note in §1). It is a fixed-c32
  result.
- Percentiles are run-level averages, so they show the tail lower than pooled.
  Valid for comparing conditions, not to be quoted as absolutes (S2 §7.4.1).
- **The cause of the p95/p99 degradation remains unverified.** None of S3.6
  §4.3's five candidates (per-connection in-flight imbalance / H2-internal queue
  variance / bursty NPU arrivals / transport queueing / the general tail growth
  that accompanies higher throughput) was excluded.
- This is a 1-node result. At three nodes the server holds N×3 connections
  (S3.8).

## 6. Reproduction

```bash
bash scripts/run-connection-sweep.sh sweep 5     # 25 runs, about 40 minutes
PYTHONIOENCODING=utf-8 python scripts/analyze-connection-sweep.py \
    results/connection-sweep-20260820/raw/results.csv
python scripts/make-sweep-figures.py \
    results/connection-sweep-20260820/raw/results.csv \
    results/connection-sweep-20260820/figures
```

---

# S3.7b — Concurrency sweep

## 0. The thing to tune was two-dimensional, not one

This is the structure that emerged through S3.7a and b. **There are two knees.**

```text
Concurrency knee   how many requests in flight are needed to saturate the device?
Connection knee    how many connections should those requests be split across?
```

So what needs tuning is not "connection count" alone but a **two-dimensional
operating point of load concurrency × connection parallelism**.

This lands exactly on NPUDure's original question — "why won't it scale?"
**Past saturation, pushing more in does not make the NPU do more work; it only
piles queues up inside the system.** §2 below is that captured by measurement.

## 1. Definition of operating concurrency (an experimental rule)

Pinned to a number. Without it, results like 132.8 / 134.1 / 134.3 turn "where
is the knee" into a human judgement every time.

> **operating concurrency = the lowest concurrency delivering at least 98% of
> peak throughput**

**Why 98%**: the observed run-to-run SD is around ±1 inf/s, so a 99% threshold
would overlap measurement noise. This definition lives as a constant in
`analyze-concurrency-sweep.py`.

## 2. First range (c24–c64) — all of it was overload

Candidates **c2 · c4**, 3 runs each.

| conc | conn2 tp | conn2 p95 | conn2 p99 | conn4 tp | conn4 p95 | conn4 p99 |
|---:|---:|---:|---:|---:|---:|---:|
| **24** | **134.3 ± 1.1** | **306.9** | **357.5** | **139.3 ± 1.2** | **390.9** | **480.2** |
| 32 | 133.7 | 431.5 | 505.7 | 138.3 | 576.5 | 715.0 |
| 40 | 134.2 | 572.1 | 674.4 | 137.7 | 719.5 | 932.9 |
| 48 | 133.8 | 697.6 | 832.0 | 137.6 | 958.6 | 1200.9 |
| 64 | 132.9 | 946.0 | 1132.3 | 137.9 | 1254.4 | 1566.7 |

Error rate 0 throughout.

**Throughput is completely flat across c24–c64** (conn2 ≈ 134, conn4 ≈ 138),
while the tail grows nearly linearly — conn4 @c64 reaches p99 1567 ms, max
2128 ms.

> **This entire range is past saturation.** What the data says:
>
> **Throughput saturation occurs at concurrency ≤ 24. Additional concurrency
> past saturation does not increase throughput and only increases tail latency.**

Textbook queueing. Requests pushed in beyond that go to a queue, not to
computation.

### 2.1 ~~The trade-off is stable across load~~ — **wrong (refuted in §4)**

| | S3.7a @c32 | S3.7b @c24 |
|---|---:|---:|
| throughput | +3.8% | +3.7% |
| p95 | +28.2% | +27.4% |
| p99 | +35.6% | +34.3% |

The two agreed closely, so this was initially written up as "not a coincidence
of one concurrency but **a trade-off that 4 connections create as such**".
**That interpretation was wrong.**

c32 and c24 agreed because **both are in the overload region and we saw the same
phenomenon twice.** Descending to the true operating point (c12) in §4, the p95
penalty vanishes from **+28% to +1.2%**. It was not a property of four
connections but **a property of post-saturation queueing.**

> The lesson: **two measurements agreeing does not mean the interpretation is
> right.** If both are biased in the same direction, reproducibility only
> confirms the bias.

### 2.2 So the sweep direction was wrong

Both candidates peak at **the bottom of the sweep (c24)**. The saturation point
is therefore **below** c24, and the operating point (the lowest concurrency
yielding the ceiling) has not been seen.
→ **Re-sweep downwards over c8/c12/c16/c20/c24.**

## 3. Re-measure the conn1 baseline over the same range

Skip this and the interpretation gets mixed. Placing the two points we have side
by side:

```text
conn1 @c32 ->  115.6 inf/s,  p95 392
conn2 @c24 ->  134.3 inf/s,  p95 307
```

It is tempting to write "2 connections improved **both** throughput and
latency". But **two variables changed at once** — connections 1→2 and
concurrency 32→24. Causality cannot be separated.

The question only stands if each connection count's **operating point is found
by the same rule (§1)** and then compared.

The question narrows to one.

> **Under an identical saturation criterion, how does connection parallelism
> affect throughput and tail latency?**

## 4. Second range (c8–c24) — the result inverts

conn **1 · 2 · 4** on **the same grid** (c8/12/16/20/24), 3 runs each, 45 runs
total. Error rate 0.

| conc | conn1 tp | conn1 p95 | conn2 tp | conn2 p95 | conn4 tp | conn4 p95 |
|---:|---:|---:|---:|---:|---:|---:|
| 8 | 112.1 | 101.3 | 120.4 | 93.9 | 111.0 | 105.0 |
| **12** | **114.8** | **147.6** | **136.4** | **119.8** | **138.1** | **121.2** |
| 16 | 115.1 | 191.5 | 136.2 | 178.7 | 138.8 | 210.0 |
| 20 | 114.9 | 239.2 | 135.1 | 245.4 | 138.5 | 306.1 |
| 24 | 115.9 | 286.8 | 134.0 | 307.7 | 139.1 | 392.3 |

### 4.1 The operating point is c12 for all three

Applying the 98% rule (§1):

| connections | operating conc | throughput | p50 | p95 | p99 | vs peak |
|---:|---:|---:|---:|---:|---:|---:|
| **1** | **12** | 114.8 | 102.1 | 147.6 | 167.2 | 99.1% |
| **2** | **12** | **136.4** | 85.8 | **119.8** | **137.4** | 100.0% |
| **4** | **12** | 138.1 | 83.4 | 121.2 | 145.7 | 99.3% |

**For all three connection counts tested (1, 2, 4), the 98%-criterion operating
concurrency was observed at c12.**

> **Within the tested range, the concurrency knee remained invariant to
> connection parallelism.**

That is evidence the concurrency knee did not move when connection parallelism
changed. It **strongly suggests, but does not prove**, that the two knees are
independent — with a different model, payload size, node count or network it
could move. §0's two-dimensional structure should be read as observed within
this range.

### 4.2 At the operating point there is no trade-off — conn2 dominates conn1

| conn2 vs conn1 @c12 | |
|---|---:|
| throughput | **+18.8%** |
| p50 | **−16.0%** |
| p95 | **−18.8%** |
| p99 | **−17.8%** |

**Throughput rises while latency falls at every percentile.** Not a trade-off but
a **strict Pareto improvement** — on the measured throughput and latency
metrics, that is. It does not mean a whole-system Pareto including CPU, memory
and connection resources.
→ [`fig_sweep_pareto.png`](../results/s37b-operating-point/figures/fig_sweep_pareto.png)

**conn4 is not absolutely worse.** If maximum throughput is the priority, conn4
is a legitimate choice too (138.1 vs 136.4).

The basis for making conn2 the default operating point is not "conn4 is bad".

| What conn4 gives extra | What conn4 spends extra |
|---|---|
| throughput **+1.2%** — close to measurement variation (SD ±0.3–1.6) | **twice** the connection resources |
| | p99 **+6.0%** |

> **2 connections is the lowest-complexity configuration that captures
> nearly all available throughput.**

conn2 because it takes nearly all of the ceiling with the fewest resources.

### 4.3 So the earlier "tail degradation" was not the connections' fault

S3.6 §4.3 and S3.7a recorded that adding connections worsens the tail (p95 +46%,
+43%). **Those measurements are right and the interpretation was wrong.**

Those experiments all **measured at c32, and c32 is the overload region for all
three configurations** (the operating point is c12). That comparison was
therefore not "which configuration is better" but **"which configuration
degrades more gracefully under overload"**.

```text
seen at c32   1ch -> 4ch :  throughput +21%, p95 +46%   <- overload comparison
seen at c12   1ch -> 2ch :  throughput +19%, p95 -19%   <- operating-point comparison
```

**S3.6's and S3.7a's numbers are not wrong. The question was different.**

| What was asked | Answer |
|---|---|
| **Fixed-c32 comparison** — how does each configuration behave **under overload**? | more connections raise the ceiling slightly but **amplify the tail more** |
| **Operating-point comparison** — which configuration is better **in operation**? | conn2 dominates conn1 on both axes |

So the c32 results are not to be discarded but remain **a separate valid
result** — a result about overload behaviour. They just must not be used as
grounds for operating decisions.

> **Optimize at the operating point, not in the overload region.**

Comparing configurations at a fixed load without defining the operating point
means **seeing overload behaviour rather than a configuration effect, and the
conclusion can invert.** This is the most practical lesson S3.7 leaves.

## 5. S3.7b conclusion

> **Selected operating point: 2 connections @ concurrency 12
> — 136.4 inf/s, p95 119.8 ms, p99 137.4 ms**

Against the conn1 baseline under the same rule (114.8 @c12): **throughput
+18.8%, p95 −18.8%**. Of the 46.7 gap to local direct (161.5), **21.6 (46%) was
recovered by configuration alone**, with the tail improving alongside.

## 6. Limitations

- **Grid resolution.** The knee lies between c8 (88% of peak) and c12, and with
  a step of 4 **we do not know whether c12 is the true knee or c10**. This is
  fine for comparing three configurations on the same grid, but the caveat
  attaches when quoting the operating point as an absolute.
- The analyser flags "saturation unconfirmed (peak at the top of the sweep)" for
  conn1 and conn4. But conn1 is c12 114.8 ± 0.7 vs c24 115.9 ± 0.8, and conn4 is
  c12 138.1 ± 1.6 vs c24 139.1 ± 0.8 — **noise within a flat region**. The
  warning is left in place, conservatively.
- This is a 1-node result. At three nodes the server holds 2×3 = 6 connections
  (S3.8).
- Percentiles are run-level averages (S2 §7.4.1).

# S3.7c — RPS at the selected operating point

Settled operating point: **2 connections @ c12**.

This now becomes an experiment that asks one question with nothing else mixed in.

> **Does RPS improve the selected operating point?**

If a null comes back as in S3.5b, **that is a good result too**. It would
substantially weaken the hypothesis that "RPS was ineffective because there was
only one flow" — if nothing changes with two flows, evidence accumulates that
**IRQ/RX-side distribution is not this workload's bottleneck.**

`rps_cpus` OFF/ON at the settled operating point. In S3.5b there was one flow
and nothing to distribute. Now there are several flows, and under S3.6's 4ch
condition CPU0 was at busy 81% / soft 74%.

**If c2 and c4 tie ambiguously in S3.7b, run the RPS A/B on both.** Ten runs per
condition suffices, so it is cheap, and **the RPS effect may differ between two
flows and four** — which is itself information aimed at ②-a (TCP per-flow
processing).

- If it rises → releasing the single-connection constraint exposed a NIC
  processing bottleneck
- If unchanged → CPU0 softirq is **merely a correlation, not a throughput
  limiter** (a much stronger exclusion than S3.5b alone)

## Result — a null. And this null is much stronger

conn2 @ c12 fixed, `rps_cpus` = `00` (CPU0 only) vs `fe` (cores 1–7), 5 runs
each.

| | throughput | p50 | p95 | p99 | board idle | **CPU0 busy** | **CPU0 %soft** |
|---|---:|---:|---:|---:|---:|---:|---:|
| RPS off | **136.8 ± 0.6** | 85.4 | 119.1 | 137.7 | 49.3% | **78.7%** | **68.0%** |
| RPS on | 135.6 ± 0.4 | 86.4 | 119.7 | 139.3 | 49.1% | **74.6%** | **56.0%** |
| difference | **−0.8%** | +1.2% | +0.5% | +1.2% | — | −4.1 pp | **−12.0 pp** |

Error rate 0. The −0.8% throughput difference is within the SD (±0.4–0.6).

### Why this null is stronger than S3.5b's

In S3.5b there was a counter-argument — **with one flow, RPS had nothing to
distribute.** That counter-argument is blocked here.

1. **There are two flows.** There really is something for RPS to hash apart.
2. **RPS actually worked.** CPU0 %soft came down 12 pp, **68.0% → 56.0%**, and
   CPU0 busy fell from 78.7% to 74.6%. The setting was not ignored.
3. **CPU0 was not idling.** At 78.7% busy it was under real load (comparable to
   81% under S3.6's c32/4ch condition). "There was too little load for RPS to
   act on" does not hold either.

> **At the selected operating point, RPS reduced CPU0 softirq load
> substantially but produced no measurable throughput or tail-latency
> improvement. Therefore, CPU0 receive-side processing was not
> performance-limiting under the tested configuration.**

**Read the scope precisely.** What this says is not "CPU0 softirq is not a
limiter" but **"it is not a limiter at this operating point, in this
configuration"**. Under a different load, model, payload size or node count it
could differ.

Within that scope it is quite strong — the mechanism demonstrably worked and did
not touch the end-to-end limiter. S3.5's (§4.3) nomination of CPU0 as "the next
bottleneck candidate" is excluded **for this configuration.**

## Overall S3.7 conclusion

| Candidate | Verdict |
|---|---|
| Link bandwidth | excluded (51% per direction) — S3.5 |
| Board CPU capacity | excluded (49–63% idle) — S3.5, S3.7c |
| Server and scheduler | **reopened** — excluded at baseline, but optimized 3N eff 95.3% (S3.8) |
| **CPU0 softirq / RPS** | **excluded.** Throughput unchanged after taking 12 pp off — S3.7c |
| H2 flow control window | enlarging to 64 MB is harmful at −36.3% — S3.6 |
| **Connections per node** | **primary constraint.** 1→2 gives +18.8% and improves the tail — S3.7b |
| protobuf, copies, syscalls | **unseparated.** May lie within the remaining 15.5% |

**Selected operating point: 2 connections **per node** @ concurrency 12 —
136.4 inf/s, p95 119.8 ms, p99 137.4 ms**

> **Always state the unit.** `[transport] node_connections` is a **per-node**
> value (`GrpcNodePool` creates N channels per `NodeId`). It is not a
> cluster-wide total.
>
> | Nodes | node_connections | cluster-wide connections |
> |---:|---:|---:|
> | 1 | 2 | 2 |
> | 2 | 2 | 4 |
> | 3 | 2 | 6 |
>
> Fixing "2 connections" cluster-wide in S3.8 would not preserve the per-node
> condition, and at 3N **connection supply itself would become a new
> bottleneck**. That is an entirely different experiment and must not be
> confused with this one.

**15.5%** still remains to local direct at 161.5.

> ⚠️ **The exclusion table shrank the candidate space; it did not identify what
> the remaining 15.5% is.** Several candidates remain.
>
> | Candidates for the remaining gap |
> |---|
> | protobuf serialization |
> | memcpy / buffer ownership (`to_vec()` and the like) |
> | syscall / submission path |
> | HTTP/2 implementation overhead |
> | userspace scheduling (tokio workers ↔ blocking pool contention) |
> | NPU submission / RKNN runtime overhead |
> | other |

**io_uring is now a legitimate candidate. But "the next bottleneck is syscalls
and copies" is not yet established.** So S4's question is framed this way:

```text
no    Does io_uring recover the remaining 15.5%?
yes   Is the syscall / submission path actually a meaningful cost?
```

Confirm syscall and copy cost by profiling first, and go to io_uring only if the
answer is yes. That is TECHSPEC §15.1's order and the same principle held since
S3.5 — **measurement decides implementation.**

Next is **S3.8** — re-verify 1N/2N/3N scale-out at this operating point.

---

<a id="experiments-s3-8-optimized-scaleout"></a>

# S3.8 — Optimized gRPC Scale-out

*[한국어 원문](experiments/S3_8_OPTIMIZED_SCALEOUT.ko.md)*

- Experiment ID: **S3.8**
- Measured: 2026-08-20
- Code: `0af696d` + `[transport] node_connections = 2`
- Status: **complete (36 runs, node-count verification passed on 9/9 configurations)**
- Raw data: [`../../results/scaleout-optimized-20260820/`](../results/scaleout-optimized-20260820)
- Predecessor: [`S3_7_CONNECTION_TUNING.md`](#experiments-s3-7-connection-tuning)

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
> → [S3.9a](#experiments-s3-9a-scaleout-profile)

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

![Absolute values rise at every scale while efficiency goes 98.9% -> 95.3%](../results/scaleout-optimized-20260820/figures/fig_scaleout_optimized.png)

**`fig_scaleout_optimized.png`** — absolute values rise at every scale while
efficiency goes 98.9% → 95.3%

Regenerate: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s3-9a-scaleout-profile"></a>

# S3.9a — Scale-out Efficiency Loss Profiling

*[한국어 원문](experiments/S3_9A_SCALEOUT_PROFILE.ko.md)*

- Experiment ID: **S3.9a**
- Measured: 2026-08-21
- Code: `e1ad9ed` + `[transport] node_connections = 2`
- Status: **complete (9 runs, 3 configurations × 3, node-count verification passed 9/9)**
- Raw data: [`../../results/scaleout-profile-20260821/`](../results/scaleout-profile-20260821)
- Predecessor: [`S3_8_OPTIMIZED_SCALEOUT.md`](#experiments-s3-8-optimized-scaleout)

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
- Scripts: [`run-scaleout-profile.sh`](../scripts/run-scaleout-profile.sh),
  [`server-profile-collect.sh`](../scripts/server-profile-collect.sh),
  [`analyze-scaleout-profile.py`](../scripts/analyze-scaleout-profile.py).

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
> boards' 2.5GbE ([S3.5 §4.1](#experiments-s3-5-transport-profile)), and then the same
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

![p50 flat (+0%), p95 +23%, p99 +36% - the loss is entirely in the tail](../results/scaleout-profile-20260821/figures/fig_efficiency_loss_is_tail.png)

**`fig_efficiency_loss_is_tail.png`** — p50 flat (+0%), p95 +23%, p99 +36%; the
loss is entirely in the tail

Regenerate: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s3-9b-node-residual"></a>

# S3.9b — Node-side Residual Cost Profiling

*[한국어 원문](experiments/S3_9B_NODE_RESIDUAL.ko.md)*

- Experiment ID: **S3.9b**
- Measured: 2026-08-21
- Code: `62855bd`
- Status: **complete** (4 conditions × 45 s of collection, 0 errors)
- Raw data: [`../../results/node-residual-20260821/`](../results/node-residual-20260821)
- Predecessors: [`S3_5_TRANSPORT_PROFILE.md`](#experiments-s3-5-transport-profile) ·
  [`S3_9A_SCALEOUT_PROFILE.md`](#experiments-s3-9a-scaleout-profile)

---

## 1. Research Question (narrow)

> **In the residual gap between 161.5 and 135.5, do node-side serialization,
> copy and syscall costs account for a meaningful share?**

**Explaining the whole gap is not the objective.** S3.9a separately surfaced the
scale-out tail/TCP cost, so there is no reason a node-side profile should have
to account for all 26.0 inf/s. Whatever is not explained stays unexplained.

The decision rule was fixed **before measuring**.

| Result | Decision |
|---|---|
| syscall and copy are **large enough** | proceed to S4 io_uring |
| **small** | **cancel/shelve S4** |
| **some other term is large** | record that term only. If it is outside the core scope, dig no further |

## 2. Method

The decisive difference from S3.5 is that **this measures at the operating
point**.

```text
S3.5    c32 . conn1   116.6 inf/s   overload . baseline
S3.9b   c12 . conn2   136.6 inf/s   operating point . optimized
```

Overload-region values are not used for operating decisions (README §4.1). This
repository has already fallen into that trap once — the 13.2% misquotation
incident.

- One node (king only). queen and jack are brought down so round-robin cannot
  split the load, and a probe leaves evidence that **the only responding node ID
  is king**.
- Of the 80 s of load, only **45 s starting at t+20** is collected, excluding
  the ramp and warmup.
- Four conditions: `idle` (instrument floor) / `op` (operating point) /
  `strace` / `local` (direct, 8 threads).

### 2.1 Choice of instrument — there is no perf

The boards have no `perf`, `bpftrace` or `gdb` (kernel 6.1.141, vendor tree).
Symbol-level profiling is impossible. Instead, the **utime/stime split from
`/proc/PID/stat`** is used.

```text
utime  user time   - protobuf serialization, user-space copies, HTTP/2 framing
stime  kernel time - syscall entry, TCP stack, copy_to_user, skb, driver
```

**What io_uring reduces is a portion of stime.** So the whole of stime is
io_uring's absolute upper bound, and what is actually recoverable is smaller
than that.

As a secondary instrument, `strace -c` for 10 s. Because ptrace stops the
process at every syscall, the reported residency is **inflated**, so it is used
**only as an upper bound** — if the inflated figure is small, the real one is
conclusively smaller. It is a test that is valid in one direction only.

## 3. Results

### 3.1 Node CPU per request

| Condition | throughput | utime/req | stime/req | **CPU-ms/req** | user% | kernel% |
|---|---:|---:|---:|---:|---:|---:|
| op (operating point) | 136.6 | 14.50 | 11.09 | **25.59** | 56.7 | 43.3 |
| local direct | 157.9 | 5.14 | 4.10 | **9.23** | 55.6 | 44.4 |
| **transport cost** | | **9.37** | **6.99** | **16.35** | **57.3** | **42.7** |

The operating point of 136.6 agrees with S3.8's 135.5 ± 0.4 and S3.7b's
136.4 ± 0.3 — confirmation that the condition was set up correctly.

> ⚠️ The local figure of 157.9 is the average over the **full 80 s** and so
> includes the ramp. The steady-state rate within the collection window
> (t+20–65) was 162.6. That difference overestimates local's per-request CPU by
> about 3%, which **overestimates rather than underestimates the transport
> cost** — so it does not weaken the conclusion below (that the cost is small).

### 3.2 No core is saturated

```text
op    cpu0  soft=68.3  idle=21.2   <- the only hot core (78.8% busy)
      cpu1-3            idle 61-64
      cpu4-7            idle 42-47
      overall           idle 48.9
local overall           idle 82.5   softirq 0
```

Even the hottest core, cpu0, has 21% left. cpu0's load is mostly **softirq**
(the NIC's single receive queue) — and **S3.5 §4.3 already spread that with RPS
and got a −0.2% null.** cpu0 softirq is not a constraint either.

### 3.3 Syscalls — many calls, small cost

`strace -c` over 10 s (about 1,284 requests):

| syscall | residency | calls | calls/req | |
|---|---:|---:|---:|---|
| futex | 30.07s | 48,565 | 37.8 | thread synchronization **wait** |
| ioctl | 24.72s | 68,924 | 53.7 | RKNN driver (NPU submission) |
| epoll_pwait | 9.78s | 37,157 | 28.9 | event **wait** |
| **recvfrom** | 9.50s | 136,602 | **106.4** | request receive ← io_uring target |
| **writev** | 5.91s | 69,245 | **53.9** | response send ← io_uring target |
| **write** | 0.35s | 5,524 | **4.3** | response send ← io_uring target |

**Network syscall residency is 15.77 s / 80.36 s = 19.6%.** The other 80.4% is
futex (synchronization wait), ioctl (NPU driver) and epoll (event wait) — **none
of which io_uring touches.**

## 4. Verdict — **S4 io_uring cancelled/shelved**

Network syscalls per request come to about **165** (recvfrom 106 + writev 54 +
write 4). Even taking aarch64 syscall entry cost **generously at 1 µs**:

```text
165 calls x 1 us = 0.165 ms/req
0.165 / 16.35    = 1.0% of per-request transport CPU
```

Even **assuming** registered buffers eliminate the 1.2 MB copy in both
directions (roughly 0.6–1.2 ms at RK3576 memory bandwidth), the total is
**1.4 ms/req ≈ 8% of transport cost**.

And recovering all of that 8% **would not raise throughput.** The board CPU is
48.9% idle, no core is saturated, and spreading the hottest core's (cpu0)
softirq with RPS produced a −0.2% null.

> **CPU-ms/req is a cost, not a constraint.** Reducing consumption of an
> unsaturated resource does not raise throughput.

```text
Question   Does io_uring recover the remaining 16.1%?
Answer     No. What it targets (syscall entry) is 1% of transport cost, and 8%
           under the most generous assumptions. And CPU is not the constraint.
```

**S4 is cancelled/shelved.** The io_uring item in TECHSPEC §15 changes status
from "necessity unproven" to **"refuted by measurement"**.

## 5. The third branch of the decision rule — record the large term separately

The question bundled three things — serialization / copy / syscall — and the
answer split them.

| Term | Size | Verdict |
|---|---|---|
| **syscall** | ~1% of transport cost | **small** |
| **serialization / user-space copy** | **9.37 ms/req = 57%** | **large** |

**User time exceeds kernel time** (9.37 vs 6.99). The majority of transport cost
is protobuf serialization, user-space copies and HTTP/2 framing. io_uring does
not touch that side.

But **we stop here**, per the third branch of the pre-registered rule: record
the large term, but as long as CPU is not the constraint there is no guarantee
that reducing this raises throughput either. There is not yet grounds to dig in.

## 6. So what is the 26.0 inf/s gap — out of scope, observation only

Not this experiment's job, but the direction is observable. At fixed
concurrency, throughput = concurrency / latency.

```text
op     c12,  136.6 inf/s  ->  mean latency 87.8 ms
local  8 threads, 157.9   ->  mean latency 50.5 ms   (wrapper measured 50,531 us)
                              difference +37.3 ms
```

Of that, node CPU work is only 16.35 ms; the rest is **waiting**. With a 1.2 MB
request and a 1.2 MB response, on the measured link (2.34 Gbps ≈ 292 MB/s)
**pure transfer time alone is about 4.1 ms per direction, 8.2 ms round trip.**
The scheduler hop and queueing add to that.

> The gap looks like a **path-latency** problem rather than a CPU-cost one. The
> lever for reducing it is not io_uring but **payload size** (ADR-008's raw
> 640×640×3 transfer). That is outside S3.9b's scope, so it is **left as an
> observation only.**

## 7. Limitations

- One run per condition (45 s of collection). The utime/stime deltas are stable
  because they accumulate over 45 s, but there is no run-to-run SD.
- The seconds reported by `strace -c` are **residency including blocking**, not
  CPU time. That is why futex and epoll top the list, and the 19.6% network
  syscall share is only valid within that same scale. The primary basis for the
  verdict is the utime/stime split; strace is secondary.
- The 1 µs syscall entry cost is not measured but a **generous** take on the
  usual aarch64 figure. Measuring it would need a microbenchmark, but since the
  1 µs assumption already yields 1%, the conclusion does not flip.
- local direct uses `sustained_load_test` (a separate binary), so its code path
  is not identical to the node's. It is the reference baseline used consistently
  since S3.5.

## 8. The instrument error caught in this experiment

The regular expression parsing the `strace -c` summary read the **`usecs/call`
and `calls` columns swapped**. The call count came out 100× too small, and we
nearly concluded "strace attached to only one thread → the upper-bound test is
invalid". It was caught by comparing against the expected value (83.4 writes per
request, from `/proc/PID/io`).

> When an instrument's output differs from expectation, **suspect the instrument
> first** (README §4.10). This time it was not the measurement but the parser
> that was wrong.

---

## Figure

![The user/kernel split of transport cost and the share io_uring can reach (about 8%)](../results/node-residual-20260821/figures/fig_transport_cost_split.png)

**`fig_transport_cost_split.png`** — the user/kernel split of transport cost and
the share io_uring can reach (≈8%)

Regenerate: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s3-saturation"></a>

# S3 — Per-configuration Saturation

*[한국어 원문](experiments/S3_SATURATION.ko.md)*

- Experiment ID: **S3**
- Measured: 2026-08-20
- Frozen commit: `1da69d4` (bench code `254d560`, same as S2). No changes during measurement
- Status: **complete (45 runs)**
- Raw data: [`../../results/saturation-20260820/raw/`](../results/saturation-20260820/raw) · figure: [`figures/fig3`](../results/saturation-20260820/figures/fig3_saturation_sweep.png)
- Predecessor: [`S2_GRPC_BASELINE.md`](#experiments-s2-grpc-baseline)

---

## 1. Research Question

> **What is the maximum sustainable throughput (ceiling) of each cluster
> configuration, and at what concurrency is it reached?**

**This is a different question from S2.** S2 looked at linearity under
*identical per-node load* (c = 8×N). S3 explores each configuration's **true
ceiling** (1/2/3 node) by raising concurrency. The two experiments are not
mixed.

## 2. Method

- Concurrency sweep, past per-node load and up to saturation:
  ```text
  1 node : c4, c8, c16, c32, c48
  2 node : c8, c16, c24, c32, c48
  3 node : c12, c24, c32, c48, c64
  ```
- **3 runs of 30 s** per point. Condition order rotates. 45 runs total.
- Fixed conditions match S2 (INT8, want_float=0, performance, active cooling,
  round-robin, 8 workers, gRPC). The freeze holds.
- Script: [`scripts/run-saturation-sweep.sh`](../scripts/run-saturation-sweep.sh).

## 3. Results — Saturation Curves

Mean of 3 runs (inf/s); every SD is ≤ 2.2:

| concurrency | 1 node | 2 node | 3 node |
|---:|---:|---:|---:|
| c4 | 84.0 | | |
| c8 | 112.6 | 168.3 | |
| c12 | | | 252.2 |
| c16 | 113.8 | 228.1 | |
| c24 | | **232.0** | 339.4 |
| c32 | **115.2** | 230.2 | **341.8** |
| c48 | 114.1 | 230.3 | 339.2 |
| c64 | | | 335.9 |

**Ceilings:**

| Config | Ceiling | @ concurrency | per-node concurrency |
|---|---:|---:|---:|
| 1 node | **115.2** inf/s | c32 | 32 |
| 2 node | **232.0** inf/s | c24 | 12 |
| 3 node | **341.8** inf/s | c32 | ~11 |

→ [Figure 3](../results/saturation-20260820/figures/fig3_saturation_sweep.png)

## 4. Interpretation

**Finding — near-linear even at the ceiling.**

| Config | Ceiling | Speedup (vs 1-node ceiling) | Efficiency |
|---|---:|---:|---:|
| 1 node | 115.2 | 1.00× | 100% |
| 2 node | 232.0 | 2.01× | 101% |
| 3 node | 341.8 | **2.97×** | **99%** |

S2 showed linearity under identical load; S3 shows it at maximum throughput.
**Near-linear scaling is confirmed independently from two angles.** The 3-node
ceiling of 341.8 inf/s is 2.97× the 1-node ceiling.

Three regions of the curve:

- **Low concurrency (unsaturated):** throughput is held down by round-trip
  latency (≈68 ms, S2 §7.4). Being closed-loop, too few in-flight requests
  leave the pipeline empty (1N c4 = 84, 3N c12 = 252).
- **Plateau (saturated):** maximum at roughly 10–16 concurrent per node. Once
  the pipeline keeps the 8 workers fed, raising it further adds nothing.
- **Overload (slight decline):** beyond that, only queueing grows and throughput
  dips slightly (3N c32 341.8 → c64 335.9). Errors remain 0 — the scheduler and
  node queues absorb it.

## 5. Limitations

- Same as S2: short measurement window (30 s, before throttling), active cooling
  only, closed-loop, one 2-node combination (king+queen).
- **The duration differs from S2 (30 vs 60 s).** The ceiling values
  (115/232/342) are close to S2's c8/c16/c24 (112.9/229.0/338.4) but the
  conditions are not identical. Saturation is about the shape of the curve and
  where the ceiling sits; for absolute values, S2 takes precedence.
- The decline past the ceiling is a closed-loop queueing effect — it may look
  different under an open model
  ([`adrs/028`](../adrs/028-bench-run-validity.md)).

## 6. Reproduction

```bash
bash scripts/run-saturation-sweep.sh    # 45 runs -> server:/tmp/sat30
python scripts/make-figures.py          # regenerate Figure 3
```
Frozen commit `1da69d4`.

## 7. Raw Data & Conclusion

- 45 raw files: [`../../results/saturation-20260820/raw/`](../results/saturation-20260820/raw)
  (`sat_n{nodes}_c{concurrency}_r{round}.json`)

**Conclusion.** The throughput ceiling of each configuration is
**115 / 232 / 342 inf/s** at 1/2/3 nodes, and 3 nodes reach **2.97× (99%)** of
the 1-node ceiling — **near-linear by the ceiling measure too**. Saturation
occurs at roughly 10–16 concurrent per node. This re-confirms S2's linear-scaling
conclusion from the maximum-throughput perspective.

→ Next: **S4 (io_uring)** — compare, under conditions identical to this
baseline, how much it reduces the cost of the payload-transfer path (S2 §8: 94%
of non-inference latency).

---

<a id="results"></a>

# NPUDure measurement results — first pass

*[한국어 원문](RESULTS.ko.md)*

- Compiled: **2026-08-14** (the single-node lineage)
- Period covered: 2026-08-07 to 2026-08-12
- Raw data: `results/`, `benchmarks/`
- The discussion: `discuss.md` (chronological); work history:
  `board-worklog.md`

> ## ⚠️ This document is **the first pass over the single-node lineage** (updated 2026-08-21)
>
> On 2026-08-20 and 21 **the cluster measurement lineage ran through to
> completion** (S2 · S3 · S3.5–3.9b · S0-A–D, 421 runs, error rate 0). Those
> results are not here but in
> **[`experiments/README.md`](#experiments-readme)**. Start there for
> multi-node figures.
>
> §2.5 below is **a pilot measurement from before that lineage** and has been
> **superseded by later measurements.** The single-node results in §1–§4 and the
> failure list in §5–§6 remain valid.

> This document collects **results only**. Why those conclusions were reached is
> in `discuss.md`; what happened is in `board-worklog.md`.
>
> **Every number carries its measurement conditions.** We learned this time that
> a number without conditions is useless three months later.

---

# 1. One page

A distributed inference cluster was built from three RK3576 boards (6 TOPS NPU).
This document covers **single-node characteristics and the software.** The
multi-node lineage ran separately on 2026-08-20 and 21 and is complete —
**three nodes at 387.2 inf/s (operating point, 2.86× / 95.3%)**,
[`experiments/README.md`](#experiments-readme).

The three most important numbers so far.

| Item | Value | Meaning |
|---|---|---|
| Per-node throughput (120 s) | **84.3 inf/s** (FP16) / **157.2 inf/s** (INT8) | on `want_float=0` |
| Per-node **steady state** (300 s) | **59.7 inf/s** (FP16) | **−27%** against the start. CPU throttling |
| Two application optimizations | **+0.1%, −1.8%** (`core_mask`, zero-copy) | there is almost nothing left to squeeze inside the node |
| INT8 quantization | **1.86×** | the measure that landed hardest |
| `want_float=0` | **+17.3%** (INT8) / **+15.7%** (FP16) | output a quarter too. Network and throughput move together |

And the most important **non-numeric** result.

> **Four measurements were wrong, and all four "looked like success."**
> Half this project's output is that list of failures. See §6.

---

# 2. Settled figures

## 2.1 Hardware

| Item | Value | Source |
|---|---|---|
| SoC | Rockchip RK3576 | measured |
| CPU | 4× Cortex-A72 @2208 MHz + 4× A53 @2016 MHz | `cpufreq` |
| NPU | 2 cores, 950 MHz, 6 TOPS (nominal) | `/sys/kernel/debug/rknpu` |
| RAM | 4GB LPDDR4X | measured |
| Power | 5V DC. **A 4A adapter is required** | §6.2 |
| Network | 2.5GbE × 2 (r8125) | measured |
| RKNN Runtime | 2.3.0 (`c949ad889d@2024-11-07`) | `strings librknnrt.so` |
| RKNPU driver | v0.9.8 | `/sys/kernel/debug/rknpu/version` |

The three nodes' kernel (6.1.141), `librknnrt.so` hash, driver version and model
hash all match. `preflight-check.sh` confirms this before every measurement.

## 2.2 Inference performance

**Conditions: `king`, 8 threads, 120 s sustained, CPU governor `performance`,
fanless, a dedicated RKNN context per thread.**

| Model | Throughput | Mean latency | Model size |
|---|---:|---:|---:|
| YOLOv8n FP16 | **84.3 inf/s** | 94.5 ms | 9.65 MB |
| YOLOv8n INT8 | **157.2 inf/s** | 50.8 ms | 6.46 MB |
| Ratio | **1.86×** | −46% | −33% |

Related figures.

| Item | Value | Conditions |
|---|---|---|
| Optimal `worker_count` | **8** | +27% over 4. Has not bent yet at 8 |
| Contribution of the NPU's second core | **1.51×** (not 2×) | single core 48.2 → two cores 73.0 inf/s |
| Kernel ioctls per inference | **76** | identical for FP16 and INT8 |
| CPU governor effect | +7% | `ondemand` → `performance`. **A 120-second measurement.** Unverified under sustained load |
| `want_float=0` effect | **INT8 +17.3% / FP16 +15.7%** | output becomes a quarter too |

> ⚠️ **Throughput figures in documents from before 2026-08-11 are on
> `ondemand`** (FP16 79.0 / INT8 146.2). Do not compare directly.

## 2.3 Thermal characteristics (pilot, not S0)

**Conditions: 3 boards concurrently, 8 threads, 900 s, fanless, no desk fan,
governor `ondemand`.** The plateau is from 300 s after load to the end.

| Board | NPU mean | NPU peak | Throughput |
|---|---:|---:|---:|
| king | 73.0 °C | 75.8 °C | 80.5 inf/s |
| queen | 67.5 °C | 70.2 °C | 77.7 inf/s |
| jack | 72.6 °C | 74.8 °C | 77.8 inf/s |

- **Node-to-node spread 5.6 °C**
- **No NPU throttling** — all 928 samples at 950 MHz
- ⚠️ **But the CPU is downgraded.** That verdict looked only at the NPU clock.
  The CPU clocks in the same log show A72 2208 → 816 MHz and A53 2016 → 600 MHz.
  `discuss.md` §12
- Never exceeded 90 °C. Did not reach the current thresholds
  (`degraded 80` / `disable 90`)
- Idle temperature 35–40 °C, minimum input voltage 5.046 V

**Sustained 8-thread load is possible fanless.** It completes without errors.
But **throughput is not sustained** — over 300 seconds it goes 81.6 → 59.7 inf/s
(−27%). Because it is the CPU, not the NPU, being downgraded by heat.
`discuss.md` §12

The formal S0 (30 min × 2 conditions) is still to come. This is a 15-minute
measurement for checking node-to-node spread.

## 2.4 Accuracy

**Conditions: real board (`king`), one COCO val2017 image, preprocessing done in
one place so both see the same input bytes.**

| Comparison | box cosine | Detection cells | Class agreement |
|---|---|---|---|
| FP16 vs ONNX | 0.99999 | 10/10 | 100% |
| INT8 vs FP16 | 0.997 | 10/10 | 100% |

For INT8 the top detection's cell moves by one and its score is −5.5%. **The
detection set and classes are identical.** Getting 1.86× for that price is worth
it.

The RKNN simulator cannot infer with a built `.rknn` (after `load_rknn`,
`init_runtime` refuses). Verification has to use the same file that gets
deployed, so it was measured on a real board.

## 2.5 Multi-node scalability (2026-08-20, pilot) — **superseded**

> **This section has been superseded by later measurements.** It is a
> single-run pilot; the formal results are below. When quoting values, use the
> individual experiment documents rather than this section.
>
> | This section (pilot) | Formal | Document |
> |---|---|---|
> | 3 nodes 337.7 inf/s | **338.4** (30 runs) / ceiling **341.8** | [S2](#experiments-s2-grpc-baseline) · [S3](#experiments-s3-saturation) |
> | scaling efficiency ~98% | **98.9%** (baseline) / **95.3%** (operating point) | [S2](#experiments-s2-grpc-baseline) · [S3.8](#experiments-s3-8-optimized-scaleout) |
> | node ceiling 115 | operating point **135.5** (2 connections/node) | [S3.7](#experiments-s3-7-connection-tuning) |
> | −27% against local 157 | **−16.1%** against local direct **161.5** | [S3.9b](#experiments-s3-9b-node-residual) |
>
> The cooling-condition mismatch flagged by the ⚠️ below is also resolved —
> since S3.5 the local direct reference has been unified as **161.5 with active
> cooling and 8 workers.**

**Conditions: gRPC via the scheduler (server .9), INT8, want_float=0,
governor=performance, active cooling (a dedicated fan per node), round-robin,
30 s (20 s for the 1-node sweep), a single run, preflight passed.** Not the
formal S2 — no repeated runs, no fanless comparison, no `--with-inference`. Raw
data `results/scaling-20260820/`.

Equal per-node load (concurrency = 8 × node count):

| Configuration | Throughput | Distribution | Error rate |
|---|---:|---|---:|
| 1 node | 111.6 inf/s | king 100% | 0% |
| 2 nodes | 228.7 inf/s | 50 / 50 | 0% |
| 3 nodes | **337.7 inf/s** | 33 / 33 / 33 | 0% |

1-node concurrency sweep (the saturation point): c8 111.6 → c16 114.0 → c32
**115.1 (saturated)**.

**Scaling efficiency ~98% (nearly linear).** Against the 1-node saturation of
115, three nodes at 337.7 is **2.93×**. Data parallelism (`adrs/001`) holds and
the scheduler is not a bottleneck even with three nodes.

**But the cluster node ceiling of 115 < the local sustained 157 (−27%).** The
round-trip p50 is 69 ms while the node reports inference at 24–28 ms — 40 ms+
appears to be overhead from going through the scheduler's gRPC (serialization +
transferring 1.17 MiB in and out + queueing and routing).

> ⚠️ **The 27% cannot be attributed purely to gRPC overhead.** The reference of
> 157 is **fanless** sustained (08-11/12) while the cluster's 115 is **active
> cooling** (today), so the cooling conditions differ. It gets settled after
> re-measuring the local baseline under the same fan conditions.

> **The first answer to the central question, "do three 6 TOPS units make
> 18 TOPS": 2.93× (98%) on a cluster basis.** The bottleneck is not scaling but
> per-node overhead. The source of that 27% gets broken down next with the
> `TimingBreakdown` stages. `board-worklog.md` §2.25.

Raw data and detailed report: `results/scaling-20260820/`.

> ✅ **Reproduced across 30 runs (2026-08-20).** 1/2/3 nodes = 112.9±0.5 /
> 229.0±0.9 / **338.4±1.1 inf/s**, speedup 3.00×, error 0%, balance 0 pp. Tiny
> SD. Promoted from "a value that came out once" to "a repeatedly confirmed
> result". Experiment report: `docs/experiments/S2_GRPC_BASELINE.md`, raw data:
> `results/baseline-20260820/`.
>
> ✅ **S3 saturation (2026-08-20).** Each configuration's ceiling =
> **115 / 232 / 342 inf/s** (1/2/3 node), 3N = 2.97× (99%). Near-linear by the
> ceiling measure too. `docs/experiments/S3_SATURATION.md`.
>
> ✅ **S3.5 transport profiling (2026-08-20).** Settled what the −30% loss above
> actually is. **Not bandwidth** (51% of the link used per direction), **not
> board CPU capacity** (63% idle), **not kernel softirq concentration** (RPS A/B
> −0.2%), **not the server or scheduler** (three nodes scaling 3.00×). What
> remains is the **scheduler↔node HTTP/2 transport path** — the bench uses 32
> connections, one per worker, while the scheduler funnels to one connection per
> node, and the h2 windows are all at the default (64 KB). The node yields
> 161.5 inf/s local direct on the same board while managing only 116 in the
> cluster, with `node_queue` ≈ 0 showing headroom. Which of ①flow control
> ②connection/TCP ③protobuf and copies it is within that path **gets separated
> by the S3.6 A/B.** That result fixes S4 as either `gRPC optimized` or
> `io_uring`.
> `docs/experiments/S3_5_TRANSPORT_PROFILE.md`.
>
> ✅ **S3.6 H2/channel A/B (2026-08-20, 20 runs).** **Confirmed the single
> gRPC/HTTP2 connection per node as the primary factor limiting throughput.**
> Changing connections 1 → 4 alone gave **115.3 → 140.1 inf/s (+21.5%)**,
> recovering **54% of the 46.2 gap to local direct through configuration alone**
> (a connection pool, with no architectural change). But which of TCP per-flow /
> H2 multiplexing and locking / flow control interaction it is within that
> structure is **still unseparated**.
> Meanwhile **a 64 MB-class large window was badly harmful on this workload at
> −36.3%** — meaning the 64 KB default was functioning as backpressure (the
> default is kept). But 64 KB→64 MB is a 1000× extreme A/B, so it is **not
> concluded as "window tuning is ineffective"** (intermediate values unmeasured).
> The cost is **p95 46% worse** (393 → 573 ms) — five candidate causes, all
> unverified.
> CPU0 saturation surfaced as a new bottleneck (busy 81%, soft 74%), and with
> four flows there is now room to revisit S3.5b's RPS null.
> **io_uring is pushed back in the ordering** — syscall/req is nearly invariant
> at 80–94 across conditions while throughput differs twofold, 73.5–140.1. (The
> syscall *count* being equal and the syscall/copy *CPU time* being small are
> different questions, so this does not mean it has no effect.)
> **The S2 and S3 numbers are not updated yet** — they are 1-node results, and
> at three nodes the server would hold 12 connections. 1N/2N/3N get re-measured
> after S3.7 fixes N.
> `docs/experiments/S3_6_H2_CHANNEL_AB.md`.
>
> ✅ **S3.7 + S3.8 optimized gRPC (2026-08-20, 146 runs total).** The operating
> point was settled at **2 connections per node @ concurrency 12** and scale-out
> re-verified. The operating-point definition is **the lowest concurrency
> delivering at least 98% of peak.**
>
> | | baseline (conn1) | optimized (conn2/node) | gain |
> |---|---:|---:|---:|
> | 1N | 115.2 | **135.5** | +17.6% |
> | 2N | 232.0 | **263.3** | +13.5% |
> | 3N | 341.8 | **387.2** | +13.3% |
> | scaling | 2.97× (98.9%) | **2.86× (95.3%)** | |
>
> Zero errors, 0.0 pp distribution deviation. But **scaling efficiency slipped
> slightly** — the per-node gain shrinks from +17.6% (1N) to +13.3% (3N), so the
> single-node optimization is not fully preserved at multiple nodes. The server
> 10G link rising from 67% to **76%** is the leading candidate but is
> unconfirmed → next is a **server-side profile**.
> `docs/experiments/S3_7_CONNECTION_TUNING.md`, `S3_8_OPTIMIZED_SCALEOUT.md`.

---

# 3. Software status

| Crate | Status | Note |
|---|---|---|
| `npuforge-common` | ✅ | types, error codes (16 kinds), configuration, backend interface |
| `npuforge-proto` | ✅ | gRPC definitions. `SchedulerService` / `NodeService` |
| `npuforge-mock-backend` | ✅ | deterministic seed. Development without hardware |
| `npuforge-rknn` | ✅ | context pool, multiple outputs, real-hardware verification |
| `npuforge-node` | ✅ | worker pool, gRPC server, registration and heartbeat |
| `npuforge-scheduler` | ✅ | three policies, registry, retries |
| `npuforge-bench` | ✅ | load, aggregation, run-validity judgement |

**209 tests, clippy `-D warnings`, fmt clean.** (as of 2026-08-14)

## 3.1 Verified behaviour

The local Mock 3-node integration test
(`crates/npuforge-scheduler/tests/mock_cluster.rs`). It runs over real gRPC, so
the transport path is the same as on real hardware.

| Item | Result |
|---|---|
| Requests spread across 3 nodes | ✅ round-robin |
| Bypass when 1 node dies | ✅ 6/6 succeeded |
| All nodes dead | ✅ `NPF-1302` + the list of nodes attempted |
| Timing breakdown | ✅ both node and scheduler sections |
| Avoiding a slow node | ✅ least-queue |

Also confirmed with four real processes (scheduler + 3 nodes). **Killing the
scheduler and bringing it back has all three nodes re-register by themselves
within about 1.3 seconds.**

The six real-hardware RKNN integration tests
(`crates/npuforge-rknn/tests/real_device.rs`) pass too — returning 9 outputs,
determinism, uncontaminated results under 4-thread concurrent inference, and so
on.

## 3.2 Not implemented

- Prometheus metrics
- The REST management API and dashboard
- JPEG decoding (currently raw RGB8/BGR8 only)
- Postprocessing (NMS). The node returns raw tensors as-is

---

# 4. Inverted conclusions

**Things whose conclusion changed on re-measurement.** Publishing the first
conclusion would have meant saying something wrong.

## 4.1 "king runs 19 °C hotter" → did not reproduce

Under 08-10 sustained load, `king` alone hit NPU 91.3 °C, 19 °C above the other
two (70.2/72.1). It was judged a physical placement problem and raised to top
priority.

Re-measured under controlled conditions on 08-11, the **spread was 5.6 °C.**

The cause was not placement but **a difference in load profile.**

| | 08-10 | 08-11 |
|---|---|---|
| Tool | `thread_safety_test` | `sustained_load_test` |
| Load | sequential sweep 1→8 threads | fixed at 8 threads |
| Start | king led by 6 minutes | simultaneous |

`thread_safety_test` runs single- and two-thread baselines before reaching the
target thread count. `king` had been heating for far longer by the time the
other two entered 8 threads.

The decisive evidence: **`queen`'s peak temperature was an identical 70.2 °C in
both measurements.** Only `king` moved.

→ **Do not compare temperatures between different load profiles.**

## 4.2 "78 inf/s is a driver characteristic" → scope narrowed

Having confirmed that roughly 80 kernel ioctls per inference get serialized, the
node ceiling of 78 inf/s was defined as a driver characteristic. It also fit
with all three application optimizations being meaningless.

Then INT8 came in at 1.85×. The ioctl counts were checked.

```
strace -c -f -e trace=ioctl, 1 thread, 20 s
  FP16  315 inferences  15.7 inf/s  24,079 ioctls  76.4 per inference
  INT8  718 inferences  35.8 inf/s  54,707 ioctls  76.2 per inference
```

**The count is the same and throughput is 2.28×.** What sets the ceiling is not
the ioctl count but **how long one inference holds the serialized section.**

The CPU governor experiment (+7%) reinforces this. That time includes not only
NPU execution but **the CPU work either side of it.**

→ "It cannot be exceeded by application optimization" holds. But **quantization
  is a model change, not an application optimization.**

## 4.3 "RKNN is thread-safe" → true, but the sequence is not atomic

`environment-matrix.md` §3.1 had concluded that "RKNN Runtime 2.3.0 is
thread-safe". Sharing one context would simplify the implementation.

But that verification **counted only API return codes and never compared output
contents.**

One inference is three calls.

```
rknn_inputs_set  ->  rknn_run  ->  rknn_outputs_get
```

Even with each call thread-safe, **the sequence is not atomic.**

| Configuration | API errors | **Result mismatches** |
|---|---:|---:|
| Shared context | 0 | **200 / 200 (100%)** |
| Per-thread dedicated context | 0 | 0 / 200 (0%) |

**A shared context produces 100% wrong answers with no errors.**

The nature of this defect is especially bad.

- No exception and no error code
- Never reproduces single-threaded
- **The throughput metric actually looks better** (in §3.1, shared 34.8 >
  dedicated 33.2 — it was producing wrong answers faster)
- The detections come from another frame, so they look plausible to the eye

Left alone, it would have reached a public talk **with all throughput valid and
only the detections quietly wrong.**

→ `RknnContext::infer` takes `&mut self`. **The compiler blocks concurrent
  calls.** Writing a rule in a comment and blocking it with a type are different
  things.

---

# 5. Verified but ineffective

The basis for there being almost nothing left to squeeze inside the node.

| Attempt | Result | Verdict |
|---|---:|---|
| Manual core assignment via `core_mask` | **+0.1%** | not used. `CORE_AUTO`'s distribution is already even |
| Zero-copy buffer reuse | **−1.8%** | not used. Hypothesis refuted |

`core_mask` was +9% at 4 threads but **vanishes to +0.1% at 8.** `CORE_0_1` is
actually a loss at −11.5% at 8 threads.

> **`want_float=0` was originally in this table.** The 08-10 measurement put it
> at FP16 +5.4%, close to "no effect". But that measurement was mostly a
> single-thread condition, and re-measuring on 08-12 at 8 threads for 120 s gave
> **INT8 +17.3% / FP16 +15.7%.** The time output conversion holds the serialized
> section grows with the number of concurrent threads.
> → moved to §2.2. **It was not ineffective; the measurement's conditions were
> inadequate.** `discuss.md` §12

---

# 6. The list of measurement failures

**The highest reuse-value result this project has.** All of them are failures
that "looked like success".

## 6.1 Not checking what a metric counts (4 times)

| # | What | Actually |
|---|---|---|
| 1 | Reading `RKNN_QUERY_PERF_RUN.run_duration` as NPU occupancy time | a value that included queue wait |
| 2 | Sampling NPU load at 0.2 s intervals with `delayms=3000` still set | it was reading a 3-second average |
| 3 | Judging thread-safety by API return codes alone | output contents were never compared |
| 4 | Judging throttling **by NPU clock alone** | the CPU was bending from A72 2208 to 816 MHz |

Number 1 was self-discovered through a contradiction ("5.03 cores in use" on a
2-core NPU). Number 2 was pointed out by a ChatGPT review. Number 3 was found by
suspicion while implementing the backend. Number 4 had the CPU clocks **already
recorded in the same log** and simply did not use them in the verdict — it was
found only when the FP16 re-measurement came out at 66.9 rather than 84.3
(§2.3).

**What they share: reading a metric's name and assuming its meaning.**

## 6.2 Not noticing a changed premise

| What | Result |
|---|---|
| A stale `king` IP in the documents (`.22`, actually `.12`) | misdiagnosed as "the node is dead"; scanned the whole subnet |
| Comparing two measurements with different load profiles | a 19 °C gap was misread (§4.1) |
| Misdiagnosing the cause of board resets three times | shared PSU → bootloader → 12V input. Actually **insufficient adapter current** |

`~/.ssh/config` had had the correct IP all along. **Only the IP pinned into the
documents was stale.** → Boards are reached by alias, not by IP.

## 6.3 Remote execution where failure looks like success

Found while building `preflight-check.sh`. The check **was silently not
working.**

**`pgrep -f` counts itself.** The ssh wrapper's command line contains the
pattern string, so it matches. The bracket trick (`[s]ustained`) is also
neutralised once a form without brackets appears on the same command line.

| Situation | Actual | pgrep reports |
|---|---|---|
| Load running | 1 | **0 (missed)** |
| No load | 0 | **2 (its own shell)** |

**`cd DIR && setsid nohup ... &` does not come up.** The `&` applies to the
whole list, and if ssh disconnects immediately the subshell dies before reaching
`setsid`. **Exit code 0, empty stderr.** Without checking, you measure "the
temperature with no load" for fifteen minutes.

**A heredoc inside ssh nested with sudo does not create the file.** Encountered
while deploying a systemd unit. This too gave exit code 0.

→ All of them hardened into functions in `scripts/lib/remote.sh`.

## 6.4 Judging "could not read" as "identical"

`/sys/kernel/debug/rknpu/version` is readable only by root. All three nodes
returned an empty value and it **passed on the grounds that the values matched.**
A variant of 6.1.

→ Empty values and placeholders such as `unknown` are treated as failures.

## 6.5 Tooling

To stop the same mistakes recurring, the checks were put into tools.

| Tool | What it blocks |
|---|---|
| `preflight-check.sh` | alias↔hostname, hash matching, governor, temperature, voltage, residual load, **inference accuracy** |
| `npuforge-bench` | warmup exclusion, reboot detection via `boot_id`, insufficient-sample verdicts, excluding failures from throughput |
| `run-thermal-comparison.sh` | simultaneous start, hash verification, confirming load actually started |
| `scripts/lib/remote.sh` | the remote execution pitfalls |

**Accuracy is checked before performance** — `preflight --with-inference` checks
that the three boards give the same answer to the same input. A configuration
that produces wrong answers fast must not win a benchmark.

---

# 7. Reproduction

## 7.1 Without hardware

```bash
cargo test --workspace          # 209 tests
cargo clippy --workspace --all-targets -- -D warnings
```

The Mock 3-node cluster also runs without hardware.

```bash
cargo build --release -p npuforge-scheduler -p npuforge-node -p npuforge-bench
./target/release/npuforge-scheduler --config configs/scheduler.example.toml &
for i in 1 2 3; do ./target/release/npuforge-node --config configs/mock/node-0$i.toml & done
./target/release/npuforge-bench --scheduler http://127.0.0.1:50051 \
  --model yolov8n --concurrency 6 --duration 15
```

## 7.2 On real hardware

Prerequisites: the `npuforge-k/q/j` aliases in `~/.ssh/config`, and
`NPUFORGE_SUDO_PASS` set.

```bash
bash scripts/preflight-check.sh --with-inference   # no measuring until it passes
bash scripts/run-thermal-comparison.sh 900 8       # thermal comparison
```

## 7.3 Model conversion

```bash
python tools/model-converter/fetch_calibration.py --out datasets/coco-calib --count 200
docker run --rm -v "$PWD/models:/work/models" -v "$PWD/datasets:/work/datasets" \
  npuforge-converter:2.3.1 python3 /work/tools/convert_yolov8n.py \
  --onnx models/yolov8n.onnx --out models/yolov8n-int8.rknn \
  --dataset datasets/coco-calib --calib-limit 200
```

> **The model is converted once and the same file deployed to all three nodes.**
> INT8 conversion is not byte-reproducible — converting three times from the
> same input gave a different hash each time (1.8% of bytes differing). But
> **the inference results are completely identical** (all 9 tensors at cosine
> 1.000000). The difference is in serialization, not in computation.
>
> `model.toml`'s `sha256` guarantees **deployment integrity**, not identity of
> the conversion recipe.

---

# 8. What comes next

## 8.1 Blocked — 10G aggregation is needed

One raw RGB input is `640 × 640 × 3 = 1,228,800 byte`.

```text
INT8  1,228,800 x 157.2 x 8 = 1.545 Gbps / node   ->  3 nodes 4.636 Gbps
FP16  1,228,800 x  84.3 x 8 = 0.829 Gbps / node   ->  3 nodes 2.486 Gbps
```

> **Corrected 2026-08-12.** Earlier documents had 1.43 / 4.3 Gbps, because
> converting MiB/s to Gbps used the binary prefix (÷1024). **Network speeds are
> decimal.** The correct values are above.

### The output is larger

The above is **input (TX) only.** The node does not postprocess and returns nine
raw tensors, and with `want_float=1` the output is **3.96× the input.**

```text
input                       1,228,800 byte
output (want_float=1, f32)  4,872,000 byte
output (want_float=0, int8) 1,218,000 byte
```

The scheduler-side link load at three-node saturation:

| Configuration | Model | 3-node TX | 3-node RX | Fits in 10G? |
|---|---|---:|---:|---|
| `want_float=1` (old default) | INT8 | 4.64 Gbps | **18.38 Gbps** | **no** |
| `want_float=1` (old default) | FP16 | 2.49 Gbps | **9.86 Gbps** | barely |
| **`want_float=0` (current default)** | INT8 | 4.64 Gbps | 4.60 Gbps | yes |
| **`want_float=0` (current default)** | FP16 | 2.49 Gbps | 2.46 Gbps | yes |

**Had `want_float=1` remained, even 10G could not have carried three INT8
nodes.**

→ One of two things was needed before M3.
  **(A)** switch to `want_float=0` — the output becomes a quarter
  **(B)** postprocess (NMS) on the node — the response shrinks to a few KB, but
  it is unimplemented

  `want_float=0`, filed in §5 as "an optional optimization, deferred", was
  **promoted to a precondition for M3.** **The grounds for the promotion were
  not throughput but RX bandwidth.**

> ✅ **(A) completed 2026-08-12.** The node configuration `[worker] want_float`
> default changed to `false`, and the blob was bumped to **v2** to carry
> `qnt_type`, `scale` and `zero_point` per tensor. Without those, sending int8
> leaves the receiver unable to interpret it. Dequantization was confirmed on a
> real board to match float32 (9 tensors, **maximum error 9.5e-7** — the limit
> of float32 precision).
> Throughput rose alongside — **INT8 +17.3% / FP16 +15.7%** (§2.2,
> `discuss.md` §12).
>
> Additionally, `sustained_load_test` had hardcoded `want_float=0` from the
> beginning. So §2.2's 157.2 / 84.3 were **already on `want_float=0`**, and this
> change **brought the Rust backend in line with the measurement conditions.**

### Summary

- It is the **aggregation link, not the worker links (2.5G), that fills up
  first**
- **10G** is needed on the scheduler side
- **A measure to reduce output size** has to accompany it

**Calculating the input and not looking at the output was this section's
original error.** The failure list in §6 already has three of the same type.

## 8.2 Possible without the switch

- Prometheus metrics
- Configuring an NTP server on `dealer`
- The formal S0 thermal characterisation (30 min × fanless/cooled, 2 conditions)
- `ondemand` vs `performance` over 300 s — §2.2's +7% is a 120-second value
- INT8's thermal behaviour (does less computation mean less heat)

---

# 9. Document guide

| Document | Content |
|---|---|
| `../adrs/` | **Why it was decided that way.** 28 decisions, by topic |
| `TODO.md` | **What to do now.** The resumption procedure is at the top |
| `RESULTS.md` | This document. A collection of results |
| `discuss.md` | The discussion and reasoning. 11 chronological sections |
| `board-worklog.md` | Work history. Failed hypotheses are preserved |
| `environment-matrix.md` | Settled environment values |
| `infrastructure.md` | A snapshot of the current setup |
| `00-PRD.md` – `03-*.md` | Requirements and design specifications |

When numbers disagree between documents, **`environment-matrix.md` is the
authority** (the document authority order in `00-PRD.md` §0).

---

<a id="discuss"></a>

# NPUDure 기술 논의

이 문서는 설계 판단이 갈리는 지점의 논의를 기록한다. 출처(누구의 의견인지)를 명시해 나중에 어떤 근거로 결정했는지 추적할 수 있게 한다.

측정 원본은 `benchmarks/`, 확정 사실은 `environment-matrix.md`, 작업 이력은 `board-worklog.md`.

**각 절에 작성 시각(KST)과 커밋 해시를 적는다.** 같은 날 여러 실험을
하면 날짜만으로는 순서를 알 수 없고, 나중에 "이 결론이 저 측정보다
먼저였나 나중이었나"를 판단할 수 없다.

---

## 읽는 순서

논의는 시간순으로 배치한다. 새 의견은 문서 끝에 덧붙인다.

| # | 절 | 작성 시각 (KST) | 작성 | 요지 |
|---|---|---|---|---|
| 1 | NPU 점유율 판별 실험 | 08-10 (시각 불명) | Claude | 최초 측정과 해석 |
| 2 | ChatGPT 답변/의견 | 08-10 (시각 불명) | ChatGPT | 표현 완화와 재검증 요구 |
| 3 | Claude 재검토 | 08-10 (시각 불명) | Claude | 지적 수용 및 재측정 |
| 4 | core_mask 분배 실험 | **08-10 17:03** | Claude | 대조군 추가, `worker_count=8` 확정 |
| 5 | want_float 실험 | **08-10 17:15** | Claude | 출력 변환 제거, +5.4% |
| 6 | syscall 분해 | **08-10 17:26** | Claude | 병목 확정: 드라이버 ioctl 직렬화 |
| 7 | zero-copy 실험 | **08-10 17:44** | Claude | 가설 반증 |
| 8 | INT8 실측 | **08-11 16:45** | Claude | **1.85배**. 6·7절 결론을 정교화 |
| 9 | 공유 컨텍스트 실험 | **08-11 16:45** | Claude | "오류 0건"은 정답이 아니다 |
| 10 | 벤치 도구 설계 | **08-11 17:15** | Claude | 실수를 도구에 박아 넣기 |
| 11 | CPU governor 영향 | **08-12 10:16** | Claude | **+7%**. 기존 수치는 전부 `ondemand` 기준 — **현재 유효한 결론** |

1~3절은 최초 커밋(`eda93a3`, 08-10 16:29)에 함께 들어가 절 단위 시각을
복원할 수 없다. 4절부터는 커밋 시각이 그대로 작성 시각이다.

**1절의 일부 수치는 3절에서 정정되었다.** 결론만 필요하면 3절을 본다.
**6·7절의 "78 inf/s 상한" 표현은 8절에서 범위가 좁혀졌다.**

---

# NPU 점유율 판별 실험 — Claude 결과/의견

> ⚠️ **이 절의 NPU load 수치(30%)와 일부 결론은 후속 재측정으로 정정되었다.**
> 문서 하단 「Claude 재검토」를 함께 볼 것. 원문은 판단 과정을 남기기 위해 보존한다.

- 작성: 2026-08-10 (최초 커밋 `eda93a3` 16:29 에 포함. 절 단위 시각은 불명)
- 측정 노드: `queen` (NanoPi R76S, RK3576)
- 모델: `yolov8n-fp16.rknn` (FP16, SHA-256 `459602ea70479c1c...`)
- 도구: `crates/npuforge-rknn/native/npu_occupancy_test.c`

## 배경: 무엇을 판별하려 했나

thread-safety 시험에서 **NPU가 2코어인데 8스레드에서 처리량이 5.55배**로 올랐다. 원인 가설이 둘로 갈렸다.

| 가설 | 내용 | 최적화 방향 |
|---|---|---|
| **A** | NPU submission pipeline이 덜 채워져 있었다 | NPU를 더 잘 먹이기 (배칭, 큐잉) |
| **B** | 호출당 시간의 상당 부분이 CPU 구간이고, 스레드가 이를 병렬화했다 | CPU 전처리·후처리 최적화 |

두 가설은 이후 최적화 방향을 정반대로 이끈다. 잘못 해석하면 M7 전체가 틀어진다.

## 측정 결과

### 스레드 수별

| 스레드 | 처리량 | NPU Core0 / Core1 | CPU | inputs_set | run | outputs_get |
|---:|---:|---|---:|---:|---:|---:|
| 1 | 17.0 /s | **16% / 0%** | 9.8% | 17.7 ms | 28.4 ms | 12.3 ms |
| 2 | 33.3 | **17% / 15%** | 17.2% | 17.8 | 29.6 | 12.3 |
| 4 | 56.8 | **25% / 24%** | 26.9% | 19.1 | 40.9 | 10.3 |
| 8 | 76.0 | **32% / 30%** | 43.0% | 24.6 | 65.9 | 12.4 |

NPU 점유율은 커널 debugfs(`/sys/kernel/debug/rknpu/load`)에서 0.2초 간격으로 샘플링했다.

### 1스레드 호출 구성

```text
총 58.9 ms
  inputs_set     17.7 ms  (30%)   CPU
  run            28.4 ms  (48%)   NPU 제출 + 실행 + 대기
  outputs_get    12.3 ms  (21%)   CPU (want_float=1 이므로 역양자화 포함)
  release         0.5 ms  ( 1%)
```

## ⚠️ 먼저 정정: 내 지표가 틀렸다

`RKNN_QUERY_PERF_RUN`의 `run_duration`을 "실제 NPU 점유 시간"으로 해석했으나 **큐 대기를 포함한 값**이었다. 헤더 주석의 `real inference time (us)` 표현에 오도되었다.

**증거:**

- 2코어인데 내 계산으로 `npu_cores_busy = 5.03`이 나왔다. 물리적으로 불가능하다
- `run_duration`이 `rknn_run`의 wall time과 모든 구간에서 정확히 일치한다. 즉 같은 값이다

**커널 debugfs가 신뢰할 수 있는 출처였다.** RKNN API가 보고하는 값을 검증 없이 쓰면 안 된다.

이것은 이 실험 전에 저지른 두 번째 성급한 해석이다. 첫 번째는 "5.55배니까 CPU가 병목"이었고, 그것도 틀렸다.

## 결론: 두 가설 모두 부분적으로만 맞다

### 가설 A가 주된 답이다

1스레드에서 **Core0 16%, Core1 0%** 로 NPU가 사실상 놀고 있다. 스레드를 늘리자 두 코어가 함께 동작하며 처리량이 17 → 76 inf/s(4.5배)로 올랐다.

**단, NPU는 끝까지 포화되지 않는다.** 8스레드에서도 30% 수준이다.

### 그러나 CPU도 병목이 아니다

8스레드에서 CPU 사용률이 **43%** 다. 8코어 중 약 3.4코어만 사용한다. 여유가 있다.

### 진짜 상한은 다른 곳에 있다

`run`이 28.4 → 65.9 ms로 **2.3배** 늘어난 반면, `inputs_set`(17.7→24.6)과 `outputs_get`(12.3→12.4)은 거의 변하지 않았다.

```text
NPU 점유율   30%   ← 바쁘지 않다
CPU 사용률   43%   ← 바쁘지 않다
rknn_run     66ms  ← 그런데 여기서 기다린다
```

**둘 다 포화가 아닌데 지연만 늘어나는 것은 큐잉 병목의 전형이다.** NPU 제출 경로 어딘가에서 직렬화가 일어나고 있다.

후보:

| 후보 | 설명 |
|---|---|
| RKNN 런타임 내부 락 | 여러 context가 하나의 제출 경로를 공유 |
| 커널 드라이버 직렬화 | ioctl 경로 또는 IOMMU 매핑의 배타 구간 |
| NPU 스케줄링 정책 | `CORE_AUTO`가 코어를 충분히 활용하지 못함 |

## 프로젝트에 주는 의미

### 최적화 우선순위가 바뀐다

| 최적화 | 효과 예상 | 근거 |
|---|---|---|
| CPU 전처리 최적화 | 지연시간 개선. **처리량 상한은 못 올림** | CPU가 병목이 아님 (43%) |
| io_uring | **무관** | 네트워크가 관여하지 않는 구간의 문제 |
| **NPU 제출 경로 직렬화 해소** | **상한 자체를 올림** | 여기가 실제 병목 |
| INT8 전환 | `run` 시간 단축 가능 | FP16이 NPU에서 비효율적일 수 있음 |

### 프로젝트 논지와 부합한다

**NPU가 70% 놀고 있다.** "6 TOPS × 3 = 18 TOPS가 아닌" 이유가 네트워크나 스케줄링이 아니라 **노드 하나 안에서 NPU를 다 쓰지 못하기 때문**일 수 있다.

이는 벤더 스펙시트의 TOPS가 실제 처리량을 대표하지 못한다는 본 프로젝트의 문제 정의(PRD §2)를 정면으로 뒷받침한다.

## 다음에 확인할 것

우선순위 순이다.

1. **`core_mask` 명시 분배** — `CORE_0`/`CORE_1`을 직접 지정하면 30% 벽이 깨지는가
   - thread-safety 시험에서는 코어 분리가 오히려 소폭 느렸으나, 그것은 **2스레드** 조건이었다
   - 8스레드에서는 결과가 다를 수 있다
2. **INT8 vs FP16** — `run` 시간이 줄면 상한이 올라간다. calibration 데이터 확정 후
3. **`want_float=0`** — `outputs_get`의 역양자화 제거. 지연시간 개선
4. **`rknn_dup_context`** — 컨텍스트 복제가 개별 `rknn_init`보다 나은가

1번이 가장 빠르게 답이 나온다.

## 측정 방법 메모

재현에 필요한 사항이다.

```bash
# 빌드
gcc -O2 -Wall -Wextra -o npu_occupancy_test npu_occupancy_test.c -lrknnrt -lpthread

# 실행 (모델, 반복횟수, 스레드수)
./npu_occupancy_test yolov8n-fp16.rknn 120 8

# NPU 점유율은 별도로 샘플링해야 한다 (sudo 필요)
sudo cat /sys/kernel/debug/rknpu/load
```

**`RKNN_QUERY_PERF_RUN`은 `RKNN_FLAG_COLLECT_PERF_MASK` 없이 조회 가능하며 오버헤드가 없다.** `PERF_DETAIL`만 플래그를 요구하고 프레임률을 떨어뜨린다.

다만 위에 적었듯 `run_duration`은 큐 대기를 포함하므로 **NPU 점유율 판단에 사용하면 안 된다.**

---

---

# ChatGPT 답변/의견

> 최신 결론은 문서 하단 「Claude 재검토 — ChatGPT 지적 반영 및 재측정」을 따른다.
> 이 절은 재측정을 유도한 검토 의견으로 보존한다.

- 작성: 2026-08-10 (최초 커밋 `eda93a3` 16:29 에 포함. 절 단위 시각은 불명)
- 대상: `NPU 점유율 판별 실험 — Claude 결과/의견`

## 총평

`discuss.md`의 방향은 좋다. 특히 **"5.55배니까 CPU 병목"이라는 성급한 해석을 철회한 것**, 그리고 `RKNN_QUERY_PERF_RUN.run_duration`을 실제 NPU 점유시간으로 보지 말아야 한다고 정정한 부분은 중요하다.

이 논의는 남겨둘 가치가 있다. 나중에 발표에서도 "처음 세운 병목 가설을 실측으로 어떻게 수정했는가"를 보여주는 좋은 자료가 된다.

다만 몇 군데는 표현을 조금 낮추는 것이 좋다.

## 1. "CPU도 병목이 아니다"는 아직 단정하지 않는다

전체 CPU 사용률 43%는 평균값이다. 특정 thread 하나가 락, 드라이버, ioctl 경로에서 막히거나 단일 코어가 포화되는 상황을 가릴 수 있다.

8코어 전체 평균이 낮아도 병목이 병렬화 불가능한 CPU 경로에 있으면 처리량 상한이 생긴다.

따라서 현재 문장의 의미는 다음 정도로 제한하는 것이 정확하다.

```text
CPU 전체 사용률 기준으로는 여유가 있다.
다만 단일 코어 포화, runtime lock, ioctl/off-CPU wait 가능성은 별도로 확인해야 한다.
```

## 2. "io_uring 무관"도 너무 강하다

이 실험은 단일 노드 내부 RKNN 호출 병목을 본 것이다. 이 결과만으로 분산 추론 경로의 `io_uring` 가치를 판단할 수는 없다.

맞는 결론은 다음이다.

```text
io_uring은 이 단일 노드 RKNN scaling 문제의 직접 원인은 아니다.
분산 transport 최적화 여부는 M2/M3의 network_* timing과 syscall 계측 후 판단한다.
```

즉, 현재 관측된 8스레드 scaling 한계의 원인은 네트워크 I/O가 아니다. 그러나 NPUDure 전체에서 `io_uring`이 의미 있는지는 gRPC baseline 이후 별도 측정해야 한다.

## 3. NPU load 30% 해석은 한 번 더 검증한다

`/sys/kernel/debug/rknpu/load`가 가장 믿을 만한 출처라는 판단은 타당하다. 다만 그 값의 의미가 직전 샘플 구간 평균인지, 드라이버 내부 누적/감쇠 값인지 확인이 필요하다.

8스레드에서 76 inf/s가 나오는데 Core0/Core1이 30%대라는 것은 가능하지만 꽤 강한 신호다. 다음을 확인하면 해석이 더 단단해진다.

- 샘플링 주기 0.2초의 영향
- 부하 없음 상태의 baseline
- `watch -n`과 직접 루프 샘플링의 차이
- read 직후 값 변화 여부
- NPU devfreq와 load 값의 동시 기록

## 4. 실제 상한 후보는 조금 넓게 둔다

"RKNN/NPU 제출 경로 직렬화"는 유력한 가설이다. 다만 후보를 너무 좁히지는 않는 것이 좋다.

확인할 후보:

- RKNN runtime 내부 lock
- kernel driver ioctl 직렬화
- IOMMU 또는 buffer mapping 비용
- DDR 또는 memory bandwidth
- output conversion 또는 hidden copy
- thermal/frequency 변화
- `CORE_AUTO` 스케줄링 한계

특히 `inputs_set`이 17~25ms인 것은 생각보다 크다. 100KB JPEG 네트워크 전송보다 훨씬 큰 시간이다. 실제 모델 입력 tensor 크기, copy 횟수, cache behavior, zero-copy input 가능 여부도 봐야 한다.

## 5. 다음 실험 제안

우선순위는 다음과 같다.

1. `CORE_AUTO` vs `CORE_0`/`CORE_1` 명시 분배를 2스레드뿐 아니라 4/8스레드에서 다시 측정
2. `want_float=0`으로 `outputs_get` 비용 제거 후 처리량 변화 확인
3. `perf top`/`perf record`로 user/kernel hot path 확인
4. `strace -c` 또는 `perf stat`으로 ioctl/syscall 수 확인
5. per-core CPU 사용률과 off-CPU time 확인
6. INT8 모델 생성 후 FP16과 같은 실험 반복

## 정리 의견

현재 문서의 방향은 맞다. 다만 최종 결론은 조금 강하므로 다음 정도로 정리하는 것이 더 안전하다.

```text
현재 실험은 8스레드 처리량 증가가 단순 CPU 전처리 병렬화만으로 설명되지 않음을 보였다.
1스레드에서는 NPU가 충분히 사용되지 않았고, 다중 스레드에서 NPU submission pipeline이 더 채워진다.
그러나 8스레드에서도 NPU/CPU 전체가 포화되지 않았으므로, RKNN runtime 또는 kernel driver 제출 경로의 직렬화/큐잉 가능성이 높다.
이 문제는 io_uring과는 별개의 단일 노드 내부 병목이며, 분산 transport 최적화 여부는 gRPC baseline 이후 TimingBreakdown으로 따로 판단한다.
```

---

---

# Claude 재검토 — ChatGPT 지적 반영 및 재측정

- 작성: 2026-08-10 (최초 커밋 `eda93a3` 16:29 에 포함. 절 단위 시각은 불명)
- 대상: 위 `ChatGPT 답변/의견` 5개 항목
- 재측정 도구: 동일 (`npu_occupancy_test.c`) + 샘플링 절차 수정

## 요약

**지적 5개 중 3개를 실측으로 확인했고, 그 결과 앞선 수치가 과소평가였음이 드러났다.**

가장 큰 것은 3번(NPU load 재검증)이다. 측정 방법 자체에 결함이 있었다.

## 지적 3 확인: `delayms = 3000` — 측정 방법 결함

`/sys/kernel/debug/rknpu/` 아래에 `delayms` 파일이 있고 값이 **3000**이었다.

**NPU load가 3초 창(window)으로 평균되는데 0.2초 간격으로 샘플링했다.** 같은 값을 15회씩 중복해서 읽었고, 부하 종료 후의 0% 구간까지 평균에 섞였다.

`delayms`는 쓰기 가능하다. **100ms로 낮추고**, 워밍업 4초 후 정상 구간만 샘플링해 재측정했다.

```bash
sudo sh -c 'echo 100 > /sys/kernel/debug/rknpu/delayms'
```

무부하 baseline은 0%로, 값이 누적/감쇠되지 않음을 확인했다.

### 재측정 결과

| 스레드 | 처리량 | Core0 avg/max | Core1 avg/max | CPU (cpu0~3 / cpu4~7) |
|---:|---:|---|---|---|
| 1 | 16.7 /s | 18.9% / 39% | **0.5% / 1%** | 3,1,2,2 / **30,23**,6,7 |
| 2 | 36.0 | 23.4% / 48% | 16.0% / 33% | 3,2,3,3 / 28,36,40,19 |
| 4 | 55.6 | 29.6% / 62% | 26.9% / 54% | 18,8,7,4 / 48,45,46,48 |
| 8 | 75.0 | **38.9% / 86%** | **37.0% / 81%** | 47,43,44,42 / 48,49,42,46 |

**앞선 30%는 과소평가였다.** 실제 평균은 38.9%, **순간 최대는 86%** 다.

## 지적 1 확인: "CPU도 병목이 아니다"는 부정확했다

전체 평균 43%가 **코어별 불균형을 가리고 있었다.**

```text
1스레드  cpu4=30%, cpu5=23%      big 코어 일부만 사용, little 은 유휴
8스레드  전 코어 42~49%          고르게 분산
```

`cpu0~3`이 little(A53 2.016GHz), `cpu4~7`이 big(A72 2.208GHz)이다. 1스레드에서 big 코어 하나가 30%를 쓰는 것은 단일 스레드 기준으로 작지 않은 부하다.

**정정된 표현:**

```text
CPU 전체 사용률 기준으로는 여유가 있다 (8스레드에서 최대 49%).
단일 코어 포화는 관측되지 않았다.
다만 runtime lock, ioctl 직렬화, off-CPU wait 는 별도 계측이 필요하다.
```

`perf record` / `strace -c` / off-CPU 분석은 아직 하지 않았다.

## 지적 2 수용: "io_uring 무관"은 범위를 넘은 표현

이 실험은 **단일 노드 내부 RKNN 호출 경로**를 본 것이다. 분산 transport의 가치를 판단할 근거가 아니다.

**정정된 표현:**

```text
이 단일 노드 RKNN scaling 한계의 직접 원인은 네트워크 I/O 가 아니다.
NPUDure 전체에서 io_uring 이 의미 있는지는 gRPC baseline 이후
TimingBreakdown 과 syscall 계측으로 별도 판단한다.
```

앞 절 「최적화 우선순위」 표의 `io_uring — 무관` 항목은 이 문장으로 대체한다.

## 지적 4 수용: 상한 후보를 넓힌다

특히 **`inputs_set` 17~25ms** 가 크다는 지적이 타당하다. 입력 텐서가 640×640×3 = 1,228,800 bytes 인데 17ms 면 약 70MB/s 다. 단순 memcpy 라기엔 느리므로 포맷 변환이나 다중 복사가 의심된다.

확인할 후보 (좁히지 않는다):

| 후보 | 확인 방법 |
|---|---|
| RKNN runtime 내부 lock | `perf record`, off-CPU 분석 |
| kernel driver ioctl 직렬화 | `strace -c`, ioctl 횟수·소요시간 |
| IOMMU / buffer mapping 비용 | `perf`, 드라이버 trace |
| DDR / memory bandwidth | `inputs_set` 처리율 대비 이론 대역폭 |
| output conversion / hidden copy | `want_float=0` 비교 |
| thermal / frequency 변화 | devfreq `cur_freq` 동시 기록 (이번엔 950MHz 고정 확인) |
| `CORE_AUTO` 스케줄링 한계 | `core_mask` 명시 분배 비교 |

## 수정된 결론

```text
8스레드 처리량 증가는 CPU 전처리 병렬화만으로 설명되지 않는다.

1스레드에서 NPU Core1 이 0.5% 로 사실상 미사용이었고,
스레드를 늘리자 두 코어가 균등하게 38~39% 까지 올라갔다.
순간 최대는 86% 로 포화에 근접하나 평균은 40% 미만이다.

즉 NPU 를 연속적으로 먹이지 못하고 간헐적으로 비는 구간이 있다.
CPU 는 전체 49% 이하이고 단일 코어 포화도 없다.

따라서 RKNN runtime 또는 kernel driver 제출 경로의
직렬화·큐잉 가능성이 높으나, lock/off-CPU 계측 전까지 확정하지 않는다.

이는 단일 노드 내부 병목이며 io_uring 과는 별개 문제다.
분산 transport 최적화 여부는 gRPC baseline 이후 별도 판단한다.
```

## 운영에 반영할 것

**`delayms` 는 재부팅 시 3000 으로 돌아간다.** NPU load 를 telemetry 로 쓰려면 측정 전 설정이 필요하다.

`preflight-check.sh` 에 추가할 항목:

```bash
sudo sh -c 'echo 100 > /sys/kernel/debug/rknpu/delayms'
# 확인
[ "$(sudo cat /sys/kernel/debug/rknpu/delayms)" = "100" ] || 중단
```

그리고 NPU load 샘플링 규칙:

- 샘플 간격은 `delayms` 이상으로 둔다 (중복 읽기 방지)
- 워밍업 구간과 종료 직후 구간을 평균에서 제외한다
- 평균과 함께 **최대값을 기록한다** (평균만 보면 순간 포화를 놓친다)

## 아직 하지 않은 검증

ChatGPT 제안 중 미실시 항목이다.

- [x] `CORE_AUTO` vs `CORE_0`/`CORE_1` 명시 분배 (4/8스레드) — 4절 참조
- [x] `want_float=0` 으로 `outputs_get` 비용 제거 — 5절 참조
- [x] `perf` 는 커널 버전 불일치로 불가. `time` + `strace -c` 로 대체 — 6절
- [x] `strace -c` 로 ioctl 수 확인 — 추론당 80회, 6절
- [x] off-CPU 분석 — 8스레드에서 블록 58ms/호출, 6절
- [ ] INT8 모델로 동일 실험 반복 (calibration 데이터 확정 후)

## 메타: 같은 실수를 두 번 했다

이번 건에서 성급한 해석이 세 번 있었다.

| # | 잘못된 판단 | 원인 |
|---|---|---|
| 1 | "5.55배니까 CPU 병목" | 대안 가설을 배제하지 않음 |
| 2 | "`run_duration` = NPU 점유시간" | API 주석을 검증 없이 신뢰 |
| 3 | "NPU load 30%" | 측정 도구의 샘플링 특성 미확인 |

공통점은 **측정값이 무엇을 의미하는지 확인하기 전에 결론을 냈다는 것**이다.

2번은 자체 모순(2코어인데 5.03)으로 잡았고, 3번은 외부 지적으로 잡았다. 1번은 실측으로 잡았다.

교훈: **새 측정 지표를 쓸 때는 값의 의미·갱신 주기·경계 조건을 먼저 확인한다.** 특히 커널이 노출하는 값은 문서가 없는 경우가 많으므로 무부하 baseline과 극단값으로 검증한다.

---

# core_mask 분배 실험 — Claude 결과/의견

- 작성: **2026-08-10 17:03 KST** (커밋 `0e6e264`)
- 측정 노드: `queen`
- 모델: `yolov8n-fp16.rknn` (FP16)
- 원본: `benchmarks/results/2026-08-10-coremask/coremask-queen.txt`
- 조건: 스레드당 200회, `delayms=100`, 워밍업 4초 후 샘플링

## 확인하려던 것

앞 절에서 "`CORE_AUTO`가 코어를 충분히 활용하지 못한다"를 병목 후보로 두었다. 이를 검증한다.

**대조군을 추가했다.** 이전까지는 Core1 점유율이 38%라는 숫자만 봤을 뿐, 그것이 실제 처리량 기여로 이어지는지 확인한 적이 없다. 모든 스레드를 코어 하나에 고정한 `CORE_0_ONLY`와 비교하면 판정할 수 있다.

| mode | 설정 | 의도 |
|---|---|---|
| 0 | `CORE_AUTO` | 런타임이 선택 (현재 기본값) |
| 1 | `ALTERNATE` | 스레드를 `CORE_0`/`CORE_1`에 번갈아 고정 |
| 2 | `CORE_0_1` | 모든 스레드가 두 코어를 함께 사용 |
| 3 | **`CORE_0_ONLY`** | **전부 코어 0에 고정 — 대조군** |

## 결과

처리량 (inf/s)

| 스레드 | AUTO | ALTERNATE | CORE_0_1 | **CORE_0_ONLY** |
|---:|---:|---:|---:|---:|
| 1 | 16.7 | 16.7 | **18.2** | 16.5 |
| 2 | 36.2 | 36.5 | 36.4 | **26.4** |
| 4 | 52.4 | **57.1** | 48.5 | **38.5** |
| 8 | 72.9 | **73.0** | 64.5 | **48.2** |

`run` 시간 (µs) 및 NPU 점유율 (평균/최대 %)

| 스레드 | mode | run | Core0 | Core1 |
|---:|---|---:|---|---|
| 8 | AUTO | 69,046 | 39/85 | 37/78 |
| 8 | ALTERNATE | 66,314 | 38/81 | 38/81 |
| 8 | CORE_0_1 | 83,175 | 38/80 | 29/60 |
| 8 | **CORE_0_ONLY** | **120,608** | 46/96 | **0/1** |

## 발견 1: 두 번째 코어는 실제로 기여한다

8스레드에서 **단일 코어 48.2 → 두 코어 73.0 inf/s, 1.51배**다.

Core1의 38% 점유율은 장식이 아니라 실제 처리량이었다. 이전 절에서 확인하지 못한 부분을 대조군이 채웠다.

**다만 2배가 아니라 1.51배다.** 코어를 두 배로 늘려도 처리량은 절반만 는다. 코어 밖에 공유 자원이 있다는 뜻이며, 앞 절의 "제출 경로 직렬화" 가설과 부합한다.

`CORE_0_ONLY`의 `run`이 120.6ms로 폭증하는 것도 같은 현상이다. 코어 하나에 8스레드가 몰리니 대기가 그대로 쌓인다.

## 발견 2: 명시 분배는 이득이 거의 없다

```text
4스레드   52.4 -> 57.1   +9.0%
8스레드   72.9 -> 73.0   +0.1%
```

4스레드에서만 개선되고 8스레드에서는 차이가 없다.

게다가 4스레드 개선분을 뜯어보면 `outputs_get`이 13.6 → 10.0ms로 줄어든 것이 대부분이다. 코어 분배 효과인지 측정 노이즈인지 분리되지 않는다.

**`AUTO`의 분배는 이미 균등하다** (8스레드에서 39%/37%). 런타임 스케줄러가 제 역할을 하고 있어 수동 개입의 여지가 없다.

## 발견 3: `CORE_0_1`은 오히려 손해다

```text
8스레드   72.9 -> 64.5   -11.5%
```

모든 스레드에 두 코어를 열어주면 더 느려진다. 컨텍스트마다 코어를 오가며 스케줄링 오버헤드나 캐시 무효화가 발생하는 것으로 보인다.

**예외는 1스레드다.** 여기서만 16.7 → 18.2(+9%)로 이득이고 `run`이 29.7 → 23.7ms로 줄었다.

단일 요청은 두 코어를 함께 쓰는 것이 유리하다. **지연시간이 중요한 워크로드에서 쓸 수 있는 카드**이며, deadline 요청 처리 시 고려할 만하다.

## 결론

```toml
[worker]
worker_count = 8      # 4보다 27% 높음
                      # core_mask 는 설정하지 않는다 (CORE_AUTO)
```

`ALTERNATE`의 4스레드 이득(+9%)은 8스레드에서 사라진다. **8스레드로 가는 편이 코어를 수동 배정하는 것보다 낫고, 코드도 단순해진다** — `rknn_set_core_mask` 호출 자체가 불필요하다.

## 병목 후보가 좁혀졌다

| 후보 | 판정 |
|---|---|
| ~~`CORE_AUTO` 스케줄링 한계~~ | **배제** — 이미 균등 분배, 수동 개입 무의미 |
| ~~두 번째 코어 미사용~~ | **배제** — 1.51배 기여 확인 |
| **코어 밖 공유 자원 직렬화** | **유력** — 2코어인데 1.51배에 그침 |

앞 절 §4의 후보 목록에서 `CORE_AUTO` 항목이 제거된다. 남은 것:

- RKNN runtime 내부 lock
- kernel driver ioctl 직렬화
- IOMMU / buffer mapping 비용
- DDR / memory bandwidth
- output conversion / hidden copy

## 다음

`want_float=0` 이 답이 빠르다. `outputs_get` 이 모든 조건에서 12~15ms로 일정한데, 이것이 역양자화 비용이라면 제거만으로 지연이 줄어든다. 줄지 않으면 그것도 정보다.

---

# want_float 실험 — Claude 결과/의견

- 작성: **2026-08-10 17:15 KST** (커밋 `e0025b4`)
- 측정 노드: `queen`
- 모델: `yolov8n-fp16.rknn` (FP16)
- 원본: `benchmarks/results/2026-08-10-wantfloat/wantfloat-queen.txt`
- 조건: 스레드당 250회, `CORE_AUTO`

## 확인하려던 것

앞선 측정에서 `outputs_get` 이 모든 조건에서 12~15ms로 **거의 변하지 않았다.** 스레드를 8배로 늘려도 그대로였다. 두 가지 해석이 가능했다.

| 해석 | `want_float=0` 에서 예상되는 결과 |
|---|---|
| 역양자화 CPU 비용이 지배적 | `outputs_get` 크게 감소, 처리량 증가 |
| 커널·드라이버 전송 비용이 지배적 | `outputs_get` 소폭 감소, 처리량 변화 없음 |

`rknn_output.want_float` 를 0으로 두면 모델 네이티브 출력을 그대로 받는다.

## 결과

| 스레드 | wf | 처리량 | total | inputs_set | run | **outputs_get** | out bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1 | 16.6 | 60,116 | 17,828 | 29,343 | **12,424** | 1,638,400 |
| 1 | **0** | **17.6** | 56,704 | 19,826 | 28,897 | **7,620** | **819,200** |
| 2 | 1 | 36.6 | 54,668 | 17,604 | 26,781 | **10,071** | 1,638,400 |
| 2 | **0** | **36.9** | 54,204 | 19,370 | 27,890 | **6,783** | **819,200** |
| 4 | 1 | 55.9 | 71,440 | 18,361 | 41,228 | **11,614** | 1,638,400 |
| 4 | **0** | **56.9** | 70,274 | 19,491 | 41,564 | **8,965** | **819,200** |
| 8 | 1 | 73.5 | 108,341 | 23,781 | 68,664 | **15,041** | 1,638,400 |
| 8 | **0** | **77.5** | 102,422 | 27,036 | 66,442 | **8,625** | **819,200** |

시간 단위는 µs, 호출당 평균이다.

## 발견 1: 출력이 정확히 절반이다

```text
want_float=1   1,638,400 bytes   FP32
want_float=0     819,200 bytes   FP16 (모델 네이티브)
```

FP16 모델이므로 네이티브 출력이 FP16(2바이트)인데 `want_float=1` 이 FP32(4바이트)로 확장하고 있었다. **역양자화가 아니라 정밀도 확장이다.**

INT8 모델이라면 1바이트 → 4바이트로 4배 차이가 나므로 효과가 더 클 것으로 예상된다. calibration 데이터 확정 후 재측정한다.

## 발견 2: `outputs_get` 은 확실히 줄어든다

```text
1스레드   12,424 -> 7,620    -39%
2스레드   10,071 -> 6,783    -33%
4스레드   11,614 -> 8,965    -23%
8스레드   15,041 -> 8,625    -43%
```

**8스레드에서 6.4ms 절감**되며, 스레드가 늘어도 `outputs_get` 이 8.6ms 근처에서 안정된다. `want_float=1` 일 때는 15ms까지 증가했다.

## 발견 3: 그런데 처리량 이득은 작다

```text
1스레드   16.6 -> 17.6   +6.0%
2스레드   36.6 -> 36.9   +0.8%
4스레드   55.9 -> 56.9   +1.8%
8스레드   73.5 -> 77.5   +5.4%
```

**호출당 6.4ms를 줄였는데 처리량은 5.4% 늘었다.** 8스레드 기준으로 호출당 총 108ms 중 6% 감소인데, 처리량 증가도 비슷한 수준이다.

CPU 작업을 제거해도 그만큼만 이득이고 배수 효과가 없다. **시스템이 CPU 바운드가 아니라는 증거다.**

## 발견 4: `inputs_set` 이 일관되게 증가한다

예상하지 못한 관측이다.

```text
1스레드   17,828 -> 19,826   +2,000
2스레드   17,604 -> 19,370   +1,766
4스레드   18,361 -> 19,491   +1,130
8스레드   23,781 -> 27,036   +3,255
```

`want_float` 는 출력 경로 설정인데 입력 시간이 늘어난다. 인과관계가 없어야 한다.

**추정: 메모리 할당자 거동 차이.** `want_float=1` 은 1.6MB 출력 버퍼를 할당·해제하고, `want_float=0` 은 0.8MB를 쓴다. `rknn_outputs_release` 가 해제한 블록을 다음 반복의 `rknn_inputs_set` 이 재사용하는데, 크기가 달라 재사용률이 떨어졌을 수 있다.

**확인 필요.** 버퍼 풀을 도입하면(`01-TECHSPEC.md` §15.1-4) 이 변동이 사라지는지 보면 판별된다.

## 결론

**`want_float=0` 을 채택한다.**

- 8스레드에서 처리량 +5.4%, `outputs_get` -43%
- 출력 전송량 절반 → 노드→스케줄러 네트워크 부하도 절반
- YOLOv8 후처리는 어차피 CPU에서 수행하므로 FP16을 직접 다루면 된다

`npuforge-rknn` 구현 시 `want_float=0` 을 기본으로 하고, 후처리 코드가 모델 네이티브 타입을 처리하도록 작성한다. 출력 타입은 `RKNN_QUERY_OUTPUT_ATTR` 로 조회해 분기한다.

## 병목 판정: 두 번째 해석이 맞다

앞의 두 가설 중 **"커널·드라이버 전송 비용이 지배적"** 쪽이다.

역양자화를 완전히 제거해도 `outputs_get` 이 8.6ms 남는다. 그리고 처리량 이득이 제거한 시간에 비례할 뿐 배수가 아니다.

즉 **출력 경로도 같은 공유 자원을 통과한다.** `run` 이 66ms로 여전히 지배적이고 여기가 손대지 못한 구간이다.

이로써 후보가 더 좁혀진다.

| 후보 | 판정 |
|---|---|
| ~~`CORE_AUTO` 스케줄링 한계~~ | 배제 (§4) |
| ~~두 번째 코어 미사용~~ | 배제 (§4) |
| ~~output conversion 비용~~ | **배제** — 제거해도 +5.4% |
| **RKNN runtime lock** | 유력 |
| **kernel driver ioctl 직렬화** | 유력 |
| IOMMU / buffer mapping | 가능 |
| DDR / memory bandwidth | 가능 (발견 4와 관련 가능성) |

## 다음

`perf record` 와 `strace -c` 로 hot path 와 ioctl 횟수를 본다. 지금까지는 블랙박스 밖에서 입출력만 재었으나, 남은 후보들은 그 안을 봐야 구분된다.

`rknn_run` 이 66ms인데 NPU 점유율이 40%라는 것은 **26ms 이상이 NPU 밖에서 소비된다**는 뜻이다. 그 시간이 어디로 가는지가 다음 질문이다.

---

# syscall 분해 — 병목 확정: 드라이버 ioctl 직렬화

> ⚠️ **이 절의 "78 inf/s 는 드라이버 특성" 표현은 8절에서 범위가 좁혀졌다.**
> 추론당 ioctl 횟수는 INT8 에서도 76회로 같은데 처리량은 1.85배다.
> 상한을 정하는 것은 ioctl **횟수**가 아니라 직렬화 구간에서 한 건이
> 붙잡고 있는 **시간**이다. 애플리케이션 최적화로 못 넘는다는 결론 자체는
> 유효하다 — 양자화는 애플리케이션 최적화가 아니라 모델 변경이다.

- 작성: **2026-08-10 17:26 KST** (커밋 `3656401`)
- 측정 노드: `queen`
- 원본: `benchmarks/results/2026-08-10-syscall/`

## 확인하려던 것

`rknn_run` 이 66ms 인데 NPU 점유율은 40% 다. **26ms 이상이 NPU 밖에서 소비된다.** 그 시간의 정체를 찾는다.

`perf` 는 사용할 수 없었다. 보드 커널이 BSP 6.1.141 인데 Ubuntu 저장소의 `linux-tools` 는 6.8.0 용이라 버전이 맞지 않는다. 대신 다음 두 가지를 썼다.

1. bash 내장 `time` — user/sys 분해. 오버헤드 없음
2. `strace -f -c` — syscall 횟수와 소요시간

## 측정 1: 블록 시간이 지배적이다

| 스레드 | real | user | sys | on-CPU 비율 |
|---:|---:|---:|---:|---:|
| 1 | 14.75 | 6.40 | 1.87 | 56% |
| 2 | 13.88 | 12.94 | 2.88 | 57% |
| 4 | 17.90 | 28.76 | 5.23 | 47% |
| 8 | 25.85 | 74.13 | 9.56 | **40%** |

8스레드 호출당 (총 99.9ms):

```text
37.1 ms   유저스페이스 CPU   librknnrt 내부 (주로 inputs_set)
 4.8 ms   커널 CPU           ioctl 처리 자체
58.0 ms   블록               자면서 대기
```

**커널 CPU 가 4.8ms 뿐이라는 것이 중요하다.** ioctl 처리에 CPU 를 태우는 것이 아니다. 스레드가 **자면서 기다린다.**

그런데 NPU 점유율은 40% 다. **NPU 연산을 기다리는 것이 아니다.**

## 측정 2: ioctl 이 직렬화된다

`strace -f -c`, 30회/스레드.

| | 1 스레드 | 8 스레드 | |
|---|---:|---:|---|
| ioctl 호출 수 | 2,419 | 19,072 | |
| **추론당 ioctl** | **80.6** | **79.5** | 동일 |
| **ioctl 당 소요** | **69 µs** | **374 µs** | **5.4배** |
| futex 호출 | 3 | 395 | 무시 가능 |

두 가지가 결정적이다.

**첫째, 추론당 ioctl 횟수가 스레드 수와 무관하게 ~80 으로 일정하다.** 일이 늘어난 것이 아니다.

**둘째, 개당 지연이 5.4배로 늘어난다.** 같은 일을 하는데 8스레드에서 훨씬 오래 걸린다. 드라이버 내부에 배타 구간이 있다는 뜻이다.

**셋째, futex 가 395회뿐이다.** 유저스페이스 락 경합이라면 수만 건이 나왔어야 한다. **librknnrt 내부 락이 아니다.**

## 결론: 커널 드라이버 ioctl 직렬화

```text
80 ioctl × 374 µs ≈ 30 ms
```

블록 시간 58ms 의 절반 이상을 설명한다. 나머지는 실제 NPU 연산 대기와 스케줄링 지연으로 보인다.

병목 후보 목록이 정리되었다.

| 후보 | 판정 |
|---|---|
| ~~`CORE_AUTO` 스케줄링 한계~~ | 배제 (§4) |
| ~~두 번째 코어 미사용~~ | 배제 (§4) |
| ~~output conversion 비용~~ | 배제 (§5) |
| ~~RKNN runtime 내부 lock~~ | **배제** — futex 395회 |
| **kernel driver ioctl 직렬화** | **확정** |
| IOMMU / buffer mapping | ioctl 의 내용일 가능성 높음 |

## 진짜 문제는 ioctl 이 80회라는 것이다

직렬화 자체는 드라이버 구현이므로 우리가 고칠 수 없다. 그러나 **호출 횟수는 줄일 수 있다.**

추론 한 건에 ioctl 80회는 과다하다. `rknn_inputs_set` / `rknn_outputs_get` 이 매 호출마다 버퍼를 할당·매핑·해제하는 것으로 추정된다.

RKNN 은 이를 피하는 **zero-copy 메모리 API** 를 제공한다. 헤더에서 확인했다.

```c
rknn_create_mem(ctx, size)              /* 버퍼를 한 번 할당 */
rknn_set_io_mem(ctx, mem, attr)         /* context 에 바인딩 */
rknn_destroy_mem(ctx, mem)
RKNN_QUERY_NATIVE_INPUT_ATTR  = 8       /* 네이티브 레이아웃 조회 */
RKNN_QUERY_NATIVE_OUTPUT_ATTR = 9
RKNN_FLAG_MEM_ALLOC_OUTSIDE   = 0x10
```

버퍼를 한 번 할당해 재사용하면 per-call 매핑 ioctl 이 사라질 수 있다.

**이것이 현재 가장 유망한 최적화다.** 그리고 `01-TECHSPEC.md` §15.1-6 의 "등록 버퍼 또는 Zero-Copy 검토" 항목과 정확히 일치한다. 다만 문서는 이를 네트워크 경로 최적화로 상정했는데, **실제로 필요한 곳은 NPU 입출력 경로였다.**

## 프로젝트에 주는 의미

이 결과는 발표 서사에 직접 쓰인다.

```text
6 TOPS × 3 = 18 TOPS 가 되지 않는 이유를 찾다 보니
네트워크도 스케줄링도 아니고,
노드 하나 안에서 NPU 에 데이터를 넣고 빼는 경로가 병목이었다.

추론 한 건에 커널 ioctl 이 80회 발생하고,
동시 실행 시 그 지연이 5.4배로 늘어난다.
NPU 는 40% 만 일하고 나머지 시간은 대기다.
```

io_uring 논의와도 연결된다. **네트워크 I/O 를 최적화해도 이 구간은 건드리지 못한다.** 최적화 대상을 데이터로 골라야 한다는 앞선 판단(§3, §5)이 여기서 확인된다.

## 다음

zero-copy 메모리 API 로 ioctl 횟수가 줄어드는지 측정한다.

```text
현재  rknn_inputs_set -> rknn_run -> rknn_outputs_get -> rknn_outputs_release
      추론당 ioctl 80회

목표  rknn_create_mem (1회) -> rknn_set_io_mem (1회)
      -> [rknn_run 반복]
      -> rknn_destroy_mem (1회)
      추론당 ioctl 횟수 감소 기대
```

측정 항목: 추론당 ioctl 횟수, ioctl 당 지연, 처리량, on-CPU 비율.

---

# zero-copy 실험 — 가설 반증

> ⚠️ **이 절의 "78 inf/s 는 드라이버 특성" 표현은 8절에서 범위가 좁혀졌다.**
> 추론당 ioctl 횟수는 INT8 에서도 76회로 같은데 처리량은 1.85배다.
> 상한을 정하는 것은 ioctl **횟수**가 아니라 직렬화 구간에서 한 건이
> 붙잡고 있는 **시간**이다. 애플리케이션 최적화로 못 넘는다는 결론 자체는
> 유효하다 — 양자화는 애플리케이션 최적화가 아니라 모델 변경이다.

- 작성: **2026-08-10 17:44 KST** (커밋 `7a0379b`)
- 측정 노드: `queen`
- 도구: `crates/npuforge-rknn/native/zerocopy_test.c`
- 원본: `benchmarks/results/2026-08-10-zerocopy/`

## 가설

§6에서 추론당 ioctl 이 약 80회이고 8스레드에서 개당 지연이 5.4배로 늘어남을 확인했다. 직렬화는 드라이버 구현이므로 고칠 수 없으나 **호출 횟수는 줄일 수 있다**고 보았다.

`rknn_inputs_set` / `rknn_outputs_get` 이 매 호출마다 버퍼를 할당·매핑·해제한다면, zero-copy 메모리 API 로 버퍼를 재사용해 per-call 매핑 ioctl 을 없앨 수 있다.

```c
rknn_create_mem(ctx, size)          /* 1회 */
rknn_set_io_mem(ctx, mem, attr)     /* 1회 */
  -> [ memcpy + rknn_mem_sync + rknn_run 반복 ]
rknn_destroy_mem(ctx, mem)          /* 1회 */
```

## 결과: ioctl 이 줄지 않았다

| | 추론당 ioctl | ioctl 당 지연 | 8스레드 처리량 |
|---|---:|---:|---:|
| NORMAL | 79.7 | 54 µs | 78.5 inf/s |
| ZEROCOPY | **89.8** | 56 µs | 77.1 inf/s |

**오히려 10회 늘었다.** 내가 추가한 `rknn_mem_sync`(입력 1 + 출력 9 = 10회)가 그 자체로 ioctl 이기 때문이다.

CPU 사용량도 줄지 않았다.

| | real | user | sys | 처리량 |
|---|---:|---:|---:|---:|
| NORMAL | 15.91 | 44.59 | 5.77 | 78.7 |
| ZEROCOPY | 16.38 | 46.48 | 6.64 | 76.7 |

## 구간 시간은 극적으로 이동했다

1스레드 기준이다.

| 구간 | NORMAL | ZEROCOPY |
|---|---:|---:|
| prepare | 20,208 µs | **1,025 µs** |
| run | 28,152 | **58,560** |
| fetch | 8,233 | **778** |
| **총합** | **56,593** | **60,364** |

`prepare` 와 `fetch` 는 95% 가까이 사라졌으나 `run` 이 2배로 늘어 **총합은 오히려 나빠졌다.**

일이 없어진 것이 아니라 `rknn_run` 안으로 옮겨갔을 뿐이다.

## 결론: 80회의 ioctl 은 추론 제출에 내재한다

버퍼 관리 방식과 무관하다. `rknn_run` 자체가 드라이버와 약 80회 주고받는다.

YOLOv8n 이 여러 레이어로 구성되고 드라이버가 태스크 단위로 제출한다면 자연스러운 수치다. **애플리케이션 계층에서 줄일 수 있는 대상이 아니다.**

따라서 §6에서 확인한 처리량 상한(약 78 inf/s)은 **드라이버와 하드웨어의 성질**이며 우리가 우회할 수 없다.

## ⚠️ 이 실험의 공정성 한계

**두 경로가 같은 일을 하지 않았다.**

```text
native_in_bytes = 2,457,600 = 640 × 640 × 3 × 2   ← FP16
모델 입력(uint8)  = 1,228,800
```

네이티브 입력이 **FP16** 이다. NORMAL 경로의 `rknn_inputs_set` 은 uint8 → FP16 변환과 정규화를 수행한다. 그것이 `prepare` 20ms 의 내용이다.

ZEROCOPY 경로에서는 그 변환을 하지 않고 더미 데이터를 `memset` 했다. **변환 비용을 측정에서 제외한 셈이다.**

즉 실제 애플리케이션이라면 그 변환을 직접 해야 하므로, `prepare` 1ms 는 달성 불가능한 수치다.

**그럼에도 결론은 유지된다.** ioctl 횟수와 CPU 사용량이 줄지 않았고, 총 시간도 개선되지 않았기 때문이다. 변환을 추가하면 더 나빠질 뿐이다.

## 그래도 남는 가치

zero-copy 자체는 버릴 카드가 아니다. 다음 조건에서는 유효할 수 있다.

**입력을 애플리케이션이 이미 네이티브 형식으로 만들 수 있을 때.** JPEG 디코딩과 리사이즈를 직접 수행하는 파이프라인에서는 그 출력을 곧바로 FP16 으로 쓰면 중간 변환이 사라진다. 다만 처리량 상한은 여전히 드라이버가 정한다.

**INT8 모델에서 재검토.** 네이티브 입력이 int8 이면 uint8 입력과 크기가 같아 변환이 단순해진다. calibration 데이터 확정 후 다시 측정한다.

## 최적화 후보 정리

| 후보 | 판정 |
|---|---|
| ~~`CORE_AUTO` 스케줄링~~ | 배제 (§4) |
| ~~두 번째 코어 미사용~~ | 배제 (§4) |
| ~~output conversion~~ | 배제 (§5) |
| ~~RKNN runtime lock~~ | 배제 (§6) |
| ~~zero-copy 로 ioctl 감소~~ | **배제 (§7)** |
| kernel driver ioctl 직렬화 | 확정. **우회 불가** |
| INT8 전환 | **미검증. 남은 유일한 큰 카드** |

## 프로젝트에 주는 의미

**노드 하나의 처리량 상한은 약 78 inf/s 이며, 이는 드라이버 특성이다.**

애플리케이션 최적화로 넘을 수 없다는 것이 세 번의 실험(§4 core_mask, §5 want_float, §7 zero-copy)으로 확인되었다. 각각 +0.1%, +5.4%, -1.8% 였다.

이 사실은 확장 효율 측정에 직접 영향을 준다. **노드당 상한이 고정되어 있으므로 3노드 확장 효율은 스케줄링과 네트워크만으로 결정된다.** 노드 내부 최적화 여지가 거의 없다는 것이 오히려 실험 조건을 단순하게 만든다.

발표 서사에도 쓰인다.

```text
NPU 를 더 잘 쓰려고 세 가지를 시도했다.
코어 수동 배정, 출력 변환 제거, zero-copy 버퍼 재사용.
각각 +0.1%, +5.4%, -1.8% 였다.

추론 한 건에 커널 ioctl 이 80회 발생하고 그것이 직렬화되는데,
그 횟수는 애플리케이션이 줄일 수 없었다.

TOPS 수치가 말해주지 않는 것은 연산 능력이 아니라
그 연산기에 데이터를 넣고 빼는 경로의 비용이다.
```

## 다음

**INT8 모델**이 남은 유일한 큰 변수다. FP16 대비 다음이 달라진다.

- NPU 연산 시간 (`run` 의 실제 계산 부분)
- 네이티브 입력이 int8 이면 변환 비용 감소
- 출력 크기 4분의 1

calibration 데이터가 확정되어야 진행할 수 있다. 그전까지는 노드당 78 inf/s 를 기준값으로 삼는다.

---

# INT8 실측 — Claude 결과/의견

- 작성: **2026-08-11 16:45 KST** (커밋 `547333c`)
- 측정 노드: `king`
- 도구: `crates/npuforge-rknn/native/sustained_load_test.c`
- 정확도 원본: `results/accuracy/README.md`

7절 끝에서 "INT8 이 남은 유일한 큰 변수"라고 적었다. 측정했다.

## 결과

`king`, `sustained_load_test`, 8스레드 고정, 120초.

| 모델 | 처리량 | 평균 지연 | 모델 크기 |
|---|---:|---:|---:|
| FP16 | 79.0 inf/s | 100.9 ms | 9.65 MB |
| **INT8** | **146.2 inf/s** | **54.6 ms** | 6.46 MB |
| 배율 | **1.85×** | -46% | -33% |

> 이 수치는 CPU governor 가 `ondemand` 인 상태에서 측정했다.
> `performance` 로 바꾸면 FP16 84.3 / INT8 157.2 inf/s 다(11절).
> **배율 1.86× 는 그대로다.**

애플리케이션 최적화 세 가지(+0.1%, +5.4%, -1.8%)와 자릿수가 다르다.

## 6·7절 결론과 충돌한다

6절은 "추론 한 건에 ioctl 약 80회가 발생하고 그것이 직렬화된다"를 근거로
**노드 상한 78 inf/s 를 드라이버 특성으로 규정**했다. 7절은 그 표현을
그대로 이어받았다.

그런데 INT8 이 1.85배라면, ioctl 횟수가 상한을 정하는 것이 아니다.
확인했다.

```text
strace -c -f -e trace=ioctl, 1스레드 20초

              추론    처리량      평균 지연   ioctl 총계   추론당 ioctl
FP16          315    15.7 inf/s   63.3 ms      24,079      76.4
INT8          718    35.8 inf/s   27.8 ms      54,707      76.2
```

**추론당 ioctl 횟수는 76.4 vs 76.2 로 사실상 같다.** 그런데 처리량은
2.28배다(1스레드 기준). 즉 상한을 정하는 것은 ioctl **횟수**가 아니라
직렬화 구간에서 **한 건이 붙잡고 있는 시간**이다.

## 수정된 모형

두 진술은 모순되지 않는다. 합치면 이렇게 된다.

```text
처리량 ≈ 1 / (직렬화되는 구간의 건당 소요시간)

  - 직렬화가 일어난다는 것        → 6절이 맞다 (ioctl, futex 395회뿐)
  - 8스레드에서 지연이 5.4배로
    늘어나는 것                   → 6절이 맞다 (직렬화의 증거)
  - 그 시간의 크기를 줄이면
    상한이 올라간다               → 8절 (INT8 이 실제 연산량을 줄인다)
```

**애플리케이션 계층에서 못 넘는다는 것**과 **넘을 방법이 없다는 것**은
다르다. core_mask·want_float·zero-copy 는 ioctl 횟수도 건당 연산량도
줄이지 못했다. 양자화는 연산량을 줄인다.

## 표현 정정

6·7절의 다음 문장을 좁힌다.

| 기존 | 정정 |
|---|---|
| "노드 상한 78 inf/s 는 드라이버 특성이다" | "**FP16 기준** 노드 상한은 약 78 inf/s 이고, 이 값은 애플리케이션 최적화로 못 넘는다" |
| "애플리케이션 최적화로 넘을 수 없다" | 유지. 단 **양자화는 애플리케이션 최적화가 아니라 모델 변경**이다 |

## 발표 서사 갱신

7절의 서사에 한 줄이 더 붙는다. 이 편이 더 정직하고 더 유용하다.

```text
NPU 를 더 잘 쓰려고 세 가지를 시도했다.
코어 수동 배정, 출력 변환 제거, zero-copy 버퍼 재사용.
각각 +0.1%, +5.4%, -1.8% 였다.

추론 한 건에 커널 ioctl 이 76회 발생하고 그것이 직렬화된다.
그 횟수는 애플리케이션이 줄일 수 없었다.

그런데 INT8 양자화는 1.85배였다.
ioctl 횟수는 76회로 똑같았다.

줄여야 할 것은 호출 횟수가 아니라
한 번의 호출이 붙잡고 있는 시간이었다.
```

## 정확도 대가

무손실은 아니다. 실보드 검출 수준 비교(`results/accuracy/README.md`).

| 비교 | box cosine | 검출 셀 | 클래스 |
|---|---|---|---|
| FP16 vs ONNX | 0.99999 | 10/10 | 100% |
| INT8 vs FP16 | 0.997 | 10/10 | 100% |

최고 검출의 셀이 한 칸 이동하고 점수가 -5.5% 였다. **검출 집합과 클래스는
동일하다.** 1.85배를 이 정도 대가로 얻는다면 쓸 만하다.

### 정확도 검증에서 걸린 함정

**원시 텐서 코사인 유사도가 이 모델에서는 오해를 부른다.** FP16 vs ONNX
에서도 일부 텐서가 0.16 까지 떨어진다. 양자화 문제가 아니다.

YOLOv8n 출력 9개 중 텐서 2/5/8 은 클래스 점수 80개의 합이다. RKNN 의
sigmoid 는 정확히 0 을 내지 않고 하한 0.001831 이 있어서, 80배 증폭된
0.1465 오프셋이 생긴다(실측 하한과 정확히 일치). 배경 셀이 대부분이라
이 오프셋이 코사인을 지배한다. **모든 셀에 같은 값이 더해지므로 순위는
바뀌지 않고 검출은 그대로다.**

수락 기준을 검출 수준으로 바꿨다. `compare_detections.py`.

**INT8 변환은 바이트 재현성이 없다.** 같은 ONNX·같은 calibration 목록으로
3회 변환하니 해시가 매번 달랐다(파일 크기는 같고 1.8% 바이트 상이).
다만 추론 결과는 완전히 동일하다(9개 텐서 전부 cosine 1.000000, 오차 0.0).
차이는 직렬화·레이아웃에 있고 수치 계산에는 없다.

→ 모델은 한 번만 변환해 같은 파일을 세 노드에 배포한다. `model.toml` 의
  `sha256` 은 배포 무결성을 보장하지 변환 레시피의 동일성을 보장하지 않는다.

## 다음에 확인할 것

- **INT8 + `want_float=0`.** 5절에서 FP16 기준 +5.4% 였다. INT8 은 출력이
  int8 이라 역양자화 비용이 상대적으로 더 클 수 있어 이득이 더 클지 모른다.
- **INT8 의 열 거동.** 연산량이 줄면 발열도 줄어드는지. 8스레드 지속
  부하에서 FP16 대비 온도를 비교한다. 팬리스 조건에서는 이쪽이 더 중요할
  수 있다.
- **3노드 확장 효율에 미치는 영향.** 노드당 상한이 올라가면 네트워크가
  상대적으로 더 빨리 병목이 된다.

  > ⚠️ 여기 처음 적은 1.43 / 4.3 Gbps 는 **틀렸다.** MiB/s 를 Gbps 로
  > 옮기며 2진 접두(÷1024)를 썼다. 네트워크 속도는 10진이다.
  > 올바른 값: `1,228,800 × 157.2 × 8 = 1.545 Gbps/node`, 3노드 4.636 Gbps.
  > FP16 도 3노드 2.486 Gbps 로 2.5GbE 한 링크를 넘는다.
  > `RESULTS.md` §8.1 참조.

  **S2 확장성 실험의 설계를 다시 봐야 한다.**

---

# 공유 컨텍스트 실험 — "오류 0건"은 "정답"이 아니다

- 작성: **2026-08-11 16:45 KST** (커밋 `547333c`, 측정은 `d228cda` 16:33)
- 측정 노드: `king`
- 도구: `crates/npuforge-rknn/native/shared_context_test.c`

`npuforge-rknn` 백엔드를 구현하면서 컨텍스트를 공유할지 스레드마다 둘지
정해야 했다. `environment-matrix.md` §3.1 은 "RKNN Runtime 2.3.0 은
thread-safe" 로 결론이 나 있었다. 그대로 믿으면 컨텍스트 하나로 끝난다.

## 의심한 이유

추론 한 건은 세 번의 호출이다.

```text
rknn_inputs_set  →  rknn_run  →  rknn_outputs_get
```

**개별 호출이 thread-safe 라는 것과 이 시퀀스가 원자적이라는 것은 다르다.**
스레드가 사이에 끼어들면 입력과 출력의 짝이 어긋날 수 있다.

그리고 §3.1 의 측정을 다시 보니 **API 반환 코드만 셌다.** 출력 내용을
대조하지 않았다. 결과가 섞여도 `ok / err` 는 `40 / 0` 으로 나온다.

## 측정

`native/shared_context_test.c`. 스레드마다 다른 입력을 주고, 먼저 단독으로
추론해 기준 출력을 저장한 뒤, 동시 실행 결과가 자기 기준과 같은지 대조한다.

`king`, FP16, 4스레드 × 50회.

| 구성 | API 오류 | **결과 불일치** |
|---|---:|---:|
| 컨텍스트 공유 | 0 | **200 / 200 (100%)** |
| 스레드별 전용 컨텍스트 | 0 | 0 / 200 (0%) |

**공유 컨텍스트는 오류 하나 없이 100% 틀린 답을 낸다.**

## 무엇이 위험했나

이 결함은 다음 성질을 전부 갖는다.

- 예외도 오류 코드도 남기지 않는다
- 단일 스레드 테스트에서는 절대 재현되지 않는다
- 처리량 지표는 오히려 좋아 보인다 (§3.1 에서 2스레드 공유가 34.8 inf/s 로
  전용 33.2 보다 빨랐다 — **틀린 답을 더 빨리 내고 있었다**)
- 검출 결과를 육안으로 봐도 "그럴듯한" 박스가 나온다. 다른 프레임의 결과이지
  쓰레기가 아니기 때문이다

만약 이대로 벤치마크를 돌렸다면, **처리량 수치는 전부 유효하고 검출 결과만
조용히 틀린 상태**로 발표까지 갔을 가능성이 크다.

## 반영

- `environment-matrix.md` §3.1 에 정정 블록을 넣었다. "context 공유" 행의
  처리량 수치는 성능 비교에서 제외한다.
- `RknnContext::infer` 가 `&mut self` 를 받는다. **컴파일러가 동시 호출을
  막는다.** 주석으로 규칙을 적어 두는 것과 타입으로 막는 것은 다르다.
- `ContextPool` 이 `worker_count` 만큼 컨텍스트를 만들어 하나씩 점유한다.
- `supports_concurrent_infer = true` 는 유지하되 근거가 바뀌었다.
  "런타임이 알아서 해준다" 가 아니라 **"백엔드가 풀로 직렬화한다"** 이다.

## 이 프로젝트에 주는 의미

같은 실수를 세 번째로 했다. 3절 「메타」에 두 번을 적어 두었다.

```text
1. RKNN_QUERY_PERF_RUN.run_duration 을 NPU 점유시간으로 읽었다
   → 큐 대기가 포함된 값이었다
2. NPU load 를 delayms=3000 인 채로 0.2초 간격 샘플링했다
   → 3초 평균을 읽고 있었다
3. thread-safety 를 API 반환 코드로만 판정했다
   → 결과 내용을 대조하지 않았다
4. throttling 을 NPU 클럭만으로 판정했다  (12절)
   → 같은 로그에 있던 CPU 클럭이 63~70% 떨어지고 있었다
```

공통점이 분명하다. **지표가 무엇을 세는지 확인하지 않고 이름만 보고 믿었다.**

`preflight-check.sh` 에 넣을 항목이 하나 늘었다.
**성능 측정 전에 정확도부터 확인한다.** 틀린 답을 빨리 내는 구성이
벤치마크에서 이기는 것을 막아야 한다.

---

# 벤치 도구 설계 — 실수를 도구에 박아 넣기

- 작성: **2026-08-11 17:15 KST** (구현 커밋 `b2cae0d`, 17:12)
- 대상: `crates/npuforge-bench/`
- 검증: Mock 3노드 종단 실행

## 왜 이 절을 남기나

`npuforge-bench` 는 새 측정 결과가 아니라 **도구**다. 그런데 이 도구의
설계 근거가 전부 앞 절들의 실패에서 나왔으므로 여기에 남긴다.

지금까지 나온 측정 실수를 모아 보면 성격이 세 가지다.

```text
A. 지표가 무엇을 세는지 확인하지 않았다
   - run_duration 을 NPU 점유시간으로 읽음 (3절)
   - delayms=3000 인 채로 0.2초 샘플링 (3절)
   - thread-safety 를 API 반환 코드로만 판정 (9절)

B. 조건이 달라진 것을 모르고 값을 비교했다
   - 부하 프로파일이 다른 두 측정을 비교해 19°C 격차로 오해
     (board-worklog.md §2.19)
   - 문서의 IP 가 낡아 노드를 사망으로 오판 (§2.20)

C. 무효한 데이터를 유효한 것으로 취급할 뻔했다
   - 어댑터 용량 부족으로 리셋된 보드의 처리량 (§2.17.2)
```

**주석으로 "조심하자"고 적어 두는 것은 통하지 않았다.** 세 번 다 알고
있으면서 당했다. 그래서 도구가 강제하게 했다.

## 도구에 박아 넣은 규칙

| 과거 실수 | 도구가 하는 일 |
|---|---|
| 첫 추론 지연이 튄다 | 예열 요청을 집계에서 제외 |
| 리셋된 보드를 "성능 저하"로 읽음 | `boot_id` 변화 → run 무효 |
| 표본 20건으로 p99 를 냄 | 성공 100건 미만이면 무효 |
| — | 실패를 처리량·노드 몫에서 제외 |
| — | 조건(동시성·시드·정책·노드 수)을 결과에 동봉 |
| — | 백분위는 nearest-rank, 보간 금지 |

### 실패를 처리량에 넣지 않는 이유

넣으면 **노드가 전부 죽었을 때 처리량이 가장 높아진다.** 실패는 즉시
반환되므로 초당 건수가 폭증한다. S4 장애 대응 실험에서 이 지표를 그대로
보면 "장애 시 성능 향상"이라는 결과가 나온다.

노드 몫도 같다. 실패 요청의 `node_id` 는 비어 있는데, 이것을 세면 죽은
노드가 "많이 처리한" 것으로 잡힌다.

### 백분위를 보간하지 않는 이유

선형 보간은 표본이 적을 때 **실제로 관측되지 않은 값**을 만든다.
1~10 에서 p95 를 보간하면 9.55 가 나오는데, 그런 지연을 겪은 요청은 없다.
발표 자료에 "p95 = 9.55ms" 라고 적으면 그것은 측정값이 아니라 계산물이다.

nearest-rank 로 고정하고 정의를 모듈 문서에 박았다.

### 무효 경고를 숫자보다 먼저 출력하는 이유

```text
!!!!!! 이 run 은 무효다 !!!!!!
  - 오류율 100.00% 가 허용치 1.00% 를 넘었다
  - 성공 표본 0건은 최소 100건에 못 미친다
아래 수치를 인용하지 말 것.

요청 : 200 (성공 0 / 실패 200, ...)
```

숫자를 먼저 보여주면 사람은 그것부터 믿는다. 경고를 아래에 두면 스크롤
없이 보이는 첫 화면이 숫자가 되고, 그 숫자가 표에 옮겨 적힌다.

무효 run 을 **삭제하지는 않는다.** 사유와 함께 남아야 원인을 추적할 수
있고, 재부팅이 반복되면 그 자체가 발견이다.

## 구현 중에 잡은 문제 하나

처음에는 노드 상태를 하트비트 RPC 로 조회하려 했다. 스케줄러에 노드
목록 API 가 없었기 때문이다.

**그런데 그것이 스케줄러의 노드 상태를 덮어쓴다.** 하트비트는 관측값을
기록하는 호출이고, 벤치가 빈 `health` 를 보내면 스케줄러가 그것을 실제
관측으로 받아들여 온도·큐 깊이를 0 으로 만든다. **측정 직전에 측정
대상의 상태를 오염시키는** 셈이다.

읽기 전용 `ListNodes` RPC 를 따로 만들었다. 이것도 A 유형(부작용을
확인하지 않고 API 를 씀)의 변종이다.

정책 이름도 스케줄러가 보고한 값을 우선하게 했다. `--policy round-robin`
으로 손으로 적으면 틀리고, **틀린 정책 이름이 붙은 결과는 S3 정책 비교
실험을 통째로 망친다.**

## 도구가 보장하지 않는 것

닫힌 모델(closed loop) 부하다. 동시성 N 을 고정하고 응답을 받은 뒤 다음
요청을 보낸다.

이 방식은 **coordinated omission** 에 취약하다. 시스템이 느려지면
클라이언트도 덩달아 천천히 보내므로 지연 분포가 낙관적으로 나온다.
실제로 느린 요청이 뒤이을 요청의 발사 시각을 미루는데, 그 미뤄진 시간은
어느 요청의 지연에도 계상되지 않는다.

**절대 지연을 SLA 처럼 인용하지 않는다.** 구성 간 비교에만 쓴다.
이 문장을 결과 파일의 `caveats` 에 넣어 결과만 떼어 봐도 알 수 있게 했다.

열린 모델(목표 RPS 고정)을 쓰지 않은 이유는 노드 큐가 유한하기 때문이다.
RPS 를 올리면 금방 `NPF-1303` 거절로 끝나 지연 분포를 볼 수 없다.
두 모델 다 필요하면 M7 에서 추가한다.

## Mock 3노드 확인 결과

```text
요청 : 395 (성공 395 / 실패 0)
처리량: 23.3 inf/s  (17.0초)
재시도: 31건

지연 (왕복, ms)
  min 23.7  p50 45.1  p90 256.9  p95 302.8  p99 3092.9  max 3214.4

노드별 분배
  mock-01     160건   40.5%   p50    30.3 ms  p99  3059.0 ms
  mock-02     157건   39.7%   p50   220.8 ms  p99  3178.3 ms
  mock-03      78건   19.7%   p50    30.0 ms  p99    75.1 ms
```

**p99 가 3.09초인 것은 버그가 아니다.** Mock 노드의
`queue_timeout_ms = 3000` 에 걸린 요청이 `NPF-1303` 으로 거절되고
스케줄러가 다른 노드로 재시도해 성공한 것이다(재시도 31건). 동시성 6 에
`worker_count = 1` 인 Mock 이라 큐가 쌓인다.

즉 이 수치는 **재시도 경로가 실제로 동작한다는 증거**다. 실장비에서는
`worker_count = 8` 이므로 다른 그림이 나온다.

종료 코드로 야간 자동 실행을 지원한다.

```text
0  유효
3  무효   ← 스크립트가 이것으로 재실행 여부를 판단한다
2  인자 오류
1  실행 실패
```

## 다음

M3 실장비 측정(1/2/3노드 확장 효율)은 **10G aggregation 구성이 있어야** 시작한다.

8절에서 계산했듯 INT8 노드 하나가 **1.545 Gbps** 를 요구한다(8절의 1.43 은
2진 접두를 쓴 오류로 정정되었다). 현재 관리망
1GbE 로는 **노드 한 대분도 받지 못한다.** 지금 측정하면 네트워크 병목을
확장 효율로 잘못 보고하게 된다 — B 유형 실수를 그대로 반복하는 것이다.

스위치 도착 전까지는 Prometheus 메트릭, `preflight-check.sh`,
`dealer` NTP 서버 구성을 진행한다.

---

# CPU governor 영향 — 기존 수치는 전부 `ondemand` 기준이었다

- 작성: **2026-08-12 10:16 KST**
- 측정 노드: `king` (동일 조건 재측정)
- 계기: `preflight-check.sh` 가 `ondemand` 를 하드 실패로 막았고, 그래서 바꿨다

## 결과

같은 도구·같은 조건(8스레드, 120초, `king`)에서 governor 만 바꿨다.

| 모델 | `ondemand` | `performance` | 변화 |
|---|---:|---:|---:|
| FP16 | 79.0 inf/s | **84.3 inf/s** | **+6.7%** |
| INT8 | 146.2 inf/s | **157.2 inf/s** | **+7.5%** |

평균 지연도 함께 줄었다 (FP16 100.9 → 94.5 ms, INT8 54.6 → 50.8 ms).

## 왜 CPU governor 가 NPU 처리량을 바꾸나

추론 한 건은 NPU 실행만이 아니다.

```text
입력 설정(CPU) → NPU 실행 → 출력 취득·역양자화(CPU)
```

3절에서 8스레드가 2스레드보다 빠른 이유로 이미 확인한 구조다. 한 스레드가
CPU 구간에 있는 동안 다른 스레드가 NPU 를 점유하는 파이프라이닝이 일어난다.
**그 CPU 구간의 속도가 전체 처리량에 직접 반영된다.**

`ondemand` 는 부하에 따라 주파수를 올리는데, NPU 대기 중에는 CPU 사용률이
낮아 보여 주파수가 내려간다. 그 상태에서 다음 요청의 CPU 구간이 시작되면
느린 클럭으로 시작한다. 관측된 유휴 클럭이 1008~1800MHz 로 흔들린 것이
이것이다(최대는 A53 2016 / A72 2208MHz).

## 이 결과가 뜻하는 것

**8절까지의 모든 처리량 수치는 `ondemand` 기준이다.** 앞으로 나올 수치와
직접 비교하면 안 된다. 문서의 확정 수치를 `performance` 기준으로 갱신했다.

다만 **결론은 바뀌지 않는다.**

| 결론 | 근거 |
|---|---|
| INT8 이 FP16 대비 1.85배 | `performance` 에서 157.2/84.3 = **1.86배**. 그대로다 |
| 애플리케이션 최적화 3종이 무의미 | governor 는 애플리케이션 최적화가 아니다 |
| 상한은 직렬화 구간의 건당 시간 | CPU 구간이 그 시간의 일부라는 것이 오히려 보강된다 |

8절에서 "상한을 정하는 것은 ioctl 횟수가 아니라 한 건이 붙잡고 있는
시간"이라고 썼는데, 이번 결과가 그 시간에 **CPU 전후처리도 포함된다**는
것을 보여준다. ioctl 횟수는 governor 와 무관하게 76회로 같을 것이다.

## 조치

`scripts/set-cpu-governor.sh` 로 세 노드를 `performance` 로 고정하고
systemd 유닛으로 영구화했다.

재부팅 유지를 실제로 확인했다. `jack` 을 재부팅해 `boot_id` 가
`6caea6bd → 83d2981f` 로 바뀐 뒤에도 governor 가 유지되었다.

`cpufrequtils` 패키지를 쓰지 않았다. 설치하면 세 노드의 패키지 목록이
달라져 환경 일치가 깨진다.

## 유휴 온도는 거의 안 올랐다

| 노드 | `ondemand` | `performance` |
|---|---:|---:|
| king | 36.1°C | 37.0°C |
| queen | 35.2°C | 36.1°C |

클럭은 항상 최대지만 유휴 코어는 여전히 halt 상태라 발열이 늘지 않는다.
**팬리스 S0 측정에 부담을 주지 않는다.**

## 남은 함정 하나

`set-cpu-governor.sh` 를 만들 때 ssh 안에서 heredoc 과 sudo 를 중첩했다가
**유닛 파일이 아예 생기지 않았는데 종료 코드는 0** 이었다. 스크립트는
"적용 실패"를 보고했지만, 값 확인 단계를 넣지 않았다면 "영구화 완료"로
넘어갔을 것이다.

유닛 파일을 로컬에서 만들어 `scp` 로 전송하는 방식으로 바꿨다.
board-worklog.md §2.21 의 원격 실행 함정과 같은 계열이다.

---

# want_float=0 전환과 CPU throttling — Claude 결과/의견

- 작성: **2026-08-12 17:40 KST**
- 측정 노드: `king`
- 계기: 네트워크 계산에서 `want_float=0` 이 M3 전제 조건으로 승격됨

## 1. want_float=0 의 처리량 효과 (미측정이었던 것)

5절의 `+5.4%` 는 FP16 에서 잰 값이라 INT8 에 옮길 수 없다고 적어 두었다.
측정했다. `king`, 8스레드, 120초.

| 모델 | `want_float=0` | `want_float=1` | 이득 |
|---|---:|---:|---:|
| INT8 | **156.7 inf/s** | 133.6 inf/s | **+17.3%** |
| FP16 | 66.9 inf/s | 57.8 inf/s | **+15.7%** |

5절의 +5.4% 보다 훨씬 크다. 5절은 1스레드 위주 조건이었고, 여기서는
8스레드 동시 실행이라 출력 변환이 직렬화 구간을 더 오래 붙잡는다.

**네트워크와 처리량이 같은 방향을 가리킨다.** 출력이 4분의 1이 되고
처리량이 15~17% 오른다. `want_float=0` 을 안 쓸 이유가 없다.

> 그리고 이제야 알았는데, **`sustained_load_test` 는 처음부터
> `want_float=0` 을 하드코딩하고 있었다.** 즉 문서의 157.2 / 84.3 은
> 이미 `want_float=0` 기준이었다. Rust 백엔드만 `true` 였으므로,
> 이번 전환은 **소프트웨어를 측정 조건에 맞춘 것**이다.

## 2. 그런데 더 큰 것이 나왔다 — CPU 가 열로 꺾인다

FP16 을 다시 재는데 값이 84.3 이 아니라 66.9 로 나왔다. INT8 은
156.7 로 일치했다. FP16 만 다른 것이 이상해 확인했다.

**원인은 측정 순서였다.** FP16 측정이 INT8 측정 두 번 뒤에 붙어 있었다.

| 시작 온도 | FP16 처리량 |
|---|---:|
| 53.6°C (냉각 후) | **81.6 inf/s** |
| 71.2°C (연속 측정 중) | 66.9 inf/s |

**-18%.** 그래서 부하 중 클럭을 직접 관찰했다. governor 는 `performance` 다.

```text
        NPU온도   npu_clk   cpu4(A72)   cpu0(A53)
 +15s   86.8°C    950 MHz   2208 MHz    2016 MHz
 +30s   90.4°C    950 MHz   1416 MHz    1200 MHz
 +45s   89.5°C    950 MHz   1008 MHz     816 MHz
 +60s   87.8°C    950 MHz    816 MHz     600 MHz
+120s   87.8°C    950 MHz    816 MHz     600 MHz
```

**NPU 클럭은 950 MHz 에서 한 번도 안 떨어진다. CPU 가 63~70% 떨어진다.**

300초 지속 측정에서 처리량이 이렇게 수렴한다.

```text
 +10s  81.6 inf/s   ← 시작
+120s  63.6
+300s  59.7         ← 정상 상태
평균   71.3 inf/s
```

**시작 대비 -27%.**

## 3. 이것이 뒤집는 것

### 3.1 "throttling 없음" 은 틀렸다

`RESULTS.md` §2.3 과 `environment-matrix.md` §9.0 에 이렇게 적혀 있다.

> throttling 없음 — 928 샘플 전부 NPU 950MHz, 한 번도 안 떨어짐

**NPU 클럭만 봤다.** CPU 클럭도 같은 로그에 기록되어 있었는데 판정에
쓰지 않았다. 추론 한 건은 `입력 설정(CPU) → NPU → 출력 취득(CPU)` 이고
CPU 구간이 처리량에 직접 반영된다는 것을 11절에서 이미 확인했으면서도,
throttling 판정은 NPU 만으로 했다.

**이것이 같은 유형의 네 번째 실수다.** 3절 「메타」 목록에 추가한다.

```text
1. run_duration 을 NPU 점유시간으로 읽음      → 큐 대기 포함
2. NPU load 를 delayms=3000 인 채로 샘플링    → 3초 평균
3. thread-safety 를 API 반환 코드로만 판정    → 결과 미대조
4. throttling 을 NPU 클럭만으로 판정          → CPU 가 꺾이고 있었다
```

### 3.2 CPU governor 결론의 범위가 좁아진다

11절의 **+7%** 는 120초 측정이다. 그 구간은 아직 CPU 가 완전히 강등되기
전이다. **지속 부하에서는 `performance` 가 더 유리하다고 단정할 수 없다.**

`performance` 는 유휴에도 최대 클럭을 유지하므로 부하 시작 시점의 열
여유가 적다. 더 빨리 뜨거워지고 더 일찍 강등될 수 있다.

**측정하지 않았다.** `ondemand` 와 `performance` 를 동일한 300초 조건에서
비교해야 한다. 그전까지 11절의 +7% 는 **"짧은 측정에서의 이득"** 으로만
읽는다.

### 3.3 Peak vs Sustained 격차가 이 프로젝트의 핵심 수치가 된다

지금까지 "Peak vs Sustained 약 10%" 로 적어 두었다. 이번 측정은
**300초에 -27%** 다. 원인이 NPU 가 아니라 **CPU thermal throttling** 이라는
것까지 짚었다.

> 벤더가 공개하는 TOPS 는 순간 성능이다. 팬리스 엣지에서 실제로 무엇이
> 먼저 무너지는가 — **NPU 가 아니라 그 앞뒤를 처리하는 CPU 였다.**

발표 서사로서 이쪽이 훨씬 낫다. S0 을 제대로 돌리면 확정 수치가 된다.

## 4. 조치

**한 것**

- `want_float` 을 노드 설정(`[worker] want_float`)으로 노출하고 기본값을
  `false` 로 바꿨다. Rust 백엔드가 측정 도구와 같은 조건이 되었다
- blob 형식을 **v2** 로 올려 텐서마다 `qnt_type`·`zero_point`·`scale` 을
  싣는다. 이것 없이 int8 을 보내면 받는 쪽이 해석할 수 없다
- 실보드에서 역양자화가 float32 와 일치함을 확인했다
  (텐서 9개, **최대 오차 9.5e-7** — float32 정밀도 한계)

**해야 할 것**

- `ondemand` vs `performance` 를 동일한 300초 조건에서 비교
- S0 를 30분으로 돌려 정상 상태 처리량과 강등 시점을 확정
- 열 판정에 **CPU 클럭을 포함**하도록 `run-thermal-comparison.sh` 수정
- `RESULTS.md` §2.3 과 `environment-matrix.md` §9.0 의 "throttling 없음"
  표현 정정 (이번 커밋에 포함)

---

<a id="board-worklog"></a>

# NPUDure 보드 작업 로그

- 문서명: `board-worklog.md`
- 대상: NanoPi R76S × 3 (`king` / `queen` / `jack`)
- 목적: 보드에 가한 모든 변경을 시간순으로 기록한다

---

# 0. 이 문서의 규칙

보드에 실행한 명령과 그 결과를 **시간순으로 append**한다. 기존 항목은 수정하지 않는다.

기록하는 이유는 세 가지다.

1. **재현성.** 보드를 다시 세팅하거나 네 번째 노드를 추가할 때 이 문서만 따라가면 된다.
2. **원인 추적.** 벤치마크 결과가 노드마다 다르게 나올 때, 세 보드에 무엇이 다르게 적용됐는지 여기서 확인한다.
3. **오픈소스 공개.** 외부 사용자가 같은 환경을 만들 수 있어야 한다.

각 항목에 다음을 남긴다.

```text
날짜 / 대상 노드 / 실행한 명령 / 결과 / 판단 근거
```

**되돌릴 수 없는 변경**(패키지 업그레이드, 커널 교체, 파티션 조작)은 실행 전에 별도로 표시하고 승인 여부를 기록한다.

---

# 1. 노드 명칭

물리 보드에 붙인 라벨을 그대로 사용한다.

| 라벨 | hostname | Node ID | 관리망 IP | SSH 별칭 |
|---|---|---|---|---|
| K | `king` | `king` | `192.168.123.12` | `npuforge-k` |
| Q | `queen` | `queen` | `192.168.123.16` | `npuforge-q` |
| J | `jack` | `jack` | `192.168.123.33` | `npuforge-j` |

Scheduler 호스트(개발 PC): `192.168.123.26`

---

# 2. 2026-08-07

## 2.1 SSH 접속 확보

**상황.** 세 보드가 모두 `192.168.123.0/24`로 이동해 PC(`192.168.123.26`)와 같은 대역이 되었다. 세 대 모두 ping 및 tcp/22 응답 확인.

**문제.** `ssh-copy-id`가 3대 모두에서 즉시 실패했다.

```text
Permission denied, please try again.   (호스트당 2회, 3대 모두 즉시)
```

**원인.** 비밀번호가 틀린 것이 아니라 **TTY가 없었다.** SSH는 비밀번호를 stdin이 아니라 제어 터미널(`/dev/tty`)에서 읽는다. 자동화 환경에는 TTY가 없으므로 프롬프트가 뜨지 못하고 즉시 EOF로 실패했다. 호스트당 정확히 2회씩, 3대가 한 번에 끝난 패턴이 근거다.

**조치.** OpenSSH 9.7의 `SSH_ASKPASS_REQUIRE=force`를 사용해 TTY 없이 비밀번호를 전달했다.

```bash
ASKPASS=$(mktemp)
printf '#!/bin/sh\nprintf "%%s\\n" "$NPUFORGE_SUDO_PASS"\n' > "$ASKPASS"; chmod 700 "$ASKPASS"
SSH_ASKPASS="$ASKPASS" SSH_ASKPASS_REQUIRE=force DISPLAY=dummy \
  ssh-copy-id -i ~/.ssh/id_ed25519_npuforge.pub npuforge-k
```

작업 후 헬퍼 파일은 `shred -u`로 삭제했다.

**결과.** 3대 모두 키 인증 성공. 계정은 `pi`.

**PC 측 설정.**

- 전용 키 생성: `~/.ssh/id_ed25519_npuforge` (passphrase 없음, 자동화용)
- `~/.ssh/config`에 `npuforge-k` / `npuforge-q` / `npuforge-j` 별칭 추가

> 이 키는 자동화 전용이며 passphrase가 없다. 외부 공개 저장소나 신뢰할 수 없는 네트워크에 노출되지 않도록 한다.

## 2.2 하드웨어 실측 수집

**명령.** `scripts/collect-node-info.sh`를 3대에 원격 실행.

```bash
for pair in "k:npuforge-k" "q:npuforge-q" "j:npuforge-j"; do
  name="${pair%%:*}"; host="${pair##*:}"
  ssh "$host" 'bash -s' < scripts/collect-node-info.sh > "benchmarks/node-info/${name}.txt"
done
```

**원본.** `benchmarks/node-info/{k,q,j}.txt` (각 66줄)

**확정된 스펙.** 상세는 `environment-matrix.md` §2.1 참조.

```text
보드    FriendlyElec NanoPi R76S / friendlyelec,nanopi-r76s rockchip,rk3576
CPU     8코어 — little 2.016GHz(policy0) + big 2.208GHz(policy4)
RAM     4GB LPDDR4X (3,997,848 kB)
eMMC    64GB (rootfs 50G 여유)
NPU     2코어 (Core0, Core1), 300~950MHz, IOMMU 활성
        RKNPU driver v0.9.8
RKNN    Runtime 2.3.0 (c949ad889d@2024-11-07T11:35:33)
        librknnrt.so SHA-256 3대 동일
OS      Ubuntu 24.04, 커널 6.1.141, glibc 2.39
열센서  6개 (soc / bigcore / little-core / ddr / npu / gpu)
```

**중요.** NPU가 **2코어**다. RK3588은 3코어이므로 RK3588 기준 `core_mask` 예제를 그대로 쓸 수 없다.

`rknn_api.h`의 `rknn_core_mask` enum은 코어 3개까지 정의하지만(`RKNN_NPU_CORE_2`), RK3576에서 실제 사용 가능한 것은 `CORE_0`, `CORE_1`, `CORE_0_1`, `CORE_AUTO`, `CORE_ALL`이다.

## 2.3 NIC 스펙 확인

**배경.** 초기 수집에서 `eth1`이 `speed=1000`으로 나와 1G 포트로 오인할 소지가 있었다.

**명령.**

```bash
sudo apt-get install -y ethtool
sudo ethtool -i eth0 ; sudo ethtool eth0
sudo ethtool -i eth1 ; sudo ethtool eth1
```

**결과. 두 포트 모두 2.5G다.**

| 항목 | eth0 | eth1 |
|---|---|---|
| 드라이버 | `r8125` 9.010.01-NAPI | `r8125` 9.010.01-NAPI |
| PCIe 버스 | `0001:21:00.0` | `0000:01:00.0` |
| Supported link modes | 10/100/1000/**2500** baseT | 10/100/1000/**2500** baseT |
| 현재 링크 | 없음 (down) | 1000Mb/s Full |

`eth1`이 1000Mb/s인 것은 **1G 허브에 연결되어 협상된 결과**이지 포트 성능 한계가 아니다.

두 포트가 서로 다른 PCIe 버스에 있어 대역폭을 공유하지 않는다. 관리망/추론망 분리에 유리하다.

**결정.**

```text
eth1 → 관리망 (현재 1G 허브, 192.168.123.0/24)
eth0 → 추론망 (2.5G 스위치 도입 시, 10.20.0.0/24)
```

`eth0`이 3대 모두 비어 있으므로 추론망 전용으로 그대로 사용한다.

## 2.4 hostname 변경

**변경 전.**

| 노드 | hostname |
|---|---|
| K | `NanoPi-R76S` |
| Q | `NanoPi-R76S` |
| J | `localhost.localdomain` |

K와 Q가 동일해 로그·대시보드에서 구분이 불가능했다.

**명령.**

```bash
sudo hostnamectl set-hostname <king|queen|jack>
sudo sed -i "s/^127\.0\.1\.1.*/127.0.1.1\t<new>/" /etc/hosts
```

**결과.** `king` / `queen` / `jack` 로 변경 완료.

### 부수 발견: jack의 `/etc/hosts`가 비어 있었다

`jack`은 `/etc/hosts`가 **0바이트**였다. hostname이 `localhost.localdomain`이었던 원인이다.

king의 파일을 참조본으로 삼아 동일 내용으로 복원했다.

```text
127.0.0.1	localhost
::1		localhost ip6-localhost ip6-loopback
ff02::1		ip6-allnodes
ff02::2		ip6-allrouters

127.0.1.1	jack
```

**판단.** 세 보드는 **완전한 동일 복제본이 아니다.** `/etc/hosts` 부재와 Ubuntu 패치 레벨 차이(§2.5)가 함께 나타난 것으로 보아, jack은 다른 시점 또는 다른 경로로 세팅되었을 가능성이 있다.

### 작업 중 발견한 스크립트 함정

sudo 비밀번호를 파이프로 넘기는 헬퍼 함수에 파일 내용을 다시 파이프로 넣으면 충돌한다.

```bash
S() { printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" "$@"; }

printf "text\n" | S tee -a /etc/hosts    # 동작하지 않음
```

`sudo -S`가 stdin의 첫 줄을 비밀번호로 소비하므로 뒤따르는 명령은 EOF를 받는다. 파일을 쓸 때는 다음을 사용한다.

```bash
cat > /tmp/file.new <<'EOF'
...
EOF
printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" cp /tmp/file.new /etc/target
```

## 2.5 발견된 노드 간 불일치 (미해결)

세 노드는 "동일 OS 이미지"여야 한다(`02-HARDWARE-SETUP.md` §5.1). 현재 다음이 어긋나 있다.

| # | 항목 | king | queen | jack | 위험 |
|---|---|---|---|---|---|
| 1 | Ubuntu 패치 레벨 | 24.04.**3** | 24.04.4 | 24.04.4 | 라이브러리 차이가 노드별 성능 편차로 나타남 |
| 2 | gcc | `~24.04` | `~24.04.1` | `~24.04.1` | 위와 동일 |
| 3 | 미적용 업데이트 | 374개 | 280개 | 279개 | 위와 동일 |
| 4 | SSH 호스트 키 | 3대 완전 동일 (`<redacted-fingerprint>`) | | | 노드 식별 불가, MITM 탐지 불가 |
| 5 | CPU Governor | `ondemand` | `ondemand` | `ondemand` | 주파수 변동으로 측정 재현성 저하 |

**일치하는 항목** (문제 없음): 커널 6.1.141, glibc 2.39, Python 3.12.3, RKNN Runtime 2.3.0 및 `librknnrt.so` SHA-256, RKNPU driver v0.9.8, NPU 2코어, RAM 4GB, eMMC 64GB.

### ⚠️ 커널 업그레이드 금지

커널 `6.1.141`은 FriendlyElec BSP 커널이며 **RKNPU 드라이버 v0.9.8이 여기에 묶여 있다.**

`apt upgrade`가 커널을 교체하면 NPU가 동작하지 않을 수 있다. 패키지 동기화 시 반드시 커널 관련 패키지를 hold 한다.

```bash
sudo apt-mark hold linux-image-* linux-headers-* linux-modules-*
```

이 작업은 되돌리기 번거로우므로 **승인 후 실행**한다. 현재 미실행.

## 2.6 보드 소프트웨어 현황

| 항목 | 상태 |
|---|---|
| `librknnrt.so` | `/usr/lib/librknnrt.so` (2.3.0) |
| `rknn_api.h` | `/usr/include/rknn_api.h` |
| `rknn_matmul_api.h` | 설치됨 |
| `rknn_custom_op.h` | 설치됨 |
| `rknn_server` | `/usr/bin/rknn_server` (Toolkit2 연결 디버깅용) |
| `.rknn` 모델 파일 | **없음** — 변환 필요 |
| gcc | 13.3.0 |
| rustc | 미설치 (크로스컴파일 사용하므로 정상) |
| ethtool | 2026-08-07 설치 (king만. queen/jack 미설치) |

`rknn_server`가 있으므로 RKNN-Toolkit2의 연결 모드로 PC에서 보드의 NPU를 직접 호출해 모델을 검증할 수 있다.

---

## 2.7 C Wrapper 실기 검증

**배경.** `crates/npuforge-rknn/native/rknn_wrapper.c`는 RKNN API 문서만 보고 작성했고 실장비 검증 전이었다. 파일 상단에 그 사실을 명시해 두었다.

**검증 방법.** 실제 `rknn_api.h`의 시그니처를 추출해 대조한 뒤, 보드에서 직접 컴파일했다.

```bash
scp crates/npuforge-rknn/native/rknn_wrapper.{c,h} npuforge-k:~/npuforge-rknn-test/
ssh npuforge-k 'cd ~/npuforge-rknn-test && gcc -c -Wall -Wextra -O2 rknn_wrapper.c -o rknn_wrapper.o'
```

**결과. 경고 없이 컴파일 성공.** 작성한 시그니처가 실제 헤더와 일치했다.

| 항목 | 확인 결과 |
|---|---|
| `rknn_init(rknn_context*, void*, uint32_t, uint32_t, rknn_init_extend*)` | 일치 |
| `rknn_query(rknn_context, rknn_query_cmd, void*, uint32_t)` | 일치 |
| `rknn_inputs_set(rknn_context, uint32_t, rknn_input[])` | 일치 |
| `rknn_run(rknn_context, rknn_run_extend*)` | 일치 |
| `rknn_outputs_get(rknn_context, uint32_t, rknn_output[], rknn_output_extend*)` | 일치 |
| `rknn_outputs_release(rknn_context, uint32_t, rknn_output[])` | 일치 |
| `rknn_input` 필드 (`index/buf/size/pass_through/type/fmt`) | 일치 |
| `rknn_output` 필드 (`want_float/is_prealloc/index/buf/size`) | 일치 |
| `rknn_sdk_version` (`api_version[256]`, `drv_version[256]`) | 일치 |
| `rknn_context` | `uint64_t` (aarch64) |
| `RKNN_SUCC` | 0 |

**추가 확인.** `rknn_set_core_mask(rknn_context, rknn_core_mask)`가 존재한다. `rknn_core_mask` enum은 코어 3개까지 정의하지만 RK3576은 2코어이므로 `CORE_0`, `CORE_1`, `CORE_0_1`, `CORE_AUTO`, `CORE_ALL`만 유효하다.

**미해결.** `npf_rknn_get_runtime_version()`은 컨텍스트 없이 `rknn_query`를 호출하도록 작성했는데, 이 호출이 실제로 성공하는지는 모델이 있어야 확인 가능하다. 실패한다면 노드 시작 시 임시 컨텍스트를 만들어 조회한 뒤 캐시하는 방식으로 바꾼다.

## 2.8 Thread-safety 테스트 프로그램 작성

**파일.** `crates/npuforge-rknn/native/thread_safety_test.c`

**빌드 확인.**

```bash
gcc -O2 -Wall -Wextra -o thread_safety_test thread_safety_test.c -lrknnrt -lpthread
# 경고 없이 성공, 71,888 bytes
```

**검증 시나리오.**

| # | 구성 | 확인 대상 |
|---|---|---|
| 기준선 | 스레드 1, 전용 context | 단일 스레드 처리량 |
| 1 | 스레드 2, **context 공유** | 동일 context 동시 호출 가능 여부 |
| 2 | 스레드 2, 각자 전용 context, `CORE_AUTO` | 전용 context 병렬 가능 여부 |
| 3 | 스레드 2, 각자 전용 context, `CORE_0` / `CORE_1` 분리 | 명시적 코어 분리 효과 |
| 4 | 스레드 4 (코어 수 2 초과) | 과다 워커의 역효과 |

**판정 기준.**

```text
시나리오 1에서 err > 0        → 동일 context 동시 호출 불가
                                모델당 전용 워커 스레드로 직렬화 필요
시나리오 2가 기준선 대비 ~2배 → 전용 context 로 2-way 병렬 가능, worker_count = 2
시나리오 2가 ~1배             → 런타임 내부 직렬화, worker_count = 1 유지
시나리오 3 > 시나리오 2       → 명시적 코어 분리가 유효
시나리오 4 < 시나리오 2       → 코어 수를 넘는 워커는 역효과
```

**⛔ 실행 보류. 모델 파일이 없다.**

보드에 `.rknn` 파일이 하나도 없다. 프로그램은 준비 완료 상태이며, 모델이 생기는 즉시 실행하면 된다.

```bash
ssh npuforge-k 'cd ~/npuforge-rknn-test && ./thread_safety_test model.rknn 50'
```

### 모델 확보 경로

| 경로 | 가능 여부 | 비고 |
|---|---|---|
| 보드에서 다운로드 | ✗ | `curl`, `wget` 미설치 |
| 보드에서 변환 | ✗ | RKNN-Toolkit2는 x86_64 Linux 전용 |
| **PC WSL2에서 변환** | **✓** | WSL2 Ubuntu 확인됨 (현재 Stopped) |
| `rknn_server` 연결 모드 | ✓ | Toolkit2가 PC에서 보드 NPU를 직접 호출 |

`rknn_server`가 보드에 설치되어 있으므로, Toolkit2 구축 후 PC에서 보드 NPU를 원격 호출해 모델을 즉석 검증할 수 있다.

## 2.9 노드 일치 스크립트 준비 (미실행)

**파일.** `scripts/fix-node-consistency.sh`

기본 동작이 DRY RUN이며 `--apply`를 줘야 실제로 실행된다. `--only`로 단계를 나눠 실행할 수 있다.

| 단계 | `--only` 값 | 내용 | 위험도 |
|---|---|---|---|
| 1 | `kernelhold` | 커널 패키지 `apt-mark hold` | 낮음 |
| 2 | `hostkeys` | SSH 호스트 키 재생성 + PC `known_hosts` 정리 | 낮음 |
| 3 | `packages` | 기본 패키지 설치 (curl, ethtool, iperf3, chrony 등) | 낮음 |
| 4 | `chrony` | 시간 동기화 활성화 | 낮음 |
| 5 | `upgrade` | 패키지 업그레이드 (24.04.3 → 24.04.4) | **높음** |
| 6 | `governor` | CPU Governor → `performance` | 중간 |

**안전 장치.**

- 5단계는 커널 hold 여부를 먼저 확인하고, hold되지 않았으면 중단한다
- 6단계는 발열이 올라가므로 S0 열 특성 측정 후 적용을 권장한다
- DRY RUN으로 3대 접속 및 단계 출력 확인 완료

**권장 실행 순서.**

```bash
./scripts/fix-node-consistency.sh --apply --only kernelhold
./scripts/fix-node-consistency.sh --apply --only hostkeys
./scripts/fix-node-consistency.sh --apply --only packages,chrony
./scripts/fix-node-consistency.sh --apply --only upgrade     # 단독 실행
# (S0 측정 후)
./scripts/fix-node-consistency.sh --apply --only governor
```

업그레이드 후 반드시 확인할 것:

```bash
ssh npuforge-k 'uname -r'                                              # 6.1.141 유지?
ssh npuforge-k 'printf "$NPUFORGE_SUDO_PASS\n" | sudo -S cat /sys/kernel/debug/rknpu/version'  # NPU 살아있나?
ssh npuforge-k 'sha256sum /usr/lib/librknnrt.so'                       # 73993ed4... 유지?
```

---

# 3. 미완료 작업

| # | 작업 | 상태 | 비고 |
|---|---|---|---|
| 1 | RKNN thread-safety 검증 | 진행 예정 | `worker_count` 결정. 모델 파일 필요 |
| 2 | `rknn_wrapper.c`를 실제 헤더로 검증 | 진행 예정 | 미검증 상태로 작성됨 |
| 3 | 노드 간 불일치 해소 (§2.5) | 스크립트 준비 후 승인 대기 | 커널 hold 필수 |
| 4 | SSH 호스트 키 재생성 | 스크립트 준비 후 승인 대기 | |
| 5 | CPU Governor → `performance` | 벤치마크 직전 적용 | |
| 6 | 기본 패키지 설치 | 미실행 | `02-HARDWARE-SETUP.md` §5.2 |
| 7 | 추론망 구성 (`eth0`, 10.20.0.0/24) | 2.5G 스위치 도입 후 | |
| 8 | 모델 변환 환경 구축 | 미실행 | Toolkit2를 Runtime 2.3.0에 맞출 것 |

## 3.1 다음 단계: 모델 확보

thread-safety 검증(1번)이 모델 파일에 막혀 있고, 모델은 다른 모든 실장비 작업의 전제이기도 하다. 따라서 이것이 최우선이다.

```text
PC WSL2 (Ubuntu, 현재 Stopped)
  → rknn-toolkit2==2.3.0 설치        ← Runtime 2.3.0에 맞춤
  → YOLOv8n ONNX 확보
  → rknn.config(target_platform='rk3576')
  → yolov8n.rknn 생성
  → scp 로 3노드 배포 + SHA-256 확인
  → thread_safety_test 실행
  → environment-matrix.md §3.1, §6 기록
```

**주의.** Toolkit2 버전이 Runtime보다 높으면 변환한 모델이 로딩되지 않을 수 있다. `rknn-toolkit2==2.3.0`을 우선 시도한다.

---

## 2.10 Scheduler 호스트 실측 (노트북)

**대상.** Samsung 370E5J 계열 구형 노트북, `192.168.123.14`

**측정 결과.** 상세는 `environment-matrix.md` §4.2.

```text
CPU     Intel i7-4712MQ (Haswell, 4C/8T @2.30GHz)
RAM     3.5GB (가용 1.8GB)          ← 노드(4GB)보다 적음
NIC     RTL8111/8168 (r8169), 1GbE 상한. 2.5G 미지원
USB     Bus 004 = USB 3.0 (5000M, 4포트). 나머지는 USB 2.0
TB      없음
Docker  설치됨
아키텍처 x86_64
```

### 링크 속도 100Mb/s 문제 (해결)

최초 측정에서 `Speed: 100Mb/s`로 협상되어 있었다. 포트는 `1000baseT/Full`을 지원하므로 물리 계층 문제였다.

케이블 교체 후 **1000Mb/s로 정상화**되었다.

**영향 분석.** 방치했다면 JPEG 100KB 기준 약 125 FPS에서 링크가 포화되어, NPU 확장 효율이 아니라 케이블을 측정할 뻔했다. 보드 3대는 처음부터 1000Mb/s였으므로 허브가 아니라 노트북 쪽 케이블이 원인이었다.

**후속 조치.** 매 실험 전 링크 속도를 확인하는 절차를 벤치마크 스크립트에 넣는다.

```bash
ethtool enp3s0 | grep Speed
```

### 판정

| 역할 | 판정 | 근거 |
|---|---|---|
| 모델 변환 | **적합** | x86_64 Linux + Docker |
| 개발용 Scheduler (M2~M5) | **충분** | 링크 속도는 기능 정확성과 무관 |
| 공식 벤치마크 (JPEG) | **조건부 적합** | 실측 FPS 확인 후 판단 |
| 공식 벤치마크 (Raw RGB, S6) | **부적합** | 1GbE 초과 |

**2.5G 어댑터 구매는 보류한다.** 노드당 실제 FPS를 모르는 상태에서는 필요 여부를 판단할 수 없다. S0/S1 측정 후 결정한다.

노드당 40 FPS 가정 시 3노드 120 FPS × 100KB ≈ 96 Mbps로 1GbE에 여유가 있다. 측정으로 판단하는 것이 이 프로젝트의 방식과도 일치한다.

**RAM 3.5GB가 NIC보다 실질적인 제약이다.** 대응은 하드웨어 구매가 아니라 운용 방침으로 한다 — 공식 측정 중에는 Prometheus·Dashboard를 중지하고 `npuforge-bench`가 JSONL 원본만 기록한다.

### 미확인 항목

```bash
cat /etc/os-release      # 프롬프트가 [root@localhost ~]# 형태 — 배포판 확인 필요
uname -r
df -h /                  # Docker 이미지 5~8GB 필요
```

hostname이 `localhost`다. 결과 파일에 측정 호스트가 남아야 하므로 이름을 부여한다(제안: `dealer`).

## 2.11 모델 변환 환경 구축

**결정.** WSL2가 아니라 **노트북(x86_64 Linux)** 에 구축한다. Docker가 이미 설치되어 있고, RKNN-Toolkit2가 요구하는 x86_64 Linux 조건을 만족한다. WSL2를 별도로 세팅할 이유가 없다.

**Docker로 감싸는 이유.** 변환 결과가 호스트 환경에 따라 달라지면 재현성이 깨진다. 이미지가 Python·Toolkit·의존성 버전을 고정하므로 누구 PC에서 돌려도 같은 `.rknn`이 나온다. 오픈소스 공개 시 "이 이미지로 재현하세요"가 가능해진다.

**작성한 파일.**

```text
tools/model-converter/
├── Dockerfile            Ubuntu 22.04 + rknn-toolkit2==2.3.0
├── requirements.txt
├── convert_yolov8n.py    ONNX -> RKNN, 메타데이터 자동 기록
└── README.md             사용법 및 배포 절차
```

**버전 고정.** Toolkit 2.3.0은 보드의 Runtime 2.3.0에 맞춘 값이다. Toolkit이 Runtime보다 높으면 변환된 모델이 로딩되지 않을 수 있다.

**타깃 플랫폼.** `target_platform='rk3576'`으로 고정했다. `rk3588`로 변환한 `.rknn`은 RK3576에서 동작하지 않는다.

**재현성 기록.** `convert_yolov8n.py`는 변환 시 다음을 JSON으로 남긴다.

```text
ONNX SHA-256 / RKNN SHA-256 / calibration manifest SHA-256
calibration 이미지 수 / 양자화 방식 / 변환 옵션 전체
toolkit 버전 / python 버전 / 플랫폼
```

calibration 이미지 목록은 정렬해 고정한다. 순서가 양자화 결과에 영향을 주기 때문이다.

## 2.12 Scheduler 호스트(`dealer`) 접속 및 설정

**대상.** `192.168.123.14`, 계정 `yoo2`

### 배포판 확인: Rocky Linux 9.7

SSH 배너에서 `OpenSSH_8.7` + `gssapi-keyex`가 보여 RHEL 계열임을 먼저 파악했다. 확인 결과 **Rocky Linux 9.7**이다.

```text
PRETTY_NAME  Rocky Linux 9.7 (Blue Onyx)
kernel       5.14.0-611.13.1.el9_7.x86_64
glibc        2.34
패키지 관리자 dnf
Docker       29.2.1 (overlayfs)
디스크       60GB 여유
Swap         3.9GB
```

**앞서 실행한 `sudo apt install ...`은 조용히 실패했었다.** `2>/dev/null`로 오류가 가려졌고, `ethtool`·`lspci`·`dmidecode`가 이미 설치되어 있어 출력은 정상으로 보였다. 이 호스트에서는 `dnf`를 써야 한다.

### 접속 확보 과정에서 겪은 것

**1차 실패.** `printf` 기반 askpass 헬퍼가 비밀번호를 제대로 내보내지 못했다. 헬퍼 출력을 직접 확인해 원인을 좁혔다.

```bash
printf "[%s]\n" "$("$ASKPASS")"    # 실제로 무엇이 나오는지 확인
```

heredoc 방식으로 바꾸니 정상 동작했다.

```sh
#!/bin/sh
cat <<'PW'
<password>
PW
```

**2차 문제 — sudo 불가.** `yoo2`가 `wheel` 그룹에 없었다(`id -nG yoo2` → `yoo2`). Rocky는 기본적으로 사용자를 `wheel`에 넣지 않는다.

**3차 문제 — root SSH 차단.** `PermitRootLogin`이 막혀 있어 root로 직접 붙을 수 없었다.

**해결 — `su` 승격.** `su`는 stdin이 아니라 제어 터미널에서 비밀번호를 읽으므로 `ssh -tt`로 PTY를 할당해야 한다. 그리고 **프롬프트가 뜰 시간을 줘야 한다.**

```bash
# 실패: su 가 읽기 전에 비밀번호가 흘러가 에코됨
printf 'PW\n' | ssh -tt host 'su -c "..."'

# 성공: 지연을 넣는다
( sleep 3; printf 'PW\n'; sleep 2 ) | ssh -tt host 'su -c "..."'
```

이 패턴은 §2.1의 SSH 비밀번호 문제와 같은 원인(TTY 부재)이지만 해법이 다르다. SSH는 `SSH_ASKPASS_REQUIRE=force`로 우회되고, `su`는 PTY 할당이 필요하다.

### 적용한 변경

| 항목 | 변경 |
|---|---|
| hostname | `localhost.localdomain` → **`dealer`** |
| `yoo2` 그룹 | `wheel` 추가 (sudo 가능) |
| `yoo2` 그룹 | `docker` 추가 (sudo 없이 docker 사용) |
| SSH 키 | `id_ed25519_npuforge` 설치 |
| SSH 별칭 | `npuforge-dealer` |

`dealer`는 카드 딜러에서 따왔다. 노드가 `king`/`queen`/`jack`이므로 명칭 체계가 일관된다.

### ⚠️ 호스트와 노드의 배포판이 다르다

| | `dealer` | `king`/`queen`/`jack` |
|---|---|---|
| 배포판 | Rocky Linux 9.7 | Ubuntu 24.04 |
| glibc | 2.34 | 2.39 |
| 패키지 관리자 | `dnf` | `apt` |

**바이너리 배포 방향은 안전하다.** 낮은 glibc(2.34)로 빌드한 바이너리는 높은 glibc(2.39)에서 동작한다. 반대는 성립하지 않는다. 따라서 `dealer`에서 크로스컴파일해 보드로 배포하는 것은 문제없다.

`scripts/fix-node-consistency.sh`는 `apt` 전용이며 노드 대상이므로 그대로 두어도 된다. 호스트까지 다루는 스크립트를 쓸 때는 패키지 관리자를 분기해야 한다.

## 2.13 모델 변환 이미지 빌드

**1차 시도 실패.** Dockerfile이 `validate_rknn.py`를 COPY 하는데 파일이 존재하지 않았다.

```text
ERROR: "/validate_rknn.py": not found
```

**부수 교훈.** 백그라운드 실행 시 `docker build ... | tail -40` 형태로 파이프를 걸면 종료 코드가 `tail`의 것이 되어 **실패가 성공으로 보고된다.** 로그를 파일로 남기고 종료 코드를 따로 확인하도록 바꿨다.

```bash
docker build -t img . > /tmp/build.log 2>&1; echo "EXIT=$?"; tail -25 /tmp/build.log
```

**조치.** `validate_rknn.py`를 작성했다. 변환된 모델의 입출력 shape을 확인하고, `onnxruntime`이 있으면 ONNX 원본과 코사인 유사도를 비교한다. 기본 기준은 0.98이다.

DEV-REQ §2.2의 검증 대상 중 "ONNX 결과 ↔ RKNN Simulator 결과" 비교에 해당한다. 보드 3대 실측 비교는 별도로 수행한다.

### 개선 여지: 이미지 용량

빌드 로그에서 `rknn-toolkit2`가 의존성으로 `torch`를 끌어오고, 그 과정에서 **NVIDIA CUDA 라이브러리를 수백 MB씩 내려받는 것**을 확인했다.

```text
nvidia_cusolver_cu12   124.2 MB
nvidia_cusparse_cu12   196.0 MB
nvidia_nccl_cu12       176.2 MB
...
```

`dealer`에는 GPU가 없으므로 전부 사용되지 않는다. CPU 전용 torch를 먼저 설치하면 수 GB를 줄일 수 있다.

```dockerfile
RUN python3 -m pip install torch --index-url https://download.pytorch.org/whl/cpu \
    && python3 -m pip install "rknn-toolkit2==${RKNN_TOOLKIT_VERSION}"
```

디스크 여유가 51GB 남아 있어 당장 문제는 아니다. 빌드가 완료된 뒤 최적화한다.

## 2.14 YOLOv8n ONNX 확보

### ⚠️ 표준 Ultralytics export는 RKNN에 부적합하다

RKNN용 YOLOv8은 **Rockchip이 수정한 exporter**로 만들어야 한다. 표준 Ultralytics export는 DFL·NMS 후처리가 ONNX 그래프에 포함되는데, 이 연산들이 NPU에 매핑되지 않아 CPU fallback이 대량 발생한다.

수정판은 **decode 이전의 raw 텐서를 출력**하고 후처리를 CPU에서 따로 수행한다.

```text
공식 원본 : 출력 1개 (decode·NMS 포함)
최적화판  : 출력 3그룹
            [1,64,80,80]  박스 좌표
            [1,80,80,80]  80개 클래스별 confidence
            [1,1,80,80]   confidence 합
```

이것이 `environment-matrix.md` §6의 "CPU fallback 연산 목록" 항목과 직결된다. **잘못 export하면 NPU가 아니라 CPU를 측정하게 된다.**

### 확보한 파일

`rknn_model_zoo`가 사전 최적화된 ONNX를 배포한다. 직접 export할 필요가 없어 위험이 줄었다.

```text
출처      airockchip/rknn_model_zoo  examples/yolov8
원본      airockchip/ultralytics_yolov8
경로      ~/npuforge/models/yolov8n.onnx  (dealer)
크기      12,650,184 bytes
SHA-256   0c8716701f471067932b797eeb67c8e5db47c693c2557c881d7679ec12e21bc5
형식      PyTorch 2.0 export
```

**RK3576이 공식 지원 목록에 있다.**

```text
RK3562, RK3566, RK3568, RK3576, RK3588, RV1126B, RV1109, RV1126, RK1808, RK3399PRO
```

### 라이선스

`rknn_model_zoo` 저장소는 Apache-2.0이지만 **모델 자체는 AGPL-3.0**이다(Ultralytics 원본 상속). 저장소 라이선스와 데이터 라이선스는 별개다.

상세와 대응 방침은 `MODEL_LICENSES.md` 참조. 요약하면 모델 파일을 저장소에 포함하지 않고 사용자가 직접 내려받게 한다.

---

## 2.15 모델 변환 성공

### onnx 버전 충돌

첫 변환이 실패했다.

```text
AttributeError: module 'onnx' has no attribute 'mapping'
```

**원인.** `rknn-toolkit2`의 의존성 명세가 onnx 버전을 제한하지 않아 최신 버전(1.22.0)이 설치되었다. `onnx.mapping`은 onnx 1.16에서 제거되었는데 rknn-toolkit2 2.3.0이 이를 사용한다.

**해결.** `onnx==1.14.1`로 고정하니 즉시 변환에 성공했다. Dockerfile에 고정과 검증 단계를 넣었다.

```dockerfile
RUN python3 -m pip install "onnx==1.14.1" \
    && python3 -c "import onnx; assert hasattr(onnx, 'mapping'), 'onnx.mapping 없음'"
```

**함께 적용한 개선.** torch를 CPU 전용 인덱스에서 설치하도록 바꿨다. GPU가 없는 호스트에서 NVIDIA CUDA 라이브러리를 수 GB 받는 낭비를 없앤다.

### FP16 모델 생성

Calibration 데이터가 확정되지 않아 INT8 대신 FP16으로 먼저 변환했다. **양자화 없이도 thread-safety 검증에는 지장이 없다.**

```text
파일      yolov8n-fp16.rknn
크기      9,645,065 bytes
SHA-256   459602ea70479c1ce4fdd7419aa81e10e2f795fe6fe87444f3607f25b7054c0f
```

3노드에 배포하고 SHA-256이 모두 일치함을 확인했다. 3대 모두에서 테스트 프로그램 컴파일도 성공했다.

## 2.16 Thread-safety 검증 — 진행 중

### 예비 관측 (반복 2회)

```text
RKNN api        2.3.0 (c949ad889d@2024-11-07T11:35:33)
RKNN driver     0.9.8
입력/출력 개수  1 / 9              최적화판의 3그룹 × 3
입력 크기       1,228,800 bytes    = 640×640×3, 문서 계산과 일치
FP16 추론시간   78.8 ~ 116.1 ms
```

**시나리오 1(context 공유, 2스레드)에서 오류가 0건이었다.** 반복 2회는 표본이 너무 적어 단정할 수 없으므로 20회로 재측정 중이다.

FP16이 약 100ms이므로 노드당 10 FPS 수준이다. INT8은 통상 3~5배 빠르므로 30~50 FPS를 예상한다. **이 수치가 2.5G 스위치 구매 판단의 근거가 된다.**

### 실행 과정에서 겪은 함정 두 가지

**1. 파이프의 `head`가 출력을 삼킨다.**

```bash
ssh host './test model 30' | grep -v ... | head -70    # 출력 0바이트
```

`head`가 조기에 파이프를 닫아 SIGPIPE가 발생하고 원격 명령이 중단되었다. 백그라운드 작업은 exit 0으로 보고되어 성공처럼 보였다.

**2. 파일 리다이렉트 시 블록 버퍼링 + SIGHUP.**

```bash
ssh host './test model 50 > run50.log 2>&1'
```

stdout이 파일이면 libc가 라인 버퍼링 대신 **블록 버퍼링**을 쓴다. SSH 세션이 끊기며 프로세스가 SIGHUP으로 종료되었고, 버퍼에 있던 출력이 통째로 유실되었다. 로그에는 stderr로 나간 한 줄만 남았다(stderr는 항상 무버퍼).

**해결.** 세션에서 분리하고 라인 버퍼링을 강제한다.

```bash
nohup bash -c 'stdbuf -oL -eL ./test model 20 > run20.log 2>&1; echo DONE=$? > done.marker' &
```

완료 마커 파일을 폴링해 결과를 가져온다. **장시간 실행되는 벤치마크에도 같은 패턴이 필요하다** — `run-benchmark.sh`의 무인 실행 요구사항(`01-TECHSPEC.md` §20.4)에 반영한다.

---

# 2.17 ⚠️ 고부하에서 보드가 재부팅된다 (미해결)

## 증상

스레드 수 스윕(3~8스레드)을 실행하면 **`king`과 `jack`이 재부팅된다.** `queen`은 동일 테스트를 완주한다.

| 노드 | 부팅 횟수 | uptime (2026-08-10 02:00) | 스윕 결과 |
|---|---:|---|---|
| `king` | **13** | 15분 | **재부팅 3회** (01:26, 01:38, 01:45) |
| `queen` | 5 | **3일 17시간** | **완주** |
| `jack` | 5 | 26분 | 재부팅 |

`king`은 다른 두 대보다 부팅 횟수가 8회 많다. 모두 오늘 스윕을 돌린 시점과 일치한다.

## 하드 리셋이다

재부팅 직전 로그에 **종료 시퀀스가 전혀 없다.** SSH 세션이 열린 직후 로그가 그냥 끊긴다.

```text
Aug 10 01:45:45 king sshd[1586]: Accepted publickey for pi ...
Aug 10 01:45:45 king systemd-logind[488]: New session 4 of user pi.
(로그 끝 — 커널 패닉도, 종료 메시지도 없음)
```

kernel panic, OOM killer, thermal shutdown 메시지가 없다. **전원 차단 또는 watchdog에 의한 하드 리셋**으로 보인다.

## 원인 후보

| 후보 | 근거 | 판정 |
|---|---|---|
| **전원 공급 부족** | 하드 리셋, 로그 없음, 노드별 편차 | **유력** |
| 발열 | 재부팅 시점 온도 45~50°C | **배제** (임계치와 거리 멂) |
| 메모리 부족 | 가용 3.2GB, OOM 로그 없음 | 배제 |
| 보드 개체 불량 | `queen`만 정상 | 가능 |

전원이 유력한 이유는 **부하 특성**과 맞기 때문이다. 8스레드는 CPU 8코어와 NPU 2코어를 동시에 최대로 쓴다. 순간 전류가 어댑터 용량을 넘으면 전압이 떨어지고 보드가 리셋된다. 로그가 남지 않는 것도 이와 일치한다.

`queen`이 3일 17시간 무중단으로 같은 테스트를 완주했다는 점이 **소프트웨어 문제가 아니라 개체별 하드웨어 조건 차이**임을 시사한다.

## 문서의 전원 가정을 정정해야 한다

`02-HARDWARE-SETUP.md` §8은 **USB-C PD 어댑터**를 전제하는데, 커널 로그의 레귤레이터 이름은 다음과 같다.

```text
vcc12v_dcin      12V DC 입력
vcc_sys
rk806-regulator
```

**실제 전원 입력 방식을 확인해야 한다.** 12V DC라면 USB-C PD 전제로 쓴 §8 전체가 틀렸다.

## 프로젝트에 미치는 영향 — 심각하다

공식 벤치마크는 **300초 지속 부하 × 5회 반복 × 143 run, 총 22시간**이다(`01-TECHSPEC.md` §20.4).

지금 상태로는:

- 측정 중 노드가 재부팅되어 run이 무효가 된다
- 재부팅을 "노드 장애"로 기록하면 **소프트웨어 장애 감지 성능을 잘못 측정**하게 된다
- S4 장애 복구 실험에서 의도한 장애와 전원 문제를 구분할 수 없다
- 무인 야간 실행이 불가능하다

**S0 열 특성 측정 전에 반드시 해결해야 한다.**

## 조치 계획

1. **세 노드의 전원 어댑터 확인** — 제조사, 모델, 정격 출력. 물리 확인 필요
2. 입력 방식 확인 — USB-C PD인지 12V DC 배럴잭인지
3. `queen`의 어댑터를 `king`에 물려 재현 시도 — 어댑터 문제인지 보드 문제인지 판별
4. 동일 모델 어댑터 3개로 통일 (`infrastructure.md` §5 구매 목록)
5. 해결 후 스윕 재실행으로 3대 일관성 확인

**해결 전까지 고부하 테스트를 반복하지 않는다.** 재부팅을 반복시켜 eMMC 손상 위험을 키울 이유가 없다.

## 유효한 데이터는 남아 있다

`queen`이 전체 스윕을 완주했으므로 **thread-safety 결론(§3.1)은 유효하다.** 다만 3대 재현성 확인은 전원 문제 해결 후로 미룬다.

## 정정: 두 개의 서로 다른 현상이었다

절대 시각으로 부팅 이력을 다시 확인한 결과, 위 분석을 정정한다. **uptime만 비교해 하나의 원인으로 묶은 것이 성급했다.**

### 사건 A — 부하 중 개별 재부팅 (조사 대상)

```text
01:26:16  king  재부팅
01:34:40  jack  재부팅
01:38:12  king  재부팅
01:45:58  king  재부팅
```

모두 스윕 테스트 실행 시각과 일치한다. 이 구간 내내 **`queen`은 3일 17시간 무중단이었다.**

부하와 상관관계가 있고 노드별로 다르게 나타나므로, 전원 공급 또는 개체 편차 가설은 **이 사건에 대해서는 유효하다.**

### 사건 B — 3대 동시 재부팅 (부하와 무관)

```text
king   이전 부팅 종료  02:01:00
queen  이전 부팅 종료  02:05:20
jack   이전 부팅 종료  02:05:10
       ↓ 약 27분 정전
세 대 모두 약 02:32 부팅   (04:19 기준 uptime 1시간 47분으로 동일)
```

세 대가 4분 이내에 함께 내려갔고 **27분간 꺼져 있다가 함께 올라왔다.** 이 시각에는 부하 테스트를 실행하지 않았다.

이는 **공용 전원 차단**(정전, 멀티탭 차단, 물리적 재배치)이며 사건 A와 원인이 다르다. 부하로 인한 리셋은 즉시 재부팅되지 실행이 27분간 멈추지 않는다.

**따라서 "고부하가 3대를 모두 재부팅시킨다"는 앞선 서술은 과했다.** 부하와 연결된 것은 사건 A뿐이다.

### 사건 B의 정체: 전원 재배치 작업

사용자 확인 결과, **세 보드의 전원을 각각 독립 소스로 분리하는 작업**이 있었다. 02:05의 3대 동시 정지와 27분 공백이 이 작업 시간과 일치한다.

**따라서 사건 B는 장애가 아니라 계획된 물리 작업이다.** 이것도 원인 미상 재부팅으로 기록했다면 잘못된 추적이 될 뻔했다.

`02-HARDWARE-SETUP.md` §8.1의 "멀티포트 충전기 하나에 세 대를 몰지 않음" 요구가 이로써 충족되었다.

### 진단에 남은 것

| 사건 | 시각 | 원인 | 상태 |
|---|---|---|---|
| A: king ×3, jack ×1 | 01:26~01:45 | 전원 재배치 **이전** 구성에서 고부하 | **재검증 필요** |
| B: 3대 동시 27분 | 02:01~02:32 | 전원 재배치 작업 | 해결 (장애 아님) |

**사건 A는 재배치 이전에 발생했다.** 전원이 독립 소스로 분리된 지금은 재현되지 않을 수 있다. 동일 조건으로 재검증한다.

### 교훈: uptime 비교만으로 판단하지 않는다

처음에 `uptime`만 보고 "고부하가 3대를 재부팅시킨다"고 결론지었으나, 절대 시각으로 보니 서로 다른 두 사건이었다. 게다가 하나는 장애가 아니라 계획된 작업이었다.

**벤치마크 중 노드 재시작을 기록할 때는 절대 시각과 작업 이력을 함께 남긴다.** 그렇지 않으면 물리 작업을 소프트웨어 장애로 오독하게 된다. 이 문서의 존재 이유가 여기에 있다.

## 2.17.1 원인 확정: `king`의 부트로더 펌웨어가 구버전이다

전원 가설을 두 차례 검증한 끝에 실제 원인을 찾았다.

### 전원이 원인이 아니라는 증거

| 관측 | 함의 |
|---|---|
| 3포트 공유 전원 시절에도 `queen`은 8스레드 완주 | 공유 전원 자체가 문제가 아니다 |
| 개별 전원으로 교체 후에도 `king`은 5스레드에서 리셋 | 어댑터 용량 문제가 아니다 |
| 세 어댑터가 동일 조건 | 개체별 어댑터 차이가 아니다 |

### 펌웨어 비교

```bash
grep -oE 'androidboot\.fwver=[^ ]*' /proc/cmdline
```

| 구성요소 | `king` | `queen` | `jack` |
|---|---|---|---|
| DDR init | **v1.09** | v1.13 | v1.13 |
| SPL | **v1.07** | v1.09 | v1.09 |
| **BL31 (ATF)** | **v1.17** | **v1.24** | **v1.24** |
| BL32 | **v1.05** | v1.10 | v1.10 |
| U-Boot | **2025-07-17** | 2026-07-10 | 2026-07-10 |
| PMIC 초기화 | **`ON:0x20 OFF:0x2`** | `ON:0x40 OFF:0x0` | `ON:0x40 OFF:0x0` |

`queen`과 `jack`은 완전히 일치하고 **`king`만 약 1년 낡았다.**

**BL31은 ARM Trusted Firmware이며 Rockchip 플랫폼에서 DVFS와 전압 조절을 담당한다.** v1.17과 v1.24 사이에 전압 테이블이나 DVFS 로직이 바뀌었다면, 구버전이 고부하 전압을 감당하지 못해 리셋되는 것이 정확히 관측된 증상이다.

DDR 펌웨어 차이(v1.09 vs v1.13)도 메모리 트래픽이 큰 다중 스레드 조건에서 불안정성을 유발할 수 있다.

PMIC 초기화 레지스터가 다른 것은 펌웨어 차이의 결과다.

### 잘못된 진단으로 인한 비용

전원을 의심해 사용자가 어댑터 3개를 전부 교체했으나 원인이 아니었다. 개별 전원 구성 자체는 `02-HARDWARE-SETUP.md` §8.2의 요구사항을 충족하므로 낭비는 아니지만, **진단 방향을 잘못 잡아 시간을 소모했다.**

`queen`이 공유 전원에서도 8스레드를 완주했다는 사실이 이미 전원 가설을 약화시키고 있었는데, 그 신호를 충분히 무겁게 다루지 않았다.

### 문서의 구멍

`environment-matrix.md`에 커널·glibc·RKNN 버전은 있었으나 **부트로더 펌웨어 항목이 없었다.** `collect-node-info.sh`도 수집하지 않았다.

"동일한 3대"를 검증한다면서 **전력 관리를 담당하는 계층을 빠뜨린 것**이다. 두 가지를 모두 보완했다(2026-08-10).

### 이미지 버전이 특정되었다

```text
/etc/rom-version
  king   20251222     2025-12-22 이미지
  queen  20260721     2026-07-21 이미지
  jack   20260721
```

`king`만 7개월 낡은 이미지다. 펌웨어 차이의 출처가 여기다.

`/etc/friendlyelec-release`는 세 대가 동일하다(`BOARD=NanoPi-R76S`, `LINUXFAMILY=nanopi-m5`, `BRANCH=dev`). 구분되는 것은 `rom-version`이므로 **이 값을 노드 일치 검증 항목에 포함한다.**

### 조치: `king` OS 재설치 (2026-08-10 결정)

부트로더만 갱신하는 대신 **OS를 재설치한다.** 근거는 다음과 같다.

- `king`은 OS 패치 레벨도 뒤처져 있다(24.04.3 vs 24.04.4). 재설치로 함께 해결된다
- 진단 과정에서 6회 하드 리셋시켜 파일시스템 상태를 신뢰하기 어렵다
- 부트로더만 갱신하는 절차는 `rkdeveloptool`/`eflasher`가 필요해 오히려 복잡하다

**목표 이미지: `rom-version = 20260721`** (NanoPi-R76S용 Ubuntu 24.04, FriendlyElec 배포본)

재설치 후 `scripts/setup-node.sh`로 자동 세팅한다.

```bash
./scripts/setup-node.sh 192.168.123.12 king npuforge-k
```

이 스크립트가 수행하는 것:

| 단계 | 내용 |
|---|---|
| 1 | SSH 키 설치 (`SSH_ASKPASS_REQUIRE=force` 사용) |
| 2 | `~/.ssh/config` 별칭 등록 |
| 3 | hostname 설정, `/etc/hosts` 정리 |
| 4 | **SSH 호스트 키 재생성** (이미지 복제 시 중복 방지) |
| 5 | **커널 패키지 hold** (RKNPU 드라이버 보호) |
| 6 | 기본 패키지 설치, chrony 활성화 |
| 7 | **기준 노드(`queen`)와 환경 비교** — `rom-version`, `fwver`, 커널, glibc, RKNN 버전 및 해시, NPU 코어 수, RAM |

7단계가 핵심이다. 재설치가 목적을 달성했는지 스크립트가 직접 판정한다.

### 재설치 후 검증 순서

```bash
# 1. 실측 수집
ssh npuforge-k 'bash -s' < scripts/collect-node-info.sh > benchmarks/node-info/king.txt

# 2. 펌웨어 일치 확인 (setup-node.sh 가 자동 비교하지만 재확인)
for h in npuforge-k npuforge-q npuforge-j; do
  ssh $h 'printf "%s %s\n" "$(hostname)" "$(grep -oE "androidboot.fwver=[^ ]*" /proc/cmdline)"'
done

# 3. 안정성 재검증 — 이전에 리셋되던 5~8 스레드 구간
ssh npuforge-k 'cd ~/npuforge-rknn-test && ./thread_safety_test yolov8n-fp16.rknn 20 5 8'
```

3번이 통과하면 `worker_count`를 세 노드에 동일하게 설정할 수 있고, "동일한 3대" 전제가 회복된다.

## 2.17.2 원인 확정: 전원 어댑터 전류 부족 (해결됨, 2026-08-10)

### 결정적 증거: 입력 전압 실측

보드에 입력 전압 센서가 있다는 것을 뒤늦게 발견했다.

```bash
cat /sys/class/power_supply/simple-vin/voltage_now
```

| 시점 | 유휴 전압 |
|---|---|
| **교체 전 어댑터** | **4.983 V** ← 무부하에서도 이미 5V 미만 |
| **5V 4A 어댑터** | **5.26 ~ 5.31 V** |

교체 전 어댑터는 **부하가 없는 상태에서도 5V를 유지하지 못했다.** 고부하에서 더 떨어져 보드의 브라운아웃 임계를 넘은 것이 재부팅의 원인이다.

새 어댑터의 부하 중 전압(`king`, 8스레드까지 984샘플):

```text
최소 5.061 V   평균 5.260 V   최대 5.341 V   변동폭 0.280 V
```

부하가 걸려도 5V 아래로 내려가지 않는다.

### 검증 결과: 3대 모두 8스레드 완주

| 노드 | 8스레드 처리량 | 오류 | 재부팅 |
|---|---:|---:|---|
| `king` | 77.3 inf/s | 0 | **없음** |
| `queen` | 70.2 inf/s | 0 | **없음** |
| `jack` | 78.0 inf/s | 0 | **없음** |

`king`은 4스레드도 통과했다(54.1 inf/s). 이전에는 3스레드에서도 재부팅했다.

### ⚠️ 전압을 12V로 오판했던 기록

커널 로그의 `vcc12v_dcin: 12000 mV`를 실제 입력 전압으로 단정하고 문서에 "12V DC 입력"으로 기록했다. **틀렸다.**

이 이름은 디바이스 트리의 fixed-regulator 선언이며, Rockchip 디바이스 트리가 보드 간 복사되면서 남은 것이다. 실제 입력은 5V다.

**확인했어야 할 것은 선언이 아니라 실측값이었다.**

```text
선언 (디바이스 트리)  vcc12v_dcin: 12000 mV     ← 신뢰 불가
실측 (센서)           simple-vin: 4983000 µV    ← 이것이 사실
```

사용자가 5V 4A 어댑터로 교체하겠다고 했을 때 "5V는 위험하다"고 경고할 뻔했다. 실측을 먼저 확인해 오류를 막았다.

### 진단 과정에서 틀렸던 가설들

| # | 가설 | 결과 | 반증 근거 |
|---|---|---|---|
| 1 | 공용 3포트 전원이 원인 | **틀림** | 공용 전원에서 `queen`은 8스레드 완주 |
| 2 | 부트로더 펌웨어 구버전 | **틀림** | `king` 재설치로 펌웨어 일치시켜도 재부팅. `jack`은 처음부터 같은 펌웨어인데 실패 |
| 3 | 입력 전압이 12V | **틀림** | 실측 4.983V |
| 4 | **어댑터 전류 부족** | **맞음** | 유휴 4.983V → 5.3V 교체 후 3대 모두 8스레드 완주 |

**가설 1이 원인을 절반 맞혔는데도 반증으로 처리했다.** "공용이냐 개별이냐"가 아니라 "용량이 충분한가"가 문제였는데, 구성 방식에 집중하느라 용량을 놓쳤다. 개별 전원으로 교체했을 때 오히려 악화된 것이 그 증거였는데(새 어댑터가 더 약했다), 그때도 전류 용량으로 돌아가지 않고 펌웨어로 방향을 틀었다.

`queen`이 공용 전원에서 8스레드를 완주했다는 사실은 "그 어댑터가 충분했다"는 뜻이지 "전원이 원인이 아니다"라는 뜻이 아니었다.

### 교훈: 실측 센서를 먼저 찾는다

`/sys/class/power_supply/`는 처음 수집한 `collect-node-info.sh`에 포함되지 않았다. 전원을 의심하기 시작한 시점에 이 센서를 찾았다면 **가설 2와 3을 거치지 않고 바로 확정할 수 있었다.**

`collect-node-info.sh`에 입력 전압 항목을 추가했다.

### 지속 부하 검증 (3대 동시, 8스레드)

순간 부하 통과가 지속 부하 통과를 보장하지 않으므로 별도로 확인했다.

**전압 — 문제 없다.**

| 노드 | 최소 전압 |
|---|---|
| `king` | 5.061 V |
| `queen` | 5.157 V |
| `jack` | 5.124 V |

3대를 동시에 최대 부하로 돌려도 5V 아래로 내려가지 않는다. 재부팅도 없다. **전원 문제는 해결되었다.**

**온도 — 새로운 문제가 드러났다.**

| 노드 | 최고 SoC | 최고 NPU |
|---|---:|---:|
| **`king`** | **88.7 °C** | **91.3 °C** ⚠️ |
| `queen` | 70.2 °C | 70.2 °C |
| `jack` | 71.2 °C | 72.1 °C |

`king`이 다른 두 대보다 **약 19°C 높다.** 그리고 **`disable_temperature_c`(90°C)를 초과했다.**

세 보드가 동일 모델·동일 펌웨어·동일 부하이므로 소프트웨어 원인은 배제된다. 후보는 다음과 같다.

- 물리적 배치 차이 (공기 흐름, 벽면 근접, 보드 간 간격)
- 방열 접촉 상태
- 개체 편차

`king`은 다른 두 대보다 약 6분 먼저 부하가 시작되었으나, `queen`/`jack`도 이미 평탄역(70~72°C)에 들어갔으므로 시간 차만으로 19°C를 설명할 수 없다.

**§2.19에서 별도로 다룬다.**

### 남은 확인 사항

- 8스레드에서도 처리량이 꺾이지 않았으므로 `MAX_THREADS`를 늘려 최적점을 다시 찾는다
- 전류 측정 수단이 없다. `voltage_now`만 있고 `current_now`가 없어 소비전력을 계산할 수 없다. FPS/Watt 지표에는 외부 전력계가 필요하다

## 2.19 `king`의 온도가 19°C 높다 (재현되지 않음, 2026-08-11)

지속 부하 시험에서 발견했다. 동일 조건인데 `king`만 NPU 91.3°C에 도달해 스케줄링 제외 임계치를 넘었다.

### 왜 중요한가

**노드별 온도 편차는 확장 효율 측정을 직접 오염시킨다.**

- `king`이 먼저 throttling에 들어가면 처리량이 떨어진다
- 스케줄러는 이를 "느린 노드"로 인식해 부하를 줄인다
- 결과적으로 3노드 확장 효율이 낮게 측정되는데, **원인이 스케줄링이 아니라 물리적 배치**다
- 90°C를 넘으면 스케줄링에서 아예 제외되어 사실상 2노드 실험이 된다

`02-HARDWARE-SETUP.md` §9.1이 "동일한 주변 온도, 동일한 배치 방향, 보드 사이 최소 10cm"를 요구하는 이유가 이것이다.

### 확인해야 할 것

| 항목 | 방법 |
|---|---|
| 물리적 배치 | 세 보드의 간격, 방향, 주변 장애물 확인 |
| 적층 여부 | 겹쳐 놓았다면 분리 |
| 공기 흐름 | 벽면·구석·케이블 뭉치에 막혔는지 |
| 주변 온도 | 각 보드 위치의 실제 온도 (햇빛, 다른 장비 발열) |
| 방열판 접촉 | 케이스 장착 상태 |

배치를 균일하게 맞춘 뒤 동일 시험을 반복해 편차가 사라지는지 확인한다. 그래도 남으면 개체 편차이므로 결과에 명시한다.

### 유휴 온도에는 편차가 없다 (2026-08-11 확인)

부하 종료 19.9시간 뒤 세 보드를 동시에 측정했다.

| 보드 | NPU (유휴) | SoC | load1 | 부하 시 NPU (2026-08-10) |
|---|---|---|---|---|
| `king` | 39.8°C | 40.7°C | 1.34 | 91.3°C |
| `queen` | 36.1°C | 36.1°C | 0.07 | 70.2°C |
| `jack` | 37.0°C | 38.8°C | 0.23 | 72.1°C |

**유휴 편차는 2.8~3.7°C에 불과하다.** 그나마도 측정 시점에 `king`에서
`gnome-control-center` 세션이 돌고 있었고(load 1.34), 나머지 두 대는 사실상
유휴였다. 즉 유휴 상태에서 세 보드는 사실상 같다.

이것이 뜻하는 바:

- 19°C는 **지속 부하에서만 벌어지는 격차**다. 방열 능력 차이(공기 흐름)로
  설명하기에 부합한다 — 유휴 발열량에서는 차이가 드러나지 않고, 발열량이
  커질수록 방열 조건 차이가 온도 차로 증폭된다
- 개체 불량(예: 방열판 접촉 불량)이었다면 유휴에서도 어느 정도 드러났을
  가능성이 높다. 완전히 배제할 수는 없으나 배치 가설이 더 유력하다
- **따라서 재측정은 반드시 부하 조건에서 해야 한다.** 유휴 온도만 보고
  "해결됐다"고 판단하면 안 된다

세 보드 모두 `graphical.target` + `gdm` active로 구성이 동일함도 함께 확인했다.
데스크톱 세션이 한 대에만 떠 있으면 그 자체가 측정 오염원이므로, 벤치마크
직전에 세션 상태를 맞춘다(`preflight-check.sh` 항목).

### 통제된 재측정: 19°C 격차는 재현되지 않는다 (2026-08-11)

전용 부하 도구(`sustained_load_test`)로 세 보드에 **동시에** 8스레드 부하를
15분간 걸었다. 평탄역(부하 후 300초~종료, 보드당 약 557샘플) 요약이다.

| 보드 | NPU 평균 | NPU 최고 | SoC 평균 | 입력전압 최저 | 처리량 |
|---|---|---|---|---|---|
| `king` | 73.0°C | **75.8°C** | 71.2°C | 5.070 V | **80.5 inf/s** |
| `queen` | 67.5°C | 70.2°C | 65.8°C | 5.090 V | 77.7 inf/s |
| `jack` | 72.6°C | 74.8°C | 71.6°C | 5.046 V | 77.8 inf/s |

**최대 편차 5.6°C. 90°C 초과 없음. NPU 클럭 강하 없음**
(928 샘플 전부 950 MHz, 하나도 떨어지지 않았다).

상승 곡선도 세 보드가 나란하다.

```text
 t(s)   king  queen   jack
    0   37.0   35.2   37.0
   60   66.5   61.9   66.5
  120   72.1   65.6   69.3
  300   73.0   67.5   73.0
  600   73.9   67.5   73.0
  880   74.8   68.4   72.1
```

### 이전 측정과 무엇이 달랐나

08-10 측정(`king` 91.3 / `queen` 70.2 / `jack` 72.1)과 직접 비교할 수 없다.
**부하 프로파일이 달랐다.**

| | 2026-08-10 | 2026-08-11 |
|---|---|---|
| 도구 | `thread_safety_test` | `sustained_load_test` |
| 부하 형태 | 1→8 스레드 순차 스윕 | 8스레드 고정 |
| 시작 시각 | `king`이 약 6분 선행 | 동시 |
| 지속 | 스윕 완료까지 | 900초 고정 |

`thread_safety_test` 는 목표 스레드 수에 도달하기 전에 단일/2스레드 기준선을
먼저 돌린다. 즉 `king` 은 다른 두 대가 8스레드에 들어갈 무렵 이미 훨씬 오래
가열된 상태였다. 6분 선행까지 겹치면 격차가 부풀려질 조건이 갖춰진다.

`queen` 의 최고 온도는 두 측정에서 **70.2°C 로 동일**하고 `jack` 은
72.1 → 74.8°C 로 소폭 올랐다. 움직인 것은 `king` 뿐이다(91.3 → 75.8°C).
배치를 바꾸지 않았다는 점을 감안하면, 격차의 상당 부분은 **물리적 배치가
아니라 측정 방법의 문제**였을 가능성이 크다.

물론 배치 요인을 완전히 배제할 수는 없다. 다만 지금 조건에서는

- 어떤 보드도 `degraded_temperature_c`(80°C)에 닿지 않는다
- 어떤 보드도 throttling 되지 않는다
- 처리량 편차가 3.5% 이내다 (80.5 / 77.7 / 77.8 inf/s)

이므로 **벤치마크를 막는 요인이 아니다.** S0 실험을 진행할 수 있다.

`king` 이 가장 뜨거우면서 동시에 가장 빠르다는 점도 일관된다 — 15분간
72,481회로 `queen`(69,928회)보다 3.6% 더 많은 일을 했다. 다만 3.6%의 일량
차이가 5.5°C를 다 설명하지는 못하므로, 작은 방열 조건 차이는 남아 있다고
본다.

### 여기서 얻은 측정 원칙

**부하 프로파일이 다르면 온도를 비교하지 않는다.** 같은 "고부하"라도
도달 경로가 다르면 축적 열량이 다르다. S0 이후 모든 열 비교는
`scripts/run-thermal-comparison.sh` 로 수행한다. 이 스크립트는

- 별칭↔hostname 일치를 먼저 검증하고 (§2.20)
- 세 보드의 바이너리/모델 해시가 같은지 확인하고
- 유휴 기준선을 먼저 재고
- 세 보드에 **동시에** 부하를 걸고
- run 전후 `boot_id` 를 비교해 도중 리셋된 보드를 무효 처리한다

### 임계치 재검토가 필요하다

현재 설정값은 초안 그대로다.

```text
degraded_temperature_c = 80.0
disable_temperature_c  = 90.0
```

팬리스 보드가 정상 동작 중에 70~91°C에 도달한다면 이 값들은 **보호가 아니라 측정 방해**가 된다. S0 결과로 재설정한다(`02-HARDWARE-SETUP.md` §9.2).

RK3576의 실제 임계 온도(Tj max)를 확인해 그보다 충분히 낮게, 그러나 정상 동작 범위보다는 높게 잡아야 한다.

## 2.20 문서에 적힌 `king`의 IP가 틀렸다 (2026-08-11)

`king`을 `192.168.123.22`로 기록해 두었으나 **실제 주소는 `192.168.123.12`** 다.
`.22`는 서브넷 전체 스윕에서 ARP 응답조차 없는 빈 주소였고, 그 결과
"`king`이 죽었다"는 잘못된 결론을 냈다.

### 왜 놓쳤나

`~/.ssh/config` 의 `npuforge-k` 별칭에는 **처음부터 `.12`가 올바르게** 들어
있었다. 틀린 것은 문서와 스크립트에 하드코딩한 IP뿐이다. 별칭을 썼다면
애초에 드러나지 않았을 문제다.

| 위치 | 값 | 상태 |
|---|---|---|
| `~/.ssh/config` `npuforge-k` | `.12` | 정상 |
| `board-worklog.md` §1 표 | `.22` | **오류** |
| `environment-matrix.md` §7 | `.22` | **오류** |
| `infrastructure.md` | `.22` | **오류** |
| `setup-node.sh` 사용 예 | `.22` | **오류** |
| `fix-node-consistency.sh` IP 목록 | `.22` | **오류** |

모두 `.12`로 정정했다.

### 재발 방지

**보드 접속은 IP가 아니라 별칭(`npuforge-k/q/j`)으로 한다.** IP는 DHCP라
바뀔 수 있고, 문서에 박아 두면 반드시 한 곳이 낡는다. 별칭은 한 곳
(`~/.ssh/config`)만 고치면 된다.

`preflight-check.sh` 에 다음을 넣는다.

- 세 별칭이 모두 접속되는가
- 각 별칭이 붙은 호스트의 `hostname` 이 `king/queen/jack` 과 일치하는가

이름이 어긋난 채로 벤치마크를 돌리면 결과가 엉뚱한 노드에 귀속된다.
이번처럼 "노드가 죽었다"로 끝나면 차라리 낫고, 조용히 다른 보드에
붙는 쪽이 훨씬 위험하다.

### 참고: 보드 MAC 은 OUI 가 없다

세 보드 모두 locally administered MAC 을 쓴다(`82:`, `66:`, `26:` — 두 번째
니블이 2/6/A/E). 제조사 OUI 로 보드를 식별할 수 없다는 뜻이라, 네트워크
스캔으로 보드를 찾는 방법은 통하지 않는다.

다만 `addr_assign_type = 0`(permanent) 이므로 **재부팅해도 MAC 은 유지된다.**
DHCP 리스가 흔들릴 이유는 없다. 그래도 IP 고정(정적 할당 또는 DHCP
예약)을 해 두는 편이 안전하다.

## 2.21 원격 백그라운드 실행의 두 가지 함정 (2026-08-11)

`preflight-check.sh` 를 만들면서 검사가 **조용히 작동하지 않는** 것을
발견했다. 부하가 도는데 "남은 부하 없음"으로 통과했다.

두 가지가 겹쳐 있었다.

### 함정 1: `pgrep -f` 는 자기 자신을 센다

`pgrep -f` 는 명령줄 전체를 매칭한다. ssh 가 보내는 래퍼는

```text
bash -c "... pgrep -f \"[s]ustained_load_test|...\" | wc -l"
```

이고, 이 명령줄에 패턴 문자열이 들어 있다. 대괄호 트릭
(`[s]ustained`)은 같은 명령줄에 괄호 없는 형태가 섞이면 무력해진다.

**양방향으로 틀렸다.**

| 상황 | 실제 | pgrep 보고 |
|---|---|---|
| 부하 실행 중 | 1개 | 0 (놓침) |
| 부하 없음 | 0개 | 2 (자기 셸을 셈) |

`/proc/PID/exe` 심볼릭 링크를 읽는 방식으로 바꿨다. 이것은 실제 실행
파일을 가리키므로 셸이 끼어들 여지가 없다.

```bash
n=0
for p in /proc/[0-9]*; do
  case "$(readlink "$p/exe" 2>/dev/null)" in
    *sustained_load_test) n=$((n+1)) ;;
  esac
done
```

### 함정 2: `cd DIR && setsid nohup ... &` 는 뜨지 않는다

같은 조건에서 두 형태를 비교했다.

| 형태 | 결과 |
|---|---|
| `ssh -n H "cd $DIR && setsid nohup ./prog ... &"` | **실행 안 됨** |
| `ssh -n H "setsid nohup $DIR/prog ... &"` | 실행됨 |

`&` 는 `cd && prog` 리스트 전체에 걸린다. ssh 가 명령을 보내고 즉시
끊는데, 백그라운드 서브셸이 `cd` 를 거쳐 `setsid` 에 닿기 전에 세션이
사라지면 죽는다. 절대경로를 쓰면 중간 단계가 없어 경합이 생기지 않는다.

**실패해도 아무 신호가 없다.** 종료 코드는 0 이고 stderr 도 비어 있다.
확인하지 않으면 "부하 없는 상태의 온도"를 15분 동안 측정하게 된다.

`run-thermal-comparison.sh` 는 원래 절대경로 형태를 쓰고 있어서
2026-08-11 열 측정은 영향을 받지 않았다. 다만 **띄운 뒤 실제로 도는지
확인하는 단계**를 추가했다.

### 공통 교훈

두 함정 모두 **실패가 성공처럼 보인다.** discuss.md §10 의 A 유형
(지표가 무엇을 세는지 확인하지 않음)과 같은 계열이다.

검사를 새로 만들면 **일부러 깨뜨려 보고 실제로 잡히는지 확인한다.**
이번에도 그 절차 덕에 발견했다. 통과만 보고 믿었다면 preflight 는
아무것도 걸러내지 못하는 채로 남았을 것이다.

## 2.29 S3 saturation sweep — ceiling 기준도 near-linear (2026-08-20)

각 노드 수의 진짜 처리량 상한을 concurrency sweep 으로 찾았다(S2 는 동일
부하 선형성, S3 는 최대 처리량 — 별개 실험). 45 run, 동결 `1da69d4`.

| Config | Ceiling @ conc | Speedup | Eff |
|---|---|---:|---:|
| 1N | 115.2 @ c32 | 1.00× | 100% |
| 2N | 232.0 @ c24 | 2.01× | 101% |
| 3N | **341.8 @ c32** | **2.97×** | **99%** |

- 곡선: 미포화(왕복 지연) → plateau(노드당 ~10-16 동시) → 과부하 살짝 하락.
  오류 0(큐가 흡수). SD ≤ 2.2.
- **S2(동일부하)와 S3(ceiling) 두 각도에서 near-linear 재확인.**
- 보고서: `docs/experiments/S3_SATURATION.md`, 원본: `results/saturation-20260820/`.

다음: S4 io_uring — payload-transfer(비-추론 지연의 94%) 비용 절감 비교.

## 2.28 gRPC baseline 30회 반복 — 재현 확인, baseline 동결 (2026-08-20)

1차 결과를 "재현된 결과"로 승격시켰다. 코드·설정 동결(bench `254d560`)
상태에서 1N/2N/3N 각 10회, 60초, **조건 순서 rotate**(시간·온도 변동 분산).
`scripts/run-grpc-baseline30.sh`. 원본·집계: `results/baseline-20260820/`.

### 결과

| N | Throughput Mean±SD | Speedup | Eff | p50/p99 ms | Err | Bal |
|---:|---:|---:|---:|---|---:|---:|
| 1 | 112.9 ± 0.5 | 1.00× | 100% | 68.0 / 116.3 | 0% | 0.00 |
| 2 | 229.0 ± 0.9 | 2.03× | 101% | 67.0 / 118.6 | 0% | 0.00 |
| 3 | **338.4 ± 1.1** | **3.00×** | 100% | 67.6 / 123.9 | 0% | 0.00 |

- **첫 측정 337.7 이 338.4 ± 1.1 로 재현.** SD 0.5~1.1 로 극히 작다.
- 30/30 active node 정확, invalid 0, 오류 0%, balance 0%p.
- saturation(115) 기준 3N efficiency 98%, 1N c8 기준 speedup 3.00×.

### TimingBreakdown 도 재현 (30회 p50 평균)

3N: network_to_node 17.11 + network_to_client 17.11 = 34.21 ms
  = non-inference overhead(36.34) 의 **94%**, E2E(58.83) 의 58%.
scheduler_queue/route 는 1N·3N 모두 ~0 — 스케줄러 병목 없음 재확인.
1N·3N 의 network 가 거의 같아(17.7 vs 17.1) 전송 시간은 노드 수 무관.

### 승격된 문장

"한 번 337.7" → **"3-node near-linear scaling 을 30회 반복 실험으로 확인
(338.4 ± 1.1 inf/s, speedup 3.00×, error 0%)."** gRPC baseline 동결.

다음: saturation sweep → (동결 유지) → io_uring 동일 조건 비교.

## 2.27 로컬 팬 baseline 재측정 — 오버헤드 27% → 28.8% 확정 (2026-08-20)

27% 의 기준값 157 이 팬리스(08-11/12)라 냉각 조건이 클러스터(팬)와 달랐다.
같은 팬 조건에서 로컬 sustained 를 다시 쟀다. king 노드를 중지하고 순수
로컬 `sustained_load_test`(gRPC 없음), INT8, governor=performance, 팬 ON.

```text
8스레드(worker 8, 클러스터 동일조건) 60초 × 3:  159.2 / 162.0 / 163.2 → 161.5
16스레드(saturation 확인):                       165.7
```

**확정: 오버헤드 = (161.5 - 115) / 161.5 = 28.8%** (냉각·worker·측정시간 통일).

### 발견 — 27% 는 냉각으로 무너지지 않았다

우려는 "팬이면 로컬이 157 보다 훨씬 높아 오버헤드가 크게 벌어진다"였다.
실제로는 팬 161.5 vs 팬리스 157.2 로 **차이가 작았다.** 이유:

**60초/30초 측정은 throttling 발현 전이다.** CPU throttling 은 300초에
-27% 로 나타난다(§2.24, discuss §12). 짧은 측정 구간에서는 팬이든 팬리스든
초기 처리량이 비슷하므로 냉각 조건의 영향이 작다.

→ **27% 는 냉각 때문에 무효가 아니라 28.8% 로 소폭 조정**됐다. 병목 위치
(페이로드 전송, §8·§2.26)는 애초에 냉각과 무관해 그대로다. **가장 단단한
사실 두 개는 흔들리지 않았다**: (1) 확장 효율 ~98% 선형, (2) non-inference
latency 의 94% 가 페이로드 전송.

**두 측정량을 곱하지 않는다.** throughput loss 28.8%(처리량)와 latency
breakdown 94%(지연 구성비)는 다른 축이다. "28.8% 의 94%" 는 틀린 곱이다.
정확한 표현: 클러스터 단일노드 처리량은 로컬 대비 28.8% 낮았고, 별도
latency breakdown 에서 non-inference latency 의 94% 가 payload-transfer 였다.

### 남은 것 (별도 조건)

- **지속 부하(300초) 오버헤드**: 팬 이득이 커지면 오버헤드가 더 벌어질 수
  있다. throttling 이 로컬(sustained)과 클러스터(노드)에 어떻게 다르게
  걸리는지가 다음 질문. 단, 이건 "짧은 측정 28.8%"와 별도 축이다.
- saturation: 16스레드(165.7) > 8스레드(161.5) 라 worker 8 이 로컬 최대는
  아니다. 클러스터 노드가 worker 8 이라 동일조건 비교는 8스레드가 맞다.

## 2.26 TimingBreakdown 첫 실측 — 오버헤드는 페이로드 전송 (2026-08-20)

bench 를 확장해 응답의 `Timing`(proto) 11단계를 전부 수집하게 했다(기존엔
`inference_us` 하나만). 27% 노드당 오버헤드를 단계로 쪼갠 첫 실측이다.

측정: 3노드 / c24 / 10초 / Active Cooling / gRPC.

```text
단계 (p50 ms)
  scheduler_queue      0.00
  scheduler_route      0.00
  network_to_node     17.16   ┐ 페이로드 전송
  node_queue           0.02   │
  decode/preprocess    0.00   │
  npu_input            0.00   │
  inference (NPU)     22.49   │ ← 실제 추론
  postprocess          0.00   │
  network_to_client   17.16   ┘
  end_to_end          58.99
```

**발견: 노드당 오버헤드의 정체는 페이로드 네트워크 전송이다.**
payload transfer = `network_to_node + network_to_client` = 34.32 ms.
protobuf 직렬화도, 스케줄러 큐(~0)도, 노드 큐(~0)도 아니다. 1.17 MiB
입력·출력을 2.5G 로 실어 나르는 시간이 대부분이다.

**분모를 명확히 구분한다(혼동 방지):**

```text
payload transfer / E2E latency            = 34.32 / 58.99 = 58%
payload transfer / non-inference overhead = 34.32 / 36.50 = 94%
  (non-inference overhead = E2E - inference = 58.99 - 22.49 = 36.50 ms)
```

정확한 표현: **"노드당 오버헤드(=E2E−inference)의 94%가 페이로드 전송"**,
그리고 "E2E 지연의 58%가 페이로드 전송, 38%가 순수 추론".

→ io_uring·zero-copy·JPEG 입력·후처리(NMS)로 응답 축소 가 겨냥할 지점이
**네트워크 전송 경로**임이 실측으로 확정됐다.

### 계측의 한계 (정직하게)

- gRPC **직렬화 시간은 단독 분리 안 됨** — proto `Timing` 에 별도 필드가
  없다. 재려면 계측점 추가 필요. 현재 잔차(~2ms)에 섞여 있다.
- bench↔스케줄러는 **같은 호스트(loopback)** 라 client→scheduler 는 ~0.
  실제 네트워크는 스케줄러↔노드 2.5G 구간뿐이다.
- **냉각 조건 미확정:** 이 분해는 클러스터 내부라 냉각 무관하게 유효하지만,
  "27%" 자체는 팬리스 157 vs 팬 클러스터 115 라 아직 확정 아님(§2.24).
- c24(동시 24) 값이라 `network_*` 는 concurrency 의존. 단일 요청 전송 시간은
  낮은 concurrency 에서 따로 봐야 한다.

작업용 집계표는 `results/NPUForge_Benchmark_Result_Workbook.md` §8 (로컬 전용).

## 2.25 S2 확장성 첫 측정 — 확장 효율 98%, 노드당 오버헤드 발견 (2026-08-20)

model_file 버그 수정 후 preflight 통과 상태에서 1/2/3노드 확장성을 처음
쟀다. **정식 근접(preflight 통과·30초·조건 통제)이나 단일 run·
--with-inference 스킵이라 확정 수치 아님.**

측정: INT8, want_float=0, governor=performance, **Active Cooling(노드마다
전용 팬, 측정 시작부터)**, 스케줄러(.9) 경유 gRPC, round-robin. 노드 축소는
프로세스 중지(jack→queen 순), 사이 cooldown.

> ⚠️ **냉각 조건 정정 (2026-08-20 사후).** 이 세션의 모든 측정은 팬 장착
> 상태였다. 처음엔 "cold/팬리스"로 적었으나 실제로는 시작부터 큰 팬이
> 달려 있었다. 이것이 27% 계산에 영향을 준다 — 아래 결론 참조.

### 노드당 동일 부하 (concurrency = 8 × 노드수)

| 구성 | 처리량 | 분배 |
|---|---:|---|
| 1노드 c8  | 111.6 inf/s | king 100% |
| 2노드 c16 | 228.7 inf/s | 50/50 |
| 3노드 c24 | 337.7 inf/s | 33/33/33 |

오류율 0%, round-robin 이 정확히 균등 분배. 3노드/1노드 = **3.03배**.

### 1노드 concurrency 스윕 — 상한 ~115

| c8 | c16 | c32 |
|---:|---:|---:|
| 111.6 | 114.0 | 115.1 |

concurrency 를 올려도 **~115 inf/s 에서 포화**. 이것이 스케줄러 경유
단일 노드 상한이다.

### 두 가지 결론

**1. 확장 효율 ~98% (거의 선형).** 1노드 포화 115 기준 3노드 337.7 =
2.93배. 데이터 병렬(`adrs/001`)이 성립하고 스케줄러가 3노드 동시에도
병목이 아니다. `adrs/003` 의 단일 스케줄러가 이 규모에서 충분함을 실측.

**2. 클러스터 노드 상한 115 < 로컬 sustained 157 (-27%).** 왕복 p50 69ms
인데 노드 보고 추론은 24~28ms — **40ms+ 가 스케줄러 gRPC 경유 오버헤드**
(직렬화 + 1.17MB 입력·출력 전송 + 큐/라우팅). 확장은 선형인데 노드당
절대 상한이 네트워크·스케줄링에 깎인다.

> 프로젝트 핵심 질문 "6 TOPS 세 대는 정말 18 TOPS 가 되는가" 의 첫 실측 답:
> **클러스터 기준 2.93배(98%).** 병목은 확장이 아니라 노드당 오버헤드다.
> 이 27% 가 어디서 오는지는 `TimingBreakdown` 단계 분해로 다음에 쪼갠다.

### 마이너 이슈

- bench run 파일명이 전부 `-n3` — run_id 의 노드 수가 측정 시점 활성이
  아니라 **초기 ListNodes(등록) 기준**이다. jack/queen 중지 후에도 스케줄러
  등록이 남아 3으로 찍혔다. 실제 노드 수는 결과의 분배로만 확정된다.
  run_id 를 측정 종료 시점 활성 노드로 잡는 것이 옳다.
- 노드 축소를 프로세스 kill 로 했다. drain RPC 가 있으면 진행 중 요청을
  흘려보내고 깨끗이 뺄 수 있다(`adrs/027`). S2 정식에서 검토.

### 정식 S2 에 남은 것

반복 run(분산), 팬 조건(S0-B), --with-inference, concurrency 스윕 전체,
2노드 조합(king+queen vs king+jack), TimingBreakdown 오버헤드 분해.

## 2.24 M3 첫 3노드 클러스터 실동작 (2026-08-20)

인프라·빌드·IP 고정이 끝나 실제 3노드 추론 클러스터를 처음 띄웠다.
스케줄러(server .9) + king/queen/jack, 실 gRPC.

### 배포

- king 에서 노드 빌드(`cargo build --release -p npuforge-node --features rknn`,
  1m37s, 24MB) → 개발 PC 경유로 queen/jack 배포
- 모델: INT8 `model.rknn`(dba155d2) + `model.toml` 3보드, 해시 검증 통과
- 스케줄러: server 에서 `scheduler.example.toml`(policy round-robin), 50051

### 예비 벤치 (정식 아님)

preflight 미실행, Active Cooling(팬 ON), 12초. **조건 통제 전이라 확정
수치로 쓰지 않는다.**

| 동시성 | 처리량 | 노드 추론 p50 | 왕복 p50 | 분배 |
|---:|---:|---:|---:|---|
| 6  | 146.3 inf/s | 14.4 ms | 39.8 ms | 33.3% 균등 |
| 24 | 336.4 inf/s | 22.2 ms | 67.7 ms | 33.3% 균등 |

오류율 0%, round-robin 이 세 노드를 정확히 3등분. 단일 노드 INT8 상한
157 대비 c24 에서 약 2.1배 — **다중 노드 확장이 실제로 일어난다.** 정식
S2 는 preflight + concurrency 스윕 + 지속시간으로 별도.

### 이번에 걸린 버그 3개 (전부 "성공처럼 안 보이는" 실패라 빨리 잡힘)

**1. model.toml `model_file` 상대경로가 로딩 실패로 이어진다 (코드 버그, 미수정)**

`main.rs` 는 `load_spec` 이 만든 절대경로 `PathBuf` 로 sha256 을 검증하는데
(`:77`), 정작 `backend.load_model(&spec)`(`:81`)에는 `spec.model_file`(원본
상대경로 `"model.rknn"`)을 넘긴다. 백엔드가 CWD 기준으로 파일을 찾아
`rknn_init` 전에 read 실패 → `status=-2`(read_file 실패도 rknn_init 실패도
같은 NPF_RKNN_ERR_MODEL_LOAD 라 구분 안 됨). RKNN 은 stderr 를 안 남긴다.
→ **수정 완료 (2026-08-20).** `main.rs` 가 `load_model` 직전에
`spec.model_file` 을 `load_spec` 이 해석한 절대경로로 교체한다. 상대경로
`model.toml` 로 3노드가 정상 로딩·등록되고 벤치 재검증(c24 336 inf/s, 오류
0%)까지 통과했다. real_device 테스트는 spec.model_file 에 절대경로를 직접
넣어서 이 버그를 못 잡았다 — 상대경로 케이스 회귀 테스트가 없다.

**2. 죽은 노드가 NPU 컨텍스트를 안 놓아 재기동이 status=-2 로 실패**

노드를 죽였다 바로 다시 띄우면 rknn_init 이 실패한다. `pkill -9` +
수 초 대기로 확실히 정리해야 뜬다. 노드의 graceful shutdown(ContextPool
drop → rknn_destroy)이 SIGTERM 에서 확실히 도는지 점검 필요.

**3. `pkill -f npuforge-node` 가 자기 셸을 죽였다 — ADR-017 함정 1 재현**

정리 명령의 셸 명령줄에 패턴 문자열이 들어 있어 pkill 이 자신을 죽이고
이후 명령이 조용히 안 돌았다. **배포/정리는 `pkill`(comm, `-f` 없이)로.**
내가 문서에 적어 둔 함정에 그대로 걸렸다.

현재 3노드 + 스케줄러는 실행 유지 중. server 방화벽 50051/8080/9090 은
런타임 규칙(재부팅 시 사라짐).

## 2.23 네트워크 개편 — 10G aggregation 구축·실측 (2026-08-20)

§2.22 에서 대기하던 장비가 들어와 M3 네트워크를 구성했다. **차단 요소가
전부 해소됐다.**

### 도입한 것

| 장비 | 사양 |
|---|---|
| 스위치 | **NEXI NS-S25G10G-N** — 2.5G×4 + 10G×2, 전부 RJ45 |
| 서버 | Xeon E5-2630L ×2 (24T) / 16GB / Rocky 9.4 / x86_64 |
| 서버 NIC | `enp4s0` 10GBASE-T (DAC/SFP+ 아님) |

포트 배선: 1=인터넷(ipTIME), 2=king, 3=queen, 4=jack, 5=개발PC(10G포트지만
NIC 1G), 6=server(10G).

### 겪은 것

1. **보드 IP 가 통째로 바뀌었다.** DHCP 라 `.12/.16/.33` → `.3/.4/.5` 로
   재할당됐고, `~/.ssh/config` 의 별칭이 낡아 세 노드 전부 접속 실패했다.
   `adrs/019-ssh-alias-not-ip.md` 가 경고한 상황 그대로다. config 갱신 +
   `npuforge-server` 별칭 추가로 복구.

2. **서버가 10G IP 를 못 받았다.** 원인은 케이블·스위치가 아니라
   NetworkManager 였다 — `enp4s0` 이 `UP LOWER_UP`(링크 붙음)인데 연결
   프로파일이 없어 DHCP 를 안 돌렸다. `nmcli device connect enp4s0` 로
   즉시 `192.168.123.9` 획득. Rocky 9 에 새 NIC 꽂으면 나오는 전형적 상황.

3. **원격 iperf3 기동이 안 떴다** — `setsid nohup iperf3 ... &` 가 조용히
   실패(`adrs/017` 함정 2). 절대경로 형태로 재기동해 해결.

### 실측

```text
server enp4s0        10000 Mb/s full     ethtool
단일 king→server     2.34 Gbps           iperf3   (2.5G 실효 상한)
3노드 동시 →server   각 1.70, 합 5.11 Gbps  nc     (세 스트림 균등 유지)
```

세 스트림이 균등 유지 → **서버 10G aggregation 이 병목이 아니다.** INT8
3노드 목표 RX 4.60 Gbps 를 여유 있게 수용. 상세 판단은
`adrs/014-10g-aggregation-separate-scheduler.md` 구축 결과 절.

### 정리한 것

측정용 방화벽 런타임 규칙(5201-5210)·임시 리스너·파일은 측정 후 전부
제거했다. 서버 영구 상태는 바꾸지 않았다.

### 남은 것

- **IP static 고정** — server(.9) 완료. 보드 3대는 pi sudo 비번 대기.
  라우터 예약 대신 호스트 static 채택 (`infrastructure.md` §2.3)
- INT8 모델 queen·jack 배포
- server gRPC 방화벽 개방

`dealer`(옛 스케줄러, 노트북 .14)는 응답 없음 — 제거됐다. 역할은 server 로 이관.

### IP 고정 방식 결정 (2026-08-20)

라우터(ipTIME) DHCP 예약이 아니라 **호스트 NetworkManager static** 을 택했다.
라우터가 바뀌어도 설정이 호스트에 남아 측정 재현성이 낫고, 현재 IP 를 그대로
고정하므로 SSH 도 안 끊긴다. server 는 root 라 즉시 적용했고
(`nmcli con mod enp4s0 ipv4.method manual ...`), 보드는 `pi` 계정의 sudo 비번이
있어야 한다. 남는 리스크(DHCP 풀 충돌)는 `infrastructure.md` §2.3.

### 스케줄러 빌드 경로 결정 (2026-08-20)

옛 dealer 는 Rust 가 없어 미정이었다. server 로 확정한다.

- toolchain `stable`, MSRV 1.85. server dnf 의 rust/cargo **1.92** 로 충분
- Windows→Linux 크로스빌드는 링커 문제로 회피. **server 24스레드 네이티브**가
  빠르고 확실하다
- server 에 `rust cargo gcc gcc-c++ protobuf-compiler git` 설치
  (tonic-build 0.12 가 protoc 요구). github 접근 OK, foxden 직접 불가라
  소스는 `git archive` tarball 을 scp 로 전송
- 노드(aarch64)는 종전대로 king 네이티브 빌드. 스케줄러(x86_64)만 server

**함정: protoc 가 Rocky 9 기본 리포에 없다.** `dnf install protobuf-compiler`
가 "No match" 로 실패하고, `dnf install -y a b c ...` 는 하나만 못 찾아도
전체가 실패해 rust 까지 설치 안 됐다. **CRB 리포**를 켜야
(`dnf config-manager --set-enabled crb`) protobuf-compiler 가 잡힌다.

**빌드 검증 완료 (2026-08-20).** `cargo build --release -p npuforge-scheduler
-p npuforge-bench` 성공.

```text
cargo 1.92.0 / rustc 1.92.0 / libprotoc 3.14.0 / gcc 11.5.0
npuforge-scheduler  25 MB
npuforge-bench      19 MB
config 파싱·기동 정상 (--config configs/scheduler.example.toml)
```

스케줄러 빌드 경로의 불확실성이 사라졌다. 실제 배포·기동은 M3 착수 때.

## 2.22 작업 중단 시점 상태 (2026-08-12, 10G 스케줄러 구성 대기)

> **후속: §2.23 (2026-08-20) 에서 이 대기가 해소됐다.** 아래는 중단 시점 기록이다.

M3 실장비 측정은 10G aggregation 구성이 있어야 시작할 수 있다. 그때까지 작업을
멈추므로 재개에 필요한 상태를 남긴다.

### 보드 상태

| 항목 | king | queen | jack |
|---|---|---|---|
| SSH 별칭 | `npuforge-k` | `npuforge-q` | `npuforge-j` |
| IP | 192.168.123.12 | .16 | .33 |
| CPU governor | `performance` (영구화) | 동일 | 동일 |
| 유휴 NPU 온도 | 37.9°C | 37.9°C | 38.8°C |
| 잔존 부하 프로세스 | 없음 | 없음 | 없음 |

세 노드의 커널·`librknnrt.so`·RKNPU 드라이버·모델 해시가 모두 일치한다.
`preflight-check.sh --with-inference` 전항목 통과 상태로 멈췄다.

### 보드에 설치한 것 (원래 없던 것)

| 노드 | 추가 | 이유 |
|---|---|---|
| `king` | Rust 툴체인 (rustup) | `npuforge-node --features rknn` 네이티브 빌드. 크로스 컴파일은 aarch64 sysroot 와 RKNN SDK 를 함께 맞춰야 해 실패 지점이 많다 |
| `king` | `protobuf-compiler` | `npuforge-proto` 빌드 |
| 3노드 | `strace` | syscall 분해 측정 |
| 3노드 | `/etc/systemd/system/npuforge-cpu-governor.service` | governor 영구화 |
| 3노드 | `~/npuforge-rknn-test/` C 도구들 | 측정 도구 |

**`king` 에만 Rust 가 있다.** 환경 일치가 깨진 항목이지만 빌드 전용이고
런타임에 영향이 없다. 바이너리는 한 번 빌드해 세 노드에 배포한다
(모델과 같은 원칙).

### 확정된 수치 (governor=performance 기준)

| 항목 | 값 |
|---|---|
| FP16 8스레드 지속 처리량 | **84.3 inf/s** (지연 94.5 ms) |
| INT8 8스레드 지속 처리량 | **157.2 inf/s** (지연 50.8 ms) |
| INT8 / FP16 배율 | **1.86배** |
| 추론당 커널 ioctl | 76회 (FP16·INT8 동일) |
| 노드 간 열 편차 | 5.6°C, **NPU** throttling 없음 |
| CPU thermal 강등 | A72 2208→816MHz / A53 2016→600MHz (부하 60초 후) |
| 지속 부하 NPU 온도 | 67.5~75.8°C (ondemand 기준, 15분) |
| `want_float=0` 효과 | INT8 +17.3% / FP16 +15.7%, 출력 4분의 1 |

이전 문서의 79.0 / 146.2 inf/s 는 `ondemand` 기준이다. discuss.md §11.

### 재개할 때 먼저 할 일

1. `bash scripts/preflight-check.sh --with-inference`
   - 보드가 재부팅했을 수 있다. governor 는 유지되지만 `boot_id` 는 바뀐다
   - 실패하면 그 항목부터 해소한다. 통과 전에 측정하지 않는다
2. 2.5G/10G 스위치 연결 후 추론망 IP 대역을 정하고 `advertise_address` 갱신
   (스케줄러는 10G SFP+ 업링크. `02-HARDWARE-SETUP.md` §3.3.2)
3. `npuforge-node` 를 `king` 에서 빌드해 세 노드에 배포
4. S2 확장성 실험 설계 재검토 — INT8 노드당 **1.545 Gbps**, 3노드
   **4.636 Gbps**. 출력은 입력의 3.96배라 RX 가 최대 18.4 Gbps 다.
   **10G aggregation 이 필요하다.** `02-HARDWARE-SETUP.md` §3.3.2
   (여기 처음 적은 1.43/4.3 은 Gbps 를 2진 접두로 계산한 오류였다)

### 재개 시 주의할 함정 (이번에 겪은 것)

- 보드 접속은 **IP 가 아니라 별칭**으로 한다 (§2.20)
- 원격 백그라운드 실행은 **절대경로**로 하고 실제로 떴는지 확인한다 (§2.21)
- 프로세스 확인은 `pgrep -f` 가 아니라 `/proc/PID/exe` 로 한다 (§2.21)
- ssh 안에서 heredoc + sudo 중첩은 조용히 실패한다. 파일은 `scp` 로 보낸다
- 열 비교는 **부하 프로파일이 같을 때만** 한다 (§2.19)

## 2.18 RTC가 유지되지 않는다

부팅 이력 조회에서 별개 문제를 발견했다.

```text
queen  현재 부팅 시작 시각  Tue 2025-11-25 18:16:31 UTC
jack   현재 부팅 시작 시각  Tue 2025-11-25 18:16:31 UTC
king   현재 부팅 시작 시각  Fri 2025-07-11 18:52:59 UTC
```

세 노드 모두 **부팅 직후 시스템 시각이 과거의 고정값**이다. RTC 배터리가 없거나 동작하지 않아 전원이 끊기면 시계가 초기화된다. NTP가 동기화되기 전까지 로그 타임스탬프가 틀린다.

### 영향

- 부팅 직후 기록된 로그의 타임스탬프를 신뢰할 수 없다
- 노드 간 이벤트 순서를 맞출 수 없다 (`02-HARDWARE-SETUP.md` §10)
- 벤치마크 결과에 잘못된 시각이 기록될 수 있다

### 조치

`chrony`를 활성화하고, **동기화 완료를 확인한 뒤에 측정을 시작**해야 한다. `scripts/fix-node-consistency.sh`의 `chrony` 단계에 포함되어 있으나 아직 실행하지 않았다.

벤치마크 스크립트는 실행 전 다음을 확인하도록 한다.

```bash
chronyc tracking | grep -E "Leap status|System time"
# Leap status : Normal 이어야 하며 Not synchronised 면 대기
```

각 노드가 자신이 측정한 duration만 응답에 담고 절대 시각을 비교하지 않는 설계(§10.1)라 측정값 자체는 영향받지 않는다. 문제는 **로그 상관 분석**이다.

---

# 3.5 문서 재구성 (2026-08-07)

작업 이력이 길어져 두 문서로 나눴다.

| 문서 | 역할 |
|---|---|
| `board-worklog.md` (이 문서) | **시간순 작업 이력.** append 전용. 왜 그렇게 했는지 |
| `infrastructure.md` | **현재 상태 스냅샷.** 지금 어떤 상태인지 |
| `environment-matrix.md` | **버전·해시 고정.** 재현에 필요한 값 |

"지금 상태가 어떤가"를 알려면 `infrastructure.md`를, "어쩌다 이렇게 됐나"를 알려면 이 문서를 본다.

---

# 4. PC 측 변경 사항

보드가 아니라 개발 PC(`192.168.123.26`)에 적용한 내용이다.

| 날짜 | 항목 | 내용 |
|---|---|---|
| 2026-08-07 | SSH 키 | `~/.ssh/id_ed25519_npuforge` 생성 (passphrase 없음, 자동화용) |
| 2026-08-07 | SSH config | `npuforge-k` / `npuforge-q` / `npuforge-j` 별칭 추가. 기존 config는 `~/.ssh/config.bak.*`로 백업 |

## 4.1 SSH 별칭

```text
npuforge-k → pi@192.168.123.12  (king)
npuforge-q → pi@192.168.123.16  (queen)
npuforge-j → pi@192.168.123.33  (jack)
```

별칭은 `npuforge-k/q/j`로 유지하고 hostname만 `king/queen/jack`으로 두었다. 별칭을 바꾸면 이미 작성한 스크립트를 모두 수정해야 하므로, 추론망 구성 시 한 번에 정리한다.

## 4.2 sudo 실행 패턴

`pi` 계정은 sudo에 비밀번호를 요구한다. 자동화에서는 다음 형태를 사용한다.

```bash
ssh npuforge-k 'printf "$NPUFORGE_SUDO_PASS\n" | sudo -S -p "" <command>'
```

§2.4에 기록한 파이프 충돌 함정에 주의한다.

**개선 여지.** 벤치마크 자동화에서 sudo 호출이 늘어나면 특정 명령에 한해 NOPASSWD sudoers 규칙을 두는 편이 낫다. 다만 이는 권한 확대이므로 별도 승인 후 진행한다.

---

<a id="todo"></a>

# NPUDure status

*[한국어 원문](TODO.ko.md)*

- Last updated: **2026-08-21**
- To the talk: **D-99** (2026-11-28)
- Feature freeze: 2026-11-15

> This document exists to see **what has to be done now** at a glance.
> Why it was done that way is in `board-worklog.md`, the values are in
> `environment-matrix.md`, and the state is in `infrastructure.md`.

---

# ▶ Current state: **the measurement lineage is closed** (2026-08-21)

Everything from S2 through S3.9b and S0-D is closed. **421 measurements, error
rate 0 throughout.** What remains is presentation material (figures).

| Lineage | Status | Conclusion |
|---|---|---|
| **Transport** | closed | operating point = **2 connections per node @ c12**. 3N 387.2 inf/s (+13.3%) |
| **Scaling** | closed | 3N **2.86× (95.3%)**. The loss **shows up in the tail** — p50 flat, p99 +36% (S3.9a). The micro-mechanism was not isolated |
| **Sustained load** | closed | under active cooling, short-run = sustained (−1.9%) |
| **Policy** | closed | RR is vulnerable to heterogeneity; adaptive gives tail −37%. Default **stays `ect`** |
| **io_uring (S4)** | **refuted** | what it targets is 1% of transport cost. CPU is not the constraint (S3.9b) |

```text
local direct 161.5   operating point 135.5   residual gap 26.0 inf/s = 16.1% of direct
  -> looks like path latency rather than CPU cost (out of scope, observation only)
```

## Operating the cluster

```bash
# connect + preflight
for h in npuforge-k npuforge-q npuforge-j npuforge-server; do ssh $h hostname; done
bash scripts/preflight-check.sh --with-inference

# restarting a node - pkill by comm (never -f, ADR-017), log redirect mandatory
ssh npuforge-k 'pkill -9 npuforge-node; sleep 3;   setsid nohup ~/npuforge/npuforge-node.s36 --config ~/npuforge/node.toml   >>~/npuforge/node.log 2>&1 & disown'
# helper: npuforge_restore_cluster (scripts/lib/remote.sh)

# recount the run total
bash scripts/count-runs.sh
```

> **The two harness invariants** (`experiments/README.md` §4.12) — new harnesses
> keep both.
> 1. Verify shared-resource state at the shared resource
>    (`npuforge_assert_cluster_free`).
> 2. Do not treat the results path as an overwritable scratch directory.

## The M3 topology (settled and measured 2026-08-20)

```text
        server 192.168.123.9  (Xeon x2 24T / 16GB / Rocky 9.4)
                    |
                  10GbE          <- aggregation. 10G full measured
                    |
          NEXI NS-S25G10G-N  (2.5G x4 + 10G x2)
              |-- 2.5G -- king  .3
              |-- 2.5G -- queen .5
              \-- 2.5G -- jack  .4
```

Worker links are 2.5G and **only aggregation is 10G.** The old `dealer` (a
laptop) was removed and the scheduler role moved to `server`.
`infrastructure.md` §1.

> **Static IPs done.** The rework changed the board IPs wholesale
> (`.12/.16/.33` → `.3/.4/.5`) and left the SSH aliases stale. All four are now
> pinned host-side static (`manual`) (§1.1, `infrastructure.md` §2.3).

---

# ▶ The policy lineage — what closed and what was deferred (2026-08-21)

**Closed.** RR is vulnerable to heterogeneity, and load-aware scheduling with
state freshness fixed improves RR's tail markedly (p99 −37%). Both LQ and ECT
work, with no regression. **The default stays `ect`.**

**Deferred (future work).** Whether ECT beats LQ under strong heterogeneity is
undetermined. But **that outcome does not change the central conclusion** — the
core is "load-aware scheduling absorbs heterogeneity", and it holds either way.

S0-D's calibration made that question **reproducible.** An answer takes 40
minutes whenever wanted (fan on, no preheat needed).

```text
king CPU cap    1200   1008    816    600
node latency spread 1.33x  1.79x  2.26x  3.93x     <- 816 reproduces S0-A (2.4x)
```

→ [`experiments/S0_D_CAPACITY_HETERO.md`](#experiments-s0-d-capacity-hetero) §6

---

# ▶ S3.9b complete (2026-08-21) — **S4 io_uring cancelled/shelved**

```text
Question   Does io_uring recover the remaining 16.1%?
Answer     No. What it targets (syscall entry) is 1% of transport cost, and 8%
           under the most generous assumption. And CPU is not the constraint.
```

| | Value |
|---|---|
| Transport cost | **16.35 CPU-ms/req** (user 9.37 / kernel 6.99) |
| Network syscalls | ~165 per request × 1 µs = **0.165 ms = 1.0%** |
| Board CPU | **48.9% idle**, the hottest core cpu0 at 78.8% busy (softirq) |
| cpu0 softirq | spread with RPS in S3.5 §4.3 → **−0.2% null** |

**CPU-ms/req is a cost, not a constraint.** Reducing usage of an unsaturated
resource does not raise throughput.

The large term was recorded separately — **user time exceeds kernel time**
(serialization and user-space copies are 57% of transport cost). But per the
third branch of the pre-registered rule, **we stop here**: as long as CPU is not
the constraint, there is no guarantee reducing this raises throughput.

An out-of-scope observation: the gap looks like **path latency** rather than CPU
cost (of the +37.3 ms of latency, node CPU is 16.35 ms, and the 1.2 MB payload
round trip alone is 8.2 ms). If there is a lever, it is **payload size**, not
io_uring.

→ [`experiments/S3_9B_NODE_RESIDUAL.md`](#experiments-s3-9b-node-residual)

---

# ▶ (the completed plan) S3.9b — node-side residual cost profiling

## The question (narrow)

> **In the residual gap between 161.5 and 135.5, do node-side serialization,
> copy and syscall costs account for a meaningful share?**

```text
local direct 161.5   operating point 135.5   gap 26.0 inf/s = 16.1% of direct
                                             (1 - 135.5/161.5)
```

> **The 13.2% that had been circulating is wrong.** That percentage came from
> 140.1 (S3.6 C), and 140.1 is a **c32 = overload region** measurement unusable
> for operating decisions (README §4.1). Paired with an operating-point number,
> it mixed two lineages.

**The objective is not to explain the whole gap.** S3.9a separately surfaced the
scale-out tail/TCP cost, so there is no reason a node-side profile has to account
for all 26 inf/s. Whatever is not explained stays unexplained.

## The verdict

| Result | Decision |
|---|---|
| syscall and copy are **large enough** | **proceed to S4 io_uring** |
| **small** | **cancel/shelve S4** |
| **some other term is large** | record that term only. **If it is outside the core scope, dig no further** |

The third row matters. Even if the profile points at an unexpected term, chasing
it is not this experiment's job. Record it and stop if it is out of scope.

## The harness invariants (established 2026-08-21)

Kept without exception when writing a new harness. Both came from real
incidents.

1. **Verify shared-resource state at the shared resource.**
   `npuforge_assert_cluster_free` — do not start if `npuforge-bench` is running
   on the server. Local process observation lies depending on the platform.
2. **Do not treat the results path as an appendable/overwritable scratch
   directory.** Stop if the existing directory is not empty. Distinguish with
   `NPUFORGE_SUFFIX`.

---

# 0. At a glance

| Area | Status |
|---|---|
| Software (M0) | ✅ done — workspace, common, mock backend, policy engine, CI |
| Hardware infrastructure | ✅ done — thermal spread 5.6 °C, no **NPU** throttling. But **the CPU is downgraded** (fans are for the S0-B comparison) |
| RKNN verification | ✅ backend implemented, context sharing risk confirmed by measurement |
| Model conversion | ✅ FP16 and INT8 done, accuracy verified |
| gRPC communication (M2) | 🟡 nearly done — wiring, retries and Mock cluster verified; metrics remain |
| Benchmarks (M3) | ✅ **done (2026-08-21)** — S2, S3, S3.5–3.9b, S0-A–D, **421 runs / error rate 0** |
| Dashboard (M6) | ⬜ not started |

**M3 blockers — all resolved (2026-08-20)**

| # | Item | Status |
|---|---|---|
| 1 | 2.5G/10G switch | ✅ NEXI NS-S25G10G-N |
| 2 | A server with a PCIe slot | ✅ Xeon x2 / 16GB / Rocky 9.4 (.9) |
| 3 | 10G NIC + cable | ✅ `enp4s0` 10GBASE-T, 10G full measured |
| 4 | ~~switch to `want_float=0`~~ | ✅ 2026-08-12 |

**Remaining work (as of 2026-08-21)**

| Item | Scale | Why |
|---|---|---|
| ~~more figures for the talk~~ | ✅ **done (2026-08-21)** | 7 added — `scripts/make-experiment-figures.py`. Paths in handoff §5 |
| Prometheus metrics (the M2 remainder) | — | the last piece of the gRPC communication item |
| Dashboard (M6) | — | not started |
| Move to systemd | — | draft at `scripts/npuforge-node.service.in`. Together with `pkill`→`systemctl stop` |
| Regenerate queen and jack's SSH host keys | — | the two boards share a key and cannot be told apart |

> **No more measurement is needed.** Before starting an additional experiment,
> read `experiments/README.md` §2 (the exclusion table) and §7 (the open list)
> first — to check whether the candidate is already excluded or conditionally
> open.

---

# 1. Immediate tasks

## 1.1 User tasks (physical, purchasing)

- [x] **Make the three boards' placement uniform** — concluded as needing no
  action (2026-08-11)
  - The 19 °C gap **did not reproduce** under controlled re-measurement
  - 15 minutes of concurrent 8-thread load: king 75.8 / queen 70.2 / jack
    74.8 °C (spread 5.6 °C)
  - Never exceeded 90 °C, no NPU clock drop (all 928 samples at 950 MHz)
  - Throughput spread 3.5% (80.5 / 77.7 / 77.8 inf/s)
  - The earlier 19 °C is judged to have been inflated by a load profile
    difference (sweep vs fixed, a 6-minute head start)
  - Details: `board-worklog.md` §2.19
- [ ] **A fanless (S0-A) cluster measurement** — today's baseline is active
  cooling (condition B). Settling the 27% requires measuring condition A over
  the same gRPC path too (§9)
- [x] **Install three identical fans** (2026-08-20) — 120 mm 5V USB, one per
  node (larger than the board). All measurements on 2026-08-20 were under this
  active cooling (condition B)
- [x] **2.5G/10G switch** (2026-08-20) — NEXI NS-S25G10G-N (2.5G×4 + 10G×2)
- [x] **Obtain the scheduler server** (2026-08-20) — Xeon E5-2630L ×2 / 16GB /
  Rocky 9.4 (.9)
- [x] **10G NIC + cable** (2026-08-20) — the server's onboard 10GBASE-T, 10G
  full measured
- [x] **Static IPs** (2026-08-20) — all four pinned via host NetworkManager
  static (handled as development work). An ipTIME router reservation is
  optional; if done, use the table below.
  ```text
  king 22-94-FF-34-46-B1 ->.3   jack 62-CE-3B-B6-E4-41 ->.4
  queen 7E-D8-D7-40-45-82 ->.5  server 6C-B3-11-13-2F-38 ->.9
  ```
- [x] **Decide the calibration image approach** — 200 COCO val2017 images
  adopted (2026-08-11)
  - Deterministic selection with `tools/model-converter/fetch_calibration.py`
    (fixed seed)
  - The images are not put in the repository; only the manifest (licensing)

## 1.2 Development tasks

- [x] `preflight-check.sh` — a hard-failing check before benchmarks (2026-08-11)
  - alias↔hostname, matching kernel/RKNN/driver/model hashes
  - governor, idle temperature, input voltage, residual load, NTP, session count
  - `--with-inference`: whether the three boards give the same answer to the same
    input (the §9 lesson)
  - Negative tests confirm it actually detects (a swapped model, residual load)
- [x] Record `boot_id` — detect and invalidate a reset during a run
  - The node reports it in the heartbeat; the scheduler warns on a change
    (implemented in M2)
- [x] Extend benchmark telemetry — collect temperature, voltage and boot_id via
  the `ListNodes` RPC
  - Querying via heartbeat has the scheduler record those values as observations
    and overwrite its state. A read-only RPC was added separately.
- [ ] **Regenerate queen and jack's SSH host keys** — they are identical and
  cannot be told apart cryptographically. A changed IP attaches you to the wrong
  board without a warning. With DHCP the IPs do change (as happened in the
  2026-08-20 rework), so this must not be left alone.
  ```bash
  ssh npuforge-j 'sudo rm -f /etc/ssh/ssh_host_* &&     sudo ssh-keygen -A && sudo systemctl restart ssh'
  ssh-keygen -R npuforge-j   # clean up known_hosts on the PC
  ```
- [x] **Switch to `want_float=0`** (2026-08-12) — the `[worker] want_float`
  setting
  - blob v2 carries `scale` and `zero_point`. Real-board dequantization verified
    to a maximum error of 9.5e-7
  - Throughput **INT8 +17.3% / FP16 +15.7%**, output size a quarter
- [ ] **A per-request latency raw dump option in the bench** (need confirmed
  2026-08-20)
  The bench currently writes only summary percentiles per run to JSON. So p95/p99
  in tables pooling several runs are **the average of run-level percentiles**,
  not pooled percentiles over all requests (S2 §7.4.1).
  - Run-level averaging dilutes each run's worst window and **makes the tail read
    low.** Fine for comparing conditions, but the absolute values must not be
    quoted as "this system's p99".
  - S3.7 chooses the operating point by the tail, so this distinction actually
    started to matter.
  - To do: dump per-request latency with `--dump-samples <path>` and have the
    analyser compute pooled percentiles. **But the bench is a measurement tool
    and is not changed while S3.7/S3.8 are running** — after the frozen period.
- [x] **Recover the jack node** (2026-08-20) — the hardware was fine
  - eth0 **2.5G up**, IP 192.168.123.4, binary, configuration, model hash
    (`dba155d2…`) and governor all normal. No trace of OOM or segfault.
  - `dmesg` showed the history: at boot the cable was on **eth1**
    (`t=13.6s eth1 link up`), at `t=620819s` eth1 link down, and at `t=689135s`
    **eth0 link up** — the cable had been physically moved. But a link drop does
    not kill the process (the node retries registration).
  - **The cause was never established.** Because there were no logs — the
    startup procedure is `setsid nohup ... &` and the log redirect was missing,
    so stdout was thrown away.
  - Verified after recovery: 3 nodes at 335.4 inf/s, jack 33.3% (3362
    requests), 0 errors, every preflight `--with-inference` item passed (**the
    three nodes' inference output hash identical**, `e84c5b53…`).
  - Preventing recurrence: `lib/remote.sh`'s `npuforge_restore_cluster` restores
    all three nodes and **forces the log redirect**. The problem persisted
    because the measurement scripts killed queen and jack to make a single-node
    configuration but only restored queen.
- [ ] **Move node startup to systemd** — draft at
  `scripts/npuforge-node.service.in`
  It brings log retention (journald), the last exit status and a restart policy.
  In the jack case above, "not knowing why it died" was worse than the process
  dying.
  - ⚠️ **Do not install it now.** The measurement scripts make a single-node
    configuration with `pkill -9 npuforge-node`. With `Restart` set, systemd
    revives it immediately and **a 1-node measurement silently becomes 3-node.**
    The kind of accident you do not know is wrong.
  - To be done alongside: replace `pkill` in `run-*.sh` with `systemctl stop`.
    Handled after the measurement campaign (S3.8) ends.
- [ ] **Compare `ondemand` vs `performance` over 300 s** ← checking the scope of
  §11's conclusion
  - The +7% is a 120-second measurement. Under sustained load `performance` may
    heat up faster and be worse. `discuss.md` §12
- [ ] **Extend S0 to 30 minutes** — settle steady-state throughput and the timing
  of CPU downgrade
- [ ] **Deploy the INT8 model to queen and jack** — currently only on `king`
- [x] **The scheduler build path = native on server** (verified 2026-08-20) —
  rust/cargo (dnf 1.92, satisfying MSRV 1.85), gcc, protoc and git installed on
  server, with a `git archive` tarball scp'd across. The node (aarch64) is still
  built on king. Cross-building is avoided because of linker problems
- [x] **Static IP pinning** (2026-08-20) — server, king, queen and jack all
  manual. Host NetworkManager static rather than router reservations. Same IPs,
  so SSH was uninterrupted
- [ ] **Open the gRPC port in server's firewall** — firewalld public zone, before
  measuring
- [ ] Configure `server` as an NTP server + wait with `chronyc waitsync`
- [x] **Ease the scheduler RSS concern** (2026-08-20) — server RAM 3GB →
  **16GB**. The dealer laptop constraint is resolved. RSS is still observed in S2
  (`environment-matrix.md` §10.1)
- [x] CPU governor → `performance` (2026-08-12) — made permanent with a systemd
  unit
  - `scripts/set-cpu-governor.sh`, survival across reboot confirmed
  - **+7% throughput.** All existing figures had been on ondemand
    (discuss.md §11)
- [x] Settle `worker_count` — **8**, `core_mask` unset (discuss.md §4)

---

# 2. Progress by milestone

## M0. Repository and environment — ✅ done

- [x] Rust workspace (7 crates, edition 2024)
- [x] `npuforge-common` — types, error codes, configuration, backend interface
- [x] `npuforge-mock-backend` — deterministic seed, injection of latency, error
  rate and speed variance
- [x] `npuforge-rknn` stub — the Windows build passes via the feature gate
- [x] Three scheduling policies (round-robin / least-queue / ect)
- [x] Node registry + state machine + drain/disable
- [x] CI (fmt, clippy, test, aarch64 cross, cargo-deny)
- [x] LICENSE (Apache-2.0), NOTICE, DEPENDENCIES.md, MODEL_LICENSES.md
- [x] Configuration examples (scheduler, node, mock 3-node)
- [x] Tests passing — 81 at M0, **currently 209 across the workspace**
  (2026-08-14)

## M1. Single-node inference — 🟡 in progress

- [x] The RKNN C wrapper written and **compile-verified on real hardware**
- [x] Thread-safety verified — **RKNN 2.3.0 confirmed thread-safe**
- [x] FFI signatures checked against the real headers
- [x] The model conversion environment (Docker, rknn-toolkit2 2.3.0)
- [x] YOLOv8n FP16 converted and deployed to 3 nodes
- [x] INT8 conversion — 6.46MB (−33% against FP16's 9.65MB)
- [x] Inference accuracy verified — detection-level comparison on a real board,
  `results/accuracy/README.md`
- [x] The real `npuforge-rknn` implementation — context pool + multiple outputs
  (2026-08-11)
  - Six real-hardware integration tests pass (`tests/real_device.rs`)
  - **A shared context produces 100% wrong results with 0 API errors** —
    measured (the correction in `environment-matrix.md` §3.1)
- [ ] 1,000-iteration inference stability (24,000 confirmed via soak; a formal
  test is separate)

## M2. Remote inference — 🟡 only metrics remain

- [x] `npuforge-proto` — the .proto definitions and tonic wiring
- [x] The `NodeService` gRPC server (node side)
- [x] Node registration / heartbeat (registration backoff retries,
  `must_reregister` re-registration)
- [x] The `SchedulerService` gRPC server
- [x] The scheduler → node gRPC client (per-node channel reuse)
- [x] Local queue + worker pool
- [x] Error handling and retries (selecting a different node on retry)
- [x] Model directory loading + SHA-256 verification
- [x] **Local 3-node Mock cluster confirmed working** (without hardware)
- [ ] Basic metrics (Prometheus)
- [x] The `npuforge-bench` CLI — load generation, aggregation, run-validity
  judgement (2026-08-11)

### What was verified (2026-08-11)

The integration test `crates/npuforge-scheduler/tests/mock_cluster.rs` — it runs
over real gRPC with the scheduler attached to 3 nodes. It is one process, but the
transport path is the same as on real hardware.

| Item | Result |
|---|---|
| Requests spread across 3 nodes | ✅ round-robin uses all three |
| Bypass when 1 node dies | ✅ 6/6 succeeded; the dead node produces no results |
| All nodes dead | ✅ `NPF-1302` + the list of nodes attempted |
| Timing breakdown | ✅ both node-measured and scheduler-measured sections populated |
| Avoiding a slow node | ✅ least-queue uses the fast nodes more |

Also confirmed with four real processes (scheduler + 3 nodes).
Killing the scheduler and bringing it back has all three nodes **re-register by
themselves within about 1.3 seconds.** A node switches a failed heartbeat
straight to re-registration — a transient network error and a scheduler restart
are indistinguishable, so the more expensive option was taken, and registration
is idempotent.

## M3. Multiple nodes — 🟡 cluster operation confirmed (2026-08-20)

- [x] **Three real nodes registered** — king/queen/jack confirmed registered with
  the scheduler (.9)
- [x] **Round Robin routing** — an exact 33.3% three-way split in the pilot bench
- [x] The `npuforge-bench` CLI
- [x] **Pilot 3-node inference** — c6 146 / c24 336 inf/s, 0% errors
- [x] **The first S2 scalability measurement** (2026-08-20) — 1/2/3 nodes at
  111.6/228.7/337.7 inf/s, **scaling efficiency ~98%**. Cluster node ceiling 115
  < local 157 (27% scheduler overhead). Preflight passed. RESULTS §2.5,
  board-worklog §2.25
- [ ] **The formal S2** — repeated runs, fan conditions, `--with-inference`,
  decomposing the overhead with TimingBreakdown
- [x] Fixed the model.toml `model_file` relative path bug (2026-08-20, §6 issue
  8)

## M4. Dynamic scheduling — ⬜

- [ ] Least Queue / ECT verified on real hardware
- [ ] Policy comparison (S3)

## M5. Failure recovery — ⬜

- [ ] Health checks verified on real hardware
- [ ] Automatic exclusion / re-admission
- [ ] The retry path verified
- [ ] **Distinguish a board hard reset from an intentional failure** (boot_id)

## M6. Dashboard — ⬜

- [ ] Cluster overview / node view / benchmark view / event timeline
- [ ] SSE live transport
- [ ] Voltage, temperature and frequency display

## M7. Optimization experiments — ⬜

- [ ] **Re-examine the S2 scalability experiment design** ← the premise changed
  with the INT8 results
  - 1.545 Gbps per node (INT8) / 0.829 Gbps (FP16)
  - 4.636 / 2.486 Gbps at three nodes — **both exceed a single 2.5GbE link**
  - Raise the aggregation link to 10G (the §4 topology)
  - See `discuss.md` §8, `RESULTS.md` §8.1

- [ ] Buffer pool
- [ ] CPU profile (checking the preprocessing share)
- [ ] Decide whether to apply io_uring

## M8. Talk release — ⬜

- [ ] The v0.1 tag, README, installation scripts
- [ ] Publish the raw benchmark data
- [ ] Presentation material, demo video, backup video

---

# 3. Benchmark scenarios

**Premise: S0 determines every other scenario's thresholds and cooldown. It
comes first, without exception.**

- [ ] **S0-A** thermal characterisation (fanless) — 3 nodes × 1,800 s
- [ ] **S0-B** thermal characterisation (cooled) — 3 nodes × 1,800 s
- [ ] S1 single-node baseline
- [ ] S2 scalability (1/2/3 nodes)
- [ ] S3 scheduler policy comparison
- [ ] S4 failure handling
- [ ] S5 network implementation comparison
- [ ] S6 input size comparison

146 runs in total, about 23.4 hours. Unattended overnight execution is required.

---

# 4. Infrastructure status

| Item | Status |
|---|---|
| The 3 boards (king/queen/jack) | 🟡 OS, kernel, RKNN, gcc and governor match; eth0 2.5G measured. **queen and jack share an SSH host key (unresolved)** |
| SSH aliases and key auth | ✅ IPs updated (.3/.5/.4/.9), `npuforge-server` added |
| `server` (Rocky 9.4, scheduler and bench) | ✅ **Xeon x2 24T / 16GB / 10G**. Rust and Docker not installed |
| Power 5V 4A × 3 | ✅ verified under sustained load |
| **2.5G/10G switch** | ✅ NEXI NS-S25G10G-N |
| **Inference network bandwidth** | ✅ worker 2.5G / aggregation 10G, 5.11 Gbps measured across three nodes |
| Management/inference network separation | ⬜ sharing a single range; to be decided before M3 |
| Static IPs | ✅ all four static (manual). Router reservation optional |
| Board physical placement | ✅ spread 5.6 °C, no NPU throttling (confirmed 2026-08-11) |
| Cooling (3 fans) | ⬜ not purchased |
| CPU governor | ✅ fixed to `performance`, survives reboot |
| NTP synchronisation | ⚠️ chrony installed, making `server` the server not done |
| Temperature thresholds | ⚠️ draft values (80/90 °C) — reset after S0 |

---

# 5. Purchase list

The equipment blocking M3 (switch, server, 10G NIC) has **all been obtained**
(2026-08-20). What remains is for measurement quality.

| Item | Qty | Priority | Note |
|---|---:|---|---|
| **Identical-model fans** | 3 | medium | 5V USB, same speed. For S0-B |
| Cat6/6a cables (spare) | 2–3 | low | 10G spares. The current link is fine |
| USB power meters | 3 | low | a 5V input, so USB meters work |
| Spare cables and adapters | 1 each | medium | for the talk |

**The power adapters are resolved** (replaced with 5V 4A × 3).

---

# 6. Known issues

| # | Issue | Severity | Status |
|---|---|---|---|
| 1 | ~~`king` runs 19 °C hotter~~ | resolved | did not reproduce under controlled re-measurement (spread 5.6 °C) |
| 2 | The temperature thresholds conflict with the normal operating range | **high** | reset after S0 |
| 3 | No RTC — the clock is wrong immediately after boot | medium | chrony wait logic needed |
| 4 | No current sensor → FPS/Watt cannot be computed | medium | an external USB power meter is needed |
| 5 | Throughput has not bent at 8 threads | low | MAX_THREADS needs widening |
| 7 | Only the board (king) has the Rust toolchain | low | build-only. The binary is built once and deployed |
| 6 | The collected `npu_cores` value is the devfreq count (1) | low | fix the metric definition |
| ~~8~~ | ~~model.toml `model_file` relative path unresolved~~ | resolved | `main.rs` replaces `spec.model_file` with an absolute path before `load_model`. Re-verified with 3-node loading and a bench on a relative-path model.toml (2026-08-20) |
| 9 | The NPU context is not released on node restart | medium | a dead node holds the context and restart gives status=-2. `pkill -9` plus a wait is needed. Graceful shutdown to be reviewed |

---

# 7. Settled headline figures

Measured values that can be quoted in the talk and the documents.

| Item | Value | Source |
|---|---|---|
| SoC | RK3576, 2-core NPU, 6 TOPS | measured |
| RKNN concurrency | **dedicated contexts can run concurrently / sharing a context is forbidden** | when shared, 0 API errors but 200/200 result mismatches |
| FP16 8-thread burst throughput | 70–78 inf/s | measured on 3 nodes |
| **FP16 8-thread sustained throughput** | **84.3 inf/s** | governor=performance, 120 s |
| **INT8 8-thread sustained throughput** | **157.2 inf/s** | governor=performance, **1.86× against FP16** |
| INT8 mean latency | 50.8 ms | −46% against FP16's 94.5 ms |
| CPU governor effect | +7% | ondemand→performance. **A 120-second measurement.** Unverified under sustained load |
| `want_float=0` effect | **INT8 +17.3% / FP16 +15.7%** | output a quarter too (discuss.md §12) |
| **Steady-state throughput (300 s)** | **FP16 59.7 inf/s** | **−27%** against the starting 81.6. CPU throttling |
| (reference) on ondemand | FP16 79.0 / INT8 146.2 | every measurement before 08-11 is on this basis |
| Kernel ioctls per inference | **76 (identical for FP16 and INT8)** | strace; the ceiling is set by time, not count |
| **Peak vs sustained degradation** | **about 10%** | 77.3 → 69.7 |
| Recommended `worker_count` | **8** (`core_mask` unset) | the core_mask sweep |
| Actual contribution of the NPU's 2 cores | **1.51×** (not 2×) | against a control group |
| NPU temperature under sustained load | **67.5–75.8 °C** (3 boards, 8 threads, 15 min, FP16) | controlled measurement 2026-08-11 |
| INT8 accuracy (vs FP16) | detection cells 10/10, classes 100%, box cos 0.997 | detection level on a real board |
| **Shared-context result mismatch** | **100%** (0 API errors) | why a context pool is mandatory |
| Node-to-node thermal spread | **5.6 °C** (no NPU throttling) | concurrent load |
| **CPU thermal downgrade** | A72 2208→**816MHz**, A53 2016→**600MHz** | after 60 s of load. The NPU holds 950 MHz |
| Input voltage under load | minimum 5.05V | measured on all three simultaneously |

> The peak vs sustained gap is a figure absent from vendor spec sheets and one of
> this project's central outputs.
> But the current value is contaminated by desk fan intervention and **has to be
> cleanly re-measured in S0.**
>
> The controlled measurement on 2026-08-11 (15 min × 3 boards) gives sustained
> throughput of **77.7–80.5 inf/s**, higher than the soak's 69.7 inf/s. The soak
> conditions differ (24,000 iterations, longer duration) so they are not compared
> directly. S0 unifies the conditions and settles it.
