# ADR-006. Split into seven crates and confine `unsafe` to one of them

*[한국어 원문](006-crate-split-unsafe-isolation.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-004](004-backend-abstraction-mock-first.md), [ADR-005](005-rknn-feature-gate-off-by-default.md), [ADR-007](007-per-thread-rknn-context.md) |

---

## In one line

> `unsafe` code calling the C library exists **only inside `npuforge-rknn`.**
> The other six crates use safe Rust only. When a memory problem appears, there
> is **one place to look.**

## Context

Rust has the compiler guarantee memory safety, but that guarantee ends the
moment a C function is called. The RKNN Runtime is a C library.

```c
int rknn_init(rknn_context* ctx, void* model, uint32_t size, uint32_t flag, ...);
int rknn_inputs_set(rknn_context ctx, uint32_t n, rknn_input inputs[]);
int rknn_outputs_get(rknn_context ctx, uint32_t n, rknn_output outputs[], ...);
```

Pointers, lifetimes and release timing have to be managed by hand. Scatter that
code around the repository and there is no way to find where a bug like
use-after-free came from.

## Decision

**1. Split into seven crates.**

| Crate | Responsibility | `unsafe` |
|---|---|---|
| `npuforge-common` | types, error codes, configuration, backend interface | none |
| `npuforge-proto` | gRPC definitions (.proto → tonic generated) | none |
| `npuforge-scheduler` | policies, registry, retries, health checks | none |
| `npuforge-node` | worker pool, queue, registration and heartbeat | none |
| `npuforge-mock-backend` | the hardware-free backend | none |
| `npuforge-bench` | load generation, aggregation, validity judgement | none |
| **`npuforge-rknn`** | **RKNN FFI and its safe wrapper** | **only here** |

**2. `unsafe` does not leave `npuforge-rknn`.** The crate's documentation says
so — "unsafe code is confined to this crate".

**3. Convert to safe types at the boundary.** The outside sees only the
`InferenceBackend` / `LoadedModel` interfaces. Pointers do not cross the
boundary.

**4. Express dangerous contracts as types.** For example `RknnContext::infer`
takes `&mut self` so the compiler blocks concurrent calls
([ADR-007](007-per-thread-rknn-context.md)).

## Rationale

### 1. There is one place to look

When a memory error, a strange crash or an unexplainable value appears,
`npuforge-rknn` is where you start. That crate is a small share of the whole
workspace, so scanning it is cheap.

### 2. The other crates can be verified without hardware

`unsafe` and the hardware dependency sit in the same place, so removing that
leaves everything else as pure Rust. Swapping in the Mock is possible thanks to
the same separation.

### 3. It gives grounds to keep the C wrapper thin

The FFI goes through `native/rknn_wrapper.c`. That wrapper had **its signatures
verified against the real headers**, down to confirming that `rknn_context` is a
`uint64_t` on aarch64. Being in one place is what makes such a check possible.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| A single crate | `unsafe` spreads through everything. Keeping the Windows build alive with a feature gate would also get harder |
| Split into more crates | Seven is already plenty at this scale. Splitting further only raises dependency management cost |
| Auto-generate FFI with `bindgen` | The headers are not in the repository and it ties to an SDK version. Writing by hand and checking against real hardware was more controllable |
| `unsafe` directly inside the node | The node becomes tied to RKNN and the Mock path does not hold |

## Consequences

**Gained**

- The `unsafe` audit scope is fixed to one crate
- Six crates are tested without hardware
- The backend swap point is clear

**Lost / the cost**

- Refactors crossing crate boundaries are cumbersome. Types sometimes have to be
  lifted into `npuforge-common`
- `npuforge-common` is everyone's dependency, so touching it recompiles
  everything

**New constraints introduced**

- **Be careful about what goes into `npuforge-common`.** It is the contract, so
  lifting something needed by only one crate raises coupling
- The moment there is an urge to use `unsafe` in another crate, that is a signal
  to re-examine the design

## What would overturn this

- **If another NPU backend is added**, a new crate appears alongside
  `npuforge-rknn`. "unsafe in one place" widens to "unsafe only in the backend
  crates". At that point, where common FFI utilities live has to be decided
