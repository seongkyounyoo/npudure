# S0-C 4차 — **INVALID: concurrent harness collision**

> **Do not use for performance conclusions.**
> 성능 결론에 쓰지 않는다.

2026-08-21. 강한 이질(2.4×)에서 LQ vs ECT 를 판정하려던 4차 시도의
잔해다. 실험 자체는 [`S0_C_POLICY_AB.md`](../../docs/experiments/S0_C_POLICY_AB.md)
§17~19, 사고 경위는 [`S0_D_CAPACITY_HETERO.md`](../../docs/experiments/S0_D_CAPACITY_HETERO.md)
§4 에 있다.

## 무엇이 유효하고 무엇이 아닌가

| 구간 | 상태 | 비고 |
|---|---|---|
| `round-robin,1` | **유효** | 게이트 판정(§18.1)의 근거. 편차 1.10× |
| `raw/thermal/*.log` | **유효** | 1초 열 로거. §18.2 의 soc·CPU 클럭 집계 근거 |
| `least-queue,1` · `ect,1` | **무효** | 이 시점에 팬이 켜져 팬리스 조건이 아니다 |
| `least-queue,2` | **무효** | 위 + 두 번째 하네스의 c36 벤치와 충돌 |

## 왜 무효인가

두 가지가 겹쳤다.

1. **냉각 조건이 실험 도중 바뀌었다.** 중단했다고 판단하고 팬을 켰는데
   하네스가 살아 있어 팬리스 라벨로 계속 측정했다.
2. **하네스 충돌.** 중단이 실패한 정책 A/B 하네스와 새로 띄운 capacity
   교정 하네스가 **같은 3노드를 각각 c36 으로 때렸다**(합 72).

`least-queue,2` 의 208.5 inf/s 는 정책 성능이 아니라 충돌의 산물이다.
같은 시각 정리 후 재측정한 값은 **391.2 inf/s / 오류 0 / 편차 1.02×** 다.

## 남겨 두는 이유

이 사고 자체가 방법론 기록이다 —
[`experiments/README.md`](../../docs/experiments/README.md) §4.11
("중단했다 를 믿지 말고 공유 자원 쪽에서 확인한다"). 재발 방지로
`npuforge_assert_cluster_free` 가드가 추가됐고, 그 근거가 이 데이터다.

`raw/harness.log` 에 충돌 구간이 그대로 남아 있다.
