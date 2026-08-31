# datasets

INT8 양자화 캘리브레이션에 쓴 이미지는 **이 저장소에 포함하지 않는다.**

COCO 이미지는 Flickr 출처라 재배포 조건이 이미지마다 다르다. 조건은
[`../MODEL_LICENSES.md`](../MODEL_LICENSES.md) 를 참조한다.

캘리브레이션을 재현하려면 직접 내려받는다.

```bash
bash scripts/fetch-calib-images.sh
```

측정 결과 재현에는 필요하지 않다 — 모델은 이미 양자화된 상태로
배포되며, 벤치마크는 합성 입력을 쓴다.
