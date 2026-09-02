# ADR-016. Detect mid-measurement reboots with `boot_id` and invalidate the run

*[한국어 원문](016-boot-id-run-invalidation.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-015](015-preflight-hard-fail.md), [ADR-028](028-bench-run-validity.md), [ADR-027](027-node-state-machine-drain-disable.md) |

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
  ([ADR-025](025-heartbeat-failure-reregister.md)), so a re-registration event
  alone does not distinguish them

## What would overturn this

The check becomes unnecessary when "boards never reset" is proven, and there is
no way to prove it. **It stays.**
