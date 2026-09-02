# ADR-021. The node does no postprocessing (NMS) and returns raw tensors

*[한국어 원문](021-no-node-side-postprocessing.ko.md)*

| | |
|---|---|
| **Status** | provisional |
| **Date** | 2026-08-12 |
| **Related** | [ADR-012](012-want-float-zero-blob-v2.md), [ADR-014](014-10g-aggregation-separate-scheduler.md), [ADR-013](013-fanless-thermal-as-measurement.md) |

---

## In one line

> The node returns **the model's 9 output tensors as they are**, not detections.
> The response gets larger, and in exchange **the node's CPU load stays out of
> what is being measured.** Node-side postprocessing is ultimately right, but not
> now.

## Context

The output of a detection model like YOLOv8n is not directly usable.

```text
NPU output   9 tensors (box candidates and class scores per grid cell)
                 |  postprocessing (NMS: resolve overlapping boxes, apply thresholds)
final result "1 person, 2 cars" - a few KB
```

The question is **where that postprocessing happens.**

| | Response size | Node CPU |
|---|---|---|
| Postprocess on the node | a few KB | goes up |
| Postprocess on scheduler/client | 1.2 MB | unchanged |

## Decision

**The node does no postprocessing.** It returns the raw tensors bundled into a
single blob ([ADR-012](012-want-float-zero-blob-v2.md)).

**The status is left as "provisional".** Not because it is best, but because it
is **the right choice under current conditions.**

## Rationale

### 1. Node CPU is already the bottleneck

Throughput falls 27% over 300 seconds of sustained load, and the cause is not
the NPU but **CPU thermal throttling**. The A72 is downgraded from 2208 to
816 MHz ([ADR-013](013-fanless-thermal-as-measurement.md)).

Adding NMS on top increases CPU load further. That would destabilise the very
value this project is trying to measure.

```text
now:              measuring NPU scaling efficiency while the CPU interferes  <- already a problem
with postprocess: making it use more CPU and measuring the same value        <- worse
```

### 2. It adds another measurement variable

**NMS cost varies with the input.** An image with many detections takes longer
and one with few finishes quickly. Doing it on the node makes per-node
processing time vary with input content.

In an experiment measuring three-node scaling efficiency, that variable is noise.

### 3. It is not implemented

The simplest reason. There is no NMS implementation, and building one brings
verification (accuracy comparison) with it. Not a priority while waiting on
equipment.

### 4. The network problem was solved another way

The cost of returning raw tensors is response size. With `want_float=1` the
response was 3.96× the request and even 10G was insufficient.

That was **solved with `want_float=0` rather than postprocessing.** The response
became a quarter of its size and 3-node RX went from 18.38 to 4.60 Gbps. It fits
inside 10G.

So **there is no immediate pressure to postprocess.**

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Run NMS on the node | **This is ultimately the right answer.** But it worsens the CPU bottleneck and adds a measurement variable. Unimplemented |
| NMS on the scheduler | The scheduler is a single point, so three nodes' worth of postprocessing piles up in one place. The scheduler becomes the bottleneck |
| Compress the response | Compression CPU enters the inference path. CPU is already the bottleneck |
| Client-side postprocessing | This is the current approach. The bench tool and comparison scripts understand the blob |

## Consequences

**Gained**

- What the node does is narrow and uniform: **preprocess → NPU → serialize**
- Per-node processing time depends less on input content
- What is being measured stays clean

**Lost / the cost**

- The response is 1.2 MB, where sending only the detections would end at a few KB
- **The receiver has to understand the blob.** Changing the format means fixing
  three places together (blob.rs / dump_output_test.c / compare_detections.py)
- As a real-world API it is unfriendly. It is not an API that "gives you
  detections"

**New constraints introduced**

- The client is responsible for both dequantization and NMS
- The network budget is tied to response size. In experiments that increase
  input size (S6), the response grows with it

## What would overturn this

**This ADR is scheduled to be overturned.**

- **If the CPU bottleneck is resolved** (cooling condition B, or preprocessing
  optimization), there is room to move postprocessing to the node
- **If a real-world API becomes a requirement**, returning raw tensors is hard to
  sustain
- **If the network fills up again in experiments that increase input size**,
  postprocessing becomes the most effective means — the response shrinks to a
  few KB and RX effectively disappears

**What must be measured alongside** when overturning it: sustained throughput
and the timing of CPU clock downgrade, before and after putting postprocessing
on the node. Do not judge from response size alone.
