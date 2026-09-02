# NPUDure 아키텍처 개요

*[English](OVERVIEW.md) — 영문이 정본이다.*

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
질문을 계속 받는 항목이라 [ADR-001](001-data-parallel-only.md) 에 따로
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
| **결정 전체 목록** | **[README.md](README.md)** |
| 왜 모델을 쪼개지 않나 | [ADR-001](001-data-parallel-only.md) |
| 왜 스케줄러가 하나뿐인가 | [ADR-003](003-central-simple-scheduler.md) |
| 왜 하드웨어 없이 전부 돌아가나 | [ADR-004](004-backend-abstraction-mock-first.md) |
| 왜 NPU 컨텍스트를 스레드마다 만드나 | [ADR-007](007-per-thread-rknn-context.md) |
| 왜 INT8 인가 | [ADR-011](011-int8-quantization.md) |
| 왜 노드가 float 이 아니라 정수를 보내나 | [ADR-012](012-want-float-zero-blob-v2.md) |
| 왜 팬을 안 다나 | [ADR-013](013-fanless-thermal-as-measurement.md) |
| 왜 지금 측정을 안 하고 기다리나 | [ADR-014](014-10g-aggregation-separate-scheduler.md) |
| 왜 측정 전에 preflight 를 돌리나 | [ADR-015](015-preflight-hard-fail.md) |
| 무엇이 어떻게 동작하나 (전체 명세) | `docs/01-TECHSPEC.md` |
| 어떤 수치가 나왔나 | `docs/RESULTS.md` |
| 지금 뭘 해야 하나 | `docs/TODO.md` |
| 값의 최종 기준 | `docs/environment-matrix.md` |
