# 모델 정확도 검증 — 2026-08-11

`docs/03-DEVELOPMENT-REQUIREMENTS.md` §2.2 의 "ONNX vs RKNN 결과 비교".

## 왜 시뮬레이터를 쓰지 않았나

RKNN 시뮬레이터는 이미 빌드된 `.rknn` 을 추론하지 못한다.
`load_rknn()` 후 `init_runtime()` 이 다음과 같이 거부한다.

```
RKNN model that loaded by 'load_rknn' not support inference on the simulator
```

배포되는 것과 같은 파일로 검증해야 의미가 있으므로 **실보드(king)** 에서 측정했다.

## 방법

전처리를 한 곳에서 수행해 양쪽이 **같은 입력 바이트**를 보게 했다.
보드와 PC 가 각자 리사이즈하면 그 차이가 양자화 손실로 잘못 계상된다.

1. `make_reference.py` — 이미지 → `input.bin`(640x640x3 uint8 NHWC RGB) + ONNX 출력
2. `dump_output_test` (보드) — 같은 `input.bin` 으로 RKNN 추론, 9개 출력 전부 덤프
3. `compare_detections.py` — 검출 수준 비교

테스트 이미지: COCO val2017 `000000006040.jpg`

## 결과

### FP16 vs ONNX — 사실상 무손실

| scale | box cosine | 검출 셀 | IoU | class |
|---|---|---|---|---|
| 80x80 | 0.999996 | 0/0 | 100% | 100% |
| 40x40 | 0.999994 | 0/0 | 100% | 100% |
| 20x20 | 0.999993 | 10/10 | 100% | 100% |

최고 검출: 양쪽 모두 scale 2, cell (10,12), class 6. 점수 0.8809 vs 0.8799.

### INT8 vs FP16 — 검출 동일

| scale | box cosine | 검출 셀 | IoU | class |
|---|---|---|---|---|
| 80x80 | 0.997215 | 0/0 | 100% | 100% |
| 40x40 | 0.997196 | 0/0 | 100% | 100% |
| 20x20 | 0.997957 | 10/10 | 100% | 100% |

최고 검출: 클래스 동일(6), 셀이 (10,12)→(11,12) 로 한 칸 이동, 점수 0.880→0.832 (-5.5%).

## 함정: 원시 텐서 코사인 유사도는 이 모델에서 오해를 부른다

`compare_outputs.py` 로 텐서를 그대로 비교하면 **FP16 vs ONNX 에서도**
일부 텐서의 코사인 유사도가 0.16 까지 떨어진다. 양자화 문제가 아니다.

원인을 추적한 결과:

- YOLOv8n 출력 9개 중 텐서 2/5/8 은 **클래스 점수 80개의 합**이다
- RKNN 의 sigmoid 는 정확히 0 을 내지 않고 **하한 0.001831** 이 있다
- 80개를 더하면 `0.001831 × 80 = 0.1465` 의 상수 오프셋이 생긴다
- 실측 하한이 정확히 0.1465 로, 가설과 일치한다

배경 셀이 대부분이라 이 오프셋이 코사인 유사도를 지배한다. 그러나 **모든
셀에 같은 값이 더해지므로 순위가 바뀌지 않고 검출 결과는 그대로다.**

→ 이 모델의 수락 기준은 원시 텐서 코사인이 아니라 **검출 수준 비교**다.
   `compare_detections.py` 를 쓴다.

## 함정: INT8 변환은 바이트 재현성이 없다

같은 ONNX, 같은 calibration 목록으로 3회 변환한 결과다.

```
run1  rknn=bb02f5836bfa7cbb5e135f3c
run2  rknn=baa395e31cbe354bb92fb306
run3  rknn=b1b38f0c22c2c0918abd1bbf
```

파일 크기는 6,459,083 바이트로 같지만 115,346 바이트(1.8%)가 다르다.

**다만 추론 결과는 완전히 동일하다.** repro-1 과 repro-2 의 출력을 비교하면
9개 텐서 전부 cosine 1.000000, 최대 절대오차 0.0 이다. 즉 차이는 파일
직렬화·레이아웃에 있고 수치 계산에는 없다.

실무 규칙:

- **모델은 한 번만 변환해 같은 파일을 세 노드에 배포한다.** 노드마다
  변환하면 해시가 달라 "같은 모델인가" 확인이 불가능해진다.
- `model.toml` 의 `sha256` 은 **배포 무결성**을 보장하지 파일이 같은
  변환 레시피에서 나왔음을 보장하지 않는다.

## 파일

| 파일 | 내용 |
|---|---|
| `onnx.bin` | ONNX 원본 출력 (dealer, onnxruntime CPU) |
| `out-yolov8n-fp16.bin` | FP16 RKNN 출력 (king) |
| `out-yolov8n-int8.bin` | INT8 RKNN 출력 (king) |
| `out-repro-1.bin`, `out-repro-2.bin` | 재현성 확인용 INT8 2회 변환 |

재분석:

```bash
python tools/model-converter/compare_detections.py \
  --ref results/accuracy/out-yolov8n-fp16.bin \
  --test results/accuracy/out-yolov8n-int8.bin
```
