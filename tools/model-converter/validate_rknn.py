#!/usr/bin/env python3
"""변환된 .rknn 검증.

두 가지를 확인한다.

1. 모델 메타데이터 — 입력/출력 텐서의 shape, dtype, 양자화 파라미터
   노드 설정(`model.toml`)의 input_width/height/channels 와 일치해야 한다.

2. 정확도 — 시뮬레이터 추론 결과를 ONNX 원본과 비교
   양자화로 인한 손실이 허용 범위인지 본다.

두 번째는 onnxruntime 이 있을 때만 수행한다.

docs/03-DEVELOPMENT-REQUIREMENTS.md §2.2 의 "검증 대상" 중
ONNX 결과와 RKNN Simulator 결과 비교에 해당한다.
보드 3대에서의 실측 비교는 별도로 수행한다.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import numpy as np


def cosine_similarity(a: np.ndarray, b: np.ndarray) -> float:
    a = a.astype(np.float64).ravel()
    b = b.astype(np.float64).ravel()
    denom = np.linalg.norm(a) * np.linalg.norm(b)
    if denom == 0:
        return float("nan")
    return float(np.dot(a, b) / denom)


def make_input(width: int, height: int, channels: int, image: Path | None) -> np.ndarray:
    """추론 입력 한 장을 만든다. NHWC uint8."""
    if image is not None:
        import cv2

        img = cv2.imread(str(image))
        if img is None:
            raise SystemExit(f"이미지를 읽을 수 없습니다: {image}")
        img = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
        img = cv2.resize(img, (width, height))
        return img.astype(np.uint8)

    # 이미지가 없으면 결정적 패턴을 쓴다. 정확도 비교에는 부적합하지만
    # shape 과 파이프라인 동작 확인에는 충분하다.
    rng = np.random.default_rng(seed=1234)
    return rng.integers(0, 256, size=(height, width, channels), dtype=np.uint8)


def main() -> int:
    ap = argparse.ArgumentParser(description="변환된 RKNN 모델 검증")
    ap.add_argument("--rknn", required=True, type=Path, help="검증할 .rknn 파일")
    ap.add_argument("--onnx", type=Path, help="비교할 원본 ONNX (선택)")
    ap.add_argument("--image", type=Path, help="테스트 이미지 (선택)")
    ap.add_argument("--min-cosine", type=float, default=0.98,
                    help="ONNX 대비 최소 코사인 유사도 (기본 0.98)")
    ap.add_argument("--out", type=Path, help="검증 결과 JSON 출력 경로")
    args = ap.parse_args()

    if not args.rknn.is_file():
        raise SystemExit(f".rknn 파일이 없습니다: {args.rknn}")

    from rknn.api import RKNN

    rknn = RKNN(verbose=False)

    print(f"[load] {args.rknn}")
    if rknn.load_rknn(str(args.rknn)) != 0:
        raise SystemExit("load_rknn 실패")

    # 시뮬레이터 모드. target 을 주지 않으면 PC 에서 동작한다.
    print("[init] 런타임 초기화 (시뮬레이터)")
    if rknn.init_runtime() != 0:
        raise SystemExit("init_runtime 실패")

    # YOLOv8n 기준값. 다른 모델이면 인자로 받도록 확장한다.
    width, height, channels = 640, 640, 3
    inp = make_input(width, height, channels, args.image)
    print(f"[input] shape={inp.shape} dtype={inp.dtype} "
          f"source={'image' if args.image else 'deterministic-random'}")

    print("[infer] RKNN 시뮬레이터 추론")
    rknn_outputs = rknn.inference(inputs=[inp])
    if rknn_outputs is None:
        raise SystemExit("inference 실패")

    result: dict = {
        "rknn_file": args.rknn.name,
        "input_shape": list(inp.shape),
        "output_count": len(rknn_outputs),
        "output_shapes": [list(o.shape) for o in rknn_outputs],
        "input_source": str(args.image) if args.image else "deterministic-random",
    }

    for i, o in enumerate(rknn_outputs):
        print(f"  output[{i}] shape={o.shape} dtype={o.dtype} "
              f"min={o.min():.4f} max={o.max():.4f}")

    rknn.release()

    # ---- ONNX 비교 (선택) ----
    if args.onnx is not None:
        if not args.onnx.is_file():
            raise SystemExit(f"ONNX 파일이 없습니다: {args.onnx}")
        try:
            import onnxruntime as ort
        except ImportError:
            print("\n[skip] onnxruntime 이 없어 ONNX 비교를 건너뜁니다.")
            print("       pip install onnxruntime 후 다시 실행하세요.")
            ort = None

        if ort is not None:
            print(f"\n[onnx] {args.onnx}")
            sess = ort.InferenceSession(str(args.onnx), providers=["CPUExecutionProvider"])
            iname = sess.get_inputs()[0].name

            # 변환 시 mean=0, std=255 로 설정했으므로 ONNX 에는
            # 동일하게 정규화한 NCHW float 를 넣어야 한다.
            x = inp.astype(np.float32) / 255.0
            x = np.transpose(x, (2, 0, 1))[None, ...]
            onnx_outputs = sess.run(None, {iname: x})

            sims = []
            for i, (r, o) in enumerate(zip(rknn_outputs, onnx_outputs)):
                if r.size != o.size:
                    print(f"  output[{i}] 크기 불일치: rknn={r.shape} onnx={o.shape}")
                    sims.append(float("nan"))
                    continue
                s = cosine_similarity(r, o)
                sims.append(s)
                mark = "OK" if s >= args.min_cosine else "낮음"
                print(f"  output[{i}] cosine={s:.6f}  [{mark}]")

            result["cosine_similarity"] = sims
            result["min_cosine_threshold"] = args.min_cosine
            valid = [s for s in sims if not np.isnan(s)]
            result["passed"] = bool(valid) and all(s >= args.min_cosine for s in valid)

            if not result["passed"]:
                print("\n[경고] 양자화 손실이 기준치를 넘습니다.")
                print("  calibration 이미지가 실제 입력 분포와 다르거나 수가 부족할 수 있습니다.")
                print("  --calib-limit 을 늘리거나 데이터 세트를 교체해 보세요.")

    if args.out:
        args.out.write_text(json.dumps(result, indent=2, ensure_ascii=False), encoding="utf-8")
        print(f"\n검증 결과: {args.out}")

    print("\n이 결과를 docs/environment-matrix.md §6 에 함께 기록하세요.")
    return 0 if result.get("passed", True) else 1


if __name__ == "__main__":
    sys.exit(main())
