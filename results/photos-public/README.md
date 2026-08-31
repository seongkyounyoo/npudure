# 실장비 사진

NPUForge 3노드 클러스터의 실제 장비 사진이다. 측정에 쓴 그 하드웨어다.

| 파일 | 내용 |
|---|---|
| `boards-labeled-01.jpg` `-02.jpg` | NanoPi R76S 3장. 라벨의 J / Q / K 가 `jack` / `queen` / `king` |
| `fans-01.jpg` `-02.jpg` `-03.jpg` | 120mm 팬 3개. 능동 냉각 조건에 쓴 것 |
| **`cluster-overview-01.jpg` `-02.jpg`** | **전체 구성** — 스케줄러 호스트(좌) · 노드 3대와 냉각 팬(중앙) · 대시보드가 도는 화면(우) |
| `cluster-front.jpg` | 조립 완료 정면. 우측 상단에 스위치와 배선 |
| `cluster-side-01.jpg` `-02.jpg` | 측면 로우앵글. 팬 아래 보드의 LED 점등 |
| `cluster-top-01.jpg` `-02.jpg` | 상단 뷰. 스위치·전원 계통이 함께 보인다 |
| `server-chassis-01.jpg` `-02.jpg` | 스케줄러 호스트 데스크톱 내부 (케이스 개방) |
| `server-i7-internal-01.jpg` `-02.jpg` | 같은 호스트 근접 |
| `server-nic-10g-01.jpg` `-02.jpg` | **Intel X550T 10GBASE-T** 가 PCIe 슬롯에 장착된 상태 |
| `switch-nexi-01.jpg` `-02.jpg` | **NEXI NS-S25G10G-N** — `2.5G Link` · `10G` 포트 표시와 링크 LED |

모두 EXIF 를 제거하고 긴 변 2048px 로 줄였다.

## 하드웨어 구성

사양·토폴로지는 [`../../docs/infrastructure.md`](../../docs/infrastructure.md)
에 있다.

- 노드 3대 — NanoPi R76S (RK3576, NPU 2코어), 각 2.5GbE
- 스위치 — NEXI NS-S25G10G-N (2.5G ×4 + 10G ×2)
- 스케줄러 호스트 — 10GbE. 3노드 트래픽의 합류점이다

**10G 는 선택이 아니라 요건이었다.** INT8 기준 노드 하나가 1.545 Gbps 를
요구하고 3노드 입력만 4.6 Gbps 다. 스케줄러 호스트가 그 합류점이라
2.5G 로는 받지 못한다. `switch-nexi-*.jpg` 의 `10G` 포트에 꽂힌 케이블이
그 업링크이고, `server-nic-10g-*.jpg` 가 반대쪽 끝이다.

**팬은 장식이 아니다.** 능동 냉각 여부가 지속 부하에서의 운영점을 가른다.
팬을 켜면 부하 중에도 CPU 클럭이 강등되지 않는다.
→ [`../../docs/experiments/S0_SUSTAINED_LOAD.md`](../../docs/experiments/S0_SUSTAINED_LOAD.md)

> **스케줄러 호스트는 두 번 바뀌었다.** 측정 421건은 Xeon E5-2630L ×2
> 서버에서 얻었고, 2026-08-26 에 Core i7-4790 데스크톱으로 교체됐다.
> `server-*.jpg` 는 **교체된 뒤**의 호스트다. 그 교체가 기준선 처리량을
> 바꿨고 경위는 `infrastructure.md` §3.2.1 에 있다. 구서버 사진은
> 공개분에 없다.

## 아직 없는 컷

- **스위치가 들어간 전체 구성** — `cluster-overview-*` 는 스케줄러 호스트 ·
  노드 · 화면을 한 프레임에 담았지만, 스위치는 배선 더미에 가려 보이지
  않는다. 스위치 단독 컷은 `switch-nexi-*` 에 있다.
