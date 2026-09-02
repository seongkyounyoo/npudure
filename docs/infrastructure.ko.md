# NPUDure 인프라 현황

*[English](infrastructure.md) — 영문이 정본이다.*

- 문서명: `infrastructure.md`
- 최종 갱신: 2026-08-20
- 관련 문서: `board-worklog.md` (시간순 작업 이력), `environment-matrix.md` (버전 고정)

이 문서는 **현재 상태의 스냅샷**이다. 어떻게 그 상태에 도달했는지는 `board-worklog.md`를 본다.

> **2026-08-20 대개편.** 2.5G/10G 스위치·10G 서버 도입으로 M3 차단 요소가
> 전부 해소되고 IP·역할이 크게 바뀌었다. 이전 판(dealer 노트북 + 1G 관리망)은
> 폐기됐다. 경위는 `board-worklog.md` §2.23.

---

# 1. 장비 구성

```text
                    ┌──────────────────────────────────┐
                    │ server   192.168.123.9           │
                    │ Rocky Linux 9.4 / x86_64         │
                    │ Core i7-4790 (4C/8T) / 16GB      │
                    │                                  │
                    │ · Scheduler (예정)               │
                    │ · Benchmark Client (예정)        │
                    └────────────────┬─────────────────┘
                                     │ 10GbE (enp1s0)   ← aggregation
                                     │
                    ┌────────────────▼─────────────────┐
                    │ NEXI NS-S25G10G-N                │
                    │ 2.5G x4 + 10G x2 (전부 RJ45)     │
                    └─┬────┬──────┬──────┬──────┬───────┘
              10G ────┘    │2.5G  │2.5G  │2.5G  └──2.5G── 인터넷(ipTIME)
          개발 PC(노트북)  │      │      │
          (1G NIC, 미활용) │      │      │
                    ┌──────▼┐ ┌───▼───┐ ┌▼──────┐
                    │ king  │ │ queen │ │ jack  │
                    │  .3   │ │  .5   │ │  .4   │
                    │ 6 TOPS│ │6 TOPS │ │6 TOPS │
                    └───────┘ └───────┘ └───────┘
                       Ubuntu 24.04 / RK3576 / aarch64
                       각 eth0 2.5G, static
```

| 호스트 | IP | 역할 | OS | 아키텍처 | 스위치 포트 |
|---|---|---|---|---|---|
| `server` | 192.168.123.9 | Scheduler / Bench | Rocky Linux 9.4 | x86_64 | **10G (6)** |
| `king` | 192.168.123.3 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (2) |
| `jack` | 192.168.123.4 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (4) |
| `queen` | 192.168.123.5 | NPU Worker | Ubuntu 24.04 | aarch64 | 2.5G (3) |
| 개발 PC | 192.168.123.26 | 코드 작성 / 원격 조작 | Windows | x86_64 | 10G (5) — **1G NIC** |
| 인터넷 | — | ipTIME 상위 | — | — | 2.5G (1) |

> **IP 고정 완료 (2026-08-20).** 개편 때 보드 IP 가 통째로 바뀌어
> (`.12/.16/.33` → `.3/.4/.5`) SSH 별칭이 낡아 노드를 못 찾았다. 라우터
> DHCP 예약 대신 **각 호스트에서 현재 IP 를 NetworkManager static 으로 고정**
> 했다(측정 재현성상 호스트 설정이 낫다). **4대 전부 `ipv4.method=manual`**
> (§2.3). `adrs/019-ssh-alias-not-ip.md`.

**`dealer`(옛 스케줄러, 노트북 .14) 는 제거됐다.** 응답 없음. 역할(스케줄러·
벤치)은 `server` 로 이관됐다. 모델 변환 Docker 도 dealer 에 있었으므로
변환 환경은 재구축 대상이다 (§6). 다만 모델은 이미 변환 완료라 당장 필요치 않다.

개발 PC 는 스위치 10G 포트(5)에 물려 있으나 **NIC 이 1G(현재 100Mb/s 협상)라
10G 를 못 낸다.** 벤치 클라이언트는 개발 PC 가 아니라 `server` 에서 돌린다.

---

# 2. 접속

## 2.1 SSH 별칭

개발 PC의 `~/.ssh/config`에 등록되어 있다. **IP 는 여기 한 곳에만 둔다**
(`adrs/019-ssh-alias-not-ip.md`).

```text
npuforge-k        → pi@192.168.123.3     (king)
npuforge-q        → pi@192.168.123.5     (queen)
npuforge-j        → pi@192.168.123.4     (jack)
npuforge-server   → root@192.168.123.9   (server)
```

모두 `~/.ssh/id_ed25519_npuforge` 키로 비밀번호 없이 접속된다.

> 이 키는 자동화 전용이며 passphrase가 없다. 공개 저장소나 신뢰할 수 없는 네트워크에 노출하지 않는다.

## 2.2 권한 승격

| 호스트 | 계정 | sudo | 비고 |
|---|---|---|---|
| king / queen / jack | `pi` | `NPUFORGE_SUDO_PASS` 로 전달 | `printf '%s\n' "$NPUFORGE_SUDO_PASS" \| sudo -S -p "" <cmd>` |
| server | `root` | 불필요 (root 직접) | 자동화 키가 root `authorized_keys` 에 등록됨 |

`sudo -S`는 stdin의 첫 줄을 비밀번호로 소비한다. 파일 내용을 파이프로 넘길 수
없으므로 파일을 쓸 때는 임시 파일을 거친다.

### 2.2.1 보드 자격증명은 벤더 기본값 그대로다 — 의도된 선택

보드 계정과 sudo 비밀번호는 **OS 이미지의 벤더 기본값을 바꾸지 않았다.**
숨기지 않고 적어 두는 편이 낫다고 판단했다.

| | |
|---|---|
| 전제 | 보드는 `192.168.123.0/24` 사설 대역, NAT 뒤. 인바운드 포워딩 없음 |
| 기본값 유지 | 벤더 기본값은 **이미 공개된 정보**다. 적어도 새로 알려주는 것이 없다 |
| 바꾸지 않는 이유 | 커스텀 값을 쓰면 **없던 비밀이 하나 생긴다.** 그 값이 문서·이력·사진 어디로든 새면 비밀번호 **패턴**이 노출되고, 그건 이 랩 밖으로 번지는 정보다 |

> **조건이 바뀌면 이 판단도 바뀐다.** 22번 포트를 외부로 포워딩하거나
> 보드를 격리되지 않은 망에 두는 순간 기본값은 즉시 문제가 된다.
> 배제와 마찬가지로 이 결정에도 **어떤 조건에서** 가 붙는다.

```bash
S() { printf '%s\n' "$NPUFORGE_SUDO_PASS" | sudo -S -p "" "$@"; }
cat > /tmp/f.new <<'H'
...
H
S cp /tmp/f.new /etc/target       # printf "text" | S tee ... 는 동작하지 않음
```

원격 실행 함정(백그라운드 기동·프로세스 카운트)은 `adrs/017-remote-exec-pitfalls-library.md`.

## 2.3 IP 고정 (호스트 static)

DHCP 재할당으로 IP 가 바뀌는 것을 막기 위해 **각 호스트에서 현재 IP 를
NetworkManager static 으로 고정**한다. 라우터(ipTIME) DHCP 예약 대신
호스트 설정을 쓰는 이유는 라우터가 바뀌어도 설정이 남아 측정 재현성이 낫기
때문이다. **현재 IP 를 그대로 고정하므로 SSH 세션은 끊기지 않는다.**

공통 파라미터: gateway `192.168.123.254`, DNS `210.94.0.73 210.220.163.82`,
prefix `/24`. 전부 NetworkManager 관리(netplan/networkd 아님).

```bash
# server (root, 연결명 enp1s0) — 완료
nmcli con mod enp1s0 ipv4.method manual \
  ipv4.addresses 192.168.123.9/24 ipv4.gateway 192.168.123.254 \
  ipv4.dns "210.94.0.73 210.220.163.82"
nmcli con up enp1s0

# 보드 (pi/sudo, 연결명 'Wired connection 1', eth0) — 완료 (2026-08-20)
#   king .3 / queen .5 / jack .4. 같은 IP 라 SSH 유지, 외부 도달 확인
```

> ⚠️ **DHCP 풀 충돌 주의.** `.3/.4/.5/.9` 가 ipTIME DHCP 풀 안이면 라우터가
> 그 주소를 다른 기기에 임대할 수 있다(호스트 static 은 라우터가 모른다).
> 완전 회피는 ipTIME 에서 해당 주소를 풀 밖으로 빼는 것(라우터 UI 작업).
> 소규모 홈랜에서 위험은 낮지만 남는 리스크다.

## 2.4 sudo 비번 파일

보드 자동화용 sudo 비번은 개발 PC 로컬 `~/.npuforge/sudo-pass` (chmod 600)
또는 환경변수 `NPUFORGE_SUDO_PASS` 로 전달한다. 저장소에 넣지 않는다.
`preflight-check.sh` 와 배포 스크립트가 이 경로를 읽는다.

---

# 3. 소프트웨어 현황

## 3.1 노드 (king / queen / jack)

| 항목 | 값 | 3대 일치 |
|---|---|---|
| SoC | Rockchip RK3576 | ✓ |
| NPU | 2코어, 300~950MHz, IOMMU 활성 | ✓ |
| RKNN Runtime | 2.3.0 (`librknnrt.so` SHA-256 동일) | ✓ |
| RKNPU Driver | v0.9.8 | ✓ |
| 커널 | 6.1.141 | ✓ |
| glibc | 2.39 | ✓ |
| RAM / eMMC | 4GB / 64GB | ✓ |
| Ubuntu 패치 레벨 | 24.04.4 | ✓ |
| gcc | 13.3.0-6ubuntu2~24.04.1 | ✓ |
| CPU Governor | **`performance`** | ✓ 재부팅 유지 |
| eth0 링크 | **2.5G (2500Mb/s)** | ✓ 2026-08-20 실측 |
| **SSH 호스트 키** | **queen·jack 동일** | ✗ **미해결** |
| `.rknn` 모델 (FP16) | `459602ea…` 3대 배포 완료 | ✓ |
| `.rknn` 모델 (INT8) | `dba155d2…` **`king` 에만** | ✗ 배포 필요 |
| Rust 툴체인 | 1.97.1 | **`king` 에만.** 빌드 전용 |
| 측정 C 도구 | `~/npuforge-rknn-test/` | ✓ 해시 동일 |

**SSH 호스트 키가 queen·jack 에서 같다.** 두 보드를 암호학적으로 구분할 수
없어, IP 가 바뀌면 경고 없이 엉뚱한 보드에 붙는다. DHCP 라 IP 가 실제로
바뀌므로(§1) 위험이 크다. 조치 명령은 `TODO.md` §1.2.

`preflight-check.sh` 가 매 측정 전에 위 항목의 일치를 확인한다.

## 3.2 server (192.168.123.9)

| 항목 | 값 |
|---|---|
| OS | Rocky Linux 9.4 (Blue Onyx), 커널 5.14.0-427.13.1.el9_4 |
| 메인보드 | ASUS H81M-K (H81 칩셋) |
| CPU / RAM | **Core i7-4790 (4C/8T, 3.6~4.0GHz)** / **16GB DDR3-1600 non-ECC** |
| 디스크 | ST2000VN004 2TB, root LVM 70GB (65GB 여유) |
| NIC | `enp1s0` **Intel X550T 10GBASE-T, 10G full 실측** (2026-08-26), 드라이버 `ixgbe` |
| NIC 슬롯 | `PCIEX16_1` (CPU 직결). **PCIe 2.0 x4 로 동작** |
| 슬롯 상한 근거 | 루트 포트 `00:01.0` 의 `LnkCap: Speed 5GT/s, Width x16` — **메인보드 x16 슬롯 자체가 PCIe 2.0 상한**이다. 카드(`LnkCap 8GT/s x4`)가 아니라 슬롯이 정한다. 조치 불가 |
| 병목 여부 | **아니다.** PCIe 2.0 x4 = 방향당 약 16Gbps. 실사용은 3노드 합쳐 방향당 ~4.6Gbps 로 3배 여유 |
| 전체 인벤토리 | 신서버 [`hosts/server-i7-4790-20260826.md`](hosts/server-i7-4790-20260826.md) · 구서버 [`hosts/server-xeon-e5-2630l-20260826.md`](hosts/server-xeon-e5-2630l-20260826.md) |
| 방화벽 | firewalld active, zone `public`. gRPC 포트 개방 필요(측정 전) |
| 빌드 툴체인 | **rust/cargo 1.92, gcc 11.5, protoc 3.14, git** (2026-08-20 설치) |
| Docker | 미설치 — 모델 변환 필요 시 구축 |

> protoc 는 Rocky 9 기본 리포에 없고 **CRB 리포**(`dnf config-manager
> --set-enabled crb`)를 켜야 `protobuf-compiler` 가 잡힌다. tonic-build 0.12
> 가 시스템 protoc 를 요구한다.

**dealer(노트북)의 두 제약이 이 서버로 해소됐다.**

1. **RAM 3GB → 16GB.** 스케줄러 RSS 우려(페이로드 중계)가 크게 완화됐다.
   `environment-matrix.md` §10.1, `adrs/003-central-simple-scheduler.md`
2. **1GbE → 10GbE.** aggregation 대역 확보. §4 에서 실측했다.

**스케줄러(x86_64)는 server 에서 네이티브 빌드한다.** MSRV 1.85 < dnf rust
1.92 라 stable 채널로 빌드된다. 소스는 `git archive` tarball 을 scp 로 넘긴다
(server 는 foxden 직접 접근 불가, github 는 OK). 노드(aarch64)는 종전대로
king 에서 빌드한다. Windows→Linux 크로스빌드는 링커 문제로 쓰지 않는다.

## 3.2.1 서버 교체 (2026-08-26) — 기준선이 −7.5% 낮아졌다

구서버(Xeon E5-2630L ×2, **24 스레드**)가 물리적으로 교체되어 여분의
데스크톱(i7-4790, **8 스레드**)으로 옮겼다. **스케줄러 호스트가 바뀌었을 뿐
노드 3대·스위치·모델·바이너리는 그대로다.**

| | 구서버 (~2026-08-24) | 신서버 (2026-08-26~) |
|---|---|---|
| CPU | Xeon E5-2630L ×2 · 24T · **2.0~2.5GHz** | Core i7-4790 · 8T · 3.6~4.0GHz |
| RAM | 16GB | 16GB DDR3-1600 |
| NIC | **Intel X550T** `enp4s0` | **같은 카드** `enp1s0` (PCIe 2.0 x4) |
| **기준선 처리량** | **~391 inf/s** | **~360 inf/s** (3 run: 360.5 / 362.5 / 357.2) |
| 왕복 p50 | ~86 ms | ~93 ms |
| 노드 편차 | ~1.02× | ~1.07× |
| 오류율 | 0 | 0 |

> **10G NIC 은 같은 물리 카드다.** Intel X550T 가 한 장뿐이라 구서버에
> 꽂아 쓰다가 빼서 신서버에 옮겨 꽂았다. 그래서 **10G 경로의 하드웨어는
> 두 측정에서 동일하다** — NIC 은 통제된 변수이고, 바뀐 것은 호스트
> (CPU · 메인보드 · PCIe 슬롯)뿐이다. 아래 판정이 그만큼 좁혀진다.
>
> **그 카드가 두 호스트에서 어떤 링크로 물렸는지는 2026-08-26 에 구서버를
> 다시 켜서 확인했다** — 카드가 빠진 뒤에도 슬롯 능력은 남는다.
>
> | | 구서버 (R620) | 신서버 (H81M-K) |
> |---|---|---|
> | 슬롯 세대 | **PCIe 3.0** (`LnkCap 8GT/s`) | PCIe 2.0 (`LnkCap 5GT/s`) |
> | X550T 링크 | 8GT/s × x4 | 5GT/s × x4 |
> | 방향당 대역 | **약 32 Gbps** | 약 16 Gbps |
>
> **링크 대역이 절반으로 줄었다. 그래도 병목은 아니다** — 3노드 실사용이
> 방향당 ~4.6Gbps 라 16Gbps 도 3.5배 여유다. 이제 추정이 아니라 측정값이다.
> → [`hosts/server-xeon-e5-2630l-20260826.md`](hosts/server-xeon-e5-2630l-20260826.md)

### 원인 — 스케줄러 호스트가 CPU 로 좁혀졌다

측정 중 서버 CPU 사용률이 **82.2%** (8 스레드 합)다.

```text
스케줄러        45.3%  ≈ 3.6 코어
기타(벤치+커널)  36.9%  ≈ 2.9 코어
────────────────────────────────
합계            82.2%
```

**벤치 클라이언트가 스케줄러와 같은 호스트에서 돈다.** 구서버에서는 같은
일이 24 스레드의 ~27% 였고, 신서버에서는 8 스레드의 82% 다.

손실 위치가 이를 뒷받침한다. 노드 쪽은 변화가 없고(NPU 추론 p50 28.35ms,
분배 33.3% 균등, 온도 53~57°C), `scheduler_queue` 0.00ms ·
`scheduler_route` 0.01ms 로 스케줄러 내부 큐도 비어 있다. 늘어난 시간은
전부 전송 구간(`network_to_node` / `network_to_client` 각 p50 24.2ms)에
있다 — 애플리케이션 큐가 아니라 **호스트의 CPU 경합**이다.

> **PCIe 강등은 원인이 아니다.** `LnkSta 5GT/s x4` 는 방향당 16Gbps 로,
> 실사용(~4.6Gbps)의 3배 여유가 있다. H81M-K 의 x16 슬롯이 PCIe 2.0 이라
> 생기는 하드웨어 한계이며 조치할 수 없고, 조치할 필요도 없다.

> **원본 데이터.** 위 3 run 의 bench JSON 은
> [`../results/baseline-20260826-althost/`](../results/baseline-20260826-althost/)
> 에 있다. `-althost` 접미사 때문에 `count-runs.sh` 가 421 에 합산하지 않고
> 따로 센다.

### 기존 측정 결과에 미치는 영향 — 없다

**측정 421건은 전부 구서버에서 얻은 것이고, 그 값은 그대로 유효하다.**
숫자를 소급해 고치지 않는다. 신서버 값은 "다른 스케줄러 호스트에서의
재현치" 로 여기 따로 적는다.

다만 S3.9a 가 내린 판정 — **스케줄러는 자원 병목이 아니다** — 은
**조건부였음이 드러났다.** 그 판정은 24 스레드 호스트에서 성립했다.
8 스레드에서는 성립하지 않는다.

> 실험 대장 §4 의 원칙 그대로다. **배제는 조건부다.** 한 번 배제한 후보도
> 조건이 바뀌면 다시 열린다. 판정에는 "어떤 조건에서" 가 붙어야 한다.

이후 측정을 신서버에서 이어간다면 **구서버 값과 직접 비교하지 않는다.**
비교가 필요하면 신서버에서 기준선을 다시 깔고 그 위에서 상대 비교한다.

## 3.3 배포판 차이

```text
server  Rocky Linux 9.4   glibc 2.34   dnf   x86_64
nodes   Ubuntu 24.04      glibc 2.39   apt   aarch64
```

노드 바이너리는 aarch64 라 `king` 에서 네이티브 빌드해 세 노드에 배포한다
(세 보드 glibc 2.39 동일). 스케줄러는 x86_64 라 별개 빌드다.

---

# 4. 네트워크

## 4.1 현재 (2026-08-20 개편 완료)

```text
                server (10G) ─┐
                              ├── NS-S25G10G-N ──┬── king  (2.5G)
        개발PC (10G포트/1G NIC)┘                  ├── queen (2.5G)
                                                 ├── jack  (2.5G)
                                                 └── 인터넷 (2.5G, ipTIME)
```

- **worker 링크 2.5G, aggregation(server) 10G.** ADR-014 설계대로다.
- 아직 **관리망과 추론망이 분리되지 않았다.** 전부 `192.168.123.0/24` 단일
  대역이고, 보드 eth1 은 비어 있다. 측정 오염 방지를 위한 VLAN/서브넷 분리는
  M3 본측정 전에 결정한다.

## 4.2 대역폭 실측 (2026-08-20)

| 측정 | 값 | 도구 | 뜻 |
|---|---:|---|---|
| server enp1s0 협상 | 10000 Mb/s full | ethtool | 10G 링크 확정 |
| 단일 king→server | **2.34 Gbps** | iperf3 | 2.5G 실효 상한 |
| **3노드 동시 →server** | **각 1.70, 합 5.11 Gbps** | nc | **aggregation 병목 아님** |

3노드 동시 전송에서 세 스트림이 **균등하게(각 213 MB/s) 유지**됐다. 서버가
병목이면 합이 어딘가에서 깎였을 텐데 그러지 않았다. INT8 3노드 목표 RX
**4.60 Gbps** 를 여유 있게 수용한다(`RESULTS.md` §8.1).

> 개별 1.70 Gbps 가 링크 상한(2.34)보다 낮은 것은 nc/보드 CPU 단일코어
> 처리 한계지 스위치·서버 한계가 아니다. 실제 M3 는 gRPC 추론 트래픽이므로
> 이 값은 "인프라가 4.6 Gbps aggregate 를 받아내는가"의 검증으로만 쓴다 — 답은 예.

## 4.3 링크 속도 확인은 매번 한다

케이블 불량으로 협상이 낮아지는 사고가 반복됐다(옛 dealer 100Mb/s, 현
개발 PC 100Mb/s). 10GBASE-T 는 Cat6/6a 를 요구하고, Cat5e 면 조용히
2.5G/5G 로 떨어진다. 방치하면 NPU 가 아니라 케이블을 측정한다.

```bash
ssh npuforge-server 'ethtool enp1s0 | grep Speed'
for h in npuforge-k npuforge-q npuforge-j; do
  ssh "$h" 'printf "%s eth0=%s\n" "$(hostname)" "$(cat /sys/class/net/eth0/speed)"'
done
```

---

# 5. 구매 필요 목록

M3 를 막던 장비는 **전부 확보됐다.**

| 항목 | 상태 |
|---|---|
| ~~2.5G/10G 스위치~~ | ✅ NEXI NS-S25G10G-N (2.5G×4 + 10G×2) |
| ~~PCIe 슬롯 서버~~ | ✅ i7-4790 / 16GB / Rocky 9.4 (2026-08-26 교체) |
| ~~10G NIC~~ | ✅ Intel X550T `enp1s0` 10GBASE-T |
| ~~10G 케이블~~ | ✅ 10G full 협상 확인 (DAC 아닌 RJ45) |

남은 구매는 측정 품질용이며 M3 착수를 막지 않는다.

| 항목 | 수량 | 우선순위 | 근거 |
|---|---|---|---|
| 동일 모델 팬 | 3 | 중간 | S0-B 냉각 조건 비교용 |
| USB 전력 측정기 | 3 | 낮음 | FPS/Watt 산출 시 |
| Cat6/6a 케이블 (여유분) | 2~3 | 낮음 | 10G 링크 예비. 현 링크는 정상 |

냉각 장비(상시)는 목록에 없다. 팬리스를 유지하고 thermal throttling 을 측정
대상으로 삼는다(`adrs/013-fanless-thermal-as-measurement.md`).

---

# 6. 미해결 항목

| # | 항목 | 상태 | 차단 요소 |
|---|---|---|---|
| 1 | ~~IP static 고정~~ | ✅ 4대 전부 manual (2026-08-20) | — |
| 2 | **SSH 호스트 키 중복 (queen·jack)** | 미조치 | 없음. 명령은 `TODO.md` §1.2 |
| 3 | **INT8 모델을 queen·jack 에 배포** | 미조치 | 없음 |
| 4 | **스케줄러 빌드·배포 경로 확정** | 미정 | server 에 Rust 없음 (§3.2) |
| 5 | **server gRPC 포트 방화벽 개방** | 미조치 | 측정 전. firewalld public zone |
| 6 | 모델 변환 환경 재구축 | 보류 | dealer 소멸. 모델 이미 변환 완료라 급하지 않음 |
| 7 | 관리망/추론망 분리 | 미결정 | M3 본측정 전 |
| 8 | 실측 TX/RX 기록 (추론 트래픽) | 미측정 | 노드 소프트웨어 기동 후 |
| 9 | S0 열 특성 (30분 × 2조건) | 미실시 | 팬 3개 (S0-B 용) |

**호스트별 MAC / 고정 IP** (§1 에서 확인한 실제 MAC). 라우터 예약을 병행하면
이 표를 쓴다:

```text
king    22-94-FF-34-46-B1  →  192.168.123.3
jack    62-CE-3B-B6-E4-41  →  192.168.123.4
queen   7E-D8-D7-40-45-82  →  192.168.123.5
server  6C-B3-11-13-2F-38  →  192.168.123.9
```

해소된 항목(2026-08-20): 2.5G/10G 스위치, 10G 스케줄러 서버, 10G NIC·케이블,
aggregation 대역 실측, dealer RAM 3GB 제약. 이전 해소분: RKNN thread-safety
(컨텍스트 공유 금지), 모델 변환(FP16·INT8), Calibration(COCO 200장),
CPU governor(`performance`), 보드 배치 편차, OS 패치 레벨.
