# ADR-024. Fix errors to an `NPF-xxxx` code scheme and keep it stable in the external API

*[한국어 원문](024-error-code-scheme.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-008](008-grpc-tonic-protobuf.md), [ADR-026](026-retry-different-node.md) |

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
([ADR-009](009-three-policies-shared-filter.md)). Gathering the parsing in one
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
