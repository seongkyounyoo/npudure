# ADR-003. One scheduler, and no high availability

*[한국어 원문](003-central-simple-scheduler.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-001](001-data-parallel-only.md), [ADR-014](014-10g-aggregation-separate-scheduler.md), `docs/01-TECHSPEC.md` §2.3 |

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
| P2P / gossip | No benefit at three nodes. It introduces inter-node communication, breaking [ADR-001](001-data-parallel-only.md)'s premise that nodes do not know each other |
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
  (→ [ADR-014](014-10g-aggregation-separate-scheduler.md))
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
