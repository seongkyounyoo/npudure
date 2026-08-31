#!/usr/bin/env python3
"""검출 수준으로 두 모델을 비교한다.

**왜 코사인 유사도만으로는 부족한가.**

YOLOv8n(rknn_model_zoo 형식)의 출력 9개 중 3개는 "클래스 점수 80개의 합"
이다. RKNN 의 sigmoid 는 정확히 0 을 내지 않고 하한(약 0.0018)이 있어서,
이 합 텐서에는 80배 증폭된 상수 오프셋(약 0.1465)이 생긴다. 배경 셀이
대부분이라 이 오프셋이 코사인 유사도를 지배해 0.15 까지 떨어뜨린다.

그런데 검출 결과는 멀쩡하다. 모든 셀에 같은 값이 더해지면 순위가 바뀌지
않기 때문이다. 즉 **코사인 유사도 0.15 를 보고 "양자화가 망가졌다"고
결론내면 틀린다.**

그래서 실제로 중요한 것을 본다.

  1. 클래스 점수 상위 K 셀이 같은가 (검출 위치)
  2. 그 셀의 argmax 클래스가 같은가 (무엇으로 분류하는가)
  3. 박스 회귀 텐서가 얼마나 일치하는가 (박스 좌표)

사용법:
    python compare_detections.py --ref fp16.bin --test int8.bin
"""

from __future__ import annotations

import argparse
import struct
import sys
from pathlib import Path

import numpy as np

MAGIC = 0x544E4B52

# rknn_model_zoo YOLOv8 출력 배치. 스케일마다 (박스, 클래스, 합) 3개씩.
SCALES = [(0, 1, 2), (3, 4, 5), (6, 7, 8)]


def read_blob(path: Path) -> list[np.ndarray]:
    d = path.read_bytes()
    magic, version, count, dtype_flag = struct.unpack_from("<IIII", d, 0)
    if magic != MAGIC:
        raise SystemExit(f"{path}: magic 불일치")
    off = 16
    metas = []
    for _ in range(count):
        ln, nd = struct.unpack_from("<II", d, off)
        dims = struct.unpack_from("<4I", d, off + 8)
        metas.append((ln, dims[:nd]))
        off += 24
    out = []
    for ln, dims in metas:
        a = np.frombuffer(d[off:off + ln], dtype=np.float32 if dtype_flag else np.int8)
        if dims and int(np.prod(dims)) == a.size:
            a = a.reshape(dims)
        out.append(a)
        off += ln
    return out


def cosine(a, b) -> float:
    a = a.astype(np.float64).ravel()
    b = b.astype(np.float64).ravel()
    d = np.linalg.norm(a) * np.linalg.norm(b)
    return float("nan") if d == 0 else float(np.dot(a, b) / d)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--ref", required=True, type=Path)
    ap.add_argument("--test", required=True, type=Path)
    ap.add_argument("--topk", type=int, default=20, help="스케일별 상위 셀 수")
    ap.add_argument("--conf", type=float, default=0.25,
                    help="검출로 간주할 신뢰도 임계값. YOLO 기본값 0.25")
    ap.add_argument("--min-overlap", type=float, default=0.9,
                    help="임계값 이상 셀의 일치율 하한")
    ap.add_argument("--min-box-cosine", type=float, default=0.99)
    args = ap.parse_args()

    ref = read_blob(args.ref)
    test = read_blob(args.test)
    if len(ref) != len(test) or len(ref) != 9:
        print(f"YOLOv8n 9출력 형식이 아니다: ref {len(ref)}, test {len(test)}",
              file=sys.stderr)
        return 1

    ok = True

    print(f"스케일별 비교 (검출 임계값 conf >= {args.conf})")
    print(f"{'scale':>6} {'grid':>9} {'box_cos':>9} {'topK':>9} "
          f"{'topK_cls':>9} {'score_d':>10} {'det r/t':>11} {'det_IoU':>8} {'det_cls':>8}")

    for si, (bi, ci, _) in enumerate(SCALES):
        rb, tb = ref[bi], test[bi]
        rc, tc = ref[ci], test[ci]

        box_cos = cosine(rb, tb)

        # (1, C, H, W) 에서 셀별 최대 클래스 점수와 argmax
        r_max = rc[0].max(axis=0).ravel()
        t_max = tc[0].max(axis=0).ravel()
        r_arg = rc[0].argmax(axis=0).ravel()
        t_arg = tc[0].argmax(axis=0).ravel()

        k = min(args.topk, r_max.size)
        r_top = set(np.argsort(-r_max)[:k].tolist())
        t_top = set(np.argsort(-t_max)[:k].tolist())
        overlap = len(r_top & t_top) / k

        # 겹치는 셀에서 클래스가 같은가
        common = sorted(r_top & t_top)
        cls_match = (
            float(np.mean([r_arg[i] == t_arg[i] for i in common])) if common else 0.0
        )
        score_diff = float(np.max(np.abs(r_max - t_max)))

        # 신뢰도 임계값 이상인 셀만. 이것이 실제로 검출로 나가는 집합이다.
        # 상위-K 는 대부분 저신뢰 배경 셀이라 순서가 노이즈로 뒤바뀌는데,
        # 그 흔들림은 후처리에서 어차피 걸러진다.
        r_det = set(np.flatnonzero(r_max >= args.conf).tolist())
        t_det = set(np.flatnonzero(t_max >= args.conf).tolist())
        union = r_det | t_det
        det_iou = len(r_det & t_det) / len(union) if union else 1.0
        det_cls = (
            float(np.mean([r_arg[i] == t_arg[i] for i in sorted(r_det & t_det)]))
            if (r_det & t_det) else 1.0
        )

        grid = f"{rc.shape[-2]}x{rc.shape[-1]}"
        print(f"{si:>6} {grid:>9} {box_cos:>9.6f} {overlap:>10.1%} "
              f"{cls_match:>10.1%} {score_diff:>12.6f} "
              f"{len(r_det):>5}/{len(t_det):<5} {det_iou:>8.1%} {det_cls:>8.1%}")

        # 판정은 검출 집합 기준으로 한다. 상위-K 는 참고용이다.
        if box_cos < args.min_box_cosine or det_iou < args.min_overlap:
            ok = False

    # 전체 최고 점수 셀 — 가장 확실한 검출 하나가 일치하는가
    print()
    for name, blob in (("ref", ref), ("test", test)):
        best = None
        for si, (_, ci, _) in enumerate(SCALES):
            c = blob[ci][0]
            m = c.max(axis=0)
            idx = np.unravel_index(np.argmax(m), m.shape)
            score = float(m[idx])
            cls = int(c[:, idx[0], idx[1]].argmax())
            if best is None or score > best[0]:
                best = (score, si, idx, cls)
        print(f"  {name:<5} 최고점수={best[0]:.6f} scale={best[1]} "
              f"cell={best[2]} class={best[3]}")

    print()
    print("PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
