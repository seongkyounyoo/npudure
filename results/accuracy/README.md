# Model accuracy verification — 2026-08-11

*[한국어 원문](README.ko.md)*

The "compare ONNX against RKNN results" of
`docs/03-DEVELOPMENT-REQUIREMENTS.md` §2.2.

## Why the simulator was not used

The RKNN simulator cannot infer with an already-built `.rknn`. After
`load_rknn()`, `init_runtime()` refuses:

```
RKNN model that loaded by 'load_rknn' not support inference on the simulator
```

Verification is only meaningful against the same file that gets deployed, so it
was measured on **a real board (king).**

## Method

Preprocessing was done in one place so both sides saw **the same input bytes.**
Had the board and the PC each resized, that difference would have been
miscounted as quantization loss.

1. `make_reference.py` — image → `input.bin` (640x640x3 uint8 NHWC RGB) + the
   ONNX outputs
2. `dump_output_test` (on the board) — RKNN inference on the same `input.bin`,
   dumping all 9 outputs
3. `compare_detections.py` — comparison at the detection level

Test image: COCO val2017 `000000006040.jpg`

## Results

### FP16 vs ONNX — effectively lossless

| scale | box cosine | detection cells | IoU | class |
|---|---|---|---|---|
| 80x80 | 0.999996 | 0/0 | 100% | 100% |
| 40x40 | 0.999994 | 0/0 | 100% | 100% |
| 20x20 | 0.999993 | 10/10 | 100% | 100% |

Top detection: both at scale 2, cell (10,12), class 6. Scores 0.8809 vs 0.8799.

### INT8 vs FP16 — identical detections

| scale | box cosine | detection cells | IoU | class |
|---|---|---|---|---|
| 80x80 | 0.997215 | 0/0 | 100% | 100% |
| 40x40 | 0.997196 | 0/0 | 100% | 100% |
| 20x20 | 0.997957 | 10/10 | 100% | 100% |

Top detection: same class (6), the cell moves by one from (10,12) to (11,12),
score 0.880 → 0.832 (−5.5%).

## Trap: raw tensor cosine similarity misleads on this model

Comparing tensors directly with `compare_outputs.py` drops the cosine
similarity of some tensors to 0.16 **even for FP16 vs ONNX.** It is not a
quantization problem.

Tracing the cause:

- Of YOLOv8n's 9 outputs, tensors 2/5/8 are **the sum of 80 class scores**
- RKNN's sigmoid does not output exactly 0 but has **a floor of 0.001831**
- Adding 80 of them produces a constant offset of `0.001831 × 80 = 0.1465`
- The measured floor is exactly 0.1465, matching the hypothesis

Most cells are background, so this offset dominates the cosine similarity. But
**the same value is added to every cell, so the ranking does not change and the
detections are unaffected.**

→ This model's acceptance criterion is not raw tensor cosine but **comparison at
  the detection level.** Use `compare_detections.py`.

## Trap: INT8 conversion is not byte-reproducible

The result of converting three times from the same ONNX with the same
calibration list:

```
run1  rknn=bb02f5836bfa7cbb5e135f3c
run2  rknn=baa395e31cbe354bb92fb306
run3  rknn=b1b38f0c22c2c0918abd1bbf
```

The file size is an identical 6,459,083 bytes, but 115,346 bytes (1.8%) differ.

**Yet the inference results are completely identical.** Comparing repro-1's and
repro-2's outputs gives cosine 1.000000 on all 9 tensors with a maximum absolute
error of 0.0. The difference is in file serialization and layout, not in
numerical computation.

Practical rules:

- **Convert the model once and deploy the same file to all three nodes.**
  Converting per node gives different hashes and makes "is this the same model"
  impossible to confirm.
- `model.toml`'s `sha256` guarantees **deployment integrity**, not that the file
  came from the same conversion recipe.

## Files

| File | Content |
|---|---|
| `onnx.bin` | the ONNX reference output (dealer, onnxruntime CPU) |
| `out-yolov8n-fp16.bin` | the FP16 RKNN output (king) |
| `out-yolov8n-int8.bin` | the INT8 RKNN output (king) |
| `out-repro-1.bin`, `out-repro-2.bin` | two INT8 conversions, for the reproducibility check |

Re-analyse:

```bash
python tools/model-converter/compare_detections.py \
  --ref results/accuracy/out-yolov8n-fp16.bin \
  --test results/accuracy/out-yolov8n-int8.bin
```
