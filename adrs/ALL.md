<a id="index"></a>

# NPUDure 아키텍처 결정 기록 (ADR)

> **이 파일은 생성물이다. 직접 편집하지 않는다.**
> `adrs/` 의 원본 31개를 읽기·인쇄·공유용으로 이어 붙인 것이다.
> 고칠 것이 있으면 원본을 고치고 다시 만든다.
>
> ```bash
> python scripts/build-adr-bundle.py $(git log -1 --format=%cs -- adrs/)
> ```
>
> - 생성 기준: **미상** (`adrs/` 최종 커밋일)
> - 원본: `adrs/README.md`, `adrs/OVERVIEW.md`, ADR 28건, `adrs/TEMPLATE.md`
> - 파일 간 링크는 문서 내 앵커로 바뀌어 있다

이 폴더는 **"왜 이렇게 되어 있는가"** 에만 답한다.

무엇이 어떻게 동작하는지는 `docs/01-TECHSPEC.md` 가, 무슨 값이 나왔는지는
`docs/RESULTS.md` 가 담당한다. 여기에는 **갈림길에서 무엇을 골랐고 무엇을
버렸는지**, 그리고 **어떤 관측이 나오면 그 선택이 뒤집히는지**를 적는다.

---

## ADR 이 왜 따로 필요한가

이 저장소의 다른 문서는 대부분 **시간순**이다.

| 문서 | 축 |
|---|---|
| `docs/discuss.md` | 실험한 순서대로 |
| `docs/board-worklog.md` | 작업한 순서대로 |
| `docs/RESULTS.md` | 결과를 주제별로 모았지만, 근거는 위 둘에 흩어져 있다 |

그래서 "지금 노드가 왜 정수를 보내지?" 같은 질문 하나에 답하려면 세 문서를
오가며 시각순으로 읽어야 한다. 게다가 이 프로젝트는 **측정으로 결론이
뒤집힌 것이 다섯 건**이라, 앞부분만 읽으면 이미 폐기된 판단을 현재 결정으로
착각하기 쉽다.

ADR 은 같은 내용을 **주제순**으로 다시 자른다. 결정 하나에 파일 하나다.

---

## 상태 표기

| 상태 | 뜻 |
|---|---|
| **확정** | 현재 유효하다. 코드와 문서가 이 결정을 따른다 |
| **잠정** | 지금은 이렇게 하지만 근거가 약하다. 재측정 조건이 본문에 있다 |
| **대체됨** | 다른 ADR 이 이 결정을 뒤집었다. 헤더에 대체한 번호를 적는다 |

### 뒤집힌 결정을 다루는 규칙

**폐기된 결정으로 새 ADR 파일을 만들지 않는다.** 대신 그것을 대체한 ADR 의
「배경」 절에 경위를 넣는다.

이 프로젝트는 뒤집힌 판단이 다섯 건(컨텍스트 공유, 노드 상한 78 inf/s,
throttling 없음, `king` 19°C, 2.5GbE 로 충분)이라 파일로 따로 두면 목록의
절반이 폐기 문서가 된다. 읽는 사람이 유효한 결정을 찾기 어려워진다.

다만 **뒤집힌 경위 자체는 반드시 남긴다.** 이 프로젝트에서 가장 재사용
가치가 높은 산출물이 그 목록이기 때문이다(`docs/RESULTS.md` §6).

---

## 통합본

읽기·인쇄·공유용으로 전체를 하나로 묶은 **[ALL.md](ALL.md)** 가 있다.
**생성물이므로 직접 편집하지 않는다.** 원본을 고친 뒤 다시 만든다.

```bash
python scripts/build-adr-bundle.py $(git log -1 --format=%cs -- adrs/)
```

---

## 읽는 순서

처음 오는 사람은 이 순서를 권한다.

1. **[OVERVIEW.md](#overview)** — 시스템 전체 지도. ADR 을 읽기 전에 본다
2. **[001](#adr-001), [002](#adr-002), [003](#adr-003), [004](#adr-004)** — 프로젝트의 방향과 골격
3. **[007](#adr-007), [011](#adr-011), [012](#adr-012), [013](#adr-013)** — NPU 와 열을 실제로 다루며 나온 결정. 실측 밀도가 가장 높다
4. **[015](#adr-015), [017](#adr-017), [028](#adr-028)** — "성공처럼 보이는 실패" 를 막는 장치들
5. 나머지는 필요할 때 찾아 읽는다

### 시간이 없다면 셋만

| # | 왜 |
|---|---|
| [007](#adr-007) | 오류 0건에 결과 100% 불일치. 이 프로젝트의 성격을 가장 잘 보여준다 |
| [013](#adr-013) | 먼저 무너지는 것은 NPU 가 아니라 CPU 였다 |
| [002](#adr-002) | 왜 나쁜 수치를 그대로 내는가 |

---

## 목록

### 프로젝트 방향

| # | 제목 | 상태 |
|---|---|---|
| [001](#adr-001) | 모델을 쪼개지 않고 요청을 나눈다 (데이터 병렬) | 확정 |
| [002](#adr-002) | 성공 기준을 수치가 아니라 측정 가능성으로 둔다 | 확정 |
| [022](#adr-022) | 문서마다 규범 영역을 정하고 값이 다르면 규범 문서를 따른다 | 확정 |

### 시스템 구조

| # | 제목 | 상태 |
|---|---|---|
| [003](#adr-003) | 스케줄러를 하나만 두고 고가용성을 구현하지 않는다 | 확정 |
| [004](#adr-004) | 백엔드를 인터페이스로 분리하고 Mock 을 1급으로 둔다 | 확정 |
| [005](#adr-005) | RKNN 링크를 feature 뒤에 두고 기본값을 끈다 | 확정 |
| [006](#adr-006) | 크레이트를 7개로 나누고 `unsafe` 를 한 곳에 가둔다 | 확정 |
| [008](#adr-008) | 내부 통신을 gRPC(tonic + Protocol Buffers)로 한다 | 확정 |

### 스케줄링

| # | 제목 | 상태 |
|---|---|---|
| [009](#adr-009) | 정책은 세 개로 고정하고 후보 필터는 셋이 공유한다 | 확정 |
| [010](#adr-010) | ECT 점수식과 그 안의 각 항 | 확정 (실장비 검증 전) |
| [026](#adr-026) | 재시도는 반드시 다른 노드로, 백오프는 짧게 | 확정 |
| [027](#adr-027) | 노드 상태 머신과 drain·disable 분리 | 확정 (임계치 초안) |

### NPU 런타임

| # | 제목 | 상태 |
|---|---|---|
| [007](#adr-007) | 스레드마다 전용 RKNN 컨텍스트 — 공유를 타입으로 막는다 | 확정 |
| [011](#adr-011) | 기준 모델을 INT8 로 한다 | 확정 |
| [012](#adr-012) | 노드는 역양자화하지 않고 정수를 보낸다 (`want_float=0`, blob v2) | 확정 |
| [020](#adr-020) | `worker_count = 8`, `core_mask` 미설정 | 확정 |
| [021](#adr-021) | 노드는 후처리(NMS)를 하지 않는다 | **잠정** |

### 하드웨어와 측정 환경

| # | 제목 | 상태 |
|---|---|---|
| [013](#adr-013) | 팬리스를 기본으로 두고 throttling 을 측정 대상으로 삼는다 | 확정 |
| [014](#adr-014) | aggregation 만 10G, 스케줄러는 별도 서버 | 확정 (구축·실측 완료) |
| [018](#adr-018) | 모델은 한 번만 변환해 세 노드에 배포한다 | 확정 |
| [019](#adr-019) | 보드는 IP 가 아니라 SSH 별칭으로 접근한다 | 확정 |
| [023](#adr-023) | CPU governor 를 `performance` 로 — 단 근거의 범위를 명시 | **잠정** |

### 측정 규율

| # | 제목 | 상태 |
|---|---|---|
| [015](#adr-015) | 측정 전 preflight 하드 실패 검사 | 확정 |
| [016](#adr-016) | `boot_id` 로 측정 중 재부팅을 감지해 run 을 무효화한다 | 확정 |
| [017](#adr-017) | 원격 실행 함정을 라이브러리 함수로 굳힌다 | 확정 |
| [028](#adr-028) | 벤치 도구가 run 유효성을 스스로 판정한다 | 확정 |

### 프로토콜과 정책 세부

| # | 제목 | 상태 |
|---|---|---|
| [024](#adr-024) | 오류를 `NPF-xxxx` 코드 체계로 고정한다 | 확정 |
| [025](#adr-025) | 하트비트 실패는 곧바로 재등록 — 등록은 멱등 | 확정 |

---

## 실패에서 나온 ADR

이 프로젝트는 **같은 유형의 실수를 네 번** 했다. 전부 "지표가 무엇을 세는지
확인하지 않고 이름만 보고 믿은" 것이다.

```text
1. run_duration 을 NPU 점유시간으로 읽음        → 큐 대기가 포함된 값
2. NPU load 를 delayms=3000 인 채로 샘플링      → 3초 평균을 읽고 있었음
3. thread-safety 를 API 반환 코드로만 판정      → 결과 내용 미대조   → ADR-007
4. throttling 을 NPU 클럭만으로 판정            → CPU 가 꺾이고 있었음 → ADR-013
```

여기서 나온 장치들이 따로 있다.

| ADR | 막는 것 |
|---|---|
| [015](#adr-015) | 전제가 틀린 채로 측정을 시작하는 것 |
| [016](#adr-016) | 재부팅을 "성능 저하" 로 읽는 것 |
| [017](#adr-017) | 원격 명령이 실패했는데 종료 코드 0 인 것 |
| [019](#adr-019) | 엉뚱한 보드의 결과를 다른 노드에 귀속시키는 것 |
| [028](#adr-028) | 무효한 run 의 숫자가 결과 표로 넘어가는 것 |

---

## 새 ADR 을 쓸 때

1. [TEMPLATE.md](#template) 를 복사한다
2. 파일 이름은 `NNN-ascii-kebab-slug.md`. 번호는 **029 부터** 이어 붙인다
3. 번호는 **재사용하지 않는다.** 폐기된 결정도 번호를 그대로 유지한다
4. 이 README 의 목록과 상태를 같이 갱신한다

### 쓸 때 지키는 것

- **수치에는 측정 조건을 반드시 붙인다.** 노드, 스레드 수, 지속 시간,
  governor, 모델. 조건 없는 숫자는 3개월 뒤에 쓸모가 없다는 것을 이미 겪었다
- **버린 대안을 적는다.** 무엇을 골랐는지보다 무엇을 왜 버렸는지가 오래 간다
- **모르는 것은 모른다고 적는다.** 「잠정」 상태와 「뒤집힌다면」 절이 그 자리다
- **재검증 방법에 "무엇을 보면 안 되는지" 를 적는다.** 틀린 지표로 통과
  판정을 낸 적이 네 번 있다

---

<a id="overview"></a>

# NPUDure 아키텍처 개요

ADR 을 읽기 전에 보는 문서다. **시스템 전체가 어떻게 생겼는지** 한 번에
훑는 것이 목적이고, 개별 선택의 근거는 각 ADR 로 넘긴다.

---

## 1. 한 문장

저렴한 엣지 NPU 보드 세 대에 추론 요청을 나눠 던지고,
**세 대가 정말 세 배가 되는지 측정하는** Rust 런타임이다.

## 2. 무엇을 하고 무엇을 안 하나

```text
하는 것                          하지 않는 것
─────────────────────────       ─────────────────────────
독립 요청을 여러 노드에 분산     모델 하나를 여러 노드에 쪼개기
노드 부하를 보고 고르기          단일 요청을 3배 빠르게
죽은 노드 빼고 살아나면 넣기     Kubernetes 급 범용 오케스트레이션
단계별로 시간 쪼개서 재기        NPU 세 개를 한 개처럼 보이게
```

오른쪽 열이 **명시적 비목표**다. 특히 첫 두 개는 "그럼 그건 왜 안 하냐"는
질문을 계속 받는 항목이라 [ADR-001](#adr-001) 에 따로
근거를 적어 두었다.

핵심만 말하면 이렇다. **이 시스템은 요청 하나를 빠르게 만들지 못한다.**
요청이 많을 때 전체를 많이 처리하게 만들 뿐이다.

## 3. 세 층

```text
┌─────────────────────────────────────────────────────────┐
│  Client                                                 │
│  벤치마크 CLI · 데모 웹 · 직접 호출하는 API 클라이언트  │
└───────────────────────────┬─────────────────────────────┘
                            │  gRPC : Infer(model, image)
                            ▼
┌─────────────────────────────────────────────────────────┐
│  Scheduler   (보드가 아닌 별도 호스트에서 돈다)         │
│                                                         │
│   Node Registry   누가 살아 있나                        │
│   Scheduler       이번 요청은 누구에게                  │
│   Retry Manager   실패하면 다른 노드로                  │
│   Health Monitor  하트비트가 끊기면 후보에서 뺀다       │
└──────────┬──────────────┬──────────────┬────────────────┘
           │              │              │  gRPC
           ▼              ▼              ▼
    ┌───────────┐  ┌───────────┐  ┌───────────┐
    │  king     │  │  queen    │  │  jack     │
    │  RK3576   │  │  RK3576   │  │  RK3576   │
    │  NPU 6TOPS│  │  NPU 6TOPS│  │  NPU 6TOPS│
    └───────────┘  └───────────┘  └───────────┘
      각 노드가 같은 모델 전체를 갖고 있다
```

**세 노드는 완전히 대등하다.** 같은 바이너리, 같은 모델 파일, 같은 설정을
쓰고 `[node]` 섹션의 `id` 와 주소만 다르다. 노드 사이에는 통신이 없다 —
서로의 존재조차 모른다.

스케줄러를 보드에서 돌리지 않는 이유는, 한 노드에만 스케줄러 부하가 실리면
1/2/3노드 비교가 그 순간 오염되기 때문이다.

## 4. 요청 하나의 일생

```text
 ① Client ──────────────► Scheduler      이미지 1장 (640×640×3 = 1.23 MB)
 ② Scheduler                             후보 노드 추리기 (죽은 노드 제외)
 ③ Scheduler                             정책 실행 → 노드 선택
 ④ Scheduler ───────────► Node           선택된 노드로 전달
 ⑤ Node                                  로컬 큐에 넣기
 ⑥ Node                                  워커가 집어서 전처리
 ⑦ Node                                  NPU 추론  ← 여기만 NPU, 나머지는 CPU
 ⑧ Node ───────────────► Scheduler       원시 텐서 9개를 blob 하나로
 ⑨ Scheduler ──────────► Client          결과 + 단계별 소요시간
```

각 화살표와 각 상자에서 걸린 시간을 **따로따로 기록**한다.

```rust
scheduler_queue_us   scheduler_route_us   network_to_node_us
node_queue_us        decode_us            preprocess_us
npu_input_us         inference_us         postprocess_us
network_to_client_us end_to_end_us
```

이 분해가 프로젝트의 존재 이유에 가깝다. "3노드가 2.4배밖에 안 나왔다"는
답이 아니고, **어느 칸에서 새는지**가 답이다.

> ⑦ 이 NPU 구간이고 나머지는 전부 CPU 다. 그리고 실측에서 **먼저 무너지는
> 쪽은 NPU 가 아니라 CPU** 였다 — 지속 부하 300초에 처리량이 -27% 떨어지는데
> NPU 클럭은 950MHz 로 고정이고 CPU 가 A72 2208 → 816MHz 로 강등된다.

### 실패하면

```text
④ 에서 실패 ─► 원인 분류 ─► 재시도 가능? ─► 그 노드를 후보에서 빼고
                                              ─► 다른 노드로 재시도
                                              ─► 다 실패하면 NPF-1302
```

같은 노드로 다시 던지지 않는다. 방금 실패한 노드는 다음 시도에서도 실패할
가능성이 높다.

## 5. 크레이트 지도

```text
                    npuforge-common
                    타입 · 오류코드 · 설정 · InferenceBackend 인터페이스
                            │ 전부가 이걸 참조
        ┌───────────────────┼───────────────────┐
        │                   │                   │
 npuforge-scheduler   npuforge-node      npuforge-bench
 정책 3종 · 레지스트리  워커풀 · 큐 · 등록   부하 생성 · 집계
        │                   │
        └──── npuforge-proto ┘          gRPC 정의 (.proto → tonic)
                            │
                 ┌──────────┴──────────┐
                 │                     │
        npuforge-rknn          npuforge-mock-backend
        실제 NPU. unsafe 는     하드웨어 없이 도는 가짜 백엔드
        전부 여기에만 있다      결정적 시드 · 지연/오류율 주입
```

| 크레이트 | 한 줄 |
|---|---|
| `npuforge-common` | 모두가 공유하는 타입과 인터페이스. 여기가 계약서다 |
| `npuforge-proto` | gRPC 서비스 정의 |
| `npuforge-scheduler` | 어느 노드로 보낼지 정하고, 실패하면 다시 보낸다 |
| `npuforge-node` | 보드 위에서 도는 에이전트. 큐 + 워커 풀 |
| `npuforge-rknn` | RKNN Runtime FFI. **`unsafe` 격리 구역** |
| `npuforge-mock-backend` | NPU 흉내. 하드웨어 없이 전체를 돌리기 위한 것 |
| `npuforge-bench` | 부하 걸고 통계 내고 **이 run 이 유효한지 판정**한다 |

두 백엔드는 같은 `InferenceBackend` 인터페이스를 구현한다. 그래서
**RK3576 보드가 한 대도 없어도** `cargo test --workspace` 가 통과하고 3노드
클러스터가 로컬에서 돈다. 이건 편의 기능이 아니라 설계 원칙이다.

## 6. 물리 구성

```text
현재 (측정 불가)                    계획 (M3)

 관리망 1GbE                          Scheduler 서버
   ├── king                              │ 10GbE  ← aggregation
   ├── queen                             │
   ├── jack                        2.5G/10G 스위치
   └── dealer (스케줄러, 노트북)      ├─2.5G─ king
                                       ├─2.5G─ queen
 추론망 없음. 스위치 미구매            └─2.5G─ jack
```

**worker 링크는 2.5G 로 충분한데 aggregation 만 10G 가 필요하다.** 세 노드의
트래픽이 한 점에서 합쳐지기 때문이다. 지금 측정하면 NPU 확장 효율이 아니라
링크 포화를 재게 되므로 M3 는 시작하지 않은 상태다.

현재 스케줄러 호스트인 `dealer` 는 노트북이라 PCIe 슬롯이 없어 10G NIC 을
꽂을 수 없다. 별도 서버가 필요하다.

## 7. 지금 어디까지 되어 있나

| | 상태 |
|---|---|
| 소프트웨어 골격 | ✅ 209 tests, clippy `-D warnings`, fmt clean |
| 단일 노드 실측 | ✅ INT8 157.2 inf/s / FP16 84.3 inf/s (8스레드, 120초) |
| Mock 3노드 클러스터 | ✅ 실제 gRPC 로 붙는다. 하드웨어 없이 |
| 실장비 3노드 | ⬜ **네트워크 장비 대기로 중단** |
| Prometheus · 대시보드 | ⬜ |

막힌 이유는 코드가 아니라 장비다. 자세한 재개 절차는 `docs/TODO.md` 최상단.

## 8. 더 읽을 곳

| 궁금한 것 | 갈 곳 |
|---|---|
| **결정 전체 목록** | **[README.md](#index)** |
| 왜 모델을 쪼개지 않나 | [ADR-001](#adr-001) |
| 왜 스케줄러가 하나뿐인가 | [ADR-003](#adr-003) |
| 왜 하드웨어 없이 전부 돌아가나 | [ADR-004](#adr-004) |
| 왜 NPU 컨텍스트를 스레드마다 만드나 | [ADR-007](#adr-007) |
| 왜 INT8 인가 | [ADR-011](#adr-011) |
| 왜 노드가 float 이 아니라 정수를 보내나 | [ADR-012](#adr-012) |
| 왜 팬을 안 다나 | [ADR-013](#adr-013) |
| 왜 지금 측정을 안 하고 기다리나 | [ADR-014](#adr-014) |
| 왜 측정 전에 preflight 를 돌리나 | [ADR-015](#adr-015) |
| 무엇이 어떻게 동작하나 (전체 명세) | `docs/01-TECHSPEC.md` |
| 어떤 수치가 나왔나 | `docs/RESULTS.md` |
| 지금 뭘 해야 하나 | `docs/TODO.md` |
| 값의 최종 기준 | `docs/environment-matrix.md` |

---

<a id="adr-001"></a>

# ADR-001. Split requests, not the model (data parallelism)

*[한국어 원문](001-data-parallel-only.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-012](#adr-012), `docs/00-PRD.md` §4, `docs/01-TECHSPEC.md` §2.1 |

---

## In one line

> Three nodes each hold **the entire same model** and handle **different
> requests**. Splitting the model layer-wise across nodes is not done in v0.1.

## Context

There are broadly two ways to use several NPUs at once.

### Approach A. Split the model (model parallelism / layer partitioning)

```text
one request --> [node1: layers 1-10] --intermediate tensor--> [node2: layers 11-20] --> result
```

Node 1 computes the model's front section and node 2 the back. **Intermediate
results (feature maps)** travel between the nodes. LLM tensor parallelism and
pipeline parallelism are of this family.

- Advantage: the processing time of **a single** request can fall. A model too
  large for one node can be run
- Disadvantage: inter-node communication lands **inside the inference path**.
  If one node is slow, everything waits

### Approach B. Split the requests (data parallelism)

```text
request A --> [node1: whole model] --> result A
request B --> [node2: whole model] --> result B
request C --> [node3: whole model] --> result C
```

Each node holds the whole model and takes a different request end to end.

- Advantage: there is **no** inter-node communication. If one dies, the rest
  keep running
- Disadvantage: **a single** request never gets faster

Everything about the system follows from this choice — the scheduler's role,
failure handling, network requirements, and even what gets measured.

## Decision

**Implement approach B (data parallelism) only.** Approach A is an explicit
non-goal for v0.1.

The following become non-goals along with it.

- Splitting one large model layer-wise across several nodes
- LLM tensor parallelism / pipeline parallelism
- Hardware-level integration making several NPUs appear as one physical NPU
- Reducing a single inference request's latency in proportion to node count

## Rationale

### 1. The goal is throughput, not latency

The question this project sets out to answer is **"do three 6 TOPS units really
make 18 TOPS?"** That asks how much gets processed in total when requests pile
up, not how quickly one is finished.

The assumed usage is the same. Multiple cameras, multiple requests —
**independent requests arrive in bunches to begin with.** Data parallelism is
the natural shape for that load, and splitting the model would be a net loss.

### 2. On this hardware, partitioning cannot afford the communication

One input is already large.

```text
raw RGB 640 x 640 x 3 = 1,228,800 byte
```

**The input alone** comes to 4.64 Gbps at three-node saturation (at INT8's
157.2 inf/s). That is already why the aggregation link needs 10G.

Layer partitioning **adds intermediate-tensor round trips inside the inference
path** on top of that. Each partition point adds one inter-node transfer, and on
2.5GbE a 1 MB-class tensor computes to somewhere near 4 ms. Against a **total**
INT8 inference of 50.8 ms, a few partition points erase the benefit.

> That 4 ms is a figure divided by link speed, not a measurement. But it was
> judged not to be a difference worth measuring to confirm — the benefit
> partitioning would bring is not a goal in the first place.

### 3. This model has no reason to be split

Partitioning is **forced** when the model does not fit on one node.

```text
node RAM              4 GB
YOLOv8n INT8 model    6.46 MB
YOLOv8n FP16 model    9.65 MB
```

Three orders of magnitude apart. There is no need to split anything.

### 4. The measurements have to be interpretable

This project's output is **"where does it leak?"** With nodes independent of
one another, when scaling efficiency comes up short the cause can be cleanly
divided into scheduling, network and node-internal.

Adding layer partitioning creates inter-node dependency, so when three nodes
yield only 2.4×, separating whether that is partition-point communication,
scheduling or the NPU becomes hard. **A project whose purpose is measurement
must not choose a structure in which causes cannot be decomposed.**

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Layer-wise partitioning | Communication cost enters the inference path. This model fits comfortably in 4 GB, so there is no reason to split |
| LLM tensor parallelism | The target model is a CNN detector. There is nothing to apply it to |
| Hardware-level NPU integration | Not something achievable on top of the RKNN Runtime. Driver and SoC-level work |
| Supporting **both** data parallelism and partitioning | Neither could be measured properly within v0.1. Doing both halfway makes both sets of figures unusable |

## Consequences

**Gained**

- No communication between nodes. The nodes do not know the others exist
- Failure handling becomes simple — drop the dead node from the candidates and
  that is it. Work in progress on other nodes is unaffected
- Measuring one node's performance ceiling predicts the cluster's ceiling.
  **This is why so much effort went into single-node measurement**
- The scheduler only has to decide "one request → one node"

**Lost / the cost**

- **Single-request latency never falls with more nodes.** 50.8 ms for one INT8
  inference is 50.8 ms on three nodes too. That is design, not a bug
- A model that does not fit on one node cannot be run
- Each node needs its own copy of the model (not a problem at this scale)

**New constraints introduced**

- Talks and documents **must not say "3× faster".** "Processes 3× as much" is
  correct. Mixing the two makes an audience expect a latency reduction
- Benchmark scenarios must use **concurrent request** load. Measuring by
  throwing one request at a time is meaningless in this structure

## What would overturn this

Revisit if any of the following holds.

- **When the target model does not fit in node memory.** If a model above 4 GB
  has to run, partitioning stops being a choice and becomes forced
- **When single-request latency becomes a requirement.** Not now, but if, say,
  frame-level real-time control became the goal, the premise changes
- **When the inter-node link becomes fast enough to carry inference-internal
  communication.** Though that changes this project's own premise of 2.5GbE-class
  edge boards

All three are outside v0.1's scope. Even a re-examination should come **after
v0.1's data-parallel measurements are finished** — without a comparison
baseline there is no way to judge whether partitioning is a gain.

---

<a id="adr-002"></a>

# ADR-002. Define success as "can it be measured and explained", not "did the number come out"

*[한국어 원문](002-success-criteria-measurability.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-05 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-001](#adr-001), [ADR-015](#adr-015), [ADR-028](#adr-028), `docs/00-PRD.md` §3 |

---

## In one line

> **No result value is set as a success condition**, such as "2.5× or better at
> three nodes". Even if scaling efficiency comes out low, even if io_uring has
> no effect, it is a success as long as the cause can be explained
> quantitatively.

## Context

What happens when a measurement project sets a target number as its success
criterion.

```text
goal: "3-node scaling efficiency of 80% or better"

measured 65%  ->  has to be recorded as a failure
              ->  nobody wants to fail
              ->  favourable conditions start getting found
                 measure briefly . smaller input . preheat well . keep only the good runs
```

**This is not something only dishonest people do.** Given freedom in choosing
conditions and a target hanging over you, the favourable option gets picked
unconsciously. And each of those choices can be given a plausible reason.

This project has unusually large freedom in choosing conditions. Governor,
thread count, duration, cooling, input size, model — all of them move the
numbers. On the same board, the governor alone moved 7% and duration alone
moved 27%.

## Decision

**Success is defined as the following.**

1. Was it measured
2. Were the measurement conditions recorded with it
3. Can the cause of the result be explained
4. Is it reproducible

**The following results are explicitly counted as valid outcomes.**

- io_uring producing no meaningful performance improvement
- Zero-copy applying to only a limited scope
- The NPU or preprocessing, rather than the network, being confirmed as the
  primary bottleneck
- Three-node scaling efficiency being lower than expected
- A single high-performance device being more favourable on cost

## Rationale

### It actually helped

Results that would have been discarded without this criterion became the central
output instead.

| Result | Under a target criterion | What actually happened |
|---|---|---|
| Three application-level optimizations at +0.1 / +5.4 / −1.8% | Failure. Bury it and try something else | Became **the basis for "there is nothing left to squeeze inside the node"** |
| Zero-copy at −1.8% | Failure | Hypothesis refuted. Led to the discovery that 76 ioctls are intrinsic to inference submission |
| −27% under fanless sustained load | A bad number | **The peak vs sustained gap** — a value absent from vendor spec sheets. Became the central narrative of the talk |

The third is decisive. Had the goal been "high throughput", we would have
attached a fan, measured for 120 seconds and reported 84.3 inf/s. That figure
**does not reproduce in the field.**

### It becomes possible to publish inverted conclusions

Measurement inverted this project's conclusions five times. With a target number
hanging over it, inverting is itself a loss — the already-reported number
becomes void.

With "can it be explained" as the criterion, **inverting becomes an outcome
instead.** That is why `docs/RESULTS.md` §4 "Inverted conclusions" and §6 "List
of measurement failures" can exist.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Set a target number (e.g. 80% scaling efficiency) | Creates condition-selection bias. The most dangerous thing in a measurement project |
| Target number + "state the reason if missed" | The reason section becomes a paragraph of excuses. The same problem remains the moment a miss is defined as failure |
| Set no criterion at all | There is no way to know when it is finished. Measurement goes on indefinitely |

## Consequences

**Gained**

- Unfavourable results can be published as they are
- Failure cases become output — with more reuse value than the numbers
- There is no longer any reason to hide measurement conditions

**Lost / the cost**

- **"So how many times faster is it?" is hard to answer in one line.** A
  disadvantage in a talk. The conditions have to be said alongside, so the
  sentence gets longer
- The success/failure verdict can look subjective. Hence the four explicit
  conditions above

**New constraints introduced**

- **Every number has to carry its measurement conditions.** A number without
  conditions is void under this criterion. Nodes, threads, duration, governor
  and model are always written alongside
- Invalid runs must not be used as though valid → enforced by tooling
  ([ADR-028](#adr-028))

## What would overturn this

If this project becomes **a product rather than an experimental tool**, the
criterion changes. A product needs a line of "it has to reach at least this to
be usable".

v0.1's purpose is measurement, so this criterion stands.

---

<a id="adr-003"></a>

# ADR-003. One scheduler, and no high availability

*[한국어 원문](003-central-simple-scheduler.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-001](#adr-001), [ADR-014](#adr-014), `docs/01-TECHSPEC.md` §2.3 |

---

## In one line

> **A single central scheduler** decides which node a request goes to. No
> distributed consensus, no leader election, no scheduler redundancy is built.
> Instead, **the cost of the scheduler dying and coming back is made cheap.**

## Context

There are broadly four structures for spreading requests across nodes.

| Approach | Who decides |
|---|---|
| **Central scheduler** | one machine in the middle decides everything |
| Client-side distribution | the client picks the node itself (no scheduler) |
| P2P / gossip | nodes exchange state and decide among themselves |
| A general-purpose orchestrator | hand it to an off-the-shelf system like Kubernetes |

Further down the list, the single point of failure disappears and things hold up
at larger scale. In exchange, implementation and operation get heavier.

## Decision

**Use a single central scheduler.** And in v0.1, **do not implement** any of the
following.

- Distributed consensus (Raft and the like)
- Leader election
- Multi-scheduler high availability
- Kubernetes-level general-purpose orchestration

**The scheduler is a single point of failure.** This is written into the
documents not as a defect but as **a constraint accepted knowingly.**

## Rationale

### 1. What is being measured is the scheduling policy itself

This project runs an experiment (S3) that **swaps between three policies** —
Round Robin / Least Queue / ECT — and compares them. For that, **the point where
the decision is made has to be one place.**

If distribution scatters to clients or nodes, the very notion of "this run's
distribution policy" gets blurred. A policy comparison would end up measuring
differences in implementation location rather than policy.

### 2. ECT can only be computed with global state

The default policy, ECT, picks a candidate like this.

```text
ECT = ((queue_depth + in_flight + 1) x EWMA_inference
       + EWMA_network + thermal_penalty + error_penalty) / load_factor
```

The values that go in — each node's queue depth, in-flight count, moving average
of inference time, temperature — only compare if **all nodes are visible at
once**. A node deciding from its own state alone cannot satisfy this formula.

### 3. There are three nodes

The scale at which a consensus protocol or gossip earns its keep is tens to
hundreds of nodes. At three, implementation and debugging cost more than they
return.

### 4. The time budget

The goal is **finishing the measurements** within the period leading up to the
talk. Time spent implementing consensus is time not spent measuring what
actually needs measuring. What is decided against matters as much as what is
decided for.

## How the single point of failure is handled

Instead of eliminating it, **recovery is made cheap.**

- When a heartbeat fails, the node **switches immediately to re-registration**
- Registration is **idempotent**. Doing it repeatedly causes no problem
- So killing the scheduler and bringing it back has **all three nodes return by
  themselves within about 1.3 seconds** (verified with four real processes)

From the node's perspective, a transient network error and a scheduler restart
are indistinguishable. So it **unconditionally takes the more expensive option
(re-registration)**. That choice is available because registration is idempotent,
so wasted effort does not translate into loss. (→ ADR-025)

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Client-side distribution | The policy comparison experiment does not hold. There is also no way for a client to see global state |
| P2P / gossip | No benefit at three nodes. It introduces inter-node communication, breaking [ADR-001](#adr-001)'s premise that nodes do not know each other |
| Kubernetes | An explicit non-goal. Container orchestration is unrelated to the question this project asks and only adds noise to measurement |
| Two schedulers + leader election | Large implementation and verification cost. That time is time not spent measuring. The availability gained at three-node scale does not justify it |

## Consequences

**Gained**

- The three policies can be swapped in the same place → the S3 experiment became
  possible
- Retries, state machines and health checks are all in one process, making them
  easy to trace
- Scheduler restart recovery in 1.3 seconds

**Lost / the cost**

- **If the scheduler dies the whole cluster stops.** The nodes are alive but
  there is no path for requests to reach them
- The scheduler itself is part of the throughput ceiling. However many nodes are
  added, if the scheduler cannot keep up it stops there

**New constraints introduced**

- **The scheduler host became part of the measurement conditions.** Where it
  runs changes the numbers. That is why official benchmarks run it on a separate
  host rather than a board
  (→ [ADR-014](#adr-014))
- The scheduler host's resources become an experimental constraint. `dealer`
  currently has 3 GB of RAM, which could fall short once a 1.17 MiB payload ×
  concurrent count piles up. **Not yet observed**

## What would overturn this

- **When there are tens of nodes.** This decision presumes three
- **When the scheduler is actually measured as the bottleneck.** The basis for
  that judgement is already prepared — check whether `TimingBreakdown`'s
  `scheduler_queue_us` / `scheduler_route_us` occupy a meaningful share of
  `end_to_end_us`. **Do not guess; read that field**
- **When availability becomes a requirement.** This is experimental equipment
  today, and if the scheduler dies a person restarts it. Becoming an operational
  system changes the premise

---

<a id="adr-004"></a>

# ADR-004. Separate the backend behind an interface, with Mock as a first-class backend

*[한국어 원문](004-backend-abstraction-mock-first.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 (original), 2026-08-19 (written up as an ADR) |
| **Related** | [ADR-005](#adr-005) (feature gate), [ADR-007](#adr-007), `docs/03-DEVELOPMENT-REQUIREMENTS.md` §4.1 |

---

## In one line

> Push NPU calls behind an `InferenceBackend` interface and make **a fake
> backend that slots into that place a proper implementation**. The whole system
> runs without a single RK3576 board. **This is a design principle, not a
> convenience feature.**

## Context

This project's development environment looks like this.

- The three boards sit on a desk and are not always powered on
- The development PC is **Windows/x86**. The RKNN Runtime is ARM64 Linux only
- CI runs on GitHub Actions. There is obviously no NPU there

Developing with no provision for this leads to: **code can only be written when
a board is on, tests only run when a board is on, and CI verifies nothing.**

But look closely and **the part of this system that actually needs an NPU is
very narrow.**

```text
three scheduling policies       NPU-independent
node registry, state machine    NPU-independent
retries, timeouts               NPU-independent
queues, worker pool             NPU-independent
gRPC wiring                     NPU-independent
health checks, drain            NPU-independent
────────────────────────────────────────────
one actual inference            <- only here is the NPU
```

## Decision

**1. Hide inference behind an interface.**

```rust
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn load_model(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>>;
    fn backend_name(&self) -> &'static str;      // "rknn" or "mock"
    fn runtime_version(&self) -> Result<String>;
}

#[async_trait]
pub trait LoadedModel: Send + Sync {
    async fn infer(&self, input: InferenceInput) -> Result<InferenceOutput>;
    fn model_info(&self) -> &LoadedModelInfo;
}
```

The scheduler and node agent know only this interface. They never call
`npuforge-rknn` directly.

**2. Make the Mock backend a proper backend, not a test helper.**

It is chosen in the configuration file. It is not a stub hidden inside test
code.

```toml
[backend]
type = "mock"          # or "rknn"
base_latency_ms = 20
jitter_ms = 5
error_rate = 0.02
```

**3. Put fault injection in the Mock.** On top of a deterministic seed it can
produce latency, latency variance, error rates and per-node speed differences.
The three nodes in `configs/mock/` **deliberately have different speeds and
error rates**.

**4. Set the verification bar at "passes without hardware."**
`cargo test --workspace` has to pass on Windows/x86.

## Rationale

### 1. Policy comparison has to show up in Mock first

If the difference between Round Robin and ECT can only be seen on real
hardware, every policy change means powering on boards, deploying and
measuring. The iteration cycle becomes minutes.

This is why the three nodes in `configs/mock/` have different speeds. **If the
speeds were equal, Least Queue and Round Robin would give the same answer.** The
conditions were made deliberately asymmetric so that policy differences surface
locally.

### 2. It can produce conditions that are hard to create on real hardware

"A node fails 2% of the time", "one node is 3× slower", "a node dies mid-request"
— reproducing these with real boards is cumbersome and poorly reproducible. With
a fixed seed the Mock produces them **in the same order every time.**

### 3. The transport path is real

The Mock 3-node integration test
(`crates/npuforge-scheduler/tests/mock_cluster.rs`) **runs over real gRPC.** It
is one process, but the wiring is the same as on real hardware.

| Verified | Result |
|---|---|
| Requests spread across 3 nodes | ✅ round-robin uses all three |
| Bypass when 1 node dies | ✅ 6/6 succeeded |
| All nodes dead | ✅ `NPF-1302` plus the list of nodes attempted |
| Timing breakdown | ✅ both node and scheduler sections populated |
| Avoiding a slow node | ✅ least-queue uses the fast nodes more |

### 4. CI actually verifies something

209 tests run without hardware. Without this, CI is decoration that only checks
that it compiles.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Keep only a `#[cfg(test)]` stub | It lives only inside tests. Bringing up a 3-node cluster and poking at it by hand becomes impossible |
| Require real hardware | Development stops when the boards are off. CI becomes meaningless. Contributors would have to buy a board to participate |
| Use the RKNN simulator | It cannot infer with a built `.rknn` — after `load_rknn`, `init_runtime` refuses. This was actually attempted and did not work |
| No interface, branch with conditional compilation | `#[cfg]` spreads through every call site and the two paths silently diverge |

## Consequences

**Gained**

- 209 tests pass on Windows/x86
- A 3-node cluster can be brought up locally and actually operated
- `unsafe` is confined to one place, `npuforge-rknn` (→ ADR-006)
- Contributors can participate without a board — important for an open-source
  project

**Lost / the cost**

- The cost of maintaining the interface. Every backend has to honour the same
  contract
- The risk of the two implementations diverging. Metadata such as
  `runtime_version` is meaningless in the Mock, creating places filled in for
  form only

**⚠️ New constraint introduced — the Mock is not omnipotent**

This is the most important sentence in this ADR.

**The Mock only imitates what passes through the interface.** It will never
catch a defect specific to RKNN. In fact,
[ADR-007](#adr-007)'s shared-context problem — 0 errors and
100% result mismatch — cannot reproduce in the Mock at all, because the Mock has
no concept of a context.

That is why **real-hardware integration tests have to exist separately.** The
six in `crates/npuforge-rknn/tests/real_device.rs` occupy that place.

```text
What the Mock guards           What only real hardware can guard
────────────────────           ─────────────────────────────────
policies, retries, state       RKNN concurrency contract
queues, timeouts               dequantization accuracy
gRPC wiring                    actual throughput and thermal behaviour
failure bypass paths           output tensor shapes
```

**Never conclude "the Mock tests passed, so we are fine."**

## What would overturn this

- **If cases of Mock and real hardware diverging accumulate.** At that point a
  choice is needed between raising the Mock's fidelity and narrowing it to
  policy verification only
- **If there are three or more backends**, the interface needs re-examination.
  Two is a minimal sample and it is hard to be confident the abstraction is right

---

<a id="adr-005"></a>

# ADR-005. RKNN 링크를 feature 뒤에 두고 기본값을 끈다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 |
| **관련** | [ADR-004](#adr-004), [ADR-006](#adr-006) |

---

## 한 줄 요약

> `npuforge-rknn` 은 workspace 멤버지만 **기본 빌드에서 RKNN 을 링크하지
> 않는다.** 그래서 Windows/x86 개발 PC 와 CI 에서 `cargo build --workspace`
> 가 통과한다. 실장비 빌드만 `--features rknn` 을 켠다.

## 배경

RKNN Runtime(`librknnrt.so`)은 **ARM64 Linux 전용 공유 라이브러리**다.
Rockchip 이 배포하고, 이 저장소에는 포함되지 않는다.

이 크레이트를 그냥 workspace 에 넣으면 이렇게 된다.

```text
Windows 개발 PC 에서 cargo build --workspace
  → npuforge-rknn 이 librknnrt.so 를 찾는다
  → 없다
  → workspace 전체 빌드 실패
  → 아무 코드도 못 짠다
```

[ADR-004](#adr-004) 에서 "하드웨어 없이 전체가
돌아야 한다" 를 원칙으로 정했는데, 링크 단계에서 그것이 깨진다.

## 결정

**1. `rknn` feature 를 만들고 기본값을 비운다.**

```toml
[features]
default = []
rknn = []
```

**2. 실장비 빌드만 명시적으로 켠다.**

```bash
cargo build --release --target aarch64-unknown-linux-gnu \
      -p npuforge-node --features rknn
```

저장소에는 `cargo build-node` 별칭으로 등록해 두었다.

**3. feature 가 꺼진 빌드에서도 타입은 존재한다.** 자리표시자 구현이
컴파일되고, 추론을 시도하면 명확한 오류를 낸다.

```rust
async fn infer(&self, _input: InferenceInput) -> Result<InferenceOutput> {
    Err(NpuForgeError::new(
        ErrorCode::BackendError,
        "RKNN 지원 없이 빌드된 바이너리입니다",
    ))
}
```

**4. 빌드와 설정이 어긋나면 시작 시점에 죽는다.**

```rust
pub const fn is_rknn_enabled() -> bool { cfg!(feature = "rknn") }
```

노드 에이전트가 기동 시 이 값을 확인한다. `[backend] type = "rknn"` 설정을
RKNN 없이 빌드한 바이너리에 주면 **첫 요청을 받기 전에** 멈춘다.

## 근거

### 실수 하나를 구조적으로 막는다

**Mock 전용 바이너리를 실제 노드에 배포하는 사고**가 가장 무섭다. 그대로
돌면 노드가 가짜 결과를 내는데, 처리량은 오히려 좋게 나온다. Mock 은
NPU 를 안 쓰니까.

`is_rknn_enabled()` 검사가 없으면 이 사고는 **벤치마크 결과가 다 나온 뒤에야**
발견된다. 기동 시점에 죽으면 즉시 알 수 있다.

이 프로젝트에서 이미 겪은 유형이다 — 컨텍스트 공유도, 원격 실행 실패도
"성공처럼 보이는 실패" 였다.

### `publish = false` 도 같은 이유

`librknnrt.so` 가 없는 환경에서 실수로 발행되지 않도록 막아 두었다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 기본값을 켠다 (`default = ["rknn"]`) | Windows/CI 빌드가 깨진다. ADR-004 의 전제가 무너진다 |
| `npuforge-rknn` 을 workspace 에서 뺀다 | 별도 빌드가 되어 CI 가 이 크레이트의 컴파일조차 확인하지 않는다 |
| `#[cfg(target_arch)]` 로 자동 판별 | aarch64 Linux 라고 해서 RKNN SDK 가 있다는 보장이 없다. 빌드 환경과 실행 환경이 다를 수도 있다 |
| 런타임 `dlopen` 으로 동적 로드 | 링크 문제는 풀리지만 FFI 시그니처 검증을 컴파일 타임에 못 한다. 실기 헤더 대조로 이미 잡은 위험을 다시 열게 된다 |

## 결과

**얻은 것**

- `cargo test --workspace` 가 Windows/x86 에서 통과한다 (209 tests)
- CI 가 RKNN SDK 없이 fmt·clippy·test·aarch64 크로스까지 돈다
- Mock 바이너리 오배포가 기동 시점에 잡힌다

**잃은 것 / 대가**

- **`--features rknn` 경로는 CI 에서 실행 검증을 못 한다.** 크로스 컴파일은
  하지만 돌려 보지는 못한다. 그래서 실장비 통합 테스트가 따로 필요하다
  (`crates/npuforge-rknn/tests/real_device.rs`)
- 빌드 명령이 두 갈래가 된다. 실장비 배포 시 feature 를 빠뜨리면 안 된다
  (그래서 `is_rknn_enabled()` 검사가 있다)

**새로 생긴 제약**

- `#[cfg(feature = "rknn")]` 두 경로가 **같은 인터페이스를 유지해야 한다.**
  한쪽만 고치면 다른 쪽이 컴파일되지 않거나, 더 나쁘게는 조용히 갈라진다

## 뒤집힌다면

- **개발 PC 가 전부 ARM64 Linux 가 되면** 이 분리의 이유가 줄어든다.
  다만 CI 러너까지 바꿔야 하므로 가능성은 낮다
- **다른 NPU 백엔드가 추가되면** feature 이름과 구조를 다시 봐야 한다.
  `rknn` 하나를 전제로 짜여 있다

---

<a id="adr-006"></a>

# ADR-006. 크레이트를 7개로 나누고 `unsafe` 를 한 곳에 가둔다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 |
| **관련** | [ADR-004](#adr-004), [ADR-005](#adr-005), [ADR-007](#adr-007) |

---

## 한 줄 요약

> C 라이브러리를 직접 부르는 `unsafe` 코드는 **`npuforge-rknn` 안에서만**
> 존재한다. 나머지 여섯 크레이트는 안전한 Rust 만 쓴다. 메모리 문제가
> 생기면 **찾아볼 곳이 한 군데**다.

## 배경

Rust 는 메모리 안전을 컴파일러가 보장하지만, C 함수를 부르는 순간 그
보장이 끊긴다. RKNN Runtime 은 C 라이브러리다.

```c
int rknn_init(rknn_context* ctx, void* model, uint32_t size, uint32_t flag, ...);
int rknn_inputs_set(rknn_context ctx, uint32_t n, rknn_input inputs[]);
int rknn_outputs_get(rknn_context ctx, uint32_t n, rknn_output outputs[], ...);
```

포인터, 수명, 해제 시점을 사람이 관리해야 한다. 이 코드가 저장소 여기저기에
흩어지면 "해제 후 사용" 같은 버그가 어디서 왔는지 찾을 수 없게 된다.

## 결정

**1. 크레이트를 7개로 나눈다.**

| 크레이트 | 책임 | `unsafe` |
|---|---|---|
| `npuforge-common` | 타입, 오류 코드, 설정, 백엔드 인터페이스 | 없음 |
| `npuforge-proto` | gRPC 정의 (.proto → tonic 생성) | 없음 |
| `npuforge-scheduler` | 정책, 레지스트리, 재시도, 헬스체크 | 없음 |
| `npuforge-node` | 워커 풀, 큐, 등록·하트비트 | 없음 |
| `npuforge-mock-backend` | 하드웨어 없는 백엔드 | 없음 |
| `npuforge-bench` | 부하 생성, 집계, 유효성 판정 | 없음 |
| **`npuforge-rknn`** | **RKNN FFI 와 안전한 래퍼** | **여기만** |

**2. `unsafe` 는 `npuforge-rknn` 밖으로 나가지 않는다.** 이 크레이트의
설명문에 그렇게 적어 두었다 — "unsafe 코드는 이 크레이트로 제한한다".

**3. 경계에서 안전한 타입으로 바꾼다.** 바깥은 `InferenceBackend` /
`LoadedModel` 인터페이스만 본다. 포인터는 경계를 넘지 않는다.

**4. 위험한 계약은 타입으로 표현한다.** 예: `RknnContext::infer` 가
`&mut self` 를 받아 동시 호출을 컴파일러가 막는다
([ADR-007](#adr-007)).

## 근거

### 1. 찾아볼 곳이 한 군데다

메모리 오류, 이상한 크래시, 설명 안 되는 값이 나오면 `npuforge-rknn` 부터
본다. 이 크레이트는 workspace 전체에서 작은 비중이라 훑는 비용이 낮다.

### 2. 나머지 크레이트를 하드웨어 없이 검증할 수 있다

`unsafe` 와 하드웨어 의존이 같은 자리에 모여 있어서, 그것만 떼면 나머지가
전부 순수 Rust 다. Mock 으로 갈아 끼우는 것도 이 분리 덕분에 가능하다.

### 3. C wrapper 를 얇게 유지할 근거가 된다

FFI 는 `native/rknn_wrapper.c` 를 거친다. 이 wrapper 는 **실기 헤더와 대조해
시그니처를 확인**했고, `rknn_context` 가 aarch64 에서 `uint64_t` 라는 것까지
확인해 두었다. 한 곳에 모여 있으니 이런 대조가 가능하다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 단일 크레이트 | `unsafe` 가 전체에 번진다. feature gate 로 Windows 빌드를 살리기도 어려워진다 |
| 크레이트를 더 잘게 쪼갠다 | 7개도 이 규모에서는 충분히 많다. 더 쪼개면 의존 관리 비용만 는다 |
| `bindgen` 으로 FFI 자동 생성 | 헤더가 저장소에 없고 SDK 버전에 묶인다. 손으로 쓰고 실기 대조하는 편이 통제 가능했다 |
| `unsafe` 를 노드 안에 직접 | 노드가 RKNN 에 묶여 Mock 경로가 성립하지 않는다 |

## 결과

**얻은 것**

- `unsafe` 감사 범위가 한 크레이트로 고정
- 여섯 크레이트가 하드웨어 없이 테스트된다
- 백엔드 교체 지점이 명확하다

**잃은 것 / 대가**

- 크레이트 경계를 넘는 리팩터가 번거롭다. 타입을 `npuforge-common` 으로
  올려야 하는 경우가 생긴다
- `npuforge-common` 이 모두의 의존이라 여기를 고치면 전체가 재컴파일된다

**새로 생긴 제약**

- **`npuforge-common` 에 무엇을 넣을지 신중해야 한다.** 여기가 계약서라서,
  한 크레이트에만 필요한 것을 올리면 결합이 늘어난다
- `unsafe` 를 다른 크레이트에서 쓰고 싶어지는 순간이 오면, 그건 설계를
  다시 볼 신호다

## 뒤집힌다면

- **다른 NPU 백엔드가 추가되면** `npuforge-rknn` 과 나란히 새 크레이트가
  생긴다. "unsafe 는 한 곳" 이 "unsafe 는 백엔드 크레이트들에만" 으로
  넓어진다. 그때 공통 FFI 유틸을 어디 둘지 정해야 한다

---

<a id="adr-007"></a>

# ADR-007. A dedicated RKNN context per thread, with sharing blocked by the type system

*[한국어 원문](007-per-thread-rknn-context.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Supersedes** | the judgement in `environment-matrix.md` §3.1 that "RKNN 2.3.0 is thread-safe, so the context may be shared" |
| **Related** | [ADR-020](#adr-020) (`worker_count=8`), `docs/discuss.md` §9, `docs/RESULTS.md` §4.3 |

---

## In one line

> Sharing a context produces **answers that are 100% wrong while raising not a
> single error**. Confirmed by measurement. So each worker gets its own
> context, and `infer` takes `&mut self` so that sharing does not compile at
> all.

## Context

### What a context is

In the RKNN Runtime a **context** is the handle produced by loading a model
into memory. Opening a `.rknn` file yields one context, and inference is
performed against it.

One inference is **three** function calls.

```text
rknn_inputs_set   put the input image into the context
rknn_run          run it on the NPU
rknn_outputs_get  take the result out
```

### What had to be decided

The node runs 8 workers (→ ADR-020). For those 8 to infer concurrently, one of
two choices had to be made.

| | |
|---|---|
| **Shared** | 8 workers use one context together. Less memory. Shorter code |
| **Dedicated** | each worker holds its own context. Uses more memory |

### Sharing was the original decision

`environment-matrix.md` §3.1 already recorded the conclusion that **"RKNN
Runtime 2.3.0 is thread-safe"**. If that is right, sharing is the obvious
choice. There is no reason to make eight of something when one will do.

### But it was suspicious

Two things stood out.

**First, one call being safe and a sequence being safe are different things.**

```text
thread A:  inputs_set(photoA) -----------> outputs_get()  <- what comes out?
thread B:            inputs_set(photoB) -> run()
```

Even if each individual `inputs_set` call is thread-safe, if B cuts in
**between** A putting in its input and taking out its result, A receives B's
result. The atomicity of individual calls and **the atomicity of a sequence**
are separate matters.

**Second, checking what that "thread-safe" verdict had actually looked at — it
was counting API return codes only.** It never compared output contents. Even
with results getting mixed up, the return codes come back a healthy
`ok 40 / err 0`.

## Decision

**1. Give each worker a dedicated context.** `ContextPool` creates
`worker_count` contexts and a semaphore has each worker take an idle one.

```rust
pub struct ContextPool {
    contexts: Vec<Mutex<RknnContext>>,   // an independent lock per context
    permits: Arc<Semaphore>,             // issued to the number of free slots
    ...
}
```

Since the semaphore permit is acquired first, **at least one must be free** in
the subsequent `try_lock` scan. If none is found, the semaphore and lock counts
have diverged, so it raises an internal error instead of quietly moving on —
left alone it would look only like an unexplained performance drop.

**2. Let the compiler block sharing.**

```rust
/// Taking `&mut self` is this type's concurrency contract.
/// The compiler blocks concurrent calls on the same context.
pub fn infer(&mut self, input: &[u8]) -> Result<Vec<u8>>
```

With `&self`, a shared call is **syntactically possible**. With `&mut self`,
code using the same context from two places at once simply does not build.

> This is the most important part of this ADR. **Writing "do not share" in a
> comment and blocking it with a type are different things.** This defect
> cannot be found by eye, so leaving it to human attention means it comes back
> eventually.

**3. Pool creation is all-or-nothing.** If any one of the 8 contexts fails to
open, the whole node fails. A node that came up half-way and quietly runs at
lower throughput is worse than one that dies clearly — in a benchmark such a
node gets recorded as "the slow node" and contaminates the conclusion.

## Rationale

### The measurement

Measured with `native/shared_context_test.c`. Each thread is given **a different
input**; a reference output is first captured by inferring alone, then the
concurrent results are compared against each thread's own reference.

```text
conditions: king, FP16, 4 threads x 50 = 200 inferences
```

| Configuration | API errors | **Result mismatches** |
|---|---:|---:|
| Shared context | 0 | **200 / 200 (100%)** |
| Per-thread dedicated | 0 | 0 / 200 (0%) |

**Sharing raised not a single error and got everything wrong.**

### Why this defect is especially bad

- **No exception and no error code.** Nothing is left in the logs
- **It never reproduces in a single-threaded test.** It passes CI
- **The throughput metric actually looks better.** Two threads sharing reached
  34.8 inf/s against 33.2 dedicated — **it was producing wrong answers faster**
- **It looks plausible to the eye.** Being detections from another frame, the
  output is not garbage but "boxes that make sense"

Had this gone unnoticed, it would very likely have reached a public talk with
**all throughput figures valid and only the detection results quietly wrong**.
The structure was one where performance gets boasted about and accuracy gets
checked by nobody.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| One shared context | 100% wrong answers when measured. Out of the question |
| One context + a mutex to serialize | Correct answers, but the NPU is used one at a time. The point of 8 workers disappears |
| Duplicate with `rknn_dup_context` | Not verified. Individual `rknn_init` already gives correct answers with adequate performance, so it dropped down the priority list |
| Stating "do not share" in comments and docs | This defect is invisible. A rule that humans have to keep gets broken eventually |

## Consequences

**Gained**

- Zero result mismatches under 8-worker concurrent inference
- Sharing code became **impossible to write**
- The check is part of six real-hardware integration tests
  (`crates/npuforge-rknn/tests/real_device.rs`)

**Lost / the cost**

- Uses the memory of 8 contexts. **How much more was not measured.** With 4 GB
  of node RAM and a 6.46 MB model (INT8) it was judged unlikely to matter and
  left there — not a reasoned judgement, just deferred because the headroom
  looked large
- Pool creation time scales with context count (once at node startup)

**New constraint introduced**

- **The meaning of `supports_concurrent_infer = true` has changed.** It used to
  mean "the runtime handles it", and now means **"the backend serializes it
  through a pool"**. The value is the same; the basis differs
- Raising `worker_count` raises the context count with it. This value must not
  be increased without checking memory headroom

## What would overturn this

If RKNN ships a version that separates per-call context state, it can be
revisited.

**But the re-verification criteria are pinned down in advance.**

- ❌ Do not judge by API return codes. That method missed this defect
- ✅ **Give each thread a different input and compare byte-for-byte against the
  standalone reference output.** Zero mismatches to pass
- ✅ Higher throughput is not grounds for passing. We have already seen that a
  configuration producing wrong answers fast looks faster

## The lesson left behind

This incident was **the third** of the same type of mistake.

```text
1. reading run_duration as NPU occupancy time      -> it included queue wait
2. sampling NPU load with delayms=3000 still set   -> it was reading a 3-second average
3. judging thread-safety by API return codes only  -> results never compared   <- this ADR
4. judging throttling by NPU clock alone           -> the CPU was the one bending
```

What they share: **not checking what a metric counts and trusting it by its
name.**

The rule that came out of this is `preflight-check.sh --with-inference`.
**Before measuring performance, check that the three boards give the same
answer to the same input.** A configuration that produces wrong answers fast
must not win a benchmark.

---

<a id="adr-008"></a>

# ADR-008. Internal communication uses gRPC (tonic + Protocol Buffers)

*[한국어 원문](008-grpc-tonic-protobuf.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-003](#adr-003), [ADR-012](#adr-012), [ADR-024](#adr-024), `docs/01-TECHSPEC.md` §5.3, §7 |

---

## In one line

> Client↔scheduler and scheduler↔node communication uses **gRPC**. The schema
> lives in one place as `.proto` and Rust code is generated from it. The
> management API and the dashboard's REST/JSON are kept separate.

## Context

Most of what moves through this system is **a large binary blob**.

```text
request   raw RGB 640x640x3   = 1,228,800 byte
response  9 raw tensor blobs  = 1,218,000 byte  (want_float=0)
```

And although there are only three nodes, hundreds of these move per second
(the INT8 3-node target is 471 inf/s).

There were three protocol candidates.

| | |
|---|---|
| REST + JSON | works everywhere and is easy to debug. Inflates binary with base64 |
| gRPC | binary as-is, schema enforcement, code generation |
| A hand-rolled binary protocol | could be fastest. Everything has to be built by hand |

## Decision

**1. Internal RPC is gRPC + Protocol Buffers.** Implemented with `tonic`.

**2. The schema lives in one place, the `npuforge-proto` crate.** Rust types
are generated from `.proto` at build time.

**3. The services are split in two.**

| Service | Direction | Purpose |
|---|---|---|
| `SchedulerService` | client → scheduler | `Infer`, `BatchInfer`, `ListNodes` |
| `NodeService` | scheduler → node | inference delegation, status queries |

Node registration and heartbeats also travel over gRPC.

**4. The management API and dashboard are separate, on REST/JSON + axum.**
They are called directly from the browser, so putting them on gRPC would add
another gateway.

**5. The payload arrives as a single `bytes` field.** The tensor structure is
described not by protobuf but by our own blob format
([ADR-012](#adr-012)).

## Rationale

### 1. Avoid base64

Sending a 1.23 MB image over REST/JSON requires base64 encoding. That makes it
**about 1.33× larger** and adds encode/decode CPU on both ends.

Both are damaging in this project. The network is already close to saturation
at aggregation
([ADR-014](#adr-014)), and CPU is already a
bottleneck under sustained load.

### 2. The schema has to be in one place or three nodes drift apart

The three nodes run **the same binary**, but the scheduler runs on a separate
host. If message definitions are scattered through the code, one side gets
updated without the other.

With `.proto` as the single source, both sides are generated from the same
definition.

### 3. The timing breakdown fields have to travel structured

Eleven timing fields (`TimingBreakdown`) come back with each response. They are
this project's central output, so **a field must not silently disappear.** The
protobuf schema enforces that.

### 4. The Rust ecosystem is ready

`tonic` runs on Tokio and comes with streaming, timeouts and connection reuse
built in. Reusing a per-node channel to reduce connection cost also works
directly.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| REST + JSON (internally too) | 1.33× base64 inflation plus encoding CPU. Both network and CPU are already tight |
| REST + multipart/octet-stream | Avoids the inflation but loses schema enforcement. The timing fields would have to be kept in sync by hand |
| A hand-rolled binary protocol | Could be fastest, but reconnection, streaming and error propagation all have to be built. That time is time not spent measuring |
| Extending gRPC to the management API | Cannot be called directly from a browser. Adds a grpc-web gateway |

## Consequences

**Gained**

- Binary carried without inflation
- A single source for message definitions
- The mock 3-node integration test **runs over real gRPC** — it is one process,
  but the transport path is the same as on real hardware

**Lost / the cost**

- Cannot be poked directly with `curl`. Another debugging tool is needed
- Changing `.proto` involves the build pipeline (`build.rs`)
- There are now two protocols (gRPC + REST). Error representation has to agree
  across both → [ADR-024](#adr-024)'s `NPF-xxxx` is that glue

**New constraint introduced**

- Message size limits have to be managed explicitly. A 1.23 MB request fits
  under the default 4 MB limit, but experiments that increase input size (S6)
  will need to check

## What would overturn this

- **If the input becomes JPEG and payloads drop to the 100 KB class**, the
  absolute cost of base64 inflation shrinks. The schema-enforcement reason
  still stands
- **If a public external API becomes a requirement**, consider putting a REST
  gateway in front of gRPC. That is not a reason to change the internal
  protocol

---

<a id="adr-009"></a>

# ADR-009. Fix the policies at three, and have all three share the candidate filter

*[한국어 원문](009-three-policies-shared-filter.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-06 |
| **Related** | [ADR-003](#adr-003), [ADR-010](#adr-010), `docs/01-TECHSPEC.md` §10.0, §10.4 |

---

## In one line

> There are only `round-robin` / `least-queue` / `ect`. And **all three pass
> through exactly the same candidate filter.** If the filters differed, a policy
> comparison would measure the filters rather than the policies.

## Context

Comparing scheduling policies (scenario S3) is one of this project's
measurement items. It aims to measure "how much better is choosing by load than
simply going round in order".

A policy consists of two parts.

```text
1. candidate filter   who is eligible  (exclude dead nodes, only nodes holding the model ...)
2. selection rule     who among the candidates  (in order / shortest queue / estimated completion time)
```

There is a trap here. **If part 1 is made different per policy**, then when
policy A comes out ahead of policy B, there is no way to know whether that was
the selection rule or the filter.

For instance, if only ECT carried "exclude nodes above 85 °C", whether ECT wins
because it is smarter or because it avoids hot nodes cannot be separated.

## Decision

**1. Fix the policy identifiers at three.**

| Identifier | Policy | Purpose |
|---|---|---|
| `round-robin` | Round Robin | comparison baseline |
| `least-queue` | Least Queue | intermediate comparison |
| `ect` | Estimated Completion Time | recommended default |

**2. All three pass through an identical candidate filter.**

```text
- must be in an is_schedulable() state
- must hold the requested model in a Ready state
- temperature must be below disable_temperature_c
```

**3. Parse the identifier string in exactly one place.**

```rust
#[serde(rename_all = "kebab-case")]
pub enum SchedulingPolicyKind { RoundRobin, LeastQueue, Ect }
```

The configuration file, CLI arguments, metric labels, logs and dashboard all use
**the same strings**. Variants such as `queue-aware`,
`estimated-completion-time` or `queue_aware` are not used.

**4. Narrow the interface to the selection rule alone.**

```rust
pub trait SchedulingPolicy: Send + Sync {
    fn select_node(&self, task: &InferenceTask, candidates: &[NodeSnapshot])
        -> Result<NodeId, ScheduleError>;
}
```

`candidates` is **a list that has already passed the filter**. Since the policy
never sees the full node list, the room for a policy to add its own filter is
structurally reduced.

## Rationale

### Policy comparison is one of this project's measurement items

S3 is an experiment measuring "the difference between policies". There must be
one variable. Without a shared filter, the experimental design itself is void.

### A wobbling identifier contaminates the results

This actually came up while designing the bench tool. Having `--policy
round-robin` typed by hand invites a typo, or a value attached to the results
that differs from the scheduler's actual configuration. **A result labelled with
the wrong policy name ruins the whole of S3.**

So the bench tool **prefers the value the scheduler reports** over the one typed
by hand. It pairs with this decision.

### Three is enough

- `round-robin` is the baseline. Without it there is no way to know whether the
  rest are good
- `least-queue` answers "is looking at the queue alone sufficient?"
- `ect` looks at queue, speed, temperature and errors together

A fourth would multiply the experimental combinations and increase S3's run
count. It would not be worth it within the budget of 146 runs and roughly 23.4
hours.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| A different filter per policy | S3 would measure filter differences. **The thing most to be avoided** |
| Open the policies up as plugins | The comparison set becomes unbounded. Fixed is better for a measurement project |
| Implement only one policy (ECT) | Without a baseline there is no way to say "how much better" |
| Free-form identifier strings | Typos and notation drift contaminate the result labels |

## Consequences

**Gained**

- The S3 policy comparison holds — the single variable is the selection rule
- Policy names in configuration, logs, metrics and the dashboard are always the
  same
- Policy implementations get shorter. They do not each write a filter

**Lost / the cost**

- Policy-specific candidate conditions cannot be added. Adding one means
  **putting it in the shared filter and applying it to all three**
- Adding a new policy means editing the enum (deliberate friction)

**New constraint introduced**

- Changing the filter makes results **incomparable with the three policies' past
  measurements**. A filter change is treated as a change of experimental
  conditions and has to be recorded

## What would overturn this

- **If a candidate condition is found that genuinely must differ per policy.**
  At that point, first check whether it can be expressed as a score inside the
  selection rule — ECT's `load_factor` works that way
  ([ADR-010](#adr-010))
- **If the M7 optimization experiments need a new policy**, add a fourth. But
  only after S3's baseline comparison has already finished with three

---

<a id="adr-010"></a>

# ADR-010. ECT 점수식과 그 안의 각 항

| | |
|---|---|
| **상태** | 확정 (실장비 검증 전) |
| **날짜** | 2026-08-06 |
| **관련** | [ADR-009](#adr-009), [ADR-027](#adr-027), `docs/01-TECHSPEC.md` §10.4 |

---

## 한 줄 요약

> 기본 정책 ECT 는 **"이 요청을 저 노드에 주면 언제 끝나는가"** 를 점수로
> 매겨 가장 낮은 노드를 고른다. 식의 각 항은 전부 이유가 있고, 특히
> `+ 1` 과 `load_factor` 는 없으면 잘못 동작한다.

## 배경

Least Queue 는 "큐가 짧은 노드" 를 고른다. 노드 성능이 같으면 그것으로
충분하지만, 실제로는 다르다.

```text
노드 A  큐 2건, 한 건에 50 ms   →  약 100 ms 뒤 빔
노드 B  큐 1건, 한 건에 200 ms  →  약 200 ms 뒤 빔
```

Least Queue 는 B 를 고른다. **틀렸다.** 큐 길이만으로는 "언제 빌지" 를 알 수
없다. 노드마다 속도가 다르고, 온도로 느려지기도 하고, 최근 실패가 잦을 수도
있다.

## 결정

```text
ECT = ((queue_depth + in_flight + 1) × EWMA_inference_time
       + EWMA_network_time
       + thermal_penalty
       + error_penalty)
      / load_factor
```

가장 낮은 점수의 노드를 고른다. 동점이면 **Node ID 사전순**.

### 각 항

| 항 | 뜻 |
|---|---|
| `queue_depth` | 그 노드가 아직 시작 못 한 대기 건수 |
| `in_flight` | 지금 처리 중인 건수 |
| `+ 1` | **지금 배정하려는 이 요청 자신** |
| `EWMA_inference_time` | 최근 추론 시간의 이동평균. 노드별 실제 속도 |
| `EWMA_network_time` | 스케줄러↔노드 왕복 이동평균 |
| `thermal_penalty` | 온도가 높으면 가산 |
| `error_penalty` | 최근 오류가 잦으면 가산 |
| `load_factor` | 노드 상태별 가중치. 나누는 값 |

## 근거

### `+ 1` 이 없으면 안 되는 이유

두 가지다.

**첫째, ECT 의 정의가 그렇다.** "이 요청이 언제 끝나는가" 를 추정하는
값이므로 **자기 추론 시간이 포함되어야** 한다. 앞에 2건 있는 노드에 넣으면
내 것까지 3건이 걸린다.

**둘째, 없으면 `load_factor` 가 무력화된다.**

```text
큐가 빈 노드:  (0 + 0) × EWMA = 0
               0 / load_factor = 0     ← 상태가 뭐든 항상 0
```

0 은 무엇으로 나눠도 0 이다. 아래 `Recovering` 억제가 통째로 사라진다.

### `load_factor` 가 푸는 문제

| 상태 | load_factor |
|---|---:|
| Healthy | 1.0 |
| Busy | 1.0 |
| Degraded | 0.5 |
| Recovering | 0.25 |
| 그 외 | 0.0 (후보 제외) |

**`Recovering` 노드는 큐가 비어 있어서 점수만 보면 항상 이긴다.** 방금
살아난 노드에 요청이 전부 몰리고, 같은 원인으로 다시 죽는다.

PRD FR-07 은 "복구된 노드에는 제한된 요청만 할당" 을 요구한다. 이걸
별도 카운터나 토큰 버킷으로 구현할 수도 있었지만, **점수 하나로 표현했다.**
`0.25` 로 나누면 점수가 4배가 되어 자연히 덜 뽑힌다.

상태를 후보 필터가 아니라 **점수에 넣은 것**이 요점이다. 필터로 빼면
"쓰거나 안 쓰거나" 둘 뿐인데, 점수로 두면 **정도**를 표현할 수 있다.

### 동점 처리를 Node ID 사전순으로 고정한 이유

**재현성 때문이다.** 동점을 무작위나 해시 순서로 깨면 같은 조건의 반복
실험이 매번 다른 분배를 낸다. 그러면 확장 효율 측정의 분산이 커지고,
그 분산이 어디서 왔는지 설명할 수 없다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| Least Queue 만 쓴다 | 노드 속도 차이를 반영 못 한다. 위 A/B 예시에서 틀린 답 |
| `Recovering` 을 후보에서 제외 | 복구된 노드가 영영 안 들어온다. 언제 넣을지 또 정해야 한다 |
| 별도 토큰 버킷으로 복구 노드 제한 | 상태가 하나 더 생긴다. 점수식 하나로 되는 일 |
| 동점을 무작위로 | 재현성이 깨진다 |
| 온도·오류를 필터로만 처리 | 이분법이 된다. 79°C 와 81°C 가 전혀 다르게 취급된다 |

## 결과

**얻은 것**

- 노드 속도 차이·온도·오류율·복구 상태를 **점수 하나로 통합**
- 복구 노드 억제가 별도 상태 없이 구현됨
- 동점이 결정적이라 반복 실험이 재현된다

**잃은 것 / 대가**

- **튜닝 파라미터가 늘었다.** EWMA 계수, `thermal_penalty` / `error_penalty`
  의 크기, `load_factor` 값 — 전부 정해야 한다
- 식이 복잡해 로그만 보고 "왜 이 노드를 골랐는지" 즉시 알기 어렵다

**새로 생긴 제약**

- **아직 실장비에서 검증하지 않았다.** Mock 3노드에서 동작은 확인했지만,
  `load_factor` 나 penalty 값이 실제로 맞는지는 M4 에서 봐야 한다.
  현재 값은 **초안**이다
- 온도 임계치(80 / 90°C)도 초안이다. 정식 S0 열 측정 후 재설정한다

## 뒤집힌다면

- **M4 실장비 검증에서 ECT 가 Least Queue 보다 낫지 않으면** 식을 의심한다.
  다만 그 결과 자체도 유효한 산출물이다 ([ADR-002](#adr-002))
- **`Recovering` 노드가 0.25 로도 여전히 과부하를 받으면** 값을 낮추거나
  절대 상한을 추가한다
- **penalty 항이 실제로 아무 효과가 없으면** 빼는 것도 결과다. 항이 있다는
  것과 그것이 동작한다는 것은 다르다

---

<a id="adr-011"></a>

# ADR-011. The reference model is INT8

*[한국어 원문](011-int8-quantization.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-012](#adr-012), [ADR-014](#adr-014), [ADR-018](#adr-018) (model deployment), `docs/discuss.md` §8 |

---

## In one line

> INT8 quantization is worth **1.86×**. It landed an order of magnitude harder
> than any software optimization attempted so far. The cost is −5.5% on the top
> detection score, and **the detection set and classes are identical**.

## Context

### What quantization is

A neural network normally computes in reals (FP32). **Shrinking those reals to
8-bit integers** for computation is INT8 quantization. Each multiplication gets
cheaper and less memory moves. In exchange, values get coarser and a little
accuracy is lost.

FP16 sits in between — still real, just half the bits.

### Why this choice mattered

Starting from FP16, three things were tried to raise one node's throughput, and
**all three failed.**

| Attempt | Result |
|---|---:|
| Manual NPU core assignment via `core_mask` | +0.1% |
| `want_float=0` (measured mostly single-threaded at the time) | +5.4% |
| Zero-copy buffer reuse | **−1.8%** |

The reason was found too. Each inference triggers about 76 kernel `ioctl` calls
and those get **serialized**. Not something the application could reduce. So the
conclusion at the time was **"the node ceiling of 78 inf/s is a driver
characteristic."**

INT8 was the last big variable still outstanding.

## Decision

**1. The reference model is YOLOv8n INT8.**

**2. FP16 is not deleted but kept as a comparison condition.** Presenting the
two models side by side is itself the result of "how much does quantization
buy".

**3. Define the accuracy acceptance criterion at the detection level rather than
raw tensor similarity.** (See the trap section below for why.)

## Rationale

### 1.86×

```text
conditions: king, sustained_load_test, 8 threads fixed, 120 s,
            governor=performance, fanless
```

| Model | Throughput | Mean latency | Model size |
|---|---:|---:|---:|
| YOLOv8n FP16 | 84.3 inf/s | 94.5 ms | 9.65 MB |
| **YOLOv8n INT8** | **157.2 inf/s** | **50.8 ms** | 6.46 MB |
| Ratio | **1.86×** | −46% | −33% |

> Initial values measured with the `ondemand` governor were FP16 79.0 / INT8
> 146.2. **The 1.85–1.86× ratio holds regardless of governor.**

### This measurement corrected an earlier conclusion

If INT8 is 1.85×, that conflicts with the explanation that "76 ioctls set the
ceiling". So INT8's ioctls were counted too.

```text
strace -c -f -e trace=ioctl, 1 thread, 20 s

        inferences  throughput    ioctls per inference
FP16    315         15.7 inf/s    76.4
INT8    718         35.8 inf/s    76.2
```

**The call count is identical and throughput is 2.28×.**

What sets the ceiling is not the **number** of ioctls but **how long one
inference holds the serialized section.** So the scope of the previous
conclusion was narrowed.

| Previously | Corrected |
|---|---|
| "The node ceiling of 78 inf/s is a driver characteristic" | "**On FP16**, the node ceiling is about 78 inf/s, and that value cannot be exceeded by application optimization" |
| "It cannot be exceeded by application optimization" | Stands. But **quantization is a model change, not an application optimization** |

### The accuracy cost is acceptable

```text
conditions: real board king, COCO val2017 images,
            preprocessing done in one place so both see the same input bytes
```

| Comparison | box cosine | Detection cells | Class agreement |
|---|---|---|---|
| FP16 vs ONNX | 0.99999 | 10/10 | 100% |
| **INT8 vs FP16** | **0.997** | **10/10** | **100%** |

The top detection's cell moves by one and its score is −5.5%. **The detection
set and classes are identical.** Buying 1.86× at that price is a good trade.

## ⚠️ The trap hit during accuracy verification

**Using raw-tensor cosine similarity as the acceptance criterion misjudges this
model.**

Even for FP16 vs ONNX — a comparison with no quantization at all — **the cosine
of some tensors falls to 0.16.** Looking at that number alone leads to "the FP16
conversion broke the model". A wrong conclusion.

The cause is this.

- Of YOLOv8n's 9 outputs, tensors 2/5/8 are **the sum of 80 class scores**
- RKNN's sigmoid does not output exactly 0 but has **a floor of 0.001831**
- Amplified 80×, that produces **a 0.1465 offset** (matching the measured floor
  exactly)
- Most output cells are background, so this offset dominates the cosine

**The same value is added to every cell, so the ranking does not change. The
detections are unaffected.**

→ The acceptance criterion was changed to the **detection level** (detection
set, classes, box cosine). `tools/model-converter/compare_detections.py`
compares against that criterion.

This too is one of this project's recurring failure types. **A metric's name was
read and its meaning assumed.** "Low cosine similarity = different results" is
generally true, but not for this output structure.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Stay on FP16 | Throws away 1.86×. And it has already been confirmed there is no way to produce that much in software |
| FP32 | Meaningless on this NPU. Big and slow |
| INT8 + accuracy-loss compensation (QAT and the like) | Requires retraining. This project builds an inference runtime, not trains models |
| A larger model (YOLOv8s and the like) at INT8 | The comparison baseline changes. Model selection is a separate decision, and only one variable moves at a time here |

## Consequences

**Gained**

- 157.2 inf/s per node. 1.86× against FP16
- Mean latency 94.5 → 50.8 ms
- Model size −33%

**Lost / the cost**

- Top detection score −5.5%, top detection cell moved by one
- **Calibration data became necessary.** 200 COCO val2017 images are chosen
  deterministically (`fetch_calibration.py`, fixed seed). The images are not put
  in the repository for licensing reasons; only a manifest is kept
- **INT8 conversion is not byte-reproducible.** Converting three times from the
  same input gave a different hash each time (same size, 1.8% of bytes
  differing). But **the inference results are completely identical** (all 9
  tensors at cosine 1.000000). The difference is in serialization and layout,
  not in computation → the model is converted once and deployed to all three
  nodes (ADR-018)

**New constraint introduced**

- **Network load went up instead.** With throughput at 1.86×, the bytes moving
  per second rise by the same factor — 1.545 Gbps per node, 4.636 Gbps across
  three. This decision is the direct cause of
  [ADR-014](#adr-014)'s 10G aggregation
- Kept as a case of something else filling up when performance improves

## What would overturn this

- **If an input or model appears where the detection set differs.** The current
  basis is a single image. That the sample is small is acknowledged in using it
- **Re-verification is done at the detection level, not by tensor cosine.** The
  trap section above is why. Forgetting this criterion and judging by cosine
  would mean discarding a perfectly good model

---

<a id="adr-012"></a>

# ADR-012. The node sends integers without dequantizing (`want_float=0`, blob v2)

*[한국어 원문](012-want-float-zero-blob-v2.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-12 |
| **Related** | [ADR-011](#adr-011) (INT8 adopted), [ADR-014](#adr-014) (10G aggregation), [ADR-021](#adr-021) (node-side postprocessing not implemented), `docs/discuss.md` §12 |

---

## In one line

> If the node converts results to `float32` before sending, **the response
> becomes 3.96× the request**, and even a 10G link is not enough at three-node
> saturation. So it sends **the quantized integers as they are**, with `scale`
> and `zero_point` included in the response so the receiver can convert back.

## Context

### Quantization and dequantization

An INT8 model computes in integers. Its outputs come out as integers too, and
converting them back to reals needs two values attached to each tensor.

```text
real = (quantized - zero_point) x scale
```

RKNN has **an option to do that conversion for you**.

| | |
|---|---|
| `want_float = 1` | the runtime converts to `float32` and hands that over. Convenient |
| `want_float = 0` | gives the model's native type as-is (int8 for an INT8 model) |

The default is `1`, and that is what was used at first, because it was
convenient.

### But in this project the output goes out over the network

The node does not postprocess (no NMS). It **sends all nine raw tensors back to
the scheduler** (→ ADR-021). So the output type is the link load.

```text
input                        1,228,800 byte   (640 x 640 x 3)
output want_float=1 (f32)    4,872,000 byte   <- 3.96x the input
output want_float=0 (int8)   1,218,000 byte   <- 0.99x the input
```

The load on the scheduler-side link at three-node saturation:

| Configuration | Model | 3-node TX | 3-node RX | Fits in 10G? |
|---|---|---:|---:|---|
| `want_float=1` | INT8 | 4.64 Gbps | **18.38 Gbps** | **no** |
| `want_float=1` | FP16 | 2.49 Gbps | 9.86 Gbps | barely |
| `want_float=0` | INT8 | 4.64 Gbps | 4.60 Gbps | yes |
| `want_float=0` | FP16 | 2.49 Gbps | 2.46 Gbps | yes |

The original error was **calculating the network from the input alone and
omitting the output**. Recomputing with the output inverted the conclusion —
even laying 10G would not carry three INT8 nodes.

At that point `want_float=0` was promoted from "a nice optimization to have" to
**a precondition for starting M3**. The grounds for the promotion were not
throughput but **RX bandwidth**.

## Decision

**1. Change the default of `want_float` to `false` and expose it in node
configuration.**

```toml
[worker]
want_float = false
```

**2. Bump the response blob format to v2 and carry dequantization parameters
per tensor.**

```text
magic    u32  "RKNT"
version  u32  = 2
count    u32  number of tensors
dtype    u32  0 = model's native dtype, 1 = float32
per tensor (36 byte):
  len  n_dims  dims x 4   <- present in v1 too (24 byte)
  qnt_type  zero_point  scale   <- added in v2 (12 byte)
followed by the tensor data
```

**Why this is not optional**: send int8 without `scale` and `zero_point` and
the receiver has no way at all to interpret those bytes. The numbers arrive and
nobody knows what they mean. The moment the decision was made to send integers,
carrying the parameters became an obligation that follows from it.

**3. Old blobs are not accepted.** `decode` rejects `version != 2` as an error.
Reading a 36-byte descriptor as 24 bytes because only the header says v1
produces silently misaligned values, and that is the failure mode this project
most wants to avoid.

## Rationale

### Accuracy — matches float32 on real hardware

Since we do the dequantization ourselves, it has to be checked against the
runtime's result.

```text
measured: real board king, 9 tensors
(a) the float32 received with want_float=1
(b) the int8 received with want_float=0, dequantized by hand
```

**Maximum error 9.5e-7.** At the limit of `float32` precision, so effectively
identical. (`crates/npuforge-rknn/tests/real_device.rs`)

### Throughput — 15–17% higher as a bonus

```text
conditions: king, 8 threads, 120 s, governor=performance
```

| Model | `want_float=0` | `want_float=1` | Gain |
|---|---:|---:|---:|
| INT8 | **156.7 inf/s** | 133.6 inf/s | **+17.3%** |
| FP16 | 66.9 inf/s | 57.8 inf/s | **+15.7%** |

Dequantization is done by the CPU. Not doing that work makes it faster.

> **Why was it +5.4% before.** The first measurement on 2026-08-10 was mostly a
> single-thread condition and came out at +5.4%, which got it filed as "an
> optimization with no effect". The reason the gap widens at 8 threads is that
> **the time output conversion holds the serialized section** accumulates with
> the number of concurrent threads. Kept as a case of the same experiment
> yielding a different conclusion under different conditions.

### As it turns out, the measurement tool was on this setting all along

`sustained_load_test` had **hardcoded `want_float=0` from the beginning**. So
the **157.2 / 84.3 inf/s written into the documents as settled figures were
already on `want_float=0`**, and only the Rust backend was on `true`.

Which means this change did not raise performance; it **brought the software in
line with the measurement conditions**. Put the other way: until the change,
the actual node was running 15–17% slower than the documented figures and
nobody knew.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Keep `want_float=1` + 10G | RX computes to 18.38 Gbps. **Even 10G does not work.** 25G is outside this project's budget and purpose |
| Postprocess (NMS) on the node | **This is ultimately the right answer.** The response shrinks to a few KB and RX effectively disappears. But it is unimplemented, and putting postprocessing on the node shifts CPU load to the node and changes the measurement conditions again → ADR-021 |
| Compress the response | Compression/decompression CPU enters the inference path. CPU is already the bottleneck, and this stacks on top |
| Parameters as separate fields rather than in the blob | The values differ per tensor, so attaching them to the tensor descriptor is natural. Separating them creates room for the ordering to break |

## Consequences

**Gained**

- 3-node RX 18.38 → 4.60 Gbps. **M3 became possible on 10G aggregation**
- Throughput INT8 +17.3% / FP16 +15.7%
- The node now actually matches the measurement conditions written in the docs

**Lost / the cost**

- **Responsibility for dequantization moved to the receiver.** The client has to
  understand the blob
- Changing the format means **fixing three places together**
  - `crates/npuforge-rknn/src/blob.rs`
  - `native/dump_output_test.c` (board verification tool)
  - `tools/model-converter/compare_detections.py` (accuracy comparison)
- Incompatible with v1 blobs (intentionally)

**Known flaw**

The response's `result_format` string is still **`"rknn-tensors-v1"`**. The
actual blob header says `version = 2` and the descriptor changed from 24 to 36
bytes. A client identifying the format by that string would mistake it for v1
and read in 24-byte units.

It does not surface today because every consumer is inside this repository.
**The name and the reality disagree, so this has to be cleaned up before going
public.**

## What would overturn this

- **Implementing node-side postprocessing (NMS)** shrinks the response to a few
  KB of detections and makes the blob itself largely unnecessary. At that point
  this ADR is superseded by ADR-021
- **If the input format becomes JPEG**, input TX drops tenfold and the whole
  link budget has to be recomputed. The output-side conclusion stands regardless
- Revisit if an observation shows dequantization error changing postprocessing
  results. The current basis is a per-tensor maximum error of 9.5e-7, and
  **no comparison was made at the level of detection boxes**

---

<a id="adr-013"></a>

# ADR-013. Make fanless the default, and treat throttling as something to measure rather than eliminate

*[한국어 원문](013-fanless-thermal-as-measurement.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-10 |
| **Related** | [ADR-002](#adr-002), [ADR-023](#adr-023), `docs/02-HARDWARE-SETUP.md` §9 |

---

## In one line

> Attaching a fan improves the numbers. But **edge devices sit in the field
> without one.** So fanless is the default condition, and performance falling
> from heat is treated as **something to measure, not something to remove.**
> Cooled conditions are measured separately as a comparison group.

## Context

RK3576 boards ship fanless. Put them under sustained load and they get hot and
slow down.

There are two branches here.

```text
branch 1. attach a fan
  -> the numbers improve
  -> good for a talk
  -> but those numbers do not occur in the field

branch 2. measure fanless
  -> the numbers get worse
  -> and the amount by which they get worse is a value nobody publishes
```

The TOPS vendors publish is **instantaneous performance**. How much of it is
sustained under load — **the gap between peak FPS and sustained FPS** — is
barely covered in public material.

## Decision

**1. Fanless (condition A) is the default measurement condition.**

**2. Active cooling (condition B) is measured alongside as a comparison group.**
Three fans of the same model are fixed at the same speed. Different speeds would
give the nodes different cooling conditions and break three-node symmetry.

**3. Thermal characterisation (S0) comes before every other scenario**, because
S0 determines the thresholds and cooldown times for the rest of the experiments.

**4. Do not mix improvised cooling into a measurement.** A desk fan was used
once during diagnosis; **it was valid for diagnosis but unusable as a
measurement condition.** There is a checklist item to confirm the desk fan is
off before a fanless measurement.

## Rationale

### Some questions can only be answered by measuring both conditions

```text
fanless only  ->  you do not know "how much better does cooling make it"
cooled only   ->  you do not know "how much do you get in a real edge deployment"
```

**Measure both and "the effect of cooling on scaling efficiency" becomes a
result in itself.** That is a value absent from vendor spec sheets, and it fits
this project's identity of settling things by measurement.

### Measured — it finishes fanless, but throughput is not sustained

```text
conditions: 3 boards concurrently, 8 threads, 900 s, fanless, no desk fan
```

| Board | NPU mean | NPU peak | Throughput |
|---|---:|---:|---:|
| king | 73.0 °C | 75.8 °C | 80.5 inf/s |
| queen | 67.5 °C | 70.2 °C | 77.7 inf/s |
| jack | 72.6 °C | 74.8 °C | 77.8 inf/s |

- Node-to-node spread **5.6 °C**
- Completed with 0 errors
- Never exceeded 90 °C

**Sustained 8-thread load is possible fanless.** But throughput is not
sustained.

```text
 +10s  81.6 inf/s   <- start
+120s  63.6
+300s  59.7         <- steady state.  -27% against the start
```

### ⚠️ What was collapsing was the CPU, not the NPU

The initial verdict was "no NPU throttling", because all 928 samples were at
950 MHz. **Only the NPU clock had been looked at.**

Looking at the CPU clocks in the same log:

```text
        NPU temp   npu_clk   cpu4(A72)   cpu0(A53)
 +15s   86.8 C     950 MHz   2208 MHz    2016 MHz
 +30s   90.4 C     950 MHz   1416 MHz    1200 MHz
 +60s   87.8 C     950 MHz    816 MHz     600 MHz
+120s   87.8 C     950 MHz    816 MHz     600 MHz
```

**The NPU never drops and the CPU falls 63–70%.**

One inference is `set input (CPU) → NPU → get output (CPU)`, so the CPU sections
feed directly into throughput. That was known, and the throttling verdict was
still made on the NPU alone. It is the **fourth** mistake of this type in this
project.

> The discovery actually improved the result. **"What collapses first on a
> fanless edge device is not the NPU but the CPU handling either side of it"** —
> a far better narrative for a talk.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Measure with a fan attached | Numbers that do not reproduce in the field. The opposite direction to the project's premise |
| Measure fanless only | Cannot answer "how much better does cooling make it" |
| Lower the load to avoid throttling | What happens under sustained load is exactly what is being measured, and this avoids it |
| Standardise conditions with improvised cooling (a desk fan) | Not reproducible and not uniform across nodes |

## Consequences

**Gained**

- The peak vs sustained gap became one of the project's central outputs
- The discovery that the bottleneck is the CPU rather than the NPU
- Readiness to quantify the cooling effect (S0-A / S0-B)

**Lost / the cost**

- Throughput figures come out lower. Instead of "84.3 inf/s" it has to be
  "81.6 at the start, 59.7 at 300 seconds"
- Measurement takes longer. Cooldown has to be waited out, and fanless is slow.
  So cooldown has **an upper bound**, and when that bound is hit the actual
  starting temperature is recorded with the result

**New constraints introduced**

- **Thermal verdicts must include the CPU clock.** Judging by NPU clock alone
  was confirmed wrong. `run-thermal-comparison.sh` has to be fixed accordingly
- Do not compare temperatures between two measurements with different load
  profiles. A sweep load was once compared against a fixed load and a 19 °C gap
  was misread
- The temperature thresholds (80 / 90 °C) are **a draft**. They are reset after
  the formal S0

## What would overturn this

- **If a case or heatsink becomes the standard configuration**, condition A's
  definition changes
- **If fanless exceeds 90 °C in S0 and nodes start dropping out of scheduling**,
  measurement itself becomes impossible. At that point condition B is promoted
  to default and condition A is redefined as "the limit condition"

---

<a id="adr-014"></a>

# ADR-014. Leave the worker links at 2.5G, raise only aggregation to 10G, and put the scheduler on a separate server

*[한국어 원문](014-10g-aggregation-separate-scheduler.ko.md)*

| | |
|---|---|
| **Status** | accepted (equipment obtained and measured 2026-08-20) |
| **Date** | 2026-08-12 (decision), 2026-08-20 (build and measurement) |
| **Supersedes** | the judgement that "2.5GbE is sufficient as the reference network" |
| **Related** | [ADR-003](#adr-003), [ADR-011](#adr-011), [ADR-012](#adr-012), `docs/02-HARDWARE-SETUP.md` §3.3.2 |

---

## In one line

> Traffic from three nodes converges at one point, the scheduler. **It is that
> confluence, not the worker link (2.5G), that fills up first.** So only
> aggregation is raised to 10G, and a **separate server** that can take a 10G
> NIC becomes the scheduler host.

## Context

### The original calculation went like this

```text
3 nodes 150 FPS x 1.23 MB ~ 184 MB/s ~ 1.5 Gbps  -> exceeds 1GbE, 2.5GbE is enough
```

That calculation set "the reference network is 2.5GbE". **Two things were
wrong.**

**(a) The throughput assumption was stale.** 150 FPS was assumed as the
**total** across three nodes. Measurement says **one** node does 157.2 inf/s at
INT8. Three nodes is 471 inf/s.

**(b) The output direction was ignored.** Only the input was counted. The node
does not postprocess and sends raw tensors back, so the response uses the link
too. With `want_float=1` the response was **3.96×** the request.

On top of that there was a unit error — converting MiB/s to Gbps used the
binary prefix (÷1024). **Network speeds are decimal.**

### The recomputed values

```text
one raw RGB input = 640 x 640 x 3 = 1,228,800 byte

                     per node        3-node total
INT8  157.2 inf/s    1.545 Gbps      4.636 Gbps
FP16   84.3 inf/s    0.829 Gbps      2.486 Gbps
```

**Even FP16's three-node total of 2.486 Gbps exceeds a single 2.5GbE link
(effectively about 2.35 Gbps).** INT8 exceeds it by nearly double.

## Decision

**1. Leave the worker links at 2.5G.** At most 1.545 Gbps per node, which fits.

**2. Raise only the aggregation link to 10G.**

```text
        Benchmark / Scheduler Server
                    |
                  10GbE          <- this is the point
                    |
            2.5G / 10G Switch
              |-- 2.5G -- king
              |-- 2.5G -- queen
              \-- 2.5G -- jack
```

**3. Make the scheduler host a separate server with a PCIe slot.**

**4. Reduce the output alongside.** Even with 10G laid, `want_float=1` puts RX
at 18.38 Gbps and it is still not enough. →
Solved in [ADR-012](#adr-012).

**5. Keep 1GbE rather than removing it, as a comparison condition.** Presenting
"the network is the bottleneck" and "it is not" side by side has value as a
bottleneck-analysis result (scenarios S5 and S6).

## Rationale

### Why aggregation rather than the workers

Each node uses only its own link. At most 1.545 Gbps, which fits inside 2.5G.
But **all three nodes' traffic converges in front of the scheduler.** The load
at the confluence is threefold.

```text
king  --1.5G--\
queen --1.5G---+--> 4.6 Gbps --> scheduler   <- impossible on 2.5G
jack  --1.5G--/
```

**Only this point degrades linearly as nodes are added.** In a project measuring
three-node scaling efficiency, if something fills up first as you scale, **you
end up measuring link saturation rather than NPU scaling efficiency.**

### Why a separate server — two reasons overlap

**(1) Symmetry of the measurement conditions.** Running the scheduler on one of
the nodes raises CPU and network load on that node alone. The three nodes'
conditions diverge and the 1/2/3-node comparison is distorted. You could no
longer call it a "like-for-like comparison" in a talk.

**(2) The PCIe slot.** A 10G SFP+ NIC is a PCIe card. The current scheduler
host, `dealer`, is **a laptop with nowhere to put it.**

(1) alone required a separate host, and (2) narrowed "any host" to "a server
with a PCIe slot".

## Build result (2026-08-20)

The equipment was assembled as designed and the bandwidth measured. It came
together as **10GBASE-T (RJ45) rather than DAC/SFP+** — the switch is a NEXI
NS-S25G10G-N (2.5G×4 + 10G×2, all RJ45), so the SFP+ plan became RJ45. No
effect on the conclusion.

```text
server (Rocky 9.4, Xeon x2 24T / 16GB)
  \ enp4s0 10GBASE-T -- measured 10G full (ethtool)
                        |
              NS-S25G10G-N -+ 2.5G - king  .3
                            + 2.5G - queen .5
                            \ 2.5G - jack  .4
```

| Measurement | Value | Tool |
|---|---:|---|
| Server link negotiation | 10000 Mb/s full | ethtool |
| Single king→server | 2.34 Gbps | iperf3 (the effective 2.5G ceiling) |
| **3 nodes concurrently →server** | 1.70 each, **5.11 Gbps total** | nc |

With three nodes concurrent, the three streams **stayed even** — had the server
been the bottleneck the total would have been cut, and it was not. It
comfortably accommodates the INT8 3-node RX target of **4.60 Gbps**. (The
individual 1.70 being below the 2.34 link ceiling is an nc/board-CPU limit, not
a switch or server limit. Actual M3 traffic is gRPC, so this figure is for
infrastructure verification.)

As a side effect the **scheduler host's RAM went from 3 GB (dealer) to 16 GB
(server)**, easing ADR-003's concern about scheduler RSS.

> ⚠️ Because the boards use DHCP, this rework changed their IPs wholesale
> (`.12/.16/.33` → `.3/.4/.5`). The [ADR-019](#adr-019) situation
> recurred, with stale SSH aliases failing to find the nodes. MAC-based static
> IPs are follow-up work.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| 10G everywhere (workers included) | Wasteful at 1.545 Gbps per node. The boards' NICs are 2.5G anyway |
| 25G or above | Needed to keep `want_float=1`, but reducing the output is cheaper and more correct (ADR-012) |
| Run the scheduler on `king` | Breaks the three nodes' experimental conditions. Allowed for development and demos only, never for official figures |
| Keep 2.5GbE and just measure | **The most dangerous choice.** Link saturation would get reported as scaling efficiency |
| Switch input to JPEG to reduce TX | The decode cost lands on node CPU. CPU is already the bottleneck and this stacks on it. Valid as an S6 comparison item |

## Consequences

**Gained**

- The premise for measuring scaling efficiency holds — the link does not fill
  up first
- The required equipment became clear: a 2.5G/10G switch, a PCIe server, a 10G
  NIC and an SFP+ DAC

**Lost / the cost**

- **M3 was blocked for a while on procurement.** From the decision on
  2026-08-12 to the build on 08-20. A hardware problem, not a code one
- Cost went up — switch, server, NIC, cables

**The biggest consequence of this decision: choosing not to start measuring**

Measuring without the equipment would still produce numbers. **And those numbers
would be wrong.** Scaling efficiency would come out low at three nodes, with the
cause being the link rather than the NPU. Publishing results in that state would
invalidate the project's central claim.

So **the choice was to stop measuring and wait.** That is a decision too.

**New constraints introduced**

- The scheduler host formally joined the experimental equipment list. Changing
  its specification is a change of measurement conditions
- Before starting M3, **measured TX/RX must be recorded rather than calculated**
  (`02-HARDWARE-SETUP.md` §3.3.3). This section's original error was trusting
  calculation alone

## What would overturn this

- **Switching the input format to JPEG** cuts TX roughly tenfold and the whole
  link budget has to be recomputed. 2.5G might suffice in that case — but where
  the decode CPU cost lands has to be considered alongside
- **Implementing node-side postprocessing (NMS)** effectively removes RX. What
  remains is TX at 4.64 Gbps, lowering the requirement → ADR-021
- **Adding more nodes** raises the aggregation requirement proportionally. Five
  nodes at INT8 comes to 7.7 Gbps, leaving no headroom even at 10G

---

<a id="adr-015"></a>

# ADR-015. Run a hard-failing preflight check before measuring, and measure nothing until it passes

*[한국어 원문](015-preflight-hard-fail.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-007](#adr-007), [ADR-016](#adr-016), [ADR-019](#adr-019), [ADR-028](#adr-028) |

---

## In one line

> What has ruined measurements so far has mostly been **the premises, not the
> measurement itself**. So a machine checks the premises immediately before
> measuring. **On a hard failure, measurement does not start.** And accuracy is
> checked **before** performance.

## Context

Measurements have been wrong several times, and the cause was always outside
the measurement code.

| What happened | Result |
|---|---|
| A stale IP in the docs pointed somewhere else | Misdiagnosed as a dead node; scanned the whole subnet |
| Compared two measurements with different load profiles | A 19 °C gap was misread |
| A board reset by insufficient adapter current | Its throughput was nearly read as performance |
| Sharing a context | 0 errors and 100% result mismatch |

**What they share: it was already wrong before measurement began.** And all four
give no signal while running.

## Decision

**1. Create `scripts/preflight-check.sh`, and do not measure until it passes.**

The verdict is the exit code.

```text
0  pass (warnings are possible)
1  hard failure. Measuring in this state makes the result invalid
2  script usage error
```

**2. Divide the checks into six groups.**

| Group | What it looks at |
|---|---|
| 1. Connection and identity | alias ↔ hostname agreement |
| 2. Software identity | kernel, RKNN, driver and model hashes identical across the three nodes |
| 3. Measurement conditions | governor, idle temperature, input voltage, residual load, NTP, session count |
| 4. **Inference accuracy** | do the three boards give the same answer to the same input |
| 5. Network measurement | record M3's premise values |
| 6. Cluster registration | are the three nodes attached to the scheduler |

**3. This script does not fix anything. It only judges.** Fixing is
`fix-node-consistency.sh`'s job.

**4. Treat empty values and placeholders as failures.**

**5. When adding a check, break it deliberately and confirm it actually
catches.**

## Rationale

### Why accuracy comes before performance

That is what `--with-inference` does. It gives the three boards the same input
and checks that the same answer comes out.

The reason is [ADR-007](#adr-007). The shared-context
configuration **produced wrong answers faster** (at two threads, shared 34.8 >
dedicated 33.2 inf/s).

**A configuration that produces wrong answers fast must not win a benchmark.**
Measure performance alone and such a configuration gets reported as optimal.

### The incident where "could not read" was judged as "identical"

`/sys/kernel/debug/rknpu/version` is readable only by root. Reading it without
permission returned an empty string on all three nodes, and it **passed on the
grounds that the values matched**.

```text
king  ""      \
queen ""      +- the three values match -> pass OK   <- nothing was verified
jack  ""      /
```

A variant of the mistake of not checking what a metric counts. So empty values
and placeholders such as `unknown` are treated as **failures**.

### Why the alias ↔ hostname check is number 1

**This is far more dangerous than a connection failure.** A failed connection is
known immediately. But if `npuforge-k` points at `queen`, the measurement
finishes normally and **the result is attributed to the wrong node.** It fails
quietly.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Keep the checklist in a document and have a human verify | "Let's be careful" in a document did not work. Several failures happened while knowing better |
| Put the checks inside the bench tool | Some are ([ADR-028](#adr-028)). But SSH, sudo and hash comparison are outside the tool's domain, so they were separated |
| Make check failures warnings only | Warnings get ignored, especially when you want to start measuring quickly |
| Fix things automatically | Mixing judgement with remedy leaves no record of "what had been wrong" |

## Consequences

**Gained**

- Premise failures are caught **before** measurement
- Pass/fail is an exit code, so it drops straight into automation
- The measurement conditions get recorded (`--json`)

**Lost / the cost**

- It takes time to start measuring, especially `--with-inference`, which runs
  real inference
- A sudo password is needed. It is not committed to the repository but taken
  from an environment variable or a `~/.npuforge/` file — **this project is
  going public, and anything in the commit history would need a history rewrite
  to remove**

**New constraint introduced**

- **The check itself can be wrong.** `pgrep -f` once counted itself and passed
  quietly ([ADR-017](#adr-017)). That is why "break
  it deliberately" became a rule

## What would overturn this

The list of checks keeps growing. **It never shrinks.** Adding an entry every
time a new failure mode is encountered is this script's design intent.

---

<a id="adr-016"></a>

# ADR-016. `boot_id` 로 측정 중 재부팅을 감지해 run 을 무효화한다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-11 |
| **관련** | [ADR-015](#adr-015), [ADR-028](#adr-028), [ADR-027](#adr-027) |

---

## 한 줄 요약

> 보드가 측정 도중 리셋되면 그 run 의 수치는 무효다. 그런데 **겉으로는
> "성능이 떨어진 노드"로 보인다.** Linux 의 `boot_id` 를 하트비트로 받아
> 값이 바뀌면 run 을 무효 처리한다.

## 배경

이 프로젝트는 실제로 보드가 재부팅되는 것을 겪었다. 원인을 세 번 오판했다.

```text
공용 PSU 문제로 추정  →  아니었다
부트로더 펌웨어 문제  →  일부만 맞았다
12V 입력 문제        →  아니었다
실제 원인: 전원 어댑터 전류 부족
```

문제는 원인 규명이 아니라 **그동안 나온 측정값을 어떻게 다룰 것인가** 였다.

부하 중 보드가 리셋되면 이렇게 보인다.

```text
처리량이 뚝 떨어진다        → "thermal throttling 인가?"
응답이 한동안 없다          → "네트워크 지연인가?"
그러다 다시 정상으로 돌아온다 → "회복됐네"
```

**전부 그럴듯한 해석이 붙는다.** 재부팅됐다는 사실을 모르면 이 데이터를
"고온에서의 성능 저하" 로 읽고 그래프에 그린다.

## 결정

**1. 노드가 `boot_id` 를 하트비트로 보고한다.**

Linux 는 부팅할 때마다 새 UUID 를 만든다.

```text
/proc/sys/kernel/random/boot_id
```

이 값은 재부팅하면 반드시 바뀌고, 그 외에는 절대 안 바뀐다.

**2. 스케줄러가 변화를 감지하면 경고한다.** 노드가 같은 `node_id` 로
돌아왔는데 `boot_id` 가 다르면, 그건 "잠깐 끊긴 노드" 가 아니라 **다른
인스턴스**다.

**3. 벤치 도구가 run 유효성 판정에 쓴다.** run 시작 시점의 `boot_id` 를
기록해 두고, 끝날 때 달라져 있으면 그 run 을 무효로 표시한다.

**4. preflight 가 기준값을 남긴다.** 측정 직전 세 노드의 `boot_id` 를 찍어
둔다.

**5. 무효 run 을 삭제하지 않는다.** 사유와 함께 남긴다. 재부팅이 반복되면
그 자체가 발견이다 — 실제로 어댑터 문제를 그렇게 찾았다.

## 근거

### 왜 다른 지표로는 안 되나

| 후보 | 왜 안 되나 |
|---|---|
| uptime 이 작아짐 | 폴링 간격 사이에 리셋되고 다시 올라오면 놓친다 |
| 연결이 끊김 | 네트워크 순단과 구분되지 않는다 |
| 처리량 급락 | throttling 과 구분되지 않는다. **이게 정확히 우리가 겪은 문제** |
| 프로세스 PID 변화 | 노드 프로세스만 재시작해도 바뀐다. 보드 리셋과 다른 사건이다 |

`boot_id` 는 **커널이 부팅을 셌다는 사실 그 자체**다. 해석의 여지가 없다.

### 의도된 장애와 하드 리셋을 구분해야 한다

시나리오 S4 는 **일부러 노드를 죽이고** 복구를 관찰하는 실험이다. 이때
"노드가 사라졌다" 는 정상 동작이다.

그런데 전원 문제로 보드가 죽는 것도 똑같이 보인다. 둘을 구분하지 못하면
S4 의 결과와 장비 결함을 섞어서 보고하게 된다.

`boot_id` 가 바뀌었으면 하드 리셋, 안 바뀌었으면 프로세스 수준 장애다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 재부팅을 안 나게 만든다 | 그렇게 했다(어댑터 교체). 그래도 **감지 장치는 필요하다** — 다음 원인은 다른 것일 수 있다 |
| 사람이 로그를 보고 판단 | 야간 무인 실행(146 run, 23.4시간)에서는 불가능하다 |
| dmesg 를 파싱 | 무겁고 권한이 필요하다. 한 줄 읽으면 되는 값이 있다 |
| 무효 run 을 자동 삭제 | 원인 추적이 불가능해진다. 반복 패턴 자체가 정보다 |

## 결과

**얻은 것**

- "성능 저하" 로 위장한 재부팅을 잡는다
- 무인 야간 실행에서도 데이터 유효성이 자동 판정된다
- 의도된 장애와 장비 결함이 구분된다

**잃은 것 / 대가**

- 하트비트 메시지에 필드가 하나 늘었다 (사실상 무시할 수 있는 비용)
- `boot_id` 는 재부팅만 잡는다. **커널이 살아 있는 채로 생기는 문제는 못
  잡는다** — 그건 다른 검사의 몫이다

**새로 생긴 제약**

- 노드 프로세스만 재시작한 경우와 보드 리셋은 다르게 취급해야 한다. 둘 다
  재등록을 유발하므로([ADR-025](#adr-025))
  재등록 이벤트만으로는 구분되지 않는다

## 뒤집힌다면

이 검사가 불필요해지는 상황은 "보드가 절대 리셋되지 않는다" 가 증명될 때인데,
증명할 방법이 없다. **유지한다.**

---

<a id="adr-017"></a>

# ADR-017. 원격 실행 함정을 라이브러리 함수로 굳힌다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-11 |
| **관련** | [ADR-015](#adr-015), [ADR-019](#adr-019) |

---

## 한 줄 요약

> `ssh` 로 원격 명령을 돌릴 때 **실패가 성공처럼 보이는** 함정이 셋 있다.
> 셋 다 종료 코드 0 에 stderr 가 비어 있다. 매번 조심하는 대신
> `scripts/lib/remote.sh` 의 함수로 굳혔다.

## 배경

`preflight-check.sh` 를 만들다가 발견했다. **검사가 조용히 작동하지 않았다.**
부하가 도는데 "남은 부하 없음" 으로 통과시켰다.

파고들었더니 함정이 셋이었고, 전부 같은 성질을 갖는다 — **틀렸다는 신호가
전혀 없다.**

## 함정 1. `pgrep -f` 는 자기 자신을 센다

`pgrep -f` 는 명령줄 전체를 매칭한다. 그런데 ssh 가 보내는 래퍼의 명령줄에
**패턴 문자열 자체가 들어 있다.**

```text
bash -c "... pgrep -f \"[s]ustained_load_test|...\" | wc -l"
                       ^^^^^^^^^^^^^^^^^^^^^^^^ 이게 매칭된다
```

대괄호 트릭(`[s]ustained`)도 같은 명령줄에 괄호 없는 형태가 섞이면 무력하다.

**양방향으로 틀렸다.**

| 상황 | 실제 | pgrep 보고 |
|---|---|---|
| 부하 실행 중 | 1개 | **0 (놓침)** |
| 부하 없음 | 0개 | **2 (자기 셸을 셈)** |

**해결**: `/proc/PID/exe` 심볼릭 링크를 읽는다. 실제 실행 파일을 가리키므로
셸이 끼어들 여지가 없다.

```bash
n=0
for p in /proc/[0-9]*; do
  case "$(readlink "$p/exe" 2>/dev/null)" in
    *sustained_load_test) n=$((n+1)) ;;
  esac
done
```

## 함정 2. `cd DIR && setsid nohup ... &` 는 뜨지 않는다

| 형태 | 결과 |
|---|---|
| `ssh -n H "cd $DIR && setsid nohup ./prog ... &"` | **실행 안 됨** |
| `ssh -n H "setsid nohup $DIR/prog ... &"` | 실행됨 |

`&` 는 `cd && prog` **리스트 전체**에 걸린다. ssh 가 명령을 보내고 즉시
끊는데, 백그라운드 서브셸이 `cd` 를 거쳐 `setsid` 에 닿기 전에 세션이
사라지면 그대로 죽는다.

절대경로를 쓰면 중간 단계가 없어 경합이 생기지 않는다.

**대가가 크다.** 실패해도 종료 코드는 0 이고 stderr 도 비어 있다. 확인하지
않으면 **"부하 없는 상태의 온도" 를 15분 동안 측정**하게 된다.

## 함정 3. ssh 안 heredoc + sudo 중첩은 파일을 만들지 않는다

systemd 유닛을 배포하다 겪었다. 이것도 **종료 코드 0** 이었다.

## 결정

**1. 세 함정의 회피 형태를 `scripts/lib/remote.sh` 의 함수로 만든다.**
스크립트가 ssh 를 직접 부르지 않고 이 함수를 쓴다.

**2. 원격 프로세스를 셀 때는 `/proc/PID/exe` 를 읽는다.** `pgrep -f` 를
쓰지 않는다.

**3. 백그라운드 기동은 절대경로 + `setsid nohup` 형태로만 한다.**

**4. 띄운 뒤 실제로 도는지 확인하는 단계를 넣는다.** 기동 명령의 종료
코드를 신뢰하지 않는다.

**5. 새 검사를 만들면 일부러 깨뜨려 보고 실제로 잡히는지 확인한다.**

## 근거

### 5번이 이 ADR 의 핵심이다

함정 1 을 발견한 것이 정확히 그 절차 덕분이다. **통과만 보고 믿었다면
preflight 는 아무것도 걸러내지 못하는 채로 남았을 것이다.**

검사 코드는 특히 위험하다. 평소에는 "통과" 만 출력하므로, 고장 나도 아무도
모른다. 오히려 **더 조용해질 뿐**이다.

### 왜 문서가 아니라 코드인가

이 세 함정은 전부 "알고 있으면 피할 수 있는" 것들이다. 그런데 이 프로젝트는
알면서 당한 사례가 이미 여러 건이다. 원격 명령을 새로 짤 때마다 세 가지를
기억해 내야 한다면 언젠가 빠뜨린다.

함수로 만들면 **기본 경로가 안전한 형태**가 된다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 주석과 문서로 남긴다 | 통하지 않는다는 것이 이미 확인됨 |
| Ansible 같은 도구 도입 | 의존이 늘고, 세 대짜리 실험 환경에 과하다. 함정 2 같은 문제는 여전히 남는다 |
| ssh 대신 에이전트를 상주 | 그게 `npuforge-node` 다. 다만 측정 스크립트는 노드 프로세스와 무관하게 돌아야 한다 |
| 종료 코드만 확인 | **세 함정 모두 종료 코드가 0 이다.** 근본적으로 안 통한다 |

## 결과

**얻은 것**

- 새 스크립트가 기본적으로 안전한 형태를 쓴다
- 함정을 겪은 기록이 코드 옆에 남는다

**잃은 것 / 대가**

- 스크립트가 `lib/remote.sh` 에 의존한다. 단독 실행이 어려워진다
- `/proc` 순회는 `pgrep` 보다 느리다 (검사 빈도를 생각하면 무시 가능)

**새로 생긴 제약**

- **원격 실행을 새로 짤 때 이 라이브러리를 거쳐야 한다.** 직접 `ssh` 를
  부르면 함정이 다시 열린다

## 뒤집힌다면

함정이 넷째로 늘어나면 여기에 추가된다. **줄어들 이유는 없다.**

---

<a id="adr-018"></a>

# ADR-018. 모델은 한 번만 변환해 같은 파일을 세 노드에 배포한다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-11 |
| **관련** | [ADR-011](#adr-011), [ADR-015](#adr-015) |

---

## 한 줄 요약

> **INT8 변환은 바이트 재현성이 없다.** 같은 입력으로 3회 변환하니 해시가
> 매번 달랐다. 추론 결과는 완전히 같은데도 그렇다. 그래서 노드마다
> 변환하지 않고, **한 번 변환한 파일 하나**를 세 노드에 배포한다.

## 배경

노드가 세 대이므로 모델을 준비하는 방법이 둘이다.

```text
방법 A. 노드마다 변환한다     각 보드에서 ONNX → .rknn
방법 B. 한 번 변환해 배포한다  한 곳에서 만든 .rknn 을 복사
```

A 가 자연스러워 보인다. 변환 스크립트가 결정적이면 세 노드에 같은 파일이
생길 테니까.

**그런데 결정적이지 않았다.**

## 근거

### 실측: 같은 입력, 다른 바이트

같은 ONNX 와 같은 calibration 목록으로 **3회 변환**했다.

```text
파일 크기   같음
해시        3회 모두 다름
바이트 차이 1.8%
```

그런데 추론 결과는 이랬다.

```text
출력 텐서 9개 전부 cosine 1.000000, 오차 0.0
```

**차이는 직렬화·레이아웃에 있고 수치 계산에는 없다.** 그래도 파일이 다르면
"세 노드가 같은 모델을 쓴다" 를 해시로 증명할 수 없게 된다.

### 왜 그게 문제인가

이 프로젝트는 **세 노드의 조건이 같다는 것**이 전제다. 1/2/3노드 확장 효율을
재려면 노드가 대칭이어야 한다.

preflight 는 세 노드의 모델 해시가 같은지 검사한다. 노드마다 변환하면 이
검사가 **항상 실패**한다. 그렇다고 검사를 빼면 "정말 같은 모델인가" 를
확인할 수단이 사라진다.

## 결정

**1. 모델은 한 곳에서 한 번만 변환한다.** 변환 환경은 Docker 로 고정한다
(rknn-toolkit2 2.3.0).

**2. 생성된 `.rknn` 파일을 세 노드에 복사한다.**

**3. `model.toml` 의 `sha256` 으로 배포 무결성을 검증한다.** 노드가 모델을
로딩할 때 해시를 확인한다.

**4. 그 해시가 무엇을 보장하는지 명시한다.**

```text
sha256 이 보장하는 것      배포 무결성 — 세 노드가 같은 파일을 갖는다
sha256 이 보장하지 않는 것  변환 레시피의 동일성 — 같은 절차로 만들었는지
```

**5. calibration 이미지 선택을 결정적으로 만든다.** COCO val2017 에서
seed 를 고정해 200장을 고른다(`fetch_calibration.py`). 이미지 자체는
라이선스 때문에 저장소에 넣지 않고 **manifest 만** 남긴다.

**6. 같은 원칙을 노드 바이너리에도 적용한다.** `king` 에만 Rust 툴체인이
있고, 거기서 한 번 빌드해 `queen`·`jack` 에 배포한다. 노드마다 빌드하지
않는다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 노드마다 변환 | 해시가 달라져 "같은 모델" 검사가 불가능해진다 |
| 변환을 결정적으로 만든다 | rknn-toolkit2 내부 동작이라 우리가 통제할 수 없다 |
| 해시 대신 추론 결과로 동일성 검증 | 그것도 한다(`preflight --with-inference`). 다만 배포 시점 검증으로는 무겁다 |
| 해시 검사를 뺀다 | 파일이 깨지거나 다른 버전이 섞여도 모른다. 실제로 방지하려던 사고다 |

## 결과

**얻은 것**

- 세 노드가 **바이트 단위로 같은 모델**을 갖는다
- preflight 의 모델 해시 일치 검사가 의미를 갖는다
- 변환 환경 문제가 세 배로 늘어나지 않는다

**잃은 것 / 대가**

- 배포 단계가 하나 늘어난다
- 변환 환경(Docker, rknn-toolkit2 버전)이 재현성의 일부가 된다.
  `environment-matrix.md` 에 고정한다

**새로 생긴 제약**

- **`sha256` 을 변환 레시피 검증으로 착각하면 안 된다.** 같은 해시는 같은
  파일을 뜻할 뿐, 같은 절차로 만들었다는 뜻이 아니다. 재현하려면 변환
  명령·데이터셋·툴킷 버전을 따로 기록해야 한다
- 모델을 다시 변환하면 **모든 노드에 다시 배포**해야 한다. 한 노드만 갱신하면
  preflight 가 막는다 (의도한 동작)

## 뒤집힌다면

- **rknn-toolkit2 가 결정적 변환을 보장하게 되면** 노드별 변환도 가능해진다.
  다만 그래도 한 번 변환해 배포하는 쪽이 단순하다
- **모델이 노드마다 달라야 하는 실험**이 생기면(예: 노드별 다른 정밀도)
  전제가 바뀐다. 그 경우 "노드 대칭" 자체가 실험 변수가 된다

---

<a id="adr-019"></a>

# ADR-019. 보드는 IP 가 아니라 SSH 별칭으로 접근한다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-11 |
| **관련** | [ADR-015](#adr-015), [ADR-017](#adr-017) |

---

## 한 줄 요약

> 문서에 박아 둔 IP 가 낡아서 **노드가 죽었다고 오판**하고 서브넷 전체를
> 스캔했다. `~/.ssh/config` 에는 처음부터 올바른 값이 있었다.
> 접근은 `npuforge-k` / `-q` / `-j` 별칭으로만 한다.

## 배경

2026-08-11 에 `king` 에 접속이 안 됐다.

```text
문서에 적힌 IP     10.20.0.22
실제 IP            10.20.0.12
```

노드가 죽은 줄 알고 서브넷을 훑었다. 그런데 `~/.ssh/config` 에는 **처음부터
올바른 IP 가 있었다.** 낡은 것은 문서뿐이었다.

이게 왜 위험한가. 접속이 아예 안 되면 그나마 낫다 — 즉시 알 수 있으니까.
**진짜 위험한 경우는 그 IP 에 다른 보드가 있을 때다.**

```text
npuforge-k 로 측정 → 실제로는 queen 에 붙음 → 측정은 정상 종료
                                            → 결과가 king 것으로 기록됨
```

조용히 틀린다. 이 프로젝트에서 가장 경계하는 실패 형태다.

## 결정

**1. 보드 접근은 SSH 별칭으로만 한다.**

```text
npuforge-k   king
npuforge-q   queen
npuforge-j   jack
```

**2. 문서와 스크립트에 IP 를 직접 쓰지 않는다.** IP 는 `~/.ssh/config`
한 곳에만 있다.

**3. preflight 의 **첫 번째** 검사가 별칭 ↔ hostname 일치다.** 붙은 곳이
정말 그 보드인지 확인한다.

**4. SSH host key 를 노드마다 다르게 유지한다.**

## 근거

### 단일 출처

IP 는 바뀐다. DHCP 임대가 갱신되거나, 네트워크를 재구성하거나, 스위치를
바꾸면 달라진다. 그때마다 문서 여러 곳을 고쳐야 한다면 반드시 하나가 남는다.

`~/.ssh/config` 는 **접속에 실제로 쓰이는 값**이라 틀리면 바로 드러난다.
문서의 IP 는 아무도 안 쓰기 때문에 틀린 채로 오래 남는다.

### 별칭도 틀릴 수 있다 — 그래서 검사한다

별칭 자체는 IP 를 가리키므로, IP 가 재배치되면 별칭이 엉뚱한 보드를 가리킬
수 있다. 그래서 preflight 1번 검사가 필요하다.

```text
ssh npuforge-k hostname   →  "king" 이어야 한다
```

이 검사가 **연결 실패 검사보다 우선**이다. 연결 실패는 시끄럽게 실패하지만,
잘못된 매핑은 조용히 성공하기 때문이다.

### host key 가 같으면 구분이 안 된다

현재 `queen` 과 `jack` 의 SSH host key 가 동일하다. 클론하거나 이미지를
복사해서 생긴 문제로 보인다.

이 상태에서는 **IP 가 바뀌어 다른 보드에 붙어도 SSH 가 경고하지 않는다.**
host key 는 "이 서버가 아까 그 서버가 맞는가" 를 확인하는 장치인데, 둘이
같으면 그 기능이 죽는다.

이미 노드 오판을 한 번 겪었으므로 방치하면 안 된다. **미해결 과제로
`docs/TODO.md` 에 남아 있다.**

```bash
ssh npuforge-j 'sudo rm -f /etc/ssh/ssh_host_* && sudo ssh-keygen -A && sudo systemctl restart ssh'
ssh-keygen -R npuforge-j   # PC 의 known_hosts 정리
```

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 문서의 IP 를 잘 관리한다 | 이미 실패했다. 쓰이지 않는 값은 낡는다 |
| 고정 IP 를 부여한다 | 그래도 문서에 복제되면 같은 문제. 별칭은 그 위에서도 유효하다 |
| mDNS / hostname 으로 접근 | 환경에 따라 안 되는 경우가 있고, 별칭이 그 위 계층이라 함께 쓸 수 있다 |
| 별칭만 쓰고 검사는 생략 | 별칭이 엉뚱한 보드를 가리키는 경우를 못 잡는다 |

## 결과

**얻은 것**

- IP 의 단일 출처가 생겼다
- 잘못된 노드에 측정이 귀속되는 사고를 preflight 가 잡는다

**잃은 것 / 대가**

- 새 사람이 저장소를 받으면 `~/.ssh/config` 를 직접 만들어야 한다.
  재현 절차에 전제로 명시했다

**새로 생긴 제약**

- **문서에서 IP 를 보면 의심한다.** 남아 있다면 그건 낡았을 가능성이 높다
- `queen`·`jack` host key 재생성 전까지는 IP 재배치 시 경고 없이 엉뚱한
  보드에 붙을 수 있다. **알려진 위험**이다

## 뒤집힌다면

없다. IP 를 문서에 다시 박을 이유가 생기지 않는다.

---

<a id="adr-020"></a>

# ADR-020. `worker_count = 8` 로 하고 `core_mask` 는 설정하지 않는다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-10 |
| **관련** | [ADR-007](#adr-007), [ADR-011](#adr-011), `docs/discuss.md` §4 |

---

## 한 줄 요약

> 워커 8개가 4개보다 **+27%** 다. NPU 코어를 손으로 배정하는
> `core_mask` 는 8스레드에서 **+0.1%** — 사실상 없다. `CORE_AUTO` 의 분배가
> 이미 균등하다. **손대지 않는 것이 결론이다.**

## 배경

RK3576 의 NPU 는 코어가 2개다. RKNN 은 어느 코어를 쓸지 지정하는
`core_mask` 를 제공한다.

측정 초기에 "Core1 점유율이 38% 밖에 안 된다" 는 관찰이 있었고, 두 번째
코어가 놀고 있다는 가설이 나왔다. 코어를 명시적으로 배정하면 처리량이
올라갈 것으로 봤다.

**그런데 그 38% 가 실제로 처리량에 기여하는지 확인한 적이 없었다.**
점유율 숫자만 봤다.

## 근거

### 대조군을 넣었다

이전 측정에는 "코어 하나만 쓰면 얼마나 나오는가" 가 없었다. 그걸 넣어야
두 번째 코어의 기여를 판정할 수 있다.

```text
측정 조건: queen, FP16, 스레드당 200회, 워밍업 4초 후 샘플링
```

| 스레드 | AUTO | ALTERNATE | CORE_0_1 | **CORE_0_ONLY** |
|---:|---:|---:|---:|---:|
| 1 | 16.7 | 16.7 | **18.2** | 16.5 |
| 2 | 36.2 | 36.5 | 36.4 | 26.4 |
| 4 | 52.4 | **57.1** | 48.5 | 38.5 |
| 8 | 72.9 | **73.0** | 64.5 | **48.2** |

### 발견 1. 두 번째 코어는 실제로 기여한다 — 다만 1.51배다

```text
8스레드   단일 코어 48.2  →  두 코어 73.0 inf/s   =  1.51배
```

38% 점유율은 장식이 아니었다. **그런데 2배가 아니라 1.51배다.** 코어를 두
배로 늘려도 처리량은 절반만 는다. 코어 밖에 공유 자원이 있다는 뜻이고,
이후 확인된 "제출 경로 직렬화" 와 맞는다.

### 발견 2. 명시 배정은 이득이 없다

```text
4스레드   52.4 → 57.1   +9.0%
8스레드   72.9 → 73.0   +0.1%
```

4스레드에서만 오르고 8스레드에서 사라진다. 게다가 4스레드 개선분을 뜯어보면
대부분이 `outputs_get` 감소(13.6 → 10.0 ms)라 **코어 배정 효과인지 측정
노이즈인지 분리되지 않는다.**

`AUTO` 의 분배는 이미 균등하다 — 8스레드에서 Core0 39% / Core1 37%.
런타임 스케줄러가 제 역할을 하고 있어 수동 개입의 여지가 없다.

### 발견 3. `CORE_0_1` 은 오히려 손해다

```text
8스레드   72.9 → 64.5   -11.5%
```

모든 스레드가 두 코어를 함께 쓰게 하면 더 느려진다.

## 결정

**1. `worker_count = 8` 을 실장비 기본값으로 한다.** 4 대비 +27% 이고,
8에서 아직 꺾이지 않았다.

**2. `core_mask` 를 설정하지 않는다.** `CORE_AUTO` 에 맡긴다.

**3. 설정 기본값은 1 로 두고, 실장비 설정에서 명시적으로 8 을 준다.**
기본값 1 은 백엔드를 모르는 상태에서의 안전값이다.

**4. `worker_count` 는 컨텍스트 수와 직결된다는 것을 명시한다.**
백엔드가 이 수만큼 RKNN 컨텍스트를 만든다
([ADR-007](#adr-007)).

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| `core_mask = ALTERNATE` | 8스레드에서 +0.1%. 설정 항목만 늘고 이득이 없다 |
| `core_mask = CORE_0_1` | -11.5%. 명확히 손해 |
| `worker_count = 4` | 8 대비 -27% |
| `worker_count` 를 더 키운다 | **아직 꺾이는 지점을 못 찾았다.** 다만 컨텍스트가 그만큼 늘어나므로 메모리 확인이 먼저다 |

## 결과

**얻은 것**

- 튜닝 항목이 하나 줄었다. **설정하지 않기로 한 것도 결정이다**
- NPU 2코어의 실제 기여가 1.51배라는 수치를 확보 — 이후 병목 분석의 근거

**잃은 것 / 대가**

- 4스레드 조건에서는 +9% 를 포기한다. 다만 실장비는 8워커로 돌린다

**새로 생긴 제약**

- **`worker_count` 를 늘리면 RKNN 컨텍스트가 그만큼 늘어난다.** 메모리 여유를
  확인하지 않고 키우면 안 된다. 현재 컨텍스트당 메모리 증가량은 **측정하지
  않았다**
- 8이 상한이라는 근거는 없다. "8에서 아직 안 꺾였다" 가 정확한 표현이다.
  `MAX_THREADS` 를 넓혀 다시 재는 것이 미확정 항목으로 남아 있다

## 뒤집힌다면

- **INT8 기준으로 다시 재면 최적값이 달라질 수 있다.** 위 스윕은 FP16 이다.
  INT8 은 건당 시간이 짧아 최적 동시성이 다를 수 있다. **아직 확인하지 않았다**
- **`MAX_THREADS` 를 넓혀 12·16 을 재면** 더 좋은 값이 나올 수 있다
- 메모리가 부족해지면 8을 낮춰야 한다

---

<a id="adr-021"></a>

# ADR-021. The node does no postprocessing (NMS) and returns raw tensors

*[한국어 원문](021-no-node-side-postprocessing.ko.md)*

| | |
|---|---|
| **Status** | provisional |
| **Date** | 2026-08-12 |
| **Related** | [ADR-012](#adr-012), [ADR-014](#adr-014), [ADR-013](#adr-013) |

---

## In one line

> The node returns **the model's 9 output tensors as they are**, not detections.
> The response gets larger, and in exchange **the node's CPU load stays out of
> what is being measured.** Node-side postprocessing is ultimately right, but not
> now.

## Context

The output of a detection model like YOLOv8n is not directly usable.

```text
NPU output   9 tensors (box candidates and class scores per grid cell)
                 |  postprocessing (NMS: resolve overlapping boxes, apply thresholds)
final result "1 person, 2 cars" - a few KB
```

The question is **where that postprocessing happens.**

| | Response size | Node CPU |
|---|---|---|
| Postprocess on the node | a few KB | goes up |
| Postprocess on scheduler/client | 1.2 MB | unchanged |

## Decision

**The node does no postprocessing.** It returns the raw tensors bundled into a
single blob ([ADR-012](#adr-012)).

**The status is left as "provisional".** Not because it is best, but because it
is **the right choice under current conditions.**

## Rationale

### 1. Node CPU is already the bottleneck

Throughput falls 27% over 300 seconds of sustained load, and the cause is not
the NPU but **CPU thermal throttling**. The A72 is downgraded from 2208 to
816 MHz ([ADR-013](#adr-013)).

Adding NMS on top increases CPU load further. That would destabilise the very
value this project is trying to measure.

```text
now:              measuring NPU scaling efficiency while the CPU interferes  <- already a problem
with postprocess: making it use more CPU and measuring the same value        <- worse
```

### 2. It adds another measurement variable

**NMS cost varies with the input.** An image with many detections takes longer
and one with few finishes quickly. Doing it on the node makes per-node
processing time vary with input content.

In an experiment measuring three-node scaling efficiency, that variable is noise.

### 3. It is not implemented

The simplest reason. There is no NMS implementation, and building one brings
verification (accuracy comparison) with it. Not a priority while waiting on
equipment.

### 4. The network problem was solved another way

The cost of returning raw tensors is response size. With `want_float=1` the
response was 3.96× the request and even 10G was insufficient.

That was **solved with `want_float=0` rather than postprocessing.** The response
became a quarter of its size and 3-node RX went from 18.38 to 4.60 Gbps. It fits
inside 10G.

So **there is no immediate pressure to postprocess.**

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Run NMS on the node | **This is ultimately the right answer.** But it worsens the CPU bottleneck and adds a measurement variable. Unimplemented |
| NMS on the scheduler | The scheduler is a single point, so three nodes' worth of postprocessing piles up in one place. The scheduler becomes the bottleneck |
| Compress the response | Compression CPU enters the inference path. CPU is already the bottleneck |
| Client-side postprocessing | This is the current approach. The bench tool and comparison scripts understand the blob |

## Consequences

**Gained**

- What the node does is narrow and uniform: **preprocess → NPU → serialize**
- Per-node processing time depends less on input content
- What is being measured stays clean

**Lost / the cost**

- The response is 1.2 MB, where sending only the detections would end at a few KB
- **The receiver has to understand the blob.** Changing the format means fixing
  three places together (blob.rs / dump_output_test.c / compare_detections.py)
- As a real-world API it is unfriendly. It is not an API that "gives you
  detections"

**New constraints introduced**

- The client is responsible for both dequantization and NMS
- The network budget is tied to response size. In experiments that increase
  input size (S6), the response grows with it

## What would overturn this

**This ADR is scheduled to be overturned.**

- **If the CPU bottleneck is resolved** (cooling condition B, or preprocessing
  optimization), there is room to move postprocessing to the node
- **If a real-world API becomes a requirement**, returning raw tensors is hard to
  sustain
- **If the network fills up again in experiments that increase input size**,
  postprocessing becomes the most effective means — the response shrinks to a
  few KB and RX effectively disappears

**What must be measured alongside** when overturning it: sustained throughput
and the timing of CPU clock downgrade, before and after putting postprocessing
on the node. Do not judge from response size alone.

---

<a id="adr-022"></a>

# ADR-022. 문서마다 규범 영역을 정하고, 값이 다르면 규범 문서를 따른다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-05 |
| **관련** | [ADR-002](#adr-002), `docs/00-PRD.md` §0 |

---

## 한 줄 요약

> 같은 값이 여러 문서에 적히면 반드시 하나가 낡는다. 그래서 **영역마다
> 규범 문서를 하나씩** 정하고, 값이 다르면 그 문서를 따른다. 나머지 문서는
> 복제하지 말고 참조한다.

## 배경

이 저장소는 문서가 많다. PRD, TECHSPEC, 하드웨어, 개발요구사항,
environment-matrix, RESULTS, TODO, discuss, board-worklog.

같은 숫자가 여러 곳에 나온다. 예를 들어 "노드당 처리량 157.2 inf/s" 는
RESULTS 에도, TODO 에도, board-worklog 에도, environment-matrix 에도 나온다.

**하나를 고치면 나머지가 낡는다.** 실제로 겪었다.

```text
want_float=0 전환 후
  RESULTS §2.2  갱신됨       "INT8 +17.3%"
  RESULTS §5    안 갱신됨    "INT8 처리량 영향은 미측정"   ← 같은 문서 안에서 모순
  TECHSPEC §3.2 안 갱신됨    폐기된 네트워크 계산이 그대로
```

## 결정

**1. 영역마다 규범 문서를 하나씩 정한다.**

| 영역 | 규범 문서 |
|---|---|
| 목표, 비목표, 기능 요구사항, 성공 기준 | `00-PRD.md` |
| 저장소 구조, 프로토콜, 설정 스키마, 스케줄링 알고리즘, 오류 코드 | `01-TECHSPEC.md` |
| 물리 구성, 네트워크, 전원, 냉각, 실험 조건 | `02-HARDWARE-SETUP.md` |
| 개발환경, 도구, 배포 자동화, 라이선스 | `03-DEVELOPMENT-REQUIREMENTS.md` |
| 버전 조합 및 해시 고정 | `environment-matrix.md` |

**2. 값이 서로 다르면 규범 문서를 따른다.**

**3. 복제하지 말고 참조한다.** PRD 는 "왜" 와 "무엇을" 만 다룬다. 계산식,
크레이트 이름, 설정 키, 식별자 문자열은 PRD 에 쓰지 않고 TECHSPEC 을
가리킨다.

**4. 성격이 다른 문서는 규범 대상이 아니다.**

| 문서 | 성격 |
|---|---|
| `discuss.md` | 시간순 논의. 뒤 절이 앞 절을 정정한다 |
| `board-worklog.md` | 작업 이력. 틀린 가설도 보존한다 |
| `RESULTS.md` | 결과 모음. 값의 최종 기준은 environment-matrix |
| `TODO.md` | 현재 할 일 |
| `adrs/` | 결정과 근거 |

## 근거

### 복제하지 않는 것이 유일한 방법이다

정합성을 유지하는 방법은 둘뿐이다.

```text
1. 복제해 놓고 고칠 때마다 전부 찾아 고친다   → 반드시 하나를 빠뜨린다
2. 애초에 한 곳에만 둔다                      → 낡을 곳이 없다
```

이 프로젝트는 1번으로 이미 실패했다. `want_float=0` 전환 하나에
`RESULTS.md`·`TECHSPEC`·`environment-matrix`·`TODO`·`board-worklog` 다섯
문서가 관련됐고, 한 번의 sync 로 다 잡히지 않았다.

### 시간순 문서를 규범에서 뺀 이유

`discuss.md` 는 **틀린 결론을 일부러 남긴다.** 5절의 "+5.4%" 는 지금 기준으로
낡았지만, 12절이 왜 그것을 정정했는지 이해하려면 5절이 그대로 있어야 한다.

이런 문서를 규범으로 삼으면 앞 절을 읽은 사람이 폐기된 값을 인용하게 된다.
그래서 **시간순 문서는 근거 자료지 기준이 아니다.**

### ADR 이 이 구조를 보완한다

규범 문서는 "지금 값이 무엇인가" 에 답한다. 시간순 문서는 "무슨 일이
있었나" 에 답한다. **"왜 그렇게 정했나" 에 답하는 자리가 비어 있었다.**

`adrs/` 가 그 자리다. 규범 문서에서 값을 가져오고, 시간순 문서에서 경위를
가져와 결정 단위로 다시 묶는다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 문서를 하나로 합친다 | 만 줄이 넘는다. 용도가 다른 독자를 한 문서로 감당할 수 없다 |
| 우선순위 없이 관리 | 값이 충돌했을 때 무엇이 맞는지 판정할 방법이 없다 |
| 값을 자동 생성 | 일부는 가능하지만(테스트 수 등) 측정값은 사람이 조건과 함께 판단해 적어야 한다 |

## 결과

**얻은 것**

- 값이 충돌했을 때 판정 기준이 있다
- 각 문서의 역할이 명확하다
- 고칠 곳을 특정할 수 있다

**잃은 것 / 대가**

- 한 주제를 알려면 문서를 오가야 한다. **이 불편함이 `adrs/` 를 만든 직접적
  이유다**
- 규범 문서가 어디인지 기억해야 한다

**새로 생긴 제약**

- **복제를 발견하면 지우고 참조로 바꾼다.** 편의상 값을 옮겨 적고 싶은
  순간이 계속 온다
- ADR 도 값을 인용한다. 인용한 값이 낡을 수 있으므로 **측정 조건과 출처를
  함께** 적는다

## 뒤집힌다면

문서가 더 늘어나면 규범 영역을 추가한다. 줄어들면 통합한다. 원칙 자체는
바뀌지 않는다.

---

<a id="adr-023"></a>

# ADR-023. CPU governor 를 `performance` 로 고정한다 — 단, 근거의 범위를 명시한다

| | |
|---|---|
| **상태** | 잠정 |
| **날짜** | 2026-08-12 |
| **관련** | [ADR-013](#adr-013), [ADR-002](#adr-002), `docs/discuss.md` §11·§12 |

---

## 한 줄 요약

> `ondemand` → `performance` 로 바꾸니 **+7%** 다. 그래서 고정했다.
> **그런데 그 +7% 는 120초 측정이다.** 지속 부하에서는 `performance` 가 더
> 빨리 뜨거워져 불리할 수 있고, **아직 확인하지 않았다.**

## 배경

Linux 의 CPU governor 는 부하에 따라 클럭을 조절하는 정책이다.

| governor | 동작 |
|---|---|
| `ondemand` | 부하가 있을 때만 클럭을 올린다. 기본값 |
| `performance` | 항상 최대 클럭을 유지한다 |

추론 한 건은 `입력 설정(CPU) → NPU → 출력 취득(CPU)` 이라 CPU 클럭이
처리량에 직접 반영된다. 그래서 governor 가 변수가 된다.

## 결정

**1. 세 노드의 governor 를 `performance` 로 고정한다.** systemd 유닛으로
영구화해 재부팅해도 유지된다 (`scripts/set-cpu-governor.sh`).

**2. preflight 가 매 측정 전에 확인한다.**

**3. 기존 수치의 기준을 명시한다.** 2026-08-11 이전 측정은 전부 `ondemand`
기준이다.

```text
ondemand      FP16 79.0 / INT8 146.2 inf/s
performance   FP16 84.3 / INT8 157.2 inf/s
```

**4. +7% 라는 결론의 범위를 문서에 못 박는다.** "짧은 측정에서의 이득" 으로만
읽는다.

## 근거

### 왜 고정하는가

값 자체보다 **조건을 통일하는 것**이 중요하다. governor 가 노드마다 다르거나
run 마다 다르면 3노드 비교가 무의미해진다.

`performance` 를 고른 이유는 두 가지다.

- 120초 측정에서 +7%
- **동작이 단순하다.** `ondemand` 는 부하 패턴에 따라 클럭이 오르내려서,
  측정값의 분산이 governor 의 판단에서 오는지 다른 데서 오는지 분리하기
  어렵다

두 번째가 더 중요하다. 재현성 관점에서 예측 가능한 쪽이 낫다.

## ⚠️ 이 결정의 근거가 약한 부분

**+7% 는 120초 측정이다.** 그 구간은 CPU 가 아직 완전히 강등되기 전이다.

지속 부하에서 실제로 일어나는 일은 이렇다.

```text
        NPU온도   cpu4(A72)   cpu0(A53)
 +15s   86.8°C    2208 MHz    2016 MHz
 +30s   90.4°C    1416 MHz    1200 MHz
 +60s   87.8°C     816 MHz     600 MHz   ← 63~70% 강등
+120s   87.8°C     816 MHz     600 MHz
```

**`performance` 는 유휴 상태에서도 최대 클럭을 유지한다.** 그래서 부하
시작 시점의 열 여유가 `ondemand` 보다 적다. 더 빨리 뜨거워지고 더 일찍
강등될 수 있다.

즉 **짧게 재면 `performance` 가 이기고, 길게 재면 질 수도 있다.**
그리고 우리가 재려는 것은 **지속 처리량**이다.

**측정하지 않았다.** `ondemand` 와 `performance` 를 동일한 300초 조건에서
비교해야 한다. 그 전까지 이 ADR 의 상태는 **「잠정」**이다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| `ondemand` 유지 | 클럭이 오르내려 측정 분산의 원인을 분리하기 어렵다 |
| governor 를 실험 변수로 둔다 | **결국 그렇게 해야 한다.** 다만 지금은 다른 조건을 고정해야 해서 하나를 골랐다 |
| `powersave` 나 고정 주파수 | 이 프로젝트가 재려는 것은 "가능한 최대" 에 가깝다 |
| 온도에 따라 governor 를 바꾼다 | 측정 대상을 측정 중에 바꾸는 것. 해석 불가능해진다 |

## 결과

**얻은 것**

- 세 노드의 조건이 통일됐다
- 재부팅해도 유지된다
- 기존 수치의 기준(`ondemand`)이 명시적으로 기록됐다

**잃은 것 / 대가**

- **2026-08-11 이전 수치와 직접 비교할 수 없다.** 문서에 경고를 달아 두었다
- 지속 부하에서 불리할 가능성을 안고 간다

**새로 생긴 제약**

- 측정값을 인용할 때 **governor 를 반드시 함께 적는다**. "84.3 inf/s" 는
  조건 없이는 무의미한 숫자다
- preflight 가 governor 를 검사한다. 한 노드만 다르면 하드 실패

## 뒤집힌다면

**재검증 계획이 이미 정해져 있다.**

```text
ondemand vs performance, 동일한 300초 조건, 3노드
비교 항목: 정상 상태 처리량, CPU 강등 시점, 평균 온도
```

`performance` 의 300초 처리량이 `ondemand` 보다 낮으면 이 결정을 뒤집는다.
그 결과 자체도 유효한 산출물이다 —
**"엣지에서는 최대 클럭 고정이 오히려 손해"** 는 공개 가치가 있는 결론이다.

---

<a id="adr-024"></a>

# ADR-024. 오류를 `NPF-xxxx` 코드 체계로 고정하고 외부 API 에서 안정적으로 유지한다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 |
| **관련** | [ADR-008](#adr-008), [ADR-026](#adr-026) |

---

## 한 줄 요약

> 오류를 `NPF-1302` 같은 **안정된 코드**로 표현한다. 번호대가 오류의 성격을
> 나타내고, 그 성격이 **재시도 여부를 결정**한다. 메시지 문자열은 바뀔 수
> 있지만 코드는 바뀌지 않는다.

## 배경

이 시스템에서 오류는 여러 경계를 넘는다.

```text
노드 백엔드  →  노드 에이전트  →  gRPC  →  스케줄러  →  gRPC  →  클라이언트
                                              ↓
                                        재시도할지 판단
```

스케줄러가 재시도 여부를 판단하려면 **노드가 보낸 오류가 무엇인지** 알아야
한다. 메시지 문자열로 판단하면 문구를 고칠 때마다 판단 로직이 깨진다.

## 결정

**1. 번호대로 성격을 나눈다.**

| 번호대 | 성격 | 예 |
|---|---|---|
| 1000 | 요청 자체의 문제 | `NPF-1001 INVALID_REQUEST`, `NPF-1002 PAYLOAD_TOO_LARGE` |
| 1100 | 모델 문제 | `NPF-1101 MODEL_NOT_FOUND`, `NPF-1102 MODEL_VERSION_MISMATCH` |
| 1200 | 스케줄링 문제 | `NPF-1201 NO_AVAILABLE_NODE`, `NPF-1202 DEADLINE_UNSATISFIABLE` |
| 1300 | 노드 문제 | `NPF-1301 NODE_TIMEOUT`, `NPF-1302 NODE_UNAVAILABLE`, `NPF-1303 NODE_OVERLOADED` |
| 1400 | 백엔드 문제 | `NPF-1401 BACKEND_ERROR`, `NPF-1402 INFERENCE_FAILED` |
| 1500 | 내부 오류 | `NPF-1501 INTERNAL_ERROR` |

**2. 열거형 하나로 정의하고 문자열 변환을 양방향으로 제공한다.**

```rust
pub const fn as_str(self) -> &'static str { ... }   // NPF-1302
pub fn from_str_code(s: &str) -> Option<Self>       // 모르면 None
```

역방향이 필요한 이유: **노드가 보낸 코드를 스케줄러가 재시도 판정에 써야
한다.**

**3. 모르는 코드는 `None` 이고, 호출자가 보수적인 기본값을 정한다.**
새 코드가 추가된 노드와 옛 스케줄러가 섞여도 조용히 오작동하지 않는다.

**4. 코드는 외부 API 에서 안정적으로 유지한다.** 번호를 재사용하지 않고,
의미를 바꾸지 않는다.

## 근거

### 재시도 판정이 코드에 달려 있다

| 재시도 가능 | 재시도 불가 |
|---|---|
| 네트워크 연결 실패 | 잘못된 입력 |
| `NPF-1301` 노드 타임아웃 | 지원하지 않는 모델 |
| `NPF-1302` 노드 사용 불가 | 지원하지 않는 입력 형식 |
| `NPF-1303` 노드 과부하 | 모델 버전 불일치 |
| 일시적 런타임 오류 | payload 크기 초과 |

**1300 번대는 재시도, 1000·1100 번대는 재시도 불가** — 번호대만 봐도 대략
갈린다. 잘못된 입력을 다른 노드에 다시 보내 봐야 똑같이 실패한다.

### 문자열 매칭을 코드 여러 곳에 흩지 않는다

`SchedulingPolicyKind` 와 같은 원칙이다
([ADR-009](#adr-009)). 파싱을 한 곳에 모아 두면
표기 흔들림이 생길 자리가 없다.

### 진단에 실제로 쓰였다

Mock 3노드 통합 테스트에서 "전 노드 사망" 케이스의 기대값이
**`NPF-1302` + 시도한 노드 목록**이다. 코드가 안정적이라 테스트가 이것을
단언할 수 있다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| gRPC status code 만 사용 | 종류가 부족하고 도메인 의미를 못 담는다. `UNAVAILABLE` 하나로 노드 사망·과부하·타임아웃이 뭉개진다 |
| 메시지 문자열로 판단 | 문구를 고치면 로직이 깨진다. 다국어도 불가능 |
| HTTP 상태 코드 | 내부 RPC 가 gRPC 라 어울리지 않는다. 관리 API 에서는 함께 쓸 수 있다 |
| 오류를 계층별로 다르게 정의 | 경계를 넘을 때마다 변환이 필요하고, 변환 과정에서 정보가 사라진다 |

## 결과

**얻은 것**

- 재시도 판정이 코드 하나로 결정된다
- 로그·메트릭·테스트가 같은 식별자를 쓴다
- gRPC 와 REST 양쪽에서 같은 오류 표현이 가능하다

**잃은 것 / 대가**

- 코드를 한 번 공개하면 **바꿀 수 없다.** 추가만 가능하다
- 새 오류마다 번호를 정해야 한다

**새로 생긴 제약**

- **번호를 재사용하지 않는다.** 폐기해도 자리를 비워 둔다
- 새 코드를 추가할 때 **재시도 가능 여부를 함께 정해야 한다.** 정하지 않으면
  호출자가 보수적 기본값(재시도 불가)으로 처리한다

## 뒤집힌다면

번호대 구획이 부족해지면 확장한다(예: 1600 번대). **기존 번호의 의미는
바꾸지 않는다.**

---

<a id="adr-025"></a>

# ADR-025. 하트비트가 실패하면 곧바로 재등록한다 — 등록은 멱등하게 만든다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-11 |
| **관련** | [ADR-003](#adr-003), [ADR-016](#adr-016), [ADR-027](#adr-027) |

---

## 한 줄 요약

> 노드 입장에서 **일시적 네트워크 오류와 스케줄러 재시작은 구분할 수 없다.**
> 그래서 구분하려 애쓰지 않고, 하트비트가 실패하면 무조건 재등록한다.
> 등록이 멱등이라 헛수고가 손해로 이어지지 않는다.

## 배경

노드는 주기적으로 하트비트를 보낸다(기본 1~2초). 이게 실패하면 두 경우다.

```text
경우 A. 네트워크가 잠깐 끊겼다      → 잠시 뒤 다시 되면 그만
경우 B. 스케줄러가 재시작했다        → 스케줄러의 노드 목록이 비었다
                                       재등록하지 않으면 영영 안 들어간다
```

**노드는 둘을 구분할 수 없다.** 둘 다 "응답이 없다" 로 똑같이 보인다.

구분하려면 스케줄러의 인스턴스 식별자 같은 것을 주고받아야 하는데, 그러면
스케줄러가 그 값을 유지·전파해야 하고 상태가 늘어난다.

## 결정

**1. 하트비트 실패를 곧바로 재등록으로 전환한다.** 구분하지 않는다.

**2. 등록을 멱등하게 만든다.** 같은 노드가 여러 번 등록해도 결과가 같다.

**3. 스케줄러가 재등록을 요구할 수 있게 한다.** 응답에 `must_reregister`
플래그를 둔다. 스케줄러가 모르는 노드에게서 하트비트를 받으면 이걸 켠다.

**4. 최초 등록에는 백오프 재시도를 둔다.** 노드가 스케줄러보다 먼저 뜨는
경우가 정상이기 때문이다.

## 근거

### 더 비싼 쪽을 택했다

두 선택지의 비용을 비교하면 이렇다.

| | 비용 |
|---|---|
| 재등록했는데 필요 없었다 | RPC 한 번. 멱등이라 상태 변화 없음 |
| 재등록 안 했는데 필요했다 | **노드가 클러스터에서 영구히 빠진다** |

비대칭이 크다. 싼 쪽 실수를 반복하는 편이 낫다.

### 실측: 1.3초

실제 프로세스 4개(스케줄러 + 노드 3)로 확인했다.

```text
스케줄러를 죽인다  →  다시 띄운다  →  세 노드가 약 1.3초 안에 스스로 복귀
```

이 값이 [ADR-003](#adr-003) 의 "단일 장애점을 받아들이되
복구를 싸게 만든다" 를 실제로 뒷받침한다. **스케줄러 이중화 없이도 재시작
비용이 1.3초라면, 실험 장비로서는 충분하다.**

### 멱등성이 이 결정의 전제다

등록이 멱등이 아니면 이 설계가 성립하지 않는다. 중복 등록이 노드를 두 개로
만들거나 상태를 리셋하면, 재등록을 남발하는 순간 클러스터가 망가진다.

그래서 **등록은 "이 노드가 존재한다" 를 선언하는 것**이지 "새로 추가한다"
가 아니다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 스케줄러 인스턴스 ID 로 재시작 감지 | 상태가 늘고, 그 값이 틀리면 같은 문제가 다시 생긴다. 얻는 것이 RPC 몇 번 |
| 하트비트 실패 N 회 후 재등록 | 복구가 N 배 느려진다. 얻는 것은 RPC 절약뿐 |
| 스케줄러가 노드 목록을 디스크에 저장 | 재시작 시 복원되지만 낡은 정보일 수 있다. 노드가 사라졌는데 있다고 믿는다 |
| 노드가 재등록하지 않고 스케줄러가 발견 | 스케줄러가 노드를 몰라서 못 찾는다. 발견 메커니즘(브로드캐스트 등)이 또 필요하다 |

## 결과

**얻은 것**

- 스케줄러 재시작 복구 1.3초
- 스케줄러가 노드 목록을 영속화하지 않아도 된다
- 실패 처리 경로가 하나다 (구분 없음 = 분기 없음)

**잃은 것 / 대가**

- 네트워크가 불안정하면 불필요한 등록 RPC 가 늘어난다. 멱등이라 무해하지만
  트래픽은 발생한다
- "왜 재등록했는지" 가 로그에 남지만 원인(순단인지 재시작인지)은 알 수 없다

**새로 생긴 제약**

- **등록 처리는 반드시 멱등을 유지해야 한다.** 여기에 부작용을 추가하면
  전체 설계가 무너진다
- 재등록 이벤트만으로는 **보드 리셋과 프로세스 재시작을 구분할 수 없다.**
  그건 `boot_id` 의 몫이다 ([ADR-016](#adr-016))

## 뒤집힌다면

- **노드가 수십 대가 되면** 동시 재등록이 스케줄러에 몰릴 수 있다.
  그때는 지터를 넣는다
- **재등록 비용이 커지면**(등록 시 모델 목록 전송 등) 구분할 이유가 생긴다.
  현재 등록 메시지는 가볍다

---

<a id="adr-026"></a>

# ADR-026. 재시도는 반드시 다른 노드로 보내고, 백오프를 짧게 유지한다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 |
| **관련** | [ADR-024](#adr-024), [ADR-009](#adr-009), `docs/01-TECHSPEC.md` §12 |

---

## 한 줄 요약

> 실패한 노드에 다시 보내지 않는다. **실패 노드를 후보에서 일시 제외하고
> 다른 노드를 고른다.** 재시도는 기본 1회, 백오프는 10~100 ms — 실시간
> 추론이라 긴 exponential backoff 를 쓰지 않는다.

## 배경

추론 요청이 실패했을 때 선택지는 셋이다.

```text
1. 그냥 실패로 돌려준다
2. 같은 노드에 다시 보낸다
3. 다른 노드에 보낸다
```

추론 요청은 **부작용이 없다.** 같은 입력을 두 번 처리해도 상태가 바뀌지
않는다. 그래서 재시도가 안전하다 — 이것이 전제다.

## 결정

**1. 재시도 가능 여부를 오류 코드로 판정한다.**

| 재시도 가능 | 재시도 불가 |
|---|---|
| 네트워크 연결 실패 | 잘못된 입력 |
| 노드 타임아웃 (`NPF-1301`) | 지원하지 않는 모델 |
| 노드 사용 불가 (`NPF-1302`) | 지원하지 않는 입력 형식 |
| 노드 과부하 (`NPF-1303`) | 모델 버전 불일치 |
| 일시적 런타임 오류 | payload 크기 초과 / 인증 실패 |

**2. 재시도 시 실패한 노드를 후보에서 일시 제외한다.** 그 다음 정책이
남은 후보 중에서 고른다.

**3. 기본값을 짧게 잡는다.**

```text
최대 재시도       1회
전체 요청 timeout  5초
retry backoff     10~100 ms
```

**4. 긴 exponential backoff 를 쓰지 않는다.**

**5. 시도한 노드 목록을 오류에 담는다.** 전부 실패하면 `NPF-1302` 와 함께
어느 노드를 시도했는지 돌려준다.

## 근거

### 같은 노드에 다시 보내면 안 되는 이유

실패 원인이 노드에 있으면 **다시 보내도 똑같이 실패한다.**

```text
노드가 죽었다      → 다시 보내도 죽어 있다
노드가 과부하다    → 다시 보내면 더 과부하가 된다   ← 더 나쁘다
노드가 뜨겁다      → 다시 보내면 더 뜨거워진다
```

특히 `NPF-1303` 과부하는 재시도가 **문제를 악화**시킨다. 이미 큐가 찬 노드에
같은 요청을 또 넣는 셈이다.

### 왜 백오프가 짧은가

이건 배치 작업이 아니라 **실시간 추론**이다. 클라이언트는 지금 답을 기다리고
있다.

```text
exponential backoff (1s, 2s, 4s...)   →  성공해도 이미 늦었다
짧은 backoff (10~100ms)               →  다른 노드가 살아 있으면 거의 안 늦는다
```

전체 요청 타임아웃이 5초인데 백오프에 4초를 쓰면 재시도할 시간이 없다.

### 왜 재시도가 1회인가

노드가 3대다. 한 번 실패하고 다른 노드에서도 실패하면, 세 번째를 시도할
이유가 약하다 — 공통 원인(모델 문제, 요청 문제)일 가능성이 높아진다.

그리고 재시도가 늘수록 **장애 시 지연 분포가 오염된다.** S4 장애 대응 실험에서
재시도 횟수가 많으면 "장애 시 지연" 이 재시도 정책의 함수가 되어 버린다.

### 시도한 노드 목록이 필요한 이유

전부 실패했을 때 "아무 노드도 없다" 만으로는 진단이 안 된다. 어느 노드를
시도했고 각각 왜 실패했는지가 있어야 원인을 좁힌다.

Mock 3노드 통합 테스트의 "전 노드 사망" 케이스가 이걸 단언한다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 같은 노드에 재시도 | 원인이 노드에 있으면 무의미하고, 과부하는 악화시킨다 |
| exponential backoff | 실시간 추론에 맞지 않는다. 성공해도 늦다 |
| 재시도 3회 이상 | 지연 분포가 재시도 정책에 지배된다. 노드가 3대뿐이라 실익도 적다 |
| 재시도 안 함 | 노드 하나가 잠깐 흔들려도 클라이언트가 실패를 본다. 장애 허용이 목표 중 하나다 |
| 모든 오류를 재시도 | 잘못된 입력을 세 노드에 돌려가며 실패시킨다. 낭비이고 오류 원인만 흐려진다 |

## 결과

**얻은 것**

- 노드 하나가 죽어도 클라이언트가 성공을 본다 (Mock 테스트 6/6 성공)
- 과부하 노드에 부하가 더 쏠리지 않는다
- 실패해도 진단 정보가 남는다

**잃은 것 / 대가**

- 재시도된 요청은 지연이 늘어난다. 이 값이 지연 분포의 꼬리를 만든다
- **재시도 건수를 결과에 함께 기록해야 한다.** 안 그러면 지연 분포가 왜
  두꺼운지 설명할 수 없다 (벤치 도구가 기록한다)

**새로 생긴 제약**

- 재시도가 성공한 요청도 **한 건**으로 센다. 두 번 처리했다고 처리량을 두 배로
  세면 안 된다
- 실패 요청은 처리량과 노드 몫에서 제외한다
  ([ADR-028](#adr-028))

## 뒤집힌다면

- **노드가 많아지면** 재시도 횟수를 늘릴 여지가 생긴다. 3대에서는 실익이 적다
- **부작용이 있는 요청**(상태를 바꾸는 API)이 추가되면 이 전제가 깨진다.
  그때는 멱등 키가 필요하다. 현재 스케줄러는 짧은 TTL 의 Request ID 캐시로
  중복 제출만 감지하고, 결과 캐시는 v0.1 필수가 아니다

---

<a id="adr-027"></a>

# ADR-027. 노드 상태를 명시적 상태 머신으로 두고, drain 과 disable 을 나눈다

| | |
|---|---|
| **상태** | 확정 (임계치는 초안) |
| **날짜** | 2026-08-06 |
| **관련** | [ADR-009](#adr-009), [ADR-010](#adr-010), [ADR-025](#adr-025) |

---

## 한 줄 요약

> 노드를 "살았다/죽었다" 둘로 보지 않는다. **여덟 상태의 명시적 전이**로
> 관리하고, 특히 **계획된 제외(drain)와 강제 차단(disable)을 다른 것으로**
> 다룬다.

## 배경

노드가 요청을 받을 수 있는지는 이분법이 아니다.

```text
살아 있지만 느리다
살아 있지만 뜨겁다
살아 있지만 오류가 잦다
방금 살아났는데 아직 못 믿겠다
살아 있지만 곧 끌 예정이다
```

전부 다르게 다뤄야 한다. `bool is_alive` 하나로는 표현할 수 없다.

## 결정

**1. 상태를 명시적으로 정의하고 전이 조건을 고정한다.**

```text
Registering
   │ 등록 성공
   ▼
Healthy ──────────────┐
   │ 부하 높음         │ 수동 drain
   ▼                   ▼
Busy                Draining
   │ 오류 증가          │ 큐가 빔
   ▼                    ▼
Degraded            Disabled
   │ 헬스체크 실패
   ▼
Unreachable
   │ 헬스체크 성공
   ▼
Recovering
   │ 연속 성공
   └───────────────→ Healthy
```

**2. `Draining` 과 `Disabled` 를 구분한다.**

| | 뜻 | 진행 중인 요청 |
|---|---|---|
| `Draining` | 새 요청은 안 받지만 **하던 건 끝낸다** | 완료를 기다린다 |
| `Disabled` | 스케줄링에서 완전히 뺀다 | 이미 비어 있다 |

`Draining` → 큐가 비면 → `Disabled` 로 넘어간다.

**3. 임계치를 전부 설정 가능하게 한다.**

```text
Heartbeat interval     2초
Health timeout         1초
연속 실패 3회      →  Unreachable
연속 성공 3회      →  Recovering 에서 Healthy
큐 길이 초과       →  Busy
최근 오류율 10% 초과 →  Degraded
온도 80°C 이상      →  Degraded
온도 90°C 이상      →  스케줄링 제외
```

**4. 상태를 후보 필터와 점수 양쪽에서 쓴다.** 필터는 `is_schedulable()` 로
자격을 보고, ECT 는 `load_factor` 로 **정도**를 본다
([ADR-010](#adr-010)).

## 근거

### drain 을 나눈 이유

측정 중에 노드를 빼야 하는 상황이 있다. 그때 즉시 끊으면 **진행 중이던
요청이 실패로 기록**되고, 그 실패가 오류율 통계에 들어간다.

```text
즉시 차단     진행 중 3건이 실패 → 오류율 상승 → 측정 결과 오염
drain 사용    진행 중 3건 완료 후 조용히 빠짐 → 통계 깨끗
```

S4 장애 대응 실험에서 **의도된 제외**와 **실제 장애**를 구분해야 하는데,
drain 이 없으면 둘이 똑같이 실패로 보인다.

### `Recovering` 을 따로 둔 이유

죽었다 살아난 노드를 바로 `Healthy` 로 올리면, 큐가 비어 있어서 요청이 전부
몰린다. 같은 원인으로 다시 죽는다.

`Recovering` 은 "살아났지만 아직 못 믿는" 상태다. 연속 성공 3회를 채워야
`Healthy` 가 되고, 그동안 ECT 는 `load_factor 0.25` 로 억제한다.

### 온도를 두 단계로 나눈 이유

```text
80°C  →  Degraded          받긴 받되 덜 받는다
90°C  →  스케줄링 제외      아예 안 준다
```

한 단계만 두면 이분법이 된다. 79°C 와 81°C 가 전혀 다르게 취급되면 경계에서
노드가 들락날락한다.

## ⚠️ 임계치는 초안이다

**현재 온도 임계치(80 / 90°C)는 정상 동작 범위와 충돌한다.**

실측에서 지속 부하 시 NPU 온도가 67.5~75.8°C 이고, 부하 프로파일에 따라
86~90°C 까지 올라간 기록도 있다. 즉 **정상 동작 중에 `Degraded` 로 떨어질
수 있다.**

정식 S0 열 측정 후 재설정해야 한다. 그전까지 이 값은 **초안**이고, 알려진
이슈로 `docs/TODO.md` §6 에 올라 있다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| `bool is_alive` 하나 | 느림·뜨거움·복구 중을 표현할 수 없다 |
| drain 없이 즉시 차단 | 진행 중 요청이 실패로 기록되어 통계가 오염된다 |
| `Recovering` 없이 바로 `Healthy` | 복구 직후 전량을 받아 다시 죽는다 |
| 온도 임계치 한 단계 | 경계에서 진동한다 |
| 상태를 정책마다 다르게 해석 | 정책 비교 실험이 무효가 된다 ([ADR-009](#adr-009)) |

## 결과

**얻은 것**

- 계획된 제외와 장애가 구분된다
- 복구 노드 과부하가 구조적으로 억제된다
- 상태 전이가 이벤트로 기록되어 타임라인 재구성이 가능하다

**잃은 것 / 대가**

- 상태가 8개라 전이 조합을 다 검증해야 한다
- 튜닝할 임계치가 8개 늘었다

**새로 생긴 제약**

- **임계치를 바꾸면 실험 조건이 바뀐 것이다.** 결과에 함께 기록해야 한다
- 온도 임계치가 초안이라, S0 전 측정에서는 노드가 예상치 못하게 `Degraded`
  로 떨어질 수 있다. 그 경우 run 해석에 주의해야 한다

## 뒤집힌다면

- **S0 결과로 온도 임계치를 확정한다.** 이건 예정된 변경이다
- **상태가 더 필요해지면** 추가한다. 다만 상태 하나가 늘면 전이 검증이
  비선형으로 늘어난다는 것을 감안한다

---

<a id="adr-028"></a>

# ADR-028. The bench tool judges run validity itself, and prints warnings above the numbers

*[한국어 원문](028-bench-run-validity.ko.md)*

| | |
|---|---|
| **Status** | accepted |
| **Date** | 2026-08-11 |
| **Related** | [ADR-002](#adr-002), [ADR-015](#adr-015), [ADR-016](#adr-016) |

---

## In one line

> Past measurement mistakes are **built into the tool, not written in
> comments.** Warmup excluded, reboot detected via `boot_id`, insufficient
> samples flagged, failures excluded from throughput, percentile interpolation
> forbidden. And **invalidity warnings print above the numbers.**

## Context

`npuforge-bench` is not a new measurement but **a tool**. Yet the rationale for
its entire design comes from earlier failures.

Collecting the measurement mistakes so far, they fall into three kinds.

```text
A. did not check what a metric counts
B. compared values without noticing a condition had changed
C. nearly treated invalid data as valid
```

**Writing "let's be careful" in a comment did not work.** All three happened
while knowing better. So the tool enforces it.

## Decision

**1. Past mistakes are pinned as rules.**

| Past mistake | What the tool does |
|---|---|
| The first inference's latency spikes | Warmup requests excluded from aggregation |
| A reset board read as "degraded performance" | A change in `boot_id` → run invalid |
| p99 computed from 20 samples | Fewer than 100 successes → invalid |
| — | Failures excluded from throughput and per-node shares |
| — | Conditions (concurrency, seed, policy, node count) carried with the result |
| — | Percentiles are nearest-rank; interpolation forbidden |

**2. Invalidity warnings print before the numbers.**

```text
!!!!!! THIS RUN IS INVALID !!!!!!
  - error rate 100.00% exceeds the 1.00% allowance
  - 0 successful samples is below the minimum of 100
Do not quote the figures below.

requests : 200 (0 succeeded / 200 failed, ...)
```

**3. Invalid runs are not deleted.** They are kept with the reason.

**4. The policy name prefers the value the scheduler reports.**

**5. What the tool does not guarantee is written into the result file.**

## Rationale

### Why failures must not go into throughput

Include them and **throughput is highest when every node is dead.** Failures
return immediately, so requests per second explode.

```text
read this metric as-is in the S4 failure-handling experiment
  ->  the result reads "performance improves during an outage"
```

Per-node shares are the same. A failed request's `node_id` is empty, and
counting that makes **a dead node look like it "processed a lot"**.

### Why percentiles are not interpolated

Linear interpolation invents **values never actually observed** when samples are
few.

```text
interpolating p95 over observations 1-10  ->  9.55
no request experienced that latency
```

Writing "p95 = 9.55 ms" in a presentation makes it a computation, not a
measurement. It is fixed to nearest-rank and the definition is pinned in the
module documentation.

### Why the warning goes on top

**Show the numbers first and people believe them first.** Put the warning below
and the first screenful without scrolling is the numbers, and those numbers get
copied into a table.

### Why invalid runs are not deleted

They have to remain with their reason for the cause to be traceable. And
**repeated reboots are themselves a finding** — that is in fact how the power
adapter problem was found.

### Why the policy name comes from the scheduler

Typing `--policy round-robin` by hand goes wrong. **A result labelled with the
wrong policy name ruins the whole S3 policy comparison.**

### One problem caught during implementation

The first approach queried node state via the heartbeat RPC, because the
scheduler had no node listing API.

**But that overwrites the scheduler's node state.** A heartbeat is a call that
records observations, so a bench sending an empty `health` has the scheduler
accept it as a real observation and zero out temperature and queue depth. It
**contaminates the state of the thing being measured, immediately before
measuring it.**

A read-only `ListNodes` RPC was added separately. This too is a variant of type
A (using an API without checking its side effects).

## ⚠️ What the tool does not guarantee

**The load is a closed loop.** Concurrency N is fixed and the next request is
sent after the response arrives.

That approach is vulnerable to **coordinated omission**. When the system slows
down the client slows down with it, so **the latency distribution comes out
optimistic.** A slow request delays the launch time of subsequent requests, and
that delay is not charged to any request's latency.

→ **Never quote absolute latency as an SLA. Use it only for comparison between
configurations.** That sentence goes into the result file's `caveats` so it is
visible even when the results are read in isolation.

An open model (fixed target RPS) was not used because the node queue is finite.
Raising RPS quickly ends in `NPF-1303` rejections and the latency distribution
cannot be seen. If both are needed, that is added in M7.

## Alternatives and why they were rejected

| Alternative | Why rejected |
|---|---|
| Have a human look at the results and judge | Impossible across 146 runs / 23.4 hours of unattended overnight execution |
| Keep the rules in a document | Already confirmed not to work |
| Delete invalid runs automatically | Cause tracing becomes impossible. The pattern of repetition is itself information |
| Linear percentile interpolation (the common practice) | Invents unobserved values. Especially dangerous with few samples |
| Open-model load | The node queue is finite and it ends in rejections |

## Consequences

**Gained**

- Invalid data does not make it into the result tables
- Validity is judged automatically even in unattended runs
- The tool's limitations are written inside the result file

**Lost / the cost**

- The validity thresholds (100 successes, 1% error rate) are arbitrary values.
  There is room to sharpen the rationale
- The closed loop's optimistic latency is carried along

**New constraints introduced**

- **Absolute latency must not be quoted as an SLA.** For comparison between
  configurations only
- Each new mistake encountered adds a rule here

## What would overturn this

- **Adding an open model in M7** changes how the latency distribution is
  interpreted. The two models' results are not mixed
- The validity thresholds can be adjusted after S0, based on the actual
  distribution

---

<a id="template"></a>

# ADR-NNN. 결정을 한 문장으로 (동사로 끝낸다)

| | |
|---|---|
| **상태** | 확정 / 잠정 / 대체됨 |
| **날짜** | YYYY-MM-DD |
| **대체** | ADR-NNN 을 대체함 (없으면 지운다) |
| **관련** | ADR-NNN, `docs/xxx.md` §N |

---

## 한 줄 요약

> 이 줄만 읽고 덮어도 결론이 남아야 한다.

## 배경

무슨 상황이었나. **이 분야를 모르는 사람 기준**으로 쓴다. 용어가 나오면
그 자리에서 한 문장으로 푼다.

이전에 다른 결정이 있었고 그것을 뒤집는 거라면, 여기에 경위를 적는다.
무엇을 믿었고, 왜 그렇게 믿었고, 무엇이 그 믿음을 깼는지.

## 결정

무엇을 하기로 했나. 여러 개면 번호를 붙인다.

## 근거

왜 그렇게 했나. **측정값이 있으면 조건과 함께** 적는다.

```text
측정 조건: 노드, 스레드 수, 지속 시간, governor, 모델
```

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| | |

## 결과

- **얻은 것**
- **잃은 것 / 대가**
- **새로 생긴 제약** — 이 결정 때문에 앞으로 조심해야 하는 것

## 뒤집힌다면

어떤 관측이나 조건이 나오면 이 결정을 다시 봐야 하나.

재검증 방법도 적는다. 특히 **무엇을 보면 안 되는지**가 중요하다 —
이 프로젝트는 "API 오류 0건" 이나 "NPU 클럭 고정" 처럼 **틀린 지표로
통과 판정**을 낸 적이 네 번 있다.
