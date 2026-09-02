# 열 비교 측정 — 2026-08-11 10:48

*[English](README.md) — 영문이 정본이다.*

`king` 이 다른 두 대보다 NPU 온도 19°C 높다는 08-10 관측(§2.19)을
통제 조건에서 재확인하기 위한 측정이다.

## 조건

| 항목 | 값 |
|---|---|
| 부하 | `sustained_load_test` 8스레드 고정, 900초 |
| 모델 | `yolov8n-fp16.rknn` (sha 459602ea70479c1c) |
| 시작 | 세 보드 동시 |
| 냉각 | 팬리스, 선풍기 없음 |
| 로거 | 1초 간격 (온도·CPU/NPU 클럭·입력전압) |

바이너리·모델 해시가 세 보드 동일함을 사전 검증했다 (`preflight.txt`).
run 전후 `boot_id` 비교 결과 재부팅 없음 (`INVALID.txt` 없음).

## 결과

| 보드 | NPU 평균 | NPU 최고 | 처리량 |
|---|---|---|---|
| king | 73.0°C | 75.8°C | 80.5 inf/s |
| queen | 67.5°C | 70.2°C | 77.7 inf/s |
| jack | 72.6°C | 74.8°C | 77.8 inf/s |

**최대 편차 5.6°C. 90°C 초과 없음. NPU 클럭 강하 없음(928샘플 전부 950MHz).**

19°C 격차는 재현되지 않았다. 해석은 `docs/board-worklog.md` §2.19 참조.

## 파일

- `preflight.txt` — 호스트명·해시·boot_id 사전 검증 기록
- `baseline.txt` — 유휴 기준선
- `<보드>-temp.log` — 1초 간격 센서 로그
- `<보드>-load.log` — 10초 간격 처리량 + 최종 요약

재분석: `python scripts/analyze-thermal.py results/thermal-20260811-104811`
