# S3.5 — Transport Cost Profiling

- 실험 ID: **S3.5** (+ **S3.5b** RPS A/B)
- 측정일: 2026-08-20
- 동결 commit: `01f29a2`. 노드·스케줄러·모델·bench **무변경**
- 상태: **완료**
- 원본: [`../../results/transport-profile-20260820/raw/`](../../results/transport-profile-20260820/raw/) ·
  [`../../results/rps-ab-20260820/`](../../results/rps-ab-20260820/)
- 선행: [`S2_GRPC_BASELINE.md`](S2_GRPC_BASELINE.md), [`S3_SATURATION.md`](S3_SATURATION.md)
- 후속: **S3.6** (H2 / channel A/B — 이 문서가 남긴 ①②③을 가른다, §7)

---

## 1. Research Question

> **노드당 상한 ~115 inf/s (로컬 direct ~160 대비 −30%) 를 실제로 무엇이
> 누르고 있는가?**

S2 는 이 손실이 **payload-transfer path** 에 있다는 것까지 밝혔다(non-inference
latency 의 94%). 하지만 그 경로 안에서 *무엇이* 비용인지는 열려 있었다.
후보는 최소 넷이다 — 링크 대역폭, 보드 CPU 총량, 커널 네트워크 스택,
그리고 전송 계층 구조.

**S4(io_uring)를 시작하기 전에 이 질문이 먼저 닫혀야 한다.** io_uring 은
syscall 과 복사 비용을 줄이는 도구다. 병목이 거기가 아니면 큰 구현을 하고도
아무것도 못 얻는다. `01-TECHSPEC.md` §15.1 이 정한 순서(2.CPU profile →
3.syscall·복사 비용 → 4.버퍼 풀 → 5.io_uring)에서 2~4 가 비어 있었다.

또한 §15.4 가 요구하는 지표(syscalls/req, ctx switches/req, cycles/req)는
S4 의 **before** 기준값으로 어차피 필요한데 저장소에 하나도 없었다
(S2 raw 30건에 CPU 항목 없음). 먼저 만들고 나서 재면 개선을 무엇에 귀속시킬
근거가 없다.

## 2. Method

같은 보드(king)에서 세 조건을 잰다. 냉각·governor·모델은 S2·S3 와 동일.

| 조건 | 부하 | 의미 |
|---|---|---|
| `idle` | 없음 | 계측기 자체의 바닥값 |
| `cluster` | 1노드 클러스터 c32 | S3 ceiling 조건 |
| `local` | 로컬 direct 8스레드 | 네트워크 경로가 통째로 빠진 조건 |

`cluster` 와 `local` 의 차이가 곧 transport 가 보드에서 쓰는 비용이다.

- 부하 80초, 그 안쪽 **t+20 부터 45초**만 수집. 램프와 warmup 을 제외한다.
- 보드에서는 `/proc` 원본만 떠 오고 계산은 개발 PC 에서 한다. 나중에 다른
  각도로 다시 볼 수 있어야 한다.
- 수집: `mpstat -P ALL`(코어별), `pidstat -t`(스레드별), `/proc/PID/io`
  (syscr·syscw), `/proc/PID/task/*/status`(ctx switch), `/proc/net/dev`,
  `/proc/interrupts`, `/proc/softirqs`.
- 스크립트: [`run-transport-profile.sh`](../../scripts/run-transport-profile.sh),
  [`node-profile-collect.sh`](../../scripts/node-profile-collect.sh),
  [`analyze-transport-profile.py`](../../scripts/analyze-transport-profile.py).

> `perf` 는 보드에 없다(커널 6.1.141 vendor, apt 는 6.8 용만 제공).
> cycles/req 는 PMU 값이 아니라 코어별 busy 시간 × 고정 클럭(A53 2016 /
> A72 2208 MHz, governor=performance)으로 환산한 **근사값**이다.

## 3. Results

수집 창 45.1초, king, 팬, performance.

| | idle | cluster | local |
|---|---:|---:|---:|
| throughput (inf/s) | 0 | **116.6** | **159.1** |
| **%idle (8코어 전체)** | 99.9 | **63.1** | 82.9 |
| %usr / %sys / %soft | 0.0 / 0.0 / 0.0 | 18.3 / 12.2 / 6.4 | 9.7 / 7.3 / 0.0 |
| **CPU0 busy** | 0.3 | **69.7** | 21.5 |
| **CPU0 %soft** | 0.0 | **51.5** | 0.0 |
| eth0 RX / TX (Gbps) | 0 | **1.196 / 1.194** | 0 |
| **링크 실측(2.34) 대비** | — | **51.1% / 51.0%** | — |
| RX 패킷/s | 9 | 112,008 | 8 |
| NET_RX softirq/s | 10 | 10,954 | 8 |

코어별 busy%:

```text
cluster :  c0=70  c1=38  c2=37  c3=37  c4=30  c5=29  c6=27  c7=27
local   :  c0=21  c1=19  c2=19  c3=19  c4=15  c5=15  c6=15  c7=15
```

### 요청당 비용 (TECHSPEC §15.4 — S4 의 before 기준값)

| | cluster | local | 차이 |
|---|---:|---:|---:|
| **syscalls/req** | **84.5** | ~0.0 | +84.5 |
| ├ read/req | 0.1 | 0.0 | |
| └ write/req | **84.4** | 0.0 | |
| ctx switch/req (vol) | 157.6 | 221.6 | −64.0 |
| ctx switch/req (nonvol) | 0.7 | 0.1 | |
| 프로세스 CPU-ms/req | **22.2** | 9.0 | **+13.2** |
| 보드 전체 CPU-ms/req | **25.3** | 8.6 | **+16.7** |
| ≈ Mcycles/req | 52.9 | 18.1 | +34.8 |
| RX 패킷/req | 960.7 | 0 | |

transport 는 추론 1건당 보드 CPU 를 **약 2.9배** 쓰게 만든다(8.6 → 25.3 ms).
write syscall 이 요청당 **84.4회** — 응답 1,218,000 byte 를 약 14.4 KB 씩
쪼개 내보내고 있다(HTTP/2 프레임 크기와 일치).

## 4. 병목 후보를 하나씩 배제한다

### 4.1 링크 대역폭 — 아니다

2.5GbE 는 full-duplex 라 요청과 응답이 방향을 나눠 쓴다.

| | 바이트/추론 | @116.6 inf/s | 실측 링크(2.34 Gbps) 대비 |
|---|---:|---:|---:|
| RX (요청 640×640×3) | 1,228,800 | 1.196 Gbps | **51.1%** |
| TX (응답 want_float=0) | 1,218,000 | 1.194 Gbps | **51.0%** |

방향당 절반이 남는다. `/proc/net/dev` 실측이 ADR-008 의 페이로드 크기와
4.7% 오차(HTTP/2 + TCP/IP 헤더) 안에서 일치하므로 계산이 아니라 관측이다.

서버 쪽 aggregation 도 아니다. 3노드에서 3.00× 선형 확장이 나왔으므로
(S2 Finding 1) 공유 10G 링크와 스케줄러는 이 지점에서 병목이 아니다.
**같은 이유로 서버·스케줄러 자체도 배제된다** — 1노드에서 116 을 못 넘기게
하는 것이 서버였다면 3노드 342 가 나올 수 없다. 병목은 노드 쪽에 있다.

> ⚠️ **[2026-08-20 추가 — S3.8]** 이 배제는 **baseline(노드당 커넥션 1개)
> 조건에서만 유효하다.** 노드당 커넥션을 2개로 올리자 shared path 부하가
> 늘어 optimized 3N 의 scaling efficiency 가 **98.9% → 95.3%** 로 내려갔고,
> 서버 10G 링크도 67% → **76%** 로 올라왔다. **서버·스케줄러는 다시 후보다.**
> → [S3.8 §4.3](S3_8_OPTIMIZED_SCALEOUT.md)
>
> 배제 판정에는 **어떤 조건에서 배제됐는지**가 함께 붙어야 한다.

### 4.2 보드 CPU 총량 — 아니다

8코어 전체 **63.1% idle**. 가장 바쁜 CPU0 도 30.3% 남는다.

### 4.3 커널 softirq 편중 (CPU0) — **A/B 로 반증됨**

프로파일은 CPU0 이 유독 바쁘다고 지목했다(busy 69.7%, 그중 %soft 51.5%.
나머지 코어는 27~38%). eth0 는 **RX 큐 1개**, IRQ 는 CPU0 고정, **RPS 꺼짐**
(`rps_cpus=00`, [`nic-topology.txt`](../../results/transport-profile-20260820/raw/nic-topology.txt)).
그래서 NET_RX softirq 가 전부 CPU0 에서 직렬 처리된다.

코드 0줄로 검증 가능하므로 먼저 쟀다 — **S3.5b**: `rps_cpus` 를 `00`(CPU0만)
과 `fe`(코어 1~7)로 번갈아, 각 3회 60초, c32.

| rps_cpus | throughput | CPU0 %soft |
|---|---:|---:|
| `00` (기본) | **115.9 ± 0.7** | 50.4 / 50.9 / 51.3 |
| `fe` (코어 1~7) | **115.6 ± 0.9** | 42.4 / 41.9 / 42.0 |
| 차이 | **−0.3 inf/s (−0.2%)** | |

**효과 없음.** softirq 는 실제로 이동했는데(51% → 42%) 처리량은 그대로다.
CPU0 은 병목이 아니었다 — busy 69.7% 로 이미 30% 남아 있었던 것과 일치한다.

> 이 null 결과는 §4.4 의 근거가 된다. RPS 는 **flow 해시**로 분산한다.
> 흐름이 하나뿐이면 나눌 것이 없다. 그리고 실제로 흐름은 하나다.

### 4.4 HTTP/2 전송 경로 — **남는 것은 여기다**

부하 중 실제 TCP 연결을 셌다.

```text
king  ← scheduler   : 1 connection   192.168.123.3:51001 ← 192.168.123.9:37992
server: bench → scheduler : 32 connections (c32, 워커당 1개)
```

코드도 같은 말을 한다.

- bench 는 **동시성 워커마다 채널을 하나씩** 만든다
  ([`driver.rs:83-90`](../../crates/npuforge-bench/src/driver.rs)).
- 스케줄러는 **노드당 채널 하나**를 캐시해 재사용한다
  ([`node_client.rs:31-79`](../../crates/npuforge-scheduler/src/node_client.rs)).
  HTTP/2 다중화를 믿고 내린 결정이고, 요청마다 핸드셰이크를 피한다는
  근거 자체는 옳다.

결과적으로 **클라이언트 쪽 32 연결이 노드 앞에서 1 연결로 수렴한다.**
동시 요청 32건이 전부 이 연결 하나의 HTTP/2 스트림으로 흐른다. 그 연결은

- h2 커넥션 상태 기계 하나가 직렬로 프레이밍한다(단일 태스크),
- **64 KB 커넥션 flow-control window 하나를 32 스트림이 나눠 쓴다**
  — tonic 0.12.3 / h2 0.4.15 에서 window 설정이 코드 어디에도 없어
  전부 기본값(65,535)이다,
- TCP 흐름이 하나라 RPS·RSS 로 나눌 수 없다(§4.3).

다만 **이 셋은 아직 한 덩어리다.** HTTP/2 는 원래 커넥션 하나에서 스트림을
다중화하라고 만든 프로토콜이다. "커넥션이 1개" 라는 사실만으로 병목이라고
단정할 수 없다. 최소한 셋으로 갈라야 한다.

| 하위 후보 | 내용 |
|---|---|
| ① flow control | 64 KB 기본 window 가 1.2 MB 메시지를 stop-and-wait 로 만든다 |
| ② 커넥션/TCP 경로 | h2 커넥션 상태 기계·소켓 하나가 직렬화 지점이다 |
| ③ protobuf·복사 | 프레이밍과 encode/decode, `to_vec()` 복사 비용 |

**S3.6 이 이 셋을 가른다**(§7). 아래 정합성은 "전송 경로가 의심된다" 까지를
지지하는 것이지, 셋 중 어느 것인지를 지목하지 않는다.

관측이 전부 이 그림과 맞는다.

| 관측 | 단일 커넥션 가설과의 정합 |
|---|---|
| 대역폭 51%, CPU 63% idle | 자원이 아니라 **대기**가 상한을 만든다 |
| RPS 무효 | 흐름이 하나라 분산할 대상이 없다 |
| `node_queue` ≈ 0.02 ms | 요청이 워커를 기다리는 게 아니라 **도착을 못 한다** |
| 같은 보드 로컬 direct 8워커 = **161.5 inf/s** | 클러스터 116. 노드에 여유가 남는다 |
| S3 plateau (노드당 c10~16 이후 무증가) | 스트림을 늘려도 커넥션 상한은 그대로 |
| write syscall 84.4/req (≈14.4 KB) | 한 커넥션이 프레임 단위로 직렬 송신 |

특히 **`node_queue` ≈ 0 과 로컬 direct 161.5 inf/s** 가 결정적이다. 노드가
자기 상한(161.5)에 걸렸다면 c32 부하에서 워커 대기가 쌓여야 한다. 그런데
`node_queue` 는 0.02 ms 다. 받은 것을 즉시 처리하고 여유가 남는다는 뜻이다.
병목은 워커 풀 **앞**, 전송 계층에 있다.

> ⚠️ **`8워커 / inference_us 24.7 ms ≈ 324 inf/s` 를 노드 용량으로 쓰면
> 안 된다.** 같은 보드의 로컬 direct 8워커가 161.5 inf/s 에 그치므로 워커
> 8개가 독립적으로 돌지 않는다 — RKNN 런타임·NPU 내부 경합이 이미 있다.
> starvation 의 비교 기준은 **161.5** 다. 회수 가능한 gap 은 116 → 161.5,
> 약 30% 이지 116 → 324 가 아니다.

## 5. Interpretation

노드당 −30% 손실(116 → 161.5)의 정체는 **compute 도, 대역폭도, 커널 스택도
아니라 스케줄러↔노드 HTTP/2 전송 경로**다. 그 경로 안에서 flow control /
커넥션 / 직렬화 중 무엇인지는 **S3.6 에서 가른다.**

S2 Finding 2("오버헤드는 payload transfer path 에 있다")는 유효하다. S3.5 는
그 경로 안에서 비용의 성격을 바꿔 놓는다 — **바쁜 비용이 아니라 기다리는
비용**이다. 보드는 63% 놀고 링크는 49% 비어 있는데 처리량이 안 오른다.

## 6. Limitations

- **§4.4 는 아직 반증 실패이지 증명이 아니다.** 다른 셋을 배제했고 모든
  관측이 정합하지만, 커넥션 수나 window 를 바꿔 처리량이 오르는 것을
  직접 보여야 확정된다. 그 검증은 코드 변경이 필요해 동결을 벗어난다.
- 단일 보드(king), 단일 조건(c32, 45초 창). 3노드 프로파일은 없다.
- cycles/req 는 PMU 없는 근사값(§2 주). 절대값이 아니라 조건 간 비교용.
- `local` 조건의 도구(`sustained_load_test`)는 노드와 다른 프로그램이다.
  지연 정의가 달라(50.2 ms vs `inference_us` 24.7 ms) 두 값을 직접 빼면
  안 된다. 처리량과 CPU 점유만 비교했다.
- S3.5b 는 `rps_cpus` 만 바꿨다. RSS(다중 RX 큐)는 r8125 단일 큐라 불가.
- **S3.5b 의 per-run bench JSON 은 마지막 1건만 남았다.** 스크립트가 run
  사이에 `rm -f *.json` 으로 출력 디렉터리를 비워 앞선 원본을 함께 지웠다
  (수정 완료). 처리량·CPU0 %soft 는 `raw/results.csv` 와 `raw/mpstat_*` 에
  6건 모두 남아 있어 §4.3 의 결론은 영향받지 않는다.

## 7. S4 에 대한 함의

**io_uring 은 이 병목을 겨냥하지 않는다.** io_uring 이 줄이는 것은 syscall
진입 비용과 복사다. 지금 보드는 CPU 가 63% 놀고 있으므로, syscall 을 더
싸게 만들어도 상한이 오르지 않는다. TECHSPEC §15.3 이 적어 둔 비적용 조건
("구현 복잡도 대비 개선이 5% 미만")에 정면으로 해당한다.

측정된 비용 순서대로, 훨씬 싼 수단이 앞에 있다.

**io_uring 을 취소하는 것이 아니다.** 그 정도의 칼을 꺼낼 문제인지를 마지막으로
확인하는 단계를 넣는다.

```text
S2   scaling baseline      DONE
S3   saturation            DONE
S3.5 transport profiling   DONE  ← 이 문서
S3.6 H2 / channel A/B      다음   ← 원인을 셋으로 가른다
       ↓
     원인 확정
       ↓
S4 ├─ H2 tuning 이 답이면 → gRPC optimized
   └─ 아니면              → io_uring
```

S3.6 은 최소 변경으로 §4.4 의 ①②를 분리한다. 1노드 saturation 동일 조건에서:

| Test | 노드당 커넥션 | H2 window | 목적 |
|---|---:|---|---|
| A | 1 | default | baseline (= 현재 115) |
| B | 1 | 크게 확대 | **flow control 검증** |
| C | 4 | default | **커넥션/TCP 경로 검증** |
| D | 4 | 확대 | 결합 효과 |

해석은 깨끗하다.

- **B 만 상승** → 범인은 커넥션 수가 아니라 HTTP/2 flow control
- **C 만 상승** → 범인은 단일 커넥션 / TCP 경로
- **B·C 둘 다 상승** → 둘 다 영향
- **D 까지 그대로** → HTTP/2 가설 약화 → ③(protobuf·복사·syscall)로 복귀,
  **이때 io_uring 이 훨씬 강한 근거를 갖는다** (대역폭도, CPU 배치도,
  flow control 도 아니었다)
  — 단 "스케줄러도 아니다" 는 이후 S3.8 에서 **철회**됐다(위 §4.1 주 참조)

window 는 최적값 탐색이 아니라 **64 KB 급 기본값이 막고 있었는지 여부**만
본다. 수 MB~수십 MB 수준으로 충분히 크게 잡는다.

만약 window 만 키워 115 → 145~155 가 나온다면 S4 의 결론이 바뀐다 —
"gRPC 가 느린 게 아니라 **기본 HTTP/2 설정이 대형 payload workload 와 맞지
않았다**". 수천 줄짜리 transport 를 새로 만들기 전에 몇 줄짜리 설정으로
30% 의 상당 부분을 회수한다면, 시스템 연구로서 그쪽이 더 강한 판단이다.

그 밖에 측정으로 뒷받침되는 수단:

| 수단 | 근거 |
|---|---|
| **응답 페이로드 축소** — 노드 postprocess 후 검출결과만 반환 (1.218 MB → 수 KB) | 와이어·protobuf·복사 부하를 절반 제거 |

## 8. Reproduction

```bash
bash scripts/run-transport-profile.sh              # 세 조건 (약 5분)
bash scripts/run-transport-profile.sh --only local # 한 조건만
PYTHONIOENCODING=utf-8 python scripts/analyze-transport-profile.py

bash scripts/run-rps-ab.sh                         # S3.5b (약 10분)
```

동결 commit `01f29a2`. `run-rps-ab.sh` 는 `rps_cpus` 를 런타임으로만 바꾸고
끝에서 원래 값(`00`)으로 되돌린다.

## 9. Conclusion

노드당 ~116 inf/s 상한의 원인은 **스케줄러↔노드 HTTP/2 전송 경로**에 있다.
링크 대역폭(방향당 51% 사용), 보드 CPU 총량(63% idle), 커널 softirq 편중
(RPS A/B −0.2%), 서버·스케줄러(3노드 3.00× 선형)는 모두 배제됐다. 노드는
같은 보드에서 로컬 direct 161.5 inf/s 를 내면서 클러스터에서는 116 에
그치고, `node_queue` ≈ 0 으로 여유가 남는다.

전송 경로 안에서 ①flow control ②커넥션/TCP ③protobuf·복사 중 무엇인지는
**아직 갈리지 않았다.** → **S3.6** 이 최소 변경 A/B 로 이를 가르고, 그
결과가 S4 를 `gRPC optimized` 와 `io_uring` 중 하나로 확정한다(§7).
