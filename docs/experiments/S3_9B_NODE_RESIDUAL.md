# S3.9b — Node-side Residual Cost Profiling

- 실험 ID: **S3.9b**
- 측정일: 2026-08-21
- 코드: `62855bd`
- 상태: **완료** (4 조건 × 45초 수집, 오류 0)
- 원본: [`../../results/node-residual-20260821/`](../../results/node-residual-20260821/)
- 선행: [`S3_5_TRANSPORT_PROFILE.md`](S3_5_TRANSPORT_PROFILE.md) ·
  [`S3_9A_SCALEOUT_PROFILE.md`](S3_9A_SCALEOUT_PROFILE.md)

---

## 1. Research Question (좁게)

> **161.5 → 135.5 사이의 residual gap 에서 node-side serialization /
> copy / syscall 비용이 유의미한 비중을 차지하는가?**

**gap 전체를 설명하는 것이 목적이 아니다.** S3.9a 에서 scale-out
tail/TCP 쪽 비용이 별도로 드러났으므로 node-side 프로파일이 26.0 inf/s
전부를 설명해야 할 이유가 없다. 설명 못 한 잔여는 잔여로 남긴다.

판정 규칙은 **측정 전에** 정했다.

| 결과 | 결정 |
|---|---|
| syscall·copy 가 **충분히 큼** | S4 io_uring 진입 |
| **작음** | **S4 취소/보류** |
| **다른 항이 큼** | 그 항만 기록. 핵심 범위 밖이면 더 안 판다 |

## 2. Method

S3.5 와의 결정적 차이는 **운영점에서 잰다**는 것이다.

```text
S3.5    c32 · conn1   116.6 inf/s   과부하 · baseline
S3.9b   c12 · conn2   136.6 inf/s   운영점 · optimized
```

과부하 구간 값을 운영 판단에 쓰지 않는다(README §4.1). 이 저장소는 같은
함정에 이미 한 번 걸렸다 — 13.2% 오인용 사건.

- 1노드(king only). queen·jack 을 내려 RR 이 나눠 가지 못하게 하고,
  probe 로 **응답한 노드 ID 가 king 하나**임을 물증으로 남긴다.
- 부하 80초 중 **t+20 부터 45초**만 수집. 램프와 warmup 을 뺀다.
- 조건 4개: `idle`(계측기 바닥값) / `op`(운영점) / `strace` / `local`(direct 8스레드).

### 2.1 계기 선택 — perf 가 없다

보드에 `perf` · `bpftrace` · `gdb` 가 없다(커널 6.1.141, 벤더 트리).
심볼 단위 프로파일은 불가능하다. 대신 **`/proc/PID/stat` 의 utime/stime
분리**를 쓴다.

```text
utime  유저 시간 — protobuf 직렬화, 유저공간 copy, HTTP/2 프레이밍
stime  커널 시간 — syscall 진입, TCP 스택, copy_to_user, skb, 드라이버
```

**io_uring 이 줄이는 것은 stime 의 일부다.** 따라서 stime 전체가
io_uring 의 절대 상한이고, 실제 회수 가능분은 그보다 작다.

보조로 `strace -c` 를 10초. ptrace 가 syscall 마다 정지시켜 체류시간이
**부풀려져** 나오므로 **상한으로만** 쓴다 — 부풀린 값이 작으면 실제는
확정적으로 더 작다. 한쪽 방향으로만 유효한 검정이다.

## 3. Results

### 3.1 요청당 노드 CPU

| 조건 | throughput | utime/req | stime/req | **CPU-ms/req** | user% | kernel% |
|---|---:|---:|---:|---:|---:|---:|
| op (운영점) | 136.6 | 14.50 | 11.09 | **25.59** | 56.7 | 43.3 |
| local direct | 157.9 | 5.14 | 4.10 | **9.23** | 55.6 | 44.4 |
| **transport 비용** | | **9.37** | **6.99** | **16.35** | **57.3** | **42.7** |

운영점 136.6 은 S3.8 의 135.5±0.4, S3.7b 의 136.4±0.3 과 일치한다 —
조건이 제대로 잡혔다는 확인이다.

> ⚠️ local 의 157.9 는 80초 **전체 평균**이라 램프를 포함한다. 수집 창
> (t+20~65)의 정상 구간 속도는 162.6 이었다. 이 차이는 local 의
> 요청당 CPU 를 약 3% 과대평가하는 방향이며, **transport 비용을 과소가
> 아니라 과대 추정**하므로 아래 결론(비용이 작다)을 약화시키지 않는다.

### 3.2 어느 코어도 포화가 아니다

```text
op    cpu0  soft=68.3  idle=21.2   ← 유일한 뜨거운 코어 (78.8% busy)
      cpu1~3            idle 61~64
      cpu4~7            idle 42~47
      전체              idle 48.9
local 전체              idle 82.5   softirq 0
```

가장 뜨거운 cpu0 도 21% 남는다. cpu0 의 부하는 대부분 **softirq**
(NIC 단일 수신 큐)인데, **S3.5 §4.3 이 이미 RPS 로 분산시켜 보고
−0.2% null 을 얻었다.** cpu0 softirq 도 제약이 아니다.

### 3.3 syscall — 횟수는 많고 비용은 작다

`strace -c` 10초 (요청 약 1,284건):

| syscall | 체류시간 | 호출 수 | calls/req | |
|---|---:|---:|---:|---|
| futex | 30.07s | 48,565 | 37.8 | 스레드 동기화 **대기** |
| ioctl | 24.72s | 68,924 | 53.7 | RKNN 드라이버 (NPU 제출) |
| epoll_pwait | 9.78s | 37,157 | 28.9 | 이벤트 **대기** |
| **recvfrom** | 9.50s | 136,602 | **106.4** | 요청 수신 ← io_uring 대상 |
| **writev** | 5.91s | 69,245 | **53.9** | 응답 송신 ← io_uring 대상 |
| **write** | 0.35s | 5,524 | **4.3** | 응답 송신 ← io_uring 대상 |

**네트워크 syscall 체류시간은 15.77s / 80.36s = 19.6%** 다. 나머지
80.4% 는 futex(동기화 대기) · ioctl(NPU 드라이버) · epoll(이벤트 대기)로,
**io_uring 이 손대는 영역이 아니다.**

## 4. 판정 — **S4 io_uring 취소/보류**

요청당 네트워크 syscall 은 약 **165회**(recvfrom 106 + writev 54 + write 4).
aarch64 syscall 진입 비용을 **넉넉히 1 µs** 로 잡아도

```text
165 회 × 1 µs = 0.165 ms/req
0.165 / 16.35 = 요청당 transport CPU 의 1.0%
```

등록 버퍼로 1.2 MB copy 를 양방향 모두 없앤다고 **가정해도**(RK3576
메모리 대역폭 기준 약 0.6~1.2 ms) 합계는 **1.4 ms/req ≈ transport
비용의 8%** 다.

그리고 그 8% 를 다 회수해도 **처리량은 오르지 않는다.** 보드 CPU 가
48.9% idle 이고 어느 코어도 포화가 아니며, 가장 뜨거운 cpu0 의 softirq
는 RPS 로 분산해도 −0.2% null 이었기 때문이다.

> **CPU-ms/req 는 비용이지 제약이 아니다.** 포화되지 않은 자원의
> 사용량을 줄이는 것은 처리량을 올리지 않는다.

```text
질문   io_uring 이 남은 16.1% 를 회수하는가?
답     아니다. 회수 대상(syscall 진입)이 transport 비용의 1%,
       가장 관대한 가정으로도 8%. 게다가 CPU 는 제약이 아니다.
```

**S4 는 취소/보류한다.** TECHSPEC §15 의 io_uring 항목은 "필요성 미증명"
이 아니라 **"측정으로 반박됨"** 으로 상태가 바뀐다.

## 5. 판정 규칙 3번째 가지 — 큰 항은 따로 기록한다

질문은 "serialization / copy / syscall" 셋을 묶어 물었는데, 답이 갈렸다.

| 항 | 크기 | 판정 |
|---|---|---|
| **syscall** | transport 비용의 ~1% | **작다** |
| **serialization / 유저공간 copy** | **9.37 ms/req = 57%** | **크다** |

**유저 시간이 커널 시간보다 크다**(9.37 vs 6.99). transport 비용의
과반이 protobuf 직렬화·유저공간 copy·HTTP/2 프레이밍이다. io_uring 은
이쪽을 건드리지 않는다.

다만 **여기서 멈춘다.** 사전 판정 규칙의 3번째 가지대로다 — 큰 항을
기록하되, CPU 가 제약이 아닌 이상 이것을 줄이는 것도 처리량을 올린다는
보장이 없다. 파고들 근거가 아직 없다.

## 6. 그러면 gap 26.0 inf/s 는 무엇인가 — 범위 밖, 관측만 남긴다

이 실험의 임무가 아니지만 방향은 관측된다. 고정 동시성에서
처리량 = 동시성 / 지연이다.

```text
op     c12,  136.6 inf/s  ->  평균 지연 87.8 ms
local  8스레드, 157.9      ->  평균 지연 50.5 ms   (래퍼 실측 50,531 µs)
                              차이 +37.3 ms
```

그중 노드 CPU 작업은 16.35 ms 뿐이고 나머지는 **대기**다. 페이로드가
요청 1.2 MB · 응답 1.2 MB 이므로 실측 링크(2.34 Gbps ≈ 292 MB/s)에서
**순수 전송 시간만 방향당 약 4.1 ms, 왕복 8.2 ms** 다. 여기에 스케줄러
홉과 큐잉이 더해진다.

> gap 은 CPU 비용이 아니라 **경로 지연**의 문제로 보인다. 이것을 줄이는
> 지렛대는 io_uring 이 아니라 **페이로드 크기**다(ADR-008 의 640×640×3
> raw 전송). 다만 이는 S3.9b 의 범위 밖이므로 **관측으로만 남긴다.**

## 7. Limitations

- 조건당 1 run(45초 수집)이다. utime/stime 델타는 45초 누적이라 안정적
  이지만 run 간 SD 는 없다.
- `strace -c` 의 seconds 는 **블로킹 포함 체류시간**이지 CPU 시간이
  아니다. futex·epoll 이 상위인 것은 그 때문이며, 네트워크 syscall
  비중 19.6% 도 같은 척도 안에서만 유효하다. 판정의 주 근거는
  utime/stime 분리이고 strace 는 보조다.
- syscall 진입 비용 1 µs 는 실측이 아니라 aarch64 통상값을 **넉넉히**
  잡은 것이다. 실측하려면 마이크로벤치가 필요하나, 1 µs 가정에서 이미
  1% 이므로 결론이 뒤집히지 않는다.
- local direct 는 `sustained_load_test`(별도 바이너리)라 노드와 코드
  경로가 완전히 같지 않다. 비교의 기준선으로서 S3.5 이래 일관되게 쓴 값이다.

## 8. 이 실험에서 잡은 계기 오류

`strace -c` 요약을 파싱하는 정규식이 **`usecs/call` 과 `calls` 컬럼을
뒤바꿔** 읽었다. 호출 수가 100배 작게 나와 "strace 가 한 스레드에만
붙었다 → 상한 검정 무효" 로 판단할 뻔했다. 기대치(요청당 write 83.4회,
`/proc/PID/io`)와 대조해 잡았다.

> 계측기의 출력이 예상과 다르면 **계측기부터 의심한다**(README §4.10).
> 이번엔 측정이 아니라 파서가 틀렸다.

---

## Figure

![transport 비용의 유저/커널 분해와 io_uring 이 닿는 몫(≈8%)](../../results/node-residual-20260821/figures/fig_transport_cost_split.png)

**`fig_transport_cost_split.png`** — transport 비용의 유저/커널 분해와 io_uring 이 닿는 몫(≈8%)

재생성: `python scripts/make-experiment-figures.py`
