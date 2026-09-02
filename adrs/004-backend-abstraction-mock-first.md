# ADR-004. Separate the backend behind an interface, with Mock as a first-class backend

*[한국어 원문](004-backend-abstraction-mock-first.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-005](005-rknn-feature-gate-off-by-default.md) (feature gate), [ADR-007](007-per-thread-rknn-context.md), `docs/03-DEVELOPMENT-REQUIREMENTS.md` §4.1 |

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
[ADR-007](007-per-thread-rknn-context.md)'s shared-context problem — 0 errors and
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
