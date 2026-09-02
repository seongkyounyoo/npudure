# ADR-022. Assign each document a normative domain, and follow the normative one when values disagree

*[한국어 원문](022-document-authority-order.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-05 |
| **Related** | [ADR-002](002-success-criteria-measurability.md), `docs/00-PRD.md` §0 |

---

## In one line

> When the same value is written in several documents, one of them inevitably
> goes stale. So **each domain gets one normative document**, and when values
> disagree, that document wins. Other documents reference rather than duplicate.

## Context

This repository has many documents: PRD, TECHSPEC, hardware, development
requirements, environment-matrix, RESULTS, TODO, discuss, board-worklog.

The same number appears in several places. "157.2 inf/s per node", for example,
appears in RESULTS, in TODO, in board-worklog and in environment-matrix.

**Fix one and the rest go stale.** This actually happened.

```text
after the switch to want_float=0
  RESULTS §2.2  updated       "INT8 +17.3%"
  RESULTS §5    not updated   "INT8 throughput impact unmeasured"   <- contradictory within one document
  TECHSPEC §3.2 not updated   the discarded network calculation left as-is
```

## Decision

**1. Each domain gets one normative document.**

| Domain | Normative document |
|---|---|
| Goals, non-goals, functional requirements, success criteria | `00-PRD.md` |
| Repository structure, protocol, config schema, scheduling algorithm, error codes | `01-TECHSPEC.md` |
| Physical setup, network, power, cooling, experimental conditions | `02-HARDWARE-SETUP.md` |
| Development environment, tooling, deployment automation, licensing | `03-DEVELOPMENT-REQUIREMENTS.md` |
| Version combinations and hash pinning | `environment-matrix.md` |

**2. When values disagree, the normative document wins.**

**3. Reference rather than duplicate.** The PRD covers only "why" and "what".
Formulas, crate names, configuration keys and identifier strings are not written
in the PRD; it points at TECHSPEC.

**4. Documents of a different nature are not normative.**

| Document | Nature |
|---|---|
| `discuss.md` | Chronological discussion. Later sections correct earlier ones |
| `board-worklog.md` | Work history. Wrong hypotheses are preserved |
| `RESULTS.md` | A collection of results. The final authority for values is environment-matrix |
| `TODO.md` | What is to be done now |
| `adrs/` | Decisions and their rationale |

## Rationale

### Not duplicating is the only method

There are only two ways to maintain consistency.

```text
1. duplicate, and find and fix every copy on each change   -> one will always be missed
2. keep it in one place from the start                     -> there is nowhere to go stale
```

This project already failed with option 1. The single switch to `want_float=0`
involved five documents — `RESULTS.md`, `TECHSPEC`, `environment-matrix`, `TODO`
and `board-worklog` — and one sync pass did not catch them all.

### Why chronological documents are excluded from normativity

`discuss.md` **deliberately keeps wrong conclusions.** Section 5's "+5.4%" is
stale by current standards, but section 5 has to stay as-is for anyone to
understand why section 12 corrected it.

Making such a document normative means a reader who got as far as the earlier
section quotes a discarded value. So **chronological documents are supporting
material, not authority.**

## ADRs complement this structure

Normative documents answer "what is the value now". Chronological documents
answer "what happened". **The place answering "why was it decided that way" was
empty.**

`adrs/` is that place. It takes values from the normative documents and the
story from the chronological ones, and re-bundles them by decision.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Merge into a single document | It runs past ten thousand lines. Readers with different purposes cannot be served by one document |
| Manage without priority | No way to decide which is right when values conflict |
| Generate values automatically | Possible for some (test counts and the like), but measured values have to be written by a person along with their conditions |

## Consequences

**Gained**

- There is a rule for deciding when values conflict
- Each document's role is clear
- Where to make a change can be pinpointed

**Lost / the cost**

- Learning about one topic means moving between documents. **That inconvenience
  is the direct reason `adrs/` exists**
- You have to remember which document is normative

**New constraints introduced**

- **When duplication is found, delete it and replace with a reference.** The
  urge to copy a value over for convenience keeps arriving
- ADRs quote values too. A quoted value can go stale, so **the measurement
  conditions and the source are written alongside**

## What would overturn this

As documents grow, normative domains get added. As they shrink, they get merged.
The principle itself does not change.
