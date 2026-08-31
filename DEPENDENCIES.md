# 제3자 의존성

NPUForge는 Apache-2.0이다. 이 문서는 함께 사용하는 제3자 구성요소의 라이선스를 정리한다.

모델과 데이터 세트는 별도 문서를 따른다 → [`MODEL_LICENSES.md`](MODEL_LICENSES.md)

---

# 1. Rust 크레이트

2026-08-11 기준 **101개** 크레이트(전이 의존성 포함).
M2 에서 `sha2`(모델 해시 검증), `tokio-stream`(테스트용 리스너)와 전이 의존성이 추가되었다.
모두 MIT / Apache-2.0 계열이며 copyleft 는 없다.

## 1.1 라이선스 분포

| 개수 | 라이선스 |
|---:|---|
| 57 | MIT OR Apache-2.0 |
| 19 | MIT |
| 5 | Apache-2.0 |
| 4 | Apache-2.0 OR MIT |
| 3 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT |
| 2 | Unlicense OR MIT |
| 2 | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| 2 | BSD-2-Clause OR Apache-2.0 OR MIT |
| 1 | (MIT OR Apache-2.0) AND Unicode-3.0 |

**전부 허용적 라이선스이며 GPL 또는 AGPL 전용 의존성이 없다.** Apache-2.0 배포에 문제가 없다.

`OR`로 여러 선택지를 제시하는 크레이트는 Apache-2.0을 선택한다. `LGPL-2.1-or-later`를 포함하는 2개도 MIT 또는 Apache-2.0을 선택할 수 있으므로 LGPL 의무가 발생하지 않는다.

## 1.2 검증 방법

허용 목록은 [`deny.toml`](deny.toml)에 정의되어 있고 CI에서 검사한다.

```bash
cargo install cargo-deny
cargo deny check licenses
```

수동 집계가 필요하면 다음을 사용한다.

```bash
cargo metadata --format-version 1 --all-features \
  | python3 -c "
import json,sys,collections
m=json.load(sys.stdin); c=collections.Counter()
for p in m['packages']: c[p.get('license') or 'UNKNOWN'] += 1
for lic,n in c.most_common(): print(f'{n:4d}  {lic}')
"
```

**새 의존성을 추가할 때마다 확인한다.** `deny.toml`의 허용 목록에 없는 라이선스가 들어오면 CI가 실패한다.

## 1.3 직접 의존성

`Cargo.toml`에 명시한 것들이다.

| 크레이트 | 용도 |
|---|---|
| `tokio` | 비동기 런타임 |
| `serde` / `serde_json` / `toml` | 직렬화, 설정 파싱 |
| `thiserror` | 오류 타입 |
| `uuid` | Request ID |
| `bytes` | 페이로드 버퍼 |
| `async-trait` | 백엔드 인터페이스 |
| `tracing` / `tracing-subscriber` | 구조화 로그 |
| `parking_lot` | 레지스트리 잠금 |
| `rand` | Mock Backend 결정적 난수 |
| `libc` | RKNN FFI (unix 전용) |
| `cc` | C wrapper 빌드 |

---

# 2. RKNN 소프트웨어

**어느 것도 이 저장소에 포함하지 않는다.**

| 구성요소 | 출처 | 조건 |
|---|---|---|
| RKNN Runtime (`librknnrt.so`) | 보드 OS 이미지 사전 설치 | Rockchip 자체 조건 |
| `rknn_api.h`, `rknn_matmul_api.h`, `rknn_custom_op.h` | 보드 OS 이미지 사전 설치 | 동일 |
| RKNN-Toolkit2 | PyPI (`rknn-toolkit2==2.3.0`) | 동일 |
| `rknn_model_zoo` | GitHub, **Apache-2.0** | 코드 인용 가능 |

`npuforge-rknn` 크레이트는 `rknn` feature가 켜졌을 때만 `librknnrt.so`에 동적 링크한다. 기본값은 꺼져 있어 SDK 없이도 워크스페이스가 빌드된다.

**주의.** `rknn_model_zoo` 저장소는 Apache-2.0이지만, 그 안에서 배포하는 **모델 파일에는 각 원본 라이선스가 적용된다.** 저장소 라이선스와 데이터 라이선스는 별개다. → `MODEL_LICENSES.md` §2

---

# 3. 컨테이너 이미지

`tools/model-converter/Dockerfile`이 사용하는 것들이다.

| 구성요소 | 라이선스 |
|---|---|
| `ubuntu:22.04` 베이스 | 각 패키지별 (대부분 GPL/LGPL/MIT) |
| `rknn-toolkit2` | Rockchip 자체 조건 |
| `torch`, `onnx` (전이 의존성) | BSD-3-Clause / Apache-2.0 |
| `opencv-python-headless` | Apache-2.0 |
| `pillow` | MIT-CMU |

이 이미지는 **변환 도구일 뿐 배포 산출물이 아니다.** 사용자가 각자 빌드한다. 이미지 자체를 레지스트리에 공개하려면 포함된 패키지의 재배포 조건을 별도로 검토해야 한다.

---

# 4. 미해결 항목

| 항목 | 상태 |
|---|---|
| `cargo-deny` CI 통합 | `deny.toml` 작성 완료, `.github/workflows/ci.yml`에 잡 정의됨. 실행 검증 미완 |
| RKNN Runtime 재배포 조건 원문 확인 | 미완 |
| `NOTICE` 파일 갱신 | 의존성 확정 후 |
| 컨테이너 이미지 공개 여부 결정 | 미정 |
