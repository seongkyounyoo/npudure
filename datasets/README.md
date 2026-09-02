# datasets

*[한국어 원문](README.ko.md)*

The images used for INT8 quantization calibration are **not included in this
repository.**

COCO images come from Flickr, so redistribution terms differ per image. See
[`../MODEL_LICENSES.md`](../MODEL_LICENSES.md) for the terms.

To reproduce the calibration, download them yourself.

```bash
bash scripts/fetch-calib-images.sh
```

They are not needed to reproduce the measurement results — the model is
distributed already quantized, and the benchmark uses synthetic input.
