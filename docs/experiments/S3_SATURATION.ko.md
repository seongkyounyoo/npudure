# S3 — Per-configuration Saturation

*[English](S3_SATURATION.md) — 영문이 정본이다.*

- 실험 ID: **S3**
- 측정일: 2026-08-20
- 동결 commit: `1da69d4` (bench `254d560` 코드, S2 와 동일). 측정 중 무변경
- 상태: **완료 (45 runs)**
- 원본: [`../../results/saturation-20260820/raw/`](../../results/saturation-20260820/raw/) · 그래프: [`figures/fig3`](../../results/saturation-20260820/figures/fig3_saturation_sweep.png)
- 선행: [`S2_GRPC_BASELINE.md`](S2_GRPC_BASELINE.md)

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
- 스크립트: [`scripts/run-saturation-sweep.sh`](../../scripts/run-saturation-sweep.sh).

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

→ [Figure 3](../../results/saturation-20260820/figures/fig3_saturation_sweep.png)

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
  다르게 나타날 수 있다([`adrs/028`](../../adrs/028-bench-run-validity.md)).

## 6. Reproduction

```bash
bash scripts/run-saturation-sweep.sh    # 45 run → server:/tmp/sat30
python scripts/make-figures.py          # Figure 3 재생성
```
동결 commit `1da69d4`.

## 7. Raw Data & Conclusion

- 원본 45건: [`../../results/saturation-20260820/raw/`](../../results/saturation-20260820/raw/)
  (`sat_n{노드}_c{concurrency}_r{라운드}.json`)

**Conclusion.** 각 구성의 처리량 상한은 1/2/3-node 에서 **115 / 232 / 342 inf/s**
이며, 3-node 는 1-node 상한의 **2.97× (99%)** 로 **ceiling 기준으로도
near-linear** 하다. 포화는 노드당 ~10–16 동시에서 일어난다. 이로써 S2 의
선형 확장 결론이 최대 처리량 관점에서도 재확인됐다.

→ 다음: **S4 (io_uring)** — 이 baseline 과 동일 조건에서 payload-transfer
경로(S2 §8: non-inference latency 의 94%) 비용을 얼마나 줄이는지 비교한다.
