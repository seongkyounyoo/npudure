# 기술 용어 정리 (Glossary)

*[English](GLOSSARY.md) — 영문이 정본이다.*

- 최종 갱신: **2026-08-21**
- 범위: S2~S4 실험 계보에서 실제로 등장한 용어 전부. 정의만 적지 않고
  **이 프로젝트에서 어떤 값·판단과 묶여 있는지**를 함께 적는다.
- 관련: [`experiments/README.md`](experiments/README.md)(실험 대장),
  [`01-TECHSPEC.md`](01-TECHSPEC.md), [`RESULTS.md`](RESULTS.md)

---

## 1. 실험 ID 체계

| ID | 질문 | 결과 요약 |
|---|---|---|
| **S0-A** | 팬리스 지속 부하에서 운영점이 유지되는가 | 열화 11.3%, CPU 2208→816 MHz |
| **S0-B** | 능동 냉각 지속 부하 | 열화 1.9%, 클럭 강등 0회 |
| **S0-C** | 부하 인지 정책이 열 불균질 손실을 회수하는가 | 1차 herding 버그 발견 → 2·3차 회수 확인 → 4차 게이트 미달 |
| **S0-D** | 이질을 결정론적으로 만들 수 있는가 | 가능. 클럭 캡으로 편차 1.12~3.93× |
| **S2** | 노드를 늘리면 선형으로 늘어나는가 | 112.9 / 229.0 / 338.4, 3.00× |
| **S3** | 각 구성의 진짜 상한(ceiling)은 | 115.2 / 232.0 / 341.8 |
| **S3.5** | −30% 손실이 어디서 오는가 | 전송 경로로 좁힘 |
| **S3.5b** | CPU0 softirq 편중이 원인인가 | null (−0.2%) |
| **S3.6** | flow control 인가 커넥션인가 | 커넥션. window 확대는 역효과 |
| **S3.7a** | 커넥션 몇 개가 최적인가 (고정 부하) | c4 에서 knee |
| **S3.7b** | 각 구성의 **운영점**은 | 셋 다 c12. conn2 가 우세 |
| **S3.7c** | 운영점에서 RPS 는 효과가 있는가 | null (−0.8%) |
| **S3.8** | 최적화가 scale-out 을 해치는가 | 387.2 inf/s, eff 95.3% |
| **S3.9a** | 3N efficiency 손실의 출처 | 서버 자원 배제, tail 증가 |
| **S3.9b** | 노드 쪽 남은 비용 | syscall 은 ~1%. 유저 시간이 커널보다 크다 |
| **S4** | io_uring 이 필요한가 | **아니다 — 측정으로 반박됨**(S3.9b) |

> **명명 규칙** — 정수(S2, S3)는 원래 계획된 실험. 소수점(S3.5, S3.7a)은
> 측정 결과가 새로 요구한 실험이다. 계획이 아니라 **데이터가 다음 실험을
> 정했다**는 기록이기도 하다.

---

## 2. 측정 방법론

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **closed-loop** | 동시 요청 수(concurrency)를 고정하고, 응답이 와야 다음 요청을 보내는 부하 모델 | bench 가 이 방식. 절대 지연을 SLA 로 인용하면 안 되고 **구성 간 비교에만** 쓴다 |
| **open-loop** | 응답과 무관하게 정해진 도착률로 계속 보내는 모델 | 미사용 |
| **coordinated omission** | closed-loop 에서 시스템이 느려지면 요청 자체가 덜 발생해 **지연이 과소 측정**되는 현상 | bench `--help` 에 경고로 명시 |
| **Little's law** | `동시요청수 = 처리량 × 평균지연` | S3.9a 에서 efficiency 손실이 **평균 지연 증가와 정확히 일치**함을 보이는 데 사용 |
| **saturation / ceiling** | 부하를 더 줘도 처리량이 안 오르는 상한 | S3 가 구성별로 측정 |
| **operating point (운영점)** | 실제로 운전할 부하 지점 | **peak 의 98% 이상을 내는 가장 낮은 concurrency** 로 정의(코드 상수) |
| **concurrency knee** | 장치를 포화시키는 데 필요한 동시 요청 수 | 시험 범위에서 **c12/node**, 커넥션 수와 무관하게 관측 |
| **connection knee** | 그 요청을 몇 갈래 커넥션으로 나눌지의 최적점 | 고정 부하에서 c4, 운영점 기준으로 **conn2** |
| **overload region** | 포화 이후 구간. 처리량은 그대로고 지연만 증가 | c24~c64 전 구간. **여기서 구성을 비교하면 결론이 뒤집힌다** |
| **short-run / sustained operating point** | 60초 기준 / thermal steady-state 기준 운영점 | 능동 냉각에서는 같고(−1.9%), 팬리스에서는 갈라진다(−11.3%) |
| **steady-state** | 시간에 따라 값이 더 안 변하는 구간 | S0 정의: **마지막 1/3 구간 평균** |
| **degradation** | `1 − steady / peak` | 판정: <3% 없음 / 3~10% 경미 / >10% 뚜렷 |
| **scaling / efficiency** | `tp_N / tp_1` / `그것을 N 으로 나눈 값` | optimized 3N: 2.86× / 95.3% |
| **rotation (조건 회전)** | 반복마다 조건 순서를 바꿔 시간·온도 드리프트를 상쇄 | 모든 A/B 하네스에 적용 |
| **preheat / reheat** | 측정 전 부하로 열 상태를 맞추는 것 | S0-C 재실행에서 **정책마다** 재예열 |
| **freeze (동결)** | 측정 중 코드·설정·모델을 바꾸지 않는 것 | 바이너리를 `*.frozen-<commit>` 으로 보존 |
| **verdict** | bench 가 run 유효성을 스스로 판정한 결과 | `valid` + `reasons`. 이상 run 도 지우지 않는다 |
| **preflight** | 측정 직전 하드 실패 검사 | 별칭↔hostname, 해시, governor, 온도, 전압, **추론 정확도** |
| **probe bench** | 본측정 전 짧게 던져 조건을 확인하는 부하 | 노드 수 검증에 사용 — S3.8 에서 6개 구성을 걸러냈다 |
| **capacity heterogeneity** | 노드간 처리 능력 편차 | 열 유래(S0-A/C)와 **클럭 캡 유래**(S0-D)를 구분해 적는다. 스케줄러가 보는 것은 capacity 이지 그 원인이 아니다 |
| **이질 게이지** | 편차를 재는 관측량 | **round-robin 의 노드별 p50 최대/최소.** RR 은 적응하지 않으므로 균등 부하 아래 raw capacity 편차가 그대로 드러난다 |
| **utime / stime** | 유저 / 커널 CPU 시간 | `/proc/PID/stat`. 커널에 syscall 진입·TCP 스택·`copy_to_user`, 유저에 직렬화·유저공간 copy·HTTP/2 프레이밍. **io_uring 이 줄이는 것은 stime 의 일부** |
| **한쪽 방향 검정** | 편향 방향이 정해진 측정 | `strace -c` 는 ptrace 로 값을 **부풀린다** → 부풀린 값이 작으면 실제는 확정적으로 더 작다 |

### 2.1 percentile 집계

| 용어 | 뜻 |
|---|---|
| **nearest-rank** | 보간 없이 "정렬 후 그 지점 이상 첫 값". bench 가 쓰는 방식 |
| **run-level percentile** | 한 run 안에서 그 run 의 요청들로 계산한 percentile |
| **pooled percentile** | 여러 run 의 요청을 **전부 합쳐** 다시 계산한 percentile |
| **주의** | 이 저장소의 표는 전부 **run-level 의 평균**이다. pooled 가 아니다. run-level 평균은 각 run 의 최악 구간이 희석돼 **tail 을 낮게 보이게 한다**. 조건 간 비교에는 유효하나 절대값을 "이 시스템의 p99" 로 인용하면 안 된다 |

---

## 3. 성능 지표

| 용어 | 뜻 |
|---|---|
| **inf/s** | 초당 추론 건수(throughput) |
| **p50 / p95 / p99 / max** | 지연 분포의 백분위. p50=중앙값 |
| **tail latency** | 상위 백분위(p95/p99) 지연. "일부 요청이 얼마나 늦는가" |
| **tail amplification** | 처리량이 올라도 tail 이 더 크게 나빠지는 현상 |
| **balance (%p)** | 노드 간 요청 분배 편차. 0 이면 완전 균등 |
| **error_rate** | 실패 비율. 이 저장소 전 실험에서 **0** |
| **TimingBreakdown** | 응답에 실려 오는 11단계 시간 분해 (proto `Timing`) |
| `scheduler_queue` | 스케줄러 내부 대기 |
| `scheduler_route` | 정책 선택 시간 |
| `network_to_node` / `network_to_client` | **왕복 전체 − 노드 내부 시간**을 절반씩 나눈 값. 서로 다른 장치의 절대 시각을 빼지 않기 위한 방법 |
| `node_queue` | 노드 워커 풀 대기 |
| `decode / preprocess / npu_input / inference / postprocess` | 노드 내부 단계. 이 프로젝트는 raw RGB 입력·raw tensor 출력이라 inference 외에는 ~0 |
| `end_to_end` | 스케줄러가 잰 전체 |
| **syscalls/req · ctx switches/req · cycles/req** | TECHSPEC §15.4 가 요구하는 io_uring 판단 지표 |

---

## 4. 네트워크 · 커널

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **full-duplex** | 송신·수신이 각자의 대역폭을 갖는 것 | **두 방향을 한 링크 예산에 합산하면 안 된다.** S3.8 에서 이 실수로 "10G 76%" 를 썼다가 S3.9a 에서 철회(실제 방향당 40%) |
| **goodput** | 헤더를 뺀 실제 payload 처리량 | iperf3 가 재는 값. 보드 링크 실측 2.34 Gbps |
| **MTU** | 한 프레임에 담기는 최대 payload. 여기선 1500 | |
| **RSS** (Receive Side Scaling) | NIC 이 **하드웨어 다중 큐**로 패킷을 여러 코어에 분산 | 서버 NIC 은 RX 큐 24개. 보드는 **1개** |
| **RPS** (Receive Packet Steering) | 커널이 **소프트웨어로** flow 해시 기준 분산 | 보드에서 시도했으나 두 번 다 null. **흐름이 1개면 나눌 대상이 없다** |
| **softirq / NET_RX** | 커널이 인터럽트 후처리를 하는 경량 컨텍스트. 네트워크 수신 처리가 여기서 돈다 | 보드 CPU0 %soft 51.5% → 단일 큐 때문 |
| **IRQ affinity** | 인터럽트를 어느 코어가 받을지 | 보드는 NIC IRQ 가 CPU0 고정 |
| **cwnd / ssthresh** | TCP 혼잡 윈도 / 느린 시작 임계값 | 3N 에서 cwnd 176 → 106~119 로 눌림 |
| **retransmission (재전송)** | 손실·지연으로 다시 보낸 세그먼트 | 커넥션당 재전송률 0.055% → **0.19%** (3.5배) |
| **incast / speed mismatch** | 빠른 링크(10G)에서 느린 링크(2.5G)로 몰릴 때 스위치 egress 에 생기는 버퍼링·손실 | 3N efficiency 손실의 **유력 가설(미검증)** |
| **bufferbloat** | 과도한 버퍼가 지연을 부풀리는 현상 | 64 MB window 확대 시 −36.3% 의 해석 가설 |
| **/proc/stat · /proc/net/dev · /proc/interrupts · /proc/softirqs** | 커널이 노출하는 카운터 | 서버에 sysstat 이 없어 이것들의 델타로 직접 계산 |
| **ss -tin** | 소켓별 TCP 상태(rtt, cwnd, retrans, bytes_sent) | 커넥션 수·혼잡 상태 관측 |

---

## 5. HTTP/2 · gRPC

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **HTTP/2 multiplexing** | 하나의 TCP 커넥션에서 여러 스트림을 동시에 실어 나르는 것 | 그래서 "커넥션이 1개" 만으로 병목이라 단정할 수 없다 |
| **stream** | 커넥션 안의 논리적 요청/응답 한 쌍 | 요청 1건 = 스트림 1개 |
| **flow control window** | 수신자가 "여기까지 받을 수 있다" 고 광고하는 크기. **스트림 단위**와 **커넥션 단위**가 따로 있다 | h2 기본 65,535 byte. 이 프로젝트는 메시지가 1.2 MB |
| **WINDOW_UPDATE** | window 를 다시 열어주는 프레임 | window 가 작으면 이 왕복이 반복돼 stop-and-wait 가 된다 |
| **DATA frame** | 실제 payload 를 나르는 프레임 | 응답 1.218 MB 를 약 14.4 KB 씩 쪼개 보냄(write syscall 84.4회/req) |
| **head-of-line blocking** | 앞선 것이 막히면 뒤가 다 막히는 현상 | 다중화된 스트림이 같은 커넥션 자원을 다툴 때 |
| **tonic** | Rust gRPC 구현 (hyper + h2 기반) | v0.12.3 |
| **h2 / hyper** | HTTP/2 프로토콜 / HTTP 라이브러리 | h2 0.4.15, hyper 1.11.0 |
| **prost** | Protocol Buffers 코드 생성기 | `.proto` → Rust 타입 |
| **protoc** | protobuf 컴파일러 | 빌드 전제. Windows 개발 PC 에는 없어 서버·보드에서 빌드 |
| **`node_connections`** | **노드당** gRPC 커넥션 수 (이 프로젝트가 추가한 설정) | 1N→2, 2N→4, 3N→6 total. **클러스터 전체 합이 아니다** |

---

## 6. 스케줄링

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **round-robin (RR)** | 상태를 보지 않고 순서대로 배정 | 기본값. 구조적으로 균등하지만 **느려진 노드에도 똑같이 보낸다** |
| **least-queue / LOR** (least-outstanding-requests) | 미완료 요청이 가장 적은 노드 선택 | 서비스 **속도**는 모른다. 동시 버스트에서는 균등 분배가 정상 동작 |
| **ECT** (Estimated Completion Time) | `(미완료+1) × EWMA_inference + EWMA_network + 패널티` 로 완료 예상시각을 추정 | 서비스 속도 차이를 반영할 수 있는 유일한 정책. 단 **EWMA 가 채워져야** 동작 |
| **EWMA** | 지수 이동 평균 | 추론시간·네트워크 왕복시간 추적 |
| **herding (herd behavior)** | 여러 결정 주체가 **같은 낡은 정보**를 보고 동시에 같은 선택을 하는 현상 | S0-C 의 원인. 처리량 55~58% 붕괴 |
| **stale state / state freshness** | 상태 정보가 낡은 정도 | 하트비트 1초 vs 디스패치 ~3ms → **수백 배 차이** |
| **control-loop sampling problem** | 피드백 주기가 시스템 변화 주기보다 길어 제어가 실패하는 문제 | herding 의 상위 개념. 정책 튜닝 문제가 아니다 |
| **reservation (예약)** | 선택과 동시에 부하를 점유 표시하는 것 | `select_and_reserve()` 가 한 임계구역에서 처리 |
| **RAII guard** | 값이 스코프를 벗어날 때 자동 정리되는 패턴 | `Reservation` 의 `Drop` 이 감소 — 성공·오류·타임아웃·취소·재시도 **모든 경로**를 닫는다 |
| **`local_in_flight`** | 스케줄러가 보냈지만 아직 안 끝난 요청 수 (즉시 갱신) | 정책의 **1차 신호**. 하트비트 값과 더하지 않는다(같은 요청을 두 번 세게 됨) |
| **`health.in_flight` / `queue_depth`** | 노드가 하트비트에 실어 보낸 관측값 (최대 1초 stale) | health 판정·tiebreaker 용도로만 |
| **busy_queue_depth / degraded / disable temperature** | 노드 상태 분류 임계치 | 8 / 80°C / 90°C |
| **drain** | 새 요청을 안 보내고 큐를 비우는 상태 | 운영자 지정 상태 |

---

## 7. 하드웨어 · 열

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **RK3576** | Rockchip SoC. 4×Cortex-A72 + 4×Cortex-A53, NPU 2코어 | 노드 보드 3대 |
| **big.LITTLE** | 고성능/저전력 코어를 섞은 구성 | A72 2208 MHz(policy4), A53 2016 MHz(policy0) |
| **cpufreq governor** | CPU 주파수 정책 | `performance` 고정(최대 클럭 유지). 대안 `ondemand` |
| **devfreq** | CPU 외 장치(NPU·GPU·DDR)의 주파수 관리 | NPU 300~950 MHz |
| **thermal zone** | 커널이 노출하는 온도 센서 | soc / bigcore / little-core / ddr / **npu** / gpu 6개 |
| **thermal throttling** | 온도로 클럭을 낮추는 것 | **NPU 는 한 번도 안 떨어졌다.** 떨어지는 것은 CPU |
| **thermal steady-state** | 발열과 방열이 균형을 이룬 온도 평탄역 | 능동 냉각 58~61°C(5분), 팬리스 86~88°C |
| **thermal heterogeneity (열 불균질)** | 같은 모델 보드인데 열 조건이 달라 성능이 갈리는 것 | 팬리스에서 king 816 / jack 1200 / queen 1416 MHz |
| **boot_id** | 부팅마다 바뀌는 식별자 | run 도중 보드 리셋 감지 → 그 측정은 무효 |
| **입력 전압 감시** | 어댑터 용량 부족 조기 경보 | 5.00 V 미만이면 preflight 실패 |
| **2.5GbE / 10GbE** | 링크 속도 | 보드 2.5G, 서버 10G. **속도 불일치가 §4 의 incast 가설** |

---

## 8. 모델 · NPU 런타임

| 용어 | 뜻 | 이 프로젝트에서 |
|---|---|---|
| **RKNN** | Rockchip NPU 런타임 | `librknnrt.so` 2.3.0 |
| **RKNPU driver** | 커널 드라이버 | v0.9.8 |
| **YOLOv8n** | 객체 검출 모델 | 입력 640×640×3 |
| **INT8 양자화** | 가중치·활성을 8비트 정수로 | FP16 대비 처리량 +17.3% |
| **`want_float`** | 출력을 float 로 역양자화해 받을지 | **0**(정수 그대로). 출력 크기 4분의 1, 처리량 +17.3% |
| **blob v2** | 텐서 여러 개를 담는 자체 직렬화 형식 | 텐서당 36 byte 헤더에 `scale`·`zero_point` 동봉 |
| **payload 크기** | | 요청 **1,228,800 B**, 응답 **1,218,000 B** (합 2,446,800 B/추론) |
| **postprocess (DFL + NMS)** | 검출 결과 디코딩과 중복 제거 | **현재 노드에서 안 한다.** raw tensor 를 그대로 보냄 → 응답이 1.2 MB. 노드에서 하면 수 KB 로 줄어든다(미구현 아이디어) |
| **warmup** | 첫 추론의 초기화 비용을 제외하기 위한 예열 | 집계에서 제외 |
| **worker_count** | 노드의 동시 추론 워커 수 | 8. **워커가 독립적이지 않다** — 로컬 direct 8워커가 161.5 inf/s |

---

## 9. 소프트웨어 스택

| 용어 | 뜻 |
|---|---|
| **tokio** | Rust 비동기 런타임. 노드는 multi_thread(워커 = 코어 수 8) |
| **`spawn_blocking`** | 블로킹 작업을 별도 스레드 풀로 보내는 tokio API. RKNN FFI 호출이 여기서 돈다 |
| **async worker vs blocking pool** | 네트워크·protobuf 는 async 워커 8개, 추론은 blocking 풀 — **같은 8코어를 나눠 쓴다** |
| **`parking_lot`** | 더 빠른 Mutex/RwLock 구현 |
| **`Arc<AtomicU32>`** | 스레드 간 공유되는 원자적 카운터 |
| **`Bytes`** | 참조 카운팅되는 바이트 버퍼(복사 없이 공유) |
| **`to_vec()`** | 복사를 만드는 호출. 남은 gap 후보 중 하나 |
| **feature flag** | 컴파일 시 기능 선택. 노드는 `--features rknn` 필요 (없으면 Mock 백엔드가 빌드됨) |
| **`RKNN_SDK_PATH`** | 빌드 시 `rknn_api.h` 위치 |

---

## 10. 진단 도구

| 도구 | 용도 | 비고 |
|---|---|---|
| **iperf3** | 링크 대역폭 실측 | 보드→서버 2.34 Gbps |
| **mpstat** | 코어별 CPU 분해(%usr/%sys/%soft/%idle) | 보드에만 있음 |
| **pidstat** | 프로세스·스레드별 CPU | 보드에만 있음 |
| **ethtool** | 링크 속도·NIC 통계·offload 설정 | 양쪽 다 있음 |
| **ss** | 소켓 상태 | 커넥션 수·TCP 내부 상태 |
| **perf** | PMU 기반 프로파일링 | **양쪽 다 없음.** cycles/req 는 근사값 |
| **`/proc` 델타** | sysstat 없이 CPU·네트워크·syscall 집계 | 서버 프로파일에 사용 |
| **thermal-logger.sh** | 보드 온도·주파수·전압 1초 샘플러 | |

---

## 11. 이 프로젝트의 구성요소

| 이름 | 역할 |
|---|---|
| `npuforge-scheduler` | 중앙 스케줄러. 클라이언트 요청을 노드로 분배 (x86_64, 서버) |
| `npuforge-node` | 노드 에이전트. NPU 추론 수행 (aarch64, 보드 3대) |
| `npuforge-bench` | 부하 생성·집계·run 유효성 판정 |
| `npuforge-proto` | `.proto` 단일 출처 |
| `npuforge-rknn` | RKNN 백엔드 |
| `npuforge-mock-backend` | 하드웨어 없이 개발·테스트용 |
| `npuforge-common` | 타입·오류코드·설정·백엔드 인터페이스 |
| **king / queen / jack** | 노드 보드 3대의 이름 (SSH 별칭 `npuforge-k/q/j`) |
| **server** | 스케줄러 + bench 호스트 (`npuforge-server`) |

### 11.1 오류 코드

| 코드 | 뜻 |
|---|---|
| `NPF-0000` | 성공 |
| `NPF-1002` | payload 크기 초과 |
| `NPF-1303` | 노드 과부하(큐 가득) |
| `NodeUnavailable` | 전송 실패 → 헬스 카운터에 반영 |
| `NoAvailableNode` | 처리 가능한 노드 없음 |

---

## 12. 이 프로젝트에서 정한 실험 규칙

측정 전에 정하고 **결과에 맞춰 바꾸지 않는** 값들이다.

| 규칙 | 값 | 근거 |
|---|---|---|
| operating concurrency | peak 의 **98%** 이상을 내는 가장 낮은 concurrency | 99% 는 run 간 SD(±1 inf/s)와 겹친다 |
| steady-state | 마지막 **1/3** 구간 평균 | |
| degradation 판정 | <3% / 3~10% / >10% | |
| Selected operating point | 처리량 최대의 **97%** 이상 중 p95 최소 | 통계적 최적이 아니라 **engineering heuristic** |
| 정책 이동 판정 | 분배 **3%p** 이상 이동 = 이동, 처리량 **2%** 이상 = 회수 | |
| 강한 이질 게이트 | RR 노드 p50 최대/최소 **≥ 2.0×** | S0-A 2.4× ↔ S0-C 2차 1.33× 사이 (S0-C §17.2) |
| LQ vs ECT 판정 밴드 | 처리량 **2%**, p99 **5%** | n=4 에서 그보다 작은 차이는 못 쓴다 (S0-C §17.3) |
| 현직 tie-break | 밴드를 못 넘으면 **기존 기본값 유지** | 현직을 끌어내리려면 적극적 근거가 필요하다 |

---

## 13. 방법론 교훈에서 나온 표현

| 표현 | 뜻 |
|---|---|
| **"배제는 조건부다"** | 한 번 배제한 병목 후보도 조건이 바뀌면 다시 열린다. 판정에는 **어떤 조건에서** 가 붙어야 한다 |
| **"Optimize at the operating point, not in the overload region"** | 과부하 구간에서 구성을 비교하면 configuration effect 가 아니라 overload behavior 를 본다 |
| **"조용한 실패를 큰 소리로"** | 하네스가 조건 미달이면 그냥 멈춘다. 노드 수 검증·설정 주입 검증·TCP 커넥션 수 물증 |
| **"프로세스가 떠 있다 ≠ 트래픽을 받는다"** | 노드 수는 probe bench 의 **응답 노드 ID 분포**로 확인한다 |
| **"두 측정이 일치해도 해석이 옳다는 뜻은 아니다"** | 둘 다 같은 편향이면 재현성은 편향만 확인해 준다 |
| **"성능이 이상하면 구현이 의도대로 도는가를 먼저"** | 55% 는 품질 차이의 크기가 아니다 |
| **"두 측정량을 곱하지 않는다"** | 처리량 손실 %와 지연 구성비 %는 다른 축 |
| **"비용이지 제약이 아니다"** | 포화되지 않은 자원의 사용량(CPU-ms/req)을 줄여도 처리량은 오르지 않는다. S4 판정의 핵심 |
| **"계기가 다른 물리량을 재고 있을 수 있다"** | 출력이 예상과 다르면 **계측기부터 의심한다.** 임계를 옮기는 것과 계기를 고치는 것은 다르다 |
| **"중단했다를 믿지 말고 공유 자원 쪽에서 확인한다"** | 로컬 프로세스 관측은 플랫폼에 따라 거짓말을 한다. 클러스터가 비었는지는 **클러스터에게 묻는다** |
| **"손으로 관리하는 파생 수치는 갈라진다"** | run 합계·백분율은 스크립트가 세게 하고 출처를 적는다 |
