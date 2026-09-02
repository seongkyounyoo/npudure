# The NPUDure model conversion environment

*[한국어 원문](README.ko.md)*

Converts an ONNX model into a `.rknn` for RK3576.

## Prerequisites

| Item | Value |
|---|---|
| Target platform | **`rk3576`** (NanoPi R76S) |
| The boards' RKNN Runtime | 2.3.0 |
| Toolkit version | 2.3.0 (matched to the Runtime) |
| Execution environment | **x86_64 Linux only** |

**A `.rknn` converted for `rk3588` does not work on RK3576.** They are not
compatible across platforms, so reference examples or pre-converted models found
online cannot be used as-is.

If the Toolkit version is higher than the boards' Runtime, converted models may
fail to load. The boards' measured values follow
`docs/environment-matrix.md` §3.

## Why Docker

If the conversion result varies with the host environment, reproducibility
breaks. The image pins the Python, Toolkit and dependency versions, so the same
`.rknn` comes out on anyone's machine.

For an open-source release it has to be "reproduce it with this image" rather
than "it works in our environment".

## Usage

### 1. Build the image

```bash
docker build -t npuforge-converter:2.3.0 tools/model-converter
```

Including `torch`, `onnx` and the rest, the image comes to around 5–8 GB. Check
your disk space first.

### 2. Prepare the YOLOv8n ONNX

```bash
mkdir -p models datasets/calib
# put yolov8n.onnx in models/
# put 100-300 calibration images in datasets/calib/
```

INT8 quantization needs calibration images. Their distribution has to resemble
the real inference input for accuracy to hold.

### 3. Convert

```bash
docker run --rm \
  -v "$PWD/models:/work/models" \
  -v "$PWD/datasets:/work/datasets" \
  npuforge-converter:2.3.0 \
  python3 convert_yolov8n.py \
    --onnx  models/yolov8n.onnx \
    --out   models/yolov8n.rknn \
    --dataset datasets/calib \
    --calib-limit 200
```

To convert to FP16 for comparison against INT8, pass `--no-quant`.

### 4. Record the metadata

When conversion finishes, `models/yolov8n.rknn.meta.json` is generated.

```json
{
  "target_platform": "rk3576",
  "quantization": "INT8",
  "onnx_sha256": "...",
  "rknn_sha256": "...",
  "calibration_images": 200,
  "calibration_manifest_sha256": "...",
  "toolkit_version": "..."
}
```

**Transcribe these values into `docs/environment-matrix.md` §6.** Performance
figures measured with an unrecorded model are not used as official results.

### 5. Deploy to the boards

```bash
for h in npuforge-k npuforge-q npuforge-j; do
  scp models/yolov8n.rknn "$h:/tmp/"
  ssh "$h" "printf '%s\n' \"\$NPUFORGE_SUDO_PASS\" | sudo -S -p '' install -D -m644 /tmp/yolov8n.rknn /opt/npuforge/models/yolov8n/model.rknn"
  ssh "$h" 'sha256sum /opt/npuforge/models/yolov8n/model.rknn'
done
```

**All three nodes' SHA-256 have to match.** If they do not, do not proceed.

## Memory constraint

The scheduler host has 3.5 GB of RAM (`docs/environment-matrix.md` §4.2).
YOLOv8n is a small model so conversion itself is possible, but with many
calibration images the quantization step can get tight.

If it fails on memory, reduce `--calib-limit` (to 100, say). Changing the image
count changes the quantization result, so always use the value reflected in the
metadata.

## Next steps

A converted model is needed before the following are possible.

1. **Thread-safety verification** —
   `crates/npuforge-rknn/native/thread_safety_test.c`
   The biggest unknown in deciding the node's `worker_count`
2. Single-node inference accuracy verification (compared against the ONNX
   results)
3. S0 thermal characterisation
