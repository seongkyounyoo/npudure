//! RKNN Runtime 백엔드.
//!
//! # 빌드 게이트
//!
//! 이 크레이트는 workspace 멤버지만 기본 feature가 꺼져 있다. 개발 PC(Windows/x86)에서
//! `cargo build --workspace`가 통과해야 Mock 기반 개발이 하드웨어에 묶이지 않는다.
//!
//! ```text
//! 개발 PC        : cargo build --workspace              (rknn off, 스텁)
//! 노드 바이너리  : cargo build-node                     (rknn on, 실제 링크)
//! ```
//!
//! # 안전성 규칙
//!
//! `docs/01-TECHSPEC.md` §14.3, `docs/03-DEVELOPMENT-REQUIREMENTS.md` §2.4를 따른다.
//!
//! - `unsafe` 코드는 `ffi` 와 `context` 모듈 안에만 둔다
//! - RKNN context는 RAII로 관리하고 해제를 `Drop`에서 보장한다
//! - Raw pointer를 공개 API에 노출하지 않는다
//! - 입력 버퍼는 추론 호출이 끝날 때까지 살아 있어야 한다
//! - RKNN 이 소유한 출력 버퍼를 함수 밖으로 내보내지 않는다.
//!   빌린 즉시 직렬화하고 해제한다
//!
//! # 동시성
//!
//! RKNN Runtime 2.3.0 은 thread-safe 지만 `inputs_set → run → outputs_get`
//! 시퀀스는 원자적이지 않다. 자세한 근거는 `context` 모듈 문서에 있다.
//! 동시 실행 수만큼 컨텍스트를 만들어 하나씩 점유해 쓴다.
//!
//! # 출력 형식
//!
//! YOLOv8n 은 출력 텐서를 9개 낸다. 하나의 `Bytes` 로 합치되 경계 정보를
//! 잃지 않도록 [`blob`] 의 자기 기술적 형식(`rknn-tensors-v1`)을 쓴다.

#![cfg_attr(not(feature = "rknn"), allow(dead_code))]

use npuforge_common::{
    ErrorCode, NpuForgeError, Result,
    backend::{InferenceBackend, InferenceInput, InferenceOutput, LoadedModel, LoadedModelInfo},
    model::ModelSpec,
};
#[cfg(feature = "rknn")]
use npuforge_common::{model::ModelState, protocol::TimingBreakdown};

pub mod blob;

#[cfg(feature = "rknn")]
mod context;
#[cfg(feature = "rknn")]
mod ffi;

#[cfg(feature = "rknn")]
pub use context::{ContextPool, RknnModelInfo};

/// RKNN Runtime 백엔드.
pub struct RknnBackend {
    #[allow(dead_code)]
    runtime_library: String,
    /// 컨텍스트 수. 노드의 `worker_count` 와 맞춘다.
    context_count: usize,
    /// 출력을 float32 로 받을지 여부.
    ///
    /// **기본값은 `false`(원본 dtype)다.** 네트워크 대역폭이 이유다.
    ///
    /// 노드는 후처리를 하지 않고 원시 텐서를 반환하므로, `true` 면 출력이
    /// 입력의 3.96배가 된다. 3노드 포화 시 스케줄러 RX 가 18.4 Gbps 로
    /// 10G 로도 감당하지 못한다. `false` 면 4.6 Gbps 로 들어온다.
    /// `docs/02-HARDWARE-SETUP.md` §3.3.2 참조.
    ///
    /// `false` 로 받은 데이터는 양자화된 정수이므로, 받는 쪽이
    /// `blob::TensorShape` 의 `scale`·`zero_point` 로 역양자화해야 한다.
    /// blob 형식 v2 가 그 값을 함께 싣는다.
    ///
    /// FP16 기준 처리량 이득은 +5.4% 였다(`docs/discuss.md` §5).
    /// **INT8 에서의 처리량 영향은 아직 측정하지 않았다.**
    want_float: bool,
}

impl RknnBackend {
    pub fn new(runtime_library: impl Into<String>) -> Self {
        Self {
            runtime_library: runtime_library.into(),
            context_count: 1,
            want_float: false,
        }
    }

    /// 컨텍스트 수를 지정한다. 노드의 `worker_count` 와 같아야 대기가 없다.
    pub fn with_context_count(mut self, n: usize) -> Self {
        self.context_count = n.max(1);
        self
    }

    pub fn with_want_float(mut self, want_float: bool) -> Self {
        self.want_float = want_float;
        self
    }
}

#[async_trait::async_trait]
impl InferenceBackend for RknnBackend {
    async fn load_model(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        #[cfg(not(feature = "rknn"))]
        {
            let _ = spec;
            Err(NpuForgeError::new(
                ErrorCode::BackendError,
                "이 바이너리는 RKNN 지원 없이 빌드되었습니다. \
                 `--features rknn` 으로 aarch64 대상 빌드가 필요합니다",
            ))
        }
        #[cfg(feature = "rknn")]
        {
            let path = spec.model_file.clone();
            let n = self.context_count;
            let want_float = self.want_float;

            // 컨텍스트 생성은 모델 파일을 읽고 NPU 를 초기화하므로 수백 ms
            // 걸린다. 비동기 런타임을 막지 않도록 블로킹 풀에서 돌린다.
            let pool = tokio::task::spawn_blocking(move || {
                context::ContextPool::open(&path, n, want_float)
            })
            .await
            .map_err(|e| NpuForgeError::internal(format!("모델 로딩 태스크 실패: {e}")))??;

            validate_against_spec(&pool.info, spec)?;

            Ok(Box::new(RknnModel {
                info: LoadedModelInfo {
                    spec: spec.clone(),
                    state: ModelState::Ready,
                    // 컨텍스트 풀이 직렬화를 처리하므로 노드는 동시 호출해도 된다.
                    supports_concurrent_infer: true,
                },
                pool,
            }))
        }
    }

    fn backend_name(&self) -> &'static str {
        "rknn"
    }

    fn runtime_version(&self) -> Result<String> {
        #[cfg(not(feature = "rknn"))]
        {
            Err(NpuForgeError::new(
                ErrorCode::BackendError,
                "RKNN 지원 없이 빌드된 바이너리입니다",
            ))
        }
        #[cfg(feature = "rknn")]
        {
            ffi::runtime_version()
        }
    }
}

/// 모델이 실제로 보고하는 형태와 `model.toml` 선언이 맞는지 확인한다.
///
/// 어긋나면 노드가 잘못된 크기의 입력을 받아들이고, RKNN 은 조용히 다른
/// 메모리를 읽는다. 로딩 시점에 막는 편이 훨씬 싸다.
#[cfg(feature = "rknn")]
fn validate_against_spec(info: &context::RknnModelInfo, spec: &ModelSpec) -> Result<()> {
    let declared = spec.raw_input_bytes();
    let actual = u64::from(info.input_bytes);
    if actual != 0 && declared != actual {
        return Err(NpuForgeError::new(
            ErrorCode::BackendError,
            format!(
                "model.toml 의 입력 크기가 모델과 다릅니다.\n  \
                 선언: {}x{}x{} = {declared} bytes\n  \
                 실제: {}x{}x{} = {actual} bytes",
                spec.input_width,
                spec.input_height,
                spec.input_channels,
                info.input_width,
                info.input_height,
                info.input_channels,
            ),
        ));
    }
    if info.input_count != 1 {
        return Err(NpuForgeError::new(
            ErrorCode::BackendError,
            format!(
                "입력 텐서가 {}개입니다. v0.1 은 단일 입력 모델만 지원합니다",
                info.input_count
            ),
        ));
    }
    Ok(())
}

/// 로딩된 RKNN 모델.
#[cfg(feature = "rknn")]
pub struct RknnModel {
    info: LoadedModelInfo,
    pool: context::ContextPool,
}

#[cfg(feature = "rknn")]
impl RknnModel {
    /// 이 모델이 보고하는 RKNN SDK 버전.
    pub fn sdk_version(&self) -> &str {
        &self.pool.sdk_version
    }

    pub fn context_count(&self) -> usize {
        self.pool.size()
    }
}

#[cfg(feature = "rknn")]
#[async_trait::async_trait]
impl LoadedModel for RknnModel {
    async fn infer(&self, input: InferenceInput) -> Result<InferenceOutput> {
        use npuforge_common::protocol::InputFormat;
        use std::time::Instant;

        // v0.1 은 디코딩을 하지 않는다. RGB8/BGR8 원본만 받는다.
        // JPEG 디코딩은 CPU 전처리 비중 실험(M7)에서 다룬다.
        if !matches!(input.format, InputFormat::Rgb8 | InputFormat::Bgr8) {
            return Err(NpuForgeError::new(
                ErrorCode::UnsupportedInputFormat,
                format!(
                    "RKNN 백엔드는 아직 {:?} 를 디코딩하지 않습니다. \
                     RGB8 또는 BGR8 원본을 보내세요",
                    input.format
                ),
            ));
        }

        let started = Instant::now();
        let result = self.pool.infer(&input.payload).await?;
        let elapsed = started.elapsed();

        Ok(InferenceOutput {
            result: bytes::Bytes::from(result),
            result_format: blob::RESULT_FORMAT.to_owned(),
            timing: TimingBreakdown {
                // 여기서 측정한 구간은 컨텍스트 대기 + NPU 실행 전체다.
                // 세부 분해는 RKNN_QUERY_PERF_RUN 이 필요한데, 그 값은
                // 큐 대기를 포함하므로 NPU 점유시간으로 해석하면 안 된다.
                // docs/discuss.md §2 참조.
                inference_us: elapsed.as_micros() as u64,
                ..Default::default()
            },
        })
    }

    fn model_info(&self) -> &LoadedModelInfo {
        &self.info
    }
}

/// RKNN 지원 없이 빌드했을 때의 자리표시자.
///
/// 노드는 `is_rknn_enabled()` 로 미리 걸러내므로 여기까지 오지 않는다.
#[cfg(not(feature = "rknn"))]
pub struct RknnModel {
    info: LoadedModelInfo,
}

#[cfg(not(feature = "rknn"))]
#[async_trait::async_trait]
impl LoadedModel for RknnModel {
    async fn infer(&self, _input: InferenceInput) -> Result<InferenceOutput> {
        Err(NpuForgeError::new(
            ErrorCode::BackendError,
            "RKNN 지원 없이 빌드된 바이너리입니다",
        ))
    }

    fn model_info(&self) -> &LoadedModelInfo {
        &self.info
    }
}

/// 이 빌드에 RKNN 지원이 포함되어 있는지 여부.
///
/// 노드 에이전트가 시작 시 설정과 빌드가 맞는지 확인하는 데 사용한다.
/// Mock 전용 바이너리를 실제 노드에 배포하는 사고를 조기에 잡기 위한 것이다.
pub const fn is_rknn_enabled() -> bool {
    cfg!(feature = "rknn")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_name_is_stable() {
        // 메트릭 레이블로 쓰이므로 바뀌면 안 된다.
        assert_eq!(
            RknnBackend::new("/usr/lib/librknnrt.so").backend_name(),
            "rknn"
        );
    }

    #[test]
    fn context_count_is_at_least_one() {
        // worker_count = 0 설정이 와도 컨텍스트 0개로 뜨면 안 된다.
        let b = RknnBackend::new("x").with_context_count(0);
        assert_eq!(b.context_count, 1);
        assert_eq!(RknnBackend::new("x").with_context_count(8).context_count, 8);
    }

    #[test]
    fn want_float_defaults_to_false() {
        // 기본값은 네트워크 대역폭 우선이다. want_float=1 이면 출력이
        // 입력의 3.96배라 3노드 RX 가 10G 를 넘는다.
        assert!(!RknnBackend::new("x").want_float);
        assert!(RknnBackend::new("x").with_want_float(true).want_float);
    }

    #[test]
    #[cfg(not(feature = "rknn"))]
    fn stub_build_reports_missing_support_clearly() {
        // 개발 PC 빌드를 노드에 배포했을 때 원인을 바로 알 수 있어야 한다.
        let backend = RknnBackend::new("/usr/lib/librknnrt.so");
        let err = backend.runtime_version().unwrap_err();
        assert_eq!(err.code, ErrorCode::BackendError);
        assert!(!is_rknn_enabled());
    }
}
