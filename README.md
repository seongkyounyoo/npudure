# NPUForge

*[English README](README.en.md)*

여러 대의 저비용 엣지 NPU를 하나의 분산 추론 클러스터로 운영하는 Rust 기반 오픈소스 런타임.

> **6 TOPS NPU 세 대는 정말 18 TOPS가 되는가?**
>
> NPUForge는 그 차이가 어디에서 발생하는지 측정하고, 실제로 확장 가능한 조건을 찾아가는 프로젝트다.

**상태: 측정 계보 종료 (2026-08-21).** S2 부터 S3.9b · S0-D 까지 닫혔다.
**측정 421건, 전 구간 오류율 0.** 남은 것은 Prometheus 메트릭(M2)과
대시보드(M6)다. v0.1 태그는 컨퍼런스 발표 일정에 맞출 계획이지만
**CFP 심사 중이라 시점은 미정이다.**

> ### 결과 한 장
>
> | | 값 | 문서 |
> |---|---|---|
> | **3노드 near-linear** | **3.00×** · 112.9 / 229.0 / 338.4 inf/s (30 run) | [S2](docs/experiments/S2_GRPC_BASELINE.md) |
> | **운영점 최적화** | **387.2 inf/s** (+13.3%) — 단 efficiency 는 98.9 → **95.3%** | [S3.8](docs/experiments/S3_8_OPTIMIZED_SCALEOUT.md) |
> | 남은 손실 | **tail 에서 나타난다** — p50 +0%, p99 **+36%** | [S3.9a](docs/experiments/S3_9A_SCALEOUT_PROFILE.md) |
> | 지속 부하 | 능동 냉각 −1.9% vs 팬리스 −11.3% | [S0](docs/experiments/S0_SUSTAINED_LOAD.md) |
> | 스케줄링 | stale 상태 herding 버그 수정 → p99 **−37%** | [S0-C](docs/experiments/S0_C_POLICY_AB.md) |
> | **io_uring** | **적용하지 않는다 — 측정으로 반박됨** | [S3.9b](docs/experiments/S3_9B_NODE_RESIDUAL.md) |
>
> 절대 처리량과 scaling efficiency 가 **반대로 움직인다**. 둘 중 하나만
> 인용하면 트레이드오프가 숨는다 — 이 저장소는 항상 둘 다 적는다.
>
> **여기서 시작한다 →** [`docs/experiments/README.md`](docs/experiments/README.md)
> (실험 대장 · 배제표 · 방법론 교훈)

---

## 무엇을 하는 프로젝트인가

RK3576 기반 6 TOPS NPU 노드 3대를 네트워크로 묶고, 단일 노드 대비 2노드·3노드 구성에서 **실제 추론 처리량이 어디까지 확장되는지 측정**한다.

명목 TOPS를 합산하는 것이 목표가 아니다. 네트워크 지연, 메모리 복사, 전처리, 스케줄링 불균형, 노드 장애로 인한 손실을 **정량적으로 측정하고 공개**하는 것이 핵심이다.

### 하는 것

- 독립적인 추론 요청을 여러 NPU 노드에 분산 (데이터 병렬)
- 노드 부하와 상태를 반영한 동적 스케줄링
- 장애 노드 자동 제외 및 복구 후 자동 재편입
- 단계별 지연시간 분해 측정
- 재현 가능한 벤치마크와 원본 데이터 공개

### 하지 않는 것

- 여러 NPU를 하나의 물리 NPU처럼 보이게 하는 하드웨어 통합
- 단일 요청의 지연시간을 노드 수에 비례해 단축
- 모델을 레이어 단위로 분할하거나 LLM 텐서 병렬
- Kubernetes 수준의 범용 오케스트레이션

---

## 기준 하드웨어

| 항목 | 값 |
|---|---|
| 보드 | NanoPi R76S × 3 |
| SoC | Rockchip RK3576 (4× A72 @2.2GHz + 4× A53 @1.8GHz) |
| NPU | 6 TOPS |
| 네트워크 | worker 2.5GbE, **aggregation 10GbE** (NEXI 스위치, 구축·실측 완료) |
| 냉각 | 팬리스(조건 A)·능동냉각(조건 B) 두 조건. throttling은 제거 대상이 아니라 **측정 대상** |
| Scheduler | **별도 x86 서버 (10GbE, 16 스레드 이상 권장).** 3노드 합류점이라 2.5G 로는 부족하다. 측정 421건은 Dell PowerEdge R620 / Xeon E5-2630L ×2 (24T) 에서 나왔다 — **CPU 스레드 수가 처리량을 가른다** ([§3.3.4](docs/02-HARDWARE-SETUP.md)) |

> **스케줄러 호스트는 2026-08-26 에 교체됐다.** 측정 421건은 위 R620 에서
> 얻은 값이며 그대로 유효하다. 교체 후 호스트에서는 기준선이 다르게
> 나온다 — 경위와 두 호스트의 규격은 [`docs/hosts/`](docs/hosts/) 와
> [`docs/infrastructure.md`](docs/infrastructure.md) §3.2.1 에 있다.

> 실장비 사진은 [`results/photos-public/`](results/photos-public/) 에 있다.

팬리스를 유지하는 이유는 엣지 디바이스의 실제 배치 형태가 그렇기 때문이다. 팬을 달아 얻은 수치는 현장에서 재현되지 않는다. 벤더가 공개하는 TOPS는 순간 성능이며, 지속 부하에서의 **Peak FPS 대비 Sustained FPS 격차**는 공개 자료가 거의 없다. 이 격차를 측정하는 것 자체가 이 프로젝트의 문제의식과 같은 방향이다.

## 구조

```text
Client → Scheduler → Node 1 (RK3576 / RKNN)
                   → Node 2 (RK3576 / RKNN)
                   → Node 3 (RK3576 / RKNN)
```

| 크레이트 | 역할 |
|---|---|
| `npuforge-common` | 공통 타입, 오류 코드, 설정 스키마, 백엔드 인터페이스 |
| `npuforge-proto` | gRPC 서비스 정의 |
| `npuforge-scheduler` | 중앙 스케줄러. 레지스트리, 정책, 헬스체크, 재시도 |
| `npuforge-node` | 노드 에이전트. 모델 로딩, 추론, 상태 보고 |
| `npuforge-rknn` | RKNN Runtime FFI. `unsafe` 코드는 여기로 제한 |
| `npuforge-mock-backend` | 하드웨어 없이 동작하는 Mock 백엔드 |
| `npuforge-bench` | 부하 발생 및 통계 계산 CLI |

---

## 하드웨어 없이 시작하기

RK3576 장비가 없어도 Mock Backend로 핵심 구조를 실행할 수 있다. 이는 부가 기능이 아니라 설계 원칙이다.

### 전제 조건

| | |
|---|---|
| **Rust** | stable 툴체인 (`rust-toolchain.toml` 이 채널을 고정한다) |
| **protoc** | **필수** — gRPC 정의를 빌드 시점에 컴파일한다 |
| 네트워크 | crates.io 접근. 의존성은 `Cargo.lock` 으로 버전이 고정돼 있으나 vendor 되어 있지는 않다 |

```bash
# protoc — 플랫폼에 맞게
sudo apt install protobuf-compiler     # Ubuntu / Debian
sudo dnf install protobuf-compiler     # Rocky / RHEL / Fedora
brew install protobuf                  # macOS
winget install protobuf                # Windows
```

`protoc` 이 없으면 빌드가 `npuforge-proto` 에서 멈추며 **빠진 도구를
이름으로 지목한다.** 조용히 실패하지 않는다.

```bash
git clone https://github.com/seongkyounyoo/npuforge.git
cd npuforge

# 전체 빌드와 테스트. RKNN SDK 없이 통과한다.
cargo test --workspace

# 설정 검증
cargo run -p npuforge-scheduler -- --config configs/scheduler.example.toml
cargo run -p npuforge-node -- --config configs/mock/node-01.toml
```

`configs/mock/` 의 세 노드는 서로 다른 속도와 오류율을 갖도록 되어 있다. Round Robin과 ECT의 차이가 로컬에서도 드러나게 하기 위한 것이다.

---

## 스케줄링 정책

| 식별자 | 정책 | 설명 |
|---|---|---|
| `round-robin` | Round Robin | 순차 분배. 비교 기준 |
| `least-queue` | Least Queue | 가장 짧은 큐 선택 |
| `ect` | Estimated Completion Time | 예상 완료시간 기반. 기본값 |

```text
ECT = ((queue_depth + in_flight + 1) × EWMA_inference
       + EWMA_network + thermal_penalty + error_penalty) / load_factor
```

세 정책은 동일한 후보 필터를 공유한다. 정책마다 필터가 다르면 정책 비교 실험이 정책이 아니라 필터의 차이를 측정하게 되기 때문이다.

---

## 실장비 빌드

노드 바이너리는 개발 PC에서 ARM64로 크로스컴파일해 세 노드에 **동일 바이너리**를 배포한다.

```bash
# RKNN SDK 설치 후
export RKNN_SDK_PATH=/path/to/rknn/include
export RKNN_LIB_PATH=/path/to/rknn/lib

cargo build-node   # = --release --target aarch64-unknown-linux-gnu -p npuforge-node --features rknn
```

`npuforge-rknn` 의 `rknn` feature는 기본으로 꺼져 있다. 개발 PC(Windows/x86 포함)에서 `cargo build --workspace` 가 통과해야 Mock 기반 개발이 하드웨어에 묶이지 않기 때문이다.

RKNN 지원 없이 빌드한 바이너리에 RKNN 설정을 주면 시작 시점에 명확한 오류로 중단된다.

---

## 문서

| 문서 | 내용 |
|---|---|
| **[`docs/experiments/README.md`](docs/experiments/README.md)** | **실험 대장 — 여기서 시작한다.** 무엇을 묻고 무엇이 배제됐는지 한 장. 원본 데이터 대응표와 방법론 교훈 포함 |
| [`adrs/`](adrs/README.md) | **왜 이렇게 되어 있는가 — 아키텍처 결정 기록 28건** |
| [`docs/experiments/`](docs/experiments/) | 실험 보고서 12건 (S2 · S3 · S3.5~3.9b · S0-A~D) |
| [`docs/GLOSSARY.md`](docs/GLOSSARY.md) | 기술 용어 13개 절 — 실험 ID 체계, 측정 방법론, 사전 등록 규칙 |
| [`docs/RESULTS.md`](docs/RESULTS.md) | 단일 노드 계보 1차 정리 (다중 노드는 `experiments/`) |
| [`docs/TODO.md`](docs/TODO.md) | 진행 현황과 다음에 할 일 |
| [`docs/00-PRD.md`](docs/00-PRD.md) | 목표, 비목표, 기능 요구사항, 성공 기준 |
| [`docs/01-TECHSPEC.md`](docs/01-TECHSPEC.md) | 구조, 프로토콜, 설정 스키마, 스케줄링, 벤치마크 설계 |
| [`docs/02-HARDWARE-SETUP.md`](docs/02-HARDWARE-SETUP.md) | 물리 구성, 네트워크, 전원, 냉각, 실험 조건 |
| [`docs/03-DEVELOPMENT-REQUIREMENTS.md`](docs/03-DEVELOPMENT-REQUIREMENTS.md) | 개발환경, 도구, 배포 자동화, 라이선스 |
| [`docs/environment-matrix.md`](docs/environment-matrix.md) | 버전 조합과 해시 고정 |
| [`docs/hosts/`](docs/hosts/) | 스케줄러 호스트 하드웨어 인벤토리 (기계 수집) |
| [`docs/ALL.md`](docs/ALL.md) | **위 문서 전부를 한 파일로.** 읽기·인쇄·검토용 생성물 |

문서가 서로 다른 값을 기술하면 `docs/00-PRD.md` §0의 우선순위를 따른다.

---

## 성공 기준에 대해

이 프로젝트의 성공 기준은 **"특정 수치가 나왔는가"가 아니라 "측정하고 설명할 수 있는가"** 이다.

측정을 목적으로 하는 프로젝트에서 결과값을 성공 조건으로 걸면, 목표 수치가 나오지 않을 때 실험 조건을 유리하게 선택할 유인이 생긴다. 따라서 다음 결과도 유효한 성과로 간주한다.

- ✅ **io_uring이 유의미한 성능 개선을 만들지 못함 — 실현됨 (2026-08-21)**
- Zero-Copy 적용 범위가 제한적임
- 네트워크보다 NPU 또는 전처리가 주요 병목으로 확인됨
- ✅ **3노드 확장 효율이 예상보다 낮음 — 부분 실현** (운영점 95.3%)
- 단일 고성능 장치가 비용 면에서 더 유리함

병목 원인과 적용 조건을 정량적으로 제시하는 것이 결과다.

> **첫 항목은 측정 전에 적어 둔 것이고, 실제로 그렇게 됐다.**
> 요청당 네트워크 syscall 은 약 165회지만 진입 비용은 transport CPU 의
> 1%에 그치고, 부하 중 보드는 48.9% idle 이다 — **CPU 는 비용이지
> 제약이 아니다.** 만들지 않기로 결정하고 그 근거를
> [`01-TECHSPEC.md` §15](docs/01-TECHSPEC.md) 에 기록했다.
> → [S3.9b](docs/experiments/S3_9B_NODE_RESIDUAL.md)

---

## 알려진 제한사항

- **노드가 3대뿐이다.** 4노드 이상에서 이 결론이 유지되는지는 미측정이다
- **반복 수가 작다.** 구성당 3~4 run 이 많다. p50/p95 차이는 SD 가 작아
  신뢰할 만하나 **처리량 1% 미만 차이는 우열 근거로 쓰지 않았다**
- **percentile 은 run-level 평균이다.** pooled 가 아니다 — tail 을 낮게
  보이게 하므로 절대값을 "이 시스템의 p99" 로 인용하면 안 된다
- **잔여 gap 16.1% 의 정체는 미확정.** CPU 비용이 아니라 경로 지연으로
  보이지만(페이로드 1.2MB 왕복만 8.2ms) 특정하지 못했다
- **Prometheus 메트릭 미구현.** REST 관리 API와 대시보드(M6)도 아직이다
- **후처리(NMS) 없음.** 노드는 원시 텐서를 그대로 반환한다
- **JPEG 디코딩 없음.** 입력은 RGB8/BGR8 원본만 받는다
- **인증·TLS 없음.** gRPC 는 평문이고 노드 등록에 검증이 없다.
  **신뢰된 사설망 안에서만 쓰도록 설계했다** — 결함이 아니라 범위다

---

## 라이선스

[Apache License 2.0](LICENSE).

RKNN Runtime과 SDK는 이 저장소에 포함되지 않는다. 공식 경로에서 별도로 설치해야 한다.
모델 파일과 데이터 세트의 재배포 조건은 각 원본 라이선스를 따른다.
