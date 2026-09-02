# ADR-007. A dedicated RKNN context per thread, with sharing blocked by the type system

*[한국어 원문](007-per-thread-rknn-context.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Supersedes** | the judgement in `environment-matrix.md` §3.1 that "RKNN 2.3.0 is thread-safe, so the context may be shared" |
| **Related** | [ADR-020](020-worker-count-8-no-core-mask.md) (`worker_count=8`), `docs/discuss.md` §9, `docs/RESULTS.md` §4.3 |

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
