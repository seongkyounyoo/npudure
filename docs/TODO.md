# NPUDure 진행 현황

- 최종 갱신: **2026-08-21**
- 발표까지: **D-99** (2026-11-28)
- 기능 동결: 2026-11-15

> 이 문서는 **지금 뭘 해야 하는지** 한눈에 보기 위한 것이다.
> 왜 그렇게 했는지는 `board-worklog.md`, 값은 `environment-matrix.md`, 상태는 `infrastructure.md`.

---

# ▶ 현재 상태: **측정 계보 종료** (2026-08-21)

S2 부터 S3.9b·S0-D 까지 전부 닫혔다. **421건 측정, 전 구간 오류율 0.**
남은 것은 발표 자료(그림)뿐이다.

| 계보 | 상태 | 결론 |
|---|---|---|
| **전송** | 닫힘 | 운영점 = **노드당 커넥션 2개 @ c12**. 3N 387.2 inf/s (+13.3%) |
| **확장** | 닫힘 | 3N **2.86× (95.3%)**. 손실은 **tail 에서 나타난다** — p50 평평, p99 +36% (S3.9a). micro-mechanism 까지 분리한 것은 아니다 |
| **지속 부하** | 닫힘 | 능동 냉각에서 short-run = sustained (−1.9%) |
| **정책** | 닫힘 | RR 은 이질에 취약, adaptive 가 tail −37%. 기본값 **`ect` 유지** |
| **io_uring (S4)** | **반박됨** | 회수 대상이 transport 비용의 1%. CPU 는 제약이 아니다 (S3.9b) |

```text
로컬 direct 161.5   운영점 135.5   잔여 gap 26.0 inf/s = direct 기준 16.1%
  -> CPU 비용이 아니라 경로 지연으로 보인다 (범위 밖, 관측만)
```

## 클러스터 조작 요령

```bash
# 접속 + 사전 검사
for h in npuforge-k npuforge-q npuforge-j npuforge-server; do ssh $h hostname; done
bash scripts/preflight-check.sh --with-inference

# 노드 재기동 — pkill 은 comm 으로 (-f 금지, ADR-017), 로그 리다이렉트 필수
ssh npuforge-k 'pkill -9 npuforge-node; sleep 3;   setsid nohup ~/npuforge/npuforge-node.s36 --config ~/npuforge/node.toml   >>~/npuforge/node.log 2>&1 & disown'
# 헬퍼: npuforge_restore_cluster (scripts/lib/remote.sh)

# run 합계 재계산
bash scripts/count-runs.sh
```

> **하네스 불변조건 2개**(`experiments/README.md` §4.12) — 새 하네스는 반드시 지킨다.
> ① 공유 자원의 상태는 공유 자원 쪽에서 검증한다(`npuforge_assert_cluster_free`).
> ② 결과 경로를 덮어쓸 수 있는 임시 폴더처럼 다루지 않는다.

## M3 토폴로지 (2026-08-20 확정, 실측 완료)

```text
        server 192.168.123.9  (Xeon x2 24T / 16GB / Rocky 9.4)
                    │
                  10GbE          ← aggregation. 10G full 실측
                    │
          NEXI NS-S25G10G-N  (2.5G x4 + 10G x2)
              ├── 2.5G ── king  .3
              ├── 2.5G ── queen .5
              └── 2.5G ── jack  .4
```

worker 링크는 2.5G, **aggregation 만 10G.** 옛 `dealer`(노트북)는 제거되고
스케줄러 역할이 `server` 로 이관됐다. `infrastructure.md` §1.

> **IP 고정 완료.** 개편 때 보드 IP 가 통째로 바뀌어(`.12/.16/.33` → `.3/.4/.5`)
> SSH 별칭이 낡았었다. 4대 전부 호스트 static(`manual`)으로 고정했다 (§1.1,
> `infrastructure.md` §2.3).

---

# ▶ 정책 계보 — 닫힌 것과 미룬 것 (2026-08-21)

**닫혔다.** RR 은 이질성에 취약하고, 상태 신선도를 고친 부하 인지
스케줄링이 RR 의 tail 을 크게 개선한다(p99 −37%). LQ·ECT 둘 다 정상
동작하며 regression 이 없다. **기본값은 `ect` 유지.**

**미뤘다 (Future Work).** 강한 이질에서 ECT 가 LQ 보다 우월한지는
미확정이다. 다만 **그 우열은 핵심 결론을 바꾸지 않는다** — 핵심은
"부하 인지 스케줄링이 이질을 흡수한다" 이고 어느 쪽으로도 성립한다.

S0-D 교정이 그 질문을 **재현 가능하게** 만들어 뒀다. 언제든 40분이면
답이 나온다(팬 ON, 예열 불필요).

```text
king CPU 캡   1200   1008    816    600
노드 지연 편차 1.33x  1.79x  2.26x  3.93x     ← 816 이 S0-A(2.4x) 재현
```

→ [`experiments/S0_D_CAPACITY_HETERO.md`](experiments/S0_D_CAPACITY_HETERO.md) §6

---

# ▶ S3.9b 완료 (2026-08-21) — **S4 io_uring 취소/보류**

```text
질문   io_uring 이 남은 16.1% 를 회수하는가?
답     아니다. 회수 대상(syscall 진입)이 transport 비용의 1%,
       가장 관대한 가정으로도 8%. 게다가 CPU 는 제약이 아니다.
```

| | 값 |
|---|---|
| transport 비용 | **16.35 CPU-ms/req** (유저 9.37 / 커널 6.99) |
| 네트워크 syscall | 요청당 ~165회 × 1µs = **0.165ms = 1.0%** |
| 보드 CPU | **48.9% idle**, 최고 코어 cpu0 78.8% busy(softirq) |
| cpu0 softirq | S3.5 §4.3 에서 RPS 분산 → **−0.2% null** |

**CPU-ms/req 는 비용이지 제약이 아니다.** 포화되지 않은 자원의 사용량을
줄이는 것은 처리량을 올리지 않는다.

큰 항은 따로 기록했다 — **유저 시간이 커널보다 크다**(직렬화·유저공간
copy 가 transport 비용의 57%). 다만 사전 규칙 3번째 가지대로 **여기서
멈춘다**: CPU 가 제약이 아닌 이상 이것을 줄여도 처리량이 오른다는 보장이
없다.

범위 밖 관측: gap 은 CPU 비용이 아니라 **경로 지연**으로 보인다
(지연 +37.3ms 중 노드 CPU 는 16.35ms, 페이로드 1.2MB 왕복 전송만 8.2ms).
지렛대가 있다면 io_uring 이 아니라 **페이로드 크기**다.

→ [`experiments/S3_9B_NODE_RESIDUAL.md`](experiments/S3_9B_NODE_RESIDUAL.md)

---

# ▶ (완료된 계획) S3.9b — node-side residual cost profiling

## 질문 (좁게)

> **161.5 → 135.5 사이의 residual gap 에서 node-side serialization /
> copy / syscall 비용이 유의미한 비중을 차지하는가?**

```text
로컬 direct 161.5   운영점 135.5   gap 26.0 inf/s = direct 기준 16.1%
                                   (1 − 135.5/161.5)
```

> 이전에 돌던 **13.2% 는 틀린 값이다.** 그 백분율은 140.1(S3.6 C)에서
> 나왔고 140.1 은 **c32 = 과부하 구간** 측정이라 운영 판단에 쓸 수 없다
> (README §4.1). 운영점 숫자와 짝지어져 두 계보가 섞였다.

**목적은 gap 을 전부 설명하는 것이 아니다.** S3.9a 에서 scale-out
tail/TCP 쪽 비용이 별도로 드러났으므로, node-side 프로파일이 26 inf/s
전체를 설명해야 할 이유가 없다. 설명 못 한 잔여는 잔여로 남긴다.

## 판정

| 결과 | 결정 |
|---|---|
| syscall·copy 가 **충분히 큼** | **S4 io_uring 진입** |
| **작음** | **S4 취소/보류** |
| **다른 항이 큼** | 그 항만 기록. **핵심 범위 밖이면 더 안 판다** |

세 번째 행이 중요하다. 프로파일이 예상 밖의 항을 가리켜도 그것을
쫓아가는 것은 이 실험의 임무가 아니다. 기록하고 범위 밖이면 멈춘다.

## 하네스 불변조건 (2026-08-21 확립)

새 하네스를 짤 때 반드시 지킨다. 둘 다 실제 사고에서 나왔다.

1. **공유 자원의 상태는 공유 자원 쪽에서 검증한다.**
   `npuforge_assert_cluster_free` — 서버에 `npuforge-bench` 가 돌면
   시작하지 않는다. 로컬 프로세스 관측은 플랫폼에 따라 거짓말을 한다.
2. **결과 경로를 append/overwrite 가능한 임시 폴더처럼 다루지 않는다.**
   기존 디렉터리가 비어 있지 않으면 멈춘다. `NPUFORGE_SUFFIX` 로 구분.

---

# 0. 한눈에 보기

| 영역 | 상태 |
|---|---|
| 소프트웨어 (M0) | ✅ 완료 — workspace, common, mock backend, 정책 엔진, CI |
| 하드웨어 인프라 | ✅ 완료 — 열 편차 5.6°C, **NPU** throttling 없음. 단 **CPU 는 강등된다** (팬은 S0-B 비교용) |
| RKNN 검증 | ✅ 백엔드 구현 완료, 컨텍스트 공유 위험 실측 확인 |
| 모델 변환 | ✅ FP16·INT8 완료, 정확도 검증 완료 |
| gRPC 통신 (M2) | 🟡 거의 완료 — 배선·재시도·Mock 클러스터 검증 끝, 메트릭 남음 |
| 벤치마크 (M3) | ✅ **완료 (2026-08-21)** — S2·S3·S3.5~3.9b·S0-A~D, **421건 / 오류율 0** |
| 대시보드 (M6) | ⬜ 미착수 |

**M3 차단 요소 — 전부 해소됨 (2026-08-20)**

| # | 항목 | 상태 |
|---|---|---|
| 1 | 2.5G/10G 스위치 | ✅ NEXI NS-S25G10G-N |
| 2 | PCIe 슬롯 서버 | ✅ Xeon x2 / 16GB / Rocky 9.4 (.9) |
| 3 | 10G NIC + 케이블 | ✅ `enp4s0` 10GBASE-T, 10G full 실측 |
| 4 | ~~`want_float=0` 전환~~ | ✅ 2026-08-12 |

**남은 작업 (2026-08-21 기준)**

| 항목 | 규모 | 왜 |
|---|---|---|
| ~~발표용 그림 보강~~ | ✅ **완료 (2026-08-21)** | 7개 추가 — `scripts/make-experiment-figures.py`. 경로는 handoff §5 |
| Prometheus 메트릭 (M2 잔여) | — | gRPC 통신 항목의 마지막 조각 |
| 대시보드 (M6) | — | 미착수 |
| systemd 이전 | — | 초안 `scripts/npuforge-node.service.in`. `pkill`→`systemctl stop` 과 함께 |
| queen·jack SSH host key 재생성 | — | 두 보드 키가 같아 구분 불가 |

> **측정은 더 필요하지 않다.** 추가 실험을 시작하기 전에
> `experiments/README.md` §2(배제표)와 §7(미해결)을 먼저 본다 —
> 이미 배제됐거나 조건부로 열려 있는 후보인지 확인하기 위해서다.

---

# 1. 즉시 할 일

## 1.1 사용자 작업 (물리·구매)

- [x] **보드 3대 배치 균일화** — 조치 불필요로 결론 (2026-08-11)
  - 통제된 재측정에서 19°C 격차가 **재현되지 않음**
  - 8스레드 동시 부하 15분: king 75.8 / queen 70.2 / jack 74.8°C (편차 5.6°C)
  - 90°C 초과 없음, NPU 클럭 강하 없음(928샘플 전부 950MHz)
  - 처리량 편차 3.5% (80.5 / 77.7 / 77.8 inf/s)
  - 이전 19°C는 부하 프로파일 차이(스윕 vs 고정, 6분 선행)로 부풀려진 것으로 판단
  - 상세: `board-worklog.md` §2.19
- [ ] **팬리스(S0-A) 클러스터 측정** — 오늘 baseline 은 능동 냉각(조건 B)이다.
  27% 를 확정하려면 조건 A 도 같은 gRPC 경로로 재야 한다 (§9)
- [x] **동일 모델 팬 3개 설치** (2026-08-20) — 120mm 5V USB, 노드당 1개(보드보다 큼).
  2026-08-20 측정 전체가 이 능동 냉각(조건 B)에서 수행됨
- [x] **2.5G/10G 스위치** (2026-08-20) — NEXI NS-S25G10G-N (2.5G×4 + 10G×2)
- [x] **스케줄러 서버 확보** (2026-08-20) — Xeon E5-2630L ×2 / 16GB / Rocky 9.4 (.9)
- [x] **10G NIC + 케이블** (2026-08-20) — 서버 내장 10GBASE-T, 10G full 실측
- [x] **IP 고정** (2026-08-20) — 호스트 NetworkManager static 으로 4대 전부
  고정(개발 작업으로 처리). ipTIME 라우터 예약은 선택 사항이며, 하면 아래 표를 쓴다.
  ```text
  king 22-94-FF-34-46-B1 →.3   jack 62-CE-3B-B6-E4-41 →.4
  queen 7E-D8-D7-40-45-82 →.5  server 6C-B3-11-13-2F-38 →.9
  ```
- [x] **calibration 이미지 방향 결정** — COCO val2017 200장 채택 (2026-08-11)
  - `tools/model-converter/fetch_calibration.py` 로 결정적 선택(seed 고정)
  - 이미지는 저장소에 넣지 않는다. manifest 만 남긴다 (라이선스)

## 1.2 개발 작업

- [x] `preflight-check.sh` — 벤치마크 전 하드 실패 검사 (2026-08-11)
  - 별칭↔hostname, 커널/RKNN/드라이버/모델 해시 일치
  - governor, 유휴 온도, 입력 전압, 남은 부하, NTP, 세션 수
  - `--with-inference`: 세 보드가 같은 입력에 같은 답을 내는지 (§9 교훈)
  - 음성 검사로 실제 검출 확인 (모델 바꿔치기, 부하 잔존)
- [x] `boot_id` 기록 — run 중 리셋 감지 및 무효화
  - 노드가 하트비트로 보고, 스케줄러가 변화 감지 시 경고 (M2에서 구현)
- [x] 벤치마크 telemetry 확장 — `ListNodes` RPC 로 온도·전압·boot_id 수집
  - 하트비트로 조회하면 스케줄러가 그 값을 관측으로 기록해 상태를 덮어쓴다.
    읽기 전용 RPC 를 따로 뒀다.
- [ ] **queen·jack SSH host key 재생성** — 둘이 동일해 암호학적으로 구분 불가.
  IP 가 바뀌면 경고 없이 엉뚱한 보드에 붙는다. DHCP 라 실제로 IP 가 바뀌므로
  (2026-08-20 개편에서 겪었다) 방치하면 안 된다.
  ```bash
  ssh npuforge-j 'sudo rm -f /etc/ssh/ssh_host_* &&     sudo ssh-keygen -A && sudo systemctl restart ssh'
  ssh-keygen -R npuforge-j   # PC 의 known_hosts 정리
  ```
- [x] **`want_float=0` 전환** (2026-08-12) — 설정 `[worker] want_float`
  - blob v2 로 `scale`·`zero_point` 동봉. 실보드 역양자화 검증 최대 오차 9.5e-7
  - 처리량 **INT8 +17.3% / FP16 +15.7%**, 출력 크기 4분의 1
- [ ] **bench 에 per-request 지연 원본 덤프 옵션** (2026-08-20 필요성 확인)
  현재 bench 는 run 마다 요약 percentile 만 JSON 에 남긴다. 그래서 여러 run 을
  묶은 표의 p95/p99 는 **run-level percentile 의 평균**이지 요청을 전부 합친
  pooled percentile 이 아니다(S2 §7.4.1).
  - run-level 평균은 각 run 의 최악 구간이 희석돼 **tail 을 낮게 보이게 한다.**
    조건 간 비교에는 문제없지만 절대값을 "이 시스템의 p99" 로 인용하면 안 된다.
  - S3.7 이 tail 로 운영점을 고르므로 이 구분이 실제로 중요해졌다.
  - 할 일: `--dump-samples <path>` 로 per-request 지연을 남기고, 분석기가
    pooled percentile 을 계산하게 한다. **단 bench 는 측정 도구라 S3.7/S3.8
    진행 중에는 바꾸지 않는다** — 동결 구간이 끝난 뒤에.
- [x] **jack 노드 복구** (2026-08-20) — 하드웨어는 정상이었다
  - eth0 **2.5G up**, IP 192.168.123.4, 바이너리·설정·모델 해시
    (`dba155d2…`)·governor 전부 정상. OOM·segfault 흔적 없음.
  - `dmesg` 가 이력을 보여줬다: 부팅 시 케이블이 **eth1** 에 있었고
    (`t=13.6s eth1 link up`), `t=620819s` 에 eth1 link down,
    `t=689135s` 에 **eth0 link up** — 즉 케이블이 물리적으로 옮겨졌다.
    다만 링크 단절은 프로세스를 죽이지 않는다(노드는 등록을 재시도한다).
  - **원인은 확정하지 못했다.** 로그가 없었기 때문이다 — 기동 절차가
    `setsid nohup ... &` 인데 로그 리다이렉트가 빠져 표준출력이 버려졌다.
  - 복구 후 검증: 3노드 335.4 inf/s, jack 33.3%(3362건), 오류 0,
    preflight `--with-inference` 전 항목 통과(**3노드 추론 출력 해시 동일**
    `e84c5b53…`).
  - 재발 방지: `lib/remote.sh` 의 `npuforge_restore_cluster` 가 세 노드를
    모두 복구하고 **로그 리다이렉트를 강제**한다. 측정 스크립트가 1노드
    구성을 만들려고 queen·jack 을 죽이면서 복구는 queen 만 하던 것이
    문제를 지속시켰다.
- [ ] **노드 기동을 systemd 로 옮기기** — 초안 `scripts/npuforge-node.service.in`
  로그 보관(journald)·마지막 종료 상태·재시작 정책을 얻는다. 위 jack 건에서
  "왜 죽었는지 알 수 없는" 상태가 프로세스가 죽는 것보다 나빴다.
  - ⚠️ **지금 설치하면 안 된다.** 측정 스크립트는 `pkill -9 npuforge-node`
    로 1노드 구성을 만든다. `Restart` 가 걸리면 systemd 가 즉시 되살려
    **1노드 측정이 조용히 3노드가 된다.** 틀린 줄 모르는 종류의 사고다.
  - 함께 해야 할 것: `run-*.sh` 의 `pkill` 을 `systemctl stop` 으로 교체.
    측정 캠페인(S3.8) 종료 후에 처리한다.
- [ ] **`ondemand` vs `performance` 300초 비교** ← §11 결론의 범위 확인
  - +7% 는 120초 측정이다. 지속 부하에서는 `performance` 가 더 빨리
    뜨거워져 불리할 수 있다. `discuss.md` §12
- [ ] **S0 를 30분으로** — 정상 상태 처리량과 CPU 강등 시점 확정
- [ ] **INT8 모델을 queen·jack 에 배포** — 현재 `king` 에만 있다
- [x] **스케줄러 빌드 경로 = server 네이티브** (2026-08-20 검증 완료) — server 에
  rust/cargo(dnf 1.92, MSRV 1.85 충족)·gcc·protoc·git 설치, `git archive`
  tarball 을 scp. 노드(aarch64)는 종전대로 king. 크로스빌드는 링커 문제로 회피
- [x] **IP static 고정** (2026-08-20) — server·king·queen·jack 4대 전부 manual.
  라우터 예약 대신 호스트 NetworkManager static. 같은 IP 라 SSH 무중단
- [ ] **server 방화벽 gRPC 포트 개방** — firewalld public zone, 측정 전
- [ ] `server`를 NTP 서버로 구성 + `chronyc waitsync` 대기
- [x] **스케줄러 RSS 우려 완화** (2026-08-20) — 서버 RAM 3GB → **16GB**.
  dealer 노트북 제약이 해소됐다. 그래도 S2 에서 RSS 는 관찰한다
  (`environment-matrix.md` §10.1)
- [x] CPU governor → `performance` (2026-08-12) — systemd 유닛으로 영구화
  - `scripts/set-cpu-governor.sh`, 재부팅 유지 확인 완료
  - **처리량 +7%.** 기존 수치는 전부 ondemand 기준이었다 (discuss.md §11)
- [x] `worker_count` 확정 — **8**, `core_mask` 미설정 (discuss.md §4)

---

# 2. 마일스톤별 진행

## M0. 저장소 및 환경 — ✅ 완료

- [x] Rust workspace (7 크레이트, edition 2024)
- [x] `npuforge-common` — 타입, 오류 코드, 설정, 백엔드 인터페이스
- [x] `npuforge-mock-backend` — 결정적 시드, 지연·오류율·속도편차 주입
- [x] `npuforge-rknn` 스텁 — feature 게이트로 Windows 빌드 통과
- [x] 스케줄링 정책 3종 (round-robin / least-queue / ect)
- [x] 노드 레지스트리 + 상태머신 + drain/disable
- [x] CI (fmt, clippy, test, aarch64 크로스, cargo-deny)
- [x] LICENSE (Apache-2.0), NOTICE, DEPENDENCIES.md, MODEL_LICENSES.md
- [x] 설정 예제 (scheduler, node, mock 3노드)
- [x] 테스트 통과 — M0 시점 81개, **현재 workspace 209개** (2026-08-14)

## M1. 단일 노드 추론 — 🟡 진행 중

- [x] RKNN C Wrapper 작성 및 **실기 컴파일 검증**
- [x] Thread-safety 검증 — **RKNN 2.3.0은 thread-safe 확정**
- [x] FFI 시그니처 실제 헤더와 대조 완료
- [x] 모델 변환 환경 (Docker, rknn-toolkit2 2.3.0)
- [x] YOLOv8n FP16 변환 및 3노드 배포
- [x] INT8 변환 — 6.46MB (FP16 9.65MB 대비 -33%)
- [x] 추론 정확도 검증 — 실보드 검출 수준 비교, `results/accuracy/README.md`
- [x] `npuforge-rknn` 실제 구현 — 컨텍스트 풀 + 다중 출력 (2026-08-11)
  - 실장비 통합 테스트 6종 통과 (`tests/real_device.rs`)
  - **공유 컨텍스트는 API 오류 0건으로 100% 틀린 결과를 낸다** — 실측
    (`environment-matrix.md` §3.1 정정)
- [ ] 1,000회 반복 추론 안정성 (soak으로 24,000회 확인, 정식 테스트는 별도)

## M2. 원격 추론 — 🟡 메트릭만 남음

- [x] `npuforge-proto` — .proto 정의 및 tonic 연결
- [x] `NodeService` gRPC 서버 (노드 측)
- [x] 노드 등록 / 하트비트 (등록 백오프 재시도, `must_reregister` 재등록)
- [x] `SchedulerService` gRPC 서버
- [x] 스케줄러 → 노드 gRPC 클라이언트 (노드별 채널 재사용)
- [x] 로컬 큐 + 워커 풀
- [x] 오류 처리 및 재시도 (재시도 시 다른 노드 선택)
- [x] 모델 디렉터리 로딩 + SHA-256 검증
- [x] **로컬 3노드 Mock 클러스터 동작 확인** (하드웨어 없이)
- [ ] 기본 메트릭 (Prometheus)
- [x] `npuforge-bench` CLI — 부하 발생·집계·run 유효성 판정 (2026-08-11)

### 검증한 것 (2026-08-11)

통합 테스트 `crates/npuforge-scheduler/tests/mock_cluster.rs` — 실제 gRPC 를 타고
스케줄러 ↔ 3노드가 붙는다. 프로세스만 하나일 뿐 전송 경로는 실장비와 같다.

| 검증 항목 | 결과 |
|---|---|
| 요청이 3노드에 분산 | ✅ round-robin 이 세 노드를 모두 사용 |
| 노드 1대 사망 시 우회 | ✅ 6/6 성공, 죽은 노드는 결과를 내지 않음 |
| 전 노드 사망 | ✅ `NPF-1302` + 시도한 노드 목록 |
| 타이밍 분해 | ✅ 노드 측정 구간과 스케줄러 측정 구간이 모두 채워짐 |
| 느린 노드 회피 | ✅ least-queue 가 빠른 노드를 더 많이 사용 |

실제 프로세스 4개(스케줄러 + 노드 3)로도 확인했다.
스케줄러를 죽였다 다시 띄우면 세 노드가 **약 1.3초 안에 스스로 재등록**한다.
노드는 하트비트 실패를 곧바로 재등록으로 전환한다 — 일시적 네트워크 오류와
스케줄러 재시작을 구분할 수 없으므로 더 비싼 쪽을 택했고, 등록은 멱등이다.

## M3. 다중 노드 — 🟡 클러스터 실동작 확인 (2026-08-20)

- [x] **실장비 3노드 등록** — king/queen/jack 스케줄러(.9) 등록 확인
- [x] **Round Robin 라우팅** — 예비 벤치에서 33.3% 정확 3등분
- [x] `npuforge-bench` CLI
- [x] **예비 3노드 추론** — c6 146 / c24 336 inf/s, 오류 0%
- [x] **S2 확장성 첫 측정** (2026-08-20) — 1/2/3노드 111.6/228.7/337.7 inf/s,
  **확장 효율 ~98%**. 클러스터 노드 상한 115 < 로컬 157 (스케줄러 오버헤드 27%).
  preflight 통과. RESULTS §2.5, board-worklog §2.25
- [ ] **S2 정식** — 반복 run·팬 조건·`--with-inference`·TimingBreakdown 오버헤드 분해
- [x] model.toml `model_file` 상대경로 버그 수정 (2026-08-20, §6 이슈 8)

## M4. 동적 스케줄링 — ⬜

- [ ] Least Queue / ECT 실장비 검증
- [ ] 정책 비교 (S3)

## M5. 장애 복구 — ⬜

- [ ] 헬스체크 실장비 검증
- [ ] 자동 제외 / 복귀
- [ ] 재시도 경로 검증
- [ ] **보드 하드 리셋과 의도된 장애 구분** (boot_id)

## M6. 대시보드 — ⬜

- [ ] 클러스터 개요 / 노드 뷰 / 벤치마크 뷰 / 이벤트 타임라인
- [ ] SSE 실시간 전송
- [ ] 전압·온도·주파수 표시

## M7. 최적화 실험 — ⬜

- [ ] **S2 확장성 실험 설계 재검토** ← INT8 결과로 전제가 바뀜
  - 노드당 1.545 Gbps (INT8) / 0.829 Gbps (FP16)
  - 3노드 4.636 / 2.486 Gbps — **둘 다 2.5GbE 한 링크를 넘는다**
  - aggregation 링크를 10G 로 (§4 토폴로지)
  - `discuss.md` §8, `RESULTS.md` §8.1 참조

- [ ] 버퍼 풀
- [ ] CPU 프로파일 (전처리 비중 확인)
- [ ] io_uring 적용 여부 판단

## M8. 발표 릴리스 — ⬜

- [ ] v0.1 태그, README, 설치 스크립트
- [ ] 벤치마크 원본 공개
- [ ] 발표자료, 데모 영상, 예비 영상

---

# 3. 벤치마크 시나리오

**전제: S0가 다른 모든 시나리오의 임계치와 cooldown을 결정한다. 반드시 먼저.**

- [ ] **S0-A** 열 특성 (팬리스) — 3노드 × 1,800초
- [ ] **S0-B** 열 특성 (냉각) — 3노드 × 1,800초
- [ ] S1 단일 노드 기준
- [ ] S2 확장성 (1/2/3노드)
- [ ] S3 스케줄러 정책 비교
- [ ] S4 장애 대응
- [ ] S5 네트워크 구현 비교
- [ ] S6 입력 크기 비교

총 146 run, 약 23.4시간. 무인 야간 실행 필요.

---

# 4. 인프라 현황

| 항목 | 상태 |
|---|---|
| 보드 3대 (king/queen/jack) | 🟡 OS·커널·RKNN·gcc·governor 일치, eth0 2.5G 실측. **SSH host key queen·jack 동일 (미해결)** |
| SSH 별칭·키 인증 | ✅ IP 갱신 완료 (.3/.5/.4/.9), `npuforge-server` 추가 |
| `server` (Rocky 9.4, 스케줄러·벤치) | ✅ **Xeon x2 24T / 16GB / 10G**. Rust·Docker 미설치 |
| 전원 5V 4A × 3 | ✅ 지속 부하 검증 완료 |
| **2.5G/10G 스위치** | ✅ NEXI NS-S25G10G-N |
| **추론망 대역** | ✅ worker 2.5G / aggregation 10G, 3노드 합 5.11 Gbps 실측 |
| 관리망/추론망 분리 | ⬜ 단일 대역 공유 중, M3 전 결정 |
| IP 고정 | ✅ 4대 static (manual). 라우터 예약은 선택 |
| 보드 물리 배치 | ✅ 편차 5.6°C, NPU throttling 없음 (2026-08-11 확인) |
| 냉각 (팬 3개) | ⬜ 미구매 |
| CPU governor | ✅ `performance` 고정 + 재부팅 유지 |
| NTP 동기화 | ⚠️ chrony 설치됨, `server` 서버화 미완 |
| 온도 임계치 | ⚠️ 초안값 (80/90°C) — S0 후 재설정 |

---

# 5. 구매 목록

M3 를 막던 장비(스위치·서버·10G NIC)는 **전부 확보됐다** (2026-08-20).
남은 것은 측정 품질용이다.

| 항목 | 수량 | 우선순위 | 비고 |
|---|---:|---|---|
| **동일 모델 팬** | 3 | 중간 | 5V USB, 동일 회전수. S0-B 용 |
| Cat6/6a 케이블 (여유) | 2~3 | 낮음 | 10G 예비. 현 링크는 정상 |
| USB 전력 측정기 | 3 | 낮음 | 5V 입력이라 USB 계측기 가능 |
| 예비 케이블·어댑터 | 각 1 | 중간 | 발표 대비 |

**전원 어댑터는 해결됨** (5V 4A × 3 교체 완료).

---

# 6. 알려진 이슈

| # | 이슈 | 심각도 | 상태 |
|---|---|---|---|
| 1 | ~~`king` 온도 19°C 높음~~ | 해소 | 통제 재측정에서 재현 안 됨 (편차 5.6°C) |
| 2 | 온도 임계치가 정상 동작 범위와 충돌 | **높음** | S0 후 재설정 |
| 3 | RTC 없음 — 부팅 직후 시각 틀림 | 중간 | chrony 대기 로직 필요 |
| 4 | 전류 센서 없음 → FPS/Watt 산출 불가 | 중간 | 외부 USB 전력계 필요 |
| 5 | 8스레드에서 처리량 안 꺾임 | 낮음 | MAX_THREADS 확장 필요 |
| 7 | 보드(king)에만 Rust 툴체인 설치됨 | 낮음 | 빌드 전용. 바이너리는 한 번 빌드해 배포 |
| 6 | `npu_cores` 수집값이 devfreq 개수(1) | 낮음 | 지표 정의 수정 |
| ~~8~~ | ~~model.toml `model_file` 상대경로 미해석~~ | 해소 | `main.rs` 가 `load_model` 전에 `spec.model_file` 을 절대경로로 교체. 상대경로 model.toml 로 3노드 로딩·벤치 재검증 완료 (2026-08-20) |
| 9 | 노드 재기동 시 NPU 컨텍스트 미해제 | 중간 | 죽은 노드가 컨텍스트를 안 놓아 재기동 status=-2. `pkill -9`+대기 필요. graceful shutdown 점검 |

---

# 7. 확정된 주요 수치

발표와 문서에 인용 가능한 실측값이다.

| 항목 | 값 | 출처 |
|---|---|---|
| SoC | RK3576, NPU 2코어 6 TOPS | 실측 |
| RKNN 동시성 | **전용 context 동시 실행 가능 / context 공유 금지** | 공유 시 API 오류 0인데 결과 200/200 불일치 |
| FP16 8스레드 순간 처리량 | 70~78 inf/s | 3노드 실측 |
| **FP16 8스레드 지속 처리량** | **84.3 inf/s** | governor=performance, 120초 |
| **INT8 8스레드 지속 처리량** | **157.2 inf/s** | governor=performance, **FP16 대비 1.86배** |
| INT8 평균 지연 | 50.8 ms | FP16 94.5 ms 대비 -46% |
| CPU governor 영향 | +7% | ondemand→performance. **120초 측정.** 지속 부하 미검증 |
| `want_float=0` 효과 | **INT8 +17.3% / FP16 +15.7%** | 출력도 4분의 1 (discuss.md §12) |
| **정상 상태 처리량 (300초)** | **FP16 59.7 inf/s** | 시작 81.6 대비 **-27%**. CPU throttling |
| (참고) ondemand 기준 | FP16 79.0 / INT8 146.2 | 08-11 이전 측정은 전부 이 기준 |
| 추론당 커널 ioctl | **76회 (FP16·INT8 동일)** | strace, 상한은 횟수가 아니라 시간 |
| **Peak vs Sustained 저하** | **약 10%** | 77.3 → 69.7 |
| 권장 `worker_count` | **8** (`core_mask` 미설정) | core_mask 스윕 |
| NPU 2코어 실제 기여 | **1.51배** (2배 아님) | 대조군 비교 |
| 지속 부하 시 NPU 온도 | **67.5~75.8°C** (3대, 8스레드 15분, FP16) | 2026-08-11 통제 측정 |
| INT8 정확도 (vs FP16) | 검출 셀 10/10, 클래스 100%, box cos 0.997 | 실보드 검출 수준 |
| **공유 컨텍스트 결과 불일치** | **100%** (API 오류 0건) | 컨텍스트 풀이 필수인 이유 |
| 노드 간 온도 편차 | **5.6°C** (NPU throttling 없음) | 동시 부하 |
| **CPU thermal 강등** | A72 2208→**816MHz**, A53 2016→**600MHz** | 부하 60초 후. NPU 는 950MHz 유지 |
| 부하 중 입력 전압 | 최소 5.05V | 3대 동시 실측 |

> Peak vs Sustained 격차는 벤더 스펙시트에 없는 수치이며 본 프로젝트의 핵심 산출물 중 하나다.
> 단, 현재 값은 선풍기 개입으로 오염되어 있어 **S0에서 깨끗하게 재측정해야 한다.**
>
> 2026-08-11 통제 측정(15분 × 3대)의 지속 처리량은 **77.7~80.5 inf/s** 로,
> soak 의 69.7 inf/s 보다 높다. soak 조건(24,000회, 더 긴 지속)과 다르므로
> 직접 비교하지 않는다. S0 에서 조건을 통일해 확정한다.
