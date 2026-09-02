# ADR-018. Convert the model once and deploy the same file to all three nodes

*[한국어 원문](018-convert-model-once-deploy.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-011](011-int8-quantization.md), [ADR-015](015-preflight-hard-fail.md) |

---

## In one line

> **INT8 conversion is not byte-reproducible.** Converting three times from the
> same input gave a different hash each time — even though the inference results
> are completely identical. So the model is not converted per node; **one file,
> converted once**, is deployed to all three.

## Context

With three nodes there are two ways to prepare the model.

```text
approach A. convert per node        ONNX -> .rknn on each board
approach B. convert once and deploy copy a .rknn built in one place
```

A looks natural. If the conversion script is deterministic, the same file should
appear on all three nodes.

**But it was not deterministic.**

## Rationale

### Measured: same input, different bytes

**Converted three times** from the same ONNX with the same calibration list.

```text
file size    identical
hash         different all three times
byte diff    1.8%
```

And the inference results were:

```text
all 9 output tensors at cosine 1.000000, error 0.0
```

**The difference is in serialization and layout, not in numerical computation.**
But with different files, "the three nodes use the same model" can no longer be
proved by hash.

### Why that matters

This project's premise is **that the three nodes' conditions are identical.**
Measuring 1/2/3-node scaling efficiency requires the nodes to be symmetric.

Preflight checks that the three nodes' model hashes match. Converting per node
makes that check **fail always**. Removing the check instead loses the means to
confirm "is it really the same model".

## Decision

**1. The model is converted once, in one place.** The conversion environment is
pinned with Docker (rknn-toolkit2 2.3.0).

**2. The resulting `.rknn` file is copied to all three nodes.**

**3. Deployment integrity is verified via `sha256` in `model.toml`.** The node
checks the hash when loading the model.

**4. State explicitly what that hash guarantees.**

```text
what sha256 guarantees      deployment integrity - the three nodes hold the same file
what sha256 does not        identity of the conversion recipe - that it was made the same way
```

**5. Make calibration image selection deterministic.** 200 images are chosen
from COCO val2017 with a fixed seed (`fetch_calibration.py`). The images
themselves are not put in the repository for licensing reasons; **only the
manifest** is kept.

**6. Apply the same principle to the node binary.** Only `king` has a Rust
toolchain; it builds once there and deploys to `queen` and `jack`. There is no
per-node build.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Convert per node | The hashes differ and the "same model" check becomes impossible |
| Make the conversion deterministic | That is internal to rknn-toolkit2 and outside our control |
| Verify identity by inference results instead of hash | That is also done (`preflight --with-inference`). But it is heavy as a deploy-time check |
| Drop the hash check | A corrupted file or a mixed-in different version would go unnoticed. That is exactly the accident it was meant to prevent |

## Consequences

**Gained**

- The three nodes hold **a byte-identical model**
- Preflight's model hash match check becomes meaningful
- Conversion environment problems do not multiply by three

**Lost / the cost**

- One more deployment step
- The conversion environment (Docker, rknn-toolkit2 version) becomes part of
  reproducibility. It is pinned in `environment-matrix.md`

**New constraints introduced**

- **Do not mistake `sha256` for verification of the conversion recipe.** The
  same hash means the same file, not that it was made by the same procedure.
  Reproducing it requires recording the conversion command, dataset and toolkit
  version separately
- Re-converting the model means **redeploying to every node.** Updating one node
  alone gets blocked by preflight (intended behaviour)

## What would overturn this

- **If rknn-toolkit2 comes to guarantee deterministic conversion**, per-node
  conversion becomes possible. Though converting once and deploying is still
  simpler
- **If an experiment requires the model to differ per node** (different
  precision per node, say), the premise changes. In that case "node symmetry"
  itself becomes an experimental variable
