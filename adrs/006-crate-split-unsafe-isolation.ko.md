# ADR-006. 크레이트를 7개로 나누고 `unsafe` 를 한 곳에 가둔다

*[English](006-crate-split-unsafe-isolation.md) — 영문이 정본이다.*

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 |
| **관련** | [ADR-004](004-backend-abstraction-mock-first.md), [ADR-005](005-rknn-feature-gate-off-by-default.md), [ADR-007](007-per-thread-rknn-context.md) |

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
([ADR-007](007-per-thread-rknn-context.md)).

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
