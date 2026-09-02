# ADR-027. Node state is an explicit state machine, with drain and disable kept separate

*[한국어 원문](027-node-state-machine-drain-disable.ko.md)*

| | |
|---|---|
| **Status** | accepted (thresholds are a draft) |
| **Date** | 2026-08-06 |
| **Related** | [ADR-009](009-three-policies-shared-filter.md), [ADR-010](010-ect-formula.md), [ADR-025](025-heartbeat-failure-reregister.md) |

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
`load_factor` ([ADR-010](010-ect-formula.md)).

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
| Interpret state differently per policy | Invalidates the policy comparison experiment ([ADR-009](009-three-policies-shared-filter.md)) |

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
