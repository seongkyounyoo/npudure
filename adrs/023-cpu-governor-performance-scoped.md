# ADR-023. Fix the CPU governor to `performance` — but state the scope of the evidence

*[한국어 원문](023-cpu-governor-performance-scoped.ko.md)*

| | |
|---|---|
| **Status** | provisional |
| **Date** | 2026-08-12 |
| **Related** | [ADR-013](013-fanless-thermal-as-measurement.md), [ADR-002](002-success-criteria-measurability.md), `docs/discuss.md` §11, §12 |

---

## In one line

> Switching `ondemand` → `performance` gives **+7%**, so it is fixed there.
> **But that +7% is a 120-second measurement.** Under sustained load
> `performance` heats up faster and may be worse, and **that has not been
> checked yet.**

## Context

Linux's CPU governor is the policy that adjusts clock speed with load.

| governor | Behaviour |
|---|---|
| `ondemand` | raises the clock only under load. The default |
| `performance` | always holds the maximum clock |

One inference is `set input (CPU) → NPU → get output (CPU)`, so CPU clock feeds
directly into throughput. That makes the governor a variable.

## Decision

**1. Fix all three nodes' governor to `performance`.** Made permanent with a
systemd unit so it survives reboots (`scripts/set-cpu-governor.sh`).

**2. Preflight verifies it before every measurement.**

**3. State the basis of the existing figures.** Every measurement before
2026-08-11 is on `ondemand`.

```text
ondemand      FP16 79.0 / INT8 146.2 inf/s
performance   FP16 84.3 / INT8 157.2 inf/s
```

**4. Pin down the scope of the "+7%" conclusion in the documents.** Read only as
"a gain in short measurements".

## Rationale

### Why fix it

More than the value itself, **unifying the condition** is what matters. A
governor that differs per node or per run makes three-node comparison
meaningless.

`performance` was chosen for two reasons.

- +7% in a 120-second measurement
- **Its behaviour is simple.** With `ondemand` the clock rises and falls with
  the load pattern, making it hard to separate whether variance in the
  measurements comes from the governor's decisions or from somewhere else

The second matters more. For reproducibility, the predictable option is better.

## ⚠️ Where this decision's evidence is weak

**The +7% is a 120-second measurement.** That window is before the CPU has been
fully downgraded.

What actually happens under sustained load:

```text
        NPU temp   cpu4(A72)   cpu0(A53)
 +15s   86.8 C     2208 MHz    2016 MHz
 +30s   90.4 C     1416 MHz    1200 MHz
 +60s   87.8 C      816 MHz     600 MHz   <- 63-70% downgrade
+120s   87.8 C      816 MHz     600 MHz
```

**`performance` holds the maximum clock even at idle.** So it has less thermal
headroom at the moment load starts. It may heat up faster and be downgraded
earlier.

That is, **measure short and `performance` wins; measure long and it may lose.**
And what we are trying to measure is **sustained throughput.**

**It has not been measured.** `ondemand` and `performance` have to be compared
under identical 300-second conditions. Until then this ADR's status is
**"provisional"**.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Stay on `ondemand` | The clock rises and falls, making it hard to isolate the source of measurement variance |
| Treat the governor as an experimental variable | **That is what has to happen eventually.** But for now other conditions had to be fixed, so one was chosen |
| `powersave` or a fixed frequency | What this project measures is close to "the maximum achievable" |
| Change governor according to temperature | Changing the subject of measurement mid-measurement. It becomes uninterpretable |

## Consequences

**Gained**

- The three nodes' conditions are unified
- It survives reboots
- The basis of the existing figures (`ondemand`) is explicitly recorded

**Lost / the cost**

- **Figures from before 2026-08-11 cannot be compared directly.** A warning is
  attached in the documents
- The possibility of being worse under sustained load is carried along

**New constraints introduced**

- **Always write the governor alongside** when quoting a measurement. "84.3
  inf/s" is a meaningless number without its conditions
- Preflight checks the governor. One node differing is a hard failure

## What would overturn this

**The re-verification plan is already set.**

```text
ondemand vs performance, identical 300-second conditions, 3 nodes
compared on: steady-state throughput, timing of CPU downgrade, mean temperature
```

If `performance`'s 300-second throughput is lower than `ondemand`'s, this
decision is overturned. That result is itself a valid output —
**"pinning the maximum clock is actually a loss at the edge"** is a conclusion
worth publishing.
