# ADR-012. The node sends integers without dequantizing (`want_float=0`, blob v2)

*[한국어 원문](012-want-float-zero-blob-v2.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-12 |
| **Related** | [ADR-011](011-int8-quantization.md) (INT8 adopted), [ADR-014](014-10g-aggregation-separate-scheduler.md) (10G aggregation), [ADR-021](021-no-node-side-postprocessing.md) (node-side postprocessing not implemented), `docs/discuss.md` §12 |

---

## In one line

> If the node converts results to `float32` before sending, **the response
> becomes 3.96× the request**, and even a 10G link is not enough at three-node
> saturation. So it sends **the quantized integers as they are**, with `scale`
> and `zero_point` included in the response so the receiver can convert back.

## Context

### Quantization and dequantization

An INT8 model computes in integers. Its outputs come out as integers too, and
converting them back to reals needs two values attached to each tensor.

```text
real = (quantized - zero_point) x scale
```

RKNN has **an option to do that conversion for you**.

| | |
|---|---|
| `want_float = 1` | the runtime converts to `float32` and hands that over. Convenient |
| `want_float = 0` | gives the model's native type as-is (int8 for an INT8 model) |

The default is `1`, and that is what was used at first, because it was
convenient.

### But in this project the output goes out over the network

The node does not postprocess (no NMS). It **sends all nine raw tensors back to
the scheduler** (→ ADR-021). So the output type is the link load.

```text
input                        1,228,800 byte   (640 x 640 x 3)
output want_float=1 (f32)    4,872,000 byte   <- 3.96x the input
output want_float=0 (int8)   1,218,000 byte   <- 0.99x the input
```

The load on the scheduler-side link at three-node saturation:

| Configuration | Model | 3-node TX | 3-node RX | Fits in 10G? |
|---|---|---:|---:|---|
| `want_float=1` | INT8 | 4.64 Gbps | **18.38 Gbps** | **no** |
| `want_float=1` | FP16 | 2.49 Gbps | 9.86 Gbps | barely |
| `want_float=0` | INT8 | 4.64 Gbps | 4.60 Gbps | yes |
| `want_float=0` | FP16 | 2.49 Gbps | 2.46 Gbps | yes |

The original error was **calculating the network from the input alone and
omitting the output**. Recomputing with the output inverted the conclusion —
even laying 10G would not carry three INT8 nodes.

At that point `want_float=0` was promoted from "a nice optimization to have" to
**a precondition for starting M3**. The grounds for the promotion were not
throughput but **RX bandwidth**.

## Decision

**1. Change the default of `want_float` to `false` and expose it in node
configuration.**

```toml
[worker]
want_float = false
```

**2. Bump the response blob format to v2 and carry dequantization parameters
per tensor.**

```text
magic    u32  "RKNT"
version  u32  = 2
count    u32  number of tensors
dtype    u32  0 = model's native dtype, 1 = float32
per tensor (36 byte):
  len  n_dims  dims x 4   <- present in v1 too (24 byte)
  qnt_type  zero_point  scale   <- added in v2 (12 byte)
followed by the tensor data
```

**Why this is not optional**: send int8 without `scale` and `zero_point` and
the receiver has no way at all to interpret those bytes. The numbers arrive and
nobody knows what they mean. The moment the decision was made to send integers,
carrying the parameters became an obligation that follows from it.

**3. Old blobs are not accepted.** `decode` rejects `version != 2` as an error.
Reading a 36-byte descriptor as 24 bytes because only the header says v1
produces silently misaligned values, and that is the failure mode this project
most wants to avoid.

## Rationale

### Accuracy — matches float32 on real hardware

Since we do the dequantization ourselves, it has to be checked against the
runtime's result.

```text
measured: real board king, 9 tensors
(a) the float32 received with want_float=1
(b) the int8 received with want_float=0, dequantized by hand
```

**Maximum error 9.5e-7.** At the limit of `float32` precision, so effectively
identical. (`crates/npuforge-rknn/tests/real_device.rs`)

### Throughput — 15–17% higher as a bonus

```text
conditions: king, 8 threads, 120 s, governor=performance
```

| Model | `want_float=0` | `want_float=1` | Gain |
|---|---:|---:|---:|
| INT8 | **156.7 inf/s** | 133.6 inf/s | **+17.3%** |
| FP16 | 66.9 inf/s | 57.8 inf/s | **+15.7%** |

Dequantization is done by the CPU. Not doing that work makes it faster.

> **Why was it +5.4% before.** The first measurement on 2026-08-10 was mostly a
> single-thread condition and came out at +5.4%, which got it filed as "an
> optimization with no effect". The reason the gap widens at 8 threads is that
> **the time output conversion holds the serialized section** accumulates with
> the number of concurrent threads. Kept as a case of the same experiment
> yielding a different conclusion under different conditions.

### As it turns out, the measurement tool was on this setting all along

`sustained_load_test` had **hardcoded `want_float=0` from the beginning**. So
the **157.2 / 84.3 inf/s written into the documents as settled figures were
already on `want_float=0`**, and only the Rust backend was on `true`.

Which means this change did not raise performance; it **brought the software in
line with the measurement conditions**. Put the other way: until the change,
the actual node was running 15–17% slower than the documented figures and
nobody knew.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Keep `want_float=1` + 10G | RX computes to 18.38 Gbps. **Even 10G does not work.** 25G is outside this project's budget and purpose |
| Postprocess (NMS) on the node | **This is ultimately the right answer.** The response shrinks to a few KB and RX effectively disappears. But it is unimplemented, and putting postprocessing on the node shifts CPU load to the node and changes the measurement conditions again → ADR-021 |
| Compress the response | Compression/decompression CPU enters the inference path. CPU is already the bottleneck, and this stacks on top |
| Parameters as separate fields rather than in the blob | The values differ per tensor, so attaching them to the tensor descriptor is natural. Separating them creates room for the ordering to break |

## Consequences

**Gained**

- 3-node RX 18.38 → 4.60 Gbps. **M3 became possible on 10G aggregation**
- Throughput INT8 +17.3% / FP16 +15.7%
- The node now actually matches the measurement conditions written in the docs

**Lost / the cost**

- **Responsibility for dequantization moved to the receiver.** The client has to
  understand the blob
- Changing the format means **fixing three places together**
  - `crates/npuforge-rknn/src/blob.rs`
  - `native/dump_output_test.c` (board verification tool)
  - `tools/model-converter/compare_detections.py` (accuracy comparison)
- Incompatible with v1 blobs (intentionally)

**Known flaw**

The response's `result_format` string is still **`"rknn-tensors-v1"`**. The
actual blob header says `version = 2` and the descriptor changed from 24 to 36
bytes. A client identifying the format by that string would mistake it for v1
and read in 24-byte units.

It does not surface today because every consumer is inside this repository.
**The name and the reality disagree, so this has to be cleaned up before going
public.**

## What would overturn this

- **Implementing node-side postprocessing (NMS)** shrinks the response to a few
  KB of detections and makes the blob itself largely unnecessary. At that point
  this ADR is superseded by ADR-021
- **If the input format becomes JPEG**, input TX drops tenfold and the whole
  link budget has to be recomputed. The output-side conclusion stands regardless
- Revisit if an observation shows dequantization error changing postprocessing
  results. The current basis is a per-tensor maximum error of 9.5e-7, and
  **no comparison was made at the level of detection boxes**
