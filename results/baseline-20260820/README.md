# gRPC Baseline 30회 반복 — 재현 확인 (2026-08-20)

- 측정일: 2026-08-20
- 대상: RK3576 3노드 + 스케줄러(server .9), gRPC
- **동결 commit:** `254d560` (bench timing 확장). 측정 중 코드·설정 무변경
- 원본: [`raw/`](raw/) — bench JSON 30건 (`n{노드}_r{라운드}.json`)
- 요약: `docs/RESULTS.md` §2.5, 경위: `docs/board-worklog.md` §2.28

> **"한 번 337.7 이 나왔다" → "3-node near-linear scaling 을 30회 반복
> 실험으로 확인했다."** 로 승격된 단계.
>
> 이 문서는 **원본 데이터·집계·그래프**다. 실험 보고서(연구 질문·해석·결론)는
> [`docs/experiments/S2_GRPC_BASELINE.md`](../../docs/experiments/S2_GRPC_BASELINE.md).

---

## 1. 측정 설계

- 1N/2N/3N 각 **10회, 60초**, concurrency = 8 × 노드수 (c8/c16/c24)
- **조건 순서 rotate** (R1 1-2-3, R2 2-3-1, R3 3-1-2, …) — 시간·온도 변동을
  한 조건에 몰지 않는다
- 노드 축소는 프로세스 중지, 각 run 사이 cooldown
- 조건 고정: INT8, want_float=0, governor=performance, **Active Cooling**,
  round-robin, worker 8

## 2. 무결성 (측정 신뢰성)

| 검사 | 결과 |
|---|---|
| run 수 | 30 / 30 |
| active node 판정 | **30/30 정확** (n1=1, n2=2, n3=3, 버그 수정 확인) |
| invalid run | 0 |
| 오류율 | **0.00%** (전 run) |
| 재시도 | 0건 |
| load balance 편차 | **0.00 %p** (round-robin 완전 균등) |

## 3. 목표 표 — 반복 측정 결과

| Nodes | Throughput Mean ± SD | Speedup | Efficiency | p50 ms | p99 ms | Error | Balance |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | **112.9 ± 0.5** inf/s | 1.00× | 100% | 68.0 | 116.3 | 0% | 0.00 %p |
| 2 | **229.0 ± 0.9** inf/s | 2.03× | 101% | 67.0 | 118.6 | 0% | 0.00 %p |
| 3 | **338.4 ± 1.1** inf/s | 3.00× | 100% | 67.6 | 123.9 | 0% | 0.00 %p |

- **Speedup 은 1노드 c8 기준.** near-linear (3.00×).
- single-node saturation(~115) 기준으로는 3N = 338.4/115 = **2.94× (98%)**.
- **SD 가 0.5~1.1 로 극히 작다** — 30회에 걸쳐 처리량이 사실상 흔들리지 않았다.
  337.7(첫 측정)이 338.4 ± 1.1 로 재현됐다.

## 4. TimingBreakdown (30회 p50 평균)

| 단계 | 1N | 3N |
|---|---:|---:|
| scheduler_queue | 0.00 | 0.00 |
| scheduler_route | 0.00 | 0.00 |
| **network_to_node** | 17.72 | 17.11 |
| node_queue | 0.02 | 0.02 |
| **inference (NPU)** | 24.70 | 22.49 |
| **network_to_client** | 17.72 | 17.11 |
| **end_to_end** | 61.54 | 58.83 |

**첫 실측이 재현됐다 (3N):**

```text
non-inference overhead = end_to_end - inference = 58.83 - 22.49 = 36.34 ms
payload transfer       = network_to_node + network_to_client = 34.21 ms
  = non-inference overhead 의 94%
  = E2E 의 58%
```

- scheduler_queue/route 는 노드 수와 무관하게 **~0** — 단일 스케줄러가 3노드
  동시에도 병목이 아니다(`adrs/003` 재확인).
- 1N 과 3N 의 `network_to_node`(17.72 vs 17.11)가 거의 같다 — 단일 요청
  전송 시간은 노드 수와 무관하다.

> ⛔ **두 측정량을 곱하지 않는다.** throughput loss(28.8%, §5)와 latency
> breakdown(94%)은 다른 축이다. 정확한 문장은 §5.

## 5. 노드당 오버헤드 (냉각·worker 통일)

| Mode | Cooling | Worker | Throughput |
|---|---|---:|---:|
| Local direct RKNN (gRPC 없음) | Active Cooling | 8 | 161.5 inf/s |
| Cluster gRPC (single node) | Active Cooling | 8 | 112.9 inf/s |

**Throughput loss = (161.5 - 112.9) / 161.5 = 30.1%** (30회 1N mean 기준).
로컬 팬 baseline 은 `board-worklog.md` §2.27.

> 발표 문장(곱 금지): **클러스터 경유 시 단일 노드 처리량은 로컬 대비 약
> 30% 낮았으며(throughput), 별도 latency breakdown 에서 non-inference
> latency 의 94% 가 payload-transfer path 에서 관측됐다.**

## 6. 결론

1. **3-node near-linear scaling 을 30회 반복으로 확인** — 338.4 ± 1.1 inf/s,
   speedup 3.00×(1N 기준)/2.94×(saturation 기준), error 0%, balance 0%p.
2. **노드당 오버헤드의 정체 = 페이로드 전송** (overhead 의 94%). 직렬화·
   스케줄러 큐·노드 큐 아님. io_uring·zero-copy·JPEG·후처리가 겨냥할 지점.

**여기서 gRPC baseline 은 동결한다.** 다음: saturation sweep → (동결 유지)
→ io_uring 을 **동일 조건**에서 비교.

## 7. 재현

```bash
bash scripts/run-grpc-baseline30.sh      # 30 run, server:/tmp/baseline30 저장
# 로컬 팬 baseline: king 노드 중지 후
ssh npuforge-k 'cd ~/npuforge-rknn-test; ./sustained_load_test yolov8n-int8.rknn 60 8'
```
