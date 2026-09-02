# ADR-009. Fix the policies at three, and have all three share the candidate filter

*[한국어 원문](009-three-policies-shared-filter.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-003](003-central-simple-scheduler.md), [ADR-010](010-ect-formula.md), `docs/01-TECHSPEC.md` §10.0, §10.4 |

---

## In one line

> There are only `round-robin` / `least-queue` / `ect`. And **all three pass
> through exactly the same candidate filter.** If the filters differed, a policy
> comparison would measure the filters rather than the policies.

## Context

Comparing scheduling policies (scenario S3) is one of this project's
measurement items. It aims to measure "how much better is choosing by load than
simply going round in order".

A policy consists of two parts.

```text
1. candidate filter   who is eligible  (exclude dead nodes, only nodes holding the model ...)
2. selection rule     who among the candidates  (in order / shortest queue / estimated completion time)
```

There is a trap here. **If part 1 is made different per policy**, then when
policy A comes out ahead of policy B, there is no way to know whether that was
the selection rule or the filter.

For instance, if only ECT carried "exclude nodes above 85 °C", whether ECT wins
because it is smarter or because it avoids hot nodes cannot be separated.

## Decision

**1. Fix the policy identifiers at three.**

| Identifier | Policy | Purpose |
|---|---|---|
| `round-robin` | Round Robin | comparison baseline |
| `least-queue` | Least Queue | intermediate comparison |
| `ect` | Estimated Completion Time | recommended default |

**2. All three pass through an identical candidate filter.**

```text
- must be in an is_schedulable() state
- must hold the requested model in a Ready state
- temperature must be below disable_temperature_c
```

**3. Parse the identifier string in exactly one place.**

```rust
#[serde(rename_all = "kebab-case")]
pub enum SchedulingPolicyKind { RoundRobin, LeastQueue, Ect }
```

The configuration file, CLI arguments, metric labels, logs and dashboard all use
**the same strings**. Variants such as `queue-aware`,
`estimated-completion-time` or `queue_aware` are not used.

**4. Narrow the interface to the selection rule alone.**

```rust
pub trait SchedulingPolicy: Send + Sync {
    fn select_node(&self, task: &InferenceTask, candidates: &[NodeSnapshot])
        -> Result<NodeId, ScheduleError>;
}
```

`candidates` is **a list that has already passed the filter**. Since the policy
never sees the full node list, the room for a policy to add its own filter is
structurally reduced.

## Rationale

### Policy comparison is one of this project's measurement items

S3 is an experiment measuring "the difference between policies". There must be
one variable. Without a shared filter, the experimental design itself is void.

### A wobbling identifier contaminates the results

This actually came up while designing the bench tool. Having `--policy
round-robin` typed by hand invites a typo, or a value attached to the results
that differs from the scheduler's actual configuration. **A result labelled with
the wrong policy name ruins the whole of S3.**

So the bench tool **prefers the value the scheduler reports** over the one typed
by hand. It pairs with this decision.

### Three is enough

- `round-robin` is the baseline. Without it there is no way to know whether the
  rest are good
- `least-queue` answers "is looking at the queue alone sufficient?"
- `ect` looks at queue, speed, temperature and errors together

A fourth would multiply the experimental combinations and increase S3's run
count. It would not be worth it within the budget of 146 runs and roughly 23.4
hours.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| A different filter per policy | S3 would measure filter differences. **The thing most to be avoided** |
| Open the policies up as plugins | The comparison set becomes unbounded. Fixed is better for a measurement project |
| Implement only one policy (ECT) | Without a baseline there is no way to say "how much better" |
| Free-form identifier strings | Typos and notation drift contaminate the result labels |

## Consequences

**Gained**

- The S3 policy comparison holds — the single variable is the selection rule
- Policy names in configuration, logs, metrics and the dashboard are always the
  same
- Policy implementations get shorter. They do not each write a filter

**Lost / the cost**

- Policy-specific candidate conditions cannot be added. Adding one means
  **putting it in the shared filter and applying it to all three**
- Adding a new policy means editing the enum (deliberate friction)

**New constraint introduced**

- Changing the filter makes results **incomparable with the three policies' past
  measurements**. A filter change is treated as a change of experimental
  conditions and has to be recorded

## What would overturn this

- **If a candidate condition is found that genuinely must differ per policy.**
  At that point, first check whether it can be expressed as a score inside the
  selection rule — ECT's `load_factor` works that way
  ([ADR-010](010-ect-formula.md))
- **If the M7 optimization experiments need a new policy**, add a fourth. But
  only after S3's baseline comparison has already finished with three
