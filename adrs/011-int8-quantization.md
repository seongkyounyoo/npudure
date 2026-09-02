# ADR-011. The reference model is INT8

*[한국어 원문](011-int8-quantization.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-012](012-want-float-zero-blob-v2.md), [ADR-014](014-10g-aggregation-separate-scheduler.md), [ADR-018](018-convert-model-once-deploy.md) (model deployment), `docs/discuss.md` §8 |

---

## In one line

> INT8 quantization is worth **1.86×**. It landed an order of magnitude harder
> than any software optimization attempted so far. The cost is −5.5% on the top
> detection score, and **the detection set and classes are identical**.

## Context

### What quantization is

A neural network normally computes in reals (FP32). **Shrinking those reals to
8-bit integers** for computation is INT8 quantization. Each multiplication gets
cheaper and less memory moves. In exchange, values get coarser and a little
accuracy is lost.

FP16 sits in between — still real, just half the bits.

### Why this choice mattered

Starting from FP16, three things were tried to raise one node's throughput, and
**all three failed.**

| Attempt | Result |
|---|---:|
| Manual NPU core assignment via `core_mask` | +0.1% |
| `want_float=0` (measured mostly single-threaded at the time) | +5.4% |
| Zero-copy buffer reuse | **−1.8%** |

The reason was found too. Each inference triggers about 76 kernel `ioctl` calls
and those get **serialized**. Not something the application could reduce. So the
conclusion at the time was **"the node ceiling of 78 inf/s is a driver
characteristic."**

INT8 was the last big variable still outstanding.

## Decision

**1. The reference model is YOLOv8n INT8.**

**2. FP16 is not deleted but kept as a comparison condition.** Presenting the
two models side by side is itself the result of "how much does quantization
buy".

**3. Define the accuracy acceptance criterion at the detection level rather than
raw tensor similarity.** (See the trap section below for why.)

## Rationale

### 1.86×

```text
conditions: king, sustained_load_test, 8 threads fixed, 120 s,
            governor=performance, fanless
```

| Model | Throughput | Mean latency | Model size |
|---|---:|---:|---:|
| YOLOv8n FP16 | 84.3 inf/s | 94.5 ms | 9.65 MB |
| **YOLOv8n INT8** | **157.2 inf/s** | **50.8 ms** | 6.46 MB |
| Ratio | **1.86×** | −46% | −33% |

> Initial values measured with the `ondemand` governor were FP16 79.0 / INT8
> 146.2. **The 1.85–1.86× ratio holds regardless of governor.**

### This measurement corrected an earlier conclusion

If INT8 is 1.85×, that conflicts with the explanation that "76 ioctls set the
ceiling". So INT8's ioctls were counted too.

```text
strace -c -f -e trace=ioctl, 1 thread, 20 s

        inferences  throughput    ioctls per inference
FP16    315         15.7 inf/s    76.4
INT8    718         35.8 inf/s    76.2
```

**The call count is identical and throughput is 2.28×.**

What sets the ceiling is not the **number** of ioctls but **how long one
inference holds the serialized section.** So the scope of the previous
conclusion was narrowed.

| Previously | Corrected |
|---|---|
| "The node ceiling of 78 inf/s is a driver characteristic" | "**On FP16**, the node ceiling is about 78 inf/s, and that value cannot be exceeded by application optimization" |
| "It cannot be exceeded by application optimization" | Stands. But **quantization is a model change, not an application optimization** |

### The accuracy cost is acceptable

```text
conditions: real board king, COCO val2017 images,
            preprocessing done in one place so both see the same input bytes
```

| Comparison | box cosine | Detection cells | Class agreement |
|---|---|---|---|
| FP16 vs ONNX | 0.99999 | 10/10 | 100% |
| **INT8 vs FP16** | **0.997** | **10/10** | **100%** |

The top detection's cell moves by one and its score is −5.5%. **The detection
set and classes are identical.** Buying 1.86× at that price is a good trade.

## ⚠️ The trap hit during accuracy verification

**Using raw-tensor cosine similarity as the acceptance criterion misjudges this
model.**

Even for FP16 vs ONNX — a comparison with no quantization at all — **the cosine
of some tensors falls to 0.16.** Looking at that number alone leads to "the FP16
conversion broke the model". A wrong conclusion.

The cause is this.

- Of YOLOv8n's 9 outputs, tensors 2/5/8 are **the sum of 80 class scores**
- RKNN's sigmoid does not output exactly 0 but has **a floor of 0.001831**
- Amplified 80×, that produces **a 0.1465 offset** (matching the measured floor
  exactly)
- Most output cells are background, so this offset dominates the cosine

**The same value is added to every cell, so the ranking does not change. The
detections are unaffected.**

→ The acceptance criterion was changed to the **detection level** (detection
set, classes, box cosine). `tools/model-converter/compare_detections.py`
compares against that criterion.

This too is one of this project's recurring failure types. **A metric's name was
read and its meaning assumed.** "Low cosine similarity = different results" is
generally true, but not for this output structure.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Stay on FP16 | Throws away 1.86×. And it has already been confirmed there is no way to produce that much in software |
| FP32 | Meaningless on this NPU. Big and slow |
| INT8 + accuracy-loss compensation (QAT and the like) | Requires retraining. This project builds an inference runtime, not trains models |
| A larger model (YOLOv8s and the like) at INT8 | The comparison baseline changes. Model selection is a separate decision, and only one variable moves at a time here |

## Consequences

**Gained**

- 157.2 inf/s per node. 1.86× against FP16
- Mean latency 94.5 → 50.8 ms
- Model size −33%

**Lost / the cost**

- Top detection score −5.5%, top detection cell moved by one
- **Calibration data became necessary.** 200 COCO val2017 images are chosen
  deterministically (`fetch_calibration.py`, fixed seed). The images are not put
  in the repository for licensing reasons; only a manifest is kept
- **INT8 conversion is not byte-reproducible.** Converting three times from the
  same input gave a different hash each time (same size, 1.8% of bytes
  differing). But **the inference results are completely identical** (all 9
  tensors at cosine 1.000000). The difference is in serialization and layout,
  not in computation → the model is converted once and deployed to all three
  nodes (ADR-018)

**New constraint introduced**

- **Network load went up instead.** With throughput at 1.86×, the bytes moving
  per second rise by the same factor — 1.545 Gbps per node, 4.636 Gbps across
  three. This decision is the direct cause of
  [ADR-014](014-10g-aggregation-separate-scheduler.md)'s 10G aggregation
- Kept as a case of something else filling up when performance improves

## What would overturn this

- **If an input or model appears where the detection set differs.** The current
  basis is a single image. That the sample is small is acknowledged in using it
- **Re-verification is done at the detection level, not by tensor cosine.** The
  trap section above is why. Forgetting this criterion and judging by cosine
  would mean discarding a perfectly good model
