# ADR-005. RKNN 링크를 feature 뒤에 두고 기본값을 끈다

*[English](005-rknn-feature-gate-off-by-default.md) — 영문이 정본이다.*

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 |
| **관련** | [ADR-004](004-backend-abstraction-mock-first.md), [ADR-006](006-crate-split-unsafe-isolation.md) |

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

[ADR-004](004-backend-abstraction-mock-first.md) 에서 "하드웨어 없이 전체가
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
