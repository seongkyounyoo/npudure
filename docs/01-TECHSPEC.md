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
> | Final experiment state | [`experiments/README.md`](experiments/README.md) §5–§7 |
> | The io_uring verdict (**not adopted**) | **§15 of this document** |
> | Settled figures | [`RESULTS.md`](RESULTS.md) · [`experiments/`](experiments/) |
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
> → [`experiments/S3_9B_NODE_RESIDUAL.md`](experiments/S3_9B_NODE_RESIDUAL.md)
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
