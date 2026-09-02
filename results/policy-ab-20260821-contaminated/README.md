# S0-C 4th attempt — **INVALID: concurrent harness collision**

*[한국어 원문](README.ko.md)*

> **Do not use for performance conclusions.**

2026-08-21. The wreckage of a fourth attempt to decide LQ vs ECT under strong
heterogeneity (2.4×). The experiment itself is in
[`S0_C_POLICY_AB.md`](../../docs/experiments/S0_C_POLICY_AB.md) §17–19, and the
incident is in
[`S0_D_CAPACITY_HETERO.md`](../../docs/experiments/S0_D_CAPACITY_HETERO.md) §4.

## What is valid and what is not

| Section | Status | Note |
|---|---|---|
| `round-robin,1` | **valid** | the basis for the gate verdict (§18.1). Spread 1.10× |
| `raw/thermal/*.log` | **valid** | the 1-second thermal logger. The basis for §18.2's soc and CPU clock aggregation |
| `least-queue,1` · `ect,1` | **invalid** | the fan was on by this point, so it is not a fanless condition |
| `least-queue,2` | **invalid** | the above, plus a collision with a second harness's c36 bench |

## Why it is invalid

Two things overlapped.

1. **The cooling condition changed mid-experiment.** Believing it had been
   stopped, the fan was switched on, but the harness was still alive and kept
   measuring under a fanless label.
2. **A harness collision.** The policy A/B harness that failed to stop and the
   newly started capacity calibration harness **hit the same three nodes at c36
   each** (72 combined).

`least-queue,2`'s 208.5 inf/s is not policy performance but a product of the
collision. Re-measured after cleanup at the same time, the value was
**391.2 inf/s / 0 errors / spread 1.02×**.

## Why it is kept

The incident itself is a methodology record —
[`experiments/README.md`](../../docs/experiments/README.md) §4.11 ("do not trust
'I stopped it' — verify at the shared resource"). The
`npuforge_assert_cluster_free` guard was added to prevent recurrence, and this
data is its basis.

The collision section survives verbatim in `raw/harness.log`.
