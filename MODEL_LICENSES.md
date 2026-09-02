# Model and dataset licenses

*[한국어 원문](MODEL_LICENSES.ko.md)*

NPUDure's source code is Apache-2.0. But **the models and datasets used for
benchmarking follow separate licenses.**

This document makes that distinction clear.

---

# 1. Summary

| Component | License | Included in the repository |
|---|---|---|
| NPUDure source code | Apache-2.0 | included |
| RKNN Runtime / Toolkit2 | Rockchip's own terms | **not included** |
| YOLOv8n weights and the derived ONNX | **AGPL-3.0** | **not included** |
| The converted `.rknn` | AGPL-3.0 (a derivative) | **not included** |
| Calibration and benchmark data | undecided | **not included** |

`.gitignore` excludes `*.rknn`, `*.onnx`, `*.pt` and `/datasets/`, so this
repository redistributes no model weights of any kind.

Users obtain the model themselves with the conversion scripts
(`tools/model-converter/`).

---

# 2. ⚠️ YOLOv8 is AGPL-3.0

## 2.1 The facts

The YOLOv8n used as the reference model is Ultralytics' work and is distributed
under **AGPL-3.0**.

The ONNX actually used is Rockchip's RKNN-optimized version.

```text
Ultralytics YOLOv8 (AGPL-3.0)
  |- airockchip/ultralytics_yolov8  (stays AGPL-3.0, output structure modified for RKNN)
       |- yolov8n.onnx  (the rknn_model_zoo distribution)
            |- yolov8n.rknn  (converted by NPUDure)   <- a derivative. AGPL-3.0 applies
```

The `airockchip/rknn_model_zoo` repository is itself Apache-2.0, but **that does
not change the license of the YOLOv8 model distributed inside it to
Apache-2.0.** A repository's license and a datum's license are separate things.

## 2.2 The effect on NPUDure

**The source code is unaffected.** NPUDure does not link YOLOv8 code. It loads a
model file through the RKNN Runtime, which is use of a work rather than
combination with it.

**Redistributing the model file is affected.** The converted `.rknn` is a
derivative of AGPL-3.0 weights. Distributing it means following AGPL-3.0's
terms.

**So the following policy applies.**

- No `.rknn`, `.onnx` or `.pt` in the repository
- Nor in the release artefacts
- Users download and convert it themselves with the scripts
- Benchmark results (numbers, JSONL, CSV) are not model files and are published
  freely

This is the approach many open-source projects take, and it does not harm
reproducibility. With the conversion script and the source hash published,
anyone can produce the same file.

## 2.3 What remains to be judged

Saying only "NPUDure is Apache-2.0" in a talk or a paper invites
misunderstanding. **The model used in the demo is AGPL-3.0**, and that is stated
alongside.

For users who have to avoid AGPL in a commercial setting, the alternative models
in §3 are documented.

---

# 3. Alternative model candidates

Candidates should the reference model change. **None is currently adopted.**

| Model | License | RKNN support | Note |
|---|---|---|---|
| **YOLOv8n** (current) | AGPL-3.0 | an official example exists | RK3576 support confirmed |
| YOLOX-nano | **Apache-2.0** | an example exists in model_zoo | a clean license |
| PP-YOLOE | **Apache-2.0** | an example exists in model_zoo | of the PaddlePaddle family |
| RTMDet-tiny | **Apache-2.0** | example to be confirmed | of the MMDetection family |
| YOLOv6 | GPL-3.0 | — | milder than AGPL but still copyleft |
| MobileNetV3 (classification) | Apache-2.0 / BSD | an example exists | not object detection, so poor for talk visuals |

## 3.1 Why YOLOv8n stays

- There is overwhelmingly more RKNN official example and reference material
- RK3576 support is confirmed in documentation
- A pre-optimized ONNX is provided, removing the risk of the export step
- Detection results can be shown intuitively on a talk screen

**What this project measures is not the model's performance but a distributed
inference runtime's scaling efficiency.** The model is a means of generating
load, so using whichever has the best tool support carries less risk.

## 3.2 When to revisit

Switching to an alternative model is considered if any of the following occurs.

- Repeated AGPL-related enquiries from commercial users
- The license being raised as a problem in paper review
- Excessive CPU fallback in YOLOv8n conversion distorting the measurements

---

# 4. RKNN software

| Component | Source | Included in the repository |
|---|---|---|
| RKNN Runtime (`librknnrt.so`) | pre-installed in the board OS image | not included |
| `rknn_api.h` and other headers | pre-installed in the board OS image | not included |
| RKNN-Toolkit2 | PyPI | not included (installed during the Docker build) |
| `rknn_model_zoo` | GitHub, Apache-2.0 | not included (cloned if needed) |

**RKNN SDK binaries are not included in the NPUDure repository.** Users install
them from the image the board manufacturer provides, or from Rockchip's official
source.

`rknn_model_zoo` is Apache-2.0 so its code can be quoted. But **the model files
it distributes carry their own original licenses** (see §2.1).

---

# 5. Datasets

## 5.1 Status: undecided

The calibration images and benchmark input data are not yet settled.

## 5.2 Selection criteria

| Criterion | Reason |
|---|---|
| Redistributable | being able to include it in the repository or a release makes reproduction easy |
| Distribution similar to the real input | INT8 quantization accuracy depends on it |
| 100–300 images | enough for calibration without excessive conversion time |
| Resolution | resized to 640×640, so anything larger suffices |

## 5.3 Candidates

| Dataset | License | Redistribution |
|---|---|---|
| COCO 2017 val | per-image Flickr terms; the annotations are CC BY 4.0 | ⚠️ the image redistribution terms are not uniform |
| Open Images | CC BY 4.0 (needs per-image confirmation) | conditional |
| Photographed ourselves | owned by the project | **possible** |
| Unsplash / Pexels | each service's license | conditional |

**Photographing it ourselves is cleanest.** Once the scene for the talk demo is
decided, using images shot in that environment for both calibration and
benchmarking is favourable on both licensing and quantization accuracy.

## 5.4 Recording obligation

Whatever data is used, the following is recorded in
`docs/environment-matrix.md` §7.

```text
dataset name / source / license / redistribution terms
image count / input format / manifest SHA-256
```

---

# 6. Items still to confirm

| Item | Status |
|---|---|
| A precise review of Ultralytics AGPL-3.0's scope | incomplete |
| Confirming the RKNN Runtime redistribution terms in the original text | incomplete |
| Settling the calibration dataset | undecided |
| Settling the benchmark input dataset | undecided |
| Generating `THIRD_PARTY_NOTICES.md` (Rust dependencies) | to be automated with `cargo deny check licenses` |

Anything requiring legal judgement is outside this document's scope. What is
recorded here is **the facts and the project's policy in response.**
