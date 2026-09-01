# S3.7 — Connection Tuning (a: sweep, b: concurrency, c: RPS)

- 실험 ID: **S3.7a · S3.7b · S3.7c** (완료)
- 측정일: 2026-08-20
- 코드: `4e64bf4` (`[transport]` 설정. 기본값은 동결과 동일 동작)
- 원본: [`../../results/connection-sweep-20260820/`](../../results/connection-sweep-20260820/)
- 선행: [`S3_6_H2_CHANNEL_AB.md`](S3_6_H2_CHANNEL_AB.md)

---

## 0. 이 실험이 답하는 것

S3.6 은 커넥션 1 → 4 만 비교해 +21.5% 를 봤다. **4 가 최적이라는 근거는
없었고**, 처리량이 오르는 대신 **p95 가 46% 나빠졌다.**

그래서 S3.7 은 "최대 처리량 찾기" 가 아니라 **운영점 선택(operating point
selection)** 문제로 둔다.

```text
S3.7a  커넥션 1/2/4/8/16 @ c32 고정      → Pareto 후보 선정        ← 완료
S3.7b  상위 후보에 대해 concurrency sweep → 실제 operating point 확정
S3.7c  그 운영점에서 RPS OFF/ON           → optimized gRPC 동결
```

---

# S3.7a — Fixed-load connection-count A/B

## 1. Method

1노드(king), **c32 고정**, 60초, 커넥션 1/2/4/8/16, 조건당 **5 run** (총 25).
window 는 기본값(S3.6 결론: 64 MB 급 확대는 −36.3%). 라운드마다 순서를 뒤집어
온도·시간 경과가 한 조건에 몰리지 않게 했다. run 마다 노드의 실제 TCP 커넥션
수를 `ss` 로 세어 기록했다.

> **이 실험은 각 설정의 ceiling 이 아니다.** 부하를 c32 로 고정했으므로
> 커넥션 수의 *순수 효과* 비교로는 좋지만, 커넥션을 늘리면 saturation
> concurrency 가 c32 위로 이동했을 수 있다. 그래서 S3.7b 가 따로 있다.

## 2. Results

오류율 전 구간 **0**. 지연은 모두 **run-level percentile 의 run 간 평균**
(pooled 아님 — S2 §7.4.1).

| conn | TCP 실측 | throughput | vs c1 | p50 | p95 | p99 | max | →node |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **1** | 1 | 115.6 ± 0.7 | — | 268.0 | **392.4** | **452.4** | 597.6 | 114.9 |
| **2** | 2 | **134.4 ± 0.7** | **+16.3%** | 226.7 | **438.2** | **514.9** | 679.7 | 92.5 |
| **4** | 4 | **139.5 ± 0.2** | **+20.7%** | 169.5 | 561.6 | 698.2 | 944.5 | 63.1 |
| **8** | 8 | 139.1 ± 0.6 | +20.4% | 157.4 | 597.0 | 827.0 | 1222.5 | 56.9 |
| **16** | 16 | 136.8 ± 0.7 | +18.4% | 173.4 | 584.2 | 895.1 | 1481.5 | 65.3 |

- 대표 그림: [`fig_sweep_pareto.png`](../../results/connection-sweep-20260820/figures/fig_sweep_pareto.png)
  (X=p95, Y=throughput, 점=커넥션 수)
- 보조: [`fig_sweep_throughput.png`](../../results/connection-sweep-20260820/figures/fig_sweep_throughput.png),
  [`fig_sweep_latency.png`](../../results/connection-sweep-20260820/figures/fig_sweep_latency.png)

## 3. Interpretation

### 3.1 connection parallelism 에는 knee 가 있다

> ⚠️ **[정정 — §4]** 아래는 전부 **c32 고정** 측정이다. 세 구성 모두 운영점이
> c12 이므로 c32 는 overload 구간이고, 여기서 보이는 tail 악화는 상당 부분
> **커넥션 수가 아니라 과부하 큐잉** 때문이다. knee 가 존재한다는 관찰은
> 유효하지만, 그 knee 는 **connection knee 이자 동시에 concurrency knee 와
> 얽혀 있다**(§0).

처리량은 **c4 에서 평평해진다**(139.5). c8 은 139.1 로 사실상 같고 c16 은
136.8 로 오히려 내려간다. 반면 tail 은 **단조롭게 악화**한다.

```text
p99   c1 452  →  c2 515  →  c4 698  →  c8 827  →  c16 895
max   c1 598  →  c2 680  →  c4 945  →  c8 1223 →  c16 1482
```

**데이터가 증명한 것은 여기까지다.**

> **c4 이후의 추가 connection parallelism 은 c32 workload 에서 처리량을
> 개선하지 못하고 tail latency 를 악화시킨다.**

⚠️ **"커넥션 관리 비용·queueing 때문에 꺾인다" 는 아직 원인 확정이 아니다.**
가능한 기여 요인이 섞여 있고 어느 것도 분리하지 않았다 — H2 내부 queueing,
커넥션별 in-flight 편차, TCP 처리, NPU 도착 burst 등(S3.6 §4.3 과 같은 목록).

또 하나 흥미로운 것은 **median 과 tail 이 반대 방향으로 움직인다**는 점이다.
커넥션을 늘리면 평균적인 요청은 빨라지는데(p50 268 → 157) 일부 요청은 훨씬
늦어진다(p99 452 → 895). "커넥션이 많을수록 빠르다" 가 아니라 **처리량–tail
트레이드오프가 실재한다.**

### 3.2 진짜 선택은 c2 냐 c4 냐다

> ⚠️ **[정정 — §4]** 이 절의 트레이드오프는 **c32(overload 구간)** 에서 잰
> 것이다. 운영점 c12 에서는 conn4 가 conn2 대비 처리량 +1.2% 에 p95 +1.2% 로
> **거의 무승부**다(§4.1). 아래 "+3.8% 를 위해 tail 28~39%" 는 과부하
> 구간에서만 성립한다.

| | c2 | c4 | c4 가 치르는 값 |
|---|---:|---:|---|
| throughput | 134.4 | 139.5 | **+3.8%** |
| p95 | 438.2 | 561.6 | **+28.2%** |
| p99 | 514.9 | 698.2 | **+35.6%** |
| max | 679.7 | 944.5 | **+39.0%** |

**c4 는 처리량 +3.8% 를 위해 tail 을 28~39% 내준다.** 실시간 추론에서는
나쁜 거래로 보인다.

c1 기준으로 보면 더 분명하다 — c2 는 c4 가 얻은 이득의 **79%**(+16.3 / +20.7)를
tail 비용의 **약 4분의 1**로 얻는다(p95 +11.7% vs +43.1%).

로컬 direct(161.5) 까지의 gap 회수율:

```text
c1 115.6  ─ gap 45.9 ─▶  로컬 161.5
c2 134.4  회수 18.8 (41%)
c4 139.5  회수 23.9 (52%)
```

### 3.3 heuristic 은 c4 를 골랐다 — 그리고 그건 아슬아슬하다

분석기 규칙("처리량 최대의 97% 이상 중 p95 최소")은 **c4** 를 고른다.
c2 가 **96.4%** 로 임계를 **0.6%p 차이로** 놓쳤기 때문이다.

> **임계값을 결과에 맞춰 옮기지 않는다.** 96.4% 를 담으려고 97% → 96% 로
> 내리면 그건 heuristic 이 아니라 사후 합리화다. 규칙은 그대로 두고,
> **규칙이 이 경계에서 결론을 내주지 못한다는 사실 자체를 결과로 기록한다.**
>
> 이것이 §0 에서 "Selected operating point 는 통계적 최적값이 아니라 의도적
> engineering heuristic" 이라고 못 박은 이유다. 표를 같이 내보이는 것도
> 그래서다 — 경계에서는 사람이 판단해야 하고, 판단의 근거가 표에 있어야 한다.

## 4. S3.7a 결론과 다음 수

- **c8·c16 은 S3.7b 후보에서 제외한다.**

  > 이것은 "어떤 concurrency 에서도 c8/c16 이 열등하다" 가 **아니다.**
  > S3.7a 는 c32 fixed-load 라 c8/c16 의 절대 ceiling 을 재지 않았다.
  > 근거는 **우선순위**다 — c32 에서 이미 tail 비용이 이만큼 크므로
  > (p99 827·895, max 1223·1482) 추가 탐색 비용 대비 기대값이 낮다.
  > 필요하면 나중에 되돌아올 수 있다.

- **c2·c4 를 S3.7b 후보로 올린다.** c32 고정에서는 둘의 우열이 갈리지 않는다.
  c2 가 아직 포화가 아니라면 더 높은 concurrency 에서 역전할 수 있고,
  c4 의 tail 이 concurrency 증가에 더 빨리 무너질 수도 있다.

  현 상태의 성격을 요약하면 **c2 = efficiency point, c4 = performance point**
  다. 어느 쪽이 운영점인지는 ceiling 을 봐야 정해진다.

## 5. Limitations

- **각 설정의 ceiling 이 아니다**(§1 주). c32 고정 결과다.
- percentile 은 run-level 평균이라 pooled 보다 tail 을 낮게 보인다.
  조건 간 비교에는 유효하나 절대값 인용은 안 된다(S2 §7.4.1).
- p95/p99 악화의 **원인은 여전히 미검증**이다. S3.6 §4.3 의 후보 5개
  (커넥션별 in-flight 불균형 / H2 내부 큐 편차 / NPU 도착 burst /
  transport queueing / 처리량 상승에 따른 일반적 tail 증가) 중 어느 것도
  배제하지 못했다.
- 1노드 결과다. 3노드면 서버가 커넥션을 N×3 개 들게 된다(S3.8).

## 6. Reproduction

```bash
bash scripts/run-connection-sweep.sh sweep 5     # 25 run, 약 40분
PYTHONIOENCODING=utf-8 python scripts/analyze-connection-sweep.py \
    results/connection-sweep-20260820/raw/results.csv
python scripts/make-sweep-figures.py \
    results/connection-sweep-20260820/raw/results.csv \
    results/connection-sweep-20260820/figures
```

---

# S3.7b — Concurrency sweep

## 0. 튜닝 대상은 1차원이 아니라 2차원이었다

S3.7a·b 를 거치며 드러난 구조가 이것이다. **knee 가 둘 있다.**

```text
Concurrency knee   요청을 몇 개까지 동시에 넣어야 장치를 포화시키는가?
Connection knee    그 요청을 몇 개의 커넥션으로 나누는 것이 효율적인가?
```

즉 튜닝해야 할 것은 "커넥션 수" 하나가 아니라
**load concurrency × connection parallelism 의 2차원 운영점**이다.

이것은 NPUDure 의 원래 질문 —"왜 안 늘어나지?"— 에 정확히 닿는다.
**포화 이후에는 더 밀어넣어도 NPU 가 더 일하는 게 아니라 시스템 안에 큐만
쌓인다.** 아래 §2 가 그것을 실측으로 잡은 것이다.

## 1. 운영 concurrency 의 정의 (실험 규칙)

숫자로 못 박아 둔다. 그러지 않으면 132.8 / 134.1 / 134.3 같은 결과에서
"어디가 knee 냐" 가 매번 사람 판단으로 들어간다.

> **operating concurrency = peak 처리량의 98% 이상을 내는 가장 낮은 concurrency**

**98% 인 이유**: 관측된 run 간 SD 가 ±1 inf/s 수준이라 99% 로 잡으면 임계가
측정 noise 와 겹친다. 이 정의는 `analyze-concurrency-sweep.py` 에 상수로
들어가 있다.

## 2. 1차 범위 (c24~c64) — 전 구간이 overload 였다

후보 **c2 · c4**, 각 3 run.

| conc | conn2 tp | conn2 p95 | conn2 p99 | conn4 tp | conn4 p95 | conn4 p99 |
|---:|---:|---:|---:|---:|---:|---:|
| **24** | **134.3 ± 1.1** | **306.9** | **357.5** | **139.3 ± 1.2** | **390.9** | **480.2** |
| 32 | 133.7 | 431.5 | 505.7 | 138.3 | 576.5 | 715.0 |
| 40 | 134.2 | 572.1 | 674.4 | 137.7 | 719.5 | 932.9 |
| 48 | 133.8 | 697.6 | 832.0 | 137.6 | 958.6 | 1200.9 |
| 64 | 132.9 | 946.0 | 1132.3 | 137.9 | 1254.4 | 1566.7 |

오류율 전 구간 0.

**처리량이 c24~c64 내내 완전히 평평하다**(conn2 ≈ 134, conn4 ≈ 138). 반면
tail 은 거의 선형으로 커진다 — conn4 @c64 는 p99 1567 ms, max 2128 ms.

> **이 범위는 전부 포화 이후 구간이다.** 데이터가 말하는 것:
>
> **처리량 포화는 concurrency ≤ 24 에서 일어난다. 포화 이후의 추가
> concurrency 는 처리량을 늘리지 않고 tail latency 만 증가시킨다.**

전형적인 queueing 이다. 더 밀어넣은 요청은 계산으로 가지 않고 대기열로 간다.

### 2.1 ~~트레이드오프는 부하에 안정적이다~~ — **틀렸다 (§4 에서 반증)**

| | S3.7a @c32 | S3.7b @c24 |
|---|---:|---:|
| throughput | +3.8% | +3.7% |
| p95 | +28.2% | +27.4% |
| p99 | +35.6% | +34.3% |

두 값이 거의 같아서, 처음에는 "특정 concurrency 의 우연이 아니라 **4 커넥션
자체가 만드는 트레이드오프**" 라고 썼다. **그 해석은 틀렸다.**

c32 와 c24 가 일치한 것은 **둘 다 overload 구간이라 같은 현상을 두 번 본
것**이었다. §4 에서 진짜 운영점(c12)으로 내려가자 p95 페널티가
**+28% → +1.2%** 로 사라진다. 커넥션 4개의 성질이 아니라 **포화 이후 큐잉의
성질**이었다.

> 교훈: **두 측정이 일치한다는 것이 곧 그 해석이 옳다는 뜻은 아니다.**
> 둘 다 같은 방향으로 편향돼 있으면 재현성은 편향을 확인해 줄 뿐이다.

### 2.2 그래서 sweep 방향이 틀렸다

두 후보 모두 최고점이 **sweep 하단(c24)** 이다. 즉 포화점은 c24 **이하**에
있고, 운영점(= ceiling 을 내는 가장 낮은 concurrency)을 아직 못 봤다.
→ **c8/c12/c16/c20/c24 로 아래쪽을 다시 훑는다.**

## 3. conn1 baseline 을 같은 범위에서 다시 잰다

이걸 안 하면 해석이 섞인다. 지금 가진 두 점을 나란히 놓으면

```text
conn1 @c32 →  115.6 inf/s,  p95 392
conn2 @c24 →  134.3 inf/s,  p95 307
```

"2 커넥션이 처리량과 지연을 **둘 다** 개선했다" 고 쓰고 싶어진다. 그러나
**변수가 두 개 동시에 바뀌었다** — 커넥션 1→2, concurrency 32→24. 인과를
분리할 수 없다.

각 커넥션 수의 **운영점을 같은 규칙(§1)으로 찾아** 비교해야 질문이 성립한다.

답할 질문은 하나로 좁혀진다.

> **동일한 saturation criterion 에서 connection parallelism 은 처리량과
> tail latency 에 어떤 영향을 주는가?**

## 4. 2차 범위 (c8~c24) — 결과가 뒤집힌다

conn **1 · 2 · 4** 를 **같은 격자**(c8/12/16/20/24)에서 각 3 run, 총 45 run.
오류율 0.

| conc | conn1 tp | conn1 p95 | conn2 tp | conn2 p95 | conn4 tp | conn4 p95 |
|---:|---:|---:|---:|---:|---:|---:|
| 8 | 112.1 | 101.3 | 120.4 | 93.9 | 111.0 | 105.0 |
| **12** | **114.8** | **147.6** | **136.4** | **119.8** | **138.1** | **121.2** |
| 16 | 115.1 | 191.5 | 136.2 | 178.7 | 138.8 | 210.0 |
| 20 | 114.9 | 239.2 | 135.1 | 245.4 | 138.5 | 306.1 |
| 24 | 115.9 | 286.8 | 134.0 | 307.7 | 139.1 | 392.3 |

### 4.1 운영점은 셋 다 c12 다

98% 규칙(§1) 적용 결과:

| connections | operating conc | throughput | p50 | p95 | p99 | peak 대비 |
|---:|---:|---:|---:|---:|---:|---:|
| **1** | **12** | 114.8 | 102.1 | 147.6 | 167.2 | 99.1% |
| **2** | **12** | **136.4** | 85.8 | **119.8** | **137.4** | 100.0% |
| **4** | **12** | 138.1 | 83.4 | 121.2 | 145.7 | 99.3% |

**시험한 세 커넥션 수(1·2·4) 모두에서 98% 기준 운영 concurrency 가 c12 로
관측됐다.**

> **Within the tested range, the concurrency knee remained invariant to
> connection parallelism.**

커넥션 병렬도를 바꿔도 concurrency knee 가 움직이지 않았다는 증거다. 두
knee 가 서로 독립임을 **강하게 시사하지만 증명한 것은 아니다** — 다른 모델,
페이로드 크기, 노드 수, 네트워크에서는 움직일 수 있다. §0 의 2차원 구조는
이 범위 안에서 관측된 것으로 읽어야 한다.

### 4.2 운영점에서는 트레이드오프가 없다 — conn2 가 conn1 을 지배한다

| conn2 vs conn1 @c12 | |
|---|---:|
| throughput | **+18.8%** |
| p50 | **−16.0%** |
| p95 | **−18.8%** |
| p99 | **−17.8%** |

**처리량이 오르면서 지연이 모든 분위에서 함께 내려간다.** 트레이드오프가
아니라 **strict Pareto improvement** 다 — 단, **측정한 처리량·지연 지표
기준**이다(on the measured throughput/latency metrics). CPU·메모리·커넥션
자원까지 포함한 전 시스템 Pareto 라는 뜻은 아니다.
→ [`fig_sweep_pareto.png`](../../results/s37b-operating-point/figures/fig_sweep_pareto.png)

**conn4 가 절대적으로 나쁜 것은 아니다.** 처리량 최고값을 우선한다면 conn4 도
정당한 선택이다(138.1 vs 136.4).

conn2 를 기본 운영점으로 삼는 근거는 "conn4 가 나빠서" 가 아니다.

| conn4 가 더 주는 것 | conn4 가 더 쓰는 것 |
|---|---|
| 처리량 **+1.2%** — 측정 변동(SD ±0.3~1.6)과 가까운 수준 | 커넥션 자원 **2배** |
| | p99 **+6.0%** |

> **2 connections is the lowest-complexity configuration that captures
> nearly all available throughput.**

최소 자원으로 ceiling 을 거의 다 먹기 때문에 conn2 다.

### 4.3 그래서 앞선 "tail 악화" 는 커넥션 탓이 아니었다

S3.6 §4.3 과 S3.7a 는 커넥션을 늘리면 tail 이 나빠진다고 기록했다
(p95 +46%, +43%). **그 측정 자체는 맞지만 해석이 틀렸다.**

그 실험들은 전부 **c32 에서 쟀는데, c32 는 세 구성 모두에게 overload 구간**
이다(운영점이 c12). 즉 그 비교는 "어느 구성이 더 좋은가" 가 아니라
**"어느 구성이 과부하에서 더 완만하게 무너지는가"** 를 잰 것이었다.

```text
c32 에서 본 것   1ch → 4ch :  처리량 +21%, p95 +46%   ← overload 구간 비교
c12 에서 본 것   1ch → 2ch :  처리량 +19%, p95 −19%   ← 운영점 비교
```

**S3.6·S3.7a 의 숫자가 틀린 것이 아니다. 질문이 달랐다.**

| 무엇을 물었나 | 답 |
|---|---|
| **고정 c32 비교** — 각 구성이 **과부하에서** 어떻게 behaving 하는가? | 커넥션이 많을수록 ceiling 은 조금 높지만 **tail amplification 이 커진다** |
| **운영점 비교** — 어느 구성이 **운영상** 더 나은가? | conn2 가 conn1 을 양 축에서 지배한다 |

그래서 c32 결과는 폐기 대상이 아니라 **별도의 유효한 결과**로 남는다 —
과부하 거동에 대한 결과다. 다만 그것을 운영 판단의 근거로 쓰면 안 된다.

> **Optimize at the operating point, not in the overload region.**

운영점을 정의하지 않고 고정 부하에서 구성을 비교하면 **configuration effect
가 아니라 overload behavior 를 보게 되고, 결론이 뒤집힐 수 있다.**
이것이 S3.7 이 남기는 가장 실용적인 교훈이다.

## 5. S3.7b 결론

> **Selected operating point: 커넥션 2개 @ concurrency 12
> — 136.4 inf/s, p95 119.8 ms, p99 137.4 ms**

동일 규칙의 conn1 baseline(114.8 @c12) 대비 **처리량 +18.8%, p95 −18.8%**.
로컬 direct(161.5) 까지의 gap 46.7 중 **21.6 (46%) 를 설정만으로 회수**하면서
tail 도 함께 개선했다.

## 6. Limitations

- **격자 해상도.** knee 는 c8(peak 의 88%)과 c12 사이에 있는데 격자가 4
  단위라 **c12 가 진짜 knee 인지 c10 인지는 모른다.** 세 구성을 같은 격자로
  비교하는 데는 문제없으나, 운영점 절대값으로 인용할 때는 이 한계를 붙인다.
- 분석기가 conn1·conn4 에 "포화 미확인(최고점이 sweep 상단)" 을 찍는다.
  다만 conn1 은 c12 114.8±0.7 vs c24 115.9±0.8, conn4 는 c12 138.1±1.6 vs
  c24 139.1±0.8 로 **평평한 구간 안의 noise** 다. 경고는 보수적으로 남긴다.
- 1노드 결과다. 3노드면 서버가 2×3 = 6 커넥션을 든다(S3.8).
- percentile 은 run-level 평균(S2 §7.4.1).

# S3.7c — RPS at the selected operating point

확정 운영점: **커넥션 2개 @ c12**.

이제 다른 변수를 섞지 않고 질문 하나만 묻는 실험이 된다.

> **Does RPS improve the selected operating point?**

여기서 S3.5b 처럼 다시 null 이 나오면 **그것도 좋은 결과**다. 그때는
"RPS 가 무효였던 건 흐름이 하나뿐이라서" 라는 가설이 상당 부분 약해진다 —
흐름이 2개인데도 변화가 없다면 **IRQ/RX-side 분산이 이 워크로드의 병목이
아니라는 쪽**으로 증거가 쌓인다.

확정된 운영점에서 `rps_cpus` OFF/ON. S3.5b 는 흐름이 하나뿐이라 분산할 대상이
없었다. 이제 흐름이 여러 개이고 S3.6 의 4ch 조건에서 CPU0 는 busy 81% /
soft 74% 였다.

**S3.7b 에서 c2·c4 가 애매하게 비기면 둘 다 RPS A/B 를 한다.** 조건당 10 run
이면 되므로 싸고, **흐름이 2개냐 4개냐에 따라 RPS 효과가 달라질 수 있다** —
그 자체가 ②-a(TCP per-flow 처리)를 겨냥한 정보다.

- 오르면 → 단일 커넥션 제약을 풀자 NIC 처리 병목이 드러난 것
- 그대로면 → CPU0 softirq 는 **상관관계일 뿐 처리량 limiter 가 아니다**
  (S3.5b 단독보다 훨씬 강한 배제)

## 결과 — null 이다. 그리고 이번 null 은 훨씬 강하다

conn2 @ c12 고정, `rps_cpus` = `00`(CPU0만) vs `fe`(코어 1~7), 각 5 run.

| | throughput | p50 | p95 | p99 | 보드 idle | **CPU0 busy** | **CPU0 %soft** |
|---|---:|---:|---:|---:|---:|---:|---:|
| RPS off | **136.8 ± 0.6** | 85.4 | 119.1 | 137.7 | 49.3% | **78.7%** | **68.0%** |
| RPS on | 135.6 ± 0.4 | 86.4 | 119.7 | 139.3 | 49.1% | **74.6%** | **56.0%** |
| 차이 | **−0.8%** | +1.2% | +0.5% | +1.2% | — | −4.1%p | **−12.0%p** |

오류율 0. 처리량 차이 −0.8% 는 SD(±0.4~0.6) 범위다.

### 왜 이 null 이 S3.5b 보다 강한가

S3.5b 때는 반박이 가능했다 — **흐름이 하나뿐이라 RPS 가 분산할 대상 자체가
없었다.** 이번에는 그 반박이 막힌다.

1. **흐름이 2개다.** RPS 가 해시로 나눌 것이 실재한다.
2. **RPS 가 실제로 작동했다.** CPU0 %soft 가 **68.0% → 56.0%** 로 12%p
   내려갔고 CPU0 busy 도 78.7% → 74.6% 로 떨어졌다. 설정이 무시된 것이 아니다.
3. **CPU0 는 놀고 있지 않았다.** busy 78.7% 로 충분히 부하가 걸린 상태였다
   (S3.6 의 c32/4ch 조건 81% 와 비슷하다). "부하가 낮아 RPS 가 손댈 여지가
   없었다" 는 설명도 성립하지 않는다.

> **At the selected operating point, RPS reduced CPU0 softirq load
> substantially but produced no measurable throughput or tail-latency
> improvement. Therefore, CPU0 receive-side processing was not
> performance-limiting under the tested configuration.**

**범위를 정확히 읽어야 한다.** 이 실험이 말하는 것은 "CPU0 softirq 는
limiter 가 아니다" 가 아니라 **"이 운영점·이 구성에서는 limiter 가 아니다"**
다. 다른 부하·모델·페이로드 크기·노드 수에서는 달라질 수 있다.

그 범위 안에서는 상당히 강하다 — mechanism 은 분명히 작동했는데
end-to-end limiter 를 건드리지 못했다. S3.5(§4.3)가 CPU0 을 "다음 병목
후보" 로 지목했던 것은 **이 구성에 한해** 배제된다.

## S3.7 전체 결론

| 후보 | 판정 |
|---|---|
| 링크 대역폭 | 배제 (방향당 51%) — S3.5 |
| 보드 CPU 총량 | 배제 (49~63% idle) — S3.5·S3.7c |
| 서버·스케줄러 | **재개방** — baseline 에서는 배제됐으나 optimized 3N eff 95.3% (S3.8) |
| **CPU0 softirq / RPS** | **배제.** 12%p 덜어도 처리량 불변 — S3.7c |
| H2 flow control window | 64 MB 급 확대는 −36.3% 로 해로움 — S3.6 |
| **노드당 커넥션 수** | **주요 제약.** 1→2 로 +18.8%, tail 도 개선 — S3.7b |
| protobuf·복사·syscall | **미분리.** 남은 15.5% 안에 있을 수 있다 |

**Selected operating point: 노드당 커넥션 2개(2 connections **per node**)
@ concurrency 12 — 136.4 inf/s, p95 119.8 ms, p99 137.4 ms**

> **단위를 반드시 명시한다.** `[transport] node_connections` 는 **노드당**
> 값이다(`GrpcNodePool` 이 `NodeId` 마다 채널을 N 개 만든다). 클러스터 전체
> 합이 아니다.
>
> | 노드 수 | node_connections | 클러스터 전체 커넥션 |
> |---:|---:|---:|
> | 1 | 2 | 2 |
> | 2 | 2 | 4 |
> | 3 | 2 | 6 |
>
> S3.8 에서 "2 connections" 를 클러스터 전체로 고정하면 노드당 조건이
> 보존되지 않고, 3N 에서 **커넥션 공급 자체가 새 병목**이 된다. 완전히
> 다른 실험이 되므로 혼동하면 안 된다.

로컬 direct 161.5 까지 아직 **15.5%** 남아 있다.

> ⚠️ **배제표는 후보 공간을 줄인 것이지, 남은 15.5% 의 정체를 특정한 것이
> 아니다.** 남은 후보는 여전히 여럿이다.
>
> | 남은 gap 의 후보 |
> |---|
> | protobuf serialization |
> | memcpy / buffer ownership (`to_vec()` 등) |
> | syscall / submission path |
> | HTTP/2 구현 오버헤드 |
> | userspace 스케줄링 (tokio 워커 ↔ blocking 풀 경합) |
> | NPU submission / RKNN 런타임 오버헤드 |
> | 그 밖 |

**io_uring 은 이제 정당한 후보가 됐다. 그러나 "다음 병목이 syscall·복사다"
는 아직 아니다.** 그래서 S4 의 질문을 이렇게 둔다.

```text
아니다   io_uring 이 남은 15.5% 를 회수하는가?
맞다     syscall / submission path 가 실제로 유의미한 비용인가?
```

프로파일로 syscall·복사 비용을 먼저 확인하고, 그 답이 "그렇다" 일 때
io_uring 으로 간다. TECHSPEC §15.1 의 순서이자 S3.5 이후 지켜 온 원칙과
같다 — **측정이 구현을 결정한다.**

다음은 **S3.8** — 이 운영점으로 1N/2N/3N scale-out 을 재검증한다.
