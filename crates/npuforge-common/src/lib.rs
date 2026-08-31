//! NPUForge 공통 타입.
//!
//! 이 크레이트는 스케줄러, 노드 에이전트, 벤치마크 도구가 공유하는 데이터 모델과
//! 백엔드 인터페이스를 제공한다. 의존성은 최소로 유지하며, 네트워크 및 런타임
//! 관련 크레이트에 의존하지 않는다.
//!
//! 정의의 근거는 `docs/01-TECHSPEC.md`에 있다.

pub mod backend;
pub mod config;
pub mod error;
pub mod model;
pub mod policy;
pub mod protocol;
pub mod telemetry;

pub use error::{ErrorCode, NpuForgeError, Result};
pub use model::{ModelId, ModelSpec, ModelState, ModelVersion};
pub use policy::SchedulingPolicyKind;
pub use protocol::{
    InferenceTask, InputFormat, NodeDescriptor, NodeState, RequestId, TimingBreakdown,
};

/// 노드 식별자.
///
/// 노드 설정 파일의 `[node] id` 값이며 클러스터 내에서 고유해야 한다.
pub type NodeId = String;
