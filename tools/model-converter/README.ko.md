# NPUDure 모델 변환 환경

*[English](README.md) — 영문이 정본이다.*

ONNX 모델을 RK3576 용 `.rknn` 으로 변환한다.

## 전제

| 항목 | 값 |
|---|---|
| 대상 플랫폼 | **`rk3576`** (NanoPi R76S) |
| 보드 RKNN Runtime | 2.3.0 |
| Toolkit 버전 | 2.3.0 (Runtime 과 일치시킴) |
| 실행 환경 | **x86_64 Linux 전용** |

**`rk3588` 로 변환한 `.rknn` 은 RK3576 에서 동작하지 않는다.** 플랫폼 간 호환되지 않으므로 인터넷의 참고 예제나 사전 변환 모델을 그대로 쓸 수 없다.

Toolkit 버전이 보드의 Runtime 보다 높으면 변환된 모델이 로딩되지 않을 수 있다. 보드 실측값은 `docs/environment-matrix.md` §3 을 따른다.

## 왜 Docker 인가

변환 결과가 호스트 환경에 따라 달라지면 재현성이 깨진다. 이미지가 Python·Toolkit·의존성 버전을 고정하므로 누구 PC에서 돌려도 같은 `.rknn` 이 나온다.

오픈소스 공개 시 "우리 환경에서는 됩니다"가 아니라 "이 이미지로 재현하세요"가 되어야 한다.

## 사용법

### 1. 이미지 빌드

```bash
docker build -t npuforge-converter:2.3.0 tools/model-converter
```

`torch`, `onnx` 등이 포함되어 이미지가 5~8GB 정도 된다. 디스크 여유를 먼저 확인한다.

### 2. YOLOv8n ONNX 준비

```bash
mkdir -p models datasets/calib
# yolov8n.onnx 를 models/ 에 둔다
# calibration 용 이미지 100~300장을 datasets/calib/ 에 둔다
```

INT8 양자화에는 calibration 이미지가 필요하다. 실제 추론 입력과 분포가 비슷해야 정확도가 유지된다.

### 3. 변환

```bash
docker run --rm \
  -v "$PWD/models:/work/models" \
  -v "$PWD/datasets:/work/datasets" \
  npuforge-converter:2.3.0 \
  python3 convert_yolov8n.py \
    --onnx  models/yolov8n.onnx \
    --out   models/yolov8n.rknn \
    --dataset datasets/calib \
    --calib-limit 200
```

FP16 으로 변환해 INT8 과 비교하려면 `--no-quant` 를 준다.

### 4. 메타데이터 기록

변환이 끝나면 `models/yolov8n.rknn.meta.json` 이 생성된다.

```json
{
  "target_platform": "rk3576",
  "quantization": "INT8",
  "onnx_sha256": "...",
  "rknn_sha256": "...",
  "calibration_images": 200,
  "calibration_manifest_sha256": "...",
  "toolkit_version": "..."
}
```

**이 값을 `docs/environment-matrix.md` §6 에 옮겨 적는다.** 기록되지 않은 모델로 측정한 성능 수치는 공식 결과로 사용하지 않는다.

### 5. 보드 배포

```bash
for h in npuforge-k npuforge-q npuforge-j; do
  scp models/yolov8n.rknn "$h:/tmp/"
  ssh "$h" "printf '%s\n' \"\$NPUFORGE_SUDO_PASS\" | sudo -S -p '' install -D -m644 /tmp/yolov8n.rknn /opt/npuforge/models/yolov8n/model.rknn"
  ssh "$h" 'sha256sum /opt/npuforge/models/yolov8n/model.rknn'
done
```

**세 노드의 SHA-256 이 모두 같아야 한다.** 다르면 그대로 진행하지 않는다.

## 메모리 제약

Scheduler 호스트의 RAM 이 3.5GB 다(`docs/environment-matrix.md` §4.2). YOLOv8n 은 작은 모델이라 변환 자체는 가능하지만, calibration 이미지가 많으면 양자화 단계에서 빠듯할 수 있다.

메모리 부족으로 실패하면 `--calib-limit` 를 줄인다(예: 100). 이미지 수를 바꾸면 양자화 결과가 달라지므로 반드시 메타데이터에 반영된 값을 사용한다.

## 다음 단계

변환된 모델이 있어야 다음이 가능하다.

1. **thread-safety 검증** — `crates/npuforge-rknn/native/thread_safety_test.c`
   노드의 `worker_count` 를 결정하는 최대 미지수
2. 단일 노드 추론 정확도 검증 (ONNX 결과와 비교)
3. S0 열 특성 측정
