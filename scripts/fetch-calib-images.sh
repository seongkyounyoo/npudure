#!/usr/bin/env bash
# INT8 캘리브레이션용 COCO 이미지를 내려받는다.
#
# 저장소에 이미지를 담지 않는 이유: COCO 이미지는 Flickr 출처라
# 재배포 조건이 이미지마다 다르다. MODEL_LICENSES.md 참조.
#
# 사용법: bash scripts/fetch-calib-images.sh [장수]
set -euo pipefail

N="${1:-200}"
OUT="datasets/coco-calib"
SRC_URL="http://images.cocodataset.org/zips/val2017.zip"

mkdir -p "$OUT"
if [ "$(find "$OUT" -name '*.jpg' | wc -l)" -ge "$N" ]; then
    echo "이미 $N 장 이상 있다: $OUT"; exit 0
fi

echo "COCO val2017 을 내려받는다 (약 780MB)..."
echo "  라이선스: https://cocodataset.org/#termsofuse"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
curl -L --fail -o "$tmp/val2017.zip" "$SRC_URL"
unzip -q "$tmp/val2017.zip" -d "$tmp"
find "$tmp/val2017" -name '*.jpg' | sort | head -"$N" | while read -r f; do
    cp "$f" "$OUT/"
done
echo "완료: $(find "$OUT" -name '*.jpg' | wc -l) 장 → $OUT"
