# NPUDure Technical Specification

> ## ⚠️ 이 문서는 **규범 문서이자 일부는 계획 기준선**이다
>
> 구조·프로토콜·설정 스키마는 현행이다. 반면 **일정과 후보 기능**
> (S5 · 11월 io_uring 비교 등)은 측정 전에 정한 계획이며 그대로 두었다.
> 계획을 사후에 고치면 "무엇을 예상했고 무엇이 빗나갔는가" 가 사라지므로
> 그대로 둔다.
>
> **실제로 무엇이 닫혔고 무엇이 열려 있는지는 여기가 아니다.**
>
> | 무엇 | 어디 |
> |---|---|
> | 실험 최종 상태 | [`experiments/README.md`](experiments/README.md) §5~§7 |
> | io_uring 판정 (**적용하지 않는다**) | **이 문서 §15** |
> | 확정 수치 | [`RESULTS.md`](RESULTS.md) · [`experiments/`](experiments/) |
>
> 특히 이 문서가 비교 실험으로 들고 있는 **io_uring 은 구현하지 않기로
> 했다.** S3.9b 가 회수 가능한 몫을 ≈8% 로 측정했고, 그 근거로 배제했다.

- 문서명: `01-TECHSPEC.md`
- 프로젝트명: NPUDure
- 문서 버전: v0.2
- 대상 릴리스: NPUDure v0.1
- 목표 발표: 2026년 11월 FOSS for All Conference
- 작성일: 2026-08-05
- 최종 수정: 2026-08-06 (본문)
- 2026-08-27: 최종 상태를 가리키는 배너만 추가. **본문은 계획 기준선 그대로 둔다**
- 상태: Draft
- 관련 문서:
  - `00-PRD.md`
  - `02-HARDWARE-SETUP.md`
  - `03-DEVELOPMENT-REQUIREMENTS.md`
  - `environment-matrix.md`

본 문서는 저장소 구조, 프로토콜, 설정 스키마, 스케줄링 알고리즘, 오류 코드에 대한 규범 문서다. 해당 영역에서 다른 문서와 값이 다를 경우 본 문서를 따른다. 물리 구성과 실험 조건은 `02-HARDWARE-SETUP.md`를 따른다.

---

# 1. 문서 목적

본 문서는 NPUDure v0.1의 구현 구조, 컴포넌트 책임, 통신 프로토콜, 데이터 모델, 스케줄링 방식, 장애 처리, 메트릭 수집, 벤치마크 방법 및 배포 구조를 정의한다.

NPUDure v0.1은 RK3576 기반 6 TOPS NPU 노드 최대 3대를 하나의 분산 추론 클러스터로 운영하는 Rust 기반 오픈소스 런타임이다.

이 문서의 목표는 다음과 같다.

1. 개발자가 추가 해석 없이 구현을 시작할 수 있도록 한다.
2. 기능 범위와 비기능 요구사항을 기술적으로 구체화한다.
3. 성능 비교가 가능한 기준 구현을 정의한다.
4. 발표 데모와 연구용 벤치마크의 재현성을 확보한다.
5. RKNN 종속 코드를 격리하여 향후 다른 NPU 백엔드로 확장 가능하게 한다.

---

# 2. 설계 원칙

## 2.1 데이터 병렬 우선

NPUDure v0.1은 하나의 모델을 여러 노드에 분할하지 않는다.

각 노드는 동일한 전체 모델을 보유하고 서로 다른 추론 요청을 독립적으로 처리한다.

```text
Request A → Node 1
Request B → Node 2
Request C → Node 3
```

목표는 단일 요청 지연시간 단축이 아니라 전체 처리량 증가와 장애 허용이다.

## 2.2 측정 가능한 최적화

io_uring, 버퍼 풀, 메모리 재사용, Zero-Copy는 이름 자체가 목표가 아니다.

각 최적화는 다음 조건을 만족해야 한다.

- 기준 구현이 존재할 것
- 동일한 실험 조건에서 비교할 수 있을 것
- 처리량, 지연시간, CPU 사용률 또는 복사 횟수 중 하나 이상이 측정될 것
- 효과가 없거나 부정적이어도 결과를 기록할 것

## 2.3 단순한 중앙 스케줄러

v0.1은 중앙 스케줄러 구조를 사용한다.

분산 합의, 리더 선출, 다중 스케줄러 고가용성은 구현하지 않는다.

```text
Client
   │
   ▼
Scheduler
   ├── Node 1
   ├── Node 2
   └── Node 3
```

## 2.4 백엔드 분리

RKNN Runtime 직접 호출은 `npuforge-rknn` 크레이트에 격리한다.

스케줄러와 노드 에이전트는 공통 `InferenceBackend` 인터페이스만 사용한다.

## 2.5 재현성 우선

모든 실험은 다음 정보를 함께 저장해야 한다.

- Git commit hash
- OS 및 커널 버전
- RKNN Runtime 버전
- 모델 식별자와 해시
- 입력 데이터 세트 해시
- 노드 수
- 네트워크 구성
- 스케줄링 정책
- 동시성
- 테스트 시간
- 온도 및 전력 조건
- 원본 측정값

---

# 3. 전체 아키텍처

## 3.1 논리 구조

```text
┌───────────────────────────────────────────────────────────┐
│ Clients                                                   │
│                                                           │
│  ┌────────────────┐  ┌────────────────┐  ┌─────────────┐ │
│  │ Demo Web Client│  │ Benchmark CLI  │  │ API Client  │ │
│  └───────┬────────┘  └───────┬────────┘  └──────┬──────┘ │
└──────────┼───────────────────┼───────────────────┼────────┘
           │                   │                   │
           └───────────────────┴───────────────────┘
                               │
                               ▼
┌───────────────────────────────────────────────────────────┐
│ NPUDure Scheduler                                        │
│                                                           │
│  API Gateway                                              │
│  Node Registry                                            │
│  Scheduler Engine                                         │
│  Retry Manager                                            │
│  Health Monitor                                           │
│  Metrics Collector                                        │
│  Event Logger                                             │
└──────────────┬────────────────┬────────────────┬───────────┘
               │                │                │
               ▼                ▼                ▼
      ┌────────────────┐ ┌────────────────┐ ┌────────────────┐
      │ NPUDure Node 1│ │ NPUDure Node 2│ │ NPUDure Node 3│
      │ RK3576 / RKNN  │ │ RK3576 / RKNN  │ │ RK3576 / RKNN  │
      └────────────────┘ └────────────────┘ └────────────────┘
```

## 3.2 물리 구성

기준 하드웨어는 다음과 같다.

- RK3576 기반 NanoPi R76S 또는 동급 보드 3대
- 각 노드에 동일한 OS, 커널, RKNN Runtime 설치
- 유선 Ethernet 연결
- 중앙 스케줄러는 별도 x86 또는 ARM Linux 장치에서 실행
- 네트워크는 **worker 2.5GbE / aggregation 10GbE** (아래 근거)

### 네트워크 기준선 근거

**2026-08-12 개정.** 이전 판은 "3노드 150 FPS × 1.23MB ≈ 1.5 Gbps 이므로
2.5GbE 로 충분" 이었다. 두 가지가 틀렸다 — (a) 150 FPS 는 3노드 **합계**
가정인데 실측은 노드 **한 대**가 INT8 157.2 inf/s 다, (b) **출력 방향을
계산하지 않았다.** 아래는 실측 기반으로 다시 계산한 값이다.

입력 payload 한 장은 `640 × 640 × 3 = 1,228,800 byte` 다.

```text
                     노드당          3노드 합계
INT8  157.2 inf/s    1.545 Gbps      4.636 Gbps
FP16   84.3 inf/s    0.829 Gbps      2.486 Gbps
```

**FP16 조차 3노드 합계가 2.5GbE 한 링크(실효 약 2.35 Gbps)를 넘는다.**

즉 **worker 링크가 아니라 aggregation 링크가 먼저 막힌다.** 노드는 각자
자기 링크만 쓰지만(최대 1.545 Gbps), 세 노드의 트래픽이 스케줄러 앞에서
합쳐지기 때문이다.

- **worker 링크: 2.5GbE** — 노드당 1.545 Gbps 이므로 충분하다
- **aggregation 링크: 10GbE** — 여기가 실제 제약이다
- 스케줄러 호스트는 10G SFP+ NIC 을 꽂을 **PCIe 슬롯**이 있어야 한다

응답 방향도 링크를 쓴다. 노드가 후처리 없이 원시 텐서를 반환하므로
`want_float=1` 이면 응답이 요청의 3.96배가 되어 3노드 RX 가 18.38 Gbps 에
이른다. 10G 로도 부족하다. 이 문제는 `want_float=0` 전환으로 해결했다
(§16.2, `02-HARDWARE-SETUP.md` §3.3.2, `adrs/012-want-float-zero-blob-v2.md`).

1GbE 를 기준으로 삼으면 대용량 입력 조건에서 NPU 확장 효율이 아니라 링크
포화를 측정하게 된다. 이는 본 프로젝트의 측정 목적과 맞지 않는다.

단, 1GbE 는 제거하지 않고 §20.2 S5 및 S6 의 **비교 조건**으로 유지한다.
"네트워크가 병목인 조건" 과 "그렇지 않은 조건" 을 나란히 제시하는 것이
병목 분석 결과로서 가치가 있다.

상세 근거와 토폴로지는 `adrs/014-10g-aggregation-separate-scheduler.md`.

### 스케줄러 배치 제약

공식 벤치마크에서는 스케줄러를 RK3576 노드에서 실행하지 **않는다.**

한 노드에만 스케줄러 부하가 실리면 세 노드의 실험 조건이 달라져 1/2/3노드 비교가 왜곡된다. 상세 근거는 `02-HARDWARE-SETUP.md` §2.1을 따른다.

단순 개발 및 이동형 데모에 한해 노드 중 하나에서 함께 실행할 수 있으나, 이때 측정한 값은 공식 성능 수치로 사용하지 않는다.

## 3.3 프로세스 구성

### Scheduler Host

- `npuforge-scheduler`
- `npuforge-dashboard`
- Prometheus
- 선택적으로 Grafana

### NPU Node

- `npuforge-node`
- RKNN Runtime
- 모델 파일
- 하드웨어 메트릭 수집기

### Benchmark Host

- `npuforge-bench`
- 테스트 데이터 세트
- 결과 저장 디렉터리

---

# 4. 저장소 구조

```text
npuforge/
├── Cargo.toml
├── Cargo.lock
├── LICENSE
├── README.md              (영문 — 기본)
├── README.ko.md           (한글)
├── rust-toolchain.toml
├── crates/
│   ├── npuforge-common/
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── model.rs
│   │   │   ├── protocol.rs
│   │   │   ├── telemetry.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── npuforge-proto/
│   │   ├── proto/
│   │   │   └── npuforge.proto
│   │   ├── build.rs
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   ├── npuforge-scheduler/
│   │   ├── src/
│   │   │   ├── api/
│   │   │   ├── health/
│   │   │   ├── registry/
│   │   │   ├── retry/
│   │   │   ├── scheduler/
│   │   │   ├── telemetry/
│   │   │   ├── state.rs
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── npuforge-node/
│   │   ├── src/
│   │   │   ├── backend/
│   │   │   ├── health/
│   │   │   ├── inference/
│   │   │   ├── metrics/
│   │   │   ├── model_manager/
│   │   │   ├── worker/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── npuforge-rknn/
│   │   ├── include/
│   │   ├── native/
│   │   │   ├── rknn_wrapper.c
│   │   │   └── rknn_wrapper.h
│   │   ├── src/
│   │   │   ├── ffi.rs
│   │   │   ├── backend.rs
│   │   │   ├── buffer.rs
│   │   │   ├── error.rs
│   │   │   └── lib.rs
│   │   ├── build.rs
│   │   └── Cargo.toml
│   ├── npuforge-bench/
│   │   ├── src/
│   │   │   ├── load.rs
│   │   │   ├── output.rs
│   │   │   ├── scenario.rs
│   │   │   ├── statistics.rs
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   └── npuforge-mock-backend/
│       ├── src/lib.rs
│       └── Cargo.toml
├── dashboard/
│   ├── src/
│   ├── public/
│   └── package.json
├── configs/
│   ├── scheduler.example.toml
│   ├── node.example.toml
│   └── benchmark.example.toml
├── deploy/
│   ├── systemd/
│   ├── docker/
│   └── scripts/
├── scripts/
├── tools/
│   └── model-converter/
├── examples/
│   ├── image-classification/
│   └── object-detection/
├── benchmarks/
│   ├── scenarios/
│   ├── results/
│   └── analysis/
├── docs/
│   ├── 00-PRD.md
│   ├── 01-TECHSPEC.md
│   ├── 02-HARDWARE-SETUP.md
│   ├── 03-DEVELOPMENT-REQUIREMENTS.md
│   ├── environment-matrix.md
│   ├── quick-start.md
│   ├── rknn-installation.md
│   ├── model-conversion.md
│   ├── benchmark-guide.md
│   ├── deployment-guide.md
│   ├── failure-demo.md
│   ├── performance-analysis.md
│   └── troubleshooting.md
└── .github/
    └── workflows/
```

## 4.1 scripts와 deploy의 구분

두 디렉터리의 역할이 겹치지 않도록 다음과 같이 나눈다.

| 디렉터리 | 용도 | 예 |
|---|---|---|
| `scripts/` | 개발자와 실험자가 실행하는 운영 자동화 | `build-arm64.sh`, `deploy-all.sh`, `run-benchmark.sh`, `check-versions.sh` |
| `deploy/` | 대상 장비에 설치되는 산출물과 설치 스크립트 | systemd unit, Dockerfile, `install-node.sh` |

`scripts/`의 전체 목록은 `03-DEVELOPMENT-REQUIREMENTS.md` §4.3을 따른다.

`tools/model-converter/`의 구성은 `03-DEVELOPMENT-REQUIREMENTS.md` §2.1을 따른다.

---

# 5. 기술 스택

## 5.1 Rust

- Rust Stable
- Edition 2024 사용 검토
- 최소 지원 Rust 버전은 프로젝트 초기 검증 후 고정
- Workspace 기반 멀티 크레이트 구성

## 5.2 비동기 런타임

기준 구현:

- Tokio
- tonic gRPC
- axum 관리 API
- tower middleware

실험 구현:

- io_uring 기반 별도 transport 또는 특정 데이터 전송 경로
- 필요 시 `tokio-uring` 또는 직접 래퍼 검토
- 기준 구현과 실험 구현을 기능 플래그로 분리

## 5.3 직렬화

- 내부 RPC: Protocol Buffers
- 설정: TOML
- 구조화 로그: JSON
- 벤치마크 결과: JSON Lines 및 CSV
- 관리 API: JSON

## 5.4 관측성

- tracing
- tracing-subscriber
- metrics 또는 prometheus crate
- Prometheus endpoint
- OpenTelemetry는 v0.1 선택 기능

## 5.5 웹 대시보드

다음 중 하나를 선택한다.

우선안:

- React 또는 단순 TypeScript SPA
- Scheduler의 REST API와 WebSocket/SSE 사용

단순화안:

- Rust + askama 또는 minijinja
- htmx 기반 UI

발표 일정이 우선이므로 기능보다 구현 속도를 기준으로 선택한다.

---

# 6. 컴포넌트 상세

## 6.1 npuforge-common

공통 타입과 설정을 제공한다.

주요 타입:

```rust
pub type NodeId = String;
pub type RequestId = uuid::Uuid;
pub type ModelId = String;
pub type ModelVersion = String;
```

주요 모듈:

- `config`: TOML 설정 구조
- `error`: 공통 오류 코드
- `model`: 모델 식별 및 메타데이터
- `protocol`: 공통 요청·응답 데이터 모델
- `telemetry`: 메트릭 타입

의존성은 최소화한다.

## 6.2 npuforge-proto

gRPC 서비스 정의와 생성 코드를 포함한다.

서비스:

- `SchedulerService`
- `NodeService`
- `ControlService`

프로토콜 변경은 버전 호환성을 고려하여 필드를 삭제하지 않고 예약 처리한다.

## 6.3 npuforge-scheduler

중앙 제어 컴포넌트다.

책임:

- 외부 추론 요청 수신
- 노드 등록 및 제거
- 헬스체크
- 스케줄링
- 재시도
- 타임아웃
- 메트릭 집계
- 이벤트 발행
- 대시보드 API 제공

내부 상태는 초기 버전에서 메모리 기반으로 유지한다.

영구 저장이 필요한 항목:

- 벤치마크 결과
- 이벤트 로그
- 설정 스냅샷

PostgreSQL은 v0.1 필수가 아니다.

## 6.4 npuforge-node

각 NPU 장치에서 실행된다.

책임:

- Scheduler 등록
- 모델 로딩
- 추론 요청 처리
- 전처리 및 후처리
- 로컬 작업 큐 관리
- 메트릭 보고
- 상태 보고
- graceful shutdown

노드 내부 워커 수는 모델과 RKNN Runtime 제약에 따라 설정 가능하게 한다.

## 6.5 npuforge-rknn

RKNN Runtime 연동 전용 크레이트다.

책임:

- C API FFI
- 안전한 Rust 래퍼
- 모델 컨텍스트 생성 및 해제
- 입력·출력 버퍼 관리
- 추론 호출
- 오류 변환
- 모델 메타데이터 조회

`unsafe` 코드는 이 크레이트 내부로 제한한다.

## 6.6 npuforge-bench

부하 발생 및 통계 계산 도구다.

책임:

- 고정 동시성
- 점진적 부하 증가
- 고정 요청 수
- 고정 시간 테스트
- 입력 데이터 순환
- 결과 저장
- 요약 통계
- 실패율 계산

---

# 7. API 및 통신 프로토콜

## 7.1 외부 추론 API

기준 API는 gRPC로 한다.

### Infer

```protobuf
rpc Infer(InferRequest) returns (InferResponse);
```

예시 구조:

```protobuf
message InferRequest {
  string request_id = 1;
  string model_id = 2;
  bytes payload = 3;
  string input_format = 4;
  int32 priority = 5;
  int64 deadline_unix_ms = 6;
  map<string, string> metadata = 7;
}

message InferResponse {
  string request_id = 1;
  string node_id = 2;
  bytes result = 3;
  string result_format = 4;
  Timing timing = 5;
  string error_code = 6;
  string error_message = 7;
}
```

### BatchInfer

v0.1에서는 선택 기능이다.

클라이언트 배칭과 노드 내부 배칭을 구분해야 한다.

## 7.2 노드 등록 API

```protobuf
rpc RegisterNode(RegisterNodeRequest) returns (RegisterNodeResponse);
rpc Heartbeat(HeartbeatRequest) returns (HeartbeatResponse);
rpc DeregisterNode(DeregisterNodeRequest) returns (DeregisterNodeResponse);
```

등록 시 전달 정보:

```protobuf
message NodeDescriptor {
  string node_id = 1;
  string hostname = 2;
  string address = 3;
  string device_type = 4;
  string npu_type = 5;
  uint32 npu_core_count = 6;
  uint64 memory_bytes = 7;
  string agent_version = 8;
  string runtime_version = 9;
  repeated ModelDescriptor models = 10;
}
```

## 7.3 노드 추론 API

Scheduler가 Node에 호출한다.

```protobuf
service NodeService {
  rpc Infer(NodeInferRequest) returns (NodeInferResponse);
  rpc Health(HealthRequest) returns (HealthResponse);
  rpc ListModels(ListModelsRequest) returns (ListModelsResponse);
  rpc Warmup(WarmupRequest) returns (WarmupResponse);
}
```

## 7.4 관리 API

REST 기반으로 제공한다.

예시:

```text
GET  /api/v1/cluster
GET  /api/v1/nodes
GET  /api/v1/nodes/{node_id}
GET  /api/v1/models
GET  /api/v1/metrics/summary
GET  /api/v1/events
POST /api/v1/scheduler/policy
POST /api/v1/nodes/{node_id}/drain
POST /api/v1/nodes/{node_id}/enable
```

## 7.5 메트릭 API

```text
GET /metrics
```

Prometheus 형식으로 노출한다.

---

# 8. 주요 데이터 모델

## 8.1 NodeRecord

```rust
pub struct NodeRecord {
    pub descriptor: NodeDescriptor,
    pub state: NodeState,
    pub last_heartbeat_at: Instant,
    pub consecutive_health_failures: u32,
    pub consecutive_health_successes: u32,
    pub queue_depth: u32,
    pub in_flight: u32,
    pub ewma_inference_ms: f64,
    pub ewma_network_ms: f64,
    pub error_rate: f64,
    pub temperature_c: Option<f64>,
    pub cpu_percent: Option<f64>,
    pub memory_percent: Option<f64>,
    pub npu_percent: Option<f64>,
}
```

## 8.2 NodeState

```rust
pub enum NodeState {
    Registering,
    Healthy,
    Busy,
    Degraded,
    Unreachable,
    Recovering,
    Draining,
    Disabled,
}
```

## 8.3 InferenceTask

```rust
pub struct InferenceTask {
    pub request_id: RequestId,
    pub model_id: ModelId,
    pub payload: bytes::Bytes,
    pub input_format: InputFormat,
    pub priority: i32,
    pub deadline: Option<Instant>,
    pub created_at: Instant,
    pub attempt: u32,
    pub max_attempts: u32,
}
```

## 8.4 TimingBreakdown

```rust
pub struct TimingBreakdown {
    pub scheduler_queue_us: u64,
    pub scheduler_route_us: u64,
    pub network_to_node_us: u64,
    pub node_queue_us: u64,
    pub decode_us: u64,
    pub preprocess_us: u64,
    pub npu_input_us: u64,
    pub inference_us: u64,
    pub postprocess_us: u64,
    pub network_to_client_us: u64,
    pub end_to_end_us: u64,
}
```

---

# 9. 노드 상태 머신

## 9.1 상태 전이

```text
Registering
   │ registration success
   ▼
Healthy ──────────────┐
   │ high load         │ manual drain
   ▼                   ▼
Busy                Draining
   │ errors             │ queue empty
   ▼                    ▼
Degraded            Disabled
   │ health fail
   ▼
Unreachable
   │ health success
   ▼
Recovering
   │ consecutive success
   └───────────────→ Healthy
```

## 9.2 기본 임계치

초기 기본값:

- Heartbeat interval: 2초
- Health timeout: 1초
- 연속 실패 3회: `Unreachable`
- 연속 성공 3회: `Recovering`에서 `Healthy`
- 큐 길이 임계치 초과: `Busy`
- 최근 오류율 10% 초과: `Degraded`
- 온도 80°C 이상: `Degraded`
- 온도 90°C 이상: 스케줄링 제외

모든 값은 설정 가능해야 한다.

---

# 10. 스케줄링

## 10.0 정책 식별자

정책 식별자는 다음 세 개로 고정한다. 설정 파일, CLI 인자, 메트릭 레이블, 로그, 대시보드에서 **모두 동일한 문자열**을 사용한다.

| 식별자 | 정책 | 용도 |
|---|---|---|
| `round-robin` | Round Robin | 비교 기준 |
| `least-queue` | Least Queue | 중간 비교군 |
| `ect` | Estimated Completion Time | 권장 기본값 |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchedulingPolicyKind {
    RoundRobin,
    LeastQueue,
    Ect,
}
```

식별자는 Rust enum과 serde로만 파싱하고, 문자열 비교를 코드 여러 곳에 흩어 두지 않는다.

`queue-aware`, `estimated-completion-time`, `queue_aware` 등의 표기는 사용하지 않는다. "부하 기반 스케줄링"과 "Queue-aware"는 `round-robin`이 아닌 정책 전체를 가리키는 산문상의 총칭으로만 사용하며, 식별자로 쓰지 않는다.

## 10.1 Scheduler 인터페이스

```rust
pub trait SchedulingPolicy: Send + Sync {
    fn select_node(
        &self,
        task: &InferenceTask,
        candidates: &[NodeSnapshot],
    ) -> Result<NodeId, ScheduleError>;
}
```

## 10.2 Round Robin

기준 구현이다.

조건:

- `Healthy`, `Busy` 상태만 후보
- 요청 모델을 로딩한 노드만 후보
- `Draining`, `Disabled`, `Unreachable` 제외

장점:

- 구현 단순
- 비교 기준 제공

단점:

- 노드별 처리속도와 큐 상태를 반영하지 못함

## 10.3 Least Queue

가장 작은 큐를 가진 노드를 선택한다.

동률이면 다음 기준 적용:

1. 낮은 in-flight 수
2. 낮은 평균 추론시간
3. Node ID 정렬

## 10.4 Estimated Completion Time

권장 정책이다.

예상 비용:

```text
ECT =
((queue_depth + in_flight + 1) × EWMA_inference_time
 + EWMA_network_time
 + thermal_penalty
 + error_penalty)
/ load_factor
```

가장 낮은 점수를 가진 노드를 선택한다.

### `+ 1`의 근거

대기 중인 요청만 세지 않고 지금 배정하려는 요청 자신을 포함한다.

ECT는 "이 요청이 언제 끝나는가"를 추정하는 값이므로 자기 추론시간이 포함되어야 한다. 또한 큐가 빈 노드의 점수가 0이 되면 아래의 `load_factor` 보정이 무력화된다.

### `load_factor`의 근거

노드 상태별 가중치다.

| 상태 | load_factor |
|---|---:|
| Healthy | 1.0 |
| Busy | 1.0 |
| Degraded | 0.5 |
| Recovering | 0.25 |
| 그 외 | 0.0 (후보 제외) |

`Recovering` 노드는 큐가 비어 있으므로 점수만 보면 항상 이긴다. 복구 직후 전량을 받으면 같은 원인으로 다시 실패할 수 있으므로, PRD FR-07의 "제한된 요청만 할당" 요구를 별도 카운터 없이 점수 하나로 구현한다.

### 동점 처리

점수가 같으면 Node ID 사전순으로 결정한다. 동점 처리가 흔들리면 같은 조건의 반복 실험 결과가 달라져 재현성이 깨진다.

### 후보 필터 공유

세 정책은 모두 동일한 후보 필터를 거친다.

- `is_schedulable()` 상태일 것
- 요청 모델을 `Ready` 상태로 보유할 것
- 온도가 `disable_temperature_c` 미만일 것

정책마다 필터가 다르면 §20.2 S3의 정책 비교가 정책의 차이가 아니라 필터의 차이를 측정하게 된다.

## 10.5 우선순위

v0.1에서는 단순 정수 우선순위를 지원한다.

- 0: normal
- 10: high
- -10: low

Starvation 방지를 위해 대기시간 기반 aging을 적용할 수 있다.

## 10.6 Deadline

Deadline이 존재하는 요청은 예상 완료시간이 deadline을 초과하는 노드를 후보에서 제외할 수 있다.

모든 노드가 deadline을 만족하지 못하면 다음 중 하나를 선택한다.

- 즉시 `DEADLINE_UNSATISFIABLE` 반환
- 가장 빠른 노드에 best-effort 전송

기본 정책은 best-effort다.

---

# 11. 요청 처리 흐름

## 11.1 정상 흐름

```text
1. Client가 Infer 요청 전송
2. Scheduler가 요청 검증
3. Request ID 생성 또는 검증
4. 후보 노드 조회
5. 스케줄링 정책 실행
6. 선택 노드로 요청 전송
7. Node가 로컬 큐에 등록
8. 전처리
9. RKNN 추론
10. 후처리
11. Node가 결과 반환
12. Scheduler가 메트릭 기록
13. Client에 응답
```

## 11.2 실패 흐름

```text
1. Node 호출 실패 또는 timeout
2. 실패 원인 분류
3. 재시도 가능 여부 확인
4. attempt 증가
5. 실패 노드를 후보에서 일시 제외
6. 다른 노드 선택
7. 재시도
8. 최대 횟수 초과 시 오류 반환
```

## 11.3 중복 처리

추론 요청은 원칙적으로 side effect가 없으므로 재시도가 가능하다.

Scheduler는 짧은 TTL의 Request ID 캐시를 유지해 중복 제출을 감지한다.

v0.1에서는 결과 캐시까지 필수로 구현하지 않는다.

---

# 12. 재시도 및 타임아웃

## 12.1 타임아웃 종류

- Client request timeout
- Scheduler queue timeout
- Node RPC timeout
- Node local queue timeout
- Inference timeout

## 12.2 재시도 가능 오류

- 네트워크 연결 실패
- Node timeout
- 일시적 runtime 오류
- Node overloaded
- Node unavailable

## 12.3 재시도 불가 오류

- 잘못된 입력
- 지원하지 않는 모델
- 지원하지 않는 입력 형식
- 모델 버전 불일치
- payload 크기 초과
- 인증 실패

## 12.4 기본값

- 최대 재시도: 1회
- Node RPC timeout: 모델별 설정
- 전체 요청 timeout: 5초
- retry backoff: 10~100ms 범위의 짧은 지연

실시간 추론 특성상 긴 exponential backoff는 사용하지 않는다.

---

# 13. 모델 관리

## 13.1 모델 디렉터리

각 노드는 설정 파일에서 모델 디렉터리를 지정한다.

```text
/opt/npuforge/models/
├── yolov8n/
│   ├── model.rknn
│   ├── model.toml
│   └── labels.txt
└── mobilenet_v3/
    ├── model.rknn
    └── model.toml
```

## 13.2 모델 메타데이터

예시:

```toml
id = "yolov8n"
version = "1.0.0"
backend = "rknn"
model_file = "model.rknn"
input_width = 640
input_height = 640
input_channels = 3
input_format = "rgb8"
output_format = "yolo-detections"
sha256 = "..."
```

## 13.3 모델 상태

```rust
pub enum ModelState {
    Unloaded,
    Loading,
    Ready,
    Failed,
    Draining,
}
```

## 13.4 Warmup

노드 시작 시 모델별 warmup 횟수를 설정할 수 있다.

기본 3회 수행 후 실제 요청을 받는다.

Warmup 결과는 벤치마크에서 제외한다.

---

# 14. RKNN 백엔드

## 14.1 InferenceBackend 인터페이스

```rust
#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn load_model(
        &self,
        spec: &ModelSpec,
    ) -> Result<Box<dyn LoadedModel>, BackendError>;

    fn backend_name(&self) -> &'static str;
    fn runtime_version(&self) -> Result<String, BackendError>;
}

#[async_trait::async_trait]
pub trait LoadedModel: Send + Sync {
    async fn infer(
        &self,
        input: InferenceInput,
    ) -> Result<InferenceOutput, BackendError>;

    fn model_info(&self) -> &LoadedModelInfo;
}
```

RKNN 호출 자체가 blocking이라면 `spawn_blocking` 또는 전용 worker thread를 사용한다.

## 14.2 C Wrapper

Rust가 RKNN 헤더에 직접 강하게 결합되지 않도록 최소 C Wrapper를 둔다.

C Wrapper 책임:

- context 생성 및 해제
- model load
- input set
- run
- output get
- output release
- 오류 코드 단순화

## 14.3 메모리 수명

안전 규칙:

- RKNN context는 RAII로 관리
- 출력 버퍼 해제를 Drop에서 보장
- Raw pointer는 FFI 모듈 밖으로 노출하지 않음
- 입력 버퍼는 추론 호출 종료까지 생존
- **확인 완료 (2026-08-11).** RKNN Runtime 2.3.0 의 개별 호출은 thread-safe 다.
- **그러나 `inputs_set → run → outputs_get` 시퀀스는 원자적이지 않다.**
  같은 컨텍스트를 여러 스레드가 쓰면 **API 오류 0건으로 100% 틀린 결과**가
  나온다(4스레드 × 50회, 200/200 불일치). `environment-matrix.md` §3.1 정정 참조.
- 따라서 **동시 실행 수만큼 컨텍스트를 만들어 하나씩 점유한다.**
  `npuforge-rknn` 의 `ContextPool` 이 담당하며, `RknnContext::infer` 가
  `&mut self` 를 받아 컴파일러가 동시 호출을 막는다.

## 14.4 버퍼 풀

v0.1 권장 구현:

- 입력 크기별 버퍼 풀
- 출력 구조체 재사용
- 이미지 디코딩 버퍼 재사용
- 요청마다 Vec 재할당 최소화

버퍼 풀 적용 전후를 벤치마크한다.

---

# 15. io_uring 및 데이터 복사 최적화

> ## ⛔ 판정: **적용하지 않는다** (2026-08-21, S3.9b)
>
> §15.1 의 2·3 단계(CPU profile, syscall·복사 비용)를 운영점에서 측정한
> 결과, **§15.3 의 비적용 조건 중 둘이 실제로 걸렸다.**
>
> | §15.3 조건 | 실측 |
> |---|---|
> | gRPC 직렬화가 더 큰 병목 | **걸림.** 유저 9.37 > 커널 6.99 ms/req — transport 비용의 과반이 직렬화·유저공간 copy |
> | 개선이 5% 미만 | **걸림.** syscall 진입은 transport 비용의 **1.0%**, 가장 관대한 가정(1.2MB copy 양방향 제거)으로도 8% |
>
> 그리고 더 근본적으로 — **보드 CPU 가 제약이 아니다.** 부하 중 48.9%
> idle 이고 어느 코어도 포화가 아니다. 가장 뜨거운 cpu0(softirq 68.3%)
> 조차 RPS 로 분산했을 때 **−0.2% null** 이었다(S3.5 §4.3).
> 포화되지 않은 자원의 사용량을 줄이는 것은 처리량을 올리지 않는다.
>
> ```text
> 질문   io_uring 이 남은 16.1% 를 회수하는가?
> 답     아니다. 회수 대상이 1%(관대히 8%), 게다가 CPU 는 제약이 아니다.
> ```
>
> 상태는 **"필요성 미증명" 이 아니라 "측정으로 반박됨"** 이다.
> §15.4 의 측정 항목은 전부 채워졌다.
> → [`experiments/S3_9B_NODE_RESIDUAL.md`](experiments/S3_9B_NODE_RESIDUAL.md)
>
> 아래 §15.1~15.4 는 **그 판정에 이르기까지 쓴 계획**으로 남긴다.
> 조건이 바뀌면(페이로드 축소로 CPU 가 제약이 되는 등) 다시 열린다 —
> 배제는 조건부다(`experiments/README.md` §4.5).

## 15.1 단계별 적용

1. Tokio/gRPC 기준 구현
2. CPU profile 측정
3. 네트워크 시스템 호출과 복사 비용 확인
4. 버퍼 풀 적용
5. io_uring 실험
6. 가능한 구간에서 등록 버퍼 또는 Zero-Copy 검토

## 15.2 적용 범위 후보

- Benchmark Client → Scheduler
- Scheduler → Node
- 대용량 이미지 payload 수신
- 파일 기반 데이터 세트 읽기
- 결과 저장

## 15.3 비적용 가능성

다음 조건에서는 io_uring을 적용하지 않을 수 있다.

- NPU 추론이 전체 시간 대부분을 차지
- 입력 데이터가 작음
- gRPC 직렬화가 더 큰 병목
- RKNN 입력 버퍼로 최종 복사가 필수
- 구현 복잡도 대비 개선이 5% 미만

## 15.4 측정 항목

- syscalls/request
- context switches/request
- CPU cycles/request
- memory copies/request
- requests/sec
- p95 latency
- CPU utilization

---

# 16. 설정 파일

## 16.1 Scheduler 설정

`configs/scheduler.example.toml`

```toml
[server]
grpc_listen = "0.0.0.0:50051"
http_listen = "0.0.0.0:8080"
metrics_listen = "0.0.0.0:9090"
max_payload_bytes = 10485760

[scheduler]
policy = "ect"
request_timeout_ms = 5000
node_rpc_timeout_ms = 3000
max_retries = 1

[health]
heartbeat_interval_ms = 2000
health_timeout_ms = 1000
failure_threshold = 3
recovery_threshold = 3

[thresholds]
busy_queue_depth = 8
degraded_error_rate = 0.10
degraded_temperature_c = 80.0
disable_temperature_c = 90.0

[telemetry]
json_log = true
log_level = "info"
prometheus = true
```

## 16.2 Node 설정

`configs/node.example.toml`

```toml
[node]
id = "king"
scheduler_address = "http://10.20.0.10:50051"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.21:51001"

[worker]
worker_count = 8          # 실측 확정값. environment-matrix.md §3.1
max_queue_depth = 32
queue_timeout_ms = 3000
want_float = false        # 출력 역양자화 여부. 기본 false

[backend]
type = "rknn"
runtime_library = "/usr/lib/librknnrt.so"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]
warmup_runs = 3

[telemetry]
json_log = true
log_level = "info"
prometheus = true
heartbeat_interval_ms = 1000
temperature_path = "/sys/class/thermal/thermal_zone0/temp"
```

`worker_count`와 `max_queue_depth`는 `[node]`가 아니라 `[worker]` 아래에 둔다.

`want_float = false` 면 노드가 모델 네이티브 dtype(INT8 모델은 int8)을 그대로
반환한다. 기본값을 `false` 로 둔 근거는 처리량이 아니라 **네트워크**다 —
`true` 면 출력이 입력의 3.96배가 되어 3노드 포화 시 스케줄러 RX 가
18.38 Gbps 에 이른다(`02-HARDWARE-SETUP.md` §3.3.2). 받는 쪽이 역양자화할 수
있도록 **응답 blob 은 v2 이며 텐서마다 `qnt_type`·`scale`·`zero_point` 를
싣는다.** 부수 효과로 처리량도 INT8 +17.3% / FP16 +15.7% 오른다.

`[node]`는 노드 정체성과 주소만 담당하고, 실행 동시성은 별도 섹션으로 분리한다. 노드 간 차이는 `[node]` 섹션의 세 개 값(`id`, `advertise_address`, hostname)으로 제한되어야 하므로 이 분리가 설정 diff 검증을 단순하게 만든다.

예제 IP는 `02-HARDWARE-SETUP.md` §3.2의 공식 대역인 `10.20.0.0/24`를 사용한다.

## 16.3 Benchmark 설정

`configs/benchmark.example.toml`

```toml
[target]
scheduler_address = "http://10.20.0.10:50051"
model_id = "yolov8n"

[load]
mode = "fixed-duration"
duration_seconds = 300
concurrency = 16
request_timeout_ms = 5000

[data]
directory = "./datasets/coco-sample"
shuffle = true
repeat = true

[output]
directory = "./benchmarks/results"
formats = ["jsonl", "csv"]
```

---

# 17. 메트릭

## 17.1 Scheduler 메트릭

```text
npuforge_requests_total
npuforge_requests_in_flight
npuforge_requests_failed_total
npuforge_requests_retried_total
npuforge_request_latency_seconds
npuforge_scheduler_queue_seconds
npuforge_scheduler_route_seconds
npuforge_nodes_total
npuforge_nodes_healthy
npuforge_nodes_unreachable
npuforge_node_queue_depth
npuforge_node_in_flight
npuforge_node_error_rate
```

## 17.2 Node 메트릭

```text
npuforge_node_requests_total
npuforge_node_inference_seconds
npuforge_node_preprocess_seconds
npuforge_node_postprocess_seconds
npuforge_node_temperature_celsius
npuforge_node_cpu_percent
npuforge_node_memory_percent
npuforge_node_npu_percent
npuforge_node_network_rx_bytes_total
npuforge_node_network_tx_bytes_total
```

## 17.3 레이블

허용 레이블:

- node_id
- model_id
- status
- scheduler_policy
- error_code

Request ID를 메트릭 레이블로 사용하지 않는다.

---

# 18. 로그 및 이벤트

## 18.1 로그 형식

JSON 구조화 로그를 기본으로 한다.

```json
{
  "timestamp": "2026-09-12T10:00:00.000Z",
  "level": "INFO",
  "component": "scheduler",
  "event": "request_completed",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "node_id": "queen",
  "model_id": "yolov8n",
  "latency_ms": 31.2
}
```

## 18.2 주요 이벤트

- scheduler_started
- node_registered
- node_state_changed
- model_loaded
- request_received
- request_scheduled
- request_completed
- request_failed
- request_retried
- node_removed
- node_recovered
- benchmark_started
- benchmark_completed

---

# 19. 대시보드

## 19.1 필수 화면

### Cluster Overview

- 전체 노드 수
- Healthy/Busy/Degraded/Unreachable 수
- 전체 처리량
- p50/p95/p99
- 오류율
- 현재 스케줄러 정책

### Node View

- Node ID
- 상태
- 큐 길이
- in-flight
- FPS
- 평균 추론시간
- 온도
- CPU/RAM/NPU 사용률
- 최근 오류

### Benchmark View

- 현재 시나리오
- 경과 시간
- 목표 동시성
- 실시간 처리량
- 지연시간
- 성공/실패 건수
- 노드별 분배 비율

### Event Timeline

- 장애 감지
- 노드 제외
- 복구
- 정책 변경

## 19.2 실시간 전송

SSE 또는 WebSocket 사용.

발표 데모 안정성을 고려하면 SSE를 우선 검토한다.

---

# 20. 벤치마크 설계

## 20.1 기준 모델

v0.1 기본 모델은 RKNN 변환과 실시간 성능이 안정적인 경량 객체탐지 모델을 사용한다.

우선 후보:

- YOLOv8n
- MobileNet 계열 분류 모델

최종 모델은 다음 조건으로 선정한다.

- 3개 노드에서 동일하게 실행 가능
- CPU fallback 최소
- 입력 크기 고정
- 결과 검증 가능
- 라이선스와 배포 조건 명확

## 20.2 테스트 시나리오

### 축 고정

측정 조합은 곱셈으로 증가하므로 축을 먼저 고정한다.

```text
기본 동시성 축: 1, 4, 16, 64   (4배 간격 4단계)
기본 노드 수 축: 1, 2, 3
기준 정책      : ect
비교 지점      : 동시성 16, 3노드
반복           : 5회
```

동시성은 2배 간격 7단계가 아니라 **4배 간격 4단계**로 한다. 확장 곡선의 형태를 보는 데는 4단계로 충분하며, 단계를 늘리면 총 측정시간이 실행 불가능한 수준으로 증가한다(§20.3 참조).

정책 비교와 구현 비교는 전체 동시성 축에서 반복하지 않고 **동시성 16, 3노드 한 지점**에서만 수행한다. 이 지점은 노드가 포화되기 시작하되 큐가 폭주하지 않는 구간이며, 예비 실험 결과에 따라 조정할 수 있다.

반복 5회는 축소하지 않는다. 재현성이 본 프로젝트의 핵심 산출물이므로 시간을 줄여야 한다면 반복이 아니라 축의 개수를 줄인다.

### S0. 열 특성 파악 (선행 필수, 두 조건)

NanoPi R76S는 팬리스 보드이므로 지속 부하에서 thermal throttling이 발생한다. 다른 모든 시나리오의 임계치와 cooldown 시간이 이 결과에서 나오므로 **가장 먼저 수행한다.**

**냉각 조건 두 가지를 각각 측정한다**(`02-HARDWARE-SETUP.md` §9.1).

| 조건 | 냉각 | 목적 |
|---|---|---|
| **S0-A** | 팬리스 | 실제 엣지 배치에서의 지속 성능 |
| **S0-B** | 동일 팬 3개 | throttling 없는 조건의 상한 |

두 결과의 차이가 **"냉각이 확장 효율에 미치는 영향"** 이며, 그 자체가 발표 자료가 된다.

- 노드 수: 1 (나머지 두 노드로 재현성 확인)
- 동시성: 16 고정
- 지속시간: **1,800초** (30분). 정상 상태 온도에 도달할 때까지
- 온도 임계치: 비활성화 (`disable_temperature_c`를 측정 중에는 매우 높게 설정)
- 샘플링: 1초 간격
- **전제: 세 노드의 물리 배치가 균일할 것.** 2026-08-10 측정에서 배치 차이만으로 노드 간 19°C 편차가 발생했다

측정 항목:

```text
시간에 따른 온도 곡선
시간에 따른 FPS 곡선
throttling 시작 시점 (FPS가 정상 상태 대비 5% 이상 떨어지는 시점)
peak FPS (throttling 이전)
sustained FPS (정상 상태)
성능 저하율 = 1 - (sustained FPS / peak FPS)
정상 상태 온도
idle 복귀 시간
CPU/NPU 주파수 변화
```

산출물:

- 이후 모든 시나리오에 사용할 `degraded_temperature_c` / `disable_temperature_c` 확정값
- cooldown 시간 확정값
- **peak vs sustained 성능 격차** — 벤더 스펙시트에 없는 수치이며, 발표의 핵심 자료 중 하나다

세 노드 모두에서 수행해 개체 편차를 확인한다. 편차가 크면 이후 노드별 비교에서 그만큼을 감안해야 한다.

### S1. 단일 노드 기준

- 노드 수: 1
- 동시성: 1, 4, 16, 64
- 정책: `round-robin`
- 목적: 단일 노드 최대 처리량 측정 및 확장 배율의 분모 확보
- 전제: S0에서 확정한 임계치와 cooldown 사용

### S2. 확장성

- 노드 수: 1, 2, 3
- 동시성: 1, 4, 16, 64
- 정책: `ect`
- 목적: Scale-out efficiency 측정
- 비고: S1과 합쳐 60 run

### S3. 스케줄러 비교

- 정책: `round-robin`, `least-queue`, `ect`
- 노드 수: 3 고정
- 동시성: 16 고정
- 목적: 정책 간 처리량 및 p95 차이 확인

### S4. 장애 대응

- 3노드 정상 동작
- 1노드 강제 종료
- 장애 감지
- 2노드 운영
- 노드 복구
- 재편입

### S5. 네트워크 구현 비교

- 구현: Tokio/gRPC, Tokio/gRPC + 버퍼 풀, io_uring 실험 구현, 복사 최적화 적용
- 노드 수: 3 고정
- 동시성: 16, 64
- 목적: 네트워크 경로 최적화 효과 정량화

### S6. 입력 크기 비교

- 입력: JPEG 소형, JPEG 대형, Raw RGB
- 노드 수: 3 고정
- 동시성: 16 고정
- 링크: 2.5GbE 기준, Raw RGB 조건에 한해 1GbE 비교 추가
- 목적: 입력 크기에 따른 병목 이동 확인 및 네트워크 기준선 근거 확보(§3.2)

## 20.3 실험 시간

각 run의 구성:

- Warmup: 30초
- 측정: 300초
- 반복 사이 cooldown: **최대 180초** 또는 시작 온도 도달 시점 중 빠른 쪽
- 반복: 5회

```text
1 run ≈ 30 + 300 + 180 = 510초 ≈ 8.5분  (cooldown 상한 기준 최악값)
```

팬리스 보드라 cooldown이 60초로는 부족하다. 상한을 180초로 두되, 상한에 걸린 경우 실제 시작 온도를 결과에 기록한다. 무한 대기는 총 예산을 무너뜨리므로 허용하지 않는다.

정확한 cooldown 값은 S0의 idle 복귀 시간 측정 결과로 확정한다.

## 20.4 총 측정시간 예산

축을 확정하기 전에 총 소요시간을 계산한다.

| 시나리오 | 조합 | run 수 | 소요시간 |
|---|---|---:|---:|
| **S0-A** (팬리스) | 노드 3 × 1,800초 + cooldown | 3 | 약 1.8시간 |
| **S0-B** (냉각) | 노드 3 × 1,800초 + cooldown | 3 | 약 1.8시간 |
| S1 + S2 | 노드 3 × 동시성 4 × 반복 5 | 60 | 약 8.5시간 |
| S3 | 정책 3 × 반복 5 | 15 | 약 2.1시간 |
| S4 | 장애 시나리오 × 반복 5 | 5 | 약 0.7시간 |
| S5 | 구현 4 × 동시성 2 × 반복 5 | 40 | 약 5.7시간 |
| S6 | 입력 3 × 반복 5 + 1GbE 비교 5 | 20 | 약 2.8시간 |
| **합계** | | **146** | **약 23.4시간** |

**S1~S6은 냉각 조건 하나에서만 수행한다.** 두 조건 전체를 반복하면 46시간이 되어 실행이 불가능하다.

기본 측정 조건은 **S0 결과를 보고 정한다.** 팬리스에서 throttling이 심해 확장 효율 측정이 오염된다면 냉각 조건을 기본으로 삼고, 팬리스는 S0 결과로만 제시한다.

팬리스 유지 결정으로 cooldown이 60초에서 180초로 늘어나 총 예산이 16시간에서 22시간으로 증가했다. 11월 1일부터 15일 사이에 야간 무인 실행을 전제하면 여전히 소화 가능하지만, 여유가 줄었다. S0 결과에서 실제 idle 복귀 시간이 180초보다 짧게 나오면 예산을 다시 계산한다.

참고로 동시성을 7단계(1/2/4/8/16/32/64)로 두고 S2와 S3를 전체 조합으로 돌리면 이 두 시나리오만 315 run, 약 45시간이 되어 실행이 불가능하다.

### 무인 실행 요구사항

총 16시간은 대화형으로 진행할 수 없다.

`run-benchmark.sh`는 다음을 만족해야 한다.

- 시나리오 목록을 파일로 받아 순차 실행
- run 사이 cooldown 및 온도 안정화 대기 자동 처리
- 개별 run 실패 시 전체 중단 없이 기록 후 계속 진행
- 각 run 종료 시 원본 결과 즉시 flush
- 재현 메타데이터(§2.5) 자동 수집
- 중단 지점부터 재개 가능

## 20.5 결과 계산

### 확장 배율

```text
scale_factor(N) =
throughput(N nodes) / throughput(1 node)
```

### 확장 효율

```text
scale_efficiency(N) =
throughput(N nodes) /
(throughput(1 node) × N)
```

### 비용당 처리량

```text
cost_efficiency =
throughput / total_hardware_cost
```

### 에너지당 처리량

```text
energy_efficiency =
requests / watt-hour
```

## 20.6 통계

- 평균
- 중앙값
- 표준편차
- p50
- p95
- p99
- 최소/최대
- 95% 신뢰구간은 가능할 경우 제공

---

# 21. 오류 코드

예시:

```text
NPF-0000 OK
NPF-1001 INVALID_REQUEST
NPF-1002 PAYLOAD_TOO_LARGE
NPF-1003 UNSUPPORTED_INPUT_FORMAT
NPF-1101 MODEL_NOT_FOUND
NPF-1102 MODEL_VERSION_MISMATCH
NPF-1201 NO_AVAILABLE_NODE
NPF-1202 DEADLINE_UNSATISFIABLE
NPF-1301 NODE_TIMEOUT
NPF-1302 NODE_UNAVAILABLE
NPF-1303 NODE_OVERLOADED
NPF-1401 BACKEND_ERROR
NPF-1402 INFERENCE_FAILED
NPF-1501 INTERNAL_ERROR
```

오류 코드는 외부 API에서 안정적으로 유지한다.

---

# 22. 보안

v0.1 기본 전제는 신뢰 가능한 로컬 네트워크다.

최소 구현:

- 최대 payload 크기 제한
- 입력 형식 검증
- Node registration token
- 관리 API token
- 로그에 원본 이미지와 민감 데이터 미기록
- 디렉터리 traversal 방지
- 모델 경로 allowlist
- 프로세스는 root가 아닌 전용 사용자로 실행

선택 구현:

- mTLS
- TLS
- API key
- 모델 서명 검증

---

# 23. 배포

## 23.1 systemd

기본 배포 방식이다.

서비스:

```text
npuforge-scheduler.service
npuforge-node.service
npuforge-dashboard.service
```

## 23.2 Docker

Scheduler와 Dashboard는 Docker 지원 가능하다.

Node는 RKNN Runtime과 장치 접근 문제로 host 설치를 기본으로 한다.

## 23.3 설치 스크립트

```bash
./deploy/scripts/install-scheduler.sh
./deploy/scripts/install-node.sh
./deploy/scripts/install-dashboard.sh
```

스크립트는 idempotent하게 작성한다.

---

# 24. 테스트 전략

## 24.1 단위 테스트

- 스케줄링 점수 계산
- 상태 전이
- 재시도 판단
- 설정 파싱
- 오류 변환
- 통계 계산

## 24.2 통합 테스트

Mock Backend를 이용한다.

시나리오:

- 3개 Mock Node 등록
- 처리속도 차이
- 오류율 차이
- 노드 장애
- 복구
- timeout
- 재시도

## 24.3 하드웨어 테스트

RK3576 실장비에서 수행한다.

- 모델 로딩
- 반복 추론 안정성
- 1시간 지속 부하
- 온도 상승
- 네트워크 단절
- 프로세스 강제 종료
- 재시작 후 자동 등록

## 24.4 CI

GitHub Actions:

- fmt
- clippy
- unit test
- mock integration test
- build
- dependency audit

RKNN 하드웨어 테스트는 self-hosted runner 또는 수동 테스트로 분리한다.

---

# 25. 성능 프로파일링

도구 후보:

- perf
- flamegraph
- bpftrace
- strace
- sar
- pidstat
- ethtool
- iperf3
- tcpdump
- valgrind massif는 필요 시 제한적으로 사용

측정 단계:

1. Scheduler CPU flamegraph
2. Node CPU flamegraph
3. 네트워크 throughput
4. syscall 빈도
5. context switch
6. memory allocation
7. copy 구간
8. NPU 추론시간
9. 온도와 throttling

---

# 26. 개발 마일스톤

## M0. 저장소 및 환경

완료 조건:

- Rust workspace 생성
- 기본 CI
- 문서 구조
- 라이선스
- Mock Backend 동작

## M1. 단일 노드 추론

완료 조건:

- RKNN FFI
- 모델 로딩
- 이미지 1건 추론
- 반복 추론
- timing 측정

## M2. 원격 추론

완료 조건:

- Scheduler → Node gRPC
- 단일 노드 원격 추론
- 오류 처리
- 기본 메트릭

## M3. 다중 노드

완료 조건:

- 3개 노드 등록
- Round Robin
- 1/2/3노드 벤치마크

## M4. 동적 스케줄링

완료 조건:

- Least Queue
- ECT
- 정책 비교

## M5. 장애 복구

완료 조건:

- 헬스체크
- 자동 제외
- 요청 재시도
- 자동 복귀

## M6. 대시보드

완료 조건:

- 실시간 노드 상태
- 처리량
- 지연시간
- 장애 이벤트

## M7. 최적화 실험

완료 조건:

- 버퍼 풀
- profile 결과
- io_uring 적용 여부 결정
- 적용 시 기준 구현과 비교

## M8. 발표 릴리스

완료 조건:

- v0.1 tag
- README
- 설치 스크립트
- 벤치마크 원본
- 발표자료
- 데모 영상

---

# 27. 일정

## 2026년 8월

- `00-PRD.md`
- `01-TECHSPEC.md`
- 저장소 초기화
- Mock Backend
- Rust-RKNN FFI 최소 검증

## 2026년 9월

- 단일 노드 원격 추론
- 3노드 등록
- Round Robin
- Benchmark CLI
- 예비 결과

## 2026년 10월

- Least Queue 및 ECT scheduling
- 헬스체크
- 장애 제외 및 복구
- 메트릭
- 대시보드
- 기준 성능 확정

## 2026년 11월 1일~15일

- 프로파일링
- 버퍼 최적화
- io_uring 실험
- 최종 벤치마크
- 문서화

## 2026년 11월 16일~22일

- 기능 동결
- 발표 자료
- 데모 영상
- 리허설

## 2026년 11월 28일

- FOSS for All Conference 발표
- NPUDure v0.1 공개

---

# 28. 범위 통제

11월 발표 이전에는 다음 기능을 추가하지 않는다.

- Kubernetes 연동
- 다중 Scheduler HA
- PostgreSQL 기반 상태 저장
- 사용자 계정
- 과금
- 자동 모델 변환
- 모델 자동 배포
- LLM 텐서 병렬
- 모델 레이어 분할
- Hailo/Jetson 백엔드
- WAN 클러스터
- 모바일 앱
- 복잡한 권한 관리

추가 요청은 v0.2 Backlog로 이동한다.

---

# 29. 발표 데모 구성

## 29.1 메인 화면

- NPUDure 로고 및 버전
- Node 1/2/3 상태
- 노드별 FPS
- 전체 FPS
- p95 latency
- 확장 배율
- 확장 효율
- 온도
- 큐 길이

## 29.2 데모 순서

1. 단일 노드 실행
2. 2노드 추가
3. 3노드 추가
4. 처리량 증가 확인
5. Round Robin과 ECT 비교
6. Node 2 프로세스 종료
7. 자동 제외
8. 2노드로 서비스 지속
9. Node 2 재시작
10. 자동 복귀

## 29.3 실패 대비

- 동일 시나리오 녹화 영상
- 사전 생성 벤치마크 결과
- 네트워크 없이 동작하는 로컬 시뮬레이션
- Mock Backend 모드

---

# 30. 완료 정의

NPUDure v0.1은 다음 조건을 모두 충족할 때 완료로 간주한다.

- RK3576 NPU 3노드 동작
- Rust Scheduler 동작
- 단일/2/3노드 벤치마크
- 3노드 확장 배율 및 확장 효율 측정, 목표 미달 시 원인 분석
- Round Robin, Least Queue, ECT 비교
- 노드 장애 자동 감지
- 장애 노드 제외
- 서비스 지속
- 노드 자동 복귀
- p50/p95/p99 제공
- 원본 결과 공개
- 설치 문서 공개
- GitHub 공개
- 발표 가능한 안정적 데모
- 2026년 11월 FOSS for All Conference 발표 준비 완료

---

# 31. 최종 기술 정의

NPUDure는 여러 엣지 NPU를 물리적으로 결합하는 기술이 아니다.

NPUDure는 독립적인 추론 요청을 여러 NPU 노드에 분산하고, 각 노드의 부하와 상태를 기준으로 요청을 스케줄링하며, 장애 발생 시 서비스를 지속하고, 실제 성능 손실과 확장 효율을 재현 가능하게 측정하는 Linux/Rust 기반 오픈소스 분산 추론 런타임이다.
