# ADR-013. Make fanless the default, and treat throttling as something to measure rather than eliminate

*[한국어 원문](013-fanless-thermal-as-measurement.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-10 |
| **Related** | [ADR-002](002-success-criteria-measurability.md), [ADR-023](023-cpu-governor-performance-scoped.md), `docs/02-HARDWARE-SETUP.md` §9 |

---

## In one line

> Attaching a fan improves the numbers. But **edge devices sit in the field
> without one.** So fanless is the default condition, and performance falling
> from heat is treated as **something to measure, not something to remove.**
> Cooled conditions are measured separately as a comparison group.

## Context

RK3576 boards ship fanless. Put them under sustained load and they get hot and
slow down.

There are two branches here.

```text
branch 1. attach a fan
  -> the numbers improve
  -> good for a talk
  -> but those numbers do not occur in the field

branch 2. measure fanless
  -> the numbers get worse
  -> and the amount by which they get worse is a value nobody publishes
```

The TOPS vendors publish is **instantaneous performance**. How much of it is
sustained under load — **the gap between peak FPS and sustained FPS** — is
barely covered in public material.

## Decision

**1. Fanless (condition A) is the default measurement condition.**

**2. Active cooling (condition B) is measured alongside as a comparison group.**
Three fans of the same model are fixed at the same speed. Different speeds would
give the nodes different cooling conditions and break three-node symmetry.

**3. Thermal characterisation (S0) comes before every other scenario**, because
S0 determines the thresholds and cooldown times for the rest of the experiments.

**4. Do not mix improvised cooling into a measurement.** A desk fan was used
once during diagnosis; **it was valid for diagnosis but unusable as a
measurement condition.** There is a checklist item to confirm the desk fan is
off before a fanless measurement.

## Rationale

### Some questions can only be answered by measuring both conditions

```text
fanless only  ->  you do not know "how much better does cooling make it"
cooled only   ->  you do not know "how much do you get in a real edge deployment"
```

**Measure both and "the effect of cooling on scaling efficiency" becomes a
result in itself.** That is a value absent from vendor spec sheets, and it fits
this project's identity of settling things by measurement.

### Measured — it finishes fanless, but throughput is not sustained

```text
conditions: 3 boards concurrently, 8 threads, 900 s, fanless, no desk fan
```

| Board | NPU mean | NPU peak | Throughput |
|---|---:|---:|---:|
| king | 73.0 °C | 75.8 °C | 80.5 inf/s |
| queen | 67.5 °C | 70.2 °C | 77.7 inf/s |
| jack | 72.6 °C | 74.8 °C | 77.8 inf/s |

- Node-to-node spread **5.6 °C**
- Completed with 0 errors
- Never exceeded 90 °C

**Sustained 8-thread load is possible fanless.** But throughput is not
sustained.

```text
 +10s  81.6 inf/s   <- start
+120s  63.6
+300s  59.7         <- steady state.  -27% against the start
```

### ⚠️ What was collapsing was the CPU, not the NPU

The initial verdict was "no NPU throttling", because all 928 samples were at
950 MHz. **Only the NPU clock had been looked at.**

Looking at the CPU clocks in the same log:

```text
        NPU temp   npu_clk   cpu4(A72)   cpu0(A53)
 +15s   86.8 C     950 MHz   2208 MHz    2016 MHz
 +30s   90.4 C     950 MHz   1416 MHz    1200 MHz
 +60s   87.8 C     950 MHz    816 MHz     600 MHz
+120s   87.8 C     950 MHz    816 MHz     600 MHz
```

**The NPU never drops and the CPU falls 63–70%.**

One inference is `set input (CPU) → NPU → get output (CPU)`, so the CPU sections
feed directly into throughput. That was known, and the throttling verdict was
still made on the NPU alone. It is the **fourth** mistake of this type in this
project.

> The discovery actually improved the result. **"What collapses first on a
> fanless edge device is not the NPU but the CPU handling either side of it"** —
> a far better narrative for a talk.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Measure with a fan attached | Numbers that do not reproduce in the field. The opposite direction to the project's premise |
| Measure fanless only | Cannot answer "how much better does cooling make it" |
| Lower the load to avoid throttling | What happens under sustained load is exactly what is being measured, and this avoids it |
| Standardise conditions with improvised cooling (a desk fan) | Not reproducible and not uniform across nodes |

## Consequences

**Gained**

- The peak vs sustained gap became one of the project's central outputs
- The discovery that the bottleneck is the CPU rather than the NPU
- Readiness to quantify the cooling effect (S0-A / S0-B)

**Lost / the cost**

- Throughput figures come out lower. Instead of "84.3 inf/s" it has to be
  "81.6 at the start, 59.7 at 300 seconds"
- Measurement takes longer. Cooldown has to be waited out, and fanless is slow.
  So cooldown has **an upper bound**, and when that bound is hit the actual
  starting temperature is recorded with the result

**New constraints introduced**

- **Thermal verdicts must include the CPU clock.** Judging by NPU clock alone
  was confirmed wrong. `run-thermal-comparison.sh` has to be fixed accordingly
- Do not compare temperatures between two measurements with different load
  profiles. A sweep load was once compared against a fixed load and a 19 °C gap
  was misread
- The temperature thresholds (80 / 90 °C) are **a draft**. They are reset after
  the formal S0

## What would overturn this

- **If a case or heatsink becomes the standard configuration**, condition A's
  definition changes
- **If fanless exceeds 90 °C in S0 and nodes start dropping out of scheduling**,
  measurement itself becomes impossible. At that point condition B is promoted
  to default and condition A is redefined as "the limit condition"
