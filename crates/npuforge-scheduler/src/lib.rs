//! NPUDure 중앙 스케줄러.
//!
//! v0.1은 단일 중앙 스케줄러 구조를 사용한다. 분산 합의, 리더 선출,
//! 다중 스케줄러 고가용성은 구현하지 않는다.
//! `docs/01-TECHSPEC.md` §2.3 참조.

pub mod node_client;
pub mod policy;
pub mod registry;
pub mod service;

pub use node_client::{GrpcNodePool, NodeLink, RetryBackoff};
pub use policy::{NodeSnapshot, ScheduleError, SchedulingPolicy, policy_for};
pub use registry::{HeartbeatObservation, HeartbeatOutcome, NodeRecord, NodeStatusView, Registry};
pub use service::SchedulerServer;
