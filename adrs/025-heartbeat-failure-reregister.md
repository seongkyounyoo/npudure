# ADR-025. Re-register immediately when a heartbeat fails — and make registration idempotent

*[한국어 원문](025-heartbeat-failure-reregister.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-003](003-central-simple-scheduler.md), [ADR-016](016-boot-id-run-invalidation.md), [ADR-027](027-node-state-machine-drain-disable.md) |

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
[ADR-003](003-central-simple-scheduler.md)'s "accept the single point of failure
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
  ([ADR-016](016-boot-id-run-invalidation.md))

## What would overturn this

- **With tens of nodes**, simultaneous re-registration could pile onto the
  scheduler. Add jitter at that point
- **If registration becomes expensive** (sending a model list at registration,
  for instance), a reason to distinguish appears. The registration message is
  currently light
