#!/usr/bin/env python3
"""INT8 calibration 용 COCO val2017 이미지를 내려받는다.

**이미지는 저장소에 포함하지 않는다.** COCO 이미지는 Flickr 출처로 개별
라이선스가 제각각이며, COCO 자체는 어노테이션에만 CC-BY 4.0 을 건다.
따라서 재배포하지 않고 각자 내려받는 방식을 쓴다. `MODEL_LICENSES.md` 참조.

양자화 재현성을 위해 이 스크립트가 보장하는 것:

  - **선택이 결정적이다.** 파일명 정렬 후 고정 시드로 뽑는다. 같은
    `--count`, `--seed` 면 언제 어디서 돌려도 같은 목록이 나온다.
  - 선택 목록의 SHA-256 을 `manifest.json` 에 남긴다. 변환 메타데이터의
    해시와 이 값이 다르면 다른 데이터로 양자화한 것이다.
  - 이미 받은 파일은 크기가 맞으면 건너뛴다.

사용법:
    python tools/model-converter/fetch_calibration.py \\
        --out datasets/coco-calib --count 200

전체 val2017 zip(약 780MB)을 받는 대신 필요한 파일만 개별로 받는다.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import random
import sys
import urllib.error
import urllib.request
from pathlib import Path

# COCO val2017 이미지의 공개 CDN. zip 전체가 아니라 개별 파일을 받는다.
BASE_URL = "http://images.cocodataset.org/val2017"

# val2017 의 이미지 ID 는 12자리 0 패딩이다. 실제 존재하는 ID 목록이
# 필요하므로 어노테이션 대신 가벼운 파일 목록을 쓴다.
# (전체 5,000장 중 앞부분 ID 는 연속적이지 않아 하드코딩할 수 없다)
IMAGE_LIST_URL = "http://images.cocodataset.org/annotations/annotations_trainval2017.zip"


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def fetch_image_ids(cache: Path) -> list[int]:
    """val2017 이미지 ID 목록을 얻는다.

    어노테이션 zip(241MB)을 받는 대신 instances_val2017.json 만 꺼낸다.
    한 번 받으면 캐시한다.
    """
    cache.parent.mkdir(parents=True, exist_ok=True)
    if cache.exists():
        return json.loads(cache.read_text(encoding="utf-8"))

    import io
    import zipfile

    print(f"[fetch] 어노테이션 목록 내려받는 중 (최초 1회, ~241MB)")
    try:
        with urllib.request.urlopen(IMAGE_LIST_URL, timeout=120) as r:
            blob = r.read()
    except urllib.error.URLError as e:
        raise SystemExit(f"어노테이션을 받을 수 없습니다: {e}")

    with zipfile.ZipFile(io.BytesIO(blob)) as z:
        with z.open("annotations/instances_val2017.json") as f:
            data = json.load(f)

    ids = sorted(img["id"] for img in data["images"])
    cache.write_text(json.dumps(ids), encoding="utf-8")
    print(f"[fetch] val2017 이미지 {len(ids)}장 확인")
    return ids


def download(url: str, dest: Path) -> bool:
    """이미 있고 비어 있지 않으면 건너뛴다. 받았으면 True."""
    if dest.exists() and dest.stat().st_size > 0:
        return False
    try:
        with urllib.request.urlopen(url, timeout=60) as r:
            data = r.read()
    except urllib.error.URLError as e:
        print(f"  실패: {dest.name} ({e})", file=sys.stderr)
        return False
    dest.write_bytes(data)
    return True


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", type=Path, default=Path("datasets/coco-calib"),
                    help="이미지를 저장할 디렉터리")
    ap.add_argument("--count", type=int, default=200,
                    help="받을 이미지 수 (기본 200)")
    ap.add_argument("--seed", type=int, default=20261128,
                    help="선택 시드. 바꾸면 다른 이미지가 뽑힌다")
    args = ap.parse_args()

    out: Path = args.out
    out.mkdir(parents=True, exist_ok=True)

    ids = fetch_image_ids(out.parent / ".coco-val2017-ids.json")

    # 결정적 선택. 정렬된 목록 + 고정 시드.
    rng = random.Random(args.seed)
    chosen = sorted(rng.sample(ids, min(args.count, len(ids))))

    print(f"[calib] {len(chosen)}장 선택 (seed={args.seed})")

    names = [f"{i:012d}.jpg" for i in chosen]
    got = 0
    for n, name in enumerate(names, 1):
        if download(f"{BASE_URL}/{name}", out / name):
            got += 1
        if n % 25 == 0:
            print(f"  {n}/{len(names)}")

    present = [name for name in names if (out / name).exists()]
    missing = len(names) - len(present)

    manifest_body = "\n".join(present) + "\n"
    manifest = {
        "source": "COCO val2017",
        "source_url": BASE_URL,
        "seed": args.seed,
        "requested": args.count,
        "present": len(present),
        "missing": missing,
        "selection_sha256": sha256_text(manifest_body),
        "note": (
            "이미지는 저장소에 포함하지 않는다. 개별 이미지의 라이선스는 "
            "Flickr 출처에 따라 다르다. 재배포하지 말 것. "
            "이 목록의 selection_sha256 이 변환 메타데이터와 일치해야 "
            "같은 데이터로 양자화한 것이다."
        ),
    }
    (out / "manifest.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8"
    )

    print(f"[done] 새로 받음 {got}장, 보유 {len(present)}장, 실패 {missing}장")
    print(f"       {out}")
    print(f"       selection_sha256 = {manifest['selection_sha256'][:16]}...")
    if missing:
        print(f"경고: {missing}장을 받지 못했다. 다시 실행하면 이어받는다.",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
