# NPUForge Hardware Setup Guide

- 문서명: `02-HARDWARE-SETUP.md`
- 프로젝트명: NPUForge
- 문서 버전: v0.2
- 대상 릴리스: NPUForge v0.1
- 목표 발표: 2026년 11월 FOSS for All Conference
- 작성일: 2026-08-05
- 최종 수정: 2026-08-06
- 상태: Draft
- 관련 문서:
  - `00-PRD.md`
  - `01-TECHSPEC.md`
  - `03-DEVELOPMENT-REQUIREMENTS.md`
  - `environment-matrix.md`

본 문서는 물리 구성, 네트워크, 전원, 냉각, 실험 조건에 대한 규범 문서다. 해당 영역에서 다른 문서와 값이 다를 경우 본 문서를 따른다.

---

# 1. 권장 구성

NanoPi R76S 3대는 모두 동일한 NPU Worker로 구성한다.

중앙 스케줄러, 대시보드, 벤치마크 클라이언트는 **별도 서버**에서 실행한다.
**PCIe 슬롯이 있어야 한다** — 10G NIC 을 꽂아야 하기 때문이다(§3.3.2).

```text
              ┌────────────────────────────┐
              │ Benchmark / Scheduler      │
              │ Server (PCIe 슬롯 필요)    │
              │                            │
              │ · NPUForge Scheduler       │
              │ · Benchmark Client         │
              │ · Dashboard                │
              │ · Prometheus               │
              └─────────────┬──────────────┘
                            │ 10GbE (SFP+ DAC)   ← aggregation
                  ┌─────────▼──────────┐
                  │ 2.5G / 10G Switch  │
                  └────┬─────┬─────┬───┘
                       │2.5G │2.5G │2.5G
               ┌───────▼┐ ┌──▼────┐ ┌▼──────┐
               │ KING   │ │ QUEEN │ │ JACK  │
               │ Worker │ │ Worker│ │ Worker│
               │ 6 TOPS │ │ 6 TOPS│ │ 6 TOPS│
               └────────┘ └───────┘ └───────┘
```

**worker 링크는 2.5G, aggregation 만 10G** 다. 노드당 최대 1.545 Gbps
이므로 2.5G 로 충분하고, 세 노드가 합류하는 스케줄러 쪽이 병목이 된다.
근거는 §3.3.2.

핵심 원칙:

```text
3대 NanoPi = 동일한 Worker
별도 Linux PC = Scheduler
2.5GbE = 추론망
동일 OS·커널·RKNN·모델
독립 전원
동일 냉각
중앙에서 모든 벤치마크 기록
```

---

# 2. 장비별 역할

| 장비 | 역할 | 비고 |
|---|---|---|
| Linux PC | Scheduler | 요청 접수와 노드 선택 |
| Linux PC | Benchmark Client | 부하 발생과 결과 저장 |
| Linux PC | Dashboard | 실시간 처리량·장애 표시 |
| Linux PC | Metrics Server | Prometheus 및 선택적 Grafana |
| KING | NPU Worker | RKNN 추론 |
| QUEEN | NPU Worker | RKNN 추론 |
| JACK | NPU Worker | RKNN 추론 |

## 2.1 NanoPi를 모두 Worker로 사용하는 이유

한 대의 NanoPi에서 Scheduler와 NPU Worker를 함께 실행하면 다음 문제가 발생한다.

- 해당 노드만 CPU와 네트워크 부하가 증가한다.
- 세 노드의 실험 조건이 달라진다.
- 1노드, 2노드, 3노드 비교가 왜곡될 수 있다.
- Scheduler 병목과 NPU 병목을 분리하기 어렵다.
- 발표에서 동일 조건의 비교라고 설명하기 어려워진다.

따라서 공식 벤치마크에서는 세 대를 완전히 대칭적인 Worker로 유지한다.

단순 개발 또는 이동형 데모에서는 KING에 Scheduler를 함께 실행할 수 있지만, 공식 성능 수치에는 사용하지 않는다.

---

# 3. 네트워크 구성

## 3.1 권장 토폴로지

**2.5G/10G 스위치**를 중심으로 한 스타 구조를 사용한다.
worker 는 2.5G, 스케줄러 업링크만 10G 다(§3.3.2).

필요 장비:

- **2.5G/10G 스위치 1대** — 2.5G 포트 ≥ 4 + SFP+ 업링크
- **스케줄러 서버 1대** — PCIe 슬롯 필요
- **10G NIC (PCIe, SFP+)** 1장 — 예: Intel X520
- **SFP+ DAC 케이블** 1개 — 서버 ↔ 스위치
- CAT5e 케이블 3개 — 스위치 ↔ 노드 (2.5G 는 Cat5e 로 충분)
- NanoPi R76S 3대

```text
Linux PC ───┐
KING ─────┤
QUEEN ─────┼── 2.5GbE Switch
JACK ─────┘
```

## 3.2 IP 주소 계획

NPUForge 전용 추론망 예시:

```text
Network     : 10.20.0.0/24
Scheduler   : 10.20.0.10
KING      : 10.20.0.21
QUEEN      : 10.20.0.22
JACK      : 10.20.0.23
```

Hostname:

```text
npuforge-scheduler
npuforge-king
npuforge-queen
npuforge-jack
```

`/etc/hosts` 예시:

```text
10.20.0.10  npuforge-scheduler
10.20.0.21  npuforge-king
10.20.0.22  npuforge-queen
10.20.0.23  npuforge-jack
```

## 3.3 관리망 분리 (필수)

NanoPi R76S는 **2.5GbE 포트를 2개** 가지고 있다. 따라서 관리망 분리가 추가 비용 없이 가능하며, 선택이 아니라 기본 구성으로 둔다.

```text
포트 1 → 기존 사내/가정 네트워크   = 관리망
           SSH, apt, 바이너리 배포, 로그 수집

포트 2 → 전용 스위치               = 추론망
           추론 트래픽만. 다른 트래픽 없음
```

관리망을 분리하는 이유는 편의가 아니라 **측정 오염 방지**다. SSH 세션, `apt` 다운로드, 로그 전송이 추론망에 섞이면 네트워크 지연 측정값에 원인 불명의 튀는 값이 생긴다.

포트 이름은 보드와 커널에 따라 다르므로 반드시 확인한다.

```bash
ip -br a
for n in /sys/class/net/e*; do
  echo "$(basename $n): speed=$(cat $n/speed 2>/dev/null) mac=$(cat $n/address)"
done
```

어느 포트를 추론망에 쓸지는 세 노드에서 **동일해야 한다.** `environment-matrix.md` §8에 노드별로 기록한다.

## 3.3.1 단계적 구축

추론망 스위치 도입 전에도 개발은 진행한다.

| 단계 | 관리망 | 추론망 | 가능한 작업 |
|---|---|---|---|
| 현재 | 기존 1G 허브 | 없음 | 보드 세팅, RKNN 검증, 단일 노드 |
| 중간 | 기존 1G 허브 | 1G 허브 공유 | M2~M5 전체 (gRPC, 3노드, 장애 복구) |
| 최종 | 기존 1G 허브 | 2.5GbE 전용 스위치 | 공식 벤치마크 |

**중간 단계로도 M5까지 전부 개발 가능하다.** 링크 속도는 기능 정확성에 영향을 주지 않으며, 2.5GbE가 필요한 것은 공식 성능 수치뿐이다.

JPEG 입력 기준으로는 1GbE로도 대역폭이 충분하다(§3.1 근거 계산 참조). Raw RGB 입력 시나리오에서만 1GbE가 먼저 포화된다.

## 3.3.2 Scheduler 호스트의 링크 속도 — **10G 필요** (2026-08-12 개정)

Scheduler 호스트는 세 노드 트래픽이 모두 합류하는 지점이다. 실측 처리량이
나오고 나서 계산하니 **2.5GbE 로는 부족하다**는 것이 확인되었다.

raw RGB 입력 한 장은 `640 × 640 × 3 = 1,228,800 byte` 다.

```text
                    노드당           3노드 합계
INT8  157.2 inf/s   1.545 Gbps       4.636 Gbps
FP16   84.3 inf/s   0.829 Gbps       2.486 Gbps
```

**FP16 조차 3노드 합계가 2.486 Gbps 로 2.5GbE 한 링크(실효 약 2.35 Gbps)를
넘는다.** INT8 은 4.636 Gbps 로 두 배 가까이 넘는다.

즉 **worker 링크가 아니라 aggregation 링크가 먼저 막힌다.**

### 개정된 토폴로지

```text
        Benchmark / Scheduler Server
                    │
                  10GbE          ← aggregation. 여기가 핵심
                    │
            2.5G / 10G Switch
              ├── 2.5G ── king
              ├── 2.5G ── queen
              └── 2.5G ── jack
```

- **worker 링크는 2.5G 그대로** 다. 노드당 최대 1.545 Gbps 이므로 충분하다
- **aggregation 링크만 10G** 로 올린다
- 스위치는 2.5G 포트 + 10G(SFP+) 업링크를 가진 모델

### 필요한 것

| 항목 | 사양 | 비고 |
|---|---|---|
| Scheduler 호스트 | **PCIe 슬롯이 있는 서버** | 노트북은 10G 카드를 못 꽂는다 |
| ┗ CPU | **16 스레드 이상 권장** | 2026-08-26 실측. §3.3.4 |
| 10G NIC | PCIe, SFP+ (예: Intel X520) | |
| DAC 케이블 | SFP+ Direct Attach | 광 모듈보다 싸고 짧은 거리에 적합 |
| 스위치 | 2.5G 포트 ≥ 4 + SFP+ 업링크 | |

> **이전 판(2.5GbE NIC 확보)은 폐기한다.** 그 계산은 노드당 실측
> 처리량이 나오기 전의 추정이었다. 확보하지 못한 상태에서 측정한 값을
> 공식 수치로 쓰지 않는다는 원칙은 그대로다.

## 3.3.3 M3 이전에 기록할 실측 네트워크 값

계산이 아니라 **실측**이다. `dealer`(또는 새 스케줄러 서버)에서
인터페이스 카운터로 잰다.

| 조건 | 기록할 것 |
|---|---|
| 단일 요청 | 스케줄러 TX bytes / RX bytes |
| 1노드 포화 | TX Gbps / RX Gbps |
| 3노드 포화 | 합계 TX / RX Gbps |

측정 방법 예시.

```bash
# 인터페이스 카운터 스냅샷 → 부하 → 다시 스냅샷
IF=enp3s0
read t0 r0 <<< "$(awk -v i=$IF '$1==i":"{print $10, $2}' /proc/net/dev)"
# ... 부하 ...
read t1 r1 <<< "$(awk -v i=$IF '$1==i":"{print $10, $2}' /proc/net/dev)"
echo "TX $(( (t1-t0)*8/1000000 )) Mb   RX $(( (r1-r0)*8/1000000 )) Mb"
```

이 값이 계산값(입력 1.545 Gbps/node)과 크게 다르면 **계산의 전제가
틀린 것이다.** 어느 쪽이 맞는지 확인하기 전에 S2 를 진행하지 않는다.

---

## 3.3.4 Scheduler 호스트의 CPU — **스레드 수가 처리량을 가른다** (2026-08-26)

§3.3.2 는 **대역**만 다뤘다. 실제로 호스트를 바꿔 보니 **CPU 가 먼저
좁아졌다.**

| | 구서버 | 신서버 |
|---|---|---|
| 호스트 | Dell PowerEdge R620 | ASUS H81M-K 데스크톱 |
| CPU | Xeon E5-2630L ×2 · **24 스레드** · 2.0~2.5GHz | Core i7-4790 · **8 스레드** · 3.6~4.0GHz |
| 10G NIC | **같은 Intel X550T 카드** (옮겨 꽂았다) | |
| 측정 중 서버 CPU | 42% | **82.2%** |
| **처리량** | **~391 inf/s** | **~360 inf/s** (−7.5%) |
| 오류율 | 0 | 0 |

**싱글스레드 성능은 신서버가 확실히 빠른데도 처리량이 떨어졌다.**
스케줄러 워크로드는 단일 요청 지연이 아니라 **동시 스트림 처리량**에
좌우된다.

### 왜 CPU 가 먼저 좁아지나

```text
스케줄러        45.3%  ≈ 3.6 코어
기타(벤치+커널)  36.9%  ≈ 2.9 코어
────────────────────────────────
합계            82.2%  (8 스레드 기준)
```

**벤치 클라이언트가 스케줄러와 같은 호스트에서 돈다.** 세 노드로 보내고
받는 일과 부하를 만드는 일을 한 CPU 가 나눠 한다. 24 스레드에서는 같은
일이 42% 였다.

애플리케이션 큐는 두 조건 모두 비어 있다(`scheduler_queue` 0.00ms ·
`scheduler_route` 0.01ms). 좁아진 곳은 큐가 아니라 **호스트의 CPU** 다.

### 따라할 때

| | |
|---|---|
| **권장** | 16 스레드 이상 |
| 최소 | 8 스레드로도 **오류 0 으로 정상 동작**한다. 처리량 기준선이 달라질 뿐이다 |
| 확인법 | 측정 중 서버 CPU 사용률을 본다. 80% 를 넘으면 호스트가 제약이다 |
| 대안 | 벤치를 다른 호스트에서 돌린다. 단 그 호스트도 10G 여야 한다 |

**낮은 사양을 쓸 거면 그 호스트에서 기준선을 새로 깔고, 다른 호스트 값과
직접 비교하지 않는다.**

> PCIe 세대는 병목이 아니었다. 같은 X550T 가 R620 에서는 PCIe 3.0 x4
> (방향당 ~32Gbps), 신서버에서는 2.0 x4(~16Gbps)로 물렸지만 3노드
> 실사용이 방향당 ~4.6Gbps 라 양쪽 다 여유가 크다.

근거: `infrastructure.md` §3.2.1 · `hosts/` · `../results/baseline-20260826-althost/`

### ⚠️ 출력 방향이 더 크다 — `want_float=1` 이면 10G 로도 부족했다

위 계산은 **입력(TX) 방향만** 이다. 출력을 계산하니 결론이 뒤집힌다.

노드는 후처리를 하지 않고 **원시 텐서 9개를 그대로 반환**한다.

```text
입력                      1,228,800 byte
출력 (want_float=1, f32)  4,872,000 byte   ← 입력의 3.96배
출력 (want_float=0, int8) 1,218,000 byte   ← 입력의 0.99배
```

3노드 포화 시 스케줄러 링크에 걸리는 부하다.

| 구성 | 모델 | 3노드 TX | 3노드 RX | 10G 로 되나 |
|---|---|---:|---:|---|
| `want_float=1` (구 기본값) | INT8 | 4.64 Gbps | **18.38 Gbps** | **불가** |
| `want_float=1` (구 기본값) | FP16 | 2.49 Gbps | **9.86 Gbps** | 겨우 (여유 없음) |
| **`want_float=0` (현재 기본값)** | INT8 | 4.64 Gbps | 4.60 Gbps | 가능 |
| **`want_float=0` (현재 기본값)** | FP16 | 2.49 Gbps | 2.46 Gbps | 가능 |

**`want_float=1` 이었다면 10G 로도 INT8 3노드를 감당하지 못했다.**

### 따라서 M3 전에 둘 중 하나가 필요했다

**(A) `want_float=0` 으로 전환** — ✅ **2026-08-12 완료**

출력을 원본 dtype 그대로 받아 **RX 를 4분의 1로 줄인다.** 대신 받는 쪽이
역양자화를 직접 해야 하므로, blob 을 **v2** 로 올려 텐서마다
`qnt_type`·`scale`·`zero_point` 를 함께 싣는다. 노드 설정
`[worker] want_float` 의 기본값이 `false` 다.

> 실보드에서 역양자화 결과가 float32 와 일치함을 확인했다
> (텐서 9개, **최대 오차 9.5e-7** — float32 정밀도 한계).
>
> **처리량도 함께 올랐다 — INT8 +17.3% / FP16 +15.7%** (`king`, 8스레드,
> 120초). `discuss.md` §5 의 +5.4% 는 1스레드 위주 FP16 값이라 작게
> 나온 것이다. 승격 근거는 처리량이 아니라 **RX 대역폭**이었지만,
> 두 지표가 같은 방향을 가리켰다. `discuss.md` §12

**(B) 노드에서 후처리(NMS)까지 수행** — 미구현

검출 결과만 반환하면 응답이 수 KB 로 줄어 RX 가 사실상 사라진다.
올바른 최종 형태지만 구현이 남아 있다.

두 방향 모두 **입력 TX 4.64 Gbps 는 그대로**이므로 10G aggregation 은
여전히 필요하다.

### 그래도 실측한다

위는 전부 계산이다. M3 시작 전에 실제 TX/RX 를 측정해 기록한다(§3.3.3).
**입력만 계산하고 출력을 안 본 것이 이번 오류의 원인이다.** 계산만 믿고
넘어가면 같은 실수를 반복한다.

---

## 3.4 초기 네트워크 제한

초기에는 다음 구성을 사용하지 않는다.

- Wi-Fi
- 노드 직렬 연결
- Docker Overlay Network
- Kubernetes Network
- 복잡한 VLAN
- Jumbo Frame
- 다중 서브넷 라우팅

기본 MTU는 모든 장치에서 1500으로 통일한다.

Jumbo Frame은 기준 성능을 확보한 뒤 별도 실험으로 비교한다.

---

# 4. 저장장치 구성

## 4.1 우선순위

1. **eMMC** (온보드 32GB 또는 64GB)
2. 고내구성 microSD
3. 일반 microSD는 초기 개발에만 사용

NPU Worker는 대용량 데이터를 장기간 저장하지 않으므로 32GB eMMC면 기본 운영에 충분하다.

```text
/opt/npuforge/
├── bin/
├── config/
├── models/
└── logs/
```

벤치마크 데이터 세트, 원본 결과, 그래프와 발표 자료는 Scheduler PC에 저장한다.

## 4.2 NVMe는 사용하지 않는다

NanoPi R76S의 M.2 슬롯은 **SDIO 기반이며 Wi-Fi 모듈용이다.** NVMe SSD를 장착할 수 없다.

따라서 다음 항목은 v0.1 범위에서 제외한다.

- 노드에서의 대용량 영상 파일 저장
- 노드 파일 I/O 성능 비교 실험
- 노드에 여러 모델 로컬 보관
- 노드에서의 장기 로그 보존

벤치마크 데이터 세트, 원본 결과, 로그는 모두 Scheduler 호스트에 저장한다. 세 노드의 단순 추론 구성에서는 이 제약이 문제가 되지 않는다.

M.2 슬롯을 Wi-Fi에 사용하는 것도 v0.1에서는 하지 않는다(§3.4의 Wi-Fi 배제).

---

# 5. 운영체제 구성

## 5.1 권장 OS

세 노드에는 동일한 Debian 또는 Ubuntu Server 계열 Headless 이미지를 설치한다.

라우터 중심의 배포판보다 일반 Linux 개발환경이 적합하다.

필수 통일 항목:

```text
동일 OS 이미지
동일 커널 버전
동일 NPU 드라이버
동일 RKNN Runtime
동일 Rust 바이너리
동일 모델 파일
동일 CPU Governor
동일 냉각 조건
```

## 5.2 기본 패키지

```bash
sudo apt update

sudo apt install -y \
    build-essential \
    pkg-config \
    cmake \
    git \
    curl \
    chrony \
    iperf3 \
    ethtool \
    jq \
    htop \
    sysstat \
    linux-perf
```

배포판에 따라 `linux-perf` 패키지명은 달라질 수 있다.

## 5.3 Rust 바이너리 배포

각 노드에서 개별 빌드하기보다 다음 방식을 권장한다.

1. 빌드 PC에서 ARM64용 바이너리 생성
2. 동일한 바이너리를 세 노드에 배포
3. SHA-256 해시 확인
4. systemd 서비스로 실행

이 방식이 빌드 환경 차이를 줄이고 재현성을 높인다.

---

# 6. RKNN 구성

## 6.1 모델 변환과 실행 분리

```text
개발 PC
  ONNX/PyTorch
      ↓ RKNN-Toolkit2
  model.rknn
      ↓ 배포
KING / QUEEN / JACK
      ↓ RKNN Runtime
  NPU 추론
```

모델 변환은 개발 PC에서 수행하고, NanoPi에서는 변환된 RKNN 모델만 실행한다.

## 6.2 세 노드 간 일치 항목

```text
RKNN Runtime 버전
RKNPU 커널 드라이버 버전
model.rknn SHA-256
전처리 설정
후처리 코드
입력 해상도
양자화 방식
NPU Core 설정
```

모델 해시 확인 예시:

```bash
sha256sum /opt/npuforge/models/yolov8n/model.rknn
```

세 노드의 결과가 동일해야 한다.

## 6.3 모델 디렉터리 예시

```text
/opt/npuforge/models/
└── yolov8n/
    ├── model.rknn
    ├── model.toml
    └── labels.txt
```

---

# 7. 노드 설정

노드별 차이는 다음 세 가지로 제한한다.

```text
Node ID
IP 주소
Hostname
```

그 외 설정, 모델, 바이너리와 런타임 버전은 모두 같아야 한다.

## 7.1 KING

```toml
[node]
id = "king"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.21:51001"
scheduler_address = "http://10.20.0.10:50051"

[backend]
type = "rknn"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]

[worker]
worker_count = 1
max_queue_depth = 32
```

## 7.2 QUEEN

```toml
[node]
id = "queen"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.22:51001"
scheduler_address = "http://10.20.0.10:50051"

[backend]
type = "rknn"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]

[worker]
worker_count = 1
max_queue_depth = 32
```

## 7.3 JACK

```toml
[node]
id = "jack"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.23:51001"
scheduler_address = "http://10.20.0.10:50051"

[backend]
type = "rknn"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]

[worker]
worker_count = 1
max_queue_depth = 32
```

---

# 8. 전원 구성

## 8.1 입력 방식: 12V DC (USB-C PD 아님)

문서 초안은 USB-C PD를 전제했으나 **틀렸다.** NanoPi R76S는 **12V DC 입력**을 사용한다.

2026-08-10 커널 로그 실측:

```text
vcc12v_dcin: 12000 mV, enabled          ← 주 전원 입력
vcc_sys: supplied by vcc12v_dcin
vbus5v0_typec: 5000 mV, disabled        ← Type-C 는 5V 출력용, 입력 아님
power_supply: simple-vin
PMIC: rk806
```

Type-C 포트는 데이터와 5V VBUS **출력**용이며 전원 입력 경로가 아니다.

따라서 전력 측정 계획도 바뀐다. USB-C 전력 측정기가 아니라 **12V DC 라인용 전력계**가 필요하다(§14.2).

## 8.2 권장 방식

각 보드에 독립적인 12V DC 어댑터를 사용한다.

```text
12V Adapter 1 → KING
12V Adapter 2 → QUEEN
12V Adapter 3 → JACK
```

권장 조건:

- **12V, 2A(24W) 이상**
- 동일 제조사, 동일 모델
- 동일 길이 케이블
- 멀티탭 하나에 세 대를 몰더라도 어댑터는 개별로 둔다

### ⚠️ 전류 용량이 부족하면 고부하에서 보드가 리셋된다

2026-08-10 실측에서 노드별로 안정성 한계가 달랐다.

| 노드 | 안정 한계 | 증상 |
|---|---|---|
| `queen` | 8 스레드 완주 | 정상 |
| `king` | **4 스레드까지만** | 5 스레드 이상에서 하드 리셋 |
| `jack` | 미확정 | 1회 리셋 관측 |

세 보드가 동일 모델이고 소프트웨어도 같으므로, **전원 공급 능력 차이**가 유력하다. 상세는 `board-worklog.md` §2.17.

CPU 8코어와 NPU 2코어를 동시에 최대로 쓰면 순간 전류가 크게 오른다. 어댑터 용량이 부족하면 전압이 떨어지고 PMIC가 리셋한다. 커널 로그에 아무것도 남지 않는 것이 이 경우의 특징이다.

**노드마다 안정 한계가 다르면 "동일한 3대"라는 실험 전제가 깨진다.** 확장 효율 측정 전에 반드시 해결한다.

## 8.2 전력 측정

에너지 효율을 논문 또는 발표에 포함하려면 노드별 전력 측정이 필요하다.

권장 방식:

- USB-C 전력 측정기 3개
- 또는 동일 조건에서 한 대씩 반복 측정
- 대기 전력과 추론 부하 전력 분리
- 스위치와 Scheduler 전력은 별도 기록

측정 지표:

```text
Idle Watt
Peak Watt
Average Watt
Requests per Watt-hour
FPS per Watt
```

---

# 9. 냉각 구성

## 9.1 냉각 조건을 두 가지로 측정한다 (2026-08-10 결정)

**팬리스와 능동 냉각 두 조건에서 각각 측정한다.**

```text
조건 A  팬리스        출고 상태. throttling 발생
조건 B  능동 냉각      동일 팬 3개. throttling 억제
```

### 근거

초안은 팬리스만 측정하기로 했으나, 2026-08-10 지속 부하 시험에서 다음이 관측되었다.

| 조건 | 8스레드 처리량 |
|---|---:|
| 순간 부하 (20회 반복) | 77.3 inf/s |
| 지속 부하 (3,000회 반복) | 69.7 inf/s ⚠️ `ondemand` 기준. 현재값은 `RESULTS.md` §2.2 |

**약 10% 저하.** 그리고 `king`이 NPU 91.3°C로 `disable_temperature_c`(90°C)를 초과했다.

즉 냉각 조건이 처리량과 노드 가용성 양쪽에 직접 영향을 준다. 한 조건만 측정하면 다음을 답할 수 없다.

- 팬리스만 측정 → "냉각하면 얼마나 나아지는가"를 모른다
- 냉각만 측정 → "실제 엣지 배치에서 얼마나 나오는가"를 모른다

**두 조건을 모두 측정하면 "냉각이 확장 효율에 미치는 영향"이 결과가 된다.** 이는 벤더 스펙시트에 없는 수치이며, 측정으로 밝힌다는 본 프로젝트의 정체성과 일치한다.

### 조건 A: 팬리스

출고 상태 그대로 사용한다. thermal throttling은 제거 대상이 아니라 **측정 대상**이다.

### 조건 B: 능동 냉각

세 노드에 **동일 모델 팬 3개**를 동일한 방식으로 장착한다.

- 동일 제조사, 동일 모델, 동일 회전수
- 동일한 거리와 각도
- 팬 소비전력을 별도 기록 (전력 효율 계산 시 분리)

**실제 설치 (2026-08-20):** 120mm 급 5V USB 팬 3개, 노드마다 1개를 보드 위에
얹었다 — **팬이 보드(NanoPi R76S)보다 크다.** 라벨 K/Q/J, 전원은 USB 허브.
보드가 팬 그릴 바로 아래에 놓여 상판 전체로 바람을 받는다.

> ⚠️ **2026-08-20 의 모든 측정(예비·S2)은 이 조건 B(능동 냉각)에서 수행됐다.**
> 처음엔 "팬리스(S0-A)"로 잘못 기록했다가 정정했다. 이 대형 팬에서는
> throttling 이 사실상 억제되므로, 팬리스(조건 A) sustained 157 을 그대로
> 이 조건의 노드 상한 비교 기준으로 쓰면 안 된다 —
> `results/scaling-20260820/README.md` §4.2 의 27% caveat 참조.
> **조건 A vs 조건 B 를 같은 gRPC 경로에서 나란히 재는 것이 §9.1 의 목적**이며,
> 아직 조건 A(팬리스) 쪽 클러스터 측정이 없다.

### ⛔ 책상 선풍기는 사용하지 않는다

2026-08-10 진단 중 책상 선풍기로 냉각한 사례가 있다. **진단에는 유효했으나 측정 조건으로는 사용할 수 없다.**

- 세 보드에 바람이 균등하게 도달하지 않는다
- "선풍기를 이 각도로 틀었다"는 재현이 불가능하다
- 조건 B의 요구사항(동일 팬, 동일 조건)을 만족하지 않는다

### 두 조건에 공통으로 적용되는 것

- 동일한 케이스 또는 전부 케이스 없음
- 동일한 방향과 간격으로 배치
- 동일한 주변 온도
- 보드 사이 최소 10cm 간격 (인접 보드의 열이 서로 영향을 주지 않도록)

```text
[KING]  ←10cm→  [QUEEN]  ←10cm→  [JACK]
        동일 주변 온도, 동일 배치 방향, 동일 냉각
```

주변 온도는 매 실험마다 기록한다. 계절과 실내 냉방에 따라 달라지므로, 이 값이 없으면 다른 날의 결과와 비교할 수 없다.

### ⚠️ 배치 균일화가 선행되어야 한다

2026-08-10 측정에서 동일 부하인데 **`king`이 다른 두 대보다 19°C 높았다**(NPU 91.3 vs 70.2 / 72.1°C).

선풍기를 틀자 세 대가 56~62°C로 수렴했으므로 **개체 불량이 아니라 공기 흐름 문제**로 확인되었다.

**두 조건 중 어느 쪽을 측정하든, 배치를 균일하게 맞추기 전에는 유효한 데이터가 나오지 않는다.** 노드별 온도 편차는 확장 효율 측정을 직접 오염시킨다. 상세는 `board-worklog.md` §2.19 참조.

## 9.2 온도 임계치는 보호 장치이지 측정 도구가 아니다

스케줄러의 `degraded_temperature_c`(80°C)와 `disable_temperature_c`(90°C)는 **하드웨어 보호**를 위한 것이다.

팬리스 환경에서 이 값을 그대로 두면 다음 문제가 생긴다.

```text
300초 지속 부하 → 세 노드 모두 90°C 초과 → 전부 스케줄링 제외
→ NPF-1201 NO_AVAILABLE_NODE → 벤치마크 중단
```

이 경우 측정되는 것은 하드웨어 성능이 아니라 **스케줄러의 온도 정책**이다.

따라서 순서를 다음과 같이 한다.

1. **S0 열 특성 파악**(`01-TECHSPEC.md` §20.2)을 먼저 수행해 정상 상태 온도를 확인한다.
2. 그 결과를 근거로 임계치를 설정한다. 정상 상태 온도보다 충분히 높아야 한다.
3. 확정한 임계치를 `environment-matrix.md` §10에 기록한다.
4. 이후 모든 공식 벤치마크는 같은 임계치를 사용한다.

온도로 인한 노드 제외가 실제로 발생하면 그것도 결과로 기록한다. 다만 **확장 효율 측정과는 분리해서 보고한다.** 두 가지가 섞이면 어느 쪽 원인인지 설명할 수 없다.

## 9.3 벤치마크 온도 조건

```text
시작 온도: S0에서 확인한 idle 온도 + 5°C 이내
Warmup: 30초
측정: 300초
반복 횟수: 5회
반복 사이 cooldown: 최대 180초 또는 시작 온도 도달 시점 중 빠른 쪽
```

팬리스라 냉각이 느리므로 cooldown에 **상한을 둔다.** 상한에 걸린 경우 그 사실과 실제 시작 온도를 결과에 기록한다. 무한정 기다리면 총 16시간 예산(§20.4)이 무너진다.

## 9.4 필수 기록 항목

노드별로 다음을 결과와 함께 저장한다.

```text
주변 온도
시작 온도
최고 온도
정상 상태 온도
throttling 시작 시점 (초)
CPU 주파수 변화
NPU 주파수 변화 (조회 가능한 경우)
온도로 인한 스케줄링 제외 발생 여부와 횟수
```

온도 조건이 다르면 특정 노드의 thermal throttling이 스케줄러 또는 네트워크 문제처럼 나타난다. 이것이 이 프로젝트에서 온도 기록을 선택 사항으로 두지 않는 이유다.

---

# 10. 시간 동기화

Scheduler와 세 노드 모두 `chrony`를 사용한다.

```bash
sudo systemctl enable --now chrony

chronyc tracking
chronyc sources
```

## 10.1 시간 측정 원칙

서로 다른 장치의 monotonic clock 값을 직접 비교하지 않는다.

Scheduler:

- End-to-end latency
- Scheduler queue time
- Routing time
- Node RPC round-trip time

Node:

- Local queue time
- Decode time
- Preprocess time
- NPU input preparation time
- Inference time
- Postprocess time

노드는 각 단계의 duration을 응답에 포함한다.

NTP 또는 chrony는 구조화 로그의 사건 순서를 맞추는 용도로 사용한다.

---

# 11. 프로세스 실행

## 11.1 NanoPi Worker

각 NanoPi에는 `npuforge-node`만 실행한다.

```text
systemd
└── npuforge-node.service
```

서비스 예시:

```ini
[Unit]
Description=NPUForge Node Agent
After=network-online.target
Wants=network-online.target

[Service]
User=npuforge
Group=npuforge
ExecStart=/opt/npuforge/bin/npuforge-node \
    --config /etc/npuforge/node.toml
Restart=always
RestartSec=2
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

## 11.2 Scheduler PC

Scheduler PC에는 다음 프로세스를 실행한다.

```text
npuforge-scheduler
npuforge-dashboard
Prometheus
npuforge-bench
```

선택적으로 Grafana를 추가할 수 있다.

---

# 12. 벤치마크 구성

## 12.1 1노드

```text
활성: KING
비활성: QUEEN, JACK
```

## 12.2 2노드

```text
활성: KING, QUEEN
비활성: JACK
```

## 12.3 3노드

```text
활성: KING, QUEEN, JACK
```

공식 실험에서는 프로세스나 전원을 끄기보다 Scheduler의 drain 또는 disable 기능을 사용한다.

이렇게 해야 네트워크, 전원, 온도 및 장비 배치 조건을 유지할 수 있다.

## 12.4 기본 부하 조건

```text
동시성: 1, 4, 16, 64
Warmup: 30초
측정: 300초
Cooldown: 60초 또는 시작 온도 이하 도달 시점까지
반복: 5회
```

시나리오별 축과 총 측정시간 예산은 `01-TECHSPEC.md` §20.2 및 §20.4를 따른다.

## 12.5 측정 지표

- Requests/sec
- FPS
- p50 latency
- p95 latency
- p99 latency
- 오류율
- 재시도율
- CPU 사용률
- 메모리 사용률
- NPU 사용률
- 네트워크 사용량
- 노드 온도
- 전력 사용량
- 확장 배율
- 확장 효율

---

# 13. 발표 데모용 물리 구성

```text
┌──────────────┐
│ 노트북       │ ← Dashboard
└──────┬───────┘
       │
┌──────▼───────┐
│ 2.5G Switch  │
└─┬────┬────┬──┘
  │    │    │
┌─▼─┐┌─▼─┐┌─▼─┐
│01 ││02 ││03 │
└───┘└───┘└───┘
```

각 노드에는 번호 라벨을 부착한다.

선택적으로 외부 상태 LED를 사용할 수 있다.

- 녹색: Healthy
- 노란색: Busy 또는 Degraded
- 빨간색: Unreachable
- 파란색: Recovering

## 13.1 장애 데모

발표 중에는 전원을 뽑기보다 QUEEN의 네트워크 케이블을 분리한다.

```text
3노드 처리
→ QUEEN 네트워크 분리
→ 헬스체크 실패
→ 자동 제외
→ 2노드로 서비스 지속
→ 케이블 재연결
→ Recovering
→ 자동 재편입
```

네트워크 분리는 전원 차단보다 복구 시간이 짧고 데모 진행이 안정적이다.

## 13.2 발표 장애 대비

- 예비 Ethernet 케이블
- 예비 USB-C 전원 어댑터
- 녹화된 동일 데모 영상
- Mock Backend 모드
- 사전 생성 벤치마크 결과
- 인터넷 없이 동작 가능한 구성

---

# 14. 권장 BOM

## 14.1 보유 현황 (2026-08-06 기준)

| 항목 | 수량 | 상태 |
|---|---:|---|
| NanoPi R76S | 3 | 보유. RAM 사양 확인 필요 |
| 1GbE 스위칭 허브 | 1 | 보유. 관리망 및 중간 단계 추론망으로 사용 |
| CAT6 케이블 | 1 | 보유 |
| Linux PC | 1 | 보유. NIC 속도 확인 필요 |
| USB-TTL UART 어댑터 | 1 | 보유 |

## 14.2 추가 확보 필요

우선순위 순이다.

| 항목 | 수량 | 우선순위 | 비고 |
|---|---:|---|---|
| **동일 모델 팬** | **3** | **최우선** | §9.1 조건 B. 동일 제조사·모델·회전수. 5V USB 팬 권장 |
| **2.5G/10G 스위치** | 1 | **최우선** | 2.5G 포트 ≥ 4 + SFP+ 업링크. ~~2.5GbE 전용 스위치~~ 는 폐기 |
| **스케줄러 서버** | 1 | **최우선** | PCIe 슬롯 필요. `dealer`(노트북)로는 불가 |
| **10G NIC (PCIe SFP+)** | 1 | **최우선** | 예: Intel X520 |
| **SFP+ DAC 케이블** | 1 | **최우선** | 서버 ↔ 스위치 업링크 |
| Ethernet 케이블 | 6~7 | 높음 | 관리망 3 + 추론망 4. **Cat5e로 충분** |
| USB 전력 측정기 | 3 | 중간 | 보드가 5V 입력이므로 USB 전력계로 가능. FPS/Watt 산출에 필요 |
| 예비 Ethernet 케이블 | 1 | 중간 | 발표 장애 대비 |
| 예비 전원 어댑터 | 1 | 중간 | 발표 장애 대비. **5V 4A** |

### 전원 어댑터는 해결됨 (2026-08-10)

**5V 4A × 3개로 교체 완료.** 구매 목록에서 제외한다.

교체 전 어댑터는 무부하에서도 4.983V로 5V를 유지하지 못해 고부하에서 보드가 하드 리셋되었다. 교체 후 3대 동시 지속 부하에서도 5.05V 이상을 유지한다.

**보드 입력은 5V다.** 커널 디바이스 트리의 `vcc12v_dcin` 표기에 속지 않는다. 실측은 `/sys/class/power_supply/simple-vin/voltage_now`로 확인한다.

### 팬 선정 기준

| 항목 | 기준 |
|---|---|
| 전원 | 5V USB 권장 (보드와 동일 전압, 별도 어댑터 불필요) |
| 수량 | **3개, 동일 모델** |
| 속도 | 고정 회전수 또는 세 대 동일 설정 가능한 것 |
| 소음 | 발표 데모에서 사용하므로 고려 |
| 장착 | 보드에 직접 부착하거나 동일 거리·각도로 거치 |

**속도 조절 기능이 있다면 세 팬을 같은 값으로 고정한다.** 회전수가 다르면 노드별 냉각 조건이 달라져 §9.1의 전제가 깨진다.

### 케이블 등급에 대하여

**CAT6를 추가 구매할 필요는 없다.**

- 1GbE: Cat5/Cat5e로 충분
- 2.5GBASE-T(IEEE 802.3bz): **Cat5e로 100m까지 지원**한다. 이 규격 자체가 기존 Cat5e 배선을 재활용하려고 만들어졌다.

보유 중인 케이블을 그대로 쓰고, 부족한 수량만 채우면 된다. 실제로 구매가 필요한 것은 **2.5G/10G 스위치, 스케줄러 서버, 10G NIC, SFP+ DAC** 네 가지다(§3.3.2).

### 냉각 장비를 구매하지 않는 이유

방열 케이스와 팬은 BOM에서 제외했다. §9.1의 팬리스 유지 결정에 따른 것이며, thermal throttling을 제거 대상이 아니라 측정 대상으로 다루기 때문이다.

---

# 15. 초기 구축 순서

## Step 1. 하드웨어 통일

- 세 보드 RAM 사양 확인
- 동일 저장장치 준비
- 동일 방열판과 팬 설치
- 동일 전원 어댑터 사용

## Step 2. OS 복제

- 기준 노드 한 대 구성
- OS, 커널, 패키지, RKNN Runtime 설치
- 이미지를 나머지 두 노드에 복제
- Hostname과 IP만 변경

## Step 3. 네트워크 검증

```bash
ping 10.20.0.21
ping 10.20.0.22
ping 10.20.0.23

iperf3 -s
iperf3 -c <target-ip>
```

노드별 링크 속도 확인:

```bash
ethtool eth0
```

## Step 4. RKNN 단일 노드 검증

- 동일 모델 실행
- 동일 입력으로 결과 확인
- 반복 추론 안정성 확인
- 추론시간 기록

## Step 5. 세 노드 일치 검증

- 모델 SHA-256 확인
- 바이너리 SHA-256 확인
- Runtime 버전 확인
- 커널 및 NPU 드라이버 확인
- 동일 입력 결과 비교

## Step 6. NPUForge Node 배포

- 전용 사용자 생성
- 바이너리 설치
- 설정 파일 배포
- systemd 등록
- Scheduler 자동 등록 확인

## Step 7. 기준 벤치마크

- 1노드
- 2노드
- 3노드
- Round Robin
- 온도 및 전력 기록

---

# 16. 최종 구성 기준

NPUForge v0.1 공식 하드웨어 구성은 다음과 같이 정의한다.

```text
Worker Node:
  NanoPi R76S × 3
  SoC     : Rockchip RK3576 (4× A72 @2.2GHz + 4× A53 @1.8GHz)
  NPU     : 6 TOPS
  GPU     : Mali-G52 MC3
  Network : 2.5GbE × 2 (관리망 1 + 추론망 1)
  Storage : eMMC (M.2는 SDIO이므로 NVMe 불가)
  Cooling : 팬리스 유지. throttling은 측정 대상
  동일 OS / Kernel / RKNN Runtime / Model / Power Supply

Scheduler:
  별도 Linux PC (2.5GbE NIC 필수)
  NanoPi에서 실행하지 않음

Network:
  관리망 : 기존 네트워크, 1GbE
  추론망 : 2.5GbE Star Topology, 10.20.0.0/24, Static IP, MTU 1500

Storage:
  Worker는 eMMC
  Benchmark 데이터와 결과는 Scheduler에 저장
```

## 16.1 RK3588에서 RK3576으로의 변경 (2026-08-06)

문서 초안은 RK3588 기반 NanoPi R6C를 전제로 작성했으나, 실제 보유 장비는 **RK3576 기반 NanoPi R76S**로 확인되었다.

주요 차이와 영향:

| 항목 | RK3588 (초안 전제) | RK3576 (실제) | 영향 |
|---|---|---|---|
| CPU | A76 + A55 | A72 @2.2 + A53 @1.8 | 전처리·디코딩 성능 낮음. 병목이 NPU가 아닐 가능성 ↑ |
| NPU | 6 TOPS | 6 TOPS | **없음.** 발표 제목 유지 |
| 네트워크 | 2.5G + 1G | **2.5G × 2** | 관리망 분리가 기본 구성이 됨 |
| M.2 | NVMe 가능 | SDIO (Wi-Fi 전용) | NVMe 실험 제외 |
| 냉각 | 팬 전제 | 팬리스 | throttling을 측정 대상으로 전환 |
| RKNN | `target_platform='rk3588'` | `target_platform='rk3576'` | 모델 재변환 필요. `.rknn` 파일은 플랫폼 간 비호환 |

CPU가 약해진 것은 이 프로젝트에서 오히려 다룰 거리가 늘어난 것에 가깝다. 전처리와 JPEG 디코딩은 CPU가 수행하므로, 병목이 NPU가 아니라 CPU 전처리로 나타날 경우 그 자체가 "TOPS 수치가 실제 처리량을 대표하지 못한다"는 본 프로젝트의 주장을 뒷받침하는 결과가 된다.

이 구성은 다음 세 목적을 동시에 만족한다.

- 2026년 11월 FOSS for All Conference 발표 데모
- 재현 가능한 오픈소스 벤치마크
- 박사논문 및 후속 연구용 실험 플랫폼
