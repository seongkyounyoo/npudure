# S0 — Sustained Load (조건 A 팬리스 / 조건 B 능동 냉각)

- 실험 ID: **S0-A · S0-B**
- 측정일: 2026-08-21
- 코드: `bb3f7ab` + `[transport] node_connections = 2`
- 상태: **둘 다 완료** (각 30 run × 60초 ≈ 31분 연속)
- 원본: [`../../results/sustained-20260821-fan/`](../../results/sustained-20260821-fan/) ·
  [`../../results/sustained-20260821-fanless/`](../../results/sustained-20260821-fanless/)
- 선행: [`S3_8_OPTIMIZED_SCALEOUT.md`](S3_8_OPTIMIZED_SCALEOUT.md)

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

![31분 연속 — 능동 냉각 −1.9% vs 팬리스 −11.3%](../../results/sustained-20260821-fanless/figures/fig_sustained_thermal.png)

**`fig_sustained_thermal.png`** — 31분 연속 — 능동 냉각 −1.9% vs 팬리스 −11.3%

재생성: `python scripts/make-experiment-figures.py`
