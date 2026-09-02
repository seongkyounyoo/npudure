# ADR-015. Run a hard-failing preflight check before measuring, and measure nothing until it passes

*[한국어 원문](015-preflight-hard-fail.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-007](007-per-thread-rknn-context.md), [ADR-016](016-boot-id-run-invalidation.md), [ADR-019](019-ssh-alias-not-ip.md), [ADR-028](028-bench-run-validity.md) |

---

## In one line

> What has ruined measurements so far has mostly been **the premises, not the
> measurement itself**. So a machine checks the premises immediately before
> measuring. **On a hard failure, measurement does not start.** And accuracy is
> checked **before** performance.

## Context

Measurements have been wrong several times, and the cause was always outside
the measurement code.

| What happened | Result |
|---|---|
| A stale IP in the docs pointed somewhere else | Misdiagnosed as a dead node; scanned the whole subnet |
| Compared two measurements with different load profiles | A 19 °C gap was misread |
| A board reset by insufficient adapter current | Its throughput was nearly read as performance |
| Sharing a context | 0 errors and 100% result mismatch |

**What they share: it was already wrong before measurement began.** And all four
give no signal while running.

## Decision

**1. Create `scripts/preflight-check.sh`, and do not measure until it passes.**

The verdict is the exit code.

```text
0  pass (warnings are possible)
1  hard failure. Measuring in this state makes the result invalid
2  script usage error
```

**2. Divide the checks into six groups.**

| Group | What it looks at |
|---|---|
| 1. Connection and identity | alias ↔ hostname agreement |
| 2. Software identity | kernel, RKNN, driver and model hashes identical across the three nodes |
| 3. Measurement conditions | governor, idle temperature, input voltage, residual load, NTP, session count |
| 4. **Inference accuracy** | do the three boards give the same answer to the same input |
| 5. Network measurement | record M3's premise values |
| 6. Cluster registration | are the three nodes attached to the scheduler |

**3. This script does not fix anything. It only judges.** Fixing is
`fix-node-consistency.sh`'s job.

**4. Treat empty values and placeholders as failures.**

**5. When adding a check, break it deliberately and confirm it actually
catches.**

## Rationale

### Why accuracy comes before performance

That is what `--with-inference` does. It gives the three boards the same input
and checks that the same answer comes out.

The reason is [ADR-007](007-per-thread-rknn-context.md). The shared-context
configuration **produced wrong answers faster** (at two threads, shared 34.8 >
dedicated 33.2 inf/s).

**A configuration that produces wrong answers fast must not win a benchmark.**
Measure performance alone and such a configuration gets reported as optimal.

### The incident where "could not read" was judged as "identical"

`/sys/kernel/debug/rknpu/version` is readable only by root. Reading it without
permission returned an empty string on all three nodes, and it **passed on the
grounds that the values matched**.

```text
king  ""      \
queen ""      +- the three values match -> pass OK   <- nothing was verified
jack  ""      /
```

A variant of the mistake of not checking what a metric counts. So empty values
and placeholders such as `unknown` are treated as **failures**.

### Why the alias ↔ hostname check is number 1

**This is far more dangerous than a connection failure.** A failed connection is
known immediately. But if `npuforge-k` points at `queen`, the measurement
finishes normally and **the result is attributed to the wrong node.** It fails
quietly.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Keep the checklist in a document and have a human verify | "Let's be careful" in a document did not work. Several failures happened while knowing better |
| Put the checks inside the bench tool | Some are ([ADR-028](028-bench-run-validity.md)). But SSH, sudo and hash comparison are outside the tool's domain, so they were separated |
| Make check failures warnings only | Warnings get ignored, especially when you want to start measuring quickly |
| Fix things automatically | Mixing judgement with remedy leaves no record of "what had been wrong" |

## Consequences

**Gained**

- Premise failures are caught **before** measurement
- Pass/fail is an exit code, so it drops straight into automation
- The measurement conditions get recorded (`--json`)

**Lost / the cost**

- It takes time to start measuring, especially `--with-inference`, which runs
  real inference
- A sudo password is needed. It is not committed to the repository but taken
  from an environment variable or a `~/.npuforge/` file — **this project is
  going public, and anything in the commit history would need a history rewrite
  to remove**

**New constraint introduced**

- **The check itself can be wrong.** `pgrep -f` once counted itself and passed
  quietly ([ADR-017](017-remote-exec-pitfalls-library.md)). That is why "break
  it deliberately" became a rule

## What would overturn this

The list of checks keeps growing. **It never shrinks.** Adding an entry every
time a new failure mode is encountered is this script's design intent.
