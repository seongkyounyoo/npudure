# ADR-NNN. The decision in one sentence (end with a verb)

*[한국어 원문](TEMPLATE.ko.md)*

| | |
|---|---|
| **Status** | accepted / provisional / superseded |
| **Date** | YYYY-MM-DD |
| **Supersedes** | ADR-NNN (delete this row if none) |
| **Related** | ADR-NNN, `docs/xxx.md` §N |

---

## In one line

> The conclusion has to survive reading this line alone and closing the file.

## Context

What the situation was. Write it **for someone who does not know this field**.
When a term appears, explain it in one sentence on the spot.

If there was an earlier decision and this overturns it, put the story here.
What was believed, why it was believed, and what broke that belief.

## Decision

What was decided. Number them if there are several.

## Rationale

Why it was done that way. **If there are measurements, give them with their
conditions.**

```text
conditions: nodes, thread count, duration, governor, model
```

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| | |

## Consequences

- **Gained**
- **Lost / the cost**
- **New constraints introduced** — what has to be watched from now on because of
  this decision

## What would overturn this

What observation or condition would require revisiting this decision.

Write the re-verification method too. **What must not be looked at** matters in
particular — this project has reached a pass verdict **from the wrong metric**
four times, on things like "0 API errors" and "the NPU clock is pinned".
