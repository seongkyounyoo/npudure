#!/usr/bin/env python3
"""정확도 검증용 기준 입력과 ONNX 기준 출력을 만든다.

RKNN 시뮬레이터는 이미 빌드된 `.rknn` 을 추론하지 못한다
(`load_rknn` 후 `init_runtime` 이 거부한다). 따라서 정확도 검증은
**실보드에서** 한다. 이 스크립트는 그 준비물을 만든다.

  1. `input.bin`  — 640x640x3 uint8 NHWC RGB. 보드의 `dump_output_test` 입력.
  2. `onnx.bin`   — 같은 이미지에 대한 ONNX 원본 출력 (보드 출력과 같은 형식)

전처리를 한 곳에서 하는 것이 중요하다. 보드와 PC 가 각자 리사이즈하면
그 차이가 양자화 손실로 잘못 계상된다. 그래서 `input.bin` 을 만들어
양쪽이 **같은 바이트**를 쓰게 한다.

RKNN 모델은 mean=0, std=255 로 빌드되어 정규화를 모델 안에서 한다.
따라서 보드 입력은 uint8 원본이고, ONNX 입력만 여기서 /255 한다.

사용법:
    python make_reference.py --image x.jpg --onnx yolov8n.onnx --out-dir ref/
"""

from __future__ import annotations

import argparse
import struct
import sys
from pathlib import Path

import numpy as np

MAGIC = 0x544E4B52  # "RKNT"


def write_tensor_blob(path: Path, tensors: list[np.ndarray], dtype_flag: int) -> None:
    """보드의 `dump_output_test` 와 같은 형식으로 쓴다."""
    with path.open("wb") as f:
        f.write(struct.pack("<IIII", MAGIC, 1, len(tensors), dtype_flag))
        for t in tensors:
            dims = list(t.shape)[:4]
            dims += [0] * (4 - len(dims))
            f.write(struct.pack("<II", t.nbytes, min(len(t.shape), 4)))
            f.write(struct.pack("<4I", *dims))
        for t in tensors:
            f.write(np.ascontiguousarray(t).tobytes())


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--image", required=True, type=Path)
    ap.add_argument("--onnx", required=True, type=Path)
    ap.add_argument("--out-dir", required=True, type=Path)
    ap.add_argument("--size", type=int, default=640)
    args = ap.parse_args()

    args.out_dir.mkdir(parents=True, exist_ok=True)

    try:
        import cv2
    except ImportError:
        print("opencv 가 필요합니다", file=sys.stderr)
        return 1

    img = cv2.imread(str(args.image))
    if img is None:
        print(f"이미지를 읽을 수 없습니다: {args.image}", file=sys.stderr)
        return 1

    # 종횡비를 유지하지 않고 단순 리사이즈한다. 정확도 비교가 목적이지
    # 검출 품질 측정이 아니므로 letterbox 는 쓰지 않는다. 양쪽이 같은
    # 바이트를 보는 것이 유일한 요구사항이다.
    img = cv2.resize(img, (args.size, args.size), interpolation=cv2.INTER_LINEAR)
    rgb = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)  # NHWC uint8

    input_path = args.out_dir / "input.bin"
    input_path.write_bytes(np.ascontiguousarray(rgb).tobytes())
    print(f"[input] {input_path} ({rgb.nbytes} bytes, {rgb.shape} uint8 NHWC RGB)")

    try:
        import onnxruntime as ort
    except ImportError:
        print("onnxruntime 이 없어 ONNX 기준 출력은 만들지 않았습니다.", file=sys.stderr)
        print("보드 모델끼리의 비교는 가능합니다.", file=sys.stderr)
        return 0

    sess = ort.InferenceSession(str(args.onnx), providers=["CPUExecutionProvider"])
    iname = sess.get_inputs()[0].name

    # ONNX 는 NCHW float32 0~1 을 받는다. RKNN 은 모델 안에서 정규화하므로
    # 여기서만 나눈다.
    x = rgb.astype(np.float32) / 255.0
    x = np.transpose(x, (2, 0, 1))[None, ...]  # NCHW

    outs = sess.run(None, {iname: x})
    onnx_path = args.out_dir / "onnx.bin"
    write_tensor_blob(onnx_path, [np.ascontiguousarray(o, dtype=np.float32) for o in outs], 1)
    print(f"[onnx ] {onnx_path} ({len(outs)}개 텐서)")
    for i, o in enumerate(outs):
        print(f"   [{i}] shape={o.shape} dtype={o.dtype}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
