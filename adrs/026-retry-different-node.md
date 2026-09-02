# ADR-026. A retry always goes to a different node, and the backoff stays short

*[한국어 원문](026-retry-different-node.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-024](024-error-code-scheme.md), [ADR-009](009-three-policies-shared-filter.md), `docs/01-TECHSPEC.md` §12 |

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
  ([ADR-028](028-bench-run-validity.md))

## What would overturn this

- **With more nodes** there is room to raise the retry count. At three there is
  little to gain
- **If requests with side effects** (state-changing APIs) are added, this
  premise breaks. Idempotency keys would then be needed. The scheduler currently
  only detects duplicate submissions with a short-TTL Request ID cache, and a
  result cache is not required for v0.1
