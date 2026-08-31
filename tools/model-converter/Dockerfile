# RKNN 모델 변환 환경.
#
# 변환은 호스트 환경에 따라 결과가 달라지면 안 된다. 이 이미지가
# Python·Toolkit·의존성 버전을 고정해 누구나 같은 .rknn 을 얻게 한다.
#
# docs/03-DEVELOPMENT-REQUIREMENTS.md §2.1
# docs/environment-matrix.md §3
#
# 빌드:
#   docker build -t npuforge-converter:2.3.0 tools/model-converter
#
# 실행:
#   docker run --rm -v "$PWD/models:/work/models" npuforge-converter:2.3.0 \
#       python3 convert_yolov8n.py --onnx models/yolov8n.onnx --out models/yolov8n.rknn
#
# RKNN-Toolkit2 는 x86_64 Linux 전용이다. ARM 호스트에서는 빌드되지 않는다.

FROM ubuntu:22.04

# Toolkit 버전은 보드의 Runtime 버전과 맞춰야 한다.
# 보드 실측: librknnrt 2.3.0 (c949ad889d@2024-11-07)
# Toolkit 이 Runtime 보다 높으면 변환된 모델이 로딩되지 않을 수 있다.
ARG RKNN_TOOLKIT_VERSION=2.3.0

ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONUNBUFFERED=1 \
    PIP_NO_CACHE_DIR=1

RUN apt-get update && apt-get install -y --no-install-recommends \
        python3 \
        python3-pip \
        python3-dev \
        libgl1 \
        libglib2.0-0 \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /work

COPY requirements.txt /work/requirements.txt

# torch 를 CPU 전용으로 먼저 설치한다.
#
# rknn-toolkit2 는 torch 를 의존성으로 요구하는데, 기본 인덱스에서 설치하면
# NVIDIA CUDA 라이브러리를 수 GB 끌어온다 (cusolver 124MB, cusparse 196MB,
# nccl 176MB 등). 변환은 CPU 에서만 수행하므로 전부 낭비다.
RUN python3 -m pip install --upgrade pip setuptools wheel \
    && python3 -m pip install torch --index-url https://download.pytorch.org/whl/cpu

RUN python3 -m pip install "rknn-toolkit2==${RKNN_TOOLKIT_VERSION}"

# ⚠️ onnx 버전을 반드시 고정한다.
#
# rknn-toolkit2 2.3.0 은 `onnx.mapping` 을 사용하는데, 이 모듈은 onnx 1.16
# 에서 제거되었다. 고정하지 않으면 최신 onnx(1.22 등)가 설치되어 변환이
# 다음 오류로 실패한다.
#
#   AttributeError: module 'onnx' has no attribute 'mapping'
#
# rknn-toolkit2 의 의존성 명세가 onnx 버전을 제한하지 않으므로 여기서 고정한다.
# 실측으로 1.14.1 에서 동작을 확인했다 (2026-08-07).
RUN python3 -m pip install "onnx==1.14.1" \
    && python3 -c "import onnx; assert hasattr(onnx, 'mapping'), 'onnx.mapping 없음'; print('onnx', onnx.__version__, 'OK')"

RUN python3 -m pip install -r /work/requirements.txt

COPY convert_yolov8n.py validate_rknn.py /work/

# 변환 환경 자체를 기록에 남길 수 있게 한다.
RUN python3 -c "from rknn.api import RKNN; print('rknn-toolkit2 import OK')" \
    && python3 -m pip freeze > /work/pip-freeze.txt

LABEL org.opencontainers.image.title="NPUForge model converter" \
      org.opencontainers.image.description="RKNN-Toolkit2 기반 ONNX -> RKNN 변환 환경" \
      org.opencontainers.image.licenses="Apache-2.0"

CMD ["python3", "convert_yolov8n.py", "--help"]
