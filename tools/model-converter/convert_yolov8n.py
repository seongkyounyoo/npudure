#!/usr/bin/env python3
"""ONNX -> RKNN 변환.

보유 장비는 RK3576 기반 NanoPi R76S 다. rk3588 로 변환한 .rknn 은
RK3576 에서 동작하지 않는다. 플랫폼 간 호환되지 않는다.

변환 결과의 재현성을 위해 다음을 모두 기록한다.

  - 입력 ONNX 의 SHA-256
  - 출력 RKNN 의 SHA-256
  - calibration 데이터 목록의 SHA-256
  - 변환 옵션 전체
  - toolkit / python 버전

출력된 메타데이터는 docs/environment-matrix.md §6 에 옮겨 적는다.

docs/03-DEVELOPMENT-REQUIREMENTS.md §2.1, §2.2
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import sys
from pathlib import Path

TARGET_PLATFORM = "rk3576"


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fp:
        for chunk in iter(lambda: fp.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def build_calibration_list(dataset_dir: Path, out_file: Path, limit: int) -> tuple[int, str]:
    """calibration.txt 를 만들고 **이식 가능한** 목록 해시를 돌려준다.

    RKNN-Toolkit2 는 이미지 경로가 한 줄에 하나씩 적힌 텍스트 파일을 받는다.
    목록의 순서가 양자화 결과에 영향을 주므로 정렬해 고정한다.

    파일에는 절대경로를 쓰지만 **해시는 데이터셋 기준 상대경로로 낸다.**
    절대경로를 해싱하면 컨테이너 안(`/work/datasets/...`)과 밖에서 값이
    달라져 "같은 데이터로 양자화했는가"를 확인할 수 없다. 이 해시는
    `fetch_calibration.py` 가 남기는 manifest 와 대조하기 위한 것이므로
    머신에 무관해야 한다.
    """
    exts = {".jpg", ".jpeg", ".png", ".bmp"}
    images = sorted(p for p in dataset_dir.rglob("*") if p.suffix.lower() in exts)
    if not images:
        raise SystemExit(f"calibration 이미지를 찾을 수 없습니다: {dataset_dir}")

    images = images[:limit]

    # RKNN 에 넘길 파일: 절대경로
    out_file.write_text(
        "\n".join(str(p.resolve()) for p in images) + "\n", encoding="utf-8"
    )

    # 기록에 남길 해시: 데이터셋 기준 상대경로 (POSIX 구분자로 통일)
    rel = "\n".join(p.relative_to(dataset_dir).as_posix() for p in images) + "\n"
    return len(images), sha256_text(rel)


def main() -> int:
    ap = argparse.ArgumentParser(description="YOLOv8n ONNX -> RKNN 변환 (RK3576)")
    ap.add_argument("--onnx", required=True, type=Path, help="입력 ONNX 파일")
    ap.add_argument("--out", required=True, type=Path, help="출력 .rknn 파일")
    ap.add_argument("--dataset", type=Path,
                    help="calibration 이미지 디렉터리. 생략하면 양자화하지 않는다")
    ap.add_argument("--calib-limit", type=int, default=200,
                    help="calibration 이미지 최대 개수 (기본 200)")
    ap.add_argument("--no-quant", action="store_true",
                    help="FP16 으로 변환한다. INT8 과 비교할 때 사용")
    ap.add_argument("--meta", type=Path,
                    help="메타데이터 JSON 출력 경로 (기본: <out>.meta.json)")
    ap.add_argument("--verbose", action="store_true")
    args = ap.parse_args()

    if not args.onnx.is_file():
        raise SystemExit(f"ONNX 파일이 없습니다: {args.onnx}")

    quantize = not args.no_quant
    if quantize and args.dataset is None:
        raise SystemExit(
            "INT8 양자화에는 --dataset 이 필요합니다. "
            "FP16 으로 변환하려면 --no-quant 를 주세요."
        )

    from rknn.api import RKNN  # 이미지 안에서만 임포트 가능

    args.out.parent.mkdir(parents=True, exist_ok=True)

    calib_file = None
    calib_count = 0
    calib_hash = None
    if quantize:
        calib_file = args.out.parent / "calibration.txt"
        calib_count, calib_hash = build_calibration_list(
            args.dataset, calib_file, args.calib_limit
        )
        print(f"[calib] {calib_count}장, manifest sha256={calib_hash[:16]}...")

    onnx_hash = sha256_file(args.onnx)
    print(f"[input] {args.onnx.name} sha256={onnx_hash[:16]}...")

    # YOLOv8 계열은 0~255 uint8 입력을 0~1 로 정규화한다.
    # mean/std 를 여기서 지정하면 정규화가 NPU 안에서 처리되어
    # CPU 전처리 비용이 줄어든다.
    config = {
        "mean_values": [[0, 0, 0]],
        "std_values": [[255, 255, 255]],
        "target_platform": TARGET_PLATFORM,
    }

    rknn = RKNN(verbose=args.verbose)
    print(f"[config] target_platform={TARGET_PLATFORM}")
    rknn.config(**config)

    print("[load] ONNX 로딩")
    if rknn.load_onnx(model=str(args.onnx)) != 0:
        raise SystemExit("load_onnx 실패")

    print(f"[build] quantization={'INT8' if quantize else 'FP16'}")
    build_kwargs = {"do_quantization": quantize}
    if quantize:
        build_kwargs["dataset"] = str(calib_file)
    if rknn.build(**build_kwargs) != 0:
        raise SystemExit("build 실패")

    print(f"[export] {args.out}")
    if rknn.export_rknn(str(args.out)) != 0:
        raise SystemExit("export_rknn 실패")

    rknn.release()

    rknn_hash = sha256_file(args.out)

    try:
        import rknn as _rknn_pkg
        toolkit_version = getattr(_rknn_pkg, "__version__", "unknown")
    except Exception:
        toolkit_version = "unknown"

    meta = {
        "target_platform": TARGET_PLATFORM,
        "quantization": "INT8" if quantize else "FP16",
        "onnx_file": args.onnx.name,
        "onnx_sha256": onnx_hash,
        "rknn_file": args.out.name,
        "rknn_sha256": rknn_hash,
        "rknn_bytes": args.out.stat().st_size,
        "calibration_images": calib_count,
        "calibration_manifest_sha256": calib_hash,
        "config": config,
        "toolkit_version": toolkit_version,
        "python_version": platform.python_version(),
        "platform": platform.platform(),
    }

    meta_path = args.meta or args.out.with_suffix(args.out.suffix + ".meta.json")
    meta_path.write_text(json.dumps(meta, indent=2, ensure_ascii=False), encoding="utf-8")

    print("\n=== 변환 완료 ===")
    for key in ("rknn_file", "rknn_sha256", "rknn_bytes", "quantization",
                "calibration_images", "toolkit_version"):
        print(f"  {key:26} {meta[key]}")
    print(f"\n메타데이터: {meta_path}")
    print("이 값을 docs/environment-matrix.md §6 에 기록하세요.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
