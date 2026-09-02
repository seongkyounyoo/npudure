# S2 다중 노드 확장성 — 첫 측정 (2026-08-20)

*[English](README.md) — 영문이 정본이다.*

- 측정일: 2026-08-20
- 대상: RK3576 3노드 (king/queen/jack) + 스케줄러(server .9)
- 상태: **S2 첫 측정.** 정식 아님 — 단일 run · `--with-inference` 미실시
- **냉각: Active Cooling (노드마다 전용 팬, 측정 시작부터 장착).** 오늘 데이터 전체가 팬 ON 상태다
- 원본: [`raw/`](raw/) (bench JSON 3건)
- 요약 반영: `docs/RESULTS.md` §2.5, 경위: `docs/board-worklog.md` §2.25

> 이 프로젝트의 핵심 질문 **"6 TOPS NPU 세 대는 정말 18 TOPS가 되는가"** 에
> 대한 첫 실측 답이다.

---

## 1. 한 줄 결론

**확장 효율 ~98% (거의 선형).** 3노드가 단일 노드(클러스터 기준)의 **2.93배**.
병목은 확장이 아니라 **노드당 gRPC 오버헤드**다 — 클러스터 노드 상한 115가
로컬 상한 157보다 27% 낮다.

---

## 2. 측정 조건

**이 측정의 baseline 조건 (고정):**

```text
Cooling      : Active cooling — 120mm 5V USB fan per node (보드보다 큼)
CPU governor : performance
Network      : 2.5GbE / node, 10GbE server (aggregation)
Transport    : gRPC (tonic + protobuf)
Model        : YOLOv8n INT8, want_float=0
```

| 항목 | 값 |
|---|---|
| 모델 | YOLOv8n **INT8** (sha256 `dba155d2…`) |
| 출력 | `want_float=0` (blob v2, 역양자화 파라미터 동봉) |
| CPU governor | `performance` |
| 냉각 | **Active Cooling — 노드마다 전용 팬** (측정 시작부터) |
| 통신 | **gRPC** (tonic + protobuf), 스케줄러 경유 |
| 정책 | round-robin |
| worker_count | 8 (노드마다 전용 RKNN 컨텍스트) |
| 입력 | raw RGB 640×640×3 = 1,228,800 byte/요청 |
| 부하 도구 | `npuforge-bench` (closed-loop), server(.9)에서 실행 |
| 지속 | 30초 (1노드 concurrency 스윕은 20초) |
| run 수 | 조건당 1 (반복 없음) |
| preflight | 통과 (별칭·해시·governor·온도·전압·NTP) |

측정 경로 (세 홉 전부 gRPC):

```text
npuforge-bench ──gRPC──► 스케줄러(.9:50051) ──gRPC──► 노드(.X:51001) ──► RKNN
   SchedulerService.Infer         NodeService                    (여기만 NPU)
   (bench 와 스케줄러는 같은        (2.5G 실제 네트워크 구간)
    호스트라 loopback)
```

노드 축소는 프로세스 중지(jack → 이어서 queen)로 했고, 사이에 cooldown 을 뒀다.

---

## 3. 결과

### 3.1 노드당 동일 부하 (concurrency = 8 × 노드수)

| 구성 | concurrency | 처리량 | 분배 | 오류율 | 원본 |
|---|---:|---:|---|---:|---|
| 1노드 (king) | 8 | **111.6 inf/s** | king 100% | 0% | `raw/yolov8n-c8-n3.json` |
| 2노드 (king·queen) | 16 | **228.7 inf/s** | 50 / 50 | 0% | `raw/yolov8n-c16-n3.json` |
| 3노드 | 24 | **337.7 inf/s** | 33 / 33 / 33 | 0% | `raw/yolov8n-c24-n3.json` |

round-robin 이 정확히 균등 분배했고, 실패는 0건이었다.

### 3.2 단일 노드 concurrency 스윕 (포화점)

king 단독, 20초. concurrency 를 올려 노드 상한을 찾는다.

| concurrency | 8 | 16 | 32 |
|---|---:|---:|---:|
| 처리량 | 111.6 | 114.0 | **115.1 (포화)** |

concurrency 를 4배로 올려도 **~115 inf/s 에서 포화**. 이것이 스케줄러 경유
단일 노드 상한이다.

### 3.3 지연 (3노드 c24 기준)

```text
왕복(클라이언트 관점)   min 35.2  p50 67.8  p90 92.7  p99 122.5  ms
노드 보고 추론시간       p50 22.3  p99 44.6  ms
```

---

## 4. 분석

### 4.1 확장은 선형이다 (효율 ~98%)

```text
1노드 포화 115 기준
  2노드 228.7 = 1.99×   (효율 99%)
  3노드 337.7 = 2.93×   (효율 98%)
```

데이터 병렬(`adrs/001`)이 성립하고, **단일 스케줄러(`adrs/003`)가 3노드
동시에도 병목이 아니다.** 노드가 서로 독립이라 스케줄러·네트워크가 노드 수에
비례해 나빠지지 않는다.

### 4.2 노드당 상한이 27% 깎인다

| 측정 방식 | 노드 상한 | 냉각 | 차이 |
|---|---:|---|---|
| 로컬 `sustained_load_test` (gRPC 없음) | **161.5** inf/s | Active Cooling, worker 8 | 기준 |
| 클러스터 (스케줄러 gRPC 경유) | ~115 inf/s | Active Cooling, worker 8 | **-28.8%** |
| (참고) 로컬 팬리스 | 157.2 inf/s | 팬리스 08-11/12 | |

왕복 p50 69 ms 인데 노드 보고 추론은 24~28 ms. 나머지 40 ms+ 는 gRPC 경유
오버헤드로 보인다 — protobuf 직렬화 + 1.17 MiB 입력·출력 전송 + 스케줄러
큐·라우팅. loopback 인 bench↔스케줄러 구간은 거의 기여하지 않으므로 대부분
**스케줄러↔노드 2.5G gRPC 구간**이다.

> ✅ **냉각 통일 완료 (2026-08-20).** 로컬 팬 baseline 재측정 = 161.5 inf/s
> (8스레드, worker 8, `board-worklog.md` §2.27). 냉각·worker 를 클러스터와
> 맞춘 오버헤드는 **(161.5-115)/161.5 = 28.8%**. 팬리스 157.2 와 차이가 작은
> 이유는 30/60초가 throttling 전이라 냉각 영향이 작기 때문이다. **27% →
> 28.8% 로 확정**(짧은 측정 기준). 지속 부하(300초)에서는 팬 이득이 커져
> 오버헤드가 더 벌어질 수 있다. 오버헤드의 **94% 가 페이로드 전송**임은
> `TimingBreakdown` 으로 분해했다(§2.26).

---

## 5. 원본 파일 안내

`raw/` 의 bench JSON 3건. **파일명의 `-n3` 은 부정확하다** — bench 가 run_id
의 노드 수를 측정 시점 활성 노드가 아니라 **초기 ListNodes(스케줄러 등록)**
기준으로 붙인다. 노드를 중지해도 스케줄러 등록이 남아 3으로 찍혔다.

**실제 측정 노드는 각 JSON 의 `summary.per_node`(활성 분배)와
`nodes_after`(온도 상승분)로 확정된다.**

| 파일 | run_id | 실제 노드 (`per_node`) | 근거 |
|---|---|---|---|
| `yolov8n-c8-n3.json` | c8-n3 | **1노드** king | nodes_after 에서 king 만 36→49.9°C |
| `yolov8n-c16-n3.json` | c16-n3 | **2노드** king·queen | per_node 2개 |
| `yolov8n-c24-n3.json` | c24-n3 | **3노드** | per_node 3개 |

→ **2026-08-20 수정 완료.** bench 의 run_id·`node_count` 를 `per_node.len()`
(실제 활성 노드)로 바꿨다. 이 `raw/` 파일들은 **수정 전** 측정이라 `-n3` 으로
남아 있다 — 데이터는 유효하고 실제 노드는 위 표대로다.

각 JSON 의 `verdict.valid = true`, `caveats` 에 closed-loop 주의가 들어 있다
(`adrs/028`: 절대 지연을 SLA 로 인용하지 않는다, 구성 간 비교 전용).

---

## 6. 재현

```bash
# 3노드 클러스터 기동 (스케줄러 + king/queen/jack) 후
ssh npuforge-server '/root/npuforge/target/release/npuforge-bench \
  --scheduler http://127.0.0.1:50051 --model yolov8n \
  --concurrency 24 --duration 30 --policy round-robin --out /tmp/s2'

# 노드 축소는 프로세스 중지 (comm 기준, -f 금지 — adrs/017 함정 1)
ssh npuforge-j 'pkill -9 npuforge-node; sleep 3'   # → 2노드
ssh npuforge-q 'pkill -9 npuforge-node; sleep 3'   # → 1노드
```

---

## 7. 정식 S2 에 남은 것

- 반복 run (분산 확인)
- **팬리스(S0-A) 조건과 비교** — 오늘 baseline 은 Active Cooling(팬 ON)이다
- `preflight --with-inference` (세 보드 정확도 일치)
- concurrency 스윕 전체 (각 노드수의 상한 곡선)
- 2노드 조합 비교 (king+queen vs king+jack)
- **로컬 baseline 을 같은 팬 조건에서 재측정** — 27% 를 정확히 확정하려면 필수
- **`TimingBreakdown` 오버헤드 분해** — 27% 가 전송인지 큐잉인지 직렬화인지
- 노드 축소를 drain RPC 로 (진행 중 요청을 흘려보내고 깨끗이 제외, `adrs/027`)
