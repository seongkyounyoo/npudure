#!/usr/bin/env python3
"""텐서 덤프 두 개를 비교한다.

`dump_output_test`(보드)와 `make_reference.py`(ONNX)가 같은 형식으로 쓴다.

비교 지표
  - 코사인 유사도: 방향이 얼마나 같은가. 검출 결과의 순위에 직결된다.
  - 최대 절대오차 / RMSE: 값이 얼마나 벌어졌는가.

**텐서마다 따로 본다.** 전부 이어붙여 하나로 계산하면 크기가 큰 텐서가
지표를 지배해서, 작은 텐서가 완전히 망가져도 드러나지 않는다.

사용법:
    python compare_outputs.py --ref onnx.bin --test int8.bin [--min-cosine 0.98]
"""

from __future__ import annotations

import argparse
import struct
import sys
from pathlib import Path

import numpy as np

MAGIC = 0x544E4B52


def read_blob(path: Path) -> tuple[list[np.ndarray], int]:
    data = path.read_bytes()
    if len(data) < 16:
        raise SystemExit(f"{path}: 너무 짧다")
    magic, version, count, dtype_flag = struct.unpack_from("<IIII", data, 0)
    if magic != MAGIC:
        raise SystemExit(f"{path}: magic 불일치 (0x{magic:08x})")
    if version != 1:
        raise SystemExit(f"{path}: 알 수 없는 버전 {version}")

    off = 16
    metas = []
    for _ in range(count):
        length, n_dims = struct.unpack_from("<II", data, off)
        dims = struct.unpack_from("<4I", data, off + 8)
        metas.append((length, n_dims, dims[:n_dims]))
        off += 24

    tensors = []
    for length, n_dims, dims in metas:
        raw = data[off:off + length]
        if len(raw) != length:
            raise SystemExit(f"{path}: 텐서 데이터가 잘렸다")
        arr = np.frombuffer(raw, dtype=np.float32 if dtype_flag else np.int8)
        if dims and int(np.prod(dims)) == arr.size:
            arr = arr.reshape(dims)
        tensors.append(arr)
        off += length
    return tensors, dtype_flag


def cosine(a: np.ndarray, b: np.ndarray) -> float:
    a = a.astype(np.float64).ravel()
    b = b.astype(np.float64).ravel()
    d = np.linalg.norm(a) * np.linalg.norm(b)
    return float("nan") if d == 0 else float(np.dot(a, b) / d)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--ref", required=True, type=Path, help="기준 (보통 ONNX 또는 FP16)")
    ap.add_argument("--test", required=True, type=Path, help="비교 대상")
    ap.add_argument("--min-cosine", type=float, default=0.98)
    args = ap.parse_args()

    ref, ref_dt = read_blob(args.ref)
    test, test_dt = read_blob(args.test)

    if len(ref) != len(test):
        print(f"텐서 개수가 다르다: ref {len(ref)}, test {len(test)}", file=sys.stderr)
        return 1
    if ref_dt != test_dt:
        print(f"경고: dtype flag 가 다르다 (ref={ref_dt}, test={test_dt}). "
              f"want_float 설정을 맞추세요.", file=sys.stderr)

    print(f"{'idx':>3} {'elems':>9} {'cosine':>10} {'max_abs':>12} {'rmse':>12}")
    worst = 1.0
    failed = []
    for i, (a, b) in enumerate(zip(ref, test)):
        if a.size != b.size:
            print(f"{i:>3} 크기 불일치: {a.size} vs {b.size}")
            failed.append(i)
            continue
        c = cosine(a, b)
        af = a.astype(np.float64).ravel()
        bf = b.astype(np.float64).ravel()
        max_abs = float(np.max(np.abs(af - bf)))
        rmse = float(np.sqrt(np.mean((af - bf) ** 2)))
        print(f"{i:>3} {a.size:>9} {c:>10.6f} {max_abs:>12.6f} {rmse:>12.6f}")
        worst = min(worst, c)
        if c < args.min_cosine:
            failed.append(i)

    print()
    print(f"최저 코사인 유사도: {worst:.6f}  (기준 {args.min_cosine})")
    if failed:
        print(f"기준 미달 텐서: {failed}")
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
