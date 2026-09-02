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
> | Final experiment state | [`experiments/README.md`](experiments/README.md) §5–§7 |
> | The io_uring verdict (**not adopted**) | [`01-TECHSPEC.md`](01-TECHSPEC.md) §15 |
> | Settled figures | [`RESULTS.md`](RESULTS.md) · [`experiments/`](experiments/) |
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
