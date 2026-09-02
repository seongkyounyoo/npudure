# ADR-002. Define success as "can it be measured and explained", not "did the number come out"

*[한국어 원문](002-success-criteria-measurability.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-05 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-001](001-data-parallel-only.md), [ADR-015](015-preflight-hard-fail.md), [ADR-028](028-bench-run-validity.md), `docs/00-PRD.md` §3 |

---

## In one line

> **No result value is set as a success condition**, such as "2.5× or better at
> three nodes". Even if scaling efficiency comes out low, even if io_uring has
> no effect, it is a success as long as the cause can be explained
> quantitatively.

## Context

What happens when a measurement project sets a target number as its success
criterion.

```text
goal: "3-node scaling efficiency of 80% or better"

measured 65%  ->  has to be recorded as a failure
              ->  nobody wants to fail
              ->  favourable conditions start getting found
                 measure briefly . smaller input . preheat well . keep only the good runs
```

**This is not something only dishonest people do.** Given freedom in choosing
conditions and a target hanging over you, the favourable option gets picked
unconsciously. And each of those choices can be given a plausible reason.

This project has unusually large freedom in choosing conditions. Governor,
thread count, duration, cooling, input size, model — all of them move the
numbers. On the same board, the governor alone moved 7% and duration alone
moved 27%.

## Decision

**Success is defined as the following.**

1. Was it measured
2. Were the measurement conditions recorded with it
3. Can the cause of the result be explained
4. Is it reproducible

**The following results are explicitly counted as valid outcomes.**

- io_uring producing no meaningful performance improvement
- Zero-copy applying to only a limited scope
- The NPU or preprocessing, rather than the network, being confirmed as the
  primary bottleneck
- Three-node scaling efficiency being lower than expected
- A single high-performance device being more favourable on cost

## Rationale

### It actually helped

Results that would have been discarded without this criterion became the central
output instead.

| Result | Under a target criterion | What actually happened |
|---|---|---|
| Three application-level optimizations at +0.1 / +5.4 / −1.8% | Failure. Bury it and try something else | Became **the basis for "there is nothing left to squeeze inside the node"** |
| Zero-copy at −1.8% | Failure | Hypothesis refuted. Led to the discovery that 76 ioctls are intrinsic to inference submission |
| −27% under fanless sustained load | A bad number | **The peak vs sustained gap** — a value absent from vendor spec sheets. Became the central narrative of the talk |

The third is decisive. Had the goal been "high throughput", we would have
attached a fan, measured for 120 seconds and reported 84.3 inf/s. That figure
**does not reproduce in the field.**

### It becomes possible to publish inverted conclusions

Measurement inverted this project's conclusions five times. With a target number
hanging over it, inverting is itself a loss — the already-reported number
becomes void.

With "can it be explained" as the criterion, **inverting becomes an outcome
instead.** That is why `docs/RESULTS.md` §4 "Inverted conclusions" and §6 "List
of measurement failures" can exist.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Set a target number (e.g. 80% scaling efficiency) | Creates condition-selection bias. The most dangerous thing in a measurement project |
| Target number + "state the reason if missed" | The reason section becomes a paragraph of excuses. The same problem remains the moment a miss is defined as failure |
| Set no criterion at all | There is no way to know when it is finished. Measurement goes on indefinitely |

## Consequences

**Gained**

- Unfavourable results can be published as they are
- Failure cases become output — with more reuse value than the numbers
- There is no longer any reason to hide measurement conditions

**Lost / the cost**

- **"So how many times faster is it?" is hard to answer in one line.** A
  disadvantage in a talk. The conditions have to be said alongside, so the
  sentence gets longer
- The success/failure verdict can look subjective. Hence the four explicit
  conditions above

**New constraints introduced**

- **Every number has to carry its measurement conditions.** A number without
  conditions is void under this criterion. Nodes, threads, duration, governor
  and model are always written alongside
- Invalid runs must not be used as though valid → enforced by tooling
  ([ADR-028](028-bench-run-validity.md))

## What would overturn this

If this project becomes **a product rather than an experimental tool**, the
criterion changes. A product needs a line of "it has to reach at least this to
be usable".

v0.1's purpose is measurement, so this criterion stands.
