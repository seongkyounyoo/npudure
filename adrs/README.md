# NPUDure Architecture Decision Records (ADR)

*[한국어 원문](README.ko.md)*

This folder answers only **"why is it like this?"**

What does what is `docs/01-TECHSPEC.md`'s job, and what numbers came out is
`docs/RESULTS.md`'s. Written here is **what was chosen and what was discarded at
each fork**, and **what observation would overturn that choice**.

---

## Why ADRs are needed separately

Most other documents in this repository are **chronological**.

| Document | Axis |
|---|---|
| `docs/discuss.md` | in the order experiments were run |
| `docs/board-worklog.md` | in the order work was done |
| `docs/RESULTS.md` | results gathered by topic, but the reasoning is scattered across the two above |

So answering a single question like "why does the node send integers now?" means
moving between three documents and reading chronologically. And since this
project has had **five conclusions inverted by measurement**, reading only the
early parts makes it easy to mistake an already-discarded judgement for a
current decision.

ADRs re-cut the same material **by topic**. One decision, one file.

---

## Status labels

| Status | Meaning |
|---|---|
| **accepted** | currently valid. The code and documents follow this decision |
| **provisional** | this is what is done for now, but the evidence is weak. The re-measurement conditions are in the body |
| **superseded** | another ADR overturned this decision. The header names the superseding number |

### The rule for handling inverted decisions

**Do not create a new ADR file for a discarded decision.** Instead, put the story
into the "Context" section of the ADR that superseded it.

This project has five inverted judgements (shared context, the 78 inf/s node
ceiling, no throttling, `king` at 19 °C, 2.5GbE being sufficient), so keeping
them as separate files would make half the list obsolete documents. That makes
it hard for a reader to find the valid decisions.

But **the story of the inversion is always kept.** That list is the highest
reuse-value output this project has (`docs/RESULTS.md` §6).

---

## Bundle

There is an **[ALL.md](ALL.md)** with everything concatenated, for reading,
printing and sharing. **It is generated, so do not edit it directly.** Fix the
source and regenerate.

```bash
python scripts/build-adr-bundle.py $(git log -1 --format=%cs -- adrs/)
```

---

## Reading order

For someone arriving for the first time, this order is recommended.

1. **[OVERVIEW.md](OVERVIEW.md)** — a map of the whole system. Read before the
   ADRs
2. **[001](001-data-parallel-only.md), [002](002-success-criteria-measurability.md), [003](003-central-simple-scheduler.md), [004](004-backend-abstraction-mock-first.md)** — the project's direction and skeleton
3. **[007](007-per-thread-rknn-context.md), [011](011-int8-quantization.md), [012](012-want-float-zero-blob-v2.md), [013](013-fanless-thermal-as-measurement.md)** — decisions that came out of actually handling the NPU and heat. The highest density of measurement
4. **[015](015-preflight-hard-fail.md), [017](017-remote-exec-pitfalls-library.md), [028](028-bench-run-validity.md)** — the devices that block "failures that look like success"
5. The rest can be looked up when needed

### If there is no time, just three

| # | Why |
|---|---|
| [007](007-per-thread-rknn-context.md) | 0 errors and 100% result mismatch. It shows this project's character best |
| [013](013-fanless-thermal-as-measurement.md) | what collapses first is the CPU, not the NPU |
| [002](002-success-criteria-measurability.md) | why bad numbers get published as they are |

---

## The list

### Project direction

| # | Title | Status |
|---|---|---|
| [001](001-data-parallel-only.md) | Split requests, not the model (data parallelism) | accepted |
| [002](002-success-criteria-measurability.md) | Define success as measurability, not a number | accepted |
| [022](022-document-authority-order.md) | Assign each document a normative domain; the normative one wins on disagreement | accepted |

### System structure

| # | Title | Status |
|---|---|---|
| [003](003-central-simple-scheduler.md) | One scheduler, and no high availability | accepted |
| [004](004-backend-abstraction-mock-first.md) | Separate the backend behind an interface, with Mock first-class | accepted |
| [005](005-rknn-feature-gate-off-by-default.md) | Put the RKNN link behind a feature and default it off | accepted |
| [006](006-crate-split-unsafe-isolation.md) | Split into seven crates and confine `unsafe` to one | accepted |
| [008](008-grpc-tonic-protobuf.md) | Internal communication uses gRPC (tonic + Protocol Buffers) | accepted |

### Scheduling

| # | Title | Status |
|---|---|---|
| [009](009-three-policies-shared-filter.md) | Fix the policies at three; all three share the candidate filter | accepted |
| [010](010-ect-formula.md) | The ECT score formula and each term inside it | accepted (before real-hardware validation) |
| [026](026-retry-different-node.md) | Retries always go to a different node; keep the backoff short | accepted |
| [027](027-node-state-machine-drain-disable.md) | The node state machine, with drain and disable separated | accepted (thresholds are a draft) |

### NPU runtime

| # | Title | Status |
|---|---|---|
| [007](007-per-thread-rknn-context.md) | A dedicated RKNN context per thread — sharing blocked by the type system | accepted |
| [011](011-int8-quantization.md) | The reference model is INT8 | accepted |
| [012](012-want-float-zero-blob-v2.md) | The node sends integers without dequantizing (`want_float=0`, blob v2) | accepted |
| [020](020-worker-count-8-no-core-mask.md) | `worker_count = 8`, `core_mask` unset | accepted |
| [021](021-no-node-side-postprocessing.md) | The node does no postprocessing (NMS) | **provisional** |

### Hardware and measurement environment

| # | Title | Status |
|---|---|---|
| [013](013-fanless-thermal-as-measurement.md) | Fanless as the default; throttling is something to measure | accepted |
| [014](014-10g-aggregation-separate-scheduler.md) | 10G on aggregation only; the scheduler on a separate server | accepted (built and measured) |
| [018](018-convert-model-once-deploy.md) | Convert the model once and deploy to all three nodes | accepted |
| [019](019-ssh-alias-not-ip.md) | Reach the boards by SSH alias, not by IP | accepted |
| [023](023-cpu-governor-performance-scoped.md) | CPU governor to `performance` — but state the scope of the evidence | **provisional** |

### Measurement discipline

| # | Title | Status |
|---|---|---|
| [015](015-preflight-hard-fail.md) | A hard-failing preflight check before measuring | accepted |
| [016](016-boot-id-run-invalidation.md) | Detect mid-measurement reboots with `boot_id` and invalidate the run | accepted |
| [017](017-remote-exec-pitfalls-library.md) | Harden the remote-execution pitfalls into library functions | accepted |
| [028](028-bench-run-validity.md) | The bench tool judges run validity itself | accepted |

### Protocol and policy details

| # | Title | Status |
|---|---|---|
| [024](024-error-code-scheme.md) | Fix errors to an `NPF-xxxx` code scheme | accepted |
| [025](025-heartbeat-failure-reregister.md) | A failed heartbeat re-registers immediately — registration is idempotent | accepted |

---

## The ADRs that came out of failures

This project made **the same type of mistake four times.** All of them were
"trusting a metric by its name without checking what it counts".

```text
1. reading run_duration as NPU occupancy time      -> it included queue wait
2. sampling NPU load with delayms=3000 still set   -> it was reading a 3-second average
3. judging thread-safety by API return codes only  -> results never compared   -> ADR-007
4. judging throttling by NPU clock alone           -> the CPU was the one bending -> ADR-013
```

The devices that came out of those are kept separately.

| ADR | What it blocks |
|---|---|
| [015](015-preflight-hard-fail.md) | starting a measurement on a false premise |
| [016](016-boot-id-run-invalidation.md) | reading a reboot as "performance degradation" |
| [017](017-remote-exec-pitfalls-library.md) | a remote command failing with exit code 0 |
| [019](019-ssh-alias-not-ip.md) | attributing one board's results to another node |
| [028](028-bench-run-validity.md) | an invalid run's numbers reaching the result tables |

---

## Writing a new ADR

1. Copy [TEMPLATE.md](TEMPLATE.md)
2. The filename is `NNN-ascii-kebab-slug.md`. Numbering continues **from 029**
3. Numbers are **not reused.** A discarded decision keeps its number
4. Update this README's list and statuses alongside

### What to hold to when writing

- **Always attach the measurement conditions to a number.** Nodes, thread count,
  duration, governor, model. We have already experienced that a number without
  conditions is useless three months later
- **Write down the rejected alternatives.** What was rejected and why lasts
  longer than what was chosen
- **Write down what is not known.** The "provisional" status and the "what would
  overturn this" section are where that goes
- **In the re-verification method, write what must not be looked at.** A pass
  verdict has been reached from the wrong metric four times
