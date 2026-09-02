# ADR-010. The ECT score formula and each term inside it

*[한국어 원문](010-ect-formula.ko.md)*

| | |
|---|---|
| **Status** | accepted (before real-hardware validation) |
| **Date** | 2026-08-06 |
| **Related** | [ADR-009](009-three-policies-shared-filter.md), [ADR-027](027-node-state-machine-drain-disable.md), `docs/01-TECHSPEC.md` §10.4 |

---

## In one line

> The default policy, ECT, scores **"if this request goes to that node, when
> will it finish"** and picks the lowest. Every term in the formula has a
> reason, and in particular `+ 1` and `load_factor` make it misbehave if
> removed.

## Context

Least Queue picks "the node with the shortest queue". That is enough when nodes
perform identically, but in practice they do not.

```text
node A  queue 2, 50 ms each   ->  free in about 100 ms
node B  queue 1, 200 ms each  ->  free in about 200 ms
```

Least Queue picks B. **Wrong.** Queue length alone cannot tell you "when it will
be free". Nodes differ in speed, they slow down with heat, and they may have
been failing recently.

## Decision

```text
ECT = ((queue_depth + in_flight + 1) x EWMA_inference_time
       + EWMA_network_time
       + thermal_penalty
       + error_penalty)
      / load_factor
```

The node with the lowest score is chosen. Ties break on **Node ID in
lexicographic order**.

### Each term

| Term | Meaning |
|---|---|
| `queue_depth` | requests on that node not yet started |
| `in_flight` | requests currently being processed |
| `+ 1` | **this very request being assigned** |
| `EWMA_inference_time` | moving average of recent inference times. The node's actual speed |
| `EWMA_network_time` | moving average of scheduler↔node round trip |
| `thermal_penalty` | added when the temperature is high |
| `error_penalty` | added when errors have been frequent recently |
| `load_factor` | a per-state weight. The divisor |

## Rationale

### Why `+ 1` cannot be omitted

Two reasons.

**First, that is ECT's definition.** It estimates "when will this request
finish", so **its own inference time has to be included**. Placing it on a node
with 2 ahead means 3 including mine.

**Second, without it `load_factor` is neutralised.**

```text
a node with an empty queue:  (0 + 0) x EWMA = 0
                             0 / load_factor = 0     <- always 0, whatever the state
```

Zero divided by anything is zero. The `Recovering` suppression below disappears
entirely.

### The problem `load_factor` solves

| State | load_factor |
|---|---:|
| Healthy | 1.0 |
| Busy | 1.0 |
| Degraded | 0.5 |
| Recovering | 0.25 |
| Otherwise | 0.0 (excluded from candidates) |

**A `Recovering` node has an empty queue, so on score alone it always wins.**
Every request piles onto a node that has just come back, and it dies again from
the same cause.

PRD FR-07 requires "assign only limited requests to a recovered node". This
could have been implemented with a separate counter or token bucket, but it is
**expressed as a single score.** Dividing by `0.25` quadruples the score, so it
naturally gets picked less.

The point is putting state into **the score rather than the candidate filter**.
Filtering gives only "use it or do not"; a score can express **degree**.

### Why tie-breaking is fixed to lexicographic Node ID

**Reproducibility.** Breaking ties randomly or by hash order would give a
different distribution each time the same experiment is repeated. That inflates
the variance of scaling-efficiency measurements, with no way to explain where
the variance came from.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Use Least Queue only | Does not reflect node speed differences. Wrong answer in the A/B example above |
| Exclude `Recovering` from candidates | A recovered node never comes back in. Then when to admit it has to be decided anyway |
| A separate token bucket to limit recovered nodes | One more piece of state. The score formula alone does the job |
| Break ties randomly | Reproducibility breaks |
| Handle temperature and errors as filters only | Becomes binary. 79 °C and 81 °C get treated as entirely different |

## Consequences

**Gained**

- Node speed differences, temperature, error rate and recovery state
  **unified into one score**
- Recovered-node suppression implemented without additional state
- Ties are deterministic, so repeated experiments reproduce

**Lost / the cost**

- **More tuning parameters.** The EWMA coefficients, the magnitudes of
  `thermal_penalty` / `error_penalty`, the `load_factor` values — all have to be
  set
- The formula is complex enough that "why was this node picked" is hard to read
  straight off a log

**New constraints introduced**

- **Not yet validated on real hardware.** Behaviour was confirmed on a 3-node
  Mock, but whether `load_factor` and the penalty values are actually right has
  to be seen in M4. The current values are **a draft**
- The temperature thresholds (80 / 90 °C) are a draft too. They are reset after
  the formal S0 thermal measurement

## What would overturn this

- **If ECT is not better than Least Queue in M4's real-hardware validation**,
  suspect the formula. Though that result is itself a valid output
  ([ADR-002](002-success-criteria-measurability.md))
- **If a `Recovering` node still gets overloaded even at 0.25**, lower the value
  or add an absolute cap
- **If the penalty terms turn out to have no effect at all**, removing them is
  also a result. A term existing and a term working are different things
