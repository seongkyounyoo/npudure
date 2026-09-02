# ADR-028. The bench tool judges run validity itself, and prints warnings above the numbers

*[한국어 원문](028-bench-run-validity.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-002](002-success-criteria-measurability.md), [ADR-015](015-preflight-hard-fail.md), [ADR-016](016-boot-id-run-invalidation.md) |

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
