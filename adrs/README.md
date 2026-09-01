# NPUDure 아키텍처 결정 기록 (ADR)

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

1. **[OVERVIEW.md](OVERVIEW.md)** — 시스템 전체 지도. ADR 을 읽기 전에 본다
2. **[001](001-data-parallel-only.md), [002](002-success-criteria-measurability.md), [003](003-central-simple-scheduler.md), [004](004-backend-abstraction-mock-first.md)** — 프로젝트의 방향과 골격
3. **[007](007-per-thread-rknn-context.md), [011](011-int8-quantization.md), [012](012-want-float-zero-blob-v2.md), [013](013-fanless-thermal-as-measurement.md)** — NPU 와 열을 실제로 다루며 나온 결정. 실측 밀도가 가장 높다
4. **[015](015-preflight-hard-fail.md), [017](017-remote-exec-pitfalls-library.md), [028](028-bench-run-validity.md)** — "성공처럼 보이는 실패" 를 막는 장치들
5. 나머지는 필요할 때 찾아 읽는다

### 시간이 없다면 셋만

| # | 왜 |
|---|---|
| [007](007-per-thread-rknn-context.md) | 오류 0건에 결과 100% 불일치. 이 프로젝트의 성격을 가장 잘 보여준다 |
| [013](013-fanless-thermal-as-measurement.md) | 먼저 무너지는 것은 NPU 가 아니라 CPU 였다 |
| [002](002-success-criteria-measurability.md) | 왜 나쁜 수치를 그대로 내는가 |

---

## 목록

### 프로젝트 방향

| # | 제목 | 상태 |
|---|---|---|
| [001](001-data-parallel-only.md) | 모델을 쪼개지 않고 요청을 나눈다 (데이터 병렬) | 확정 |
| [002](002-success-criteria-measurability.md) | 성공 기준을 수치가 아니라 측정 가능성으로 둔다 | 확정 |
| [022](022-document-authority-order.md) | 문서마다 규범 영역을 정하고 값이 다르면 규범 문서를 따른다 | 확정 |

### 시스템 구조

| # | 제목 | 상태 |
|---|---|---|
| [003](003-central-simple-scheduler.md) | 스케줄러를 하나만 두고 고가용성을 구현하지 않는다 | 확정 |
| [004](004-backend-abstraction-mock-first.md) | 백엔드를 인터페이스로 분리하고 Mock 을 1급으로 둔다 | 확정 |
| [005](005-rknn-feature-gate-off-by-default.md) | RKNN 링크를 feature 뒤에 두고 기본값을 끈다 | 확정 |
| [006](006-crate-split-unsafe-isolation.md) | 크레이트를 7개로 나누고 `unsafe` 를 한 곳에 가둔다 | 확정 |
| [008](008-grpc-tonic-protobuf.md) | 내부 통신을 gRPC(tonic + Protocol Buffers)로 한다 | 확정 |

### 스케줄링

| # | 제목 | 상태 |
|---|---|---|
| [009](009-three-policies-shared-filter.md) | 정책은 세 개로 고정하고 후보 필터는 셋이 공유한다 | 확정 |
| [010](010-ect-formula.md) | ECT 점수식과 그 안의 각 항 | 확정 (실장비 검증 전) |
| [026](026-retry-different-node.md) | 재시도는 반드시 다른 노드로, 백오프는 짧게 | 확정 |
| [027](027-node-state-machine-drain-disable.md) | 노드 상태 머신과 drain·disable 분리 | 확정 (임계치 초안) |

### NPU 런타임

| # | 제목 | 상태 |
|---|---|---|
| [007](007-per-thread-rknn-context.md) | 스레드마다 전용 RKNN 컨텍스트 — 공유를 타입으로 막는다 | 확정 |
| [011](011-int8-quantization.md) | 기준 모델을 INT8 로 한다 | 확정 |
| [012](012-want-float-zero-blob-v2.md) | 노드는 역양자화하지 않고 정수를 보낸다 (`want_float=0`, blob v2) | 확정 |
| [020](020-worker-count-8-no-core-mask.md) | `worker_count = 8`, `core_mask` 미설정 | 확정 |
| [021](021-no-node-side-postprocessing.md) | 노드는 후처리(NMS)를 하지 않는다 | **잠정** |

### 하드웨어와 측정 환경

| # | 제목 | 상태 |
|---|---|---|
| [013](013-fanless-thermal-as-measurement.md) | 팬리스를 기본으로 두고 throttling 을 측정 대상으로 삼는다 | 확정 |
| [014](014-10g-aggregation-separate-scheduler.md) | aggregation 만 10G, 스케줄러는 별도 서버 | 확정 (구축·실측 완료) |
| [018](018-convert-model-once-deploy.md) | 모델은 한 번만 변환해 세 노드에 배포한다 | 확정 |
| [019](019-ssh-alias-not-ip.md) | 보드는 IP 가 아니라 SSH 별칭으로 접근한다 | 확정 |
| [023](023-cpu-governor-performance-scoped.md) | CPU governor 를 `performance` 로 — 단 근거의 범위를 명시 | **잠정** |

### 측정 규율

| # | 제목 | 상태 |
|---|---|---|
| [015](015-preflight-hard-fail.md) | 측정 전 preflight 하드 실패 검사 | 확정 |
| [016](016-boot-id-run-invalidation.md) | `boot_id` 로 측정 중 재부팅을 감지해 run 을 무효화한다 | 확정 |
| [017](017-remote-exec-pitfalls-library.md) | 원격 실행 함정을 라이브러리 함수로 굳힌다 | 확정 |
| [028](028-bench-run-validity.md) | 벤치 도구가 run 유효성을 스스로 판정한다 | 확정 |

### 프로토콜과 정책 세부

| # | 제목 | 상태 |
|---|---|---|
| [024](024-error-code-scheme.md) | 오류를 `NPF-xxxx` 코드 체계로 고정한다 | 확정 |
| [025](025-heartbeat-failure-reregister.md) | 하트비트 실패는 곧바로 재등록 — 등록은 멱등 | 확정 |

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
| [015](015-preflight-hard-fail.md) | 전제가 틀린 채로 측정을 시작하는 것 |
| [016](016-boot-id-run-invalidation.md) | 재부팅을 "성능 저하" 로 읽는 것 |
| [017](017-remote-exec-pitfalls-library.md) | 원격 명령이 실패했는데 종료 코드 0 인 것 |
| [019](019-ssh-alias-not-ip.md) | 엉뚱한 보드의 결과를 다른 노드에 귀속시키는 것 |
| [028](028-bench-run-validity.md) | 무효한 run 의 숫자가 결과 표로 넘어가는 것 |

---

## 새 ADR 을 쓸 때

1. [TEMPLATE.md](TEMPLATE.md) 를 복사한다
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
