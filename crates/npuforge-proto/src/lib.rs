//! NPUDure gRPC 프로토콜.
//!
//! `proto/npuforge.proto` 에서 생성된 코드와, `npuforge-common` 타입 사이의
//! 변환을 제공한다.
//!
//! **변환을 여기 모아두는 이유.** 스케줄러와 노드가 각자 변환을 구현하면
//! 필드 하나를 빠뜨렸을 때 한쪽에서만 값이 사라진다. 왕복 변환 테스트로
//! 그것을 막는다.

// tonic 이 생성하는 서비스 트레이트는 모든 메서드가
// `Result<Response<T>, tonic::Status>` 를 반환한다. `tonic::Status` 가
// 176 바이트라 clippy 의 `result_large_err`(임계 128B)에 걸린다.
//
// **생성 코드라 우리가 시그니처를 바꿀 수 없다.** tonic 이 Status 를
// 박싱하기 전까지는 방법이 없다. 이 모듈에만 끈다 — 우리가 직접 쓴
// 코드에는 그대로 적용된다.
//
// 로컬(1.97)에서는 안 걸리고 CI(1.98)에서만 걸렸다. 툴체인 드리프트다.
#[allow(clippy::result_large_err)]
pub mod v1 {
    tonic::include_proto!("npuforge.v1");
}

pub mod convert;

pub use v1::{
    node_service_client::NodeServiceClient, node_service_server::NodeService,
    node_service_server::NodeServiceServer, scheduler_service_client::SchedulerServiceClient,
    scheduler_service_server::SchedulerService, scheduler_service_server::SchedulerServiceServer,
};
