# ADR-004. 백엔드를 인터페이스로 분리하고, Mock 을 1급 백엔드로 둔다

*[English](004-backend-abstraction-mock-first.md) — 영문이 정본이다.*

| | |
|---|---|
| **상태** | 확정 |
| **날짜** | 2026-08-06 (최초), 2026-08-19 (ADR 로 정리) |
| **관련** | [ADR-005](005-rknn-feature-gate-off-by-default.md) (feature gate), [ADR-007](007-per-thread-rknn-context.md), `docs/03-DEVELOPMENT-REQUIREMENTS.md` §4.1 |

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
잡지 못한다. 실제로 [ADR-007](007-per-thread-rknn-context.md) 의 컨텍스트
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
