# Provenance

This repository is a **curated public snapshot**, not the original
development repository.

| | |
|---|---|
| Measurement period | 2026-08-07 ~ 2026-08-21 |
| Snapshot exported | 2026-08-31 |
| Original development history | retained privately, not published |
| Public release | curated snapshot of `phase-1-complete` |
| Valid measurements | **421** (418 benchmark runs + 3 profiling conditions) |
| Scheduler host | Dell PowerEdge R620, dual Xeon E5-2630L (24 threads) |
| Excluded runs | **4** (contaminated by concurrent harness collision) |
| Errors in valid runs | **0** |

## Experiment lineage

```text
S2  → S3  → S3.5 → S3.6 → S3.7 → S3.8 → S3.9 → S0
scaling ceiling  transport  window/conn  operating  optimized  residual  thermal
                  profile     A/B          point    scale-out   cost     & policy
```

Each experiment states the question it asked, the decision rule fixed
**before** the results arrived, and what was ruled out under which
conditions. Start at
[`experiments/README.md`](experiments/README.md).

## What is not here

Session handoff notes, publication strategy drafts, raw photographs
before review, and the calibration image set. None of them are research
artifacts; the first three are internal working material and the last has
per-image redistribution terms.

Reviewed photographs of the rig **are** included, in
`results/photos-public/`. Host hardware inventories are in `docs/hosts/`.

The scheduler host was replaced after the measurement period; the
measurements above are unaffected and are not restated. See
`docs/infrastructure.md` §3.2.1.

## Why the history is not published

The lineage that matters in this project lives in the experiment reports,
the ADRs, the reversed conclusions, and the failure list — not in the
commit graph. Publishing 90 internal commit messages would require a
separate audit for little gain.
