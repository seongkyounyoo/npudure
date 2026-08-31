<a id="index"></a>

# NPUForge 문서 묶음

> **이 파일은 생성물이다. 직접 편집하지 않는다.**
> `docs/` 의 원본 26개를 읽기·인쇄·검토용으로 이어 붙인 것이다.
> 고칠 것이 있으면 원본을 고치고 다시 만든다.
>
> ```bash
> python scripts/build-docs-bundle.py $(git log -1 --format=%cs -- docs/)
> ```
>
> - 생성 기준: **2026-08-28** (`docs/` 최종 커밋일)
> - 파일 간 링크는 문서 내 앵커로 바뀌어 있다
> - `docs/` 밖을 가리키는 링크(`../results/...`)는 그대로다
> - **세션 인수인계 메모(`handoff-*.md`)와 공개 준비 문서(`public/`)는 빠져 있다.**
>   연구 산출물이 아니다
>
> 아키텍처 결정 기록은 별도 묶음이다 — [`adrs/ALL.md`](../adrs/ALL.md)

## 차례

- [NPUForge Product Requirements Document](#00-prd)  ·  `docs/00-PRD.md`
- [NPUForge Technical Specification](#01-techspec)  ·  `docs/01-TECHSPEC.md`
- [NPUForge Hardware Setup Guide](#02-hardware-setup)  ·  `docs/02-HARDWARE-SETUP.md`
- [NPUForge Development Requirements](#03-development-requirements)  ·  `docs/03-DEVELOPMENT-REQUIREMENTS.md`
- [기술 용어 정리 (Glossary)](#glossary)  ·  `docs/GLOSSARY.md`
- [NPUForge 인프라 현황](#infrastructure)  ·  `docs/infrastructure.md`
- [NPUForge Environment Matrix](#environment-matrix)  ·  `docs/environment-matrix.md`
- [호스트 인벤토리](#hosts-readme)  ·  `docs/hosts/README.md`
- [호스트 인벤토리 — server](#hosts-server-i7-4790-20260826)  ·  `docs/hosts/server-i7-4790-20260826.md`
- [호스트 인벤토리 — Dell PowerEdge R620 (구 스케줄러 서버)](#hosts-server-xeon-e5-2630l-20260826)  ·  `docs/hosts/server-xeon-e5-2630l-20260826.md`
- [실험 대장 (Experiment Index)](#experiments-readme)  ·  `docs/experiments/README.md`
- [S0-C — Scheduling Policy A/B (팬리스)](#experiments-s0-c-policy-ab)  ·  `docs/experiments/S0_C_POLICY_AB.md`
- [S0-D — Capacity Heterogeneity (결정론적 이질)](#experiments-s0-d-capacity-hetero)  ·  `docs/experiments/S0_D_CAPACITY_HETERO.md`
- [S0 — Sustained Load (조건 A 팬리스 / 조건 B 능동 냉각)](#experiments-s0-sustained-load)  ·  `docs/experiments/S0_SUSTAINED_LOAD.md`
- [S2 — gRPC Multi-node Scaling Baseline](#experiments-s2-grpc-baseline)  ·  `docs/experiments/S2_GRPC_BASELINE.md`
- [S3.5 — Transport Cost Profiling](#experiments-s3-5-transport-profile)  ·  `docs/experiments/S3_5_TRANSPORT_PROFILE.md`
- [S3.6 — HTTP/2 Window × Connections-per-Node A/B](#experiments-s3-6-h2-channel-ab)  ·  `docs/experiments/S3_6_H2_CHANNEL_AB.md`
- [S3.7 — Connection Tuning (a: sweep, b: concurrency, c: RPS)](#experiments-s3-7-connection-tuning)  ·  `docs/experiments/S3_7_CONNECTION_TUNING.md`
- [S3.8 — Optimized gRPC Scale-out](#experiments-s3-8-optimized-scaleout)  ·  `docs/experiments/S3_8_OPTIMIZED_SCALEOUT.md`
- [S3.9a — Scale-out Efficiency Loss Profiling](#experiments-s3-9a-scaleout-profile)  ·  `docs/experiments/S3_9A_SCALEOUT_PROFILE.md`
- [S3.9b — Node-side Residual Cost Profiling](#experiments-s3-9b-node-residual)  ·  `docs/experiments/S3_9B_NODE_RESIDUAL.md`
- [S3 — Per-configuration Saturation](#experiments-s3-saturation)  ·  `docs/experiments/S3_SATURATION.md`
- [NPUForge 측정 결과 — 1차 정리](#results)  ·  `docs/RESULTS.md`
- [NPUForge 기술 논의](#discuss)  ·  `docs/discuss.md`
- [NPUForge 보드 작업 로그](#board-worklog)  ·  `docs/board-worklog.md`
- [NPUForge 진행 현황](#todo)  ·  `docs/TODO.md`

---

<a id="00-prd"></a>

# NPUForge Product Requirements Document

- 문서명: `00-PRD.md`
- 문서 버전: v0.2
- 프로젝트명: NPUForge
- 프로젝트 유형: 오픈소스 분산 엣지 NPU 추론 런타임
- 개발 언어: Rust
- 대상 플랫폼: Linux / RK3576 기반 엣지 장치
- 목표 발표: 2026년 11월 FOSS for All Conference
- 목표 공개 버전: NPUForge v0.1
- 문서 상태: Draft
- 작성일: 2026-08-05
- 최종 수정: 2026-08-06 (본문)
- 2026-08-27: 최종 상태를 가리키는 배너만 추가. **본문은 계획 기준선 그대로 둔다**
- 관련 문서:
  - `01-TECHSPEC.md`
  - `02-HARDWARE-SETUP.md`
  - `03-DEVELOPMENT-REQUIREMENTS.md`
  - `environment-matrix.md`

> ## ⚠️ 이 문서는 **Phase 1 계획 기준선**이다
>
> 여기 적힌 연구 질문·일정·후보 기능은 **측정을 시작하기 전에 정한 것**이다.
> 계획을 사후에 고치면 "무엇을 예상했고 무엇이 빗나갔는가" 가 사라지므로
> 그대로 둔다.
>
> **실제로 무엇이 닫혔고 무엇이 열려 있는지는 여기가 아니다.**
>
> | 무엇 | 어디 |
> |---|---|
> | 실험 최종 상태 | [`experiments/README.md`](#experiments-readme) §5~§7 |
> | io_uring 판정 (**적용하지 않는다**) | [`01-TECHSPEC.md`](#01-techspec) §15 |
> | 확정 수치 | [`RESULTS.md`](#results) · [`experiments/`](experiments) |
>
> 특히 이 문서가 비교 실험으로 들고 있는 **io_uring 은 구현하지 않기로
> 했다.** S3.9b 가 회수 가능한 몫을 ≈8% 로 측정했고, 그 근거로 배제했다.

---

# 0. 문서 역할과 우선순위

문서가 서로 다른 값을 기술할 경우 다음 우선순위를 따른다.

| 영역 | 규범 문서 |
|---|---|
| 목표, 비목표, 기능 요구사항, 성공 기준 | `00-PRD.md` |
| 저장소 구조, 프로토콜, 설정 스키마, 스케줄링 알고리즘, 오류 코드 | `01-TECHSPEC.md` |
| 물리 구성, 네트워크, 전원, 냉각, 실험 조건 | `02-HARDWARE-SETUP.md` |
| 개발환경, 도구, 배포 자동화, 라이선스 | `03-DEVELOPMENT-REQUIREMENTS.md` |
| 버전 조합 및 해시 고정 | `environment-matrix.md` |

본 문서는 "왜"와 "무엇을"만 다룬다.

알고리즘 계산식, 크레이트 이름, 설정 파일 키, 식별자 문자열은 본 문서에 기술하지 않고 `01-TECHSPEC.md`를 참조한다. 같은 내용을 두 문서에 복제하지 않는 것이 문서 정합성 유지의 유일한 방법이다.

---

# 1. 프로젝트 개요

NPUForge는 여러 대의 저비용 엣지 NPU 장치를 하나의 분산 추론 자원처럼 운영하기 위한 Rust 기반 오픈소스 런타임이다.

1차 구현에서는 6 TOPS급 RK3576 NPU 장치 3대를 네트워크로 연결하고, 단일 장치 대비 2대 및 3대 구성에서 실제 추론 처리량이 어느 정도까지 확장되는지 측정한다.

본 프로젝트는 단순히 각 장치의 명목 TOPS를 합산하는 것을 목표로 하지 않는다. 실제 애플리케이션 환경에서 발생하는 네트워크 지연, 메모리 복사, 요청 스케줄링, 전처리 및 후처리, 노드 장애 등으로 인한 손실을 정량적으로 측정하고 분석하는 것을 핵심 목표로 한다.

---

# 2. 문제 정의

엣지 AI 장치는 일반적으로 장치별로 독립 운영된다. 여러 장치가 존재하더라도 각각의 NPU 자원이 통합되지 않아 일부 장치는 과부하 상태이고 다른 장치는 유휴 상태인 상황이 발생할 수 있다.

또한 제조사가 제공하는 TOPS 수치는 이론적 최대 연산량이므로 실제 모델의 종단 간 추론 성능을 직접 나타내지 않는다.

예를 들어 6 TOPS NPU 세 대를 연결하더라도 다음과 같은 문제로 인해 실제 성능은 18 TOPS에 미치지 못할 수 있다.

- 네트워크 데이터 전송 지연
- 입력 및 출력 버퍼 복사
- 이미지 디코딩과 전처리 비용
- NPU 입력 메모리 변환
- 지원되지 않는 연산의 CPU 실행
- 요청 분배 불균형
- 노드별 온도 및 성능 편차
- 동시 요청 부족
- 장애 노드로 인한 요청 실패
- 중앙 스케줄러 병목

현재 저비용 NPU 여러 대를 실제로 연결했을 때 나타나는 확장 효율, 비용 효율, 장애 대응 능력을 재현 가능한 형태로 검증할 수 있는 공개 소프트웨어와 실험 자료가 부족하다.

---

# 3. 프로젝트 목표

## 3.1 핵심 목표

NPUForge v0.1의 핵심 목표는 다음과 같다.

1. RK3576 기반 NPU 노드 3대를 하나의 추론 클러스터로 구성한다.
2. Rust 기반 중앙 스케줄러가 각 노드에 추론 요청을 분배한다.
3. 단일, 2노드, 3노드 구성의 실제 처리량과 지연시간을 비교한다.
4. 노드 상태와 대기열을 고려한 동적 요청 분배를 구현한다.
5. 장애 노드를 자동으로 제외하고 복구 후 다시 편입한다.
6. 시스템의 성능과 상태를 실시간으로 확인할 수 있는 대시보드를 제공한다.
7. 설치 방법, 소스코드, 실험 조건, 벤치마크 결과를 공개한다.
8. 2026년 11월 FOSS for All Conference에서 동작 데모와 실험 결과를 발표한다.

## 3.2 연구 목표

다음 질문에 실험적으로 답하는 것을 목표로 한다.

- 6 TOPS NPU 세 대의 실제 처리량은 단일 장치 대비 몇 배 증가하는가?
- 노드 수 증가에 따른 확장 효율은 어느 수준인가?
- 단순 Round Robin과 부하 기반 스케줄링의 차이는 얼마인가?
- 네트워크와 메모리 복사가 전체 지연시간에서 차지하는 비율은 얼마인가?
- Tokio 기반 네트워크 처리와 io_uring 기반 처리 사이에 의미 있는 차이가 있는가?
- 입력 크기와 동시 요청 수에 따라 병목 지점은 어떻게 달라지는가?
- 노드 장애 발생 시 서비스 처리량과 지연시간은 어떻게 변화하는가?
- 저가형 다중 NPU 구성은 단일 고성능 엣지 가속기 대비 어떤 조건에서 유리한가?

---

# 4. 비목표

NPUForge v0.1에서는 다음 항목을 목표로 하지 않는다.

- 여러 NPU를 하나의 물리적 NPU처럼 보이게 만드는 하드웨어 수준 통합
- 단일 추론 요청의 지연시간을 노드 수에 비례하여 단축
- 하나의 대규모 모델을 여러 노드에 레이어 단위로 분할
- 대규모 언어모델의 텐서 병렬 및 파이프라인 병렬 구현
- Kubernetes 수준의 범용 클러스터 오케스트레이션
- 모든 NPU 제조사와 런타임 지원
- 완전한 무복사 데이터 경로 보장
- 상용 SLA 및 보안 인증
- 인터넷 환경의 광역 분산 추론
- 모바일 및 Windows 클라이언트 지원
- 최종 제품 수준의 사용자 관리와 과금 시스템

v0.1은 독립적인 추론 요청을 여러 NPU 노드가 병렬로 처리하는 데이터 병렬 구조에 집중한다.

---

# 5. 목표 사용자

## 5.1 주요 사용자

### 엣지 AI 개발자

저비용 ARM 보드와 NPU를 이용하여 다중 카메라 또는 다중 요청 추론 시스템을 개발하려는 사용자.

### 임베디드 Linux 개발자

Linux 네트워크, 장치 드라이버, NPU 런타임, Rust 기반 시스템 소프트웨어에 관심이 있는 사용자.

### AI 시스템 연구자

분산 엣지 추론의 성능, 지연시간, 에너지 효율, 확장성 등을 실험하려는 연구자.

### 산업용 AI 솔루션 개발자

공장, 설비, 출입 시스템, CCTV 등 다수의 영상 또는 센서 데이터를 현장에서 처리하려는 개발자.

## 5.2 부사용자

- Rust 개발자
- RK3576 보드 사용자
- 오픈소스 기여자
- 대학원생 및 연구실
- 산업용 게이트웨이 제조사
- Edge AI 플랫폼 개발사

---

# 6. 주요 사용 시나리오

## 6.1 다중 이미지 추론

사용자가 여러 이미지를 NPUForge에 전송하면 스케줄러가 사용 가능한 NPU 노드에 요청을 분배한다.

```text
Client
  → Scheduler
      → NPU Node 1
      → NPU Node 2
      → NPU Node 3
```

각 노드는 독립적으로 추론을 수행하고 결과를 스케줄러에 반환한다.

## 6.2 다중 카메라 분석

여러 카메라에서 입력되는 프레임을 각 NPU 노드에 분산한다.

예상 적용 분야:

- 공장 안전 모니터링
- 설비 이상 탐지
- 출입 인원 분석
- 객체 탐지
- 불량 검사
- 다중 CCTV 분석

## 6.3 노드 장애 대응

클러스터 운영 중 특정 노드가 중단되면 스케줄러는 해당 노드를 요청 대상에서 자동 제외한다.

노드가 다시 정상 상태가 되면 일정 횟수의 헬스체크 성공 후 클러스터에 재편입한다.

## 6.4 성능 비교

사용자는 동일한 모델과 데이터 세트를 이용하여 다음 구성을 비교할 수 있다.

- 단일 노드
- 2노드
- 3노드
- Round Robin 스케줄링
- 부하 기반 스케줄링
- Tokio 기반 네트워크 처리
- io_uring 기반 네트워크 처리
- 데이터 복사 최적화 전후

---

# 7. 핵심 기능 요구사항

## FR-01. NPU 노드 등록

각 NPU 노드는 시작 시 중앙 스케줄러에 자신의 정보를 등록해야 한다.

등록 정보:

- Node ID
- IP 주소
- 포트
- 장치 유형
- NPU 유형
- NPU 코어 수
- 모델 목록
- 모델 버전
- 런타임 버전
- 메모리 용량
- 소프트웨어 버전

## FR-02. 헬스체크

스케줄러는 일정 주기로 각 노드의 상태를 확인해야 한다.

상태 구분:

- Registering
- Healthy
- Busy
- Degraded
- Unreachable
- Recovering
- Draining
- Disabled

`Draining`과 `Disabled`는 운영자가 명시적으로 전환하는 상태이며 FR-15에서 정의한다.

상태 전이 조건과 임계치는 `01-TECHSPEC.md` §9에서 정의한다.

헬스체크 정보:

- 응답 시간
- 현재 큐 길이
- 최근 추론 성공률
- CPU 사용률
- 메모리 사용률
- NPU 사용률
- 장치 온도
- 최근 오류

## FR-03. 추론 요청 API

클라이언트는 네트워크 API를 통해 추론 요청을 제출할 수 있어야 한다.

요청 정보:

- Request ID
- Model ID
- 입력 데이터
- 입력 형식
- 우선순위
- 요청 제한시간
- 추적 정보

응답 정보:

- Request ID
- 결과 데이터
- 처리 노드
- 대기시간
- 전처리 시간
- 추론 시간
- 후처리 시간
- 전체 처리시간
- 오류 코드

## FR-04. Round Robin 스케줄링

기본 스케줄러는 정상 상태의 노드에 요청을 순차적으로 분배해야 한다.

Round Robin은 다른 스케줄링 정책의 비교 기준으로 사용한다.

## FR-05. 부하 기반 스케줄링

스케줄러는 각 노드의 큐 길이와 최근 처리시간을 기준으로 요청을 분배할 수 있어야 한다.

기본 계산 요소:

- 현재 큐 길이
- 처리 중인 요청 수
- 이동 평균 추론시간
- 이동 평균 네트워크 시간
- 최근 오류율
- 노드 온도
- 노드 상태

정책 종류, 점수 계산식, 설정 및 CLI 식별자는 `01-TECHSPEC.md` §10에서 정의한다.

본 문서에서 "부하 기반 스케줄링"은 Round Robin이 아닌 정책 전체를 가리키는 산문상의 총칭이며, 설정값이나 CLI 인자로 사용하지 않는다.

## FR-06. 장애 노드 자동 제외

다음 조건 중 하나가 충족되면 노드를 요청 분배 대상에서 제외해야 한다.

- 연속 헬스체크 실패
- 추론 요청 제한시간 초과
- 오류율 임계치 초과
- 온도 임계치 초과
- 런타임 비정상 종료

## FR-07. 노드 자동 복귀

제외된 노드가 연속 헬스체크에 성공하면 Recovering 상태를 거쳐 다시 Healthy 상태로 전환해야 한다.

복구 직후에는 제한된 요청만 할당하여 안정성을 확인해야 한다.

## FR-08. 재시도

추론 요청이 실패한 경우 다른 정상 노드로 제한된 횟수만큼 재시도할 수 있어야 한다.

중복 실행으로 인한 문제를 방지하기 위해 Request ID 기반 중복 처리를 관리해야 한다.

## FR-09. 모델 관리

각 노드가 어떤 모델을 실행할 수 있는지 스케줄러가 확인할 수 있어야 한다.

v0.1에서는 모델 파일의 자동 배포보다 다음 기능을 우선한다.

- 모델 식별
- 버전 확인
- 모델 로딩 상태 확인
- 모델별 요청 라우팅
- 모델 불일치 감지

## FR-10. 메트릭 수집

시스템은 다음 메트릭을 수집해야 한다.

- 전체 requests/sec
- 노드별 requests/sec
- 전체 FPS
- 노드별 FPS
- p50 지연시간
- p95 지연시간
- p99 지연시간
- 오류율
- 재시도율
- 노드별 큐 길이
- CPU 사용률
- 메모리 사용률
- NPU 사용률
- 네트워크 송수신량
- 온도
- 노드 가용률

## FR-11. 실시간 대시보드

사용자는 웹 브라우저를 통해 시스템 상태를 확인할 수 있어야 한다.

대시보드 주요 화면:

- 클러스터 전체 상태
- 노드별 상태
- 전체 처리량
- 노드별 처리량
- 지연시간 분포
- 현재 큐 길이
- 장애 및 복구 이벤트
- 시스템 구성
- 벤치마크 실행 상태

## FR-12. 벤치마크 실행 도구

동일한 조건으로 실험을 반복할 수 있는 CLI 도구를 제공해야 한다.

CLI 옵션 예시:

```bash
npuforge-bench \
  --model yolov8n \
  --dataset ./dataset \
  --concurrency 16 \
  --duration 300 \
  --scheduler ect
```

정확한 인자 목록과 정책 식별자는 `01-TECHSPEC.md` §10.0 및 `03-DEVELOPMENT-REQUIREMENTS.md` §3.1을 따른다.

출력 형식:

- 콘솔 요약
- JSON
- CSV

## FR-13. 단계별 지연시간 측정

각 요청은 다음 처리 시간을 분리하여 기록해야 한다.

- Client transmission
- Scheduler queue
- Scheduler routing
- Node reception
- Decode
- Preprocess
- NPU input preparation
- NPU inference
- Postprocess
- Result transmission
- End-to-end latency

## FR-14. 이벤트 로그

다음 이벤트를 구조화된 로그로 기록해야 한다.

- 노드 등록
- 노드 연결 종료
- 헬스체크 실패
- 장애 노드 제외
- 노드 복구
- 요청 재시도
- 제한시간 초과
- 모델 불일치
- 스케줄러 정책 변경

## FR-15. 노드 Drain 및 Disable

운영자는 노드를 물리적으로 중단하지 않고 요청 분배 대상에서 제외할 수 있어야 한다.

- **Drain**: 신규 요청 할당을 중지하고, 처리 중인 요청이 완료될 때까지 대기한다.
- **Disable**: 즉시 후보에서 제외한다.
- **Enable**: 다시 후보로 편입한다.

공식 벤치마크의 1노드, 2노드, 3노드 비교는 전원 차단이나 프로세스 종료가 아니라 이 기능으로 수행한다.

노드 수를 바꾸더라도 전원, 온도, 네트워크, 장비 배치 조건이 동일하게 유지되어야 실험 조건이 성립하기 때문이다. 상세 근거는 `02-HARDWARE-SETUP.md` §12.3을 따른다.

장애 실험(§11.6)에서의 강제 종료 및 네트워크 차단은 이와 별개이며, 자동 감지 동작을 검증하기 위한 것이다.

---

# 8. 비기능 요구사항

## NFR-01. 성능

- 3노드 확장 효율 80% 이상, 즉 단일 노드 대비 총 처리량 2.4배 이상을 1차 목표로 한다.
- 3노드 확장 효율 85% 이상을 최종 목표로 한다.
- 정상 부하에서 스케줄러 자체 CPU 사용률이 전체 시스템 병목이 되지 않아야 한다.
- 중앙 스케줄러에서 발생하는 라우팅 오버헤드는 전체 지연시간의 5% 이하를 목표로 한다.

확장 효율은 다음과 같이 계산한다.

```text
확장 효율 =
3노드 총 처리량 /
(단일 노드 처리량 × 3)
```

본 절의 수치는 **목표치이며 성공 조건이 아니다.**

목표 미달 자체는 실패가 아니다. 성공 여부의 판단 기준은 §12.1에 따른다.

## NFR-02. 신뢰성

- 단일 노드 장애가 전체 서비스 중단으로 이어지지 않아야 한다.
- 장애 노드 감지 후 설정된 시간 내 요청 분배 대상에서 제외해야 한다.
- 실패한 요청은 다른 노드에서 재시도할 수 있어야 한다.
- 노드 재기동 후 수동 개입 없이 복구할 수 있어야 한다.

## NFR-03. 재현성

- 모든 벤치마크 조건을 설정 파일로 저장해야 한다.
- 사용한 모델, 데이터 세트, 런타임, 커널, 보드 정보가 기록돼야 한다.
- 동일 조건에서 반복 실험할 수 있어야 한다.
- 결과 원본을 JSON 또는 CSV로 보존해야 한다.

## NFR-04. 이식성

코어 스케줄러는 특정 NPU 런타임에 직접 종속되지 않아야 한다.

백엔드 인터페이스를 통해 NPU 런타임을 분리한다.

```text
InferenceBackend
  ├─ RKNN Backend
  ├─ CPU Mock Backend
  └─ Future Backend
```

## NFR-05. 보안

v0.1에서는 폐쇄된 로컬 네트워크 환경을 기준으로 한다.

최소 요구사항:

- 요청 크기 제한
- 비정상 입력 검증
- 노드 등록 토큰
- 관리 API 접근 제한
- 로그 내 민감정보 제외

TLS와 사용자 인증은 선택 기능으로 두되, 외부 네트워크 공개 시 필수로 전환한다.

## NFR-06. 오픈소스 품질

- 명확한 라이선스를 적용한다.
- README에 설치 및 실행 방법을 제공한다.
- 샘플 설정 파일을 제공한다.
- 최소 한 개의 재현 가능한 데모 시나리오를 제공한다.
- 주요 모듈에 단위 테스트를 제공한다.
- GitHub Actions 또는 자체 CI를 구성한다.

---

# 9. 기술 구성

## 9.1 전체 구조

```text
┌──────────────────────┐
│ Benchmark Client     │
│ Demo Web Client      │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ NPUForge Scheduler   │
│                      │
│ · API Gateway        │
│ · Node Registry      │
│ · Health Monitor     │
│ · Load Scheduler     │
│ · Metrics Collector  │
└──────┬──────┬────────┘
       │      │
       ▼      ▼
┌──────────┐ ┌──────────┐ ┌──────────┐
│ Node 01  │ │ Node 02  │ │ Node 03  │
│ RK3576   │ │ RK3576   │ │ RK3576   │
│ RKNN NPU │ │ RKNN NPU │ │ RKNN NPU │
└──────────┘ └──────────┘ └──────────┘
```

## 9.2 주요 컴포넌트

### npuforge-scheduler

중앙 스케줄러.

주요 책임:

- API 요청 수신
- 노드 등록 및 상태 관리
- 요청 스케줄링
- 실패 요청 재시도
- 메트릭 수집
- 이벤트 기록

### npuforge-node

각 RK3576 보드에서 실행되는 추론 에이전트.

주요 책임:

- RKNN 모델 로딩
- 추론 요청 수신
- 전처리
- NPU 추론
- 후처리
- 상태 및 메트릭 보고

### npuforge-bench

성능 측정 CLI.

주요 책임:

- 동시 요청 생성
- 테스트 데이터 반복 전송
- 부하 패턴 설정
- 결과 저장
- 기본 통계 계산

### npuforge-dashboard

클러스터 상태와 실험 결과를 표시하는 웹 UI.

### npuforge-common

공통 데이터 모델, 오류 코드, 설정 구조체를 포함한다.

### npuforge-mock-backend

실제 NPU 없이 스케줄링, 장애 감지, 복구, 대시보드, CI 통합 테스트를 검증하기 위한 소프트웨어 백엔드.

추론시간, 편차, 오류율, 큐 제한을 설정으로 조절한다.

외부 사용자가 RK3576 장비 없이도 NPUForge의 핵심 구조를 실행할 수 있어야 하므로 부가 기능이 아닌 필수 구성요소로 취급한다.

크레이트 전체 목록과 정확한 이름은 `01-TECHSPEC.md` §4에서 정의한다.

---

# 10. 기술 선택

## 10.1 Rust

Rust를 사용하는 이유:

- 장시간 동작하는 시스템 소프트웨어의 메모리 안전성
- 비동기 네트워크 서버 구현
- 낮은 런타임 오버헤드
- 구조화된 동시성 구현
- C 기반 RKNN API와의 FFI 연계 가능
- Linux 오픈소스 프로젝트와의 높은 적합성

## 10.2 Tokio

v0.1 기본 네트워크 런타임으로 Tokio를 사용한다.

Tokio 구현을 성능 기준선으로 삼고, 실제 병목이 확인될 경우 io_uring 기반 구현과 비교한다.

## 10.3 io_uring

io_uring은 필수 기능이 아니라 비교 실험 기능이다.

적용 조건:

- 네트워크 또는 시스템 호출 오버헤드가 측정 가능한 병목일 것
- 동시 요청 수가 충분히 높을 것
- 일반 Tokio 구현 대비 비교 가능한 실험을 구성할 수 있을 것

## 10.4 Zero-Copy

Zero-Copy는 마케팅 용어가 아니라 실제 복사 횟수 감소 여부로 평가한다.

v0.1에서는 다음 경로를 분석한다.

```text
Network Buffer
→ User Buffer
→ Decode Buffer
→ Preprocess Buffer
→ NPU Input Buffer
```

각 구간의 복사 여부를 확인하고 제거 가능한 복사만 최적화한다.

완전한 Zero-Copy는 보장하지 않는다.

## 10.5 RKNN

RK3576 NPU 실행은 초기 버전에서 RKNN Runtime을 이용한다.

RKNN 종속성을 최소화하기 위해 추론 백엔드 인터페이스를 별도 계층으로 분리한다.

---

# 11. 실험 설계

## 11.1 기본 실험 구성

- 동일한 RK3576 보드 3대
- 동일한 OS와 커널
- 동일한 RKNN Runtime
- 동일한 모델
- 동일한 입력 데이터
- 2.5GbE 유선 네트워크
- 동일한 전원 및 냉각 조건

물리 구성의 상세 조건은 `02-HARDWARE-SETUP.md`를 따른다.

## 11.2 노드 수 비교

- 1노드
- 2노드
- 3노드

측정 지표:

- requests/sec
- FPS
- p50
- p95
- p99
- 오류율
- CPU 사용률
- 메모리 사용률
- 네트워크 사용량
- 전력 사용량
- 온도

## 11.3 동시 요청 수

기본 동시성 조건은 4배 간격의 네 단계로 한다.

- 1
- 4
- 16
- 64

동시성 축은 노드 수, 정책, 반복 횟수와 곱해지므로 단계를 하나 늘릴 때마다 총 측정시간이 수 시간 단위로 증가한다.

축을 추가하기 전에 반드시 총 소요시간을 먼저 계산한다. 시나리오별 적용 동시성은 `01-TECHSPEC.md` §20.2에서 정의한다.

## 11.4 스케줄러 비교

- Round Robin
- Least Queue
- Estimated Completion Time

세 정책의 점수 계산식과 설정 및 CLI 식별자는 `01-TECHSPEC.md` §10에서 정의한다.

## 11.5 네트워크 구현 비교

- Tokio TCP 또는 gRPC
- Tokio + 버퍼 풀
- io_uring
- io_uring + 적용 가능한 복사 최적화

## 11.6 장애 실험

- 노드 프로세스 강제 종료
- 네트워크 케이블 분리
- 높은 온도로 인한 성능 저하
- 모델 로딩 실패
- 요청 처리 지연
- 노드 복구 및 재편입

## 11.7 비용 비교

다음 항목을 포함한다.

- 하드웨어 구매비용
- 전력 소비
- 처리량당 비용
- FPS당 비용
- 운영 복잡도
- 장비 수
- 장애 지점 수

---

# 12. 성공 기준

## 12.1 필수 성공 기준

성공 기준은 "특정 수치가 나왔는가"가 아니라 **"측정하고 설명할 수 있는가"** 로 정의한다.

측정을 목적으로 하는 프로젝트에서 결과값을 성공 조건으로 걸면, 목표 수치가 나오지 않을 때 실험 조건을 유리하게 선택할 유인이 생긴다. 이는 본 프로젝트의 핵심 가치인 재현성과 정면으로 충돌한다.

따라서 성능 목표치는 §8 NFR-01에 두고, 달성 여부는 성공 조건이 아니라 결과로 보고한다.

다음 조건을 충족하면 NPUForge v0.1을 성공으로 판단한다.

- RK3576 NPU 노드 3대 연결
- 단일, 2노드, 3노드 추론 성공
- 1/2/3노드 확장 배율과 확장 효율 측정, 목표 미달 시 병목 원인의 정량적 제시
- 노드 장애 시 서비스 지속
- 장애 노드 자동 제외 및 복구
- p50, p95, p99 측정
- 실시간 상태 대시보드 동작
- GitHub 소스 공개
- 설치 및 실행 문서 공개
- 재현 가능한 벤치마크 공개
- FOSS for All Conference 발표 자료와 데모 준비

## 12.2 권장 성공 기준

- 3노드 확장 효율 85% 이상
- 부하 기반 스케줄러가 Round Robin 대비 의미 있는 개선
- io_uring 효과를 정량적으로 설명
- 데이터 복사 횟수와 비용 분석
- 노드 장애 중 요청 실패율 최소화
- 자동화된 설치 스크립트
- Docker 또는 systemd 기반 실행 지원
- CPU Mock Backend 제공
- 외부 기여자가 재현 가능한 수준의 문서화

## 12.2.1 열 특성 측정

보유 장비인 NanoPi R76S는 팬리스 보드다. 능동 냉각을 추가하지 않고 **thermal throttling을 측정 대상에 포함**한다.

측정 결과:

- Peak FPS 대비 Sustained FPS의 격차
- throttling 시작 시점
- 정상 상태 온도와 성능 저하율

벤더가 공개하는 TOPS는 순간 성능이며, 팬리스 엣지 디바이스의 지속 성능은 공개 자료가 거의 없다. 이 격차를 측정하는 것은 "TOPS 수치가 실제 처리량을 대표하지 못한다"는 본 프로젝트의 문제 정의(§2)와 직접 연결된다.

단, **열 특성과 확장 효율은 분리해서 보고한다.** 두 가지가 섞이면 처리량 저하의 원인이 스케줄링인지 온도인지 구분할 수 없다.

상세 설계는 `01-TECHSPEC.md` §20.2 S0, 물리 조건은 `02-HARDWARE-SETUP.md` §9를 따른다.

## 12.3 실패해도 의미 있는 결과

다음 결과도 유효한 기술 성과로 간주한다.

- io_uring이 유의미한 성능 개선을 만들지 못함
- Zero-Copy 적용 범위가 제한적임
- 네트워크보다 NPU 또는 전처리가 주요 병목으로 확인됨
- 3노드 확장 효율이 예상보다 낮음
- 단일 고성능 장치가 비용 면에서 더 유리함

이 경우에도 병목 원인과 적용 조건을 정량적으로 제시하면 발표 및 연구 결과로 가치가 있다.

---

# 13. 발표 데모 시나리오

## 데모 1. 단일 노드와 3노드 비교

동일한 영상 또는 이미지 데이터 세트를 사용하여 단일 노드와 3노드의 처리량을 실시간 비교한다.

화면 표시:

- 단일 노드 FPS
- 3노드 전체 FPS
- 확장 배율
- 확장 효율
- p95 지연시간

## 데모 2. 실시간 부하 분산

각 노드로 요청이 분배되는 모습을 대시보드에서 보여준다.

표시 항목:

- 노드별 큐
- 노드별 처리량
- 노드별 온도
- 노드별 상태
- 요청 라우팅 현황

## 데모 3. 노드 장애

3노드 동작 중 한 노드의 프로세스를 종료하거나 네트워크를 차단한다.

예상 동작:

1. 헬스체크 실패
2. 장애 노드 자동 제외
3. 나머지 2노드로 요청 재분배
4. 서비스 지속
5. 노드 재시작
6. 정상 확인 후 자동 재편입

## 데모 4. 스케줄러 비교

Round Robin과 ECT 정책의 처리량 및 지연시간을 비교한다.

---

# 14. 일정

## 2026년 8월

- PRD 작성
- 기술 사양서 작성
- 프로젝트 저장소 생성
- 개발 환경 정리
- 단일 RK3576 RKNN 추론 검증
- Rust FFI 최소 검증

## 2026년 9월

- NPU 노드 에이전트 구현
- 중앙 스케줄러 구현
- 단일, 2노드, 3노드 연결
- Round Robin 구현
- 기본 벤치마크 도구 구현
- 예비 결과 확보

## 2026년 10월

- Least Queue 및 ECT 스케줄러 구현
- 헬스체크 구현
- 장애 노드 제외 및 복구
- 요청 재시도
- 메트릭 수집
- 대시보드 구현
- Tokio 기준 성능 확정
- io_uring 적용 여부 판단

## 2026년 11월 1일~15일

- io_uring 비교 실험
- 데이터 복사 분석
- 최종 벤치마크
- 비용 및 전력 분석
- 코드 정리
- GitHub 문서 작성
- 데모 안정화

## 2026년 11월 16일~22일

- 기능 동결
- 발표자료 작성
- 데모 영상 촬영
- 발표 리허설
- 장애 상황 대비 예비 영상 준비

## 2026년 11월 28일

- FOSS for All Conference 발표
- NPUForge v0.1 공개

---

# 15. 주요 위험과 대응

## 위험 1. RKNN Rust FFI 불안정

대응:

- 최소 C Wrapper를 작성한다.
- unsafe 영역을 별도 모듈로 격리한다.
- 입력 및 출력 버퍼 수명 관리를 명확히 한다.
- 단위 테스트와 반복 추론 테스트를 수행한다.

## 위험 2. 3노드 성능 향상이 낮음

대응:

- NPU 연산과 전처리 시간을 분리 측정한다.
- 입력 데이터 크기를 비교한다.
- 동시 요청 수를 조정한다.
- 네트워크와 스케줄러 오버헤드를 측정한다.
- 실패 결과도 병목 분석 자료로 활용한다.

## 위험 3. io_uring 효과가 미미함

대응:

- Tokio를 기본 구현으로 유지한다.
- io_uring은 실험 브랜치로 분리한다.
- 효과가 없으면 적용 조건과 원인을 결과로 정리한다.

## 위험 4. Zero-Copy 구현 난도

대응:

- 완전한 Zero-Copy를 성공 기준에서 제외한다.
- 버퍼 풀과 메모리 재사용을 우선한다.
- 복사 횟수 감소 자체를 측정한다.

## 위험 5. 발표 일정 부족

대응:

- 데모 필수 범위를 제한한다.
- 기능 완성보다 벤치마크 재현성을 우선한다.
- 11월 15일 이후 기능 추가를 금지한다.
- 라이브 데모 장애에 대비해 녹화 영상을 준비한다.

## 위험 6. 프로젝트 범위 확대

대응:

v0.1에서는 다음 요청을 거절한다.

- LLM 모델 병렬
- Kubernetes 연동
- 다중 제조사 NPU 지원
- 클라우드 관리 서비스
- 사용자 계정 시스템
- 자동 모델 변환
- 범용 AI 플랫폼 기능

---

# 16. 오픈소스 공개 계획

## 저장소 구조

`01-TECHSPEC.md` §4에서 정의한다.

## 공개 산출물

- 전체 소스코드
- 빌드 방법
- 설치 방법
- 샘플 설정
- 테스트 데이터 안내
- 벤치마크 실행 방법
- 원본 벤치마크 결과
- 아키텍처 문서
- 알려진 제한사항
- 발표자료
- 데모 영상

## 라이선스 후보

우선 검토 대상:

- Apache License 2.0
- MIT License

특허 조항과 기업 활용 가능성을 고려하면 Apache License 2.0을 우선 검토한다.

---

# 17. 향후 확장

v0.1 이후 검토 가능한 기능:

- Hailo 백엔드
- Jetson TensorRT 백엔드
- OpenVINO 백엔드
- 모델 자동 배포
- 다중 모델 스케줄링
- SLA 기반 스케줄링
- 에너지 최적화 스케줄링
- 카메라 스트림 직접 입력
- 분산 추적
- WebAssembly 클라이언트
- Kubernetes Device Plugin
- 광역 엣지 노드 연동
- LLM 요청 단위 분산
- 박사논문용 적응형 스케줄링 알고리즘

---

# 18. 최종 제품 정의

NPUForge v0.1은 6 TOPS NPU 세 대를 물리적으로 합쳐 하나의 18 TOPS NPU를 만드는 제품이 아니다.

NPUForge는 독립적인 추론 요청을 여러 엣지 NPU에 효율적으로 분배하고, 실제 성능과 병목을 측정하며, 노드 장애 상황에서도 서비스를 지속할 수 있도록 하는 오픈소스 분산 추론 런타임이다.

프로젝트의 핵심 가치는 높은 TOPS 수치 자체가 아니라 다음에 있다.

- 실제 확장 효율의 검증
- 병목의 정량적 분석
- 비용 대비 처리량 비교
- 장애 허용 구조
- 재현 가능한 오픈소스 실험
- Linux, Rust, Edge AI 기술의 통합

---

# 19. 발표 핵심 메시지

> 6 TOPS NPU 세 대를 연결한다고 자동으로 18 TOPS가 되는 것은 아니다.  
> NPUForge는 그 차이가 어디에서 발생하는지 측정하고, 실제로 확장 가능한 조건을 찾아가는 오픈소스 프로젝트다.

발표 제목:

> **6 TOPS NPU 세 대는 정말 18 TOPS가 되는가?**  
> Rust 기반 분산 엣지 추론 런타임 NPUForge 개발기

---

<a id="01-techspec"></a>

# NPUForge Technical Specification

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
> | 실험 최종 상태 | [`experiments/README.md`](#experiments-readme) §5~§7 |
> | io_uring 판정 (**적용하지 않는다**) | **이 문서 §15** |
> | 확정 수치 | [`RESULTS.md`](#results) · [`experiments/`](experiments) |
>
> 특히 이 문서가 비교 실험으로 들고 있는 **io_uring 은 구현하지 않기로
> 했다.** S3.9b 가 회수 가능한 몫을 ≈8% 로 측정했고, 그 근거로 배제했다.

- 문서명: `01-TECHSPEC.md`
- 프로젝트명: NPUForge
- 문서 버전: v0.2
- 대상 릴리스: NPUForge v0.1
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

본 문서는 NPUForge v0.1의 구현 구조, 컴포넌트 책임, 통신 프로토콜, 데이터 모델, 스케줄링 방식, 장애 처리, 메트릭 수집, 벤치마크 방법 및 배포 구조를 정의한다.

NPUForge v0.1은 RK3576 기반 6 TOPS NPU 노드 최대 3대를 하나의 분산 추론 클러스터로 운영하는 Rust 기반 오픈소스 런타임이다.

이 문서의 목표는 다음과 같다.

1. 개발자가 추가 해석 없이 구현을 시작할 수 있도록 한다.
2. 기능 범위와 비기능 요구사항을 기술적으로 구체화한다.
3. 성능 비교가 가능한 기준 구현을 정의한다.
4. 발표 데모와 연구용 벤치마크의 재현성을 확보한다.
5. RKNN 종속 코드를 격리하여 향후 다른 NPU 백엔드로 확장 가능하게 한다.

---

# 2. 설계 원칙

## 2.1 데이터 병렬 우선

NPUForge v0.1은 하나의 모델을 여러 노드에 분할하지 않는다.

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
│ NPUForge Scheduler                                        │
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
      │ NPUForge Node 1│ │ NPUForge Node 2│ │ NPUForge Node 3│
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
├── README.md
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
> → [`experiments/S3_9B_NODE_RESIDUAL.md`](#experiments-s3-9b-node-residual)
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
- NPUForge v0.1 공개

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

- NPUForge 로고 및 버전
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

NPUForge v0.1은 다음 조건을 모두 충족할 때 완료로 간주한다.

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

NPUForge는 여러 엣지 NPU를 물리적으로 결합하는 기술이 아니다.

NPUForge는 독립적인 추론 요청을 여러 NPU 노드에 분산하고, 각 노드의 부하와 상태를 기준으로 요청을 스케줄링하며, 장애 발생 시 서비스를 지속하고, 실제 성능 손실과 확장 효율을 재현 가능하게 측정하는 Linux/Rust 기반 오픈소스 분산 추론 런타임이다.

---

<a id="02-hardware-setup"></a>

# NPUForge Hardware Setup Guide

- 문서명: `02-HARDWARE-SETUP.md`
- 프로젝트명: NPUForge
- 문서 버전: v0.2
- 대상 릴리스: NPUForge v0.1
- 목표 발표: 2026년 11월 FOSS for All Conference
- 작성일: 2026-08-05
- 최종 수정: 2026-08-06
- 상태: Draft
- 관련 문서:
  - `00-PRD.md`
  - `01-TECHSPEC.md`
  - `03-DEVELOPMENT-REQUIREMENTS.md`
  - `environment-matrix.md`

본 문서는 물리 구성, 네트워크, 전원, 냉각, 실험 조건에 대한 규범 문서다. 해당 영역에서 다른 문서와 값이 다를 경우 본 문서를 따른다.

---

# 1. 권장 구성

NanoPi R76S 3대는 모두 동일한 NPU Worker로 구성한다.

중앙 스케줄러, 대시보드, 벤치마크 클라이언트는 **별도 서버**에서 실행한다.
**PCIe 슬롯이 있어야 한다** — 10G NIC 을 꽂아야 하기 때문이다(§3.3.2).

```text
              ┌────────────────────────────┐
              │ Benchmark / Scheduler      │
              │ Server (PCIe 슬롯 필요)    │
              │                            │
              │ · NPUForge Scheduler       │
              │ · Benchmark Client         │
              │ · Dashboard                │
              │ · Prometheus               │
              └─────────────┬──────────────┘
                            │ 10GbE (SFP+ DAC)   ← aggregation
                  ┌─────────▼──────────┐
                  │ 2.5G / 10G Switch  │
                  └────┬─────┬─────┬───┘
                       │2.5G │2.5G │2.5G
               ┌───────▼┐ ┌──▼────┐ ┌▼──────┐
               │ KING   │ │ QUEEN │ │ JACK  │
               │ Worker │ │ Worker│ │ Worker│
               │ 6 TOPS │ │ 6 TOPS│ │ 6 TOPS│
               └────────┘ └───────┘ └───────┘
```

**worker 링크는 2.5G, aggregation 만 10G** 다. 노드당 최대 1.545 Gbps
이므로 2.5G 로 충분하고, 세 노드가 합류하는 스케줄러 쪽이 병목이 된다.
근거는 §3.3.2.

핵심 원칙:

```text
3대 NanoPi = 동일한 Worker
별도 Linux PC = Scheduler
2.5GbE = 추론망
동일 OS·커널·RKNN·모델
독립 전원
동일 냉각
중앙에서 모든 벤치마크 기록
```

---

# 2. 장비별 역할

| 장비 | 역할 | 비고 |
|---|---|---|
| Linux PC | Scheduler | 요청 접수와 노드 선택 |
| Linux PC | Benchmark Client | 부하 발생과 결과 저장 |
| Linux PC | Dashboard | 실시간 처리량·장애 표시 |
| Linux PC | Metrics Server | Prometheus 및 선택적 Grafana |
| KING | NPU Worker | RKNN 추론 |
| QUEEN | NPU Worker | RKNN 추론 |
| JACK | NPU Worker | RKNN 추론 |

## 2.1 NanoPi를 모두 Worker로 사용하는 이유

한 대의 NanoPi에서 Scheduler와 NPU Worker를 함께 실행하면 다음 문제가 발생한다.

- 해당 노드만 CPU와 네트워크 부하가 증가한다.
- 세 노드의 실험 조건이 달라진다.
- 1노드, 2노드, 3노드 비교가 왜곡될 수 있다.
- Scheduler 병목과 NPU 병목을 분리하기 어렵다.
- 발표에서 동일 조건의 비교라고 설명하기 어려워진다.

따라서 공식 벤치마크에서는 세 대를 완전히 대칭적인 Worker로 유지한다.

단순 개발 또는 이동형 데모에서는 KING에 Scheduler를 함께 실행할 수 있지만, 공식 성능 수치에는 사용하지 않는다.

---

# 3. 네트워크 구성

## 3.1 권장 토폴로지

**2.5G/10G 스위치**를 중심으로 한 스타 구조를 사용한다.
worker 는 2.5G, 스케줄러 업링크만 10G 다(§3.3.2).

필요 장비:

- **2.5G/10G 스위치 1대** — 2.5G 포트 ≥ 4 + SFP+ 업링크
- **스케줄러 서버 1대** — PCIe 슬롯 필요
- **10G NIC (PCIe, SFP+)** 1장 — 예: Intel X520
- **SFP+ DAC 케이블** 1개 — 서버 ↔ 스위치
- CAT5e 케이블 3개 — 스위치 ↔ 노드 (2.5G 는 Cat5e 로 충분)
- NanoPi R76S 3대

```text
Linux PC ───┐
KING ─────┤
QUEEN ─────┼── 2.5GbE Switch
JACK ─────┘
```

## 3.2 IP 주소 계획

NPUForge 전용 추론망 예시:

```text
Network     : 10.20.0.0/24
Scheduler   : 10.20.0.10
KING      : 10.20.0.21
QUEEN      : 10.20.0.22
JACK      : 10.20.0.23
```

Hostname:

```text
npuforge-scheduler
npuforge-king
npuforge-queen
npuforge-jack
```

`/etc/hosts` 예시:

```text
10.20.0.10  npuforge-scheduler
10.20.0.21  npuforge-king
10.20.0.22  npuforge-queen
10.20.0.23  npuforge-jack
```

## 3.3 관리망 분리 (필수)

NanoPi R76S는 **2.5GbE 포트를 2개** 가지고 있다. 따라서 관리망 분리가 추가 비용 없이 가능하며, 선택이 아니라 기본 구성으로 둔다.

```text
포트 1 → 기존 사내/가정 네트워크   = 관리망
           SSH, apt, 바이너리 배포, 로그 수집

포트 2 → 전용 스위치               = 추론망
           추론 트래픽만. 다른 트래픽 없음
```

관리망을 분리하는 이유는 편의가 아니라 **측정 오염 방지**다. SSH 세션, `apt` 다운로드, 로그 전송이 추론망에 섞이면 네트워크 지연 측정값에 원인 불명의 튀는 값이 생긴다.

포트 이름은 보드와 커널에 따라 다르므로 반드시 확인한다.

```bash
ip -br a
for n in /sys/class/net/e*; do
  echo "$(basename $n): speed=$(cat $n/speed 2>/dev/null) mac=$(cat $n/address)"
done
```

어느 포트를 추론망에 쓸지는 세 노드에서 **동일해야 한다.** `environment-matrix.md` §8에 노드별로 기록한다.

## 3.3.1 단계적 구축

추론망 스위치 도입 전에도 개발은 진행한다.

| 단계 | 관리망 | 추론망 | 가능한 작업 |
|---|---|---|---|
| 현재 | 기존 1G 허브 | 없음 | 보드 세팅, RKNN 검증, 단일 노드 |
| 중간 | 기존 1G 허브 | 1G 허브 공유 | M2~M5 전체 (gRPC, 3노드, 장애 복구) |
| 최종 | 기존 1G 허브 | 2.5GbE 전용 스위치 | 공식 벤치마크 |

**중간 단계로도 M5까지 전부 개발 가능하다.** 링크 속도는 기능 정확성에 영향을 주지 않으며, 2.5GbE가 필요한 것은 공식 성능 수치뿐이다.

JPEG 입력 기준으로는 1GbE로도 대역폭이 충분하다(§3.1 근거 계산 참조). Raw RGB 입력 시나리오에서만 1GbE가 먼저 포화된다.

## 3.3.2 Scheduler 호스트의 링크 속도 — **10G 필요** (2026-08-12 개정)

Scheduler 호스트는 세 노드 트래픽이 모두 합류하는 지점이다. 실측 처리량이
나오고 나서 계산하니 **2.5GbE 로는 부족하다**는 것이 확인되었다.

raw RGB 입력 한 장은 `640 × 640 × 3 = 1,228,800 byte` 다.

```text
                    노드당           3노드 합계
INT8  157.2 inf/s   1.545 Gbps       4.636 Gbps
FP16   84.3 inf/s   0.829 Gbps       2.486 Gbps
```

**FP16 조차 3노드 합계가 2.486 Gbps 로 2.5GbE 한 링크(실효 약 2.35 Gbps)를
넘는다.** INT8 은 4.636 Gbps 로 두 배 가까이 넘는다.

즉 **worker 링크가 아니라 aggregation 링크가 먼저 막힌다.**

### 개정된 토폴로지

```text
        Benchmark / Scheduler Server
                    │
                  10GbE          ← aggregation. 여기가 핵심
                    │
            2.5G / 10G Switch
              ├── 2.5G ── king
              ├── 2.5G ── queen
              └── 2.5G ── jack
```

- **worker 링크는 2.5G 그대로** 다. 노드당 최대 1.545 Gbps 이므로 충분하다
- **aggregation 링크만 10G** 로 올린다
- 스위치는 2.5G 포트 + 10G(SFP+) 업링크를 가진 모델

### 필요한 것

| 항목 | 사양 | 비고 |
|---|---|---|
| Scheduler 호스트 | **PCIe 슬롯이 있는 서버** | 노트북은 10G 카드를 못 꽂는다 |
| ┗ CPU | **16 스레드 이상 권장** | 2026-08-26 실측. §3.3.4 |
| 10G NIC | PCIe, SFP+ (예: Intel X520) | |
| DAC 케이블 | SFP+ Direct Attach | 광 모듈보다 싸고 짧은 거리에 적합 |
| 스위치 | 2.5G 포트 ≥ 4 + SFP+ 업링크 | |

> **이전 판(2.5GbE NIC 확보)은 폐기한다.** 그 계산은 노드당 실측
> 처리량이 나오기 전의 추정이었다. 확보하지 못한 상태에서 측정한 값을
> 공식 수치로 쓰지 않는다는 원칙은 그대로다.

## 3.3.3 M3 이전에 기록할 실측 네트워크 값

계산이 아니라 **실측**이다. `dealer`(또는 새 스케줄러 서버)에서
인터페이스 카운터로 잰다.

| 조건 | 기록할 것 |
|---|---|
| 단일 요청 | 스케줄러 TX bytes / RX bytes |
| 1노드 포화 | TX Gbps / RX Gbps |
| 3노드 포화 | 합계 TX / RX Gbps |

측정 방법 예시.

```bash
# 인터페이스 카운터 스냅샷 → 부하 → 다시 스냅샷
IF=enp3s0
read t0 r0 <<< "$(awk -v i=$IF '$1==i":"{print $10, $2}' /proc/net/dev)"
# ... 부하 ...
read t1 r1 <<< "$(awk -v i=$IF '$1==i":"{print $10, $2}' /proc/net/dev)"
echo "TX $(( (t1-t0)*8/1000000 )) Mb   RX $(( (r1-r0)*8/1000000 )) Mb"
```

이 값이 계산값(입력 1.545 Gbps/node)과 크게 다르면 **계산의 전제가
틀린 것이다.** 어느 쪽이 맞는지 확인하기 전에 S2 를 진행하지 않는다.

---

## 3.3.4 Scheduler 호스트의 CPU — **스레드 수가 처리량을 가른다** (2026-08-26)

§3.3.2 는 **대역**만 다뤘다. 실제로 호스트를 바꿔 보니 **CPU 가 먼저
좁아졌다.**

| | 구서버 | 신서버 |
|---|---|---|
| 호스트 | Dell PowerEdge R620 | ASUS H81M-K 데스크톱 |
| CPU | Xeon E5-2630L ×2 · **24 스레드** · 2.0~2.5GHz | Core i7-4790 · **8 스레드** · 3.6~4.0GHz |
| 10G NIC | **같은 Intel X550T 카드** (옮겨 꽂았다) | |
| 측정 중 서버 CPU | 42% | **82.2%** |
| **처리량** | **~391 inf/s** | **~360 inf/s** (−7.5%) |
| 오류율 | 0 | 0 |

**싱글스레드 성능은 신서버가 확실히 빠른데도 처리량이 떨어졌다.**
스케줄러 워크로드는 단일 요청 지연이 아니라 **동시 스트림 처리량**에
좌우된다.

### 왜 CPU 가 먼저 좁아지나

```text
스케줄러        45.3%  ≈ 3.6 코어
기타(벤치+커널)  36.9%  ≈ 2.9 코어
────────────────────────────────
합계            82.2%  (8 스레드 기준)
```

**벤치 클라이언트가 스케줄러와 같은 호스트에서 돈다.** 세 노드로 보내고
받는 일과 부하를 만드는 일을 한 CPU 가 나눠 한다. 24 스레드에서는 같은
일이 42% 였다.

애플리케이션 큐는 두 조건 모두 비어 있다(`scheduler_queue` 0.00ms ·
`scheduler_route` 0.01ms). 좁아진 곳은 큐가 아니라 **호스트의 CPU** 다.

### 따라할 때

| | |
|---|---|
| **권장** | 16 스레드 이상 |
| 최소 | 8 스레드로도 **오류 0 으로 정상 동작**한다. 처리량 기준선이 달라질 뿐이다 |
| 확인법 | 측정 중 서버 CPU 사용률을 본다. 80% 를 넘으면 호스트가 제약이다 |
| 대안 | 벤치를 다른 호스트에서 돌린다. 단 그 호스트도 10G 여야 한다 |

**낮은 사양을 쓸 거면 그 호스트에서 기준선을 새로 깔고, 다른 호스트 값과
직접 비교하지 않는다.**

> PCIe 세대는 병목이 아니었다. 같은 X550T 가 R620 에서는 PCIe 3.0 x4
> (방향당 ~32Gbps), 신서버에서는 2.0 x4(~16Gbps)로 물렸지만 3노드
> 실사용이 방향당 ~4.6Gbps 라 양쪽 다 여유가 크다.

근거: `infrastructure.md` §3.2.1 · `hosts/` · `../results/baseline-20260826-althost/`

### ⚠️ 출력 방향이 더 크다 — `want_float=1` 이면 10G 로도 부족했다

위 계산은 **입력(TX) 방향만** 이다. 출력을 계산하니 결론이 뒤집힌다.

노드는 후처리를 하지 않고 **원시 텐서 9개를 그대로 반환**한다.

```text
입력                      1,228,800 byte
출력 (want_float=1, f32)  4,872,000 byte   ← 입력의 3.96배
출력 (want_float=0, int8) 1,218,000 byte   ← 입력의 0.99배
```

3노드 포화 시 스케줄러 링크에 걸리는 부하다.

| 구성 | 모델 | 3노드 TX | 3노드 RX | 10G 로 되나 |
|---|---|---:|---:|---|
| `want_float=1` (구 기본값) | INT8 | 4.64 Gbps | **18.38 Gbps** | **불가** |
| `want_float=1` (구 기본값) | FP16 | 2.49 Gbps | **9.86 Gbps** | 겨우 (여유 없음) |
| **`want_float=0` (현재 기본값)** | INT8 | 4.64 Gbps | 4.60 Gbps | 가능 |
| **`want_float=0` (현재 기본값)** | FP16 | 2.49 Gbps | 2.46 Gbps | 가능 |

**`want_float=1` 이었다면 10G 로도 INT8 3노드를 감당하지 못했다.**

### 따라서 M3 전에 둘 중 하나가 필요했다

**(A) `want_float=0` 으로 전환** — ✅ **2026-08-12 완료**

출력을 원본 dtype 그대로 받아 **RX 를 4분의 1로 줄인다.** 대신 받는 쪽이
역양자화를 직접 해야 하므로, blob 을 **v2** 로 올려 텐서마다
`qnt_type`·`scale`·`zero_point` 를 함께 싣는다. 노드 설정
`[worker] want_float` 의 기본값이 `false` 다.

> 실보드에서 역양자화 결과가 float32 와 일치함을 확인했다
> (텐서 9개, **최대 오차 9.5e-7** — float32 정밀도 한계).
>
> **처리량도 함께 올랐다 — INT8 +17.3% / FP16 +15.7%** (`king`, 8스레드,
> 120초). `discuss.md` §5 의 +5.4% 는 1스레드 위주 FP16 값이라 작게
> 나온 것이다. 승격 근거는 처리량이 아니라 **RX 대역폭**이었지만,
> 두 지표가 같은 방향을 가리켰다. `discuss.md` §12

**(B) 노드에서 후처리(NMS)까지 수행** — 미구현

검출 결과만 반환하면 응답이 수 KB 로 줄어 RX 가 사실상 사라진다.
올바른 최종 형태지만 구현이 남아 있다.

두 방향 모두 **입력 TX 4.64 Gbps 는 그대로**이므로 10G aggregation 은
여전히 필요하다.

### 그래도 실측한다

위는 전부 계산이다. M3 시작 전에 실제 TX/RX 를 측정해 기록한다(§3.3.3).
**입력만 계산하고 출력을 안 본 것이 이번 오류의 원인이다.** 계산만 믿고
넘어가면 같은 실수를 반복한다.

---

## 3.4 초기 네트워크 제한

초기에는 다음 구성을 사용하지 않는다.

- Wi-Fi
- 노드 직렬 연결
- Docker Overlay Network
- Kubernetes Network
- 복잡한 VLAN
- Jumbo Frame
- 다중 서브넷 라우팅

기본 MTU는 모든 장치에서 1500으로 통일한다.

Jumbo Frame은 기준 성능을 확보한 뒤 별도 실험으로 비교한다.

---

# 4. 저장장치 구성

## 4.1 우선순위

1. **eMMC** (온보드 32GB 또는 64GB)
2. 고내구성 microSD
3. 일반 microSD는 초기 개발에만 사용

NPU Worker는 대용량 데이터를 장기간 저장하지 않으므로 32GB eMMC면 기본 운영에 충분하다.

```text
/opt/npuforge/
├── bin/
├── config/
├── models/
└── logs/
```

벤치마크 데이터 세트, 원본 결과, 그래프와 발표 자료는 Scheduler PC에 저장한다.

## 4.2 NVMe는 사용하지 않는다

NanoPi R76S의 M.2 슬롯은 **SDIO 기반이며 Wi-Fi 모듈용이다.** NVMe SSD를 장착할 수 없다.

따라서 다음 항목은 v0.1 범위에서 제외한다.

- 노드에서의 대용량 영상 파일 저장
- 노드 파일 I/O 성능 비교 실험
- 노드에 여러 모델 로컬 보관
- 노드에서의 장기 로그 보존

벤치마크 데이터 세트, 원본 결과, 로그는 모두 Scheduler 호스트에 저장한다. 세 노드의 단순 추론 구성에서는 이 제약이 문제가 되지 않는다.

M.2 슬롯을 Wi-Fi에 사용하는 것도 v0.1에서는 하지 않는다(§3.4의 Wi-Fi 배제).

---

# 5. 운영체제 구성

## 5.1 권장 OS

세 노드에는 동일한 Debian 또는 Ubuntu Server 계열 Headless 이미지를 설치한다.

라우터 중심의 배포판보다 일반 Linux 개발환경이 적합하다.

필수 통일 항목:

```text
동일 OS 이미지
동일 커널 버전
동일 NPU 드라이버
동일 RKNN Runtime
동일 Rust 바이너리
동일 모델 파일
동일 CPU Governor
동일 냉각 조건
```

## 5.2 기본 패키지

```bash
sudo apt update

sudo apt install -y \
    build-essential \
    pkg-config \
    cmake \
    git \
    curl \
    chrony \
    iperf3 \
    ethtool \
    jq \
    htop \
    sysstat \
    linux-perf
```

배포판에 따라 `linux-perf` 패키지명은 달라질 수 있다.

## 5.3 Rust 바이너리 배포

각 노드에서 개별 빌드하기보다 다음 방식을 권장한다.

1. 빌드 PC에서 ARM64용 바이너리 생성
2. 동일한 바이너리를 세 노드에 배포
3. SHA-256 해시 확인
4. systemd 서비스로 실행

이 방식이 빌드 환경 차이를 줄이고 재현성을 높인다.

---

# 6. RKNN 구성

## 6.1 모델 변환과 실행 분리

```text
개발 PC
  ONNX/PyTorch
      ↓ RKNN-Toolkit2
  model.rknn
      ↓ 배포
KING / QUEEN / JACK
      ↓ RKNN Runtime
  NPU 추론
```

모델 변환은 개발 PC에서 수행하고, NanoPi에서는 변환된 RKNN 모델만 실행한다.

## 6.2 세 노드 간 일치 항목

```text
RKNN Runtime 버전
RKNPU 커널 드라이버 버전
model.rknn SHA-256
전처리 설정
후처리 코드
입력 해상도
양자화 방식
NPU Core 설정
```

모델 해시 확인 예시:

```bash
sha256sum /opt/npuforge/models/yolov8n/model.rknn
```

세 노드의 결과가 동일해야 한다.

## 6.3 모델 디렉터리 예시

```text
/opt/npuforge/models/
└── yolov8n/
    ├── model.rknn
    ├── model.toml
    └── labels.txt
```

---

# 7. 노드 설정

노드별 차이는 다음 세 가지로 제한한다.

```text
Node ID
IP 주소
Hostname
```

그 외 설정, 모델, 바이너리와 런타임 버전은 모두 같아야 한다.

## 7.1 KING

```toml
[node]
id = "king"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.21:51001"
scheduler_address = "http://10.20.0.10:50051"

[backend]
type = "rknn"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]

[worker]
worker_count = 1
max_queue_depth = 32
```

## 7.2 QUEEN

```toml
[node]
id = "queen"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.22:51001"
scheduler_address = "http://10.20.0.10:50051"

[backend]
type = "rknn"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]

[worker]
worker_count = 1
max_queue_depth = 32
```

## 7.3 JACK

```toml
[node]
id = "jack"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.23:51001"
scheduler_address = "http://10.20.0.10:50051"

[backend]
type = "rknn"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]

[worker]
worker_count = 1
max_queue_depth = 32
```

---

# 8. 전원 구성

## 8.1 입력 방식: 12V DC (USB-C PD 아님)

문서 초안은 USB-C PD를 전제했으나 **틀렸다.** NanoPi R76S는 **12V DC 입력**을 사용한다.

2026-08-10 커널 로그 실측:

```text
vcc12v_dcin: 12000 mV, enabled          ← 주 전원 입력
vcc_sys: supplied by vcc12v_dcin
vbus5v0_typec: 5000 mV, disabled        ← Type-C 는 5V 출력용, 입력 아님
power_supply: simple-vin
PMIC: rk806
```

Type-C 포트는 데이터와 5V VBUS **출력**용이며 전원 입력 경로가 아니다.

따라서 전력 측정 계획도 바뀐다. USB-C 전력 측정기가 아니라 **12V DC 라인용 전력계**가 필요하다(§14.2).

## 8.2 권장 방식

각 보드에 독립적인 12V DC 어댑터를 사용한다.

```text
12V Adapter 1 → KING
12V Adapter 2 → QUEEN
12V Adapter 3 → JACK
```

권장 조건:

- **12V, 2A(24W) 이상**
- 동일 제조사, 동일 모델
- 동일 길이 케이블
- 멀티탭 하나에 세 대를 몰더라도 어댑터는 개별로 둔다

### ⚠️ 전류 용량이 부족하면 고부하에서 보드가 리셋된다

2026-08-10 실측에서 노드별로 안정성 한계가 달랐다.

| 노드 | 안정 한계 | 증상 |
|---|---|---|
| `queen` | 8 스레드 완주 | 정상 |
| `king` | **4 스레드까지만** | 5 스레드 이상에서 하드 리셋 |
| `jack` | 미확정 | 1회 리셋 관측 |

세 보드가 동일 모델이고 소프트웨어도 같으므로, **전원 공급 능력 차이**가 유력하다. 상세는 `board-worklog.md` §2.17.

CPU 8코어와 NPU 2코어를 동시에 최대로 쓰면 순간 전류가 크게 오른다. 어댑터 용량이 부족하면 전압이 떨어지고 PMIC가 리셋한다. 커널 로그에 아무것도 남지 않는 것이 이 경우의 특징이다.

**노드마다 안정 한계가 다르면 "동일한 3대"라는 실험 전제가 깨진다.** 확장 효율 측정 전에 반드시 해결한다.

## 8.2 전력 측정

에너지 효율을 논문 또는 발표에 포함하려면 노드별 전력 측정이 필요하다.

권장 방식:

- USB-C 전력 측정기 3개
- 또는 동일 조건에서 한 대씩 반복 측정
- 대기 전력과 추론 부하 전력 분리
- 스위치와 Scheduler 전력은 별도 기록

측정 지표:

```text
Idle Watt
Peak Watt
Average Watt
Requests per Watt-hour
FPS per Watt
```

---

# 9. 냉각 구성

## 9.1 냉각 조건을 두 가지로 측정한다 (2026-08-10 결정)

**팬리스와 능동 냉각 두 조건에서 각각 측정한다.**

```text
조건 A  팬리스        출고 상태. throttling 발생
조건 B  능동 냉각      동일 팬 3개. throttling 억제
```

### 근거

초안은 팬리스만 측정하기로 했으나, 2026-08-10 지속 부하 시험에서 다음이 관측되었다.

| 조건 | 8스레드 처리량 |
|---|---:|
| 순간 부하 (20회 반복) | 77.3 inf/s |
| 지속 부하 (3,000회 반복) | 69.7 inf/s ⚠️ `ondemand` 기준. 현재값은 `RESULTS.md` §2.2 |

**약 10% 저하.** 그리고 `king`이 NPU 91.3°C로 `disable_temperature_c`(90°C)를 초과했다.

즉 냉각 조건이 처리량과 노드 가용성 양쪽에 직접 영향을 준다. 한 조건만 측정하면 다음을 답할 수 없다.

- 팬리스만 측정 → "냉각하면 얼마나 나아지는가"를 모른다
- 냉각만 측정 → "실제 엣지 배치에서 얼마나 나오는가"를 모른다

**두 조건을 모두 측정하면 "냉각이 확장 효율에 미치는 영향"이 결과가 된다.** 이는 벤더 스펙시트에 없는 수치이며, 측정으로 밝힌다는 본 프로젝트의 정체성과 일치한다.

### 조건 A: 팬리스

출고 상태 그대로 사용한다. thermal throttling은 제거 대상이 아니라 **측정 대상**이다.

### 조건 B: 능동 냉각

세 노드에 **동일 모델 팬 3개**를 동일한 방식으로 장착한다.

- 동일 제조사, 동일 모델, 동일 회전수
- 동일한 거리와 각도
- 팬 소비전력을 별도 기록 (전력 효율 계산 시 분리)

**실제 설치 (2026-08-20):** 120mm 급 5V USB 팬 3개, 노드마다 1개를 보드 위에
얹었다 — **팬이 보드(NanoPi R76S)보다 크다.** 라벨 K/Q/J, 전원은 USB 허브.
보드가 팬 그릴 바로 아래에 놓여 상판 전체로 바람을 받는다.

> ⚠️ **2026-08-20 의 모든 측정(예비·S2)은 이 조건 B(능동 냉각)에서 수행됐다.**
> 처음엔 "팬리스(S0-A)"로 잘못 기록했다가 정정했다. 이 대형 팬에서는
> throttling 이 사실상 억제되므로, 팬리스(조건 A) sustained 157 을 그대로
> 이 조건의 노드 상한 비교 기준으로 쓰면 안 된다 —
> `results/scaling-20260820/README.md` §4.2 의 27% caveat 참조.
> **조건 A vs 조건 B 를 같은 gRPC 경로에서 나란히 재는 것이 §9.1 의 목적**이며,
> 아직 조건 A(팬리스) 쪽 클러스터 측정이 없다.

### ⛔ 책상 선풍기는 사용하지 않는다

2026-08-10 진단 중 책상 선풍기로 냉각한 사례가 있다. **진단에는 유효했으나 측정 조건으로는 사용할 수 없다.**

- 세 보드에 바람이 균등하게 도달하지 않는다
- "선풍기를 이 각도로 틀었다"는 재현이 불가능하다
- 조건 B의 요구사항(동일 팬, 동일 조건)을 만족하지 않는다

### 두 조건에 공통으로 적용되는 것

- 동일한 케이스 또는 전부 케이스 없음
- 동일한 방향과 간격으로 배치
- 동일한 주변 온도
- 보드 사이 최소 10cm 간격 (인접 보드의 열이 서로 영향을 주지 않도록)

```text
[KING]  ←10cm→  [QUEEN]  ←10cm→  [JACK]
        동일 주변 온도, 동일 배치 방향, 동일 냉각
```

주변 온도는 매 실험마다 기록한다. 계절과 실내 냉방에 따라 달라지므로, 이 값이 없으면 다른 날의 결과와 비교할 수 없다.

### ⚠️ 배치 균일화가 선행되어야 한다

2026-08-10 측정에서 동일 부하인데 **`king`이 다른 두 대보다 19°C 높았다**(NPU 91.3 vs 70.2 / 72.1°C).

선풍기를 틀자 세 대가 56~62°C로 수렴했으므로 **개체 불량이 아니라 공기 흐름 문제**로 확인되었다.

**두 조건 중 어느 쪽을 측정하든, 배치를 균일하게 맞추기 전에는 유효한 데이터가 나오지 않는다.** 노드별 온도 편차는 확장 효율 측정을 직접 오염시킨다. 상세는 `board-worklog.md` §2.19 참조.

## 9.2 온도 임계치는 보호 장치이지 측정 도구가 아니다

스케줄러의 `degraded_temperature_c`(80°C)와 `disable_temperature_c`(90°C)는 **하드웨어 보호**를 위한 것이다.

팬리스 환경에서 이 값을 그대로 두면 다음 문제가 생긴다.

```text
300초 지속 부하 → 세 노드 모두 90°C 초과 → 전부 스케줄링 제외
→ NPF-1201 NO_AVAILABLE_NODE → 벤치마크 중단
```

이 경우 측정되는 것은 하드웨어 성능이 아니라 **스케줄러의 온도 정책**이다.

따라서 순서를 다음과 같이 한다.

1. **S0 열 특성 파악**(`01-TECHSPEC.md` §20.2)을 먼저 수행해 정상 상태 온도를 확인한다.
2. 그 결과를 근거로 임계치를 설정한다. 정상 상태 온도보다 충분히 높아야 한다.
3. 확정한 임계치를 `environment-matrix.md` §10에 기록한다.
4. 이후 모든 공식 벤치마크는 같은 임계치를 사용한다.

온도로 인한 노드 제외가 실제로 발생하면 그것도 결과로 기록한다. 다만 **확장 효율 측정과는 분리해서 보고한다.** 두 가지가 섞이면 어느 쪽 원인인지 설명할 수 없다.

## 9.3 벤치마크 온도 조건

```text
시작 온도: S0에서 확인한 idle 온도 + 5°C 이내
Warmup: 30초
측정: 300초
반복 횟수: 5회
반복 사이 cooldown: 최대 180초 또는 시작 온도 도달 시점 중 빠른 쪽
```

팬리스라 냉각이 느리므로 cooldown에 **상한을 둔다.** 상한에 걸린 경우 그 사실과 실제 시작 온도를 결과에 기록한다. 무한정 기다리면 총 16시간 예산(§20.4)이 무너진다.

## 9.4 필수 기록 항목

노드별로 다음을 결과와 함께 저장한다.

```text
주변 온도
시작 온도
최고 온도
정상 상태 온도
throttling 시작 시점 (초)
CPU 주파수 변화
NPU 주파수 변화 (조회 가능한 경우)
온도로 인한 스케줄링 제외 발생 여부와 횟수
```

온도 조건이 다르면 특정 노드의 thermal throttling이 스케줄러 또는 네트워크 문제처럼 나타난다. 이것이 이 프로젝트에서 온도 기록을 선택 사항으로 두지 않는 이유다.

---

# 10. 시간 동기화

Scheduler와 세 노드 모두 `chrony`를 사용한다.

```bash
sudo systemctl enable --now chrony

chronyc tracking
chronyc sources
```

## 10.1 시간 측정 원칙

서로 다른 장치의 monotonic clock 값을 직접 비교하지 않는다.

Scheduler:

- End-to-end latency
- Scheduler queue time
- Routing time
- Node RPC round-trip time

Node:

- Local queue time
- Decode time
- Preprocess time
- NPU input preparation time
- Inference time
- Postprocess time

노드는 각 단계의 duration을 응답에 포함한다.

NTP 또는 chrony는 구조화 로그의 사건 순서를 맞추는 용도로 사용한다.

---

# 11. 프로세스 실행

## 11.1 NanoPi Worker

각 NanoPi에는 `npuforge-node`만 실행한다.

```text
systemd
└── npuforge-node.service
```

서비스 예시:

```ini
[Unit]
Description=NPUForge Node Agent
After=network-online.target
Wants=network-online.target

[Service]
User=npuforge
Group=npuforge
ExecStart=/opt/npuforge/bin/npuforge-node \
    --config /etc/npuforge/node.toml
Restart=always
RestartSec=2
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

## 11.2 Scheduler PC

Scheduler PC에는 다음 프로세스를 실행한다.

```text
npuforge-scheduler
npuforge-dashboard
Prometheus
npuforge-bench
```

선택적으로 Grafana를 추가할 수 있다.

---

# 12. 벤치마크 구성

## 12.1 1노드

```text
활성: KING
비활성: QUEEN, JACK
```

## 12.2 2노드

```text
활성: KING, QUEEN
비활성: JACK
```

## 12.3 3노드

```text
활성: KING, QUEEN, JACK
```

공식 실험에서는 프로세스나 전원을 끄기보다 Scheduler의 drain 또는 disable 기능을 사용한다.

이렇게 해야 네트워크, 전원, 온도 및 장비 배치 조건을 유지할 수 있다.

## 12.4 기본 부하 조건

```text
동시성: 1, 4, 16, 64
Warmup: 30초
측정: 300초
Cooldown: 60초 또는 시작 온도 이하 도달 시점까지
반복: 5회
```

시나리오별 축과 총 측정시간 예산은 `01-TECHSPEC.md` §20.2 및 §20.4를 따른다.

## 12.5 측정 지표

- Requests/sec
- FPS
- p50 latency
- p95 latency
- p99 latency
- 오류율
- 재시도율
- CPU 사용률
- 메모리 사용률
- NPU 사용률
- 네트워크 사용량
- 노드 온도
- 전력 사용량
- 확장 배율
- 확장 효율

---

# 13. 발표 데모용 물리 구성

```text
┌──────────────┐
│ 노트북       │ ← Dashboard
└──────┬───────┘
       │
┌──────▼───────┐
│ 2.5G Switch  │
└─┬────┬────┬──┘
  │    │    │
┌─▼─┐┌─▼─┐┌─▼─┐
│01 ││02 ││03 │
└───┘└───┘└───┘
```

각 노드에는 번호 라벨을 부착한다.

선택적으로 외부 상태 LED를 사용할 수 있다.

- 녹색: Healthy
- 노란색: Busy 또는 Degraded
- 빨간색: Unreachable
- 파란색: Recovering

## 13.1 장애 데모

발표 중에는 전원을 뽑기보다 QUEEN의 네트워크 케이블을 분리한다.

```text
3노드 처리
→ QUEEN 네트워크 분리
→ 헬스체크 실패
→ 자동 제외
→ 2노드로 서비스 지속
→ 케이블 재연결
→ Recovering
→ 자동 재편입
```

네트워크 분리는 전원 차단보다 복구 시간이 짧고 데모 진행이 안정적이다.

## 13.2 발표 장애 대비

- 예비 Ethernet 케이블
- 예비 USB-C 전원 어댑터
- 녹화된 동일 데모 영상
- Mock Backend 모드
- 사전 생성 벤치마크 결과
- 인터넷 없이 동작 가능한 구성

---

# 14. 권장 BOM

## 14.1 보유 현황 (2026-08-06 기준)

| 항목 | 수량 | 상태 |
|---|---:|---|
| NanoPi R76S | 3 | 보유. RAM 사양 확인 필요 |
| 1GbE 스위칭 허브 | 1 | 보유. 관리망 및 중간 단계 추론망으로 사용 |
| CAT6 케이블 | 1 | 보유 |
| Linux PC | 1 | 보유. NIC 속도 확인 필요 |
| USB-TTL UART 어댑터 | 1 | 보유 |

## 14.2 추가 확보 필요

우선순위 순이다.

| 항목 | 수량 | 우선순위 | 비고 |
|---|---:|---|---|
| **동일 모델 팬** | **3** | **최우선** | §9.1 조건 B. 동일 제조사·모델·회전수. 5V USB 팬 권장 |
| **2.5G/10G 스위치** | 1 | **최우선** | 2.5G 포트 ≥ 4 + SFP+ 업링크. ~~2.5GbE 전용 스위치~~ 는 폐기 |
| **스케줄러 서버** | 1 | **최우선** | PCIe 슬롯 필요. `dealer`(노트북)로는 불가 |
| **10G NIC (PCIe SFP+)** | 1 | **최우선** | 예: Intel X520 |
| **SFP+ DAC 케이블** | 1 | **최우선** | 서버 ↔ 스위치 업링크 |
| Ethernet 케이블 | 6~7 | 높음 | 관리망 3 + 추론망 4. **Cat5e로 충분** |
| USB 전력 측정기 | 3 | 중간 | 보드가 5V 입력이므로 USB 전력계로 가능. FPS/Watt 산출에 필요 |
| 예비 Ethernet 케이블 | 1 | 중간 | 발표 장애 대비 |
| 예비 전원 어댑터 | 1 | 중간 | 발표 장애 대비. **5V 4A** |

### 전원 어댑터는 해결됨 (2026-08-10)

**5V 4A × 3개로 교체 완료.** 구매 목록에서 제외한다.

교체 전 어댑터는 무부하에서도 4.983V로 5V를 유지하지 못해 고부하에서 보드가 하드 리셋되었다. 교체 후 3대 동시 지속 부하에서도 5.05V 이상을 유지한다.

**보드 입력은 5V다.** 커널 디바이스 트리의 `vcc12v_dcin` 표기에 속지 않는다. 실측은 `/sys/class/power_supply/simple-vin/voltage_now`로 확인한다.

### 팬 선정 기준

| 항목 | 기준 |
|---|---|
| 전원 | 5V USB 권장 (보드와 동일 전압, 별도 어댑터 불필요) |
| 수량 | **3개, 동일 모델** |
| 속도 | 고정 회전수 또는 세 대 동일 설정 가능한 것 |
| 소음 | 발표 데모에서 사용하므로 고려 |
| 장착 | 보드에 직접 부착하거나 동일 거리·각도로 거치 |

**속도 조절 기능이 있다면 세 팬을 같은 값으로 고정한다.** 회전수가 다르면 노드별 냉각 조건이 달라져 §9.1의 전제가 깨진다.

### 케이블 등급에 대하여

**CAT6를 추가 구매할 필요는 없다.**

- 1GbE: Cat5/Cat5e로 충분
- 2.5GBASE-T(IEEE 802.3bz): **Cat5e로 100m까지 지원**한다. 이 규격 자체가 기존 Cat5e 배선을 재활용하려고 만들어졌다.

보유 중인 케이블을 그대로 쓰고, 부족한 수량만 채우면 된다. 실제로 구매가 필요한 것은 **2.5G/10G 스위치, 스케줄러 서버, 10G NIC, SFP+ DAC** 네 가지다(§3.3.2).

### 냉각 장비를 구매하지 않는 이유

방열 케이스와 팬은 BOM에서 제외했다. §9.1의 팬리스 유지 결정에 따른 것이며, thermal throttling을 제거 대상이 아니라 측정 대상으로 다루기 때문이다.

---

# 15. 초기 구축 순서

## Step 1. 하드웨어 통일

- 세 보드 RAM 사양 확인
- 동일 저장장치 준비
- 동일 방열판과 팬 설치
- 동일 전원 어댑터 사용

## Step 2. OS 복제

- 기준 노드 한 대 구성
- OS, 커널, 패키지, RKNN Runtime 설치
- 이미지를 나머지 두 노드에 복제
- Hostname과 IP만 변경

## Step 3. 네트워크 검증

```bash
ping 10.20.0.21
ping 10.20.0.22
ping 10.20.0.23

iperf3 -s
iperf3 -c <target-ip>
```

노드별 링크 속도 확인:

```bash
ethtool eth0
```

## Step 4. RKNN 단일 노드 검증

- 동일 모델 실행
- 동일 입력으로 결과 확인
- 반복 추론 안정성 확인
- 추론시간 기록

## Step 5. 세 노드 일치 검증

- 모델 SHA-256 확인
- 바이너리 SHA-256 확인
- Runtime 버전 확인
- 커널 및 NPU 드라이버 확인
- 동일 입력 결과 비교

## Step 6. NPUForge Node 배포

- 전용 사용자 생성
- 바이너리 설치
- 설정 파일 배포
- systemd 등록
- Scheduler 자동 등록 확인

## Step 7. 기준 벤치마크

- 1노드
- 2노드
- 3노드
- Round Robin
- 온도 및 전력 기록

---

# 16. 최종 구성 기준

NPUForge v0.1 공식 하드웨어 구성은 다음과 같이 정의한다.

```text
Worker Node:
  NanoPi R76S × 3
  SoC     : Rockchip RK3576 (4× A72 @2.2GHz + 4× A53 @1.8GHz)
  NPU     : 6 TOPS
  GPU     : Mali-G52 MC3
  Network : 2.5GbE × 2 (관리망 1 + 추론망 1)
  Storage : eMMC (M.2는 SDIO이므로 NVMe 불가)
  Cooling : 팬리스 유지. throttling은 측정 대상
  동일 OS / Kernel / RKNN Runtime / Model / Power Supply

Scheduler:
  별도 Linux PC (2.5GbE NIC 필수)
  NanoPi에서 실행하지 않음

Network:
  관리망 : 기존 네트워크, 1GbE
  추론망 : 2.5GbE Star Topology, 10.20.0.0/24, Static IP, MTU 1500

Storage:
  Worker는 eMMC
  Benchmark 데이터와 결과는 Scheduler에 저장
```

## 16.1 RK3588에서 RK3576으로의 변경 (2026-08-06)

문서 초안은 RK3588 기반 NanoPi R6C를 전제로 작성했으나, 실제 보유 장비는 **RK3576 기반 NanoPi R76S**로 확인되었다.

주요 차이와 영향:

| 항목 | RK3588 (초안 전제) | RK3576 (실제) | 영향 |
|---|---|---|---|
| CPU | A76 + A55 | A72 @2.2 + A53 @1.8 | 전처리·디코딩 성능 낮음. 병목이 NPU가 아닐 가능성 ↑ |
| NPU | 6 TOPS | 6 TOPS | **없음.** 발표 제목 유지 |
| 네트워크 | 2.5G + 1G | **2.5G × 2** | 관리망 분리가 기본 구성이 됨 |
| M.2 | NVMe 가능 | SDIO (Wi-Fi 전용) | NVMe 실험 제외 |
| 냉각 | 팬 전제 | 팬리스 | throttling을 측정 대상으로 전환 |
| RKNN | `target_platform='rk3588'` | `target_platform='rk3576'` | 모델 재변환 필요. `.rknn` 파일은 플랫폼 간 비호환 |

CPU가 약해진 것은 이 프로젝트에서 오히려 다룰 거리가 늘어난 것에 가깝다. 전처리와 JPEG 디코딩은 CPU가 수행하므로, 병목이 NPU가 아니라 CPU 전처리로 나타날 경우 그 자체가 "TOPS 수치가 실제 처리량을 대표하지 못한다"는 본 프로젝트의 주장을 뒷받침하는 결과가 된다.

이 구성은 다음 세 목적을 동시에 만족한다.

- 2026년 11월 FOSS for All Conference 발표 데모
- 재현 가능한 오픈소스 벤치마크
- 박사논문 및 후속 연구용 실험 플랫폼

---

<a id="03-development-requirements"></a>

# NPUForge Development Requirements

- 문서명: `03-DEVELOPMENT-REQUIREMENTS.md`
- 프로젝트명: NPUForge
- 문서 버전: v0.2
- 대상 릴리스: NPUForge v0.1
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

본 문서는 NPUForge v0.1 개발을 위해 추가로 필요한 소프트웨어, 개발환경, 계측 도구, 자동화, 오픈소스 공개 준비 및 발표용 구성 요소를 정의한다.

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
- FFI 오류를 NPUForge 오류 코드로 변환

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
NPUForge Dashboard
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

발표 화면은 NPUForge 자체 Dashboard를 우선한다.

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

NPUForge 자체 소스코드는 Apache License 2.0을 우선 검토한다.

권장 구조:

```text
NPUForge Source       : Apache-2.0
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

- RKNN SDK Binary를 NPUForge 저장소에 임의로 포함하지 않음
- 사용자가 공식 경로에서 Runtime을 설치하도록 안내
- 모델 원본 라이선스 확인
- 변환된 `.rknn` 파일의 재배포 조건 확인
- 데이터 세트 이미지의 재배포 가능 여부 확인
- 제3자 Rust 및 C 라이브러리 라이선스 목록화

---

## 5.2 README 및 설치 문서

README 필수 내용:

```text
NPUForge 소개
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
NPUForge Logo
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

다음 조건이 충족되면 NPUForge v0.1 본개발 준비가 완료된 것으로 판단한다.

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

현재 보유한 NanoPi R76S 3대와 별도 Linux PC만으로 NPUForge v0.1 개발은 가능하다.

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

NPUForge v0.1의 성공은 이론상 18 TOPS라는 숫자가 아니라, 실제 확장 효율과 손실 원인을 재현 가능하게 증명하는 데 있다.

---

<a id="glossary"></a>

# 기술 용어 정리 (Glossary)

- 최종 갱신: **2026-08-21**
- 범위: S2~S4 실험 계보에서 실제로 등장한 용어 전부. 정의만 적지 않고
  **이 프로젝트에서 어떤 값·판단과 묶여 있는지**를 함께 적는다.
- 관련: [`experiments/README.md`](#experiments-readme)(실험 대장),
  [`01-TECHSPEC.md`](#01-techspec), [`RESULTS.md`](#results)

---

## 1. 실험 ID 체계

| ID | 질문 | 결과 요약 |
|---|---|---|
| **S0-A** | 팬리스 지속 부하에서 운영점이 유지되는가 | 열화 11.3%, CPU 2208→816 MHz |
| **S0-B** | 능동 냉각 지속 부하 | 열화 1.9%, 클럭 강등 0회 |
| **S0-C** | 부하 인지 정책이 열 불균질 손실을 회수하는가 | 1차 herding 버그 발견 → 2·3차 회수 확인 → 4차 게이트 미달 |
| **S0-D** | 이질을 결정론적으로 만들 수 있는가 | 가능. 클럭 캡으로 편차 1.12~3.93× |
| **S2** | 노드를 늘리면 선형으로 늘어나는가 | 112.9 / 229.0 / 338.4, 3.00× |
| **S3** | 각 구성의 진짜 상한(ceiling)은 | 115.2 / 232.0 / 341.8 |
| **S3.5** | −30% 손실이 어디서 오는가 | 전송 경로로 좁힘 |
| **S3.5b** | CPU0 softirq 편중이 원인인가 | null (−0.2%) |
| **S3.6** | flow control 인가 커넥션인가 | 커넥션. window 확대는 역효과 |
| **S3.7a** | 커넥션 몇 개가 최적인가 (고정 부하) | c4 에서 knee |
| **S3.7b** | 각 구성의 **운영점**은 | 셋 다 c12. conn2 가 우세 |
| **S3.7c** | 운영점에서 RPS 는 효과가 있는가 | null (−0.8%) |
| **S3.8** | 최적화가 scale-out 을 해치는가 | 387.2 inf/s, eff 95.3% |
| **S3.9a** | 3N efficiency 손실의 출처 | 서버 자원 배제, tail 증가 |
| **S3.9b** | 노드 쪽 남은 비용 | syscall 은 ~1%. 유저 시간이 커널보다 크다 |
| **S4** | io_uring 이 필요한가 | **아니다 — 측정으로 반박됨**(S3.9b) |

> **명명 규칙** — 정수(S2, S3)는 원래 계획된 실험. 소수점(S3.5, S3.7a)은
> 측정 결과가 새로 요구한 실험이다. 계획이 아니라 **데이터가 다음 실험을
> 정했다**는 기록이기도 하다.

---

## 2. 측정 방법론

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **closed-loop** | 동시 요청 수(concurrency)를 고정하고, 응답이 와야 다음 요청을 보내는 부하 모델 | bench 가 이 방식. 절대 지연을 SLA 로 인용하면 안 되고 **구성 간 비교에만** 쓴다 |
| **open-loop** | 응답과 무관하게 정해진 도착률로 계속 보내는 모델 | 미사용 |
| **coordinated omission** | closed-loop 에서 시스템이 느려지면 요청 자체가 덜 발생해 **지연이 과소 측정**되는 현상 | bench `--help` 에 경고로 명시 |
| **Little's law** | `동시요청수 = 처리량 × 평균지연` | S3.9a 에서 efficiency 손실이 **평균 지연 증가와 정확히 일치**함을 보이는 데 사용 |
| **saturation / ceiling** | 부하를 더 줘도 처리량이 안 오르는 상한 | S3 가 구성별로 측정 |
| **operating point (운영점)** | 실제로 운전할 부하 지점 | **peak 의 98% 이상을 내는 가장 낮은 concurrency** 로 정의(코드 상수) |
| **concurrency knee** | 장치를 포화시키는 데 필요한 동시 요청 수 | 시험 범위에서 **c12/node**, 커넥션 수와 무관하게 관측 |
| **connection knee** | 그 요청을 몇 갈래 커넥션으로 나눌지의 최적점 | 고정 부하에서 c4, 운영점 기준으로 **conn2** |
| **overload region** | 포화 이후 구간. 처리량은 그대로고 지연만 증가 | c24~c64 전 구간. **여기서 구성을 비교하면 결론이 뒤집힌다** |
| **short-run / sustained operating point** | 60초 기준 / thermal steady-state 기준 운영점 | 능동 냉각에서는 같고(−1.9%), 팬리스에서는 갈라진다(−11.3%) |
| **steady-state** | 시간에 따라 값이 더 안 변하는 구간 | S0 정의: **마지막 1/3 구간 평균** |
| **degradation** | `1 − steady / peak` | 판정: <3% 없음 / 3~10% 경미 / >10% 뚜렷 |
| **scaling / efficiency** | `tp_N / tp_1` / `그것을 N 으로 나눈 값` | optimized 3N: 2.86× / 95.3% |
| **rotation (조건 회전)** | 반복마다 조건 순서를 바꿔 시간·온도 드리프트를 상쇄 | 모든 A/B 하네스에 적용 |
| **preheat / reheat** | 측정 전 부하로 열 상태를 맞추는 것 | S0-C 재실행에서 **정책마다** 재예열 |
| **freeze (동결)** | 측정 중 코드·설정·모델을 바꾸지 않는 것 | 바이너리를 `*.frozen-<commit>` 으로 보존 |
| **verdict** | bench 가 run 유효성을 스스로 판정한 결과 | `valid` + `reasons`. 이상 run 도 지우지 않는다 |
| **preflight** | 측정 직전 하드 실패 검사 | 별칭↔hostname, 해시, governor, 온도, 전압, **추론 정확도** |
| **probe bench** | 본측정 전 짧게 던져 조건을 확인하는 부하 | 노드 수 검증에 사용 — S3.8 에서 6개 구성을 걸러냈다 |
| **capacity heterogeneity** | 노드간 처리 능력 편차 | 열 유래(S0-A/C)와 **클럭 캡 유래**(S0-D)를 구분해 적는다. 스케줄러가 보는 것은 capacity 이지 그 원인이 아니다 |
| **이질 게이지** | 편차를 재는 관측량 | **round-robin 의 노드별 p50 최대/최소.** RR 은 적응하지 않으므로 균등 부하 아래 raw capacity 편차가 그대로 드러난다 |
| **utime / stime** | 유저 / 커널 CPU 시간 | `/proc/PID/stat`. 커널에 syscall 진입·TCP 스택·`copy_to_user`, 유저에 직렬화·유저공간 copy·HTTP/2 프레이밍. **io_uring 이 줄이는 것은 stime 의 일부** |
| **한쪽 방향 검정** | 편향 방향이 정해진 측정 | `strace -c` 는 ptrace 로 값을 **부풀린다** → 부풀린 값이 작으면 실제는 확정적으로 더 작다 |

### 2.1 percentile 집계

| 용어 | 뜻 |
|---|---|
| **nearest-rank** | 보간 없이 "정렬 후 그 지점 이상 첫 값". bench 가 쓰는 방식 |
| **run-level percentile** | 한 run 안에서 그 run 의 요청들로 계산한 percentile |
| **pooled percentile** | 여러 run 의 요청을 **전부 합쳐** 다시 계산한 percentile |
| **주의** | 이 저장소의 표는 전부 **run-level 의 평균**이다. pooled 가 아니다. run-level 평균은 각 run 의 최악 구간이 희석돼 **tail 을 낮게 보이게 한다**. 조건 간 비교에는 유효하나 절대값을 "이 시스템의 p99" 로 인용하면 안 된다 |

---

## 3. 성능 지표

| 용어 | 뜻 |
|---|---|
| **inf/s** | 초당 추론 건수(throughput) |
| **p50 / p95 / p99 / max** | 지연 분포의 백분위. p50=중앙값 |
| **tail latency** | 상위 백분위(p95/p99) 지연. "일부 요청이 얼마나 늦는가" |
| **tail amplification** | 처리량이 올라도 tail 이 더 크게 나빠지는 현상 |
| **balance (%p)** | 노드 간 요청 분배 편차. 0 이면 완전 균등 |
| **error_rate** | 실패 비율. 이 저장소 전 실험에서 **0** |
| **TimingBreakdown** | 응답에 실려 오는 11단계 시간 분해 (proto `Timing`) |
| `scheduler_queue` | 스케줄러 내부 대기 |
| `scheduler_route` | 정책 선택 시간 |
| `network_to_node` / `network_to_client` | **왕복 전체 − 노드 내부 시간**을 절반씩 나눈 값. 서로 다른 장치의 절대 시각을 빼지 않기 위한 방법 |
| `node_queue` | 노드 워커 풀 대기 |
| `decode / preprocess / npu_input / inference / postprocess` | 노드 내부 단계. 이 프로젝트는 raw RGB 입력·raw tensor 출력이라 inference 외에는 ~0 |
| `end_to_end` | 스케줄러가 잰 전체 |
| **syscalls/req · ctx switches/req · cycles/req** | TECHSPEC §15.4 가 요구하는 io_uring 판단 지표 |

---

## 4. 네트워크 · 커널

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **full-duplex** | 송신·수신이 각자의 대역폭을 갖는 것 | **두 방향을 한 링크 예산에 합산하면 안 된다.** S3.8 에서 이 실수로 "10G 76%" 를 썼다가 S3.9a 에서 철회(실제 방향당 40%) |
| **goodput** | 헤더를 뺀 실제 payload 처리량 | iperf3 가 재는 값. 보드 링크 실측 2.34 Gbps |
| **MTU** | 한 프레임에 담기는 최대 payload. 여기선 1500 | |
| **RSS** (Receive Side Scaling) | NIC 이 **하드웨어 다중 큐**로 패킷을 여러 코어에 분산 | 서버 NIC 은 RX 큐 24개. 보드는 **1개** |
| **RPS** (Receive Packet Steering) | 커널이 **소프트웨어로** flow 해시 기준 분산 | 보드에서 시도했으나 두 번 다 null. **흐름이 1개면 나눌 대상이 없다** |
| **softirq / NET_RX** | 커널이 인터럽트 후처리를 하는 경량 컨텍스트. 네트워크 수신 처리가 여기서 돈다 | 보드 CPU0 %soft 51.5% → 단일 큐 때문 |
| **IRQ affinity** | 인터럽트를 어느 코어가 받을지 | 보드는 NIC IRQ 가 CPU0 고정 |
| **cwnd / ssthresh** | TCP 혼잡 윈도 / 느린 시작 임계값 | 3N 에서 cwnd 176 → 106~119 로 눌림 |
| **retransmission (재전송)** | 손실·지연으로 다시 보낸 세그먼트 | 커넥션당 재전송률 0.055% → **0.19%** (3.5배) |
| **incast / speed mismatch** | 빠른 링크(10G)에서 느린 링크(2.5G)로 몰릴 때 스위치 egress 에 생기는 버퍼링·손실 | 3N efficiency 손실의 **유력 가설(미검증)** |
| **bufferbloat** | 과도한 버퍼가 지연을 부풀리는 현상 | 64 MB window 확대 시 −36.3% 의 해석 가설 |
| **/proc/stat · /proc/net/dev · /proc/interrupts · /proc/softirqs** | 커널이 노출하는 카운터 | 서버에 sysstat 이 없어 이것들의 델타로 직접 계산 |
| **ss -tin** | 소켓별 TCP 상태(rtt, cwnd, retrans, bytes_sent) | 커넥션 수·혼잡 상태 관측 |

---

## 5. HTTP/2 · gRPC

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **HTTP/2 multiplexing** | 하나의 TCP 커넥션에서 여러 스트림을 동시에 실어 나르는 것 | 그래서 "커넥션이 1개" 만으로 병목이라 단정할 수 없다 |
| **stream** | 커넥션 안의 논리적 요청/응답 한 쌍 | 요청 1건 = 스트림 1개 |
| **flow control window** | 수신자가 "여기까지 받을 수 있다" 고 광고하는 크기. **스트림 단위**와 **커넥션 단위**가 따로 있다 | h2 기본 65,535 byte. 이 프로젝트는 메시지가 1.2 MB |
| **WINDOW_UPDATE** | window 를 다시 열어주는 프레임 | window 가 작으면 이 왕복이 반복돼 stop-and-wait 가 된다 |
| **DATA frame** | 실제 payload 를 나르는 프레임 | 응답 1.218 MB 를 약 14.4 KB 씩 쪼개 보냄(write syscall 84.4회/req) |
| **head-of-line blocking** | 앞선 것이 막히면 뒤가 다 막히는 현상 | 다중화된 스트림이 같은 커넥션 자원을 다툴 때 |
| **tonic** | Rust gRPC 구현 (hyper + h2 기반) | v0.12.3 |
| **h2 / hyper** | HTTP/2 프로토콜 / HTTP 라이브러리 | h2 0.4.15, hyper 1.11.0 |
| **prost** | Protocol Buffers 코드 생성기 | `.proto` → Rust 타입 |
| **protoc** | protobuf 컴파일러 | 빌드 전제. Windows 개발 PC 에는 없어 서버·보드에서 빌드 |
| **`node_connections`** | **노드당** gRPC 커넥션 수 (이 프로젝트가 추가한 설정) | 1N→2, 2N→4, 3N→6 total. **클러스터 전체 합이 아니다** |

---

## 6. 스케줄링

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **round-robin (RR)** | 상태를 보지 않고 순서대로 배정 | 기본값. 구조적으로 균등하지만 **느려진 노드에도 똑같이 보낸다** |
| **least-queue / LOR** (least-outstanding-requests) | 미완료 요청이 가장 적은 노드 선택 | 서비스 **속도**는 모른다. 동시 버스트에서는 균등 분배가 정상 동작 |
| **ECT** (Estimated Completion Time) | `(미완료+1) × EWMA_inference + EWMA_network + 패널티` 로 완료 예상시각을 추정 | 서비스 속도 차이를 반영할 수 있는 유일한 정책. 단 **EWMA 가 채워져야** 동작 |
| **EWMA** | 지수 이동 평균 | 추론시간·네트워크 왕복시간 추적 |
| **herding (herd behavior)** | 여러 결정 주체가 **같은 낡은 정보**를 보고 동시에 같은 선택을 하는 현상 | S0-C 의 원인. 처리량 55~58% 붕괴 |
| **stale state / state freshness** | 상태 정보가 낡은 정도 | 하트비트 1초 vs 디스패치 ~3ms → **수백 배 차이** |
| **control-loop sampling problem** | 피드백 주기가 시스템 변화 주기보다 길어 제어가 실패하는 문제 | herding 의 상위 개념. 정책 튜닝 문제가 아니다 |
| **reservation (예약)** | 선택과 동시에 부하를 점유 표시하는 것 | `select_and_reserve()` 가 한 임계구역에서 처리 |
| **RAII guard** | 값이 스코프를 벗어날 때 자동 정리되는 패턴 | `Reservation` 의 `Drop` 이 감소 — 성공·오류·타임아웃·취소·재시도 **모든 경로**를 닫는다 |
| **`local_in_flight`** | 스케줄러가 보냈지만 아직 안 끝난 요청 수 (즉시 갱신) | 정책의 **1차 신호**. 하트비트 값과 더하지 않는다(같은 요청을 두 번 세게 됨) |
| **`health.in_flight` / `queue_depth`** | 노드가 하트비트에 실어 보낸 관측값 (최대 1초 stale) | health 판정·tiebreaker 용도로만 |
| **busy_queue_depth / degraded / disable temperature** | 노드 상태 분류 임계치 | 8 / 80°C / 90°C |
| **drain** | 새 요청을 안 보내고 큐를 비우는 상태 | 운영자 지정 상태 |

---

## 7. 하드웨어 · 열

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **RK3576** | Rockchip SoC. 4×Cortex-A72 + 4×Cortex-A53, NPU 2코어 | 노드 보드 3대 |
| **big.LITTLE** | 고성능/저전력 코어를 섞은 구성 | A72 2208 MHz(policy4), A53 2016 MHz(policy0) |
| **cpufreq governor** | CPU 주파수 정책 | `performance` 고정(최대 클럭 유지). 대안 `ondemand` |
| **devfreq** | CPU 외 장치(NPU·GPU·DDR)의 주파수 관리 | NPU 300~950 MHz |
| **thermal zone** | 커널이 노출하는 온도 센서 | soc / bigcore / little-core / ddr / **npu** / gpu 6개 |
| **thermal throttling** | 온도로 클럭을 낮추는 것 | **NPU 는 한 번도 안 떨어졌다.** 떨어지는 것은 CPU |
| **thermal steady-state** | 발열과 방열이 균형을 이룬 온도 평탄역 | 능동 냉각 58~61°C(5분), 팬리스 86~88°C |
| **thermal heterogeneity (열 불균질)** | 같은 모델 보드인데 열 조건이 달라 성능이 갈리는 것 | 팬리스에서 king 816 / jack 1200 / queen 1416 MHz |
| **boot_id** | 부팅마다 바뀌는 식별자 | run 도중 보드 리셋 감지 → 그 측정은 무효 |
| **입력 전압 감시** | 어댑터 용량 부족 조기 경보 | 5.00 V 미만이면 preflight 실패 |
| **2.5GbE / 10GbE** | 링크 속도 | 보드 2.5G, 서버 10G. **속도 불일치가 §4 의 incast 가설** |

---

## 8. 모델 · NPU 런타임

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **RKNN** | Rockchip NPU 런타임 | `librknnrt.so` 2.3.0 |
| **RKNPU driver** | 커널 드라이버 | v0.9.8 |
| **YOLOv8n** | 객체 검출 모델 | 입력 640×640×3 |
| **INT8 양자화** | 가중치·활성을 8비트 정수로 | FP16 대비 처리량 +17.3% |
| **`want_float`** | 출력을 float 로 역양자화해 받을지 | **0**(정수 그대로). 출력 크기 4분의 1, 처리량 +17.3% |
| **blob v2** | 텐서 여러 개를 담는 자체 직렬화 형식 | 텐서당 36 byte 헤더에 `scale`·`zero_point` 동봉 |
| **payload 크기** | | 요청 **1,228,800 B**, 응답 **1,218,000 B** (합 2,446,800 B/추론) |
| **postprocess (DFL + NMS)** | 검출 결과 디코딩과 중복 제거 | **현재 노드에서 안 한다.** raw tensor 를 그대로 보냄 → 응답이 1.2 MB. 노드에서 하면 수 KB 로 줄어든다(미구현 아이디어) |
| **warmup** | 첫 추론의 초기화 비용을 제외하기 위한 예열 | 집계에서 제외 |
| **worker_count** | 노드의 동시 추론 워커 수 | 8. **워커가 독립적이지 않다** — 로컬 direct 8워커가 161.5 inf/s |

---

## 9. 소프트웨어 스택

| 용어 | 뜻 |
|---|---|
| **tokio** | Rust 비동기 런타임. 노드는 multi_thread(워커 = 코어 수 8) |
| **`spawn_blocking`** | 블로킹 작업을 별도 스레드 풀로 보내는 tokio API. RKNN FFI 호출이 여기서 돈다 |
| **async worker vs blocking pool** | 네트워크·protobuf 는 async 워커 8개, 추론은 blocking 풀 — **같은 8코어를 나눠 쓴다** |
| **`parking_lot`** | 더 빠른 Mutex/RwLock 구현 |
| **`Arc<AtomicU32>`** | 스레드 간 공유되는 원자적 카운터 |
| **`Bytes`** | 참조 카운팅되는 바이트 버퍼(복사 없이 공유) |
| **`to_vec()`** | 복사를 만드는 호출. 남은 gap 후보 중 하나 |
| **feature flag** | 컴파일 시 기능 선택. 노드는 `--features rknn` 필요 (없으면 Mock 백엔드가 빌드됨) |
| **`RKNN_SDK_PATH`** | 빌드 시 `rknn_api.h` 위치 |

---

## 10. 진단 도구

| 도구 | 용도 | 비고 |
|---|---|---|
| **iperf3** | 링크 대역폭 실측 | 보드→서버 2.34 Gbps |
| **mpstat** | 코어별 CPU 분해(%usr/%sys/%soft/%idle) | 보드에만 있음 |
| **pidstat** | 프로세스·스레드별 CPU | 보드에만 있음 |
| **ethtool** | 링크 속도·NIC 통계·offload 설정 | 양쪽 다 있음 |
| **ss** | 소켓 상태 | 커넥션 수·TCP 내부 상태 |
| **perf** | PMU 기반 프로파일링 | **양쪽 다 없음.** cycles/req 는 근사값 |
| **`/proc` 델타** | sysstat 없이 CPU·네트워크·syscall 집계 | 서버 프로파일에 사용 |
| **thermal-logger.sh** | 보드 온도·주파수·전압 1초 샘플러 | |

---

## 11. 이 프로젝트의 구성요소

| 이름 | 역할 |
|---|---|
| `npuforge-scheduler` | 중앙 스케줄러. 클라이언트 요청을 노드로 분배 (x86_64, 서버) |
| `npuforge-node` | 노드 에이전트. NPU 추론 수행 (aarch64, 보드 3대) |
| `npuforge-bench` | 부하 생성·집계·run 유효성 판정 |
| `npuforge-proto` | `.proto` 단일 출처 |
| `npuforge-rknn` | RKNN 백엔드 |
| `npuforge-mock-backend` | 하드웨어 없이 개발·테스트용 |
| `npuforge-common` | 타입·오류코드·설정·백엔드 인터페이스 |
| **king / queen / jack** | 노드 보드 3대의 이름 (SSH 별칭 `npuforge-k/q/j`) |
| **server** | 스케줄러 + bench 호스트 (`npuforge-server`) |

### 11.1 오류 코드

| 코드 | 뜻 |
|---|---|
| `NPF-0000` | 성공 |
| `NPF-1002` | payload 크기 초과 |
| `NPF-1303` | 노드 과부하(큐 가득) |
| `NodeUnavailable` | 전송 실패 → 헬스 카운터에 반영 |
| `NoAvailableNode` | 처리 가능한 노드 없음 |

---

## 12. 이 프로젝트에서 정한 실험 규칙

측정 전에 정하고 **결과에 맞춰 바꾸지 않는** 값들이다.

| 규칙 | 값 | 근거 |
|---|---|---|
| operating concurrency | peak 의 **98%** 이상을 내는 가장 낮은 concurrency | 99% 는 run 간 SD(±1 inf/s)와 겹친다 |
| steady-state | 마지막 **1/3** 구간 평균 | |
| degradation 판정 | <3% / 3~10% / >10% | |
| Selected operating point | 처리량 최대의 **97%** 이상 중 p95 최소 | 통계적 최적이 아니라 **engineering heuristic** |
| 정책 이동 판정 | 분배 **3%p** 이상 이동 = 이동, 처리량 **2%** 이상 = 회수 | |
| 강한 이질 게이트 | RR 노드 p50 최대/최소 **≥ 2.0×** | S0-A 2.4× ↔ S0-C 2차 1.33× 사이 (S0-C §17.2) |
| LQ vs ECT 판정 밴드 | 처리량 **2%**, p99 **5%** | n=4 에서 그보다 작은 차이는 못 쓴다 (S0-C §17.3) |
| 현직 tie-break | 밴드를 못 넘으면 **기존 기본값 유지** | 현직을 끌어내리려면 적극적 근거가 필요하다 |

---

## 13. 방법론 교훈에서 나온 표현

| 표현 | 뜻 |
|---|---|
| **"배제는 조건부다"** | 한 번 배제한 병목 후보도 조건이 바뀌면 다시 열린다. 판정에는 **어떤 조건에서** 가 붙어야 한다 |
| **"Optimize at the operating point, not in the overload region"** | 과부하 구간에서 구성을 비교하면 configuration effect 가 아니라 overload behavior 를 본다 |
| **"조용한 실패를 큰 소리로"** | 하네스가 조건 미달이면 그냥 멈춘다. 노드 수 검증·설정 주입 검증·TCP 커넥션 수 물증 |
| **"프로세스가 떠 있다 ≠ 트래픽을 받는다"** | 노드 수는 probe bench 의 **응답 노드 ID 분포**로 확인한다 |
| **"두 측정이 일치해도 해석이 옳다는 뜻은 아니다"** | 둘 다 같은 편향이면 재현성은 편향만 확인해 준다 |
| **"성능이 이상하면 구현이 의도대로 도는가를 먼저"** | 55% 는 품질 차이의 크기가 아니다 |
| **"두 측정량을 곱하지 않는다"** | 처리량 손실 %와 지연 구성비 %는 다른 축 |
| **"비용이지 제약이 아니다"** | 포화되지 않은 자원의 사용량(CPU-ms/req)을 줄여도 처리량은 오르지 않는다. S4 판정의 핵심 |
| **"계기가 다른 물리량을 재고 있을 수 있다"** | 출력이 예상과 다르면 **계측기부터 의심한다.** 임계를 옮기는 것과 계기를 고치는 것은 다르다 |
| **"중단했다를 믿지 말고 공유 자원 쪽에서 확인한다"** | 로컬 프로세스 관측은 플랫폼에 따라 거짓말을 한다. 클러스터가 비었는지는 **클러스터에게 묻는다** |
| **"손으로 관리하는 파생 수치는 갈라진다"** | run 합계·백분율은 스크립트가 세게 하고 출처를 적는다 |

---

<a id="infrastructure"></a>

# NPUForge 인프라 현황

- 문서명: `infrastructure.md`
- 최종 갱신: 2026-08-20
- 관련 문서: `board-worklog.md` (시간순 작업 이력), `environment-matrix.md` (버전 고정)

이 문서는 **현재 상태의 스냅샷**이다. 어떻게 그 상태에 도달했는지는 `board-worklog.md`를 본다.

> **2026-08-20 대개편.** 2.5G/10G 스위치·10G 서버 도입으로 M3 차단 요소가
> 전부 해소되고 IP·역할이 크게 바뀌었다. 이전 판(dealer 노트북 + 1G 관리망)은
> 폐기됐다. 경위는 `board-worklog.md` §2.23.

---

# 1. 장비 구성

```text
                    ┌──────────────────────────────────┐
                    │ server   192.168.123.9           │
                    │ Rocky Linux 9.4 / x86_64         │
                    │ Core i7-4790 (4C/8T) / 16GB      │
                    │                                  │
                    │ · Scheduler (예정)               │
                    │ · Benchmark Client (예정)        │
                    └────────────────┬─────────────────┘
                                     │ 10GbE (enp1s0)   ← aggregation
                                     │
                    ┌────────────────▼─────────────────┐
                    │ NEXI NS-S25G10G-N                │
                    │ 2.5G x4 + 10G x2 (전부 RJ45)     │
                    └─┬────┬──────┬──────┬──────┬───────┘
              10G ────┘    │2.5G  │2.5G  │2.5G  └──2.5G── 인터넷(ipTIME)
          개발 PC(노트북)  │      │      │
          (1G NIC, 미활용) │      │      │
                    ┌──────▼┐ ┌───▼───┐ ┌▼──────┐
                    │ king  │ │ queen │ │ jack  │
                    │  .3   │ │  .5   │ │  .4   │
                    │ 6 TOPS│ │6 TOPS │ │6 TOPS │
                    └───────┘ └───────┘ └───────┘
                       Ubuntu 24.04 / RK3576 / aarch64
                       각 eth0 2.5G, static
```

| 호스트 | IP | 역할 | OS | 아키텍처 | 스위치 포트 |
|---|---|---|---|---|---|
| `server` | 192.168.123.9 | Scheduler / Bench | Rocky Linux 9.4 | x86_64 | **10G (6)** |
| `king` | 192.168.123.3 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (2) |
| `jack` | 192.168.123.4 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (4) |
| `queen` | 192.168.123.5 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (3) |
| 개발 PC | 192.168.123.26 | 코드 작성 / 원격 조작 | Windows | x86_64 | 10G (5) — **1G NIC** |
| 인터넷 | — | ipTIME 상위 | — | — | 2.5G (1) |

> **IP 고정 완료 (2026-08-20).** 개편 때 보드 IP 가 통째로 바뀌어
> (`.12/.16/.33` → `.3/.4/.5`) SSH 별칭이 낡아 노드를 못 찾았다. 라우터
> DHCP 예약 대신 **각 호스트에서 현재 IP 를 NetworkManager static 으로 고정**
> 했다(측정 재현성상 호스트 설정이 낫다). **4대 전부 `ipv4.method=manual`**
> (§2.3). `adrs/019-ssh-alias-not-ip.md`.

**`dealer`(옛 스케줄러, 노트북 .14) 는 제거됐다.** 응답 없음. 역할(스케줄러·
벤치)은 `server` 로 이관됐다. 모델 변환 Docker 도 dealer 에 있었으므로
변환 환경은 재구축 대상이다 (§6). 다만 모델은 이미 변환 완료라 당장 필요치 않다.

개발 PC 는 스위치 10G 포트(5)에 물려 있으나 **NIC 이 1G(현재 100Mb/s 협상)라
10G 를 못 낸다.** 벤치 클라이언트는 개발 PC 가 아니라 `server` 에서 돌린다.

---

# 2. 접속

## 2.1 SSH 별칭

개발 PC의 `~/.ssh/config`에 등록되어 있다. **IP 는 여기 한 곳에만 둔다**
(`adrs/019-ssh-alias-not-ip.md`).

```text
npuforge-k        → pi@192.168.123.3     (king)
npuforge-q        → pi@192.168.123.5     (queen)
npuforge-j        → pi@192.168.123.4     (jack)
npuforge-server   → root@192.168.123.9   (server)
```

모두 `~/.ssh/id_ed25519_npuforge` 키로 비밀번호 없이 접속된다.

> 이 키는 자동화 전용이며 passphrase가 없다. 공개 저장소나 신뢰할 수 없는 네트워크에 노출하지 않는다.

## 2.2 권한 승격

| 호스트 | 계정 | sudo | 비고 |
|---|---|---|---|
| king / queen / jack | `pi` | `NPUFORGE_SUDO_PASS` 로 전달 | `printf '%s\n' "$NPUFORGE_SUDO_PASS" \| sudo -S -p "" <cmd>` |
| server | `root` | 불필요 (root 직접) | 자동화 키가 root `authorized_keys` 에 등록됨 |

`sudo -S`는 stdin의 첫 줄을 비밀번호로 소비한다. 파일 내용을 파이프로 넘길 수
없으므로 파일을 쓸 때는 임시 파일을 거친다.

### 2.2.1 보드 자격증명은 벤더 기본값 그대로다 — 의도된 선택

보드 계정과 sudo 비밀번호는 **OS 이미지의 벤더 기본값을 바꾸지 않았다.**
숨기지 않고 적어 두는 편이 낫다고 판단했다.

| | |
|---|---|
| 전제 | 보드는 `192.168.123.0/24` 사설 대역, NAT 뒤. 인바운드 포워딩 없음 |
| 기본값 유지 | 벤더 기본값은 **이미 공개된 정보**다. 적어도 새로 알려주는 것이 없다 |
| 바꾸지 않는 이유 | 커스텀 값을 쓰면 **없던 비밀이 하나 생긴다.** 그 값이 문서·이력·사진 어디로든 새면 비밀번호 **패턴**이 노출되고, 그건 이 랩 밖으로 번지는 정보다 |

> **조건이 바뀌면 이 판단도 바뀐다.** 22번 포트를 외부로 포워딩하거나
> 보드를 격리되지 않은 망에 두는 순간 기본값은 즉시 문제가 된다.
> 배제와 마찬가지로 이 결정에도 **어떤 조건에서** 가 붙는다.

```bash
S() { printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" "$@"; }
cat > /tmp/f.new <<'H'
...
H
S cp /tmp/f.new /etc/target       # printf "text" | S tee ... 는 동작하지 않음
```

원격 실행 함정(백그라운드 기동·프로세스 카운트)은 `adrs/017-remote-exec-pitfalls-library.md`.

## 2.3 IP 고정 (호스트 static)

DHCP 재할당으로 IP 가 바뀌는 것을 막기 위해 **각 호스트에서 현재 IP 를
NetworkManager static 으로 고정**한다. 라우터(ipTIME) DHCP 예약 대신
호스트 설정을 쓰는 이유는 라우터가 바뀌어도 설정이 남아 측정 재현성이 낫기
때문이다. **현재 IP 를 그대로 고정하므로 SSH 세션은 끊기지 않는다.**

공통 파라미터: gateway `192.168.123.254`, DNS `210.94.0.73 210.220.163.82`,
prefix `/24`. 전부 NetworkManager 관리(netplan/networkd 아님).

```bash
# server (root, 연결명 enp1s0) — 완료
nmcli con mod enp1s0 ipv4.method manual \
  ipv4.addresses 192.168.123.9/24 ipv4.gateway 192.168.123.254 \
  ipv4.dns "210.94.0.73 210.220.163.82"
nmcli con up enp1s0

# 보드 (pi/sudo, 연결명 'Wired connection 1', eth0) — 완료 (2026-08-20)
#   king .3 / queen .5 / jack .4. 같은 IP 라 SSH 유지, 외부 도달 확인
```

> ⚠️ **DHCP 풀 충돌 주의.** `.3/.4/.5/.9` 가 ipTIME DHCP 풀 안이면 라우터가
> 그 주소를 다른 기기에 임대할 수 있다(호스트 static 은 라우터가 모른다).
> 완전 회피는 ipTIME 에서 해당 주소를 풀 밖으로 빼는 것(라우터 UI 작업).
> 소규모 홈랜에서 위험은 낮지만 남는 리스크다.

## 2.4 sudo 비번 파일

보드 자동화용 sudo 비번은 개발 PC 로컬 `~/.npuforge/sudo-pass` (chmod 600)
또는 환경변수 `NPUFORGE_SUDO_PASS` 로 전달한다. 저장소에 넣지 않는다.
`preflight-check.sh` 와 배포 스크립트가 이 경로를 읽는다.

---

# 3. 소프트웨어 현황

## 3.1 노드 (king / queen / jack)

| 항목 | 값 | 3대 일치 |
|---|---|---|
| SoC | Rockchip RK3576 | ✓ |
| NPU | 2코어, 300~950MHz, IOMMU 활성 | ✓ |
| RKNN Runtime | 2.3.0 (`librknnrt.so` SHA-256 동일) | ✓ |
| RKNPU Driver | v0.9.8 | ✓ |
| 커널 | 6.1.141 | ✓ |
| glibc | 2.39 | ✓ |
| RAM / eMMC | 4GB / 64GB | ✓ |
| Ubuntu 패치 레벨 | 24.04.4 | ✓ |
| gcc | 13.3.0-6ubuntu2~24.04.1 | ✓ |
| CPU Governor | **`performance`** | ✓ 재부팅 유지 |
| eth0 링크 | **2.5G (2500Mb/s)** | ✓ 2026-08-20 실측 |
| **SSH 호스트 키** | **queen·jack 동일** | ✗ **미해결** |
| `.rknn` 모델 (FP16) | `459602ea…` 3대 배포 완료 | ✓ |
| `.rknn` 모델 (INT8) | `dba155d2…` **`king` 에만** | ✗ 배포 필요 |
| Rust 툴체인 | 1.97.1 | **`king` 에만.** 빌드 전용 |
| 측정 C 도구 | `~/npuforge-rknn-test/` | ✓ 해시 동일 |

**SSH 호스트 키가 queen·jack 에서 같다.** 두 보드를 암호학적으로 구분할 수
없어, IP 가 바뀌면 경고 없이 엉뚱한 보드에 붙는다. DHCP 라 IP 가 실제로
바뀌므로(§1) 위험이 크다. 조치 명령은 `TODO.md` §1.2.

`preflight-check.sh` 가 매 측정 전에 위 항목의 일치를 확인한다.

## 3.2 server (192.168.123.9)

| 항목 | 값 |
|---|---|
| OS | Rocky Linux 9.4 (Blue Onyx), 커널 5.14.0-427.13.1.el9_4 |
| 메인보드 | ASUS H81M-K (H81 칩셋) |
| CPU / RAM | **Core i7-4790 (4C/8T, 3.6~4.0GHz)** / **16GB DDR3-1600 non-ECC** |
| 디스크 | ST2000VN004 2TB, root LVM 70GB (65GB 여유) |
| NIC | `enp1s0` **Intel X550T 10GBASE-T, 10G full 실측** (2026-08-26), 드라이버 `ixgbe` |
| NIC 슬롯 | `PCIEX16_1` (CPU 직결). **PCIe 2.0 x4 로 동작** |
| 슬롯 상한 근거 | 루트 포트 `00:01.0` 의 `LnkCap: Speed 5GT/s, Width x16` — **메인보드 x16 슬롯 자체가 PCIe 2.0 상한**이다. 카드(`LnkCap 8GT/s x4`)가 아니라 슬롯이 정한다. 조치 불가 |
| 병목 여부 | **아니다.** PCIe 2.0 x4 = 방향당 약 16Gbps. 실사용은 3노드 합쳐 방향당 ~4.6Gbps 로 3배 여유 |
| 전체 인벤토리 | 신서버 [`hosts/server-i7-4790-20260826.md`](#hosts-server-i7-4790-20260826) · 구서버 [`hosts/server-xeon-e5-2630l-20260826.md`](#hosts-server-xeon-e5-2630l-20260826) |
| 방화벽 | firewalld active, zone `public`. gRPC 포트 개방 필요(측정 전) |
| 빌드 툴체인 | **rust/cargo 1.92, gcc 11.5, protoc 3.14, git** (2026-08-20 설치) |
| Docker | 미설치 — 모델 변환 필요 시 구축 |

> protoc 는 Rocky 9 기본 리포에 없고 **CRB 리포**(`dnf config-manager
> --set-enabled crb`)를 켜야 `protobuf-compiler` 가 잡힌다. tonic-build 0.12
> 가 시스템 protoc 를 요구한다.

**dealer(노트북)의 두 제약이 이 서버로 해소됐다.**

1. **RAM 3GB → 16GB.** 스케줄러 RSS 우려(페이로드 중계)가 크게 완화됐다.
   `environment-matrix.md` §10.1, `adrs/003-central-simple-scheduler.md`
2. **1GbE → 10GbE.** aggregation 대역 확보. §4 에서 실측했다.

**스케줄러(x86_64)는 server 에서 네이티브 빌드한다.** MSRV 1.85 < dnf rust
1.92 라 stable 채널로 빌드된다. 소스는 `git archive` tarball 을 scp 로 넘긴다
(server 는 foxden 직접 접근 불가, github 는 OK). 노드(aarch64)는 종전대로
king 에서 빌드한다. Windows→Linux 크로스빌드는 링커 문제로 쓰지 않는다.

## 3.2.1 서버 교체 (2026-08-26) — 기준선이 −7.5% 낮아졌다

구서버(Xeon E5-2630L ×2, **24 스레드**)가 물리적으로 교체되어 여분의
데스크톱(i7-4790, **8 스레드**)으로 옮겼다. **스케줄러 호스트가 바뀌었을 뿐
노드 3대·스위치·모델·바이너리는 그대로다.**

| | 구서버 (~2026-08-24) | 신서버 (2026-08-26~) |
|---|---|---|
| CPU | Xeon E5-2630L ×2 · 24T · **2.0~2.5GHz** | Core i7-4790 · 8T · 3.6~4.0GHz |
| RAM | 16GB | 16GB DDR3-1600 |
| NIC | **Intel X550T** `enp4s0` | **같은 카드** `enp1s0` (PCIe 2.0 x4) |
| **기준선 처리량** | **~391 inf/s** | **~360 inf/s** (3 run: 360.5 / 362.5 / 357.2) |
| 왕복 p50 | ~86 ms | ~93 ms |
| 노드 편차 | ~1.02× | ~1.07× |
| 오류율 | 0 | 0 |

> **10G NIC 은 같은 물리 카드다.** Intel X550T 가 한 장뿐이라 구서버에
> 꽂아 쓰다가 빼서 신서버에 옮겨 꽂았다. 그래서 **10G 경로의 하드웨어는
> 두 측정에서 동일하다** — NIC 은 통제된 변수이고, 바뀐 것은 호스트
> (CPU · 메인보드 · PCIe 슬롯)뿐이다. 아래 판정이 그만큼 좁혀진다.
>
> **그 카드가 두 호스트에서 어떤 링크로 물렸는지는 2026-08-26 에 구서버를
> 다시 켜서 확인했다** — 카드가 빠진 뒤에도 슬롯 능력은 남는다.
>
> | | 구서버 (R620) | 신서버 (H81M-K) |
> |---|---|---|
> | 슬롯 세대 | **PCIe 3.0** (`LnkCap 8GT/s`) | PCIe 2.0 (`LnkCap 5GT/s`) |
> | X550T 링크 | 8GT/s × x4 | 5GT/s × x4 |
> | 방향당 대역 | **약 32 Gbps** | 약 16 Gbps |
>
> **링크 대역이 절반으로 줄었다. 그래도 병목은 아니다** — 3노드 실사용이
> 방향당 ~4.6Gbps 라 16Gbps 도 3.5배 여유다. 이제 추정이 아니라 측정값이다.
> → [`hosts/server-xeon-e5-2630l-20260826.md`](#hosts-server-xeon-e5-2630l-20260826)

### 원인 — 스케줄러 호스트가 CPU 로 좁혀졌다

측정 중 서버 CPU 사용률이 **82.2%** (8 스레드 합)다.

```text
스케줄러        45.3%  ≈ 3.6 코어
기타(벤치+커널)  36.9%  ≈ 2.9 코어
────────────────────────────────
합계            82.2%
```

**벤치 클라이언트가 스케줄러와 같은 호스트에서 돈다.** 구서버에서는 같은
일이 24 스레드의 ~27% 였고, 신서버에서는 8 스레드의 82% 다.

손실 위치가 이를 뒷받침한다. 노드 쪽은 변화가 없고(NPU 추론 p50 28.35ms,
분배 33.3% 균등, 온도 53~57°C), `scheduler_queue` 0.00ms ·
`scheduler_route` 0.01ms 로 스케줄러 내부 큐도 비어 있다. 늘어난 시간은
전부 전송 구간(`network_to_node` / `network_to_client` 각 p50 24.2ms)에
있다 — 애플리케이션 큐가 아니라 **호스트의 CPU 경합**이다.

> **PCIe 강등은 원인이 아니다.** `LnkSta 5GT/s x4` 는 방향당 16Gbps 로,
> 실사용(~4.6Gbps)의 3배 여유가 있다. H81M-K 의 x16 슬롯이 PCIe 2.0 이라
> 생기는 하드웨어 한계이며 조치할 수 없고, 조치할 필요도 없다.

> **원본 데이터.** 위 3 run 의 bench JSON 은
> [`../results/baseline-20260826-althost/`](../results/baseline-20260826-althost)
> 에 있다. `-althost` 접미사 때문에 `count-runs.sh` 가 421 에 합산하지 않고
> 따로 센다.

### 기존 측정 결과에 미치는 영향 — 없다

**측정 421건은 전부 구서버에서 얻은 것이고, 그 값은 그대로 유효하다.**
숫자를 소급해 고치지 않는다. 신서버 값은 "다른 스케줄러 호스트에서의
재현치" 로 여기 따로 적는다.

다만 S3.9a 가 내린 판정 — **스케줄러는 자원 병목이 아니다** — 은
**조건부였음이 드러났다.** 그 판정은 24 스레드 호스트에서 성립했다.
8 스레드에서는 성립하지 않는다.

> 실험 대장 §4 의 원칙 그대로다. **배제는 조건부다.** 한 번 배제한 후보도
> 조건이 바뀌면 다시 열린다. 판정에는 "어떤 조건에서" 가 붙어야 한다.

이후 측정을 신서버에서 이어간다면 **구서버 값과 직접 비교하지 않는다.**
비교가 필요하면 신서버에서 기준선을 다시 깔고 그 위에서 상대 비교한다.

## 3.3 배포판 차이

```text
server  Rocky Linux 9.4   glibc 2.34   dnf   x86_64
nodes   Ubuntu 24.04      glibc 2.39   apt   aarch64
```

노드 바이너리는 aarch64 라 `king` 에서 네이티브 빌드해 세 노드에 배포한다
(세 보드 glibc 2.39 동일). 스케줄러는 x86_64 라 별개 빌드다.

---

# 4. 네트워크

## 4.1 현재 (2026-08-20 개편 완료)

```text
                server (10G) ─┐
                              ├── NS-S25G10G-N ──┬── king  (2.5G)
        개발PC (10G포트/1G NIC)┘                  ├── queen (2.5G)
                                                 ├── jack  (2.5G)
                                                 └── 인터넷 (2.5G, ipTIME)
```

- **worker 링크 2.5G, aggregation(server) 10G.** ADR-014 설계대로다.
- 아직 **관리망과 추론망이 분리되지 않았다.** 전부 `192.168.123.0/24` 단일
  대역이고, 보드 eth1 은 비어 있다. 측정 오염 방지를 위한 VLAN/서브넷 분리는
  M3 본측정 전에 결정한다.

## 4.2 대역폭 실측 (2026-08-20)

| 측정 | 값 | 도구 | 뜻 |
|---|---:|---|---|
| server enp1s0 협상 | 10000 Mb/s full | ethtool | 10G 링크 확정 |
| 단일 king→server | **2.34 Gbps** | iperf3 | 2.5G 실효 상한 |
| **3노드 동시 →server** | **각 1.70, 합 5.11 Gbps** | nc | **aggregation 병목 아님** |

3노드 동시 전송에서 세 스트림이 **균등하게(각 213 MB/s) 유지**됐다. 서버가
병목이면 합이 어딘가에서 깎였을 텐데 그러지 않았다. INT8 3노드 목표 RX
**4.60 Gbps** 를 여유 있게 수용한다(`RESULTS.md` §8.1).

> 개별 1.70 Gbps 가 링크 상한(2.34)보다 낮은 것은 nc/보드 CPU 단일코어
> 처리 한계지 스위치·서버 한계가 아니다. 실제 M3 는 gRPC 추론 트래픽이므로
> 이 값은 "인프라가 4.6 Gbps aggregate 를 받아내는가"의 검증으로만 쓴다 — 답은 예.

## 4.3 링크 속도 확인은 매번 한다

케이블 불량으로 협상이 낮아지는 사고가 반복됐다(옛 dealer 100Mb/s, 현
개발 PC 100Mb/s). 10GBASE-T 는 Cat6/6a 를 요구하고, Cat5e 면 조용히
2.5G/5G 로 떨어진다. 방치하면 NPU 가 아니라 케이블을 측정한다.

```bash
ssh npuforge-server 'ethtool enp1s0 | grep Speed'
for h in npuforge-k npuforge-q npuforge-j; do
  ssh "$h" 'printf "%s eth0=%s\n" "$(hostname)" "$(cat /sys/class/net/eth0/speed)"'
done
```

---

# 5. 구매 필요 목록

M3 를 막던 장비는 **전부 확보됐다.**

| 항목 | 상태 |
|---|---|
| ~~2.5G/10G 스위치~~ | ✅ NEXI NS-S25G10G-N (2.5G×4 + 10G×2) |
| ~~PCIe 슬롯 서버~~ | ✅ i7-4790 / 16GB / Rocky 9.4 (2026-08-26 교체) |
| ~~10G NIC~~ | ✅ Intel X550T `enp1s0` 10GBASE-T |
| ~~10G 케이블~~ | ✅ 10G full 협상 확인 (DAC 아닌 RJ45) |

남은 구매는 측정 품질용이며 M3 착수를 막지 않는다.

| 항목 | 수량 | 우선순위 | 근거 |
|---|---|---|---|
| 동일 모델 팬 | 3 | 중간 | S0-B 냉각 조건 비교용 |
| USB 전력 측정기 | 3 | 낮음 | FPS/Watt 산출 시 |
| Cat6/6a 케이블 (여유분) | 2~3 | 낮음 | 10G 링크 예비. 현 링크는 정상 |

냉각 장비(상시)는 목록에 없다. 팬리스를 유지하고 thermal throttling 을 측정
대상으로 삼는다(`adrs/013-fanless-thermal-as-measurement.md`).

---

# 6. 미해결 항목

| # | 항목 | 상태 | 차단 요소 |
|---|---|---|---|
| 1 | ~~IP static 고정~~ | ✅ 4대 전부 manual (2026-08-20) | — |
| 2 | **SSH 호스트 키 중복 (queen·jack)** | 미조치 | 없음. 명령은 `TODO.md` §1.2 |
| 3 | **INT8 모델을 queen·jack 에 배포** | 미조치 | 없음 |
| 4 | **스케줄러 빌드·배포 경로 확정** | 미정 | server 에 Rust 없음 (§3.2) |
| 5 | **server gRPC 포트 방화벽 개방** | 미조치 | 측정 전. firewalld public zone |
| 6 | 모델 변환 환경 재구축 | 보류 | dealer 소멸. 모델 이미 변환 완료라 급하지 않음 |
| 7 | 관리망/추론망 분리 | 미결정 | M3 본측정 전 |
| 8 | 실측 TX/RX 기록 (추론 트래픽) | 미측정 | 노드 소프트웨어 기동 후 |
| 9 | S0 열 특성 (30분 × 2조건) | 미실시 | 팬 3개 (S0-B 용) |

**호스트별 MAC / 고정 IP** (§1 에서 확인한 실제 MAC). 라우터 예약을 병행하면
이 표를 쓴다:

```text
king    22-94-FF-34-46-B1  →  192.168.123.3
jack    62-CE-3B-B6-E4-41  →  192.168.123.4
queen   7E-D8-D7-40-45-82  →  192.168.123.5
server  6C-B3-11-13-2F-38  →  192.168.123.9
```

해소된 항목(2026-08-20): 2.5G/10G 스위치, 10G 스케줄러 서버, 10G NIC·케이블,
aggregation 대역 실측, dealer RAM 3GB 제약. 이전 해소분: RKNN thread-safety
(컨텍스트 공유 금지), 모델 변환(FP16·INT8), Calibration(COCO 200장),
CPU governor(`performance`), 보드 배치 편차, OS 패치 레벨.

---

<a id="environment-matrix"></a>

# NPUForge Environment Matrix

- 문서명: `environment-matrix.md`
- 프로젝트명: NPUForge
- 대상 릴리스: NPUForge v0.1
- 작성일: 2026-08-06
- 상태: **확정.** S0 열 특성까지 닫혔다 (§9). 미해결 목록은 `experiments/README.md` §7
- 관련 문서:
  - `01-TECHSPEC.md` §2.5 재현성
  - `03-DEVELOPMENT-REQUIREMENTS.md` §2.1, §9

---

# 1. 문서 목적

본 문서는 NPUForge v0.1의 **버전 조합과 해시를 고정**하기 위한 단일 출처다.

여기에 기록하는 값은 소스코드, 설정 파일, git 이력 어디에서도 유도할 수 없다. RKNN Toolkit과 Runtime, 커널 드라이버의 조합은 외부에서 주어지는 사실이며, 조합이 바뀌면 이전 벤치마크 결과와 비교할 수 없게 된다.

`03-DEVELOPMENT-REQUIREMENTS.md` §9의 즉시 수행 항목 1번이 본 문서를 채우는 작업이다.

**이 표가 채워지기 전에 기록한 성능 수치는 공식 결과로 사용하지 않는다.**

---

# 2. 확정 상태

| 항목 | 상태 | 값 |
|---|---|---|
| 보드 및 SoC | **확정 (2026-08-06)** | RK3576, NPU 2코어 — §2.1 |
| RKNN 버전 조합 | **확정 (2026-08-07)** | Runtime 2.3.0 / Driver v0.9.8 / Toolkit2 2.3.0 — §3 |
| 커널 및 드라이버 | **확정 (2026-08-07)** | 6.1.141, 3노드 동일 — §4 |
| 기준 모델 해시 | **확정 (2026-08-12)** | FP16 `459602ea…` / INT8 `dba155d2…` — §6 |
| 데이터 세트 해시 | **확정 (2026-08-11)** | COCO val2017 200장 `224b8beb…` — §7 |
| Rust 툴체인 | **확정 (2026-08-12)** | 1.97.1 / edition 2024 / MSRV 1.85 — §8 |
| 노드 인벤토리 | **확정 (2026-08-07)** | serial·MAC — §8.1 |
| RKNN 동시성 계약 | **확정 (2026-08-11)** | 컨텍스트 공유 금지 — §3.1 정정 |
| CPU governor | **확정 (2026-08-12)** | `performance` 고정 + 영구화 — §4 |
| 열 특성 및 온도 임계치 | **확정** — degraded 80 / disable 90°C | S0 결과. §9.2 |
| OS 패치 레벨 동일성 | **확정 (2026-08-12)** | 3노드 모두 24.04.4 — §4 |
| SSH host key 고유성 | ⚠️ **미해결** | queen·jack 이 동일 — §8.1 |

확정 시점에 각 행의 상태를 `확정 (YYYY-MM-DD)`으로 변경하고 값을 채운다.

값을 변경할 경우 이전 값을 §11 변경 이력에 남긴다.

---

# 2.1 보드 및 SoC (확정)

2026-08-07 세 노드 실측으로 확정했다. 수집 방법은 `scripts/collect-node-info.sh`, 원본은 `benchmarks/node-info/{k,q,j}.txt`.

| 항목 | 값 | 확인 방법 |
|---|---|---|
| 보드 | FriendlyElec NanoPi R76S | `/proc/device-tree/model` |
| device-tree compatible | `friendlyelec,nanopi-r76s rockchip,rk3576` | `/proc/device-tree/compatible` |
| SoC | **Rockchip RK3576** | 위와 동일 |
| CPU 코어 수 | 8 | `nproc` |
| CPU little 클러스터 최대 | 2,016,000 kHz (2.016GHz) | `cpufreq/policy0` |
| CPU big 클러스터 최대 | 2,208,000 kHz (2.208GHz) | `cpufreq/policy4` |
| GPU | Mali-G52 MC3 | 제품 사양 |
| NPU | 6 TOPS | 제품 사양 |
| **NPU 코어 수** | **2 (Core0, Core1)** | `/sys/kernel/debug/rknpu/load` |
| NPU 주파수 | 300~950 MHz, 기본 950 MHz | `devfreq/27700000.npu` |
| NPU IOMMU | 활성화 | `dmesg` |
| RAM | **4GB LPDDR4X** (3,997,848 kB) | `/proc/meminfo` |
| eMMC | **64GB** (122,142,720 × 512B ≈ 62.5GB) | `/sys/block/mmcblk2/size` |
| rootfs 여유 | 50GB | `df -h /` |
| 네트워크 | **2.5GbE × 2** (`eth0`, `eth1`) — 드라이버 `r8125`, 별도 PCIe 버스 | `ethtool` |
| M.2 | SDIO (Wi-Fi 전용, NVMe 불가) | 제품 사양 |
| 냉각 | 팬리스 | 제품 사양 |

**NPU가 2코어다.** RK3588의 3코어와 다르므로, RK3588 기준의 `core_mask` 예제를 그대로 쓸 수 없다. `worker_count` 결정에 직접 영향을 준다(§3.1).

RAM 4GB는 워커 여러 개를 띄우기에 충분하다. 2GB 변형이었다면 제약이 있었을 것이다.

문서 초안은 RK3588/NanoPi R6C를 전제했으나 2026-08-06에 정정했다. 상세 영향은 `02-HARDWARE-SETUP.md` §16.1 참조.

## 2.2 열 센서

thermal zone이 6개다. NPU 전용 센서가 있어 §9의 열 특성 측정에 그대로 쓸 수 있다.

| zone | type | idle 온도 (2026-08-07) |
|---|---|---|
| 0 | `soc-thermal` | 44.4 ~ 46.2°C |
| 1 | `bigcore-thermal` | 45.3°C |
| 2 | `little-core-thermal` | 45.3°C |
| 3 | `ddr-thermal` | 44.4°C |
| 4 | **`npu-thermal`** | 42.5 ~ 45.3°C |
| 5 | `gpu-thermal` | 46.2°C |

노드 설정의 `temperature_path`는 `soc-thermal`(zone0)을 스케줄링 판단에 사용하고, `npu-thermal`(zone4)을 별도로 기록한다.

**idle 상태에서 이미 42~46°C다.** 팬리스 보드라 초안 문서의 "시작 온도 45°C 이하" 조건은 idle에서도 아슬아슬하다. §9.2에서 실측 후 재설정한다.

---

# 3. RKNN 스택

세 노드 전체가 동일해야 한다.

2026-08-07 실측. 세 노드의 `librknnrt.so` SHA-256이 모두 동일함을 확인했다.

| 항목 | 값 | 확인 방법 |
|---|---|---|
| 변환 타깃 플랫폼 | **`rk3576`** | 고정. `rk3588` 아님 |
| **RKNN Runtime 버전** | **2.3.0** (`c949ad889d@2024-11-07T11:35:33`) | `strings librknnrt.so` |
| **RKNPU Driver 버전** | **v0.9.8** | `/sys/kernel/debug/rknpu/version` |
| **NPU 코어 수** | **2** | `/sys/kernel/debug/rknpu/load` |
| `librknnrt.so` 경로 | `/usr/lib/librknnrt.so` | 3노드 동일 |
| `librknnrt.so` SHA-256 | `73993ed4b440460825f21611731564503cc1d5a0c123746477da6cd574f34885` | 3노드 동일 |
| 헤더 | `/usr/include/rknn_api.h` | 설치됨 |
| RKNN-Toolkit2 버전 | **2.3.0** | `dealer` 의 Docker 이미지 `npuforge-converter:2.3.1`. Runtime 과 일치 |

**Toolkit2는 Runtime 2.3.0에 맞춰야 한다.** Toolkit 버전이 Runtime보다 높으면 변환된 모델이 로딩되지 않을 수 있다. 개발 PC에 설치할 때 `rknn-toolkit2==2.3.0`을 우선 시도한다.

NPU가 2코어이므로 core_mask 전략은 RK3588(3코어) 예제와 다르다.

## 3.1 Thread-safety 검증 결과 (확정 2026-08-07)

노드 아키텍처가 이 결과에 직접 의존한다. 동시 호출이 불가능하면 모델당 전용 워커 스레드와 mutex가 필요하고, 가능하면 `worker_count`를 1보다 크게 설정할 수 있다.

**측정 조건.** `king`, FP16 모델(`yolov8n-fp16.rknn`), 스레드당 20회 반복.
도구는 `crates/npuforge-rknn/native/thread_safety_test.c`.

| 구성 | 스레드 | ok / err | 평균 지연 | 처리량 | 기준선 대비 |
|---|---:|---:|---:|---:|---:|
| 기준선 (전용 context) | 1 | 20 / **0** | 62.62 ms | 16.0 inf/s | 1.00× |
| **context 공유** | 2 | 40 / **0** | 57.28 ms | 34.8 inf/s | 2.18× |
| 전용 context (`CORE_AUTO`) | 2 | 40 / **0** | 58.77 ms | 33.2 inf/s | 2.08× |
| 전용 context + 코어 분리 | 2 | 40 / **0** | 62.58 ms | 31.9 inf/s | 1.99× |
| 전용 context | 4 | 80 / **0** | 76.22 ms | **52.3 inf/s** | **3.27×** |

### 결론

| 항목 | 결과 |
|---|---|
| 동일 context 동시 호출 | **가능** (오류 0건) |
| 서로 다른 context 동시 호출 | **가능** (오류 0건) |
| 모델당 전용 워커 스레드 직렬화 | **불필요** |
| 명시적 `core_mask` 분리 | **불필요** — 8스레드에서 +0.1% |
| 권장 `worker_count` | **8** (4 대비 +27%) |
| NPU 2코어의 실제 기여 | **1.51배** (단일코어 48.2 → 두코어 73.0 inf/s) |

**RKNN Runtime 2.3.0은 thread-safe다.** 어떤 조합에서도 오류가 발생하지 않았다.

> ### ⚠️ 2026-08-11 정정: "오류 0건"은 "결과가 옳다"가 아니다
>
> 위 표는 **API 반환 코드만 셌고 출력 내용을 대조하지 않았다.**
> 출력을 실제로 비교하니 결론이 달라진다.
>
> 추론 한 건은 세 번의 호출이다.
>
> ```text
> rknn_inputs_set  →  rknn_run  →  rknn_outputs_get
> ```
>
> 개별 호출이 thread-safe 여도 **이 시퀀스는 원자적이지 않다.**
> 두 스레드가 같은 컨텍스트에서 겹쳐 실행하면 서로의 결과를 가져간다.
>
> `native/shared_context_test.c` 로 확인했다. 스레드마다 다른 입력을 주고
> 단독 실행 결과와 대조했다(4스레드 × 50회, `king`).
>
> | 구성 | API 오류 | **결과 불일치** |
> |---|---:|---:|
> | 컨텍스트 공유 | 0 | **200 / 200 (100%)** |
> | 스레드별 전용 컨텍스트 | 0 | 0 / 200 (0%) |
>
> **공유 컨텍스트는 오류 없이 100% 틀린 답을 낸다.**
>
> 따라서 `supports_concurrent_infer = true` 는 유지하되, 그 근거는
> "런타임이 알아서 해준다"가 아니라 **"백엔드가 컨텍스트 풀로 직렬화한다"**
> 이다. `crates/npuforge-rknn/src/context.rs` 참조.
>
> 위 표의 처리량 수치 중 "context 공유" 행(2스레드 34.8 inf/s)은 **틀린
> 결과를 낸 상태의 속도**이므로 성능 비교에 쓰지 않는다.

### NPU가 2코어인데 4스레드가 더 빠른 이유

추론 한 건은 NPU 실행만이 아니라 **입력 설정 → NPU 실행 → 출력 취득**으로 구성되며, 앞뒤 구간은 CPU가 처리한다. 스레드가 코어 수보다 많으면 한 스레드가 CPU 구간에 있는 동안 다른 스레드가 NPU를 점유할 수 있어 파이프라이닝 효과가 생긴다.

**지연시간과 처리량이 상충한다.**

```text
1 스레드 : 62.6 ms,  16.0 inf/s   지연 최소
2 스레드 : 58.8 ms,  33.2 inf/s
4 스레드 : 76.2 ms,  52.3 inf/s   처리량 최대 (측정 범위 내)
```

**본 프로젝트는 처리량이 목표이므로 스레드를 늘리는 쪽이 맞다.** 단, deadline이 있는 요청에는 지연 증가가 불리하므로 `max_queue_depth`와 함께 조정한다.

### 명시적 코어 분리를 쓰지 않는 이유 (2026-08-10 재측정으로 확정)

대조군(`CORE_0_ONLY`)을 포함한 4가지 모드를 1/2/4/8 스레드에서 비교했다. 상세는 `docs/discuss.md` §4.

| 스레드 | `CORE_AUTO` | `ALTERNATE` | `CORE_0_1` | `CORE_0_ONLY` |
|---:|---:|---:|---:|---:|
| 1 | 16.7 | 16.7 | **18.2** | 16.5 |
| 4 | 52.4 | **57.1** | 48.5 | 38.5 |
| 8 | **72.9** | 73.0 | 64.5 | 48.2 |

**결론: `core_mask`를 설정하지 않는다.**

- `ALTERNATE`의 이득은 4스레드에서 +9%, **8스레드에서 +0.1%로 소멸**한다
- `CORE_0_1`은 8스레드에서 -11.5%로 오히려 손해다
- `CORE_AUTO`의 분배가 이미 균등하다 (8스레드에서 Core0 39% / Core1 37%)

8스레드로 가는 편이 코어를 수동 배정하는 것보다 낫고, `rknn_set_core_mask` 호출이 불필요해져 구현이 단순해진다.

**두 번째 코어는 실제로 기여한다.** 대조군 대비 48.2 → 73.0 inf/s로 **1.51배**다. 다만 2배가 아니라는 점이 코어 밖 공유 자원의 직렬화를 시사한다.

**예외: 단일 요청 지연이 중요한 경우.** 1스레드에서만 `CORE_0_1`이 +9%로 유리하다(`run` 29.7 → 23.7ms). deadline 요청 처리에 고려할 수 있다.

`rknn_api.h`의 `rknn_core_mask`는 코어 3개까지 정의하지만 RK3576은 2코어이므로 `CORE_2`는 사용할 수 없다.

### FP16 기준 성능과 그 함의

FP16 에서 노드당 **84.3 inf/s** (8스레드, governor `performance`), INT8 은 **157.2 inf/s** 다.
(초기 측정의 16~52 inf/s 는 1~4스레드 · `ondemand` 기준이다. `RESULTS.md` §2.2)

3노드 합산 시 네트워크 소요다. **입력과 출력을 함께 본다.**

```text
                              노드당        3노드
INT8 입력 (raw RGB 1.23MB)   1.545 Gbps    4.636 Gbps
INT8 출력 (want_float=1)     6.128 Gbps   18.383 Gbps   ← 10G 도 부족
INT8 출력 (want_float=0)     1.532 Gbps    4.596 Gbps
FP16 입력                    0.829 Gbps    2.486 Gbps
FP16 출력 (want_float=1)     3.286 Gbps    9.858 Gbps
```

**worker 링크(2.5G)가 아니라 aggregation 링크가 먼저 막힌다.**
스케줄러 쪽에 10G 가 필요하고, 그 위에 출력을 줄이는 조치가 따라와야 한다.
**출력 축소는 `want_float=0` 전환으로 해결했다 (2026-08-12, 기본값).**
남은 것은 10G aggregation 확보다. `02-HARDWARE-SETUP.md` §3.3.2.

> **이전 판은 폐기되었다.** "3노드 156 FPS, Raw RGB 1.5 Gbps, 2.5GbE 는
> S6 에서만 필요"라고 적혀 있었다. 그 계산은 (a) 4스레드·`ondemand` 기준
> 52 inf/s 를 썼고 (b) **출력 방향을 보지 않았다.** 실측 후 두 전제가
> 모두 바뀌었다.

### 미확정

- 8스레드에서도 처리량이 꺾이지 않았으므로 `MAX_THREADS`를 넘는 구간은 미탐색
- S0 열 특성 (30분 × 팬리스/냉각 2조건)
- `ondemand` vs `performance` 를 **동일한 300초 조건**에서 비교
  (§3.1 의 +7% 는 120초 값이라 CPU 강등 전 구간만 본 것이다)

> **해소됨 — `want_float=0` 의 INT8 처리량 영향** (2026-08-12).
> "§5 의 +5.4% 는 FP16 에서 잰 값이라 옮길 수 없다"로 남겨 두었던 항목이다.
> 8스레드 120초로 측정했다. **INT8 156.7 vs 133.6 inf/s (+17.3%),
> FP16 66.9 vs 57.8 inf/s (+15.7%).** §5 보다 큰 이유는 §5 가 1스레드 위주
> 조건이었기 때문이다 — 동시 스레드가 늘수록 출력 변환이 직렬화 구간을
> 더 오래 붙잡는다. `discuss.md` §12

### 남은 병목

NPU 40%, CPU 49%로 **둘 다 포화되지 않은 상태에서 `rknn_run` 대기만 늘어난다.** 코어 밖 공유 자원의 직렬화로 추정하며 후보는 다음과 같다.

- RKNN runtime 내부 lock
- kernel driver ioctl 직렬화
- IOMMU / buffer mapping 비용
- DDR / memory bandwidth
- output conversion / hidden copy

`perf record`, `strace -c`, off-CPU 분석이 필요하다. `docs/discuss.md` 참조.

---

# 4. 운영체제 및 커널

| 항목 | 값 | 상태 |
|---|---|---|
| 배포판 | Ubuntu 24.04 LTS (Noble Numbat) | 확정 |
| **패치 레벨** | **24.04.4 LTS** | ✅ 3노드 동일 (2026-08-12 확인. king 이 24.04.3 → 24.04.4 로 올라옴) |
| 커널 버전 | 6.1.141 (aarch64) | 3노드 동일 |
| glibc | 2.39 | 3노드 동일 |
| gcc | 13.3.0-6ubuntu2~24.04.1 | ✅ 3노드 동일 (2026-08-12 확인) |
| Python | 3.12.3 | 3노드 동일 |
| rustc | **`king` 만 1.97.1 설치** | 노드 바이너리 네이티브 빌드용. queen/jack 은 미설치 |
| **CPU Governor** | **`performance`** | ✅ 2026-08-12 고정. systemd 유닛으로 영구화 (처리량 +7%) |
| 미적용 패키지 업데이트 | K: 274 / Q: 280 / J: 280 | ⚠️ 측정 전 통일 권장. 커널은 hold 상태라 안전 |
| OS 이미지 파일명 | 미기록 | 보드 수령 시 기록하지 못했다. 재설치 시 반드시 남길 것 |
| OS 이미지 SHA-256 | 미기록 | 위와 같음 |
| io_uring 지원 여부 | **지원** | `/proc/kallsyms` 에 `io_uring_setup` 존재 확인 (2026-08-12) |

## 4.0 부트로더 펌웨어 ⚠️

**전력 관리(BL31/ATF)와 DDR 타이밍을 담당하는 계층이다.** 노드 간 버전이 다르면 고부하 안정성이 달라진다.

2026-08-10 실측:

| 구성요소 | `king` | `queen` | `jack` |
|---|---|---|---|
| DDR init | **v1.09** | v1.13 | v1.13 |
| SPL | **v1.07** | v1.09 | v1.09 |
| **BL31 (ATF)** | **v1.17** | **v1.24** | **v1.24** |
| BL32 | **v1.05** | v1.10 | v1.10 |
| U-Boot | **`44f011c4ba` 2025-07-17** | `c5c053fa55` 2026-07-10 | `c5c053fa55` 2026-07-10 |
| PMIC 초기화 | **`ON:0x20 OFF:0x2`** | `ON:0x40 OFF:0x0` | `ON:0x40 OFF:0x0` |

`queen`과 `jack`은 완전히 동일하고 **`king`만 약 1년 낡았다.**

### 이것이 `king`의 고부하 리셋 원인으로 보인다

`king`은 5스레드 이상에서 하드 리셋된다(`board-worklog.md` §2.17). BL31은 Rockchip에서 DVFS와 전압 조절을 담당하므로, 구버전의 전압 테이블이 고부하를 감당하지 못하면 정확히 이 증상이 나온다. DDR 펌웨어 차이도 메모리 트래픽이 큰 다중 스레드 조건에서 불안정성을 유발할 수 있다.

PMIC 초기화 레지스터가 다른 것도 펌웨어 차이의 결과다.

### 확인 방법

```bash
grep -oE 'androidboot\.fwver=[^ ]*' /proc/cmdline
```

`scripts/collect-node-info.sh`가 이 값을 수집한다(2026-08-10 추가).

### 조치

**`king`의 부트로더를 `queen`/`jack`과 동일한 버전으로 갱신해야 한다.** 갱신 후 5~8스레드 테스트로 재검증한다.

세 노드의 `fwver` 문자열이 완전히 일치해야 "동일한 3대" 전제가 성립한다. 이 항목이 §4.1의 필수 일치 목록에 포함되지 않았던 것은 문서의 누락이었다.

## 4.1 미해결 불일치

세 노드는 "동일 OS 이미지"여야 한다(`02-HARDWARE-SETUP.md` §5.1). 현재 다음이 어긋나 있다.

| 항목 | 내용 | 위험 |
|---|---|---|
| Ubuntu 패치 레벨 | K만 24.04.3 | 라이브러리 버전 차이가 노드별 성능 편차로 나타날 수 있음 |
| 보류 중인 업데이트 | 279~374개 | 위와 동일 |
| SSH 호스트 키 | **queen·jack 동일** (king 은 재설치로 고유) | ⚠️ **미해결.** 이미지 복제 시 재생성 누락. queen 과 jack 을 암호학적으로 구분할 수 없다 — IP 가 바뀌면 경고 없이 엉뚱한 보드에 붙는다 (§2.20 유형)|
| hostname | K·Q 모두 `NanoPi-R76S`, J는 `localhost.localdomain` | 로그와 대시보드에서 노드 구분 불가 |
| CPU Governor | **`performance`** | 2026-08-12 고정 + systemd 영구화. `ondemand` 대비 처리량 +7% |

**커널 업그레이드 주의.** 커널 6.1.141은 FriendlyElec BSP 커널이며 RKNPU 드라이버 v0.9.8이 여기에 묶여 있다. `apt upgrade`가 커널을 교체하면 NPU가 동작하지 않을 수 있다. 업데이트 시 커널 패키지를 hold 한다.

## 4.2 Scheduler 호스트

2026-08-07 실측. 구형 노트북을 Scheduler / Benchmark / 모델 변환 호스트로 사용한다.

| 항목 | 값 | 판정 |
|---|---|---|
| hostname | **`dealer`** | 2026-08-07 설정 (K/Q/J 카드 명칭과 통일) |
| 기종 | Samsung 370E5J / 380E5Q 계열 | |
| **배포판** | **Rocky Linux 9.7 (Blue Onyx)** | ⚠️ 보드는 Ubuntu 24.04 |
| 커널 | 5.14.0-611.13.1.el9_7.x86_64 | |
| glibc | **2.34** | ⚠️ 보드는 2.39 |
| 패키지 관리자 | **`dnf`** | ⚠️ 보드는 `apt` |
| CPU | Intel Core i7-4712MQ @2.30GHz (Haswell, 4C/8T) | 부하 생성에 충분 |
| RAM | **3.5GB** (가용 약 1.8GB) | ⚠️ 가장 큰 제약 |
| Swap | 3.9GB | 변환 시 메모리 부족 완화 |
| 디스크 여유 | **60GB** (`/`, 70GB 중 16% 사용) | Docker 이미지에 충분 |
| 아키텍처 | x86_64 | RKNN-Toolkit2 구동 가능 |
| NIC | Realtek RTL8111/8168 (`r8169`), **1GbE 상한** | 2.5G 미지원 |
| 링크 속도 | **1000 Mb/s** | 정상 |
| 관리망 IP | `192.168.123.14/24` (`enp3s0`) | 보드와 동일 대역 |
| MAC | `<redacted-mac>` | |
| USB 3.0 | Bus 004 (`xhci_hcd`, 5000M, 4포트) | 2.5G 어댑터 확장 가능 |
| Thunderbolt | 없음 | |
| Docker | **29.2.1**, storage `overlayfs` | 모델 변환 환경 |
| Python (호스트) | 3.9.23 | 변환은 컨테이너 안에서 하므로 무관 |
| 계정 | `yoo2` (`wheel`, `docker` 소속) | 2026-08-07 그룹 추가 |
| root SSH | 차단됨 | 승격은 `su` 사용 |

### ⚠️ 호스트와 노드의 배포판이 다르다

| | Scheduler 호스트 | 노드 3대 |
|---|---|---|
| 배포판 | Rocky Linux 9.7 | Ubuntu 24.04 |
| glibc | 2.34 | 2.39 |
| 패키지 관리자 | `dnf` | `apt` |

**영향 1 — 스크립트.** `scripts/fix-node-consistency.sh`는 `apt` 전용이다. 노드 대상이므로 문제 없으나, 호스트에도 적용하는 스크립트를 쓸 때는 패키지 관리자를 분기해야 한다.

**영향 2 — 바이너리 배포.** 다행히 방향이 안전한 쪽이다.

```text
빌드 호스트 glibc 2.34  →  실행 대상 glibc 2.39   (구 → 신, 호환됨)
```

낮은 glibc로 빌드한 바이너리는 높은 glibc에서 동작한다. 반대는 성립하지 않는다.

다만 **현재 `dealer` 에는 Rust 가 설치되어 있지 않다.** 실제 빌드는 `king` 에서
네이티브로 하고 결과물을 세 노드에 배포한다. 세 보드의 glibc 가 2.39 로 같으므로
이 방향에는 문제가 없다.

단, `npuforge-scheduler`(x86_64) 바이너리는 `dealer`에서 직접 빌드하거나 glibc 2.34 이하 환경에서 빌드해야 한다.

**영향 3 — 재현성 기록.** 오픈소스 공개 시 "Ubuntu에서 개발했다"고 쓸 수 없다. 호스트와 노드의 배포판을 각각 명시한다.

### 100Mb/s 링크 문제 (해결됨)

최초 측정 시 `Speed: 100Mb/s`로 협상되어 있었다. 포트는 1000baseT를 지원하므로 케이블 문제였고, 교체 후 1000Mb/s로 정상화되었다.

이 상태를 방치했다면 JPEG 100KB 기준 약 125 FPS에서 링크가 포화되어, NPU가 아니라 케이블을 측정할 뻔했다. **매 실험 전 링크 속도를 확인하는 절차를 둔다.**

```bash
ethtool enp3s0 | grep Speed
```

### RAM 제약에 대한 대응

Scheduler 호스트(3.5GB)가 노드(4GB)보다 메모리가 적다. `npuforge-scheduler` + `npuforge-bench` + Prometheus + Dashboard를 동시에 최대 부하로 돌릴 수 없다.

**대응 방침:**

| 상황 | 구성 |
|---|---|
| 공식 벤치마크 | Scheduler + bench 만 실행. Prometheus·Dashboard 중지. 원본은 JSONL로 기록 |
| 발표 데모 | Scheduler + Dashboard. 부하는 낮게 |
| 개발 | 제한 없음 |

원본 데이터가 결과물이고 대시보드는 데모용이므로 둘을 동시에 최대로 돌릴 필요가 없다. `npuforge-bench`는 실행 중 호스트 CPU·메모리 사용률을 함께 기록해, 클라이언트가 병목이었는지 사후에 판별할 수 있게 한다.

### 2.5GbE 확장 판단 보류

현재 1GbE다. 2.5G 필요 여부는 **S0/S1에서 노드당 실제 FPS를 측정한 뒤** 결정한다.

```text
노드당 40 FPS 가정 → 3노드 120 FPS × 100KB ≈ 96 Mbps   → 1GbE로 충분
Raw RGB 입력 (S6)  → 120 FPS × 1.23MB ≈ 1.2 Gbps        → 1GbE 초과
```

~~USB 3.0 2.5GbE 어댑터~~ 로는 부족하다. 3노드 aggregation 에 **10G** 가
필요하고(위 표), USB 어댑터는 2.5G 가 상한이다. `dealer` 는 PCIe 슬롯이
없으므로 **스케줄러 호스트를 서버로 교체**해야 한다.
`02-HARDWARE-SETUP.md` §3.3.2.

USB NIC 사용 시 결과에 그 사실을 명시한다.

---

# 5. Rust 및 빌드 툴체인

| 항목 | 값 |
|---|---|
| Rust 버전 | **1.97.1** (`king` 에만 설치) |
| Edition | **2024** |
| MSRV | **1.85** |
| 크로스 링커 | `aarch64-linux-gnu-gcc` |
| 크로스 툴체인 버전 | 미사용 — `king` 네이티브 빌드 (gcc 13.3.0) |
| protoc 버전 | **libprotoc 3.21.12** (`king`) |

빌드 산출물 해시는 릴리스마다 달라지므로 본 문서가 아니라 릴리스 노트에 기록한다.

---

# 6. 기준 모델

## 6.1 ONNX 원본

| 항목 | 값 |
|---|---|
| 모델 | YOLOv8n (RKNN 최적화판) |
| 출처 | `airockchip/rknn_model_zoo` → `examples/yolov8` |
| 원본 프로젝트 | `airockchip/ultralytics_yolov8` |
| **라이선스** | **AGPL-3.0** (`MODEL_LICENSES.md` §2 참조) |
| 파일 | `yolov8n.onnx` |
| 크기 | 12,650,184 bytes |
| **SHA-256** | `0c8716701f471067932b797eeb67c8e5db47c693c2557c881d7679ec12e21bc5` |
| export 도구 | PyTorch 2.0 |
| 입력 해상도 | 640 × 640 RGB |

### ⚠️ 표준 Ultralytics export를 쓰지 않는 이유

공식 원본은 DFL·NMS 후처리가 ONNX 그래프에 포함되어 있다. 이 연산들은 NPU에 매핑되지 않아 CPU fallback이 대량 발생한다. **그 상태로 측정하면 NPU가 아니라 CPU를 측정하게 된다.**

Rockchip 최적화판은 decode 이전의 raw 텐서를 출력하고 후처리를 CPU에서 별도 수행한다.

```text
공식 원본 : 출력 1개 (decode·NMS 포함)
최적화판  : 출력 3그룹
            [1,64,80,80]  박스 좌표
            [1,80,80,80]  80개 클래스별 confidence
            [1,1,80,80]   confidence 합
```

RK3576은 공식 지원 목록에 포함된다(RK3562/3566/3568/**3576**/3588/RV1126B/RV1109/RV1126/RK1808/RK3399PRO).

## 6.2 변환된 RKNN

### FP16 (thread-safety 검증용, 2026-08-07)

Calibration 데이터가 확정되지 않아 먼저 FP16으로 변환했다. 양자화 없이도 동시성 검증에는 지장이 없다.

| 항목 | 값 |
|---|---|
| 파일 | `yolov8n-fp16.rknn` |
| 크기 | 9,645,065 bytes |
| **SHA-256** | `459602ea70479c1ce4fdd7419aa81e10e2f795fe6fe87444f3607f25b7054c0f` |
| 양자화 | 없음 (FP16) |
| target_platform | `rk3576` |
| 3노드 배포 및 해시 일치 | 확인 |

### INT8 (기준 모델) — **생성 및 검증 완료 (2026-08-12)**

| 항목 | 값 |
|---|---|
| 양자화 방식 | INT8 |
| Calibration 이미지 수 | **200** (COCO val2017, seed 20261128) |
| Calibration manifest SHA-256 | `d8d189fc386897dd…` ⚠️ 절대경로 기반. 이식 가능한 값은 `224b8bebd5f3a4ce…` |
| RKNN SHA-256 | INT8 `dba155d2088df622…` / FP16 `459602ea70479c1c…` |
| CPU fallback 연산 목록 | 미조사. 변환 로그의 `not support` 경고로 확인할 수 있다 |

Calibration 데이터 세트 확정 후 생성한다(§7).

## 6.3 변환 환경

| 항목 | 값 |
|---|---|
| 이미지 | `npuforge-converter:2.3.0` (9.61GB) |
| 베이스 | `ubuntu:22.04` |
| Python | 3.10.12 |
| **rknn-toolkit2** | **2.3.0** (보드 Runtime 2.3.0과 일치) |
| **onnx** | **1.14.1 (고정 필수)** |
| torch | 2.4.0 (CPU 전용으로 전환 예정) |
| numpy | 1.26.4 |
| protobuf | 4.25.4 |

### ⚠️ onnx 버전을 반드시 고정해야 한다

`rknn-toolkit2`의 의존성 명세가 onnx 버전을 제한하지 않아 최신 버전(실측 시 1.22.0)이 설치되었고, 변환이 실패했다.

```text
AttributeError: module 'onnx' has no attribute 'mapping'
```

`onnx.mapping`은 onnx 1.16에서 제거되었는데 rknn-toolkit2 2.3.0이 이를 사용한다. **1.14.1로 고정하면 정상 동작한다**(2026-08-07 실측).

Dockerfile에 고정 및 검증 단계를 넣었다.

```dockerfile
RUN python3 -m pip install "onnx==1.14.1" \
    && python3 -c "import onnx; assert hasattr(onnx, 'mapping')"
```

CPU fallback 목록은 확장 효율 분석에 직접 쓰이므로 반드시 기록한다. NPU가 아닌 CPU에서 실행되는 연산이 많을수록 노드 간 편차와 온도 영향이 커진다.

---

# 7. 벤치마크 데이터 세트

| 항목 | 값 |
|---|---|
| 데이터 세트 이름 | **COCO val2017 부분집합** |
| 출처 | `http://images.cocodataset.org/val2017` |
| 재배포 조건 | **재배포 금지.** 개별 이미지는 Flickr 출처로 라이선스가 제각각이다. COCO 는 어노테이션에만 CC-BY 4.0 을 건다. 저장소에는 manifest 만 넣는다 |
| 이미지 수 | **200** |
| 선택 방식 | 정렬 후 고정 시드(20261128) 추출. `tools/model-converter/fetch_calibration.py` |
| 입력 포맷 | 640×640×3 uint8 NHWC RGB (전처리는 `make_reference.py` 가 수행) |
| Manifest SHA-256 | `224b8bebd5f3a4ce906388d2fab1371ce0b84bf92e352226fb270f2fe3560fec` |

현재는 calibration 과 정확도 검증에 같은 데이터를 쓴다. **벤치마크 부하는
`npuforge-bench` 가 결정적으로 생성하는 합성 입력**을 쓴다(시드 고정).
실제 이미지로 부하를 걸 필요가 생기면 여기에 별도 세트를 정의한다.

---

# 8. 노드 인벤토리

`03-DEVELOPMENT-REQUIREMENTS.md` §4.4에서 요구하는 관리 정보다.

보드에는 물리적으로 **K / Q / J** 라벨이 붙어 있다. Node ID와 hostname을 여기에 맞춘다.

| 항목 | K | Q | J |
|---|---|---|---|
| Node ID | `king` | `queen` | `jack` |
| hostname | `king` | `queen` | `jack` |
| 변경 전 hostname | `NanoPi-R76S` | `NanoPi-R76S` | `localhost.localdomain` |
| 관리망 IP (현재) | `192.168.123.12` | `192.168.123.16` | `192.168.123.33` |
| 관리망 포트 | `eth1` | `eth1` | `eth1` |
| 관리망 MAC | `<redacted-mac>` | (수집됨) | (수집됨) |
| 관리망 링크 | 1000 Mbps (1G 허브에 협상됨. 포트는 2.5G 지원) | 1000 Mbps | 1000 Mbps |
| 추론망 IP (예정) | `10.20.0.21` | `10.20.0.22` | `10.20.0.23` |
| 추론망 포트 | `eth0` (2.5G, 미연결) | `eth0` (2.5G, 미연결) | `eth0` (2.5G, 미연결) |
| 추론망 MAC | `<redacted-mac>` | (수집됨) | (수집됨) |
| Serial | `aaf2afcf6887055` | `64901d66a690b679` | `5b1e0475e81e50e4` |
| RAM | 4GB | 4GB | 4GB |
| eMMC | 64GB | 64GB | 64GB |
| 전원 어댑터 | 5V 4A | 5V 4A | 5V 4A |

MAC 전체 목록은 `benchmarks/node-info/{k,q,j}.txt` 참조.

**`eth0`이 세 노드 모두 `down` 상태다.** 두 번째 2.5G 포트가 비어 있으므로 추론망 전용으로 그대로 쓸 수 있다. `eth1`은 현재 1G 허브에 연결되어 관리망 역할을 하고 있다.

세 노드 모두 팬리스 출고 상태를 유지한다(`02-HARDWARE-SETUP.md` §9.1).

**어느 물리 포트를 추론망에 쓸지는 세 노드에서 동일해야 한다.** 포트가 섞이면 노드별 네트워크 특성이 달라져 비교가 무의미해진다.

Scheduler 호스트는 `npuforge-scheduler` / `10.20.0.10`.

## 8.1 일치 검증

```bash
./scripts/check-versions.sh
./scripts/check-model-hashes.sh
```

세 노드의 출력이 모두 동일해야 하며, 공식 벤치마크 실행 전에 매번 확인한다.

---

# 8.2 전원 (확정 2026-08-10)

## 입력 방식

**5V 입력이다.** 커널 디바이스 트리의 `vcc12v_dcin: 12000 mV`는 fixed-regulator 선언일 뿐 실제 입력 전압이 아니다. Rockchip 디바이스 트리가 보드 간 복사되며 남은 항목이다.

**반드시 센서 실측값을 확인한다.**

```bash
cat /sys/class/power_supply/simple-vin/voltage_now   # 마이크로볼트
```

| 항목 | 값 |
|---|---|
| 입력 전압 | **5V** |
| 센서 경로 | `/sys/class/power_supply/simple-vin/` |
| 어댑터 정격 | **5V 4A (20W)** × 3, 노드별 독립 |

## 어댑터 교체 전후

| 상태 | 유휴 전압 | 고부하 안정성 |
|---|---|---|
| 교체 전 | **4.983 V** (5V 미만) | 3~5 스레드에서 하드 리셋 |
| 교체 후 (5V 4A) | **5.27 ~ 5.31 V** | 8 스레드 완주 |

교체 전 어댑터는 **무부하에서도 5V를 유지하지 못했다.** 고부하에서 더 떨어져 브라운아웃 임계를 넘은 것이 재부팅의 원인이었다.

## 지속 부하 중 전압 (3대 동시, 8스레드)

| 노드 | 최소 전압 |
|---|---|
| `king` | 5.061 V |
| `queen` | 5.157 V |
| `jack` | 5.124 V |

3대를 동시에 최대 부하로 돌려도 5V 아래로 내려가지 않는다. **전원 여유는 확보되었다.**

## 벤치마크 시 기록 의무

전압을 온도와 함께 기록한다. 전압 강하는 성능 저하와 리셋의 선행 지표다.

```text
psu_simple-vin_voltage_v    측정 시작 / 최소 / 평균 / 종료
```

`scripts/collect-node-info.sh`가 수집하며, 벤치마크 실행 중에는 1초 간격으로 샘플링한다.

---

# 9. 열 특성 (S0 결과)

팬리스 구성이므로 이 값들이 다른 모든 실험의 전제가 된다.
**S0 로 측정을 마쳤다 (2026-08-21).** 원본은
[`experiments/S0_SUSTAINED_LOAD.md`](#experiments-s0-sustained-load).

> ⚠️ **원래 이 절은 노드별 Peak/Sustained FPS 표였다.** 계획 당시에는
> 보드를 따로 재는 그림이었는데, S0 는 **클러스터 단위 30분 지속 부하**로
> 설계됐다. 노드별로는 온도·클럭·지연이 나오고 FPS 는 클러스터 합계로
> 나온다. **측정하지 않은 칸을 채우지 않는다** — 표를 실제 산출 구조에
> 맞춰 다시 썼다.

### 클러스터 (3노드 합계, 30분)

| 항목 | B: 능동 냉각 | A: 팬리스 |
|---|---:|---:|
| peak | 387.7 inf/s | 389.4 inf/s |
| **steady (뒤 1/3)** | **380.3 ± 2.2** | **345.4 ± 3.8** |
| **성능 저하율** | **1.9%** | **11.3%** |
| soc 최대 | 58.2 ~ 61.0°C | **85.9 ~ 86.8°C** |
| npu 최대 | 59.2 ~ 61.0°C | **86.8 ~ 87.8°C** |
| NPU 최저 클럭 | 950 MHz | 950 MHz (**강등 없음**) |
| 노드 제외 | 0 | 0 |
| 오류율 | 0 | 0 |

### 노드별 (팬리스 조건에서 갈린다)

| 항목 | KING | QUEEN | JACK |
|---|---:|---:|---:|
| 유휴 시작 온도 | 38.8 ~ 41.6°C (두 조건 모두, 노드 간 차이 작음) |||
| npu 최대 (팬리스) | 86.8°C | 85.9°C | 86.8°C |
| **CPU 최저 클럭 (팬리스)** | **816 MHz (−63%)** | 1416 MHz (−36%) | 1200 MHz (−46%) |
| p50 지연 (팬리스) | **156.9 ms** | 66.0 ms | 64.7 ms |
| 요청 분배 | 33.3% | 33.3% | 33.3% |

**강등된 것은 NPU 가 아니라 CPU 다.** 그리고 보드마다 정도가 다르다 —
king 이 다른 둘보다 **2.4배 느려졌는데 round-robin 은 여전히 1/3 을
보낸다.** 이 관찰이 S0-C 정책 A/B 로 이어졌다.

능동 냉각에서는 **클럭 강등 0회**다. 그래서 S2~S3.9a 의 60초 결과가
지속 운전에 그대로 적용된다.

> 측정하지 않은 것: **throttling 시작 시점(초)** 과 **Idle 복귀 시간** 은
> 재지 않았다. S0 는 정상 상태 도달 후의 지속 성능을 물었지 과도 구간을
> 묻지 않았다.

## 9.0 예비 측정 (S0 아님, 2026-08-11)

S0 정식 측정(30분)이 아니라 **노드 간 열 편차 확인용 15분 측정**이다.
S0 표를 채우지 않는다. `board-worklog.md` §2.19 참조.

조건: 8스레드 고정, 900초, 세 보드 동시 시작, 팬리스, 선풍기 없음.
**CPU governor 는 당시 `ondemand`.** 2026-08-12 부터 `performance` 이므로
처리량 수치는 약 7% 낮게 나온 값이다(discuss.md §11). 온도는 유휴 기준
1°C 이내 차이라 열 결론에는 영향이 없다.
도구: `scripts/run-thermal-comparison.sh` + `sustained_load_test`.
평탄역: 부하 후 300초~종료 (보드당 약 557샘플).

| 항목 | `king` | `queen` | `jack` |
|---|---|---|---|
| Idle NPU | 37.0°C | 35.2°C | 36.1°C |
| 평탄역 NPU 평균 | 73.0°C | 67.5°C | 72.6°C |
| **최고 NPU** | **75.8°C** | 70.2°C | 74.8°C |
| 평탄역 SoC 평균 | 71.2°C | 65.8°C | 71.6°C |
| 입력 전압 최저 | 5.070 V | 5.090 V | 5.046 V |
| NPU 클럭 | 950 MHz 고정 | 950 MHz 고정 | 950 MHz 고정 |
| 지속 처리량 | 80.5 inf/s | 77.7 inf/s | 77.8 inf/s |
| 총 추론 (900초) | 72,481 | 69,928 | 70,049 |
| 평균 지연 | 99.3 ms | 102.9 ms | 102.8 ms |
| 오류 | 0 | 0 | 0 |

**노드 간 최대 편차 5.6°C. NPU throttling 없음** — 928 샘플 전부 950 MHz 로,
NPU 클럭이 한 번도 떨어지지 않았다.

여기서 확정할 수 있는 것:

- 현재 임계치(`degraded 80` / `disable 90`)는 이 부하에서 **작동하지 않는다**.
  최고 75.8°C 로 80°C 에 닿지 않으므로 노드가 임의로 제외될 일이 없다.
  다만 S0(30분)에서는 더 올라갈 수 있으므로 §9.2 는 여전히 S0 결과로 정한다.
- 팬리스로 8스레드 지속 부하가 오류 없이 완주한다
- ⚠️ **그러나 CPU 는 열로 강등된다.** 위 판정은 NPU 클럭만 봤다.
  같은 로그의 CPU 클럭을 보면 A72 2208 → 816 MHz, A53 2016 → 600 MHz 다.
  처리량이 300초에 -27% 떨어진다. `discuss.md` §12
- 세 보드의 처리량 편차는 3.5% 이내다. 확장 효율 측정의 전제가 성립한다.

## 9.1 측정 조건

| 항목 | 값 |
|---|---|
| 주변 온도 | 미측정 | 온도계 없음. 유휴 NPU 35~40°C 로 간접 추정 |
| 측정 일시 | 예비 측정 2026-08-11 10:48 KST | 정식 S0 은 미실시 |
| 보드 간 간격 | 미기록 | 사진 또는 실측으로 남길 것 |
| 배치 방향 | 미기록 | 위와 같음 |
| 케이스 유무 | **없음** (기판 노출) | |

## 9.2 확정 온도 임계치

S0 결과를 근거로 정한다. 정상 상태 온도보다 충분히 높아야 벤치마크 중 노드가 임의로 제외되지 않는다.

| 설정 키 | 값 | 근거 |
|---|---:|---|
| `degraded_temperature_c` | **80.0** | 팬리스 지속 부하에서 soc 85.9~86.8°C 까지 오른다. 그 아래에 두어 열화를 신호로 잡는다 |
| `disable_temperature_c` | **90.0** | S0 전 구간에서 **노드 제외 0건** — 팬리스도 여기 닿지 않았다 |
| 반복 사이 cooldown (초) | 하네스가 유휴 온도로 게이트 | `preflight-check.sh` 가 유휴 온도 상한(50°C)을 검사한다. 고정 시간이 아니라 상태로 판단한다 |

> **`disable` 90°C 는 아직 발동한 적이 없다.** 팬리스 31분에서도 87.8°C
> 가 최고였다. 즉 이 값은 **검증된 것이 아니라 도달하지 않은 것**이다.
> 노드 제외 동작 자체는 미검증으로 남아 있다 —
> `experiments/README.md` §7.

확정 후 `configs/scheduler.example.toml`과 이 표를 함께 갱신한다.

---

# 10. Scheduler 호스트

호스트명 `dealer`. 모델 변환(Docker)과 스케줄러를 겸한다.

| 항목 | 값 | 비고 |
|---|---|---|
| 배포판 | **Rocky Linux 9.7** (Blue Onyx) | |
| 커널 | **5.14.0-611.13.1.el9_7.x86_64** | |
| CPU | **Intel i7-4712MQ @2.30GHz, 8코어** | 2014년 노트북 CPU |
| RAM | **3GB** | ⚠️ §10.1 참조 |
| NIC | **`enp3s0` 1000Mb/s** | ⚠️ 2.5GbE 미확보 |
| Rust | **미설치** | 노드 바이너리는 `king` 에서 빌드 |

Scheduler 호스트가 1GbE인 상태에서 측정한 값은 공식 수치로 사용하지 않는다.
세 노드 트래픽이 합류하는 지점이라 여기가 먼저 포화되기 때문이다.
`02-HARDWARE-SETUP.md` §3.3.2 참조.

## 10.1 확인이 필요한 제약

**RAM 3GB.** 스케줄러는 요청 페이로드를 메모리에 들고 노드로 중계한다.
640×640×3 = 1.17 MiB/요청이므로, 동시 처리 수가 커지면 무시할 수 없다.

```text
3노드 × worker_count 8 = 24 in-flight
+ 스케줄러 큐 + gRPC 버퍼(요청·응답 양쪽)
→ 1.17 MiB × 수십 = 수백 MB
```

산술적으로는 여유가 있지만 **실제로 측정해 확인해야 한다.** 부족하면
페이로드를 스트리밍하거나 참조 전달로 바꿔야 하는데, 그것은 설계 변경이다.
S2 측정 전에 스케줄러 RSS 를 관찰한다.

**NIC 1GbE + PCIe 슬롯 없음.** INT8 기준 노드 하나가 **1.545 Gbps** 를
요구한다. 지금 상태로는 **노드 한 대분도 받지 못한다.**

3노드 입력만 4.636 Gbps 이고, 출력은 입력의 3.96배라 `want_float=1` 에서
RX 가 **18.38 Gbps** 까지 간다. **2.5G 로는 어림없고 10G 가 필요하다.**

`dealer` 는 노트북이라 PCIe 10G 카드를 꽂을 수 없다. **별도 서버가 필요하다.**
`02-HARDWARE-SETUP.md` §3.3.2, `RESULTS.md` §8.1 참조.

---

## 10.2 현재 Scheduler 호스트 (2026-08-26~)

§10 · §10.1 은 `dealer`(노트북) 시절 기록이다. 그 제약은 서버 교체로
해소됐고, 서버는 다시 한 번 교체됐다. **재현에 쓸 값은 이 표다.**

| 항목 | 값 | 비고 |
|---|---|---|
| hostname | `server` | SSH 별칭 `npuforge-server` |
| 메인보드 | ASUS H81M-K (H81) | 여분 데스크톱 전용 |
| CPU | **Intel Core i7-4790, 4C/8T, 3.6~4.0GHz** | ⚠️ 구서버는 Xeon E5-2630L ×2 (24T) |
| RAM | **16GB DDR3-1600 non-ECC** | |
| 디스크 | ST2000VN004 2TB, root LVM 70GB | |
| 배포판 | **Rocky Linux 9.4** (Blue Onyx) | |
| 커널 | **5.14.0-427.13.1.el9_4.x86_64** | 구서버와 동일 |
| glibc | **2.34** | 동결 바이너리 실행 요건 충족 |
| NIC | **Intel X550T `enp1s0`**, 드라이버 `ixgbe` | 10GBASE-T, 10000Mb/s full 실측. **구서버에서 옮겨 온 같은 카드** (`enp4s0` 이었다) |
| NIC 슬롯 | PCIe **2.0 x4** (`LnkSta 5GT/s x4`) | H81 x16 슬롯 한계. 방향당 16Gbps — 병목 아님 |
| 시각 동기 | chronyd active, synchronized | 2026-08-26 활성화 |

### 이 호스트에서의 기준선

```text
처리량   ~360 inf/s   (3 run: 360.5 / 362.5 / 357.2)
왕복 p50  ~93 ms
오류율    0
노드 편차 ~1.07x
측정 중 서버 CPU 82.2% (8 스레드 합) — 스케줄러 45.3% / 벤치·커널 36.9%
```

**구서버 기준선은 ~391 inf/s 였다.** 차이(−7.5%)의 원인은 스케줄러 호스트의
CPU 여유다. 근거와 판정은 `infrastructure.md` §3.2.1 에 있고, **원본 bench
JSON 은 `results/baseline-20260826-althost/`** 에 있다.

> **측정 421건은 구서버에서 얻은 값이며 그대로 유효하다.** 소급해 고치지
> 않는다. 신서버에서 측정을 이어간다면 **구서버 값과 직접 비교하지 않고**
> 여기서 기준선을 다시 깔고 상대 비교한다. 이 문서 맨 아래 문장 그대로다 —
> 조합을 바꾸면 이전 조합과 직접 비교할 수 없다.

---

# 11. 변경 이력

| 날짜 | 항목 | 이전 값 | 변경 값 | 사유 |
|---|---|---|---|---|
| 2026-08-06 | — | — | — | 문서 생성 |
| 2026-08-06 | SoC | RK3588 | RK3576 | 실제 보유 장비가 NanoPi R76S로 확인됨 |
| 2026-08-06 | 보드 | NanoPi R6C | NanoPi R76S | 동일 |
| 2026-08-06 | 냉각 | 팬 3개 추가 | 팬리스 유지 | throttling을 측정 대상으로 전환 |
| 2026-08-06 | 네트워크 | 2.5G + 1G | 2.5G × 2 | 관리망 분리가 기본 구성이 됨 |
| 2026-08-07 | 보드/SoC/NPU/RAM/eMMC | 미확정 | 실측 확정 | 3노드 SSH 접속 후 `collect-node-info.sh` 수집 |
| 2026-08-07 | 네트워크 포트 | 미확정 | 2.5G × 2 (`r8125`, 별도 PCIe) | `ethtool` 실측 |
| 2026-08-07 | hostname | `NanoPi-R76S` ×2, `localhost.localdomain` | `king` / `queen` / `jack` | 노드 구분 불가 문제 해소 |
| 2026-08-07 | NPU 코어 수 | 미확정 | **2** | RK3588(3코어)과 다름 |
| 2026-08-07 | RKNN Runtime | 미확정 | **2.3.0** | 3노드 SHA-256 동일 |
| 2026-08-07 | RKNPU Driver | 미확정 | **v0.9.8** | 커널 6.1.141 BSP에 포함 |
| 2026-08-07 | Node ID | `r76s-01/02/03` | `king` / `queen` / `jack` | 보드 물리 라벨에 맞춤 |
| 2026-08-26 | Scheduler 호스트 CPU | Xeon E5-2630L ×2 (24T) | **Core i7-4790 (8T)** | 구서버 물리 교체. 여분 데스크톱으로 이전 |
| 2026-08-26 | Scheduler 호스트 NIC 이름 | `enp4s0` | `enp1s0` | **카드는 같다** — Intel X550T 한 장을 구서버에서 빼 신서버에 옮겨 꽂았다. 슬롯이 달라 이름만 바뀐다 |
| 2026-08-26 | 기준선 처리량 | ~391 inf/s | **~360 inf/s** | 호스트 CPU 여유 감소(24T→8T). §10.2 · `infrastructure.md` §3.2.1 |
| 2026-08-26 | `h2` (HTTP/2 구현) | **0.4.15** | **0.4.19** | RUSTSEC-2026-0258 (빈 DATA 프레임 무제한 큐잉, Low). ⚠️ **측정 421건은 0.4.15 로 수행됐다** — 아래 참조 |

> ## ⚠️ `h2` 는 이 프로젝트에서 부수적 의존성이 아니다
>
> **우리가 측정한 전송 계층 그 자체다.** S3.6 은 H2 flow control(window
> 크기)을 A/B 했고, S3.7 은 노드당 커넥션 수를 다뤘다. gRPC 위의 처리량
> 계보 전체가 이 크레이트 위에서 나왔다.
>
> 측정 421건은 **`h2` 0.4.15** 로 수행됐다. 2026-08-26 에 보안 권고
> (RUSTSEC-2026-0258)로 `Cargo.lock` 을 0.4.19 로 올렸다. **숫자를 소급해
> 고치지 않는다** — 그 값들은 0.4.15 에서 얻은 것이고 그대로 유효하다.
>
> 지금 저장소를 clone 해서 빌드하면 0.4.19 가 들어간다. 재현 시 처리량이
> 미세하게 다를 수 있고, **다르다면 그것도 결과다.** 동결 바이너리
> (`*.frozen-01f29a2`)는 0.4.15 로 빌드된 것이며 대조용으로 보존한다.
>
> 보안 권고를 무시하고 lock 을 묶어 두는 선택지도 있었으나 택하지 않았다.
> **공개 저장소가 알려진 취약점을 담고 있는 편이 더 나쁘다.**

버전 조합을 변경하면 이전 조합으로 측정한 벤치마크 결과와 직접 비교할 수 없다. 변경 시 재측정 필요 여부를 함께 판단한다.

---

<a id="hosts-readme"></a>

# 호스트 인벤토리

스케줄러 호스트의 하드웨어 규격을 기계가 수집한 그대로 남긴다.

| 파일 | 호스트 | 기간 |
|---|---|---|
| `server-xeon-e5-2630l-20260826.md` | **Dell PowerEdge R620** / Xeon E5-2630L ×2 | 2026-08-20 ~ 08-26 (**측정 421건**) |
| `server-i7-4790-20260826.md` | Core i7-4790 / ASUS H81M-K | 2026-08-26 ~ |

## 왜 있는가

**구서버(Xeon E5-2630L ×2)의 규격이 남아 있지 않았다.** 측정 421건이 나온
장비인데 CPU·RAM 용량·NIC 이름만 문서에 적혀 있고 메인보드·RAM 종류·
디스크 모델·PCIe 정보가 없었다.

2026-08-26 에 그 서버를 다시 켜서 뒤늦게 수집했다. **운이 좋았다** —
장비가 아직 손 닿는 곳에 있었기 때문이다. 그 사이 OS 는 9.4 → 9.8 로
바뀌었고 10G 카드는 빠져 있었다. **뒤늦은 수집은 당시 상태를 온전히
복원하지 못한다.**

보드에는 `collect-node-info.sh` 가 있었지만 호스트에는 없었다.
`server-profile-collect.sh` 는 성능 프로파일러(S3.9a)이지 인벤토리가 아니다.

## 수집

```bash
ssh <host> 'bash -s' < scripts/collect-host-info.sh > docs/hosts/<name>-<date>.md
```

**호스트를 바꾸면 배치 전에 먼저 돌린다.** 시리얼·자산번호·UUID 는
수집하지 않는다 — 재현에 필요한 것은 모델명과 규격이지 개체 식별자가 아니다.

---

<a id="hosts-server-i7-4790-20260826"></a>

# 호스트 인벤토리 — server

- 수집: 2026-08-26 15:26:33 KST
- 수집기: `scripts/collect-host-info.sh`

## 시스템

| 항목 | 값 |
|---|---|
| hostname | server |
| 메인보드 | ASUSTeK COMPUTER INC. H81M-K |
| BIOS | 1003 (10/24/2014) |
| 배포판 | Rocky Linux 9.4 (Blue Onyx) |
| 커널 | 5.14.0-427.13.1.el9_4.x86_64 |
| 아키텍처 | x86_64 |
| glibc | 2.34 |
| SELinux | Enforcing |

## CPU

| 항목 | 값 |
|---|---|
| CPU(s) | 8 |
| Model name | Intel(R) Core(TM) i7-4790 CPU @ 3.60GHz |
| Thread(s) per core | 2 |
| Core(s) per socket | 4 |
| Socket(s) | 1 |
| CPU(s) scaling MHz | 99% |
| CPU max MHz | 4000.0000 |
| CPU min MHz | 800.0000 |
| L3 cache | 8 MiB (1 instance) |

## 메모리

| 항목 | 값 |
|---|---|
| 총량 | 15 GB |

| 슬롯 | 용량 | 종류 | 속도 | 제조사 |
|---|---|---|---|---|
| ChannelA-DIMM0 | 8 GB | DDR3 | 1600 MT/s | Samsung |
| ChannelB-DIMM0 | 8 GB | DDR3 | 1600 MT/s | Samsung |

## 저장장치

```text
NAME  SIZE MODEL              ROTA
sda   1.8T ST2000VN004-2E4164    1

Filesystem           Size  Used Avail Use% Mounted on
/dev/mapper/rl-root   70G  5.3G   65G   8% /
```

## 네트워크

```text
enp3s0           DOWN           
enp1s0           UP             192.168.123.9/24 fe80::f4c7:56a1:f4a6:5cfd/64 
```

| 인터페이스 | 속도 | 드라이버 | PCI | PCIe 링크 |
|---|---|---|---|---|
| `enp1s0` | 10000Mb/s | ixgbe | 0000:01:00.0 | Speed 5GT/s (downgraded), Width x4 (ok) |
| `enp3s0` | Unknown! | r8169 | 0000:03:00.0 | Speed 2.5GT/s (ok), Width x1 (ok) |

## PCIe 슬롯

| 슬롯 | 규격 | 사용 |
|---|---|---|
| PCIEX16_1 | x16 PCI Express | In Use |
| PCIEX1_1 | x1 PCI Express | Available |
| PCIEX1_2 | x1 PCI Express | In Use |

**루트 포트 능력** — 카드가 느리면 슬롯 탓인지 카드 탓인지 여기서 갈린다.

```text
00:01.0
  LnkCap:	Port #2, Speed 5GT/s, Width x16, ASPM L0s L1, Exit Latency L0s <256ns, L1 <8us
  LnkSta:	Speed 5GT/s (ok), Width x4 (downgraded)
00:1c.0
  LnkCap:	Port #1, Speed 5GT/s, Width x1, ASPM L0s L1, Exit Latency L0s <1us, L1 <4us
  LnkSta:	Speed 2.5GT/s (downgraded), Width x0 (downgraded)
00:1c.2
  LnkCap:	Port #3, Speed 5GT/s, Width x1, ASPM L0s L1, Exit Latency L0s <512ns, L1 <16us
  LnkSta:	Speed 2.5GT/s (downgraded), Width x1 (ok)
```

## PCI 장치

```text
00:00.0 Host bridge: Intel Corporation 4th Gen Core Processor DRAM Controller (rev 06)
00:01.0 PCI bridge: Intel Corporation Xeon E3-1200 v3/4th Gen Core Processor PCI Express x16 Controller (rev 06)
00:02.0 VGA compatible controller: Intel Corporation Xeon E3-1200 v3/4th Gen Core Processor Integrated Graphics Controller (rev 06)
00:14.0 USB controller: Intel Corporation 8 Series/C220 Series Chipset Family USB xHCI (rev 05)
00:16.0 Communication controller: Intel Corporation 8 Series/C220 Series Chipset Family MEI Controller #1 (rev 04)
00:1a.0 USB controller: Intel Corporation 8 Series/C220 Series Chipset Family USB EHCI #2 (rev 05)
00:1b.0 Audio device: Intel Corporation 8 Series/C220 Series Chipset High Definition Audio Controller (rev 05)
00:1c.0 PCI bridge: Intel Corporation 8 Series/C220 Series Chipset Family PCI Express Root Port #1 (rev d5)
00:1c.2 PCI bridge: Intel Corporation 8 Series/C220 Series Chipset Family PCI Express Root Port #3 (rev d5)
00:1d.0 USB controller: Intel Corporation 8 Series/C220 Series Chipset Family USB EHCI #1 (rev 05)
00:1f.0 ISA bridge: Intel Corporation H81 Express LPC Controller (rev 05)
00:1f.2 SATA controller: Intel Corporation 8 Series/C220 Series Chipset Family 6-port SATA Controller 1 [AHCI mode] (rev 05)
00:1f.3 SMBus: Intel Corporation 8 Series/C220 Series Chipset Family SMBus Controller (rev 05)
01:00.0 Ethernet controller: Intel Corporation Ethernet Controller 10G X550T (rev 01)
03:00.0 Ethernet controller: Realtek Semiconductor Co., Ltd. RTL8111/8168/8211/8411 PCI Express Gigabit Ethernet Controller (rev 0c)
```

## 서비스

| 항목 | 값 |
|---|---|
| firewalld | active / enabled |
| 열린 포트(영구) | 8080/tcp 9090/tcp 50051/tcp |
| chronyd | active / enabled |
| 시각 동기 | yes |

> 시리얼·자산번호·UUID 는 수집하지 않는다. 재현에 필요한 것은
> 모델명과 규격이지 개체 식별자가 아니다.

---

<a id="hosts-server-xeon-e5-2630l-20260826"></a>

# 호스트 인벤토리 — Dell PowerEdge R620 (구 스케줄러 서버)

- 수집: 2026-08-26
- 수집 방법: **콘솔에서 수동.** SSH 공개키가 등록돼 있지 않아
  `scripts/collect-host-info.sh` 를 원격 실행하지 못했다. 같은 항목을
  콘솔에서 직접 뽑아 옮겼다.
- 역할: **측정 421건의 스케줄러 호스트** (2026-08-20 ~ 08-26)

> ## ⚠️ 이 수집은 측정 시점의 상태가 아니다
>
> | | 측정 시점 (문서 기록) | 수집 시점 (2026-08-26) |
> |---|---|---|
> | 배포판 | **Rocky 9.4** | Rocky 9.8 |
> | 커널 | **`5.14.0-427.13.1.el9_4`** | `5.14.0-687.41.1.el9_8` |
> | 10G NIC | **Intel X550T 장착** (`enp4s0`) | **없음 — 카드를 빼서 신서버로 옮겼다** |
> | IP | 192.168.123.9 (static) | 192.168.123.19 (온보드 NIC, DHCP) |
>
> **측정 조건은 문서 기록 쪽이다.** 이 표는 하드웨어 규격을 남기려고
> 뒤늦게 뜬 것이고, 소프트웨어 상태는 그 사이 바뀌었다.

---

## 시스템

| 항목 | 값 |
|---|---|
| 제조사 / 모델 | **Dell Inc. PowerEdge R620** |
| 베이스보드 | `0VV3F2` |
| BIOS | `2.2.3` (2014-05-20) |
| 배포판 (수집 시점) | Rocky Linux 9.8 (Blue Onyx) |
| 커널 (수집 시점) | `5.14.0-687.41.1.el9_8.x86_64` |
| glibc | 2.34 |

## CPU

| 항목 | 값 |
|---|---|
| 모델 | **Intel Xeon E5-2630L @ 2.00GHz** |
| 소켓 | **2** |
| 소켓당 코어 | 6 |
| 코어당 스레드 | 2 |
| **총 스레드** | **24** |
| 최대 클럭 | 2500 MHz |
| L3 | 30 MiB (15 MiB × 2 instance) |

> 앞서 문서에 `1.8GHz` 로 적혀 있었으나 **틀렸다.** base 2.0GHz / turbo 2.5GHz 다.

## 메모리

총 16GB. **24 슬롯 중 4개만 채워져 있다.**

| 슬롯 | 용량 | 종류 | 속도 |
|---|---|---|---|
| DIMM_A1 | 4 GiB | DDR3 | 1333 MT/s |
| DIMM_A2 | 4 GiB | DDR3 | 1333 MT/s |
| DIMM_B1 | 4 GiB | DDR3 | 1333 MT/s |
| DIMM_B2 | 4 GiB | DDR3 | 1333 MT/s |

나머지 20개(A3~A12, B3~B12)는 비어 있다. 소켓당 2 DIMM 이므로
**소켓당 2채널만 활성**이다(E5-2630L 은 소켓당 4채널 지원).

## 저장장치

```text
NAME   SIZE MODEL      ROTA
sda  278.9G PERC H710P    1

/dev/mapper/rl-root   70G  7.3G  63G  11% /
```

`PERC H710P` (Broadcom/LSI MegaRAID SAS 2208) RAID 컨트롤러 뒤의 볼륨이다.

## 네트워크

**온보드는 Intel I350 쿼드 포트 1GbE 다. 10G 는 없다.**

| 인터페이스 | 속도 | 드라이버 | PCI |
|---|---|---|---|
| `eno1` | 1000Mb/s Full | `igb` | `01:00.0` |
| `eno2` `eno3` `eno4` | 링크 없음 | `igb` | `01:00.1~.3` |

> 이것이 **`enp4s0` 이 내장 10G 였다는 종전 기록이 틀렸다는 증거**다.
> 10G 는 Intel X550T **카드**였고, 지금은 빼서 신서버에 꽂혀 있다.

## PCIe 슬롯 — 카드가 빠진 지금도 슬롯 능력은 남는다

| 슬롯 | 규격 | 사용 |
|---|---|---|
| `PCI1` | PCI Express 3 | Available |
| `PCI2` | PCI Express 3 | Available |

### 루트 포트 능력

```text
00:01.0  LnkCap 8GT/s x8    LnkSta 5GT/s x4    <- 온보드 I350 (PCIe 2.0 x4 카드)
00:02.0  LnkCap 8GT/s x8    LnkSta x0          <- 비어 있음
00:02.2  LnkCap 8GT/s x8    LnkSta 5GT/s x8    <- PERC H710P (PCIe 2.0 x8)
00:03.0  LnkCap 8GT/s x16   LnkSta x0          <- 비어 있음
40:02.0  LnkCap 8GT/s x16   LnkSta x0          <- 비어 있음 (2번 소켓 IIO)
```

**빈 루트 포트 셋이 모두 8GT/s(PCIe 3.0)다.** 물리 슬롯 `PCI1`·`PCI2` 도
`PCI Express 3` 로 보고된다. 따라서 **X550T 를 어느 슬롯에 꽂았든 PCIe 3.0
으로 물렸다.**

### 그래서 링크 대역은 절반으로 줄었다

| | 구서버 (R620) | 신서버 (H81M-K) |
|---|---|---|
| 슬롯 세대 | **PCIe 3.0** (`LnkCap 8GT/s`) | PCIe 2.0 (`LnkCap 5GT/s`) |
| X550T 링크 | 8GT/s × x4 | 5GT/s × x4 |
| 방향당 대역 | **약 32 Gbps** | 약 16 Gbps |

**병목은 아니다.** 3노드 실사용이 방향당 ~4.6 Gbps 라 16 Gbps 도 3.5배
여유다. 다만 이것은 이제 **추정이 아니라 측정된 값**이다.

→ 기준선 차이(391 → 360)의 원인은 여전히 호스트 CPU 다.
   `../infrastructure.md` §3.2.1

## PCI 장치 (요약)

```text
01:00.0~.3  Intel I350 Gigabit Network Connection  x4
02:00.0     Broadcom/LSI MegaRAID SAS 2208 (PERC H710P)
0a:00.0     Matrox G200eR2 (BMC 통합 VGA)
07~09:xx    Renesas SH7757 PCIe Switch/Bridge (내부 브리지)
```

## 이 서버로 무엇을 했나

측정 421건 전부가 이 호스트에서 나왔다. S2 baseline · S3 saturation ·
S3.5~S3.9b transport 계보 · S0-A~D 열/정책 계보.

교체 경위와 그 영향은 `../infrastructure.md` §3.2.1 ·
`../environment-matrix.md` §10.2 · `../../results/baseline-20260826-althost/`.

---

<a id="experiments-readme"></a>

# 실험 대장 (Experiment Index)

- 최종 갱신: **2026-08-20**
- 대상: M3 클러스터 (RK3576 ×3 + Xeon 스케줄러), YOLOv8n INT8, `want_float=0`
- 고정 조건: governor `performance`, Active Cooling(노드당 120mm 팬), round-robin,
  worker 8/node, gRPC(tonic + protobuf), closed-loop bench

> 각 실험의 상세는 개별 보고서에 있다. 이 문서는 **무엇을 물었고 무엇이
> 배제됐는지**를 한 장으로 본다.
> 용어는 [`../GLOSSARY.md`](#glossary) 에 정리돼 있다.

**한 문장 요약**

> NPUForge 의 transport 최적화는 custom transport 구현에서 시작했지만,
> 측정 기반 병목 제거를 통해 **표준 gRPC 구성만으로 3노드 처리량을 13.3%
> 개선**했고, 그 과정에서 **성능 최적화보다 operating-point 선정과 실험
> 검증이 먼저**라는 사실을 확인했다.

---

## 1. 실험 대장

| ID | 질문 | 규모 | 핵심 결과 | 문서 |
|---|---|---:|---|---|
| **예비** | 3노드가 실제로 도는가 | 3 run | 336 inf/s, 오류 0, 33.3% 균등 | `board-worklog` §2.24 |
| **S2** | 노드를 늘리면 선형으로 늘어나는가 | **30 run** | **112.9 / 229.0 / 338.4 inf/s**, speedup **3.00×**, eff 100%, 오류 0 | [S2](#experiments-s2-grpc-baseline) |
| **S3** | 각 구성의 진짜 상한은 | **45 run** | ceiling **115.2 / 232.0 / 341.8**, 3N **2.97×** | [S3](#experiments-s3-saturation) |
| **S3.5** | −30% 손실이 어디서 오는가 | 3 조건 | 대역폭·CPU 총량·서버 배제 → **전송 경로**로 좁힘 | [S3.5](#experiments-s3-5-transport-profile) |
| **S3.5b** | CPU0 softirq 편중이 원인인가 | 6 run | **−0.2% (null)** — 단, 흐름이 1개라 반박 여지 | [S3.5](#experiments-s3-5-transport-profile) §4.3 |
| **S3.6** | flow control 인가 커넥션인가 | **20 run** | window 확대 **−36.3%**, 커넥션 1→4 **+21.5%** | [S3.6](#experiments-s3-6-h2-channel-ab) |
| **S3.7a** | 커넥션 몇 개가 최적인가 (c32 고정) | **25 run** | c4 에서 knee, 그 뒤 tail 만 악화 | [S3.7](#experiments-s3-7-connection-tuning) |
| **S3.7b** | 각 구성의 **운영점**은 | **75 run** | 운영점 셋 다 **c12**. conn2 가 conn1 을 **양 축에서 지배** | [S3.7](#experiments-s3-7-connection-tuning) §4 |
| **S3.7c** | 운영점에서 RPS 는 효과가 있는가 | **10 run** | **−0.8% (null)**, CPU0 %soft 68→56 인데도 불변 | [S3.7](#experiments-s3-7-connection-tuning) |
| **S3.8** | 최적화가 scale-out 을 해치지 않는가 | **36 run** | **135.5 / 263.3 / 387.2**, 3N **2.86× (95.3%)**. 절대 +13.3% 이나 eff 는 98.9→95.3% | [S3.8](#experiments-s3-8-optimized-scaleout) |

| **S3.9a** | 3N 의 4.5% efficiency 손실은 어디서 생기는가 | **9 run** | 서버 자원 **전부 배제**. 손실은 **tail 에서 나타난다** (p50 평평, p99 +36%). TCP 재전송 3.5배 — 다만 micro-mechanism 까지 분리한 것은 아니다 | [S3.9a](#experiments-s3-9a-scaleout-profile) |
| **S0-B** | 운영점이 지속 부하에서 유지되는가 (능동 냉각) | **30 run / 31분** | **degradation 1.9%**, 클럭 강등 **0회**. short-run = sustained | [S0](#experiments-s0-sustained-load) |
| **S0-A** | 팬리스면 어떻게 되는가 | **30 run / 32분** | **degradation 11.3%**. CPU 2208→**816 MHz**(king), NPU 는 950 고정. **king 2.4배 느린데 round-robin 은 1/3 유지** | [S0](#experiments-s0-sustained-load) |

| **S0-C** 1차 | 부하 인지 정책이 열 불균질 손실을 회수하는가 | **15 run** | 정책이 처리량을 **55~58% 붕괴**시킨다. 원인은 **하트비트 stale 상태 herding** — 스케줄러 버그 발견 | [S0-C](#experiments-s0-c-policy-ab) |
| **S0-C** 2차 | 위 버그 수정 후 재측정 | **12 run** | **RR 373.9 / LQ 380.9 / ECT 384.2.** 붕괴 소멸, **p99 −37%**, 노드 지연 편차 1.33×→**1.00×** | [S0-C](#experiments-s0-c-policy-ab) §8~11 |

| **S0-C** 3차 | 동질(능동 냉각)에서 정책 regression 이 없는가 | **12 run** | regression 없음(LQ −0.0%, ECT −0.3%). tail 은 동질에서도 개선. **LQ·ECT 어느 쪽도 지배 못 함** | [S0-C](#experiments-s0-c-policy-ab) §12~15 |
| **S0-C** 4차 | 강한 이질(2.4×)에서 LQ vs ECT | **1 run (중단)** | **게이트 미달 1.10×.** 열 조건은 동일했다(86.8°C) — **이질을 정하는 것은 온도가 아니라 CPU 강등 편차** | [S0-C](#experiments-s0-c-policy-ab) §17~19 |
| **S3.9b** | residual gap 에서 node-side syscall/copy 가 유의미한가 | **4 조건** | **syscall 은 아니다** — transport 비용의 ~1%(관대히 8%). 유저 시간이 커널보다 크다(9.37 vs 6.99 ms/req). CPU 는 48.9% idle = **제약이 아니다**. → **S4 io_uring 취소/보류** | [S3.9b](#experiments-s3-9b-node-residual) |
| **S0-D** 교정 | 이질을 결정론적으로 만들 수 있는가 | **12 run** | **가능하다.** 캡 2208→600 으로 편차 **1.12×→3.93×**. **캡 816 이 S0-A(2.4×)를 6ms 이내로 재현** | [S0-D](#experiments-s0-d-capacity-hetero) |
| **S0-D** 정책 | 편차가 커질수록 ECT 가 유리해지는가 | 미실시 | ECT 기본값의 설계 근거를 직접 시험한다 | [S0-D](#experiments-s0-d-capacity-hetero) §6 |

**측정 run 합계: 421건** (bench 418 + 프로파일 조건 3), 전 구간 **오류율 0**.
폐기 4건(하네스 충돌 오염, `results/policy-ab-20260821-contaminated/`)은 제외.

> 이 숫자는 **손으로 관리하지 않는다** — `bash scripts/count-runs.sh` 가 센다.
> 두 문서가 각자 들고 있다가 343 vs 420 으로 갈라진 적이 있다(2026-08-21).

### 1.1 원본 데이터 대응표

`results/` 의 모든 디렉터리가 어느 실험에 속하는지 적는다. **고아 데이터를
만들지 않기 위한 표**다 — 실제로 87건이 어느 문서에도 안 걸린 채 남아
있었다(2026-08-21 발견).

| 디렉터리 | run | 실험 |
|---|---:|---|
| `scaling-20260820` | 3 | 예비 측정 (`RESULTS.md` §2.5, **대체됨**) |
| `baseline-20260820` | 30 | S2 |
| `saturation-20260820` | 45 | S3 |
| `transport-profile-20260820` | 3 조건 | S3.5 |
| `rps-ab-20260820` | 6 | S3.5b (RPS null) |
| `h2-channel-ab-20260820` | 20 | S3.6 |
| `connection-sweep-20260820` | 25 | S3.7a (c32 고정) |
| `concurrency-sweep-20260820` | 30 | **S3.7b** — conn 2·4 × c24~64 (과부하 구간) |
| `concurrency-sweep-20260820-low` | 30 | **S3.7b** — conn 2·4 × c8~24 (운영 구간) |
| `concurrency-sweep-20260820-conn1` | 15 | **S3.7b** — conn 1 × c8~24 |
| `s37b-operating-point` | 45 | S3.7b 운영점 확정 |
| `scaleout-optimized-20260820` | 36 | S3.8 |
| `scaleout-optimized-20260820-1n-only` | 12 | **S3.8** — 1N 재측정 |
| `scaleout-profile-20260821` | 9 | S3.9a |
| `node-residual-20260821` | 4 조건 | S3.9b |
| `sustained-20260821-fan` | 30 | S0-B |
| `sustained-20260821-fanless` | 30 | S0-A |
| `policy-ab-20260821` | 15 | S0-C 1차 (herding 버그 발견) |
| `policy-ab-20260821b` | 12 | S0-C 2차 |
| `policy-ab-20260821fan` | 12 | S0-C 3차 |
| `policy-ab-20260821-contaminated` | 4 | S0-C 4차 — **폐기**(하네스 충돌) |
| `capacity-calib-20260821` | 12 | S0-D 교정 |
| `accuracy` · `thermal-20260811-*` | — | 모델 정확도 · 예비 열 측정 (`RESULTS.md`) |

> ⚠️ `results/policy-ab-20260821-contaminated/` — **concurrent harness
> collision; invalid for performance conclusions.** S0-C 4차 진행분이며
> r1 round-robin 과 1초 열 로그만 유효하다. 사고 경위는 §4.11 과
> [S0-D](#experiments-s0-d-capacity-hetero) §4. 방법론 기록으로 보존한다.

---

## 2. 병목 후보 현황 — 배제는 조건부다

**이 표의 값어치는 후보 공간을 줄인 데 있지, 남은 gap 의 정체를 특정한 데
있지 않다.** 그리고 **한 번 배제한 후보도 조건이 바뀌면 다시 열린다** —
S3.8 이 실제로 그랬다(§4.7).

| 후보 | 현재 판정 | 근거 |
|---|---|---|
| 링크 대역폭 | **배제** | 노드 eth0 방향당 51% (S3.5), 서버 10G **방향당 40%** (S3.9a). S3.8 의 "76%" 는 full-duplex 계산 오류로 **철회** |
| 보드 CPU 총량 | **배제** | 8코어 **49~63% idle** (S3.5, S3.7c) |
| 서버 CPU·NIC·스케줄러 | **배제 (24스레드 호스트 한정)** | CPU 42%, 최번 코어 47.6%, drop 0, 스레드 직렬화 없음, syscalls/req 불변 (S3.9a). ⚠️ **8스레드 호스트에서는 다시 열린다** — 아래 참조 |
| **공유 경로 혼잡 (10G→2.5G)** | **신규 유력 (미검증)** | 커넥션당 TCP 재전송률 **3.5배**, cwnd 176→106~119. p50 평평·tail 만 증가와 정합 (S3.9a) |
| 커널 RX 분산 (RPS) | **배제** | CPU0 %soft 를 **12%p 덜어도** 처리량 불변 (S3.7c) |
| HTTP/2 flow control | **극단값 역효과** | 64 MB 확대 시 **−36.3%**. **중간값(256 KB~4 MB) 미측정** (S3.6) |
| **노드당 커넥션 수** | **주요 제약** | 1→2 로 처리량 **+18.8%**, tail 도 **−18.8%** (S3.7b) |
| 남은 비용 | **미분리** | protobuf / memcpy / syscall / H2 구현 / userspace 스케줄링 / NPU submission → 프로파일 필요 |

> **"서버·스케줄러 배제" 는 baseline(conn1) 조건에서만 유효했다.** 노드당
> 전송을 최적화하자 shared path 로 가는 부하가 늘어 그 배제가 무너졌다.
> 배제 판정에는 **어떤 조건에서 배제됐는지**가 함께 붙어야 한다.

> **같은 배제가 두 번째로 무너졌다 — 이번에는 하드웨어 조건에서 (2026-08-26).**
> 스케줄러 호스트를 24스레드(Xeon E5-2630L ×2)에서 8스레드(Core i7-4790)로
> 교체하자 기준선이 **391 → 360 inf/s (−7.5%)** 로 내려갔다. 측정 중 서버
> CPU 는 **82.2%** 였다(구서버 조건에서는 42%).
>
> 흥미로운 것은 **애플리케이션 큐는 여전히 비어 있다**는 점이다 —
> `scheduler_queue` 0.00ms · `scheduler_route` 0.01ms. S3.9a 가 실제로 배제한
> 것은 그 큐였고, 그 판정 자체는 지금도 옳다. 좁아진 곳은 그 **바깥**,
> 호스트의 CPU 다. **벤치 클라이언트가 스케줄러와 같은 호스트에서 돈다**는
> 측정 구조가 이를 키운다.
>
> 측정 421건은 전부 구서버에서 얻었고 **그 값은 그대로 유효하다.** 신서버
> 재현치는 `../infrastructure.md` §3.2.1 · `../environment-matrix.md` §10.2 에
> 따로 적었다. 두 호스트의 값을 직접 비교하지 않는다.

---

## 3. 수치 계보 — 노드당 처리량은 어떻게 움직였나

**커넥션 1개** 구성은 다섯 실험에서 독립적으로 **113~117** 에 모인다.
서로 다른 날, 다른 하네스, 다른 목적의 측정이 같은 값을 낸다는 뜻이다.

```text
S2   1N @c8   (30 run)   112.9 ± 0.5
S3   1N ceiling @c32     115.2
S3.5 cluster  @c32       116.6
S3.6 A(1ch)   @c32       115.3 ± 0.8
S3.7a c1      @c32       115.6 ± 0.7
S3.7b conn1   @c12  ★    114.8 ± 0.7   ← conn1 의 운영점
─────────────────────────────────────
S3.7b conn2   @c12  ★★   136.4 ± 0.3   ← optimized 운영점  (+18.8%)
S3.8  conn2   @c12       135.5 ± 0.4   ← 다른 하네스로 독립 재현
로컬 direct (네트워크 없음)  161.5
─────────────────────────────────────
운영점 잔여 gap  161.5 − 135.5 = 26.0 inf/s = **direct 기준 16.1%**
                 (S3.7b 136.4 기준이면 25.1 = 15.5%)
```

★ 같은 규칙(peak 98%)으로 찾은 운영점끼리의 비교라야 공정하다.

---

## 4. 방법론 교훈 — 숫자보다 오래 남을 것

### 4.1 운영점에서 최적화하라, 과부하 구간에서 하지 말고

> **Optimize at the operating point, not in the overload region.**

S3.6·S3.7a 는 **c32 고정**에서 "커넥션을 늘리면 tail 이 46% 나빠진다" 를
얻었다. 측정은 옳았지만 c32 는 세 구성 **모두에게 과부하**였다. 운영점
(c12)에서 다시 재자 같은 변경이 **tail 을 18.8% 개선**했다. 결론의 부호가
뒤집혔다.

고정 부하 비교는 configuration effect 가 아니라 **overload behavior** 를
보여줄 수 있다. 그래서 c32 결과는 폐기하지 않고 "과부하 거동" 이라는 별도
결과로 남긴다 — 다만 운영 판단의 근거로 쓰지 않는다.

### 4.2 두 측정이 일치해도 해석이 옳다는 뜻은 아니다

c32(+28.2%)와 c24(+27.4%)가 거의 일치해 "4 커넥션 자체의 성질" 이라고
썼다. **둘 다 과부하 구간이라 같은 편향을 두 번 본 것**이었다.
재현성은 편향을 확인해 줄 뿐이다.

### 4.3 판정 규칙을 결과보다 먼저 정하고, 결과에 맞춰 옮기지 않는다

운영점 정의를 코드 상수로 박았다.

> operating concurrency = peak 의 **98%** 이상을 내는 **가장 낮은** concurrency
> (99% 는 run 간 SD ±1 inf/s 와 겹친다)

S3.7a 에서 c2 가 **96.4%** 로 임계를 0.6%p 차이로 놓쳤다. 임계를 96% 로
내리면 원하는 답이 나왔지만 **내리지 않았다.** 대신 "규칙이 이 경계에서
결론을 내주지 못한다" 를 결과로 기록했고, S3.7b 가 데이터로 풀었다.

### 4.4 조용한 실패를 큰 소리로 바꾼다

- **하네스가 실패하면 멈추게 한다.** 노드가 Mock 백엔드로 빌드돼 기동
  실패했을 때 하네스가 즉시 큰 소리로 죽어 바로 잡혔다.
- **설정이 먹었는지 물증을 남긴다.** run 마다 `ss` 로 실제 TCP 커넥션 수를
  세어 기록했다. 설정이 조용히 무시되면 A/B 가 아니라 같은 조건 4번이 된다.
- **노드 수를 측정 전에 검증한다.** 프로세스 존재 ≠ 트래픽 수신. S3.8 은
  probe bench 로 **응답한 노드 ID 분포**를 세고 expected ≠ observed 면
  그 구성을 건너뛴다.
- **원본을 지우지 않는다.** run 사이에 출력 디렉터리를 비우는 버그로 이전
  run 의 JSON 이 날아간 적이 있다(처리량은 CSV 에 남아 결론은 무사).
  이후 스테이징 디렉터리로 분리했다.

### 4.5 한 번 배제한 후보도 조건이 바뀌면 다시 열린다

S2 에서 3노드가 3.00× 선형이었으므로 **서버·스케줄러를 배제**했다. 그
판정은 옳았다 — **conn1 조건에서는.**

노드당 커넥션을 2개로 올리자 shared path 로 가는 부하가 늘었고,
optimized 3N 의 efficiency 가 **95.3%** 로 내려갔다. 서버가 다시 후보다.

> **배제 판정에는 "어떤 조건에서" 가 함께 붙어야 한다.** 조건이 바뀌면
> 배제표를 다시 읽어야 한다. 배제표를 한 번 쓰고 고정하면, 자기가 만든
> 최적화가 만든 새 병목을 보지 못한다.

### 4.6 throttling 판정에도 조건이 붙어야 한다

같은 하드웨어·같은 운영점·같은 부하인데 **냉각 하나로 결론이 바뀐다.**

```text
능동 냉각   degradation  1.9%   클럭 강등 0회      NPU 60°C
팬리스      degradation 11.3%   CPU 2208→816 MHz   NPU 88°C
```

worklog 의 "CPU 300초 −27%" 도 옳은 관측이었다 — **그 조건에서.**
S0-A 는 CPU 가 똑같이 816 MHz 까지 떨어지는 것을 봤지만 손실은 −11.3%
였다(클러스터 부하는 CPU 여유가 있다). **조건을 떼면 숫자가 거짓말을 한다.**

배제 판정(§4.5)과 같은 이야기다. **"throttling 이 있다/없다" 는 조건과
함께 써야 한다.** 조건을 떼면 다음 사람이 잘못된 전제로 계획을 세운다 —
실제로 이번 세션은 그 −27% 때문에 S0 를 io_uring 앞에 두었다(옳은 판단
이었지만, 근거는 확인해 보니 다른 조건의 것이었다).

### 4.7 "정책이 나쁘다" 와 "구현이 고장났다" 를 먼저 가른다

S0-C 는 부하 인지 정책이 처리량을 **55~58% 떨어뜨리는** 것을 봤다. 여기서
"부하 인지 정책은 이 워크로드에 안 맞는다" 로 끝냈다면 틀린 결론이었다.

단계 분해가 갈랐다 — `scheduler_route` 가 셋 다 0.004 ms(결정은 빠르다),
`node_queue` 0.023 ms(노드는 안 밀린다), 그런데 왕복만 2.8배. 노드 CPU 는
오히려 절반(45% → 20%). **일을 더 하는 게 아니라 더 기다린다.**

원인은 정책의 판단 품질이 아니라 **상태 신선도**였다. 정책이 보는
`queue_depth` 가 하트비트로만 갱신되고 디스패치 경로가 이를 갱신하지 않아,
초당 수백 건이 동일한 고정 스냅샷을 보고 전부 같은 노드를 고른다.

> **성능이 이상하면 "이 접근이 나쁘다" 보다 "구현이 의도대로 도는가" 를
> 먼저 본다.** 55% 는 품질 차이의 크기가 아니다.

수정 후 재측정하니 정책이 정상 동작했고 **p99 가 37% 좋아졌다.** 그대로
결론냈다면 "부하 인지 정책은 도움이 안 된다" 는 정반대 기록이 남았을 것이다.

한 가지 더 — **판정 규칙이 안 걸린 것도 결과다.** 2차에서 king 분배 이동은
0.5%p 로 3%p 규칙에 미달했다. 임계를 내리지 않고, 규칙이 안 걸린 이유
(열 편차가 1.33배로 약했고 least-outstanding 은 개수가 아니라 동시 점유를
조절하는 폐루프다)를 함께 적었다.

### 4.8 percentile 집계 방식을 명시한다

여러 run 을 묶은 표의 p95/p99 는 **run-level percentile 의 평균**이지
pooled percentile 이 아니다. run-level 평균은 각 run 의 최악 구간이 희석돼
**tail 을 낮게 보이게 한다.** 조건 간 비교에는 유효하나 절대값을 "이
시스템의 p99" 로 인용하면 안 된다 → [S2 §7.4.1](#experiments-s2-grpc-baseline)

### 4.9 로그 없는 프로세스는 사후에 아무것도 말해주지 않는다

jack 노드가 죽었는데 **원인을 확정할 수 없었다.** OOM 도 segfault 도
아니었고 로그가 아예 없었다 — 기동 절차의 `setsid nohup` 에 리다이렉트가
빠져 표준출력이 버려지고 있었다. 프로세스가 죽는 것보다 **왜 죽었는지 알 수
없는 것이 나쁘다.**

---

### 4.10 계기가 다른 물리량을 재고 있을 수 있다

정책 A/B 하네스는 run 마다 soc 온도를 기록했다. 그 값이 **78~79°C** 여서
"S0-A(86°C)보다 덜 뜨거웠다" 를 여러 문서에 적었고, 그 위에 "연속 가열이
필요하다" 는 처방까지 세웠다.

**틀렸다.** 하네스는 60초 run 이 **끝난 뒤** ssh 3번을 순차로 돌며 읽는다.
RK3576 은 부하가 끊기면 수 초 만에 식으므로 그 값은 run 간 골짜기다.
1초 열 로거로 다시 집계하니 **86.8°C 로 S0-A 와 같았다.** 열 조건은
처음부터 동일했고, 갈린 것은 CPU 강등 편차였다.

두 계기가 같은 이름(`soc`)을 달고 다른 물리량(run 중 최대 vs run 후
순간값)을 재고 있었다. **CSV 열 이름이 `max_soc_c` 였다는 점이 특히
나빴다** — 최대값이 아니었다.

> 판정 임계를 옮기는 것과, 임계를 재는 계기를 고치는 것은 다르다.
> 전자는 규칙을 결과에 맞추는 것이고(§4.3 위반), 후자는 규칙을 지키기
> 위해 필요한 일이다. 고칠 때 **기준값은 그대로 두고 출처만 바꿨다는
> 것을 문서에 명시**한다.

### 4.11 "중단했다" 를 믿지 말고 공유 자원 쪽에서 확인한다

하네스를 중단하고 다른 하네스를 띄웠다. 실제로는 **중단이 실패해 둘이
같은 3노드를 각각 c36 으로 때렸다.** 기준선이 197 inf/s(정상 391)로
나왔고 다음 run 은 오류율 82% 였다. **클러스터 고장으로 오진하기
직전이었다** — 정리 후 재측정하니 391.2 / 오류 0 이었다.

살아남은 하네스가 자기 설정으로 스케줄러를 계속 재기동했기 때문에,
"기본 설정으로 복구" 한 것이 몇 초 뒤 조용히 덮여 있었다.

**로컬 관측이 거짓말을 했다.** git-bash 에서 `ps -ef` 에 안 보였고
`pkill -f` 도 못 잡았다. PowerShell `Get-CimInstance Win32_Process` 로만
보였다. 프로세스 관측은 플랫폼에 따라 신뢰할 수 없다.

그래서 확인을 **공유 자원 쪽으로 옮겼다.** `npuforge_assert_cluster_free`
는 서버에 `npuforge-bench` 가 돌고 있으면 하네스를 시작하지 않는다.
서버에서 도는 벤치는 거짓말을 하지 않는다. (§4.4 의 연장이다 — 조용한
실패를 큰 소리로.)

### 4.12 하네스 불변조건 — §4.4·§4.11 을 규칙으로 굳힌다

두 사고(§4.11, 그리고 결과 경로 덮어쓰기)에서 나온 규칙이다. 새 하네스는
둘 다 지킨다.

1. **공유 자원의 상태는 공유 자원 쪽에서 검증한다.**
   로컬에서 "내가 중단했다" 를 아는 것으로는 부족하다. 클러스터가
   비었는지는 **클러스터에게 묻는다**(`npuforge_assert_cluster_free`).
2. **결과 경로를 append/overwrite 가능한 임시 폴더처럼 다루지 않는다.**
   `results/<실험>-<날짜>` 는 하루에 두 번 돌면 덮어쓴다. 실제로
   S0-C 1차(15 run)를 덮어썼고, git 추적 중이 아니었다면 사라졌다.
   기존 디렉터리가 비어 있지 않으면 멈춘다.

### 4.13 계측기가 거짓말한 여섯 번 — 권위 목록

§4.10·§4.11 이 두 건을 다루지만, 발표·공개 자료가 "여섯 번"을 인용한다.
**숫자를 쓰려면 목록이 있어야 한다.** 여기가 그 목록이다.

**범위: 클러스터 측정 캠페인(2026-08-20 ~ 08-21).** 단일 노드 시절의
실패 4건은 [`../RESULTS.md`](#results) §6 에 따로 있다. 둘을 합쳐
세지 않는다.

| # | 무엇이 거짓말했나 | 어떻게 드러났나 | 근거 |
|---:|---|---|---|
| 1 | **run 종료 후 온도 샘플링.** `max_soc_c` 열이 최대값이 아니라 run 간 냉각 골짜기였다 — 실제보다 ~5°C 낮다 | 1초 열 로거와 대조 | [S0-C §17.5](#experiments-s0-c-policy-ab) · §4.10 |
| 2 | **그 값 위에 세운 설명.** "2차가 1.33× 에 그친 건 덜 뜨거워서" — 실제로는 2차도 86.8°C 였다 | 2차 열 로그 재집계 | [S0-C §18.4](#experiments-s0-c-policy-ab) |
| 3 | **13.2% 가 과부하 구간 값이었다.** 그 백분율은 140.1(c32)에서 나왔는데 운영점 135.5 와 짝지어져 여러 문서에 퍼졌다. 실제는 16.1% | 분모를 직접 계산 | §3 · commit `62855bd` |
| 4 | **하네스 충돌.** 중단한 줄 안 하네스가 살아남아 새 하네스와 같은 클러스터를 각각 c36 으로 때렸다. 기준선 197 inf/s(정상 391), 다음 run 오류율 82% | 서버 프로세스 목록 확인 | [S0-D §4](#experiments-s0-d-capacity-hetero) · §4.11 |
| 5 | **결과 경로 덮어쓰기.** 같은 날짜 경로 재사용으로 S0-C 1차(15 run)가 4줄로 덮였다 | `git status` | [S0-D §4.2](#experiments-s0-d-capacity-hetero) · §4.12 |
| 6 | **`strace -c` 파서의 컬럼 오독.** `usecs/call` 과 `calls` 를 뒤바꿔 읽어 호출 수가 100배 작게 나왔다 | `/proc/PID/io` 기대치와 대조 | [S3.9b §8](#experiments-s3-9b-node-residual) |

**여섯의 공통점: 전부 "성공처럼 보였다."** 숫자가 나왔고, 그럴듯했고,
아무도 멈추지 않았다. 넷은 **다른 측정과 대조해서** 잡혔고(1·2·4·6),
둘은 **도구가 큰 소리로 알려줘서** 잡혔다(3·5).

> 여섯 중 **셋(1·2·3)이 같은 뿌리**다 — 계기 하나가 틀렸고, 그 위에
> 설명을 세웠고, 그 설명이 다른 문서로 퍼졌다. 계측 오류는 단독으로
> 끝나지 않는다.

## 5. 현재 확정 상태

**측정 계보는 2026-08-21 에 닫혔다.** S2 부터 S3.9b · S0-D 까지.

**두 계보를 섞지 않는다.** 전송 운영점과 스케줄링 정책은 근거가 다르다.

```text
Transport operating point  ── 확정 ──────────────────────────────
    2 connections/node @ concurrency 12/node

    1N   135.5 inf/s   p95 120.7 ms
    3N   387.2 inf/s   p95 151.1 ms   scaling 2.86x, eff 95.3%
    31분 지속(능동 냉각)  380.3 inf/s  (−1.9%, 클럭 강등 0회)

Adaptive policy  ── 확정 ───────────────────────────────────────
    기본값 `ect` 유지.

    RR 은 후보에서 빠졌다 — 이질 조건에서 p99 SD 34.7
    (adaptive 는 ~1). 부하 인지 스케줄링이 tail 을 크게 개선한다.

    LQ 와 ECT 는 **어느 쪽도 지배적이지 않다.**
      팬리스(이질)  LQ p99 146.9 / ECT 384.2 inf/s
      능동냉각(동질) 둘 다 정상. regression 없음 (S0-C 3차)
```

> **`node_connections` 는 "기본값" 이 두 가지다.** 헷갈리기 쉬워 적어 둔다.
>
> | | 값 | 무엇인가 |
> |---|---:|---|
> | 라이브러리 fallback | **1** | `SchedulerTransportConfig::default()`. baseline 재현용 — 설정을 안 주면 측정 초기 조건이 나온다 |
> | 권장 운영값 | **2** | `configs/scheduler.example.toml`. S3.7b 가 확정한 운영점 |
>
> 코드 기본을 2 로 올리면 과거 baseline 을 재현하려는 사람이 조용히 다른
> 조건을 얻는다. 그래서 **fallback 은 1 로 두고 예제가 2 를 권한다.**

> ECT 와 LQ 의 처리량 차이 0.9% 는 **우열 근거로 쓰지 않는다.** ECT 의
> 근거는 **노드 지연 편차를 1.00× 로 흡수했다**는 것 — 설계 의도대로
> heterogeneous capacity 를 반영했다는 뜻이다.

> **커넥션 단위 주의** — `node_connections` 는 **노드당** 값이다.
> 1N → 2 total, 2N → 4 total, 3N → 6 total.

conn1 baseline(114.8, 같은 규칙) 대비 **처리량 +18.8%, p95 −18.8%** —
측정한 처리량·지연 지표 기준으로 strict Pareto improvement.
로컬 direct(161.5)까지의 gap 46.7 중 **21.6(46%)를 설정만으로 회수**했다.

---

## 6. 계보가 어떻게 닫혔나

| 단계 | 결과 |
|---|---|
| ~~S3.8~~ | 운영점으로 scale-out 재검증. **+13.3%, efficiency 98.9→95.3%** |
| ~~S3.9a~~ | 서버 쪽 프로파일. **서버 자원 전부 배제** — 손실은 tail 에서 나타난다 |
| ~~S0-A / S0-B~~ | 지속 부하 30분. 팬리스 −11.3% / 능동 냉각 −1.9% |
| ~~정책 실장비 검증~~ | S0-C. 정책이 55~58% 붕괴 → **상태 신선도 결함(herding) 발견** |
| ~~herding 수정~~ | `local_in_flight` 원자적 예약 + RAII 가드 |
| ~~S0-C 3차~~ | 수정 후 정책이 적응한다. 동질 조건 regression 없음 |
| ~~S0-D~~ | 결정론적 이질 fixture(클럭 캡). 열에 기대지 않고 이질을 만든다 |
| ~~S3.9b~~ | 노드 쪽 남은 비용. **io_uring 이 닿는 몫은 ≈8%** |
| ~~S4 (io_uring)~~ | **적용하지 않는다.** 측정이 반박했다 → `01-TECHSPEC.md` §15 |

### 6.1 S0 결과 — 운영점은 냉각 조건에 딸려 있다

```text
short-run operating point                   3N 387~389 inf/s
sustained (능동 냉각)                       3N 380.3      (−1.9%)
sustained (팬리스)                          3N 345.4      (−11.3%)
```

능동 냉각에서는 클럭 강등 **0회**, 온도 58~61°C 평탄역 — **S2~S3.9a 의
60초 결과가 지속 운전에 그대로 적용된다.**

팬을 빼면 갈라진다. 강등된 것은 **NPU 가 아니라 CPU** 이고(950 고정 vs
2208 → 816 MHz), 보드마다 정도가 다르다.

그리고 **팬리스 손실은 순수한 열 문제가 아니다** — king 이 2.4배 느려졌는데
round-robin 은 여전히 1/3 을 보낸다(S0 §4.3). **열 편차 × 부하 무인지 정책**
의 곱이다. 그래서 다음이 io_uring 이 아니라 **정책 검증**이 됐다.

### 6.2 S4 의 질문이 바뀌었고, 답은 "하지 않는다" 였다

```text
처음   io_uring 이 gRPC 보다 얼마나 빠른가?
지금   표준 gRPC 스택을 제대로 구성했을 때 어디까지 가고, 그 뒤에 남는 비용은 무엇인가?
```

S3.9b 가 답했다. 운영점에서 남은 gap 중 **io_uring 이 닿을 수 있는 몫은
약 8%** 다. 구현 비용에 비해 회수량이 작다. **구현하지 않기로 했고 그
판정을 문서로 남긴다** — `01-TECHSPEC.md` §15.

---

## 7. 미해결 — 닫지 않고 열어 둔 것

측정 계보는 닫혔지만 아래는 답하지 않았다. **모르는 것을 안다고 적지
않는다.**

| 항목 | 상태 |
|---|---|
| **강한 이질(2.4배)에서 LQ vs ECT** | 미측정. 기본값 `ect` 유지 근거는 동질 sanity 통과. **S0-D 가 이 질문을 재현 가능하게 만들어 뒀다** |
| **3N efficiency 손실의 micro-mechanism** | tail 에서 나타나는 것까지는 확인(p50 평평, p99 +36%). 공유 경로 혼잡(10G→2.5G) 가설은 **미검증** — 스위치 카운터 접근이 필요 |
| **short-window 분배** | 60초 aggregate 만 있다. `bench --dump-samples` 가 필요 |
| **pooled percentile** | 같은 옵션이 필요. 현재 percentile 그림은 **run-level 평균**이다 |
| **노드 제외 동작** | 팬리스에서도 90°C 임계에 닿지 않아 미검증 |
| **격자 해상도** | 운영점 c12 가 진짜 knee 인지 c10 인지 미확정 (격자 4 단위) |
| **H2 window 중간값** | 256KB~4MB 미측정. 64KB↔64MB 극단만 봤다 |
| **c8/c16 커넥션 ceiling** | S3.7b 후보에서 제외했을 뿐 열등 증명은 아니다 |

> 이 표에 있는 것은 **하다 만 것이 아니라 하지 않기로 한 것**이다.
> 각 항목이 왜 열려 있는지가 함께 적혀 있다.

---

<a id="experiments-s0-c-policy-ab"></a>

# S0-C — Scheduling Policy A/B (팬리스)

- 실험 ID: **S0-C**
- 측정일: 2026-08-21
- 코드: `7281411` + `[transport] node_connections = 2`
- 상태: **1차(15 run, 버그 발견) · 2차 팬리스(12 run) · 3차 능동냉각(12 run)** 완료
- 원본: [`../../results/policy-ab-20260821/`](../results/policy-ab-20260821) (1차) ·
  [`../../results/policy-ab-20260821b/`](../results/policy-ab-20260821b) (2차 팬리스) ·
  [`../../results/policy-ab-20260821fan/`](../results/policy-ab-20260821fan) (3차 능동냉각)
- 선행: [`S0_SUSTAINED_LOAD.md`](#experiments-s0-sustained-load)

---

## 1. Research Question (원래 의도)

> S0-A 는 팬리스에서 king 이 2.4배 느려졌는데 RR 이 여전히 1/3 을 보내는
> 것을 봤다. **부하 인지 정책(`least-queue`/`ect`)이 그 손실을 회수하는가?**

## 2. 1차 Results — 정책이 붕괴한다

팬리스, 15분 예열 후, 3노드 / 커넥션 2·node / c36, 정책당 5 run.
오류율 **전 구간 0**.

| Policy | Throughput | p50 | p95 | p99 | king% | jack% | queen% |
|---|---:|---:|---:|---:|---:|---:|---:|
| **round-robin** | **379.9 ± 13.5** | 85.5 | 165.4 | 213.8 | 33.3 | 33.3 | 33.3 |
| **least-queue** | **169.8 ± 2.3** | 199.8 | 390.0 | 477.1 | 35.1 | 33.3 | 31.6 |
| **ect** | **158.5 ± 2.1** | 219.6 | 397.9 | 483.0 | 34.8 | 36.3 | 28.9 |

**부하 인지 정책이 처리량을 55~58% 떨어뜨린다.** 예상과 반대 방향이고,
"조금 나쁘다" 가 아니라 **절반 이하**다.

노드별 CPU busy% (`/proc/stat` 델타):

| Policy | king | jack | queen |
|---|---:|---:|---:|
| round-robin | 51.5% | 44.5% | 45.3% |
| least-queue | 22.6% | 21.1% | 20.7% |
| ect | 21.0% | 21.7% | 17.7% |

**노드는 오히려 놀고 있다.** 처리량이 반토막인데 CPU 사용률도 반토막이다.

## 3. 원인 — 하트비트 stale 상태로 인한 herding

단계 분해가 범인을 좁힌다(p50, ms):

| Policy | scheduler_route | scheduler_queue | node_queue | inference | **end_to_end** |
|---|---:|---:|---:|---:|---:|
| round-robin | **0.004** | 0.000 | 0.023 | 30.27 | **75.8** |
| least-queue | **0.003** | 0.000 | 0.023 | 33.69 | **192.4** |
| ect | **0.004** | 0.000 | 0.023 | 33.97 | **212.5** |

- **정책 선택은 느리지 않다** — route 가 셋 다 0.004 ms.
- **노드 큐도 아니다** — node_queue 0.023 ms 로 동일.
- **추론도 아니다** — 30~34 ms 로 비슷.
- 늘어난 것은 **왕복 전체**다(75.8 → 212.5).

즉 요청이 **노드에 도착하기 전에** 어딘가에서 기다린다.

### 3.1 정책이 보는 상태는 하트비트로만 갱신된다

```rust
// registry.rs:110-121  — 정책에 넘어가는 스냅샷
queue_depth: self.health.queue_depth,
in_flight:   self.health.in_flight,
```

`self.health` 는 **하트비트 수신 시에만** 통째로 교체된다
(`on_health_success`: `self.health = health;`).

그리고 **디스패치 경로는 이 상태를 전혀 갱신하지 않는다** —
`service.rs` 전체에 `in_flight`·`queue_depth` 참조가 **하나도 없다.**

하트비트 주기는 노드 **1000 ms**, 스케줄러 기대 2000 ms.

### 3.2 그래서 결정이 결정적으로 쏠린다

```text
RR 기준 처리량 380 inf/s
하트비트 1회(1초) 사이에 약 380건이 디스패치된다
그 380건은 전부 같은 (고정된) 스냅샷을 본다

LeastQueue.choose() = min_by(queue_depth, in_flight, ewma_inference)
스냅샷이 고정이면 이 함수는 결정적이다
  → 380건이 전부 같은 노드를 고른다
  → 다음 하트비트에 그 노드의 queue_depth 가 치솟음
  → 그 다음 1초는 전부 다른 노드로
```

**전형적인 stale load information 하의 herd behavior** 다. 노드당 커넥션이
2개뿐이라 한 노드로 몰린 수백 건이 그 2개 커넥션에서 밀린다 — 그래서
**노드는 놀고(CPU 20%) 왕복만 길어진다(212 ms).** node_queue 가 0 인 것도
같은 이유다. 요청은 워커 풀이 아니라 **전송 계층**에 쌓여 있다.

집계 분배가 ~33% 로 보이는 것은 **60초 동안 쏠림이 번갈아 일어나 평균이
난 것**이지 고르게 나간 것이 아니다.

round-robin 은 상태를 안 보고 구조적으로 분산하므로 이 문제가 없다.

## 4. 판정

> **`least-queue` 와 `ect` 는 이 부하에서 사용 가능한 상태가 아니다.**
> 정책 품질 문제가 아니라 **상태 신선도(state freshness) 설계 결함**이다.
> 초당 수백 건을 초당 1회 갱신되는 상태로 라우팅하면 herding 이 일어난다.

이것은 **S0-C 가 의도한 질문의 답이 아니다.** 정책이 열 불균질을 회수하는지
는 여전히 **미확정**이다 — 구현이 그것을 시험할 상태가 아니었다.

## 5. ⚠️ 이 실험의 결함 두 가지

### 5.1 열 조건이 유지되지 않았다 (치명적)

RR 결과가 라운드에 따라 크게 다르다.

| round | RR 처리량 | king p50 |
|---:|---:|---:|
| 1 | **355.7** | **144.7** ← 예열 직후, 뜨거움 |
| 2 | 385.1 | 88.7 |
| 3 | 387.2 | 89.7 |
| 4 | 386.1 | 86.5 |
| 5 | 385.2 | 89.5 |

**저부하 정책(LQ/ECT)이 도는 동안 보드가 식었다.** CPU busy 가 45~51% →
17~22% 로 떨어지니 발열이 줄고, 이어지는 RR run 은 시원한 상태에서 돈다.
라운드 2부터 king p50 이 144.7 → 87 로 정상화됐다 — **S0-A 가 만든 열
불균질 조건이 사라졌다.**

즉 이 실험은 "팬리스 열 불균질 상태에서의 정책 비교" 가 **아니게 됐다.**

> **다행히 정책 붕괴 자체는 온도와 무관하다.** LQ/ECT 는 5 라운드 내내
> 166~172 / 155~160 으로 일정했다(뜨겁든 식었든). RR 만 온도를 따라
> 355 → 385 로 움직였다. §3 의 결론은 이 결함에 영향받지 않는다.

### 5.2 온도 수집이 실패했다

`max_soc_c` 열이 전부 비어 있다. ssh 안에서 awk 프로그램을 **큰따옴표**로
감싸 원격 셸이 `$1` 을 위치 인자로 먹었다.

```bash
# 틀림 — 원격 셸이 $1 을 빈 문자열로 치환
ssh "$h" 'awk "{print $1/1000}" /sys/.../temp'
# 맞음 — awk 프로그램은 작은따옴표로
ssh "$h" "awk '{print \$1/1000}' /sys/.../temp"
```

`thermal-logger.sh` 는 작은따옴표를 써서 정상이었다. 인라인으로 다시 쓸 때
같은 함정을 밟았다.

## 6. 다음

1. **정책 상태 신선도 수정** — 디스패치 시점에 `in_flight` 를 증감시키면
   (스케줄러가 자기가 보낸 요청 수를 안다) 하트비트 없이도 상태가 최신이
   된다. 수십 줄 규모다.
2. **수정 후 S0-C 재실행** — 그래야 원래 질문에 답할 수 있다.
3. 재실행 시 §5.1 을 피하려면 **정책 사이에 열 조건을 다시 맞춰야 한다**
   (각 정책 앞에 RR 로 재가열, 또는 정책마다 별도 세션).

## 7. Conclusion

**부하 인지 정책을 켜자 처리량이 55~58% 무너졌다.** 원인은 정책 품질이
아니라 **상태 신선도**다 — 정책이 보는 `queue_depth`/`in_flight` 가 하트비트
(1초)로만 갱신되고 디스패치 경로가 이를 전혀 갱신하지 않아, 초당 수백 건이
동일한 고정 스냅샷을 보고 **전부 같은 노드를 고른다**(herding).

노드는 놀고(CPU 20%) 요청은 전송 계층에 쌓인다(node_queue 0, 왕복 212 ms).

**원래 질문 — 부하 인지 정책이 열 불균질 손실을 회수하는가 — 는 미확정으로
남는다.** 구현이 그것을 시험할 상태가 아니었고, 실험 도중 열 조건도 사라졌다
(§5.1). 다만 **이 결함을 먼저 찾은 것이 더 중요하다** — 고치지 않았다면
"부하 인지 정책은 도움이 안 된다" 는 정반대 결론을 낼 뻔했다.
---

# 2차 (수정 후) — 정책이 실제로 적응한다

수정 내용은 커밋 `ece4eba` — `local_in_flight` 원자적 예약, `Reservation`
RAII 가드, `select_and_reserve()` 임계구역, 정책의 1차 신호 교체.
98 tests pass.

## 8. Method 변경점

1차의 치명적 결함(§5.1)을 고쳤다. **정책마다 RR 로 3 run 재예열**해
**시작** thermal state 를 맞춘다.

> 맞추는 것은 **시작** 조건이다. 정책 실행 중 온도가 갈라지는 것은 그대로
> 둔다 — adaptive scheduler 가 king 의 부하를 줄여 king 이 식는다면 그것
> 자체가 정책 효과의 일부다.

실제로 통제됐다 — 전 run 시작 soc **81~82°C**, run 중 최대 78.5~79.5°C.

## 9. Results (12 run, 정책당 4회)

오류율 **전 구간 0**.

| Policy | Throughput | p50 | **p95** | **p99** | king% | jack% | queen% |
|---|---:|---:|---:|---:|---:|---:|---:|
| round-robin | 373.9 ± 4.5 | 87.1 | 170.7 | 232.0 | 33.3 | 33.3 | 33.3 |
| least-queue | 380.9 ± 2.1 | 92.2 | **127.1** | **146.9** | 32.9 | 33.3 | 33.7 |
| **ect** | **384.2 ± 0.8** | 90.6 | 130.9 | 156.4 | 32.8 | 33.3 | 33.9 |

**붕괴가 사라졌다.** 169.8 / 158.5 → **380.9 / 384.2**.

### 9.1 tail 이 크게 좋아진다

| | RR | least-queue | ect |
|---|---:|---:|---:|
| p95 | 170.7 | **127.1** (−25.5%) | 130.9 (−23.3%) |
| p99 | 232.0 | **146.9** (−36.7%) | 156.4 (−32.6%) |

### 9.2 노드별 지연이 평준화된다

| Policy | king | jack | queen | 최대/최소 |
|---|---:|---:|---:|---:|
| round-robin | 103.2 | 85.4 | 77.6 | **1.33×** |
| least-queue | 93.3 | 92.0 | 91.1 | **1.02×** |
| **ect** | 90.4 | 90.7 | 90.5 | **1.00×** |

### 9.3 CPU 사용률도 평준화된다

| Policy | king | jack | queen | 편차 |
|---|---:|---:|---:|---:|
| round-robin | 53.9% | 47.4% | 43.6% | **10.3%p** |
| least-queue | 54.4% | 52.5% | 50.4% | 4.0%p |
| ect | 53.3% | 51.9% | 50.2% | 3.1%p |

"놀고 있다" 를 지연이 아니라 utilization 으로 확인한 것이다. RR 에서는
queen 이 43.6% 로 놀고 king 이 53.9% 로 혼자 밀린다. 정책을 켜면 세 노드가
50~54% 로 모인다.

## 10. 판정 — 규칙은 안 걸렸다. 그 사실을 그대로 적는다

측정 전에 정한 규칙: **king 분배 3%p 이상 이동 = '이동'**.

```text
least-queue  king 33.3% → 32.9%   (−0.4%p)   규칙 미달
ect          king 33.3% → 32.8%   (−0.5%p)   규칙 미달
```

**임계값을 내리지 않는다.** 대신 규칙이 안 걸린 이유를 본다.

### 10.1 분배가 거의 안 움직였는데 지연·CPU 는 평준화됐다

이 둘이 모순이 아니다. **least-outstanding 은 개수를 옮기는 정책이 아니라
동시 점유를 조절하는 폐루프**다. 느린 노드에 **동시에 걸어 두는 요청 수**를
줄이면, 그 노드의 큐 대기가 줄어 지연이 내려간다. 60초 누적 처리 **건수**는
지연이 평준화된 만큼 다시 비슷해진다.

즉 이번 조건에서는 **0.5%p 이동만으로 평준화가 달성됐다.**

### 10.2 이번 열 불균질이 S0-A 보다 훨씬 약했다

| | S0-A | S0-C 2차 |
|---|---|---|
| soc 최대 | 86~88°C | **78~79°C** |
| RR 노드 p50 | 156.9 / 64.7 / 66.0 | 103.2 / 85.4 / 77.6 |
| 편차 | **2.4×** | **1.33×** |

3%p 임계는 S0-A 의 **2.4배 편차**를 상정하고 정한 값이다. 1.33배 조건에서는
필요한 이동량 자체가 작다.

원인은 하네스다 — 정책 사이에 스케줄러 재기동과 probe bench 가 들어가
평균 부하가 S0-A(30분 연속)보다 낮았다.

> ⚠️ **정정 (4차, §18.4).** 위 표의 "78~79°C" 는 **계기 오류**다. run
> 종료 후 순간값을 읽어 run 간 냉각 골짜기에 떨어진 값이며, 1초 열
> 로거로 다시 집계하면 **2차도 86.8°C 로 S0-A 와 같다.** 따라서 "평균
> 부하가 낮아서 덜 뜨거웠다" 는 이 문단의 인과는 성립하지 않는다.
> 2차가 1.33배에 그친 실제 원인은 **CPU 강등이 덜 갈린 것**이다
> (클럭 편차 1.50× vs S0-A 1.79×). 온도는 처음부터 같았다.

## 11. Conclusion (2차)

**상태 신선도 수정으로 부하 인지 정책이 정상 동작한다.** 55~58% 붕괴가
사라지고 RR 대비 처리량 +1.9%(LQ) / **+2.7%(ECT)**.

**가장 큰 이득은 tail 이다** — p99 **−37%**(232.0 → 146.9). 노드별 지연
편차가 1.33× → **1.00×**, CPU 사용률 편차가 10.3%p → 3.1%p 로 평준화된다.
**정책이 열 불균질을 실제로 흡수하고 있다.**

ECT 가 LQ 보다 처리량이 약간 높고(384.2 vs 380.9) 지연 평준화가 완전하다
(1.00× vs 1.02×). 서비스 속도를 점수에 반영하는 설계와 방향이 맞다. 다만
차이는 0.9% 로 작아 **이 조건에서 ECT 우위를 단정하기는 이르다.**

### 남은 것

- **강한 열 불균질(2.4배)에서의 회수량은 여전히 미측정.** 이번 조건은
  1.33배였다. S0-A 수준을 재현하려면 정책 사이 저부하 구간 없이 연속
  가열이 필요하다.
- 분배 이동이 작아 3%p 규칙이 안 걸렸다. 규칙은 그대로 두되, 다음 실험
  에서는 **short-window(1초) 분배**를 봐야 순간 이동량을 알 수 있다
  (bench 가 per-request 덤프를 아직 지원하지 않는다).

---

# 3차 (능동 냉각, 동질 조건) — 기본값 결정을 위한 sanity test

## 12. 왜 필요했나

2차는 **팬리스(열 이질)** 조건이다. 거기서 증명된 것은 정확히
"노드 성능이 불균질할 때 fresh state 를 쓰는 adaptive scheduling 이 RR 보다
유리하다" 이지, **정상 동질 클러스터에서도 항상 최선**까지는 아니다.

능동 냉각에서는 세 보드가 거의 같은 속도로 움직인다. 그 조건에서 adaptive
정책이 **regression 을 내지 않는지**를 확인해야 기본값을 정할 수 있다.

3N / 커넥션 2·node / c36 / **능동 냉각**, 정책당 4 run. soc 47~54°C.

## 13. Results — regression 없음, tail 은 오히려 개선

| Policy | Throughput | p50 | **p95** | **p99** | king / jack / queen p50 |
|---|---:|---:|---:|---:|---|
| round-robin | 389.9 ± 1.6 | 86.3 | 146.1 ± 1.8 | 185.6 ± 5.4 | 86.3 / 86.1 / 87.0 (1.01×) |
| **least-queue** | **389.9 ± 2.0** | 89.1 | **129.3 ± 0.9** | **151.0 ± 1.5** | 89.2 / 89.1 / 89.1 (1.00×) |
| ect | 388.6 ± 1.4 | 88.5 | 136.3 ± 0.4 | 163.2 ± 1.9 | 88.2 / 88.5 / 88.8 (1.01×) |

CPU busy 는 세 정책 모두 45% 로 동일. 분배도 33.2~33.5% 로 안 움직인다 —
**동질 조건에서는 안 움직이는 것이 정상**이다.

- **처리량 regression 없음.** LQ −0.0%, ECT −0.3%. 판정 밴드(±2%) 안이다.
- **tail 은 동질 조건에서도 좋아진다.** p99 185.6 → **151.0**(LQ, −18.6%)
  / 163.2(ECT, −12.1%).
- p50 은 소폭 는다(86.3 → 89.1 / 88.5, +3%). 중앙값을 조금 내주고 tail 을
  크게 얻는 교환이다.

## 14. 두 조건을 나란히 놓으면

| | 팬리스 (이질) | 능동 냉각 (동질) |
|---|---|---|
| **RR** | 373.9±4.5 · p95 170.7**±19.9** · p99 232.0**±34.7** | 389.9±1.6 · p95 146.1±1.8 · p99 185.6±5.4 |
| **LQ** | 380.9±2.1 · p95 **127.1±0.5** · p99 **146.9±1.0** | 389.9±2.0 · p95 **129.3±0.9** · p99 **151.0±1.5** |
| **ECT** | **384.2±0.8** · p95 130.9±0.0 · p99 156.4±0.5 | 388.6±1.4 · p95 136.3±0.4 · p99 163.2±1.9 |

세 가지가 보인다.

**① RR 은 이질 조건에서 tail 이 불안정해진다.** p95 SD **19.9**, p99 SD
**34.7** — adaptive 정책의 SD 가 ~1 인 것과 대비된다. adaptive 정책의 이득은
"tail 이 낮다" 만이 아니라 **"tail 이 예측 가능하다"** 이기도 하다.

**② LQ 가 두 조건 모두에서 tail 이 가장 낮다.** SD 가 0.4~1.9 로 작아
차이가 실재한다. ECT 대비 p99 가 팬리스 −6.1%, 냉각 −7.5%.

**③ ECT 의 처리량 우위는 이질 조건에서만 나타난다.** 팬리스 +0.9%(384.2 vs
380.9), 냉각에서는 −0.3%로 뒤집힌다.

## 15. 기본값 판단

**RR 은 기본값 후보에서 빠진다.** 두 조건 모두에서 tail 이 가장 나쁘고,
이질 조건에서는 예측 가능성까지 무너진다.

**LQ 와 ECT 는 어느 쪽도 지배적이지 않다.**

| | LQ | ECT |
|---|---|---|
| tail (양 조건) | **더 낮다** (p99 −6~8%) | |
| 처리량 (이질) | | **+0.9%** |
| 처리량 (동질) | **+0.3%** | |
| 설계 근거 | 개수 기반 | **서비스 속도 반영** — 이질이 심해질수록 유리할 소지 |

> **현재 저장소 기본값은 `ect`** 다(`policy.rs` `default()`, 테스트로 고정).
> 두 정책 모두 regression 이 없고 RR 보다 낫다는 것은 확정됐으므로,
> **기본값을 유지할 근거는 충분하다.** LQ 로 바꿀지는 "p99 6~8% 를
> 서비스 속도 인지와 바꿀 것인가" 의 판단이며, 강한 이질(2.4배) 조건에서
> 다시 재기 전에는 결정을 미룰 수 있다.

## 16. Limitations

- 강한 이질(S0-A 의 2.4배)에서의 LQ vs ECT 비교는 여전히 없다. 2차는
  1.33배였다.
- 정책 3종 × 4 run 씩이다. p50/p95 차이는 SD 가 작아 신뢰할 만하나
  처리량 0.3~0.9% 차이는 **우열 근거로 쓰기에 약하다.**
- short-window 분배 미관측(60초 aggregate 만).

---

# 4차 (강한 이질) — LQ vs ECT 기본값 결정

## 17. 사전 등록 — 판정 규칙 (결과가 나오기 전에 적는다)

> 이 절은 **측정이 도는 중에** 작성했다. `results/policy-ab-*` 가 아직
> 비어 있는 시점이다. 규칙을 결과에 맞춰 옮기지 않기 위한 장치다(§10 의
> 3%p 규칙이 안 걸렸을 때와 같은 태도).

### 17.1 조건

| | 값 |
|---|---|
| 냉각 | 팬리스 (측정 직전 물리적으로 OFF) |
| 하네스 | `run-policy-ab.sh 4 25 5` — 4라운드, 예열 25분, 재예열 5run |
| 정책 | round-robin / least-queue / ect, 라운드마다 순서 회전 |
| 그 외 | 3노드 · 커넥션 2/node · c36 · 60초/run — 2·3차와 동일 |

**2·3차 대비 하네스 변경 2건.** 둘 다 §10.2 가 지목한 "정책 사이 저부하
구간" 을 줄이기 위한 것이다.

1. `verify_nodes` 의 probe 를 `c12` → `c36`. 노드 수 검증은 부하와 무관
   하므로, 측정 직전 10초를 냉각이 아니라 가열에 쓴다. 스케줄러 재기동
   무부하 14초와 합쳐 **저부하 구간이 24초 → 14초**로 줄었다.
2. `LOG_DUR` 이 `REHEAT_RUNS` 를 반영하도록 수정. 이번 파라미터에서
   기존 식은 2,880초로 계산되어 **본측정 도중 열 로거가 죽었을** 값이다
   (실제 소요 ~6,400초).

### 17.2 게이트 — 강한 이질이 재현됐는가

**재현 실패 시 LQ vs ECT 를 판정하지 않는다.** 2차가 답을 못 낸 이유가
조건 미달이었으므로, 같은 실패를 "결과" 로 포장하지 않기 위한 게이트다.

| 지표 | 기준 | 근거 |
|---|---|---|
| **RR 라운드의 노드별 p50 최대/최소** | **≥ 2.0×** | S0-A 2.4×, 2차 1.33×. 그 사이에 선을 긋는다 |
| soc 최대 | ≥ 85°C (보조) | S0-A 86~88°C, 2차 78~79°C |

이질 게이지로 **RR 을 쓴다** — RR 은 적응하지 않으므로 균등 부하 아래
드러나는 raw capacity 편차를 그대로 보여준다. S0-A 의 2.4× 도 같은 정의다.

- 2.0× 미달 → **판정 보류.** 조건 미달로 기록하고 기본값은 손대지 않는다.
- 1.33× 근처 → 하네스 수정으로도 부족하다는 뜻. 연속 가열 설계를 다시 본다.

### 17.3 LQ vs ECT 판정 밴드

n=4/정책이라 작은 차이는 못 쓴다(§16). **밴드를 넘어야 "이겼다" 로 센다.**

| 축 | 승리 기준 | 근거 |
|---|---|---|
| 처리량 | 상대차 **≥ 2%** | §13 의 regression 밴드(±2%)와 같은 값 |
| tail (p99) | 상대차 **≥ 5%** | 2·3차 관측 6~8%, SD ~1~2. 5%면 구분된다 |

보조 지표(판정에는 안 쓰고 해석에만 쓴다): 노드별 p50 최대/최소 평준화
비율, 노드별 CPU busy 편차, king 분배 이동폭.

### 17.4 결정 매트릭스

`ect` 가 **현직**이다(`policy.rs` `default()`). 현직을 끌어내리려면
적극적 근거가 필요하다 — 이 tie-break 도 사전에 정한 규칙이다.

| 처리량 (ECT−LQ) | tail (LQ 유리) | 결정 |
|---|---|---|
| ECT ≥ +2% | < 5% | **`ect` 유지** — 질문 닫힘 |
| < 2% | LQ ≥ 5% | **`least-queue` 로 변경** — 질문 닫힘 |
| ECT ≥ +2% | LQ ≥ 5% | **지배 없음.** `ect` 유지하되 트레이드오프를 명시하고 질문은 **열어 둔다** |
| < 2% | < 5% | **구분 불가.** `ect` 유지, 질문 **닫힘** — "기본값 선택이 중요하지 않다" 는 것도 답이다 |

마지막 행이 핵심이다. 강한 이질에서까지 구분이 안 되면, 더 센 조건을
찾아다니는 대신 **이 질문을 닫고** S3.9b 로 넘어간다.

### 17.5 측정 중 발견 — soc 게이트의 계기가 틀렸다

예열 중 하네스가 `soc: 81 80 80` 을 찍어 게이트 미달로 보였다. 실제로는
**조건이 재현되고 있었다.** 같은 시각 1초 열 로거 기준:

| | 하네스 출력 | 열 로거 (최근 3분) | S0-A |
|---|---|---|---|
| king | 81 | max **86.8** · avg 85.8 · min 78.5 | 85.9~86.8 |
| queen | 80 | max **85.9** · avg 85.6 · min 78.5 | 85.0~85.9 |
| jack | 80 | max **86.8** · avg 85.8 · min 80.4 | 85.0~85.9 |
| CPU 최저 | — | **1008~1200 MHz** | 816~1800 MHz |

원인은 샘플링 시점이다. 하네스는 **60초 run 이 끝난 뒤** ssh 3번을
순차로 돌며 읽는다. RK3576 은 부하가 끊기면 수 초 만에 식으므로 그
값은 run 간 골짜기(min 78.5~80.4)에 떨어진다 — 열 로거의 min 과
정확히 일치한다. CSV 의 `max_soc_c` 열도 같은 방식이라 **이름과 달리
최대값이 아니다.**

> **게이트 기준(85°C)은 그대로 둔다. 바꾸는 것은 데이터 출처뿐이다.**
> 결과에 맞춰 임계를 옮기는 것과, 임계를 재는 계기가 다른 물리량을
> 재고 있었음을 고치는 것은 다르다. soc 보조 게이트는 **1초 열 로거의
> run 중 최대**로 판정한다. 주 게이트(RR 노드 p50 최대/최소)는 bench
> JSON 에서 오므로 영향이 없다.

부수 효과: S0-C 2차의 "78~79°C" 도 같은 계기로 얻은 값이다. **2차의
실제 부하 중 온도는 더 높았을 수 있다** — 2차가 1.33배에 그친 이유를
"덜 뜨거워서" 로 돌린 §10.2 의 설명은 재검토 대상이다. 2차 열 로그가
남아 있으므로 확인 가능하다.

## 18. Results (4차) — **게이트 미달. LQ vs ECT 를 판정하지 않는다.**

- 원본: [`../../results/policy-ab-20260821-contaminated/`](../results/policy-ab-20260821-contaminated)
  — **r1 round-robin 과 열 로그만 유효하다.** 나머지는 하네스 충돌로 무효
  (해당 디렉터리 `README.md`, [S0-D](#experiments-s0-d-capacity-hetero) §4)
- 예열 25분 완료 후 r1 만 측정하고 **의도적으로 중단**했다(§18.3).

### 18.1 주 게이트: 1.10× (기준 2.0×)

```text
r1 round-robin   384.8 inf/s   p50 88.8  p95 140.2  p99 173.9  err 0
   분배      king 33.3 / jack 33.3 / queen 33.3
   노드 p50  king 93.3  jack 88.4  queen 85.1   ->  최대/최소 1.10x
```

2차(1.33×)보다도 낮다. **강한 이질은 재현되지 않았다.**

### 18.2 그런데 열 조건은 완벽히 재현됐다

1초 열 로거를 S0-A 와 같은 방식으로 집계했다(§17.5 의 계기 수정 적용).

| | soc_max | soc_avg | **CPU p50** | CPU min | 노드 p50 편차 |
|---|---:|---:|---:|---:|---:|
| **S0-A** king | 86.8 | 84.3 | **1008** | 816 | |
| S0-A queen | 85.9 | 83.5 | **1800** | 1416 | |
| S0-A jack | 86.8 | 83.9 | **1800** | 1200 | **2.4×** |
| **4차** king | 86.8 | 84.3 | **1416** | 1008 | |
| 4차 queen | 85.9 | 83.8 | **1608** | 1200 | |
| 4차 jack | 86.8 | 84.3 | **1608** | 1008 | **1.10×** |

**soc 는 소수점까지 같다.** 갈린 것은 CPU 클럭 분포뿐이다.

| | 클럭 편차 (p50 최대/최소) | 지연 편차 |
|---|---:|---:|
| S0-A | **1.79×** | **2.4×** |
| 4차 | **1.14×** | **1.10×** |

### 18.3 그래서 무엇이 틀렸나 — 전제

핸드오프와 §10.2 는 2차가 1.33배에 그친 원인을 **"덜 뜨거워서"** 로 보고
연속 가열을 처방했다. 이번 측정이 그 전제를 부정한다.

> **열 조건은 이질의 필요조건이지 충분조건이 아니다.**
> 충분조건은 **강등이 보드마다 갈리는 것**이고, 열 제어는 온도를 목표로
> 하지 편차를 목표로 하지 않는다. 세 보드가 같은 온도에서 **같이** 내려
> 가면 이질은 생기지 않는다.

S0-A 의 1.79× 갈림은 실리콘·기류·위치 편차의 산물이며 **냉각 조건으로
불러낼 수 있는 것이 아니다.** 예열을 더 해도 갈라지지 않는다 — 이미 세
보드 모두 열 제어에 걸려 있었다.

이 판단이 서자 나머지 11 run(약 1시간 20분)은 음성 결과의 n 을 4로
올리는 것 외에 사는 게 없어 중단했다. **판정을 미달로 남기고 설계를
바꾸는 편이 낫다.**

### 18.4 §10.2 의 "78~79°C" 는 계기 오류였다 — **정정**

2차 열 로그를 다시 집계했다(`results/policy-ab-20260821b`). **2차도
86.8°C 였다.** 78~79°C 는 §17.5 의 순간값 계기가 만든 허상이다.

| 실험 | soc_max (1초 로거) | 문서에 적혔던 값 | CPU p50 | 클럭 편차 | 지연 편차 |
|---|---|---|---:|---:|---:|
| S0-A | 86.8 / 85.9 / 86.8 | 85.9~86.8 ✓ | 1008 / 1800 / 1800 | **1.79×** | **2.4×** |
| **S0-C 2차** | **86.8 / 85.9 / 86.8** | **78~79 ✗** | 1200 / 1800 / 1800 | **1.50×** | **1.33×** |
| S0-C 4차 | 86.8 / 85.9 / 86.8 | — | 1416 / 1608 / 1608 | **1.14×** | **1.10×** |

**세 실험의 열 조건이 전부 동일하다.** §10.2 의 설명("2차는 덜 뜨거웠다")
은 근거를 잃는다. 실제 원인은 **강등이 덜 갈린 것**이다.

그리고 세 점이 단조를 이룬다.

```text
클럭 편차  1.14x -> 1.50x -> 1.79x
지연 편차  1.10x -> 1.33x -> 2.40x
```

이질의 크기를 결정하는 것은 온도가 아니라 **클럭 편차**다. 클럭을 직접
잡으면 이질을 원하는 값에 놓을 수 있다(§19).

> 참고로 1차(`results/policy-ab-20260821`, 2 run)는 CPU p50 가 세 보드
> 모두 2208 MHz, soc 평균 77.8°C 였다 — 강등 자체가 없었다. 열 조건이
> 유지되지 않았다는 §5.1 의 진단은 그대로 맞다.

## 19. 다음 — 이질을 결정론적으로 만든다

열이 편차를 만들어 주기를 기다리는 대신 **열 제어가 쓰는 손잡이를 직접
잡는다.** 강등은 `scaling_max_freq` 를 끌어내리는 방식으로 구현돼 있고
(측정 중 king 이 `1008000` 으로 관측됐다), 같은 파일을 우리가 쓸 수 있다.

```text
팬 ON (열 균질·저온)  +  king 1008 MHz / queen 1800 / jack 1800
   = S0-A 의 CPU p50 프로파일을 그대로 복제
```

이점:

1. **재현 가능**하다. 실리콘 운에 기대지 않는다.
2. **열이 변수에서 빠진다.** 팬 ON 이면 캡이 그대로 유지되고(열 제어가
   더 내릴 이유가 없다) 드리프트도 없다. 정책 비교의 교란이 사라진다.
3. **편차를 쓸어볼 수 있다.** 1.0× / 1.3× / 1.8× / 2.4× 로 스윕하면
   "2.4배에서 어느 쪽이 낫나" 보다 훨씬 강한 결론이 나온다 —
   **"편차가 커질수록 ECT 가 유리해지는가"**. ECT 를 기본값으로 둔
   설계 근거(서비스 속도 반영)가 바로 이 가설이므로, 이 스윕이 그
   근거를 직접 시험한다.

대가: 열 유래가 아니므로 **thermal heterogeneity 라고 부를 수 없다.**
capacity heterogeneity 로 따로 적는다. 열 유래 이질에서 정책이 작동한다는
인과는 2차에서 이미 닫혔고(§11), 지금 남은 질문은 **편차 크기에 대한
정책의 반응**이므로 이 치환이 타당하다.

---

## Figure

![RR 의 p99 SD 가 이질 조건에서만 폭발한다(±34.7 vs ±1)](../results/policy-ab-20260821b/figures/fig_policy_tail.png)

**`fig_policy_tail.png`** — RR 의 p99 SD 가 이질 조건에서만 폭발한다(±34.7 vs ±1)

재생성: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s0-d-capacity-hetero"></a>

# S0-D — Capacity Heterogeneity (결정론적 이질)

- 실험 ID: **S0-D**
- 착수일: 2026-08-21
- 상태: **1단계 교정 완료** (12 run, 오류 0). 2단계 정책 A/B 남음
- 선행: [`S0_C_POLICY_AB.md`](#experiments-s0-c-policy-ab) §18~19 · [`S0_SUSTAINED_LOAD.md`](#experiments-s0-sustained-load)

---

## 1. Research Question

> **노드간 capacity 편차가 커질수록 ECT 가 LQ 보다 유리해지는가?**

S0-C 는 "편차가 있을 때 adaptive 가 RR 보다 낫다" 까지 닫았다(§11).
남은 것은 **LQ 와 ECT 중 무엇을 기본값으로 둘 것인가** 이고, 두
정책은 두 조건(이질 1.33× / 동질)에서 어느 쪽도 지배하지 못했다.

ECT 를 기본값으로 둔 설계 근거는 **서비스 속도를 점수에 반영한다** 는
것이다. 그렇다면 **편차가 커질수록 ECT 가 유리해져야 한다.** 이 실험은
그 가설을 직접 시험한다. "2.4배에서 어느 쪽이 나은가" 보다 강한 질문이다.

## 2. 왜 열이 아니라 클럭을 조작하는가

S0-C 4차에서 팬리스 연속 가열로 강한 이질을 재현하려다 실패했다(§18).

```text
세 실험 모두 soc 86.8 / 85.9 / 86.8°C — 열 조건은 동일
갈린 것은 CPU 강등의 보드간 편차뿐

  클럭 편차  1.14x -> 1.50x -> 1.79x
  지연 편차  1.10x -> 1.33x -> 2.40x     (S0-C 4차 / S0-C 2차 / S0-A)
```

**열 조건은 이질의 필요조건이지 충분조건이 아니다.** 충분조건은 강등이
보드마다 갈리는 것인데, 열 제어는 온도를 목표로 하지 편차를 목표로
하지 않는다. 세 보드가 같은 온도에서 같이 내려가면 이질은 생기지 않고,
그 갈림은 실리콘·기류·위치의 산물이라 **냉각으로 불러낼 수 없다.**

그래서 **열 제어가 쓰는 손잡이를 직접 잡는다.** 강등은
`scaling_max_freq` 를 내리는 방식으로 구현돼 있다(측정 중 king 이
`1008000` 으로 관측됐다). 같은 파일을 우리가 쓴다.

| | 열 유래 (S0-A/C) | **클럭 캡 (S0-D)** |
|---|---|---|
| 재현성 | 실리콘 운에 의존 | **결정론적** |
| 열 교란 | 정책 비교에 섞인다 | **팬 ON — 변수에서 뺀다** |
| 편차 지정 | 불가 | **1.3× / 1.8× / 2.4× 로 지정** |

대가: 열 유래가 아니므로 **thermal heterogeneity 라고 부를 수 없다.**
capacity heterogeneity 다. 열 유래 이질에서 정책이 작동한다는 인과는
S0-C 2차에서 이미 닫혔고, 지금 질문은 **편차 크기에 대한 정책의 반응**
이므로 이 치환이 타당하다.

## 3. Method — 1단계 교정

`scripts/run-capacity-calibration.sh`

- **팬 ON**, 보드 유휴 42~47°C. 열 제어가 캡보다 낮게 내릴 이유가 없다.
- 정책 **round-robin 고정** — 적응하지 않으므로 균등 부하 아래 raw
  capacity 편차가 그대로 드러난다. S0-A 의 2.4× 도 같은 정의다.
- king 의 CPU 캡을 사다리로: **2208 / 1608 / 1200 / 1008 / 816 / 600 MHz**
  (`policy0`·`policy4` 양쪽, 각 그룹 상한으로 clamp)
- 캡마다 c36 · 60초 × 2회. 3노드 · 커넥션 2/node — 운영점 그대로.
- run 마다 `scaling_cur_freq` 를 되읽어 **캡이 부하 중에도 유지되는지**
  확인한다. EXIT 트랩으로 중단돼도 king 을 강등된 채 남기지 않는다.

**S0-A 의 클럭을 그대로 복제하지 않는 이유**: 열 로거가 `cpu4` 만
기록해 리틀 코어 값을 모른다. 클럭을 맞추는 대신 **관측량(노드 p50
편차)에 직접 맞추는** 편이 정직하다. 교정이 캡 → 편차 대응표를 준다.

## 4. 사고 기록 — 첫 시도는 하네스 충돌로 폐기 (2026-08-21)

첫 교정 시도의 **무캡 기준선이 197.4 inf/s** 로 나왔다(정상 391.2).
노드 편차 2.73×, 다음 run 은 **오류율 82.4%**. "king·jack 만 느리다" 로
보여 **클러스터 고장으로 오진하기 직전이었다.**

원인은 고장이 아니었다. **정책 A/B 하네스가 죽지 않고 살아 있었다.**

```text
믿은 것   TaskStop 으로 정책 A/B 하네스를 중단했다
실제      래퍼 셸만 죽고 자식 bash 는 계속 돌았다
결과      두 하네스가 같은 3노드를 각각 c36 으로 때렸다 (합 72)
```

살아남은 하네스는 자기 설정(`scheduler-s0c.toml`)으로 **스케줄러를 계속
재기동했다.** 그래서 내가 "기본 설정으로 복구" 한 것이 몇 초 뒤 덮여
있었고, 그 사실이 보이지 않았다.

**관측이 거짓말을 했다.** git-bash 에서 `ps -ef` 에 안 보였고 `pkill -f`
도 못 잡았다. PowerShell `Get-CimInstance Win32_Process` 로만 보였다.

```powershell
Get-CimInstance Win32_Process -Filter "Name='bash.exe'" |
  ? { $_.CommandLine -match 'scripts/run-' } | select ProcessId,CommandLine
```

정리 후 재측정: **391.2 inf/s · p50 86.2 · 오류 0 · 편차 1.02×.**
클러스터는 처음부터 멀쩡했다.

### 4.1 재발 방지

`npuforge_assert_cluster_free`(`scripts/lib/remote.sh`)를 추가하고 정책
A/B·capacity 교정 하네스 시작점에 배선했다. **서버에 `npuforge-bench`
가 돌고 있으면 시작하지 않고 큰 소리로 멈춘다.**

로컬 프로세스 확인이 아니라 **공유 자원 쪽에서 확인**하는 것이 핵심이다
— 로컬 관측은 플랫폼에 따라 거짓말을 하지만, 서버에서 도는 벤치는
거짓말을 하지 않는다.

### 4.2 오염된 데이터

- **첫 교정 시도 — 폐기.** 같은 조건에서 재측정했다(§5).
- **4차 정책 A/B 진행분 — `results/policy-ab-20260821-contaminated/` 로
  이름을 바꿔 보존한다.** 삭제하지 않는 이유는 이 사고 자체가 방법론
  기록이기 때문이다(README §4.11). 해당 디렉터리 `README.md` 에 유효·무효
  구간을 명시했다 — **유효한 것은 r1 round-robin 과 1초 열 로그뿐**이고,
  S0-C §18 의 게이트 판정은 그 둘에만 근거하므로 영향받지 않는다.

> ⚠️ 4차 하네스가 **같은 날짜 경로를 재사용해 S0-C 1차 데이터(15 run)를
> 덮어썼다.** `git checkout` 으로 복원했다. 하네스의 출력 경로가
> `results/policy-ab-<날짜>` 라 하루에 두 번 돌리면 덮어쓴다 —
> `NPUFORGE_SUFFIX` 를 쓰거나 기존 디렉터리가 있으면 멈춰야 한다.

## 5. Results — 교정 (12 run, 오류율 0)

- 원본: [`../../results/capacity-calib-20260821/`](../results/capacity-calib-20260821)
- 팬 ON, 보드 48~55°C. **열 제어는 전 구간 개입하지 않았다** — run 마다
  되읽은 `scaling_cur_freq` 가 지정한 캡과 항상 일치했다.

| king 캡 (MHz) | throughput | king p50 | jack p50 | queen p50 | **편차** |
|---:|---:|---:|---:|---:|---:|
| 2208 (무캡) | 388.1 | 83.8 | 86.6 | 89.4 | **1.12×** |
| 1608 | 382.9 | 96.3 | 83.3 | 81.6 | **1.18×** |
| 1200 | 379.6 | 103.6 | 83.3 | 77.7 | **1.33×** |
| 1008 | 369.0 | 127.7 | 72.4 | 72.7 | **1.79×** |
| **816** | **359.6** | **149.8** | **67.9** | **66.3** | **2.26×** |
| 600 | 318.4 | 213.5 | 54.4 | 54.5 | **3.93×** |

편차 재현성은 캡마다 2 run 이 ±0.05 이내로 붙었다(예: 816 → 2.30 / 2.22).

### 5.1 캡 816 이 S0-A 를 거의 그대로 재현한다

| | king p50 | jack p50 | queen p50 | 편차 | throughput |
|---|---:|---:|---:|---:|---:|
| **S0-A** (열 유래, 팬리스 86°C) | 156.9 | 64.7 | 66.0 | **2.4×** | 345.4 |
| **캡 816** (클럭, 팬 ON 50°C) | 150.9 | 67.3 | 65.6 | **2.30×** | 359.8 |

세 노드 지연이 모두 **6ms 이내로 겹친다.** S0-A 에서 king 의 CPU
**최저값이 816 MHz** 였다는 점과도 맞는다 — 열 강등이 밀어붙인 하한을
우리가 직접 지정한 셈이다.

> **강한 이질을 결정론적으로 만들 수 있다.** S0-C §17.2 의 게이트(2.0×)를
> 넘는 조건이 이제 재현 가능하고, 30분 예열도 실리콘 운도 필요 없다.

### 5.2 부수 관측 — RR 에서 느린 노드가 빠른 노드를 놀린다

캡을 내릴수록 **king 은 느려지는데 jack·queen 은 오히려 빨라진다.**

```text
king  캡 2208  83.8ms  ->  캡 600  213.5ms   (2.5배 느려짐)
jack  캡 2208  86.6ms  ->  캡 600   54.4ms   (1.6배 빨라짐)
queen 캡 2208  89.4ms  ->  캡 600   54.5ms   (1.6배 빨라짐)
```

c36 고정에 RR 이면 클라이언트 슬롯 36개가 세 노드에 균등 분배된다.
king 이 느려지면 **더 많은 슬롯이 king 을 기다리며 묶이고**, 그만큼
jack·queen 에 동시에 떠 있는 요청이 줄어 둘은 저부하가 된다.
p50 54ms 는 놀고 있다는 뜻이다.

즉 캡 600 의 처리량 손실 **−18%**(388.1 → 318.4)는 king 의 능력 저하
그 자체가 아니라 **RR 이 놀고 있는 두 노드를 못 쓰는 몫**을 포함한다.
adaptive 정책이 회수할 수 있는 상한이 여기 있다. S0-A 에서 관측된
"king 이 2.4배 느린데 요청은 정확히 1/3" 과 같은 현상이며, 이번엔
**편차를 지정해 그 크기를 조절할 수 있다.**

## 6. 2단계 정책 A/B — **Future Work (지금 하지 않는다)**

교정이 대응표를 주었으므로 언제든 돌릴 수 있다. 다만 **지금 우선순위가
아니다** — ECT 와 LQ 의 우열은 NPUForge 의 핵심 결론을 바꾸지 않기
때문이다(§7). 본선은 S3.9b 다.

돌릴 때의 설계:

```bash
# 캡 1200 / 1008 / 816 / 600  =  편차 1.33 / 1.79 / 2.26 / 3.93x
# 정책 3종 x 4 편차 x 3 run, 팬 ON 이라 예열 불필요 — 약 40분
```

- 판정 밴드는 S0-C §17.3 을 그대로 쓴다 (처리량 2%, p99 5%).
- 가설: **편차가 커질수록 ECT 의 처리량 우위가 커진다.** 커지지 않으면
  ECT 의 설계 근거(서비스 속도 반영)가 실측으로 반박된다.
- 편차를 **연속 변수로** 다루므로 "2.4배에서 어느 쪽" 보다 강한 결론이
  나온다 — 우위가 편차에 대해 단조 증가하는지를 본다.

## 7. 이 계보의 현재 결론

교정까지의 결과를 정책 계보 전체와 합치면 이렇다.

1. **RR 은 이질성에 취약하다.** 느린 노드에도 1/3 을 계속 보내고,
   이질 조건에서 tail 의 예측 가능성까지 무너진다(p99 SD 34.7 vs ~1).
2. **fresh-state adaptive scheduling 이 RR 의 tail 을 크게 개선한다.**
   p99 −37%, 노드 지연 편차 1.33× → 1.00× (S0-C §9).
3. **LQ 와 ECT 는 둘 다 정상 동작한다.** 두 조건 모두 regression 없음.
4. **강한 이질에서 ECT 가 우위인지는 미확정이다.**
5. **그러나 그 우열은 NPUForge 의 핵심 결론을 바꾸지 않는다.** 핵심은
   "상태 신선도를 고친 부하 인지 스케줄링이 이질을 흡수한다" 이고,
   그것은 LQ·ECT 어느 쪽으로도 성립한다. 기본값은 `ect` 를 유지한다.
6. **S0-D 가 남긴 것은 답이 아니라 fixture 다** — 그 질문을 언제든
   **재현 가능하게** 시험할 수 있는 장치.

---

## Figure

![캡 → 편차 대응. 816 MHz 가 S0-A(2.4×)를 재현한다](../results/capacity-calib-20260821/figures/fig_capacity_calibration.png)

**`fig_capacity_calibration.png`** — 캡 → 편차 대응. 816 MHz 가 S0-A(2.4×)를 재현한다

재생성: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s0-sustained-load"></a>

# S0 — Sustained Load (조건 A 팬리스 / 조건 B 능동 냉각)

- 실험 ID: **S0-A · S0-B**
- 측정일: 2026-08-21
- 코드: `bb3f7ab` + `[transport] node_connections = 2`
- 상태: **둘 다 완료** (각 30 run × 60초 ≈ 31분 연속)
- 원본: [`../../results/sustained-20260821-fan/`](../results/sustained-20260821-fan) ·
  [`../../results/sustained-20260821-fanless/`](../results/sustained-20260821-fanless)
- 선행: [`S3_8_OPTIMIZED_SCALEOUT.md`](#experiments-s3-8-optimized-scaleout)

---

## 1. Research Question

> **short-run operating point 가 sustained 부하에서도 유지되는가?
> 그리고 그 답이 냉각 조건에 얼마나 의존하는가?**

지금까지의 **모든** 측정이 60초 이하 = throttling 발현 전 구간이었다.

```text
short-run operating point    60초 이하 benchmark 기준
sustained operating point    thermal steady-state 기준
```

## 2. Method

- 운영점 그대로: **3노드, 노드당 커넥션 2개, c36**(= 노드당 c12).
- **60초 run × 30회 연속**, 노드·스케줄러 **재기동 없음**.
- 세 보드 `thermal-logger.sh` **1초 간격** — 온도 4종, CPU MHz, NPU MHz, 전압.
- run 마다 **응답 노드 수와 NPU 최대 온도**를 기록. 팬리스에서 노드가 임계
  (degraded 80 / disable 90°C)에 걸려 제외되면 처리량 하락이 throttling 이
  아니라 **노드 수 감소** 때문이다. 둘을 구분해야 한다.
- 판정 규칙은 **측정 전에** 정했다: `steady = 마지막 1/3 평균`,
  `degradation = 1 − steady/peak`. <3% 없음 / 3~10% 경미 / >10% 뚜렷.
- 두 조건의 **유휴 시작 온도가 비슷**하다(팬 40.7~41.6°C, 팬리스 38.8~40.7°C).
  유휴에서는 팬 효과가 작아 시작점이 맞춰진 공정한 A/B 다.

## 3. Results

오류율 **양쪽 전 구간 0**. **노드 제외 0건** (팬리스도 90°C 임계에 닿지 않았다).

| | **B: 능동 냉각** | **A: 팬리스** |
|---|---:|---:|
| peak | 387.7 | 389.4 |
| **steady (뒤 1/3)** | **380.3 ± 2.2** | **345.4 ± 3.8** |
| **degradation** | **1.9%** | **11.3%** |
| soc 최대 | 58.2 ~ 61.0°C | **85.9 ~ 86.8°C** |
| npu 최대 | 59.2 ~ 61.0°C | **86.8 ~ 87.8°C** |
| **CPU 최저** | **2208 MHz (강등 0회)** | **816 / 1200 / 1416 MHz** |
| NPU 최저 | 950 MHz | **950 MHz (강등 없음)** |
| 노드 제외 | 0 | 0 |

시간 추이:

| t+분 | B 처리량 | A 처리량 | A vs peak |
|---:|---:|---:|---:|
| 1 | 387.7 | 389.4 | 100.0% |
| 5 | 385.8 | 380.9 | 97.8% |
| 10 | 382.5 | 359.7 | 92.4% |
| 15 | 381.7 | 356.2 | 91.5% |
| 20 | 380.2 | 355.8 | 91.4% |
| 25 | 382.3 | 342.1 | 87.9% |
| 30 | 377.3 | 343.5 | 88.2% |

## 4. Interpretation

### 4.1 냉각이 운영점을 지키고 있었다

능동 냉각에서는 **1.9%** — 판정 규칙상 "열화 없음". 클럭 강등이 보드당
1,660여 샘플 전부에서 **0회**다. 온도는 5분 내 58~61°C 평탄역에 들고
임계치까지 20°C 이상 여유가 있다.

**팬을 빼면 11.3%.** 두 운영점이 갈라진다.

```text
short-run operating point                   3N 387~389 inf/s
sustained operating point (능동 냉각)       3N 380.3      (−1.9%)
sustained operating point (팬리스)          3N 345.4      (−11.3%)
```

> **"throttling 이 있다/없다" 는 조건과 함께 써야 한다.** 같은 하드웨어,
> 같은 운영점, 같은 부하인데 냉각 하나로 결론이 바뀐다.

### 4.2 NPU 는 한 번도 강등되지 않았다 — 강등된 것은 CPU 다

**양쪽 조건 모두 NPU 950 MHz 고정.** 팬리스에서 NPU 온도가 87.8°C 까지
올라도 클럭은 안 떨어졌다.

강등된 것은 CPU 다. 그리고 **보드마다 다르다.**

| 보드 | CPU 최저 | soc 최대 |
|---|---:|---:|
| **king** | **816 MHz** (−63%) | 86.8°C |
| jack | 1200 MHz (−46%) | 86.8°C |
| queen | 1416 MHz (−36%) | 85.9°C |

worklog 가 "throttling 을 NPU 클럭만으로 판정했다" 를 네 번째 실수로 기록한
바로 그 지점이다(discuss §3.1). **이번 측정이 그 교훈을 재확인한다.**

### 4.3 진짜 발견 — round-robin 이 강등된 노드를 그대로 때린다

팬리스 마지막 5 run 의 **노드별** 지연이다.

| | p50 | p95 | **분배** |
|---|---:|---:|---:|
| jack | 64.7 | 107.0 | **33.3%** |
| **king** | **156.9** | **313.9** | **33.3%** |
| queen | 66.0 | 107.4 | **33.3%** |

**king 이 다른 두 노드보다 2.4배 느린데 요청은 정확히 1/3 씩 간다.**
round-robin 은 부하도 상태도 보지 않기 때문이다.

능동 냉각에서는 세 노드가 85.2~90.3 ms 로 고르다. 팬리스에서만 갈라진다.

그리고 queen·jack 은 팬리스에서 **오히려 빨라졌다**(85~90 → 65~66 ms).
전체 처리량이 떨어져 노드당 부하가 줄었기 때문이다.

> 지연이 크게 낮아진 것은 **queue pressure 가 줄었다는 강한 신호**이지만,
> "놀고 있다" 로 단정하려면 노드별 **CPU idle 또는 outstanding queue depth**
> 가 필요하다. 이번 측정에는 없다 — S0-C 에서 함께 남긴다.

> ⚠️ **여기까지가 관측이고, 아래는 가설이다.**
>
> **확인된 것**
> - 열 편차가 있다 (CPU 816 / 1200 / 1416 MHz)
> - king 의 service capacity 가 실제로 낮다 (p50 2.4배)
> - RR 이 느려진 king 에도 33.3% 를 계속 보낸다
>
> **아직 아닌 것**
> - "팬리스 손실 = 열 편차 × 부하 무인지 정책" — 정책을 바꿨을 때 손실이
>   **실제로 회수돼야** 마지막 인과고리가 닫힌다.
>
> 저장소에 `least-queue` 와 `ect` 가 구현돼 있으나 실장비 검증이 없다.
> **S0-C 가 이 고리를 닫는다**(§8).
>
> 반대 결과도 중요하다 — 정책을 켜도 분배가 1/3 로 유지되거나 성능이
> 그대로라면, **현재 정책의 상태 신호가 thermal-induced capacity
> degradation 을 감지하지 못한다**는 뜻이다.

### 4.4 원래 −27% 와의 관계

| | 원래(discuss §12) | 이번 S0-A |
|---|---|---|
| 부하 | 로컬 8스레드 (CPU 포화) | 클러스터 (CPU 여유) |
| 냉각 | 팬리스 | 팬리스 |
| NPU 온도 | 90.4°C | 87.8°C |
| CPU 강등 | 2208 → **816 MHz** | 2208 → **816 MHz** (king) |
| 결과 | **−27%** | **−11.3%** |

**CPU 는 똑같이 816 MHz 까지 떨어졌다.** 그런데 손실은 절반 이하다.
클러스터 운전은 보드 CPU 가 49~63% 유휴라(S3.5, S3.7c) 강등의 영향을 덜
받고, 세 보드 중 한 대만 최악으로 떨어졌기 때문이다.

→ **−27% 는 틀리지 않았다. 조건이 다를 뿐이다.**

## 5. Limitations

- **부하 인지 정책 미측정**(§4.3). round-robin 만 썼다. `least-queue`/`ect`
  로 팬리스 손실이 얼마나 회수되는지는 **가설이며 검증 대상**이다.
- 31분이다. 온도가 평탄역에 들었으므로 더 길어도 크게 다르지 않을 것으로
  보이나 **추정**이다.
- 실내 온도를 통제하지 않았다. 두 조건은 같은 날 연속 측정이다.
- run 사이 2~4초 공백(§2).
- 3노드 운영점 하나만. 1N/2N 은 미측정.
- 팬리스에서도 90°C 임계에 닿지 않아 **노드 제외 동작은 검증되지 않았다.**

## 6. Reproduction

```bash
bash scripts/run-sustained-load.sh 30 fan       # 조건 B
bash scripts/run-sustained-load.sh 30 fanless   # 조건 A (팬 제거 후)
PYTHONIOENCODING=utf-8 python scripts/analyze-sustained.py \
    results/sustained-20260821-fanless
```

## 7. Conclusion

**능동 냉각에서는 short-run 운영점이 sustained 에서도 유지된다**
(degradation **1.9%**, 클럭 강등 0회). S2~S3.9a 의 60초 결과가 지속 운전에
그대로 적용된다.

**팬을 빼면 11.3% 로 벌어진다.** 강등된 것은 NPU 가 아니라 **CPU** 이고
(950 MHz 고정 vs 2208 → 816 MHz), 그 정도가 보드마다 다르다.

가장 값진 발견은 §4.3 이다 — **king 이 2.4배 느려졌는데 round-robin 은
여전히 1/3 을 보낸다.** RR 입장에서 세 노드는 동일하지만 실제 service
capacity 는 이미 동일하지 않다.

**이것이 adaptive scheduling 의 시험장이다.** 부하 인지 정책의 실장비
검증이 이로써 기능 항목에서 **성능 항목으로 승격**된다 → **S0-C**.

다만 "손실 = 열 편차 × 정책" 은 **아직 가설**이다(§4.3 주). 정책을 바꿔
손실이 회수되는 것을 봐야 인과가 닫힌다.

## 8. 다음 — S0-C (팬을 켜기 전에 한다)

**능동 냉각에서는 세 노드가 거의 동질적이라 정책 차이가 사라질 가능성이
크다.** 지금 팬리스 상태가 정책을 검증하기 가장 좋은 조건이다. 식히기 전에
인과 검증까지 닫는다.

| Policy | Throughput | p95 | p99 | king share | jack share | queen share |
|---|---:|---:|---:|---:|---:|---:|
| round-robin | 345.4 | ? | ? | 33.3% | 33.3% | 33.3% |
| least-queue | ? | ? | ? | ? | ? | ? |
| ect | ? | ? | ? | ? | ? | ? |

보고 싶은 것은 단순한 처리량 상승이 아니다. 예를 들어 ECT 가
`king 15% / jack 42% / queen 43%` 정도로 이동하면서 345 → 370~380 에
가까워지면 이렇게 말할 수 있다.

> **Thermal heterogeneity reduces node capacity, and state-aware scheduling
> recovers performance by adapting load allocation to heterogeneous service
> rates.**

설계상 지켜야 할 것:
- **먼저 충분히 가열해 thermal steady-state 에 든 뒤** 비교한다. 정책마다
  시작 온도가 다르면 정책 효과와 thermal drift 가 섞인다.
- 정책 순서를 회전시킨다.
- 처리량·p95·p99 외에 **노드별 분배와 노드별 지연**, 그리고 **노드별 CPU
  idle** 을 함께 남긴다.

---

## Figure

![31분 연속 — 능동 냉각 −1.9% vs 팬리스 −11.3%](../results/sustained-20260821-fanless/figures/fig_sustained_thermal.png)

**`fig_sustained_thermal.png`** — 31분 연속 — 능동 냉각 −1.9% vs 팬리스 −11.3%

재생성: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s2-grpc-baseline"></a>

# S2 — gRPC Multi-node Scaling Baseline

- 실험 ID: **S2**
- 측정일: 2026-08-20
- 동결 commit: `254d560` (측정 중 코드·설정 무변경)
- 상태: **완료 · 재현 확인 (30 runs)**
- 원본 데이터: [`../../results/baseline-20260820/raw/`](../results/baseline-20260820/raw) · 그래프: [`figures/`](../results/baseline-20260820/figures) · 대시보드: [`dashboard.html`](../results/baseline-20260820/dashboard.html)

---

## 1. Research Question

> **Does aggregate inference throughput increase approximately linearly as
> identical low-cost NPU nodes are added to an Ethernet-connected edge cluster?**

저비용 엣지 NPU(RK3576, 6 TOPS)를 이더넷으로 묶었을 때, **노드를 늘리면 전체
추론 처리량이 선형에 가깝게 증가하는가.** 명목 TOPS 합산이 아니라 실측
확장 효율을 묻는다.

## 2. Hypothesis

데이터 병렬 구조([`adrs/001`](../adrs/001-data-parallel-only.md))에서 노드는
서로 독립적으로 서로 다른 요청을 처리한다. 노드 간 통신이 추론 경로에 없으므로,
**단일 중앙 스케줄러가 병목이 되지 않는 한 처리량은 노드 수에 선형**일 것으로
예측한다. 동시에, 클러스터 경유(gRPC + 네트워크)는 로컬 직접 추론보다 노드당
처리량을 **일정 비율 낮출** 것으로 본다(오버헤드).

## 3. System Under Test

| 항목 | 값 |
|---|---|
| Board | NanoPi R76S ×3 (king / queen / jack) |
| SoC / NPU | Rockchip RK3576 / 2-core 6 TOPS |
| Model | YOLOv8n **INT8** (sha256 `dba155d2…`), `want_float=0` |
| Input | raw RGB 640×640×3 = 1,228,800 byte/request |
| Scheduler host | server (.9): Xeon E5-2630L ×2 (24T) / 16GB / Rocky 9.4 |
| Network | worker 2.5GbE / aggregation 10GbE (NEXI NS-S25G10G-N) |
| Transport | **gRPC** (tonic + protobuf) |
| Topology | client → scheduler(.9) → node, 3-hop 전부 gRPC |

토폴로지·근거: [`adrs/014`](../adrs/014-10g-aggregation-separate-scheduler.md),
[`docs/infrastructure.md`](#infrastructure).

## 4. Experimental Controls

모든 run 에서 고정한 조건.

```text
Cooling      : Active cooling — 120mm 5V USB fan per node (측정 시작부터)
CPU governor : performance
Policy       : round-robin
Worker count : 8 / node  (스레드마다 전용 RKNN 컨텍스트, adrs/007)
Transport    : gRPC
Model        : YOLOv8n INT8, want_float=0
Warmup       : 제외
```

- **냉각은 Active Cooling(팬 ON).** 팬리스가 아니다 —
  [`docs/board-worklog.md`](#board-worklog) §2.24·§2.27 참조.
- 측정 전 `preflight-check.sh` 통과(별칭↔hostname, 해시, governor, 온도, 전압, NTP).

## 5. Measurement Method

- 부하 도구: `npuforge-bench` (**closed-loop**), server(.9)에서 실행.
- 노드당 동일 부하: **concurrency = 8 × 노드수** (1N c8 / 2N c16 / 3N c24).
- 각 조건 **10 runs, 60초**. 총 30 runs.
- **조건 순서를 rotate** 해 시간·온도 변동을 한 조건에 몰지 않는다:
  ```text
  Round 1: 1N → 2N → 3N
  Round 2: 2N → 3N → 1N
  Round 3: 3N → 1N → 2N   (반복)
  ```
- 노드 축소는 프로세스 중지, run 사이 cooldown.
- 스크립트: [`scripts/run-grpc-baseline30.sh`](../scripts/run-grpc-baseline30.sh).
  측정 30회 동안 코드·설정 동결.

> closed-loop 특성상 절대 지연은 SLA 로 인용하지 않고 **구성 간 비교**에만
> 쓴다([`adrs/028`](../adrs/028-bench-run-validity.md)).

## 6. Validation / Integrity Checks

30 runs 전수 검사. **측정 신뢰성의 근거다.**

| 검사 | 결과 |
|---|---|
| run 수 | 30 / 30 |
| active node 판정 | **30/30 정확** (n1=1, n2=2, n3=3) |
| invalid run (verdict) | 0 |
| 오류율 (inference) | **0.00%** (전 run) |
| 재시도 | 0 건 |
| load-balance 편차 | **0.00 %p** |

- active node 는 등록 노드가 아니라 **실제 요청을 처리한 노드**(`per_node`)로
  판정한다. 노드를 중지해도 스케줄러 등록이 남는 문제를 bench 수정으로 해결했다
  (board-worklog §2.28).
- 재시도 카운트는 응답 프로토콜의 `attempts` 필드에서 온다(스케줄러 실제 시도).

## 7. Results

### 7.1 Throughput

| Nodes | Throughput Mean ± SD |
|---:|---:|
| 1 | **112.9 ± 0.5** inf/s |
| 2 | **229.0 ± 0.9** inf/s |
| 3 | **338.4 ± 1.1** inf/s |

SD 가 0.5~1.1 로 극히 작다 — 30회에 걸쳐 처리량이 사실상 흔들리지 않았다.
첫 측정값 337.7 이 338.4 ± 1.1 로 재현됐다. → [fig1](../results/baseline-20260820/figures/fig1_throughput_vs_node.png)

### 7.2 Speedup

| 기준 | 2N | 3N |
|---|---:|---:|
| 1-node c8 (112.9) | 2.03× | **3.00×** |
| single-node saturation (~115) | 1.99× | 2.94× |

### 7.3 Scaling Efficiency

1-node c8 기준 **100% / 101% / 100%**, saturation 기준 3N ≈ **98%**.
→ [fig2](../results/baseline-20260820/figures/fig2_scaling_efficiency.png)

### 7.4 Latency (round-trip, closed-loop)

**run-level percentile 의 30회 평균**(§7.4.1 주의 참조):

| Nodes | p50 | p95 | p99 |
|---:|---:|---:|---:|
| 1 | 68.0 | 100.8 | 116.3 ms |
| 2 | 67.0 | 100.1 | 118.6 ms |
| 3 | 67.6 | 102.7 | 123.9 ms |

노드 수가 늘어도 지연 분포가 거의 평탄하다 — 확장이 지연을 악화시키지 않는다.

#### 7.4.1 주의 — 이 값은 pooled percentile 이 아니다

각 run 안에서 그 run 의 요청 전체로 percentile 을 계산하고(nearest-rank,
`stats.rs`), **그 run-level 값들을 다시 평균**한 것이다. 30회 요청을 전부
합쳐 다시 정렬해 구한 값(pooled percentile)과는 다르다.

```text
쓴 것    mean( p99(run1), p99(run2), ..., p99(run30) )
아닌 것  p99( run1 ∪ run2 ∪ ... ∪ run30 )
```

일반적으로 **run-level 평균은 pooled 보다 tail 을 낮게 보이게 한다** — 각
run 의 최악 구간이 평균에 희석되기 때문이다. 구성 간 *비교* 에는 문제가
없지만(모든 조건이 같은 방식) **절대값을 "이 시스템의 p99" 로 인용하면 안 된다.**

pooled 를 내려면 per-request 지연 원본이 필요한데 bench 는 요약 percentile 만
JSON 에 남긴다. 원본 덤프 옵션 추가는 `TODO.md` §1.2 에 올려 두었다.
→ [fig4](../results/baseline-20260820/figures/fig4_latency_percentiles.png)

### 7.5 Load Distribution

round-robin 이 3노드를 **정확히 33.3%씩** 나눴다(편차 0.00 %p).
→ [fig5](../results/baseline-20260820/figures/fig5_per_node_distribution.png)

## 8. Timing Breakdown

응답 `Timing`(proto) 11단계, 30회 p50 평균 (ms):

| 단계 | 1N | 3N |
|---|---:|---:|
| scheduler_queue | 0.00 | 0.00 |
| scheduler_route | 0.00 | 0.00 |
| **network_to_node** (input) | 17.72 | 17.11 |
| node_queue | 0.02 | 0.02 |
| **inference (NPU)** | 24.70 | 22.49 |
| **network_to_client** (output) | 17.72 | 17.11 |
| **end_to_end** | 61.54 | 58.83 |

```text
non-inference overhead = end_to_end - inference = 58.83 - 22.49 = 36.34 ms
payload transfer       = network_to_node + network_to_client = 34.21 ms
```

- scheduler_queue/route 는 노드 수와 무관하게 **~0** — 단일 스케줄러가 3노드
  동시에도 병목이 아니다([`adrs/003`](../adrs/003-central-simple-scheduler.md) 실측 확인).
- 1N·3N 의 `network_to_node`(17.72 vs 17.11)가 거의 같다 — 단일 요청 전송
  시간은 노드 수와 무관.
- → [fig7](../results/baseline-20260820/figures/fig7_timing_breakdown.png)

## 9. Local vs Cluster Overhead

| Mode | Cooling | Worker | Throughput |
|---|---|---:|---:|
| Local direct RKNN (no gRPC) | Active Cooling | 8 | 161.5 inf/s |
| Cluster gRPC (single node) | Active Cooling | 8 | 112.9 inf/s |

**Throughput loss = (161.5 − 112.9) / 161.5 = 30.1%.**
로컬 baseline 은 냉각·worker 를 클러스터와 맞춰 재측정했다(board-worklog §2.27).
→ [fig8](../results/baseline-20260820/figures/fig8_local_vs_cluster.png)

> ⛔ **두 측정량을 곱하지 않는다.** throughput loss(30.1%, 처리량)와 latency
> breakdown(94%, 지연 구성비)은 서로 다른 축이다. §10 의 문장을 쓴다.

## 10. Interpretation

**Finding 1 — near-linear scaling (재현됨).**

> Three-node throughput reached **3.00×** the one-node c8 baseline and **~98%**
> of the single-node saturation-derived ideal. All 30 runs completed without
> inference errors or retries, with effectively uniform round-robin distribution.

**Finding 2 — node-level overhead is payload transfer.**

> Local direct inference reached **161.5 inf/s** while single-node cluster
> throughput reached **112.9 inf/s**, a **30.1% throughput reduction**.
> Separately, latency decomposition showed that **94% of non-inference latency
> was observed in the payload-transfer path** — not in serialization, scheduler
> queueing, or node queueing (all ~0).

이 둘은 서로를 강화한다. 확장은 스케줄러·네트워크가 노드 수에 병목이 아니라서
선형이고(Finding 1), 노드당 절대 상한은 페이로드를 2.5G 로 나르는 시간에
깎인다(Finding 2). 최적화가 겨냥할 지점은 compute 가 아니라 **transport** 다.

## 11. Limitations

- **측정 시간이 짧다(60/30초).** CPU throttling 은 300초에 -27% 로 나타나므로
  (board-worklog §2.24), 이 결과는 **throttling 전 구간**이다. 지속 부하
  처리량은 별도 실험(S0)에서 확정한다.
- **냉각 축.** 오늘은 Active Cooling 만. 팬리스(조건 A) 클러스터 측정은 없다.
- **saturation 미확정.** 1N 은 c8/c16/c32 로 ~115 근처를 봤으나 c48 미측정,
  2N·3N 의 ceiling 은 sweep 하지 않았다 → **S3**.
- **직렬화 단독 미측정.** proto `Timing` 에 gRPC 직렬화 단독 필드가 없다.
  현재 non-inference 잔차(~2ms)에 포함. 계측점 추가가 필요.
- **closed-loop.** 절대 지연 아님, 구성 간 비교 전용.
- **단일 2노드 조합(king+queen).** king+jack 등 다른 조합은 미측정.

## 12. Reproduction

```bash
# 3노드 클러스터 기동 후 (스케줄러 + king/queen/jack)
bash scripts/run-grpc-baseline30.sh        # 30 run → server:/tmp/baseline30
# 로컬 팬 baseline (Finding 2):
ssh npuforge-k 'pkill -9 npuforge-node; sleep 3; cd ~/npuforge-rknn-test; \
  ./sustained_load_test yolov8n-int8.rknn 60 8'
# 그래프 재생성:
python scripts/make-figures.py
```

동결 commit: `254d560`. 조건 고정표는 §4.

## 13. Raw Data

- bench JSON 30건: [`../../results/baseline-20260820/raw/`](../results/baseline-20260820/raw)
  (`n{노드}_r{라운드}.json`, 각 파일에 throughput·latency·node_inference·
  TimingBreakdown·per_node·nodes_before/after(temp·voltage)·verdict·run_id)
- 집계 리포트: [`../../results/baseline-20260820/README.md`](../results/baseline-20260820/README.md)
- 그래프·대시보드: [`figures/`](../results/baseline-20260820/figures), [`dashboard.html`](../results/baseline-20260820/dashboard.html)

## 14. Conclusion

RK3576 3-node NPU cluster 는 30회 반복 실험에서 **near-linear scaling
(338.4 ± 1.1 inf/s, 3.00×, error 0%)** 을 보였다. 노드당 오버헤드는 compute
나 scheduler 가 아니라 **payload-transfer path**(non-inference latency 의 94%)
에 있음을 TimingBreakdown 으로 확인했다.

→ gRPC baseline 을 **동결**한다. 다음: **S3**(saturation / scaling limit) →
**S4**(io_uring). S4 는 이 baseline 과 **동일 조건**에서 transport 비용을
비교한다.

---

<a id="experiments-s3-5-transport-profile"></a>

# S3.5 — Transport Cost Profiling

- 실험 ID: **S3.5** (+ **S3.5b** RPS A/B)
- 측정일: 2026-08-20
- 동결 commit: `01f29a2`. 노드·스케줄러·모델·bench **무변경**
- 상태: **완료**
- 원본: [`../../results/transport-profile-20260820/raw/`](../results/transport-profile-20260820/raw) ·
  [`../../results/rps-ab-20260820/`](../results/rps-ab-20260820)
- 선행: [`S2_GRPC_BASELINE.md`](#experiments-s2-grpc-baseline), [`S3_SATURATION.md`](#experiments-s3-saturation)
- 후속: **S3.6** (H2 / channel A/B — 이 문서가 남긴 ①②③을 가른다, §7)

---

## 1. Research Question

> **노드당 상한 ~115 inf/s (로컬 direct ~160 대비 −30%) 를 실제로 무엇이
> 누르고 있는가?**

S2 는 이 손실이 **payload-transfer path** 에 있다는 것까지 밝혔다(non-inference
latency 의 94%). 하지만 그 경로 안에서 *무엇이* 비용인지는 열려 있었다.
후보는 최소 넷이다 — 링크 대역폭, 보드 CPU 총량, 커널 네트워크 스택,
그리고 전송 계층 구조.

**S4(io_uring)를 시작하기 전에 이 질문이 먼저 닫혀야 한다.** io_uring 은
syscall 과 복사 비용을 줄이는 도구다. 병목이 거기가 아니면 큰 구현을 하고도
아무것도 못 얻는다. `01-TECHSPEC.md` §15.1 이 정한 순서(2.CPU profile →
3.syscall·복사 비용 → 4.버퍼 풀 → 5.io_uring)에서 2~4 가 비어 있었다.

또한 §15.4 가 요구하는 지표(syscalls/req, ctx switches/req, cycles/req)는
S4 의 **before** 기준값으로 어차피 필요한데 저장소에 하나도 없었다
(S2 raw 30건에 CPU 항목 없음). 먼저 만들고 나서 재면 개선을 무엇에 귀속시킬
근거가 없다.

## 2. Method

같은 보드(king)에서 세 조건을 잰다. 냉각·governor·모델은 S2·S3 와 동일.

| 조건 | 부하 | 의미 |
|---|---|---|
| `idle` | 없음 | 계측기 자체의 바닥값 |
| `cluster` | 1노드 클러스터 c32 | S3 ceiling 조건 |
| `local` | 로컬 direct 8스레드 | 네트워크 경로가 통째로 빠진 조건 |

`cluster` 와 `local` 의 차이가 곧 transport 가 보드에서 쓰는 비용이다.

- 부하 80초, 그 안쪽 **t+20 부터 45초**만 수집. 램프와 warmup 을 제외한다.
- 보드에서는 `/proc` 원본만 떠 오고 계산은 개발 PC 에서 한다. 나중에 다른
  각도로 다시 볼 수 있어야 한다.
- 수집: `mpstat -P ALL`(코어별), `pidstat -t`(스레드별), `/proc/PID/io`
  (syscr·syscw), `/proc/PID/task/*/status`(ctx switch), `/proc/net/dev`,
  `/proc/interrupts`, `/proc/softirqs`.
- 스크립트: [`run-transport-profile.sh`](../scripts/run-transport-profile.sh),
  [`node-profile-collect.sh`](../scripts/node-profile-collect.sh),
  [`analyze-transport-profile.py`](../scripts/analyze-transport-profile.py).

> `perf` 는 보드에 없다(커널 6.1.141 vendor, apt 는 6.8 용만 제공).
> cycles/req 는 PMU 값이 아니라 코어별 busy 시간 × 고정 클럭(A53 2016 /
> A72 2208 MHz, governor=performance)으로 환산한 **근사값**이다.

## 3. Results

수집 창 45.1초, king, 팬, performance.

| | idle | cluster | local |
|---|---:|---:|---:|
| throughput (inf/s) | 0 | **116.6** | **159.1** |
| **%idle (8코어 전체)** | 99.9 | **63.1** | 82.9 |
| %usr / %sys / %soft | 0.0 / 0.0 / 0.0 | 18.3 / 12.2 / 6.4 | 9.7 / 7.3 / 0.0 |
| **CPU0 busy** | 0.3 | **69.7** | 21.5 |
| **CPU0 %soft** | 0.0 | **51.5** | 0.0 |
| eth0 RX / TX (Gbps) | 0 | **1.196 / 1.194** | 0 |
| **링크 실측(2.34) 대비** | — | **51.1% / 51.0%** | — |
| RX 패킷/s | 9 | 112,008 | 8 |
| NET_RX softirq/s | 10 | 10,954 | 8 |

코어별 busy%:

```text
cluster :  c0=70  c1=38  c2=37  c3=37  c4=30  c5=29  c6=27  c7=27
local   :  c0=21  c1=19  c2=19  c3=19  c4=15  c5=15  c6=15  c7=15
```

### 요청당 비용 (TECHSPEC §15.4 — S4 의 before 기준값)

| | cluster | local | 차이 |
|---|---:|---:|---:|
| **syscalls/req** | **84.5** | ~0.0 | +84.5 |
| ├ read/req | 0.1 | 0.0 | |
| └ write/req | **84.4** | 0.0 | |
| ctx switch/req (vol) | 157.6 | 221.6 | −64.0 |
| ctx switch/req (nonvol) | 0.7 | 0.1 | |
| 프로세스 CPU-ms/req | **22.2** | 9.0 | **+13.2** |
| 보드 전체 CPU-ms/req | **25.3** | 8.6 | **+16.7** |
| ≈ Mcycles/req | 52.9 | 18.1 | +34.8 |
| RX 패킷/req | 960.7 | 0 | |

transport 는 추론 1건당 보드 CPU 를 **약 2.9배** 쓰게 만든다(8.6 → 25.3 ms).
write syscall 이 요청당 **84.4회** — 응답 1,218,000 byte 를 약 14.4 KB 씩
쪼개 내보내고 있다(HTTP/2 프레임 크기와 일치).

## 4. 병목 후보를 하나씩 배제한다

### 4.1 링크 대역폭 — 아니다

2.5GbE 는 full-duplex 라 요청과 응답이 방향을 나눠 쓴다.

| | 바이트/추론 | @116.6 inf/s | 실측 링크(2.34 Gbps) 대비 |
|---|---:|---:|---:|
| RX (요청 640×640×3) | 1,228,800 | 1.196 Gbps | **51.1%** |
| TX (응답 want_float=0) | 1,218,000 | 1.194 Gbps | **51.0%** |

방향당 절반이 남는다. `/proc/net/dev` 실측이 ADR-008 의 페이로드 크기와
4.7% 오차(HTTP/2 + TCP/IP 헤더) 안에서 일치하므로 계산이 아니라 관측이다.

서버 쪽 aggregation 도 아니다. 3노드에서 3.00× 선형 확장이 나왔으므로
(S2 Finding 1) 공유 10G 링크와 스케줄러는 이 지점에서 병목이 아니다.
**같은 이유로 서버·스케줄러 자체도 배제된다** — 1노드에서 116 을 못 넘기게
하는 것이 서버였다면 3노드 342 가 나올 수 없다. 병목은 노드 쪽에 있다.

> ⚠️ **[2026-08-20 추가 — S3.8]** 이 배제는 **baseline(노드당 커넥션 1개)
> 조건에서만 유효하다.** 노드당 커넥션을 2개로 올리자 shared path 부하가
> 늘어 optimized 3N 의 scaling efficiency 가 **98.9% → 95.3%** 로 내려갔고,
> 서버 10G 링크도 67% → **76%** 로 올라왔다. **서버·스케줄러는 다시 후보다.**
> → [S3.8 §4.3](#experiments-s3-8-optimized-scaleout)
>
> 배제 판정에는 **어떤 조건에서 배제됐는지**가 함께 붙어야 한다.

### 4.2 보드 CPU 총량 — 아니다

8코어 전체 **63.1% idle**. 가장 바쁜 CPU0 도 30.3% 남는다.

### 4.3 커널 softirq 편중 (CPU0) — **A/B 로 반증됨**

프로파일은 CPU0 이 유독 바쁘다고 지목했다(busy 69.7%, 그중 %soft 51.5%.
나머지 코어는 27~38%). eth0 는 **RX 큐 1개**, IRQ 는 CPU0 고정, **RPS 꺼짐**
(`rps_cpus=00`, [`nic-topology.txt`](../results/transport-profile-20260820/raw/nic-topology.txt)).
그래서 NET_RX softirq 가 전부 CPU0 에서 직렬 처리된다.

코드 0줄로 검증 가능하므로 먼저 쟀다 — **S3.5b**: `rps_cpus` 를 `00`(CPU0만)
과 `fe`(코어 1~7)로 번갈아, 각 3회 60초, c32.

| rps_cpus | throughput | CPU0 %soft |
|---|---:|---:|
| `00` (기본) | **115.9 ± 0.7** | 50.4 / 50.9 / 51.3 |
| `fe` (코어 1~7) | **115.6 ± 0.9** | 42.4 / 41.9 / 42.0 |
| 차이 | **−0.3 inf/s (−0.2%)** | |

**효과 없음.** softirq 는 실제로 이동했는데(51% → 42%) 처리량은 그대로다.
CPU0 은 병목이 아니었다 — busy 69.7% 로 이미 30% 남아 있었던 것과 일치한다.

> 이 null 결과는 §4.4 의 근거가 된다. RPS 는 **flow 해시**로 분산한다.
> 흐름이 하나뿐이면 나눌 것이 없다. 그리고 실제로 흐름은 하나다.

### 4.4 HTTP/2 전송 경로 — **남는 것은 여기다**

부하 중 실제 TCP 연결을 셌다.

```text
king  ← scheduler   : 1 connection   192.168.123.3:51001 ← 192.168.123.9:37992
server: bench → scheduler : 32 connections (c32, 워커당 1개)
```

코드도 같은 말을 한다.

- bench 는 **동시성 워커마다 채널을 하나씩** 만든다
  ([`driver.rs:83-90`](../crates/npuforge-bench/src/driver.rs)).
- 스케줄러는 **노드당 채널 하나**를 캐시해 재사용한다
  ([`node_client.rs:31-79`](../crates/npuforge-scheduler/src/node_client.rs)).
  HTTP/2 다중화를 믿고 내린 결정이고, 요청마다 핸드셰이크를 피한다는
  근거 자체는 옳다.

결과적으로 **클라이언트 쪽 32 연결이 노드 앞에서 1 연결로 수렴한다.**
동시 요청 32건이 전부 이 연결 하나의 HTTP/2 스트림으로 흐른다. 그 연결은

- h2 커넥션 상태 기계 하나가 직렬로 프레이밍한다(단일 태스크),
- **64 KB 커넥션 flow-control window 하나를 32 스트림이 나눠 쓴다**
  — tonic 0.12.3 / h2 0.4.15 에서 window 설정이 코드 어디에도 없어
  전부 기본값(65,535)이다,
- TCP 흐름이 하나라 RPS·RSS 로 나눌 수 없다(§4.3).

다만 **이 셋은 아직 한 덩어리다.** HTTP/2 는 원래 커넥션 하나에서 스트림을
다중화하라고 만든 프로토콜이다. "커넥션이 1개" 라는 사실만으로 병목이라고
단정할 수 없다. 최소한 셋으로 갈라야 한다.

| 하위 후보 | 내용 |
|---|---|
| ① flow control | 64 KB 기본 window 가 1.2 MB 메시지를 stop-and-wait 로 만든다 |
| ② 커넥션/TCP 경로 | h2 커넥션 상태 기계·소켓 하나가 직렬화 지점이다 |
| ③ protobuf·복사 | 프레이밍과 encode/decode, `to_vec()` 복사 비용 |

**S3.6 이 이 셋을 가른다**(§7). 아래 정합성은 "전송 경로가 의심된다" 까지를
지지하는 것이지, 셋 중 어느 것인지를 지목하지 않는다.

관측이 전부 이 그림과 맞는다.

| 관측 | 단일 커넥션 가설과의 정합 |
|---|---|
| 대역폭 51%, CPU 63% idle | 자원이 아니라 **대기**가 상한을 만든다 |
| RPS 무효 | 흐름이 하나라 분산할 대상이 없다 |
| `node_queue` ≈ 0.02 ms | 요청이 워커를 기다리는 게 아니라 **도착을 못 한다** |
| 같은 보드 로컬 direct 8워커 = **161.5 inf/s** | 클러스터 116. 노드에 여유가 남는다 |
| S3 plateau (노드당 c10~16 이후 무증가) | 스트림을 늘려도 커넥션 상한은 그대로 |
| write syscall 84.4/req (≈14.4 KB) | 한 커넥션이 프레임 단위로 직렬 송신 |

특히 **`node_queue` ≈ 0 과 로컬 direct 161.5 inf/s** 가 결정적이다. 노드가
자기 상한(161.5)에 걸렸다면 c32 부하에서 워커 대기가 쌓여야 한다. 그런데
`node_queue` 는 0.02 ms 다. 받은 것을 즉시 처리하고 여유가 남는다는 뜻이다.
병목은 워커 풀 **앞**, 전송 계층에 있다.

> ⚠️ **`8워커 / inference_us 24.7 ms ≈ 324 inf/s` 를 노드 용량으로 쓰면
> 안 된다.** 같은 보드의 로컬 direct 8워커가 161.5 inf/s 에 그치므로 워커
> 8개가 독립적으로 돌지 않는다 — RKNN 런타임·NPU 내부 경합이 이미 있다.
> starvation 의 비교 기준은 **161.5** 다. 회수 가능한 gap 은 116 → 161.5,
> 약 30% 이지 116 → 324 가 아니다.

## 5. Interpretation

노드당 −30% 손실(116 → 161.5)의 정체는 **compute 도, 대역폭도, 커널 스택도
아니라 스케줄러↔노드 HTTP/2 전송 경로**다. 그 경로 안에서 flow control /
커넥션 / 직렬화 중 무엇인지는 **S3.6 에서 가른다.**

S2 Finding 2("오버헤드는 payload transfer path 에 있다")는 유효하다. S3.5 는
그 경로 안에서 비용의 성격을 바꿔 놓는다 — **바쁜 비용이 아니라 기다리는
비용**이다. 보드는 63% 놀고 링크는 49% 비어 있는데 처리량이 안 오른다.

## 6. Limitations

- **§4.4 는 아직 반증 실패이지 증명이 아니다.** 다른 셋을 배제했고 모든
  관측이 정합하지만, 커넥션 수나 window 를 바꿔 처리량이 오르는 것을
  직접 보여야 확정된다. 그 검증은 코드 변경이 필요해 동결을 벗어난다.
- 단일 보드(king), 단일 조건(c32, 45초 창). 3노드 프로파일은 없다.
- cycles/req 는 PMU 없는 근사값(§2 주). 절대값이 아니라 조건 간 비교용.
- `local` 조건의 도구(`sustained_load_test`)는 노드와 다른 프로그램이다.
  지연 정의가 달라(50.2 ms vs `inference_us` 24.7 ms) 두 값을 직접 빼면
  안 된다. 처리량과 CPU 점유만 비교했다.
- S3.5b 는 `rps_cpus` 만 바꿨다. RSS(다중 RX 큐)는 r8125 단일 큐라 불가.
- **S3.5b 의 per-run bench JSON 은 마지막 1건만 남았다.** 스크립트가 run
  사이에 `rm -f *.json` 으로 출력 디렉터리를 비워 앞선 원본을 함께 지웠다
  (수정 완료). 처리량·CPU0 %soft 는 `raw/results.csv` 와 `raw/mpstat_*` 에
  6건 모두 남아 있어 §4.3 의 결론은 영향받지 않는다.

## 7. S4 에 대한 함의

**io_uring 은 이 병목을 겨냥하지 않는다.** io_uring 이 줄이는 것은 syscall
진입 비용과 복사다. 지금 보드는 CPU 가 63% 놀고 있으므로, syscall 을 더
싸게 만들어도 상한이 오르지 않는다. TECHSPEC §15.3 이 적어 둔 비적용 조건
("구현 복잡도 대비 개선이 5% 미만")에 정면으로 해당한다.

측정된 비용 순서대로, 훨씬 싼 수단이 앞에 있다.

**io_uring 을 취소하는 것이 아니다.** 그 정도의 칼을 꺼낼 문제인지를 마지막으로
확인하는 단계를 넣는다.

```text
S2   scaling baseline      DONE
S3   saturation            DONE
S3.5 transport profiling   DONE  ← 이 문서
S3.6 H2 / channel A/B      다음   ← 원인을 셋으로 가른다
       ↓
     원인 확정
       ↓
S4 ├─ H2 tuning 이 답이면 → gRPC optimized
   └─ 아니면              → io_uring
```

S3.6 은 최소 변경으로 §4.4 의 ①②를 분리한다. 1노드 saturation 동일 조건에서:

| Test | 노드당 커넥션 | H2 window | 목적 |
|---|---:|---|---|
| A | 1 | default | baseline (= 현재 115) |
| B | 1 | 크게 확대 | **flow control 검증** |
| C | 4 | default | **커넥션/TCP 경로 검증** |
| D | 4 | 확대 | 결합 효과 |

해석은 깨끗하다.

- **B 만 상승** → 범인은 커넥션 수가 아니라 HTTP/2 flow control
- **C 만 상승** → 범인은 단일 커넥션 / TCP 경로
- **B·C 둘 다 상승** → 둘 다 영향
- **D 까지 그대로** → HTTP/2 가설 약화 → ③(protobuf·복사·syscall)로 복귀,
  **이때 io_uring 이 훨씬 강한 근거를 갖는다** (대역폭도, CPU 배치도,
  flow control 도 아니었다)
  — 단 "스케줄러도 아니다" 는 이후 S3.8 에서 **철회**됐다(위 §4.1 주 참조)

window 는 최적값 탐색이 아니라 **64 KB 급 기본값이 막고 있었는지 여부**만
본다. 수 MB~수십 MB 수준으로 충분히 크게 잡는다.

만약 window 만 키워 115 → 145~155 가 나온다면 S4 의 결론이 바뀐다 —
"gRPC 가 느린 게 아니라 **기본 HTTP/2 설정이 대형 payload workload 와 맞지
않았다**". 수천 줄짜리 transport 를 새로 만들기 전에 몇 줄짜리 설정으로
30% 의 상당 부분을 회수한다면, 시스템 연구로서 그쪽이 더 강한 판단이다.

그 밖에 측정으로 뒷받침되는 수단:

| 수단 | 근거 |
|---|---|
| **응답 페이로드 축소** — 노드 postprocess 후 검출결과만 반환 (1.218 MB → 수 KB) | 와이어·protobuf·복사 부하를 절반 제거 |

## 8. Reproduction

```bash
bash scripts/run-transport-profile.sh              # 세 조건 (약 5분)
bash scripts/run-transport-profile.sh --only local # 한 조건만
PYTHONIOENCODING=utf-8 python scripts/analyze-transport-profile.py

bash scripts/run-rps-ab.sh                         # S3.5b (약 10분)
```

동결 commit `01f29a2`. `run-rps-ab.sh` 는 `rps_cpus` 를 런타임으로만 바꾸고
끝에서 원래 값(`00`)으로 되돌린다.

## 9. Conclusion

노드당 ~116 inf/s 상한의 원인은 **스케줄러↔노드 HTTP/2 전송 경로**에 있다.
링크 대역폭(방향당 51% 사용), 보드 CPU 총량(63% idle), 커널 softirq 편중
(RPS A/B −0.2%), 서버·스케줄러(3노드 3.00× 선형)는 모두 배제됐다. 노드는
같은 보드에서 로컬 direct 161.5 inf/s 를 내면서 클러스터에서는 116 에
그치고, `node_queue` ≈ 0 으로 여유가 남는다.

전송 경로 안에서 ①flow control ②커넥션/TCP ③protobuf·복사 중 무엇인지는
**아직 갈리지 않았다.** → **S3.6** 이 최소 변경 A/B 로 이를 가르고, 그
결과가 S4 를 `gRPC optimized` 와 `io_uring` 중 하나로 확정한다(§7).

---

<a id="experiments-s3-6-h2-channel-ab"></a>

# S3.6 — HTTP/2 Window × Connections-per-Node A/B

- 실험 ID: **S3.6**
- 측정일: 2026-08-20
- 코드: `11cec9b` + `[transport]` 설정 추가 (기본값은 동결과 동일 동작)
- 상태: **완료 (20 runs, 4조건 × 5라운드)**
- 원본: [`../../results/h2-channel-ab-20260820/`](../results/h2-channel-ab-20260820)
- 선행: [`S3_5_TRANSPORT_PROFILE.md`](#experiments-s3-5-transport-profile)
- 후속: **S4** — 이 결과로 방향이 정해진다 (§7)

---

## 1. Research Question

> **S3.5 가 전송 경로로 좁힌 −30% 손실은, 그 경로 안에서 무엇 때문인가?**

S3.5 는 대역폭(방향당 51%), 보드 CPU 총량(63% idle), CPU0 softirq 편중
(RPS A/B −0.2%), 서버·스케줄러(3노드 3.00× 선형)를 배제했다. 남은 것은
스케줄러↔노드 HTTP/2 전송 경로인데, 그 안에 후보가 셋 뭉쳐 있었다.

| | 하위 후보 |
|---|---|
| ① | **flow control** — 64 KB 기본 window 가 1.2 MB 메시지를 stop-and-wait 로 만드는가 |
| ② | **커넥션/TCP** — 커넥션 상태 기계·소켓 하나가 직렬화 지점인가 |
| ③ | **protobuf·복사** — 프레이밍과 encode/decode 비용인가 |

**"커넥션이 1개" 라는 사실만으로 ②라고 단정하면 안 된다.** HTTP/2 는 원래
커넥션 하나에서 스트림을 다중화하라고 만든 프로토콜이다. 그래서 ①과 ②를
직교하게 흔든다.

## 2. Method

2×2. 1노드(king) saturation 조건(c32, 60초)에서 조건당 **5 run**, 총 20 run.

| Test | 노드당 커넥션 | H2 window | 목적 |
|---|---:|---|---|
| **A** | 1 | default (64 KB) | baseline |
| **B** | 1 | stream 8 MB / conn 64 MB | ① 검증 |
| **C** | 4 | default | ② 검증 |
| **D** | 4 | 8 MB / 64 MB | 결합 |

- window 는 최적값 탐색이 **아니다**. 64 KB 급 기본값이 막고 있었는지 여부만
  본다. 한 메시지(1.23 MB)가 WINDOW_UPDATE 없이 통째로 들어가는 크기로 잡았다.
- flow control 은 **수신자가 광고**한다. 요청(1.23 MB) 방향은 노드가, 응답
  (1.218 MB) 방향은 스케줄러가 정하므로 **양쪽 다** 설정했다.
- 라운드마다 조건 순서를 rotate. 온도·시간 경과가 한 조건에 몰리지 않게 한다.
- run 마다 노드의 실제 TCP 커넥션 수를 `ss` 로 세어 기록했다 — 설정이 조용히
  무시되면 A/B 가 아니라 같은 조건 4번이 된다.
- 스크립트: [`run-h2-channel-ab.sh`](../scripts/run-h2-channel-ab.sh),
  [`analyze-h2-channel-ab.py`](../scripts/analyze-h2-channel-ab.py).

## 3. Results

5 run 평균 ± SD. 오류율 전 구간 **0**.

| 조건 | TCP 실측 | throughput | vs A | E2E p50 | E2E p95 | →node | node_queue |
|---|---:|---:|---:|---:|---:|---:|---:|
| **A** 1ch default | 1 | **115.3 ± 0.8** | — | 269.3 | 392.8 | 115.8 | 0.02 |
| **B** 1ch bigwin | 1 | **73.5 ± 0.4** | **−36.3%** | 480.1 | 596.2 | 204.0 | 0.02 |
| **C** 4ch default | 4 | **140.1 ± 0.3** | **+21.5%** | 163.3 | 572.6 | 60.1 | 0.02 |
| **D** 4ch bigwin | 4 | **139.5 ± 1.1** | +21.0% | 172.7 | 558.6 | 64.2 | 0.02 |

SD 가 0.3~1.1 로 매우 작다. TCP 실측이 1/1/4/4 로 의도한 조건이 실제로 걸렸다.

**A 가 115.3 으로 S2·S3 baseline(115.2 ceiling)을 재현한다.** `[transport]`
추가가 기존 동작을 바꾸지 않았다는 회귀 검사가 된다.

### 보드 프로파일 (king, 조건별 5 run 평균)

| 조건 | %usr | %sys | %soft | %idle | CPU0 busy | CPU0 %soft | syscall/req |
|---|---:|---:|---:|---:|---:|---:|---:|
| A | 18.0 | 12.2 | 6.3 | **63.4** | 69.5 | 51.0 | 84.9 |
| B | 13.9 | 8.9 | 4.3 | **73.0** | 51.5 | 35.2 | 93.6 |
| C | 27.8 | 18.9 | 9.4 | **43.9** | **81.1** | **74.4** | 82.4 |
| D | 27.9 | 18.8 | 9.4 | 43.9 | 80.6 | 74.1 | 80.5 |

## 4. Interpretation

### 4.1 ② 노드당 단일 커넥션 구조가 주요 제약 요인이다 (+21.5%)

커넥션만 1 → 4 로 늘렸을 뿐인데 **115.3 → 140.1 inf/s**. window 는 기본값
그대로다. `network_to_node` 가 115.8 → 60.1 ms 로 절반이 됐고, 보드 idle 이
63.4% → 43.9% 로 떨어졌다 — **같은 시간에 실제로 더 많은 일이 진행된다.**

로컬 direct(161.5 inf/s) 기준으로 보면

```text
A  115.3  ──  회수 24.8  ──▶  C  140.1  ──  남은 21.4  ──▶  로컬 161.5
              (gap 46.2 중 54%)                (13.3% 남음)
```

**설정 한 줄로 gap 의 절반 이상을 회수했다.**

> ⚠️ **"커넥션 수가 제약" 까지가 이 실험이 보인 것이다.** 그 안에서 실제로
> 무엇이 직렬화 지점인지는 **아직 분리되지 않았다.** 최소한 셋이 남는다.
>
> | | 남은 내부 후보 |
> |---|---|
> | ②-a | TCP per-flow 처리 (소켓·softirq·혼잡제어가 흐름 단위) |
> | ②-b | HTTP/2 multiplexing / 커넥션 상태 기계의 락·직렬화 |
> | ②-c | flow control 과의 상호작용 (커넥션당 window 를 스트림이 나눠 씀) |
>
> 1ch → 4ch 라는 단일 변경으로 +21.5% 가 반복 재현됐으므로 **단일 커넥션
> 구조가 실제 제약이라는 것은 상당히 강하다.** 하지만 ②-a/b/c 중 무엇인지를
> 지목하려면 커넥션 수 sweep 과 per-flow 계측이 더 필요하다(§7).

### 4.2 ① 64 MB 급 large window 는 이 workload 에서 크게 해롭다 (−36.3%)

window 를 키우자 **처리량이 36% 떨어졌다.** 재현성도 높다(SD 0.4).

지연이 원인이다. E2E p50 269 → 480 ms, `network_to_node` 115.8 → 204.0 ms.
closed-loop c32 에서 지연 증가는 그대로 처리량 손실이다(32 / 0.48 s ≈ 67).
보드 idle 은 오히려 **올라간다**(63.4% → 73.0%) — 일을 더 하는 게 아니라
더 기다린다.

> **해석(가설).** 64 MB 커넥션 window 는 동시 요청 32건(39 MB)을 한꺼번에
> 소켓에 밀어 넣도록 허용한다. HTTP/2 는 DATA 프레임을 스트림 사이로
> 번갈아 내보내므로, 32개가 앞서거니 뒤서거니 하며 **다 같이 늦게** 끝난다.
> 64 KB 기본 window 는 in-flight 를 제한해 사실상 backpressure·pacing 으로
> 동작하고 있었고, 그래서 앞선 요청이 먼저 끝날 수 있었다.
>
> 이건 지연 분해와 idle 상승으로 뒷받침되는 **가설이지 확정이 아니다.**
> 확정하려면 in-flight 바이트와 프레임 인터리빙을 직접 계측해야 한다.

4 커넥션에서는 window 효과가 사라진다(D 139.5 ≈ C 140.1). 커넥션당 동시
스트림이 32 → 8 로 줄어 인터리빙 폭이 좁아진 것과 일관되지만, 이 역시
직접 계측하지 않았다.

**실무 결론: window 는 기본값(64 KB)을 유지한다.** 적어도 이 크기로는 손해다.

> ⚠️ **이 실험이 보인 것은 "64 MB 급 large window 가 이 workload 에서 성능을
> 크게 악화시켰다" 까지다.** "window tuning 은 효과가 없다" 가 아니다.
> 64 KB → 64 MB 는 **1000배 차이라 극단적인 A/B** 이고, 중간값(256 KB /
> 1 MB / 4 MB)에 최적점이 있을 가능성을 배제하지 못한다.
> 지금 우선순위는 아니지만 열어 둔다.

### 4.3 대가 — tail latency 는 나빠진다

> ⚠️ **[2026-08-20 정정 — S3.7b]** 이 절의 tail 결론은 **c32 에서 잰 것인데,
> c32 는 이 워크로드의 overload 구간이다.** S3.7b 가 운영점을 c12 로 확정했고
> (peak 의 98% 를 내는 가장 낮은 concurrency), **그 지점에서는 커넥션 1 → 2 가
> 처리량 +18.8% 와 함께 p95 −18.8% / p99 −17.8% 로 tail 도 개선한다.**
> 트레이드오프가 아니라 strict Pareto improvement 다.
>
> 아래 측정값 자체는 유효하다. 다만 그것이 재는 것은 "어느 구성이 더 좋은가"
> 가 아니라 **"어느 구성이 과부하에서 더 완만하게 무너지는가"** 다.
> → [`S3_7_CONNECTION_TUNING.md`](#experiments-s3-7-connection-tuning) §4.3

| | A | C |
|---|---:|---:|
| E2E p50 | 269.3 | **163.3** (−39%) |
| E2E p95 | **392.8** | 572.6 (**+46%**) |

처리량과 p50 은 좋아지는데 **p95 는 46% 나빠진다.** 평균적인 요청은 훨씬
빨라졌는데 일부 요청이 크게 늦어진다.

**이걸 round-robin 탓으로 확정하면 안 된다.** 가능한 원인이 여럿이다.

| | 후보 |
|---|---|
| a | 커넥션별 in-flight 불균형 (round-robin 이 부하를 안 봄) |
| b | HTTP/2 커넥션 내부 큐 편차 |
| c | NPU 워커 도착 burst |
| d | transport queueing |
| e | 처리량이 올라가면서 생기는 일반적인 tail queue 증가 |

전부 미검증이다. 이 트레이드오프를 숨기면 안 된다 — 실시간 추론에서 tail 은
중요한 지표이고, **다음 실험의 연구 질문이지 각주가 아니다.** S3.7 의 커넥션
sweep 은 처리량만이 아니라 **p95·p99 를 함께 보고 최적점을 정한다**(§7).

### 4.4 다음 병목이 드러났다 — 그리고 S3.5b 의 null 이 설명된다

C·D 에서 **CPU0 busy 81.1%, 그중 %soft 74.4%.** 다른 코어는 여유가 있는데
CPU0 만 포화에 가까워진다. eth0 는 RX 큐 1개이고 RPS 는 꺼져 있다.

S3.5b 에서 RPS 가 무효였던 이유가 여기서 분명해진다 — **RPS 는 flow 해시로
분산하는데 그때는 흐름이 하나뿐이었다.** 이제 흐름이 4개다. 즉 S3.5b 를
C 조건 위에서 다시 하면 결과가 달라질 수 있다(§7).

## 5. 판정

| 후보 | 판정 |
|---|---|
| ① flow control | **64 MB 급 확대는 −36.3% 로 해롭다.** 기본 64 KB 가 backpressure 로 기능하고 있었다. 중간값은 미측정이라 "튜닝 무효" 로는 결론짓지 않는다 |
| ② 커넥션/TCP | **노드당 단일 커넥션 구조가 주요 제약 요인.** 1 → 4 로 +21.5%, gap 의 54% 회수. 다만 ②-a/b/c 중 무엇인지는 미분리 |
| ③ protobuf·복사 | 남은 13.3% 안에 있을 수 있다. 아직 미분리 |

## 6. Limitations

- **4 가 최적이라는 근거는 없다.** 1 과 4 만 비교했다. 2/8/16 은 미측정.
- **1노드 결과다.** 3노드에서는 서버가 12 커넥션을 들게 된다. S2·S3 숫자를
  갱신하려면 다중 노드에서 다시 재야 한다.
- **§4.2 의 bufferbloat 설명은 가설이다.** 지연 분해·idle 상승과 정합하지만
  in-flight 바이트를 직접 재지 않았다.
- window 는 8 MB / 64 MB 한 점만 봤다. 중간 크기(예: 1~2 MB)는 미측정이라
  "크게 하면 나쁘다" 를 단조 관계로 일반화할 수 없다.
- **p95 악화의 원인은 미검증이다.** 후보가 최소 5개 있고(§4.3) 어느 것도
  배제하지 못했다.
- 조건마다 스케줄러·노드를 재기동한다. 재기동 자체의 영향은 A 가 baseline 을
  재현하는 것으로 간접 배제했다.

## 7. S4 에 대한 함의

**io_uring 은 지금도 정당화되지 않는다.** syscall/req 는 네 조건에서
80.5~93.6 으로 거의 변하지 않는데 처리량은 73.5~140.1 로 두 배 차이가 난다 —
**syscall 횟수는 지금의 1차 병목을 설명하지 못한다.**

> 단, 이것이 "io_uring 이 효과 없다" 는 뜻은 아니다. **syscall 횟수가 같다는
> 것과 syscall·복사에 쓰는 CPU 시간이 작다는 것은 다른 문제다.** 지금 말할 수
> 있는 것은 **순서**다 — 더 싼 병목이 아직 남아 있으므로 io_uring 은 뒤로 민다.

로드맵을 이렇게 갱신한다.

```text
S3.5  transport profiling   DONE  전송 경로로 좁힘
S3.6  H2/channel A/B        DONE  ← 단일 커넥션 구조가 주요 제약
        ↓
S3.7  ① 커넥션 sweep (1/2/4/8/16) → 최적 N
      ② 그 N 위에서 RPS 재시도                                  ← 다음
        ↓
optimized gRPC baseline (1N/2N/3N 재측정)
        ↓
남은 gap 분석
        ↓
필요하면 io_uring
```

**최적점은 최대 처리량이 아니라 처리량–tail latency 트레이드오프로 정한다.**
예를 들어 `4ch = 140 inf/s, p95 573` 과 `8ch = 148 inf/s, p95 900` 이면
8ch 가 더 좋은 시스템이라고 할 수 없다. 무조건 많을수록 좋은 것도 아니다 —
어느 지점부터는 커넥션 관리 비용과 queueing 으로 다시 꺾일 수 있다.

RPS 재시도가 특히 가치 있는 이유: 1커넥션일 때는 흐름이 하나라 나눌 게
없었지만 이제 흐름이 여러 개다. 그리고 CPU0 는 busy 81% / soft 74% 다.
- 여기서 오르면 → "단일 커넥션 제약을 풀자 NIC 처리 병목이 드러났고,
  multi-flow 에서 비로소 RPS 가 효과를 갖는다" 는 서사가 성립한다.
- 또 변화가 없으면 → CPU0 softirq 는 **상관관계일 뿐 처리량 limiter 가
  아니다** 로 더 강하게 배제할 수 있다.

S3.7 이 싼 이유: 커넥션 수는 이미 설정이고, RPS 는 코드 0줄이다. 이 둘을
털고 나서야 ③ 이 순수하게 남는다.

> **S2·S3 숫자는 아직 갱신하지 않는다.** 140.1 은 1노드 최적화 결과일 뿐이고,
> 3노드에서는 서버가 4 × 3 = 12 커넥션을 들게 되어 서버 쪽에 새 병목이
> 나타날 수 있다. S3.7 에서 N 을 확정한 뒤 1N/2N/3N 을 다시 돌린다.

## 8. Reproduction

```bash
bash scripts/run-h2-channel-ab.sh 5     # 20 run, 약 35분
PYTHONIOENCODING=utf-8 python scripts/analyze-h2-channel-ab.py \
    results/h2-channel-ab-20260820/raw/results.csv
```

스크립트는 끝에서 기본 설정(= 동결과 동일 동작)으로 되돌린다. 동결
바이너리는 `npuforge-{scheduler,node}.frozen-01f29a2` 로 남아 있다.

> 노드는 `--features rknn` 과 `RKNN_SDK_PATH=/usr/include` 가 필요하다.
> 빠뜨리면 Mock 백엔드 바이너리가 나와 기동에 실패한다(실제로 한 번 겪었고,
> 하네스가 큰 소리로 실패해 바로 잡혔다).

## 9. Conclusion

**노드당 단일 gRPC/HTTP2 커넥션 구조가 처리량을 제한하는 주요 요인임을
확인했다.** 노드당 커넥션을 4개로 늘리자 **115.3 → 140.1 inf/s (+21.5%)**,
로컬 direct 까지의 gap 46.2 중 **54% 를 설정만으로 회수**했다. 코드 아키텍처를
갈아엎지 않고 커넥션 풀 하나로 얻은 결과다. 대가로 p95 가 46% 나빠진다.

그 구조 안에서 TCP per-flow 처리인지, H2 multiplexing/락인지, flow control
상호작용인지는 **아직 분리되지 않았다**(§4.1).

> ⚠️ **[정정]** 아래 "대가로 p95 가 46% 나빠진다" 는 c32(overload 구간) 측정이다.
> 운영점 c12 에서는 커넥션 1 → 2 가 tail 도 개선한다(S3.7b §4.3).

**64 MB 급 large window 는 이 workload 에서 −36.3% 로 크게 해로웠다.**
기본 64 KB 가 backpressure 로 기능하고 있었다는 뜻이다. 기본값을 유지하되,
1000배 차이의 극단 A/B 였으므로 중간값에 최적점이 있을 가능성은 열어 둔다.

남은 13.3% 와 새로 드러난 CPU0 포화(busy 81%, soft 74%)는 S3.7 에서 다룬다.
io_uring 은 여전히 근거가 부족하다 — syscall/req 는 조건 간 거의 불변인데
처리량은 두 배 차이가 났다.

---

## Figure

![커넥션은 돕고 window 확대는 해친다](../results/h2-channel-ab-20260820/figures/fig_h2_window_vs_conns.png)

**`fig_h2_window_vs_conns.png`** — 커넥션은 돕고 window 확대는 해친다

재생성: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s3-7-connection-tuning"></a>

# S3.7 — Connection Tuning (a: sweep, b: concurrency, c: RPS)

- 실험 ID: **S3.7a · S3.7b · S3.7c** (완료)
- 측정일: 2026-08-20
- 코드: `4e64bf4` (`[transport]` 설정. 기본값은 동결과 동일 동작)
- 원본: [`../../results/connection-sweep-20260820/`](../results/connection-sweep-20260820)
- 선행: [`S3_6_H2_CHANNEL_AB.md`](#experiments-s3-6-h2-channel-ab)

---

## 0. 이 실험이 답하는 것

S3.6 은 커넥션 1 → 4 만 비교해 +21.5% 를 봤다. **4 가 최적이라는 근거는
없었고**, 처리량이 오르는 대신 **p95 가 46% 나빠졌다.**

그래서 S3.7 은 "최대 처리량 찾기" 가 아니라 **운영점 선택(operating point
selection)** 문제로 둔다.

```text
S3.7a  커넥션 1/2/4/8/16 @ c32 고정      → Pareto 후보 선정        ← 완료
S3.7b  상위 후보에 대해 concurrency sweep → 실제 operating point 확정
S3.7c  그 운영점에서 RPS OFF/ON           → optimized gRPC 동결
```

---

# S3.7a — Fixed-load connection-count A/B

## 1. Method

1노드(king), **c32 고정**, 60초, 커넥션 1/2/4/8/16, 조건당 **5 run** (총 25).
window 는 기본값(S3.6 결론: 64 MB 급 확대는 −36.3%). 라운드마다 순서를 뒤집어
온도·시간 경과가 한 조건에 몰리지 않게 했다. run 마다 노드의 실제 TCP 커넥션
수를 `ss` 로 세어 기록했다.

> **이 실험은 각 설정의 ceiling 이 아니다.** 부하를 c32 로 고정했으므로
> 커넥션 수의 *순수 효과* 비교로는 좋지만, 커넥션을 늘리면 saturation
> concurrency 가 c32 위로 이동했을 수 있다. 그래서 S3.7b 가 따로 있다.

## 2. Results

오류율 전 구간 **0**. 지연은 모두 **run-level percentile 의 run 간 평균**
(pooled 아님 — S2 §7.4.1).

| conn | TCP 실측 | throughput | vs c1 | p50 | p95 | p99 | max | →node |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **1** | 1 | 115.6 ± 0.7 | — | 268.0 | **392.4** | **452.4** | 597.6 | 114.9 |
| **2** | 2 | **134.4 ± 0.7** | **+16.3%** | 226.7 | **438.2** | **514.9** | 679.7 | 92.5 |
| **4** | 4 | **139.5 ± 0.2** | **+20.7%** | 169.5 | 561.6 | 698.2 | 944.5 | 63.1 |
| **8** | 8 | 139.1 ± 0.6 | +20.4% | 157.4 | 597.0 | 827.0 | 1222.5 | 56.9 |
| **16** | 16 | 136.8 ± 0.7 | +18.4% | 173.4 | 584.2 | 895.1 | 1481.5 | 65.3 |

- 대표 그림: [`fig_sweep_pareto.png`](../results/connection-sweep-20260820/figures/fig_sweep_pareto.png)
  (X=p95, Y=throughput, 점=커넥션 수)
- 보조: [`fig_sweep_throughput.png`](../results/connection-sweep-20260820/figures/fig_sweep_throughput.png),
  [`fig_sweep_latency.png`](../results/connection-sweep-20260820/figures/fig_sweep_latency.png)

## 3. Interpretation

### 3.1 connection parallelism 에는 knee 가 있다

> ⚠️ **[정정 — §4]** 아래는 전부 **c32 고정** 측정이다. 세 구성 모두 운영점이
> c12 이므로 c32 는 overload 구간이고, 여기서 보이는 tail 악화는 상당 부분
> **커넥션 수가 아니라 과부하 큐잉** 때문이다. knee 가 존재한다는 관찰은
> 유효하지만, 그 knee 는 **connection knee 이자 동시에 concurrency knee 와
> 얽혀 있다**(§0).

처리량은 **c4 에서 평평해진다**(139.5). c8 은 139.1 로 사실상 같고 c16 은
136.8 로 오히려 내려간다. 반면 tail 은 **단조롭게 악화**한다.

```text
p99   c1 452  →  c2 515  →  c4 698  →  c8 827  →  c16 895
max   c1 598  →  c2 680  →  c4 945  →  c8 1223 →  c16 1482
```

**데이터가 증명한 것은 여기까지다.**

> **c4 이후의 추가 connection parallelism 은 c32 workload 에서 처리량을
> 개선하지 못하고 tail latency 를 악화시킨다.**

⚠️ **"커넥션 관리 비용·queueing 때문에 꺾인다" 는 아직 원인 확정이 아니다.**
가능한 기여 요인이 섞여 있고 어느 것도 분리하지 않았다 — H2 내부 queueing,
커넥션별 in-flight 편차, TCP 처리, NPU 도착 burst 등(S3.6 §4.3 과 같은 목록).

또 하나 흥미로운 것은 **median 과 tail 이 반대 방향으로 움직인다**는 점이다.
커넥션을 늘리면 평균적인 요청은 빨라지는데(p50 268 → 157) 일부 요청은 훨씬
늦어진다(p99 452 → 895). "커넥션이 많을수록 빠르다" 가 아니라 **처리량–tail
트레이드오프가 실재한다.**

### 3.2 진짜 선택은 c2 냐 c4 냐다

> ⚠️ **[정정 — §4]** 이 절의 트레이드오프는 **c32(overload 구간)** 에서 잰
> 것이다. 운영점 c12 에서는 conn4 가 conn2 대비 처리량 +1.2% 에 p95 +1.2% 로
> **거의 무승부**다(§4.1). 아래 "+3.8% 를 위해 tail 28~39%" 는 과부하
> 구간에서만 성립한다.

| | c2 | c4 | c4 가 치르는 값 |
|---|---:|---:|---|
| throughput | 134.4 | 139.5 | **+3.8%** |
| p95 | 438.2 | 561.6 | **+28.2%** |
| p99 | 514.9 | 698.2 | **+35.6%** |
| max | 679.7 | 944.5 | **+39.0%** |

**c4 는 처리량 +3.8% 를 위해 tail 을 28~39% 내준다.** 실시간 추론에서는
나쁜 거래로 보인다.

c1 기준으로 보면 더 분명하다 — c2 는 c4 가 얻은 이득의 **79%**(+16.3 / +20.7)를
tail 비용의 **약 4분의 1**로 얻는다(p95 +11.7% vs +43.1%).

로컬 direct(161.5) 까지의 gap 회수율:

```text
c1 115.6  ─ gap 45.9 ─▶  로컬 161.5
c2 134.4  회수 18.8 (41%)
c4 139.5  회수 23.9 (52%)
```

### 3.3 heuristic 은 c4 를 골랐다 — 그리고 그건 아슬아슬하다

분석기 규칙("처리량 최대의 97% 이상 중 p95 최소")은 **c4** 를 고른다.
c2 가 **96.4%** 로 임계를 **0.6%p 차이로** 놓쳤기 때문이다.

> **임계값을 결과에 맞춰 옮기지 않는다.** 96.4% 를 담으려고 97% → 96% 로
> 내리면 그건 heuristic 이 아니라 사후 합리화다. 규칙은 그대로 두고,
> **규칙이 이 경계에서 결론을 내주지 못한다는 사실 자체를 결과로 기록한다.**
>
> 이것이 §0 에서 "Selected operating point 는 통계적 최적값이 아니라 의도적
> engineering heuristic" 이라고 못 박은 이유다. 표를 같이 내보이는 것도
> 그래서다 — 경계에서는 사람이 판단해야 하고, 판단의 근거가 표에 있어야 한다.

## 4. S3.7a 결론과 다음 수

- **c8·c16 은 S3.7b 후보에서 제외한다.**

  > 이것은 "어떤 concurrency 에서도 c8/c16 이 열등하다" 가 **아니다.**
  > S3.7a 는 c32 fixed-load 라 c8/c16 의 절대 ceiling 을 재지 않았다.
  > 근거는 **우선순위**다 — c32 에서 이미 tail 비용이 이만큼 크므로
  > (p99 827·895, max 1223·1482) 추가 탐색 비용 대비 기대값이 낮다.
  > 필요하면 나중에 되돌아올 수 있다.

- **c2·c4 를 S3.7b 후보로 올린다.** c32 고정에서는 둘의 우열이 갈리지 않는다.
  c2 가 아직 포화가 아니라면 더 높은 concurrency 에서 역전할 수 있고,
  c4 의 tail 이 concurrency 증가에 더 빨리 무너질 수도 있다.

  현 상태의 성격을 요약하면 **c2 = efficiency point, c4 = performance point**
  다. 어느 쪽이 운영점인지는 ceiling 을 봐야 정해진다.

## 5. Limitations

- **각 설정의 ceiling 이 아니다**(§1 주). c32 고정 결과다.
- percentile 은 run-level 평균이라 pooled 보다 tail 을 낮게 보인다.
  조건 간 비교에는 유효하나 절대값 인용은 안 된다(S2 §7.4.1).
- p95/p99 악화의 **원인은 여전히 미검증**이다. S3.6 §4.3 의 후보 5개
  (커넥션별 in-flight 불균형 / H2 내부 큐 편차 / NPU 도착 burst /
  transport queueing / 처리량 상승에 따른 일반적 tail 증가) 중 어느 것도
  배제하지 못했다.
- 1노드 결과다. 3노드면 서버가 커넥션을 N×3 개 들게 된다(S3.8).

## 6. Reproduction

```bash
bash scripts/run-connection-sweep.sh sweep 5     # 25 run, 약 40분
PYTHONIOENCODING=utf-8 python scripts/analyze-connection-sweep.py \
    results/connection-sweep-20260820/raw/results.csv
python scripts/make-sweep-figures.py \
    results/connection-sweep-20260820/raw/results.csv \
    results/connection-sweep-20260820/figures
```

---

# S3.7b — Concurrency sweep

## 0. 튜닝 대상은 1차원이 아니라 2차원이었다

S3.7a·b 를 거치며 드러난 구조가 이것이다. **knee 가 둘 있다.**

```text
Concurrency knee   요청을 몇 개까지 동시에 넣어야 장치를 포화시키는가?
Connection knee    그 요청을 몇 개의 커넥션으로 나누는 것이 효율적인가?
```

즉 튜닝해야 할 것은 "커넥션 수" 하나가 아니라
**load concurrency × connection parallelism 의 2차원 운영점**이다.

이것은 NPUForge 의 원래 질문 —"왜 안 늘어나지?"— 에 정확히 닿는다.
**포화 이후에는 더 밀어넣어도 NPU 가 더 일하는 게 아니라 시스템 안에 큐만
쌓인다.** 아래 §2 가 그것을 실측으로 잡은 것이다.

## 1. 운영 concurrency 의 정의 (실험 규칙)

숫자로 못 박아 둔다. 그러지 않으면 132.8 / 134.1 / 134.3 같은 결과에서
"어디가 knee 냐" 가 매번 사람 판단으로 들어간다.

> **operating concurrency = peak 처리량의 98% 이상을 내는 가장 낮은 concurrency**

**98% 인 이유**: 관측된 run 간 SD 가 ±1 inf/s 수준이라 99% 로 잡으면 임계가
측정 noise 와 겹친다. 이 정의는 `analyze-concurrency-sweep.py` 에 상수로
들어가 있다.

## 2. 1차 범위 (c24~c64) — 전 구간이 overload 였다

후보 **c2 · c4**, 각 3 run.

| conc | conn2 tp | conn2 p95 | conn2 p99 | conn4 tp | conn4 p95 | conn4 p99 |
|---:|---:|---:|---:|---:|---:|---:|
| **24** | **134.3 ± 1.1** | **306.9** | **357.5** | **139.3 ± 1.2** | **390.9** | **480.2** |
| 32 | 133.7 | 431.5 | 505.7 | 138.3 | 576.5 | 715.0 |
| 40 | 134.2 | 572.1 | 674.4 | 137.7 | 719.5 | 932.9 |
| 48 | 133.8 | 697.6 | 832.0 | 137.6 | 958.6 | 1200.9 |
| 64 | 132.9 | 946.0 | 1132.3 | 137.9 | 1254.4 | 1566.7 |

오류율 전 구간 0.

**처리량이 c24~c64 내내 완전히 평평하다**(conn2 ≈ 134, conn4 ≈ 138). 반면
tail 은 거의 선형으로 커진다 — conn4 @c64 는 p99 1567 ms, max 2128 ms.

> **이 범위는 전부 포화 이후 구간이다.** 데이터가 말하는 것:
>
> **처리량 포화는 concurrency ≤ 24 에서 일어난다. 포화 이후의 추가
> concurrency 는 처리량을 늘리지 않고 tail latency 만 증가시킨다.**

전형적인 queueing 이다. 더 밀어넣은 요청은 계산으로 가지 않고 대기열로 간다.

### 2.1 ~~트레이드오프는 부하에 안정적이다~~ — **틀렸다 (§4 에서 반증)**

| | S3.7a @c32 | S3.7b @c24 |
|---|---:|---:|
| throughput | +3.8% | +3.7% |
| p95 | +28.2% | +27.4% |
| p99 | +35.6% | +34.3% |

두 값이 거의 같아서, 처음에는 "특정 concurrency 의 우연이 아니라 **4 커넥션
자체가 만드는 트레이드오프**" 라고 썼다. **그 해석은 틀렸다.**

c32 와 c24 가 일치한 것은 **둘 다 overload 구간이라 같은 현상을 두 번 본
것**이었다. §4 에서 진짜 운영점(c12)으로 내려가자 p95 페널티가
**+28% → +1.2%** 로 사라진다. 커넥션 4개의 성질이 아니라 **포화 이후 큐잉의
성질**이었다.

> 교훈: **두 측정이 일치한다는 것이 곧 그 해석이 옳다는 뜻은 아니다.**
> 둘 다 같은 방향으로 편향돼 있으면 재현성은 편향을 확인해 줄 뿐이다.

### 2.2 그래서 sweep 방향이 틀렸다

두 후보 모두 최고점이 **sweep 하단(c24)** 이다. 즉 포화점은 c24 **이하**에
있고, 운영점(= ceiling 을 내는 가장 낮은 concurrency)을 아직 못 봤다.
→ **c8/c12/c16/c20/c24 로 아래쪽을 다시 훑는다.**

## 3. conn1 baseline 을 같은 범위에서 다시 잰다

이걸 안 하면 해석이 섞인다. 지금 가진 두 점을 나란히 놓으면

```text
conn1 @c32 →  115.6 inf/s,  p95 392
conn2 @c24 →  134.3 inf/s,  p95 307
```

"2 커넥션이 처리량과 지연을 **둘 다** 개선했다" 고 쓰고 싶어진다. 그러나
**변수가 두 개 동시에 바뀌었다** — 커넥션 1→2, concurrency 32→24. 인과를
분리할 수 없다.

각 커넥션 수의 **운영점을 같은 규칙(§1)으로 찾아** 비교해야 질문이 성립한다.

답할 질문은 하나로 좁혀진다.

> **동일한 saturation criterion 에서 connection parallelism 은 처리량과
> tail latency 에 어떤 영향을 주는가?**

## 4. 2차 범위 (c8~c24) — 결과가 뒤집힌다

conn **1 · 2 · 4** 를 **같은 격자**(c8/12/16/20/24)에서 각 3 run, 총 45 run.
오류율 0.

| conc | conn1 tp | conn1 p95 | conn2 tp | conn2 p95 | conn4 tp | conn4 p95 |
|---:|---:|---:|---:|---:|---:|---:|
| 8 | 112.1 | 101.3 | 120.4 | 93.9 | 111.0 | 105.0 |
| **12** | **114.8** | **147.6** | **136.4** | **119.8** | **138.1** | **121.2** |
| 16 | 115.1 | 191.5 | 136.2 | 178.7 | 138.8 | 210.0 |
| 20 | 114.9 | 239.2 | 135.1 | 245.4 | 138.5 | 306.1 |
| 24 | 115.9 | 286.8 | 134.0 | 307.7 | 139.1 | 392.3 |

### 4.1 운영점은 셋 다 c12 다

98% 규칙(§1) 적용 결과:

| connections | operating conc | throughput | p50 | p95 | p99 | peak 대비 |
|---:|---:|---:|---:|---:|---:|---:|
| **1** | **12** | 114.8 | 102.1 | 147.6 | 167.2 | 99.1% |
| **2** | **12** | **136.4** | 85.8 | **119.8** | **137.4** | 100.0% |
| **4** | **12** | 138.1 | 83.4 | 121.2 | 145.7 | 99.3% |

**시험한 세 커넥션 수(1·2·4) 모두에서 98% 기준 운영 concurrency 가 c12 로
관측됐다.**

> **Within the tested range, the concurrency knee remained invariant to
> connection parallelism.**

커넥션 병렬도를 바꿔도 concurrency knee 가 움직이지 않았다는 증거다. 두
knee 가 서로 독립임을 **강하게 시사하지만 증명한 것은 아니다** — 다른 모델,
페이로드 크기, 노드 수, 네트워크에서는 움직일 수 있다. §0 의 2차원 구조는
이 범위 안에서 관측된 것으로 읽어야 한다.

### 4.2 운영점에서는 트레이드오프가 없다 — conn2 가 conn1 을 지배한다

| conn2 vs conn1 @c12 | |
|---|---:|
| throughput | **+18.8%** |
| p50 | **−16.0%** |
| p95 | **−18.8%** |
| p99 | **−17.8%** |

**처리량이 오르면서 지연이 모든 분위에서 함께 내려간다.** 트레이드오프가
아니라 **strict Pareto improvement** 다 — 단, **측정한 처리량·지연 지표
기준**이다(on the measured throughput/latency metrics). CPU·메모리·커넥션
자원까지 포함한 전 시스템 Pareto 라는 뜻은 아니다.
→ [`fig_sweep_pareto.png`](../results/s37b-operating-point/figures/fig_sweep_pareto.png)

**conn4 가 절대적으로 나쁜 것은 아니다.** 처리량 최고값을 우선한다면 conn4 도
정당한 선택이다(138.1 vs 136.4).

conn2 를 기본 운영점으로 삼는 근거는 "conn4 가 나빠서" 가 아니다.

| conn4 가 더 주는 것 | conn4 가 더 쓰는 것 |
|---|---|
| 처리량 **+1.2%** — 측정 변동(SD ±0.3~1.6)과 가까운 수준 | 커넥션 자원 **2배** |
| | p99 **+6.0%** |

> **2 connections is the lowest-complexity configuration that captures
> nearly all available throughput.**

최소 자원으로 ceiling 을 거의 다 먹기 때문에 conn2 다.

### 4.3 그래서 앞선 "tail 악화" 는 커넥션 탓이 아니었다

S3.6 §4.3 과 S3.7a 는 커넥션을 늘리면 tail 이 나빠진다고 기록했다
(p95 +46%, +43%). **그 측정 자체는 맞지만 해석이 틀렸다.**

그 실험들은 전부 **c32 에서 쟀는데, c32 는 세 구성 모두에게 overload 구간**
이다(운영점이 c12). 즉 그 비교는 "어느 구성이 더 좋은가" 가 아니라
**"어느 구성이 과부하에서 더 완만하게 무너지는가"** 를 잰 것이었다.

```text
c32 에서 본 것   1ch → 4ch :  처리량 +21%, p95 +46%   ← overload 구간 비교
c12 에서 본 것   1ch → 2ch :  처리량 +19%, p95 −19%   ← 운영점 비교
```

**S3.6·S3.7a 의 숫자가 틀린 것이 아니다. 질문이 달랐다.**

| 무엇을 물었나 | 답 |
|---|---|
| **고정 c32 비교** — 각 구성이 **과부하에서** 어떻게 behaving 하는가? | 커넥션이 많을수록 ceiling 은 조금 높지만 **tail amplification 이 커진다** |
| **운영점 비교** — 어느 구성이 **운영상** 더 나은가? | conn2 가 conn1 을 양 축에서 지배한다 |

그래서 c32 결과는 폐기 대상이 아니라 **별도의 유효한 결과**로 남는다 —
과부하 거동에 대한 결과다. 다만 그것을 운영 판단의 근거로 쓰면 안 된다.

> **Optimize at the operating point, not in the overload region.**

운영점을 정의하지 않고 고정 부하에서 구성을 비교하면 **configuration effect
가 아니라 overload behavior 를 보게 되고, 결론이 뒤집힐 수 있다.**
이것이 S3.7 이 남기는 가장 실용적인 교훈이다.

## 5. S3.7b 결론

> **Selected operating point: 커넥션 2개 @ concurrency 12
> — 136.4 inf/s, p95 119.8 ms, p99 137.4 ms**

동일 규칙의 conn1 baseline(114.8 @c12) 대비 **처리량 +18.8%, p95 −18.8%**.
로컬 direct(161.5) 까지의 gap 46.7 중 **21.6 (46%) 를 설정만으로 회수**하면서
tail 도 함께 개선했다.

## 6. Limitations

- **격자 해상도.** knee 는 c8(peak 의 88%)과 c12 사이에 있는데 격자가 4
  단위라 **c12 가 진짜 knee 인지 c10 인지는 모른다.** 세 구성을 같은 격자로
  비교하는 데는 문제없으나, 운영점 절대값으로 인용할 때는 이 한계를 붙인다.
- 분석기가 conn1·conn4 에 "포화 미확인(최고점이 sweep 상단)" 을 찍는다.
  다만 conn1 은 c12 114.8±0.7 vs c24 115.9±0.8, conn4 는 c12 138.1±1.6 vs
  c24 139.1±0.8 로 **평평한 구간 안의 noise** 다. 경고는 보수적으로 남긴다.
- 1노드 결과다. 3노드면 서버가 2×3 = 6 커넥션을 든다(S3.8).
- percentile 은 run-level 평균(S2 §7.4.1).

# S3.7c — RPS at the selected operating point

확정 운영점: **커넥션 2개 @ c12**.

이제 다른 변수를 섞지 않고 질문 하나만 묻는 실험이 된다.

> **Does RPS improve the selected operating point?**

여기서 S3.5b 처럼 다시 null 이 나오면 **그것도 좋은 결과**다. 그때는
"RPS 가 무효였던 건 흐름이 하나뿐이라서" 라는 가설이 상당 부분 약해진다 —
흐름이 2개인데도 변화가 없다면 **IRQ/RX-side 분산이 이 워크로드의 병목이
아니라는 쪽**으로 증거가 쌓인다.

확정된 운영점에서 `rps_cpus` OFF/ON. S3.5b 는 흐름이 하나뿐이라 분산할 대상이
없었다. 이제 흐름이 여러 개이고 S3.6 의 4ch 조건에서 CPU0 는 busy 81% /
soft 74% 였다.

**S3.7b 에서 c2·c4 가 애매하게 비기면 둘 다 RPS A/B 를 한다.** 조건당 10 run
이면 되므로 싸고, **흐름이 2개냐 4개냐에 따라 RPS 효과가 달라질 수 있다** —
그 자체가 ②-a(TCP per-flow 처리)를 겨냥한 정보다.

- 오르면 → 단일 커넥션 제약을 풀자 NIC 처리 병목이 드러난 것
- 그대로면 → CPU0 softirq 는 **상관관계일 뿐 처리량 limiter 가 아니다**
  (S3.5b 단독보다 훨씬 강한 배제)

## 결과 — null 이다. 그리고 이번 null 은 훨씬 강하다

conn2 @ c12 고정, `rps_cpus` = `00`(CPU0만) vs `fe`(코어 1~7), 각 5 run.

| | throughput | p50 | p95 | p99 | 보드 idle | **CPU0 busy** | **CPU0 %soft** |
|---|---:|---:|---:|---:|---:|---:|---:|
| RPS off | **136.8 ± 0.6** | 85.4 | 119.1 | 137.7 | 49.3% | **78.7%** | **68.0%** |
| RPS on | 135.6 ± 0.4 | 86.4 | 119.7 | 139.3 | 49.1% | **74.6%** | **56.0%** |
| 차이 | **−0.8%** | +1.2% | +0.5% | +1.2% | — | −4.1%p | **−12.0%p** |

오류율 0. 처리량 차이 −0.8% 는 SD(±0.4~0.6) 범위다.

### 왜 이 null 이 S3.5b 보다 강한가

S3.5b 때는 반박이 가능했다 — **흐름이 하나뿐이라 RPS 가 분산할 대상 자체가
없었다.** 이번에는 그 반박이 막힌다.

1. **흐름이 2개다.** RPS 가 해시로 나눌 것이 실재한다.
2. **RPS 가 실제로 작동했다.** CPU0 %soft 가 **68.0% → 56.0%** 로 12%p
   내려갔고 CPU0 busy 도 78.7% → 74.6% 로 떨어졌다. 설정이 무시된 것이 아니다.
3. **CPU0 는 놀고 있지 않았다.** busy 78.7% 로 충분히 부하가 걸린 상태였다
   (S3.6 의 c32/4ch 조건 81% 와 비슷하다). "부하가 낮아 RPS 가 손댈 여지가
   없었다" 는 설명도 성립하지 않는다.

> **At the selected operating point, RPS reduced CPU0 softirq load
> substantially but produced no measurable throughput or tail-latency
> improvement. Therefore, CPU0 receive-side processing was not
> performance-limiting under the tested configuration.**

**범위를 정확히 읽어야 한다.** 이 실험이 말하는 것은 "CPU0 softirq 는
limiter 가 아니다" 가 아니라 **"이 운영점·이 구성에서는 limiter 가 아니다"**
다. 다른 부하·모델·페이로드 크기·노드 수에서는 달라질 수 있다.

그 범위 안에서는 상당히 강하다 — mechanism 은 분명히 작동했는데
end-to-end limiter 를 건드리지 못했다. S3.5(§4.3)가 CPU0 을 "다음 병목
후보" 로 지목했던 것은 **이 구성에 한해** 배제된다.

## S3.7 전체 결론

| 후보 | 판정 |
|---|---|
| 링크 대역폭 | 배제 (방향당 51%) — S3.5 |
| 보드 CPU 총량 | 배제 (49~63% idle) — S3.5·S3.7c |
| 서버·스케줄러 | **재개방** — baseline 에서는 배제됐으나 optimized 3N eff 95.3% (S3.8) |
| **CPU0 softirq / RPS** | **배제.** 12%p 덜어도 처리량 불변 — S3.7c |
| H2 flow control window | 64 MB 급 확대는 −36.3% 로 해로움 — S3.6 |
| **노드당 커넥션 수** | **주요 제약.** 1→2 로 +18.8%, tail 도 개선 — S3.7b |
| protobuf·복사·syscall | **미분리.** 남은 15.5% 안에 있을 수 있다 |

**Selected operating point: 노드당 커넥션 2개(2 connections **per node**)
@ concurrency 12 — 136.4 inf/s, p95 119.8 ms, p99 137.4 ms**

> **단위를 반드시 명시한다.** `[transport] node_connections` 는 **노드당**
> 값이다(`GrpcNodePool` 이 `NodeId` 마다 채널을 N 개 만든다). 클러스터 전체
> 합이 아니다.
>
> | 노드 수 | node_connections | 클러스터 전체 커넥션 |
> |---:|---:|---:|
> | 1 | 2 | 2 |
> | 2 | 2 | 4 |
> | 3 | 2 | 6 |
>
> S3.8 에서 "2 connections" 를 클러스터 전체로 고정하면 노드당 조건이
> 보존되지 않고, 3N 에서 **커넥션 공급 자체가 새 병목**이 된다. 완전히
> 다른 실험이 되므로 혼동하면 안 된다.

로컬 direct 161.5 까지 아직 **15.5%** 남아 있다.

> ⚠️ **배제표는 후보 공간을 줄인 것이지, 남은 15.5% 의 정체를 특정한 것이
> 아니다.** 남은 후보는 여전히 여럿이다.
>
> | 남은 gap 의 후보 |
> |---|
> | protobuf serialization |
> | memcpy / buffer ownership (`to_vec()` 등) |
> | syscall / submission path |
> | HTTP/2 구현 오버헤드 |
> | userspace 스케줄링 (tokio 워커 ↔ blocking 풀 경합) |
> | NPU submission / RKNN 런타임 오버헤드 |
> | 그 밖 |

**io_uring 은 이제 정당한 후보가 됐다. 그러나 "다음 병목이 syscall·복사다"
는 아직 아니다.** 그래서 S4 의 질문을 이렇게 둔다.

```text
아니다   io_uring 이 남은 15.5% 를 회수하는가?
맞다     syscall / submission path 가 실제로 유의미한 비용인가?
```

프로파일로 syscall·복사 비용을 먼저 확인하고, 그 답이 "그렇다" 일 때
io_uring 으로 간다. TECHSPEC §15.1 의 순서이자 S3.5 이후 지켜 온 원칙과
같다 — **측정이 구현을 결정한다.**

다음은 **S3.8** — 이 운영점으로 1N/2N/3N scale-out 을 재검증한다.

---

<a id="experiments-s3-8-optimized-scaleout"></a>

# S3.8 — Optimized gRPC Scale-out

- 실험 ID: **S3.8**
- 측정일: 2026-08-20
- 코드: `0af696d` + `[transport] node_connections = 2`
- 상태: **완료 (36 run, 9/9 구성 노드 수 검증 통과)**
- 원본: [`../../results/scaleout-optimized-20260820/`](../results/scaleout-optimized-20260820)
- 선행: [`S3_7_CONNECTION_TUNING.md`](#experiments-s3-7-connection-tuning)

---

## 1. Research Question

> **S3.7 이 찾은 노드당 운영점(커넥션 2개 @ c12)이 scale-out 을 해치지 않는가?**

S3.7 은 **1노드** 결과다. 3노드가 되면 스케줄러가 커넥션을 2×3 = 6개 들고
전송량도 3배가 된다. 서버 쪽에 새 병목이 생길 수 있다.

## 2. Method

- **노드 수마다 concurrency 를 다시 훑어 각자의 운영점을 찾는다.**
  고정 concurrency 로 비교하면 configuration effect 가 아니라 overload
  behavior 를 보게 된다(S3.7 §4.3 에서 실제로 겪었다).
- 커넥션은 **노드당 2개** — 1N→2, 2N→4, 3N→6 total.
- 1N: c8/12/16/20 · 2N: c16/24/32/40 · 3N: c24/36/48/60, 각 **3 run**, 60초.
- rep 마다 노드 수 순서와 concurrency 순서를 모두 회전.
- 운영점 정의는 S3.7 과 동일 — **peak 의 98% 이상을 내는 가장 낮은 concurrency**.

### 2.1 측정 전 노드 수 검증 — 실제로 걸렸다

각 구성마다 짧은 probe bench 를 던져 **응답한 노드 ID 분포**를 세고
`expected ≠ observed` 면 그 구성을 건너뛴다.

**1차 실행에서 2N·3N 6개 구성이 전부 걸렸다.**

```text
!! 노드 수 불일치 — expected=2 observed=1 (king). 이 구성 건너뜀
!! 노드 수 불일치 — expected=3 observed=1 (king). 이 구성 건너뜀
```

원인은 `[transport]` 설정을 쓰는 `npuforge-node.s36` 빌드가 **king 에만**
배포돼 있었던 것이다(S3.6 이후 모든 실험이 1노드라 드러나지 않았다).
기동 로직이 `pgrep || 실행` 이라 파일이 없으면 조용히 실패한다.

> **이 검증이 없었다면 1N 을 세 번 재고 "2N·3N" 으로 기록했을 것이다.**
> 그 결과는 2N 136, 3N 136 — "scale-out 이 완전히 무너졌다" 는 정반대
> 결론이 나왔을 것이다. **프로세스가 떠 있다 ≠ 트래픽을 받는다.**

세 보드에 배포 후(해시 `73227f64…` 동일) 재실행 — **9/9 구성 검증 통과**.

## 3. Results

오류율 전 구간 **0**, 노드 간 분배 편차 전 구간 **0.0%p**.
지연은 run-level percentile 의 run 간 평균(pooled 아님, S2 §7.4.1).

### 3.1 노드 수별 곡선

| conc | 1N tp | 1N p95 | | conc | 2N tp | 2N p95 | | conc | 3N tp | 3N p95 |
|---:|---:|---:|---|---:|---:|---:|---|---:|---:|---:|
| 8 | 120.2 | 94.3 | | 16 | 239.9 | 95.5 | | 24 | 354.3 | 98.3 |
| **12** | **135.5** | **120.7** | | **24** | **263.3** | **140.7** | | **36** | **387.2** | **151.1** |
| 16 | 137.4 | 175.8 | | 32 | 262.5 | 237.0 | | 48 | 385.0 | 288.7 |
| 20 | 134.9 | 245.0 | | 40 | 260.6 | 349.7 | | 60 | 385.8 | 407.3 |

세 구성 모두 **범위 안에서 포화가 확인됐다**(peak 양쪽이 더 낮다).
노드당 concurrency 는 셋 다 12 로 같다 — S3.7b 의 knee 가 다노드에서도 유지된다.

### 3.2 운영점 비교

| Nodes | Tot.conn | Op.conc | Throughput | p95 | p99 | Scaling | Efficiency |
|---:|---:|---:|---:|---:|---:|---:|---:|
| **1** | 2 | 12 | **135.5** | 120.7 | 141.0 | 1.00× | 100.0% |
| **2** | 4 | 24 | **263.3** | 140.7 | 172.7 | **1.94×** | **97.1%** |
| **3** | 6 | 36 | **387.2** | 151.1 | 201.6 | **2.86×** | **95.3%** |

### 3.3 baseline 대비

| | baseline (S3 ceiling, conn1) | optimized (S3.8, conn2/node) | 개선 | per-node |
|---|---:|---:|---:|---:|
| 1N | 115.2 | **135.5** | **+17.6%** | 135.5 |
| 2N | 232.0 | **263.3** | **+13.5%** | 131.7 |
| 3N | 341.8 | **387.2** | **+13.3%** | 129.1 |
| scaling | 2.97× (eff 98.9%) | **2.86× (eff 95.3%)** | | |

## 4. Interpretation

### 4.1 절대 처리량은 올랐다 — 3노드 +13.3%

**387.2 inf/s.** 커넥션 설정 한 줄로 얻었고, 오류 0·분배 편차 0.0%p 로
scale-out 자체는 건강하다.

### 4.2 그러나 scaling efficiency 는 조금 내려갔다 (98.9% → 95.3%)

**이 점을 좋게 포장하지 않는다.** 노드당 이득이 노드 수에 따라 줄어든다.

```text
1N  +17.6%   (115.2 → 135.5)
2N  +13.5%   (232.0 → 263.3)
3N  +13.3%   (341.8 → 387.2)
```

노드당 처리량으로 보면 **135.5 → 131.7 → 129.1** 로 단조 감소한다.
1노드 최적화의 효과가 다노드에서 **온전히 보존되지 않는다.**

이상적 3N(135.5×3 = 406.5) 대비 실제 387.2 — **19.3 inf/s 부족**하다.

### 4.3 유력한 후보: 서버 쪽

> ⚠️ **[2026-08-21 철회 — S3.9a]** 아래 "10G 76%" 는 **계산 오류**다.
> **10GbE 는 full-duplex** 라 요청(TX)과 응답(RX)이 각자의 10 Gbps 를 쓰는데,
> 둘을 한 링크 예산에 합산했다. 실측은 **방향당 40.5%** 다(S3.9a §3).
>
> S3.9a 가 서버 자원을 모두 배제했다 — CPU 42%, 링크 방향당 40%, drop 0,
> 스레드 직렬화 없음. **손실은 전적으로 tail 증가**이며 p50 은 평평하다.
> → [S3.9a](#experiments-s3-9a-scaleout-profile)

~~추론당 스케줄러↔노드 전송량은 2,446,800 byte 다. 3N 운영점에서~~

| | baseline 3N | optimized 3N |
|---|---:|---:|
| 처리량 | 341.8 | **387.2** |
| ~~서버 NIC 부하~~ | ~~6.69 Gbps~~ | ~~7.58 Gbps~~ |
| ~~10G 링크 대비~~ | ~~67%~~ | ~~76%~~ |

**위 두 줄은 철회한다.** 양방향 합산은 full-duplex 링크에 적용할 수 없다.

### 4.4 지연은 노드 수에 따라 늘어난다

노드당 부하가 셋 다 12 로 같은데도 운영점 p95 가
**120.7 → 140.7 → 151.1 ms** 로 늘어난다. 노드당 조건이 동일하므로
이 증가분은 **스케줄러 팬아웃 경로**에서 온다고 볼 수 있다. 다만 어느
단계인지는 분해하지 않았다.

## 5. Limitations

- **efficiency 하락의 원인은 미확인**(§4.3). 서버 NIC·CPU·스케줄러 팬아웃 중
  무엇인지 분리하지 않았다. S3.5 와 같은 방식의 **서버 쪽 프로파일**이 필요하다.
- 60초 측정이다. **throttling 발현 전 구간**이므로 지속 부하(S0)에서는
  다를 수 있다.
- concurrency 격자가 성기다(1N 4단위, 3N 12단위). 운영점의 절대값보다
  구성 간 비교에 쓴다.
- percentile 은 run-level 평균(S2 §7.4.1).
- 2노드 조합은 king+queen 하나만 봤다.

## 6. Reproduction

```bash
bash scripts/run-scaleout-optimized.sh 3     # 36 run, 약 50분
PYTHONIOENCODING=utf-8 python scripts/analyze-scaleout.py \
    results/scaleout-optimized-20260820/raw/results.csv
```

> 세 보드에 `npuforge-node.s36`(= `[transport]` 지원 빌드)이 배포돼 있어야
> 한다. 없으면 노드 수 검증이 그 구성을 건너뛴다 — 조용히 틀린 결과를 내는
> 대신 큰 소리로 멈춘다.

## 7. Conclusion

**노드당 운영점(커넥션 2개 @ c12)은 3노드까지 유지되며, 절대 처리량을
341.8 → 387.2 inf/s (+13.3%) 로 끌어올린다.** 오류 0, 분배 균등.

다만 **scaling efficiency 는 98.9% → 95.3% 로 소폭 내려갔고**, 노드당
이득이 +17.6%(1N) → +13.3%(3N) 으로 줄어든다. 1노드 최적화가 다노드에서
온전히 보존되지 않는다는 뜻이다. ~~서버 10G 링크가 76% 까지~~ — **S3.9a 에서 철회**(full-duplex 계산 오류,
실제 방향당 40%). S3.9a 는 서버 자원을 모두 배제하고 손실이 **tail 증가**임을
확인했다.

→ 다음은 **서버 쪽 프로파일**이다. 노드 쪽은 S3.5 에서 했고, 이번엔 서버가
후보로 올라왔다. 그 결과가 나오기 전에는 남은 gap 을 노드 쪽 비용
(protobuf·복사·syscall)으로만 돌리면 안 된다.

---

## Figure

![절대값은 모든 규모에서 오르고 efficiency 는 98.9% → 95.3%](../results/scaleout-optimized-20260820/figures/fig_scaleout_optimized.png)

**`fig_scaleout_optimized.png`** — 절대값은 모든 규모에서 오르고 efficiency 는 98.9% → 95.3%

재생성: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s3-9a-scaleout-profile"></a>

# S3.9a — Scale-out Efficiency Loss Profiling

- 실험 ID: **S3.9a**
- 측정일: 2026-08-21
- 코드: `e1ad9ed` + `[transport] node_connections = 2`
- 상태: **완료 (9 run, 3구성 × 3회, 9/9 노드 수 검증 통과)**
- 원본: [`../../results/scaleout-profile-20260821/`](../results/scaleout-profile-20260821)
- 선행: [`S3_8_OPTIMIZED_SCALEOUT.md`](#experiments-s3-8-optimized-scaleout)

---

## 1. Research Question

> **optimized 3N 에서 사라진 약 4.5% efficiency 가 shared path 의 어디서 생기는가?**

범위를 넓히지 않는다. **새 sweep 을 섞지 않고** S3.8 이 찾아 둔 각자의
운영점을 그대로 쓴다 — 1N@c12 / 2N@c24 / 3N@c36. 셋 다 노드당 커넥션 2개,
노드당 c12 다. **달라지는 것은 노드 수뿐이다.**

## 2. Method

- 서버와 king 을 **동시에** 프로파일한다. "서버가 늘었나 노드가 줄었나" 는
  나란히 봐야 구분된다.
- 서버에 sysstat 이 없다(mpstat/pidstat/sar/perf 전부). **설치하지 않고**
  `/proc/stat` 24코어 델타로 직접 계산했다 — 측정 캠페인 중 환경을 바꾸지
  않기 위해서다.
- 수집: 코어별 busy·softirq, NIC RX/TX·drop, IRQ/softirq 분포,
  **스케줄러 스레드별 CPU**, `ss -tin`(rtt·cwnd·retrans), 노드별 분배.
- 스크립트: [`run-scaleout-profile.sh`](../scripts/run-scaleout-profile.sh),
  [`server-profile-collect.sh`](../scripts/server-profile-collect.sh),
  [`analyze-scaleout-profile.py`](../scripts/analyze-scaleout-profile.py).

## 3. ⚠️ 먼저, S3.8 의 유력 후보를 철회한다

S3.8 은 **"서버 10G 링크가 76% 까지 올라왔다"** 를 efficiency 하락의 유력
후보로 지목했다. **그 계산이 틀렸다.**

```text
내가 쓴 것   387.2 inf/s × 2,446,800 byte × 8 = 7.58 Gbps → "10G 의 76%"
```

**10GbE 는 full-duplex 다.** 요청(TX)과 응답(RX)은 각자의 10 Gbps 를 쓴다.
둘을 한 링크 예산에 합산하면 안 된다.

| | TX (요청) | RX (응답) | 방향당 10G 대비 |
|---|---:|---:|---:|
| 1N | 1.34 Gbps | 1.33 Gbps | 13.4% |
| 2N | 2.61 | 2.59 | 26.1% |
| **3N** | **3.84** | **3.80** | **38.4%** |

실측(`/proc/net/dev`)도 3N 에서 RX 3.997 / TX 4.048 Gbps, **방향당 40.5%** 다.
76% 가 아니라 **40%**. **서버 10G 링크는 병목이 아니다.**

> 세션 초반에 보드의 2.5GbE 에 대해서는 full-duplex 를 정확히 따졌는데
> ([S3.5 §4.1](#experiments-s3-5-transport-profile)), 서버에서 같은 실수를 했다.
> 같은 함정은 축이 바뀌면 다시 밟힌다.

## 4. Results

오류율 0, 분배 편차 0.0%p, 9/9 노드 수 검증 통과.

### 4.1 efficiency 손실 = 평균 지연 증가, 정확히 일치한다

| N | conc | throughput | **mean** | p50 | p95 | p99 | Efficiency | mean 증가 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 12 | 136.2 | 87.63 | **85.9** | 119.7 | 137.9 | **100.0%** | +0.0% |
| 2 | 24 | 265.6 | 89.79 | **86.0** | 136.6 | 169.0 | **97.5%** | **+2.5%** |
| 3 | 36 | 390.3 | 91.55 | **85.9** | 147.4 | 187.6 | **95.5%** | **+4.5%** |

closed-loop 에서 처리량 = concurrency / mean latency 이므로 이 일치는
항등식이다. 중요한 것은 **그 평균이 어디서 늘었는가** 다.

> **p50 은 완전히 평평하다 — 85.9 / 86.0 / 85.9 ms.**
> 요청 하나를 처리하는 데 드는 일 자체는 노드 수와 무관하다.
>
> **평균을 끌어올린 것은 전적으로 tail 이다.**
> p95 +23% (119.7 → 147.4), p99 **+36%** (137.9 → 187.6).

단계 분해도 같은 말을 한다(p50, ms):

| | e2e | inference | →node | →client | payload 합 | non-inference |
|---|---:|---:|---:|---:|---:|---:|
| 1N | 79.07 | 32.77 | 22.46 | 22.46 | 44.91 | 46.30 |
| 2N | 77.92 | 30.32 | 22.91 | 22.91 | 45.82 | 47.60 |
| 3N | 76.16 | 29.33 | 22.33 | 22.33 | 44.66 | 46.84 |

**어느 단계도 노드 수에 따라 늘지 않는다.** 스케줄러 큐·라우팅은 ~0
(0.000~0.004 ms), 노드 큐도 0.022 ms 로 고정이다.

### 4.2 서버 자원은 어디도 포화하지 않았다

| N | busy 코어 | 최번 코어 | softirq 코어 | RX Gbps | TX Gbps | 방향당 10G | drop | schedCPU | sysc/req |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 2.81 | 18.4% | 0.51 | 1.390 | 1.408 | 14.1% | **0** | 1.57 | 164.7 |
| 2 | 6.03 | 29.6% | 1.04 | 2.712 | 2.749 | 27.5% | **0** | 3.32 | 164.5 |
| 3 | **10.12** / 24 | **47.6%** | 1.67 | 3.997 | 4.048 | **40.5%** | **0** | 5.62 | 164.2 |

- **CPU**: 24코어 중 10.12 사용(42%). 가장 바쁜 코어도 47.6%.
- **링크**: 방향당 40.5%. **drop 0.**
- **스레드 직렬화 없음**: 3N 상위 5 스레드가 37/35/34/31/30% 로 고르다.
  단일 스레드가 튀는 직렬화 지점이 **없다**.
- **syscalls/req 불변**: 164.7 / 164.5 / 164.2.

> 예상대로 서버는 RX 큐가 24개(보드는 1개)라 RSS 로 분산된다. 보드에서
> 걸렸던 "단일 큐 + CPU0 편중" 문제는 서버에 없다.

### 4.3 그러나 두 가지가 노드 수와 함께 자란다

**(a) 서버 CPU per request 가 +26%**

```text
1N  2.81 코어 / 136.2 inf/s = 20.6 ms·core/req
2N  6.03 / 265.5            = 22.7
3N 10.12 / 390.3            = 25.9      (+26%)
```

포화는 아니지만(42%) **초선형**이다 — 노드 3배에 CPU 3.60배.

**(b) TCP 재전송률이 커넥션당 3.5배**

`ss -tin` 원본에서 커넥션당 재전송 바이트 비율:

| | bytes_sent | bytes_retrans | 재전송률 | cwnd | ssthresh |
|---|---:|---:|---:|---:|---:|
| 1N | 3.05 GB | 1.67 MB | **0.055%** | **176** | 138 |
| 3N | 2.95 GB | 5.57 MB | **0.189%** | 118 | 103 |
| 3N | 2.94 GB | 5.93 MB | 0.201% | 119 | 66 |
| 3N | 2.95 GB | 5.33 MB | 0.181% | 106 | 59 |

**커넥션당 전송량은 비슷한데(≈3 GB) 재전송률만 3.5배**다. 그리고 혼잡
윈도가 눌려 있다 — cwnd 176 → 106~119, ssthresh 138 → 59~103.

## 5. Interpretation

정리하면 이렇다.

```text
efficiency 손실  =  평균 지연 증가  =  전적으로 tail 증가
                    (p50 은 완전히 평평)

서버 자원         포화 없음 (CPU 42%, 링크 40%/방향, drop 0, 스레드 고름)
서버 per-req 비용  +26% (초선형이지만 포화는 아님)
TCP 재전송률       커넥션당 3.5배, cwnd 눌림
```

**유력 가설(미검증): 공유 경로의 혼잡.** 서버는 10G 한 포트에서 나가고
보드는 2.5G 세 포트로 받는다. 총량은 링크 상한에 한참 못 미치지만,
**10G → 2.5G 속도 불일치**는 스위치 egress 에서 버퍼링을 만든다. 노드 수가
늘수록 스위치 fabric 을 지나는 총 트래픽이 늘고, TCP 는 재전송·cwnd 축소로
반응한다. 재전송은 중앙값을 거의 안 건드리고 **tail 만 밀어 올린다** —
관측된 신호(p50 평평, p95/p99 증가)와 정확히 맞는 모양이다.

**그러나 이것은 정합성이지 증명이 아니다.** 스위치 쪽 카운터를 보지 않았고,
재전송이 원인인지 같은 원인의 다른 증상인지 가르지 않았다.

## 6. 후보 현황 갱신

| 후보 | S3.8 시점 | **S3.9a 이후** |
|---|---|---|
| 서버 10G 링크 | 유력 후보 (76%) | **철회** — 계산 오류. 방향당 40% |
| 서버 CPU 포화 | 후보 | **배제** — 42%, 최번 코어 47.6% (**24스레드 호스트에서**. §7 참조) |
| 스케줄러 직렬화 | 후보 | **배제** — 스레드 CPU 고름 |
| 서버 NIC drop | 후보 | **배제** — drop 0 |
| **공유 경로 혼잡 (10G→2.5G)** | — | **신규 유력** — 재전송 3.5배, cwnd 눌림 |
| 서버 per-req CPU 초선형 | — | **열림** — +26%, 포화는 아님 |

## 7. Limitations

- **혼잡 가설 미검증.** 스위치 카운터(포트별 drop·pause·buffer)를 보지 않았다.
- `ss` 의 재전송·cwnd 는 **커넥션 수립 이후 누적값**이다. 통제된 창의 값이
  아니므로 절대 비교보다 **비율 비교**로만 썼다.
- 노드 쪽 프로파일(king)도 함께 수집했으나 이 문서는 서버 축만 다룬다.
- **"서버 CPU 배제" 는 그 호스트에서의 판정이다 (2026-08-26 추가).**
  스케줄러 호스트를 24스레드에서 8스레드로 교체하자 같은 부하에서 서버 CPU 가
  **42% → 82.2%** 가 되고 기준선이 **391 → 360 inf/s** 로 내려갔다.
  이 문서의 측정과 결론은 24스레드 호스트에서 유효하며 **그대로 둔다.**
  다른 호스트에서 재현할 때는 서버 CPU 사용률을 함께 본다.
  → `../infrastructure.md` §3.2.1
- 60초 측정 — throttling 전 구간.
- percentile 은 run-level 평균(S2 §7.4.1).

## 8. Reproduction

```bash
bash scripts/run-scaleout-profile.sh 3     # 9 run, 약 15분
PYTHONIOENCODING=utf-8 python scripts/analyze-scaleout-profile.py \
    results/scaleout-profile-20260821
```

## 9. Conclusion

**3N 의 4.5% efficiency 손실은 서버 자원 포화 때문이 아니다.** CPU 42%,
링크 방향당 40%, drop 0, 스레드 직렬화 없음, syscalls/req 불변이다.
S3.8 이 지목했던 "10G 76%" 는 **full-duplex 를 무시한 계산 오류**로 철회한다.

손실의 정체는 **전적으로 tail 증가**다. p50 은 85.9 ms 로 완전히 평평한데
p95 가 +23%, p99 가 +36% 늘고, 그만큼 평균이 올라 closed-loop 처리량이
깎인다. 단계 분해 어디에도 증가가 없다.

동반 신호는 **커넥션당 TCP 재전송률 3.5배와 cwnd 축소**다. 서버는 10G 한
포트, 보드는 2.5G 세 포트라 **속도 불일치로 인한 공유 경로 혼잡**이 유력한
가설이지만 **검증되지 않았다.**

→ 다음은 **S0(지속 부하)** 다. 혼잡 가설 검증은 스위치 카운터 접근이 필요해
별도 준비가 든다. 그보다 **현재 운영점이 30분에서도 유효한지**가 먼저다 —
지금 결과는 전부 60초 구간이다.

---

## Figure

![p50 평평(+0%), p95 +23%, p99 +36% — 손실은 전적으로 tail](../results/scaleout-profile-20260821/figures/fig_efficiency_loss_is_tail.png)

**`fig_efficiency_loss_is_tail.png`** — p50 평평(+0%), p95 +23%, p99 +36% — 손실은 전적으로 tail

재생성: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s3-9b-node-residual"></a>

# S3.9b — Node-side Residual Cost Profiling

- 실험 ID: **S3.9b**
- 측정일: 2026-08-21
- 코드: `62855bd`
- 상태: **완료** (4 조건 × 45초 수집, 오류 0)
- 원본: [`../../results/node-residual-20260821/`](../results/node-residual-20260821)
- 선행: [`S3_5_TRANSPORT_PROFILE.md`](#experiments-s3-5-transport-profile) ·
  [`S3_9A_SCALEOUT_PROFILE.md`](#experiments-s3-9a-scaleout-profile)

---

## 1. Research Question (좁게)

> **161.5 → 135.5 사이의 residual gap 에서 node-side serialization /
> copy / syscall 비용이 유의미한 비중을 차지하는가?**

**gap 전체를 설명하는 것이 목적이 아니다.** S3.9a 에서 scale-out
tail/TCP 쪽 비용이 별도로 드러났으므로 node-side 프로파일이 26.0 inf/s
전부를 설명해야 할 이유가 없다. 설명 못 한 잔여는 잔여로 남긴다.

판정 규칙은 **측정 전에** 정했다.

| 결과 | 결정 |
|---|---|
| syscall·copy 가 **충분히 큼** | S4 io_uring 진입 |
| **작음** | **S4 취소/보류** |
| **다른 항이 큼** | 그 항만 기록. 핵심 범위 밖이면 더 안 판다 |

## 2. Method

S3.5 와의 결정적 차이는 **운영점에서 잰다**는 것이다.

```text
S3.5    c32 · conn1   116.6 inf/s   과부하 · baseline
S3.9b   c12 · conn2   136.6 inf/s   운영점 · optimized
```

과부하 구간 값을 운영 판단에 쓰지 않는다(README §4.1). 이 저장소는 같은
함정에 이미 한 번 걸렸다 — 13.2% 오인용 사건.

- 1노드(king only). queen·jack 을 내려 RR 이 나눠 가지 못하게 하고,
  probe 로 **응답한 노드 ID 가 king 하나**임을 물증으로 남긴다.
- 부하 80초 중 **t+20 부터 45초**만 수집. 램프와 warmup 을 뺀다.
- 조건 4개: `idle`(계측기 바닥값) / `op`(운영점) / `strace` / `local`(direct 8스레드).

### 2.1 계기 선택 — perf 가 없다

보드에 `perf` · `bpftrace` · `gdb` 가 없다(커널 6.1.141, 벤더 트리).
심볼 단위 프로파일은 불가능하다. 대신 **`/proc/PID/stat` 의 utime/stime
분리**를 쓴다.

```text
utime  유저 시간 — protobuf 직렬화, 유저공간 copy, HTTP/2 프레이밍
stime  커널 시간 — syscall 진입, TCP 스택, copy_to_user, skb, 드라이버
```

**io_uring 이 줄이는 것은 stime 의 일부다.** 따라서 stime 전체가
io_uring 의 절대 상한이고, 실제 회수 가능분은 그보다 작다.

보조로 `strace -c` 를 10초. ptrace 가 syscall 마다 정지시켜 체류시간이
**부풀려져** 나오므로 **상한으로만** 쓴다 — 부풀린 값이 작으면 실제는
확정적으로 더 작다. 한쪽 방향으로만 유효한 검정이다.

## 3. Results

### 3.1 요청당 노드 CPU

| 조건 | throughput | utime/req | stime/req | **CPU-ms/req** | user% | kernel% |
|---|---:|---:|---:|---:|---:|---:|
| op (운영점) | 136.6 | 14.50 | 11.09 | **25.59** | 56.7 | 43.3 |
| local direct | 157.9 | 5.14 | 4.10 | **9.23** | 55.6 | 44.4 |
| **transport 비용** | | **9.37** | **6.99** | **16.35** | **57.3** | **42.7** |

운영점 136.6 은 S3.8 의 135.5±0.4, S3.7b 의 136.4±0.3 과 일치한다 —
조건이 제대로 잡혔다는 확인이다.

> ⚠️ local 의 157.9 는 80초 **전체 평균**이라 램프를 포함한다. 수집 창
> (t+20~65)의 정상 구간 속도는 162.6 이었다. 이 차이는 local 의
> 요청당 CPU 를 약 3% 과대평가하는 방향이며, **transport 비용을 과소가
> 아니라 과대 추정**하므로 아래 결론(비용이 작다)을 약화시키지 않는다.

### 3.2 어느 코어도 포화가 아니다

```text
op    cpu0  soft=68.3  idle=21.2   ← 유일한 뜨거운 코어 (78.8% busy)
      cpu1~3            idle 61~64
      cpu4~7            idle 42~47
      전체              idle 48.9
local 전체              idle 82.5   softirq 0
```

가장 뜨거운 cpu0 도 21% 남는다. cpu0 의 부하는 대부분 **softirq**
(NIC 단일 수신 큐)인데, **S3.5 §4.3 이 이미 RPS 로 분산시켜 보고
−0.2% null 을 얻었다.** cpu0 softirq 도 제약이 아니다.

### 3.3 syscall — 횟수는 많고 비용은 작다

`strace -c` 10초 (요청 약 1,284건):

| syscall | 체류시간 | 호출 수 | calls/req | |
|---|---:|---:|---:|---|
| futex | 30.07s | 48,565 | 37.8 | 스레드 동기화 **대기** |
| ioctl | 24.72s | 68,924 | 53.7 | RKNN 드라이버 (NPU 제출) |
| epoll_pwait | 9.78s | 37,157 | 28.9 | 이벤트 **대기** |
| **recvfrom** | 9.50s | 136,602 | **106.4** | 요청 수신 ← io_uring 대상 |
| **writev** | 5.91s | 69,245 | **53.9** | 응답 송신 ← io_uring 대상 |
| **write** | 0.35s | 5,524 | **4.3** | 응답 송신 ← io_uring 대상 |

**네트워크 syscall 체류시간은 15.77s / 80.36s = 19.6%** 다. 나머지
80.4% 는 futex(동기화 대기) · ioctl(NPU 드라이버) · epoll(이벤트 대기)로,
**io_uring 이 손대는 영역이 아니다.**

## 4. 판정 — **S4 io_uring 취소/보류**

요청당 네트워크 syscall 은 약 **165회**(recvfrom 106 + writev 54 + write 4).
aarch64 syscall 진입 비용을 **넉넉히 1 µs** 로 잡아도

```text
165 회 × 1 µs = 0.165 ms/req
0.165 / 16.35 = 요청당 transport CPU 의 1.0%
```

등록 버퍼로 1.2 MB copy 를 양방향 모두 없앤다고 **가정해도**(RK3576
메모리 대역폭 기준 약 0.6~1.2 ms) 합계는 **1.4 ms/req ≈ transport
비용의 8%** 다.

그리고 그 8% 를 다 회수해도 **처리량은 오르지 않는다.** 보드 CPU 가
48.9% idle 이고 어느 코어도 포화가 아니며, 가장 뜨거운 cpu0 의 softirq
는 RPS 로 분산해도 −0.2% null 이었기 때문이다.

> **CPU-ms/req 는 비용이지 제약이 아니다.** 포화되지 않은 자원의
> 사용량을 줄이는 것은 처리량을 올리지 않는다.

```text
질문   io_uring 이 남은 16.1% 를 회수하는가?
답     아니다. 회수 대상(syscall 진입)이 transport 비용의 1%,
       가장 관대한 가정으로도 8%. 게다가 CPU 는 제약이 아니다.
```

**S4 는 취소/보류한다.** TECHSPEC §15 의 io_uring 항목은 "필요성 미증명"
이 아니라 **"측정으로 반박됨"** 으로 상태가 바뀐다.

## 5. 판정 규칙 3번째 가지 — 큰 항은 따로 기록한다

질문은 "serialization / copy / syscall" 셋을 묶어 물었는데, 답이 갈렸다.

| 항 | 크기 | 판정 |
|---|---|---|
| **syscall** | transport 비용의 ~1% | **작다** |
| **serialization / 유저공간 copy** | **9.37 ms/req = 57%** | **크다** |

**유저 시간이 커널 시간보다 크다**(9.37 vs 6.99). transport 비용의
과반이 protobuf 직렬화·유저공간 copy·HTTP/2 프레이밍이다. io_uring 은
이쪽을 건드리지 않는다.

다만 **여기서 멈춘다.** 사전 판정 규칙의 3번째 가지대로다 — 큰 항을
기록하되, CPU 가 제약이 아닌 이상 이것을 줄이는 것도 처리량을 올린다는
보장이 없다. 파고들 근거가 아직 없다.

## 6. 그러면 gap 26.0 inf/s 는 무엇인가 — 범위 밖, 관측만 남긴다

이 실험의 임무가 아니지만 방향은 관측된다. 고정 동시성에서
처리량 = 동시성 / 지연이다.

```text
op     c12,  136.6 inf/s  ->  평균 지연 87.8 ms
local  8스레드, 157.9      ->  평균 지연 50.5 ms   (래퍼 실측 50,531 µs)
                              차이 +37.3 ms
```

그중 노드 CPU 작업은 16.35 ms 뿐이고 나머지는 **대기**다. 페이로드가
요청 1.2 MB · 응답 1.2 MB 이므로 실측 링크(2.34 Gbps ≈ 292 MB/s)에서
**순수 전송 시간만 방향당 약 4.1 ms, 왕복 8.2 ms** 다. 여기에 스케줄러
홉과 큐잉이 더해진다.

> gap 은 CPU 비용이 아니라 **경로 지연**의 문제로 보인다. 이것을 줄이는
> 지렛대는 io_uring 이 아니라 **페이로드 크기**다(ADR-008 의 640×640×3
> raw 전송). 다만 이는 S3.9b 의 범위 밖이므로 **관측으로만 남긴다.**

## 7. Limitations

- 조건당 1 run(45초 수집)이다. utime/stime 델타는 45초 누적이라 안정적
  이지만 run 간 SD 는 없다.
- `strace -c` 의 seconds 는 **블로킹 포함 체류시간**이지 CPU 시간이
  아니다. futex·epoll 이 상위인 것은 그 때문이며, 네트워크 syscall
  비중 19.6% 도 같은 척도 안에서만 유효하다. 판정의 주 근거는
  utime/stime 분리이고 strace 는 보조다.
- syscall 진입 비용 1 µs 는 실측이 아니라 aarch64 통상값을 **넉넉히**
  잡은 것이다. 실측하려면 마이크로벤치가 필요하나, 1 µs 가정에서 이미
  1% 이므로 결론이 뒤집히지 않는다.
- local direct 는 `sustained_load_test`(별도 바이너리)라 노드와 코드
  경로가 완전히 같지 않다. 비교의 기준선으로서 S3.5 이래 일관되게 쓴 값이다.

## 8. 이 실험에서 잡은 계기 오류

`strace -c` 요약을 파싱하는 정규식이 **`usecs/call` 과 `calls` 컬럼을
뒤바꿔** 읽었다. 호출 수가 100배 작게 나와 "strace 가 한 스레드에만
붙었다 → 상한 검정 무효" 로 판단할 뻔했다. 기대치(요청당 write 83.4회,
`/proc/PID/io`)와 대조해 잡았다.

> 계측기의 출력이 예상과 다르면 **계측기부터 의심한다**(README §4.10).
> 이번엔 측정이 아니라 파서가 틀렸다.

---

## Figure

![transport 비용의 유저/커널 분해와 io_uring 이 닿는 몫(≈8%)](../results/node-residual-20260821/figures/fig_transport_cost_split.png)

**`fig_transport_cost_split.png`** — transport 비용의 유저/커널 분해와 io_uring 이 닿는 몫(≈8%)

재생성: `python scripts/make-experiment-figures.py`

---

<a id="experiments-s3-saturation"></a>

# S3 — Per-configuration Saturation

- 실험 ID: **S3**
- 측정일: 2026-08-20
- 동결 commit: `1da69d4` (bench `254d560` 코드, S2 와 동일). 측정 중 무변경
- 상태: **완료 (45 runs)**
- 원본: [`../../results/saturation-20260820/raw/`](../results/saturation-20260820/raw) · 그래프: [`figures/fig3`](../results/saturation-20260820/figures/fig3_saturation_sweep.png)
- 선행: [`S2_GRPC_BASELINE.md`](#experiments-s2-grpc-baseline)

---

## 1. Research Question

> **What is the maximum sustainable throughput (ceiling) of each cluster
> configuration, and at what concurrency is it reached?**

**S2 와 다른 질문이다.** S2 는 *동일 노드당 부하*(c = 8×N)에서 선형성을 봤다.
S3 는 각 구성(1/2/3 node)의 **진짜 상한**을 concurrency 를 올려 탐색한다.
두 실험을 섞지 않는다.

## 2. Method

- concurrency sweep (노드당 부하를 넘겨 포화점까지):
  ```text
  1 node : c4, c8, c16, c32, c48
  2 node : c8, c16, c24, c32, c48
  3 node : c12, c24, c32, c48, c64
  ```
- 각 point **3 runs, 30초**. 조건 순서 rotate. 총 45 runs.
- 조건 고정은 S2 와 동일(INT8, want_float=0, performance, Active Cooling,
  round-robin, worker 8, gRPC). 동결 유지.
- 스크립트: [`scripts/run-saturation-sweep.sh`](../scripts/run-saturation-sweep.sh).

## 3. Results — Saturation Curves

3 runs 평균 (inf/s), SD 는 전부 ≤ 2.2:

| concurrency | 1 node | 2 node | 3 node |
|---:|---:|---:|---:|
| c4 | 84.0 | | |
| c8 | 112.6 | 168.3 | |
| c12 | | | 252.2 |
| c16 | 113.8 | 228.1 | |
| c24 | | **232.0** | 339.4 |
| c32 | **115.2** | 230.2 | **341.8** |
| c48 | 114.1 | 230.3 | 339.2 |
| c64 | | | 335.9 |

**Ceilings:**

| Config | Ceiling | @ concurrency | per-node concurrency |
|---|---:|---:|---:|
| 1 node | **115.2** inf/s | c32 | 32 |
| 2 node | **232.0** inf/s | c24 | 12 |
| 3 node | **341.8** inf/s | c32 | ~11 |

→ [Figure 3](../results/saturation-20260820/figures/fig3_saturation_sweep.png)

## 4. Interpretation

**Finding — near-linear even at the ceiling.**

| Config | Ceiling | Speedup (vs 1-node ceiling) | Efficiency |
|---|---:|---:|---:|
| 1 node | 115.2 | 1.00× | 100% |
| 2 node | 232.0 | 2.01× | 101% |
| 3 node | 341.8 | **2.97×** | **99%** |

S2 는 동일 부하에서의 선형성을, S3 는 최대 처리량에서의 선형성을 보였다.
**두 각도에서 독립적으로 near-linear scaling 이 확인된다.** 3-node 상한
341.8 inf/s 는 1-node 상한의 2.97배다.

곡선의 세 구간:
- **낮은 concurrency (미포화):** 왕복 지연(≈68 ms, S2 §7.4)에 막혀 처리량이
  낮다. closed-loop 이라 동시 요청이 적으면 파이프라인이 비는 구간이다
  (1N c4 = 84, 3N c12 = 252).
- **plateau (포화):** 노드당 ~10–16 동시에서 최대. worker 8 을 파이프라인이
  채우고 나면 더 올려도 오르지 않는다.
- **과부하 (살짝 하락):** 더 올리면 큐잉만 늘어 소폭 감소(3N c32 341.8 →
  c64 335.9). 오류는 여전히 0 — 스케줄러/노드 큐가 흡수한다.

## 5. Limitations

- S2 와 동일: 측정 시간 짧음(30초, throttling 전), Active Cooling 만,
  closed-loop, 2노드 조합 하나(king+queen).
- **S2 대비 duration 이 다르다(30 vs 60초).** ceiling 값(115/232/342)은
  S2 의 c8/c16/c24(112.9/229.0/338.4)와 근접하나 완전 동일 조건은 아니다.
  saturation 은 곡선 형태와 상한 위치가 목적이며, 절대값은 S2 를 우선한다.
- ceiling 을 넘는 과부하 하락은 closed-loop 큐잉 효과다 — 열린 모델에서는
  다르게 나타날 수 있다([`adrs/028`](../adrs/028-bench-run-validity.md)).

## 6. Reproduction

```bash
bash scripts/run-saturation-sweep.sh    # 45 run → server:/tmp/sat30
python scripts/make-figures.py          # Figure 3 재생성
```
동결 commit `1da69d4`.

## 7. Raw Data & Conclusion

- 원본 45건: [`../../results/saturation-20260820/raw/`](../results/saturation-20260820/raw)
  (`sat_n{노드}_c{concurrency}_r{라운드}.json`)

**Conclusion.** 각 구성의 처리량 상한은 1/2/3-node 에서 **115 / 232 / 342 inf/s**
이며, 3-node 는 1-node 상한의 **2.97× (99%)** 로 **ceiling 기준으로도
near-linear** 하다. 포화는 노드당 ~10–16 동시에서 일어난다. 이로써 S2 의
선형 확장 결론이 최대 처리량 관점에서도 재확인됐다.

→ 다음: **S4 (io_uring)** — 이 baseline 과 동일 조건에서 payload-transfer
경로(S2 §8: non-inference latency 의 94%) 비용을 얼마나 줄이는지 비교한다.

---

<a id="results"></a>

# NPUForge 측정 결과 — 1차 정리

- 정리 시점: **2026-08-14** (단일 노드 계보)
- 대상 기간: 2026-08-07 ~ 2026-08-12
- 원본 데이터: `results/`, `benchmarks/`
- 논의 과정: `discuss.md` (시각순), 작업 이력: `board-worklog.md`

> ## ⚠️ 이 문서는 **단일 노드 계보의 1차 정리**다 (2026-08-21 갱신)
>
> 2026-08-20~21 에 **클러스터 측정 계보가 통째로 진행돼 종료됐다**
> (S2 · S3 · S3.5~3.9b · S0-A~D, 421건, 오류율 0). 그 결과는 이 문서가
> 아니라 **[`experiments/README.md`](#experiments-readme)** 에 있다.
> 다중 노드 수치를 찾는다면 거기서 시작한다.
>
> 아래 §2.5 는 그 계보 **이전의 예비 측정**이며 **후속 측정으로 대체됐다.**
> §1~§4 의 단일 노드 결과와 §5~§6 의 실패 목록은 여전히 유효하다.

> 이 문서는 **결과만** 모은다. 왜 그렇게 결론냈는지는 `discuss.md`,
> 무슨 일이 있었는지는 `board-worklog.md` 를 본다.
>
> **모든 수치에 측정 조건을 함께 적었다.** 조건 없는 숫자는 3개월 뒤에
> 쓸모가 없다는 것을 이번에 배웠다.

---

# 1. 한 장 요약

3대의 RK3576 보드(6 TOPS NPU)로 분산 추론 클러스터를 만들었다.
이 문서는 **단일 노드 특성과 소프트웨어**를 정리한다. 다중 노드 계보는
2026-08-20~21 에 별도로 진행돼 종료됐다 — **3노드 387.2 inf/s
(운영점, 2.86× / 95.3%)**, [`experiments/README.md`](#experiments-readme).

지금까지 나온 가장 중요한 수치 셋.

| 항목 | 값 | 뜻 |
|---|---|---|
| 노드당 처리량 (120초) | **84.3 inf/s** (FP16) / **157.2 inf/s** (INT8) | `want_float=0` 기준 |
| 노드당 **정상 상태** (300초) | **59.7 inf/s** (FP16) | 시작 대비 **-27%**. CPU throttling |
| 애플리케이션 최적화 2종 | **+0.1%, -1.8%** (`core_mask`, zero-copy) | 노드 내부에서 짜낼 것이 거의 없다 |
| INT8 양자화 | **1.86배** | 가장 크게 먹힌 수단 |
| `want_float=0` | **+17.3%** (INT8) / **+15.7%** (FP16) | 출력도 4분의 1. 네트워크와 처리량이 같은 방향 |

그리고 가장 중요한 **비수치** 결과.

> **네 번의 측정이 틀렸고, 네 번 다 "성공처럼 보였다."**
> 이 프로젝트의 산출물 절반은 그 실패의 목록이다. §6 참조.

---

# 2. 확정 수치

## 2.1 하드웨어

| 항목 | 값 | 출처 |
|---|---|---|
| SoC | Rockchip RK3576 | 실측 |
| CPU | 4× Cortex-A72 @2208MHz + 4× A53 @2016MHz | `cpufreq` |
| NPU | 2코어, 950MHz, 6 TOPS(공칭) | `/sys/kernel/debug/rknpu` |
| RAM | 4GB LPDDR4X | 실측 |
| 전원 | 5V DC. **4A 어댑터 필요** | §6.2 |
| 네트워크 | 2.5GbE × 2 (r8125) | 실측 |
| RKNN Runtime | 2.3.0 (`c949ad889d@2024-11-07`) | `strings librknnrt.so` |
| RKNPU 드라이버 | v0.9.8 | `/sys/kernel/debug/rknpu/version` |

세 노드의 커널(6.1.141), `librknnrt.so` 해시, 드라이버 버전, 모델 해시가
모두 일치한다. `preflight-check.sh` 가 매 측정 전에 확인한다.

## 2.2 추론 성능

**측정 조건: `king`, 8스레드, 120초 지속, CPU governor `performance`,
팬리스, 스레드별 전용 RKNN 컨텍스트.**

| 모델 | 처리량 | 평균 지연 | 모델 크기 |
|---|---:|---:|---:|
| YOLOv8n FP16 | **84.3 inf/s** | 94.5 ms | 9.65 MB |
| YOLOv8n INT8 | **157.2 inf/s** | 50.8 ms | 6.46 MB |
| 배율 | **1.86×** | -46% | -33% |

관련 수치.

| 항목 | 값 | 조건 |
|---|---|---|
| 최적 `worker_count` | **8** | 4 대비 +27%. 8에서 아직 안 꺾임 |
| NPU 2번째 코어 기여 | **1.51배** (2배 아님) | 단일코어 48.2 → 두코어 73.0 inf/s |
| 추론당 커널 ioctl | **76회** | FP16·INT8 동일 |
| CPU governor 영향 | +7% | `ondemand` → `performance`. **120초 측정.** 지속 부하에서는 미검증 |
| `want_float=0` 효과 | **INT8 +17.3% / FP16 +15.7%** | 출력도 4분의 1이 된다 |

> ⚠️ **2026-08-11 이전 문서의 처리량 수치는 `ondemand` 기준이다**
> (FP16 79.0 / INT8 146.2). 직접 비교하지 말 것.

## 2.3 열 특성 (예비, S0 아님)

**측정 조건: 3보드 동시, 8스레드, 900초, 팬리스, 선풍기 없음,
governor `ondemand`.** 평탄역은 부하 후 300초~종료.

| 보드 | NPU 평균 | NPU 최고 | 처리량 |
|---|---:|---:|---:|
| king | 73.0°C | 75.8°C | 80.5 inf/s |
| queen | 67.5°C | 70.2°C | 77.7 inf/s |
| jack | 72.6°C | 74.8°C | 77.8 inf/s |

- **노드 간 편차 5.6°C**
- **NPU throttling 없음** — 928 샘플 전부 950MHz
- ⚠️ **그러나 CPU 는 강등된다.** 이 판정은 NPU 클럭만 봤다.
  같은 로그의 CPU 클럭을 보면 A72 2208 → 816 MHz, A53 2016 → 600 MHz 다.
  `discuss.md` §12
- 90°C 초과 없음. 현재 임계치(`degraded 80` / `disable 90`)에 닿지 않음
- 유휴 온도 35~40°C, 입력 전압 최저 5.046V

**팬리스로 8스레드 지속 부하가 가능하다.** 오류 없이 완주한다.
다만 **처리량은 유지되지 않는다** — 300초 지속에서 81.6 → 59.7 inf/s
(-27%). NPU 가 아니라 CPU 가 열로 강등되기 때문이다. `discuss.md` §12

정식 S0(30분 × 2조건)은 아직이다. 이것은 노드 간 편차 확인용 15분 측정이다.

## 2.4 정확도

**측정 조건: 실보드(`king`), COCO val2017 이미지 1장, 전처리를 한 곳에서
수행해 양쪽이 같은 입력 바이트를 보게 함.**

| 비교 | box cosine | 검출 셀 | 클래스 일치 |
|---|---|---|---|
| FP16 vs ONNX | 0.99999 | 10/10 | 100% |
| INT8 vs FP16 | 0.997 | 10/10 | 100% |

INT8 은 최고 검출의 셀이 한 칸 이동하고 점수가 -5.5%. **검출 집합과
클래스는 동일하다.** 1.86배를 이 대가로 얻는다면 쓸 만하다.

RKNN 시뮬레이터는 빌드된 `.rknn` 을 추론하지 못한다(`load_rknn` 후
`init_runtime` 이 거부). 배포되는 것과 같은 파일로 검증해야 하므로
실보드에서 측정했다.

## 2.5 다중 노드 확장성 (2026-08-20, 예비) — **대체됨**

> **이 절은 후속 측정으로 대체됐다.** 단일 run 예비 측정이며, 정식 결과는
> 아래와 같다. 값을 인용할 때는 이 절이 아니라 각 실험 문서를 쓴다.
>
> | 이 절 (예비) | 정식 | 문서 |
> |---|---|---|
> | 3노드 337.7 inf/s | **338.4** (30 run) / ceiling **341.8** | [S2](#experiments-s2-grpc-baseline) · [S3](#experiments-s3-saturation) |
> | 확장 효율 ~98% | **98.9%** (baseline) / **95.3%** (운영점) | [S2](#experiments-s2-grpc-baseline) · [S3.8](#experiments-s3-8-optimized-scaleout) |
> | 노드 상한 115 | 운영점 **135.5** (커넥션 2개/노드) | [S3.7](#experiments-s3-7-connection-tuning) |
> | 로컬 157 대비 −27% | 로컬 direct **161.5** 대비 **−16.1%** | [S3.9b](#experiments-s3-9b-node-residual) |
>
> 아래 ⚠️ 가 지적한 냉각 조건 불일치도 해소됐다 — S3.5 이후 로컬 direct
> 기준값은 **능동 냉각 8워커 161.5** 로 통일했다.

**측정 조건: 스케줄러(server .9) 경유 gRPC, INT8, want_float=0,
governor=performance, Active Cooling(노드마다 전용 팬), round-robin,
30초(1노드 스윕 20초), 단일 run, preflight 통과.** 정식 S2 아님 —
반복 run·팬리스 비교·`--with-inference` 미실시. 원본 `results/scaling-20260820/`.

노드당 동일 부하(concurrency = 8 × 노드수):

| 구성 | 처리량 | 분배 | 오류율 |
|---|---:|---|---:|
| 1노드 | 111.6 inf/s | king 100% | 0% |
| 2노드 | 228.7 inf/s | 50 / 50 | 0% |
| 3노드 | **337.7 inf/s** | 33 / 33 / 33 | 0% |

1노드 concurrency 스윕(포화점): c8 111.6 → c16 114.0 → c32 **115.1 (포화)**.

**확장 효율 ~98% (거의 선형).** 1노드 포화 115 기준 3노드 337.7 = **2.93배**.
데이터 병렬(`adrs/001`)이 성립하고 스케줄러가 3노드 동시에도 병목이 아니다.

**단, 클러스터 노드 상한 115 < 로컬 sustained 157 (-27%).** 왕복 p50 69 ms
인데 노드 보고 추론은 24~28 ms — 40 ms+ 가 스케줄러 gRPC 경유 오버헤드로
보인다(직렬화 + 1.17 MiB 입력·출력 전송 + 큐·라우팅).

> ⚠️ **27% 는 순수 gRPC 오버헤드로 확정할 수 없다.** 기준 157 은 **팬리스**
> (08-11/12) sustained 이고 클러스터 115 는 **Active Cooling**(오늘)이라
> 냉각 조건이 다르다. 로컬 baseline 을 같은 팬 조건에서 재측정한 뒤 확정한다.

> **핵심 질문 "6 TOPS 세 대는 18 TOPS 가 되는가" 의 첫 답: 클러스터 기준
> 2.93배(98%).** 병목은 확장이 아니라 노드당 오버헤드다. 이 27% 의 출처는
> `TimingBreakdown` 단계 분해로 다음에 쪼갠다. `board-worklog.md` §2.25.

원본 데이터·상세 리포트: `results/scaling-20260820/`.

> ✅ **30회 반복으로 재현됨 (2026-08-20).** 1/2/3노드 = 112.9±0.5 / 229.0±0.9
> / **338.4±1.1 inf/s**, speedup 3.00×, error 0%, balance 0%p. SD 극소.
> "한 번 나온 값"에서 "반복 확인된 결과"로 승격. 실험 보고서:
> `docs/experiments/S2_GRPC_BASELINE.md`, 원본: `results/baseline-20260820/`.
>
> ✅ **S3 saturation (2026-08-20).** 각 구성 ceiling = **115 / 232 / 342 inf/s**
> (1/2/3 node), 3N = 2.97× (99%). ceiling 기준으로도 near-linear.
> `docs/experiments/S3_SATURATION.md`.
>
> ✅ **S3.5 transport profiling (2026-08-20).** 위 −30% 손실의 정체를 확정.
> **대역폭 아님**(방향당 링크 51% 사용), **보드 CPU 총량 아님**(63% idle),
> **커널 softirq 편중 아님**(RPS A/B −0.2%), **서버·스케줄러 아님**(3노드
> 3.00× 선형). 남은 것은 **스케줄러↔노드 HTTP/2 전송 경로** — bench 는
> 워커당 1개씩 32 연결을 쓰는데 스케줄러는 노드당 1 연결로 모으고, h2
> window 는 전부 기본값(64 KB)이다. 노드는 같은 보드 로컬 direct 161.5
> inf/s 를 내면서 클러스터에서 116 에 그치고 `node_queue` ≈ 0 으로 여유가
> 남는다. 경로 안에서 ①flow control ②커넥션/TCP ③protobuf·복사 중
> 무엇인지는 **S3.6 A/B 가 가른다.** 그 결과가 S4 를 `gRPC optimized` 와
> `io_uring` 중 하나로 확정한다.
> `docs/experiments/S3_5_TRANSPORT_PROFILE.md`.
>
> ✅ **S3.6 H2/channel A/B (2026-08-20, 20 run).** **노드당 단일 gRPC/HTTP2
> 커넥션 구조가 처리량을 제한하는 주요 요인임을 확인**했다. 커넥션 1 → 4
> 하나만 바꿔 **115.3 → 140.1 inf/s (+21.5%)**, 로컬 direct 까지의 gap 46.2 중
> **54% 를 설정만으로 회수**(아키텍처 변경 없이 커넥션 풀만으로). 단 그 구조
> 안에서 TCP per-flow / H2 multiplexing·락 / flow control 상호작용 중
> 무엇인지는 **아직 미분리**.
> 한편 **64 MB 급 large window 는 이 workload 에서 −36.3% 로 크게 해로웠다** —
> 기본 64 KB 가 backpressure 로 기능하고 있었다는 뜻이다(기본값 유지).
> 다만 64 KB→64 MB 는 1000배 차이의 극단 A/B 라 **"window 튜닝 무효" 로는
> 결론짓지 않는다**(중간값 미측정).
> 대가로 **p95 는 46% 악화**(393 → 573 ms) — 원인 후보가 5개 있고 전부 미검증.
> 새 병목으로 CPU0 포화(busy 81%, soft 74%)가 드러났고, 흐름이 4개가 됐으므로
> S3.5b 의 RPS null 을 다시 볼 여지가 생겼다.
> **io_uring 은 순서상 뒤로 민다** — syscall/req 는 조건 간 80~94 로 거의
> 불변인데 처리량은 73.5~140.1 로 두 배 차이가 났다. (syscall *횟수*가 같다는
> 것과 syscall·복사 *CPU 시간*이 작다는 것은 다른 문제이므로 효과 없음을
> 뜻하지는 않는다.)
> **S2·S3 숫자는 아직 갱신하지 않는다** — 1노드 결과이고 3노드면 서버가
> 12 커넥션을 든다. S3.7 에서 N 확정 후 1N/2N/3N 재측정.
> `docs/experiments/S3_6_H2_CHANNEL_AB.md`.
>
> ✅ **S3.7 + S3.8 optimized gRPC (2026-08-20, 총 146 run).** 운영점을
> **노드당 커넥션 2개 @ concurrency 12** 로 확정하고 scale-out 을 재검증했다.
> 운영점 정의는 **peak 의 98% 이상을 내는 가장 낮은 concurrency**.
>
> | | baseline (conn1) | optimized (conn2/node) | 개선 |
> |---|---:|---:|---:|
> | 1N | 115.2 | **135.5** | +17.6% |
> | 2N | 232.0 | **263.3** | +13.5% |
> | 3N | 341.8 | **387.2** | +13.3% |
> | scaling | 2.97× (98.9%) | **2.86× (95.3%)** | |
>
> 오류 0, 분배 편차 0.0%p. 다만 **scaling efficiency 는 소폭 내려갔다** —
> 노드당 이득이 +17.6%(1N) → +13.3%(3N) 으로 줄어 1노드 최적화가 다노드에서
> 온전히 보존되지 않는다. 서버 10G 링크가 67% → **76%** 로 올라온 것이 유력한
> 후보지만 미확인이다 → 다음은 **서버 쪽 프로파일**.
> `docs/experiments/S3_7_CONNECTION_TUNING.md`, `S3_8_OPTIMIZED_SCALEOUT.md`.

---

# 3. 소프트웨어 현황

| 크레이트 | 상태 | 비고 |
|---|---|---|
| `npuforge-common` | ✅ | 타입, 오류 코드(16종), 설정, 백엔드 인터페이스 |
| `npuforge-proto` | ✅ | gRPC 정의. `SchedulerService` / `NodeService` |
| `npuforge-mock-backend` | ✅ | 결정적 시드. 하드웨어 없이 개발 |
| `npuforge-rknn` | ✅ | 컨텍스트 풀, 다중 출력, 실장비 검증 |
| `npuforge-node` | ✅ | 워커 풀, gRPC 서버, 등록·하트비트 |
| `npuforge-scheduler` | ✅ | 정책 3종, 레지스트리, 재시도 |
| `npuforge-bench` | ✅ | 부하·집계·run 유효성 판정 |

**209 tests, clippy `-D warnings`, fmt clean.** (2026-08-14 기준)

## 3.1 검증한 동작

로컬 Mock 3노드 통합 테스트 (`crates/npuforge-scheduler/tests/mock_cluster.rs`).
실제 gRPC 를 타므로 전송 경로는 실장비와 같다.

| 항목 | 결과 |
|---|---|
| 요청이 3노드에 분산 | ✅ round-robin |
| 노드 1대 사망 시 우회 | ✅ 6/6 성공 |
| 전 노드 사망 | ✅ `NPF-1302` + 시도 노드 목록 |
| 타이밍 분해 | ✅ 노드·스케줄러 구간 모두 |
| 느린 노드 회피 | ✅ least-queue |

실제 프로세스 4개(스케줄러 + 노드 3)로도 확인했다. **스케줄러를 죽였다
다시 띄우면 세 노드가 약 1.3초 안에 스스로 재등록**한다.

실장비 RKNN 통합 테스트 6종 (`crates/npuforge-rknn/tests/real_device.rs`)
도 통과한다 — 출력 9개 반환, 결정성, 4스레드 동시 추론 결과 무오염 등.

## 3.2 미구현

- Prometheus 메트릭
- REST 관리 API, 대시보드
- JPEG 디코딩 (현재 RGB8/BGR8 원본만)
- 후처리(NMS). 노드는 원시 텐서를 그대로 반환한다

---

# 4. 뒤집힌 결론

**측정을 다시 해서 결론이 바뀐 것들이다.** 처음 결론을 그대로 발표했다면
틀린 내용을 말할 뻔했다.

## 4.1 "king 이 19°C 더 뜨겁다" → 재현되지 않음

08-10 지속 부하에서 `king` 만 NPU 91.3°C 로 다른 두 대(70.2/72.1)보다
19°C 높았다. 물리적 배치 문제로 판단하고 최우선 과제로 올렸다.

08-11 통제 조건에서 재측정하니 **편차 5.6°C** 였다.

원인은 배치가 아니라 **부하 프로파일 차이**였다.

| | 08-10 | 08-11 |
|---|---|---|
| 도구 | `thread_safety_test` | `sustained_load_test` |
| 부하 | 1→8 스레드 순차 스윕 | 8스레드 고정 |
| 시작 | king 6분 선행 | 동시 |

`thread_safety_test` 는 목표 스레드 수 전에 단일/2스레드 기준선을 먼저
돈다. `king` 은 다른 둘이 8스레드에 들어갈 무렵 이미 훨씬 오래 가열된
상태였다.

결정적 근거: **`queen` 의 최고 온도는 두 측정에서 70.2°C 로 동일**하다.
움직인 것은 `king` 뿐이다.

→ **부하 프로파일이 다르면 온도를 비교하지 않는다.**

## 4.2 "78 inf/s 는 드라이버 특성" → 범위 축소

추론당 커널 ioctl 약 80회가 직렬화되는 것을 확인하고, 노드 상한
78 inf/s 를 드라이버 특성으로 규정했다. 애플리케이션 최적화 3종이
전부 무의미했던 것과도 맞았다.

그런데 INT8 이 1.85배였다. ioctl 횟수를 확인했다.

```
strace -c -f -e trace=ioctl, 1스레드 20초
  FP16  추론 315회  15.7 inf/s  ioctl 24,079  추론당 76.4
  INT8  추론 718회  35.8 inf/s  ioctl 54,707  추론당 76.2
```

**횟수는 같은데 처리량이 2.28배다.** 상한을 정하는 것은 ioctl 횟수가
아니라 **직렬화 구간에서 한 건이 붙잡고 있는 시간**이다.

CPU governor 실험(+7%)이 이를 보강한다. 그 시간에는 NPU 실행뿐 아니라
**CPU 전후처리도 포함**된다.

→ "애플리케이션 최적화로 못 넘는다"는 유효하다. 다만 **양자화는
  애플리케이션 최적화가 아니라 모델 변경**이다.

## 4.3 "RKNN 은 thread-safe" → 맞지만 시퀀스는 원자적이지 않다

`environment-matrix.md` §3.1 은 "RKNN Runtime 2.3.0 은 thread-safe"로
결론나 있었다. 컨텍스트 하나를 공유하면 구현이 단순해진다.

그런데 그 검증은 **API 반환 코드만 셌고 출력 내용을 대조하지 않았다.**

추론 한 건은 세 번의 호출이다.

```
rknn_inputs_set  →  rknn_run  →  rknn_outputs_get
```

개별 호출이 thread-safe 여도 **시퀀스는 원자적이지 않다.**

| 구성 | API 오류 | **결과 불일치** |
|---|---:|---:|
| 컨텍스트 공유 | 0 | **200 / 200 (100%)** |
| 스레드별 전용 컨텍스트 | 0 | 0 / 200 (0%) |

**공유 컨텍스트는 오류 없이 100% 틀린 답을 낸다.**

이 결함의 성질이 특히 나쁘다.

- 예외도 오류 코드도 없다
- 단일 스레드에서는 절대 재현되지 않는다
- **처리량 지표는 오히려 좋아 보인다** (§3.1 에서 공유 34.8 > 전용 33.2 —
  틀린 답을 더 빨리 내고 있었다)
- 검출 결과도 다른 프레임의 것이라 육안으로는 그럴듯하다

그대로 갔다면 **처리량은 전부 유효하고 검출만 조용히 틀린 채로** 발표까지
갔을 것이다.

→ `RknnContext::infer` 가 `&mut self` 를 받는다. **컴파일러가 동시 호출을
  막는다.** 주석으로 규칙을 적는 것과 타입으로 막는 것은 다르다.

---

# 5. 검증했지만 효과 없던 것

노드 내부에서 짜낼 것이 거의 없다는 근거다.

| 시도 | 결과 | 판단 |
|---|---:|---|
| `core_mask` 수동 코어 배정 | **+0.1%** | 안 쓴다. `CORE_AUTO` 분배가 이미 균등 |
| zero-copy 버퍼 재사용 | **-1.8%** | 안 쓴다. 가설 반증 |

`core_mask` 는 4스레드에서 +9% 였지만 **8스레드에서 +0.1% 로 소멸**한다.
`CORE_0_1` 은 8스레드에서 -11.5% 로 오히려 손해다.

> **`want_float=0` 은 원래 이 표에 있었다.** 08-10 측정에서 FP16 +5.4% 로
> "효과 없음"에 가깝게 분류했다. 그러나 그 측정은 1스레드 위주 조건이었고,
> 08-12 에 8스레드 120초로 다시 재니 **INT8 +17.3% / FP16 +15.7%** 였다.
> 출력 변환이 직렬화 구간을 붙잡는 시간은 동시 스레드가 늘수록 커진다.
> → §2.2 로 옮겼다. **효과 없던 것이 아니라, 조건이 부족한 측정이었다.**
> `discuss.md` §12

---

# 6. 측정 실패 목록

**이 프로젝트에서 가장 재사용 가치가 높은 결과다.** 전부 "성공처럼 보인"
실패다.

## 6.1 지표가 무엇을 세는지 확인하지 않음 (4회)

| # | 무엇 | 실제 |
|---|---|---|
| 1 | `RKNN_QUERY_PERF_RUN.run_duration` 을 NPU 점유시간으로 읽음 | 큐 대기가 포함된 값 |
| 2 | NPU load 를 `delayms=3000` 인 채로 0.2초 간격 샘플링 | 3초 평균을 읽고 있었음 |
| 3 | thread-safety 를 API 반환 코드로만 판정 | 결과 내용을 대조하지 않음 |
| 4 | throttling 을 **NPU 클럭만으로** 판정 | CPU 가 A72 2208→816MHz 로 꺾이고 있었다 |

1번은 모순으로 자가 발견했다(2코어 NPU 에서 "5.03 코어 사용 중").
2번은 ChatGPT 검토가 지적했다. 3번은 백엔드를 구현하며 의심해 찾았다.
4번은 같은 로그에 CPU 클럭이 **이미 기록되어 있었는데** 판정에 쓰지
않았다 — FP16 재측정값이 84.3 이 아니라 66.9 로 나와서야 찾았다 (§2.3).

**공통점: 지표 이름을 보고 의미를 짐작했다.**

## 6.2 전제가 바뀐 것을 모름

| 무엇 | 결과 |
|---|---|
| 문서의 `king` IP 가 낡음 (`.22`, 실제 `.12`) | "노드가 죽었다"고 오판, 서브넷 전체 스캔 |
| 부하 프로파일이 다른 두 측정을 비교 | 19°C 격차로 오해 (§4.1) |
| 보드 리셋 원인 3회 오판 | 공용 PSU → 부트로더 → 12V 입력. 실제는 **어댑터 전류 부족** |

`~/.ssh/config` 에는 처음부터 올바른 IP 가 있었다. **문서에 박은 IP 만
낡았다.** → 보드 접속은 IP 가 아니라 별칭으로 한다.

## 6.3 실패가 성공처럼 보이는 원격 실행

`preflight-check.sh` 를 만들며 발견했다. 검사가 **조용히 작동하지 않았다.**

**`pgrep -f` 는 자기 자신을 센다.** ssh 래퍼의 명령줄에 패턴 문자열이
들어 있어 매칭된다. 대괄호 트릭(`[s]ustained`)도 같은 명령줄에 괄호 없는
형태가 섞이면 무력하다.

| 상황 | 실제 | pgrep 보고 |
|---|---|---|
| 부하 실행 중 | 1개 | **0 (놓침)** |
| 부하 없음 | 0개 | **2 (자기 셸)** |

**`cd DIR && setsid nohup ... &` 는 뜨지 않는다.** `&` 가 리스트 전체에
걸리는데 ssh 가 즉시 끊기면 서브셸이 `setsid` 에 닿기 전에 죽는다.
**종료 코드 0, stderr 비어 있음.** 확인하지 않으면 "부하 없는 상태의
온도"를 15분간 측정한다.

**ssh 안 heredoc + sudo 중첩은 파일을 만들지 않는다.** systemd 유닛
배포에서 겪었다. 이것도 종료 코드 0 이었다.

→ 전부 `scripts/lib/remote.sh` 의 함수로 굳혔다.

## 6.4 "못 읽음"을 "일치"로 판정

`/sys/kernel/debug/rknpu/version` 은 root 만 읽을 수 있다. 세 노드 모두
빈 값을 냈는데 **값이 같다는 이유로 통과**시켰다. 6.1 의 변종이다.

→ 빈 값·`unknown` 같은 자리표시자는 실패로 처리한다.

## 6.5 도구화

같은 실수를 반복하지 않도록 검사를 도구에 넣었다.

| 도구 | 막는 것 |
|---|---|
| `preflight-check.sh` | 별칭↔hostname, 해시 일치, governor, 온도, 전압, 잔존 부하, **추론 정확도** |
| `npuforge-bench` | 예열 제외, `boot_id` 로 재부팅 감지, 표본 부족 판정, 실패를 처리량에서 제외 |
| `run-thermal-comparison.sh` | 동시 시작, 해시 검증, 부하 실제 기동 확인 |
| `scripts/lib/remote.sh` | 원격 실행 함정 |

**성능 측정 전에 정확도부터 확인한다** — `preflight --with-inference` 는
세 보드가 같은 입력에 같은 답을 내는지 본다. 틀린 답을 빨리 내는 구성이
벤치마크에서 이기면 안 된다.

---

# 7. 재현 방법

## 7.1 하드웨어 없이

```bash
cargo test --workspace          # 209 tests
cargo clippy --workspace --all-targets -- -D warnings
```

Mock 3노드 클러스터도 하드웨어 없이 돈다.

```bash
cargo build --release -p npuforge-scheduler -p npuforge-node -p npuforge-bench
./target/release/npuforge-scheduler --config configs/scheduler.example.toml &
for i in 1 2 3; do ./target/release/npuforge-node --config configs/mock/node-0$i.toml & done
./target/release/npuforge-bench --scheduler http://127.0.0.1:50051 \
  --model yolov8n --concurrency 6 --duration 15
```

## 7.2 실장비

전제: `~/.ssh/config` 에 `npuforge-k/q/j` 별칭, `NPUFORGE_SUDO_PASS` 설정.

```bash
bash scripts/preflight-check.sh --with-inference   # 통과 전에는 측정 금지
bash scripts/run-thermal-comparison.sh 900 8       # 열 비교
```

## 7.3 모델 변환

```bash
python tools/model-converter/fetch_calibration.py --out datasets/coco-calib --count 200
docker run --rm -v "$PWD/models:/work/models" -v "$PWD/datasets:/work/datasets" \
  npuforge-converter:2.3.1 python3 /work/tools/convert_yolov8n.py \
  --onnx models/yolov8n.onnx --out models/yolov8n-int8.rknn \
  --dataset datasets/coco-calib --calib-limit 200
```

> **모델은 한 번만 변환해 같은 파일을 세 노드에 배포한다.**
> INT8 변환은 바이트 재현성이 없다 — 같은 입력으로 3회 변환하니 해시가
> 매번 달랐다(1.8% 바이트 상이). 다만 **추론 결과는 완전히 동일**하다
> (9개 텐서 전부 cosine 1.000000). 차이는 직렬화에 있고 계산에는 없다.
>
> `model.toml` 의 `sha256` 은 **배포 무결성**을 보장하지 변환 레시피의
> 동일성을 보장하지 않는다.

---

# 8. 다음에 할 것

## 8.1 차단된 것 — 10G aggregation 필요

raw RGB 입력 한 장은 `640 × 640 × 3 = 1,228,800 byte` 다.

```text
INT8  1,228,800 × 157.2 × 8 = 1.545 Gbps / node   →  3노드 4.636 Gbps
FP16  1,228,800 ×  84.3 × 8 = 0.829 Gbps / node   →  3노드 2.486 Gbps
```

> **2026-08-12 정정.** 이전 문서에 1.43 / 4.3 Gbps 로 적혀 있었다.
> MiB/s 를 Gbps 로 옮기며 2진 접두(÷1024)를 썼기 때문이다.
> **네트워크 속도는 10진이다.** 올바른 값은 위와 같다.

### 출력이 더 크다

위는 **입력(TX)만** 이다. 노드는 후처리를 하지 않고 원시 텐서 9개를
반환하는데, `want_float=1` 이면 출력이 **입력의 3.96배**가 된다.

```text
입력                      1,228,800 byte
출력 (want_float=1, f32)  4,872,000 byte
출력 (want_float=0, int8) 1,218,000 byte
```

3노드 포화 시 스케줄러 링크 부하다.

| 구성 | 모델 | 3노드 TX | 3노드 RX | 10G 로 되나 |
|---|---|---:|---:|---|
| `want_float=1` (구 기본값) | INT8 | 4.64 Gbps | **18.38 Gbps** | **불가** |
| `want_float=1` (구 기본값) | FP16 | 2.49 Gbps | **9.86 Gbps** | 겨우 |
| **`want_float=0` (현재 기본값)** | INT8 | 4.64 Gbps | 4.60 Gbps | 가능 |
| **`want_float=0` (현재 기본값)** | FP16 | 2.49 Gbps | 2.46 Gbps | 가능 |

**`want_float=1` 이었다면 10G 로도 INT8 3노드를 감당하지 못했다.**

→ M3 전에 둘 중 하나가 필요했다.
  **(A)** `want_float=0` 전환 — 출력이 4분의 1이 된다
  **(B)** 노드에서 후처리(NMS) 수행 — 응답이 수 KB 로 줄지만 미구현

  §5 에서 "선택적 최적화, 보류"로 분류했던 `want_float=0` 이
  **M3 의 전제 조건으로 승격**했다. **승격 근거는 처리량이 아니라 RX
  대역폭이었다.**

> ✅ **2026-08-12 (A) 완료.** 노드 설정 `[worker] want_float` 기본값을
> `false` 로 바꾸고, blob 을 **v2** 로 올려 텐서마다 `qnt_type`·`scale`·
> `zero_point` 를 함께 보낸다. 이것 없이 int8 을 보내면 받는 쪽이 해석할
> 수 없다. 실보드에서 역양자화 결과가 float32 와 일치함을 확인했다
> (텐서 9개, **최대 오차 9.5e-7** — float32 정밀도 한계).
> 처리량도 함께 올랐다 — **INT8 +17.3% / FP16 +15.7%** (§2.2, `discuss.md` §12).
>
> 덧붙여, `sustained_load_test` 는 처음부터 `want_float=0` 을 하드코딩하고
> 있었다. 즉 §2.2 의 157.2 / 84.3 은 **이미 `want_float=0` 기준**이었고,
> 이번 전환은 **Rust 백엔드를 측정 조건에 맞춘 것**이다.

### 정리

- worker 링크(2.5G)가 아니라 **aggregation 링크가 먼저 막힌다**
- 스케줄러 쪽에 **10G** 가 필요하다
- 그 위에 **출력 크기를 줄이는 조치**가 반드시 따라와야 한다

**입력만 계산하고 출력을 보지 않은 것이 이 절의 원래 오류였다.**
§6 의 실패 목록에 같은 유형이 이미 세 건 있다.

## 8.2 스위치 없이 가능한 것

- Prometheus 메트릭
- `dealer` NTP 서버 구성
- 정식 S0 열 특성 (30분 × 팬리스/냉각 2조건)
- `ondemand` vs `performance` 300초 비교 — §2.2 의 +7% 는 120초 값이다
- INT8 의 열 거동 (연산량이 줄면 발열도 주는지)

---

# 9. 문서 안내

| 문서 | 내용 |
|---|---|
| `../adrs/` | **왜 그렇게 정했는지.** 결정 28건, 주제순 |
| `TODO.md` | **지금 뭘 해야 하는지.** 재개 절차가 최상단에 |
| `RESULTS.md` | 이 문서. 결과 모음 |
| `discuss.md` | 논의와 판단 근거. 시각순 11개 절 |
| `board-worklog.md` | 작업 이력. 실패한 가설도 보존 |
| `environment-matrix.md` | 확정된 환경 값 |
| `infrastructure.md` | 현재 구성 스냅샷 |
| `00-PRD.md` ~ `03-*.md` | 요구사항·설계 명세 |

수치가 문서 간에 다르면 **`environment-matrix.md` 가 기준**이다
(`00-PRD.md` §0 의 문서 권위 순서).

---

<a id="discuss"></a>

# NPUForge 기술 논의

이 문서는 설계 판단이 갈리는 지점의 논의를 기록한다. 출처(누구의 의견인지)를 명시해 나중에 어떤 근거로 결정했는지 추적할 수 있게 한다.

측정 원본은 `benchmarks/`, 확정 사실은 `environment-matrix.md`, 작업 이력은 `board-worklog.md`.

**각 절에 작성 시각(KST)과 커밋 해시를 적는다.** 같은 날 여러 실험을
하면 날짜만으로는 순서를 알 수 없고, 나중에 "이 결론이 저 측정보다
먼저였나 나중이었나"를 판단할 수 없다.

---

## 읽는 순서

논의는 시간순으로 배치한다. 새 의견은 문서 끝에 덧붙인다.

| # | 절 | 작성 시각 (KST) | 작성 | 요지 |
|---|---|---|---|---|
| 1 | NPU 점유율 판별 실험 | 08-10 (시각 불명) | Claude | 최초 측정과 해석 |
| 2 | ChatGPT 답변/의견 | 08-10 (시각 불명) | ChatGPT | 표현 완화와 재검증 요구 |
| 3 | Claude 재검토 | 08-10 (시각 불명) | Claude | 지적 수용 및 재측정 |
| 4 | core_mask 분배 실험 | **08-10 17:03** | Claude | 대조군 추가, `worker_count=8` 확정 |
| 5 | want_float 실험 | **08-10 17:15** | Claude | 출력 변환 제거, +5.4% |
| 6 | syscall 분해 | **08-10 17:26** | Claude | 병목 확정: 드라이버 ioctl 직렬화 |
| 7 | zero-copy 실험 | **08-10 17:44** | Claude | 가설 반증 |
| 8 | INT8 실측 | **08-11 16:45** | Claude | **1.85배**. 6·7절 결론을 정교화 |
| 9 | 공유 컨텍스트 실험 | **08-11 16:45** | Claude | "오류 0건"은 정답이 아니다 |
| 10 | 벤치 도구 설계 | **08-11 17:15** | Claude | 실수를 도구에 박아 넣기 |
| 11 | CPU governor 영향 | **08-12 10:16** | Claude | **+7%**. 기존 수치는 전부 `ondemand` 기준 — **현재 유효한 결론** |

1~3절은 최초 커밋(`eda93a3`, 08-10 16:29)에 함께 들어가 절 단위 시각을
복원할 수 없다. 4절부터는 커밋 시각이 그대로 작성 시각이다.

**1절의 일부 수치는 3절에서 정정되었다.** 결론만 필요하면 3절을 본다.
**6·7절의 "78 inf/s 상한" 표현은 8절에서 범위가 좁혀졌다.**

---

# NPU 점유율 판별 실험 — Claude 결과/의견

> ⚠️ **이 절의 NPU load 수치(30%)와 일부 결론은 후속 재측정으로 정정되었다.**
> 문서 하단 「Claude 재검토」를 함께 볼 것. 원문은 판단 과정을 남기기 위해 보존한다.

- 작성: 2026-08-10 (최초 커밋 `eda93a3` 16:29 에 포함. 절 단위 시각은 불명)
- 측정 노드: `queen` (NanoPi R76S, RK3576)
- 모델: `yolov8n-fp16.rknn` (FP16, SHA-256 `459602ea70479c1c...`)
- 도구: `crates/npuforge-rknn/native/npu_occupancy_test.c`

## 배경: 무엇을 판별하려 했나

thread-safety 시험에서 **NPU가 2코어인데 8스레드에서 처리량이 5.55배**로 올랐다. 원인 가설이 둘로 갈렸다.

| 가설 | 내용 | 최적화 방향 |
|---|---|---|
| **A** | NPU submission pipeline이 덜 채워져 있었다 | NPU를 더 잘 먹이기 (배칭, 큐잉) |
| **B** | 호출당 시간의 상당 부분이 CPU 구간이고, 스레드가 이를 병렬화했다 | CPU 전처리·후처리 최적화 |

두 가설은 이후 최적화 방향을 정반대로 이끈다. 잘못 해석하면 M7 전체가 틀어진다.

## 측정 결과

### 스레드 수별

| 스레드 | 처리량 | NPU Core0 / Core1 | CPU | inputs_set | run | outputs_get |
|---:|---:|---|---:|---:|---:|---:|
| 1 | 17.0 /s | **16% / 0%** | 9.8% | 17.7 ms | 28.4 ms | 12.3 ms |
| 2 | 33.3 | **17% / 15%** | 17.2% | 17.8 | 29.6 | 12.3 |
| 4 | 56.8 | **25% / 24%** | 26.9% | 19.1 | 40.9 | 10.3 |
| 8 | 76.0 | **32% / 30%** | 43.0% | 24.6 | 65.9 | 12.4 |

NPU 점유율은 커널 debugfs(`/sys/kernel/debug/rknpu/load`)에서 0.2초 간격으로 샘플링했다.

### 1스레드 호출 구성

```text
총 58.9 ms
  inputs_set     17.7 ms  (30%)   CPU
  run            28.4 ms  (48%)   NPU 제출 + 실행 + 대기
  outputs_get    12.3 ms  (21%)   CPU (want_float=1 이므로 역양자화 포함)
  release         0.5 ms  ( 1%)
```

## ⚠️ 먼저 정정: 내 지표가 틀렸다

`RKNN_QUERY_PERF_RUN`의 `run_duration`을 "실제 NPU 점유 시간"으로 해석했으나 **큐 대기를 포함한 값**이었다. 헤더 주석의 `real inference time (us)` 표현에 오도되었다.

**증거:**

- 2코어인데 내 계산으로 `npu_cores_busy = 5.03`이 나왔다. 물리적으로 불가능하다
- `run_duration`이 `rknn_run`의 wall time과 모든 구간에서 정확히 일치한다. 즉 같은 값이다

**커널 debugfs가 신뢰할 수 있는 출처였다.** RKNN API가 보고하는 값을 검증 없이 쓰면 안 된다.

이것은 이 실험 전에 저지른 두 번째 성급한 해석이다. 첫 번째는 "5.55배니까 CPU가 병목"이었고, 그것도 틀렸다.

## 결론: 두 가설 모두 부분적으로만 맞다

### 가설 A가 주된 답이다

1스레드에서 **Core0 16%, Core1 0%** 로 NPU가 사실상 놀고 있다. 스레드를 늘리자 두 코어가 함께 동작하며 처리량이 17 → 76 inf/s(4.5배)로 올랐다.

**단, NPU는 끝까지 포화되지 않는다.** 8스레드에서도 30% 수준이다.

### 그러나 CPU도 병목이 아니다

8스레드에서 CPU 사용률이 **43%** 다. 8코어 중 약 3.4코어만 사용한다. 여유가 있다.

### 진짜 상한은 다른 곳에 있다

`run`이 28.4 → 65.9 ms로 **2.3배** 늘어난 반면, `inputs_set`(17.7→24.6)과 `outputs_get`(12.3→12.4)은 거의 변하지 않았다.

```text
NPU 점유율   30%   ← 바쁘지 않다
CPU 사용률   43%   ← 바쁘지 않다
rknn_run     66ms  ← 그런데 여기서 기다린다
```

**둘 다 포화가 아닌데 지연만 늘어나는 것은 큐잉 병목의 전형이다.** NPU 제출 경로 어딘가에서 직렬화가 일어나고 있다.

후보:

| 후보 | 설명 |
|---|---|
| RKNN 런타임 내부 락 | 여러 context가 하나의 제출 경로를 공유 |
| 커널 드라이버 직렬화 | ioctl 경로 또는 IOMMU 매핑의 배타 구간 |
| NPU 스케줄링 정책 | `CORE_AUTO`가 코어를 충분히 활용하지 못함 |

## 프로젝트에 주는 의미

### 최적화 우선순위가 바뀐다

| 최적화 | 효과 예상 | 근거 |
|---|---|---|
| CPU 전처리 최적화 | 지연시간 개선. **처리량 상한은 못 올림** | CPU가 병목이 아님 (43%) |
| io_uring | **무관** | 네트워크가 관여하지 않는 구간의 문제 |
| **NPU 제출 경로 직렬화 해소** | **상한 자체를 올림** | 여기가 실제 병목 |
| INT8 전환 | `run` 시간 단축 가능 | FP16이 NPU에서 비효율적일 수 있음 |

### 프로젝트 논지와 부합한다

**NPU가 70% 놀고 있다.** "6 TOPS × 3 = 18 TOPS가 아닌" 이유가 네트워크나 스케줄링이 아니라 **노드 하나 안에서 NPU를 다 쓰지 못하기 때문**일 수 있다.

이는 벤더 스펙시트의 TOPS가 실제 처리량을 대표하지 못한다는 본 프로젝트의 문제 정의(PRD §2)를 정면으로 뒷받침한다.

## 다음에 확인할 것

우선순위 순이다.

1. **`core_mask` 명시 분배** — `CORE_0`/`CORE_1`을 직접 지정하면 30% 벽이 깨지는가
   - thread-safety 시험에서는 코어 분리가 오히려 소폭 느렸으나, 그것은 **2스레드** 조건이었다
   - 8스레드에서는 결과가 다를 수 있다
2. **INT8 vs FP16** — `run` 시간이 줄면 상한이 올라간다. calibration 데이터 확정 후
3. **`want_float=0`** — `outputs_get`의 역양자화 제거. 지연시간 개선
4. **`rknn_dup_context`** — 컨텍스트 복제가 개별 `rknn_init`보다 나은가

1번이 가장 빠르게 답이 나온다.

## 측정 방법 메모

재현에 필요한 사항이다.

```bash
# 빌드
gcc -O2 -Wall -Wextra -o npu_occupancy_test npu_occupancy_test.c -lrknnrt -lpthread

# 실행 (모델, 반복횟수, 스레드수)
./npu_occupancy_test yolov8n-fp16.rknn 120 8

# NPU 점유율은 별도로 샘플링해야 한다 (sudo 필요)
sudo cat /sys/kernel/debug/rknpu/load
```

**`RKNN_QUERY_PERF_RUN`은 `RKNN_FLAG_COLLECT_PERF_MASK` 없이 조회 가능하며 오버헤드가 없다.** `PERF_DETAIL`만 플래그를 요구하고 프레임률을 떨어뜨린다.

다만 위에 적었듯 `run_duration`은 큐 대기를 포함하므로 **NPU 점유율 판단에 사용하면 안 된다.**

---

---

# ChatGPT 답변/의견

> 최신 결론은 문서 하단 「Claude 재검토 — ChatGPT 지적 반영 및 재측정」을 따른다.
> 이 절은 재측정을 유도한 검토 의견으로 보존한다.

- 작성: 2026-08-10 (최초 커밋 `eda93a3` 16:29 에 포함. 절 단위 시각은 불명)
- 대상: `NPU 점유율 판별 실험 — Claude 결과/의견`

## 총평

`discuss.md`의 방향은 좋다. 특히 **"5.55배니까 CPU 병목"이라는 성급한 해석을 철회한 것**, 그리고 `RKNN_QUERY_PERF_RUN.run_duration`을 실제 NPU 점유시간으로 보지 말아야 한다고 정정한 부분은 중요하다.

이 논의는 남겨둘 가치가 있다. 나중에 발표에서도 "처음 세운 병목 가설을 실측으로 어떻게 수정했는가"를 보여주는 좋은 자료가 된다.

다만 몇 군데는 표현을 조금 낮추는 것이 좋다.

## 1. "CPU도 병목이 아니다"는 아직 단정하지 않는다

전체 CPU 사용률 43%는 평균값이다. 특정 thread 하나가 락, 드라이버, ioctl 경로에서 막히거나 단일 코어가 포화되는 상황을 가릴 수 있다.

8코어 전체 평균이 낮아도 병목이 병렬화 불가능한 CPU 경로에 있으면 처리량 상한이 생긴다.

따라서 현재 문장의 의미는 다음 정도로 제한하는 것이 정확하다.

```text
CPU 전체 사용률 기준으로는 여유가 있다.
다만 단일 코어 포화, runtime lock, ioctl/off-CPU wait 가능성은 별도로 확인해야 한다.
```

## 2. "io_uring 무관"도 너무 강하다

이 실험은 단일 노드 내부 RKNN 호출 병목을 본 것이다. 이 결과만으로 분산 추론 경로의 `io_uring` 가치를 판단할 수는 없다.

맞는 결론은 다음이다.

```text
io_uring은 이 단일 노드 RKNN scaling 문제의 직접 원인은 아니다.
분산 transport 최적화 여부는 M2/M3의 network_* timing과 syscall 계측 후 판단한다.
```

즉, 현재 관측된 8스레드 scaling 한계의 원인은 네트워크 I/O가 아니다. 그러나 NPUForge 전체에서 `io_uring`이 의미 있는지는 gRPC baseline 이후 별도 측정해야 한다.

## 3. NPU load 30% 해석은 한 번 더 검증한다

`/sys/kernel/debug/rknpu/load`가 가장 믿을 만한 출처라는 판단은 타당하다. 다만 그 값의 의미가 직전 샘플 구간 평균인지, 드라이버 내부 누적/감쇠 값인지 확인이 필요하다.

8스레드에서 76 inf/s가 나오는데 Core0/Core1이 30%대라는 것은 가능하지만 꽤 강한 신호다. 다음을 확인하면 해석이 더 단단해진다.

- 샘플링 주기 0.2초의 영향
- 부하 없음 상태의 baseline
- `watch -n`과 직접 루프 샘플링의 차이
- read 직후 값 변화 여부
- NPU devfreq와 load 값의 동시 기록

## 4. 실제 상한 후보는 조금 넓게 둔다

"RKNN/NPU 제출 경로 직렬화"는 유력한 가설이다. 다만 후보를 너무 좁히지는 않는 것이 좋다.

확인할 후보:

- RKNN runtime 내부 lock
- kernel driver ioctl 직렬화
- IOMMU 또는 buffer mapping 비용
- DDR 또는 memory bandwidth
- output conversion 또는 hidden copy
- thermal/frequency 변화
- `CORE_AUTO` 스케줄링 한계

특히 `inputs_set`이 17~25ms인 것은 생각보다 크다. 100KB JPEG 네트워크 전송보다 훨씬 큰 시간이다. 실제 모델 입력 tensor 크기, copy 횟수, cache behavior, zero-copy input 가능 여부도 봐야 한다.

## 5. 다음 실험 제안

우선순위는 다음과 같다.

1. `CORE_AUTO` vs `CORE_0`/`CORE_1` 명시 분배를 2스레드뿐 아니라 4/8스레드에서 다시 측정
2. `want_float=0`으로 `outputs_get` 비용 제거 후 처리량 변화 확인
3. `perf top`/`perf record`로 user/kernel hot path 확인
4. `strace -c` 또는 `perf stat`으로 ioctl/syscall 수 확인
5. per-core CPU 사용률과 off-CPU time 확인
6. INT8 모델 생성 후 FP16과 같은 실험 반복

## 정리 의견

현재 문서의 방향은 맞다. 다만 최종 결론은 조금 강하므로 다음 정도로 정리하는 것이 더 안전하다.

```text
현재 실험은 8스레드 처리량 증가가 단순 CPU 전처리 병렬화만으로 설명되지 않음을 보였다.
1스레드에서는 NPU가 충분히 사용되지 않았고, 다중 스레드에서 NPU submission pipeline이 더 채워진다.
그러나 8스레드에서도 NPU/CPU 전체가 포화되지 않았으므로, RKNN runtime 또는 kernel driver 제출 경로의 직렬화/큐잉 가능성이 높다.
이 문제는 io_uring과는 별개의 단일 노드 내부 병목이며, 분산 transport 최적화 여부는 gRPC baseline 이후 TimingBreakdown으로 따로 판단한다.
```

---

---

# Claude 재검토 — ChatGPT 지적 반영 및 재측정

- 작성: 2026-08-10 (최초 커밋 `eda93a3` 16:29 에 포함. 절 단위 시각은 불명)
- 대상: 위 `ChatGPT 답변/의견` 5개 항목
- 재측정 도구: 동일 (`npu_occupancy_test.c`) + 샘플링 절차 수정

## 요약

**지적 5개 중 3개를 실측으로 확인했고, 그 결과 앞선 수치가 과소평가였음이 드러났다.**

가장 큰 것은 3번(NPU load 재검증)이다. 측정 방법 자체에 결함이 있었다.

## 지적 3 확인: `delayms = 3000` — 측정 방법 결함

`/sys/kernel/debug/rknpu/` 아래에 `delayms` 파일이 있고 값이 **3000**이었다.

**NPU load가 3초 창(window)으로 평균되는데 0.2초 간격으로 샘플링했다.** 같은 값을 15회씩 중복해서 읽었고, 부하 종료 후의 0% 구간까지 평균에 섞였다.

`delayms`는 쓰기 가능하다. **100ms로 낮추고**, 워밍업 4초 후 정상 구간만 샘플링해 재측정했다.

```bash
sudo sh -c 'echo 100 > /sys/kernel/debug/rknpu/delayms'
```

무부하 baseline은 0%로, 값이 누적/감쇠되지 않음을 확인했다.

### 재측정 결과

| 스레드 | 처리량 | Core0 avg/max | Core1 avg/max | CPU (cpu0~3 / cpu4~7) |
|---:|---:|---|---|---|
| 1 | 16.7 /s | 18.9% / 39% | **0.5% / 1%** | 3,1,2,2 / **30,23**,6,7 |
| 2 | 36.0 | 23.4% / 48% | 16.0% / 33% | 3,2,3,3 / 28,36,40,19 |
| 4 | 55.6 | 29.6% / 62% | 26.9% / 54% | 18,8,7,4 / 48,45,46,48 |
| 8 | 75.0 | **38.9% / 86%** | **37.0% / 81%** | 47,43,44,42 / 48,49,42,46 |

**앞선 30%는 과소평가였다.** 실제 평균은 38.9%, **순간 최대는 86%** 다.

## 지적 1 확인: "CPU도 병목이 아니다"는 부정확했다

전체 평균 43%가 **코어별 불균형을 가리고 있었다.**

```text
1스레드  cpu4=30%, cpu5=23%      big 코어 일부만 사용, little 은 유휴
8스레드  전 코어 42~49%          고르게 분산
```

`cpu0~3`이 little(A53 2.016GHz), `cpu4~7`이 big(A72 2.208GHz)이다. 1스레드에서 big 코어 하나가 30%를 쓰는 것은 단일 스레드 기준으로 작지 않은 부하다.

**정정된 표현:**

```text
CPU 전체 사용률 기준으로는 여유가 있다 (8스레드에서 최대 49%).
단일 코어 포화는 관측되지 않았다.
다만 runtime lock, ioctl 직렬화, off-CPU wait 는 별도 계측이 필요하다.
```

`perf record` / `strace -c` / off-CPU 분석은 아직 하지 않았다.

## 지적 2 수용: "io_uring 무관"은 범위를 넘은 표현

이 실험은 **단일 노드 내부 RKNN 호출 경로**를 본 것이다. 분산 transport의 가치를 판단할 근거가 아니다.

**정정된 표현:**

```text
이 단일 노드 RKNN scaling 한계의 직접 원인은 네트워크 I/O 가 아니다.
NPUForge 전체에서 io_uring 이 의미 있는지는 gRPC baseline 이후
TimingBreakdown 과 syscall 계측으로 별도 판단한다.
```

앞 절 「최적화 우선순위」 표의 `io_uring — 무관` 항목은 이 문장으로 대체한다.

## 지적 4 수용: 상한 후보를 넓힌다

특히 **`inputs_set` 17~25ms** 가 크다는 지적이 타당하다. 입력 텐서가 640×640×3 = 1,228,800 bytes 인데 17ms 면 약 70MB/s 다. 단순 memcpy 라기엔 느리므로 포맷 변환이나 다중 복사가 의심된다.

확인할 후보 (좁히지 않는다):

| 후보 | 확인 방법 |
|---|---|
| RKNN runtime 내부 lock | `perf record`, off-CPU 분석 |
| kernel driver ioctl 직렬화 | `strace -c`, ioctl 횟수·소요시간 |
| IOMMU / buffer mapping 비용 | `perf`, 드라이버 trace |
| DDR / memory bandwidth | `inputs_set` 처리율 대비 이론 대역폭 |
| output conversion / hidden copy | `want_float=0` 비교 |
| thermal / frequency 변화 | devfreq `cur_freq` 동시 기록 (이번엔 950MHz 고정 확인) |
| `CORE_AUTO` 스케줄링 한계 | `core_mask` 명시 분배 비교 |

## 수정된 결론

```text
8스레드 처리량 증가는 CPU 전처리 병렬화만으로 설명되지 않는다.

1스레드에서 NPU Core1 이 0.5% 로 사실상 미사용이었고,
스레드를 늘리자 두 코어가 균등하게 38~39% 까지 올라갔다.
순간 최대는 86% 로 포화에 근접하나 평균은 40% 미만이다.

즉 NPU 를 연속적으로 먹이지 못하고 간헐적으로 비는 구간이 있다.
CPU 는 전체 49% 이하이고 단일 코어 포화도 없다.

따라서 RKNN runtime 또는 kernel driver 제출 경로의
직렬화·큐잉 가능성이 높으나, lock/off-CPU 계측 전까지 확정하지 않는다.

이는 단일 노드 내부 병목이며 io_uring 과는 별개 문제다.
분산 transport 최적화 여부는 gRPC baseline 이후 별도 판단한다.
```

## 운영에 반영할 것

**`delayms` 는 재부팅 시 3000 으로 돌아간다.** NPU load 를 telemetry 로 쓰려면 측정 전 설정이 필요하다.

`preflight-check.sh` 에 추가할 항목:

```bash
sudo sh -c 'echo 100 > /sys/kernel/debug/rknpu/delayms'
# 확인
[ "$(sudo cat /sys/kernel/debug/rknpu/delayms)" = "100" ] || 중단
```

그리고 NPU load 샘플링 규칙:

- 샘플 간격은 `delayms` 이상으로 둔다 (중복 읽기 방지)
- 워밍업 구간과 종료 직후 구간을 평균에서 제외한다
- 평균과 함께 **최대값을 기록한다** (평균만 보면 순간 포화를 놓친다)

## 아직 하지 않은 검증

ChatGPT 제안 중 미실시 항목이다.

- [x] `CORE_AUTO` vs `CORE_0`/`CORE_1` 명시 분배 (4/8스레드) — 4절 참조
- [x] `want_float=0` 으로 `outputs_get` 비용 제거 — 5절 참조
- [x] `perf` 는 커널 버전 불일치로 불가. `time` + `strace -c` 로 대체 — 6절
- [x] `strace -c` 로 ioctl 수 확인 — 추론당 80회, 6절
- [x] off-CPU 분석 — 8스레드에서 블록 58ms/호출, 6절
- [ ] INT8 모델로 동일 실험 반복 (calibration 데이터 확정 후)

## 메타: 같은 실수를 두 번 했다

이번 건에서 성급한 해석이 세 번 있었다.

| # | 잘못된 판단 | 원인 |
|---|---|---|
| 1 | "5.55배니까 CPU 병목" | 대안 가설을 배제하지 않음 |
| 2 | "`run_duration` = NPU 점유시간" | API 주석을 검증 없이 신뢰 |
| 3 | "NPU load 30%" | 측정 도구의 샘플링 특성 미확인 |

공통점은 **측정값이 무엇을 의미하는지 확인하기 전에 결론을 냈다는 것**이다.

2번은 자체 모순(2코어인데 5.03)으로 잡았고, 3번은 외부 지적으로 잡았다. 1번은 실측으로 잡았다.

교훈: **새 측정 지표를 쓸 때는 값의 의미·갱신 주기·경계 조건을 먼저 확인한다.** 특히 커널이 노출하는 값은 문서가 없는 경우가 많으므로 무부하 baseline과 극단값으로 검증한다.

---

# core_mask 분배 실험 — Claude 결과/의견

- 작성: **2026-08-10 17:03 KST** (커밋 `0e6e264`)
- 측정 노드: `queen`
- 모델: `yolov8n-fp16.rknn` (FP16)
- 원본: `benchmarks/results/2026-08-10-coremask/coremask-queen.txt`
- 조건: 스레드당 200회, `delayms=100`, 워밍업 4초 후 샘플링

## 확인하려던 것

앞 절에서 "`CORE_AUTO`가 코어를 충분히 활용하지 못한다"를 병목 후보로 두었다. 이를 검증한다.

**대조군을 추가했다.** 이전까지는 Core1 점유율이 38%라는 숫자만 봤을 뿐, 그것이 실제 처리량 기여로 이어지는지 확인한 적이 없다. 모든 스레드를 코어 하나에 고정한 `CORE_0_ONLY`와 비교하면 판정할 수 있다.

| mode | 설정 | 의도 |
|---|---|---|
| 0 | `CORE_AUTO` | 런타임이 선택 (현재 기본값) |
| 1 | `ALTERNATE` | 스레드를 `CORE_0`/`CORE_1`에 번갈아 고정 |
| 2 | `CORE_0_1` | 모든 스레드가 두 코어를 함께 사용 |
| 3 | **`CORE_0_ONLY`** | **전부 코어 0에 고정 — 대조군** |

## 결과

처리량 (inf/s)

| 스레드 | AUTO | ALTERNATE | CORE_0_1 | **CORE_0_ONLY** |
|---:|---:|---:|---:|---:|
| 1 | 16.7 | 16.7 | **18.2** | 16.5 |
| 2 | 36.2 | 36.5 | 36.4 | **26.4** |
| 4 | 52.4 | **57.1** | 48.5 | **38.5** |
| 8 | 72.9 | **73.0** | 64.5 | **48.2** |

`run` 시간 (µs) 및 NPU 점유율 (평균/최대 %)

| 스레드 | mode | run | Core0 | Core1 |
|---:|---|---:|---|---|
| 8 | AUTO | 69,046 | 39/85 | 37/78 |
| 8 | ALTERNATE | 66,314 | 38/81 | 38/81 |
| 8 | CORE_0_1 | 83,175 | 38/80 | 29/60 |
| 8 | **CORE_0_ONLY** | **120,608** | 46/96 | **0/1** |

## 발견 1: 두 번째 코어는 실제로 기여한다

8스레드에서 **단일 코어 48.2 → 두 코어 73.0 inf/s, 1.51배**다.

Core1의 38% 점유율은 장식이 아니라 실제 처리량이었다. 이전 절에서 확인하지 못한 부분을 대조군이 채웠다.

**다만 2배가 아니라 1.51배다.** 코어를 두 배로 늘려도 처리량은 절반만 는다. 코어 밖에 공유 자원이 있다는 뜻이며, 앞 절의 "제출 경로 직렬화" 가설과 부합한다.

`CORE_0_ONLY`의 `run`이 120.6ms로 폭증하는 것도 같은 현상이다. 코어 하나에 8스레드가 몰리니 대기가 그대로 쌓인다.

## 발견 2: 명시 분배는 이득이 거의 없다

```text
4스레드   52.4 -> 57.1   +9.0%
8스레드   72.9 -> 73.0   +0.1%
```

4스레드에서만 개선되고 8스레드에서는 차이가 없다.

게다가 4스레드 개선분을 뜯어보면 `outputs_get`이 13.6 → 10.0ms로 줄어든 것이 대부분이다. 코어 분배 효과인지 측정 노이즈인지 분리되지 않는다.

**`AUTO`의 분배는 이미 균등하다** (8스레드에서 39%/37%). 런타임 스케줄러가 제 역할을 하고 있어 수동 개입의 여지가 없다.

## 발견 3: `CORE_0_1`은 오히려 손해다

```text
8스레드   72.9 -> 64.5   -11.5%
```

모든 스레드에 두 코어를 열어주면 더 느려진다. 컨텍스트마다 코어를 오가며 스케줄링 오버헤드나 캐시 무효화가 발생하는 것으로 보인다.

**예외는 1스레드다.** 여기서만 16.7 → 18.2(+9%)로 이득이고 `run`이 29.7 → 23.7ms로 줄었다.

단일 요청은 두 코어를 함께 쓰는 것이 유리하다. **지연시간이 중요한 워크로드에서 쓸 수 있는 카드**이며, deadline 요청 처리 시 고려할 만하다.

## 결론

```toml
[worker]
worker_count = 8      # 4보다 27% 높음
                      # core_mask 는 설정하지 않는다 (CORE_AUTO)
```

`ALTERNATE`의 4스레드 이득(+9%)은 8스레드에서 사라진다. **8스레드로 가는 편이 코어를 수동 배정하는 것보다 낫고, 코드도 단순해진다** — `rknn_set_core_mask` 호출 자체가 불필요하다.

## 병목 후보가 좁혀졌다

| 후보 | 판정 |
|---|---|
| ~~`CORE_AUTO` 스케줄링 한계~~ | **배제** — 이미 균등 분배, 수동 개입 무의미 |
| ~~두 번째 코어 미사용~~ | **배제** — 1.51배 기여 확인 |
| **코어 밖 공유 자원 직렬화** | **유력** — 2코어인데 1.51배에 그침 |

앞 절 §4의 후보 목록에서 `CORE_AUTO` 항목이 제거된다. 남은 것:

- RKNN runtime 내부 lock
- kernel driver ioctl 직렬화
- IOMMU / buffer mapping 비용
- DDR / memory bandwidth
- output conversion / hidden copy

## 다음

`want_float=0` 이 답이 빠르다. `outputs_get` 이 모든 조건에서 12~15ms로 일정한데, 이것이 역양자화 비용이라면 제거만으로 지연이 줄어든다. 줄지 않으면 그것도 정보다.

---

# want_float 실험 — Claude 결과/의견

- 작성: **2026-08-10 17:15 KST** (커밋 `e0025b4`)
- 측정 노드: `queen`
- 모델: `yolov8n-fp16.rknn` (FP16)
- 원본: `benchmarks/results/2026-08-10-wantfloat/wantfloat-queen.txt`
- 조건: 스레드당 250회, `CORE_AUTO`

## 확인하려던 것

앞선 측정에서 `outputs_get` 이 모든 조건에서 12~15ms로 **거의 변하지 않았다.** 스레드를 8배로 늘려도 그대로였다. 두 가지 해석이 가능했다.

| 해석 | `want_float=0` 에서 예상되는 결과 |
|---|---|
| 역양자화 CPU 비용이 지배적 | `outputs_get` 크게 감소, 처리량 증가 |
| 커널·드라이버 전송 비용이 지배적 | `outputs_get` 소폭 감소, 처리량 변화 없음 |

`rknn_output.want_float` 를 0으로 두면 모델 네이티브 출력을 그대로 받는다.

## 결과

| 스레드 | wf | 처리량 | total | inputs_set | run | **outputs_get** | out bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1 | 16.6 | 60,116 | 17,828 | 29,343 | **12,424** | 1,638,400 |
| 1 | **0** | **17.6** | 56,704 | 19,826 | 28,897 | **7,620** | **819,200** |
| 2 | 1 | 36.6 | 54,668 | 17,604 | 26,781 | **10,071** | 1,638,400 |
| 2 | **0** | **36.9** | 54,204 | 19,370 | 27,890 | **6,783** | **819,200** |
| 4 | 1 | 55.9 | 71,440 | 18,361 | 41,228 | **11,614** | 1,638,400 |
| 4 | **0** | **56.9** | 70,274 | 19,491 | 41,564 | **8,965** | **819,200** |
| 8 | 1 | 73.5 | 108,341 | 23,781 | 68,664 | **15,041** | 1,638,400 |
| 8 | **0** | **77.5** | 102,422 | 27,036 | 66,442 | **8,625** | **819,200** |

시간 단위는 µs, 호출당 평균이다.

## 발견 1: 출력이 정확히 절반이다

```text
want_float=1   1,638,400 bytes   FP32
want_float=0     819,200 bytes   FP16 (모델 네이티브)
```

FP16 모델이므로 네이티브 출력이 FP16(2바이트)인데 `want_float=1` 이 FP32(4바이트)로 확장하고 있었다. **역양자화가 아니라 정밀도 확장이다.**

INT8 모델이라면 1바이트 → 4바이트로 4배 차이가 나므로 효과가 더 클 것으로 예상된다. calibration 데이터 확정 후 재측정한다.

## 발견 2: `outputs_get` 은 확실히 줄어든다

```text
1스레드   12,424 -> 7,620    -39%
2스레드   10,071 -> 6,783    -33%
4스레드   11,614 -> 8,965    -23%
8스레드   15,041 -> 8,625    -43%
```

**8스레드에서 6.4ms 절감**되며, 스레드가 늘어도 `outputs_get` 이 8.6ms 근처에서 안정된다. `want_float=1` 일 때는 15ms까지 증가했다.

## 발견 3: 그런데 처리량 이득은 작다

```text
1스레드   16.6 -> 17.6   +6.0%
2스레드   36.6 -> 36.9   +0.8%
4스레드   55.9 -> 56.9   +1.8%
8스레드   73.5 -> 77.5   +5.4%
```

**호출당 6.4ms를 줄였는데 처리량은 5.4% 늘었다.** 8스레드 기준으로 호출당 총 108ms 중 6% 감소인데, 처리량 증가도 비슷한 수준이다.

CPU 작업을 제거해도 그만큼만 이득이고 배수 효과가 없다. **시스템이 CPU 바운드가 아니라는 증거다.**

## 발견 4: `inputs_set` 이 일관되게 증가한다

예상하지 못한 관측이다.

```text
1스레드   17,828 -> 19,826   +2,000
2스레드   17,604 -> 19,370   +1,766
4스레드   18,361 -> 19,491   +1,130
8스레드   23,781 -> 27,036   +3,255
```

`want_float` 는 출력 경로 설정인데 입력 시간이 늘어난다. 인과관계가 없어야 한다.

**추정: 메모리 할당자 거동 차이.** `want_float=1` 은 1.6MB 출력 버퍼를 할당·해제하고, `want_float=0` 은 0.8MB를 쓴다. `rknn_outputs_release` 가 해제한 블록을 다음 반복의 `rknn_inputs_set` 이 재사용하는데, 크기가 달라 재사용률이 떨어졌을 수 있다.

**확인 필요.** 버퍼 풀을 도입하면(`01-TECHSPEC.md` §15.1-4) 이 변동이 사라지는지 보면 판별된다.

## 결론

**`want_float=0` 을 채택한다.**

- 8스레드에서 처리량 +5.4%, `outputs_get` -43%
- 출력 전송량 절반 → 노드→스케줄러 네트워크 부하도 절반
- YOLOv8 후처리는 어차피 CPU에서 수행하므로 FP16을 직접 다루면 된다

`npuforge-rknn` 구현 시 `want_float=0` 을 기본으로 하고, 후처리 코드가 모델 네이티브 타입을 처리하도록 작성한다. 출력 타입은 `RKNN_QUERY_OUTPUT_ATTR` 로 조회해 분기한다.

## 병목 판정: 두 번째 해석이 맞다

앞의 두 가설 중 **"커널·드라이버 전송 비용이 지배적"** 쪽이다.

역양자화를 완전히 제거해도 `outputs_get` 이 8.6ms 남는다. 그리고 처리량 이득이 제거한 시간에 비례할 뿐 배수가 아니다.

즉 **출력 경로도 같은 공유 자원을 통과한다.** `run` 이 66ms로 여전히 지배적이고 여기가 손대지 못한 구간이다.

이로써 후보가 더 좁혀진다.

| 후보 | 판정 |
|---|---|
| ~~`CORE_AUTO` 스케줄링 한계~~ | 배제 (§4) |
| ~~두 번째 코어 미사용~~ | 배제 (§4) |
| ~~output conversion 비용~~ | **배제** — 제거해도 +5.4% |
| **RKNN runtime lock** | 유력 |
| **kernel driver ioctl 직렬화** | 유력 |
| IOMMU / buffer mapping | 가능 |
| DDR / memory bandwidth | 가능 (발견 4와 관련 가능성) |

## 다음

`perf record` 와 `strace -c` 로 hot path 와 ioctl 횟수를 본다. 지금까지는 블랙박스 밖에서 입출력만 재었으나, 남은 후보들은 그 안을 봐야 구분된다.

`rknn_run` 이 66ms인데 NPU 점유율이 40%라는 것은 **26ms 이상이 NPU 밖에서 소비된다**는 뜻이다. 그 시간이 어디로 가는지가 다음 질문이다.

---

# syscall 분해 — 병목 확정: 드라이버 ioctl 직렬화

> ⚠️ **이 절의 "78 inf/s 는 드라이버 특성" 표현은 8절에서 범위가 좁혀졌다.**
> 추론당 ioctl 횟수는 INT8 에서도 76회로 같은데 처리량은 1.85배다.
> 상한을 정하는 것은 ioctl **횟수**가 아니라 직렬화 구간에서 한 건이
> 붙잡고 있는 **시간**이다. 애플리케이션 최적화로 못 넘는다는 결론 자체는
> 유효하다 — 양자화는 애플리케이션 최적화가 아니라 모델 변경이다.

- 작성: **2026-08-10 17:26 KST** (커밋 `3656401`)
- 측정 노드: `queen`
- 원본: `benchmarks/results/2026-08-10-syscall/`

## 확인하려던 것

`rknn_run` 이 66ms 인데 NPU 점유율은 40% 다. **26ms 이상이 NPU 밖에서 소비된다.** 그 시간의 정체를 찾는다.

`perf` 는 사용할 수 없었다. 보드 커널이 BSP 6.1.141 인데 Ubuntu 저장소의 `linux-tools` 는 6.8.0 용이라 버전이 맞지 않는다. 대신 다음 두 가지를 썼다.

1. bash 내장 `time` — user/sys 분해. 오버헤드 없음
2. `strace -f -c` — syscall 횟수와 소요시간

## 측정 1: 블록 시간이 지배적이다

| 스레드 | real | user | sys | on-CPU 비율 |
|---:|---:|---:|---:|---:|
| 1 | 14.75 | 6.40 | 1.87 | 56% |
| 2 | 13.88 | 12.94 | 2.88 | 57% |
| 4 | 17.90 | 28.76 | 5.23 | 47% |
| 8 | 25.85 | 74.13 | 9.56 | **40%** |

8스레드 호출당 (총 99.9ms):

```text
37.1 ms   유저스페이스 CPU   librknnrt 내부 (주로 inputs_set)
 4.8 ms   커널 CPU           ioctl 처리 자체
58.0 ms   블록               자면서 대기
```

**커널 CPU 가 4.8ms 뿐이라는 것이 중요하다.** ioctl 처리에 CPU 를 태우는 것이 아니다. 스레드가 **자면서 기다린다.**

그런데 NPU 점유율은 40% 다. **NPU 연산을 기다리는 것이 아니다.**

## 측정 2: ioctl 이 직렬화된다

`strace -f -c`, 30회/스레드.

| | 1 스레드 | 8 스레드 | |
|---|---:|---:|---|
| ioctl 호출 수 | 2,419 | 19,072 | |
| **추론당 ioctl** | **80.6** | **79.5** | 동일 |
| **ioctl 당 소요** | **69 µs** | **374 µs** | **5.4배** |
| futex 호출 | 3 | 395 | 무시 가능 |

두 가지가 결정적이다.

**첫째, 추론당 ioctl 횟수가 스레드 수와 무관하게 ~80 으로 일정하다.** 일이 늘어난 것이 아니다.

**둘째, 개당 지연이 5.4배로 늘어난다.** 같은 일을 하는데 8스레드에서 훨씬 오래 걸린다. 드라이버 내부에 배타 구간이 있다는 뜻이다.

**셋째, futex 가 395회뿐이다.** 유저스페이스 락 경합이라면 수만 건이 나왔어야 한다. **librknnrt 내부 락이 아니다.**

## 결론: 커널 드라이버 ioctl 직렬화

```text
80 ioctl × 374 µs ≈ 30 ms
```

블록 시간 58ms 의 절반 이상을 설명한다. 나머지는 실제 NPU 연산 대기와 스케줄링 지연으로 보인다.

병목 후보 목록이 정리되었다.

| 후보 | 판정 |
|---|---|
| ~~`CORE_AUTO` 스케줄링 한계~~ | 배제 (§4) |
| ~~두 번째 코어 미사용~~ | 배제 (§4) |
| ~~output conversion 비용~~ | 배제 (§5) |
| ~~RKNN runtime 내부 lock~~ | **배제** — futex 395회 |
| **kernel driver ioctl 직렬화** | **확정** |
| IOMMU / buffer mapping | ioctl 의 내용일 가능성 높음 |

## 진짜 문제는 ioctl 이 80회라는 것이다

직렬화 자체는 드라이버 구현이므로 우리가 고칠 수 없다. 그러나 **호출 횟수는 줄일 수 있다.**

추론 한 건에 ioctl 80회는 과다하다. `rknn_inputs_set` / `rknn_outputs_get` 이 매 호출마다 버퍼를 할당·매핑·해제하는 것으로 추정된다.

RKNN 은 이를 피하는 **zero-copy 메모리 API** 를 제공한다. 헤더에서 확인했다.

```c
rknn_create_mem(ctx, size)              /* 버퍼를 한 번 할당 */
rknn_set_io_mem(ctx, mem, attr)         /* context 에 바인딩 */
rknn_destroy_mem(ctx, mem)
RKNN_QUERY_NATIVE_INPUT_ATTR  = 8       /* 네이티브 레이아웃 조회 */
RKNN_QUERY_NATIVE_OUTPUT_ATTR = 9
RKNN_FLAG_MEM_ALLOC_OUTSIDE   = 0x10
```

버퍼를 한 번 할당해 재사용하면 per-call 매핑 ioctl 이 사라질 수 있다.

**이것이 현재 가장 유망한 최적화다.** 그리고 `01-TECHSPEC.md` §15.1-6 의 "등록 버퍼 또는 Zero-Copy 검토" 항목과 정확히 일치한다. 다만 문서는 이를 네트워크 경로 최적화로 상정했는데, **실제로 필요한 곳은 NPU 입출력 경로였다.**

## 프로젝트에 주는 의미

이 결과는 발표 서사에 직접 쓰인다.

```text
6 TOPS × 3 = 18 TOPS 가 되지 않는 이유를 찾다 보니
네트워크도 스케줄링도 아니고,
노드 하나 안에서 NPU 에 데이터를 넣고 빼는 경로가 병목이었다.

추론 한 건에 커널 ioctl 이 80회 발생하고,
동시 실행 시 그 지연이 5.4배로 늘어난다.
NPU 는 40% 만 일하고 나머지 시간은 대기다.
```

io_uring 논의와도 연결된다. **네트워크 I/O 를 최적화해도 이 구간은 건드리지 못한다.** 최적화 대상을 데이터로 골라야 한다는 앞선 판단(§3, §5)이 여기서 확인된다.

## 다음

zero-copy 메모리 API 로 ioctl 횟수가 줄어드는지 측정한다.

```text
현재  rknn_inputs_set -> rknn_run -> rknn_outputs_get -> rknn_outputs_release
      추론당 ioctl 80회

목표  rknn_create_mem (1회) -> rknn_set_io_mem (1회)
      -> [rknn_run 반복]
      -> rknn_destroy_mem (1회)
      추론당 ioctl 횟수 감소 기대
```

측정 항목: 추론당 ioctl 횟수, ioctl 당 지연, 처리량, on-CPU 비율.

---

# zero-copy 실험 — 가설 반증

> ⚠️ **이 절의 "78 inf/s 는 드라이버 특성" 표현은 8절에서 범위가 좁혀졌다.**
> 추론당 ioctl 횟수는 INT8 에서도 76회로 같은데 처리량은 1.85배다.
> 상한을 정하는 것은 ioctl **횟수**가 아니라 직렬화 구간에서 한 건이
> 붙잡고 있는 **시간**이다. 애플리케이션 최적화로 못 넘는다는 결론 자체는
> 유효하다 — 양자화는 애플리케이션 최적화가 아니라 모델 변경이다.

- 작성: **2026-08-10 17:44 KST** (커밋 `7a0379b`)
- 측정 노드: `queen`
- 도구: `crates/npuforge-rknn/native/zerocopy_test.c`
- 원본: `benchmarks/results/2026-08-10-zerocopy/`

## 가설

§6에서 추론당 ioctl 이 약 80회이고 8스레드에서 개당 지연이 5.4배로 늘어남을 확인했다. 직렬화는 드라이버 구현이므로 고칠 수 없으나 **호출 횟수는 줄일 수 있다**고 보았다.

`rknn_inputs_set` / `rknn_outputs_get` 이 매 호출마다 버퍼를 할당·매핑·해제한다면, zero-copy 메모리 API 로 버퍼를 재사용해 per-call 매핑 ioctl 을 없앨 수 있다.

```c
rknn_create_mem(ctx, size)          /* 1회 */
rknn_set_io_mem(ctx, mem, attr)     /* 1회 */
  -> [ memcpy + rknn_mem_sync + rknn_run 반복 ]
rknn_destroy_mem(ctx, mem)          /* 1회 */
```

## 결과: ioctl 이 줄지 않았다

| | 추론당 ioctl | ioctl 당 지연 | 8스레드 처리량 |
|---|---:|---:|---:|
| NORMAL | 79.7 | 54 µs | 78.5 inf/s |
| ZEROCOPY | **89.8** | 56 µs | 77.1 inf/s |

**오히려 10회 늘었다.** 내가 추가한 `rknn_mem_sync`(입력 1 + 출력 9 = 10회)가 그 자체로 ioctl 이기 때문이다.

CPU 사용량도 줄지 않았다.

| | real | user | sys | 처리량 |
|---|---:|---:|---:|---:|
| NORMAL | 15.91 | 44.59 | 5.77 | 78.7 |
| ZEROCOPY | 16.38 | 46.48 | 6.64 | 76.7 |

## 구간 시간은 극적으로 이동했다

1스레드 기준이다.

| 구간 | NORMAL | ZEROCOPY |
|---|---:|---:|
| prepare | 20,208 µs | **1,025 µs** |
| run | 28,152 | **58,560** |
| fetch | 8,233 | **778** |
| **총합** | **56,593** | **60,364** |

`prepare` 와 `fetch` 는 95% 가까이 사라졌으나 `run` 이 2배로 늘어 **총합은 오히려 나빠졌다.**

일이 없어진 것이 아니라 `rknn_run` 안으로 옮겨갔을 뿐이다.

## 결론: 80회의 ioctl 은 추론 제출에 내재한다

버퍼 관리 방식과 무관하다. `rknn_run` 자체가 드라이버와 약 80회 주고받는다.

YOLOv8n 이 여러 레이어로 구성되고 드라이버가 태스크 단위로 제출한다면 자연스러운 수치다. **애플리케이션 계층에서 줄일 수 있는 대상이 아니다.**

따라서 §6에서 확인한 처리량 상한(약 78 inf/s)은 **드라이버와 하드웨어의 성질**이며 우리가 우회할 수 없다.

## ⚠️ 이 실험의 공정성 한계

**두 경로가 같은 일을 하지 않았다.**

```text
native_in_bytes = 2,457,600 = 640 × 640 × 3 × 2   ← FP16
모델 입력(uint8)  = 1,228,800
```

네이티브 입력이 **FP16** 이다. NORMAL 경로의 `rknn_inputs_set` 은 uint8 → FP16 변환과 정규화를 수행한다. 그것이 `prepare` 20ms 의 내용이다.

ZEROCOPY 경로에서는 그 변환을 하지 않고 더미 데이터를 `memset` 했다. **변환 비용을 측정에서 제외한 셈이다.**

즉 실제 애플리케이션이라면 그 변환을 직접 해야 하므로, `prepare` 1ms 는 달성 불가능한 수치다.

**그럼에도 결론은 유지된다.** ioctl 횟수와 CPU 사용량이 줄지 않았고, 총 시간도 개선되지 않았기 때문이다. 변환을 추가하면 더 나빠질 뿐이다.

## 그래도 남는 가치

zero-copy 자체는 버릴 카드가 아니다. 다음 조건에서는 유효할 수 있다.

**입력을 애플리케이션이 이미 네이티브 형식으로 만들 수 있을 때.** JPEG 디코딩과 리사이즈를 직접 수행하는 파이프라인에서는 그 출력을 곧바로 FP16 으로 쓰면 중간 변환이 사라진다. 다만 처리량 상한은 여전히 드라이버가 정한다.

**INT8 모델에서 재검토.** 네이티브 입력이 int8 이면 uint8 입력과 크기가 같아 변환이 단순해진다. calibration 데이터 확정 후 다시 측정한다.

## 최적화 후보 정리

| 후보 | 판정 |
|---|---|
| ~~`CORE_AUTO` 스케줄링~~ | 배제 (§4) |
| ~~두 번째 코어 미사용~~ | 배제 (§4) |
| ~~output conversion~~ | 배제 (§5) |
| ~~RKNN runtime lock~~ | 배제 (§6) |
| ~~zero-copy 로 ioctl 감소~~ | **배제 (§7)** |
| kernel driver ioctl 직렬화 | 확정. **우회 불가** |
| INT8 전환 | **미검증. 남은 유일한 큰 카드** |

## 프로젝트에 주는 의미

**노드 하나의 처리량 상한은 약 78 inf/s 이며, 이는 드라이버 특성이다.**

애플리케이션 최적화로 넘을 수 없다는 것이 세 번의 실험(§4 core_mask, §5 want_float, §7 zero-copy)으로 확인되었다. 각각 +0.1%, +5.4%, -1.8% 였다.

이 사실은 확장 효율 측정에 직접 영향을 준다. **노드당 상한이 고정되어 있으므로 3노드 확장 효율은 스케줄링과 네트워크만으로 결정된다.** 노드 내부 최적화 여지가 거의 없다는 것이 오히려 실험 조건을 단순하게 만든다.

발표 서사에도 쓰인다.

```text
NPU 를 더 잘 쓰려고 세 가지를 시도했다.
코어 수동 배정, 출력 변환 제거, zero-copy 버퍼 재사용.
각각 +0.1%, +5.4%, -1.8% 였다.

추론 한 건에 커널 ioctl 이 80회 발생하고 그것이 직렬화되는데,
그 횟수는 애플리케이션이 줄일 수 없었다.

TOPS 수치가 말해주지 않는 것은 연산 능력이 아니라
그 연산기에 데이터를 넣고 빼는 경로의 비용이다.
```

## 다음

**INT8 모델**이 남은 유일한 큰 변수다. FP16 대비 다음이 달라진다.

- NPU 연산 시간 (`run` 의 실제 계산 부분)
- 네이티브 입력이 int8 이면 변환 비용 감소
- 출력 크기 4분의 1

calibration 데이터가 확정되어야 진행할 수 있다. 그전까지는 노드당 78 inf/s 를 기준값으로 삼는다.

---

# INT8 실측 — Claude 결과/의견

- 작성: **2026-08-11 16:45 KST** (커밋 `547333c`)
- 측정 노드: `king`
- 도구: `crates/npuforge-rknn/native/sustained_load_test.c`
- 정확도 원본: `results/accuracy/README.md`

7절 끝에서 "INT8 이 남은 유일한 큰 변수"라고 적었다. 측정했다.

## 결과

`king`, `sustained_load_test`, 8스레드 고정, 120초.

| 모델 | 처리량 | 평균 지연 | 모델 크기 |
|---|---:|---:|---:|
| FP16 | 79.0 inf/s | 100.9 ms | 9.65 MB |
| **INT8** | **146.2 inf/s** | **54.6 ms** | 6.46 MB |
| 배율 | **1.85×** | -46% | -33% |

> 이 수치는 CPU governor 가 `ondemand` 인 상태에서 측정했다.
> `performance` 로 바꾸면 FP16 84.3 / INT8 157.2 inf/s 다(11절).
> **배율 1.86× 는 그대로다.**

애플리케이션 최적화 세 가지(+0.1%, +5.4%, -1.8%)와 자릿수가 다르다.

## 6·7절 결론과 충돌한다

6절은 "추론 한 건에 ioctl 약 80회가 발생하고 그것이 직렬화된다"를 근거로
**노드 상한 78 inf/s 를 드라이버 특성으로 규정**했다. 7절은 그 표현을
그대로 이어받았다.

그런데 INT8 이 1.85배라면, ioctl 횟수가 상한을 정하는 것이 아니다.
확인했다.

```text
strace -c -f -e trace=ioctl, 1스레드 20초

              추론    처리량      평균 지연   ioctl 총계   추론당 ioctl
FP16          315    15.7 inf/s   63.3 ms      24,079      76.4
INT8          718    35.8 inf/s   27.8 ms      54,707      76.2
```

**추론당 ioctl 횟수는 76.4 vs 76.2 로 사실상 같다.** 그런데 처리량은
2.28배다(1스레드 기준). 즉 상한을 정하는 것은 ioctl **횟수**가 아니라
직렬화 구간에서 **한 건이 붙잡고 있는 시간**이다.

## 수정된 모형

두 진술은 모순되지 않는다. 합치면 이렇게 된다.

```text
처리량 ≈ 1 / (직렬화되는 구간의 건당 소요시간)

  - 직렬화가 일어난다는 것        → 6절이 맞다 (ioctl, futex 395회뿐)
  - 8스레드에서 지연이 5.4배로
    늘어나는 것                   → 6절이 맞다 (직렬화의 증거)
  - 그 시간의 크기를 줄이면
    상한이 올라간다               → 8절 (INT8 이 실제 연산량을 줄인다)
```

**애플리케이션 계층에서 못 넘는다는 것**과 **넘을 방법이 없다는 것**은
다르다. core_mask·want_float·zero-copy 는 ioctl 횟수도 건당 연산량도
줄이지 못했다. 양자화는 연산량을 줄인다.

## 표현 정정

6·7절의 다음 문장을 좁힌다.

| 기존 | 정정 |
|---|---|
| "노드 상한 78 inf/s 는 드라이버 특성이다" | "**FP16 기준** 노드 상한은 약 78 inf/s 이고, 이 값은 애플리케이션 최적화로 못 넘는다" |
| "애플리케이션 최적화로 넘을 수 없다" | 유지. 단 **양자화는 애플리케이션 최적화가 아니라 모델 변경**이다 |

## 발표 서사 갱신

7절의 서사에 한 줄이 더 붙는다. 이 편이 더 정직하고 더 유용하다.

```text
NPU 를 더 잘 쓰려고 세 가지를 시도했다.
코어 수동 배정, 출력 변환 제거, zero-copy 버퍼 재사용.
각각 +0.1%, +5.4%, -1.8% 였다.

추론 한 건에 커널 ioctl 이 76회 발생하고 그것이 직렬화된다.
그 횟수는 애플리케이션이 줄일 수 없었다.

그런데 INT8 양자화는 1.85배였다.
ioctl 횟수는 76회로 똑같았다.

줄여야 할 것은 호출 횟수가 아니라
한 번의 호출이 붙잡고 있는 시간이었다.
```

## 정확도 대가

무손실은 아니다. 실보드 검출 수준 비교(`results/accuracy/README.md`).

| 비교 | box cosine | 검출 셀 | 클래스 |
|---|---|---|---|
| FP16 vs ONNX | 0.99999 | 10/10 | 100% |
| INT8 vs FP16 | 0.997 | 10/10 | 100% |

최고 검출의 셀이 한 칸 이동하고 점수가 -5.5% 였다. **검출 집합과 클래스는
동일하다.** 1.85배를 이 정도 대가로 얻는다면 쓸 만하다.

### 정확도 검증에서 걸린 함정

**원시 텐서 코사인 유사도가 이 모델에서는 오해를 부른다.** FP16 vs ONNX
에서도 일부 텐서가 0.16 까지 떨어진다. 양자화 문제가 아니다.

YOLOv8n 출력 9개 중 텐서 2/5/8 은 클래스 점수 80개의 합이다. RKNN 의
sigmoid 는 정확히 0 을 내지 않고 하한 0.001831 이 있어서, 80배 증폭된
0.1465 오프셋이 생긴다(실측 하한과 정확히 일치). 배경 셀이 대부분이라
이 오프셋이 코사인을 지배한다. **모든 셀에 같은 값이 더해지므로 순위는
바뀌지 않고 검출은 그대로다.**

수락 기준을 검출 수준으로 바꿨다. `compare_detections.py`.

**INT8 변환은 바이트 재현성이 없다.** 같은 ONNX·같은 calibration 목록으로
3회 변환하니 해시가 매번 달랐다(파일 크기는 같고 1.8% 바이트 상이).
다만 추론 결과는 완전히 동일하다(9개 텐서 전부 cosine 1.000000, 오차 0.0).
차이는 직렬화·레이아웃에 있고 수치 계산에는 없다.

→ 모델은 한 번만 변환해 같은 파일을 세 노드에 배포한다. `model.toml` 의
  `sha256` 은 배포 무결성을 보장하지 변환 레시피의 동일성을 보장하지 않는다.

## 다음에 확인할 것

- **INT8 + `want_float=0`.** 5절에서 FP16 기준 +5.4% 였다. INT8 은 출력이
  int8 이라 역양자화 비용이 상대적으로 더 클 수 있어 이득이 더 클지 모른다.
- **INT8 의 열 거동.** 연산량이 줄면 발열도 줄어드는지. 8스레드 지속
  부하에서 FP16 대비 온도를 비교한다. 팬리스 조건에서는 이쪽이 더 중요할
  수 있다.
- **3노드 확장 효율에 미치는 영향.** 노드당 상한이 올라가면 네트워크가
  상대적으로 더 빨리 병목이 된다.

  > ⚠️ 여기 처음 적은 1.43 / 4.3 Gbps 는 **틀렸다.** MiB/s 를 Gbps 로
  > 옮기며 2진 접두(÷1024)를 썼다. 네트워크 속도는 10진이다.
  > 올바른 값: `1,228,800 × 157.2 × 8 = 1.545 Gbps/node`, 3노드 4.636 Gbps.
  > FP16 도 3노드 2.486 Gbps 로 2.5GbE 한 링크를 넘는다.
  > `RESULTS.md` §8.1 참조.

  **S2 확장성 실험의 설계를 다시 봐야 한다.**

---

# 공유 컨텍스트 실험 — "오류 0건"은 "정답"이 아니다

- 작성: **2026-08-11 16:45 KST** (커밋 `547333c`, 측정은 `d228cda` 16:33)
- 측정 노드: `king`
- 도구: `crates/npuforge-rknn/native/shared_context_test.c`

`npuforge-rknn` 백엔드를 구현하면서 컨텍스트를 공유할지 스레드마다 둘지
정해야 했다. `environment-matrix.md` §3.1 은 "RKNN Runtime 2.3.0 은
thread-safe" 로 결론이 나 있었다. 그대로 믿으면 컨텍스트 하나로 끝난다.

## 의심한 이유

추론 한 건은 세 번의 호출이다.

```text
rknn_inputs_set  →  rknn_run  →  rknn_outputs_get
```

**개별 호출이 thread-safe 라는 것과 이 시퀀스가 원자적이라는 것은 다르다.**
스레드가 사이에 끼어들면 입력과 출력의 짝이 어긋날 수 있다.

그리고 §3.1 의 측정을 다시 보니 **API 반환 코드만 셌다.** 출력 내용을
대조하지 않았다. 결과가 섞여도 `ok / err` 는 `40 / 0` 으로 나온다.

## 측정

`native/shared_context_test.c`. 스레드마다 다른 입력을 주고, 먼저 단독으로
추론해 기준 출력을 저장한 뒤, 동시 실행 결과가 자기 기준과 같은지 대조한다.

`king`, FP16, 4스레드 × 50회.

| 구성 | API 오류 | **결과 불일치** |
|---|---:|---:|
| 컨텍스트 공유 | 0 | **200 / 200 (100%)** |
| 스레드별 전용 컨텍스트 | 0 | 0 / 200 (0%) |

**공유 컨텍스트는 오류 하나 없이 100% 틀린 답을 낸다.**

## 무엇이 위험했나

이 결함은 다음 성질을 전부 갖는다.

- 예외도 오류 코드도 남기지 않는다
- 단일 스레드 테스트에서는 절대 재현되지 않는다
- 처리량 지표는 오히려 좋아 보인다 (§3.1 에서 2스레드 공유가 34.8 inf/s 로
  전용 33.2 보다 빨랐다 — **틀린 답을 더 빨리 내고 있었다**)
- 검출 결과를 육안으로 봐도 "그럴듯한" 박스가 나온다. 다른 프레임의 결과이지
  쓰레기가 아니기 때문이다

만약 이대로 벤치마크를 돌렸다면, **처리량 수치는 전부 유효하고 검출 결과만
조용히 틀린 상태**로 발표까지 갔을 가능성이 크다.

## 반영

- `environment-matrix.md` §3.1 에 정정 블록을 넣었다. "context 공유" 행의
  처리량 수치는 성능 비교에서 제외한다.
- `RknnContext::infer` 가 `&mut self` 를 받는다. **컴파일러가 동시 호출을
  막는다.** 주석으로 규칙을 적어 두는 것과 타입으로 막는 것은 다르다.
- `ContextPool` 이 `worker_count` 만큼 컨텍스트를 만들어 하나씩 점유한다.
- `supports_concurrent_infer = true` 는 유지하되 근거가 바뀌었다.
  "런타임이 알아서 해준다" 가 아니라 **"백엔드가 풀로 직렬화한다"** 이다.

## 이 프로젝트에 주는 의미

같은 실수를 세 번째로 했다. 3절 「메타」에 두 번을 적어 두었다.

```text
1. RKNN_QUERY_PERF_RUN.run_duration 을 NPU 점유시간으로 읽었다
   → 큐 대기가 포함된 값이었다
2. NPU load 를 delayms=3000 인 채로 0.2초 간격 샘플링했다
   → 3초 평균을 읽고 있었다
3. thread-safety 를 API 반환 코드로만 판정했다
   → 결과 내용을 대조하지 않았다
4. throttling 을 NPU 클럭만으로 판정했다  (12절)
   → 같은 로그에 있던 CPU 클럭이 63~70% 떨어지고 있었다
```

공통점이 분명하다. **지표가 무엇을 세는지 확인하지 않고 이름만 보고 믿었다.**

`preflight-check.sh` 에 넣을 항목이 하나 늘었다.
**성능 측정 전에 정확도부터 확인한다.** 틀린 답을 빨리 내는 구성이
벤치마크에서 이기는 것을 막아야 한다.

---

# 벤치 도구 설계 — 실수를 도구에 박아 넣기

- 작성: **2026-08-11 17:15 KST** (구현 커밋 `b2cae0d`, 17:12)
- 대상: `crates/npuforge-bench/`
- 검증: Mock 3노드 종단 실행

## 왜 이 절을 남기나

`npuforge-bench` 는 새 측정 결과가 아니라 **도구**다. 그런데 이 도구의
설계 근거가 전부 앞 절들의 실패에서 나왔으므로 여기에 남긴다.

지금까지 나온 측정 실수를 모아 보면 성격이 세 가지다.

```text
A. 지표가 무엇을 세는지 확인하지 않았다
   - run_duration 을 NPU 점유시간으로 읽음 (3절)
   - delayms=3000 인 채로 0.2초 샘플링 (3절)
   - thread-safety 를 API 반환 코드로만 판정 (9절)

B. 조건이 달라진 것을 모르고 값을 비교했다
   - 부하 프로파일이 다른 두 측정을 비교해 19°C 격차로 오해
     (board-worklog.md §2.19)
   - 문서의 IP 가 낡아 노드를 사망으로 오판 (§2.20)

C. 무효한 데이터를 유효한 것으로 취급할 뻔했다
   - 어댑터 용량 부족으로 리셋된 보드의 처리량 (§2.17.2)
```

**주석으로 "조심하자"고 적어 두는 것은 통하지 않았다.** 세 번 다 알고
있으면서 당했다. 그래서 도구가 강제하게 했다.

## 도구에 박아 넣은 규칙

| 과거 실수 | 도구가 하는 일 |
|---|---|
| 첫 추론 지연이 튄다 | 예열 요청을 집계에서 제외 |
| 리셋된 보드를 "성능 저하"로 읽음 | `boot_id` 변화 → run 무효 |
| 표본 20건으로 p99 를 냄 | 성공 100건 미만이면 무효 |
| — | 실패를 처리량·노드 몫에서 제외 |
| — | 조건(동시성·시드·정책·노드 수)을 결과에 동봉 |
| — | 백분위는 nearest-rank, 보간 금지 |

### 실패를 처리량에 넣지 않는 이유

넣으면 **노드가 전부 죽었을 때 처리량이 가장 높아진다.** 실패는 즉시
반환되므로 초당 건수가 폭증한다. S4 장애 대응 실험에서 이 지표를 그대로
보면 "장애 시 성능 향상"이라는 결과가 나온다.

노드 몫도 같다. 실패 요청의 `node_id` 는 비어 있는데, 이것을 세면 죽은
노드가 "많이 처리한" 것으로 잡힌다.

### 백분위를 보간하지 않는 이유

선형 보간은 표본이 적을 때 **실제로 관측되지 않은 값**을 만든다.
1~10 에서 p95 를 보간하면 9.55 가 나오는데, 그런 지연을 겪은 요청은 없다.
발표 자료에 "p95 = 9.55ms" 라고 적으면 그것은 측정값이 아니라 계산물이다.

nearest-rank 로 고정하고 정의를 모듈 문서에 박았다.

### 무효 경고를 숫자보다 먼저 출력하는 이유

```text
!!!!!! 이 run 은 무효다 !!!!!!
  - 오류율 100.00% 가 허용치 1.00% 를 넘었다
  - 성공 표본 0건은 최소 100건에 못 미친다
아래 수치를 인용하지 말 것.

요청 : 200 (성공 0 / 실패 200, ...)
```

숫자를 먼저 보여주면 사람은 그것부터 믿는다. 경고를 아래에 두면 스크롤
없이 보이는 첫 화면이 숫자가 되고, 그 숫자가 표에 옮겨 적힌다.

무효 run 을 **삭제하지는 않는다.** 사유와 함께 남아야 원인을 추적할 수
있고, 재부팅이 반복되면 그 자체가 발견이다.

## 구현 중에 잡은 문제 하나

처음에는 노드 상태를 하트비트 RPC 로 조회하려 했다. 스케줄러에 노드
목록 API 가 없었기 때문이다.

**그런데 그것이 스케줄러의 노드 상태를 덮어쓴다.** 하트비트는 관측값을
기록하는 호출이고, 벤치가 빈 `health` 를 보내면 스케줄러가 그것을 실제
관측으로 받아들여 온도·큐 깊이를 0 으로 만든다. **측정 직전에 측정
대상의 상태를 오염시키는** 셈이다.

읽기 전용 `ListNodes` RPC 를 따로 만들었다. 이것도 A 유형(부작용을
확인하지 않고 API 를 씀)의 변종이다.

정책 이름도 스케줄러가 보고한 값을 우선하게 했다. `--policy round-robin`
으로 손으로 적으면 틀리고, **틀린 정책 이름이 붙은 결과는 S3 정책 비교
실험을 통째로 망친다.**

## 도구가 보장하지 않는 것

닫힌 모델(closed loop) 부하다. 동시성 N 을 고정하고 응답을 받은 뒤 다음
요청을 보낸다.

이 방식은 **coordinated omission** 에 취약하다. 시스템이 느려지면
클라이언트도 덩달아 천천히 보내므로 지연 분포가 낙관적으로 나온다.
실제로 느린 요청이 뒤이을 요청의 발사 시각을 미루는데, 그 미뤄진 시간은
어느 요청의 지연에도 계상되지 않는다.

**절대 지연을 SLA 처럼 인용하지 않는다.** 구성 간 비교에만 쓴다.
이 문장을 결과 파일의 `caveats` 에 넣어 결과만 떼어 봐도 알 수 있게 했다.

열린 모델(목표 RPS 고정)을 쓰지 않은 이유는 노드 큐가 유한하기 때문이다.
RPS 를 올리면 금방 `NPF-1303` 거절로 끝나 지연 분포를 볼 수 없다.
두 모델 다 필요하면 M7 에서 추가한다.

## Mock 3노드 확인 결과

```text
요청 : 395 (성공 395 / 실패 0)
처리량: 23.3 inf/s  (17.0초)
재시도: 31건

지연 (왕복, ms)
  min 23.7  p50 45.1  p90 256.9  p95 302.8  p99 3092.9  max 3214.4

노드별 분배
  mock-01     160건   40.5%   p50    30.3 ms  p99  3059.0 ms
  mock-02     157건   39.7%   p50   220.8 ms  p99  3178.3 ms
  mock-03      78건   19.7%   p50    30.0 ms  p99    75.1 ms
```

**p99 가 3.09초인 것은 버그가 아니다.** Mock 노드의
`queue_timeout_ms = 3000` 에 걸린 요청이 `NPF-1303` 으로 거절되고
스케줄러가 다른 노드로 재시도해 성공한 것이다(재시도 31건). 동시성 6 에
`worker_count = 1` 인 Mock 이라 큐가 쌓인다.

즉 이 수치는 **재시도 경로가 실제로 동작한다는 증거**다. 실장비에서는
`worker_count = 8` 이므로 다른 그림이 나온다.

종료 코드로 야간 자동 실행을 지원한다.

```text
0  유효
3  무효   ← 스크립트가 이것으로 재실행 여부를 판단한다
2  인자 오류
1  실행 실패
```

## 다음

M3 실장비 측정(1/2/3노드 확장 효율)은 **10G aggregation 구성이 있어야** 시작한다.

8절에서 계산했듯 INT8 노드 하나가 **1.545 Gbps** 를 요구한다(8절의 1.43 은
2진 접두를 쓴 오류로 정정되었다). 현재 관리망
1GbE 로는 **노드 한 대분도 받지 못한다.** 지금 측정하면 네트워크 병목을
확장 효율로 잘못 보고하게 된다 — B 유형 실수를 그대로 반복하는 것이다.

스위치 도착 전까지는 Prometheus 메트릭, `preflight-check.sh`,
`dealer` NTP 서버 구성을 진행한다.

---

# CPU governor 영향 — 기존 수치는 전부 `ondemand` 기준이었다

- 작성: **2026-08-12 10:16 KST**
- 측정 노드: `king` (동일 조건 재측정)
- 계기: `preflight-check.sh` 가 `ondemand` 를 하드 실패로 막았고, 그래서 바꿨다

## 결과

같은 도구·같은 조건(8스레드, 120초, `king`)에서 governor 만 바꿨다.

| 모델 | `ondemand` | `performance` | 변화 |
|---|---:|---:|---:|
| FP16 | 79.0 inf/s | **84.3 inf/s** | **+6.7%** |
| INT8 | 146.2 inf/s | **157.2 inf/s** | **+7.5%** |

평균 지연도 함께 줄었다 (FP16 100.9 → 94.5 ms, INT8 54.6 → 50.8 ms).

## 왜 CPU governor 가 NPU 처리량을 바꾸나

추론 한 건은 NPU 실행만이 아니다.

```text
입력 설정(CPU) → NPU 실행 → 출력 취득·역양자화(CPU)
```

3절에서 8스레드가 2스레드보다 빠른 이유로 이미 확인한 구조다. 한 스레드가
CPU 구간에 있는 동안 다른 스레드가 NPU 를 점유하는 파이프라이닝이 일어난다.
**그 CPU 구간의 속도가 전체 처리량에 직접 반영된다.**

`ondemand` 는 부하에 따라 주파수를 올리는데, NPU 대기 중에는 CPU 사용률이
낮아 보여 주파수가 내려간다. 그 상태에서 다음 요청의 CPU 구간이 시작되면
느린 클럭으로 시작한다. 관측된 유휴 클럭이 1008~1800MHz 로 흔들린 것이
이것이다(최대는 A53 2016 / A72 2208MHz).

## 이 결과가 뜻하는 것

**8절까지의 모든 처리량 수치는 `ondemand` 기준이다.** 앞으로 나올 수치와
직접 비교하면 안 된다. 문서의 확정 수치를 `performance` 기준으로 갱신했다.

다만 **결론은 바뀌지 않는다.**

| 결론 | 근거 |
|---|---|
| INT8 이 FP16 대비 1.85배 | `performance` 에서 157.2/84.3 = **1.86배**. 그대로다 |
| 애플리케이션 최적화 3종이 무의미 | governor 는 애플리케이션 최적화가 아니다 |
| 상한은 직렬화 구간의 건당 시간 | CPU 구간이 그 시간의 일부라는 것이 오히려 보강된다 |

8절에서 "상한을 정하는 것은 ioctl 횟수가 아니라 한 건이 붙잡고 있는
시간"이라고 썼는데, 이번 결과가 그 시간에 **CPU 전후처리도 포함된다**는
것을 보여준다. ioctl 횟수는 governor 와 무관하게 76회로 같을 것이다.

## 조치

`scripts/set-cpu-governor.sh` 로 세 노드를 `performance` 로 고정하고
systemd 유닛으로 영구화했다.

재부팅 유지를 실제로 확인했다. `jack` 을 재부팅해 `boot_id` 가
`6caea6bd → 83d2981f` 로 바뀐 뒤에도 governor 가 유지되었다.

`cpufrequtils` 패키지를 쓰지 않았다. 설치하면 세 노드의 패키지 목록이
달라져 환경 일치가 깨진다.

## 유휴 온도는 거의 안 올랐다

| 노드 | `ondemand` | `performance` |
|---|---:|---:|
| king | 36.1°C | 37.0°C |
| queen | 35.2°C | 36.1°C |

클럭은 항상 최대지만 유휴 코어는 여전히 halt 상태라 발열이 늘지 않는다.
**팬리스 S0 측정에 부담을 주지 않는다.**

## 남은 함정 하나

`set-cpu-governor.sh` 를 만들 때 ssh 안에서 heredoc 과 sudo 를 중첩했다가
**유닛 파일이 아예 생기지 않았는데 종료 코드는 0** 이었다. 스크립트는
"적용 실패"를 보고했지만, 값 확인 단계를 넣지 않았다면 "영구화 완료"로
넘어갔을 것이다.

유닛 파일을 로컬에서 만들어 `scp` 로 전송하는 방식으로 바꿨다.
board-worklog.md §2.21 의 원격 실행 함정과 같은 계열이다.

---

# want_float=0 전환과 CPU throttling — Claude 결과/의견

- 작성: **2026-08-12 17:40 KST**
- 측정 노드: `king`
- 계기: 네트워크 계산에서 `want_float=0` 이 M3 전제 조건으로 승격됨

## 1. want_float=0 의 처리량 효과 (미측정이었던 것)

5절의 `+5.4%` 는 FP16 에서 잰 값이라 INT8 에 옮길 수 없다고 적어 두었다.
측정했다. `king`, 8스레드, 120초.

| 모델 | `want_float=0` | `want_float=1` | 이득 |
|---|---:|---:|---:|
| INT8 | **156.7 inf/s** | 133.6 inf/s | **+17.3%** |
| FP16 | 66.9 inf/s | 57.8 inf/s | **+15.7%** |

5절의 +5.4% 보다 훨씬 크다. 5절은 1스레드 위주 조건이었고, 여기서는
8스레드 동시 실행이라 출력 변환이 직렬화 구간을 더 오래 붙잡는다.

**네트워크와 처리량이 같은 방향을 가리킨다.** 출력이 4분의 1이 되고
처리량이 15~17% 오른다. `want_float=0` 을 안 쓸 이유가 없다.

> 그리고 이제야 알았는데, **`sustained_load_test` 는 처음부터
> `want_float=0` 을 하드코딩하고 있었다.** 즉 문서의 157.2 / 84.3 은
> 이미 `want_float=0` 기준이었다. Rust 백엔드만 `true` 였으므로,
> 이번 전환은 **소프트웨어를 측정 조건에 맞춘 것**이다.

## 2. 그런데 더 큰 것이 나왔다 — CPU 가 열로 꺾인다

FP16 을 다시 재는데 값이 84.3 이 아니라 66.9 로 나왔다. INT8 은
156.7 로 일치했다. FP16 만 다른 것이 이상해 확인했다.

**원인은 측정 순서였다.** FP16 측정이 INT8 측정 두 번 뒤에 붙어 있었다.

| 시작 온도 | FP16 처리량 |
|---|---:|
| 53.6°C (냉각 후) | **81.6 inf/s** |
| 71.2°C (연속 측정 중) | 66.9 inf/s |

**-18%.** 그래서 부하 중 클럭을 직접 관찰했다. governor 는 `performance` 다.

```text
        NPU온도   npu_clk   cpu4(A72)   cpu0(A53)
 +15s   86.8°C    950 MHz   2208 MHz    2016 MHz
 +30s   90.4°C    950 MHz   1416 MHz    1200 MHz
 +45s   89.5°C    950 MHz   1008 MHz     816 MHz
 +60s   87.8°C    950 MHz    816 MHz     600 MHz
+120s   87.8°C    950 MHz    816 MHz     600 MHz
```

**NPU 클럭은 950 MHz 에서 한 번도 안 떨어진다. CPU 가 63~70% 떨어진다.**

300초 지속 측정에서 처리량이 이렇게 수렴한다.

```text
 +10s  81.6 inf/s   ← 시작
+120s  63.6
+300s  59.7         ← 정상 상태
평균   71.3 inf/s
```

**시작 대비 -27%.**

## 3. 이것이 뒤집는 것

### 3.1 "throttling 없음" 은 틀렸다

`RESULTS.md` §2.3 과 `environment-matrix.md` §9.0 에 이렇게 적혀 있다.

> throttling 없음 — 928 샘플 전부 NPU 950MHz, 한 번도 안 떨어짐

**NPU 클럭만 봤다.** CPU 클럭도 같은 로그에 기록되어 있었는데 판정에
쓰지 않았다. 추론 한 건은 `입력 설정(CPU) → NPU → 출력 취득(CPU)` 이고
CPU 구간이 처리량에 직접 반영된다는 것을 11절에서 이미 확인했으면서도,
throttling 판정은 NPU 만으로 했다.

**이것이 같은 유형의 네 번째 실수다.** 3절 「메타」 목록에 추가한다.

```text
1. run_duration 을 NPU 점유시간으로 읽음      → 큐 대기 포함
2. NPU load 를 delayms=3000 인 채로 샘플링    → 3초 평균
3. thread-safety 를 API 반환 코드로만 판정    → 결과 미대조
4. throttling 을 NPU 클럭만으로 판정          → CPU 가 꺾이고 있었다
```

### 3.2 CPU governor 결론의 범위가 좁아진다

11절의 **+7%** 는 120초 측정이다. 그 구간은 아직 CPU 가 완전히 강등되기
전이다. **지속 부하에서는 `performance` 가 더 유리하다고 단정할 수 없다.**

`performance` 는 유휴에도 최대 클럭을 유지하므로 부하 시작 시점의 열
여유가 적다. 더 빨리 뜨거워지고 더 일찍 강등될 수 있다.

**측정하지 않았다.** `ondemand` 와 `performance` 를 동일한 300초 조건에서
비교해야 한다. 그전까지 11절의 +7% 는 **"짧은 측정에서의 이득"** 으로만
읽는다.

### 3.3 Peak vs Sustained 격차가 이 프로젝트의 핵심 수치가 된다

지금까지 "Peak vs Sustained 약 10%" 로 적어 두었다. 이번 측정은
**300초에 -27%** 다. 원인이 NPU 가 아니라 **CPU thermal throttling** 이라는
것까지 짚었다.

> 벤더가 공개하는 TOPS 는 순간 성능이다. 팬리스 엣지에서 실제로 무엇이
> 먼저 무너지는가 — **NPU 가 아니라 그 앞뒤를 처리하는 CPU 였다.**

발표 서사로서 이쪽이 훨씬 낫다. S0 을 제대로 돌리면 확정 수치가 된다.

## 4. 조치

**한 것**

- `want_float` 을 노드 설정(`[worker] want_float`)으로 노출하고 기본값을
  `false` 로 바꿨다. Rust 백엔드가 측정 도구와 같은 조건이 되었다
- blob 형식을 **v2** 로 올려 텐서마다 `qnt_type`·`zero_point`·`scale` 을
  싣는다. 이것 없이 int8 을 보내면 받는 쪽이 해석할 수 없다
- 실보드에서 역양자화가 float32 와 일치함을 확인했다
  (텐서 9개, **최대 오차 9.5e-7** — float32 정밀도 한계)

**해야 할 것**

- `ondemand` vs `performance` 를 동일한 300초 조건에서 비교
- S0 를 30분으로 돌려 정상 상태 처리량과 강등 시점을 확정
- 열 판정에 **CPU 클럭을 포함**하도록 `run-thermal-comparison.sh` 수정
- `RESULTS.md` §2.3 과 `environment-matrix.md` §9.0 의 "throttling 없음"
  표현 정정 (이번 커밋에 포함)

---

<a id="board-worklog"></a>

# NPUForge 보드 작업 로그

- 문서명: `board-worklog.md`
- 대상: NanoPi R76S × 3 (`king` / `queen` / `jack`)
- 목적: 보드에 가한 모든 변경을 시간순으로 기록한다

---

# 0. 이 문서의 규칙

보드에 실행한 명령과 그 결과를 **시간순으로 append**한다. 기존 항목은 수정하지 않는다.

기록하는 이유는 세 가지다.

1. **재현성.** 보드를 다시 세팅하거나 네 번째 노드를 추가할 때 이 문서만 따라가면 된다.
2. **원인 추적.** 벤치마크 결과가 노드마다 다르게 나올 때, 세 보드에 무엇이 다르게 적용됐는지 여기서 확인한다.
3. **오픈소스 공개.** 외부 사용자가 같은 환경을 만들 수 있어야 한다.

각 항목에 다음을 남긴다.

```text
날짜 / 대상 노드 / 실행한 명령 / 결과 / 판단 근거
```

**되돌릴 수 없는 변경**(패키지 업그레이드, 커널 교체, 파티션 조작)은 실행 전에 별도로 표시하고 승인 여부를 기록한다.

---

# 1. 노드 명칭

물리 보드에 붙인 라벨을 그대로 사용한다.

| 라벨 | hostname | Node ID | 관리망 IP | SSH 별칭 |
|---|---|---|---|---|
| K | `king` | `king` | `192.168.123.12` | `npuforge-k` |
| Q | `queen` | `queen` | `192.168.123.16` | `npuforge-q` |
| J | `jack` | `jack` | `192.168.123.33` | `npuforge-j` |

Scheduler 호스트(개발 PC): `192.168.123.26`

---

# 2. 2026-08-07

## 2.1 SSH 접속 확보

**상황.** 세 보드가 모두 `192.168.123.0/24`로 이동해 PC(`192.168.123.26`)와 같은 대역이 되었다. 세 대 모두 ping 및 tcp/22 응답 확인.

**문제.** `ssh-copy-id`가 3대 모두에서 즉시 실패했다.

```text
Permission denied, please try again.   (호스트당 2회, 3대 모두 즉시)
```

**원인.** 비밀번호가 틀린 것이 아니라 **TTY가 없었다.** SSH는 비밀번호를 stdin이 아니라 제어 터미널(`/dev/tty`)에서 읽는다. 자동화 환경에는 TTY가 없으므로 프롬프트가 뜨지 못하고 즉시 EOF로 실패했다. 호스트당 정확히 2회씩, 3대가 한 번에 끝난 패턴이 근거다.

**조치.** OpenSSH 9.7의 `SSH_ASKPASS_REQUIRE=force`를 사용해 TTY 없이 비밀번호를 전달했다.

```bash
ASKPASS=$(mktemp)
printf '#!/bin/sh\nprintf "%%s\\n" "$NPUFORGE_SUDO_PASS"\n' > "$ASKPASS"; chmod 700 "$ASKPASS"
SSH_ASKPASS="$ASKPASS" SSH_ASKPASS_REQUIRE=force DISPLAY=dummy \
  ssh-copy-id -i ~/.ssh/id_ed25519_npuforge.pub npuforge-k
```

작업 후 헬퍼 파일은 `shred -u`로 삭제했다.

**결과.** 3대 모두 키 인증 성공. 계정은 `pi`.

**PC 측 설정.**

- 전용 키 생성: `~/.ssh/id_ed25519_npuforge` (passphrase 없음, 자동화용)
- `~/.ssh/config`에 `npuforge-k` / `npuforge-q` / `npuforge-j` 별칭 추가

> 이 키는 자동화 전용이며 passphrase가 없다. 외부 공개 저장소나 신뢰할 수 없는 네트워크에 노출되지 않도록 한다.

## 2.2 하드웨어 실측 수집

**명령.** `scripts/collect-node-info.sh`를 3대에 원격 실행.

```bash
for pair in "k:npuforge-k" "q:npuforge-q" "j:npuforge-j"; do
  name="${pair%%:*}"; host="${pair##*:}"
  ssh "$host" 'bash -s' < scripts/collect-node-info.sh > "benchmarks/node-info/${name}.txt"
done
```

**원본.** `benchmarks/node-info/{k,q,j}.txt` (각 66줄)

**확정된 스펙.** 상세는 `environment-matrix.md` §2.1 참조.

```text
보드    FriendlyElec NanoPi R76S / friendlyelec,nanopi-r76s rockchip,rk3576
CPU     8코어 — little 2.016GHz(policy0) + big 2.208GHz(policy4)
RAM     4GB LPDDR4X (3,997,848 kB)
eMMC    64GB (rootfs 50G 여유)
NPU     2코어 (Core0, Core1), 300~950MHz, IOMMU 활성
        RKNPU driver v0.9.8
RKNN    Runtime 2.3.0 (c949ad889d@2024-11-07T11:35:33)
        librknnrt.so SHA-256 3대 동일
OS      Ubuntu 24.04, 커널 6.1.141, glibc 2.39
열센서  6개 (soc / bigcore / little-core / ddr / npu / gpu)
```

**중요.** NPU가 **2코어**다. RK3588은 3코어이므로 RK3588 기준 `core_mask` 예제를 그대로 쓸 수 없다.

`rknn_api.h`의 `rknn_core_mask` enum은 코어 3개까지 정의하지만(`RKNN_NPU_CORE_2`), RK3576에서 실제 사용 가능한 것은 `CORE_0`, `CORE_1`, `CORE_0_1`, `CORE_AUTO`, `CORE_ALL`이다.

## 2.3 NIC 스펙 확인

**배경.** 초기 수집에서 `eth1`이 `speed=1000`으로 나와 1G 포트로 오인할 소지가 있었다.

**명령.**

```bash
sudo apt-get install -y ethtool
sudo ethtool -i eth0 ; sudo ethtool eth0
sudo ethtool -i eth1 ; sudo ethtool eth1
```

**결과. 두 포트 모두 2.5G다.**

| 항목 | eth0 | eth1 |
|---|---|---|
| 드라이버 | `r8125` 9.010.01-NAPI | `r8125` 9.010.01-NAPI |
| PCIe 버스 | `0001:21:00.0` | `0000:01:00.0` |
| Supported link modes | 10/100/1000/**2500** baseT | 10/100/1000/**2500** baseT |
| 현재 링크 | 없음 (down) | 1000Mb/s Full |

`eth1`이 1000Mb/s인 것은 **1G 허브에 연결되어 협상된 결과**이지 포트 성능 한계가 아니다.

두 포트가 서로 다른 PCIe 버스에 있어 대역폭을 공유하지 않는다. 관리망/추론망 분리에 유리하다.

**결정.**

```text
eth1 → 관리망 (현재 1G 허브, 192.168.123.0/24)
eth0 → 추론망 (2.5G 스위치 도입 시, 10.20.0.0/24)
```

`eth0`이 3대 모두 비어 있으므로 추론망 전용으로 그대로 사용한다.

## 2.4 hostname 변경

**변경 전.**

| 노드 | hostname |
|---|---|
| K | `NanoPi-R76S` |
| Q | `NanoPi-R76S` |
| J | `localhost.localdomain` |

K와 Q가 동일해 로그·대시보드에서 구분이 불가능했다.

**명령.**

```bash
sudo hostnamectl set-hostname <king|queen|jack>
sudo sed -i "s/^127\.0\.1\.1.*/127.0.1.1\t<new>/" /etc/hosts
```

**결과.** `king` / `queen` / `jack` 로 변경 완료.

### 부수 발견: jack의 `/etc/hosts`가 비어 있었다

`jack`은 `/etc/hosts`가 **0바이트**였다. hostname이 `localhost.localdomain`이었던 원인이다.

king의 파일을 참조본으로 삼아 동일 내용으로 복원했다.

```text
127.0.0.1	localhost
::1		localhost ip6-localhost ip6-loopback
ff02::1		ip6-allnodes
ff02::2		ip6-allrouters

127.0.1.1	jack
```

**판단.** 세 보드는 **완전한 동일 복제본이 아니다.** `/etc/hosts` 부재와 Ubuntu 패치 레벨 차이(§2.5)가 함께 나타난 것으로 보아, jack은 다른 시점 또는 다른 경로로 세팅되었을 가능성이 있다.

### 작업 중 발견한 스크립트 함정

sudo 비밀번호를 파이프로 넘기는 헬퍼 함수에 파일 내용을 다시 파이프로 넣으면 충돌한다.

```bash
S() { printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" "$@"; }

printf "text\n" | S tee -a /etc/hosts    # 동작하지 않음
```

`sudo -S`가 stdin의 첫 줄을 비밀번호로 소비하므로 뒤따르는 명령은 EOF를 받는다. 파일을 쓸 때는 다음을 사용한다.

```bash
cat > /tmp/file.new <<'EOF'
...
EOF
printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" cp /tmp/file.new /etc/target
```

## 2.5 발견된 노드 간 불일치 (미해결)

세 노드는 "동일 OS 이미지"여야 한다(`02-HARDWARE-SETUP.md` §5.1). 현재 다음이 어긋나 있다.

| # | 항목 | king | queen | jack | 위험 |
|---|---|---|---|---|---|
| 1 | Ubuntu 패치 레벨 | 24.04.**3** | 24.04.4 | 24.04.4 | 라이브러리 차이가 노드별 성능 편차로 나타남 |
| 2 | gcc | `~24.04` | `~24.04.1` | `~24.04.1` | 위와 동일 |
| 3 | 미적용 업데이트 | 374개 | 280개 | 279개 | 위와 동일 |
| 4 | SSH 호스트 키 | 3대 완전 동일 (`<redacted-fingerprint>`) | | | 노드 식별 불가, MITM 탐지 불가 |
| 5 | CPU Governor | `ondemand` | `ondemand` | `ondemand` | 주파수 변동으로 측정 재현성 저하 |

**일치하는 항목** (문제 없음): 커널 6.1.141, glibc 2.39, Python 3.12.3, RKNN Runtime 2.3.0 및 `librknnrt.so` SHA-256, RKNPU driver v0.9.8, NPU 2코어, RAM 4GB, eMMC 64GB.

### ⚠️ 커널 업그레이드 금지

커널 `6.1.141`은 FriendlyElec BSP 커널이며 **RKNPU 드라이버 v0.9.8이 여기에 묶여 있다.**

`apt upgrade`가 커널을 교체하면 NPU가 동작하지 않을 수 있다. 패키지 동기화 시 반드시 커널 관련 패키지를 hold 한다.

```bash
sudo apt-mark hold linux-image-* linux-headers-* linux-modules-*
```

이 작업은 되돌리기 번거로우므로 **승인 후 실행**한다. 현재 미실행.

## 2.6 보드 소프트웨어 현황

| 항목 | 상태 |
|---|---|
| `librknnrt.so` | `/usr/lib/librknnrt.so` (2.3.0) |
| `rknn_api.h` | `/usr/include/rknn_api.h` |
| `rknn_matmul_api.h` | 설치됨 |
| `rknn_custom_op.h` | 설치됨 |
| `rknn_server` | `/usr/bin/rknn_server` (Toolkit2 연결 디버깅용) |
| `.rknn` 모델 파일 | **없음** — 변환 필요 |
| gcc | 13.3.0 |
| rustc | 미설치 (크로스컴파일 사용하므로 정상) |
| ethtool | 2026-08-07 설치 (king만. queen/jack 미설치) |

`rknn_server`가 있으므로 RKNN-Toolkit2의 연결 모드로 PC에서 보드의 NPU를 직접 호출해 모델을 검증할 수 있다.

---

## 2.7 C Wrapper 실기 검증

**배경.** `crates/npuforge-rknn/native/rknn_wrapper.c`는 RKNN API 문서만 보고 작성했고 실장비 검증 전이었다. 파일 상단에 그 사실을 명시해 두었다.

**검증 방법.** 실제 `rknn_api.h`의 시그니처를 추출해 대조한 뒤, 보드에서 직접 컴파일했다.

```bash
scp crates/npuforge-rknn/native/rknn_wrapper.{c,h} npuforge-k:~/npuforge-rknn-test/
ssh npuforge-k 'cd ~/npuforge-rknn-test && gcc -c -Wall -Wextra -O2 rknn_wrapper.c -o rknn_wrapper.o'
```

**결과. 경고 없이 컴파일 성공.** 작성한 시그니처가 실제 헤더와 일치했다.

| 항목 | 확인 결과 |
|---|---|
| `rknn_init(rknn_context*, void*, uint32_t, uint32_t, rknn_init_extend*)` | 일치 |
| `rknn_query(rknn_context, rknn_query_cmd, void*, uint32_t)` | 일치 |
| `rknn_inputs_set(rknn_context, uint32_t, rknn_input[])` | 일치 |
| `rknn_run(rknn_context, rknn_run_extend*)` | 일치 |
| `rknn_outputs_get(rknn_context, uint32_t, rknn_output[], rknn_output_extend*)` | 일치 |
| `rknn_outputs_release(rknn_context, uint32_t, rknn_output[])` | 일치 |
| `rknn_input` 필드 (`index/buf/size/pass_through/type/fmt`) | 일치 |
| `rknn_output` 필드 (`want_float/is_prealloc/index/buf/size`) | 일치 |
| `rknn_sdk_version` (`api_version[256]`, `drv_version[256]`) | 일치 |
| `rknn_context` | `uint64_t` (aarch64) |
| `RKNN_SUCC` | 0 |

**추가 확인.** `rknn_set_core_mask(rknn_context, rknn_core_mask)`가 존재한다. `rknn_core_mask` enum은 코어 3개까지 정의하지만 RK3576은 2코어이므로 `CORE_0`, `CORE_1`, `CORE_0_1`, `CORE_AUTO`, `CORE_ALL`만 유효하다.

**미해결.** `npf_rknn_get_runtime_version()`은 컨텍스트 없이 `rknn_query`를 호출하도록 작성했는데, 이 호출이 실제로 성공하는지는 모델이 있어야 확인 가능하다. 실패한다면 노드 시작 시 임시 컨텍스트를 만들어 조회한 뒤 캐시하는 방식으로 바꾼다.

## 2.8 Thread-safety 테스트 프로그램 작성

**파일.** `crates/npuforge-rknn/native/thread_safety_test.c`

**빌드 확인.**

```bash
gcc -O2 -Wall -Wextra -o thread_safety_test thread_safety_test.c -lrknnrt -lpthread
# 경고 없이 성공, 71,888 bytes
```

**검증 시나리오.**

| # | 구성 | 확인 대상 |
|---|---|---|
| 기준선 | 스레드 1, 전용 context | 단일 스레드 처리량 |
| 1 | 스레드 2, **context 공유** | 동일 context 동시 호출 가능 여부 |
| 2 | 스레드 2, 각자 전용 context, `CORE_AUTO` | 전용 context 병렬 가능 여부 |
| 3 | 스레드 2, 각자 전용 context, `CORE_0` / `CORE_1` 분리 | 명시적 코어 분리 효과 |
| 4 | 스레드 4 (코어 수 2 초과) | 과다 워커의 역효과 |

**판정 기준.**

```text
시나리오 1에서 err > 0        → 동일 context 동시 호출 불가
                                모델당 전용 워커 스레드로 직렬화 필요
시나리오 2가 기준선 대비 ~2배 → 전용 context 로 2-way 병렬 가능, worker_count = 2
시나리오 2가 ~1배             → 런타임 내부 직렬화, worker_count = 1 유지
시나리오 3 > 시나리오 2       → 명시적 코어 분리가 유효
시나리오 4 < 시나리오 2       → 코어 수를 넘는 워커는 역효과
```

**⛔ 실행 보류. 모델 파일이 없다.**

보드에 `.rknn` 파일이 하나도 없다. 프로그램은 준비 완료 상태이며, 모델이 생기는 즉시 실행하면 된다.

```bash
ssh npuforge-k 'cd ~/npuforge-rknn-test && ./thread_safety_test model.rknn 50'
```

### 모델 확보 경로

| 경로 | 가능 여부 | 비고 |
|---|---|---|
| 보드에서 다운로드 | ✗ | `curl`, `wget` 미설치 |
| 보드에서 변환 | ✗ | RKNN-Toolkit2는 x86_64 Linux 전용 |
| **PC WSL2에서 변환** | **✓** | WSL2 Ubuntu 확인됨 (현재 Stopped) |
| `rknn_server` 연결 모드 | ✓ | Toolkit2가 PC에서 보드 NPU를 직접 호출 |

`rknn_server`가 보드에 설치되어 있으므로, Toolkit2 구축 후 PC에서 보드 NPU를 원격 호출해 모델을 즉석 검증할 수 있다.

## 2.9 노드 일치 스크립트 준비 (미실행)

**파일.** `scripts/fix-node-consistency.sh`

기본 동작이 DRY RUN이며 `--apply`를 줘야 실제로 실행된다. `--only`로 단계를 나눠 실행할 수 있다.

| 단계 | `--only` 값 | 내용 | 위험도 |
|---|---|---|---|
| 1 | `kernelhold` | 커널 패키지 `apt-mark hold` | 낮음 |
| 2 | `hostkeys` | SSH 호스트 키 재생성 + PC `known_hosts` 정리 | 낮음 |
| 3 | `packages` | 기본 패키지 설치 (curl, ethtool, iperf3, chrony 등) | 낮음 |
| 4 | `chrony` | 시간 동기화 활성화 | 낮음 |
| 5 | `upgrade` | 패키지 업그레이드 (24.04.3 → 24.04.4) | **높음** |
| 6 | `governor` | CPU Governor → `performance` | 중간 |

**안전 장치.**

- 5단계는 커널 hold 여부를 먼저 확인하고, hold되지 않았으면 중단한다
- 6단계는 발열이 올라가므로 S0 열 특성 측정 후 적용을 권장한다
- DRY RUN으로 3대 접속 및 단계 출력 확인 완료

**권장 실행 순서.**

```bash
./scripts/fix-node-consistency.sh --apply --only kernelhold
./scripts/fix-node-consistency.sh --apply --only hostkeys
./scripts/fix-node-consistency.sh --apply --only packages,chrony
./scripts/fix-node-consistency.sh --apply --only upgrade     # 단독 실행
# (S0 측정 후)
./scripts/fix-node-consistency.sh --apply --only governor
```

업그레이드 후 반드시 확인할 것:

```bash
ssh npuforge-k 'uname -r'                                              # 6.1.141 유지?
ssh npuforge-k 'printf "$NPUFORGE_SUDO_PASS\n" | sudo -S cat /sys/kernel/debug/rknpu/version'  # NPU 살아있나?
ssh npuforge-k 'sha256sum /usr/lib/librknnrt.so'                       # 73993ed4... 유지?
```

---

# 3. 미완료 작업

| # | 작업 | 상태 | 비고 |
|---|---|---|---|
| 1 | RKNN thread-safety 검증 | 진행 예정 | `worker_count` 결정. 모델 파일 필요 |
| 2 | `rknn_wrapper.c`를 실제 헤더로 검증 | 진행 예정 | 미검증 상태로 작성됨 |
| 3 | 노드 간 불일치 해소 (§2.5) | 스크립트 준비 후 승인 대기 | 커널 hold 필수 |
| 4 | SSH 호스트 키 재생성 | 스크립트 준비 후 승인 대기 | |
| 5 | CPU Governor → `performance` | 벤치마크 직전 적용 | |
| 6 | 기본 패키지 설치 | 미실행 | `02-HARDWARE-SETUP.md` §5.2 |
| 7 | 추론망 구성 (`eth0`, 10.20.0.0/24) | 2.5G 스위치 도입 후 | |
| 8 | 모델 변환 환경 구축 | 미실행 | Toolkit2를 Runtime 2.3.0에 맞출 것 |

## 3.1 다음 단계: 모델 확보

thread-safety 검증(1번)이 모델 파일에 막혀 있고, 모델은 다른 모든 실장비 작업의 전제이기도 하다. 따라서 이것이 최우선이다.

```text
PC WSL2 (Ubuntu, 현재 Stopped)
  → rknn-toolkit2==2.3.0 설치        ← Runtime 2.3.0에 맞춤
  → YOLOv8n ONNX 확보
  → rknn.config(target_platform='rk3576')
  → yolov8n.rknn 생성
  → scp 로 3노드 배포 + SHA-256 확인
  → thread_safety_test 실행
  → environment-matrix.md §3.1, §6 기록
```

**주의.** Toolkit2 버전이 Runtime보다 높으면 변환한 모델이 로딩되지 않을 수 있다. `rknn-toolkit2==2.3.0`을 우선 시도한다.

---

## 2.10 Scheduler 호스트 실측 (노트북)

**대상.** Samsung 370E5J 계열 구형 노트북, `192.168.123.14`

**측정 결과.** 상세는 `environment-matrix.md` §4.2.

```text
CPU     Intel i7-4712MQ (Haswell, 4C/8T @2.30GHz)
RAM     3.5GB (가용 1.8GB)          ← 노드(4GB)보다 적음
NIC     RTL8111/8168 (r8169), 1GbE 상한. 2.5G 미지원
USB     Bus 004 = USB 3.0 (5000M, 4포트). 나머지는 USB 2.0
TB      없음
Docker  설치됨
아키텍처 x86_64
```

### 링크 속도 100Mb/s 문제 (해결)

최초 측정에서 `Speed: 100Mb/s`로 협상되어 있었다. 포트는 `1000baseT/Full`을 지원하므로 물리 계층 문제였다.

케이블 교체 후 **1000Mb/s로 정상화**되었다.

**영향 분석.** 방치했다면 JPEG 100KB 기준 약 125 FPS에서 링크가 포화되어, NPU 확장 효율이 아니라 케이블을 측정할 뻔했다. 보드 3대는 처음부터 1000Mb/s였으므로 허브가 아니라 노트북 쪽 케이블이 원인이었다.

**후속 조치.** 매 실험 전 링크 속도를 확인하는 절차를 벤치마크 스크립트에 넣는다.

```bash
ethtool enp3s0 | grep Speed
```

### 판정

| 역할 | 판정 | 근거 |
|---|---|---|
| 모델 변환 | **적합** | x86_64 Linux + Docker |
| 개발용 Scheduler (M2~M5) | **충분** | 링크 속도는 기능 정확성과 무관 |
| 공식 벤치마크 (JPEG) | **조건부 적합** | 실측 FPS 확인 후 판단 |
| 공식 벤치마크 (Raw RGB, S6) | **부적합** | 1GbE 초과 |

**2.5G 어댑터 구매는 보류한다.** 노드당 실제 FPS를 모르는 상태에서는 필요 여부를 판단할 수 없다. S0/S1 측정 후 결정한다.

노드당 40 FPS 가정 시 3노드 120 FPS × 100KB ≈ 96 Mbps로 1GbE에 여유가 있다. 측정으로 판단하는 것이 이 프로젝트의 방식과도 일치한다.

**RAM 3.5GB가 NIC보다 실질적인 제약이다.** 대응은 하드웨어 구매가 아니라 운용 방침으로 한다 — 공식 측정 중에는 Prometheus·Dashboard를 중지하고 `npuforge-bench`가 JSONL 원본만 기록한다.

### 미확인 항목

```bash
cat /etc/os-release      # 프롬프트가 [root@localhost ~]# 형태 — 배포판 확인 필요
uname -r
df -h /                  # Docker 이미지 5~8GB 필요
```

hostname이 `localhost`다. 결과 파일에 측정 호스트가 남아야 하므로 이름을 부여한다(제안: `dealer`).

## 2.11 모델 변환 환경 구축

**결정.** WSL2가 아니라 **노트북(x86_64 Linux)** 에 구축한다. Docker가 이미 설치되어 있고, RKNN-Toolkit2가 요구하는 x86_64 Linux 조건을 만족한다. WSL2를 별도로 세팅할 이유가 없다.

**Docker로 감싸는 이유.** 변환 결과가 호스트 환경에 따라 달라지면 재현성이 깨진다. 이미지가 Python·Toolkit·의존성 버전을 고정하므로 누구 PC에서 돌려도 같은 `.rknn`이 나온다. 오픈소스 공개 시 "이 이미지로 재현하세요"가 가능해진다.

**작성한 파일.**

```text
tools/model-converter/
├── Dockerfile            Ubuntu 22.04 + rknn-toolkit2==2.3.0
├── requirements.txt
├── convert_yolov8n.py    ONNX -> RKNN, 메타데이터 자동 기록
└── README.md             사용법 및 배포 절차
```

**버전 고정.** Toolkit 2.3.0은 보드의 Runtime 2.3.0에 맞춘 값이다. Toolkit이 Runtime보다 높으면 변환된 모델이 로딩되지 않을 수 있다.

**타깃 플랫폼.** `target_platform='rk3576'`으로 고정했다. `rk3588`로 변환한 `.rknn`은 RK3576에서 동작하지 않는다.

**재현성 기록.** `convert_yolov8n.py`는 변환 시 다음을 JSON으로 남긴다.

```text
ONNX SHA-256 / RKNN SHA-256 / calibration manifest SHA-256
calibration 이미지 수 / 양자화 방식 / 변환 옵션 전체
toolkit 버전 / python 버전 / 플랫폼
```

calibration 이미지 목록은 정렬해 고정한다. 순서가 양자화 결과에 영향을 주기 때문이다.

## 2.12 Scheduler 호스트(`dealer`) 접속 및 설정

**대상.** `192.168.123.14`, 계정 `yoo2`

### 배포판 확인: Rocky Linux 9.7

SSH 배너에서 `OpenSSH_8.7` + `gssapi-keyex`가 보여 RHEL 계열임을 먼저 파악했다. 확인 결과 **Rocky Linux 9.7**이다.

```text
PRETTY_NAME  Rocky Linux 9.7 (Blue Onyx)
kernel       5.14.0-611.13.1.el9_7.x86_64
glibc        2.34
패키지 관리자 dnf
Docker       29.2.1 (overlayfs)
디스크       60GB 여유
Swap         3.9GB
```

**앞서 실행한 `sudo apt install ...`은 조용히 실패했었다.** `2>/dev/null`로 오류가 가려졌고, `ethtool`·`lspci`·`dmidecode`가 이미 설치되어 있어 출력은 정상으로 보였다. 이 호스트에서는 `dnf`를 써야 한다.

### 접속 확보 과정에서 겪은 것

**1차 실패.** `printf` 기반 askpass 헬퍼가 비밀번호를 제대로 내보내지 못했다. 헬퍼 출력을 직접 확인해 원인을 좁혔다.

```bash
printf "[%s]\n" "$("$ASKPASS")"    # 실제로 무엇이 나오는지 확인
```

heredoc 방식으로 바꾸니 정상 동작했다.

```sh
#!/bin/sh
cat <<'PW'
<password>
PW
```

**2차 문제 — sudo 불가.** `yoo2`가 `wheel` 그룹에 없었다(`id -nG yoo2` → `yoo2`). Rocky는 기본적으로 사용자를 `wheel`에 넣지 않는다.

**3차 문제 — root SSH 차단.** `PermitRootLogin`이 막혀 있어 root로 직접 붙을 수 없었다.

**해결 — `su` 승격.** `su`는 stdin이 아니라 제어 터미널에서 비밀번호를 읽으므로 `ssh -tt`로 PTY를 할당해야 한다. 그리고 **프롬프트가 뜰 시간을 줘야 한다.**

```bash
# 실패: su 가 읽기 전에 비밀번호가 흘러가 에코됨
printf 'PW\n' | ssh -tt host 'su -c "..."'

# 성공: 지연을 넣는다
( sleep 3; printf 'PW\n'; sleep 2 ) | ssh -tt host 'su -c "..."'
```

이 패턴은 §2.1의 SSH 비밀번호 문제와 같은 원인(TTY 부재)이지만 해법이 다르다. SSH는 `SSH_ASKPASS_REQUIRE=force`로 우회되고, `su`는 PTY 할당이 필요하다.

### 적용한 변경

| 항목 | 변경 |
|---|---|
| hostname | `localhost.localdomain` → **`dealer`** |
| `yoo2` 그룹 | `wheel` 추가 (sudo 가능) |
| `yoo2` 그룹 | `docker` 추가 (sudo 없이 docker 사용) |
| SSH 키 | `id_ed25519_npuforge` 설치 |
| SSH 별칭 | `npuforge-dealer` |

`dealer`는 카드 딜러에서 따왔다. 노드가 `king`/`queen`/`jack`이므로 명칭 체계가 일관된다.

### ⚠️ 호스트와 노드의 배포판이 다르다

| | `dealer` | `king`/`queen`/`jack` |
|---|---|---|
| 배포판 | Rocky Linux 9.7 | Ubuntu 24.04 |
| glibc | 2.34 | 2.39 |
| 패키지 관리자 | `dnf` | `apt` |

**바이너리 배포 방향은 안전하다.** 낮은 glibc(2.34)로 빌드한 바이너리는 높은 glibc(2.39)에서 동작한다. 반대는 성립하지 않는다. 따라서 `dealer`에서 크로스컴파일해 보드로 배포하는 것은 문제없다.

`scripts/fix-node-consistency.sh`는 `apt` 전용이며 노드 대상이므로 그대로 두어도 된다. 호스트까지 다루는 스크립트를 쓸 때는 패키지 관리자를 분기해야 한다.

## 2.13 모델 변환 이미지 빌드

**1차 시도 실패.** Dockerfile이 `validate_rknn.py`를 COPY 하는데 파일이 존재하지 않았다.

```text
ERROR: "/validate_rknn.py": not found
```

**부수 교훈.** 백그라운드 실행 시 `docker build ... | tail -40` 형태로 파이프를 걸면 종료 코드가 `tail`의 것이 되어 **실패가 성공으로 보고된다.** 로그를 파일로 남기고 종료 코드를 따로 확인하도록 바꿨다.

```bash
docker build -t img . > /tmp/build.log 2>&1; echo "EXIT=$?"; tail -25 /tmp/build.log
```

**조치.** `validate_rknn.py`를 작성했다. 변환된 모델의 입출력 shape을 확인하고, `onnxruntime`이 있으면 ONNX 원본과 코사인 유사도를 비교한다. 기본 기준은 0.98이다.

DEV-REQ §2.2의 검증 대상 중 "ONNX 결과 ↔ RKNN Simulator 결과" 비교에 해당한다. 보드 3대 실측 비교는 별도로 수행한다.

### 개선 여지: 이미지 용량

빌드 로그에서 `rknn-toolkit2`가 의존성으로 `torch`를 끌어오고, 그 과정에서 **NVIDIA CUDA 라이브러리를 수백 MB씩 내려받는 것**을 확인했다.

```text
nvidia_cusolver_cu12   124.2 MB
nvidia_cusparse_cu12   196.0 MB
nvidia_nccl_cu12       176.2 MB
...
```

`dealer`에는 GPU가 없으므로 전부 사용되지 않는다. CPU 전용 torch를 먼저 설치하면 수 GB를 줄일 수 있다.

```dockerfile
RUN python3 -m pip install torch --index-url https://download.pytorch.org/whl/cpu \
    && python3 -m pip install "rknn-toolkit2==${RKNN_TOOLKIT_VERSION}"
```

디스크 여유가 51GB 남아 있어 당장 문제는 아니다. 빌드가 완료된 뒤 최적화한다.

## 2.14 YOLOv8n ONNX 확보

### ⚠️ 표준 Ultralytics export는 RKNN에 부적합하다

RKNN용 YOLOv8은 **Rockchip이 수정한 exporter**로 만들어야 한다. 표준 Ultralytics export는 DFL·NMS 후처리가 ONNX 그래프에 포함되는데, 이 연산들이 NPU에 매핑되지 않아 CPU fallback이 대량 발생한다.

수정판은 **decode 이전의 raw 텐서를 출력**하고 후처리를 CPU에서 따로 수행한다.

```text
공식 원본 : 출력 1개 (decode·NMS 포함)
최적화판  : 출력 3그룹
            [1,64,80,80]  박스 좌표
            [1,80,80,80]  80개 클래스별 confidence
            [1,1,80,80]   confidence 합
```

이것이 `environment-matrix.md` §6의 "CPU fallback 연산 목록" 항목과 직결된다. **잘못 export하면 NPU가 아니라 CPU를 측정하게 된다.**

### 확보한 파일

`rknn_model_zoo`가 사전 최적화된 ONNX를 배포한다. 직접 export할 필요가 없어 위험이 줄었다.

```text
출처      airockchip/rknn_model_zoo  examples/yolov8
원본      airockchip/ultralytics_yolov8
경로      ~/npuforge/models/yolov8n.onnx  (dealer)
크기      12,650,184 bytes
SHA-256   0c8716701f471067932b797eeb67c8e5db47c693c2557c881d7679ec12e21bc5
형식      PyTorch 2.0 export
```

**RK3576이 공식 지원 목록에 있다.**

```text
RK3562, RK3566, RK3568, RK3576, RK3588, RV1126B, RV1109, RV1126, RK1808, RK3399PRO
```

### 라이선스

`rknn_model_zoo` 저장소는 Apache-2.0이지만 **모델 자체는 AGPL-3.0**이다(Ultralytics 원본 상속). 저장소 라이선스와 데이터 라이선스는 별개다.

상세와 대응 방침은 `MODEL_LICENSES.md` 참조. 요약하면 모델 파일을 저장소에 포함하지 않고 사용자가 직접 내려받게 한다.

---

## 2.15 모델 변환 성공

### onnx 버전 충돌

첫 변환이 실패했다.

```text
AttributeError: module 'onnx' has no attribute 'mapping'
```

**원인.** `rknn-toolkit2`의 의존성 명세가 onnx 버전을 제한하지 않아 최신 버전(1.22.0)이 설치되었다. `onnx.mapping`은 onnx 1.16에서 제거되었는데 rknn-toolkit2 2.3.0이 이를 사용한다.

**해결.** `onnx==1.14.1`로 고정하니 즉시 변환에 성공했다. Dockerfile에 고정과 검증 단계를 넣었다.

```dockerfile
RUN python3 -m pip install "onnx==1.14.1" \
    && python3 -c "import onnx; assert hasattr(onnx, 'mapping'), 'onnx.mapping 없음'"
```

**함께 적용한 개선.** torch를 CPU 전용 인덱스에서 설치하도록 바꿨다. GPU가 없는 호스트에서 NVIDIA CUDA 라이브러리를 수 GB 받는 낭비를 없앤다.

### FP16 모델 생성

Calibration 데이터가 확정되지 않아 INT8 대신 FP16으로 먼저 변환했다. **양자화 없이도 thread-safety 검증에는 지장이 없다.**

```text
파일      yolov8n-fp16.rknn
크기      9,645,065 bytes
SHA-256   459602ea70479c1ce4fdd7419aa81e10e2f795fe6fe87444f3607f25b7054c0f
```

3노드에 배포하고 SHA-256이 모두 일치함을 확인했다. 3대 모두에서 테스트 프로그램 컴파일도 성공했다.

## 2.16 Thread-safety 검증 — 진행 중

### 예비 관측 (반복 2회)

```text
RKNN api        2.3.0 (c949ad889d@2024-11-07T11:35:33)
RKNN driver     0.9.8
입력/출력 개수  1 / 9              최적화판의 3그룹 × 3
입력 크기       1,228,800 bytes    = 640×640×3, 문서 계산과 일치
FP16 추론시간   78.8 ~ 116.1 ms
```

**시나리오 1(context 공유, 2스레드)에서 오류가 0건이었다.** 반복 2회는 표본이 너무 적어 단정할 수 없으므로 20회로 재측정 중이다.

FP16이 약 100ms이므로 노드당 10 FPS 수준이다. INT8은 통상 3~5배 빠르므로 30~50 FPS를 예상한다. **이 수치가 2.5G 스위치 구매 판단의 근거가 된다.**

### 실행 과정에서 겪은 함정 두 가지

**1. 파이프의 `head`가 출력을 삼킨다.**

```bash
ssh host './test model 30' | grep -v ... | head -70    # 출력 0바이트
```

`head`가 조기에 파이프를 닫아 SIGPIPE가 발생하고 원격 명령이 중단되었다. 백그라운드 작업은 exit 0으로 보고되어 성공처럼 보였다.

**2. 파일 리다이렉트 시 블록 버퍼링 + SIGHUP.**

```bash
ssh host './test model 50 > run50.log 2>&1'
```

stdout이 파일이면 libc가 라인 버퍼링 대신 **블록 버퍼링**을 쓴다. SSH 세션이 끊기며 프로세스가 SIGHUP으로 종료되었고, 버퍼에 있던 출력이 통째로 유실되었다. 로그에는 stderr로 나간 한 줄만 남았다(stderr는 항상 무버퍼).

**해결.** 세션에서 분리하고 라인 버퍼링을 강제한다.

```bash
nohup bash -c 'stdbuf -oL -eL ./test model 20 > run20.log 2>&1; echo DONE=$? > done.marker' &
```

완료 마커 파일을 폴링해 결과를 가져온다. **장시간 실행되는 벤치마크에도 같은 패턴이 필요하다** — `run-benchmark.sh`의 무인 실행 요구사항(`01-TECHSPEC.md` §20.4)에 반영한다.

---

# 2.17 ⚠️ 고부하에서 보드가 재부팅된다 (미해결)

## 증상

스레드 수 스윕(3~8스레드)을 실행하면 **`king`과 `jack`이 재부팅된다.** `queen`은 동일 테스트를 완주한다.

| 노드 | 부팅 횟수 | uptime (2026-08-10 02:00) | 스윕 결과 |
|---|---:|---|---|
| `king` | **13** | 15분 | **재부팅 3회** (01:26, 01:38, 01:45) |
| `queen` | 5 | **3일 17시간** | **완주** |
| `jack` | 5 | 26분 | 재부팅 |

`king`은 다른 두 대보다 부팅 횟수가 8회 많다. 모두 오늘 스윕을 돌린 시점과 일치한다.

## 하드 리셋이다

재부팅 직전 로그에 **종료 시퀀스가 전혀 없다.** SSH 세션이 열린 직후 로그가 그냥 끊긴다.

```text
Aug 10 01:45:45 king sshd[1586]: Accepted publickey for pi ...
Aug 10 01:45:45 king systemd-logind[488]: New session 4 of user pi.
(로그 끝 — 커널 패닉도, 종료 메시지도 없음)
```

kernel panic, OOM killer, thermal shutdown 메시지가 없다. **전원 차단 또는 watchdog에 의한 하드 리셋**으로 보인다.

## 원인 후보

| 후보 | 근거 | 판정 |
|---|---|---|
| **전원 공급 부족** | 하드 리셋, 로그 없음, 노드별 편차 | **유력** |
| 발열 | 재부팅 시점 온도 45~50°C | **배제** (임계치와 거리 멂) |
| 메모리 부족 | 가용 3.2GB, OOM 로그 없음 | 배제 |
| 보드 개체 불량 | `queen`만 정상 | 가능 |

전원이 유력한 이유는 **부하 특성**과 맞기 때문이다. 8스레드는 CPU 8코어와 NPU 2코어를 동시에 최대로 쓴다. 순간 전류가 어댑터 용량을 넘으면 전압이 떨어지고 보드가 리셋된다. 로그가 남지 않는 것도 이와 일치한다.

`queen`이 3일 17시간 무중단으로 같은 테스트를 완주했다는 점이 **소프트웨어 문제가 아니라 개체별 하드웨어 조건 차이**임을 시사한다.

## 문서의 전원 가정을 정정해야 한다

`02-HARDWARE-SETUP.md` §8은 **USB-C PD 어댑터**를 전제하는데, 커널 로그의 레귤레이터 이름은 다음과 같다.

```text
vcc12v_dcin      12V DC 입력
vcc_sys
rk806-regulator
```

**실제 전원 입력 방식을 확인해야 한다.** 12V DC라면 USB-C PD 전제로 쓴 §8 전체가 틀렸다.

## 프로젝트에 미치는 영향 — 심각하다

공식 벤치마크는 **300초 지속 부하 × 5회 반복 × 143 run, 총 22시간**이다(`01-TECHSPEC.md` §20.4).

지금 상태로는:

- 측정 중 노드가 재부팅되어 run이 무효가 된다
- 재부팅을 "노드 장애"로 기록하면 **소프트웨어 장애 감지 성능을 잘못 측정**하게 된다
- S4 장애 복구 실험에서 의도한 장애와 전원 문제를 구분할 수 없다
- 무인 야간 실행이 불가능하다

**S0 열 특성 측정 전에 반드시 해결해야 한다.**

## 조치 계획

1. **세 노드의 전원 어댑터 확인** — 제조사, 모델, 정격 출력. 물리 확인 필요
2. 입력 방식 확인 — USB-C PD인지 12V DC 배럴잭인지
3. `queen`의 어댑터를 `king`에 물려 재현 시도 — 어댑터 문제인지 보드 문제인지 판별
4. 동일 모델 어댑터 3개로 통일 (`infrastructure.md` §5 구매 목록)
5. 해결 후 스윕 재실행으로 3대 일관성 확인

**해결 전까지 고부하 테스트를 반복하지 않는다.** 재부팅을 반복시켜 eMMC 손상 위험을 키울 이유가 없다.

## 유효한 데이터는 남아 있다

`queen`이 전체 스윕을 완주했으므로 **thread-safety 결론(§3.1)은 유효하다.** 다만 3대 재현성 확인은 전원 문제 해결 후로 미룬다.

## 정정: 두 개의 서로 다른 현상이었다

절대 시각으로 부팅 이력을 다시 확인한 결과, 위 분석을 정정한다. **uptime만 비교해 하나의 원인으로 묶은 것이 성급했다.**

### 사건 A — 부하 중 개별 재부팅 (조사 대상)

```text
01:26:16  king  재부팅
01:34:40  jack  재부팅
01:38:12  king  재부팅
01:45:58  king  재부팅
```

모두 스윕 테스트 실행 시각과 일치한다. 이 구간 내내 **`queen`은 3일 17시간 무중단이었다.**

부하와 상관관계가 있고 노드별로 다르게 나타나므로, 전원 공급 또는 개체 편차 가설은 **이 사건에 대해서는 유효하다.**

### 사건 B — 3대 동시 재부팅 (부하와 무관)

```text
king   이전 부팅 종료  02:01:00
queen  이전 부팅 종료  02:05:20
jack   이전 부팅 종료  02:05:10
       ↓ 약 27분 정전
세 대 모두 약 02:32 부팅   (04:19 기준 uptime 1시간 47분으로 동일)
```

세 대가 4분 이내에 함께 내려갔고 **27분간 꺼져 있다가 함께 올라왔다.** 이 시각에는 부하 테스트를 실행하지 않았다.

이는 **공용 전원 차단**(정전, 멀티탭 차단, 물리적 재배치)이며 사건 A와 원인이 다르다. 부하로 인한 리셋은 즉시 재부팅되지 실행이 27분간 멈추지 않는다.

**따라서 "고부하가 3대를 모두 재부팅시킨다"는 앞선 서술은 과했다.** 부하와 연결된 것은 사건 A뿐이다.

### 사건 B의 정체: 전원 재배치 작업

사용자 확인 결과, **세 보드의 전원을 각각 독립 소스로 분리하는 작업**이 있었다. 02:05의 3대 동시 정지와 27분 공백이 이 작업 시간과 일치한다.

**따라서 사건 B는 장애가 아니라 계획된 물리 작업이다.** 이것도 원인 미상 재부팅으로 기록했다면 잘못된 추적이 될 뻔했다.

`02-HARDWARE-SETUP.md` §8.1의 "멀티포트 충전기 하나에 세 대를 몰지 않음" 요구가 이로써 충족되었다.

### 진단에 남은 것

| 사건 | 시각 | 원인 | 상태 |
|---|---|---|---|
| A: king ×3, jack ×1 | 01:26~01:45 | 전원 재배치 **이전** 구성에서 고부하 | **재검증 필요** |
| B: 3대 동시 27분 | 02:01~02:32 | 전원 재배치 작업 | 해결 (장애 아님) |

**사건 A는 재배치 이전에 발생했다.** 전원이 독립 소스로 분리된 지금은 재현되지 않을 수 있다. 동일 조건으로 재검증한다.

### 교훈: uptime 비교만으로 판단하지 않는다

처음에 `uptime`만 보고 "고부하가 3대를 재부팅시킨다"고 결론지었으나, 절대 시각으로 보니 서로 다른 두 사건이었다. 게다가 하나는 장애가 아니라 계획된 작업이었다.

**벤치마크 중 노드 재시작을 기록할 때는 절대 시각과 작업 이력을 함께 남긴다.** 그렇지 않으면 물리 작업을 소프트웨어 장애로 오독하게 된다. 이 문서의 존재 이유가 여기에 있다.

## 2.17.1 원인 확정: `king`의 부트로더 펌웨어가 구버전이다

전원 가설을 두 차례 검증한 끝에 실제 원인을 찾았다.

### 전원이 원인이 아니라는 증거

| 관측 | 함의 |
|---|---|
| 3포트 공유 전원 시절에도 `queen`은 8스레드 완주 | 공유 전원 자체가 문제가 아니다 |
| 개별 전원으로 교체 후에도 `king`은 5스레드에서 리셋 | 어댑터 용량 문제가 아니다 |
| 세 어댑터가 동일 조건 | 개체별 어댑터 차이가 아니다 |

### 펌웨어 비교

```bash
grep -oE 'androidboot\.fwver=[^ ]*' /proc/cmdline
```

| 구성요소 | `king` | `queen` | `jack` |
|---|---|---|---|
| DDR init | **v1.09** | v1.13 | v1.13 |
| SPL | **v1.07** | v1.09 | v1.09 |
| **BL31 (ATF)** | **v1.17** | **v1.24** | **v1.24** |
| BL32 | **v1.05** | v1.10 | v1.10 |
| U-Boot | **2025-07-17** | 2026-07-10 | 2026-07-10 |
| PMIC 초기화 | **`ON:0x20 OFF:0x2`** | `ON:0x40 OFF:0x0` | `ON:0x40 OFF:0x0` |

`queen`과 `jack`은 완전히 일치하고 **`king`만 약 1년 낡았다.**

**BL31은 ARM Trusted Firmware이며 Rockchip 플랫폼에서 DVFS와 전압 조절을 담당한다.** v1.17과 v1.24 사이에 전압 테이블이나 DVFS 로직이 바뀌었다면, 구버전이 고부하 전압을 감당하지 못해 리셋되는 것이 정확히 관측된 증상이다.

DDR 펌웨어 차이(v1.09 vs v1.13)도 메모리 트래픽이 큰 다중 스레드 조건에서 불안정성을 유발할 수 있다.

PMIC 초기화 레지스터가 다른 것은 펌웨어 차이의 결과다.

### 잘못된 진단으로 인한 비용

전원을 의심해 사용자가 어댑터 3개를 전부 교체했으나 원인이 아니었다. 개별 전원 구성 자체는 `02-HARDWARE-SETUP.md` §8.2의 요구사항을 충족하므로 낭비는 아니지만, **진단 방향을 잘못 잡아 시간을 소모했다.**

`queen`이 공유 전원에서도 8스레드를 완주했다는 사실이 이미 전원 가설을 약화시키고 있었는데, 그 신호를 충분히 무겁게 다루지 않았다.

### 문서의 구멍

`environment-matrix.md`에 커널·glibc·RKNN 버전은 있었으나 **부트로더 펌웨어 항목이 없었다.** `collect-node-info.sh`도 수집하지 않았다.

"동일한 3대"를 검증한다면서 **전력 관리를 담당하는 계층을 빠뜨린 것**이다. 두 가지를 모두 보완했다(2026-08-10).

### 이미지 버전이 특정되었다

```text
/etc/rom-version
  king   20251222     2025-12-22 이미지
  queen  20260721     2026-07-21 이미지
  jack   20260721
```

`king`만 7개월 낡은 이미지다. 펌웨어 차이의 출처가 여기다.

`/etc/friendlyelec-release`는 세 대가 동일하다(`BOARD=NanoPi-R76S`, `LINUXFAMILY=nanopi-m5`, `BRANCH=dev`). 구분되는 것은 `rom-version`이므로 **이 값을 노드 일치 검증 항목에 포함한다.**

### 조치: `king` OS 재설치 (2026-08-10 결정)

부트로더만 갱신하는 대신 **OS를 재설치한다.** 근거는 다음과 같다.

- `king`은 OS 패치 레벨도 뒤처져 있다(24.04.3 vs 24.04.4). 재설치로 함께 해결된다
- 진단 과정에서 6회 하드 리셋시켜 파일시스템 상태를 신뢰하기 어렵다
- 부트로더만 갱신하는 절차는 `rkdeveloptool`/`eflasher`가 필요해 오히려 복잡하다

**목표 이미지: `rom-version = 20260721`** (NanoPi-R76S용 Ubuntu 24.04, FriendlyElec 배포본)

재설치 후 `scripts/setup-node.sh`로 자동 세팅한다.

```bash
./scripts/setup-node.sh 192.168.123.12 king npuforge-k
```

이 스크립트가 수행하는 것:

| 단계 | 내용 |
|---|---|
| 1 | SSH 키 설치 (`SSH_ASKPASS_REQUIRE=force` 사용) |
| 2 | `~/.ssh/config` 별칭 등록 |
| 3 | hostname 설정, `/etc/hosts` 정리 |
| 4 | **SSH 호스트 키 재생성** (이미지 복제 시 중복 방지) |
| 5 | **커널 패키지 hold** (RKNPU 드라이버 보호) |
| 6 | 기본 패키지 설치, chrony 활성화 |
| 7 | **기준 노드(`queen`)와 환경 비교** — `rom-version`, `fwver`, 커널, glibc, RKNN 버전 및 해시, NPU 코어 수, RAM |

7단계가 핵심이다. 재설치가 목적을 달성했는지 스크립트가 직접 판정한다.

### 재설치 후 검증 순서

```bash
# 1. 실측 수집
ssh npuforge-k 'bash -s' < scripts/collect-node-info.sh > benchmarks/node-info/king.txt

# 2. 펌웨어 일치 확인 (setup-node.sh 가 자동 비교하지만 재확인)
for h in npuforge-k npuforge-q npuforge-j; do
  ssh $h 'printf "%s %s\n" "$(hostname)" "$(grep -oE "androidboot.fwver=[^ ]*" /proc/cmdline)"'
done

# 3. 안정성 재검증 — 이전에 리셋되던 5~8 스레드 구간
ssh npuforge-k 'cd ~/npuforge-rknn-test && ./thread_safety_test yolov8n-fp16.rknn 20 5 8'
```

3번이 통과하면 `worker_count`를 세 노드에 동일하게 설정할 수 있고, "동일한 3대" 전제가 회복된다.

## 2.17.2 원인 확정: 전원 어댑터 전류 부족 (해결됨, 2026-08-10)

### 결정적 증거: 입력 전압 실측

보드에 입력 전압 센서가 있다는 것을 뒤늦게 발견했다.

```bash
cat /sys/class/power_supply/simple-vin/voltage_now
```

| 시점 | 유휴 전압 |
|---|---|
| **교체 전 어댑터** | **4.983 V** ← 무부하에서도 이미 5V 미만 |
| **5V 4A 어댑터** | **5.26 ~ 5.31 V** |

교체 전 어댑터는 **부하가 없는 상태에서도 5V를 유지하지 못했다.** 고부하에서 더 떨어져 보드의 브라운아웃 임계를 넘은 것이 재부팅의 원인이다.

새 어댑터의 부하 중 전압(`king`, 8스레드까지 984샘플):

```text
최소 5.061 V   평균 5.260 V   최대 5.341 V   변동폭 0.280 V
```

부하가 걸려도 5V 아래로 내려가지 않는다.

### 검증 결과: 3대 모두 8스레드 완주

| 노드 | 8스레드 처리량 | 오류 | 재부팅 |
|---|---:|---:|---|
| `king` | 77.3 inf/s | 0 | **없음** |
| `queen` | 70.2 inf/s | 0 | **없음** |
| `jack` | 78.0 inf/s | 0 | **없음** |

`king`은 4스레드도 통과했다(54.1 inf/s). 이전에는 3스레드에서도 재부팅했다.

### ⚠️ 전압을 12V로 오판했던 기록

커널 로그의 `vcc12v_dcin: 12000 mV`를 실제 입력 전압으로 단정하고 문서에 "12V DC 입력"으로 기록했다. **틀렸다.**

이 이름은 디바이스 트리의 fixed-regulator 선언이며, Rockchip 디바이스 트리가 보드 간 복사되면서 남은 것이다. 실제 입력은 5V다.

**확인했어야 할 것은 선언이 아니라 실측값이었다.**

```text
선언 (디바이스 트리)  vcc12v_dcin: 12000 mV     ← 신뢰 불가
실측 (센서)           simple-vin: 4983000 µV    ← 이것이 사실
```

사용자가 5V 4A 어댑터로 교체하겠다고 했을 때 "5V는 위험하다"고 경고할 뻔했다. 실측을 먼저 확인해 오류를 막았다.

### 진단 과정에서 틀렸던 가설들

| # | 가설 | 결과 | 반증 근거 |
|---|---|---|---|
| 1 | 공용 3포트 전원이 원인 | **틀림** | 공용 전원에서 `queen`은 8스레드 완주 |
| 2 | 부트로더 펌웨어 구버전 | **틀림** | `king` 재설치로 펌웨어 일치시켜도 재부팅. `jack`은 처음부터 같은 펌웨어인데 실패 |
| 3 | 입력 전압이 12V | **틀림** | 실측 4.983V |
| 4 | **어댑터 전류 부족** | **맞음** | 유휴 4.983V → 5.3V 교체 후 3대 모두 8스레드 완주 |

**가설 1이 원인을 절반 맞혔는데도 반증으로 처리했다.** "공용이냐 개별이냐"가 아니라 "용량이 충분한가"가 문제였는데, 구성 방식에 집중하느라 용량을 놓쳤다. 개별 전원으로 교체했을 때 오히려 악화된 것이 그 증거였는데(새 어댑터가 더 약했다), 그때도 전류 용량으로 돌아가지 않고 펌웨어로 방향을 틀었다.

`queen`이 공용 전원에서 8스레드를 완주했다는 사실은 "그 어댑터가 충분했다"는 뜻이지 "전원이 원인이 아니다"라는 뜻이 아니었다.

### 교훈: 실측 센서를 먼저 찾는다

`/sys/class/power_supply/`는 처음 수집한 `collect-node-info.sh`에 포함되지 않았다. 전원을 의심하기 시작한 시점에 이 센서를 찾았다면 **가설 2와 3을 거치지 않고 바로 확정할 수 있었다.**

`collect-node-info.sh`에 입력 전압 항목을 추가했다.

### 지속 부하 검증 (3대 동시, 8스레드)

순간 부하 통과가 지속 부하 통과를 보장하지 않으므로 별도로 확인했다.

**전압 — 문제 없다.**

| 노드 | 최소 전압 |
|---|---|
| `king` | 5.061 V |
| `queen` | 5.157 V |
| `jack` | 5.124 V |

3대를 동시에 최대 부하로 돌려도 5V 아래로 내려가지 않는다. 재부팅도 없다. **전원 문제는 해결되었다.**

**온도 — 새로운 문제가 드러났다.**

| 노드 | 최고 SoC | 최고 NPU |
|---|---:|---:|
| **`king`** | **88.7 °C** | **91.3 °C** ⚠️ |
| `queen` | 70.2 °C | 70.2 °C |
| `jack` | 71.2 °C | 72.1 °C |

`king`이 다른 두 대보다 **약 19°C 높다.** 그리고 **`disable_temperature_c`(90°C)를 초과했다.**

세 보드가 동일 모델·동일 펌웨어·동일 부하이므로 소프트웨어 원인은 배제된다. 후보는 다음과 같다.

- 물리적 배치 차이 (공기 흐름, 벽면 근접, 보드 간 간격)
- 방열 접촉 상태
- 개체 편차

`king`은 다른 두 대보다 약 6분 먼저 부하가 시작되었으나, `queen`/`jack`도 이미 평탄역(70~72°C)에 들어갔으므로 시간 차만으로 19°C를 설명할 수 없다.

**§2.19에서 별도로 다룬다.**

### 남은 확인 사항

- 8스레드에서도 처리량이 꺾이지 않았으므로 `MAX_THREADS`를 늘려 최적점을 다시 찾는다
- 전류 측정 수단이 없다. `voltage_now`만 있고 `current_now`가 없어 소비전력을 계산할 수 없다. FPS/Watt 지표에는 외부 전력계가 필요하다

## 2.19 `king`의 온도가 19°C 높다 (재현되지 않음, 2026-08-11)

지속 부하 시험에서 발견했다. 동일 조건인데 `king`만 NPU 91.3°C에 도달해 스케줄링 제외 임계치를 넘었다.

### 왜 중요한가

**노드별 온도 편차는 확장 효율 측정을 직접 오염시킨다.**

- `king`이 먼저 throttling에 들어가면 처리량이 떨어진다
- 스케줄러는 이를 "느린 노드"로 인식해 부하를 줄인다
- 결과적으로 3노드 확장 효율이 낮게 측정되는데, **원인이 스케줄링이 아니라 물리적 배치**다
- 90°C를 넘으면 스케줄링에서 아예 제외되어 사실상 2노드 실험이 된다

`02-HARDWARE-SETUP.md` §9.1이 "동일한 주변 온도, 동일한 배치 방향, 보드 사이 최소 10cm"를 요구하는 이유가 이것이다.

### 확인해야 할 것

| 항목 | 방법 |
|---|---|
| 물리적 배치 | 세 보드의 간격, 방향, 주변 장애물 확인 |
| 적층 여부 | 겹쳐 놓았다면 분리 |
| 공기 흐름 | 벽면·구석·케이블 뭉치에 막혔는지 |
| 주변 온도 | 각 보드 위치의 실제 온도 (햇빛, 다른 장비 발열) |
| 방열판 접촉 | 케이스 장착 상태 |

배치를 균일하게 맞춘 뒤 동일 시험을 반복해 편차가 사라지는지 확인한다. 그래도 남으면 개체 편차이므로 결과에 명시한다.

### 유휴 온도에는 편차가 없다 (2026-08-11 확인)

부하 종료 19.9시간 뒤 세 보드를 동시에 측정했다.

| 보드 | NPU (유휴) | SoC | load1 | 부하 시 NPU (2026-08-10) |
|---|---|---|---|---|
| `king` | 39.8°C | 40.7°C | 1.34 | 91.3°C |
| `queen` | 36.1°C | 36.1°C | 0.07 | 70.2°C |
| `jack` | 37.0°C | 38.8°C | 0.23 | 72.1°C |

**유휴 편차는 2.8~3.7°C에 불과하다.** 그나마도 측정 시점에 `king`에서
`gnome-control-center` 세션이 돌고 있었고(load 1.34), 나머지 두 대는 사실상
유휴였다. 즉 유휴 상태에서 세 보드는 사실상 같다.

이것이 뜻하는 바:

- 19°C는 **지속 부하에서만 벌어지는 격차**다. 방열 능력 차이(공기 흐름)로
  설명하기에 부합한다 — 유휴 발열량에서는 차이가 드러나지 않고, 발열량이
  커질수록 방열 조건 차이가 온도 차로 증폭된다
- 개체 불량(예: 방열판 접촉 불량)이었다면 유휴에서도 어느 정도 드러났을
  가능성이 높다. 완전히 배제할 수는 없으나 배치 가설이 더 유력하다
- **따라서 재측정은 반드시 부하 조건에서 해야 한다.** 유휴 온도만 보고
  "해결됐다"고 판단하면 안 된다

세 보드 모두 `graphical.target` + `gdm` active로 구성이 동일함도 함께 확인했다.
데스크톱 세션이 한 대에만 떠 있으면 그 자체가 측정 오염원이므로, 벤치마크
직전에 세션 상태를 맞춘다(`preflight-check.sh` 항목).

### 통제된 재측정: 19°C 격차는 재현되지 않는다 (2026-08-11)

전용 부하 도구(`sustained_load_test`)로 세 보드에 **동시에** 8스레드 부하를
15분간 걸었다. 평탄역(부하 후 300초~종료, 보드당 약 557샘플) 요약이다.

| 보드 | NPU 평균 | NPU 최고 | SoC 평균 | 입력전압 최저 | 처리량 |
|---|---|---|---|---|---|
| `king` | 73.0°C | **75.8°C** | 71.2°C | 5.070 V | **80.5 inf/s** |
| `queen` | 67.5°C | 70.2°C | 65.8°C | 5.090 V | 77.7 inf/s |
| `jack` | 72.6°C | 74.8°C | 71.6°C | 5.046 V | 77.8 inf/s |

**최대 편차 5.6°C. 90°C 초과 없음. NPU 클럭 강하 없음**
(928 샘플 전부 950 MHz, 하나도 떨어지지 않았다).

상승 곡선도 세 보드가 나란하다.

```text
 t(s)   king  queen   jack
    0   37.0   35.2   37.0
   60   66.5   61.9   66.5
  120   72.1   65.6   69.3
  300   73.0   67.5   73.0
  600   73.9   67.5   73.0
  880   74.8   68.4   72.1
```

### 이전 측정과 무엇이 달랐나

08-10 측정(`king` 91.3 / `queen` 70.2 / `jack` 72.1)과 직접 비교할 수 없다.
**부하 프로파일이 달랐다.**

| | 2026-08-10 | 2026-08-11 |
|---|---|---|
| 도구 | `thread_safety_test` | `sustained_load_test` |
| 부하 형태 | 1→8 스레드 순차 스윕 | 8스레드 고정 |
| 시작 시각 | `king`이 약 6분 선행 | 동시 |
| 지속 | 스윕 완료까지 | 900초 고정 |

`thread_safety_test` 는 목표 스레드 수에 도달하기 전에 단일/2스레드 기준선을
먼저 돌린다. 즉 `king` 은 다른 두 대가 8스레드에 들어갈 무렵 이미 훨씬 오래
가열된 상태였다. 6분 선행까지 겹치면 격차가 부풀려질 조건이 갖춰진다.

`queen` 의 최고 온도는 두 측정에서 **70.2°C 로 동일**하고 `jack` 은
72.1 → 74.8°C 로 소폭 올랐다. 움직인 것은 `king` 뿐이다(91.3 → 75.8°C).
배치를 바꾸지 않았다는 점을 감안하면, 격차의 상당 부분은 **물리적 배치가
아니라 측정 방법의 문제**였을 가능성이 크다.

물론 배치 요인을 완전히 배제할 수는 없다. 다만 지금 조건에서는

- 어떤 보드도 `degraded_temperature_c`(80°C)에 닿지 않는다
- 어떤 보드도 throttling 되지 않는다
- 처리량 편차가 3.5% 이내다 (80.5 / 77.7 / 77.8 inf/s)

이므로 **벤치마크를 막는 요인이 아니다.** S0 실험을 진행할 수 있다.

`king` 이 가장 뜨거우면서 동시에 가장 빠르다는 점도 일관된다 — 15분간
72,481회로 `queen`(69,928회)보다 3.6% 더 많은 일을 했다. 다만 3.6%의 일량
차이가 5.5°C를 다 설명하지는 못하므로, 작은 방열 조건 차이는 남아 있다고
본다.

### 여기서 얻은 측정 원칙

**부하 프로파일이 다르면 온도를 비교하지 않는다.** 같은 "고부하"라도
도달 경로가 다르면 축적 열량이 다르다. S0 이후 모든 열 비교는
`scripts/run-thermal-comparison.sh` 로 수행한다. 이 스크립트는

- 별칭↔hostname 일치를 먼저 검증하고 (§2.20)
- 세 보드의 바이너리/모델 해시가 같은지 확인하고
- 유휴 기준선을 먼저 재고
- 세 보드에 **동시에** 부하를 걸고
- run 전후 `boot_id` 를 비교해 도중 리셋된 보드를 무효 처리한다

### 임계치 재검토가 필요하다

현재 설정값은 초안 그대로다.

```text
degraded_temperature_c = 80.0
disable_temperature_c  = 90.0
```

팬리스 보드가 정상 동작 중에 70~91°C에 도달한다면 이 값들은 **보호가 아니라 측정 방해**가 된다. S0 결과로 재설정한다(`02-HARDWARE-SETUP.md` §9.2).

RK3576의 실제 임계 온도(Tj max)를 확인해 그보다 충분히 낮게, 그러나 정상 동작 범위보다는 높게 잡아야 한다.

## 2.20 문서에 적힌 `king`의 IP가 틀렸다 (2026-08-11)

`king`을 `192.168.123.22`로 기록해 두었으나 **실제 주소는 `192.168.123.12`** 다.
`.22`는 서브넷 전체 스윕에서 ARP 응답조차 없는 빈 주소였고, 그 결과
"`king`이 죽었다"는 잘못된 결론을 냈다.

### 왜 놓쳤나

`~/.ssh/config` 의 `npuforge-k` 별칭에는 **처음부터 `.12`가 올바르게** 들어
있었다. 틀린 것은 문서와 스크립트에 하드코딩한 IP뿐이다. 별칭을 썼다면
애초에 드러나지 않았을 문제다.

| 위치 | 값 | 상태 |
|---|---|---|
| `~/.ssh/config` `npuforge-k` | `.12` | 정상 |
| `board-worklog.md` §1 표 | `.22` | **오류** |
| `environment-matrix.md` §7 | `.22` | **오류** |
| `infrastructure.md` | `.22` | **오류** |
| `setup-node.sh` 사용 예 | `.22` | **오류** |
| `fix-node-consistency.sh` IP 목록 | `.22` | **오류** |

모두 `.12`로 정정했다.

### 재발 방지

**보드 접속은 IP가 아니라 별칭(`npuforge-k/q/j`)으로 한다.** IP는 DHCP라
바뀔 수 있고, 문서에 박아 두면 반드시 한 곳이 낡는다. 별칭은 한 곳
(`~/.ssh/config`)만 고치면 된다.

`preflight-check.sh` 에 다음을 넣는다.

- 세 별칭이 모두 접속되는가
- 각 별칭이 붙은 호스트의 `hostname` 이 `king/queen/jack` 과 일치하는가

이름이 어긋난 채로 벤치마크를 돌리면 결과가 엉뚱한 노드에 귀속된다.
이번처럼 "노드가 죽었다"로 끝나면 차라리 낫고, 조용히 다른 보드에
붙는 쪽이 훨씬 위험하다.

### 참고: 보드 MAC 은 OUI 가 없다

세 보드 모두 locally administered MAC 을 쓴다(`82:`, `66:`, `26:` — 두 번째
니블이 2/6/A/E). 제조사 OUI 로 보드를 식별할 수 없다는 뜻이라, 네트워크
스캔으로 보드를 찾는 방법은 통하지 않는다.

다만 `addr_assign_type = 0`(permanent) 이므로 **재부팅해도 MAC 은 유지된다.**
DHCP 리스가 흔들릴 이유는 없다. 그래도 IP 고정(정적 할당 또는 DHCP
예약)을 해 두는 편이 안전하다.

## 2.21 원격 백그라운드 실행의 두 가지 함정 (2026-08-11)

`preflight-check.sh` 를 만들면서 검사가 **조용히 작동하지 않는** 것을
발견했다. 부하가 도는데 "남은 부하 없음"으로 통과했다.

두 가지가 겹쳐 있었다.

### 함정 1: `pgrep -f` 는 자기 자신을 센다

`pgrep -f` 는 명령줄 전체를 매칭한다. ssh 가 보내는 래퍼는

```text
bash -c "... pgrep -f \"[s]ustained_load_test|...\" | wc -l"
```

이고, 이 명령줄에 패턴 문자열이 들어 있다. 대괄호 트릭
(`[s]ustained`)은 같은 명령줄에 괄호 없는 형태가 섞이면 무력해진다.

**양방향으로 틀렸다.**

| 상황 | 실제 | pgrep 보고 |
|---|---|---|
| 부하 실행 중 | 1개 | 0 (놓침) |
| 부하 없음 | 0개 | 2 (자기 셸을 셈) |

`/proc/PID/exe` 심볼릭 링크를 읽는 방식으로 바꿨다. 이것은 실제 실행
파일을 가리키므로 셸이 끼어들 여지가 없다.

```bash
n=0
for p in /proc/[0-9]*; do
  case "$(readlink "$p/exe" 2>/dev/null)" in
    *sustained_load_test) n=$((n+1)) ;;
  esac
done
```

### 함정 2: `cd DIR && setsid nohup ... &` 는 뜨지 않는다

같은 조건에서 두 형태를 비교했다.

| 형태 | 결과 |
|---|---|
| `ssh -n H "cd $DIR && setsid nohup ./prog ... &"` | **실행 안 됨** |
| `ssh -n H "setsid nohup $DIR/prog ... &"` | 실행됨 |

`&` 는 `cd && prog` 리스트 전체에 걸린다. ssh 가 명령을 보내고 즉시
끊는데, 백그라운드 서브셸이 `cd` 를 거쳐 `setsid` 에 닿기 전에 세션이
사라지면 죽는다. 절대경로를 쓰면 중간 단계가 없어 경합이 생기지 않는다.

**실패해도 아무 신호가 없다.** 종료 코드는 0 이고 stderr 도 비어 있다.
확인하지 않으면 "부하 없는 상태의 온도"를 15분 동안 측정하게 된다.

`run-thermal-comparison.sh` 는 원래 절대경로 형태를 쓰고 있어서
2026-08-11 열 측정은 영향을 받지 않았다. 다만 **띄운 뒤 실제로 도는지
확인하는 단계**를 추가했다.

### 공통 교훈

두 함정 모두 **실패가 성공처럼 보인다.** discuss.md §10 의 A 유형
(지표가 무엇을 세는지 확인하지 않음)과 같은 계열이다.

검사를 새로 만들면 **일부러 깨뜨려 보고 실제로 잡히는지 확인한다.**
이번에도 그 절차 덕에 발견했다. 통과만 보고 믿었다면 preflight 는
아무것도 걸러내지 못하는 채로 남았을 것이다.

## 2.29 S3 saturation sweep — ceiling 기준도 near-linear (2026-08-20)

각 노드 수의 진짜 처리량 상한을 concurrency sweep 으로 찾았다(S2 는 동일
부하 선형성, S3 는 최대 처리량 — 별개 실험). 45 run, 동결 `1da69d4`.

| Config | Ceiling @ conc | Speedup | Eff |
|---|---|---:|---:|
| 1N | 115.2 @ c32 | 1.00× | 100% |
| 2N | 232.0 @ c24 | 2.01× | 101% |
| 3N | **341.8 @ c32** | **2.97×** | **99%** |

- 곡선: 미포화(왕복 지연) → plateau(노드당 ~10-16 동시) → 과부하 살짝 하락.
  오류 0(큐가 흡수). SD ≤ 2.2.
- **S2(동일부하)와 S3(ceiling) 두 각도에서 near-linear 재확인.**
- 보고서: `docs/experiments/S3_SATURATION.md`, 원본: `results/saturation-20260820/`.

다음: S4 io_uring — payload-transfer(비-추론 지연의 94%) 비용 절감 비교.

## 2.28 gRPC baseline 30회 반복 — 재현 확인, baseline 동결 (2026-08-20)

1차 결과를 "재현된 결과"로 승격시켰다. 코드·설정 동결(bench `254d560`)
상태에서 1N/2N/3N 각 10회, 60초, **조건 순서 rotate**(시간·온도 변동 분산).
`scripts/run-grpc-baseline30.sh`. 원본·집계: `results/baseline-20260820/`.

### 결과

| N | Throughput Mean±SD | Speedup | Eff | p50/p99 ms | Err | Bal |
|---:|---:|---:|---:|---|---:|---:|
| 1 | 112.9 ± 0.5 | 1.00× | 100% | 68.0 / 116.3 | 0% | 0.00 |
| 2 | 229.0 ± 0.9 | 2.03× | 101% | 67.0 / 118.6 | 0% | 0.00 |
| 3 | **338.4 ± 1.1** | **3.00×** | 100% | 67.6 / 123.9 | 0% | 0.00 |

- **첫 측정 337.7 이 338.4 ± 1.1 로 재현.** SD 0.5~1.1 로 극히 작다.
- 30/30 active node 정확, invalid 0, 오류 0%, balance 0%p.
- saturation(115) 기준 3N efficiency 98%, 1N c8 기준 speedup 3.00×.

### TimingBreakdown 도 재현 (30회 p50 평균)

3N: network_to_node 17.11 + network_to_client 17.11 = 34.21 ms
  = non-inference overhead(36.34) 의 **94%**, E2E(58.83) 의 58%.
scheduler_queue/route 는 1N·3N 모두 ~0 — 스케줄러 병목 없음 재확인.
1N·3N 의 network 가 거의 같아(17.7 vs 17.1) 전송 시간은 노드 수 무관.

### 승격된 문장

"한 번 337.7" → **"3-node near-linear scaling 을 30회 반복 실험으로 확인
(338.4 ± 1.1 inf/s, speedup 3.00×, error 0%)."** gRPC baseline 동결.

다음: saturation sweep → (동결 유지) → io_uring 동일 조건 비교.

## 2.27 로컬 팬 baseline 재측정 — 오버헤드 27% → 28.8% 확정 (2026-08-20)

27% 의 기준값 157 이 팬리스(08-11/12)라 냉각 조건이 클러스터(팬)와 달랐다.
같은 팬 조건에서 로컬 sustained 를 다시 쟀다. king 노드를 중지하고 순수
로컬 `sustained_load_test`(gRPC 없음), INT8, governor=performance, 팬 ON.

```text
8스레드(worker 8, 클러스터 동일조건) 60초 × 3:  159.2 / 162.0 / 163.2 → 161.5
16스레드(saturation 확인):                       165.7
```

**확정: 오버헤드 = (161.5 - 115) / 161.5 = 28.8%** (냉각·worker·측정시간 통일).

### 발견 — 27% 는 냉각으로 무너지지 않았다

우려는 "팬이면 로컬이 157 보다 훨씬 높아 오버헤드가 크게 벌어진다"였다.
실제로는 팬 161.5 vs 팬리스 157.2 로 **차이가 작았다.** 이유:

**60초/30초 측정은 throttling 발현 전이다.** CPU throttling 은 300초에
-27% 로 나타난다(§2.24, discuss §12). 짧은 측정 구간에서는 팬이든 팬리스든
초기 처리량이 비슷하므로 냉각 조건의 영향이 작다.

→ **27% 는 냉각 때문에 무효가 아니라 28.8% 로 소폭 조정**됐다. 병목 위치
(페이로드 전송, §8·§2.26)는 애초에 냉각과 무관해 그대로다. **가장 단단한
사실 두 개는 흔들리지 않았다**: (1) 확장 효율 ~98% 선형, (2) non-inference
latency 의 94% 가 페이로드 전송.

**두 측정량을 곱하지 않는다.** throughput loss 28.8%(처리량)와 latency
breakdown 94%(지연 구성비)는 다른 축이다. "28.8% 의 94%" 는 틀린 곱이다.
정확한 표현: 클러스터 단일노드 처리량은 로컬 대비 28.8% 낮았고, 별도
latency breakdown 에서 non-inference latency 의 94% 가 payload-transfer 였다.

### 남은 것 (별도 조건)

- **지속 부하(300초) 오버헤드**: 팬 이득이 커지면 오버헤드가 더 벌어질 수
  있다. throttling 이 로컬(sustained)과 클러스터(노드)에 어떻게 다르게
  걸리는지가 다음 질문. 단, 이건 "짧은 측정 28.8%"와 별도 축이다.
- saturation: 16스레드(165.7) > 8스레드(161.5) 라 worker 8 이 로컬 최대는
  아니다. 클러스터 노드가 worker 8 이라 동일조건 비교는 8스레드가 맞다.

## 2.26 TimingBreakdown 첫 실측 — 오버헤드는 페이로드 전송 (2026-08-20)

bench 를 확장해 응답의 `Timing`(proto) 11단계를 전부 수집하게 했다(기존엔
`inference_us` 하나만). 27% 노드당 오버헤드를 단계로 쪼갠 첫 실측이다.

측정: 3노드 / c24 / 10초 / Active Cooling / gRPC.

```text
단계 (p50 ms)
  scheduler_queue      0.00
  scheduler_route      0.00
  network_to_node     17.16   ┐ 페이로드 전송
  node_queue           0.02   │
  decode/preprocess    0.00   │
  npu_input            0.00   │
  inference (NPU)     22.49   │ ← 실제 추론
  postprocess          0.00   │
  network_to_client   17.16   ┘
  end_to_end          58.99
```

**발견: 노드당 오버헤드의 정체는 페이로드 네트워크 전송이다.**
payload transfer = `network_to_node + network_to_client` = 34.32 ms.
protobuf 직렬화도, 스케줄러 큐(~0)도, 노드 큐(~0)도 아니다. 1.17 MiB
입력·출력을 2.5G 로 실어 나르는 시간이 대부분이다.

**분모를 명확히 구분한다(혼동 방지):**

```text
payload transfer / E2E latency            = 34.32 / 58.99 = 58%
payload transfer / non-inference overhead = 34.32 / 36.50 = 94%
  (non-inference overhead = E2E - inference = 58.99 - 22.49 = 36.50 ms)
```

정확한 표현: **"노드당 오버헤드(=E2E−inference)의 94%가 페이로드 전송"**,
그리고 "E2E 지연의 58%가 페이로드 전송, 38%가 순수 추론".

→ io_uring·zero-copy·JPEG 입력·후처리(NMS)로 응답 축소 가 겨냥할 지점이
**네트워크 전송 경로**임이 실측으로 확정됐다.

### 계측의 한계 (정직하게)

- gRPC **직렬화 시간은 단독 분리 안 됨** — proto `Timing` 에 별도 필드가
  없다. 재려면 계측점 추가 필요. 현재 잔차(~2ms)에 섞여 있다.
- bench↔스케줄러는 **같은 호스트(loopback)** 라 client→scheduler 는 ~0.
  실제 네트워크는 스케줄러↔노드 2.5G 구간뿐이다.
- **냉각 조건 미확정:** 이 분해는 클러스터 내부라 냉각 무관하게 유효하지만,
  "27%" 자체는 팬리스 157 vs 팬 클러스터 115 라 아직 확정 아님(§2.24).
- c24(동시 24) 값이라 `network_*` 는 concurrency 의존. 단일 요청 전송 시간은
  낮은 concurrency 에서 따로 봐야 한다.

작업용 집계표는 `results/NPUForge_Benchmark_Result_Workbook.md` §8 (로컬 전용).

## 2.25 S2 확장성 첫 측정 — 확장 효율 98%, 노드당 오버헤드 발견 (2026-08-20)

model_file 버그 수정 후 preflight 통과 상태에서 1/2/3노드 확장성을 처음
쟀다. **정식 근접(preflight 통과·30초·조건 통제)이나 단일 run·
--with-inference 스킵이라 확정 수치 아님.**

측정: INT8, want_float=0, governor=performance, **Active Cooling(노드마다
전용 팬, 측정 시작부터)**, 스케줄러(.9) 경유 gRPC, round-robin. 노드 축소는
프로세스 중지(jack→queen 순), 사이 cooldown.

> ⚠️ **냉각 조건 정정 (2026-08-20 사후).** 이 세션의 모든 측정은 팬 장착
> 상태였다. 처음엔 "cold/팬리스"로 적었으나 실제로는 시작부터 큰 팬이
> 달려 있었다. 이것이 27% 계산에 영향을 준다 — 아래 결론 참조.

### 노드당 동일 부하 (concurrency = 8 × 노드수)

| 구성 | 처리량 | 분배 |
|---|---:|---|
| 1노드 c8  | 111.6 inf/s | king 100% |
| 2노드 c16 | 228.7 inf/s | 50/50 |
| 3노드 c24 | 337.7 inf/s | 33/33/33 |

오류율 0%, round-robin 이 정확히 균등 분배. 3노드/1노드 = **3.03배**.

### 1노드 concurrency 스윕 — 상한 ~115

| c8 | c16 | c32 |
|---:|---:|---:|
| 111.6 | 114.0 | 115.1 |

concurrency 를 올려도 **~115 inf/s 에서 포화**. 이것이 스케줄러 경유
단일 노드 상한이다.

### 두 가지 결론

**1. 확장 효율 ~98% (거의 선형).** 1노드 포화 115 기준 3노드 337.7 =
2.93배. 데이터 병렬(`adrs/001`)이 성립하고 스케줄러가 3노드 동시에도
병목이 아니다. `adrs/003` 의 단일 스케줄러가 이 규모에서 충분함을 실측.

**2. 클러스터 노드 상한 115 < 로컬 sustained 157 (-27%).** 왕복 p50 69ms
인데 노드 보고 추론은 24~28ms — **40ms+ 가 스케줄러 gRPC 경유 오버헤드**
(직렬화 + 1.17MB 입력·출력 전송 + 큐/라우팅). 확장은 선형인데 노드당
절대 상한이 네트워크·스케줄링에 깎인다.

> 프로젝트 핵심 질문 "6 TOPS 세 대는 정말 18 TOPS 가 되는가" 의 첫 실측 답:
> **클러스터 기준 2.93배(98%).** 병목은 확장이 아니라 노드당 오버헤드다.
> 이 27% 가 어디서 오는지는 `TimingBreakdown` 단계 분해로 다음에 쪼갠다.

### 마이너 이슈

- bench run 파일명이 전부 `-n3` — run_id 의 노드 수가 측정 시점 활성이
  아니라 **초기 ListNodes(등록) 기준**이다. jack/queen 중지 후에도 스케줄러
  등록이 남아 3으로 찍혔다. 실제 노드 수는 결과의 분배로만 확정된다.
  run_id 를 측정 종료 시점 활성 노드로 잡는 것이 옳다.
- 노드 축소를 프로세스 kill 로 했다. drain RPC 가 있으면 진행 중 요청을
  흘려보내고 깨끗이 뺄 수 있다(`adrs/027`). S2 정식에서 검토.

### 정식 S2 에 남은 것

반복 run(분산), 팬 조건(S0-B), --with-inference, concurrency 스윕 전체,
2노드 조합(king+queen vs king+jack), TimingBreakdown 오버헤드 분해.

## 2.24 M3 첫 3노드 클러스터 실동작 (2026-08-20)

인프라·빌드·IP 고정이 끝나 실제 3노드 추론 클러스터를 처음 띄웠다.
스케줄러(server .9) + king/queen/jack, 실 gRPC.

### 배포

- king 에서 노드 빌드(`cargo build --release -p npuforge-node --features rknn`,
  1m37s, 24MB) → 개발 PC 경유로 queen/jack 배포
- 모델: INT8 `model.rknn`(dba155d2) + `model.toml` 3보드, 해시 검증 통과
- 스케줄러: server 에서 `scheduler.example.toml`(policy round-robin), 50051

### 예비 벤치 (정식 아님)

preflight 미실행, Active Cooling(팬 ON), 12초. **조건 통제 전이라 확정
수치로 쓰지 않는다.**

| 동시성 | 처리량 | 노드 추론 p50 | 왕복 p50 | 분배 |
|---:|---:|---:|---:|---|
| 6  | 146.3 inf/s | 14.4 ms | 39.8 ms | 33.3% 균등 |
| 24 | 336.4 inf/s | 22.2 ms | 67.7 ms | 33.3% 균등 |

오류율 0%, round-robin 이 세 노드를 정확히 3등분. 단일 노드 INT8 상한
157 대비 c24 에서 약 2.1배 — **다중 노드 확장이 실제로 일어난다.** 정식
S2 는 preflight + concurrency 스윕 + 지속시간으로 별도.

### 이번에 걸린 버그 3개 (전부 "성공처럼 안 보이는" 실패라 빨리 잡힘)

**1. model.toml `model_file` 상대경로가 로딩 실패로 이어진다 (코드 버그, 미수정)**

`main.rs` 는 `load_spec` 이 만든 절대경로 `PathBuf` 로 sha256 을 검증하는데
(`:77`), 정작 `backend.load_model(&spec)`(`:81`)에는 `spec.model_file`(원본
상대경로 `"model.rknn"`)을 넘긴다. 백엔드가 CWD 기준으로 파일을 찾아
`rknn_init` 전에 read 실패 → `status=-2`(read_file 실패도 rknn_init 실패도
같은 NPF_RKNN_ERR_MODEL_LOAD 라 구분 안 됨). RKNN 은 stderr 를 안 남긴다.
→ **수정 완료 (2026-08-20).** `main.rs` 가 `load_model` 직전에
`spec.model_file` 을 `load_spec` 이 해석한 절대경로로 교체한다. 상대경로
`model.toml` 로 3노드가 정상 로딩·등록되고 벤치 재검증(c24 336 inf/s, 오류
0%)까지 통과했다. real_device 테스트는 spec.model_file 에 절대경로를 직접
넣어서 이 버그를 못 잡았다 — 상대경로 케이스 회귀 테스트가 없다.

**2. 죽은 노드가 NPU 컨텍스트를 안 놓아 재기동이 status=-2 로 실패**

노드를 죽였다 바로 다시 띄우면 rknn_init 이 실패한다. `pkill -9` +
수 초 대기로 확실히 정리해야 뜬다. 노드의 graceful shutdown(ContextPool
drop → rknn_destroy)이 SIGTERM 에서 확실히 도는지 점검 필요.

**3. `pkill -f npuforge-node` 가 자기 셸을 죽였다 — ADR-017 함정 1 재현**

정리 명령의 셸 명령줄에 패턴 문자열이 들어 있어 pkill 이 자신을 죽이고
이후 명령이 조용히 안 돌았다. **배포/정리는 `pkill`(comm, `-f` 없이)로.**
내가 문서에 적어 둔 함정에 그대로 걸렸다.

현재 3노드 + 스케줄러는 실행 유지 중. server 방화벽 50051/8080/9090 은
런타임 규칙(재부팅 시 사라짐).

## 2.23 네트워크 개편 — 10G aggregation 구축·실측 (2026-08-20)

§2.22 에서 대기하던 장비가 들어와 M3 네트워크를 구성했다. **차단 요소가
전부 해소됐다.**

### 도입한 것

| 장비 | 사양 |
|---|---|
| 스위치 | **NEXI NS-S25G10G-N** — 2.5G×4 + 10G×2, 전부 RJ45 |
| 서버 | Xeon E5-2630L ×2 (24T) / 16GB / Rocky 9.4 / x86_64 |
| 서버 NIC | `enp4s0` 10GBASE-T (DAC/SFP+ 아님) |

포트 배선: 1=인터넷(ipTIME), 2=king, 3=queen, 4=jack, 5=개발PC(10G포트지만
NIC 1G), 6=server(10G).

### 겪은 것

1. **보드 IP 가 통째로 바뀌었다.** DHCP 라 `.12/.16/.33` → `.3/.4/.5` 로
   재할당됐고, `~/.ssh/config` 의 별칭이 낡아 세 노드 전부 접속 실패했다.
   `adrs/019-ssh-alias-not-ip.md` 가 경고한 상황 그대로다. config 갱신 +
   `npuforge-server` 별칭 추가로 복구.

2. **서버가 10G IP 를 못 받았다.** 원인은 케이블·스위치가 아니라
   NetworkManager 였다 — `enp4s0` 이 `UP LOWER_UP`(링크 붙음)인데 연결
   프로파일이 없어 DHCP 를 안 돌렸다. `nmcli device connect enp4s0` 로
   즉시 `192.168.123.9` 획득. Rocky 9 에 새 NIC 꽂으면 나오는 전형적 상황.

3. **원격 iperf3 기동이 안 떴다** — `setsid nohup iperf3 ... &` 가 조용히
   실패(`adrs/017` 함정 2). 절대경로 형태로 재기동해 해결.

### 실측

```text
server enp4s0        10000 Mb/s full     ethtool
단일 king→server     2.34 Gbps           iperf3   (2.5G 실효 상한)
3노드 동시 →server   각 1.70, 합 5.11 Gbps  nc     (세 스트림 균등 유지)
```

세 스트림이 균등 유지 → **서버 10G aggregation 이 병목이 아니다.** INT8
3노드 목표 RX 4.60 Gbps 를 여유 있게 수용. 상세 판단은
`adrs/014-10g-aggregation-separate-scheduler.md` 구축 결과 절.

### 정리한 것

측정용 방화벽 런타임 규칙(5201-5210)·임시 리스너·파일은 측정 후 전부
제거했다. 서버 영구 상태는 바꾸지 않았다.

### 남은 것

- **IP static 고정** — server(.9) 완료. 보드 3대는 pi sudo 비번 대기.
  라우터 예약 대신 호스트 static 채택 (`infrastructure.md` §2.3)
- INT8 모델 queen·jack 배포
- server gRPC 방화벽 개방

`dealer`(옛 스케줄러, 노트북 .14)는 응답 없음 — 제거됐다. 역할은 server 로 이관.

### IP 고정 방식 결정 (2026-08-20)

라우터(ipTIME) DHCP 예약이 아니라 **호스트 NetworkManager static** 을 택했다.
라우터가 바뀌어도 설정이 호스트에 남아 측정 재현성이 낫고, 현재 IP 를 그대로
고정하므로 SSH 도 안 끊긴다. server 는 root 라 즉시 적용했고
(`nmcli con mod enp4s0 ipv4.method manual ...`), 보드는 `pi` 계정의 sudo 비번이
있어야 한다. 남는 리스크(DHCP 풀 충돌)는 `infrastructure.md` §2.3.

### 스케줄러 빌드 경로 결정 (2026-08-20)

옛 dealer 는 Rust 가 없어 미정이었다. server 로 확정한다.

- toolchain `stable`, MSRV 1.85. server dnf 의 rust/cargo **1.92** 로 충분
- Windows→Linux 크로스빌드는 링커 문제로 회피. **server 24스레드 네이티브**가
  빠르고 확실하다
- server 에 `rust cargo gcc gcc-c++ protobuf-compiler git` 설치
  (tonic-build 0.12 가 protoc 요구). github 접근 OK, foxden 직접 불가라
  소스는 `git archive` tarball 을 scp 로 전송
- 노드(aarch64)는 종전대로 king 네이티브 빌드. 스케줄러(x86_64)만 server

**함정: protoc 가 Rocky 9 기본 리포에 없다.** `dnf install protobuf-compiler`
가 "No match" 로 실패하고, `dnf install -y a b c ...` 는 하나만 못 찾아도
전체가 실패해 rust 까지 설치 안 됐다. **CRB 리포**를 켜야
(`dnf config-manager --set-enabled crb`) protobuf-compiler 가 잡힌다.

**빌드 검증 완료 (2026-08-20).** `cargo build --release -p npuforge-scheduler
-p npuforge-bench` 성공.

```text
cargo 1.92.0 / rustc 1.92.0 / libprotoc 3.14.0 / gcc 11.5.0
npuforge-scheduler  25 MB
npuforge-bench      19 MB
config 파싱·기동 정상 (--config configs/scheduler.example.toml)
```

스케줄러 빌드 경로의 불확실성이 사라졌다. 실제 배포·기동은 M3 착수 때.

## 2.22 작업 중단 시점 상태 (2026-08-12, 10G 스케줄러 구성 대기)

> **후속: §2.23 (2026-08-20) 에서 이 대기가 해소됐다.** 아래는 중단 시점 기록이다.

M3 실장비 측정은 10G aggregation 구성이 있어야 시작할 수 있다. 그때까지 작업을
멈추므로 재개에 필요한 상태를 남긴다.

### 보드 상태

| 항목 | king | queen | jack |
|---|---|---|---|
| SSH 별칭 | `npuforge-k` | `npuforge-q` | `npuforge-j` |
| IP | 192.168.123.12 | .16 | .33 |
| CPU governor | `performance` (영구화) | 동일 | 동일 |
| 유휴 NPU 온도 | 37.9°C | 37.9°C | 38.8°C |
| 잔존 부하 프로세스 | 없음 | 없음 | 없음 |

세 노드의 커널·`librknnrt.so`·RKNPU 드라이버·모델 해시가 모두 일치한다.
`preflight-check.sh --with-inference` 전항목 통과 상태로 멈췄다.

### 보드에 설치한 것 (원래 없던 것)

| 노드 | 추가 | 이유 |
|---|---|---|
| `king` | Rust 툴체인 (rustup) | `npuforge-node --features rknn` 네이티브 빌드. 크로스 컴파일은 aarch64 sysroot 와 RKNN SDK 를 함께 맞춰야 해 실패 지점이 많다 |
| `king` | `protobuf-compiler` | `npuforge-proto` 빌드 |
| 3노드 | `strace` | syscall 분해 측정 |
| 3노드 | `/etc/systemd/system/npuforge-cpu-governor.service` | governor 영구화 |
| 3노드 | `~/npuforge-rknn-test/` C 도구들 | 측정 도구 |

**`king` 에만 Rust 가 있다.** 환경 일치가 깨진 항목이지만 빌드 전용이고
런타임에 영향이 없다. 바이너리는 한 번 빌드해 세 노드에 배포한다
(모델과 같은 원칙).

### 확정된 수치 (governor=performance 기준)

| 항목 | 값 |
|---|---|
| FP16 8스레드 지속 처리량 | **84.3 inf/s** (지연 94.5 ms) |
| INT8 8스레드 지속 처리량 | **157.2 inf/s** (지연 50.8 ms) |
| INT8 / FP16 배율 | **1.86배** |
| 추론당 커널 ioctl | 76회 (FP16·INT8 동일) |
| 노드 간 열 편차 | 5.6°C, **NPU** throttling 없음 |
| CPU thermal 강등 | A72 2208→816MHz / A53 2016→600MHz (부하 60초 후) |
| 지속 부하 NPU 온도 | 67.5~75.8°C (ondemand 기준, 15분) |
| `want_float=0` 효과 | INT8 +17.3% / FP16 +15.7%, 출력 4분의 1 |

이전 문서의 79.0 / 146.2 inf/s 는 `ondemand` 기준이다. discuss.md §11.

### 재개할 때 먼저 할 일

1. `bash scripts/preflight-check.sh --with-inference`
   - 보드가 재부팅했을 수 있다. governor 는 유지되지만 `boot_id` 는 바뀐다
   - 실패하면 그 항목부터 해소한다. 통과 전에 측정하지 않는다
2. 2.5G/10G 스위치 연결 후 추론망 IP 대역을 정하고 `advertise_address` 갱신
   (스케줄러는 10G SFP+ 업링크. `02-HARDWARE-SETUP.md` §3.3.2)
3. `npuforge-node` 를 `king` 에서 빌드해 세 노드에 배포
4. S2 확장성 실험 설계 재검토 — INT8 노드당 **1.545 Gbps**, 3노드
   **4.636 Gbps**. 출력은 입력의 3.96배라 RX 가 최대 18.4 Gbps 다.
   **10G aggregation 이 필요하다.** `02-HARDWARE-SETUP.md` §3.3.2
   (여기 처음 적은 1.43/4.3 은 Gbps 를 2진 접두로 계산한 오류였다)

### 재개 시 주의할 함정 (이번에 겪은 것)

- 보드 접속은 **IP 가 아니라 별칭**으로 한다 (§2.20)
- 원격 백그라운드 실행은 **절대경로**로 하고 실제로 떴는지 확인한다 (§2.21)
- 프로세스 확인은 `pgrep -f` 가 아니라 `/proc/PID/exe` 로 한다 (§2.21)
- ssh 안에서 heredoc + sudo 중첩은 조용히 실패한다. 파일은 `scp` 로 보낸다
- 열 비교는 **부하 프로파일이 같을 때만** 한다 (§2.19)

## 2.18 RTC가 유지되지 않는다

부팅 이력 조회에서 별개 문제를 발견했다.

```text
queen  현재 부팅 시작 시각  Tue 2025-11-25 18:16:31 UTC
jack   현재 부팅 시작 시각  Tue 2025-11-25 18:16:31 UTC
king   현재 부팅 시작 시각  Fri 2025-07-11 18:52:59 UTC
```

세 노드 모두 **부팅 직후 시스템 시각이 과거의 고정값**이다. RTC 배터리가 없거나 동작하지 않아 전원이 끊기면 시계가 초기화된다. NTP가 동기화되기 전까지 로그 타임스탬프가 틀린다.

### 영향

- 부팅 직후 기록된 로그의 타임스탬프를 신뢰할 수 없다
- 노드 간 이벤트 순서를 맞출 수 없다 (`02-HARDWARE-SETUP.md` §10)
- 벤치마크 결과에 잘못된 시각이 기록될 수 있다

### 조치

`chrony`를 활성화하고, **동기화 완료를 확인한 뒤에 측정을 시작**해야 한다. `scripts/fix-node-consistency.sh`의 `chrony` 단계에 포함되어 있으나 아직 실행하지 않았다.

벤치마크 스크립트는 실행 전 다음을 확인하도록 한다.

```bash
chronyc tracking | grep -E "Leap status|System time"
# Leap status : Normal 이어야 하며 Not synchronised 면 대기
```

각 노드가 자신이 측정한 duration만 응답에 담고 절대 시각을 비교하지 않는 설계(§10.1)라 측정값 자체는 영향받지 않는다. 문제는 **로그 상관 분석**이다.

---

# 3.5 문서 재구성 (2026-08-07)

작업 이력이 길어져 두 문서로 나눴다.

| 문서 | 역할 |
|---|---|
| `board-worklog.md` (이 문서) | **시간순 작업 이력.** append 전용. 왜 그렇게 했는지 |
| `infrastructure.md` | **현재 상태 스냅샷.** 지금 어떤 상태인지 |
| `environment-matrix.md` | **버전·해시 고정.** 재현에 필요한 값 |

"지금 상태가 어떤가"를 알려면 `infrastructure.md`를, "어쩌다 이렇게 됐나"를 알려면 이 문서를 본다.

---

# 4. PC 측 변경 사항

보드가 아니라 개발 PC(`192.168.123.26`)에 적용한 내용이다.

| 날짜 | 항목 | 내용 |
|---|---|---|
| 2026-08-07 | SSH 키 | `~/.ssh/id_ed25519_npuforge` 생성 (passphrase 없음, 자동화용) |
| 2026-08-07 | SSH config | `npuforge-k` / `npuforge-q` / `npuforge-j` 별칭 추가. 기존 config는 `~/.ssh/config.bak.*`로 백업 |

## 4.1 SSH 별칭

```text
npuforge-k → pi@192.168.123.12  (king)
npuforge-q → pi@192.168.123.16  (queen)
npuforge-j → pi@192.168.123.33  (jack)
```

별칭은 `npuforge-k/q/j`로 유지하고 hostname만 `king/queen/jack`으로 두었다. 별칭을 바꾸면 이미 작성한 스크립트를 모두 수정해야 하므로, 추론망 구성 시 한 번에 정리한다.

## 4.2 sudo 실행 패턴

`pi` 계정은 sudo에 비밀번호를 요구한다. 자동화에서는 다음 형태를 사용한다.

```bash
ssh npuforge-k 'printf "$NPUFORGE_SUDO_PASS\n" | sudo -S -p "" <command>'
```

§2.4에 기록한 파이프 충돌 함정에 주의한다.

**개선 여지.** 벤치마크 자동화에서 sudo 호출이 늘어나면 특정 명령에 한해 NOPASSWD sudoers 규칙을 두는 편이 낫다. 다만 이는 권한 확대이므로 별도 승인 후 진행한다.

---

<a id="todo"></a>

# NPUForge 진행 현황

- 최종 갱신: **2026-08-21**
- 발표까지: **D-99** (2026-11-28)
- 기능 동결: 2026-11-15

> 이 문서는 **지금 뭘 해야 하는지** 한눈에 보기 위한 것이다.
> 왜 그렇게 했는지는 `board-worklog.md`, 값은 `environment-matrix.md`, 상태는 `infrastructure.md`.

---

# ▶ 현재 상태: **측정 계보 종료** (2026-08-21)

S2 부터 S3.9b·S0-D 까지 전부 닫혔다. **421건 측정, 전 구간 오류율 0.**
남은 것은 발표 자료(그림)뿐이다.

| 계보 | 상태 | 결론 |
|---|---|---|
| **전송** | 닫힘 | 운영점 = **노드당 커넥션 2개 @ c12**. 3N 387.2 inf/s (+13.3%) |
| **확장** | 닫힘 | 3N **2.86× (95.3%)**. 손실은 **tail 에서 나타난다** — p50 평평, p99 +36% (S3.9a). micro-mechanism 까지 분리한 것은 아니다 |
| **지속 부하** | 닫힘 | 능동 냉각에서 short-run = sustained (−1.9%) |
| **정책** | 닫힘 | RR 은 이질에 취약, adaptive 가 tail −37%. 기본값 **`ect` 유지** |
| **io_uring (S4)** | **반박됨** | 회수 대상이 transport 비용의 1%. CPU 는 제약이 아니다 (S3.9b) |

```text
로컬 direct 161.5   운영점 135.5   잔여 gap 26.0 inf/s = direct 기준 16.1%
  -> CPU 비용이 아니라 경로 지연으로 보인다 (범위 밖, 관측만)
```

## 클러스터 조작 요령

```bash
# 접속 + 사전 검사
for h in npuforge-k npuforge-q npuforge-j npuforge-server; do ssh $h hostname; done
bash scripts/preflight-check.sh --with-inference

# 노드 재기동 — pkill 은 comm 으로 (-f 금지, ADR-017), 로그 리다이렉트 필수
ssh npuforge-k 'pkill -9 npuforge-node; sleep 3;   setsid nohup ~/npuforge/npuforge-node.s36 --config ~/npuforge/node.toml   >>~/npuforge/node.log 2>&1 & disown'
# 헬퍼: npuforge_restore_cluster (scripts/lib/remote.sh)

# run 합계 재계산
bash scripts/count-runs.sh
```

> **하네스 불변조건 2개**(`experiments/README.md` §4.12) — 새 하네스는 반드시 지킨다.
> ① 공유 자원의 상태는 공유 자원 쪽에서 검증한다(`npuforge_assert_cluster_free`).
> ② 결과 경로를 덮어쓸 수 있는 임시 폴더처럼 다루지 않는다.

## M3 토폴로지 (2026-08-20 확정, 실측 완료)

```text
        server 192.168.123.9  (Xeon x2 24T / 16GB / Rocky 9.4)
                    │
                  10GbE          ← aggregation. 10G full 실측
                    │
          NEXI NS-S25G10G-N  (2.5G x4 + 10G x2)
              ├── 2.5G ── king  .3
              ├── 2.5G ── queen .5
              └── 2.5G ── jack  .4
```

worker 링크는 2.5G, **aggregation 만 10G.** 옛 `dealer`(노트북)는 제거되고
스케줄러 역할이 `server` 로 이관됐다. `infrastructure.md` §1.

> **IP 고정 완료.** 개편 때 보드 IP 가 통째로 바뀌어(`.12/.16/.33` → `.3/.4/.5`)
> SSH 별칭이 낡았었다. 4대 전부 호스트 static(`manual`)으로 고정했다 (§1.1,
> `infrastructure.md` §2.3).

---

# ▶ 정책 계보 — 닫힌 것과 미룬 것 (2026-08-21)

**닫혔다.** RR 은 이질성에 취약하고, 상태 신선도를 고친 부하 인지
스케줄링이 RR 의 tail 을 크게 개선한다(p99 −37%). LQ·ECT 둘 다 정상
동작하며 regression 이 없다. **기본값은 `ect` 유지.**

**미뤘다 (Future Work).** 강한 이질에서 ECT 가 LQ 보다 우월한지는
미확정이다. 다만 **그 우열은 핵심 결론을 바꾸지 않는다** — 핵심은
"부하 인지 스케줄링이 이질을 흡수한다" 이고 어느 쪽으로도 성립한다.

S0-D 교정이 그 질문을 **재현 가능하게** 만들어 뒀다. 언제든 40분이면
답이 나온다(팬 ON, 예열 불필요).

```text
king CPU 캡   1200   1008    816    600
노드 지연 편차 1.33x  1.79x  2.26x  3.93x     ← 816 이 S0-A(2.4x) 재현
```

→ [`experiments/S0_D_CAPACITY_HETERO.md`](#experiments-s0-d-capacity-hetero) §6

---

# ▶ S3.9b 완료 (2026-08-21) — **S4 io_uring 취소/보류**

```text
질문   io_uring 이 남은 16.1% 를 회수하는가?
답     아니다. 회수 대상(syscall 진입)이 transport 비용의 1%,
       가장 관대한 가정으로도 8%. 게다가 CPU 는 제약이 아니다.
```

| | 값 |
|---|---|
| transport 비용 | **16.35 CPU-ms/req** (유저 9.37 / 커널 6.99) |
| 네트워크 syscall | 요청당 ~165회 × 1µs = **0.165ms = 1.0%** |
| 보드 CPU | **48.9% idle**, 최고 코어 cpu0 78.8% busy(softirq) |
| cpu0 softirq | S3.5 §4.3 에서 RPS 분산 → **−0.2% null** |

**CPU-ms/req 는 비용이지 제약이 아니다.** 포화되지 않은 자원의 사용량을
줄이는 것은 처리량을 올리지 않는다.

큰 항은 따로 기록했다 — **유저 시간이 커널보다 크다**(직렬화·유저공간
copy 가 transport 비용의 57%). 다만 사전 규칙 3번째 가지대로 **여기서
멈춘다**: CPU 가 제약이 아닌 이상 이것을 줄여도 처리량이 오른다는 보장이
없다.

범위 밖 관측: gap 은 CPU 비용이 아니라 **경로 지연**으로 보인다
(지연 +37.3ms 중 노드 CPU 는 16.35ms, 페이로드 1.2MB 왕복 전송만 8.2ms).
지렛대가 있다면 io_uring 이 아니라 **페이로드 크기**다.

→ [`experiments/S3_9B_NODE_RESIDUAL.md`](#experiments-s3-9b-node-residual)

---

# ▶ (완료된 계획) S3.9b — node-side residual cost profiling

## 질문 (좁게)

> **161.5 → 135.5 사이의 residual gap 에서 node-side serialization /
> copy / syscall 비용이 유의미한 비중을 차지하는가?**

```text
로컬 direct 161.5   운영점 135.5   gap 26.0 inf/s = direct 기준 16.1%
                                   (1 − 135.5/161.5)
```

> 이전에 돌던 **13.2% 는 틀린 값이다.** 그 백분율은 140.1(S3.6 C)에서
> 나왔고 140.1 은 **c32 = 과부하 구간** 측정이라 운영 판단에 쓸 수 없다
> (README §4.1). 운영점 숫자와 짝지어져 두 계보가 섞였다.

**목적은 gap 을 전부 설명하는 것이 아니다.** S3.9a 에서 scale-out
tail/TCP 쪽 비용이 별도로 드러났으므로, node-side 프로파일이 26 inf/s
전체를 설명해야 할 이유가 없다. 설명 못 한 잔여는 잔여로 남긴다.

## 판정

| 결과 | 결정 |
|---|---|
| syscall·copy 가 **충분히 큼** | **S4 io_uring 진입** |
| **작음** | **S4 취소/보류** |
| **다른 항이 큼** | 그 항만 기록. **핵심 범위 밖이면 더 안 판다** |

세 번째 행이 중요하다. 프로파일이 예상 밖의 항을 가리켜도 그것을
쫓아가는 것은 이 실험의 임무가 아니다. 기록하고 범위 밖이면 멈춘다.

## 하네스 불변조건 (2026-08-21 확립)

새 하네스를 짤 때 반드시 지킨다. 둘 다 실제 사고에서 나왔다.

1. **공유 자원의 상태는 공유 자원 쪽에서 검증한다.**
   `npuforge_assert_cluster_free` — 서버에 `npuforge-bench` 가 돌면
   시작하지 않는다. 로컬 프로세스 관측은 플랫폼에 따라 거짓말을 한다.
2. **결과 경로를 append/overwrite 가능한 임시 폴더처럼 다루지 않는다.**
   기존 디렉터리가 비어 있지 않으면 멈춘다. `NPUFORGE_SUFFIX` 로 구분.

---

# 0. 한눈에 보기

| 영역 | 상태 |
|---|---|
| 소프트웨어 (M0) | ✅ 완료 — workspace, common, mock backend, 정책 엔진, CI |
| 하드웨어 인프라 | ✅ 완료 — 열 편차 5.6°C, **NPU** throttling 없음. 단 **CPU 는 강등된다** (팬은 S0-B 비교용) |
| RKNN 검증 | ✅ 백엔드 구현 완료, 컨텍스트 공유 위험 실측 확인 |
| 모델 변환 | ✅ FP16·INT8 완료, 정확도 검증 완료 |
| gRPC 통신 (M2) | 🟡 거의 완료 — 배선·재시도·Mock 클러스터 검증 끝, 메트릭 남음 |
| 벤치마크 (M3) | ✅ **완료 (2026-08-21)** — S2·S3·S3.5~3.9b·S0-A~D, **421건 / 오류율 0** |
| 대시보드 (M6) | ⬜ 미착수 |

**M3 차단 요소 — 전부 해소됨 (2026-08-20)**

| # | 항목 | 상태 |
|---|---|---|
| 1 | 2.5G/10G 스위치 | ✅ NEXI NS-S25G10G-N |
| 2 | PCIe 슬롯 서버 | ✅ Xeon x2 / 16GB / Rocky 9.4 (.9) |
| 3 | 10G NIC + 케이블 | ✅ `enp4s0` 10GBASE-T, 10G full 실측 |
| 4 | ~~`want_float=0` 전환~~ | ✅ 2026-08-12 |

**남은 작업 (2026-08-21 기준)**

| 항목 | 규모 | 왜 |
|---|---|---|
| ~~발표용 그림 보강~~ | ✅ **완료 (2026-08-21)** | 7개 추가 — `scripts/make-experiment-figures.py`. 경로는 handoff §5 |
| Prometheus 메트릭 (M2 잔여) | — | gRPC 통신 항목의 마지막 조각 |
| 대시보드 (M6) | — | 미착수 |
| systemd 이전 | — | 초안 `scripts/npuforge-node.service.in`. `pkill`→`systemctl stop` 과 함께 |
| queen·jack SSH host key 재생성 | — | 두 보드 키가 같아 구분 불가 |

> **측정은 더 필요하지 않다.** 추가 실험을 시작하기 전에
> `experiments/README.md` §2(배제표)와 §7(미해결)을 먼저 본다 —
> 이미 배제됐거나 조건부로 열려 있는 후보인지 확인하기 위해서다.

---

# 1. 즉시 할 일

## 1.1 사용자 작업 (물리·구매)

- [x] **보드 3대 배치 균일화** — 조치 불필요로 결론 (2026-08-11)
  - 통제된 재측정에서 19°C 격차가 **재현되지 않음**
  - 8스레드 동시 부하 15분: king 75.8 / queen 70.2 / jack 74.8°C (편차 5.6°C)
  - 90°C 초과 없음, NPU 클럭 강하 없음(928샘플 전부 950MHz)
  - 처리량 편차 3.5% (80.5 / 77.7 / 77.8 inf/s)
  - 이전 19°C는 부하 프로파일 차이(스윕 vs 고정, 6분 선행)로 부풀려진 것으로 판단
  - 상세: `board-worklog.md` §2.19
- [ ] **팬리스(S0-A) 클러스터 측정** — 오늘 baseline 은 능동 냉각(조건 B)이다.
  27% 를 확정하려면 조건 A 도 같은 gRPC 경로로 재야 한다 (§9)
- [x] **동일 모델 팬 3개 설치** (2026-08-20) — 120mm 5V USB, 노드당 1개(보드보다 큼).
  2026-08-20 측정 전체가 이 능동 냉각(조건 B)에서 수행됨
- [x] **2.5G/10G 스위치** (2026-08-20) — NEXI NS-S25G10G-N (2.5G×4 + 10G×2)
- [x] **스케줄러 서버 확보** (2026-08-20) — Xeon E5-2630L ×2 / 16GB / Rocky 9.4 (.9)
- [x] **10G NIC + 케이블** (2026-08-20) — 서버 내장 10GBASE-T, 10G full 실측
- [x] **IP 고정** (2026-08-20) — 호스트 NetworkManager static 으로 4대 전부
  고정(개발 작업으로 처리). ipTIME 라우터 예약은 선택 사항이며, 하면 아래 표를 쓴다.
  ```text
  king 22-94-FF-34-46-B1 →.3   jack 62-CE-3B-B6-E4-41 →.4
  queen 7E-D8-D7-40-45-82 →.5  server 6C-B3-11-13-2F-38 →.9
  ```
- [x] **calibration 이미지 방향 결정** — COCO val2017 200장 채택 (2026-08-11)
  - `tools/model-converter/fetch_calibration.py` 로 결정적 선택(seed 고정)
  - 이미지는 저장소에 넣지 않는다. manifest 만 남긴다 (라이선스)

## 1.2 개발 작업

- [x] `preflight-check.sh` — 벤치마크 전 하드 실패 검사 (2026-08-11)
  - 별칭↔hostname, 커널/RKNN/드라이버/모델 해시 일치
  - governor, 유휴 온도, 입력 전압, 남은 부하, NTP, 세션 수
  - `--with-inference`: 세 보드가 같은 입력에 같은 답을 내는지 (§9 교훈)
  - 음성 검사로 실제 검출 확인 (모델 바꿔치기, 부하 잔존)
- [x] `boot_id` 기록 — run 중 리셋 감지 및 무효화
  - 노드가 하트비트로 보고, 스케줄러가 변화 감지 시 경고 (M2에서 구현)
- [x] 벤치마크 telemetry 확장 — `ListNodes` RPC 로 온도·전압·boot_id 수집
  - 하트비트로 조회하면 스케줄러가 그 값을 관측으로 기록해 상태를 덮어쓴다.
    읽기 전용 RPC 를 따로 뒀다.
- [ ] **queen·jack SSH host key 재생성** — 둘이 동일해 암호학적으로 구분 불가.
  IP 가 바뀌면 경고 없이 엉뚱한 보드에 붙는다. DHCP 라 실제로 IP 가 바뀌므로
  (2026-08-20 개편에서 겪었다) 방치하면 안 된다.
  ```bash
  ssh npuforge-j 'sudo rm -f /etc/ssh/ssh_host_* &&     sudo ssh-keygen -A && sudo systemctl restart ssh'
  ssh-keygen -R npuforge-j   # PC 의 known_hosts 정리
  ```
- [x] **`want_float=0` 전환** (2026-08-12) — 설정 `[worker] want_float`
  - blob v2 로 `scale`·`zero_point` 동봉. 실보드 역양자화 검증 최대 오차 9.5e-7
  - 처리량 **INT8 +17.3% / FP16 +15.7%**, 출력 크기 4분의 1
- [ ] **bench 에 per-request 지연 원본 덤프 옵션** (2026-08-20 필요성 확인)
  현재 bench 는 run 마다 요약 percentile 만 JSON 에 남긴다. 그래서 여러 run 을
  묶은 표의 p95/p99 는 **run-level percentile 의 평균**이지 요청을 전부 합친
  pooled percentile 이 아니다(S2 §7.4.1).
  - run-level 평균은 각 run 의 최악 구간이 희석돼 **tail 을 낮게 보이게 한다.**
    조건 간 비교에는 문제없지만 절대값을 "이 시스템의 p99" 로 인용하면 안 된다.
  - S3.7 이 tail 로 운영점을 고르므로 이 구분이 실제로 중요해졌다.
  - 할 일: `--dump-samples <path>` 로 per-request 지연을 남기고, 분석기가
    pooled percentile 을 계산하게 한다. **단 bench 는 측정 도구라 S3.7/S3.8
    진행 중에는 바꾸지 않는다** — 동결 구간이 끝난 뒤에.
- [x] **jack 노드 복구** (2026-08-20) — 하드웨어는 정상이었다
  - eth0 **2.5G up**, IP 192.168.123.4, 바이너리·설정·모델 해시
    (`dba155d2…`)·governor 전부 정상. OOM·segfault 흔적 없음.
  - `dmesg` 가 이력을 보여줬다: 부팅 시 케이블이 **eth1** 에 있었고
    (`t=13.6s eth1 link up`), `t=620819s` 에 eth1 link down,
    `t=689135s` 에 **eth0 link up** — 즉 케이블이 물리적으로 옮겨졌다.
    다만 링크 단절은 프로세스를 죽이지 않는다(노드는 등록을 재시도한다).
  - **원인은 확정하지 못했다.** 로그가 없었기 때문이다 — 기동 절차가
    `setsid nohup ... &` 인데 로그 리다이렉트가 빠져 표준출력이 버려졌다.
  - 복구 후 검증: 3노드 335.4 inf/s, jack 33.3%(3362건), 오류 0,
    preflight `--with-inference` 전 항목 통과(**3노드 추론 출력 해시 동일**
    `e84c5b53…`).
  - 재발 방지: `lib/remote.sh` 의 `npuforge_restore_cluster` 가 세 노드를
    모두 복구하고 **로그 리다이렉트를 강제**한다. 측정 스크립트가 1노드
    구성을 만들려고 queen·jack 을 죽이면서 복구는 queen 만 하던 것이
    문제를 지속시켰다.
- [ ] **노드 기동을 systemd 로 옮기기** — 초안 `scripts/npuforge-node.service.in`
  로그 보관(journald)·마지막 종료 상태·재시작 정책을 얻는다. 위 jack 건에서
  "왜 죽었는지 알 수 없는" 상태가 프로세스가 죽는 것보다 나빴다.
  - ⚠️ **지금 설치하면 안 된다.** 측정 스크립트는 `pkill -9 npuforge-node`
    로 1노드 구성을 만든다. `Restart` 가 걸리면 systemd 가 즉시 되살려
    **1노드 측정이 조용히 3노드가 된다.** 틀린 줄 모르는 종류의 사고다.
  - 함께 해야 할 것: `run-*.sh` 의 `pkill` 을 `systemctl stop` 으로 교체.
    측정 캠페인(S3.8) 종료 후에 처리한다.
- [ ] **`ondemand` vs `performance` 300초 비교** ← §11 결론의 범위 확인
  - +7% 는 120초 측정이다. 지속 부하에서는 `performance` 가 더 빨리
    뜨거워져 불리할 수 있다. `discuss.md` §12
- [ ] **S0 를 30분으로** — 정상 상태 처리량과 CPU 강등 시점 확정
- [ ] **INT8 모델을 queen·jack 에 배포** — 현재 `king` 에만 있다
- [x] **스케줄러 빌드 경로 = server 네이티브** (2026-08-20 검증 완료) — server 에
  rust/cargo(dnf 1.92, MSRV 1.85 충족)·gcc·protoc·git 설치, `git archive`
  tarball 을 scp. 노드(aarch64)는 종전대로 king. 크로스빌드는 링커 문제로 회피
- [x] **IP static 고정** (2026-08-20) — server·king·queen·jack 4대 전부 manual.
  라우터 예약 대신 호스트 NetworkManager static. 같은 IP 라 SSH 무중단
- [ ] **server 방화벽 gRPC 포트 개방** — firewalld public zone, 측정 전
- [ ] `server`를 NTP 서버로 구성 + `chronyc waitsync` 대기
- [x] **스케줄러 RSS 우려 완화** (2026-08-20) — 서버 RAM 3GB → **16GB**.
  dealer 노트북 제약이 해소됐다. 그래도 S2 에서 RSS 는 관찰한다
  (`environment-matrix.md` §10.1)
- [x] CPU governor → `performance` (2026-08-12) — systemd 유닛으로 영구화
  - `scripts/set-cpu-governor.sh`, 재부팅 유지 확인 완료
  - **처리량 +7%.** 기존 수치는 전부 ondemand 기준이었다 (discuss.md §11)
- [x] `worker_count` 확정 — **8**, `core_mask` 미설정 (discuss.md §4)

---

# 2. 마일스톤별 진행

## M0. 저장소 및 환경 — ✅ 완료

- [x] Rust workspace (7 크레이트, edition 2024)
- [x] `npuforge-common` — 타입, 오류 코드, 설정, 백엔드 인터페이스
- [x] `npuforge-mock-backend` — 결정적 시드, 지연·오류율·속도편차 주입
- [x] `npuforge-rknn` 스텁 — feature 게이트로 Windows 빌드 통과
- [x] 스케줄링 정책 3종 (round-robin / least-queue / ect)
- [x] 노드 레지스트리 + 상태머신 + drain/disable
- [x] CI (fmt, clippy, test, aarch64 크로스, cargo-deny)
- [x] LICENSE (Apache-2.0), NOTICE, DEPENDENCIES.md, MODEL_LICENSES.md
- [x] 설정 예제 (scheduler, node, mock 3노드)
- [x] 테스트 통과 — M0 시점 81개, **현재 workspace 209개** (2026-08-14)

## M1. 단일 노드 추론 — 🟡 진행 중

- [x] RKNN C Wrapper 작성 및 **실기 컴파일 검증**
- [x] Thread-safety 검증 — **RKNN 2.3.0은 thread-safe 확정**
- [x] FFI 시그니처 실제 헤더와 대조 완료
- [x] 모델 변환 환경 (Docker, rknn-toolkit2 2.3.0)
- [x] YOLOv8n FP16 변환 및 3노드 배포
- [x] INT8 변환 — 6.46MB (FP16 9.65MB 대비 -33%)
- [x] 추론 정확도 검증 — 실보드 검출 수준 비교, `results/accuracy/README.md`
- [x] `npuforge-rknn` 실제 구현 — 컨텍스트 풀 + 다중 출력 (2026-08-11)
  - 실장비 통합 테스트 6종 통과 (`tests/real_device.rs`)
  - **공유 컨텍스트는 API 오류 0건으로 100% 틀린 결과를 낸다** — 실측
    (`environment-matrix.md` §3.1 정정)
- [ ] 1,000회 반복 추론 안정성 (soak으로 24,000회 확인, 정식 테스트는 별도)

## M2. 원격 추론 — 🟡 메트릭만 남음

- [x] `npuforge-proto` — .proto 정의 및 tonic 연결
- [x] `NodeService` gRPC 서버 (노드 측)
- [x] 노드 등록 / 하트비트 (등록 백오프 재시도, `must_reregister` 재등록)
- [x] `SchedulerService` gRPC 서버
- [x] 스케줄러 → 노드 gRPC 클라이언트 (노드별 채널 재사용)
- [x] 로컬 큐 + 워커 풀
- [x] 오류 처리 및 재시도 (재시도 시 다른 노드 선택)
- [x] 모델 디렉터리 로딩 + SHA-256 검증
- [x] **로컬 3노드 Mock 클러스터 동작 확인** (하드웨어 없이)
- [ ] 기본 메트릭 (Prometheus)
- [x] `npuforge-bench` CLI — 부하 발생·집계·run 유효성 판정 (2026-08-11)

### 검증한 것 (2026-08-11)

통합 테스트 `crates/npuforge-scheduler/tests/mock_cluster.rs` — 실제 gRPC 를 타고
스케줄러 ↔ 3노드가 붙는다. 프로세스만 하나일 뿐 전송 경로는 실장비와 같다.

| 검증 항목 | 결과 |
|---|---|
| 요청이 3노드에 분산 | ✅ round-robin 이 세 노드를 모두 사용 |
| 노드 1대 사망 시 우회 | ✅ 6/6 성공, 죽은 노드는 결과를 내지 않음 |
| 전 노드 사망 | ✅ `NPF-1302` + 시도한 노드 목록 |
| 타이밍 분해 | ✅ 노드 측정 구간과 스케줄러 측정 구간이 모두 채워짐 |
| 느린 노드 회피 | ✅ least-queue 가 빠른 노드를 더 많이 사용 |

실제 프로세스 4개(스케줄러 + 노드 3)로도 확인했다.
스케줄러를 죽였다 다시 띄우면 세 노드가 **약 1.3초 안에 스스로 재등록**한다.
노드는 하트비트 실패를 곧바로 재등록으로 전환한다 — 일시적 네트워크 오류와
스케줄러 재시작을 구분할 수 없으므로 더 비싼 쪽을 택했고, 등록은 멱등이다.

## M3. 다중 노드 — 🟡 클러스터 실동작 확인 (2026-08-20)

- [x] **실장비 3노드 등록** — king/queen/jack 스케줄러(.9) 등록 확인
- [x] **Round Robin 라우팅** — 예비 벤치에서 33.3% 정확 3등분
- [x] `npuforge-bench` CLI
- [x] **예비 3노드 추론** — c6 146 / c24 336 inf/s, 오류 0%
- [x] **S2 확장성 첫 측정** (2026-08-20) — 1/2/3노드 111.6/228.7/337.7 inf/s,
  **확장 효율 ~98%**. 클러스터 노드 상한 115 < 로컬 157 (스케줄러 오버헤드 27%).
  preflight 통과. RESULTS §2.5, board-worklog §2.25
- [ ] **S2 정식** — 반복 run·팬 조건·`--with-inference`·TimingBreakdown 오버헤드 분해
- [x] model.toml `model_file` 상대경로 버그 수정 (2026-08-20, §6 이슈 8)

## M4. 동적 스케줄링 — ⬜

- [ ] Least Queue / ECT 실장비 검증
- [ ] 정책 비교 (S3)

## M5. 장애 복구 — ⬜

- [ ] 헬스체크 실장비 검증
- [ ] 자동 제외 / 복귀
- [ ] 재시도 경로 검증
- [ ] **보드 하드 리셋과 의도된 장애 구분** (boot_id)

## M6. 대시보드 — ⬜

- [ ] 클러스터 개요 / 노드 뷰 / 벤치마크 뷰 / 이벤트 타임라인
- [ ] SSE 실시간 전송
- [ ] 전압·온도·주파수 표시

## M7. 최적화 실험 — ⬜

- [ ] **S2 확장성 실험 설계 재검토** ← INT8 결과로 전제가 바뀜
  - 노드당 1.545 Gbps (INT8) / 0.829 Gbps (FP16)
  - 3노드 4.636 / 2.486 Gbps — **둘 다 2.5GbE 한 링크를 넘는다**
  - aggregation 링크를 10G 로 (§4 토폴로지)
  - `discuss.md` §8, `RESULTS.md` §8.1 참조

- [ ] 버퍼 풀
- [ ] CPU 프로파일 (전처리 비중 확인)
- [ ] io_uring 적용 여부 판단

## M8. 발표 릴리스 — ⬜

- [ ] v0.1 태그, README, 설치 스크립트
- [ ] 벤치마크 원본 공개
- [ ] 발표자료, 데모 영상, 예비 영상

---

# 3. 벤치마크 시나리오

**전제: S0가 다른 모든 시나리오의 임계치와 cooldown을 결정한다. 반드시 먼저.**

- [ ] **S0-A** 열 특성 (팬리스) — 3노드 × 1,800초
- [ ] **S0-B** 열 특성 (냉각) — 3노드 × 1,800초
- [ ] S1 단일 노드 기준
- [ ] S2 확장성 (1/2/3노드)
- [ ] S3 스케줄러 정책 비교
- [ ] S4 장애 대응
- [ ] S5 네트워크 구현 비교
- [ ] S6 입력 크기 비교

총 146 run, 약 23.4시간. 무인 야간 실행 필요.

---

# 4. 인프라 현황

| 항목 | 상태 |
|---|---|
| 보드 3대 (king/queen/jack) | 🟡 OS·커널·RKNN·gcc·governor 일치, eth0 2.5G 실측. **SSH host key queen·jack 동일 (미해결)** |
| SSH 별칭·키 인증 | ✅ IP 갱신 완료 (.3/.5/.4/.9), `npuforge-server` 추가 |
| `server` (Rocky 9.4, 스케줄러·벤치) | ✅ **Xeon x2 24T / 16GB / 10G**. Rust·Docker 미설치 |
| 전원 5V 4A × 3 | ✅ 지속 부하 검증 완료 |
| **2.5G/10G 스위치** | ✅ NEXI NS-S25G10G-N |
| **추론망 대역** | ✅ worker 2.5G / aggregation 10G, 3노드 합 5.11 Gbps 실측 |
| 관리망/추론망 분리 | ⬜ 단일 대역 공유 중, M3 전 결정 |
| IP 고정 | ✅ 4대 static (manual). 라우터 예약은 선택 |
| 보드 물리 배치 | ✅ 편차 5.6°C, NPU throttling 없음 (2026-08-11 확인) |
| 냉각 (팬 3개) | ⬜ 미구매 |
| CPU governor | ✅ `performance` 고정 + 재부팅 유지 |
| NTP 동기화 | ⚠️ chrony 설치됨, `server` 서버화 미완 |
| 온도 임계치 | ⚠️ 초안값 (80/90°C) — S0 후 재설정 |

---

# 5. 구매 목록

M3 를 막던 장비(스위치·서버·10G NIC)는 **전부 확보됐다** (2026-08-20).
남은 것은 측정 품질용이다.

| 항목 | 수량 | 우선순위 | 비고 |
|---|---:|---|---|
| **동일 모델 팬** | 3 | 중간 | 5V USB, 동일 회전수. S0-B 용 |
| Cat6/6a 케이블 (여유) | 2~3 | 낮음 | 10G 예비. 현 링크는 정상 |
| USB 전력 측정기 | 3 | 낮음 | 5V 입력이라 USB 계측기 가능 |
| 예비 케이블·어댑터 | 각 1 | 중간 | 발표 대비 |

**전원 어댑터는 해결됨** (5V 4A × 3 교체 완료).

---

# 6. 알려진 이슈

| # | 이슈 | 심각도 | 상태 |
|---|---|---|---|
| 1 | ~~`king` 온도 19°C 높음~~ | 해소 | 통제 재측정에서 재현 안 됨 (편차 5.6°C) |
| 2 | 온도 임계치가 정상 동작 범위와 충돌 | **높음** | S0 후 재설정 |
| 3 | RTC 없음 — 부팅 직후 시각 틀림 | 중간 | chrony 대기 로직 필요 |
| 4 | 전류 센서 없음 → FPS/Watt 산출 불가 | 중간 | 외부 USB 전력계 필요 |
| 5 | 8스레드에서 처리량 안 꺾임 | 낮음 | MAX_THREADS 확장 필요 |
| 7 | 보드(king)에만 Rust 툴체인 설치됨 | 낮음 | 빌드 전용. 바이너리는 한 번 빌드해 배포 |
| 6 | `npu_cores` 수집값이 devfreq 개수(1) | 낮음 | 지표 정의 수정 |
| ~~8~~ | ~~model.toml `model_file` 상대경로 미해석~~ | 해소 | `main.rs` 가 `load_model` 전에 `spec.model_file` 을 절대경로로 교체. 상대경로 model.toml 로 3노드 로딩·벤치 재검증 완료 (2026-08-20) |
| 9 | 노드 재기동 시 NPU 컨텍스트 미해제 | 중간 | 죽은 노드가 컨텍스트를 안 놓아 재기동 status=-2. `pkill -9`+대기 필요. graceful shutdown 점검 |

---

# 7. 확정된 주요 수치

발표와 문서에 인용 가능한 실측값이다.

| 항목 | 값 | 출처 |
|---|---|---|
| SoC | RK3576, NPU 2코어 6 TOPS | 실측 |
| RKNN 동시성 | **전용 context 동시 실행 가능 / context 공유 금지** | 공유 시 API 오류 0인데 결과 200/200 불일치 |
| FP16 8스레드 순간 처리량 | 70~78 inf/s | 3노드 실측 |
| **FP16 8스레드 지속 처리량** | **84.3 inf/s** | governor=performance, 120초 |
| **INT8 8스레드 지속 처리량** | **157.2 inf/s** | governor=performance, **FP16 대비 1.86배** |
| INT8 평균 지연 | 50.8 ms | FP16 94.5 ms 대비 -46% |
| CPU governor 영향 | +7% | ondemand→performance. **120초 측정.** 지속 부하 미검증 |
| `want_float=0` 효과 | **INT8 +17.3% / FP16 +15.7%** | 출력도 4분의 1 (discuss.md §12) |
| **정상 상태 처리량 (300초)** | **FP16 59.7 inf/s** | 시작 81.6 대비 **-27%**. CPU throttling |
| (참고) ondemand 기준 | FP16 79.0 / INT8 146.2 | 08-11 이전 측정은 전부 이 기준 |
| 추론당 커널 ioctl | **76회 (FP16·INT8 동일)** | strace, 상한은 횟수가 아니라 시간 |
| **Peak vs Sustained 저하** | **약 10%** | 77.3 → 69.7 |
| 권장 `worker_count` | **8** (`core_mask` 미설정) | core_mask 스윕 |
| NPU 2코어 실제 기여 | **1.51배** (2배 아님) | 대조군 비교 |
| 지속 부하 시 NPU 온도 | **67.5~75.8°C** (3대, 8스레드 15분, FP16) | 2026-08-11 통제 측정 |
| INT8 정확도 (vs FP16) | 검출 셀 10/10, 클래스 100%, box cos 0.997 | 실보드 검출 수준 |
| **공유 컨텍스트 결과 불일치** | **100%** (API 오류 0건) | 컨텍스트 풀이 필수인 이유 |
| 노드 간 온도 편차 | **5.6°C** (NPU throttling 없음) | 동시 부하 |
| **CPU thermal 강등** | A72 2208→**816MHz**, A53 2016→**600MHz** | 부하 60초 후. NPU 는 950MHz 유지 |
| 부하 중 입력 전압 | 최소 5.05V | 3대 동시 실측 |

> Peak vs Sustained 격차는 벤더 스펙시트에 없는 수치이며 본 프로젝트의 핵심 산출물 중 하나다.
> 단, 현재 값은 선풍기 개입으로 오염되어 있어 **S0에서 깨끗하게 재측정해야 한다.**
>
> 2026-08-11 통제 측정(15분 × 3대)의 지속 처리량은 **77.7~80.5 inf/s** 로,
> soak 의 69.7 inf/s 보다 높다. soak 조건(24,000회, 더 긴 지속)과 다르므로
> 직접 비교하지 않는다. S0 에서 조건을 통일해 확정한다.
