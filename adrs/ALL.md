<a id="index"></a>

# NPUDure Architecture Decision Records (ADR)

> **This file is generated. Do not edit it directly.**
> It concatenates the 31 source documents under `adrs/` for reading,
> printing and sharing.
> If something needs fixing, fix the source and regenerate.
>
> ```bash
> python scripts/build-adr-bundle.py $(git log -1 --format=%cs -- adrs/)
> ```
>
> - Generated as of: **2026-09-02** (the last commit date for `adrs/`)
> - Sources: `adrs/README.md`, `adrs/OVERVIEW.md`, 28 ADRs, `adrs/TEMPLATE.md`
> - Links between files have been rewritten to in-document anchors

*[한국어 원문](README.ko.md)*

This folder answers only **"why is it like this?"**

What does what is `docs/01-TECHSPEC.md`'s job, and what numbers came out is
`docs/RESULTS.md`'s. Written here is **what was chosen and what was discarded at
each fork**, and **what observation would overturn that choice**.

---

## Why ADRs are needed separately

Most other documents in this repository are **chronological**.

| Document | Axis |
|---|---|
| `docs/discuss.md` | in the order experiments were run |
| `docs/board-worklog.md` | in the order work was done |
| `docs/RESULTS.md` | results gathered by topic, but the reasoning is scattered across the two above |

So answering a single question like "why does the node send integers now?" means
moving between three documents and reading chronologically. And since this
project has had **five conclusions inverted by measurement**, reading only the
early parts makes it easy to mistake an already-discarded judgement for a
current decision.

ADRs re-cut the same material **by topic**. One decision, one file.

---

## Status labels

| Status | Meaning |
|---|---|
| **accepted** | currently valid. The code and documents follow this decision |
| **provisional** | this is what is done for now, but the evidence is weak. The re-measurement conditions are in the body |
| **superseded** | another ADR overturned this decision. The header names the superseding number |

### The rule for handling inverted decisions

**Do not create a new ADR file for a discarded decision.** Instead, put the story
into the "Context" section of the ADR that superseded it.

This project has five inverted judgements (shared context, the 78 inf/s node
ceiling, no throttling, `king` at 19 °C, 2.5GbE being sufficient), so keeping
them as separate files would make half the list obsolete documents. That makes
it hard for a reader to find the valid decisions.

But **the story of the inversion is always kept.** That list is the highest
reuse-value output this project has (`docs/RESULTS.md` §6).

---

## Bundle

There is an **[ALL.md](ALL.md)** with everything concatenated, for reading,
printing and sharing. **It is generated, so do not edit it directly.** Fix the
source and regenerate.

```bash
python scripts/build-adr-bundle.py $(git log -1 --format=%cs -- adrs/)
```

---

## Reading order

For someone arriving for the first time, this order is recommended.

1. **[OVERVIEW.md](#overview)** — a map of the whole system. Read before the
   ADRs
2. **[001](#adr-001), [002](#adr-002), [003](#adr-003), [004](#adr-004)** — the project's direction and skeleton
3. **[007](#adr-007), [011](#adr-011), [012](#adr-012), [013](#adr-013)** — decisions that came out of actually handling the NPU and heat. The highest density of measurement
4. **[015](#adr-015), [017](#adr-017), [028](#adr-028)** — the devices that block "failures that look like success"
5. The rest can be looked up when needed

### If there is no time, just three

| # | Why |
|---|---|
| [007](#adr-007) | 0 errors and 100% result mismatch. It shows this project's character best |
| [013](#adr-013) | what collapses first is the CPU, not the NPU |
| [002](#adr-002) | why bad numbers get published as they are |

---

## The list

### Project direction

| # | Title | Status |
|---|---|---|
| [001](#adr-001) | Split requests, not the model (data parallelism) | accepted |
| [002](#adr-002) | Define success as measurability, not a number | accepted |
| [022](#adr-022) | Assign each document a normative domain; the normative one wins on disagreement | accepted |

### System structure

| # | Title | Status |
|---|---|---|
| [003](#adr-003) | One scheduler, and no high availability | accepted |
| [004](#adr-004) | Separate the backend behind an interface, with Mock first-class | accepted |
| [005](#adr-005) | Put the RKNN link behind a feature and default it off | accepted |
| [006](#adr-006) | Split into seven crates and confine `unsafe` to one | accepted |
| [008](#adr-008) | Internal communication uses gRPC (tonic + Protocol Buffers) | accepted |

### Scheduling

| # | Title | Status |
|---|---|---|
| [009](#adr-009) | Fix the policies at three; all three share the candidate filter | accepted |
| [010](#adr-010) | The ECT score formula and each term inside it | accepted (before real-hardware validation) |
| [026](#adr-026) | Retries always go to a different node; keep the backoff short | accepted |
| [027](#adr-027) | The node state machine, with drain and disable separated | accepted (thresholds are a draft) |

### NPU runtime

| # | Title | Status |
|---|---|---|
| [007](#adr-007) | A dedicated RKNN context per thread — sharing blocked by the type system | accepted |
| [011](#adr-011) | The reference model is INT8 | accepted |
| [012](#adr-012) | The node sends integers without dequantizing (`want_float=0`, blob v2) | accepted |
| [020](#adr-020) | `worker_count = 8`, `core_mask` unset | accepted |
| [021](#adr-021) | The node does no postprocessing (NMS) | **provisional** |

### Hardware and measurement environment

| # | Title | Status |
|---|---|---|
| [013](#adr-013) | Fanless as the default; throttling is something to measure | accepted |
| [014](#adr-014) | 10G on aggregation only; the scheduler on a separate server | accepted (built and measured) |
| [018](#adr-018) | Convert the model once and deploy to all three nodes | accepted |
| [019](#adr-019) | Reach the boards by SSH alias, not by IP | accepted |
| [023](#adr-023) | CPU governor to `performance` — but state the scope of the evidence | **provisional** |

### Measurement discipline

| # | Title | Status |
|---|---|---|
| [015](#adr-015) | A hard-failing preflight check before measuring | accepted |
| [016](#adr-016) | Detect mid-measurement reboots with `boot_id` and invalidate the run | accepted |
| [017](#adr-017) | Harden the remote-execution pitfalls into library functions | accepted |
| [028](#adr-028) | The bench tool judges run validity itself | accepted |

### Protocol and policy details

| # | Title | Status |
|---|---|---|
| [024](#adr-024) | Fix errors to an `NPF-xxxx` code scheme | accepted |
| [025](#adr-025) | A failed heartbeat re-registers immediately — registration is idempotent | accepted |

---

## The ADRs that came out of failures

This project made **the same type of mistake four times.** All of them were
"trusting a metric by its name without checking what it counts".

```text
1. reading run_duration as NPU occupancy time      -> it included queue wait
2. sampling NPU load with delayms=3000 still set   -> it was reading a 3-second average
3. judging thread-safety by API return codes only  -> results never compared   -> ADR-007
4. judging throttling by NPU clock alone           -> the CPU was the one bending -> ADR-013
```

The devices that came out of those are kept separately.

| ADR | What it blocks |
|---|---|
| [015](#adr-015) | starting a measurement on a false premise |
| [016](#adr-016) | reading a reboot as "performance degradation" |
| [017](#adr-017) | a remote command failing with exit code 0 |
| [019](#adr-019) | attributing one board's results to another node |
| [028](#adr-028) | an invalid run's numbers reaching the result tables |

---

## Writing a new ADR

1. Copy [TEMPLATE.md](#template)
2. The filename is `NNN-ascii-kebab-slug.md`. Numbering continues **from 029**
3. Numbers are **not reused.** A discarded decision keeps its number
4. Update this README's list and statuses alongside

### What to hold to when writing

- **Always attach the measurement conditions to a number.** Nodes, thread count,
  duration, governor, model. We have already experienced that a number without
  conditions is useless three months later
- **Write down the rejected alternatives.** What was rejected and why lasts
  longer than what was chosen
- **Write down what is not known.** The "provisional" status and the "what would
  overturn this" section are where that goes
- **In the re-verification method, write what must not be looked at.** A pass
  verdict has been reached from the wrong metric four times

---

<a id="overview"></a>

# NPUDure architecture overview

*[한국어 원문](OVERVIEW.ko.md)*

The document to read before the ADRs. Its purpose is a single pass over **what
the whole system looks like**; the rationale for individual choices is left to
each ADR.

---

## 1. One sentence

A Rust runtime that spreads inference requests across three cheap edge NPU
boards and **measures whether three really becomes three times.**

## 2. What it does and does not do

```text
does                                does not
─────────────────────────────       ─────────────────────────────
spread independent requests         split one model across nodes
choose by node load                 make a single request 3x faster
drop dead nodes, readmit revived    Kubernetes-class orchestration
break the time down by stage        make three NPUs look like one
```

The right column is the **explicit non-goals**. The first two in particular keep
attracting "so why not do that?", so the rationale is written separately in
[ADR-001](#adr-001).

The core of it: **this system cannot make one request fast.** It only makes the
whole get through more when there are many requests.

## 3. Three layers

```text
+---------------------------------------------------------+
|  Client                                                 |
|  benchmark CLI . demo web . API clients calling directly |
+---------------------------+-----------------------------+
                            |  gRPC : Infer(model, image)
                            v
+---------------------------------------------------------+
|  Scheduler   (runs on a separate host, not a board)     |
|                                                         |
|   Node Registry   who is alive                          |
|   Scheduler       who gets this request                 |
|   Retry Manager   another node on failure               |
|   Health Monitor  drop from candidates when heartbeats stop |
+----------+--------------+--------------+----------------+
           |              |              |  gRPC
           v              v              v
    +-----------+  +-----------+  +-----------+
    |  king     |  |  queen    |  |  jack     |
    |  RK3576   |  |  RK3576   |  |  RK3576   |
    |  NPU 6TOPS|  |  NPU 6TOPS|  |  NPU 6TOPS|
    +-----------+  +-----------+  +-----------+
      each node holds the entire same model
```

**The three nodes are completely equivalent.** They use the same binary, the
same model file and the same configuration, differing only in the `id` and
address in the `[node]` section. There is no communication between nodes — they
do not even know the others exist.

The scheduler does not run on a board because loading the scheduler onto one
node contaminates the 1/2/3-node comparison the moment it happens.

## 4. The life of one request

```text
 1. Client ------------> Scheduler      one image (640x640x3 = 1.23 MB)
 2. Scheduler                           shortlist candidate nodes (exclude dead ones)
 3. Scheduler                           run the policy -> select a node
 4. Scheduler ---------> Node           forward to the selected node
 5. Node                                put on the local queue
 6. Node                                a worker picks it up and preprocesses
 7. Node                                NPU inference  <- only here is the NPU; the rest is CPU
 8. Node --------------> Scheduler      9 raw tensors as a single blob
 9. Scheduler ---------> Client         result + per-stage timings
```

The time taken at each arrow and each box is **recorded separately**.

```rust
scheduler_queue_us   scheduler_route_us   network_to_node_us
node_queue_us        decode_us            preprocess_us
npu_input_us         inference_us         postprocess_us
network_to_client_us end_to_end_us
```

This breakdown is close to the project's reason for existing. "Three nodes only
reached 2.4×" is not an answer; **which field it leaks from** is.

> Step 7 is the NPU section and everything else is CPU. And in measurement,
> **what collapses first is the CPU, not the NPU** — throughput falls 27% over
> 300 seconds of sustained load while the NPU clock stays pinned at 950 MHz and
> the CPU is downgraded from A72 2208 to 816 MHz.

### On failure

```text
failure at 4 --> classify cause --> retryable? --> drop that node from candidates
                                                --> retry on another node
                                                --> NPF-1302 if all fail
```

It never throws the request back at the same node. A node that just failed is
likely to fail on the next attempt too.

## 5. Crate map

```text
                    npuforge-common
                    types . error codes . config . InferenceBackend interface
                            | everything references this
        +-------------------+-------------------+
        |                   |                   |
 npuforge-scheduler   npuforge-node      npuforge-bench
 3 policies . registry worker pool . queue  load generation . aggregation
        |                   |
        +---- npuforge-proto +          gRPC definitions (.proto -> tonic)
                            |
                 +----------+----------+
                 |                     |
        npuforge-rknn          npuforge-mock-backend
        the real NPU. all of      a fake backend that runs without hardware
        the unsafe lives here     deterministic seed . latency/error injection
```

| Crate | One line |
|---|---|
| `npuforge-common` | The types and interfaces everyone shares. This is the contract |
| `npuforge-proto` | gRPC service definitions |
| `npuforge-scheduler` | Decides which node to send to, and resends on failure |
| `npuforge-node` | The agent running on a board. Queue + worker pool |
| `npuforge-rknn` | RKNN Runtime FFI. **The `unsafe` containment zone** |
| `npuforge-mock-backend` | An NPU imitation. For running everything without hardware |
| `npuforge-bench` | Applies load, produces statistics and **judges whether this run is valid** |

The two backends implement the same `InferenceBackend` interface. So **without a
single RK3576 board**, `cargo test --workspace` passes and a 3-node cluster runs
locally. This is a design principle, not a convenience feature.

## 6. Physical setup

```text
current (cannot measure)             planned (M3)

 management 1GbE                      Scheduler server
   |-- king                              | 10GbE  <- aggregation
   |-- queen                             |
   |-- jack                        2.5G/10G switch
   \-- dealer (scheduler, laptop)     |-2.5G- king
                                      |-2.5G- queen
 no inference network. switch not bought  \-2.5G- jack
```

**The worker links are fine at 2.5G and only aggregation needs 10G**, because
the three nodes' traffic converges at one point. Measuring now would measure
link saturation rather than NPU scaling efficiency, so M3 has not started.

The current scheduler host, `dealer`, is a laptop with no PCIe slot and cannot
take a 10G NIC. A separate server is needed.

## 7. Where things stand

| | Status |
|---|---|
| Software skeleton | ✅ 209 tests, clippy `-D warnings`, fmt clean |
| Single-node measurement | ✅ INT8 157.2 inf/s / FP16 84.3 inf/s (8 threads, 120 s) |
| Mock 3-node cluster | ✅ connects over real gRPC. Without hardware |
| Real 3-node hardware | ⬜ **halted, waiting on network equipment** |
| Prometheus and dashboard | ⬜ |

What blocks it is equipment, not code. The detailed resumption procedure is at
the top of `docs/TODO.md`.

## 8. Where to read more

| Question | Where |
|---|---|
| **The full list of decisions** | **[README.md](#index)** |
| Why is the model not split | [ADR-001](#adr-001) |
| Why is there only one scheduler | [ADR-003](#adr-003) |
| Why does everything run without hardware | [ADR-004](#adr-004) |
| Why an NPU context per thread | [ADR-007](#adr-007) |
| Why INT8 | [ADR-011](#adr-011) |
| Why does the node send integers rather than floats | [ADR-012](#adr-012) |
| Why no fan | [ADR-013](#adr-013) |
| Why wait instead of measuring now | [ADR-014](#adr-014) |
| Why run preflight before measuring | [ADR-015](#adr-015) |
| What does what (the full specification) | `docs/01-TECHSPEC.md` |
| What numbers came out | `docs/RESULTS.md` |
| What to do now | `docs/TODO.md` |
| The final authority for values | `docs/environment-matrix.md` |

---

<a id="adr-001"></a>

# ADR-001. Split requests, not the model (data parallelism)

*[한국어 원문](001-data-parallel-only.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-012](#adr-012), `docs/00-PRD.md` §4, `docs/01-TECHSPEC.md` §2.1 |

---

## In one line

> Three nodes each hold **the entire same model** and handle **different
> requests**. Splitting the model layer-wise across nodes is not done in v0.1.

## Context

There are broadly two ways to use several NPUs at once.

### Approach A. Split the model (model parallelism / layer partitioning)

```text
one request --> [node1: layers 1-10] --intermediate tensor--> [node2: layers 11-20] --> result
```

Node 1 computes the model's front section and node 2 the back. **Intermediate
results (feature maps)** travel between the nodes. LLM tensor parallelism and
pipeline parallelism are of this family.

- Advantage: the processing time of **a single** request can fall. A model too
  large for one node can be run
- Disadvantage: inter-node communication lands **inside the inference path**.
  If one node is slow, everything waits

### Approach B. Split the requests (data parallelism)

```text
request A --> [node1: whole model] --> result A
request B --> [node2: whole model] --> result B
request C --> [node3: whole model] --> result C
```

Each node holds the whole model and takes a different request end to end.

- Advantage: there is **no** inter-node communication. If one dies, the rest
  keep running
- Disadvantage: **a single** request never gets faster

Everything about the system follows from this choice — the scheduler's role,
failure handling, network requirements, and even what gets measured.

## Decision

**Implement approach B (data parallelism) only.** Approach A is an explicit
non-goal for v0.1.

The following become non-goals along with it.

- Splitting one large model layer-wise across several nodes
- LLM tensor parallelism / pipeline parallelism
- Hardware-level integration making several NPUs appear as one physical NPU
- Reducing a single inference request's latency in proportion to node count

## Rationale

### 1. The goal is throughput, not latency

The question this project sets out to answer is **"do three 6 TOPS units really
make 18 TOPS?"** That asks how much gets processed in total when requests pile
up, not how quickly one is finished.

The assumed usage is the same. Multiple cameras, multiple requests —
**independent requests arrive in bunches to begin with.** Data parallelism is
the natural shape for that load, and splitting the model would be a net loss.

### 2. On this hardware, partitioning cannot afford the communication

One input is already large.

```text
raw RGB 640 x 640 x 3 = 1,228,800 byte
```

**The input alone** comes to 4.64 Gbps at three-node saturation (at INT8's
157.2 inf/s). That is already why the aggregation link needs 10G.

Layer partitioning **adds intermediate-tensor round trips inside the inference
path** on top of that. Each partition point adds one inter-node transfer, and on
2.5GbE a 1 MB-class tensor computes to somewhere near 4 ms. Against a **total**
INT8 inference of 50.8 ms, a few partition points erase the benefit.

> That 4 ms is a figure divided by link speed, not a measurement. But it was
> judged not to be a difference worth measuring to confirm — the benefit
> partitioning would bring is not a goal in the first place.

### 3. This model has no reason to be split

Partitioning is **forced** when the model does not fit on one node.

```text
node RAM              4 GB
YOLOv8n INT8 model    6.46 MB
YOLOv8n FP16 model    9.65 MB
```

Three orders of magnitude apart. There is no need to split anything.

### 4. The measurements have to be interpretable

This project's output is **"where does it leak?"** With nodes independent of
one another, when scaling efficiency comes up short the cause can be cleanly
divided into scheduling, network and node-internal.

Adding layer partitioning creates inter-node dependency, so when three nodes
yield only 2.4×, separating whether that is partition-point communication,
scheduling or the NPU becomes hard. **A project whose purpose is measurement
must not choose a structure in which causes cannot be decomposed.**

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Layer-wise partitioning | Communication cost enters the inference path. This model fits comfortably in 4 GB, so there is no reason to split |
| LLM tensor parallelism | The target model is a CNN detector. There is nothing to apply it to |
| Hardware-level NPU integration | Not something achievable on top of the RKNN Runtime. Driver and SoC-level work |
| Supporting **both** data parallelism and partitioning | Neither could be measured properly within v0.1. Doing both halfway makes both sets of figures unusable |

## Consequences

**Gained**

- No communication between nodes. The nodes do not know the others exist
- Failure handling becomes simple — drop the dead node from the candidates and
  that is it. Work in progress on other nodes is unaffected
- Measuring one node's performance ceiling predicts the cluster's ceiling.
  **This is why so much effort went into single-node measurement**
- The scheduler only has to decide "one request → one node"

**Lost / the cost**

- **Single-request latency never falls with more nodes.** 50.8 ms for one INT8
  inference is 50.8 ms on three nodes too. That is design, not a bug
- A model that does not fit on one node cannot be run
- Each node needs its own copy of the model (not a problem at this scale)

**New constraints introduced**

- Talks and documents **must not say "3× faster".** "Processes 3× as much" is
  correct. Mixing the two makes an audience expect a latency reduction
- Benchmark scenarios must use **concurrent request** load. Measuring by
  throwing one request at a time is meaningless in this structure

## What would overturn this

Revisit if any of the following holds.

- **When the target model does not fit in node memory.** If a model above 4 GB
  has to run, partitioning stops being a choice and becomes forced
- **When single-request latency becomes a requirement.** Not now, but if, say,
  frame-level real-time control became the goal, the premise changes
- **When the inter-node link becomes fast enough to carry inference-internal
  communication.** Though that changes this project's own premise of 2.5GbE-class
  edge boards

All three are outside v0.1's scope. Even a re-examination should come **after
v0.1's data-parallel measurements are finished** — without a comparison
baseline there is no way to judge whether partitioning is a gain.

---

<a id="adr-002"></a>

# ADR-002. Define success as "can it be measured and explained", not "did the number come out"

*[한국어 원문](002-success-criteria-measurability.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-05 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-001](#adr-001), [ADR-015](#adr-015), [ADR-028](#adr-028), `docs/00-PRD.md` §3 |

---

## In one line

> **No result value is set as a success condition**, such as "2.5× or better at
> three nodes". Even if scaling efficiency comes out low, even if io_uring has
> no effect, it is a success as long as the cause can be explained
> quantitatively.

## Context

What happens when a measurement project sets a target number as its success
criterion.

```text
goal: "3-node scaling efficiency of 80% or better"

measured 65%  ->  has to be recorded as a failure
              ->  nobody wants to fail
              ->  favourable conditions start getting found
                 measure briefly . smaller input . preheat well . keep only the good runs
```

**This is not something only dishonest people do.** Given freedom in choosing
conditions and a target hanging over you, the favourable option gets picked
unconsciously. And each of those choices can be given a plausible reason.

This project has unusually large freedom in choosing conditions. Governor,
thread count, duration, cooling, input size, model — all of them move the
numbers. On the same board, the governor alone moved 7% and duration alone
moved 27%.

## Decision

**Success is defined as the following.**

1. Was it measured
2. Were the measurement conditions recorded with it
3. Can the cause of the result be explained
4. Is it reproducible

**The following results are explicitly counted as valid outcomes.**

- io_uring producing no meaningful performance improvement
- Zero-copy applying to only a limited scope
- The NPU or preprocessing, rather than the network, being confirmed as the
  primary bottleneck
- Three-node scaling efficiency being lower than expected
- A single high-performance device being more favourable on cost

## Rationale

### It actually helped

Results that would have been discarded without this criterion became the central
output instead.

| Result | Under a target criterion | What actually happened |
|---|---|---|
| Three application-level optimizations at +0.1 / +5.4 / −1.8% | Failure. Bury it and try something else | Became **the basis for "there is nothing left to squeeze inside the node"** |
| Zero-copy at −1.8% | Failure | Hypothesis refuted. Led to the discovery that 76 ioctls are intrinsic to inference submission |
| −27% under fanless sustained load | A bad number | **The peak vs sustained gap** — a value absent from vendor spec sheets. Became the central narrative of the talk |

The third is decisive. Had the goal been "high throughput", we would have
attached a fan, measured for 120 seconds and reported 84.3 inf/s. That figure
**does not reproduce in the field.**

### It becomes possible to publish inverted conclusions

Measurement inverted this project's conclusions five times. With a target number
hanging over it, inverting is itself a loss — the already-reported number
becomes void.

With "can it be explained" as the criterion, **inverting becomes an outcome
instead.** That is why `docs/RESULTS.md` §4 "Inverted conclusions" and §6 "List
of measurement failures" can exist.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Set a target number (e.g. 80% scaling efficiency) | Creates condition-selection bias. The most dangerous thing in a measurement project |
| Target number + "state the reason if missed" | The reason section becomes a paragraph of excuses. The same problem remains the moment a miss is defined as failure |
| Set no criterion at all | There is no way to know when it is finished. Measurement goes on indefinitely |

## Consequences

**Gained**

- Unfavourable results can be published as they are
- Failure cases become output — with more reuse value than the numbers
- There is no longer any reason to hide measurement conditions

**Lost / the cost**

- **"So how many times faster is it?" is hard to answer in one line.** A
  disadvantage in a talk. The conditions have to be said alongside, so the
  sentence gets longer
- The success/failure verdict can look subjective. Hence the four explicit
  conditions above

**New constraints introduced**

- **Every number has to carry its measurement conditions.** A number without
  conditions is void under this criterion. Nodes, threads, duration, governor
  and model are always written alongside
- Invalid runs must not be used as though valid → enforced by tooling
  ([ADR-028](#adr-028))

## What would overturn this

If this project becomes **a product rather than an experimental tool**, the
criterion changes. A product needs a line of "it has to reach at least this to
be usable".

v0.1's purpose is measurement, so this criterion stands.

---

<a id="adr-003"></a>

# ADR-003. One scheduler, and no high availability

*[한국어 원문](003-central-simple-scheduler.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-001](#adr-001), [ADR-014](#adr-014), `docs/01-TECHSPEC.md` §2.3 |

---

## In one line

> **A single central scheduler** decides which node a request goes to. No
> distributed consensus, no leader election, no scheduler redundancy is built.
> Instead, **the cost of the scheduler dying and coming back is made cheap.**

## Context

There are broadly four structures for spreading requests across nodes.

| Approach | Who decides |
|---|---|
| **Central scheduler** | one machine in the middle decides everything |
| Client-side distribution | the client picks the node itself (no scheduler) |
| P2P / gossip | nodes exchange state and decide among themselves |
| A general-purpose orchestrator | hand it to an off-the-shelf system like Kubernetes |

Further down the list, the single point of failure disappears and things hold up
at larger scale. In exchange, implementation and operation get heavier.

## Decision

**Use a single central scheduler.** And in v0.1, **do not implement** any of the
following.

- Distributed consensus (Raft and the like)
- Leader election
- Multi-scheduler high availability
- Kubernetes-level general-purpose orchestration

**The scheduler is a single point of failure.** This is written into the
documents not as a defect but as **a constraint accepted knowingly.**

## Rationale

### 1. What is being measured is the scheduling policy itself

This project runs an experiment (S3) that **swaps between three policies** —
Round Robin / Least Queue / ECT — and compares them. For that, **the point where
the decision is made has to be one place.**

If distribution scatters to clients or nodes, the very notion of "this run's
distribution policy" gets blurred. A policy comparison would end up measuring
differences in implementation location rather than policy.

### 2. ECT can only be computed with global state

The default policy, ECT, picks a candidate like this.

```text
ECT = ((queue_depth + in_flight + 1) x EWMA_inference
       + EWMA_network + thermal_penalty + error_penalty) / load_factor
```

The values that go in — each node's queue depth, in-flight count, moving average
of inference time, temperature — only compare if **all nodes are visible at
once**. A node deciding from its own state alone cannot satisfy this formula.

### 3. There are three nodes

The scale at which a consensus protocol or gossip earns its keep is tens to
hundreds of nodes. At three, implementation and debugging cost more than they
return.

### 4. The time budget

The goal is **finishing the measurements** within the period leading up to the
talk. Time spent implementing consensus is time not spent measuring what
actually needs measuring. What is decided against matters as much as what is
decided for.

## How the single point of failure is handled

Instead of eliminating it, **recovery is made cheap.**

- When a heartbeat fails, the node **switches immediately to re-registration**
- Registration is **idempotent**. Doing it repeatedly causes no problem
- So killing the scheduler and bringing it back has **all three nodes return by
  themselves within about 1.3 seconds** (verified with four real processes)

From the node's perspective, a transient network error and a scheduler restart
are indistinguishable. So it **unconditionally takes the more expensive option
(re-registration)**. That choice is available because registration is idempotent,
so wasted effort does not translate into loss. (→ ADR-025)

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Client-side distribution | The policy comparison experiment does not hold. There is also no way for a client to see global state |
| P2P / gossip | No benefit at three nodes. It introduces inter-node communication, breaking [ADR-001](#adr-001)'s premise that nodes do not know each other |
| Kubernetes | An explicit non-goal. Container orchestration is unrelated to the question this project asks and only adds noise to measurement |
| Two schedulers + leader election | Large implementation and verification cost. That time is time not spent measuring. The availability gained at three-node scale does not justify it |

## Consequences

**Gained**

- The three policies can be swapped in the same place → the S3 experiment became
  possible
- Retries, state machines and health checks are all in one process, making them
  easy to trace
- Scheduler restart recovery in 1.3 seconds

**Lost / the cost**

- **If the scheduler dies the whole cluster stops.** The nodes are alive but
  there is no path for requests to reach them
- The scheduler itself is part of the throughput ceiling. However many nodes are
  added, if the scheduler cannot keep up it stops there

**New constraints introduced**

- **The scheduler host became part of the measurement conditions.** Where it
  runs changes the numbers. That is why official benchmarks run it on a separate
  host rather than a board
  (→ [ADR-014](#adr-014))
- The scheduler host's resources become an experimental constraint. `dealer`
  currently has 3 GB of RAM, which could fall short once a 1.17 MiB payload ×
  concurrent count piles up. **Not yet observed**

## What would overturn this

- **When there are tens of nodes.** This decision presumes three
- **When the scheduler is actually measured as the bottleneck.** The basis for
  that judgement is already prepared — check whether `TimingBreakdown`'s
  `scheduler_queue_us` / `scheduler_route_us` occupy a meaningful share of
  `end_to_end_us`. **Do not guess; read that field**
- **When availability becomes a requirement.** This is experimental equipment
  today, and if the scheduler dies a person restarts it. Becoming an operational
  system changes the premise

---

<a id="adr-004"></a>

# ADR-004. Separate the backend behind an interface, with Mock as a first-class backend

*[한국어 원문](004-backend-abstraction-mock-first.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-005](#adr-005) (feature gate), [ADR-007](#adr-007), `docs/03-DEVELOPMENT-REQUIREMENTS.md` §4.1 |

---

## In one line

> Push NPU calls behind an `InferenceBackend` interface and make **a fake
> backend that slots into that place a proper implementation**. The whole system
> runs without a single RK3576 board. **This is a design principle, not a
> convenience feature.**

## Context

This project's development environment looks like this.

- The three boards sit on a desk and are not always powered on
- The development PC is **Windows/x86**. The RKNN Runtime is ARM64 Linux only
- CI runs on GitHub Actions. There is obviously no NPU there

Developing with no provision for this leads to: **code can only be written when
a board is on, tests only run when a board is on, and CI verifies nothing.**

But look closely and **the part of this system that actually needs an NPU is
very narrow.**

```text
three scheduling policies       NPU-independent
node registry, state machine    NPU-independent
retries, timeouts               NPU-independent
queues, worker pool             NPU-independent
gRPC wiring                     NPU-independent
health checks, drain            NPU-independent
────────────────────────────────────────────
one actual inference            <- only here is the NPU
```

## Decision

**1. Hide inference behind an interface.**

```rust
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn load_model(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>>;
    fn backend_name(&self) -> &'static str;      // "rknn" or "mock"
    fn runtime_version(&self) -> Result<String>;
}

#[async_trait]
pub trait LoadedModel: Send + Sync {
    async fn infer(&self, input: InferenceInput) -> Result<InferenceOutput>;
    fn model_info(&self) -> &LoadedModelInfo;
}
```

The scheduler and node agent know only this interface. They never call
`npuforge-rknn` directly.

**2. Make the Mock backend a proper backend, not a test helper.**

It is chosen in the configuration file. It is not a stub hidden inside test
code.

```toml
[backend]
type = "mock"          # or "rknn"
base_latency_ms = 20
jitter_ms = 5
error_rate = 0.02
```

**3. Put fault injection in the Mock.** On top of a deterministic seed it can
produce latency, latency variance, error rates and per-node speed differences.
The three nodes in `configs/mock/` **deliberately have different speeds and
error rates**.

**4. Set the verification bar at "passes without hardware."**
`cargo test --workspace` has to pass on Windows/x86.

## Rationale

### 1. Policy comparison has to show up in Mock first

If the difference between Round Robin and ECT can only be seen on real
hardware, every policy change means powering on boards, deploying and
measuring. The iteration cycle becomes minutes.

This is why the three nodes in `configs/mock/` have different speeds. **If the
speeds were equal, Least Queue and Round Robin would give the same answer.** The
conditions were made deliberately asymmetric so that policy differences surface
locally.

### 2. It can produce conditions that are hard to create on real hardware

"A node fails 2% of the time", "one node is 3× slower", "a node dies mid-request"
— reproducing these with real boards is cumbersome and poorly reproducible. With
a fixed seed the Mock produces them **in the same order every time.**

### 3. The transport path is real

The Mock 3-node integration test
(`crates/npuforge-scheduler/tests/mock_cluster.rs`) **runs over real gRPC.** It
is one process, but the wiring is the same as on real hardware.

| Verified | Result |
|---|---|
| Requests spread across 3 nodes | ✅ round-robin uses all three |
| Bypass when 1 node dies | ✅ 6/6 succeeded |
| All nodes dead | ✅ `NPF-1302` plus the list of nodes attempted |
| Timing breakdown | ✅ both node and scheduler sections populated |
| Avoiding a slow node | ✅ least-queue uses the fast nodes more |

### 4. CI actually verifies something

209 tests run without hardware. Without this, CI is decoration that only checks
that it compiles.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Keep only a `#[cfg(test)]` stub | It lives only inside tests. Bringing up a 3-node cluster and poking at it by hand becomes impossible |
| Require real hardware | Development stops when the boards are off. CI becomes meaningless. Contributors would have to buy a board to participate |
| Use the RKNN simulator | It cannot infer with a built `.rknn` — after `load_rknn`, `init_runtime` refuses. This was actually attempted and did not work |
| No interface, branch with conditional compilation | `#[cfg]` spreads through every call site and the two paths silently diverge |

## Consequences

**Gained**

- 209 tests pass on Windows/x86
- A 3-node cluster can be brought up locally and actually operated
- `unsafe` is confined to one place, `npuforge-rknn` (→ ADR-006)
- Contributors can participate without a board — important for an open-source
  project

**Lost / the cost**

- The cost of maintaining the interface. Every backend has to honour the same
  contract
- The risk of the two implementations diverging. Metadata such as
  `runtime_version` is meaningless in the Mock, creating places filled in for
  form only

**⚠️ New constraint introduced — the Mock is not omnipotent**

This is the most important sentence in this ADR.

**The Mock only imitates what passes through the interface.** It will never
catch a defect specific to RKNN. In fact,
[ADR-007](#adr-007)'s shared-context problem — 0 errors and
100% result mismatch — cannot reproduce in the Mock at all, because the Mock has
no concept of a context.

That is why **real-hardware integration tests have to exist separately.** The
six in `crates/npuforge-rknn/tests/real_device.rs` occupy that place.

```text
What the Mock guards           What only real hardware can guard
────────────────────           ─────────────────────────────────
policies, retries, state       RKNN concurrency contract
queues, timeouts               dequantization accuracy
gRPC wiring                    actual throughput and thermal behaviour
failure bypass paths           output tensor shapes
```

**Never conclude "the Mock tests passed, so we are fine."**

## What would overturn this

- **If cases of Mock and real hardware diverging accumulate.** At that point a
  choice is needed between raising the Mock's fidelity and narrowing it to
  policy verification only
- **If there are three or more backends**, the interface needs re-examination.
  Two is a minimal sample and it is hard to be confident the abstraction is right

---

<a id="adr-005"></a>

# ADR-005. Put the RKNN link behind a feature and default it off

*[한국어 원문](005-rknn-feature-gate-off-by-default.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-004](#adr-004), [ADR-006](#adr-006) |

---

## In one line

> `npuforge-rknn` is a workspace member but **does not link RKNN in the default
> build.** So `cargo build --workspace` passes on a Windows/x86 development PC
> and in CI. Only real-hardware builds turn on `--features rknn`.

## Context

The RKNN Runtime (`librknnrt.so`) is **an ARM64 Linux-only shared library**. It
is distributed by Rockchip and is not included in this repository.

Simply putting this crate in the workspace leads to:

```text
cargo build --workspace on a Windows development PC
  -> npuforge-rknn looks for librknnrt.so
  -> it is not there
  -> the whole workspace build fails
  -> no code can be written at all
```

[ADR-004](#adr-004) established the principle that
"everything has to run without hardware", and this breaks it at the link stage.

## Decision

**1. Create an `rknn` feature and leave the default empty.**

```toml
[features]
default = []
rknn = []
```

**2. Only real-hardware builds turn it on explicitly.**

```bash
cargo build --release --target aarch64-unknown-linux-gnu \
      -p npuforge-node --features rknn
```

It is registered in the repository as the `cargo build-node` alias.

**3. The types exist even in a build with the feature off.** A placeholder
implementation compiles and returns a clear error if inference is attempted.

```rust
async fn infer(&self, _input: InferenceInput) -> Result<InferenceOutput> {
    Err(NpuForgeError::new(
        ErrorCode::BackendError,
        "this binary was built without RKNN support",
    ))
}
```

**4. A mismatch between build and configuration dies at startup.**

```rust
pub const fn is_rknn_enabled() -> bool { cfg!(feature = "rknn") }
```

The node agent checks this value at startup. Giving a `[backend] type = "rknn"`
configuration to a binary built without RKNN stops it **before it takes its
first request.**

## Rationale

### It structurally blocks one mistake

**Deploying a Mock-only binary to a real node** is the most frightening
accident. It would run and the node would produce fake results — with better
throughput, since the Mock does not use the NPU.

Without the `is_rknn_enabled()` check, that accident is only discovered **after
the benchmark results are all in.** Dying at startup makes it immediately known.

This is a type already encountered in this project — the shared context and the
remote execution failure were both "failures that looked like success".

### `publish = false` is for the same reason

It guards against accidental publication from an environment without
`librknnrt.so`.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Default it on (`default = ["rknn"]`) | Windows and CI builds break. ADR-004's premise collapses |
| Take `npuforge-rknn` out of the workspace | It becomes a separate build and CI stops even checking that this crate compiles |
| Auto-detect with `#[cfg(target_arch)]` | Being aarch64 Linux is no guarantee the RKNN SDK is present. The build environment and the run environment can differ |
| Load dynamically at runtime with `dlopen` | Solves the link problem but loses compile-time verification of the FFI signatures. It would reopen a risk already caught by checking against the real headers |

## Consequences

**Gained**

- `cargo test --workspace` passes on Windows/x86 (209 tests)
- CI runs fmt, clippy, test and the aarch64 cross-build without the RKNN SDK
- Misdeployment of a Mock binary is caught at startup

**Lost / the cost**

- **The `--features rknn` path cannot be execution-verified in CI.** It
  cross-compiles but is never run. Hence the need for separate real-hardware
  integration tests (`crates/npuforge-rknn/tests/real_device.rs`)
- The build command forks in two. The feature must not be forgotten when
  deploying to real hardware (hence the `is_rknn_enabled()` check)

**New constraint introduced**

- The two `#[cfg(feature = "rknn")]` paths **have to keep the same interface.**
  Fixing one leaves the other failing to compile, or worse, silently diverging

## What would overturn this

- **If every development PC becomes ARM64 Linux**, the reason for this
  separation weakens. Though the CI runners would have to change too, so it is
  unlikely
- **If another NPU backend is added**, the feature naming and structure need
  re-examination. It is written assuming `rknn` alone

---

<a id="adr-006"></a>

# ADR-006. Split into seven crates and confine `unsafe` to one of them

*[한국어 원문](006-crate-split-unsafe-isolation.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-004](#adr-004), [ADR-005](#adr-005), [ADR-007](#adr-007) |

---

## In one line

> `unsafe` code calling the C library exists **only inside `npuforge-rknn`.**
> The other six crates use safe Rust only. When a memory problem appears, there
> is **one place to look.**

## Context

Rust has the compiler guarantee memory safety, but that guarantee ends the
moment a C function is called. The RKNN Runtime is a C library.

```c
int rknn_init(rknn_context* ctx, void* model, uint32_t size, uint32_t flag, ...);
int rknn_inputs_set(rknn_context ctx, uint32_t n, rknn_input inputs[]);
int rknn_outputs_get(rknn_context ctx, uint32_t n, rknn_output outputs[], ...);
```

Pointers, lifetimes and release timing have to be managed by hand. Scatter that
code around the repository and there is no way to find where a bug like
use-after-free came from.

## Decision

**1. Split into seven crates.**

| Crate | Responsibility | `unsafe` |
|---|---|---|
| `npuforge-common` | types, error codes, configuration, backend interface | none |
| `npuforge-proto` | gRPC definitions (.proto → tonic generated) | none |
| `npuforge-scheduler` | policies, registry, retries, health checks | none |
| `npuforge-node` | worker pool, queue, registration and heartbeat | none |
| `npuforge-mock-backend` | the hardware-free backend | none |
| `npuforge-bench` | load generation, aggregation, validity judgement | none |
| **`npuforge-rknn`** | **RKNN FFI and its safe wrapper** | **only here** |

**2. `unsafe` does not leave `npuforge-rknn`.** The crate's documentation says
so — "unsafe code is confined to this crate".

**3. Convert to safe types at the boundary.** The outside sees only the
`InferenceBackend` / `LoadedModel` interfaces. Pointers do not cross the
boundary.

**4. Express dangerous contracts as types.** For example `RknnContext::infer`
takes `&mut self` so the compiler blocks concurrent calls
([ADR-007](#adr-007)).

## Rationale

### 1. There is one place to look

When a memory error, a strange crash or an unexplainable value appears,
`npuforge-rknn` is where you start. That crate is a small share of the whole
workspace, so scanning it is cheap.

### 2. The other crates can be verified without hardware

`unsafe` and the hardware dependency sit in the same place, so removing that
leaves everything else as pure Rust. Swapping in the Mock is possible thanks to
the same separation.

### 3. It gives grounds to keep the C wrapper thin

The FFI goes through `native/rknn_wrapper.c`. That wrapper had **its signatures
verified against the real headers**, down to confirming that `rknn_context` is a
`uint64_t` on aarch64. Being in one place is what makes such a check possible.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| A single crate | `unsafe` spreads through everything. Keeping the Windows build alive with a feature gate would also get harder |
| Split into more crates | Seven is already plenty at this scale. Splitting further only raises dependency management cost |
| Auto-generate FFI with `bindgen` | The headers are not in the repository and it ties to an SDK version. Writing by hand and checking against real hardware was more controllable |
| `unsafe` directly inside the node | The node becomes tied to RKNN and the Mock path does not hold |

## Consequences

**Gained**

- The `unsafe` audit scope is fixed to one crate
- Six crates are tested without hardware
- The backend swap point is clear

**Lost / the cost**

- Refactors crossing crate boundaries are cumbersome. Types sometimes have to be
  lifted into `npuforge-common`
- `npuforge-common` is everyone's dependency, so touching it recompiles
  everything

**New constraints introduced**

- **Be careful about what goes into `npuforge-common`.** It is the contract, so
  lifting something needed by only one crate raises coupling
- The moment there is an urge to use `unsafe` in another crate, that is a signal
  to re-examine the design

## What would overturn this

- **If another NPU backend is added**, a new crate appears alongside
  `npuforge-rknn`. "unsafe in one place" widens to "unsafe only in the backend
  crates". At that point, where common FFI utilities live has to be decided

---

<a id="adr-007"></a>

# ADR-007. A dedicated RKNN context per thread, with sharing blocked by the type system

*[한국어 원문](007-per-thread-rknn-context.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Supersedes** | the judgement in `environment-matrix.md` §3.1 that "RKNN 2.3.0 is thread-safe, so the context may be shared" |
| **Related** | [ADR-020](#adr-020) (`worker_count=8`), `docs/discuss.md` §9, `docs/RESULTS.md` §4.3 |

---

## In one line

> Sharing a context produces **answers that are 100% wrong while raising not a
> single error**. Confirmed by measurement. So each worker gets its own
> context, and `infer` takes `&mut self` so that sharing does not compile at
> all.

## Context

### What a context is

In the RKNN Runtime a **context** is the handle produced by loading a model
into memory. Opening a `.rknn` file yields one context, and inference is
performed against it.

One inference is **three** function calls.

```text
rknn_inputs_set   put the input image into the context
rknn_run          run it on the NPU
rknn_outputs_get  take the result out
```

### What had to be decided

The node runs 8 workers (→ ADR-020). For those 8 to infer concurrently, one of
two choices had to be made.

| | |
|---|---|
| **Shared** | 8 workers use one context together. Less memory. Shorter code |
| **Dedicated** | each worker holds its own context. Uses more memory |

### Sharing was the original decision

`environment-matrix.md` §3.1 already recorded the conclusion that **"RKNN
Runtime 2.3.0 is thread-safe"**. If that is right, sharing is the obvious
choice. There is no reason to make eight of something when one will do.

### But it was suspicious

Two things stood out.

**First, one call being safe and a sequence being safe are different things.**

```text
thread A:  inputs_set(photoA) -----------> outputs_get()  <- what comes out?
thread B:            inputs_set(photoB) -> run()
```

Even if each individual `inputs_set` call is thread-safe, if B cuts in
**between** A putting in its input and taking out its result, A receives B's
result. The atomicity of individual calls and **the atomicity of a sequence**
are separate matters.

**Second, checking what that "thread-safe" verdict had actually looked at — it
was counting API return codes only.** It never compared output contents. Even
with results getting mixed up, the return codes come back a healthy
`ok 40 / err 0`.

## Decision

**1. Give each worker a dedicated context.** `ContextPool` creates
`worker_count` contexts and a semaphore has each worker take an idle one.

```rust
pub struct ContextPool {
    contexts: Vec<Mutex<RknnContext>>,   // an independent lock per context
    permits: Arc<Semaphore>,             // issued to the number of free slots
    ...
}
```

Since the semaphore permit is acquired first, **at least one must be free** in
the subsequent `try_lock` scan. If none is found, the semaphore and lock counts
have diverged, so it raises an internal error instead of quietly moving on —
left alone it would look only like an unexplained performance drop.

**2. Let the compiler block sharing.**

```rust
/// Taking `&mut self` is this type's concurrency contract.
/// The compiler blocks concurrent calls on the same context.
pub fn infer(&mut self, input: &[u8]) -> Result<Vec<u8>>
```

With `&self`, a shared call is **syntactically possible**. With `&mut self`,
code using the same context from two places at once simply does not build.

> This is the most important part of this ADR. **Writing "do not share" in a
> comment and blocking it with a type are different things.** This defect
> cannot be found by eye, so leaving it to human attention means it comes back
> eventually.

**3. Pool creation is all-or-nothing.** If any one of the 8 contexts fails to
open, the whole node fails. A node that came up half-way and quietly runs at
lower throughput is worse than one that dies clearly — in a benchmark such a
node gets recorded as "the slow node" and contaminates the conclusion.

## Rationale

### The measurement

Measured with `native/shared_context_test.c`. Each thread is given **a different
input**; a reference output is first captured by inferring alone, then the
concurrent results are compared against each thread's own reference.

```text
conditions: king, FP16, 4 threads x 50 = 200 inferences
```

| Configuration | API errors | **Result mismatches** |
|---|---:|---:|
| Shared context | 0 | **200 / 200 (100%)** |
| Per-thread dedicated | 0 | 0 / 200 (0%) |

**Sharing raised not a single error and got everything wrong.**

### Why this defect is especially bad

- **No exception and no error code.** Nothing is left in the logs
- **It never reproduces in a single-threaded test.** It passes CI
- **The throughput metric actually looks better.** Two threads sharing reached
  34.8 inf/s against 33.2 dedicated — **it was producing wrong answers faster**
- **It looks plausible to the eye.** Being detections from another frame, the
  output is not garbage but "boxes that make sense"

Had this gone unnoticed, it would very likely have reached a public talk with
**all throughput figures valid and only the detection results quietly wrong**.
The structure was one where performance gets boasted about and accuracy gets
checked by nobody.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| One shared context | 100% wrong answers when measured. Out of the question |
| One context + a mutex to serialize | Correct answers, but the NPU is used one at a time. The point of 8 workers disappears |
| Duplicate with `rknn_dup_context` | Not verified. Individual `rknn_init` already gives correct answers with adequate performance, so it dropped down the priority list |
| Stating "do not share" in comments and docs | This defect is invisible. A rule that humans have to keep gets broken eventually |

## Consequences

**Gained**

- Zero result mismatches under 8-worker concurrent inference
- Sharing code became **impossible to write**
- The check is part of six real-hardware integration tests
  (`crates/npuforge-rknn/tests/real_device.rs`)

**Lost / the cost**

- Uses the memory of 8 contexts. **How much more was not measured.** With 4 GB
  of node RAM and a 6.46 MB model (INT8) it was judged unlikely to matter and
  left there — not a reasoned judgement, just deferred because the headroom
  looked large
- Pool creation time scales with context count (once at node startup)

**New constraint introduced**

- **The meaning of `supports_concurrent_infer = true` has changed.** It used to
  mean "the runtime handles it", and now means **"the backend serializes it
  through a pool"**. The value is the same; the basis differs
- Raising `worker_count` raises the context count with it. This value must not
  be increased without checking memory headroom

## What would overturn this

If RKNN ships a version that separates per-call context state, it can be
revisited.

**But the re-verification criteria are pinned down in advance.**

- ❌ Do not judge by API return codes. That method missed this defect
- ✅ **Give each thread a different input and compare byte-for-byte against the
  standalone reference output.** Zero mismatches to pass
- ✅ Higher throughput is not grounds for passing. We have already seen that a
  configuration producing wrong answers fast looks faster

## The lesson left behind

This incident was **the third** of the same type of mistake.

```text
1. reading run_duration as NPU occupancy time      -> it included queue wait
2. sampling NPU load with delayms=3000 still set   -> it was reading a 3-second average
3. judging thread-safety by API return codes only  -> results never compared   <- this ADR
4. judging throttling by NPU clock alone           -> the CPU was the one bending
```

What they share: **not checking what a metric counts and trusting it by its
name.**

The rule that came out of this is `preflight-check.sh --with-inference`.
**Before measuring performance, check that the three boards give the same
answer to the same input.** A configuration that produces wrong answers fast
must not win a benchmark.

---

<a id="adr-008"></a>

# ADR-008. Internal communication uses gRPC (tonic + Protocol Buffers)

*[한국어 원문](008-grpc-tonic-protobuf.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-003](#adr-003), [ADR-012](#adr-012), [ADR-024](#adr-024), `docs/01-TECHSPEC.md` §5.3, §7 |

---

## In one line

> Client↔scheduler and scheduler↔node communication uses **gRPC**. The schema
> lives in one place as `.proto` and Rust code is generated from it. The
> management API and the dashboard's REST/JSON are kept separate.

## Context

Most of what moves through this system is **a large binary blob**.

```text
request   raw RGB 640x640x3   = 1,228,800 byte
response  9 raw tensor blobs  = 1,218,000 byte  (want_float=0)
```

And although there are only three nodes, hundreds of these move per second
(the INT8 3-node target is 471 inf/s).

There were three protocol candidates.

| | |
|---|---|
| REST + JSON | works everywhere and is easy to debug. Inflates binary with base64 |
| gRPC | binary as-is, schema enforcement, code generation |
| A hand-rolled binary protocol | could be fastest. Everything has to be built by hand |

## Decision

**1. Internal RPC is gRPC + Protocol Buffers.** Implemented with `tonic`.

**2. The schema lives in one place, the `npuforge-proto` crate.** Rust types
are generated from `.proto` at build time.

**3. The services are split in two.**

| Service | Direction | Purpose |
|---|---|---|
| `SchedulerService` | client → scheduler | `Infer`, `BatchInfer`, `ListNodes` |
| `NodeService` | scheduler → node | inference delegation, status queries |

Node registration and heartbeats also travel over gRPC.

**4. The management API and dashboard are separate, on REST/JSON + axum.**
They are called directly from the browser, so putting them on gRPC would add
another gateway.

**5. The payload arrives as a single `bytes` field.** The tensor structure is
described not by protobuf but by our own blob format
([ADR-012](#adr-012)).

## Rationale

### 1. Avoid base64

Sending a 1.23 MB image over REST/JSON requires base64 encoding. That makes it
**about 1.33× larger** and adds encode/decode CPU on both ends.

Both are damaging in this project. The network is already close to saturation
at aggregation
([ADR-014](#adr-014)), and CPU is already a
bottleneck under sustained load.

### 2. The schema has to be in one place or three nodes drift apart

The three nodes run **the same binary**, but the scheduler runs on a separate
host. If message definitions are scattered through the code, one side gets
updated without the other.

With `.proto` as the single source, both sides are generated from the same
definition.

### 3. The timing breakdown fields have to travel structured

Eleven timing fields (`TimingBreakdown`) come back with each response. They are
this project's central output, so **a field must not silently disappear.** The
protobuf schema enforces that.

### 4. The Rust ecosystem is ready

`tonic` runs on Tokio and comes with streaming, timeouts and connection reuse
built in. Reusing a per-node channel to reduce connection cost also works
directly.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| REST + JSON (internally too) | 1.33× base64 inflation plus encoding CPU. Both network and CPU are already tight |
| REST + multipart/octet-stream | Avoids the inflation but loses schema enforcement. The timing fields would have to be kept in sync by hand |
| A hand-rolled binary protocol | Could be fastest, but reconnection, streaming and error propagation all have to be built. That time is time not spent measuring |
| Extending gRPC to the management API | Cannot be called directly from a browser. Adds a grpc-web gateway |

## Consequences

**Gained**

- Binary carried without inflation
- A single source for message definitions
- The mock 3-node integration test **runs over real gRPC** — it is one process,
  but the transport path is the same as on real hardware

**Lost / the cost**

- Cannot be poked directly with `curl`. Another debugging tool is needed
- Changing `.proto` involves the build pipeline (`build.rs`)
- There are now two protocols (gRPC + REST). Error representation has to agree
  across both → [ADR-024](#adr-024)'s `NPF-xxxx` is that glue

**New constraint introduced**

- Message size limits have to be managed explicitly. A 1.23 MB request fits
  under the default 4 MB limit, but experiments that increase input size (S6)
  will need to check

## What would overturn this

- **If the input becomes JPEG and payloads drop to the 100 KB class**, the
  absolute cost of base64 inflation shrinks. The schema-enforcement reason
  still stands
- **If a public external API becomes a requirement**, consider putting a REST
  gateway in front of gRPC. That is not a reason to change the internal
  protocol

---

<a id="adr-009"></a>

# ADR-009. Fix the policies at three, and have all three share the candidate filter

*[한국어 원문](009-three-policies-shared-filter.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-003](#adr-003), [ADR-010](#adr-010), `docs/01-TECHSPEC.md` §10.0, §10.4 |

---

## In one line

> There are only `round-robin` / `least-queue` / `ect`. And **all three pass
> through exactly the same candidate filter.** If the filters differed, a policy
> comparison would measure the filters rather than the policies.

## Context

Comparing scheduling policies (scenario S3) is one of this project's
measurement items. It aims to measure "how much better is choosing by load than
simply going round in order".

A policy consists of two parts.

```text
1. candidate filter   who is eligible  (exclude dead nodes, only nodes holding the model ...)
2. selection rule     who among the candidates  (in order / shortest queue / estimated completion time)
```

There is a trap here. **If part 1 is made different per policy**, then when
policy A comes out ahead of policy B, there is no way to know whether that was
the selection rule or the filter.

For instance, if only ECT carried "exclude nodes above 85 °C", whether ECT wins
because it is smarter or because it avoids hot nodes cannot be separated.

## Decision

**1. Fix the policy identifiers at three.**

| Identifier | Policy | Purpose |
|---|---|---|
| `round-robin` | Round Robin | comparison baseline |
| `least-queue` | Least Queue | intermediate comparison |
| `ect` | Estimated Completion Time | recommended default |

**2. All three pass through an identical candidate filter.**

```text
- must be in an is_schedulable() state
- must hold the requested model in a Ready state
- temperature must be below disable_temperature_c
```

**3. Parse the identifier string in exactly one place.**

```rust
#[serde(rename_all = "kebab-case")]
pub enum SchedulingPolicyKind { RoundRobin, LeastQueue, Ect }
```

The configuration file, CLI arguments, metric labels, logs and dashboard all use
**the same strings**. Variants such as `queue-aware`,
`estimated-completion-time` or `queue_aware` are not used.

**4. Narrow the interface to the selection rule alone.**

```rust
pub trait SchedulingPolicy: Send + Sync {
    fn select_node(&self, task: &InferenceTask, candidates: &[NodeSnapshot])
        -> Result<NodeId, ScheduleError>;
}
```

`candidates` is **a list that has already passed the filter**. Since the policy
never sees the full node list, the room for a policy to add its own filter is
structurally reduced.

## Rationale

### Policy comparison is one of this project's measurement items

S3 is an experiment measuring "the difference between policies". There must be
one variable. Without a shared filter, the experimental design itself is void.

### A wobbling identifier contaminates the results

This actually came up while designing the bench tool. Having `--policy
round-robin` typed by hand invites a typo, or a value attached to the results
that differs from the scheduler's actual configuration. **A result labelled with
the wrong policy name ruins the whole of S3.**

So the bench tool **prefers the value the scheduler reports** over the one typed
by hand. It pairs with this decision.

### Three is enough

- `round-robin` is the baseline. Without it there is no way to know whether the
  rest are good
- `least-queue` answers "is looking at the queue alone sufficient?"
- `ect` looks at queue, speed, temperature and errors together

A fourth would multiply the experimental combinations and increase S3's run
count. It would not be worth it within the budget of 146 runs and roughly 23.4
hours.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| A different filter per policy | S3 would measure filter differences. **The thing most to be avoided** |
| Open the policies up as plugins | The comparison set becomes unbounded. Fixed is better for a measurement project |
| Implement only one policy (ECT) | Without a baseline there is no way to say "how much better" |
| Free-form identifier strings | Typos and notation drift contaminate the result labels |

## Consequences

**Gained**

- The S3 policy comparison holds — the single variable is the selection rule
- Policy names in configuration, logs, metrics and the dashboard are always the
  same
- Policy implementations get shorter. They do not each write a filter

**Lost / the cost**

- Policy-specific candidate conditions cannot be added. Adding one means
  **putting it in the shared filter and applying it to all three**
- Adding a new policy means editing the enum (deliberate friction)

**New constraint introduced**

- Changing the filter makes results **incomparable with the three policies' past
  measurements**. A filter change is treated as a change of experimental
  conditions and has to be recorded

## What would overturn this

- **If a candidate condition is found that genuinely must differ per policy.**
  At that point, first check whether it can be expressed as a score inside the
  selection rule — ECT's `load_factor` works that way
  ([ADR-010](#adr-010))
- **If the M7 optimization experiments need a new policy**, add a fourth. But
  only after S3's baseline comparison has already finished with three

---

<a id="adr-010"></a>

# ADR-010. The ECT score formula and each term inside it

*[한국어 원문](010-ect-formula.ko.md)*

| | |
|---|---|
| **Status** | accepted (before real-hardware validation) |
| **Date** | 2026-08-06 |
| **Related** | [ADR-009](#adr-009), [ADR-027](#adr-027), `docs/01-TECHSPEC.md` §10.4 |

---

## In one line

> The default policy, ECT, scores **"if this request goes to that node, when
> will it finish"** and picks the lowest. Every term in the formula has a
> reason, and in particular `+ 1` and `load_factor` make it misbehave if
> removed.

## Context

Least Queue picks "the node with the shortest queue". That is enough when nodes
perform identically, but in practice they do not.

```text
node A  queue 2, 50 ms each   ->  free in about 100 ms
node B  queue 1, 200 ms each  ->  free in about 200 ms
```

Least Queue picks B. **Wrong.** Queue length alone cannot tell you "when it will
be free". Nodes differ in speed, they slow down with heat, and they may have
been failing recently.

## Decision

```text
ECT = ((queue_depth + in_flight + 1) x EWMA_inference_time
       + EWMA_network_time
       + thermal_penalty
       + error_penalty)
      / load_factor
```

The node with the lowest score is chosen. Ties break on **Node ID in
lexicographic order**.

### Each term

| Term | Meaning |
|---|---|
| `queue_depth` | requests on that node not yet started |
| `in_flight` | requests currently being processed |
| `+ 1` | **this very request being assigned** |
| `EWMA_inference_time` | moving average of recent inference times. The node's actual speed |
| `EWMA_network_time` | moving average of scheduler↔node round trip |
| `thermal_penalty` | added when the temperature is high |
| `error_penalty` | added when errors have been frequent recently |
| `load_factor` | a per-state weight. The divisor |

## Rationale

### Why `+ 1` cannot be omitted

Two reasons.

**First, that is ECT's definition.** It estimates "when will this request
finish", so **its own inference time has to be included**. Placing it on a node
with 2 ahead means 3 including mine.

**Second, without it `load_factor` is neutralised.**

```text
a node with an empty queue:  (0 + 0) x EWMA = 0
                             0 / load_factor = 0     <- always 0, whatever the state
```

Zero divided by anything is zero. The `Recovering` suppression below disappears
entirely.

### The problem `load_factor` solves

| State | load_factor |
|---|---:|
| Healthy | 1.0 |
| Busy | 1.0 |
| Degraded | 0.5 |
| Recovering | 0.25 |
| Otherwise | 0.0 (excluded from candidates) |

**A `Recovering` node has an empty queue, so on score alone it always wins.**
Every request piles onto a node that has just come back, and it dies again from
the same cause.

PRD FR-07 requires "assign only limited requests to a recovered node". This
could have been implemented with a separate counter or token bucket, but it is
**expressed as a single score.** Dividing by `0.25` quadruples the score, so it
naturally gets picked less.

The point is putting state into **the score rather than the candidate filter**.
Filtering gives only "use it or do not"; a score can express **degree**.

### Why tie-breaking is fixed to lexicographic Node ID

**Reproducibility.** Breaking ties randomly or by hash order would give a
different distribution each time the same experiment is repeated. That inflates
the variance of scaling-efficiency measurements, with no way to explain where
the variance came from.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Use Least Queue only | Does not reflect node speed differences. Wrong answer in the A/B example above |
| Exclude `Recovering` from candidates | A recovered node never comes back in. Then when to admit it has to be decided anyway |
| A separate token bucket to limit recovered nodes | One more piece of state. The score formula alone does the job |
| Break ties randomly | Reproducibility breaks |
| Handle temperature and errors as filters only | Becomes binary. 79 °C and 81 °C get treated as entirely different |

## Consequences

**Gained**

- Node speed differences, temperature, error rate and recovery state
  **unified into one score**
- Recovered-node suppression implemented without additional state
- Ties are deterministic, so repeated experiments reproduce

**Lost / the cost**

- **More tuning parameters.** The EWMA coefficients, the magnitudes of
  `thermal_penalty` / `error_penalty`, the `load_factor` values — all have to be
  set
- The formula is complex enough that "why was this node picked" is hard to read
  straight off a log

**New constraints introduced**

- **Not yet validated on real hardware.** Behaviour was confirmed on a 3-node
  Mock, but whether `load_factor` and the penalty values are actually right has
  to be seen in M4. The current values are **a draft**
- The temperature thresholds (80 / 90 °C) are a draft too. They are reset after
  the formal S0 thermal measurement

## What would overturn this

- **If ECT is not better than Least Queue in M4's real-hardware validation**,
  suspect the formula. Though that result is itself a valid output
  ([ADR-002](#adr-002))
- **If a `Recovering` node still gets overloaded even at 0.25**, lower the value
  or add an absolute cap
- **If the penalty terms turn out to have no effect at all**, removing them is
  also a result. A term existing and a term working are different things

---

<a id="adr-011"></a>

# ADR-011. The reference model is INT8

*[한국어 원문](011-int8-quantization.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-012](#adr-012), [ADR-014](#adr-014), [ADR-018](#adr-018) (model deployment), `docs/discuss.md` §8 |

---

## In one line

> INT8 quantization is worth **1.86×**. It landed an order of magnitude harder
> than any software optimization attempted so far. The cost is −5.5% on the top
> detection score, and **the detection set and classes are identical**.

## Context

### What quantization is

A neural network normally computes in reals (FP32). **Shrinking those reals to
8-bit integers** for computation is INT8 quantization. Each multiplication gets
cheaper and less memory moves. In exchange, values get coarser and a little
accuracy is lost.

FP16 sits in between — still real, just half the bits.

### Why this choice mattered

Starting from FP16, three things were tried to raise one node's throughput, and
**all three failed.**

| Attempt | Result |
|---|---:|
| Manual NPU core assignment via `core_mask` | +0.1% |
| `want_float=0` (measured mostly single-threaded at the time) | +5.4% |
| Zero-copy buffer reuse | **−1.8%** |

The reason was found too. Each inference triggers about 76 kernel `ioctl` calls
and those get **serialized**. Not something the application could reduce. So the
conclusion at the time was **"the node ceiling of 78 inf/s is a driver
characteristic."**

INT8 was the last big variable still outstanding.

## Decision

**1. The reference model is YOLOv8n INT8.**

**2. FP16 is not deleted but kept as a comparison condition.** Presenting the
two models side by side is itself the result of "how much does quantization
buy".

**3. Define the accuracy acceptance criterion at the detection level rather than
raw tensor similarity.** (See the trap section below for why.)

## Rationale

### 1.86×

```text
conditions: king, sustained_load_test, 8 threads fixed, 120 s,
            governor=performance, fanless
```

| Model | Throughput | Mean latency | Model size |
|---|---:|---:|---:|
| YOLOv8n FP16 | 84.3 inf/s | 94.5 ms | 9.65 MB |
| **YOLOv8n INT8** | **157.2 inf/s** | **50.8 ms** | 6.46 MB |
| Ratio | **1.86×** | −46% | −33% |

> Initial values measured with the `ondemand` governor were FP16 79.0 / INT8
> 146.2. **The 1.85–1.86× ratio holds regardless of governor.**

### This measurement corrected an earlier conclusion

If INT8 is 1.85×, that conflicts with the explanation that "76 ioctls set the
ceiling". So INT8's ioctls were counted too.

```text
strace -c -f -e trace=ioctl, 1 thread, 20 s

        inferences  throughput    ioctls per inference
FP16    315         15.7 inf/s    76.4
INT8    718         35.8 inf/s    76.2
```

**The call count is identical and throughput is 2.28×.**

What sets the ceiling is not the **number** of ioctls but **how long one
inference holds the serialized section.** So the scope of the previous
conclusion was narrowed.

| Previously | Corrected |
|---|---|
| "The node ceiling of 78 inf/s is a driver characteristic" | "**On FP16**, the node ceiling is about 78 inf/s, and that value cannot be exceeded by application optimization" |
| "It cannot be exceeded by application optimization" | Stands. But **quantization is a model change, not an application optimization** |

### The accuracy cost is acceptable

```text
conditions: real board king, COCO val2017 images,
            preprocessing done in one place so both see the same input bytes
```

| Comparison | box cosine | Detection cells | Class agreement |
|---|---|---|---|
| FP16 vs ONNX | 0.99999 | 10/10 | 100% |
| **INT8 vs FP16** | **0.997** | **10/10** | **100%** |

The top detection's cell moves by one and its score is −5.5%. **The detection
set and classes are identical.** Buying 1.86× at that price is a good trade.

## ⚠️ The trap hit during accuracy verification

**Using raw-tensor cosine similarity as the acceptance criterion misjudges this
model.**

Even for FP16 vs ONNX — a comparison with no quantization at all — **the cosine
of some tensors falls to 0.16.** Looking at that number alone leads to "the FP16
conversion broke the model". A wrong conclusion.

The cause is this.

- Of YOLOv8n's 9 outputs, tensors 2/5/8 are **the sum of 80 class scores**
- RKNN's sigmoid does not output exactly 0 but has **a floor of 0.001831**
- Amplified 80×, that produces **a 0.1465 offset** (matching the measured floor
  exactly)
- Most output cells are background, so this offset dominates the cosine

**The same value is added to every cell, so the ranking does not change. The
detections are unaffected.**

→ The acceptance criterion was changed to the **detection level** (detection
set, classes, box cosine). `tools/model-converter/compare_detections.py`
compares against that criterion.

This too is one of this project's recurring failure types. **A metric's name was
read and its meaning assumed.** "Low cosine similarity = different results" is
generally true, but not for this output structure.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Stay on FP16 | Throws away 1.86×. And it has already been confirmed there is no way to produce that much in software |
| FP32 | Meaningless on this NPU. Big and slow |
| INT8 + accuracy-loss compensation (QAT and the like) | Requires retraining. This project builds an inference runtime, not trains models |
| A larger model (YOLOv8s and the like) at INT8 | The comparison baseline changes. Model selection is a separate decision, and only one variable moves at a time here |

## Consequences

**Gained**

- 157.2 inf/s per node. 1.86× against FP16
- Mean latency 94.5 → 50.8 ms
- Model size −33%

**Lost / the cost**

- Top detection score −5.5%, top detection cell moved by one
- **Calibration data became necessary.** 200 COCO val2017 images are chosen
  deterministically (`fetch_calibration.py`, fixed seed). The images are not put
  in the repository for licensing reasons; only a manifest is kept
- **INT8 conversion is not byte-reproducible.** Converting three times from the
  same input gave a different hash each time (same size, 1.8% of bytes
  differing). But **the inference results are completely identical** (all 9
  tensors at cosine 1.000000). The difference is in serialization and layout,
  not in computation → the model is converted once and deployed to all three
  nodes (ADR-018)

**New constraint introduced**

- **Network load went up instead.** With throughput at 1.86×, the bytes moving
  per second rise by the same factor — 1.545 Gbps per node, 4.636 Gbps across
  three. This decision is the direct cause of
  [ADR-014](#adr-014)'s 10G aggregation
- Kept as a case of something else filling up when performance improves

## What would overturn this

- **If an input or model appears where the detection set differs.** The current
  basis is a single image. That the sample is small is acknowledged in using it
- **Re-verification is done at the detection level, not by tensor cosine.** The
  trap section above is why. Forgetting this criterion and judging by cosine
  would mean discarding a perfectly good model

---

<a id="adr-012"></a>

# ADR-012. The node sends integers without dequantizing (`want_float=0`, blob v2)

*[한국어 원문](012-want-float-zero-blob-v2.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-12 |
| **Related** | [ADR-011](#adr-011) (INT8 adopted), [ADR-014](#adr-014) (10G aggregation), [ADR-021](#adr-021) (node-side postprocessing not implemented), `docs/discuss.md` §12 |

---

## In one line

> If the node converts results to `float32` before sending, **the response
> becomes 3.96× the request**, and even a 10G link is not enough at three-node
> saturation. So it sends **the quantized integers as they are**, with `scale`
> and `zero_point` included in the response so the receiver can convert back.

## Context

### Quantization and dequantization

An INT8 model computes in integers. Its outputs come out as integers too, and
converting them back to reals needs two values attached to each tensor.

```text
real = (quantized - zero_point) x scale
```

RKNN has **an option to do that conversion for you**.

| | |
|---|---|
| `want_float = 1` | the runtime converts to `float32` and hands that over. Convenient |
| `want_float = 0` | gives the model's native type as-is (int8 for an INT8 model) |

The default is `1`, and that is what was used at first, because it was
convenient.

### But in this project the output goes out over the network

The node does not postprocess (no NMS). It **sends all nine raw tensors back to
the scheduler** (→ ADR-021). So the output type is the link load.

```text
input                        1,228,800 byte   (640 x 640 x 3)
output want_float=1 (f32)    4,872,000 byte   <- 3.96x the input
output want_float=0 (int8)   1,218,000 byte   <- 0.99x the input
```

The load on the scheduler-side link at three-node saturation:

| Configuration | Model | 3-node TX | 3-node RX | Fits in 10G? |
|---|---|---:|---:|---|
| `want_float=1` | INT8 | 4.64 Gbps | **18.38 Gbps** | **no** |
| `want_float=1` | FP16 | 2.49 Gbps | 9.86 Gbps | barely |
| `want_float=0` | INT8 | 4.64 Gbps | 4.60 Gbps | yes |
| `want_float=0` | FP16 | 2.49 Gbps | 2.46 Gbps | yes |

The original error was **calculating the network from the input alone and
omitting the output**. Recomputing with the output inverted the conclusion —
even laying 10G would not carry three INT8 nodes.

At that point `want_float=0` was promoted from "a nice optimization to have" to
**a precondition for starting M3**. The grounds for the promotion were not
throughput but **RX bandwidth**.

## Decision

**1. Change the default of `want_float` to `false` and expose it in node
configuration.**

```toml
[worker]
want_float = false
```

**2. Bump the response blob format to v2 and carry dequantization parameters
per tensor.**

```text
magic    u32  "RKNT"
version  u32  = 2
count    u32  number of tensors
dtype    u32  0 = model's native dtype, 1 = float32
per tensor (36 byte):
  len  n_dims  dims x 4   <- present in v1 too (24 byte)
  qnt_type  zero_point  scale   <- added in v2 (12 byte)
followed by the tensor data
```

**Why this is not optional**: send int8 without `scale` and `zero_point` and
the receiver has no way at all to interpret those bytes. The numbers arrive and
nobody knows what they mean. The moment the decision was made to send integers,
carrying the parameters became an obligation that follows from it.

**3. Old blobs are not accepted.** `decode` rejects `version != 2` as an error.
Reading a 36-byte descriptor as 24 bytes because only the header says v1
produces silently misaligned values, and that is the failure mode this project
most wants to avoid.

## Rationale

### Accuracy — matches float32 on real hardware

Since we do the dequantization ourselves, it has to be checked against the
runtime's result.

```text
measured: real board king, 9 tensors
(a) the float32 received with want_float=1
(b) the int8 received with want_float=0, dequantized by hand
```

**Maximum error 9.5e-7.** At the limit of `float32` precision, so effectively
identical. (`crates/npuforge-rknn/tests/real_device.rs`)

### Throughput — 15–17% higher as a bonus

```text
conditions: king, 8 threads, 120 s, governor=performance
```

| Model | `want_float=0` | `want_float=1` | Gain |
|---|---:|---:|---:|
| INT8 | **156.7 inf/s** | 133.6 inf/s | **+17.3%** |
| FP16 | 66.9 inf/s | 57.8 inf/s | **+15.7%** |

Dequantization is done by the CPU. Not doing that work makes it faster.

> **Why was it +5.4% before.** The first measurement on 2026-08-10 was mostly a
> single-thread condition and came out at +5.4%, which got it filed as "an
> optimization with no effect". The reason the gap widens at 8 threads is that
> **the time output conversion holds the serialized section** accumulates with
> the number of concurrent threads. Kept as a case of the same experiment
> yielding a different conclusion under different conditions.

### As it turns out, the measurement tool was on this setting all along

`sustained_load_test` had **hardcoded `want_float=0` from the beginning**. So
the **157.2 / 84.3 inf/s written into the documents as settled figures were
already on `want_float=0`**, and only the Rust backend was on `true`.

Which means this change did not raise performance; it **brought the software in
line with the measurement conditions**. Put the other way: until the change,
the actual node was running 15–17% slower than the documented figures and
nobody knew.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Keep `want_float=1` + 10G | RX computes to 18.38 Gbps. **Even 10G does not work.** 25G is outside this project's budget and purpose |
| Postprocess (NMS) on the node | **This is ultimately the right answer.** The response shrinks to a few KB and RX effectively disappears. But it is unimplemented, and putting postprocessing on the node shifts CPU load to the node and changes the measurement conditions again → ADR-021 |
| Compress the response | Compression/decompression CPU enters the inference path. CPU is already the bottleneck, and this stacks on top |
| Parameters as separate fields rather than in the blob | The values differ per tensor, so attaching them to the tensor descriptor is natural. Separating them creates room for the ordering to break |

## Consequences

**Gained**

- 3-node RX 18.38 → 4.60 Gbps. **M3 became possible on 10G aggregation**
- Throughput INT8 +17.3% / FP16 +15.7%
- The node now actually matches the measurement conditions written in the docs

**Lost / the cost**

- **Responsibility for dequantization moved to the receiver.** The client has to
  understand the blob
- Changing the format means **fixing three places together**
  - `crates/npuforge-rknn/src/blob.rs`
  - `native/dump_output_test.c` (board verification tool)
  - `tools/model-converter/compare_detections.py` (accuracy comparison)
- Incompatible with v1 blobs (intentionally)

**Known flaw**

The response's `result_format` string is still **`"rknn-tensors-v1"`**. The
actual blob header says `version = 2` and the descriptor changed from 24 to 36
bytes. A client identifying the format by that string would mistake it for v1
and read in 24-byte units.

It does not surface today because every consumer is inside this repository.
**The name and the reality disagree, so this has to be cleaned up before going
public.**

## What would overturn this

- **Implementing node-side postprocessing (NMS)** shrinks the response to a few
  KB of detections and makes the blob itself largely unnecessary. At that point
  this ADR is superseded by ADR-021
- **If the input format becomes JPEG**, input TX drops tenfold and the whole
  link budget has to be recomputed. The output-side conclusion stands regardless
- Revisit if an observation shows dequantization error changing postprocessing
  results. The current basis is a per-tensor maximum error of 9.5e-7, and
  **no comparison was made at the level of detection boxes**

---

<a id="adr-013"></a>

# ADR-013. Make fanless the default, and treat throttling as something to measure rather than eliminate

*[한국어 원문](013-fanless-thermal-as-measurement.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-10 |
| **Related** | [ADR-002](#adr-002), [ADR-023](#adr-023), `docs/02-HARDWARE-SETUP.md` §9 |

---

## In one line

> Attaching a fan improves the numbers. But **edge devices sit in the field
> without one.** So fanless is the default condition, and performance falling
> from heat is treated as **something to measure, not something to remove.**
> Cooled conditions are measured separately as a comparison group.

## Context

RK3576 boards ship fanless. Put them under sustained load and they get hot and
slow down.

There are two branches here.

```text
branch 1. attach a fan
  -> the numbers improve
  -> good for a talk
  -> but those numbers do not occur in the field

branch 2. measure fanless
  -> the numbers get worse
  -> and the amount by which they get worse is a value nobody publishes
```

The TOPS vendors publish is **instantaneous performance**. How much of it is
sustained under load — **the gap between peak FPS and sustained FPS** — is
barely covered in public material.

## Decision

**1. Fanless (condition A) is the default measurement condition.**

**2. Active cooling (condition B) is measured alongside as a comparison group.**
Three fans of the same model are fixed at the same speed. Different speeds would
give the nodes different cooling conditions and break three-node symmetry.

**3. Thermal characterisation (S0) comes before every other scenario**, because
S0 determines the thresholds and cooldown times for the rest of the experiments.

**4. Do not mix improvised cooling into a measurement.** A desk fan was used
once during diagnosis; **it was valid for diagnosis but unusable as a
measurement condition.** There is a checklist item to confirm the desk fan is
off before a fanless measurement.

## Rationale

### Some questions can only be answered by measuring both conditions

```text
fanless only  ->  you do not know "how much better does cooling make it"
cooled only   ->  you do not know "how much do you get in a real edge deployment"
```

**Measure both and "the effect of cooling on scaling efficiency" becomes a
result in itself.** That is a value absent from vendor spec sheets, and it fits
this project's identity of settling things by measurement.

### Measured — it finishes fanless, but throughput is not sustained

```text
conditions: 3 boards concurrently, 8 threads, 900 s, fanless, no desk fan
```

| Board | NPU mean | NPU peak | Throughput |
|---|---:|---:|---:|
| king | 73.0 °C | 75.8 °C | 80.5 inf/s |
| queen | 67.5 °C | 70.2 °C | 77.7 inf/s |
| jack | 72.6 °C | 74.8 °C | 77.8 inf/s |

- Node-to-node spread **5.6 °C**
- Completed with 0 errors
- Never exceeded 90 °C

**Sustained 8-thread load is possible fanless.** But throughput is not
sustained.

```text
 +10s  81.6 inf/s   <- start
+120s  63.6
+300s  59.7         <- steady state.  -27% against the start
```

### ⚠️ What was collapsing was the CPU, not the NPU

The initial verdict was "no NPU throttling", because all 928 samples were at
950 MHz. **Only the NPU clock had been looked at.**

Looking at the CPU clocks in the same log:

```text
        NPU temp   npu_clk   cpu4(A72)   cpu0(A53)
 +15s   86.8 C     950 MHz   2208 MHz    2016 MHz
 +30s   90.4 C     950 MHz   1416 MHz    1200 MHz
 +60s   87.8 C     950 MHz    816 MHz     600 MHz
+120s   87.8 C     950 MHz    816 MHz     600 MHz
```

**The NPU never drops and the CPU falls 63–70%.**

One inference is `set input (CPU) → NPU → get output (CPU)`, so the CPU sections
feed directly into throughput. That was known, and the throttling verdict was
still made on the NPU alone. It is the **fourth** mistake of this type in this
project.

> The discovery actually improved the result. **"What collapses first on a
> fanless edge device is not the NPU but the CPU handling either side of it"** —
> a far better narrative for a talk.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Measure with a fan attached | Numbers that do not reproduce in the field. The opposite direction to the project's premise |
| Measure fanless only | Cannot answer "how much better does cooling make it" |
| Lower the load to avoid throttling | What happens under sustained load is exactly what is being measured, and this avoids it |
| Standardise conditions with improvised cooling (a desk fan) | Not reproducible and not uniform across nodes |

## Consequences

**Gained**

- The peak vs sustained gap became one of the project's central outputs
- The discovery that the bottleneck is the CPU rather than the NPU
- Readiness to quantify the cooling effect (S0-A / S0-B)

**Lost / the cost**

- Throughput figures come out lower. Instead of "84.3 inf/s" it has to be
  "81.6 at the start, 59.7 at 300 seconds"
- Measurement takes longer. Cooldown has to be waited out, and fanless is slow.
  So cooldown has **an upper bound**, and when that bound is hit the actual
  starting temperature is recorded with the result

**New constraints introduced**

- **Thermal verdicts must include the CPU clock.** Judging by NPU clock alone
  was confirmed wrong. `run-thermal-comparison.sh` has to be fixed accordingly
- Do not compare temperatures between two measurements with different load
  profiles. A sweep load was once compared against a fixed load and a 19 °C gap
  was misread
- The temperature thresholds (80 / 90 °C) are **a draft**. They are reset after
  the formal S0

## What would overturn this

- **If a case or heatsink becomes the standard configuration**, condition A's
  definition changes
- **If fanless exceeds 90 °C in S0 and nodes start dropping out of scheduling**,
  measurement itself becomes impossible. At that point condition B is promoted
  to default and condition A is redefined as "the limit condition"

---

<a id="adr-014"></a>

# ADR-014. Leave the worker links at 2.5G, raise only aggregation to 10G, and put the scheduler on a separate server

*[한국어 원문](014-10g-aggregation-separate-scheduler.ko.md)*

| | |
|---|---|
| **Status** | accepted (equipment obtained and measured 2026-08-20) |
| **Date** | 2026-08-12 (decision), 2026-08-20 (build and measurement) |
| **Supersedes** | the judgement that "2.5GbE is sufficient as the reference network" |
| **Related** | [ADR-003](#adr-003), [ADR-011](#adr-011), [ADR-012](#adr-012), `docs/02-HARDWARE-SETUP.md` §3.3.2 |

---

## In one line

> Traffic from three nodes converges at one point, the scheduler. **It is that
> confluence, not the worker link (2.5G), that fills up first.** So only
> aggregation is raised to 10G, and a **separate server** that can take a 10G
> NIC becomes the scheduler host.

## Context

### The original calculation went like this

```text
3 nodes 150 FPS x 1.23 MB ~ 184 MB/s ~ 1.5 Gbps  -> exceeds 1GbE, 2.5GbE is enough
```

That calculation set "the reference network is 2.5GbE". **Two things were
wrong.**

**(a) The throughput assumption was stale.** 150 FPS was assumed as the
**total** across three nodes. Measurement says **one** node does 157.2 inf/s at
INT8. Three nodes is 471 inf/s.

**(b) The output direction was ignored.** Only the input was counted. The node
does not postprocess and sends raw tensors back, so the response uses the link
too. With `want_float=1` the response was **3.96×** the request.

On top of that there was a unit error — converting MiB/s to Gbps used the
binary prefix (÷1024). **Network speeds are decimal.**

### The recomputed values

```text
one raw RGB input = 640 x 640 x 3 = 1,228,800 byte

                     per node        3-node total
INT8  157.2 inf/s    1.545 Gbps      4.636 Gbps
FP16   84.3 inf/s    0.829 Gbps      2.486 Gbps
```

**Even FP16's three-node total of 2.486 Gbps exceeds a single 2.5GbE link
(effectively about 2.35 Gbps).** INT8 exceeds it by nearly double.

## Decision

**1. Leave the worker links at 2.5G.** At most 1.545 Gbps per node, which fits.

**2. Raise only the aggregation link to 10G.**

```text
        Benchmark / Scheduler Server
                    |
                  10GbE          <- this is the point
                    |
            2.5G / 10G Switch
              |-- 2.5G -- king
              |-- 2.5G -- queen
              \-- 2.5G -- jack
```

**3. Make the scheduler host a separate server with a PCIe slot.**

**4. Reduce the output alongside.** Even with 10G laid, `want_float=1` puts RX
at 18.38 Gbps and it is still not enough. →
Solved in [ADR-012](#adr-012).

**5. Keep 1GbE rather than removing it, as a comparison condition.** Presenting
"the network is the bottleneck" and "it is not" side by side has value as a
bottleneck-analysis result (scenarios S5 and S6).

## Rationale

### Why aggregation rather than the workers

Each node uses only its own link. At most 1.545 Gbps, which fits inside 2.5G.
But **all three nodes' traffic converges in front of the scheduler.** The load
at the confluence is threefold.

```text
king  --1.5G--\
queen --1.5G---+--> 4.6 Gbps --> scheduler   <- impossible on 2.5G
jack  --1.5G--/
```

**Only this point degrades linearly as nodes are added.** In a project measuring
three-node scaling efficiency, if something fills up first as you scale, **you
end up measuring link saturation rather than NPU scaling efficiency.**

### Why a separate server — two reasons overlap

**(1) Symmetry of the measurement conditions.** Running the scheduler on one of
the nodes raises CPU and network load on that node alone. The three nodes'
conditions diverge and the 1/2/3-node comparison is distorted. You could no
longer call it a "like-for-like comparison" in a talk.

**(2) The PCIe slot.** A 10G SFP+ NIC is a PCIe card. The current scheduler
host, `dealer`, is **a laptop with nowhere to put it.**

(1) alone required a separate host, and (2) narrowed "any host" to "a server
with a PCIe slot".

## Build result (2026-08-20)

The equipment was assembled as designed and the bandwidth measured. It came
together as **10GBASE-T (RJ45) rather than DAC/SFP+** — the switch is a NEXI
NS-S25G10G-N (2.5G×4 + 10G×2, all RJ45), so the SFP+ plan became RJ45. No
effect on the conclusion.

```text
server (Rocky 9.4, Xeon x2 24T / 16GB)
  \ enp4s0 10GBASE-T -- measured 10G full (ethtool)
                        |
              NS-S25G10G-N -+ 2.5G - king  .3
                            + 2.5G - queen .5
                            \ 2.5G - jack  .4
```

| Measurement | Value | Tool |
|---|---:|---|
| Server link negotiation | 10000 Mb/s full | ethtool |
| Single king→server | 2.34 Gbps | iperf3 (the effective 2.5G ceiling) |
| **3 nodes concurrently →server** | 1.70 each, **5.11 Gbps total** | nc |

With three nodes concurrent, the three streams **stayed even** — had the server
been the bottleneck the total would have been cut, and it was not. It
comfortably accommodates the INT8 3-node RX target of **4.60 Gbps**. (The
individual 1.70 being below the 2.34 link ceiling is an nc/board-CPU limit, not
a switch or server limit. Actual M3 traffic is gRPC, so this figure is for
infrastructure verification.)

As a side effect the **scheduler host's RAM went from 3 GB (dealer) to 16 GB
(server)**, easing ADR-003's concern about scheduler RSS.

> ⚠️ Because the boards use DHCP, this rework changed their IPs wholesale
> (`.12/.16/.33` → `.3/.4/.5`). The [ADR-019](#adr-019) situation
> recurred, with stale SSH aliases failing to find the nodes. MAC-based static
> IPs are follow-up work.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| 10G everywhere (workers included) | Wasteful at 1.545 Gbps per node. The boards' NICs are 2.5G anyway |
| 25G or above | Needed to keep `want_float=1`, but reducing the output is cheaper and more correct (ADR-012) |
| Run the scheduler on `king` | Breaks the three nodes' experimental conditions. Allowed for development and demos only, never for official figures |
| Keep 2.5GbE and just measure | **The most dangerous choice.** Link saturation would get reported as scaling efficiency |
| Switch input to JPEG to reduce TX | The decode cost lands on node CPU. CPU is already the bottleneck and this stacks on it. Valid as an S6 comparison item |

## Consequences

**Gained**

- The premise for measuring scaling efficiency holds — the link does not fill
  up first
- The required equipment became clear: a 2.5G/10G switch, a PCIe server, a 10G
  NIC and an SFP+ DAC

**Lost / the cost**

- **M3 was blocked for a while on procurement.** From the decision on
  2026-08-12 to the build on 08-20. A hardware problem, not a code one
- Cost went up — switch, server, NIC, cables

**The biggest consequence of this decision: choosing not to start measuring**

Measuring without the equipment would still produce numbers. **And those numbers
would be wrong.** Scaling efficiency would come out low at three nodes, with the
cause being the link rather than the NPU. Publishing results in that state would
invalidate the project's central claim.

So **the choice was to stop measuring and wait.** That is a decision too.

**New constraints introduced**

- The scheduler host formally joined the experimental equipment list. Changing
  its specification is a change of measurement conditions
- Before starting M3, **measured TX/RX must be recorded rather than calculated**
  (`02-HARDWARE-SETUP.md` §3.3.3). This section's original error was trusting
  calculation alone

## What would overturn this

- **Switching the input format to JPEG** cuts TX roughly tenfold and the whole
  link budget has to be recomputed. 2.5G might suffice in that case — but where
  the decode CPU cost lands has to be considered alongside
- **Implementing node-side postprocessing (NMS)** effectively removes RX. What
  remains is TX at 4.64 Gbps, lowering the requirement → ADR-021
- **Adding more nodes** raises the aggregation requirement proportionally. Five
  nodes at INT8 comes to 7.7 Gbps, leaving no headroom even at 10G

---

<a id="adr-015"></a>

# ADR-015. Run a hard-failing preflight check before measuring, and measure nothing until it passes

*[한국어 원문](015-preflight-hard-fail.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-007](#adr-007), [ADR-016](#adr-016), [ADR-019](#adr-019), [ADR-028](#adr-028) |

---

## In one line

> What has ruined measurements so far has mostly been **the premises, not the
> measurement itself**. So a machine checks the premises immediately before
> measuring. **On a hard failure, measurement does not start.** And accuracy is
> checked **before** performance.

## Context

Measurements have been wrong several times, and the cause was always outside
the measurement code.

| What happened | Result |
|---|---|
| A stale IP in the docs pointed somewhere else | Misdiagnosed as a dead node; scanned the whole subnet |
| Compared two measurements with different load profiles | A 19 °C gap was misread |
| A board reset by insufficient adapter current | Its throughput was nearly read as performance |
| Sharing a context | 0 errors and 100% result mismatch |

**What they share: it was already wrong before measurement began.** And all four
give no signal while running.

## Decision

**1. Create `scripts/preflight-check.sh`, and do not measure until it passes.**

The verdict is the exit code.

```text
0  pass (warnings are possible)
1  hard failure. Measuring in this state makes the result invalid
2  script usage error
```

**2. Divide the checks into six groups.**

| Group | What it looks at |
|---|---|
| 1. Connection and identity | alias ↔ hostname agreement |
| 2. Software identity | kernel, RKNN, driver and model hashes identical across the three nodes |
| 3. Measurement conditions | governor, idle temperature, input voltage, residual load, NTP, session count |
| 4. **Inference accuracy** | do the three boards give the same answer to the same input |
| 5. Network measurement | record M3's premise values |
| 6. Cluster registration | are the three nodes attached to the scheduler |

**3. This script does not fix anything. It only judges.** Fixing is
`fix-node-consistency.sh`'s job.

**4. Treat empty values and placeholders as failures.**

**5. When adding a check, break it deliberately and confirm it actually
catches.**

## Rationale

### Why accuracy comes before performance

That is what `--with-inference` does. It gives the three boards the same input
and checks that the same answer comes out.

The reason is [ADR-007](#adr-007). The shared-context
configuration **produced wrong answers faster** (at two threads, shared 34.8 >
dedicated 33.2 inf/s).

**A configuration that produces wrong answers fast must not win a benchmark.**
Measure performance alone and such a configuration gets reported as optimal.

### The incident where "could not read" was judged as "identical"

`/sys/kernel/debug/rknpu/version` is readable only by root. Reading it without
permission returned an empty string on all three nodes, and it **passed on the
grounds that the values matched**.

```text
king  ""      \
queen ""      +- the three values match -> pass OK   <- nothing was verified
jack  ""      /
```

A variant of the mistake of not checking what a metric counts. So empty values
and placeholders such as `unknown` are treated as **failures**.

### Why the alias ↔ hostname check is number 1

**This is far more dangerous than a connection failure.** A failed connection is
known immediately. But if `npuforge-k` points at `queen`, the measurement
finishes normally and **the result is attributed to the wrong node.** It fails
quietly.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Keep the checklist in a document and have a human verify | "Let's be careful" in a document did not work. Several failures happened while knowing better |
| Put the checks inside the bench tool | Some are ([ADR-028](#adr-028)). But SSH, sudo and hash comparison are outside the tool's domain, so they were separated |
| Make check failures warnings only | Warnings get ignored, especially when you want to start measuring quickly |
| Fix things automatically | Mixing judgement with remedy leaves no record of "what had been wrong" |

## Consequences

**Gained**

- Premise failures are caught **before** measurement
- Pass/fail is an exit code, so it drops straight into automation
- The measurement conditions get recorded (`--json`)

**Lost / the cost**

- It takes time to start measuring, especially `--with-inference`, which runs
  real inference
- A sudo password is needed. It is not committed to the repository but taken
  from an environment variable or a `~/.npuforge/` file — **this project is
  going public, and anything in the commit history would need a history rewrite
  to remove**

**New constraint introduced**

- **The check itself can be wrong.** `pgrep -f` once counted itself and passed
  quietly ([ADR-017](#adr-017)). That is why "break
  it deliberately" became a rule

## What would overturn this

The list of checks keeps growing. **It never shrinks.** Adding an entry every
time a new failure mode is encountered is this script's design intent.

---

<a id="adr-016"></a>

# ADR-016. Detect mid-measurement reboots with `boot_id` and invalidate the run

*[한국어 원문](016-boot-id-run-invalidation.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-015](#adr-015), [ADR-028](#adr-028), [ADR-027](#adr-027) |

---

## In one line

> If a board resets mid-measurement, that run's figures are void. But **from the
> outside it looks like "a node whose performance dropped".** Linux's `boot_id`
> is carried in the heartbeat, and a change in it invalidates the run.

## Context

This project actually experienced boards rebooting. The cause was misdiagnosed
three times.

```text
suspected the shared PSU        ->  it was not
bootloader firmware problem     ->  partly right
12V input problem               ->  it was not
actual cause: insufficient power adapter current
```

The problem was not identifying the cause but **what to do with the
measurements taken in the meantime.**

A board resetting under load looks like this.

```text
throughput drops sharply         -> "thermal throttling?"
no response for a while          -> "network latency?"
then it returns to normal        -> "it recovered"
```

**Every one of those gets a plausible interpretation.** Not knowing it rebooted,
this data gets read as "performance degradation at high temperature" and drawn
on a graph.

## Decision

**1. The node reports `boot_id` in the heartbeat.**

Linux generates a new UUID at every boot.

```text
/proc/sys/kernel/random/boot_id
```

The value always changes on reboot, and never changes otherwise.

**2. The scheduler warns when it detects a change.** A node returning under the
same `node_id` with a different `boot_id` is not "a node that dropped briefly"
but **a different instance**.

**3. The bench tool uses it in run-validity judgement.** The `boot_id` at the
start of a run is recorded, and if it differs at the end, the run is marked
invalid.

**4. Preflight records the reference values.** The three nodes' `boot_id`s are
captured immediately before measuring.

**5. Invalid runs are not deleted.** They are kept with the reason. Repeated
reboots are themselves a finding — that is in fact how the adapter problem was
found.

## Rationale

### Why no other signal works

| Candidate | Why it fails |
|---|---|
| uptime becoming small | Missed if it resets and comes back between polls |
| Connection dropping | Indistinguishable from a network blip |
| A sharp throughput drop | Indistinguishable from throttling. **This is exactly the problem we hit** |
| A change in process PID | Changes when only the node process restarts. That is a different event from a board reset |

`boot_id` is **the fact that the kernel counted a boot**, and nothing else.
There is no room for interpretation.

### Intentional failures and hard resets have to be distinguished

Scenario S4 is an experiment that **deliberately kills nodes** and observes
recovery. "The node disappeared" is normal behaviour there.

But a board dying from a power problem looks identical. Without distinguishing
the two, S4's results get reported mixed with equipment defects.

If `boot_id` changed it is a hard reset; if not, it is a process-level failure.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Just stop the reboots happening | That was done (the adapter was replaced). **The detection still has to exist** — the next cause may be something else |
| Have a human read the logs and judge | Impossible in unattended overnight runs (146 runs, 23.4 hours) |
| Parse dmesg | Heavy and needs permissions. There is a value that can be read in one line |
| Delete invalid runs automatically | Cause tracing becomes impossible. The pattern of repetition is itself information |

## Consequences

**Gained**

- Catches reboots disguised as "performance degradation"
- Data validity is judged automatically even in unattended overnight runs
- Intentional failures are distinguished from equipment defects

**Lost / the cost**

- One more field in the heartbeat message (an effectively negligible cost)
- `boot_id` catches only reboots. **It cannot catch problems that arise with the
  kernel still alive** — that is other checks' job

**New constraint introduced**

- A node-process-only restart and a board reset have to be treated differently.
  Both trigger re-registration
  ([ADR-025](#adr-025)), so a re-registration event
  alone does not distinguish them

## What would overturn this

The check becomes unnecessary when "boards never reset" is proven, and there is
no way to prove it. **It stays.**

---

<a id="adr-017"></a>

# ADR-017. Harden the remote-execution pitfalls into library functions

*[한국어 원문](017-remote-exec-pitfalls-library.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-015](#adr-015), [ADR-019](#adr-019) |

---

## In one line

> There are three pitfalls in running remote commands over `ssh` where
> **failure looks like success**. All three give exit code 0 with empty stderr.
> Rather than being careful every time, they are hardened into functions in
> `scripts/lib/remote.sh`.

## Context

Found while building `preflight-check.sh`. **A check was silently not working.**
It passed with "no residual load" while load was running.

Digging in, there were three pitfalls, and all of them share one property —
**there is no signal at all that something is wrong.**

## Pitfall 1. `pgrep -f` counts itself

`pgrep -f` matches the whole command line. And the command line of the wrapper
ssh sends **contains the pattern string itself.**

```text
bash -c "... pgrep -f \"[s]ustained_load_test|...\" | wc -l"
                       ^^^^^^^^^^^^^^^^^^^^^^^^ this matches
```

The bracket trick (`[s]ustained`) is also neutralised once a form without the
brackets appears on the same command line.

**It is wrong in both directions.**

| Situation | Actual | pgrep reports |
|---|---|---|
| Load running | 1 | **0 (missed)** |
| No load | 0 | **2 (counting its own shell)** |

**The fix**: read the `/proc/PID/exe` symlink. It points at the actual
executable, leaving no room for a shell to get involved.

```bash
n=0
for p in /proc/[0-9]*; do
  case "$(readlink "$p/exe" 2>/dev/null)" in
    *sustained_load_test) n=$((n+1)) ;;
  esac
done
```

## Pitfall 2. `cd DIR && setsid nohup ... &` does not come up

| Form | Result |
|---|---|
| `ssh -n H "cd $DIR && setsid nohup ./prog ... &"` | **does not run** |
| `ssh -n H "setsid nohup $DIR/prog ... &"` | runs |

The `&` applies to the **whole `cd && prog` list**. ssh sends the command and
disconnects immediately, and if the session disappears before the background
subshell gets through `cd` and reaches `setsid`, it dies right there.

Using an absolute path removes the intermediate step, so no race arises.

**The cost is large.** Even on failure the exit code is 0 and stderr is empty.
Without checking, you end up **measuring "the temperature with no load" for
fifteen minutes.**

## Pitfall 3. A heredoc inside ssh nested with sudo does not create the file

Encountered while deploying a systemd unit. This too gave **exit code 0.**

## Decision

**1. Make the avoidance form of all three pitfalls into functions in
`scripts/lib/remote.sh`.** Scripts use those functions rather than calling ssh
directly.

**2. Read `/proc/PID/exe` when counting remote processes.** Do not use
`pgrep -f`.

**3. Background startup uses only the absolute path + `setsid nohup` form.**

**4. Add a step that confirms it is actually running after starting it.** Do not
trust the startup command's exit code.

**5. When adding a new check, break it deliberately and confirm it actually
catches.**

## Rationale

### Point 5 is the heart of this ADR

Pitfall 1 was found precisely because of that procedure. **Had a pass been
trusted at face value, preflight would have remained in place filtering
nothing.**

Check code is especially dangerous. It normally prints only "pass", so nobody
notices when it breaks. It just **gets quieter.**

### Why code rather than documentation

All three pitfalls are the kind you can avoid if you know about them. And yet
this project already has several cases of being caught while knowing better. If
three things have to be recalled every time a remote command is written, one
will eventually be missed.

Making them functions makes **the default path the safe form.**

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Leave it in comments and documentation | Already confirmed not to work |
| Introduce a tool like Ansible | Adds a dependency, and is excessive for a three-machine experimental setup. Problems like pitfall 2 remain regardless |
| Keep a resident agent instead of ssh | That is what `npuforge-node` is. But the measurement scripts have to run independently of the node process |
| Just check the exit code | **All three pitfalls give exit code 0.** Fundamentally does not work |

## Consequences

**Gained**

- New scripts use the safe form by default
- The record of having hit these pitfalls lives next to the code

**Lost / the cost**

- Scripts depend on `lib/remote.sh`. Running them standalone gets harder
- Walking `/proc` is slower than `pgrep` (negligible given how often the checks
  run)

**New constraint introduced**

- **New remote execution has to go through this library.** Calling `ssh`
  directly reopens the pitfalls

## What would overturn this

If a fourth pitfall appears, it gets added here. **There is no reason for the
list to shrink.**

---

<a id="adr-018"></a>

# ADR-018. Convert the model once and deploy the same file to all three nodes

*[한국어 원문](018-convert-model-once-deploy.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-011](#adr-011), [ADR-015](#adr-015) |

---

## In one line

> **INT8 conversion is not byte-reproducible.** Converting three times from the
> same input gave a different hash each time — even though the inference results
> are completely identical. So the model is not converted per node; **one file,
> converted once**, is deployed to all three.

## Context

With three nodes there are two ways to prepare the model.

```text
approach A. convert per node        ONNX -> .rknn on each board
approach B. convert once and deploy copy a .rknn built in one place
```

A looks natural. If the conversion script is deterministic, the same file should
appear on all three nodes.

**But it was not deterministic.**

## Rationale

### Measured: same input, different bytes

**Converted three times** from the same ONNX with the same calibration list.

```text
file size    identical
hash         different all three times
byte diff    1.8%
```

And the inference results were:

```text
all 9 output tensors at cosine 1.000000, error 0.0
```

**The difference is in serialization and layout, not in numerical computation.**
But with different files, "the three nodes use the same model" can no longer be
proved by hash.

### Why that matters

This project's premise is **that the three nodes' conditions are identical.**
Measuring 1/2/3-node scaling efficiency requires the nodes to be symmetric.

Preflight checks that the three nodes' model hashes match. Converting per node
makes that check **fail always**. Removing the check instead loses the means to
confirm "is it really the same model".

## Decision

**1. The model is converted once, in one place.** The conversion environment is
pinned with Docker (rknn-toolkit2 2.3.0).

**2. The resulting `.rknn` file is copied to all three nodes.**

**3. Deployment integrity is verified via `sha256` in `model.toml`.** The node
checks the hash when loading the model.

**4. State explicitly what that hash guarantees.**

```text
what sha256 guarantees      deployment integrity - the three nodes hold the same file
what sha256 does not        identity of the conversion recipe - that it was made the same way
```

**5. Make calibration image selection deterministic.** 200 images are chosen
from COCO val2017 with a fixed seed (`fetch_calibration.py`). The images
themselves are not put in the repository for licensing reasons; **only the
manifest** is kept.

**6. Apply the same principle to the node binary.** Only `king` has a Rust
toolchain; it builds once there and deploys to `queen` and `jack`. There is no
per-node build.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Convert per node | The hashes differ and the "same model" check becomes impossible |
| Make the conversion deterministic | That is internal to rknn-toolkit2 and outside our control |
| Verify identity by inference results instead of hash | That is also done (`preflight --with-inference`). But it is heavy as a deploy-time check |
| Drop the hash check | A corrupted file or a mixed-in different version would go unnoticed. That is exactly the accident it was meant to prevent |

## Consequences

**Gained**

- The three nodes hold **a byte-identical model**
- Preflight's model hash match check becomes meaningful
- Conversion environment problems do not multiply by three

**Lost / the cost**

- One more deployment step
- The conversion environment (Docker, rknn-toolkit2 version) becomes part of
  reproducibility. It is pinned in `environment-matrix.md`

**New constraints introduced**

- **Do not mistake `sha256` for verification of the conversion recipe.** The
  same hash means the same file, not that it was made by the same procedure.
  Reproducing it requires recording the conversion command, dataset and toolkit
  version separately
- Re-converting the model means **redeploying to every node.** Updating one node
  alone gets blocked by preflight (intended behaviour)

## What would overturn this

- **If rknn-toolkit2 comes to guarantee deterministic conversion**, per-node
  conversion becomes possible. Though converting once and deploying is still
  simpler
- **If an experiment requires the model to differ per node** (different
  precision per node, say), the premise changes. In that case "node symmetry"
  itself becomes an experimental variable

---

<a id="adr-019"></a>

# ADR-019. Reach the boards by SSH alias, not by IP

*[한국어 원문](019-ssh-alias-not-ip.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-015](#adr-015), [ADR-017](#adr-017) |

---

## In one line

> An IP pinned into a document went stale, so **a node was misdiagnosed as dead**
> and the whole subnet got scanned. `~/.ssh/config` had the correct value all
> along. Access goes only through the `npuforge-k` / `-q` / `-j` aliases.

## Context

On 2026-08-11 `king` could not be reached.

```text
IP written in the document   10.20.0.22
actual IP                    10.20.0.12
```

Believing the node was dead, the subnet was swept. But `~/.ssh/config` had had
**the correct IP from the beginning.** Only the document was stale.

Why this is dangerous. Being unable to connect at all is the better case — you
find out immediately. **The genuinely dangerous case is when another board is at
that IP.**

```text
measure via npuforge-k -> actually attaches to queen -> the measurement finishes normally
                                                     -> the result is recorded as king's
```

It fails quietly. The failure mode this project guards against most.

## Decision

**1. Boards are reached only by SSH alias.**

```text
npuforge-k   king
npuforge-q   queen
npuforge-j   jack
```

**2. Do not write IPs directly in documents or scripts.** The IP lives in one
place, `~/.ssh/config`.

**3. Preflight's **first** check is alias ↔ hostname agreement.** It confirms
that what you attached to really is that board.

**4. Keep the SSH host keys distinct per node.**

## Rationale

### A single source

IPs change. A DHCP lease renews, the network gets reconfigured, a switch gets
replaced. If several documents have to be fixed each time, one will inevitably
be left behind.

`~/.ssh/config` is **the value actually used to connect**, so being wrong shows
up immediately. An IP in a document is used by nobody and stays wrong for a long
time.

### An alias can be wrong too — hence the check

The alias points at an IP, so a reassigned IP can leave the alias pointing at
the wrong board. That is why preflight's check 1 is needed.

```text
ssh npuforge-k hostname   ->  must be "king"
```

That check comes **before the connection-failure check**, because a connection
failure fails loudly while a wrong mapping succeeds quietly.

### Identical host keys make them indistinguishable

`queen` and `jack` currently have identical SSH host keys — apparently from
cloning or copying an image.

In that state, **SSH raises no warning even if a changed IP attaches you to a
different board.** A host key is the device for confirming "is this server the
same server as before", and when two are identical that function is dead.

Having already misdiagnosed a node once, this must not be left alone. **It
remains as an open item in `docs/TODO.md`.**

```bash
ssh npuforge-j 'sudo rm -f /etc/ssh/ssh_host_* && sudo ssh-keygen -A && sudo systemctl restart ssh'
ssh-keygen -R npuforge-j   # clean up known_hosts on the PC
```

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Just manage the IPs in the documents well | Already failed. A value nobody uses goes stale |
| Assign static IPs | Copied into documents it is the same problem again. The alias remains valid on top of static IPs anyway |
| Reach by mDNS / hostname | Does not work in some environments, and the alias is a layer above it so they can coexist |
| Use aliases but skip the check | Does not catch an alias pointing at the wrong board |

## Consequences

**Gained**

- A single source for the IPs
- Preflight catches the accident of a measurement being attributed to the wrong
  node

**Lost / the cost**

- Someone new taking the repository has to create `~/.ssh/config` themselves.
  Stated as a prerequisite in the reproduction procedure

**New constraints introduced**

- **Treat any IP seen in a document with suspicion.** If one is still there, it
  is likely stale
- Until `queen` and `jack`'s host keys are regenerated, an IP reassignment can
  attach you to the wrong board without a warning. **A known risk**

## What would overturn this

Nothing. No reason will arise to pin IPs back into documents.

---

<a id="adr-020"></a>

# ADR-020. Use `worker_count = 8` and do not set `core_mask`

*[한국어 원문](020-worker-count-8-no-core-mask.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-10 |
| **Related** | [ADR-007](#adr-007), [ADR-011](#adr-011), `docs/discuss.md` §4 |

---

## In one line

> Eight workers beat four by **+27%**. Assigning NPU cores by hand with
> `core_mask` is worth **+0.1%** at 8 threads — effectively nothing.
> `CORE_AUTO`'s distribution is already even. **Not touching it is the
> conclusion.**

## Context

The RK3576's NPU has two cores. RKNN provides `core_mask` to specify which core
to use.

Early in measurement there was an observation that "Core1 occupancy is only
38%", and a hypothesis followed that the second core was idling. Assigning cores
explicitly was expected to raise throughput.

**But whether that 38% actually contributed to throughput had never been
checked.** Only the occupancy number had been looked at.

## Rationale

### A control group was added

Earlier measurements lacked "how much do you get using only one core". That has
to be there to judge the second core's contribution.

```text
conditions: queen, FP16, 200 iterations per thread, sampled after a 4 s warmup
```

| Threads | AUTO | ALTERNATE | CORE_0_1 | **CORE_0_ONLY** |
|---:|---:|---:|---:|---:|
| 1 | 16.7 | 16.7 | **18.2** | 16.5 |
| 2 | 36.2 | 36.5 | 36.4 | 26.4 |
| 4 | 52.4 | **57.1** | 48.5 | 38.5 |
| 8 | 72.9 | **73.0** | 64.5 | **48.2** |

### Finding 1. The second core does contribute — but by 1.51×

```text
8 threads   single core 48.2  ->  two cores 73.0 inf/s   =  1.51x
```

The 38% occupancy was not decoration. **But it is 1.51×, not 2×.** Doubling the
cores raises throughput by only half as much. That means there is a shared
resource outside the cores, which matches the "submission path serialization"
confirmed later.

### Finding 2. Explicit assignment brings no gain

```text
4 threads   52.4 -> 57.1   +9.0%
8 threads   72.9 -> 73.0   +0.1%
```

It rises only at 4 threads and vanishes at 8. And unpacking that 4-thread
improvement, most of it is a reduction in `outputs_get` (13.6 → 10.0 ms), so
**whether it is a core-assignment effect or measurement noise is not
separated.**

`AUTO`'s distribution is already even — Core0 39% / Core1 37% at 8 threads. The
runtime scheduler is doing its job and there is no room for manual intervention.

### Finding 3. `CORE_0_1` is actually a loss

```text
8 threads   72.9 -> 64.5   -11.5%
```

Making every thread use both cores together is slower.

## Decision

**1. `worker_count = 8` is the real-hardware default.** It is +27% over 4, and
it has not yet bent at 8.

**2. Do not set `core_mask`.** Leave it to `CORE_AUTO`.

**3. The configuration default stays 1, with real-hardware configuration giving
8 explicitly.** A default of 1 is the safe value when the backend is unknown.

**4. State explicitly that `worker_count` is directly tied to context count.**
The backend creates that many RKNN contexts
([ADR-007](#adr-007)).

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| `core_mask = ALTERNATE` | +0.1% at 8 threads. Adds a configuration item and no gain |
| `core_mask = CORE_0_1` | −11.5%. Clearly a loss |
| `worker_count = 4` | −27% against 8 |
| Raise `worker_count` further | **The point where it bends has not been found yet.** But contexts grow with it, so a memory check comes first |

## Consequences

**Gained**

- One tuning item removed. **Deciding not to configure something is also a
  decision**
- Obtained the figure that the NPU's two cores contribute 1.51× — grounds for
  later bottleneck analysis

**Lost / the cost**

- +9% is given up at the 4-thread condition. But real hardware runs 8 workers

**New constraints introduced**

- **Raising `worker_count` raises the RKNN context count with it.** It must not
  be increased without checking memory headroom. The per-context memory increase
  has **not been measured**
- There is no basis for 8 being the ceiling. "It has not bent yet at 8" is the
  accurate statement. Widening `MAX_THREADS` and re-measuring remains an open
  item

## What would overturn this

- **Re-measuring against INT8 could give a different optimum.** The sweep above
  is FP16. INT8 takes less time per inference, so the optimal concurrency may
  differ. **Not yet checked**
- **Widening `MAX_THREADS` and measuring 12 and 16** could give a better value
- If memory runs short, 8 has to come down

---

<a id="adr-021"></a>

# ADR-021. The node does no postprocessing (NMS) and returns raw tensors

*[한국어 원문](021-no-node-side-postprocessing.ko.md)*

| | |
|---|---|
| **Status** | provisional |
| **Date** | 2026-08-12 |
| **Related** | [ADR-012](#adr-012), [ADR-014](#adr-014), [ADR-013](#adr-013) |

---

## In one line

> The node returns **the model's 9 output tensors as they are**, not detections.
> The response gets larger, and in exchange **the node's CPU load stays out of
> what is being measured.** Node-side postprocessing is ultimately right, but not
> now.

## Context

The output of a detection model like YOLOv8n is not directly usable.

```text
NPU output   9 tensors (box candidates and class scores per grid cell)
                 |  postprocessing (NMS: resolve overlapping boxes, apply thresholds)
final result "1 person, 2 cars" - a few KB
```

The question is **where that postprocessing happens.**

| | Response size | Node CPU |
|---|---|---|
| Postprocess on the node | a few KB | goes up |
| Postprocess on scheduler/client | 1.2 MB | unchanged |

## Decision

**The node does no postprocessing.** It returns the raw tensors bundled into a
single blob ([ADR-012](#adr-012)).

**The status is left as "provisional".** Not because it is best, but because it
is **the right choice under current conditions.**

## Rationale

### 1. Node CPU is already the bottleneck

Throughput falls 27% over 300 seconds of sustained load, and the cause is not
the NPU but **CPU thermal throttling**. The A72 is downgraded from 2208 to
816 MHz ([ADR-013](#adr-013)).

Adding NMS on top increases CPU load further. That would destabilise the very
value this project is trying to measure.

```text
now:              measuring NPU scaling efficiency while the CPU interferes  <- already a problem
with postprocess: making it use more CPU and measuring the same value        <- worse
```

### 2. It adds another measurement variable

**NMS cost varies with the input.** An image with many detections takes longer
and one with few finishes quickly. Doing it on the node makes per-node
processing time vary with input content.

In an experiment measuring three-node scaling efficiency, that variable is noise.

### 3. It is not implemented

The simplest reason. There is no NMS implementation, and building one brings
verification (accuracy comparison) with it. Not a priority while waiting on
equipment.

### 4. The network problem was solved another way

The cost of returning raw tensors is response size. With `want_float=1` the
response was 3.96× the request and even 10G was insufficient.

That was **solved with `want_float=0` rather than postprocessing.** The response
became a quarter of its size and 3-node RX went from 18.38 to 4.60 Gbps. It fits
inside 10G.

So **there is no immediate pressure to postprocess.**

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Run NMS on the node | **This is ultimately the right answer.** But it worsens the CPU bottleneck and adds a measurement variable. Unimplemented |
| NMS on the scheduler | The scheduler is a single point, so three nodes' worth of postprocessing piles up in one place. The scheduler becomes the bottleneck |
| Compress the response | Compression CPU enters the inference path. CPU is already the bottleneck |
| Client-side postprocessing | This is the current approach. The bench tool and comparison scripts understand the blob |

## Consequences

**Gained**

- What the node does is narrow and uniform: **preprocess → NPU → serialize**
- Per-node processing time depends less on input content
- What is being measured stays clean

**Lost / the cost**

- The response is 1.2 MB, where sending only the detections would end at a few KB
- **The receiver has to understand the blob.** Changing the format means fixing
  three places together (blob.rs / dump_output_test.c / compare_detections.py)
- As a real-world API it is unfriendly. It is not an API that "gives you
  detections"

**New constraints introduced**

- The client is responsible for both dequantization and NMS
- The network budget is tied to response size. In experiments that increase
  input size (S6), the response grows with it

## What would overturn this

**This ADR is scheduled to be overturned.**

- **If the CPU bottleneck is resolved** (cooling condition B, or preprocessing
  optimization), there is room to move postprocessing to the node
- **If a real-world API becomes a requirement**, returning raw tensors is hard to
  sustain
- **If the network fills up again in experiments that increase input size**,
  postprocessing becomes the most effective means — the response shrinks to a
  few KB and RX effectively disappears

**What must be measured alongside** when overturning it: sustained throughput
and the timing of CPU clock downgrade, before and after putting postprocessing
on the node. Do not judge from response size alone.

---

<a id="adr-022"></a>

# ADR-022. Assign each document a normative domain, and follow the normative one when values disagree

*[한국어 원문](022-document-authority-order.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-05 |
| **Related** | [ADR-002](#adr-002), `docs/00-PRD.md` §0 |

---

## In one line

> When the same value is written in several documents, one of them inevitably
> goes stale. So **each domain gets one normative document**, and when values
> disagree, that document wins. Other documents reference rather than duplicate.

## Context

This repository has many documents: PRD, TECHSPEC, hardware, development
requirements, environment-matrix, RESULTS, TODO, discuss, board-worklog.

The same number appears in several places. "157.2 inf/s per node", for example,
appears in RESULTS, in TODO, in board-worklog and in environment-matrix.

**Fix one and the rest go stale.** This actually happened.

```text
after the switch to want_float=0
  RESULTS §2.2  updated       "INT8 +17.3%"
  RESULTS §5    not updated   "INT8 throughput impact unmeasured"   <- contradictory within one document
  TECHSPEC §3.2 not updated   the discarded network calculation left as-is
```

## Decision

**1. Each domain gets one normative document.**

| Domain | Normative document |
|---|---|
| Goals, non-goals, functional requirements, success criteria | `00-PRD.md` |
| Repository structure, protocol, config schema, scheduling algorithm, error codes | `01-TECHSPEC.md` |
| Physical setup, network, power, cooling, experimental conditions | `02-HARDWARE-SETUP.md` |
| Development environment, tooling, deployment automation, licensing | `03-DEVELOPMENT-REQUIREMENTS.md` |
| Version combinations and hash pinning | `environment-matrix.md` |

**2. When values disagree, the normative document wins.**

**3. Reference rather than duplicate.** The PRD covers only "why" and "what".
Formulas, crate names, configuration keys and identifier strings are not written
in the PRD; it points at TECHSPEC.

**4. Documents of a different nature are not normative.**

| Document | Nature |
|---|---|
| `discuss.md` | Chronological discussion. Later sections correct earlier ones |
| `board-worklog.md` | Work history. Wrong hypotheses are preserved |
| `RESULTS.md` | A collection of results. The final authority for values is environment-matrix |
| `TODO.md` | What is to be done now |
| `adrs/` | Decisions and their rationale |

## Rationale

### Not duplicating is the only method

There are only two ways to maintain consistency.

```text
1. duplicate, and find and fix every copy on each change   -> one will always be missed
2. keep it in one place from the start                     -> there is nowhere to go stale
```

This project already failed with option 1. The single switch to `want_float=0`
involved five documents — `RESULTS.md`, `TECHSPEC`, `environment-matrix`, `TODO`
and `board-worklog` — and one sync pass did not catch them all.

### Why chronological documents are excluded from normativity

`discuss.md` **deliberately keeps wrong conclusions.** Section 5's "+5.4%" is
stale by current standards, but section 5 has to stay as-is for anyone to
understand why section 12 corrected it.

Making such a document normative means a reader who got as far as the earlier
section quotes a discarded value. So **chronological documents are supporting
material, not authority.**

## ADRs complement this structure

Normative documents answer "what is the value now". Chronological documents
answer "what happened". **The place answering "why was it decided that way" was
empty.**

`adrs/` is that place. It takes values from the normative documents and the
story from the chronological ones, and re-bundles them by decision.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Merge into a single document | It runs past ten thousand lines. Readers with different purposes cannot be served by one document |
| Manage without priority | No way to decide which is right when values conflict |
| Generate values automatically | Possible for some (test counts and the like), but measured values have to be written by a person along with their conditions |

## Consequences

**Gained**

- There is a rule for deciding when values conflict
- Each document's role is clear
- Where to make a change can be pinpointed

**Lost / the cost**

- Learning about one topic means moving between documents. **That inconvenience
  is the direct reason `adrs/` exists**
- You have to remember which document is normative

**New constraints introduced**

- **When duplication is found, delete it and replace with a reference.** The
  urge to copy a value over for convenience keeps arriving
- ADRs quote values too. A quoted value can go stale, so **the measurement
  conditions and the source are written alongside**

## What would overturn this

As documents grow, normative domains get added. As they shrink, they get merged.
The principle itself does not change.

---

<a id="adr-023"></a>

# ADR-023. Fix the CPU governor to `performance` — but state the scope of the evidence

*[한국어 원문](023-cpu-governor-performance-scoped.ko.md)*

| | |
|---|---|
| **Status** | provisional |
| **Date** | 2026-08-12 |
| **Related** | [ADR-013](#adr-013), [ADR-002](#adr-002), `docs/discuss.md` §11, §12 |

---

## In one line

> Switching `ondemand` → `performance` gives **+7%**, so it is fixed there.
> **But that +7% is a 120-second measurement.** Under sustained load
> `performance` heats up faster and may be worse, and **that has not been
> checked yet.**

## Context

Linux's CPU governor is the policy that adjusts clock speed with load.

| governor | Behaviour |
|---|---|
| `ondemand` | raises the clock only under load. The default |
| `performance` | always holds the maximum clock |

One inference is `set input (CPU) → NPU → get output (CPU)`, so CPU clock feeds
directly into throughput. That makes the governor a variable.

## Decision

**1. Fix all three nodes' governor to `performance`.** Made permanent with a
systemd unit so it survives reboots (`scripts/set-cpu-governor.sh`).

**2. Preflight verifies it before every measurement.**

**3. State the basis of the existing figures.** Every measurement before
2026-08-11 is on `ondemand`.

```text
ondemand      FP16 79.0 / INT8 146.2 inf/s
performance   FP16 84.3 / INT8 157.2 inf/s
```

**4. Pin down the scope of the "+7%" conclusion in the documents.** Read only as
"a gain in short measurements".

## Rationale

### Why fix it

More than the value itself, **unifying the condition** is what matters. A
governor that differs per node or per run makes three-node comparison
meaningless.

`performance` was chosen for two reasons.

- +7% in a 120-second measurement
- **Its behaviour is simple.** With `ondemand` the clock rises and falls with
  the load pattern, making it hard to separate whether variance in the
  measurements comes from the governor's decisions or from somewhere else

The second matters more. For reproducibility, the predictable option is better.

## ⚠️ Where this decision's evidence is weak

**The +7% is a 120-second measurement.** That window is before the CPU has been
fully downgraded.

What actually happens under sustained load:

```text
        NPU temp   cpu4(A72)   cpu0(A53)
 +15s   86.8 C     2208 MHz    2016 MHz
 +30s   90.4 C     1416 MHz    1200 MHz
 +60s   87.8 C      816 MHz     600 MHz   <- 63-70% downgrade
+120s   87.8 C      816 MHz     600 MHz
```

**`performance` holds the maximum clock even at idle.** So it has less thermal
headroom at the moment load starts. It may heat up faster and be downgraded
earlier.

That is, **measure short and `performance` wins; measure long and it may lose.**
And what we are trying to measure is **sustained throughput.**

**It has not been measured.** `ondemand` and `performance` have to be compared
under identical 300-second conditions. Until then this ADR's status is
**"provisional"**.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Stay on `ondemand` | The clock rises and falls, making it hard to isolate the source of measurement variance |
| Treat the governor as an experimental variable | **That is what has to happen eventually.** But for now other conditions had to be fixed, so one was chosen |
| `powersave` or a fixed frequency | What this project measures is close to "the maximum achievable" |
| Change governor according to temperature | Changing the subject of measurement mid-measurement. It becomes uninterpretable |

## Consequences

**Gained**

- The three nodes' conditions are unified
- It survives reboots
- The basis of the existing figures (`ondemand`) is explicitly recorded

**Lost / the cost**

- **Figures from before 2026-08-11 cannot be compared directly.** A warning is
  attached in the documents
- The possibility of being worse under sustained load is carried along

**New constraints introduced**

- **Always write the governor alongside** when quoting a measurement. "84.3
  inf/s" is a meaningless number without its conditions
- Preflight checks the governor. One node differing is a hard failure

## What would overturn this

**The re-verification plan is already set.**

```text
ondemand vs performance, identical 300-second conditions, 3 nodes
compared on: steady-state throughput, timing of CPU downgrade, mean temperature
```

If `performance`'s 300-second throughput is lower than `ondemand`'s, this
decision is overturned. That result is itself a valid output —
**"pinning the maximum clock is actually a loss at the edge"** is a conclusion
worth publishing.

---

<a id="adr-024"></a>

# ADR-024. Fix errors to an `NPF-xxxx` code scheme and keep it stable in the external API

*[한국어 원문](024-error-code-scheme.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-008](#adr-008), [ADR-026](#adr-026) |

---

## In one line

> Errors are expressed as **stable codes** such as `NPF-1302`. The number range
> indicates the error's nature, and that nature **determines whether it is
> retried**. Message strings may change; the codes do not.

## Context

Errors cross several boundaries in this system.

```text
node backend  ->  node agent  ->  gRPC  ->  scheduler  ->  gRPC  ->  client
                                                |
                                          decide whether to retry
```

For the scheduler to decide about retrying, it has to know **what the error the
node sent actually is**. Deciding from message strings breaks the decision logic
every time the wording is edited.

## Decision

**1. The number range carries the nature.**

| Range | Nature | Examples |
|---|---|---|
| 1000 | a problem with the request itself | `NPF-1001 INVALID_REQUEST`, `NPF-1002 PAYLOAD_TOO_LARGE` |
| 1100 | a model problem | `NPF-1101 MODEL_NOT_FOUND`, `NPF-1102 MODEL_VERSION_MISMATCH` |
| 1200 | a scheduling problem | `NPF-1201 NO_AVAILABLE_NODE`, `NPF-1202 DEADLINE_UNSATISFIABLE` |
| 1300 | a node problem | `NPF-1301 NODE_TIMEOUT`, `NPF-1302 NODE_UNAVAILABLE`, `NPF-1303 NODE_OVERLOADED` |
| 1400 | a backend problem | `NPF-1401 BACKEND_ERROR`, `NPF-1402 INFERENCE_FAILED` |
| 1500 | an internal error | `NPF-1501 INTERNAL_ERROR` |

**2. Defined in a single enum, with string conversion in both directions.**

```rust
pub const fn as_str(self) -> &'static str { ... }   // NPF-1302
pub fn from_str_code(s: &str) -> Option<Self>       // None when unknown
```

Why the reverse direction is needed: **the scheduler has to use the code the
node sent in its retry decision.**

**3. An unknown code is `None`, and the caller sets a conservative default.**
A node carrying new codes mixed with an old scheduler does not silently
misbehave.

**4. Codes stay stable in the external API.** Numbers are not reused and
meanings are not changed.

## Rationale

### The retry decision hangs on the code

| Retryable | Not retryable |
|---|---|
| Network connection failure | Invalid input |
| `NPF-1301` node timeout | Unsupported model |
| `NPF-1302` node unavailable | Unsupported input format |
| `NPF-1303` node overloaded | Model version mismatch |
| Transient runtime errors | Payload size exceeded |

**The 1300 range is retryable, the 1000 and 1100 ranges are not** — the number
range alone roughly separates them. Resending invalid input to another node just
fails the same way.

### String matching is not scattered through the code

The same principle as `SchedulingPolicyKind`
([ADR-009](#adr-009)). Gathering the parsing in one
place leaves nowhere for notation drift to appear.

### It was actually used in diagnosis

In the Mock 3-node integration test, the expected value for the "all nodes dead"
case is **`NPF-1302` plus the list of nodes attempted**. Because the code is
stable, the test can assert on it.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Use gRPC status codes only | Too few kinds, and they cannot carry domain meaning. A single `UNAVAILABLE` smears node death, overload and timeout together |
| Decide from message strings | Editing the wording breaks the logic. Localisation becomes impossible too |
| HTTP status codes | Ill-suited given internal RPC is gRPC. They can be used alongside in the management API |
| Define errors differently per layer | Conversion is needed at each boundary, and information disappears in the conversion |

## Consequences

**Gained**

- The retry decision is settled by one code
- Logs, metrics and tests use the same identifier
- The same error representation works over both gRPC and REST

**Lost / the cost**

- Once a code is published it **cannot be changed.** Only additions are possible
- Every new error needs a number assigned

**New constraints introduced**

- **Numbers are not reused.** Retiring one leaves the slot empty
- Adding a new code requires **deciding its retryability at the same time.**
  Without that, the caller treats it with the conservative default (not
  retryable)

## What would overturn this

If the range partitioning runs out of room, extend it (a 1600 range, for
example). **The meaning of existing numbers does not change.**

---

<a id="adr-025"></a>

# ADR-025. Re-register immediately when a heartbeat fails — and make registration idempotent

*[한국어 원문](025-heartbeat-failure-reregister.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-003](#adr-003), [ADR-016](#adr-016), [ADR-027](#adr-027) |

---

## In one line

> From the node's point of view, **a transient network error and a scheduler
> restart are indistinguishable.** So no effort is spent distinguishing them: a
> failed heartbeat always triggers re-registration. Registration is idempotent,
> so wasted effort does not translate into loss.

## Context

Nodes send heartbeats periodically (1–2 seconds by default). When one fails,
there are two cases.

```text
case A. the network dropped briefly      -> it comes back shortly and that is that
case B. the scheduler restarted          -> the scheduler's node list is empty
                                            without re-registering, the node never returns
```

**The node cannot tell them apart.** Both look identically like "no response".

Telling them apart would mean exchanging something like a scheduler instance
identifier, which then has to be maintained and propagated by the scheduler,
adding state.

## Decision

**1. A failed heartbeat switches straight to re-registration.** No
distinguishing.

**2. Registration is idempotent.** The same node registering repeatedly gives
the same result.

**3. The scheduler can demand re-registration.** The response carries a
`must_reregister` flag. The scheduler sets it on receiving a heartbeat from a
node it does not know.

**4. Initial registration has backoff retries**, because a node coming up before
the scheduler is normal.

## Rationale

### The more expensive option was chosen

Comparing the cost of the two options:

| | Cost |
|---|---|
| Re-registered when it was not needed | One RPC. Idempotent, so no state change |
| Did not re-register when it was needed | **The node drops out of the cluster permanently** |

The asymmetry is large. Better to repeat the cheap mistake.

### Measured: 1.3 seconds

Verified with four real processes (scheduler + 3 nodes).

```text
kill the scheduler  ->  bring it back  ->  all three nodes return by themselves in about 1.3 s
```

That figure is what actually supports
[ADR-003](#adr-003)'s "accept the single point of failure
but make recovery cheap". **If a restart costs 1.3 seconds without scheduler
redundancy, that is sufficient for experimental equipment.**

### Idempotency is this decision's premise

Without idempotent registration this design does not hold. If duplicate
registration created two nodes or reset state, the moment re-registration gets
issued liberally the cluster would break.

So **registration declares "this node exists"** rather than "add a new one".

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Detect restarts via a scheduler instance ID | Adds state, and if that value is wrong the same problem returns. What it buys is a few RPCs |
| Re-register after N failed heartbeats | Recovery becomes N times slower. All it buys is saved RPCs |
| Have the scheduler persist the node list to disk | It restores on restart but may be stale. It believes a node is there when it has gone |
| Have the scheduler discover nodes instead of nodes re-registering | The scheduler cannot find what it does not know about. A discovery mechanism (broadcast or similar) would then be needed |

## Consequences

**Gained**

- Scheduler restart recovery in 1.3 seconds
- The scheduler does not have to persist a node list
- One failure-handling path (no distinction = no branch)

**Lost / the cost**

- Unstable networks produce unnecessary registration RPCs. Harmless because
  idempotent, but traffic all the same
- "Why it re-registered" is in the log, but the cause (a blip or a restart)
  cannot be known

**New constraints introduced**

- **Registration handling must remain idempotent.** Adding a side effect here
  collapses the whole design
- Re-registration events alone **cannot distinguish a board reset from a process
  restart.** That is `boot_id`'s job
  ([ADR-016](#adr-016))

## What would overturn this

- **With tens of nodes**, simultaneous re-registration could pile onto the
  scheduler. Add jitter at that point
- **If registration becomes expensive** (sending a model list at registration,
  for instance), a reason to distinguish appears. The registration message is
  currently light

---

<a id="adr-026"></a>

# ADR-026. A retry always goes to a different node, and the backoff stays short

*[한국어 원문](026-retry-different-node.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-024](#adr-024), [ADR-009](#adr-009), `docs/01-TECHSPEC.md` §12 |

---

## In one line

> Never resend to the node that failed. **Temporarily exclude the failed node
> from the candidates and pick another.** One retry by default, backoff of
> 10–100 ms — this is real-time inference, so no long exponential backoff.

## Context

When an inference request fails there are three options.

```text
1. return it as a failure
2. resend to the same node
3. send to a different node
```

Inference requests have **no side effects.** Processing the same input twice
changes no state. That makes retrying safe — that is the premise.

## Decision

**1. Retryability is judged from the error code.**

| Retryable | Not retryable |
|---|---|
| Network connection failure | Invalid input |
| Node timeout (`NPF-1301`) | Unsupported model |
| Node unavailable (`NPF-1302`) | Unsupported input format |
| Node overloaded (`NPF-1303`) | Model version mismatch |
| Transient runtime errors | Payload size exceeded / authentication failure |

**2. On retry, the failed node is temporarily excluded from the candidates.**
The policy then picks from what remains.

**3. The defaults are short.**

```text
maximum retries       1
overall request timeout  5 s
retry backoff         10-100 ms
```

**4. No long exponential backoff.**

**5. The list of nodes attempted is carried in the error.** If all fail,
`NPF-1302` comes back along with which nodes were tried.

## Rationale

### Why not resend to the same node

If the cause of failure is in the node, **resending fails the same way.**

```text
the node died         -> it is still dead on resend
the node is overloaded -> resending overloads it further   <- worse
the node is hot        -> resending makes it hotter
```

`NPF-1303` overload in particular means a retry **makes the problem worse**. It
amounts to putting the same request into an already-full queue.

### Why the backoff is short

This is not batch work but **real-time inference.** A client is waiting for an
answer right now.

```text
exponential backoff (1s, 2s, 4s...)   ->  even success arrives late
short backoff (10-100ms)              ->  barely late at all if another node is alive
```

With a 5-second overall request timeout, spending 4 seconds on backoff leaves no
time to retry.

### Why one retry

There are three nodes. Failing once and failing again on another node leaves a
weak case for a third — a common cause (a model problem, a request problem)
becomes likely.

And the more retries there are, **the more the latency distribution during a
failure gets contaminated.** In the S4 failure-handling experiment, a high retry
count would make "latency during failure" a function of the retry policy.

### Why the list of nodes attempted is needed

When everything fails, "no node available" alone does not support diagnosis.
Which nodes were tried and why each failed is what narrows the cause.

The "all nodes dead" case in the Mock 3-node integration test asserts on this.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Retry on the same node | Pointless if the cause is in the node, and it worsens overload |
| Exponential backoff | Unsuited to real-time inference. Even success arrives late |
| Three or more retries | The latency distribution becomes dominated by the retry policy. With only three nodes there is little to gain |
| No retries | A client sees a failure whenever one node wobbles briefly. Fault tolerance is one of the goals |
| Retry every error | Invalid input gets failed round three nodes in turn. Wasteful, and it only blurs the cause |

## Consequences

**Gained**

- Clients see success even when one node dies (6/6 succeeded in the Mock test)
- Load does not concentrate further on an overloaded node
- Diagnostic information survives a failure

**Lost / the cost**

- Retried requests take longer. That value creates a tail in the latency
  distribution
- **The retry count has to be recorded with the results.** Otherwise there is no
  way to explain why the latency distribution is heavy (the bench tool records
  it)

**New constraints introduced**

- A request that succeeded on retry still counts as **one**. Processing it twice
  must not double the throughput count
- Failed requests are excluded from throughput and per-node shares
  ([ADR-028](#adr-028))

## What would overturn this

- **With more nodes** there is room to raise the retry count. At three there is
  little to gain
- **If requests with side effects** (state-changing APIs) are added, this
  premise breaks. Idempotency keys would then be needed. The scheduler currently
  only detects duplicate submissions with a short-TTL Request ID cache, and a
  result cache is not required for v0.1

---

<a id="adr-027"></a>

# ADR-027. Node state is an explicit state machine, with drain and disable kept separate

*[한국어 원문](027-node-state-machine-drain-disable.ko.md)*

| | |
|---|---|
| **Status** | accepted (thresholds are a draft) |
| **Date** | 2026-08-06 |
| **Related** | [ADR-009](#adr-009), [ADR-010](#adr-010), [ADR-025](#adr-025) |

---

## In one line

> A node is not seen as merely "alive or dead". It is managed as **eight states
> with explicit transitions**, and in particular **planned removal (drain) and
> forced exclusion (disable) are treated as different things.**

## Context

Whether a node can take requests is not binary.

```text
alive but slow
alive but hot
alive but failing often
just came back and not yet trustworthy
alive but about to be shut down
```

All of these need different handling. A single `bool is_alive` cannot express
them.

## Decision

**1. Define the states explicitly and fix the transition conditions.**

```text
Registering
   | registration succeeded
   v
Healthy --------------\
   | load high        | manual drain
   v                  v
Busy                Draining
   | errors rising    | queue empty
   v                  v
Degraded            Disabled
   | health check failed
   v
Unreachable
   | health check succeeded
   v
Recovering
   | consecutive successes
   \---------------> Healthy
```

**2. Distinguish `Draining` from `Disabled`.**

| | Meaning | In-flight requests |
|---|---|---|
| `Draining` | takes no new requests but **finishes what it has** | waits for completion |
| `Disabled` | removed from scheduling entirely | already empty |

`Draining` → queue empties → transitions to `Disabled`.

**3. Make every threshold configurable.**

```text
Heartbeat interval     2 s
Health timeout         1 s
3 consecutive failures     ->  Unreachable
3 consecutive successes    ->  Recovering to Healthy
queue length exceeded      ->  Busy
recent error rate over 10% ->  Degraded
temperature at or above 80 C ->  Degraded
temperature at or above 90 C ->  excluded from scheduling
```

**4. State is used both in the candidate filter and in the score.** The filter
checks eligibility via `is_schedulable()`, and ECT reads **degree** via
`load_factor` ([ADR-010](#adr-010)).

## Rationale

### Why drain is separated

There are situations where a node has to be pulled out mid-measurement. Cutting
it off immediately **records in-flight requests as failures**, and those
failures enter the error-rate statistics.

```text
immediate block   3 in-flight fail -> error rate rises -> measurement contaminated
using drain       3 in-flight finish, then it leaves quietly -> statistics clean
```

The S4 failure-handling experiment has to distinguish **intentional removal**
from **actual failure**, and without drain the two look identically like
failures.

### Why `Recovering` exists separately

Promoting a node straight to `Healthy` after it comes back means requests all
pile onto it, because its queue is empty. It dies again from the same cause.

`Recovering` is the state of "alive but not yet trusted". Three consecutive
successes are needed to reach `Healthy`, and meanwhile ECT suppresses it with
`load_factor 0.25`.

### Why temperature has two stages

```text
80 C  ->  Degraded              still takes work, but less of it
90 C  ->  excluded from scheduling   given nothing at all
```

A single stage makes it binary. If 79 °C and 81 °C are treated as entirely
different, a node flaps in and out at the boundary.

## ⚠️ The thresholds are a draft

**The current temperature thresholds (80 / 90 °C) conflict with the normal
operating range.**

Measurements show NPU temperature at 67.5–75.8 °C under sustained load, with
records of 86–90 °C depending on the load profile. That means **a node can drop
to `Degraded` during normal operation.**

They have to be reset after the formal S0 thermal measurement. Until then these
values are **a draft**, filed as a known issue in `docs/TODO.md` §6.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| A single `bool is_alive` | Cannot express slow, hot or recovering |
| Immediate blocking without drain | In-flight requests get recorded as failures and contaminate the statistics |
| Straight to `Healthy` with no `Recovering` | Takes the full load right after recovery and dies again |
| A single temperature threshold | Oscillates at the boundary |
| Interpret state differently per policy | Invalidates the policy comparison experiment ([ADR-009](#adr-009)) |

## Consequences

**Gained**

- Planned removal is distinguished from failure
- Overloading a recovering node is structurally suppressed
- State transitions are recorded as events, making timeline reconstruction
  possible

**Lost / the cost**

- With eight states, every transition combination has to be verified
- Eight more thresholds to tune

**New constraints introduced**

- **Changing a threshold is a change of experimental conditions.** It has to be
  recorded with the results
- Because the temperature thresholds are a draft, nodes can drop unexpectedly to
  `Degraded` in measurements taken before S0. Interpret those runs with care

## What would overturn this

- **S0's results settle the temperature thresholds.** That change is planned
- **If more states become necessary**, add them. But bear in mind that each
  additional state grows transition verification non-linearly

---

<a id="adr-028"></a>

# ADR-028. The bench tool judges run validity itself, and prints warnings above the numbers

*[한국어 원문](028-bench-run-validity.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-002](#adr-002), [ADR-015](#adr-015), [ADR-016](#adr-016) |

---

## In one line

> Past measurement mistakes are **built into the tool, not written in
> comments.** Warmup excluded, reboot detected via `boot_id`, insufficient
> samples flagged, failures excluded from throughput, percentile interpolation
> forbidden. And **invalidity warnings print above the numbers.**

## Context

`npuforge-bench` is not a new measurement but **a tool**. Yet the rationale for
its entire design comes from earlier failures.

Collecting the measurement mistakes so far, they fall into three kinds.

```text
A. did not check what a metric counts
B. compared values without noticing a condition had changed
C. nearly treated invalid data as valid
```

**Writing "let's be careful" in a comment did not work.** All three happened
while knowing better. So the tool enforces it.

## Decision

**1. Past mistakes are pinned as rules.**

| Past mistake | What the tool does |
|---|---|
| The first inference's latency spikes | Warmup requests excluded from aggregation |
| A reset board read as "degraded performance" | A change in `boot_id` → run invalid |
| p99 computed from 20 samples | Fewer than 100 successes → invalid |
| — | Failures excluded from throughput and per-node shares |
| — | Conditions (concurrency, seed, policy, node count) carried with the result |
| — | Percentiles are nearest-rank; interpolation forbidden |

**2. Invalidity warnings print before the numbers.**

```text
!!!!!! THIS RUN IS INVALID !!!!!!
  - error rate 100.00% exceeds the 1.00% allowance
  - 0 successful samples is below the minimum of 100
Do not quote the figures below.

requests : 200 (0 succeeded / 200 failed, ...)
```

**3. Invalid runs are not deleted.** They are kept with the reason.

**4. The policy name prefers the value the scheduler reports.**

**5. What the tool does not guarantee is written into the result file.**

## Rationale

### Why failures must not go into throughput

Include them and **throughput is highest when every node is dead.** Failures
return immediately, so requests per second explode.

```text
read this metric as-is in the S4 failure-handling experiment
  ->  the result reads "performance improves during an outage"
```

Per-node shares are the same. A failed request's `node_id` is empty, and
counting that makes **a dead node look like it "processed a lot"**.

### Why percentiles are not interpolated

Linear interpolation invents **values never actually observed** when samples are
few.

```text
interpolating p95 over observations 1-10  ->  9.55
no request experienced that latency
```

Writing "p95 = 9.55 ms" in a presentation makes it a computation, not a
measurement. It is fixed to nearest-rank and the definition is pinned in the
module documentation.

### Why the warning goes on top

**Show the numbers first and people believe them first.** Put the warning below
and the first screenful without scrolling is the numbers, and those numbers get
copied into a table.

### Why invalid runs are not deleted

They have to remain with their reason for the cause to be traceable. And
**repeated reboots are themselves a finding** — that is in fact how the power
adapter problem was found.

### Why the policy name comes from the scheduler

Typing `--policy round-robin` by hand goes wrong. **A result labelled with the
wrong policy name ruins the whole S3 policy comparison.**

### One problem caught during implementation

The first approach queried node state via the heartbeat RPC, because the
scheduler had no node listing API.

**But that overwrites the scheduler's node state.** A heartbeat is a call that
records observations, so a bench sending an empty `health` has the scheduler
accept it as a real observation and zero out temperature and queue depth. It
**contaminates the state of the thing being measured, immediately before
measuring it.**

A read-only `ListNodes` RPC was added separately. This too is a variant of type
A (using an API without checking its side effects).

## ⚠️ What the tool does not guarantee

**The load is a closed loop.** Concurrency N is fixed and the next request is
sent after the response arrives.

That approach is vulnerable to **coordinated omission**. When the system slows
down the client slows down with it, so **the latency distribution comes out
optimistic.** A slow request delays the launch time of subsequent requests, and
that delay is not charged to any request's latency.

→ **Never quote absolute latency as an SLA. Use it only for comparison between
configurations.** That sentence goes into the result file's `caveats` so it is
visible even when the results are read in isolation.

An open model (fixed target RPS) was not used because the node queue is finite.
Raising RPS quickly ends in `NPF-1303` rejections and the latency distribution
cannot be seen. If both are needed, that is added in M7.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Have a human look at the results and judge | Impossible across 146 runs / 23.4 hours of unattended overnight execution |
| Keep the rules in a document | Already confirmed not to work |
| Delete invalid runs automatically | Cause tracing becomes impossible. The pattern of repetition is itself information |
| Linear percentile interpolation (the common practice) | Invents unobserved values. Especially dangerous with few samples |
| Open-model load | The node queue is finite and it ends in rejections |

## Consequences

**Gained**

- Invalid data does not make it into the result tables
- Validity is judged automatically even in unattended runs
- The tool's limitations are written inside the result file

**Lost / the cost**

- The validity thresholds (100 successes, 1% error rate) are arbitrary values.
  There is room to sharpen the rationale
- The closed loop's optimistic latency is carried along

**New constraints introduced**

- **Absolute latency must not be quoted as an SLA.** For comparison between
  configurations only
- Each new mistake encountered adds a rule here

## What would overturn this

- **Adding an open model in M7** changes how the latency distribution is
  interpreted. The two models' results are not mixed
- The validity thresholds can be adjusted after S0, based on the actual
  distribution

---

<a id="template"></a>

# ADR-NNN. The decision in one sentence (end with a verb)

*[한국어 원문](TEMPLATE.ko.md)*

| | |
|---|---|
| **Status** | accepted / provisional / superseded |
| **Date** | YYYY-MM-DD |
| **Supersedes** | ADR-NNN (delete this row if none) |
| **Related** | ADR-NNN, `docs/xxx.md` §N |

---

## In one line

> The conclusion has to survive reading this line alone and closing the file.

## Context

What the situation was. Write it **for someone who does not know this field**.
When a term appears, explain it in one sentence on the spot.

If there was an earlier decision and this overturns it, put the story here.
What was believed, why it was believed, and what broke that belief.

## Decision

What was decided. Number them if there are several.

## Rationale

Why it was done that way. **If there are measurements, give them with their
conditions.**

```text
conditions: nodes, thread count, duration, governor, model
```

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| | |

## Consequences

- **Gained**
- **Lost / the cost**
- **New constraints introduced** — what has to be watched from now on because of
  this decision

## What would overturn this

What observation or condition would require revisiting this decision.

Write the re-verification method too. **What must not be looked at** matters in
particular — this project has reached a pass verdict **from the wrong metric**
four times, on things like "0 API errors" and "the NPU clock is pinned".
