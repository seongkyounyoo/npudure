# ADR-020. Use `worker_count = 8` and do not set `core_mask`

*[한국어 원문](020-worker-count-8-no-core-mask.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-10 |
| **Related** | [ADR-007](007-per-thread-rknn-context.md), [ADR-011](011-int8-quantization.md), `docs/discuss.md` §4 |

---

## In one line

> Eight workers beat four by **+27%**. Assigning NPU cores by hand with
> `core_mask` is worth **+0.1%** at 8 threads — effectively nothing.
> `CORE_AUTO`'s distribution is already even. **Not touching it is the
> conclusion.**

## Context

The RK3576's NPU has two cores. RKNN provides `core_mask` to specify which core
to use.

Early in measurement there was an observation that "Core1 occupancy is only
38%", and a hypothesis followed that the second core was idling. Assigning cores
explicitly was expected to raise throughput.

**But whether that 38% actually contributed to throughput had never been
checked.** Only the occupancy number had been looked at.

## Rationale

### A control group was added

Earlier measurements lacked "how much do you get using only one core". That has
to be there to judge the second core's contribution.

```text
conditions: queen, FP16, 200 iterations per thread, sampled after a 4 s warmup
```

| Threads | AUTO | ALTERNATE | CORE_0_1 | **CORE_0_ONLY** |
|---:|---:|---:|---:|---:|
| 1 | 16.7 | 16.7 | **18.2** | 16.5 |
| 2 | 36.2 | 36.5 | 36.4 | 26.4 |
| 4 | 52.4 | **57.1** | 48.5 | 38.5 |
| 8 | 72.9 | **73.0** | 64.5 | **48.2** |

### Finding 1. The second core does contribute — but by 1.51×

```text
8 threads   single core 48.2  ->  two cores 73.0 inf/s   =  1.51x
```

The 38% occupancy was not decoration. **But it is 1.51×, not 2×.** Doubling the
cores raises throughput by only half as much. That means there is a shared
resource outside the cores, which matches the "submission path serialization"
confirmed later.

### Finding 2. Explicit assignment brings no gain

```text
4 threads   52.4 -> 57.1   +9.0%
8 threads   72.9 -> 73.0   +0.1%
```

It rises only at 4 threads and vanishes at 8. And unpacking that 4-thread
improvement, most of it is a reduction in `outputs_get` (13.6 → 10.0 ms), so
**whether it is a core-assignment effect or measurement noise is not
separated.**

`AUTO`'s distribution is already even — Core0 39% / Core1 37% at 8 threads. The
runtime scheduler is doing its job and there is no room for manual intervention.

### Finding 3. `CORE_0_1` is actually a loss

```text
8 threads   72.9 -> 64.5   -11.5%
```

Making every thread use both cores together is slower.

## Decision

**1. `worker_count = 8` is the real-hardware default.** It is +27% over 4, and
it has not yet bent at 8.

**2. Do not set `core_mask`.** Leave it to `CORE_AUTO`.

**3. The configuration default stays 1, with real-hardware configuration giving
8 explicitly.** A default of 1 is the safe value when the backend is unknown.

**4. State explicitly that `worker_count` is directly tied to context count.**
The backend creates that many RKNN contexts
([ADR-007](007-per-thread-rknn-context.md)).

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| `core_mask = ALTERNATE` | +0.1% at 8 threads. Adds a configuration item and no gain |
| `core_mask = CORE_0_1` | −11.5%. Clearly a loss |
| `worker_count = 4` | −27% against 8 |
| Raise `worker_count` further | **The point where it bends has not been found yet.** But contexts grow with it, so a memory check comes first |

## Consequences

**Gained**

- One tuning item removed. **Deciding not to configure something is also a
  decision**
- Obtained the figure that the NPU's two cores contribute 1.51× — grounds for
  later bottleneck analysis

**Lost / the cost**

- +9% is given up at the 4-thread condition. But real hardware runs 8 workers

**New constraints introduced**

- **Raising `worker_count` raises the RKNN context count with it.** It must not
  be increased without checking memory headroom. The per-context memory increase
  has **not been measured**
- There is no basis for 8 being the ceiling. "It has not bent yet at 8" is the
  accurate statement. Widening `MAX_THREADS` and re-measuring remains an open
  item

## What would overturn this

- **Re-measuring against INT8 could give a different optimum.** The sweep above
  is FP16. INT8 takes less time per inference, so the optimal concurrency may
  differ. **Not yet checked**
- **Widening `MAX_THREADS` and measuring 12 and 16** could give a better value
- If memory runs short, 8 has to come down
