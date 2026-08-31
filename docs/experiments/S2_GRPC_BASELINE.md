# S2 — gRPC Multi-node Scaling Baseline

- 실험 ID: **S2**
- 측정일: 2026-08-20
- 동결 commit: `254d560` (측정 중 코드·설정 무변경)
- 상태: **완료 · 재현 확인 (30 runs)**
- 원본 데이터: [`../../results/baseline-20260820/raw/`](../../results/baseline-20260820/raw/) · 그래프: [`figures/`](../../results/baseline-20260820/figures/) · 대시보드: [`dashboard.html`](../../results/baseline-20260820/dashboard.html)

---

## 1. Research Question

> **Does aggregate inference throughput increase approximately linearly as
> identical low-cost NPU nodes are added to an Ethernet-connected edge cluster?**

저비용 엣지 NPU(RK3576, 6 TOPS)를 이더넷으로 묶었을 때, **노드를 늘리면 전체
추론 처리량이 선형에 가깝게 증가하는가.** 명목 TOPS 합산이 아니라 실측
확장 효율을 묻는다.

## 2. Hypothesis

데이터 병렬 구조([`adrs/001`](../../adrs/001-data-parallel-only.md))에서 노드는
서로 독립적으로 서로 다른 요청을 처리한다. 노드 간 통신이 추론 경로에 없으므로,
**단일 중앙 스케줄러가 병목이 되지 않는 한 처리량은 노드 수에 선형**일 것으로
예측한다. 동시에, 클러스터 경유(gRPC + 네트워크)는 로컬 직접 추론보다 노드당
처리량을 **일정 비율 낮출** 것으로 본다(오버헤드).

## 3. System Under Test

| 항목 | 값 |
|---|---|
| Board | NanoPi R76S ×3 (king / queen / jack) |
| SoC / NPU | Rockchip RK3576 / 2-core 6 TOPS |
| Model | YOLOv8n **INT8** (sha256 `dba155d2…`), `want_float=0` |
| Input | raw RGB 640×640×3 = 1,228,800 byte/request |
| Scheduler host | server (.9): Xeon E5-2630L ×2 (24T) / 16GB / Rocky 9.4 |
| Network | worker 2.5GbE / aggregation 10GbE (NEXI NS-S25G10G-N) |
| Transport | **gRPC** (tonic + protobuf) |
| Topology | client → scheduler(.9) → node, 3-hop 전부 gRPC |

토폴로지·근거: [`adrs/014`](../../adrs/014-10g-aggregation-separate-scheduler.md),
[`docs/infrastructure.md`](../infrastructure.md).

## 4. Experimental Controls

모든 run 에서 고정한 조건.

```text
Cooling      : Active cooling — 120mm 5V USB fan per node (측정 시작부터)
CPU governor : performance
Policy       : round-robin
Worker count : 8 / node  (스레드마다 전용 RKNN 컨텍스트, adrs/007)
Transport    : gRPC
Model        : YOLOv8n INT8, want_float=0
Warmup       : 제외
```

- **냉각은 Active Cooling(팬 ON).** 팬리스가 아니다 —
  [`docs/board-worklog.md`](../board-worklog.md) §2.24·§2.27 참조.
- 측정 전 `preflight-check.sh` 통과(별칭↔hostname, 해시, governor, 온도, 전압, NTP).

## 5. Measurement Method

- 부하 도구: `npuforge-bench` (**closed-loop**), server(.9)에서 실행.
- 노드당 동일 부하: **concurrency = 8 × 노드수** (1N c8 / 2N c16 / 3N c24).
- 각 조건 **10 runs, 60초**. 총 30 runs.
- **조건 순서를 rotate** 해 시간·온도 변동을 한 조건에 몰지 않는다:
  ```text
  Round 1: 1N → 2N → 3N
  Round 2: 2N → 3N → 1N
  Round 3: 3N → 1N → 2N   (반복)
  ```
- 노드 축소는 프로세스 중지, run 사이 cooldown.
- 스크립트: [`scripts/run-grpc-baseline30.sh`](../../scripts/run-grpc-baseline30.sh).
  측정 30회 동안 코드·설정 동결.

> closed-loop 특성상 절대 지연은 SLA 로 인용하지 않고 **구성 간 비교**에만
> 쓴다([`adrs/028`](../../adrs/028-bench-run-validity.md)).

## 6. Validation / Integrity Checks

30 runs 전수 검사. **측정 신뢰성의 근거다.**

| 검사 | 결과 |
|---|---|
| run 수 | 30 / 30 |
| active node 판정 | **30/30 정확** (n1=1, n2=2, n3=3) |
| invalid run (verdict) | 0 |
| 오류율 (inference) | **0.00%** (전 run) |
| 재시도 | 0 건 |
| load-balance 편차 | **0.00 %p** |

- active node 는 등록 노드가 아니라 **실제 요청을 처리한 노드**(`per_node`)로
  판정한다. 노드를 중지해도 스케줄러 등록이 남는 문제를 bench 수정으로 해결했다
  (board-worklog §2.28).
- 재시도 카운트는 응답 프로토콜의 `attempts` 필드에서 온다(스케줄러 실제 시도).

## 7. Results

### 7.1 Throughput

| Nodes | Throughput Mean ± SD |
|---:|---:|
| 1 | **112.9 ± 0.5** inf/s |
| 2 | **229.0 ± 0.9** inf/s |
| 3 | **338.4 ± 1.1** inf/s |

SD 가 0.5~1.1 로 극히 작다 — 30회에 걸쳐 처리량이 사실상 흔들리지 않았다.
첫 측정값 337.7 이 338.4 ± 1.1 로 재현됐다. → [fig1](../../results/baseline-20260820/figures/fig1_throughput_vs_node.png)

### 7.2 Speedup

| 기준 | 2N | 3N |
|---|---:|---:|
| 1-node c8 (112.9) | 2.03× | **3.00×** |
| single-node saturation (~115) | 1.99× | 2.94× |

### 7.3 Scaling Efficiency

1-node c8 기준 **100% / 101% / 100%**, saturation 기준 3N ≈ **98%**.
→ [fig2](../../results/baseline-20260820/figures/fig2_scaling_efficiency.png)

### 7.4 Latency (round-trip, closed-loop)

**run-level percentile 의 30회 평균**(§7.4.1 주의 참조):

| Nodes | p50 | p95 | p99 |
|---:|---:|---:|---:|
| 1 | 68.0 | 100.8 | 116.3 ms |
| 2 | 67.0 | 100.1 | 118.6 ms |
| 3 | 67.6 | 102.7 | 123.9 ms |

노드 수가 늘어도 지연 분포가 거의 평탄하다 — 확장이 지연을 악화시키지 않는다.

#### 7.4.1 주의 — 이 값은 pooled percentile 이 아니다

각 run 안에서 그 run 의 요청 전체로 percentile 을 계산하고(nearest-rank,
`stats.rs`), **그 run-level 값들을 다시 평균**한 것이다. 30회 요청을 전부
합쳐 다시 정렬해 구한 값(pooled percentile)과는 다르다.

```text
쓴 것    mean( p99(run1), p99(run2), ..., p99(run30) )
아닌 것  p99( run1 ∪ run2 ∪ ... ∪ run30 )
```

일반적으로 **run-level 평균은 pooled 보다 tail 을 낮게 보이게 한다** — 각
run 의 최악 구간이 평균에 희석되기 때문이다. 구성 간 *비교* 에는 문제가
없지만(모든 조건이 같은 방식) **절대값을 "이 시스템의 p99" 로 인용하면 안 된다.**

pooled 를 내려면 per-request 지연 원본이 필요한데 bench 는 요약 percentile 만
JSON 에 남긴다. 원본 덤프 옵션 추가는 `TODO.md` §1.2 에 올려 두었다.
→ [fig4](../../results/baseline-20260820/figures/fig4_latency_percentiles.png)

### 7.5 Load Distribution

round-robin 이 3노드를 **정확히 33.3%씩** 나눴다(편차 0.00 %p).
→ [fig5](../../results/baseline-20260820/figures/fig5_per_node_distribution.png)

## 8. Timing Breakdown

응답 `Timing`(proto) 11단계, 30회 p50 평균 (ms):

| 단계 | 1N | 3N |
|---|---:|---:|
| scheduler_queue | 0.00 | 0.00 |
| scheduler_route | 0.00 | 0.00 |
| **network_to_node** (input) | 17.72 | 17.11 |
| node_queue | 0.02 | 0.02 |
| **inference (NPU)** | 24.70 | 22.49 |
| **network_to_client** (output) | 17.72 | 17.11 |
| **end_to_end** | 61.54 | 58.83 |

```text
non-inference overhead = end_to_end - inference = 58.83 - 22.49 = 36.34 ms
payload transfer       = network_to_node + network_to_client = 34.21 ms
```

- scheduler_queue/route 는 노드 수와 무관하게 **~0** — 단일 스케줄러가 3노드
  동시에도 병목이 아니다([`adrs/003`](../../adrs/003-central-simple-scheduler.md) 실측 확인).
- 1N·3N 의 `network_to_node`(17.72 vs 17.11)가 거의 같다 — 단일 요청 전송
  시간은 노드 수와 무관.
- → [fig7](../../results/baseline-20260820/figures/fig7_timing_breakdown.png)

## 9. Local vs Cluster Overhead

| Mode | Cooling | Worker | Throughput |
|---|---|---:|---:|
| Local direct RKNN (no gRPC) | Active Cooling | 8 | 161.5 inf/s |
| Cluster gRPC (single node) | Active Cooling | 8 | 112.9 inf/s |

**Throughput loss = (161.5 − 112.9) / 161.5 = 30.1%.**
로컬 baseline 은 냉각·worker 를 클러스터와 맞춰 재측정했다(board-worklog §2.27).
→ [fig8](../../results/baseline-20260820/figures/fig8_local_vs_cluster.png)

> ⛔ **두 측정량을 곱하지 않는다.** throughput loss(30.1%, 처리량)와 latency
> breakdown(94%, 지연 구성비)은 서로 다른 축이다. §10 의 문장을 쓴다.

## 10. Interpretation

**Finding 1 — near-linear scaling (재현됨).**

> Three-node throughput reached **3.00×** the one-node c8 baseline and **~98%**
> of the single-node saturation-derived ideal. All 30 runs completed without
> inference errors or retries, with effectively uniform round-robin distribution.

**Finding 2 — node-level overhead is payload transfer.**

> Local direct inference reached **161.5 inf/s** while single-node cluster
> throughput reached **112.9 inf/s**, a **30.1% throughput reduction**.
> Separately, latency decomposition showed that **94% of non-inference latency
> was observed in the payload-transfer path** — not in serialization, scheduler
> queueing, or node queueing (all ~0).

이 둘은 서로를 강화한다. 확장은 스케줄러·네트워크가 노드 수에 병목이 아니라서
선형이고(Finding 1), 노드당 절대 상한은 페이로드를 2.5G 로 나르는 시간에
깎인다(Finding 2). 최적화가 겨냥할 지점은 compute 가 아니라 **transport** 다.

## 11. Limitations

- **측정 시간이 짧다(60/30초).** CPU throttling 은 300초에 -27% 로 나타나므로
  (board-worklog §2.24), 이 결과는 **throttling 전 구간**이다. 지속 부하
  처리량은 별도 실험(S0)에서 확정한다.
- **냉각 축.** 오늘은 Active Cooling 만. 팬리스(조건 A) 클러스터 측정은 없다.
- **saturation 미확정.** 1N 은 c8/c16/c32 로 ~115 근처를 봤으나 c48 미측정,
  2N·3N 의 ceiling 은 sweep 하지 않았다 → **S3**.
- **직렬화 단독 미측정.** proto `Timing` 에 gRPC 직렬화 단독 필드가 없다.
  현재 non-inference 잔차(~2ms)에 포함. 계측점 추가가 필요.
- **closed-loop.** 절대 지연 아님, 구성 간 비교 전용.
- **단일 2노드 조합(king+queen).** king+jack 등 다른 조합은 미측정.

## 12. Reproduction

```bash
# 3노드 클러스터 기동 후 (스케줄러 + king/queen/jack)
bash scripts/run-grpc-baseline30.sh        # 30 run → server:/tmp/baseline30
# 로컬 팬 baseline (Finding 2):
ssh npuforge-k 'pkill -9 npuforge-node; sleep 3; cd ~/npuforge-rknn-test; \
  ./sustained_load_test yolov8n-int8.rknn 60 8'
# 그래프 재생성:
python scripts/make-figures.py
```

동결 commit: `254d560`. 조건 고정표는 §4.

## 13. Raw Data

- bench JSON 30건: [`../../results/baseline-20260820/raw/`](../../results/baseline-20260820/raw/)
  (`n{노드}_r{라운드}.json`, 각 파일에 throughput·latency·node_inference·
  TimingBreakdown·per_node·nodes_before/after(temp·voltage)·verdict·run_id)
- 집계 리포트: [`../../results/baseline-20260820/README.md`](../../results/baseline-20260820/README.md)
- 그래프·대시보드: [`figures/`](../../results/baseline-20260820/figures/), [`dashboard.html`](../../results/baseline-20260820/dashboard.html)

## 14. Conclusion

RK3576 3-node NPU cluster 는 30회 반복 실험에서 **near-linear scaling
(338.4 ± 1.1 inf/s, 3.00×, error 0%)** 을 보였다. 노드당 오버헤드는 compute
나 scheduler 가 아니라 **payload-transfer path**(non-inference latency 의 94%)
에 있음을 TimingBreakdown 으로 확인했다.

→ gRPC baseline 을 **동결**한다. 다음: **S3**(saturation / scaling limit) →
**S4**(io_uring). S4 는 이 baseline 과 **동일 조건**에서 transport 비용을
비교한다.
