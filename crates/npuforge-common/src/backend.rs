//! 추론 백엔드 인터페이스.
//!
//! 스케줄러와 노드 에이전트는 이 인터페이스만 사용하고 RKNN을 직접 호출하지 않는다.
//! `unsafe` 코드는 `npuforge-rknn` 내부로 제한한다.
//! `docs/01-TECHSPEC.md` §14.1, `docs/00-PRD.md` NFR-04 참조.

use crate::{
    error::Result,
    model::{ModelSpec, ModelState},
    protocol::{InputFormat, TimingBreakdown},
};

/// 백엔드에 전달하는 추론 입력.
#[derive(Debug, Clone)]
pub struct InferenceInput {
    pub payload: bytes::Bytes,
    pub format: InputFormat,
}

/// 백엔드가 반환하는 추론 결과.
#[derive(Debug, Clone)]
pub struct InferenceOutput {
    pub result: bytes::Bytes,
    pub result_format: String,
    /// 백엔드가 채운 단계별 시간. 노드가 큐 대기시간을 더해 완성한다.
    pub timing: TimingBreakdown,
}

/// 로딩된 모델의 정적 정보.
#[derive(Debug, Clone)]
pub struct LoadedModelInfo {
    pub spec: ModelSpec,
    pub state: ModelState,
    /// 이 모델에 대해 동시 호출이 가능한지 여부.
    ///
    /// RKNN Runtime의 thread-safety는 실장비 검증 전까지 확정되지 않는다.
    /// `false`이면 노드는 모델당 전용 워커 스레드를 사용해 호출을 직렬화한다.
    /// `docs/environment-matrix.md` §3.1 참조.
    pub supports_concurrent_infer: bool,
}

/// 모델을 로딩할 수 있는 백엔드.
#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn load_model(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>>;

    /// `rknn` 또는 `mock`. 메트릭 레이블과 로그에 사용한다.
    fn backend_name(&self) -> &'static str;

    /// 런타임 버전 문자열. 노드 등록 시 스케줄러에 보고한다.
    fn runtime_version(&self) -> Result<String>;
}

/// 로딩이 끝나 추론을 수행할 수 있는 모델.
#[async_trait::async_trait]
pub trait LoadedModel: Send + Sync {
    async fn infer(&self, input: InferenceInput) -> Result<InferenceOutput>;

    fn model_info(&self) -> &LoadedModelInfo;
}
