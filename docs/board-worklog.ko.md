# NPUDure 보드 작업 로그

*[English](board-worklog.md) — 영문이 정본이다.*

- 문서명: `board-worklog.md`
- 대상: NanoPi R76S × 3 (`king` / `queen` / `jack`)
- 목적: 보드에 가한 모든 변경을 시간순으로 기록한다

---

# 0. 이 문서의 규칙

보드에 실행한 명령과 그 결과를 **시간순으로 append**한다. 기존 항목은 수정하지 않는다.

기록하는 이유는 세 가지다.

1. **재현성.** 보드를 다시 세팅하거나 네 번째 노드를 추가할 때 이 문서만 따라가면 된다.
2. **원인 추적.** 벤치마크 결과가 노드마다 다르게 나올 때, 세 보드에 무엇이 다르게 적용됐는지 여기서 확인한다.
3. **오픈소스 공개.** 외부 사용자가 같은 환경을 만들 수 있어야 한다.

각 항목에 다음을 남긴다.

```text
날짜 / 대상 노드 / 실행한 명령 / 결과 / 판단 근거
```

**되돌릴 수 없는 변경**(패키지 업그레이드, 커널 교체, 파티션 조작)은 실행 전에 별도로 표시하고 승인 여부를 기록한다.

---

# 1. 노드 명칭

물리 보드에 붙인 라벨을 그대로 사용한다.

| 라벨 | hostname | Node ID | 관리망 IP | SSH 별칭 |
|---|---|---|---|---|
| K | `king` | `king` | `192.168.123.12` | `npuforge-k` |
| Q | `queen` | `queen` | `192.168.123.16` | `npuforge-q` |
| J | `jack` | `jack` | `192.168.123.33` | `npuforge-j` |

Scheduler 호스트(개발 PC): `192.168.123.26`

---

# 2. 2026-08-07

## 2.1 SSH 접속 확보

**상황.** 세 보드가 모두 `192.168.123.0/24`로 이동해 PC(`192.168.123.26`)와 같은 대역이 되었다. 세 대 모두 ping 및 tcp/22 응답 확인.

**문제.** `ssh-copy-id`가 3대 모두에서 즉시 실패했다.

```text
Permission denied, please try again.   (호스트당 2회, 3대 모두 즉시)
```

**원인.** 비밀번호가 틀린 것이 아니라 **TTY가 없었다.** SSH는 비밀번호를 stdin이 아니라 제어 터미널(`/dev/tty`)에서 읽는다. 자동화 환경에는 TTY가 없으므로 프롬프트가 뜨지 못하고 즉시 EOF로 실패했다. 호스트당 정확히 2회씩, 3대가 한 번에 끝난 패턴이 근거다.

**조치.** OpenSSH 9.7의 `SSH_ASKPASS_REQUIRE=force`를 사용해 TTY 없이 비밀번호를 전달했다.

```bash
ASKPASS=$(mktemp)
printf '#!/bin/sh\nprintf "%%s\\n" "$NPUFORGE_SUDO_PASS"\n' > "$ASKPASS"; chmod 700 "$ASKPASS"
SSH_ASKPASS="$ASKPASS" SSH_ASKPASS_REQUIRE=force DISPLAY=dummy \
  ssh-copy-id -i ~/.ssh/id_ed25519_npuforge.pub npuforge-k
```

작업 후 헬퍼 파일은 `shred -u`로 삭제했다.

**결과.** 3대 모두 키 인증 성공. 계정은 `pi`.

**PC 측 설정.**

- 전용 키 생성: `~/.ssh/id_ed25519_npuforge` (passphrase 없음, 자동화용)
- `~/.ssh/config`에 `npuforge-k` / `npuforge-q` / `npuforge-j` 별칭 추가

> 이 키는 자동화 전용이며 passphrase가 없다. 외부 공개 저장소나 신뢰할 수 없는 네트워크에 노출되지 않도록 한다.

## 2.2 하드웨어 실측 수집

**명령.** `scripts/collect-node-info.sh`를 3대에 원격 실행.

```bash
for pair in "k:npuforge-k" "q:npuforge-q" "j:npuforge-j"; do
  name="${pair%%:*}"; host="${pair##*:}"
  ssh "$host" 'bash -s' < scripts/collect-node-info.sh > "benchmarks/node-info/${name}.txt"
done
```

**원본.** `benchmarks/node-info/{k,q,j}.txt` (각 66줄)

**확정된 스펙.** 상세는 `environment-matrix.md` §2.1 참조.

```text
보드    FriendlyElec NanoPi R76S / friendlyelec,nanopi-r76s rockchip,rk3576
CPU     8코어 — little 2.016GHz(policy0) + big 2.208GHz(policy4)
RAM     4GB LPDDR4X (3,997,848 kB)
eMMC    64GB (rootfs 50G 여유)
NPU     2코어 (Core0, Core1), 300~950MHz, IOMMU 활성
        RKNPU driver v0.9.8
RKNN    Runtime 2.3.0 (c949ad889d@2024-11-07T11:35:33)
        librknnrt.so SHA-256 3대 동일
OS      Ubuntu 24.04, 커널 6.1.141, glibc 2.39
열센서  6개 (soc / bigcore / little-core / ddr / npu / gpu)
```

**중요.** NPU가 **2코어**다. RK3588은 3코어이므로 RK3588 기준 `core_mask` 예제를 그대로 쓸 수 없다.

`rknn_api.h`의 `rknn_core_mask` enum은 코어 3개까지 정의하지만(`RKNN_NPU_CORE_2`), RK3576에서 실제 사용 가능한 것은 `CORE_0`, `CORE_1`, `CORE_0_1`, `CORE_AUTO`, `CORE_ALL`이다.

## 2.3 NIC 스펙 확인

**배경.** 초기 수집에서 `eth1`이 `speed=1000`으로 나와 1G 포트로 오인할 소지가 있었다.

**명령.**

```bash
sudo apt-get install -y ethtool
sudo ethtool -i eth0 ; sudo ethtool eth0
sudo ethtool -i eth1 ; sudo ethtool eth1
```

**결과. 두 포트 모두 2.5G다.**

| 항목 | eth0 | eth1 |
|---|---|---|
| 드라이버 | `r8125` 9.010.01-NAPI | `r8125` 9.010.01-NAPI |
| PCIe 버스 | `0001:21:00.0` | `0000:01:00.0` |
| Supported link modes | 10/100/1000/**2500** baseT | 10/100/1000/**2500** baseT |
| 현재 링크 | 없음 (down) | 1000Mb/s Full |

`eth1`이 1000Mb/s인 것은 **1G 허브에 연결되어 협상된 결과**이지 포트 성능 한계가 아니다.

두 포트가 서로 다른 PCIe 버스에 있어 대역폭을 공유하지 않는다. 관리망/추론망 분리에 유리하다.

**결정.**

```text
eth1 → 관리망 (현재 1G 허브, 192.168.123.0/24)
eth0 → 추론망 (2.5G 스위치 도입 시, 10.20.0.0/24)
```

`eth0`이 3대 모두 비어 있으므로 추론망 전용으로 그대로 사용한다.

## 2.4 hostname 변경

**변경 전.**

| 노드 | hostname |
|---|---|
| K | `NanoPi-R76S` |
| Q | `NanoPi-R76S` |
| J | `localhost.localdomain` |

K와 Q가 동일해 로그·대시보드에서 구분이 불가능했다.

**명령.**

```bash
sudo hostnamectl set-hostname <king|queen|jack>
sudo sed -i "s/^127\.0\.1\.1.*/127.0.1.1\t<new>/" /etc/hosts
```

**결과.** `king` / `queen` / `jack` 로 변경 완료.

### 부수 발견: jack의 `/etc/hosts`가 비어 있었다

`jack`은 `/etc/hosts`가 **0바이트**였다. hostname이 `localhost.localdomain`이었던 원인이다.

king의 파일을 참조본으로 삼아 동일 내용으로 복원했다.

```text
127.0.0.1	localhost
::1		localhost ip6-localhost ip6-loopback
ff02::1		ip6-allnodes
ff02::2		ip6-allrouters

127.0.1.1	jack
```

**판단.** 세 보드는 **완전한 동일 복제본이 아니다.** `/etc/hosts` 부재와 Ubuntu 패치 레벨 차이(§2.5)가 함께 나타난 것으로 보아, jack은 다른 시점 또는 다른 경로로 세팅되었을 가능성이 있다.

### 작업 중 발견한 스크립트 함정

sudo 비밀번호를 파이프로 넘기는 헬퍼 함수에 파일 내용을 다시 파이프로 넣으면 충돌한다.

```bash
S() { printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" "$@"; }

printf "text\n" | S tee -a /etc/hosts    # 동작하지 않음
```

`sudo -S`가 stdin의 첫 줄을 비밀번호로 소비하므로 뒤따르는 명령은 EOF를 받는다. 파일을 쓸 때는 다음을 사용한다.

```bash
cat > /tmp/file.new <<'EOF'
...
EOF
printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" cp /tmp/file.new /etc/target
```

## 2.5 발견된 노드 간 불일치 (미해결)

세 노드는 "동일 OS 이미지"여야 한다(`02-HARDWARE-SETUP.md` §5.1). 현재 다음이 어긋나 있다.

| # | 항목 | king | queen | jack | 위험 |
|---|---|---|---|---|---|
| 1 | Ubuntu 패치 레벨 | 24.04.**3** | 24.04.4 | 24.04.4 | 라이브러리 차이가 노드별 성능 편차로 나타남 |
| 2 | gcc | `~24.04` | `~24.04.1` | `~24.04.1` | 위와 동일 |
| 3 | 미적용 업데이트 | 374개 | 280개 | 279개 | 위와 동일 |
| 4 | SSH 호스트 키 | 3대 완전 동일 (`<redacted-fingerprint>`) | | | 노드 식별 불가, MITM 탐지 불가 |
| 5 | CPU Governor | `ondemand` | `ondemand` | `ondemand` | 주파수 변동으로 측정 재현성 저하 |

**일치하는 항목** (문제 없음): 커널 6.1.141, glibc 2.39, Python 3.12.3, RKNN Runtime 2.3.0 및 `librknnrt.so` SHA-256, RKNPU driver v0.9.8, NPU 2코어, RAM 4GB, eMMC 64GB.

### ⚠️ 커널 업그레이드 금지

커널 `6.1.141`은 FriendlyElec BSP 커널이며 **RKNPU 드라이버 v0.9.8이 여기에 묶여 있다.**

`apt upgrade`가 커널을 교체하면 NPU가 동작하지 않을 수 있다. 패키지 동기화 시 반드시 커널 관련 패키지를 hold 한다.

```bash
sudo apt-mark hold linux-image-* linux-headers-* linux-modules-*
```

이 작업은 되돌리기 번거로우므로 **승인 후 실행**한다. 현재 미실행.

## 2.6 보드 소프트웨어 현황

| 항목 | 상태 |
|---|---|
| `librknnrt.so` | `/usr/lib/librknnrt.so` (2.3.0) |
| `rknn_api.h` | `/usr/include/rknn_api.h` |
| `rknn_matmul_api.h` | 설치됨 |
| `rknn_custom_op.h` | 설치됨 |
| `rknn_server` | `/usr/bin/rknn_server` (Toolkit2 연결 디버깅용) |
| `.rknn` 모델 파일 | **없음** — 변환 필요 |
| gcc | 13.3.0 |
| rustc | 미설치 (크로스컴파일 사용하므로 정상) |
| ethtool | 2026-08-07 설치 (king만. queen/jack 미설치) |

`rknn_server`가 있으므로 RKNN-Toolkit2의 연결 모드로 PC에서 보드의 NPU를 직접 호출해 모델을 검증할 수 있다.

---

## 2.7 C Wrapper 실기 검증

**배경.** `crates/npuforge-rknn/native/rknn_wrapper.c`는 RKNN API 문서만 보고 작성했고 실장비 검증 전이었다. 파일 상단에 그 사실을 명시해 두었다.

**검증 방법.** 실제 `rknn_api.h`의 시그니처를 추출해 대조한 뒤, 보드에서 직접 컴파일했다.

```bash
scp crates/npuforge-rknn/native/rknn_wrapper.{c,h} npuforge-k:~/npuforge-rknn-test/
ssh npuforge-k 'cd ~/npuforge-rknn-test && gcc -c -Wall -Wextra -O2 rknn_wrapper.c -o rknn_wrapper.o'
```

**결과. 경고 없이 컴파일 성공.** 작성한 시그니처가 실제 헤더와 일치했다.

| 항목 | 확인 결과 |
|---|---|
| `rknn_init(rknn_context*, void*, uint32_t, uint32_t, rknn_init_extend*)` | 일치 |
| `rknn_query(rknn_context, rknn_query_cmd, void*, uint32_t)` | 일치 |
| `rknn_inputs_set(rknn_context, uint32_t, rknn_input[])` | 일치 |
| `rknn_run(rknn_context, rknn_run_extend*)` | 일치 |
| `rknn_outputs_get(rknn_context, uint32_t, rknn_output[], rknn_output_extend*)` | 일치 |
| `rknn_outputs_release(rknn_context, uint32_t, rknn_output[])` | 일치 |
| `rknn_input` 필드 (`index/buf/size/pass_through/type/fmt`) | 일치 |
| `rknn_output` 필드 (`want_float/is_prealloc/index/buf/size`) | 일치 |
| `rknn_sdk_version` (`api_version[256]`, `drv_version[256]`) | 일치 |
| `rknn_context` | `uint64_t` (aarch64) |
| `RKNN_SUCC` | 0 |

**추가 확인.** `rknn_set_core_mask(rknn_context, rknn_core_mask)`가 존재한다. `rknn_core_mask` enum은 코어 3개까지 정의하지만 RK3576은 2코어이므로 `CORE_0`, `CORE_1`, `CORE_0_1`, `CORE_AUTO`, `CORE_ALL`만 유효하다.

**미해결.** `npf_rknn_get_runtime_version()`은 컨텍스트 없이 `rknn_query`를 호출하도록 작성했는데, 이 호출이 실제로 성공하는지는 모델이 있어야 확인 가능하다. 실패한다면 노드 시작 시 임시 컨텍스트를 만들어 조회한 뒤 캐시하는 방식으로 바꾼다.

## 2.8 Thread-safety 테스트 프로그램 작성

**파일.** `crates/npuforge-rknn/native/thread_safety_test.c`

**빌드 확인.**

```bash
gcc -O2 -Wall -Wextra -o thread_safety_test thread_safety_test.c -lrknnrt -lpthread
# 경고 없이 성공, 71,888 bytes
```

**검증 시나리오.**

| # | 구성 | 확인 대상 |
|---|---|---|
| 기준선 | 스레드 1, 전용 context | 단일 스레드 처리량 |
| 1 | 스레드 2, **context 공유** | 동일 context 동시 호출 가능 여부 |
| 2 | 스레드 2, 각자 전용 context, `CORE_AUTO` | 전용 context 병렬 가능 여부 |
| 3 | 스레드 2, 각자 전용 context, `CORE_0` / `CORE_1` 분리 | 명시적 코어 분리 효과 |
| 4 | 스레드 4 (코어 수 2 초과) | 과다 워커의 역효과 |

**판정 기준.**

```text
시나리오 1에서 err > 0        → 동일 context 동시 호출 불가
                                모델당 전용 워커 스레드로 직렬화 필요
시나리오 2가 기준선 대비 ~2배 → 전용 context 로 2-way 병렬 가능, worker_count = 2
시나리오 2가 ~1배             → 런타임 내부 직렬화, worker_count = 1 유지
시나리오 3 > 시나리오 2       → 명시적 코어 분리가 유효
시나리오 4 < 시나리오 2       → 코어 수를 넘는 워커는 역효과
```

**⛔ 실행 보류. 모델 파일이 없다.**

보드에 `.rknn` 파일이 하나도 없다. 프로그램은 준비 완료 상태이며, 모델이 생기는 즉시 실행하면 된다.

```bash
ssh npuforge-k 'cd ~/npuforge-rknn-test && ./thread_safety_test model.rknn 50'
```

### 모델 확보 경로

| 경로 | 가능 여부 | 비고 |
|---|---|---|
| 보드에서 다운로드 | ✗ | `curl`, `wget` 미설치 |
| 보드에서 변환 | ✗ | RKNN-Toolkit2는 x86_64 Linux 전용 |
| **PC WSL2에서 변환** | **✓** | WSL2 Ubuntu 확인됨 (현재 Stopped) |
| `rknn_server` 연결 모드 | ✓ | Toolkit2가 PC에서 보드 NPU를 직접 호출 |

`rknn_server`가 보드에 설치되어 있으므로, Toolkit2 구축 후 PC에서 보드 NPU를 원격 호출해 모델을 즉석 검증할 수 있다.

## 2.9 노드 일치 스크립트 준비 (미실행)

**파일.** `scripts/fix-node-consistency.sh`

기본 동작이 DRY RUN이며 `--apply`를 줘야 실제로 실행된다. `--only`로 단계를 나눠 실행할 수 있다.

| 단계 | `--only` 값 | 내용 | 위험도 |
|---|---|---|---|
| 1 | `kernelhold` | 커널 패키지 `apt-mark hold` | 낮음 |
| 2 | `hostkeys` | SSH 호스트 키 재생성 + PC `known_hosts` 정리 | 낮음 |
| 3 | `packages` | 기본 패키지 설치 (curl, ethtool, iperf3, chrony 등) | 낮음 |
| 4 | `chrony` | 시간 동기화 활성화 | 낮음 |
| 5 | `upgrade` | 패키지 업그레이드 (24.04.3 → 24.04.4) | **높음** |
| 6 | `governor` | CPU Governor → `performance` | 중간 |

**안전 장치.**

- 5단계는 커널 hold 여부를 먼저 확인하고, hold되지 않았으면 중단한다
- 6단계는 발열이 올라가므로 S0 열 특성 측정 후 적용을 권장한다
- DRY RUN으로 3대 접속 및 단계 출력 확인 완료

**권장 실행 순서.**

```bash
./scripts/fix-node-consistency.sh --apply --only kernelhold
./scripts/fix-node-consistency.sh --apply --only hostkeys
./scripts/fix-node-consistency.sh --apply --only packages,chrony
./scripts/fix-node-consistency.sh --apply --only upgrade     # 단독 실행
# (S0 측정 후)
./scripts/fix-node-consistency.sh --apply --only governor
```

업그레이드 후 반드시 확인할 것:

```bash
ssh npuforge-k 'uname -r'                                              # 6.1.141 유지?
ssh npuforge-k 'printf "$NPUFORGE_SUDO_PASS\n" | sudo -S cat /sys/kernel/debug/rknpu/version'  # NPU 살아있나?
ssh npuforge-k 'sha256sum /usr/lib/librknnrt.so'                       # 73993ed4... 유지?
```

---

# 3. 미완료 작업

| # | 작업 | 상태 | 비고 |
|---|---|---|---|
| 1 | RKNN thread-safety 검증 | 진행 예정 | `worker_count` 결정. 모델 파일 필요 |
| 2 | `rknn_wrapper.c`를 실제 헤더로 검증 | 진행 예정 | 미검증 상태로 작성됨 |
| 3 | 노드 간 불일치 해소 (§2.5) | 스크립트 준비 후 승인 대기 | 커널 hold 필수 |
| 4 | SSH 호스트 키 재생성 | 스크립트 준비 후 승인 대기 | |
| 5 | CPU Governor → `performance` | 벤치마크 직전 적용 | |
| 6 | 기본 패키지 설치 | 미실행 | `02-HARDWARE-SETUP.md` §5.2 |
| 7 | 추론망 구성 (`eth0`, 10.20.0.0/24) | 2.5G 스위치 도입 후 | |
| 8 | 모델 변환 환경 구축 | 미실행 | Toolkit2를 Runtime 2.3.0에 맞출 것 |

## 3.1 다음 단계: 모델 확보

thread-safety 검증(1번)이 모델 파일에 막혀 있고, 모델은 다른 모든 실장비 작업의 전제이기도 하다. 따라서 이것이 최우선이다.

```text
PC WSL2 (Ubuntu, 현재 Stopped)
  → rknn-toolkit2==2.3.0 설치        ← Runtime 2.3.0에 맞춤
  → YOLOv8n ONNX 확보
  → rknn.config(target_platform='rk3576')
  → yolov8n.rknn 생성
  → scp 로 3노드 배포 + SHA-256 확인
  → thread_safety_test 실행
  → environment-matrix.md §3.1, §6 기록
```

**주의.** Toolkit2 버전이 Runtime보다 높으면 변환한 모델이 로딩되지 않을 수 있다. `rknn-toolkit2==2.3.0`을 우선 시도한다.

---

## 2.10 Scheduler 호스트 실측 (노트북)

**대상.** Samsung 370E5J 계열 구형 노트북, `192.168.123.14`

**측정 결과.** 상세는 `environment-matrix.md` §4.2.

```text
CPU     Intel i7-4712MQ (Haswell, 4C/8T @2.30GHz)
RAM     3.5GB (가용 1.8GB)          ← 노드(4GB)보다 적음
NIC     RTL8111/8168 (r8169), 1GbE 상한. 2.5G 미지원
USB     Bus 004 = USB 3.0 (5000M, 4포트). 나머지는 USB 2.0
TB      없음
Docker  설치됨
아키텍처 x86_64
```

### 링크 속도 100Mb/s 문제 (해결)

최초 측정에서 `Speed: 100Mb/s`로 협상되어 있었다. 포트는 `1000baseT/Full`을 지원하므로 물리 계층 문제였다.

케이블 교체 후 **1000Mb/s로 정상화**되었다.

**영향 분석.** 방치했다면 JPEG 100KB 기준 약 125 FPS에서 링크가 포화되어, NPU 확장 효율이 아니라 케이블을 측정할 뻔했다. 보드 3대는 처음부터 1000Mb/s였으므로 허브가 아니라 노트북 쪽 케이블이 원인이었다.

**후속 조치.** 매 실험 전 링크 속도를 확인하는 절차를 벤치마크 스크립트에 넣는다.

```bash
ethtool enp3s0 | grep Speed
```

### 판정

| 역할 | 판정 | 근거 |
|---|---|---|
| 모델 변환 | **적합** | x86_64 Linux + Docker |
| 개발용 Scheduler (M2~M5) | **충분** | 링크 속도는 기능 정확성과 무관 |
| 공식 벤치마크 (JPEG) | **조건부 적합** | 실측 FPS 확인 후 판단 |
| 공식 벤치마크 (Raw RGB, S6) | **부적합** | 1GbE 초과 |

**2.5G 어댑터 구매는 보류한다.** 노드당 실제 FPS를 모르는 상태에서는 필요 여부를 판단할 수 없다. S0/S1 측정 후 결정한다.

노드당 40 FPS 가정 시 3노드 120 FPS × 100KB ≈ 96 Mbps로 1GbE에 여유가 있다. 측정으로 판단하는 것이 이 프로젝트의 방식과도 일치한다.

**RAM 3.5GB가 NIC보다 실질적인 제약이다.** 대응은 하드웨어 구매가 아니라 운용 방침으로 한다 — 공식 측정 중에는 Prometheus·Dashboard를 중지하고 `npuforge-bench`가 JSONL 원본만 기록한다.

### 미확인 항목

```bash
cat /etc/os-release      # 프롬프트가 [root@localhost ~]# 형태 — 배포판 확인 필요
uname -r
df -h /                  # Docker 이미지 5~8GB 필요
```

hostname이 `localhost`다. 결과 파일에 측정 호스트가 남아야 하므로 이름을 부여한다(제안: `dealer`).

## 2.11 모델 변환 환경 구축

**결정.** WSL2가 아니라 **노트북(x86_64 Linux)** 에 구축한다. Docker가 이미 설치되어 있고, RKNN-Toolkit2가 요구하는 x86_64 Linux 조건을 만족한다. WSL2를 별도로 세팅할 이유가 없다.

**Docker로 감싸는 이유.** 변환 결과가 호스트 환경에 따라 달라지면 재현성이 깨진다. 이미지가 Python·Toolkit·의존성 버전을 고정하므로 누구 PC에서 돌려도 같은 `.rknn`이 나온다. 오픈소스 공개 시 "이 이미지로 재현하세요"가 가능해진다.

**작성한 파일.**

```text
tools/model-converter/
├── Dockerfile            Ubuntu 22.04 + rknn-toolkit2==2.3.0
├── requirements.txt
├── convert_yolov8n.py    ONNX -> RKNN, 메타데이터 자동 기록
└── README.md             사용법 및 배포 절차
```

**버전 고정.** Toolkit 2.3.0은 보드의 Runtime 2.3.0에 맞춘 값이다. Toolkit이 Runtime보다 높으면 변환된 모델이 로딩되지 않을 수 있다.

**타깃 플랫폼.** `target_platform='rk3576'`으로 고정했다. `rk3588`로 변환한 `.rknn`은 RK3576에서 동작하지 않는다.

**재현성 기록.** `convert_yolov8n.py`는 변환 시 다음을 JSON으로 남긴다.

```text
ONNX SHA-256 / RKNN SHA-256 / calibration manifest SHA-256
calibration 이미지 수 / 양자화 방식 / 변환 옵션 전체
toolkit 버전 / python 버전 / 플랫폼
```

calibration 이미지 목록은 정렬해 고정한다. 순서가 양자화 결과에 영향을 주기 때문이다.

## 2.12 Scheduler 호스트(`dealer`) 접속 및 설정

**대상.** `192.168.123.14`, 계정 `yoo2`

### 배포판 확인: Rocky Linux 9.7

SSH 배너에서 `OpenSSH_8.7` + `gssapi-keyex`가 보여 RHEL 계열임을 먼저 파악했다. 확인 결과 **Rocky Linux 9.7**이다.

```text
PRETTY_NAME  Rocky Linux 9.7 (Blue Onyx)
kernel       5.14.0-611.13.1.el9_7.x86_64
glibc        2.34
패키지 관리자 dnf
Docker       29.2.1 (overlayfs)
디스크       60GB 여유
Swap         3.9GB
```

**앞서 실행한 `sudo apt install ...`은 조용히 실패했었다.** `2>/dev/null`로 오류가 가려졌고, `ethtool`·`lspci`·`dmidecode`가 이미 설치되어 있어 출력은 정상으로 보였다. 이 호스트에서는 `dnf`를 써야 한다.

### 접속 확보 과정에서 겪은 것

**1차 실패.** `printf` 기반 askpass 헬퍼가 비밀번호를 제대로 내보내지 못했다. 헬퍼 출력을 직접 확인해 원인을 좁혔다.

```bash
printf "[%s]\n" "$("$ASKPASS")"    # 실제로 무엇이 나오는지 확인
```

heredoc 방식으로 바꾸니 정상 동작했다.

```sh
#!/bin/sh
cat <<'PW'
<password>
PW
```

**2차 문제 — sudo 불가.** `yoo2`가 `wheel` 그룹에 없었다(`id -nG yoo2` → `yoo2`). Rocky는 기본적으로 사용자를 `wheel`에 넣지 않는다.

**3차 문제 — root SSH 차단.** `PermitRootLogin`이 막혀 있어 root로 직접 붙을 수 없었다.

**해결 — `su` 승격.** `su`는 stdin이 아니라 제어 터미널에서 비밀번호를 읽으므로 `ssh -tt`로 PTY를 할당해야 한다. 그리고 **프롬프트가 뜰 시간을 줘야 한다.**

```bash
# 실패: su 가 읽기 전에 비밀번호가 흘러가 에코됨
printf 'PW\n' | ssh -tt host 'su -c "..."'

# 성공: 지연을 넣는다
( sleep 3; printf 'PW\n'; sleep 2 ) | ssh -tt host 'su -c "..."'
```

이 패턴은 §2.1의 SSH 비밀번호 문제와 같은 원인(TTY 부재)이지만 해법이 다르다. SSH는 `SSH_ASKPASS_REQUIRE=force`로 우회되고, `su`는 PTY 할당이 필요하다.

### 적용한 변경

| 항목 | 변경 |
|---|---|
| hostname | `localhost.localdomain` → **`dealer`** |
| `yoo2` 그룹 | `wheel` 추가 (sudo 가능) |
| `yoo2` 그룹 | `docker` 추가 (sudo 없이 docker 사용) |
| SSH 키 | `id_ed25519_npuforge` 설치 |
| SSH 별칭 | `npuforge-dealer` |

`dealer`는 카드 딜러에서 따왔다. 노드가 `king`/`queen`/`jack`이므로 명칭 체계가 일관된다.

### ⚠️ 호스트와 노드의 배포판이 다르다

| | `dealer` | `king`/`queen`/`jack` |
|---|---|---|
| 배포판 | Rocky Linux 9.7 | Ubuntu 24.04 |
| glibc | 2.34 | 2.39 |
| 패키지 관리자 | `dnf` | `apt` |

**바이너리 배포 방향은 안전하다.** 낮은 glibc(2.34)로 빌드한 바이너리는 높은 glibc(2.39)에서 동작한다. 반대는 성립하지 않는다. 따라서 `dealer`에서 크로스컴파일해 보드로 배포하는 것은 문제없다.

`scripts/fix-node-consistency.sh`는 `apt` 전용이며 노드 대상이므로 그대로 두어도 된다. 호스트까지 다루는 스크립트를 쓸 때는 패키지 관리자를 분기해야 한다.

## 2.13 모델 변환 이미지 빌드

**1차 시도 실패.** Dockerfile이 `validate_rknn.py`를 COPY 하는데 파일이 존재하지 않았다.

```text
ERROR: "/validate_rknn.py": not found
```

**부수 교훈.** 백그라운드 실행 시 `docker build ... | tail -40` 형태로 파이프를 걸면 종료 코드가 `tail`의 것이 되어 **실패가 성공으로 보고된다.** 로그를 파일로 남기고 종료 코드를 따로 확인하도록 바꿨다.

```bash
docker build -t img . > /tmp/build.log 2>&1; echo "EXIT=$?"; tail -25 /tmp/build.log
```

**조치.** `validate_rknn.py`를 작성했다. 변환된 모델의 입출력 shape을 확인하고, `onnxruntime`이 있으면 ONNX 원본과 코사인 유사도를 비교한다. 기본 기준은 0.98이다.

DEV-REQ §2.2의 검증 대상 중 "ONNX 결과 ↔ RKNN Simulator 결과" 비교에 해당한다. 보드 3대 실측 비교는 별도로 수행한다.

### 개선 여지: 이미지 용량

빌드 로그에서 `rknn-toolkit2`가 의존성으로 `torch`를 끌어오고, 그 과정에서 **NVIDIA CUDA 라이브러리를 수백 MB씩 내려받는 것**을 확인했다.

```text
nvidia_cusolver_cu12   124.2 MB
nvidia_cusparse_cu12   196.0 MB
nvidia_nccl_cu12       176.2 MB
...
```

`dealer`에는 GPU가 없으므로 전부 사용되지 않는다. CPU 전용 torch를 먼저 설치하면 수 GB를 줄일 수 있다.

```dockerfile
RUN python3 -m pip install torch --index-url https://download.pytorch.org/whl/cpu \
    && python3 -m pip install "rknn-toolkit2==${RKNN_TOOLKIT_VERSION}"
```

디스크 여유가 51GB 남아 있어 당장 문제는 아니다. 빌드가 완료된 뒤 최적화한다.

## 2.14 YOLOv8n ONNX 확보

### ⚠️ 표준 Ultralytics export는 RKNN에 부적합하다

RKNN용 YOLOv8은 **Rockchip이 수정한 exporter**로 만들어야 한다. 표준 Ultralytics export는 DFL·NMS 후처리가 ONNX 그래프에 포함되는데, 이 연산들이 NPU에 매핑되지 않아 CPU fallback이 대량 발생한다.

수정판은 **decode 이전의 raw 텐서를 출력**하고 후처리를 CPU에서 따로 수행한다.

```text
공식 원본 : 출력 1개 (decode·NMS 포함)
최적화판  : 출력 3그룹
            [1,64,80,80]  박스 좌표
            [1,80,80,80]  80개 클래스별 confidence
            [1,1,80,80]   confidence 합
```

이것이 `environment-matrix.md` §6의 "CPU fallback 연산 목록" 항목과 직결된다. **잘못 export하면 NPU가 아니라 CPU를 측정하게 된다.**

### 확보한 파일

`rknn_model_zoo`가 사전 최적화된 ONNX를 배포한다. 직접 export할 필요가 없어 위험이 줄었다.

```text
출처      airockchip/rknn_model_zoo  examples/yolov8
원본      airockchip/ultralytics_yolov8
경로      ~/npuforge/models/yolov8n.onnx  (dealer)
크기      12,650,184 bytes
SHA-256   0c8716701f471067932b797eeb67c8e5db47c693c2557c881d7679ec12e21bc5
형식      PyTorch 2.0 export
```

**RK3576이 공식 지원 목록에 있다.**

```text
RK3562, RK3566, RK3568, RK3576, RK3588, RV1126B, RV1109, RV1126, RK1808, RK3399PRO
```

### 라이선스

`rknn_model_zoo` 저장소는 Apache-2.0이지만 **모델 자체는 AGPL-3.0**이다(Ultralytics 원본 상속). 저장소 라이선스와 데이터 라이선스는 별개다.

상세와 대응 방침은 `MODEL_LICENSES.md` 참조. 요약하면 모델 파일을 저장소에 포함하지 않고 사용자가 직접 내려받게 한다.

---

## 2.15 모델 변환 성공

### onnx 버전 충돌

첫 변환이 실패했다.

```text
AttributeError: module 'onnx' has no attribute 'mapping'
```

**원인.** `rknn-toolkit2`의 의존성 명세가 onnx 버전을 제한하지 않아 최신 버전(1.22.0)이 설치되었다. `onnx.mapping`은 onnx 1.16에서 제거되었는데 rknn-toolkit2 2.3.0이 이를 사용한다.

**해결.** `onnx==1.14.1`로 고정하니 즉시 변환에 성공했다. Dockerfile에 고정과 검증 단계를 넣었다.

```dockerfile
RUN python3 -m pip install "onnx==1.14.1" \
    && python3 -c "import onnx; assert hasattr(onnx, 'mapping'), 'onnx.mapping 없음'"
```

**함께 적용한 개선.** torch를 CPU 전용 인덱스에서 설치하도록 바꿨다. GPU가 없는 호스트에서 NVIDIA CUDA 라이브러리를 수 GB 받는 낭비를 없앤다.

### FP16 모델 생성

Calibration 데이터가 확정되지 않아 INT8 대신 FP16으로 먼저 변환했다. **양자화 없이도 thread-safety 검증에는 지장이 없다.**

```text
파일      yolov8n-fp16.rknn
크기      9,645,065 bytes
SHA-256   459602ea70479c1ce4fdd7419aa81e10e2f795fe6fe87444f3607f25b7054c0f
```

3노드에 배포하고 SHA-256이 모두 일치함을 확인했다. 3대 모두에서 테스트 프로그램 컴파일도 성공했다.

## 2.16 Thread-safety 검증 — 진행 중

### 예비 관측 (반복 2회)

```text
RKNN api        2.3.0 (c949ad889d@2024-11-07T11:35:33)
RKNN driver     0.9.8
입력/출력 개수  1 / 9              최적화판의 3그룹 × 3
입력 크기       1,228,800 bytes    = 640×640×3, 문서 계산과 일치
FP16 추론시간   78.8 ~ 116.1 ms
```

**시나리오 1(context 공유, 2스레드)에서 오류가 0건이었다.** 반복 2회는 표본이 너무 적어 단정할 수 없으므로 20회로 재측정 중이다.

FP16이 약 100ms이므로 노드당 10 FPS 수준이다. INT8은 통상 3~5배 빠르므로 30~50 FPS를 예상한다. **이 수치가 2.5G 스위치 구매 판단의 근거가 된다.**

### 실행 과정에서 겪은 함정 두 가지

**1. 파이프의 `head`가 출력을 삼킨다.**

```bash
ssh host './test model 30' | grep -v ... | head -70    # 출력 0바이트
```

`head`가 조기에 파이프를 닫아 SIGPIPE가 발생하고 원격 명령이 중단되었다. 백그라운드 작업은 exit 0으로 보고되어 성공처럼 보였다.

**2. 파일 리다이렉트 시 블록 버퍼링 + SIGHUP.**

```bash
ssh host './test model 50 > run50.log 2>&1'
```

stdout이 파일이면 libc가 라인 버퍼링 대신 **블록 버퍼링**을 쓴다. SSH 세션이 끊기며 프로세스가 SIGHUP으로 종료되었고, 버퍼에 있던 출력이 통째로 유실되었다. 로그에는 stderr로 나간 한 줄만 남았다(stderr는 항상 무버퍼).

**해결.** 세션에서 분리하고 라인 버퍼링을 강제한다.

```bash
nohup bash -c 'stdbuf -oL -eL ./test model 20 > run20.log 2>&1; echo DONE=$? > done.marker' &
```

완료 마커 파일을 폴링해 결과를 가져온다. **장시간 실행되는 벤치마크에도 같은 패턴이 필요하다** — `run-benchmark.sh`의 무인 실행 요구사항(`01-TECHSPEC.md` §20.4)에 반영한다.

---

# 2.17 ⚠️ 고부하에서 보드가 재부팅된다 (미해결)

## 증상

스레드 수 스윕(3~8스레드)을 실행하면 **`king`과 `jack`이 재부팅된다.** `queen`은 동일 테스트를 완주한다.

| 노드 | 부팅 횟수 | uptime (2026-08-10 02:00) | 스윕 결과 |
|---|---:|---|---|
| `king` | **13** | 15분 | **재부팅 3회** (01:26, 01:38, 01:45) |
| `queen` | 5 | **3일 17시간** | **완주** |
| `jack` | 5 | 26분 | 재부팅 |

`king`은 다른 두 대보다 부팅 횟수가 8회 많다. 모두 오늘 스윕을 돌린 시점과 일치한다.

## 하드 리셋이다

재부팅 직전 로그에 **종료 시퀀스가 전혀 없다.** SSH 세션이 열린 직후 로그가 그냥 끊긴다.

```text
Aug 10 01:45:45 king sshd[1586]: Accepted publickey for pi ...
Aug 10 01:45:45 king systemd-logind[488]: New session 4 of user pi.
(로그 끝 — 커널 패닉도, 종료 메시지도 없음)
```

kernel panic, OOM killer, thermal shutdown 메시지가 없다. **전원 차단 또는 watchdog에 의한 하드 리셋**으로 보인다.

## 원인 후보

| 후보 | 근거 | 판정 |
|---|---|---|
| **전원 공급 부족** | 하드 리셋, 로그 없음, 노드별 편차 | **유력** |
| 발열 | 재부팅 시점 온도 45~50°C | **배제** (임계치와 거리 멂) |
| 메모리 부족 | 가용 3.2GB, OOM 로그 없음 | 배제 |
| 보드 개체 불량 | `queen`만 정상 | 가능 |

전원이 유력한 이유는 **부하 특성**과 맞기 때문이다. 8스레드는 CPU 8코어와 NPU 2코어를 동시에 최대로 쓴다. 순간 전류가 어댑터 용량을 넘으면 전압이 떨어지고 보드가 리셋된다. 로그가 남지 않는 것도 이와 일치한다.

`queen`이 3일 17시간 무중단으로 같은 테스트를 완주했다는 점이 **소프트웨어 문제가 아니라 개체별 하드웨어 조건 차이**임을 시사한다.

## 문서의 전원 가정을 정정해야 한다

`02-HARDWARE-SETUP.md` §8은 **USB-C PD 어댑터**를 전제하는데, 커널 로그의 레귤레이터 이름은 다음과 같다.

```text
vcc12v_dcin      12V DC 입력
vcc_sys
rk806-regulator
```

**실제 전원 입력 방식을 확인해야 한다.** 12V DC라면 USB-C PD 전제로 쓴 §8 전체가 틀렸다.

## 프로젝트에 미치는 영향 — 심각하다

공식 벤치마크는 **300초 지속 부하 × 5회 반복 × 143 run, 총 22시간**이다(`01-TECHSPEC.md` §20.4).

지금 상태로는:

- 측정 중 노드가 재부팅되어 run이 무효가 된다
- 재부팅을 "노드 장애"로 기록하면 **소프트웨어 장애 감지 성능을 잘못 측정**하게 된다
- S4 장애 복구 실험에서 의도한 장애와 전원 문제를 구분할 수 없다
- 무인 야간 실행이 불가능하다

**S0 열 특성 측정 전에 반드시 해결해야 한다.**

## 조치 계획

1. **세 노드의 전원 어댑터 확인** — 제조사, 모델, 정격 출력. 물리 확인 필요
2. 입력 방식 확인 — USB-C PD인지 12V DC 배럴잭인지
3. `queen`의 어댑터를 `king`에 물려 재현 시도 — 어댑터 문제인지 보드 문제인지 판별
4. 동일 모델 어댑터 3개로 통일 (`infrastructure.md` §5 구매 목록)
5. 해결 후 스윕 재실행으로 3대 일관성 확인

**해결 전까지 고부하 테스트를 반복하지 않는다.** 재부팅을 반복시켜 eMMC 손상 위험을 키울 이유가 없다.

## 유효한 데이터는 남아 있다

`queen`이 전체 스윕을 완주했으므로 **thread-safety 결론(§3.1)은 유효하다.** 다만 3대 재현성 확인은 전원 문제 해결 후로 미룬다.

## 정정: 두 개의 서로 다른 현상이었다

절대 시각으로 부팅 이력을 다시 확인한 결과, 위 분석을 정정한다. **uptime만 비교해 하나의 원인으로 묶은 것이 성급했다.**

### 사건 A — 부하 중 개별 재부팅 (조사 대상)

```text
01:26:16  king  재부팅
01:34:40  jack  재부팅
01:38:12  king  재부팅
01:45:58  king  재부팅
```

모두 스윕 테스트 실행 시각과 일치한다. 이 구간 내내 **`queen`은 3일 17시간 무중단이었다.**

부하와 상관관계가 있고 노드별로 다르게 나타나므로, 전원 공급 또는 개체 편차 가설은 **이 사건에 대해서는 유효하다.**

### 사건 B — 3대 동시 재부팅 (부하와 무관)

```text
king   이전 부팅 종료  02:01:00
queen  이전 부팅 종료  02:05:20
jack   이전 부팅 종료  02:05:10
       ↓ 약 27분 정전
세 대 모두 약 02:32 부팅   (04:19 기준 uptime 1시간 47분으로 동일)
```

세 대가 4분 이내에 함께 내려갔고 **27분간 꺼져 있다가 함께 올라왔다.** 이 시각에는 부하 테스트를 실행하지 않았다.

이는 **공용 전원 차단**(정전, 멀티탭 차단, 물리적 재배치)이며 사건 A와 원인이 다르다. 부하로 인한 리셋은 즉시 재부팅되지 실행이 27분간 멈추지 않는다.

**따라서 "고부하가 3대를 모두 재부팅시킨다"는 앞선 서술은 과했다.** 부하와 연결된 것은 사건 A뿐이다.

### 사건 B의 정체: 전원 재배치 작업

사용자 확인 결과, **세 보드의 전원을 각각 독립 소스로 분리하는 작업**이 있었다. 02:05의 3대 동시 정지와 27분 공백이 이 작업 시간과 일치한다.

**따라서 사건 B는 장애가 아니라 계획된 물리 작업이다.** 이것도 원인 미상 재부팅으로 기록했다면 잘못된 추적이 될 뻔했다.

`02-HARDWARE-SETUP.md` §8.1의 "멀티포트 충전기 하나에 세 대를 몰지 않음" 요구가 이로써 충족되었다.

### 진단에 남은 것

| 사건 | 시각 | 원인 | 상태 |
|---|---|---|---|
| A: king ×3, jack ×1 | 01:26~01:45 | 전원 재배치 **이전** 구성에서 고부하 | **재검증 필요** |
| B: 3대 동시 27분 | 02:01~02:32 | 전원 재배치 작업 | 해결 (장애 아님) |

**사건 A는 재배치 이전에 발생했다.** 전원이 독립 소스로 분리된 지금은 재현되지 않을 수 있다. 동일 조건으로 재검증한다.

### 교훈: uptime 비교만으로 판단하지 않는다

처음에 `uptime`만 보고 "고부하가 3대를 재부팅시킨다"고 결론지었으나, 절대 시각으로 보니 서로 다른 두 사건이었다. 게다가 하나는 장애가 아니라 계획된 작업이었다.

**벤치마크 중 노드 재시작을 기록할 때는 절대 시각과 작업 이력을 함께 남긴다.** 그렇지 않으면 물리 작업을 소프트웨어 장애로 오독하게 된다. 이 문서의 존재 이유가 여기에 있다.

## 2.17.1 원인 확정: `king`의 부트로더 펌웨어가 구버전이다

전원 가설을 두 차례 검증한 끝에 실제 원인을 찾았다.

### 전원이 원인이 아니라는 증거

| 관측 | 함의 |
|---|---|
| 3포트 공유 전원 시절에도 `queen`은 8스레드 완주 | 공유 전원 자체가 문제가 아니다 |
| 개별 전원으로 교체 후에도 `king`은 5스레드에서 리셋 | 어댑터 용량 문제가 아니다 |
| 세 어댑터가 동일 조건 | 개체별 어댑터 차이가 아니다 |

### 펌웨어 비교

```bash
grep -oE 'androidboot\.fwver=[^ ]*' /proc/cmdline
```

| 구성요소 | `king` | `queen` | `jack` |
|---|---|---|---|
| DDR init | **v1.09** | v1.13 | v1.13 |
| SPL | **v1.07** | v1.09 | v1.09 |
| **BL31 (ATF)** | **v1.17** | **v1.24** | **v1.24** |
| BL32 | **v1.05** | v1.10 | v1.10 |
| U-Boot | **2025-07-17** | 2026-07-10 | 2026-07-10 |
| PMIC 초기화 | **`ON:0x20 OFF:0x2`** | `ON:0x40 OFF:0x0` | `ON:0x40 OFF:0x0` |

`queen`과 `jack`은 완전히 일치하고 **`king`만 약 1년 낡았다.**

**BL31은 ARM Trusted Firmware이며 Rockchip 플랫폼에서 DVFS와 전압 조절을 담당한다.** v1.17과 v1.24 사이에 전압 테이블이나 DVFS 로직이 바뀌었다면, 구버전이 고부하 전압을 감당하지 못해 리셋되는 것이 정확히 관측된 증상이다.

DDR 펌웨어 차이(v1.09 vs v1.13)도 메모리 트래픽이 큰 다중 스레드 조건에서 불안정성을 유발할 수 있다.

PMIC 초기화 레지스터가 다른 것은 펌웨어 차이의 결과다.

### 잘못된 진단으로 인한 비용

전원을 의심해 사용자가 어댑터 3개를 전부 교체했으나 원인이 아니었다. 개별 전원 구성 자체는 `02-HARDWARE-SETUP.md` §8.2의 요구사항을 충족하므로 낭비는 아니지만, **진단 방향을 잘못 잡아 시간을 소모했다.**

`queen`이 공유 전원에서도 8스레드를 완주했다는 사실이 이미 전원 가설을 약화시키고 있었는데, 그 신호를 충분히 무겁게 다루지 않았다.

### 문서의 구멍

`environment-matrix.md`에 커널·glibc·RKNN 버전은 있었으나 **부트로더 펌웨어 항목이 없었다.** `collect-node-info.sh`도 수집하지 않았다.

"동일한 3대"를 검증한다면서 **전력 관리를 담당하는 계층을 빠뜨린 것**이다. 두 가지를 모두 보완했다(2026-08-10).

### 이미지 버전이 특정되었다

```text
/etc/rom-version
  king   20251222     2025-12-22 이미지
  queen  20260721     2026-07-21 이미지
  jack   20260721
```

`king`만 7개월 낡은 이미지다. 펌웨어 차이의 출처가 여기다.

`/etc/friendlyelec-release`는 세 대가 동일하다(`BOARD=NanoPi-R76S`, `LINUXFAMILY=nanopi-m5`, `BRANCH=dev`). 구분되는 것은 `rom-version`이므로 **이 값을 노드 일치 검증 항목에 포함한다.**

### 조치: `king` OS 재설치 (2026-08-10 결정)

부트로더만 갱신하는 대신 **OS를 재설치한다.** 근거는 다음과 같다.

- `king`은 OS 패치 레벨도 뒤처져 있다(24.04.3 vs 24.04.4). 재설치로 함께 해결된다
- 진단 과정에서 6회 하드 리셋시켜 파일시스템 상태를 신뢰하기 어렵다
- 부트로더만 갱신하는 절차는 `rkdeveloptool`/`eflasher`가 필요해 오히려 복잡하다

**목표 이미지: `rom-version = 20260721`** (NanoPi-R76S용 Ubuntu 24.04, FriendlyElec 배포본)

재설치 후 `scripts/setup-node.sh`로 자동 세팅한다.

```bash
./scripts/setup-node.sh 192.168.123.12 king npuforge-k
```

이 스크립트가 수행하는 것:

| 단계 | 내용 |
|---|---|
| 1 | SSH 키 설치 (`SSH_ASKPASS_REQUIRE=force` 사용) |
| 2 | `~/.ssh/config` 별칭 등록 |
| 3 | hostname 설정, `/etc/hosts` 정리 |
| 4 | **SSH 호스트 키 재생성** (이미지 복제 시 중복 방지) |
| 5 | **커널 패키지 hold** (RKNPU 드라이버 보호) |
| 6 | 기본 패키지 설치, chrony 활성화 |
| 7 | **기준 노드(`queen`)와 환경 비교** — `rom-version`, `fwver`, 커널, glibc, RKNN 버전 및 해시, NPU 코어 수, RAM |

7단계가 핵심이다. 재설치가 목적을 달성했는지 스크립트가 직접 판정한다.

### 재설치 후 검증 순서

```bash
# 1. 실측 수집
ssh npuforge-k 'bash -s' < scripts/collect-node-info.sh > benchmarks/node-info/king.txt

# 2. 펌웨어 일치 확인 (setup-node.sh 가 자동 비교하지만 재확인)
for h in npuforge-k npuforge-q npuforge-j; do
  ssh $h 'printf "%s %s\n" "$(hostname)" "$(grep -oE "androidboot.fwver=[^ ]*" /proc/cmdline)"'
done

# 3. 안정성 재검증 — 이전에 리셋되던 5~8 스레드 구간
ssh npuforge-k 'cd ~/npuforge-rknn-test && ./thread_safety_test yolov8n-fp16.rknn 20 5 8'
```

3번이 통과하면 `worker_count`를 세 노드에 동일하게 설정할 수 있고, "동일한 3대" 전제가 회복된다.

## 2.17.2 원인 확정: 전원 어댑터 전류 부족 (해결됨, 2026-08-10)

### 결정적 증거: 입력 전압 실측

보드에 입력 전압 센서가 있다는 것을 뒤늦게 발견했다.

```bash
cat /sys/class/power_supply/simple-vin/voltage_now
```

| 시점 | 유휴 전압 |
|---|---|
| **교체 전 어댑터** | **4.983 V** ← 무부하에서도 이미 5V 미만 |
| **5V 4A 어댑터** | **5.26 ~ 5.31 V** |

교체 전 어댑터는 **부하가 없는 상태에서도 5V를 유지하지 못했다.** 고부하에서 더 떨어져 보드의 브라운아웃 임계를 넘은 것이 재부팅의 원인이다.

새 어댑터의 부하 중 전압(`king`, 8스레드까지 984샘플):

```text
최소 5.061 V   평균 5.260 V   최대 5.341 V   변동폭 0.280 V
```

부하가 걸려도 5V 아래로 내려가지 않는다.

### 검증 결과: 3대 모두 8스레드 완주

| 노드 | 8스레드 처리량 | 오류 | 재부팅 |
|---|---:|---:|---|
| `king` | 77.3 inf/s | 0 | **없음** |
| `queen` | 70.2 inf/s | 0 | **없음** |
| `jack` | 78.0 inf/s | 0 | **없음** |

`king`은 4스레드도 통과했다(54.1 inf/s). 이전에는 3스레드에서도 재부팅했다.

### ⚠️ 전압을 12V로 오판했던 기록

커널 로그의 `vcc12v_dcin: 12000 mV`를 실제 입력 전압으로 단정하고 문서에 "12V DC 입력"으로 기록했다. **틀렸다.**

이 이름은 디바이스 트리의 fixed-regulator 선언이며, Rockchip 디바이스 트리가 보드 간 복사되면서 남은 것이다. 실제 입력은 5V다.

**확인했어야 할 것은 선언이 아니라 실측값이었다.**

```text
선언 (디바이스 트리)  vcc12v_dcin: 12000 mV     ← 신뢰 불가
실측 (센서)           simple-vin: 4983000 µV    ← 이것이 사실
```

사용자가 5V 4A 어댑터로 교체하겠다고 했을 때 "5V는 위험하다"고 경고할 뻔했다. 실측을 먼저 확인해 오류를 막았다.

### 진단 과정에서 틀렸던 가설들

| # | 가설 | 결과 | 반증 근거 |
|---|---|---|---|
| 1 | 공용 3포트 전원이 원인 | **틀림** | 공용 전원에서 `queen`은 8스레드 완주 |
| 2 | 부트로더 펌웨어 구버전 | **틀림** | `king` 재설치로 펌웨어 일치시켜도 재부팅. `jack`은 처음부터 같은 펌웨어인데 실패 |
| 3 | 입력 전압이 12V | **틀림** | 실측 4.983V |
| 4 | **어댑터 전류 부족** | **맞음** | 유휴 4.983V → 5.3V 교체 후 3대 모두 8스레드 완주 |

**가설 1이 원인을 절반 맞혔는데도 반증으로 처리했다.** "공용이냐 개별이냐"가 아니라 "용량이 충분한가"가 문제였는데, 구성 방식에 집중하느라 용량을 놓쳤다. 개별 전원으로 교체했을 때 오히려 악화된 것이 그 증거였는데(새 어댑터가 더 약했다), 그때도 전류 용량으로 돌아가지 않고 펌웨어로 방향을 틀었다.

`queen`이 공용 전원에서 8스레드를 완주했다는 사실은 "그 어댑터가 충분했다"는 뜻이지 "전원이 원인이 아니다"라는 뜻이 아니었다.

### 교훈: 실측 센서를 먼저 찾는다

`/sys/class/power_supply/`는 처음 수집한 `collect-node-info.sh`에 포함되지 않았다. 전원을 의심하기 시작한 시점에 이 센서를 찾았다면 **가설 2와 3을 거치지 않고 바로 확정할 수 있었다.**

`collect-node-info.sh`에 입력 전압 항목을 추가했다.

### 지속 부하 검증 (3대 동시, 8스레드)

순간 부하 통과가 지속 부하 통과를 보장하지 않으므로 별도로 확인했다.

**전압 — 문제 없다.**

| 노드 | 최소 전압 |
|---|---|
| `king` | 5.061 V |
| `queen` | 5.157 V |
| `jack` | 5.124 V |

3대를 동시에 최대 부하로 돌려도 5V 아래로 내려가지 않는다. 재부팅도 없다. **전원 문제는 해결되었다.**

**온도 — 새로운 문제가 드러났다.**

| 노드 | 최고 SoC | 최고 NPU |
|---|---:|---:|
| **`king`** | **88.7 °C** | **91.3 °C** ⚠️ |
| `queen` | 70.2 °C | 70.2 °C |
| `jack` | 71.2 °C | 72.1 °C |

`king`이 다른 두 대보다 **약 19°C 높다.** 그리고 **`disable_temperature_c`(90°C)를 초과했다.**

세 보드가 동일 모델·동일 펌웨어·동일 부하이므로 소프트웨어 원인은 배제된다. 후보는 다음과 같다.

- 물리적 배치 차이 (공기 흐름, 벽면 근접, 보드 간 간격)
- 방열 접촉 상태
- 개체 편차

`king`은 다른 두 대보다 약 6분 먼저 부하가 시작되었으나, `queen`/`jack`도 이미 평탄역(70~72°C)에 들어갔으므로 시간 차만으로 19°C를 설명할 수 없다.

**§2.19에서 별도로 다룬다.**

### 남은 확인 사항

- 8스레드에서도 처리량이 꺾이지 않았으므로 `MAX_THREADS`를 늘려 최적점을 다시 찾는다
- 전류 측정 수단이 없다. `voltage_now`만 있고 `current_now`가 없어 소비전력을 계산할 수 없다. FPS/Watt 지표에는 외부 전력계가 필요하다

## 2.19 `king`의 온도가 19°C 높다 (재현되지 않음, 2026-08-11)

지속 부하 시험에서 발견했다. 동일 조건인데 `king`만 NPU 91.3°C에 도달해 스케줄링 제외 임계치를 넘었다.

### 왜 중요한가

**노드별 온도 편차는 확장 효율 측정을 직접 오염시킨다.**

- `king`이 먼저 throttling에 들어가면 처리량이 떨어진다
- 스케줄러는 이를 "느린 노드"로 인식해 부하를 줄인다
- 결과적으로 3노드 확장 효율이 낮게 측정되는데, **원인이 스케줄링이 아니라 물리적 배치**다
- 90°C를 넘으면 스케줄링에서 아예 제외되어 사실상 2노드 실험이 된다

`02-HARDWARE-SETUP.md` §9.1이 "동일한 주변 온도, 동일한 배치 방향, 보드 사이 최소 10cm"를 요구하는 이유가 이것이다.

### 확인해야 할 것

| 항목 | 방법 |
|---|---|
| 물리적 배치 | 세 보드의 간격, 방향, 주변 장애물 확인 |
| 적층 여부 | 겹쳐 놓았다면 분리 |
| 공기 흐름 | 벽면·구석·케이블 뭉치에 막혔는지 |
| 주변 온도 | 각 보드 위치의 실제 온도 (햇빛, 다른 장비 발열) |
| 방열판 접촉 | 케이스 장착 상태 |

배치를 균일하게 맞춘 뒤 동일 시험을 반복해 편차가 사라지는지 확인한다. 그래도 남으면 개체 편차이므로 결과에 명시한다.

### 유휴 온도에는 편차가 없다 (2026-08-11 확인)

부하 종료 19.9시간 뒤 세 보드를 동시에 측정했다.

| 보드 | NPU (유휴) | SoC | load1 | 부하 시 NPU (2026-08-10) |
|---|---|---|---|---|
| `king` | 39.8°C | 40.7°C | 1.34 | 91.3°C |
| `queen` | 36.1°C | 36.1°C | 0.07 | 70.2°C |
| `jack` | 37.0°C | 38.8°C | 0.23 | 72.1°C |

**유휴 편차는 2.8~3.7°C에 불과하다.** 그나마도 측정 시점에 `king`에서
`gnome-control-center` 세션이 돌고 있었고(load 1.34), 나머지 두 대는 사실상
유휴였다. 즉 유휴 상태에서 세 보드는 사실상 같다.

이것이 뜻하는 바:

- 19°C는 **지속 부하에서만 벌어지는 격차**다. 방열 능력 차이(공기 흐름)로
  설명하기에 부합한다 — 유휴 발열량에서는 차이가 드러나지 않고, 발열량이
  커질수록 방열 조건 차이가 온도 차로 증폭된다
- 개체 불량(예: 방열판 접촉 불량)이었다면 유휴에서도 어느 정도 드러났을
  가능성이 높다. 완전히 배제할 수는 없으나 배치 가설이 더 유력하다
- **따라서 재측정은 반드시 부하 조건에서 해야 한다.** 유휴 온도만 보고
  "해결됐다"고 판단하면 안 된다

세 보드 모두 `graphical.target` + `gdm` active로 구성이 동일함도 함께 확인했다.
데스크톱 세션이 한 대에만 떠 있으면 그 자체가 측정 오염원이므로, 벤치마크
직전에 세션 상태를 맞춘다(`preflight-check.sh` 항목).

### 통제된 재측정: 19°C 격차는 재현되지 않는다 (2026-08-11)

전용 부하 도구(`sustained_load_test`)로 세 보드에 **동시에** 8스레드 부하를
15분간 걸었다. 평탄역(부하 후 300초~종료, 보드당 약 557샘플) 요약이다.

| 보드 | NPU 평균 | NPU 최고 | SoC 평균 | 입력전압 최저 | 처리량 |
|---|---|---|---|---|---|
| `king` | 73.0°C | **75.8°C** | 71.2°C | 5.070 V | **80.5 inf/s** |
| `queen` | 67.5°C | 70.2°C | 65.8°C | 5.090 V | 77.7 inf/s |
| `jack` | 72.6°C | 74.8°C | 71.6°C | 5.046 V | 77.8 inf/s |

**최대 편차 5.6°C. 90°C 초과 없음. NPU 클럭 강하 없음**
(928 샘플 전부 950 MHz, 하나도 떨어지지 않았다).

상승 곡선도 세 보드가 나란하다.

```text
 t(s)   king  queen   jack
    0   37.0   35.2   37.0
   60   66.5   61.9   66.5
  120   72.1   65.6   69.3
  300   73.0   67.5   73.0
  600   73.9   67.5   73.0
  880   74.8   68.4   72.1
```

### 이전 측정과 무엇이 달랐나

08-10 측정(`king` 91.3 / `queen` 70.2 / `jack` 72.1)과 직접 비교할 수 없다.
**부하 프로파일이 달랐다.**

| | 2026-08-10 | 2026-08-11 |
|---|---|---|
| 도구 | `thread_safety_test` | `sustained_load_test` |
| 부하 형태 | 1→8 스레드 순차 스윕 | 8스레드 고정 |
| 시작 시각 | `king`이 약 6분 선행 | 동시 |
| 지속 | 스윕 완료까지 | 900초 고정 |

`thread_safety_test` 는 목표 스레드 수에 도달하기 전에 단일/2스레드 기준선을
먼저 돌린다. 즉 `king` 은 다른 두 대가 8스레드에 들어갈 무렵 이미 훨씬 오래
가열된 상태였다. 6분 선행까지 겹치면 격차가 부풀려질 조건이 갖춰진다.

`queen` 의 최고 온도는 두 측정에서 **70.2°C 로 동일**하고 `jack` 은
72.1 → 74.8°C 로 소폭 올랐다. 움직인 것은 `king` 뿐이다(91.3 → 75.8°C).
배치를 바꾸지 않았다는 점을 감안하면, 격차의 상당 부분은 **물리적 배치가
아니라 측정 방법의 문제**였을 가능성이 크다.

물론 배치 요인을 완전히 배제할 수는 없다. 다만 지금 조건에서는

- 어떤 보드도 `degraded_temperature_c`(80°C)에 닿지 않는다
- 어떤 보드도 throttling 되지 않는다
- 처리량 편차가 3.5% 이내다 (80.5 / 77.7 / 77.8 inf/s)

이므로 **벤치마크를 막는 요인이 아니다.** S0 실험을 진행할 수 있다.

`king` 이 가장 뜨거우면서 동시에 가장 빠르다는 점도 일관된다 — 15분간
72,481회로 `queen`(69,928회)보다 3.6% 더 많은 일을 했다. 다만 3.6%의 일량
차이가 5.5°C를 다 설명하지는 못하므로, 작은 방열 조건 차이는 남아 있다고
본다.

### 여기서 얻은 측정 원칙

**부하 프로파일이 다르면 온도를 비교하지 않는다.** 같은 "고부하"라도
도달 경로가 다르면 축적 열량이 다르다. S0 이후 모든 열 비교는
`scripts/run-thermal-comparison.sh` 로 수행한다. 이 스크립트는

- 별칭↔hostname 일치를 먼저 검증하고 (§2.20)
- 세 보드의 바이너리/모델 해시가 같은지 확인하고
- 유휴 기준선을 먼저 재고
- 세 보드에 **동시에** 부하를 걸고
- run 전후 `boot_id` 를 비교해 도중 리셋된 보드를 무효 처리한다

### 임계치 재검토가 필요하다

현재 설정값은 초안 그대로다.

```text
degraded_temperature_c = 80.0
disable_temperature_c  = 90.0
```

팬리스 보드가 정상 동작 중에 70~91°C에 도달한다면 이 값들은 **보호가 아니라 측정 방해**가 된다. S0 결과로 재설정한다(`02-HARDWARE-SETUP.md` §9.2).

RK3576의 실제 임계 온도(Tj max)를 확인해 그보다 충분히 낮게, 그러나 정상 동작 범위보다는 높게 잡아야 한다.

## 2.20 문서에 적힌 `king`의 IP가 틀렸다 (2026-08-11)

`king`을 `192.168.123.22`로 기록해 두었으나 **실제 주소는 `192.168.123.12`** 다.
`.22`는 서브넷 전체 스윕에서 ARP 응답조차 없는 빈 주소였고, 그 결과
"`king`이 죽었다"는 잘못된 결론을 냈다.

### 왜 놓쳤나

`~/.ssh/config` 의 `npuforge-k` 별칭에는 **처음부터 `.12`가 올바르게** 들어
있었다. 틀린 것은 문서와 스크립트에 하드코딩한 IP뿐이다. 별칭을 썼다면
애초에 드러나지 않았을 문제다.

| 위치 | 값 | 상태 |
|---|---|---|
| `~/.ssh/config` `npuforge-k` | `.12` | 정상 |
| `board-worklog.md` §1 표 | `.22` | **오류** |
| `environment-matrix.md` §7 | `.22` | **오류** |
| `infrastructure.md` | `.22` | **오류** |
| `setup-node.sh` 사용 예 | `.22` | **오류** |
| `fix-node-consistency.sh` IP 목록 | `.22` | **오류** |

모두 `.12`로 정정했다.

### 재발 방지

**보드 접속은 IP가 아니라 별칭(`npuforge-k/q/j`)으로 한다.** IP는 DHCP라
바뀔 수 있고, 문서에 박아 두면 반드시 한 곳이 낡는다. 별칭은 한 곳
(`~/.ssh/config`)만 고치면 된다.

`preflight-check.sh` 에 다음을 넣는다.

- 세 별칭이 모두 접속되는가
- 각 별칭이 붙은 호스트의 `hostname` 이 `king/queen/jack` 과 일치하는가

이름이 어긋난 채로 벤치마크를 돌리면 결과가 엉뚱한 노드에 귀속된다.
이번처럼 "노드가 죽었다"로 끝나면 차라리 낫고, 조용히 다른 보드에
붙는 쪽이 훨씬 위험하다.

### 참고: 보드 MAC 은 OUI 가 없다

세 보드 모두 locally administered MAC 을 쓴다(`82:`, `66:`, `26:` — 두 번째
니블이 2/6/A/E). 제조사 OUI 로 보드를 식별할 수 없다는 뜻이라, 네트워크
스캔으로 보드를 찾는 방법은 통하지 않는다.

다만 `addr_assign_type = 0`(permanent) 이므로 **재부팅해도 MAC 은 유지된다.**
DHCP 리스가 흔들릴 이유는 없다. 그래도 IP 고정(정적 할당 또는 DHCP
예약)을 해 두는 편이 안전하다.

## 2.21 원격 백그라운드 실행의 두 가지 함정 (2026-08-11)

`preflight-check.sh` 를 만들면서 검사가 **조용히 작동하지 않는** 것을
발견했다. 부하가 도는데 "남은 부하 없음"으로 통과했다.

두 가지가 겹쳐 있었다.

### 함정 1: `pgrep -f` 는 자기 자신을 센다

`pgrep -f` 는 명령줄 전체를 매칭한다. ssh 가 보내는 래퍼는

```text
bash -c "... pgrep -f \"[s]ustained_load_test|...\" | wc -l"
```

이고, 이 명령줄에 패턴 문자열이 들어 있다. 대괄호 트릭
(`[s]ustained`)은 같은 명령줄에 괄호 없는 형태가 섞이면 무력해진다.

**양방향으로 틀렸다.**

| 상황 | 실제 | pgrep 보고 |
|---|---|---|
| 부하 실행 중 | 1개 | 0 (놓침) |
| 부하 없음 | 0개 | 2 (자기 셸을 셈) |

`/proc/PID/exe` 심볼릭 링크를 읽는 방식으로 바꿨다. 이것은 실제 실행
파일을 가리키므로 셸이 끼어들 여지가 없다.

```bash
n=0
for p in /proc/[0-9]*; do
  case "$(readlink "$p/exe" 2>/dev/null)" in
    *sustained_load_test) n=$((n+1)) ;;
  esac
done
```

### 함정 2: `cd DIR && setsid nohup ... &` 는 뜨지 않는다

같은 조건에서 두 형태를 비교했다.

| 형태 | 결과 |
|---|---|
| `ssh -n H "cd $DIR && setsid nohup ./prog ... &"` | **실행 안 됨** |
| `ssh -n H "setsid nohup $DIR/prog ... &"` | 실행됨 |

`&` 는 `cd && prog` 리스트 전체에 걸린다. ssh 가 명령을 보내고 즉시
끊는데, 백그라운드 서브셸이 `cd` 를 거쳐 `setsid` 에 닿기 전에 세션이
사라지면 죽는다. 절대경로를 쓰면 중간 단계가 없어 경합이 생기지 않는다.

**실패해도 아무 신호가 없다.** 종료 코드는 0 이고 stderr 도 비어 있다.
확인하지 않으면 "부하 없는 상태의 온도"를 15분 동안 측정하게 된다.

`run-thermal-comparison.sh` 는 원래 절대경로 형태를 쓰고 있어서
2026-08-11 열 측정은 영향을 받지 않았다. 다만 **띄운 뒤 실제로 도는지
확인하는 단계**를 추가했다.

### 공통 교훈

두 함정 모두 **실패가 성공처럼 보인다.** discuss.md §10 의 A 유형
(지표가 무엇을 세는지 확인하지 않음)과 같은 계열이다.

검사를 새로 만들면 **일부러 깨뜨려 보고 실제로 잡히는지 확인한다.**
이번에도 그 절차 덕에 발견했다. 통과만 보고 믿었다면 preflight 는
아무것도 걸러내지 못하는 채로 남았을 것이다.

## 2.29 S3 saturation sweep — ceiling 기준도 near-linear (2026-08-20)

각 노드 수의 진짜 처리량 상한을 concurrency sweep 으로 찾았다(S2 는 동일
부하 선형성, S3 는 최대 처리량 — 별개 실험). 45 run, 동결 `1da69d4`.

| Config | Ceiling @ conc | Speedup | Eff |
|---|---|---:|---:|
| 1N | 115.2 @ c32 | 1.00× | 100% |
| 2N | 232.0 @ c24 | 2.01× | 101% |
| 3N | **341.8 @ c32** | **2.97×** | **99%** |

- 곡선: 미포화(왕복 지연) → plateau(노드당 ~10-16 동시) → 과부하 살짝 하락.
  오류 0(큐가 흡수). SD ≤ 2.2.
- **S2(동일부하)와 S3(ceiling) 두 각도에서 near-linear 재확인.**
- 보고서: `docs/experiments/S3_SATURATION.md`, 원본: `results/saturation-20260820/`.

다음: S4 io_uring — payload-transfer(비-추론 지연의 94%) 비용 절감 비교.

## 2.28 gRPC baseline 30회 반복 — 재현 확인, baseline 동결 (2026-08-20)

1차 결과를 "재현된 결과"로 승격시켰다. 코드·설정 동결(bench `254d560`)
상태에서 1N/2N/3N 각 10회, 60초, **조건 순서 rotate**(시간·온도 변동 분산).
`scripts/run-grpc-baseline30.sh`. 원본·집계: `results/baseline-20260820/`.

### 결과

| N | Throughput Mean±SD | Speedup | Eff | p50/p99 ms | Err | Bal |
|---:|---:|---:|---:|---|---:|---:|
| 1 | 112.9 ± 0.5 | 1.00× | 100% | 68.0 / 116.3 | 0% | 0.00 |
| 2 | 229.0 ± 0.9 | 2.03× | 101% | 67.0 / 118.6 | 0% | 0.00 |
| 3 | **338.4 ± 1.1** | **3.00×** | 100% | 67.6 / 123.9 | 0% | 0.00 |

- **첫 측정 337.7 이 338.4 ± 1.1 로 재현.** SD 0.5~1.1 로 극히 작다.
- 30/30 active node 정확, invalid 0, 오류 0%, balance 0%p.
- saturation(115) 기준 3N efficiency 98%, 1N c8 기준 speedup 3.00×.

### TimingBreakdown 도 재현 (30회 p50 평균)

3N: network_to_node 17.11 + network_to_client 17.11 = 34.21 ms
  = non-inference overhead(36.34) 의 **94%**, E2E(58.83) 의 58%.
scheduler_queue/route 는 1N·3N 모두 ~0 — 스케줄러 병목 없음 재확인.
1N·3N 의 network 가 거의 같아(17.7 vs 17.1) 전송 시간은 노드 수 무관.

### 승격된 문장

"한 번 337.7" → **"3-node near-linear scaling 을 30회 반복 실험으로 확인
(338.4 ± 1.1 inf/s, speedup 3.00×, error 0%)."** gRPC baseline 동결.

다음: saturation sweep → (동결 유지) → io_uring 동일 조건 비교.

## 2.27 로컬 팬 baseline 재측정 — 오버헤드 27% → 28.8% 확정 (2026-08-20)

27% 의 기준값 157 이 팬리스(08-11/12)라 냉각 조건이 클러스터(팬)와 달랐다.
같은 팬 조건에서 로컬 sustained 를 다시 쟀다. king 노드를 중지하고 순수
로컬 `sustained_load_test`(gRPC 없음), INT8, governor=performance, 팬 ON.

```text
8스레드(worker 8, 클러스터 동일조건) 60초 × 3:  159.2 / 162.0 / 163.2 → 161.5
16스레드(saturation 확인):                       165.7
```

**확정: 오버헤드 = (161.5 - 115) / 161.5 = 28.8%** (냉각·worker·측정시간 통일).

### 발견 — 27% 는 냉각으로 무너지지 않았다

우려는 "팬이면 로컬이 157 보다 훨씬 높아 오버헤드가 크게 벌어진다"였다.
실제로는 팬 161.5 vs 팬리스 157.2 로 **차이가 작았다.** 이유:

**60초/30초 측정은 throttling 발현 전이다.** CPU throttling 은 300초에
-27% 로 나타난다(§2.24, discuss §12). 짧은 측정 구간에서는 팬이든 팬리스든
초기 처리량이 비슷하므로 냉각 조건의 영향이 작다.

→ **27% 는 냉각 때문에 무효가 아니라 28.8% 로 소폭 조정**됐다. 병목 위치
(페이로드 전송, §8·§2.26)는 애초에 냉각과 무관해 그대로다. **가장 단단한
사실 두 개는 흔들리지 않았다**: (1) 확장 효율 ~98% 선형, (2) non-inference
latency 의 94% 가 페이로드 전송.

**두 측정량을 곱하지 않는다.** throughput loss 28.8%(처리량)와 latency
breakdown 94%(지연 구성비)는 다른 축이다. "28.8% 의 94%" 는 틀린 곱이다.
정확한 표현: 클러스터 단일노드 처리량은 로컬 대비 28.8% 낮았고, 별도
latency breakdown 에서 non-inference latency 의 94% 가 payload-transfer 였다.

### 남은 것 (별도 조건)

- **지속 부하(300초) 오버헤드**: 팬 이득이 커지면 오버헤드가 더 벌어질 수
  있다. throttling 이 로컬(sustained)과 클러스터(노드)에 어떻게 다르게
  걸리는지가 다음 질문. 단, 이건 "짧은 측정 28.8%"와 별도 축이다.
- saturation: 16스레드(165.7) > 8스레드(161.5) 라 worker 8 이 로컬 최대는
  아니다. 클러스터 노드가 worker 8 이라 동일조건 비교는 8스레드가 맞다.

## 2.26 TimingBreakdown 첫 실측 — 오버헤드는 페이로드 전송 (2026-08-20)

bench 를 확장해 응답의 `Timing`(proto) 11단계를 전부 수집하게 했다(기존엔
`inference_us` 하나만). 27% 노드당 오버헤드를 단계로 쪼갠 첫 실측이다.

측정: 3노드 / c24 / 10초 / Active Cooling / gRPC.

```text
단계 (p50 ms)
  scheduler_queue      0.00
  scheduler_route      0.00
  network_to_node     17.16   ┐ 페이로드 전송
  node_queue           0.02   │
  decode/preprocess    0.00   │
  npu_input            0.00   │
  inference (NPU)     22.49   │ ← 실제 추론
  postprocess          0.00   │
  network_to_client   17.16   ┘
  end_to_end          58.99
```

**발견: 노드당 오버헤드의 정체는 페이로드 네트워크 전송이다.**
payload transfer = `network_to_node + network_to_client` = 34.32 ms.
protobuf 직렬화도, 스케줄러 큐(~0)도, 노드 큐(~0)도 아니다. 1.17 MiB
입력·출력을 2.5G 로 실어 나르는 시간이 대부분이다.

**분모를 명확히 구분한다(혼동 방지):**

```text
payload transfer / E2E latency            = 34.32 / 58.99 = 58%
payload transfer / non-inference overhead = 34.32 / 36.50 = 94%
  (non-inference overhead = E2E - inference = 58.99 - 22.49 = 36.50 ms)
```

정확한 표현: **"노드당 오버헤드(=E2E−inference)의 94%가 페이로드 전송"**,
그리고 "E2E 지연의 58%가 페이로드 전송, 38%가 순수 추론".

→ io_uring·zero-copy·JPEG 입력·후처리(NMS)로 응답 축소 가 겨냥할 지점이
**네트워크 전송 경로**임이 실측으로 확정됐다.

### 계측의 한계 (정직하게)

- gRPC **직렬화 시간은 단독 분리 안 됨** — proto `Timing` 에 별도 필드가
  없다. 재려면 계측점 추가 필요. 현재 잔차(~2ms)에 섞여 있다.
- bench↔스케줄러는 **같은 호스트(loopback)** 라 client→scheduler 는 ~0.
  실제 네트워크는 스케줄러↔노드 2.5G 구간뿐이다.
- **냉각 조건 미확정:** 이 분해는 클러스터 내부라 냉각 무관하게 유효하지만,
  "27%" 자체는 팬리스 157 vs 팬 클러스터 115 라 아직 확정 아님(§2.24).
- c24(동시 24) 값이라 `network_*` 는 concurrency 의존. 단일 요청 전송 시간은
  낮은 concurrency 에서 따로 봐야 한다.

작업용 집계표는 `results/NPUForge_Benchmark_Result_Workbook.md` §8 (로컬 전용).

## 2.25 S2 확장성 첫 측정 — 확장 효율 98%, 노드당 오버헤드 발견 (2026-08-20)

model_file 버그 수정 후 preflight 통과 상태에서 1/2/3노드 확장성을 처음
쟀다. **정식 근접(preflight 통과·30초·조건 통제)이나 단일 run·
--with-inference 스킵이라 확정 수치 아님.**

측정: INT8, want_float=0, governor=performance, **Active Cooling(노드마다
전용 팬, 측정 시작부터)**, 스케줄러(.9) 경유 gRPC, round-robin. 노드 축소는
프로세스 중지(jack→queen 순), 사이 cooldown.

> ⚠️ **냉각 조건 정정 (2026-08-20 사후).** 이 세션의 모든 측정은 팬 장착
> 상태였다. 처음엔 "cold/팬리스"로 적었으나 실제로는 시작부터 큰 팬이
> 달려 있었다. 이것이 27% 계산에 영향을 준다 — 아래 결론 참조.

### 노드당 동일 부하 (concurrency = 8 × 노드수)

| 구성 | 처리량 | 분배 |
|---|---:|---|
| 1노드 c8  | 111.6 inf/s | king 100% |
| 2노드 c16 | 228.7 inf/s | 50/50 |
| 3노드 c24 | 337.7 inf/s | 33/33/33 |

오류율 0%, round-robin 이 정확히 균등 분배. 3노드/1노드 = **3.03배**.

### 1노드 concurrency 스윕 — 상한 ~115

| c8 | c16 | c32 |
|---:|---:|---:|
| 111.6 | 114.0 | 115.1 |

concurrency 를 올려도 **~115 inf/s 에서 포화**. 이것이 스케줄러 경유
단일 노드 상한이다.

### 두 가지 결론

**1. 확장 효율 ~98% (거의 선형).** 1노드 포화 115 기준 3노드 337.7 =
2.93배. 데이터 병렬(`adrs/001`)이 성립하고 스케줄러가 3노드 동시에도
병목이 아니다. `adrs/003` 의 단일 스케줄러가 이 규모에서 충분함을 실측.

**2. 클러스터 노드 상한 115 < 로컬 sustained 157 (-27%).** 왕복 p50 69ms
인데 노드 보고 추론은 24~28ms — **40ms+ 가 스케줄러 gRPC 경유 오버헤드**
(직렬화 + 1.17MB 입력·출력 전송 + 큐/라우팅). 확장은 선형인데 노드당
절대 상한이 네트워크·스케줄링에 깎인다.

> 프로젝트 핵심 질문 "6 TOPS 세 대는 정말 18 TOPS 가 되는가" 의 첫 실측 답:
> **클러스터 기준 2.93배(98%).** 병목은 확장이 아니라 노드당 오버헤드다.
> 이 27% 가 어디서 오는지는 `TimingBreakdown` 단계 분해로 다음에 쪼갠다.

### 마이너 이슈

- bench run 파일명이 전부 `-n3` — run_id 의 노드 수가 측정 시점 활성이
  아니라 **초기 ListNodes(등록) 기준**이다. jack/queen 중지 후에도 스케줄러
  등록이 남아 3으로 찍혔다. 실제 노드 수는 결과의 분배로만 확정된다.
  run_id 를 측정 종료 시점 활성 노드로 잡는 것이 옳다.
- 노드 축소를 프로세스 kill 로 했다. drain RPC 가 있으면 진행 중 요청을
  흘려보내고 깨끗이 뺄 수 있다(`adrs/027`). S2 정식에서 검토.

### 정식 S2 에 남은 것

반복 run(분산), 팬 조건(S0-B), --with-inference, concurrency 스윕 전체,
2노드 조합(king+queen vs king+jack), TimingBreakdown 오버헤드 분해.

## 2.24 M3 첫 3노드 클러스터 실동작 (2026-08-20)

인프라·빌드·IP 고정이 끝나 실제 3노드 추론 클러스터를 처음 띄웠다.
스케줄러(server .9) + king/queen/jack, 실 gRPC.

### 배포

- king 에서 노드 빌드(`cargo build --release -p npuforge-node --features rknn`,
  1m37s, 24MB) → 개발 PC 경유로 queen/jack 배포
- 모델: INT8 `model.rknn`(dba155d2) + `model.toml` 3보드, 해시 검증 통과
- 스케줄러: server 에서 `scheduler.example.toml`(policy round-robin), 50051

### 예비 벤치 (정식 아님)

preflight 미실행, Active Cooling(팬 ON), 12초. **조건 통제 전이라 확정
수치로 쓰지 않는다.**

| 동시성 | 처리량 | 노드 추론 p50 | 왕복 p50 | 분배 |
|---:|---:|---:|---:|---|
| 6  | 146.3 inf/s | 14.4 ms | 39.8 ms | 33.3% 균등 |
| 24 | 336.4 inf/s | 22.2 ms | 67.7 ms | 33.3% 균등 |

오류율 0%, round-robin 이 세 노드를 정확히 3등분. 단일 노드 INT8 상한
157 대비 c24 에서 약 2.1배 — **다중 노드 확장이 실제로 일어난다.** 정식
S2 는 preflight + concurrency 스윕 + 지속시간으로 별도.

### 이번에 걸린 버그 3개 (전부 "성공처럼 안 보이는" 실패라 빨리 잡힘)

**1. model.toml `model_file` 상대경로가 로딩 실패로 이어진다 (코드 버그, 미수정)**

`main.rs` 는 `load_spec` 이 만든 절대경로 `PathBuf` 로 sha256 을 검증하는데
(`:77`), 정작 `backend.load_model(&spec)`(`:81`)에는 `spec.model_file`(원본
상대경로 `"model.rknn"`)을 넘긴다. 백엔드가 CWD 기준으로 파일을 찾아
`rknn_init` 전에 read 실패 → `status=-2`(read_file 실패도 rknn_init 실패도
같은 NPF_RKNN_ERR_MODEL_LOAD 라 구분 안 됨). RKNN 은 stderr 를 안 남긴다.
→ **수정 완료 (2026-08-20).** `main.rs` 가 `load_model` 직전에
`spec.model_file` 을 `load_spec` 이 해석한 절대경로로 교체한다. 상대경로
`model.toml` 로 3노드가 정상 로딩·등록되고 벤치 재검증(c24 336 inf/s, 오류
0%)까지 통과했다. real_device 테스트는 spec.model_file 에 절대경로를 직접
넣어서 이 버그를 못 잡았다 — 상대경로 케이스 회귀 테스트가 없다.

**2. 죽은 노드가 NPU 컨텍스트를 안 놓아 재기동이 status=-2 로 실패**

노드를 죽였다 바로 다시 띄우면 rknn_init 이 실패한다. `pkill -9` +
수 초 대기로 확실히 정리해야 뜬다. 노드의 graceful shutdown(ContextPool
drop → rknn_destroy)이 SIGTERM 에서 확실히 도는지 점검 필요.

**3. `pkill -f npuforge-node` 가 자기 셸을 죽였다 — ADR-017 함정 1 재현**

정리 명령의 셸 명령줄에 패턴 문자열이 들어 있어 pkill 이 자신을 죽이고
이후 명령이 조용히 안 돌았다. **배포/정리는 `pkill`(comm, `-f` 없이)로.**
내가 문서에 적어 둔 함정에 그대로 걸렸다.

현재 3노드 + 스케줄러는 실행 유지 중. server 방화벽 50051/8080/9090 은
런타임 규칙(재부팅 시 사라짐).

## 2.23 네트워크 개편 — 10G aggregation 구축·실측 (2026-08-20)

§2.22 에서 대기하던 장비가 들어와 M3 네트워크를 구성했다. **차단 요소가
전부 해소됐다.**

### 도입한 것

| 장비 | 사양 |
|---|---|
| 스위치 | **NEXI NS-S25G10G-N** — 2.5G×4 + 10G×2, 전부 RJ45 |
| 서버 | Xeon E5-2630L ×2 (24T) / 16GB / Rocky 9.4 / x86_64 |
| 서버 NIC | `enp4s0` 10GBASE-T (DAC/SFP+ 아님) |

포트 배선: 1=인터넷(ipTIME), 2=king, 3=queen, 4=jack, 5=개발PC(10G포트지만
NIC 1G), 6=server(10G).

### 겪은 것

1. **보드 IP 가 통째로 바뀌었다.** DHCP 라 `.12/.16/.33` → `.3/.4/.5` 로
   재할당됐고, `~/.ssh/config` 의 별칭이 낡아 세 노드 전부 접속 실패했다.
   `adrs/019-ssh-alias-not-ip.md` 가 경고한 상황 그대로다. config 갱신 +
   `npuforge-server` 별칭 추가로 복구.

2. **서버가 10G IP 를 못 받았다.** 원인은 케이블·스위치가 아니라
   NetworkManager 였다 — `enp4s0` 이 `UP LOWER_UP`(링크 붙음)인데 연결
   프로파일이 없어 DHCP 를 안 돌렸다. `nmcli device connect enp4s0` 로
   즉시 `192.168.123.9` 획득. Rocky 9 에 새 NIC 꽂으면 나오는 전형적 상황.

3. **원격 iperf3 기동이 안 떴다** — `setsid nohup iperf3 ... &` 가 조용히
   실패(`adrs/017` 함정 2). 절대경로 형태로 재기동해 해결.

### 실측

```text
server enp4s0        10000 Mb/s full     ethtool
단일 king→server     2.34 Gbps           iperf3   (2.5G 실효 상한)
3노드 동시 →server   각 1.70, 합 5.11 Gbps  nc     (세 스트림 균등 유지)
```

세 스트림이 균등 유지 → **서버 10G aggregation 이 병목이 아니다.** INT8
3노드 목표 RX 4.60 Gbps 를 여유 있게 수용. 상세 판단은
`adrs/014-10g-aggregation-separate-scheduler.md` 구축 결과 절.

### 정리한 것

측정용 방화벽 런타임 규칙(5201-5210)·임시 리스너·파일은 측정 후 전부
제거했다. 서버 영구 상태는 바꾸지 않았다.

### 남은 것

- **IP static 고정** — server(.9) 완료. 보드 3대는 pi sudo 비번 대기.
  라우터 예약 대신 호스트 static 채택 (`infrastructure.md` §2.3)
- INT8 모델 queen·jack 배포
- server gRPC 방화벽 개방

`dealer`(옛 스케줄러, 노트북 .14)는 응답 없음 — 제거됐다. 역할은 server 로 이관.

### IP 고정 방식 결정 (2026-08-20)

라우터(ipTIME) DHCP 예약이 아니라 **호스트 NetworkManager static** 을 택했다.
라우터가 바뀌어도 설정이 호스트에 남아 측정 재현성이 낫고, 현재 IP 를 그대로
고정하므로 SSH 도 안 끊긴다. server 는 root 라 즉시 적용했고
(`nmcli con mod enp4s0 ipv4.method manual ...`), 보드는 `pi` 계정의 sudo 비번이
있어야 한다. 남는 리스크(DHCP 풀 충돌)는 `infrastructure.md` §2.3.

### 스케줄러 빌드 경로 결정 (2026-08-20)

옛 dealer 는 Rust 가 없어 미정이었다. server 로 확정한다.

- toolchain `stable`, MSRV 1.85. server dnf 의 rust/cargo **1.92** 로 충분
- Windows→Linux 크로스빌드는 링커 문제로 회피. **server 24스레드 네이티브**가
  빠르고 확실하다
- server 에 `rust cargo gcc gcc-c++ protobuf-compiler git` 설치
  (tonic-build 0.12 가 protoc 요구). github 접근 OK, foxden 직접 불가라
  소스는 `git archive` tarball 을 scp 로 전송
- 노드(aarch64)는 종전대로 king 네이티브 빌드. 스케줄러(x86_64)만 server

**함정: protoc 가 Rocky 9 기본 리포에 없다.** `dnf install protobuf-compiler`
가 "No match" 로 실패하고, `dnf install -y a b c ...` 는 하나만 못 찾아도
전체가 실패해 rust 까지 설치 안 됐다. **CRB 리포**를 켜야
(`dnf config-manager --set-enabled crb`) protobuf-compiler 가 잡힌다.

**빌드 검증 완료 (2026-08-20).** `cargo build --release -p npuforge-scheduler
-p npuforge-bench` 성공.

```text
cargo 1.92.0 / rustc 1.92.0 / libprotoc 3.14.0 / gcc 11.5.0
npuforge-scheduler  25 MB
npuforge-bench      19 MB
config 파싱·기동 정상 (--config configs/scheduler.example.toml)
```

스케줄러 빌드 경로의 불확실성이 사라졌다. 실제 배포·기동은 M3 착수 때.

## 2.22 작업 중단 시점 상태 (2026-08-12, 10G 스케줄러 구성 대기)

> **후속: §2.23 (2026-08-20) 에서 이 대기가 해소됐다.** 아래는 중단 시점 기록이다.

M3 실장비 측정은 10G aggregation 구성이 있어야 시작할 수 있다. 그때까지 작업을
멈추므로 재개에 필요한 상태를 남긴다.

### 보드 상태

| 항목 | king | queen | jack |
|---|---|---|---|
| SSH 별칭 | `npuforge-k` | `npuforge-q` | `npuforge-j` |
| IP | 192.168.123.12 | .16 | .33 |
| CPU governor | `performance` (영구화) | 동일 | 동일 |
| 유휴 NPU 온도 | 37.9°C | 37.9°C | 38.8°C |
| 잔존 부하 프로세스 | 없음 | 없음 | 없음 |

세 노드의 커널·`librknnrt.so`·RKNPU 드라이버·모델 해시가 모두 일치한다.
`preflight-check.sh --with-inference` 전항목 통과 상태로 멈췄다.

### 보드에 설치한 것 (원래 없던 것)

| 노드 | 추가 | 이유 |
|---|---|---|
| `king` | Rust 툴체인 (rustup) | `npuforge-node --features rknn` 네이티브 빌드. 크로스 컴파일은 aarch64 sysroot 와 RKNN SDK 를 함께 맞춰야 해 실패 지점이 많다 |
| `king` | `protobuf-compiler` | `npuforge-proto` 빌드 |
| 3노드 | `strace` | syscall 분해 측정 |
| 3노드 | `/etc/systemd/system/npuforge-cpu-governor.service` | governor 영구화 |
| 3노드 | `~/npuforge-rknn-test/` C 도구들 | 측정 도구 |

**`king` 에만 Rust 가 있다.** 환경 일치가 깨진 항목이지만 빌드 전용이고
런타임에 영향이 없다. 바이너리는 한 번 빌드해 세 노드에 배포한다
(모델과 같은 원칙).

### 확정된 수치 (governor=performance 기준)

| 항목 | 값 |
|---|---|
| FP16 8스레드 지속 처리량 | **84.3 inf/s** (지연 94.5 ms) |
| INT8 8스레드 지속 처리량 | **157.2 inf/s** (지연 50.8 ms) |
| INT8 / FP16 배율 | **1.86배** |
| 추론당 커널 ioctl | 76회 (FP16·INT8 동일) |
| 노드 간 열 편차 | 5.6°C, **NPU** throttling 없음 |
| CPU thermal 강등 | A72 2208→816MHz / A53 2016→600MHz (부하 60초 후) |
| 지속 부하 NPU 온도 | 67.5~75.8°C (ondemand 기준, 15분) |
| `want_float=0` 효과 | INT8 +17.3% / FP16 +15.7%, 출력 4분의 1 |

이전 문서의 79.0 / 146.2 inf/s 는 `ondemand` 기준이다. discuss.md §11.

### 재개할 때 먼저 할 일

1. `bash scripts/preflight-check.sh --with-inference`
   - 보드가 재부팅했을 수 있다. governor 는 유지되지만 `boot_id` 는 바뀐다
   - 실패하면 그 항목부터 해소한다. 통과 전에 측정하지 않는다
2. 2.5G/10G 스위치 연결 후 추론망 IP 대역을 정하고 `advertise_address` 갱신
   (스케줄러는 10G SFP+ 업링크. `02-HARDWARE-SETUP.md` §3.3.2)
3. `npuforge-node` 를 `king` 에서 빌드해 세 노드에 배포
4. S2 확장성 실험 설계 재검토 — INT8 노드당 **1.545 Gbps**, 3노드
   **4.636 Gbps**. 출력은 입력의 3.96배라 RX 가 최대 18.4 Gbps 다.
   **10G aggregation 이 필요하다.** `02-HARDWARE-SETUP.md` §3.3.2
   (여기 처음 적은 1.43/4.3 은 Gbps 를 2진 접두로 계산한 오류였다)

### 재개 시 주의할 함정 (이번에 겪은 것)

- 보드 접속은 **IP 가 아니라 별칭**으로 한다 (§2.20)
- 원격 백그라운드 실행은 **절대경로**로 하고 실제로 떴는지 확인한다 (§2.21)
- 프로세스 확인은 `pgrep -f` 가 아니라 `/proc/PID/exe` 로 한다 (§2.21)
- ssh 안에서 heredoc + sudo 중첩은 조용히 실패한다. 파일은 `scp` 로 보낸다
- 열 비교는 **부하 프로파일이 같을 때만** 한다 (§2.19)

## 2.18 RTC가 유지되지 않는다

부팅 이력 조회에서 별개 문제를 발견했다.

```text
queen  현재 부팅 시작 시각  Tue 2025-11-25 18:16:31 UTC
jack   현재 부팅 시작 시각  Tue 2025-11-25 18:16:31 UTC
king   현재 부팅 시작 시각  Fri 2025-07-11 18:52:59 UTC
```

세 노드 모두 **부팅 직후 시스템 시각이 과거의 고정값**이다. RTC 배터리가 없거나 동작하지 않아 전원이 끊기면 시계가 초기화된다. NTP가 동기화되기 전까지 로그 타임스탬프가 틀린다.

### 영향

- 부팅 직후 기록된 로그의 타임스탬프를 신뢰할 수 없다
- 노드 간 이벤트 순서를 맞출 수 없다 (`02-HARDWARE-SETUP.md` §10)
- 벤치마크 결과에 잘못된 시각이 기록될 수 있다

### 조치

`chrony`를 활성화하고, **동기화 완료를 확인한 뒤에 측정을 시작**해야 한다. `scripts/fix-node-consistency.sh`의 `chrony` 단계에 포함되어 있으나 아직 실행하지 않았다.

벤치마크 스크립트는 실행 전 다음을 확인하도록 한다.

```bash
chronyc tracking | grep -E "Leap status|System time"
# Leap status : Normal 이어야 하며 Not synchronised 면 대기
```

각 노드가 자신이 측정한 duration만 응답에 담고 절대 시각을 비교하지 않는 설계(§10.1)라 측정값 자체는 영향받지 않는다. 문제는 **로그 상관 분석**이다.

---

# 3.5 문서 재구성 (2026-08-07)

작업 이력이 길어져 두 문서로 나눴다.

| 문서 | 역할 |
|---|---|
| `board-worklog.md` (이 문서) | **시간순 작업 이력.** append 전용. 왜 그렇게 했는지 |
| `infrastructure.md` | **현재 상태 스냅샷.** 지금 어떤 상태인지 |
| `environment-matrix.md` | **버전·해시 고정.** 재현에 필요한 값 |

"지금 상태가 어떤가"를 알려면 `infrastructure.md`를, "어쩌다 이렇게 됐나"를 알려면 이 문서를 본다.

---

# 4. PC 측 변경 사항

보드가 아니라 개발 PC(`192.168.123.26`)에 적용한 내용이다.

| 날짜 | 항목 | 내용 |
|---|---|---|
| 2026-08-07 | SSH 키 | `~/.ssh/id_ed25519_npuforge` 생성 (passphrase 없음, 자동화용) |
| 2026-08-07 | SSH config | `npuforge-k` / `npuforge-q` / `npuforge-j` 별칭 추가. 기존 config는 `~/.ssh/config.bak.*`로 백업 |

## 4.1 SSH 별칭

```text
npuforge-k → pi@192.168.123.12  (king)
npuforge-q → pi@192.168.123.16  (queen)
npuforge-j → pi@192.168.123.33  (jack)
```

별칭은 `npuforge-k/q/j`로 유지하고 hostname만 `king/queen/jack`으로 두었다. 별칭을 바꾸면 이미 작성한 스크립트를 모두 수정해야 하므로, 추론망 구성 시 한 번에 정리한다.

## 4.2 sudo 실행 패턴

`pi` 계정은 sudo에 비밀번호를 요구한다. 자동화에서는 다음 형태를 사용한다.

```bash
ssh npuforge-k 'printf "$NPUFORGE_SUDO_PASS\n" | sudo -S -p "" <command>'
```

§2.4에 기록한 파이프 충돌 함정에 주의한다.

**개선 여지.** 벤치마크 자동화에서 sudo 호출이 늘어나면 특정 명령에 한해 NOPASSWD sudoers 규칙을 두는 편이 낫다. 다만 이는 권한 확대이므로 별도 승인 후 진행한다.
