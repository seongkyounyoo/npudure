# ADR-001. Split requests, not the model (data parallelism)

*[한국어 원문](001-data-parallel-only.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-012](012-want-float-zero-blob-v2.md), `docs/00-PRD.md` §4, `docs/01-TECHSPEC.md` §2.1 |

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
