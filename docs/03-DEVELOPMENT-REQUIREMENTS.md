# NPUDure Development Requirements

- 문서명: `03-DEVELOPMENT-REQUIREMENTS.md`
- 프로젝트명: NPUDure
- 문서 버전: v0.2
- 대상 릴리스: NPUDure v0.1
- 목표 발표: 2026년 11월 FOSS for All Conference
- 작성일: 2026-08-05
- 최종 수정: 2026-08-06
- 상태: Draft
- 관련 문서:
  - `00-PRD.md`
  - `01-TECHSPEC.md`
  - `02-HARDWARE-SETUP.md`
  - `environment-matrix.md`

본 문서는 개발환경, 도구, 배포 자동화, 라이선스에 대한 규범 문서다. 버전 조합의 실제 고정값은 `environment-matrix.md`에 기록한다.

---

# 1. 문서 목적

본 문서는 NPUDure v0.1 개발을 위해 추가로 필요한 소프트웨어, 개발환경, 계측 도구, 자동화, 오픈소스 공개 준비 및 발표용 구성 요소를 정의한다.

현재 보유한 NanoPi R76S 3대와 별도 Linux PC를 기준으로 하며, 하드웨어 자체보다 다음 항목을 우선한다.

- RKNN 모델 변환 환경
- Rust 및 C 크로스컴파일
- RKNN C Wrapper
- Mock Backend
- 벤치마크 도구
- 메트릭 및 프로파일링
- 자동 배포
- 라이선스 검토
- 발표 데모 안정화

---

# 2. 반드시 필요한 개발 요소

## 2.1 RKNN 모델 변환 환경

NanoPi에서 모델을 직접 변환하지 않는다.

개발 PC에서 ONNX 또는 PyTorch 모델을 RKNN 형식으로 변환한 뒤 세 노드에 동일하게 배포한다.

```text
PyTorch / ONNX
      ↓
RKNN-Toolkit2
      ↓
model.rknn
      ↓
KING / QUEEN / JACK
```

권장 구성:

- Ubuntu x86_64 개발환경
- Python 가상환경
- RKNN-Toolkit2
- ONNX Runtime
- 모델 변환 스크립트
- 양자화용 Calibration 이미지
- 변환 결과 검증 스크립트
- 가능하면 Docker 기반 변환 환경

권장 디렉터리:

```text
tools/model-converter/
├── requirements.txt
├── convert_yolov8n.py
├── calibration.txt
├── validate_onnx.py
├── validate_rknn.py
└── Dockerfile
```

필수 관리 항목:

```text
RKNN-Toolkit2 버전
RKNN Runtime 버전
RKNPU Driver 버전
Python 버전
ONNX 모델 SHA-256
RKNN 모델 SHA-256
Calibration Dataset 해시
변환 옵션
양자화 방식
```

위 항목의 실제 고정값은 `environment-matrix.md`에 기록한다. 코드나 git 이력에서 유도할 수 없는 값이므로 별도 문서로 관리하는 유일한 예외다.

### 변환 타깃 플랫폼

보유 장비는 **RK3576** 기반 NanoPi R76S다.

```python
rknn.config(target_platform='rk3576')
```

`rk3588`로 변환한 `.rknn` 파일은 RK3576에서 동작하지 않는다. **플랫폼 간 호환되지 않으므로** 참고 예제나 기존 모델을 그대로 쓰지 않도록 주의한다.

RKNN-Toolkit2가 RK3576을 지원하는 최소 버전을 확인해 `environment-matrix.md` §3에 기록한다. 지원 버전보다 낮으면 변환 자체가 되지 않는다.

---

## 2.2 기준 모델

v0.1에서는 모델 한 개만 기준 모델로 사용한다.

권장 모델:

```text
Model       : YOLOv8n INT8
Input       : 640 × 640 RGB
Purpose     : Object Detection
Dataset     : 공개 이미지 100~500장
Model Hash  : SHA-256
Dataset Hash: SHA-256 Manifest
```

YOLOv8n을 우선하는 이유:

- 발표 화면에서 결과를 직관적으로 보여줄 수 있다.
- 입력 크기를 고정하기 쉽다.
- 동시 요청 처리 성능을 비교하기 쉽다.
- RK3576 및 RKNN 관련 예제와 참고자료가 상대적으로 많다.
- 단일 노드와 3노드 결과의 정확성을 비교하기 쉽다.

검증 대상:

```text
ONNX 결과
RKNN Simulator 또는 Toolkit 결과
KING 결과
QUEEN 결과
JACK 결과
```

동일 입력에 대해 탐지 결과가 허용 오차 내에서 일치해야 한다.

v0.1 발표 전에는 여러 모델 지원을 추가하지 않는다.

---

## 2.3 Rust ARM64 빌드 환경

Scheduler와 Node Agent의 빌드 타깃이 다르다.

```text
Scheduler:
  x86_64-unknown-linux-gnu

Node Agent:
  aarch64-unknown-linux-gnu
```

Ubuntu 빌드 PC 패키지 예시:

```bash
sudo apt install -y \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    pkg-config \
    cmake \
    protobuf-compiler

rustup target add aarch64-unknown-linux-gnu
```

`.cargo/config.toml` 예시:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

초기 개발 권장 순서:

1. R76S에서 Rust 네이티브 빌드 성공
2. RKNN C Wrapper와 연결 성공
3. 동일 소스를 ARM64 로 빌드 (크로스컴파일 또는 보드 네이티브 — §5.3)
4. 세 노드에 동일 바이너리 배포
5. 바이너리 SHA-256 확인

주의사항:

- 보드 OS의 glibc 버전 확인
- `librknnrt.so` 위치 확인
- 동적 링커 경로 확인
- `LD_LIBRARY_PATH` 의존 최소화
- 배포판별 OpenSSL 의존 제거 또는 통일
- 가능하면 정적 링크 가능한 의존성 사용

---

## 2.4 RKNN C Wrapper

Rust에서 RKNN C API를 직접 광범위하게 호출하지 않는다.

최소 C Wrapper를 만들고 Rust에서는 안전한 래퍼만 사용한다.

```text
Rust Application
      ↓
Safe Rust Wrapper
      ↓
Rust FFI Module
      ↓
Minimal C Wrapper
      ↓
librknnrt.so
```

최소 함수 예시:

```c
npf_rknn_create()
npf_rknn_destroy()
npf_rknn_get_model_info()
npf_rknn_infer()
npf_rknn_release_output()
npf_rknn_get_runtime_version()
```

구현 원칙:

- `unsafe` 코드는 `npuforge-rknn` 내부로 제한
- RKNN Context는 RAII로 관리
- 입력과 출력 버퍼 수명을 명시적으로 관리
- Raw Pointer는 외부 크레이트에 노출하지 않음
- C Wrapper 단독 테스트 프로그램 유지
- ~~Runtime 동시 호출 가능 여부 검증~~ → **완료.** 개별 호출은 thread-safe 이나
  시퀀스는 원자적이지 않다. 컨텍스트 풀 필수 (`environment-matrix.md` §3.1)
- Thread-safe가 아니면 모델당 전용 Worker Thread 사용
- FFI 오류를 NPUDure 오류 코드로 변환

필수 테스트:

```text
모델 로딩
1회 추론
1,000회 반복 추론
입력 오류
모델 파일 오류
출력 버퍼 해제
프로세스 종료 시 Context 정리
다중 Thread 호출
```

---

# 3. 성능 계측에 필요한 요소

## 3.1 Benchmark Client

단순한 `curl` 반복으로는 발표 및 논문 수준의 데이터가 나오지 않는다.

`npuforge-bench`는 다음을 지원해야 한다.

```text
동시성 1 / 4 / 16 / 64
고정 시간 실행
고정 요청 수 실행
Warmup 구간 제외
입력 데이터 순환
입력 Shuffle
JSONL 원본 저장
CSV 요약 저장
p50 / p95 / p99
오류율
재시도율
노드별 요청 분배 비율
Scheduler 정책 기록
```

실행 예시:

```bash
npuforge-bench \
  --model yolov8n \
  --dataset ./datasets/coco-sample \
  --concurrency 16 \
  --duration 300 \
  --scheduler ect \
  --output ./benchmarks/results
```

`--scheduler`가 받는 값은 `round-robin`, `least-queue`, `ect` 세 개다. `01-TECHSPEC.md` §10.0에서 정의한 식별자와 동일해야 한다.

요청 단위 원본 결과 예시:

```json
{
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "node_id": "queen",
  "scheduler_policy": "ect",
  "queue_us": 321,
  "network_us": 842,
  "preprocess_us": 1600,
  "inference_us": 18200,
  "postprocess_us": 900,
  "end_to_end_us": 22451,
  "success": true
}
```

원본 데이터는 요약 결과와 별도로 보관한다.

---

## 3.2 프로파일링 도구

각 노드 및 Scheduler PC에 다음 도구를 준비한다.

```text
perf
pidstat
sar
vmstat
iostat
ethtool
iperf3
strace
bpftrace
FlameGraph
```

초기 필수 도구:

```text
perf
pidstat
iperf3
ethtool
```

분석 항목:

- NPU 추론시간
- 이미지 디코딩시간
- 전처리시간
- 후처리시간
- 메모리 할당량
- System Call 수
- Context Switch
- 네트워크 대역폭
- CPU 사용률
- Scheduler CPU 병목
- 온도와 Thermal Throttling
- 모델별 Runtime 편차
- 노드별 처리량 편차

권장 프로파일링 순서:

1. 단일 노드 추론시간 확인
2. 전처리와 후처리 분리 측정
3. Scheduler CPU Profile
4. Node CPU Profile
5. 네트워크 대역폭 측정
6. System Call 및 Context Switch 확인
7. 버퍼 할당과 복사 구간 확인
8. io_uring 적용 필요 여부 결정

---

## 3.3 메트릭 수집

구성:

```text
npuforge-scheduler → /metrics
npuforge-node      → /metrics
Prometheus
NPUDure Dashboard
```

최소 메트릭:

```text
requests_total
requests_in_flight
request_latency_seconds
scheduler_queue_seconds
scheduler_route_seconds
inference_seconds
preprocess_seconds
postprocess_seconds
queue_depth
node_temperature_celsius
node_cpu_percent
node_memory_percent
node_npu_percent
node_network_rx_bytes_total
node_network_tx_bytes_total
request_failures_total
request_retries_total
```

Prometheus는 원본 시계열 데이터 수집기로 사용한다.

발표 화면은 NPUDure 자체 Dashboard를 우선한다.

Grafana는 선택 사항이다.

---

## 3.4 전력 측정

개발 초기에는 필수가 아니지만, 논문과 산업 비교를 위해 최종 벤치마크 전에는 필요하다.

권장 장비:

- USB-C 전력 측정기 3개
- 또는 노드별 반복 측정이 가능한 전력계
- Scheduler와 스위치 전력은 별도 기록

측정 지표:

```text
Idle Power
Average Load Power
Peak Power
FPS per Watt
Requests per Watt-hour
Cost per FPS
```

전력 측정 시 기록할 조건:

```text
전원 어댑터 모델
케이블 길이
주변 온도
노드 온도 (측정 시작 시점과 종료 시점)
입력 전압
측정 장비 모델
측정 간격
```

팬리스 구성이므로 팬 소비전력 항목은 없다. 대신 **온도에 따라 소비전력이 달라진다.** thermal throttling이 걸리면 주파수가 낮아져 전력도 함께 떨어지므로, 같은 워크로드라도 측정 시점에 따라 값이 달라진다. 시작·종료 온도를 반드시 함께 기록한다.

`02-HARDWARE-SETUP.md` §9 참조.

---

# 4. 개발 안정성을 위한 요소

## 4.1 Mock Backend

실제 NanoPi 3대가 항상 켜져 있어야 개발 가능한 구조를 피한다.

```text
InferenceBackend
├── RKNN Backend
└── Mock Backend
```

Mock Backend 설정 항목:

```text
기본 추론시간
추론시간 편차
오류율
응답 지연
Queue 제한
노드 장애
느린 노드
온도 상승
Timeout
```

예시 설정:

```toml
[backend]
type = "mock"
base_latency_ms = 20
jitter_ms = 5
error_rate = 0.02

[worker]
worker_count = 1
max_queue_depth = 32
```

`max_queue_depth`는 백엔드 종류와 무관한 노드 실행 설정이므로 `[worker]` 아래에 둔다. `[backend]`에는 백엔드 고유 항목만 기술한다. 전체 스키마는 `01-TECHSPEC.md` §16.2를 따른다.

Mock Backend로 검증할 항목:

- Round Robin
- Least Queue
- Estimated Completion Time
- 장애 노드 제외
- 자동 복구
- 재시도
- Deadline
- Queue Saturation
- Dashboard
- CI 통합 테스트

---

## 4.2 CI와 자동 테스트

CI 항목:

```text
cargo fmt --check
cargo clippy
cargo test
Mock 3-node Integration Test
Protocol Code Generation
x86_64 Build
aarch64 Cross Build
cargo audit
License Check
```

권장 GitHub Actions Workflow:

```text
.github/workflows/
├── ci.yml
├── build-arm64.yml
├── security.yml
└── release.yml
```

하드웨어 테스트 방식:

- 초기: 수동 테스트
- 중기: R76S 한 대를 Self-hosted Runner로 사용 가능
- 릴리스 전: 3노드 전체 테스트
- 야간 반복 테스트는 선택사항

11월 발표 전에는 일반 CI와 수동 하드웨어 테스트 조합으로 충분하다.

---

## 4.3 배포 자동화

세 노드에 수동으로 바이너리를 복사하면 버전 불일치가 쉽게 발생한다.

필수 스크립트:

```text
scripts/
├── build-arm64.sh
├── deploy-all.sh
├── start-all.sh
├── stop-all.sh
├── restart-all.sh
├── status-all.sh
├── collect-logs.sh
├── check-versions.sh
├── check-model-hashes.sh
└── run-benchmark.sh
```

`deploy-all.sh` 처리 순서:

```text
1. ARM64 바이너리 빌드
2. SHA-256 생성
3. 세 노드에 동일 바이너리 복사
4. 설정 파일 복사
5. 모델 해시 확인
6. systemd restart
7. Agent 버전 확인
8. Runtime 버전 확인
9. Health 상태 확인
```

노드가 3대뿐이므로 초기에는 Bash와 SSH로 충분하다.

Ansible은 반복 작업이 늘어난 뒤 검토한다.

---

## 4.4 직렬 콘솔 및 복구 도구

개발 중 다음 상황에 대비한다.

- 부팅 실패
- 네트워크 설정 오류
- Kernel Panic
- eMMC 손상
- NPU Driver 문제
- SSH 접속 불가

필요 장비:

```text
3.3V USB-TTL UART Adapter × 1 이상
USB microSD Card Reader
복구용 microSD
기준 OS 이미지
예비 Ethernet Cable
예비 USB-C Power Adapter
```

관리 문서에 기록할 항목:

```text
Node ID
MAC Address
IP Address
Hostname
Serial Number
Storage Type
OS Image Version
Kernel Version
RKNPU Driver Version
RKNN Runtime Version
```

---

# 5. 오픈소스 공개 준비

## 5.1 라이선스 구성

NPUDure 자체 소스코드는 Apache License 2.0을 우선 검토한다.

권장 구조:

```text
NPUDure Source       : Apache-2.0
RKNN Runtime          : 저장소에 포함하지 않음
RKNN Toolkit          : 공식 경로에서 별도 설치
RKNN Header/Binary    : 재배포 조건 확인
model.rknn            : 원본 모델 라이선스 확인
Sample Dataset        : 재배포 가능한 자료만 포함
```

필요 파일:

```text
LICENSE
NOTICE
THIRD_PARTY_NOTICES.md
DEPENDENCIES.md
MODEL_LICENSES.md
```

주의사항:

- RKNN SDK Binary를 NPUDure 저장소에 임의로 포함하지 않음
- 사용자가 공식 경로에서 Runtime을 설치하도록 안내
- 모델 원본 라이선스 확인
- 변환된 `.rknn` 파일의 재배포 조건 확인
- 데이터 세트 이미지의 재배포 가능 여부 확인
- 제3자 Rust 및 C 라이브러리 라이선스 목록화

---

## 5.2 README 및 설치 문서

README 필수 내용:

```text
NPUDure 소개
핵심 문제 정의
아키텍처
Mock 3-node Quick Start
RK3576 설치 방법
RKNN Runtime 별도 설치 안내
모델 변환 방법
Benchmark 실행 방법
장애 데모 방법
결과 재현 방법
알려진 제한사항
라이선스
```

외부 사용자는 실제 RK3576 장비가 없어도 Mock Backend로 핵심 구조를 실행할 수 있어야 한다.

권장 문서:

```text
docs/
├── quick-start.md
├── rknn-installation.md
├── model-conversion.md
├── benchmark-guide.md
├── deployment-guide.md
├── failure-demo.md
├── performance-analysis.md
└── troubleshooting.md
```

---

# 6. io_uring 및 Zero-Copy 검증

## 6.1 기본 원칙

io_uring과 Zero-Copy는 필수 성공 조건이 아니다.

다음 조건을 만족할 때만 적용한다.

- 네트워크 또는 System Call이 실제 병목으로 확인됨
- 입력 Payload가 충분히 큼
- 동시 요청이 충분히 많음
- 기준 구현과 동일 조건 비교 가능
- 구현 복잡도 대비 개선 효과가 있음

## 6.2 사전 하드웨어 검증

NanoPi에서 다음을 확인한다.

```bash
ethtool -l eth0
ethtool -k eth0
ethtool -g eth0
ethtool -n eth0
ethtool --show-priv-flags eth0
```

검증 항목:

```text
RX Queue 수
Header/Data Split 지원
Flow Steering 지원
RSS 지원
Kernel io_uring 기능
NIC Driver 지원
Registered Buffer 지원
```

R76S NIC 또는 BSP Driver가 Zero-Copy RX를 지원하지 않으면 해당 구현은 제외한다.

그 경우 다음 대체 최적화를 수행한다.

```text
Tokio/gRPC
→ Bytes 기반 공유 버퍼
→ Buffer Pool
→ 입력 버퍼 재사용
→ 메모리 재할당 감소
→ io_uring 일반 I/O 비교
```

## 6.3 측정 지표

```text
System Calls per Request
Context Switches per Request
CPU Cycles per Request
Memory Allocations per Request
Memory Copies per Request
Requests per Second
p95 Latency
CPU Utilization
```

개선율이 5% 미만이면 발표용 핵심 기능으로 채택하지 않는다.

효과가 없었던 이유도 유효한 발표 결과로 기록한다.

---

# 7. 발표용 추가 요소

## 7.1 필수 요소

```text
실시간 Dashboard
1 / 2 / 3 Node 비교
Round Robin / ECT 전환
Node 상태 카드
실시간 FPS
p95 Latency
Queue Depth
Temperature
장애 노드 자동 제외
노드 자동 복구
녹화된 예비 영상
사전 저장 Benchmark 결과
오프라인 실행
```

## 7.2 권장 요소

```text
노드 상태 LED
노드 번호 라벨
소형 거치대
NPUDure Logo
GitHub QR Code
실시간 전력 표시
객체 탐지 영상
```

## 7.3 발표 장애 대비

- 예비 Ethernet Cable
- 예비 Power Adapter
- Benchmark 결과 CSV
- 결과 그래프 PNG
- 동일 데모 녹화 영상
- Mock Backend Mode
- 인터넷 없는 환경에서 실행
- 발표 전 전체 재부팅 시나리오 확인

---

# 8. 개발 우선순위

| 구분 | 항목 | 우선순위 |
|---|---|---:|
| 모델 | YOLOv8n INT8 기준 모델 | 최우선 |
| 환경 | RKNN Toolkit 변환 환경 | 최우선 |
| 빌드 | ARM64 Rust/C Cross Compile | 최우선 |
| 연동 | RKNN C Wrapper | 최우선 |
| 테스트 | Mock Backend | 최우선 |
| 측정 | Benchmark CLI | 최우선 |
| 운영 | 3노드 배포 스크립트 | 높음 |
| 계측 | Prometheus Metrics | 높음 |
| 장비 | 2.5GbE Switch 및 동일 전원·냉각 | 높음 |
| 공개 | License 및 Third-party Notice | 높음 |
| 측정 | USB-C 전력계 | 중간 |
| 최적화 | io_uring 지원 검증 | 중간 |
| 최적화 | Zero-Copy 실험 | 낮음~중간 |
| 발표 | Dashboard 및 예비 영상 | 10~11월 |

---

# 9. 즉시 수행할 5개 항목

```text
1. RKNN Toolkit / Runtime / Driver 버전 고정
2. YOLOv8n INT8 단일 노드 추론 성공
3. Rust에서 RKNN C Wrapper 호출 성공
4. Mock 3-node Scheduler 구현
5. deploy-all.sh로 동일 바이너리 3대 배포
```

이 다섯 가지가 완료되면 핵심 기술 위험 대부분이 제거된다.

그 다음 순서:

```text
Scheduler
→ 장애 복구
→ Benchmark
→ Metrics
→ Dashboard
→ Profile
→ io_uring 검토
→ Zero-Copy 검토
```

---

# 10. 개발 준비 완료 기준

다음 조건이 충족되면 NPUDure v0.1 본개발 준비가 완료된 것으로 판단한다.

- RKNN 버전 조합 고정
- 기준 모델 선정
- 모델 변환 재현 가능
- 단일 NanoPi NPU 추론 성공
- Rust ARM64 빌드 성공
- RKNN C Wrapper 동작
- Mock Backend 동작
- 세 노드 네트워크 연결
- 동일 바이너리 자동 배포
- Benchmark 원본 결과 저장 가능
- 메트릭 수집 가능
- 라이선스 구조 정리
- GitHub 저장소 공개 준비
- 발표 일정과 기능 동결일 정의

---

# 11. 최종 판단

현재 보유한 NanoPi R76S 3대와 별도 Linux PC만으로 NPUDure v0.1 개발은 가능하다.

추가로 가장 중요한 것은 새로운 하드웨어 구매가 아니라 다음의 완성도다.

```text
모델 변환의 재현성
Rust와 RKNN 연결 안정성
동일 환경의 3노드 구성
원본 Benchmark 데이터
장애 복구
자동 배포
라이선스 정리
발표 데모 안정성
```

Zero-Copy와 io_uring은 마지막 최적화 단계에서 실제 병목이 확인될 때만 적용한다.

NPUDure v0.1의 성공은 이론상 18 TOPS라는 숫자가 아니라, 실제 확장 효율과 손실 원인을 재현 가능하게 증명하는 데 있다.
