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

# ADR-001. 모델을 쪼개지 않고 요청을 나눈다 (데이터 병렬)

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 (최초), 2026-08-19 (ADR 로 정리) |
| **관련** | [ADR-012](#adr-012), `docs/00-PRD.md` §4, `docs/01-TECHSPEC.md` §2.1 |

---

## 한 줄 요약

> 노드 세 대가 **같은 모델 전체**를 각자 갖고, **서로 다른 요청**을 처리한다.
> 모델을 레이어 단위로 쪼개 나눠 갖는 방식은 v0.1 에서 하지 않는다.

## 배경

NPU 여러 개를 한 번에 쓰는 방법은 크게 두 가지다.

### 방법 A. 모델을 쪼갠다 (모델 병렬 / 레이어 분할)

```text
요청 1장 ──► [노드1: 레이어 1~10] ──중간 텐서──► [노드2: 레이어 11~20] ──► 결과
```

모델의 앞부분은 1번 노드가, 뒷부분은 2번 노드가 계산한다. 노드 사이로
**중간 계산 결과(feature map)** 가 오간다. LLM 의 텐서 병렬·파이프라인
병렬이 이 계열이다.

- 장점: 요청 **한 장**의 처리 시간이 줄어들 수 있다. 노드 하나에 안 들어가는
  큰 모델도 돌릴 수 있다
- 단점: 노드 사이 통신이 **추론 경로 안에** 들어간다. 한 노드가 느리면 전체가
  기다린다

### 방법 B. 요청을 나눈다 (데이터 병렬)

```text
요청 A ──► [노드1: 모델 전체] ──► 결과 A
요청 B ──► [노드2: 모델 전체] ──► 결과 B
요청 C ──► [노드3: 모델 전체] ──► 결과 C
```

노드마다 모델 전체를 갖고 각자 다른 요청을 끝까지 처리한다.

- 장점: 노드끼리 통신이 **없다**. 한 대가 죽어도 나머지가 그대로 돈다
- 단점: 요청 **한 장**은 절대 빨라지지 않는다

어느 쪽을 고르느냐에 따라 시스템의 모든 것이 달라진다. 스케줄러의 역할,
장애 처리 방식, 네트워크 요구량, 측정 항목까지 전부.

## 결정

**방법 B(데이터 병렬)만 구현한다.** 방법 A 는 v0.1 의 명시적 비목표다.

이에 따라 다음도 함께 비목표가 된다.

- 하나의 대규모 모델을 여러 노드에 레이어 단위로 분할
- LLM 텐서 병렬 / 파이프라인 병렬
- 여러 NPU 를 하나의 물리 NPU 처럼 보이게 하는 하드웨어 수준 통합
- 단일 추론 요청의 지연시간을 노드 수에 비례해 단축

## 근거

### 1. 목표가 처리량이지 지연시간이 아니다

이 프로젝트가 답하려는 질문은 **"6 TOPS 세 대는 정말 18 TOPS 가 되는가"**
다. 요청이 몰릴 때 전체를 얼마나 처리하는지를 묻는 질문이지, 한 장을 얼마나
빨리 끝내는지를 묻는 것이 아니다.

상정한 사용 형태도 마찬가지다. 다중 카메라, 다중 요청 — **애초에 독립적인
요청이 다발로 들어온다.** 이런 부하에서는 데이터 병렬이 자연스러운 형태고,
모델을 쪼개면 오히려 손해다.

### 2. 이 하드웨어에서 분할은 통신비를 감당할 수 없다

입력 한 장이 이미 크다.

```text
raw RGB 640 × 640 × 3 = 1,228,800 byte
```

**입력만으로도** 3노드 포화 시 4.64 Gbps 다(INT8 157.2 inf/s 기준).
그래서 aggregation 링크에 10G 가 필요하다는 결론이 이미 나와 있다.

레이어 분할은 여기에 **중간 텐서 왕복을 추론 경로 안에 얹는다.** 분할 지점
하나마다 노드 간 전송이 한 번씩 추가되는데, 2.5GbE 에서 1 MB 급 텐서 한 번이
계산상 4 ms 근처다. INT8 추론 **전체**가 50.8 ms 인 것을 생각하면 분할 지점
몇 개만으로 이득이 사라진다.

> 이 4 ms 는 링크 속도로 나눈 계산값이지 실측이 아니다. 다만 실측을 해서
> 확인할 가치가 있는 수준의 차이가 아니라고 판단했다 — 분할로 얻는 이득
> 자체가 애초에 목표가 아니기 때문이다.

### 3. 이 모델은 쪼갤 이유가 없다

분할이 **강제되는** 상황은 모델이 노드 하나에 안 들어갈 때다.

```text
노드 RAM              4 GB
YOLOv8n INT8 모델     6.46 MB
YOLOv8n FP16 모델     9.65 MB
```

세 자릿수 차이가 난다. 쪼갤 필요가 전혀 없다.

### 4. 측정 결과를 해석할 수 있어야 한다

이 프로젝트의 산출물은 **"어디에서 새는가"** 다. 노드가 서로 독립이면
확장 효율이 안 나올 때 원인을 스케줄링·네트워크·노드 내부로 깨끗하게
나눠 볼 수 있다.

레이어 분할을 넣으면 노드 간 의존이 생겨서, 3노드가 2.4배밖에 안 나왔을 때
그것이 분할 지점 통신 때문인지 스케줄링 때문인지 NPU 때문인지 분리하기
어려워진다. **측정을 목적으로 하는 프로젝트에서 원인 분해가 안 되는 구조를
고르면 안 된다.**

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 레이어 단위 분할 | 통신비가 추론 경로에 들어간다. 이 모델은 4GB 에 여유롭게 들어가 분할할 이유가 없다 |
| LLM 텐서 병렬 | 대상 모델이 CNN 검출기다. 적용 대상 자체가 없다 |
| 하드웨어 수준 NPU 통합 | RKNN Runtime 위에서 할 수 있는 일이 아니다. 드라이버·SoC 레벨 작업 |
| 데이터 병렬 + 분할 **둘 다** 지원 | v0.1 기간에 둘 다 제대로 측정할 수 없다. 어설프게 둘 다 하면 어느 쪽 수치도 못 쓴다 |

## 결과

**얻은 것**

- 노드 사이에 통신이 없다. 노드는 서로의 존재를 모른다
- 장애 처리가 단순해진다 — 죽은 노드를 후보에서 빼면 끝이다. 진행 중이던
  다른 노드의 작업에 영향이 없다
- 노드 하나의 성능 상한을 재면 클러스터 상한을 예측할 수 있다.
  **단일 노드 측정에 이렇게 공을 들인 이유가 이것이다**
- 스케줄러가 "요청 하나 → 노드 하나" 만 결정하면 된다

**잃은 것 / 대가**

- **단일 요청 지연시간은 노드를 늘려도 절대 줄지 않는다.** INT8 한 장
  50.8 ms 는 3노드에서도 50.8 ms 다. 이건 버그가 아니라 설계다
- 노드 하나에 안 들어가는 모델은 못 돌린다
- 노드마다 모델 사본이 필요하다 (이 규모에서는 문제가 안 된다)

**새로 생긴 제약**

- 발표와 문서에서 **"3배 빨라진다"고 말하면 안 된다.** "3배 많이 처리한다"
  가 맞다. 이 둘을 섞어 쓰면 청중이 지연시간 단축을 기대하게 된다
- 벤치마크 시나리오는 반드시 **동시 요청** 부하여야 한다. 한 장씩 순차로
  던지는 측정은 이 구조에서 의미가 없다

## 뒤집힌다면

다음 중 하나라도 성립하면 다시 본다.

- **대상 모델이 노드 메모리에 안 들어갈 때.** 4GB 를 넘는 모델을 돌려야 하면
  분할이 선택이 아니라 강제가 된다
- **단일 요청 지연이 요구사항이 될 때.** 지금은 아니지만, 예를 들어 프레임
  단위 실시간 제어가 목표가 되면 전제가 바뀐다
- **노드 간 링크가 추론 내부 통신을 감당할 만큼 빨라질 때.** 다만 이건
  2.5GbE 급 엣지 보드라는 이 프로젝트의 전제 자체를 바꾸는 이야기다

셋 다 v0.1 범위 밖이다. 재검토하더라도 **v0.1 의 데이터 병렬 측정이 끝난
뒤**여야 한다 — 비교 기준선이 없으면 분할이 이득인지 판단할 수 없다.

---

<a id="adr-002"></a>

# ADR-002. 성공 기준을 "수치가 나왔는가" 가 아니라 "측정하고 설명할 수 있는가" 로 둔다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-05 (최초), 2026-08-19 (ADR 로 정리) |
| **관련** | [ADR-001](#adr-001), [ADR-015](#adr-015), [ADR-028](#adr-028), `docs/00-PRD.md` §3 |

---

## 한 줄 요약

> "3노드에서 2.5배 이상" 같은 **결과값을 성공 조건으로 걸지 않는다.**
> 확장 효율이 낮게 나와도, io_uring 이 효과가 없어도, 그 원인을 정량적으로
> 설명할 수 있으면 성공이다.

## 배경

측정 프로젝트에서 성공 기준을 목표 수치로 걸면 무슨 일이 벌어지나.

```text
목표: "3노드 확장 효율 80% 이상"

측정 결과 65%  →  실패로 기록해야 한다
                →  실패하고 싶은 사람은 없다
                →  유리한 조건을 찾게 된다
                   짧게 재기 · 입력을 작게 · 예열 충분히 · 잘 나온 run 만 채택
```

**이건 부정직한 사람만 하는 일이 아니다.** 조건을 고르는 자유도가 있고
목표가 걸려 있으면 무의식적으로 유리한 쪽을 고르게 된다. 그리고 그 선택
하나하나에는 다 그럴듯한 이유를 붙일 수 있다.

이 프로젝트는 조건 선택의 자유도가 특히 크다. governor, 스레드 수, 지속
시간, 냉각, 입력 크기, 모델 — 전부 수치를 바꾼다. 실제로 같은 보드에서
governor 하나로 7%, 지속 시간 하나로 27% 가 움직였다.

## 결정

**성공 기준을 다음으로 정의한다.**

1. 측정했는가
2. 측정 조건을 함께 기록했는가
3. 결과의 원인을 설명할 수 있는가
4. 재현 가능한가

**다음 결과도 유효한 성과로 간주한다고 명시한다.**

- io_uring 이 유의미한 성능 개선을 만들지 못함
- Zero-Copy 적용 범위가 제한적임
- 네트워크보다 NPU 또는 전처리가 주요 병목으로 확인됨
- 3노드 확장 효율이 예상보다 낮음
- 단일 고성능 장치가 비용 면에서 더 유리함

## 근거

### 실제로 도움이 됐다

이 기준이 없었으면 버렸을 결과들이 오히려 핵심 산출물이 됐다.

| 결과 | 목표 기준이었다면 | 실제로는 |
|---|---|---|
| 애플리케이션 최적화 3종이 +0.1 / +5.4 / -1.8% | 실패. 덮고 다른 걸 시도 | **"노드 내부에서 짜낼 것이 없다"는 근거**가 됐다 |
| zero-copy 가 -1.8% | 실패 | 가설 반증. ioctl 76회가 추론 제출에 내재한다는 발견으로 이어졌다 |
| 팬리스 지속 부하에서 -27% | 나쁜 수치 | **Peak vs Sustained 격차** — 벤더 스펙시트에 없는 값. 발표의 중심 서사가 됐다 |

특히 세 번째가 결정적이다. 목표가 "높은 처리량" 이었다면 팬을 달고
120초만 재서 84.3 inf/s 를 보고했을 것이다. 그 수치는 **현장에서 재현되지
않는다.**

### 뒤집힌 결론을 발표할 수 있게 된다

이 프로젝트는 측정으로 결론이 다섯 번 뒤집혔다. 목표 수치가 걸려 있었다면
뒤집는 것 자체가 손해다 — 이미 보고한 숫자가 무효가 되니까.

기준이 "설명할 수 있는가" 이면 **뒤집는 것이 오히려 성과**다.
`docs/RESULTS.md` §4 「뒤집힌 결론」과 §6 「측정 실패 목록」이 그래서 존재할
수 있다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 목표 수치를 건다 (예: 확장 효율 80%) | 조건 선택 편향이 생긴다. 측정 프로젝트에서 가장 위험한 것 |
| 목표 수치 + "미달 시 사유 기술" | 사유 기술이 변명 절이 된다. 미달을 실패로 규정한 순간 같은 문제가 남는다 |
| 기준을 안 정한다 | 언제 끝난 건지 알 수 없다. 무한정 측정하게 된다 |

## 결과

**얻은 것**

- 불리한 결과를 그대로 낼 수 있다
- 실패 사례가 산출물이 된다 — 재사용 가치가 수치보다 높다
- 측정 조건을 숨길 이유가 사라진다

**잃은 것 / 대가**

- **"그래서 몇 배인데?" 라는 질문에 한 줄로 답하기 어렵다.** 발표에서
  불리하다. 조건을 함께 말해야 하므로 문장이 길어진다
- 성공/실패 판정이 주관적으로 보일 수 있다. 그래서 위 4개 조건을 명시했다

**새로 생긴 제약**

- **모든 수치에 측정 조건을 붙여야 한다.** 조건 없는 숫자는 이 기준 아래
  에서 무효다. 노드·스레드·시간·governor·모델을 항상 함께 적는다
- 무효한 run 을 유효한 것처럼 쓰면 안 된다 → 도구가 강제한다
  ([ADR-028](#adr-028))

## 뒤집힌다면

이 프로젝트가 **실험 도구가 아니라 제품**이 되면 기준이 달라진다.
제품에는 "이 정도는 나와야 쓸 수 있다" 는 선이 필요하다.

v0.1 은 측정이 목적이므로 이 기준을 유지한다.

---

<a id="adr-003"></a>

# ADR-003. 스케줄러를 하나만 두고, 고가용성을 구현하지 않는다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 (최초), 2026-08-19 (ADR 로 정리) |
| **관련** | [ADR-001](#adr-001), [ADR-014](#adr-014), `docs/01-TECHSPEC.md` §2.3 |

---

## 한 줄 요약

> 요청을 어느 노드로 보낼지는 **중앙 스케줄러 한 대**가 정한다.
> 분산 합의도, 리더 선출도, 스케줄러 이중화도 만들지 않는다.
> 대신 **스케줄러가 죽었다 살아나는 비용을 싸게** 만들어 두었다.

## 배경

요청을 여러 노드에 나누는 구조는 크게 넷이다.

| 방식 | 누가 정하나 |
|---|---|
| **중앙 스케줄러** | 가운데 있는 한 대가 전부 정한다 |
| 클라이언트 측 분배 | 클라이언트가 직접 노드를 고른다 (스케줄러 없음) |
| P2P / gossip | 노드끼리 상태를 주고받으며 자기들끼리 정한다 |
| 범용 오케스트레이터 | Kubernetes 같은 기성 시스템에 맡긴다 |

뒤로 갈수록 단일 장애점이 사라지고 규모가 커져도 버틴다. 대신 구현과
운영이 무거워진다.

## 결정

**중앙 스케줄러 한 대를 쓴다.** 그리고 v0.1 에서 다음을 **구현하지 않는다.**

- 분산 합의 (Raft 등)
- 리더 선출
- 다중 스케줄러 고가용성
- Kubernetes 수준의 범용 오케스트레이션

**스케줄러는 단일 장애점이다.** 이것을 결함이 아니라 **알고 받아들인 제약**
으로 문서에 적는다.

## 근거

### 1. 측정 대상이 스케줄링 정책 자체다

이 프로젝트는 Round Robin / Least Queue / ECT **세 정책을 갈아 끼우며
비교**하는 실험(S3)을 한다. 그러려면 **결정이 내려지는 지점이 한 곳**이어야
한다.

분배가 클라이언트나 노드로 흩어지면 "이번 run 의 분배 정책"이라는 개념
자체가 흐려진다. 정책 비교 실험이 정책이 아니라 구현 위치의 차이를 재게 된다.

### 2. ECT 는 전역 상태를 봐야 계산된다

기본 정책인 ECT 는 이런 식으로 후보를 고른다.

```text
ECT = ((queue_depth + in_flight + 1) × EWMA_inference
       + EWMA_network + thermal_penalty + error_penalty) / load_factor
```

여기 들어가는 값 — 각 노드의 큐 깊이, 진행 중 건수, 추론 시간 이동평균,
온도 — 은 **모든 노드를 한눈에 보고 있어야** 비교가 된다. 노드가 자기
상태만 알고 결정하면 이 식이 성립하지 않는다.

### 3. 노드가 세 대다

합의 프로토콜이나 gossip 이 값을 하는 규모는 노드가 수십~수백 대일 때다.
세 대에서는 얻는 것보다 구현·디버깅 비용이 크다.

### 4. 시간 예산

발표까지 정해진 기간 안에 **측정을 끝내는 것**이 목표다. 합의 구현에
시간을 쓰면 정작 재야 할 것을 못 잰다. 만들지 않기로 한 것이 만들기로 한
것만큼 중요하다.

## 단일 장애점을 어떻게 다루나

없애는 대신 **복구를 싸게** 만들었다.

- 노드는 하트비트가 실패하면 **곧바로 재등록으로 전환**한다
- 등록은 **멱등**이다. 여러 번 해도 문제가 없다
- 그래서 스케줄러를 죽였다 다시 띄우면 **세 노드가 약 1.3초 안에 스스로
  돌아온다** (실제 프로세스 4개로 확인)

일시적 네트워크 오류와 스케줄러 재시작은 노드 입장에서 구분할 수 없다.
그래서 **더 비싼 쪽(재등록)을 무조건 택한다.** 등록이 멱등이라 헛수고가
손해로 이어지지 않기 때문에 가능한 선택이다. (→ ADR-025)

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 클라이언트 측 분배 | 정책 비교 실험이 성립하지 않는다. 클라이언트가 전역 상태를 볼 방법도 없다 |
| P2P / gossip | 노드 3대에서 이득이 없다. 노드 간 통신이 생겨 [ADR-001](#adr-001) 의 "노드는 서로를 모른다" 전제가 깨진다 |
| Kubernetes | 명시적 비목표. 컨테이너 오케스트레이션은 이 프로젝트가 답하려는 질문과 무관하고, 측정에 잡음만 더한다 |
| 스케줄러 2대 + 리더 선출 | 구현·검증 비용이 크다. 그 시간에 측정을 못 한다. 노드 3대 규모에서 얻는 가용성이 그 값을 못 한다 |

## 결과

**얻은 것**

- 정책 3종을 같은 자리에서 갈아 끼울 수 있다 → S3 실험이 가능해졌다
- 재시도·상태머신·헬스체크가 전부 한 프로세스 안에 있어 추적이 쉽다
- 스케줄러 재시작 복구가 1.3초

**잃은 것 / 대가**

- **스케줄러가 죽으면 클러스터 전체가 멈춘다.** 노드는 살아 있어도 요청을
  받을 경로가 없다
- 처리량 상한에 스케줄러 자신이 포함된다. 노드를 아무리 늘려도 스케줄러가
  못 버티면 거기서 막힌다

**새로 생긴 제약**

- **스케줄러 호스트가 측정 조건의 일부가 되었다.** 어디서 돌리느냐가 수치를
  바꾼다. 그래서 공식 벤치마크에서는 보드가 아닌 별도 호스트에서 돌린다
  (→ [ADR-014](#adr-014))
- 스케줄러 호스트의 자원이 실험 제약이 된다. 현재 `dealer` 는 RAM 3GB 라
  페이로드 1.17 MiB × 동시 처리 수가 쌓이면 부족할 수 있다. **아직 관찰하지
  않았다**

## 뒤집힌다면

- **노드가 수십 대 규모가 될 때.** 이 결정은 3대를 전제로 한다
- **스케줄러가 실제로 병목으로 측정될 때.** 판정 근거는 이미 준비되어 있다 —
  `TimingBreakdown` 의 `scheduler_queue_us` / `scheduler_route_us` 가
  `end_to_end_us` 에서 유의미한 비중을 차지하는지 보면 된다. **추측하지 말고
  이 칸을 본다**
- **가용성이 요구사항이 될 때.** 지금은 실험 장비고, 스케줄러가 죽으면 사람이
  다시 띄우면 된다. 운영 시스템이 되면 전제가 다르다

---

<a id="adr-004"></a>

# ADR-004. 백엔드를 인터페이스로 분리하고, Mock 을 1급 백엔드로 둔다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 (최초), 2026-08-19 (ADR 로 정리) |
| **관련** | [ADR-005](#adr-005) (feature gate), [ADR-007](#adr-007), `docs/03-DEVELOPMENT-REQUIREMENTS.md` §4.1 |

---

## 한 줄 요약

> NPU 호출을 `InferenceBackend` 인터페이스 뒤로 밀어 넣고, 그 자리에 끼울 수
> 있는 **가짜 백엔드를 정식 구현으로** 만든다. RK3576 보드가 한 대도 없어도
> 전체 시스템이 돌아간다. **편의 기능이 아니라 설계 원칙이다.**

## 배경

이 프로젝트의 개발 환경은 이렇다.

- 보드 3대는 책상 위에 있고, 항상 켜져 있지 않다
- 개발 PC 는 **Windows/x86** 이다. RKNN Runtime 은 ARM64 Linux 전용이다
- CI 는 GitHub Actions 위에서 돈다. NPU 가 있을 리 없다

여기서 아무 대책 없이 개발하면 이렇게 된다. **보드가 켜져 있어야만 코드를
짤 수 있고, 보드가 켜져 있어야만 테스트가 돌고, CI 는 아무것도 검증하지
못한다.**

그런데 잘 보면 이 시스템에서 **NPU 가 실제로 필요한 부분은 아주 좁다.**

```text
스케줄링 정책 3종        NPU 무관
노드 레지스트리·상태머신  NPU 무관
재시도·타임아웃          NPU 무관
큐·워커 풀              NPU 무관
gRPC 배선               NPU 무관
헬스체크·드레인          NPU 무관
─────────────────────────────────
실제 추론 한 번          ← 여기만 NPU
```

## 결정

**1. 추론을 인터페이스 뒤로 감춘다.**

```rust
#[async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn load_model(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>>;
    fn backend_name(&self) -> &'static str;      // "rknn" 또는 "mock"
    fn runtime_version(&self) -> Result<String>;
}

#[async_trait]
pub trait LoadedModel: Send + Sync {
    async fn infer(&self, input: InferenceInput) -> Result<InferenceOutput>;
    fn model_info(&self) -> &LoadedModelInfo;
}
```

스케줄러와 노드 에이전트는 이 인터페이스만 안다. `npuforge-rknn` 을
직접 부르지 않는다.

**2. Mock 백엔드를 테스트 도우미가 아니라 정식 백엔드로 만든다.**

설정 파일에서 고른다. 테스트 코드 안에 숨어 있는 스텁이 아니다.

```toml
[backend]
type = "mock"          # 또는 "rknn"
base_latency_ms = 20
jitter_ms = 5
error_rate = 0.02
```

**3. Mock 에 결함 주입을 넣는다.** 결정적 시드 위에서 지연, 지연 편차,
오류율, 노드별 속도 편차를 만들 수 있다. `configs/mock/` 의 세 노드는
**일부러 서로 다른 속도와 오류율**을 갖는다.

**4. 검증 기준을 "하드웨어 없이 통과" 로 잡는다.** `cargo test --workspace`
가 Windows/x86 에서 통과해야 한다.

## 근거

### 1. 정책 비교가 Mock 에서 먼저 보여야 한다

Round Robin 과 ECT 의 차이를 실장비에서만 볼 수 있다면, 정책을 고칠 때마다
보드를 켜고 배포하고 측정해야 한다. 반복 주기가 몇 분 단위가 된다.

`configs/mock/` 의 세 노드가 서로 다른 속도를 갖는 이유가 이것이다.
**속도가 같으면 Least Queue 와 Round Robin 이 같은 답을 낸다.** 정책 차이가
로컬에서 드러나도록 조건을 일부러 비대칭으로 만들었다.

### 2. 실장비에서 만들기 어려운 조건을 만들 수 있다

"노드가 2%의 확률로 실패한다", "한 노드만 3배 느리다", "요청 도중에 노드가
죽는다" — 실제 보드로 재현하려면 번거롭고, 재현성도 떨어진다. Mock 은
시드를 고정해 **매번 같은 순서로** 만들어낸다.

### 3. 전송 경로는 진짜다

Mock 3노드 통합 테스트(`crates/npuforge-scheduler/tests/mock_cluster.rs`)는
**실제 gRPC 를 탄다.** 프로세스만 하나일 뿐 배선은 실장비와 같다.

| 검증 항목 | 결과 |
|---|---|
| 요청이 3노드에 분산 | ✅ round-robin 이 세 노드를 모두 사용 |
| 노드 1대 사망 시 우회 | ✅ 6/6 성공 |
| 전 노드 사망 | ✅ `NPF-1302` + 시도한 노드 목록 |
| 타이밍 분해 | ✅ 노드·스케줄러 구간 모두 채워짐 |
| 느린 노드 회피 | ✅ least-queue 가 빠른 노드를 더 많이 사용 |

### 4. CI 가 실제로 뭔가를 검증한다

209개 테스트가 하드웨어 없이 돈다. 이게 없으면 CI 는 컴파일만 확인하는
장식이 된다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| `#[cfg(test)]` 스텁만 둔다 | 테스트 안에서만 살아 있다. 3노드 클러스터를 띄워 손으로 만져 보는 것이 불가능하다 |
| 실장비 필수로 간다 | 보드가 꺼지면 개발이 멈춘다. CI 가 무의미해진다. 기여자가 보드를 사야 참여할 수 있다 |
| RKNN 시뮬레이터 사용 | 빌드된 `.rknn` 을 추론하지 못한다 — `load_rknn` 후 `init_runtime` 이 거부한다. 실제로 시도했고 안 됐다 |
| 인터페이스 없이 조건부 컴파일로 분기 | 호출부마다 `#[cfg]` 가 번지고, 두 경로가 조용히 갈라진다 |

## 결과

**얻은 것**

- 209 tests 가 Windows/x86 에서 통과한다
- 3노드 클러스터를 로컬에서 띄워 실제로 조작해 볼 수 있다
- `unsafe` 가 `npuforge-rknn` 한 곳에 갇힌다 (→ ADR-006)
- 기여자가 보드 없이 참여할 수 있다 — 오픈소스로서 중요하다

**잃은 것 / 대가**

- 인터페이스를 유지하는 비용. 백엔드마다 같은 계약을 지켜야 한다
- 두 구현이 갈라질 위험. `runtime_version` 같은 메타데이터가 Mock 에서는
  의미가 없어 형식만 채우는 자리가 생긴다

**⚠️ 새로 생긴 제약 — Mock 은 만능이 아니다**

이게 이 ADR 에서 가장 중요한 문장이다.

**Mock 은 인터페이스를 통과하는 것만 흉내 낸다.** RKNN 고유의 결함은 절대
잡지 못한다. 실제로 [ADR-007](#adr-007) 의 컨텍스트
공유 문제 — 오류 0건에 결과 100% 불일치 — 는 Mock 에서 재현될 수가 없다.
Mock 에는 컨텍스트라는 개념 자체가 없기 때문이다.

그래서 **실장비 통합 테스트가 따로 있어야 한다.**
`crates/npuforge-rknn/tests/real_device.rs` 6종이 그 자리다.

```text
Mock 이 지키는 것            실장비만 지킬 수 있는 것
────────────────────        ────────────────────────
정책·재시도·상태머신         RKNN 동시성 계약
큐·타임아웃                  역양자화 정확도
gRPC 배선                    실제 처리량·열 거동
장애 우회 경로               출력 텐서 형태
```

**"Mock 테스트가 통과했으니 됐다" 는 판단을 하면 안 된다.**

## 뒤집힌다면

- **Mock 과 실장비 동작이 갈라지는 사례가 쌓이면.** 그때는 Mock 의 충실도를
  올릴지, 아니면 Mock 을 정책 검증 전용으로 좁힐지 정해야 한다
- **백엔드가 셋 이상이 되면** 인터페이스를 다시 볼 필요가 있다. 현재 두 개는
  최소 표본이라 추상화가 맞는지 확신하기 어렵다

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

# ADR-009. 정책은 세 개로 고정하고, 후보 필터는 셋이 공유한다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 |
| **관련** | [ADR-003](#adr-003), [ADR-010](#adr-010), `docs/01-TECHSPEC.md` §10.0, §10.4 |

---

## 한 줄 요약

> `round-robin` / `least-queue` / `ect` 세 개만 둔다. 그리고 **세 정책이
> 완전히 같은 후보 필터를 거친다.** 필터가 다르면 정책 비교 실험이 정책이
> 아니라 필터의 차이를 재게 된다.

## 배경

스케줄링 정책 비교(시나리오 S3)가 이 프로젝트의 측정 항목 중 하나다.
"부하를 보고 고르면 그냥 순서대로 도는 것보다 얼마나 나은가" 를 재려는 것이다.

정책은 두 부분으로 이루어진다.

```text
① 후보 필터    누가 후보 자격이 있나  (죽은 노드 제외, 모델 있는 노드만 ...)
② 선택 규칙    후보 중 누구를 고르나  (순서대로 / 큐가 짧은 쪽 / 예상 완료시간)
```

여기에 함정이 있다. **정책마다 ①을 다르게 만들면**, A 정책이 B 정책보다
좋게 나왔을 때 그것이 선택 규칙 때문인지 필터 때문인지 알 수 없다.

예를 들어 ECT 만 "온도 85°C 넘는 노드 제외" 를 넣어 두면, ECT 가 이기는
이유가 똑똑해서인지 뜨거운 노드를 피해서인지 분리되지 않는다.

## 결정

**1. 정책 식별자를 세 개로 고정한다.**

| 식별자 | 정책 | 용도 |
|---|---|---|
| `round-robin` | Round Robin | 비교 기준 |
| `least-queue` | Least Queue | 중간 비교군 |
| `ect` | Estimated Completion Time | 권장 기본값 |

**2. 세 정책이 동일한 후보 필터를 거친다.**

```text
- is_schedulable() 상태일 것
- 요청 모델을 Ready 상태로 보유할 것
- 온도가 disable_temperature_c 미만일 것
```

**3. 식별자 문자열을 한 곳에서만 파싱한다.**

```rust
#[serde(rename_all = "kebab-case")]
pub enum SchedulingPolicyKind { RoundRobin, LeastQueue, Ect }
```

설정 파일, CLI 인자, 메트릭 레이블, 로그, 대시보드가 **전부 같은 문자열**을
쓴다. `queue-aware`, `estimated-completion-time`, `queue_aware` 같은 변형을
쓰지 않는다.

**4. 인터페이스를 선택 규칙만으로 좁힌다.**

```rust
pub trait SchedulingPolicy: Send + Sync {
    fn select_node(&self, task: &InferenceTask, candidates: &[NodeSnapshot])
        -> Result<NodeId, ScheduleError>;
}
```

`candidates` 는 **이미 필터를 통과한 목록**이다. 정책이 직접 노드 전체
목록을 보지 않으므로, 정책 안에서 자기만의 필터를 추가할 여지가 구조적으로
줄어든다.

## 근거

### 정책 비교가 이 프로젝트의 측정 항목이다

S3 는 "정책의 차이" 를 재는 실험이다. 변수는 하나여야 한다. 필터가 공유되지
않으면 실험 설계 자체가 무효다.

### 식별자가 흔들리면 결과가 오염된다

벤치 도구 설계 중 실제로 나온 문제다. `--policy round-robin` 을 손으로 적게
하면 오타가 나거나 실제 스케줄러 설정과 다른 값이 결과에 붙는다.
**틀린 정책 이름이 붙은 결과는 S3 를 통째로 망친다.**

그래서 벤치 도구는 손으로 적은 값보다 **스케줄러가 보고한 값을 우선**한다.
이 결정과 짝을 이룬다.

### 세 개면 충분하다

- `round-robin` 은 기준선이다. 없으면 나머지가 좋은지 알 수 없다
- `least-queue` 는 "큐만 봐도 되는가" 에 답한다
- `ect` 는 큐·속도·온도·오류를 다 본다

넷째를 넣으면 실험 조합이 늘어나고, S3 의 run 수가 늘어난다. 총 146 run /
약 23.4시간 예산 안에서 값을 못 한다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 정책마다 필터를 다르게 | S3 가 필터 차이를 측정하게 된다. **가장 피해야 할 것** |
| 정책을 플러그인으로 열어 둔다 | 비교 대상이 무한해진다. 측정 프로젝트에서는 고정이 낫다 |
| 정책 하나(ECT)만 구현 | 기준선이 없어 "얼마나 나은지" 를 말할 수 없다 |
| 식별자를 자유 문자열로 | 오타와 표기 흔들림이 결과 레이블을 오염시킨다 |

## 결과

**얻은 것**

- S3 정책 비교가 성립한다 — 변수가 선택 규칙 하나다
- 설정·로그·메트릭·대시보드의 정책 이름이 항상 같다
- 정책 구현이 짧아진다. 필터를 각자 안 짜도 된다

**잃은 것 / 대가**

- 정책별 특수 조건을 넣을 수 없다. 넣으려면 **공유 필터에 넣어 세 정책
  모두에 적용**해야 한다
- 새 정책을 추가하려면 enum 을 고쳐야 한다 (의도한 마찰)

**새로 생긴 제약**

- 필터를 바꾸면 **세 정책의 과거 측정값과 비교 불가**가 된다. 필터 변경은
  실험 조건 변경으로 취급하고 기록해야 한다

## 뒤집힌다면

- **정책별로 반드시 달라야 하는 후보 조건이 발견되면.** 그때는 그 조건을
  선택 규칙 안의 점수로 표현할 수 있는지 먼저 본다 — ECT 의 `load_factor`
  가 그 방식이다 ([ADR-010](#adr-010))
- **M7 최적화 실험에서 새 정책이 필요해지면** 넷째를 추가한다. 단 S3 의
  기준선 비교는 세 정책으로 이미 끝난 뒤여야 한다

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

# ADR-011. 기준 모델을 INT8 로 한다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-11 |
| **관련** | [ADR-012](#adr-012), [ADR-014](#adr-014), [ADR-018](#adr-018) (모델 배포), `docs/discuss.md` §8 |

---

## 한 줄 요약

> INT8 양자화가 **1.86배**다. 지금까지 시도한 어떤 소프트웨어 최적화보다
> 두 자릿수 크게 먹혔다. 대가는 최고 검출 점수 -5.5% 이고 **검출 집합과
> 클래스는 동일**하다.

## 배경

### 양자화가 무엇인가

신경망은 원래 실수(FP32)로 계산한다. 이 실수들을 **정수 8비트로 줄여서**
계산하는 것이 INT8 양자화다. 곱셈 하나가 싸지고 메모리도 덜 오간다. 대신
값이 뭉개져 정확도를 조금 잃는다.

FP16 은 그 중간이다. 실수인데 비트 수만 절반이다.

### 왜 이 선택이 중요했나

FP16 으로 시작해 노드 하나의 처리량을 끌어올리려고 세 가지를 시도했고,
**전부 실패했다.**

| 시도 | 결과 |
|---|---:|
| `core_mask` 로 NPU 코어 수동 배정 | +0.1% |
| `want_float=0` (당시 1스레드 위주 측정) | +5.4% |
| zero-copy 버퍼 재사용 | **-1.8%** |

이유도 찾았다. 추론 한 건마다 커널 `ioctl` 이 약 76회 발생하고 그것이
**직렬화**된다. 애플리케이션이 줄일 수 있는 것이 아니었다. 그래서 당시
결론은 **"노드 상한 78 inf/s 는 드라이버 특성이다"** 였다.

INT8 은 그때까지 남아 있던 마지막 큰 변수였다.

## 결정

**1. 기준 모델을 YOLOv8n INT8 로 한다.**

**2. FP16 을 지우지 않고 비교 조건으로 유지한다.** 두 모델을 나란히 제시하는
것이 "양자화가 얼마나 먹히는가" 라는 결과 자체이기 때문이다.

**3. 정확도 수락 기준을 원시 텐서 유사도가 아니라 검출 수준으로 정의한다.**
(근거는 아래 함정 절)

## 근거

### 1.86 배

```text
측정 조건: king, sustained_load_test, 8스레드 고정, 120초,
          governor=performance, 팬리스
```

| 모델 | 처리량 | 평균 지연 | 모델 크기 |
|---|---:|---:|---:|
| YOLOv8n FP16 | 84.3 inf/s | 94.5 ms | 9.65 MB |
| **YOLOv8n INT8** | **157.2 inf/s** | **50.8 ms** | 6.46 MB |
| 배율 | **1.86×** | -46% | -33% |

> `ondemand` governor 로 잰 초기값은 FP16 79.0 / INT8 146.2 였다.
> **배율 1.85~1.86 은 governor 와 무관하게 유지된다.**

### 이 측정이 이전 결론을 정정했다

INT8 이 1.85배라면 "ioctl 76회가 상한을 정한다" 는 설명과 충돌한다.
그래서 INT8 의 ioctl 도 세어 봤다.

```text
strace -c -f -e trace=ioctl, 1스레드 20초

        추론    처리량       추론당 ioctl
FP16    315    15.7 inf/s      76.4
INT8    718    35.8 inf/s      76.2
```

**호출 횟수는 똑같은데 처리량이 2.28배다.**

상한을 정하는 것은 ioctl **횟수**가 아니라 **직렬화 구간에서 한 건이 붙잡고
있는 시간**이었다. 그래서 기존 결론의 범위를 좁혔다.

| 기존 | 정정 |
|---|---|
| "노드 상한 78 inf/s 는 드라이버 특성이다" | "**FP16 기준** 노드 상한이 약 78 inf/s 이고, 이 값은 애플리케이션 최적화로 못 넘는다" |
| "애플리케이션 최적화로 넘을 수 없다" | 유지. 단 **양자화는 애플리케이션 최적화가 아니라 모델 변경**이다 |

### 정확도 대가는 받아들일 만하다

```text
측정 조건: 실보드 king, COCO val2017 이미지,
          전처리를 한 곳에서 수행해 양쪽이 같은 입력 바이트를 보게 함
```

| 비교 | box cosine | 검출 셀 | 클래스 일치 |
|---|---|---|---|
| FP16 vs ONNX | 0.99999 | 10/10 | 100% |
| **INT8 vs FP16** | **0.997** | **10/10** | **100%** |

최고 검출의 셀이 한 칸 이동하고 점수가 -5.5% 다. **검출 집합과 클래스는
동일하다.** 1.86배를 이 대가로 사는 것이면 남는 장사다.

## ⚠️ 정확도 검증에서 걸린 함정

**원시 텐서 코사인 유사도를 수락 기준으로 쓰면 이 모델에서는 오판한다.**

FP16 vs ONNX — 양자화가 아예 없는 비교 — 에서도 **일부 텐서의 코사인이
0.16 까지 떨어진다.** 이 숫자만 보면 "FP16 변환이 모델을 망가뜨렸다" 는
결론이 나온다. 틀린 결론이다.

원인은 이렇다.

- YOLOv8n 출력 9개 중 텐서 2/5/8 은 **클래스 점수 80개의 합**이다
- RKNN 의 sigmoid 는 정확히 0 을 내지 않고 **하한 0.001831** 이 있다
- 80배 증폭되면 **0.1465 오프셋**이 생긴다 (실측 하한과 정확히 일치)
- 출력 셀 대부분이 배경이라 이 오프셋이 코사인을 지배한다

**모든 셀에 같은 값이 더해지므로 순위는 바뀌지 않는다. 검출 결과는 그대로다.**

→ 수락 기준을 **검출 수준**(검출 집합, 클래스, box cosine)으로 바꿨다.
`tools/model-converter/compare_detections.py` 가 그 기준으로 비교한다.

이것도 이 프로젝트의 단골 실패 유형이다. **지표 이름을 보고 의미를 짐작했다.**
"코사인 유사도가 낮다 = 결과가 다르다" 는 일반적으로는 맞지만, 이 출력
구조에서는 아니었다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| FP16 유지 | 1.86배를 버린다. 그리고 소프트웨어로는 그만큼을 만들 방법이 없다는 것이 이미 확인됐다 |
| FP32 | 이 NPU 에서 의미 없다. 크고 느리다 |
| INT8 + 정확도 손실 보정(QAT 등) | 재학습이 필요하다. 이 프로젝트는 추론 런타임을 만드는 것이지 모델을 학습하는 것이 아니다 |
| 더 큰 모델(YOLOv8s 등)을 INT8 로 | 비교 기준선이 바뀐다. 모델 선택은 별도 결정이고, 지금은 변수를 하나만 움직인다 |

## 결과

**얻은 것**

- 노드당 157.2 inf/s. FP16 대비 1.86배
- 평균 지연 94.5 → 50.8 ms
- 모델 크기 -33%

**잃은 것 / 대가**

- 최고 검출 점수 -5.5%, 최고 검출 셀 한 칸 이동
- **calibration 데이터가 필요해졌다.** COCO val2017 200장을 결정적으로
  선택해 쓴다(`fetch_calibration.py`, seed 고정). 이미지는 라이선스 때문에
  저장소에 넣지 않고 manifest 만 남긴다
- **INT8 변환은 바이트 재현성이 없다.** 같은 입력으로 3회 변환하니 해시가
  매번 달랐다(크기는 같고 1.8% 바이트 상이). 다만 **추론 결과는 완전히
  동일**하다(9개 텐서 전부 cosine 1.000000). 차이는 직렬화·레이아웃에 있고
  계산에는 없다 → 모델은 한 번만 변환해 세 노드에 배포한다 (ADR-018)

**새로 생긴 제약**

- **네트워크 부하가 오히려 늘었다.** 처리량이 1.86배가 되면 초당 오가는
  바이트도 그만큼 늘어난다. 노드당 1.545 Gbps, 3노드 4.636 Gbps 다.
  이 결정이 [ADR-014](#adr-014) 의
  10G aggregation 을 필요하게 만든 직접 원인이다
- 성능이 좋아지면 다른 곳이 막힌다는 사례로 남겨 둔다

## 뒤집힌다면

- **검출 집합이 달라지는 입력이나 모델이 나오면.** 현재 근거는 이미지 1장
  기준이다. 표본이 적다는 것을 인정하고 쓴다
- **재검증은 텐서 코사인이 아니라 검출 수준으로 한다.** 위 함정 절이 그
  이유다. 이 기준을 잊고 코사인으로 판정하면 멀쩡한 모델을 버리게 된다

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

# ADR-013. 팬리스를 기본으로 두고, throttling 을 제거 대상이 아니라 측정 대상으로 삼는다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-10 |
| **관련** | [ADR-002](#adr-002), [ADR-023](#adr-023), `docs/02-HARDWARE-SETUP.md` §9 |

---

## 한 줄 요약

> 팬을 달면 수치가 좋아진다. 그런데 **엣지 디바이스는 현장에서 팬 없이
> 놓인다.** 그래서 팬리스를 기본 조건으로 두고, 열 때문에 성능이 떨어지는
> 것을 **없앨 문제가 아니라 잴 대상**으로 다룬다. 냉각 조건은 비교군으로
> 따로 측정한다.

## 배경

RK3576 보드는 팬리스로 출고된다. 지속 부하를 걸면 뜨거워지고 성능이 떨어진다.

여기서 두 갈래가 있다.

```text
갈래 1. 팬을 단다
  → 수치가 좋아진다
  → 발표에 쓰기 좋다
  → 그런데 그 수치는 현장에서 안 나온다

갈래 2. 팬리스로 잰다
  → 수치가 나빠진다
  → 그 나빠지는 양 자체가 아무도 공개하지 않은 값이다
```

벤더가 공개하는 TOPS 는 **순간 성능**이다. 지속 부하에서 얼마나 유지되는지
— **Peak FPS 대비 Sustained FPS 격차** — 는 공개 자료가 거의 없다.

## 결정

**1. 팬리스(조건 A)를 기본 측정 조건으로 한다.**

**2. 능동 냉각(조건 B)을 비교군으로 함께 측정한다.** 동일 모델 팬 3개를
같은 회전수로 고정한다. 회전수가 다르면 노드별 냉각 조건이 달라져 3노드
대칭성이 깨진다.

**3. 열 특성 측정(S0)을 다른 모든 시나리오보다 먼저 한다.** S0 가 나머지
실험의 임계치와 cooldown 시간을 결정하기 때문이다.

**4. 임시 냉각을 측정에 섞지 않는다.** 진단 중 책상 선풍기를 쓴 적이
있는데, **진단에는 유효했으나 측정 조건으로는 쓸 수 없다.** 팬리스 측정
전에 선풍기가 꺼져 있는지 확인하는 항목이 체크리스트에 있다.

## 근거

### 두 조건을 다 재야 답할 수 있는 질문이 있다

```text
팬리스만 측정  →  "냉각하면 얼마나 나아지는가" 를 모른다
냉각만 측정    →  "실제 엣지 배치에서 얼마나 나오는가" 를 모른다
```

**두 조건을 모두 재면 "냉각이 확장 효율에 미치는 영향" 자체가 결과가 된다.**
이건 벤더 스펙시트에 없는 값이고, 측정으로 밝힌다는 이 프로젝트의 정체성과
맞는다.

### 실측 — 팬리스로 완주는 하지만 처리량은 유지되지 않는다

```text
측정 조건: 3보드 동시, 8스레드, 900초, 팬리스, 선풍기 없음
```

| 보드 | NPU 평균 | NPU 최고 | 처리량 |
|---|---:|---:|---:|
| king | 73.0°C | 75.8°C | 80.5 inf/s |
| queen | 67.5°C | 70.2°C | 77.7 inf/s |
| jack | 72.6°C | 74.8°C | 77.8 inf/s |

- 노드 간 편차 **5.6°C**
- 오류 0건으로 완주
- 90°C 초과 없음

**팬리스로 8스레드 지속 부하가 가능하다.** 그런데 처리량은 유지되지 않는다.

```text
 +10s  81.6 inf/s   ← 시작
+120s  63.6
+300s  59.7         ← 정상 상태.  시작 대비 -27%
```

### ⚠️ 무너지는 쪽은 NPU 가 아니라 CPU 였다

처음에는 "NPU throttling 없음" 으로 판정했다. 928 샘플 전부 950 MHz 였기
때문이다. **NPU 클럭만 봤다.**

같은 로그의 CPU 클럭을 보니 이랬다.

```text
        NPU온도   npu_clk   cpu4(A72)   cpu0(A53)
 +15s   86.8°C    950 MHz   2208 MHz    2016 MHz
 +30s   90.4°C    950 MHz   1416 MHz    1200 MHz
 +60s   87.8°C    950 MHz    816 MHz     600 MHz
+120s   87.8°C    950 MHz    816 MHz     600 MHz
```

**NPU 는 한 번도 안 떨어지고 CPU 가 63~70% 떨어진다.**

추론 한 건은 `입력 설정(CPU) → NPU → 출력 취득(CPU)` 이라 CPU 구간이
처리량에 직접 반영된다. 이걸 알고 있으면서도 throttling 판정은 NPU 만으로
했다. 이 프로젝트에서 같은 유형의 **네 번째** 실수다.

> 이 발견이 오히려 결과를 좋게 만들었다. **"팬리스 엣지에서 먼저 무너지는
> 것은 NPU 가 아니라 그 앞뒤를 처리하는 CPU 였다"** — 발표 서사로 훨씬 낫다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 팬 달고 측정 | 현장에서 재현되지 않는 수치. 프로젝트의 문제의식과 반대 방향 |
| 팬리스만 측정 | "냉각하면 얼마나 나아지나" 에 답할 수 없다 |
| throttling 을 피하도록 부하를 낮춰 측정 | 지속 부하에서 무슨 일이 일어나는지가 측정 대상인데, 그걸 피하는 것 |
| 임시 냉각(선풍기)으로 조건 통일 | 재현 불가능하고 노드별로 균일하지 않다 |

## 결과

**얻은 것**

- Peak vs Sustained 격차가 프로젝트의 핵심 산출물이 됐다
- 병목이 NPU 가 아니라 CPU 라는 발견
- 냉각 효과를 정량화할 준비 (S0-A / S0-B)

**잃은 것 / 대가**

- 처리량 수치가 낮게 나온다. "84.3 inf/s" 대신 "시작 81.6, 300초에 59.7"
  이라고 말해야 한다
- 측정 시간이 늘어난다. cooldown 을 기다려야 하고, 팬리스라 느리다.
  그래서 cooldown 에 **상한**을 두고 상한에 걸리면 실제 시작 온도를
  결과에 기록한다

**새로 생긴 제약**

- **열 판정에 CPU 클럭을 반드시 포함한다.** NPU 클럭만 보는 판정은 틀렸다는
  것이 확인됐다. `run-thermal-comparison.sh` 를 그렇게 고쳐야 한다
- 부하 프로파일이 다른 두 측정의 온도를 비교하지 않는다. 스윕 부하와 고정
  부하를 비교해 19°C 격차로 오해한 적이 있다
- 온도 임계치(80 / 90°C)는 **초안**이다. 정식 S0 후 재설정한다

## 뒤집힌다면

- **케이스나 방열판이 기본 구성이 되면** 조건 A 의 정의가 바뀐다
- **S0 에서 팬리스가 90°C 를 넘겨 노드가 스케줄링에서 빠지기 시작하면**
  측정 자체가 불가능해진다. 그때는 조건 B 를 기본으로 올리고 조건 A 를
  "한계 조건" 으로 재정의한다

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

# ADR-021. 노드는 후처리(NMS)를 하지 않고 원시 텐서를 반환한다

| | |
|---|---|
| **상태** | 잠정 |
| **날짜** | 2026-08-12 |
| **관련** | [ADR-012](#adr-012), [ADR-014](#adr-014), [ADR-013](#adr-013) |

---

## 한 줄 요약

> 노드는 검출 결과가 아니라 **모델 출력 텐서 9개를 그대로** 돌려준다.
> 응답이 커지는 대신 **노드의 CPU 부하가 측정 대상 밖으로 나간다.**
> 최종적으로는 노드 후처리가 옳지만, 지금은 아니다.

## 배경

YOLOv8n 같은 검출 모델의 출력은 바로 쓸 수 있는 형태가 아니다.

```text
NPU 출력   텐서 9개 (격자마다 박스 후보와 클래스 점수)
                ↓  후처리 (NMS: 겹치는 박스 정리, 임계치 적용)
최종 결과   "사람 1명, 자동차 2대" — 수 KB
```

이 후처리를 **어디서 할 것인가.**

| | 응답 크기 | 노드 CPU |
|---|---|---|
| 노드에서 후처리 | 수 KB | 늘어난다 |
| 스케줄러/클라이언트에서 후처리 | 1.2 MB | 그대로 |

## 결정

**노드는 후처리를 하지 않는다.** 원시 텐서를 blob 하나로 묶어 반환한다
([ADR-012](#adr-012)).

**상태를 「잠정」으로 둔다.** 이건 최선이라서가 아니라 **지금 조건에서 맞는
선택**이라서다.

## 근거

### 1. 노드 CPU 가 이미 병목이다

지속 부하 300초에서 처리량이 -27% 떨어지는데, 원인이 NPU 가 아니라
**CPU thermal throttling** 이다. A72 가 2208 → 816 MHz 로 강등된다
([ADR-013](#adr-013)).

여기에 NMS 를 얹으면 CPU 부하가 더 늘어난다. 그러면 이 프로젝트가 재려는
값 자체가 흔들린다.

```text
지금:      NPU 확장 효율을 재는데 CPU 가 방해한다  ← 이미 문제
후처리 넣으면: CPU 를 더 쓰게 만들고 같은 값을 잰다  ← 더 나쁘다
```

### 2. 측정 조건이 하나 더 늘어난다

NMS 는 **입력에 따라 비용이 달라진다.** 검출 대상이 많은 이미지는 오래
걸리고 적으면 빨리 끝난다. 노드에서 하면 노드별 처리 시간 편차가 입력
내용에 따라 생긴다.

3노드 확장 효율을 재는 실험에서 이 변수는 잡음이다.

### 3. 미구현이다

가장 단순한 이유. NMS 구현체가 없고, 만들면 검증(정확도 비교)도 따라온다.
장비 대기 중인 지금 우선순위가 아니다.

### 4. 네트워크 문제는 다른 방법으로 풀렸다

원시 텐서 반환의 대가는 응답 크기다. `want_float=1` 이면 응답이 요청의
3.96배라 10G 로도 부족했다.

이건 **후처리가 아니라 `want_float=0` 으로 해결**했다. 응답이 1/4 이 되어
3노드 RX 가 18.38 → 4.60 Gbps 다. 10G 안에 들어간다.

즉 **지금 당장 후처리를 해야 할 압력이 없다.**

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 노드에서 NMS 수행 | **최종적으로는 이쪽이 옳다.** 다만 CPU 병목을 악화시키고 측정 변수를 늘린다. 미구현 |
| 스케줄러에서 NMS | 스케줄러가 단일 지점이라 3노드 몫의 후처리가 한 곳에 몰린다. 스케줄러가 병목이 된다 |
| 응답을 압축 | 압축 CPU 가 추론 경로에 들어간다. CPU 가 이미 병목 |
| 클라이언트가 후처리 | 지금 방식이다. 벤치 도구와 비교 스크립트가 blob 을 이해한다 |

## 결과

**얻은 것**

- 노드가 하는 일이 **전처리 → NPU → 직렬화** 로 좁고 균일하다
- 노드별 처리 시간이 입력 내용에 덜 좌우된다
- 측정 대상이 깨끗하다

**잃은 것 / 대가**

- 응답이 1.2 MB 다. 검출 결과만 보내면 수 KB 로 끝날 것을
- **받는 쪽이 blob 을 이해해야 한다.** 형식을 바꾸면 세 곳을 같이 고쳐야
  한다 (blob.rs / dump_output_test.c / compare_detections.py)
- 실사용 API 로서는 불친절하다. "검출 결과를 주는" API 가 아니다

**새로 생긴 제약**

- 클라이언트가 역양자화와 NMS 를 모두 책임진다
- 네트워크 예산이 응답 크기에 묶여 있다. 입력 크기를 키우는 실험(S6)에서는
  응답도 함께 커진다

## 뒤집힌다면

**이 ADR 은 뒤집히는 것이 예정되어 있다.**

- **CPU 병목이 해소되면** (냉각 조건 B, 또는 전처리 최적화) 후처리를 노드로
  옮길 여유가 생긴다
- **실사용 API 가 요구사항이 되면** 원시 텐서 반환은 유지하기 어렵다
- **입력 크기를 키우는 실험에서 네트워크가 다시 막히면** 후처리가 가장
  효과적인 수단이 된다 — 응답이 수 KB 로 줄어 RX 가 사실상 사라진다

뒤집을 때 **반드시 함께 측정할 것**: 후처리를 노드에 넣기 전후의 지속
처리량과 CPU 클럭 강등 시점. 응답 크기만 보고 판단하면 안 된다.

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

# ADR-028. 벤치 도구가 run 유효성을 스스로 판정하고, 경고를 숫자보다 먼저 출력한다

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-11 |
| **관련** | [ADR-002](#adr-002), [ADR-015](#adr-015), [ADR-016](#adr-016) |

---

## 한 줄 요약

> 과거의 측정 실수를 **주석이 아니라 도구에 박아 넣었다.** 예열 제외,
> `boot_id` 로 재부팅 감지, 표본 부족 판정, 실패를 처리량에서 제외,
> 백분위 보간 금지. 그리고 **무효 경고를 숫자보다 위에 출력한다.**

## 배경

`npuforge-bench` 는 새로운 측정이 아니라 **도구**다. 그런데 이 도구 설계의
근거가 전부 앞선 실패에서 나왔다.

지금까지의 측정 실수를 모으면 성격이 셋이다.

```text
A. 지표가 무엇을 세는지 확인하지 않았다
B. 조건이 달라진 것을 모르고 값을 비교했다
C. 무효한 데이터를 유효한 것으로 취급할 뻔했다
```

**주석으로 "조심하자" 고 적어 두는 것은 통하지 않았다.** 세 번 다 알고
있으면서 당했다. 그래서 도구가 강제하게 했다.

## 결정

**1. 과거 실수를 규칙으로 박는다.**

| 과거 실수 | 도구가 하는 일 |
|---|---|
| 첫 추론 지연이 튄다 | 예열 요청을 집계에서 제외 |
| 리셋된 보드를 "성능 저하" 로 읽음 | `boot_id` 변화 → run 무효 |
| 표본 20건으로 p99 를 냈다 | 성공 100건 미만이면 무효 |
| — | 실패를 처리량·노드 몫에서 제외 |
| — | 조건(동시성·시드·정책·노드 수)을 결과에 동봉 |
| — | 백분위는 nearest-rank, 보간 금지 |

**2. 무효 경고를 숫자보다 먼저 출력한다.**

```text
!!!!!! 이 run 은 무효다 !!!!!!
  - 오류율 100.00% 가 허용치 1.00% 를 넘었다
  - 성공 표본 0건은 최소 100건에 못 미친다
아래 수치를 인용하지 말 것.

요청 : 200 (성공 0 / 실패 200, ...)
```

**3. 무효 run 을 삭제하지 않는다.** 사유와 함께 남긴다.

**4. 정책 이름은 스케줄러가 보고한 값을 우선한다.**

**5. 도구가 보장하지 않는 것을 결과 파일에 적는다.**

## 근거

### 실패를 처리량에 넣으면 안 되는 이유

넣으면 **노드가 전부 죽었을 때 처리량이 가장 높아진다.** 실패는 즉시
반환되므로 초당 건수가 폭증한다.

```text
S4 장애 대응 실험에서 이 지표를 그대로 보면
  →  "장애 시 성능 향상"  이라는 결과가 나온다
```

노드 몫도 같다. 실패 요청의 `node_id` 는 비어 있는데, 그것을 세면 **죽은
노드가 "많이 처리한" 것**으로 잡힌다.

### 백분위를 보간하지 않는 이유

선형 보간은 표본이 적을 때 **실제로 관측되지 않은 값**을 만든다.

```text
관측값 1~10 에서 p95 를 보간하면  →  9.55
그런 지연을 겪은 요청은 없다
```

발표 자료에 "p95 = 9.55 ms" 라고 적으면 그건 측정값이 아니라 계산물이다.
nearest-rank 로 고정하고 정의를 모듈 문서에 박았다.

### 경고를 위에 두는 이유

**숫자를 먼저 보여주면 사람은 그것부터 믿는다.** 경고를 아래에 두면 스크롤
없이 보이는 첫 화면이 숫자가 되고, 그 숫자가 표에 옮겨 적힌다.

### 무효 run 을 지우지 않는 이유

사유와 함께 남아야 원인을 추적할 수 있다. 그리고 **재부팅이 반복되면 그
자체가 발견이다** — 실제로 전원 어댑터 문제를 그렇게 찾았다.

### 정책 이름을 스케줄러에서 가져오는 이유

`--policy round-robin` 을 손으로 적으면 틀린다. **틀린 정책 이름이 붙은
결과는 S3 정책 비교 실험을 통째로 망친다.**

### 구현 중에 잡은 문제 하나

처음에는 노드 상태를 하트비트 RPC 로 조회하려 했다. 스케줄러에 노드 목록
API 가 없었기 때문이다.

**그런데 그것이 스케줄러의 노드 상태를 덮어쓴다.** 하트비트는 관측값을
기록하는 호출이라, 벤치가 빈 `health` 를 보내면 스케줄러가 그것을 실제
관측으로 받아들여 온도·큐 깊이를 0 으로 만든다. **측정 직전에 측정 대상의
상태를 오염시키는** 셈이다.

읽기 전용 `ListNodes` RPC 를 따로 만들었다. 이것도 A 유형(부작용을 확인하지
않고 API 를 씀)의 변종이다.

## ⚠️ 도구가 보장하지 않는 것

**닫힌 모델(closed loop) 부하다.** 동시성 N 을 고정하고 응답을 받은 뒤 다음
요청을 보낸다.

이 방식은 **coordinated omission** 에 취약하다. 시스템이 느려지면 클라이언트도
덩달아 천천히 보내므로 **지연 분포가 낙관적으로 나온다.** 느린 요청이 뒤이을
요청의 발사 시각을 미루는데, 그 미뤄진 시간은 어느 요청의 지연에도 계상되지
않는다.

→ **절대 지연을 SLA 처럼 인용하지 않는다. 구성 간 비교에만 쓴다.**
이 문장을 결과 파일의 `caveats` 에 넣어 결과만 떼어 봐도 알 수 있게 했다.

열린 모델(목표 RPS 고정)을 쓰지 않은 이유는 노드 큐가 유한하기 때문이다.
RPS 를 올리면 금방 `NPF-1303` 거절로 끝나 지연 분포를 볼 수 없다. 둘 다
필요하면 M7 에서 추가한다.

## 대안과 버린 이유

| 대안 | 버린 이유 |
|---|---|
| 사람이 결과를 보고 판단 | 146 run / 23.4시간 무인 야간 실행에서는 불가능 |
| 규칙을 문서로 남긴다 | 통하지 않는다는 것이 이미 확인됨 |
| 무효 run 자동 삭제 | 원인 추적 불가. 반복 패턴 자체가 정보다 |
| 백분위 선형 보간 (일반적 관행) | 관측되지 않은 값을 만든다. 표본이 적을 때 특히 위험 |
| 열린 모델 부하 | 노드 큐가 유한해 거절로 끝난다 |

## 결과

**얻은 것**

- 무효 데이터가 결과 표로 넘어가지 않는다
- 무인 실행에서도 유효성이 자동 판정된다
- 도구의 한계가 결과 파일 안에 적혀 있다

**잃은 것 / 대가**

- 유효 판정 기준(성공 100건, 오류율 1%)이 임의값이다. 근거를 더 다듬을 여지
- closed loop 의 낙관적 지연을 안고 간다

**새로 생긴 제약**

- **절대 지연을 SLA 로 인용하면 안 된다.** 구성 간 비교 전용
- 새 실수를 겪으면 여기에 규칙이 추가된다

## 뒤집힌다면

- **M7 에서 열린 모델을 추가하면** 지연 분포 해석이 달라진다. 두 모델의
  결과를 섞지 않는다
- 유효 판정 임계값은 S0 이후 실제 분포를 보고 조정할 수 있다

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
