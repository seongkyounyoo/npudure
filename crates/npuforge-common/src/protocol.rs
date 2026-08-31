//! 요청·응답 데이터 모델과 노드 상태.
//!
//! `docs/01-TECHSPEC.md` §8, §9 참조.

use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::{ModelId, NodeId, model::ModelDescriptor};

pub type RequestId = uuid::Uuid;

/// 클라이언트가 보내는 입력 데이터의 형식.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputFormat {
    /// JPEG 인코딩. 노드에서 디코딩이 필요하다.
    Jpeg,
    /// PNG 인코딩.
    Png,
    /// 디코딩 없이 바로 전처리로 넘어가는 raw RGB.
    Rgb8,
    /// raw BGR.
    Bgr8,
}

impl InputFormat {
    /// 노드에서 디코딩 단계가 필요한지 여부.
    ///
    /// 디코딩 비용은 `TimingBreakdown::decode_us`로 분리 측정한다.
    pub const fn needs_decode(self) -> bool {
        matches!(self, Self::Jpeg | Self::Png)
    }
}

/// 노드 상태.
///
/// 상태 전이는 `docs/01-TECHSPEC.md` §9.1의 상태 머신을 따른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeState {
    /// 등록 진행 중. 아직 요청을 받지 않는다.
    Registering,
    /// 정상.
    Healthy,
    /// 큐 길이 임계치 초과. 여전히 요청을 받는다.
    Busy,
    /// 오류율 또는 온도 임계치 초과. 여전히 요청을 받되 우선순위가 낮다.
    Degraded,
    /// 연속 헬스체크 실패.
    Unreachable,
    /// 헬스체크 복구 중. 제한된 요청만 할당한다.
    Recovering,
    /// 운영자가 drain 지시. 신규 요청 중지, 처리 중인 요청은 완료 대기.
    Draining,
    /// 운영자가 비활성화. 즉시 제외.
    Disabled,
}

impl NodeState {
    /// 스케줄링 후보가 될 수 있는 상태인지 여부.
    ///
    /// `Draining`과 `Disabled`는 운영자가 명시적으로 설정한 상태이므로
    /// 헬스체크 결과와 무관하게 후보에서 제외한다.
    /// `docs/01-TECHSPEC.md` §10.2 참조.
    pub const fn is_schedulable(self) -> bool {
        matches!(
            self,
            Self::Healthy | Self::Busy | Self::Degraded | Self::Recovering
        )
    }

    /// 운영자가 설정한 상태인지 여부.
    ///
    /// 이 상태에 있는 노드는 헬스체크 성공만으로 자동 복귀하지 않는다.
    /// 벤치마크에서 노드 수를 바꿀 때 이 성질에 의존한다.
    /// `docs/02-HARDWARE-SETUP.md` §12.3 참조.
    pub const fn is_operator_controlled(self) -> bool {
        matches!(self, Self::Draining | Self::Disabled)
    }

    /// 복구 중인 노드에 할당할 수 있는 최대 in-flight 요청 수의 비율.
    ///
    /// 복구 직후 전량을 보내면 같은 원인으로 다시 실패할 수 있으므로 제한한다.
    /// `docs/00-PRD.md` FR-07 참조.
    pub const fn load_factor(self) -> f64 {
        match self {
            Self::Healthy => 1.0,
            Self::Busy => 1.0,
            Self::Degraded => 0.5,
            Self::Recovering => 0.25,
            _ => 0.0,
        }
    }
}

/// 노드 등록 시 전달하는 정적 정보.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeDescriptor {
    pub node_id: NodeId,
    pub hostname: String,
    /// 스케줄러가 접속할 주소. 노드 설정의 `advertise_address`.
    pub address: String,
    pub device_type: String,
    pub npu_type: String,
    pub npu_core_count: u32,
    pub memory_bytes: u64,
    /// 노드 에이전트 바이너리 버전. 세 노드가 같아야 한다.
    pub agent_version: String,
    /// 백엔드 런타임 버전. 세 노드가 같아야 한다.
    pub runtime_version: String,
    pub models: Vec<ModelDescriptor>,
}

/// 스케줄러 내부에서 다루는 추론 작업.
///
/// `Instant`를 포함하므로 직렬화하지 않는다. 서로 다른 장치의 monotonic clock을
/// 직접 비교하지 않는다는 원칙 때문이다. `docs/02-HARDWARE-SETUP.md` §10.1 참조.
#[derive(Debug, Clone)]
pub struct InferenceTask {
    pub request_id: RequestId,
    pub model_id: ModelId,
    pub payload: bytes::Bytes,
    pub input_format: InputFormat,
    pub priority: i32,
    pub deadline: Option<Instant>,
    pub created_at: Instant,
    pub attempt: u32,
    pub max_attempts: u32,
}

impl InferenceTask {
    /// 재시도가 남아 있는지 여부.
    pub const fn can_retry(&self) -> bool {
        self.attempt + 1 < self.max_attempts
    }

    /// 대기시간. 우선순위 aging 계산에 사용한다.
    pub fn age(&self, now: Instant) -> std::time::Duration {
        now.saturating_duration_since(self.created_at)
    }
}

/// 우선순위 상수. `docs/01-TECHSPEC.md` §10.5.
pub mod priority {
    pub const HIGH: i32 = 10;
    pub const NORMAL: i32 = 0;
    pub const LOW: i32 = -10;
}

/// 단계별 지연시간. 모든 값은 마이크로초.
///
/// 각 장치는 자신이 측정한 duration만 채우고, 절대 시각을 비교하지 않는다.
/// `docs/00-PRD.md` FR-13, `docs/02-HARDWARE-SETUP.md` §10.1 참조.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimingBreakdown {
    // 스케줄러가 측정하는 구간
    pub scheduler_queue_us: u64,
    pub scheduler_route_us: u64,
    pub network_to_node_us: u64,

    // 노드가 측정해 응답에 실어 보내는 구간
    pub node_queue_us: u64,
    pub decode_us: u64,
    pub preprocess_us: u64,
    pub npu_input_us: u64,
    pub inference_us: u64,
    pub postprocess_us: u64,

    // 스케줄러가 측정하는 구간
    pub network_to_client_us: u64,
    pub end_to_end_us: u64,
}

impl TimingBreakdown {
    /// 노드 내부에서 소비한 총 시간.
    pub const fn node_total_us(&self) -> u64 {
        self.node_queue_us
            + self.decode_us
            + self.preprocess_us
            + self.npu_input_us
            + self.inference_us
            + self.postprocess_us
    }

    /// 스케줄러가 유발한 오버헤드.
    ///
    /// NFR-01의 "라우팅 오버헤드 5% 이하" 판정에 사용한다.
    pub const fn scheduler_overhead_us(&self) -> u64 {
        self.scheduler_queue_us + self.scheduler_route_us
    }

    /// 네트워크 왕복 시간.
    pub const fn network_total_us(&self) -> u64 {
        self.network_to_node_us + self.network_to_client_us
    }

    /// 스케줄러 오버헤드가 전체에서 차지하는 비율.
    ///
    /// `end_to_end_us`가 0이면 측정이 유효하지 않으므로 `None`을 반환한다.
    pub fn scheduler_overhead_ratio(&self) -> Option<f64> {
        if self.end_to_end_us == 0 {
            return None;
        }
        Some(self.scheduler_overhead_us() as f64 / self.end_to_end_us as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_states_are_never_schedulable() {
        // drain/disable로 노드를 빼는 벤치마크 방식이 이 성질에 의존한다.
        for state in [NodeState::Draining, NodeState::Disabled] {
            assert!(state.is_operator_controlled());
            assert!(!state.is_schedulable());
            assert_eq!(state.load_factor(), 0.0);
        }
    }

    #[test]
    fn unreachable_and_registering_are_not_schedulable() {
        assert!(!NodeState::Unreachable.is_schedulable());
        assert!(!NodeState::Registering.is_schedulable());
    }

    #[test]
    fn recovering_nodes_get_reduced_load() {
        // 복구 직후 전량을 보내면 같은 원인으로 다시 쓰러질 수 있다.
        assert!(NodeState::Recovering.is_schedulable());
        assert!(NodeState::Recovering.load_factor() < NodeState::Healthy.load_factor());
    }

    #[test]
    fn retry_budget_is_respected() {
        let task = InferenceTask {
            request_id: uuid::Uuid::new_v4(),
            model_id: "yolov8n".into(),
            payload: bytes::Bytes::new(),
            input_format: InputFormat::Jpeg,
            priority: priority::NORMAL,
            deadline: None,
            created_at: Instant::now(),
            attempt: 0,
            max_attempts: 2,
        };
        assert!(task.can_retry());

        let last = InferenceTask { attempt: 1, ..task };
        assert!(
            !last.can_retry(),
            "max_attempts=2 이면 attempt=1이 마지막 시도다"
        );
    }

    #[test]
    fn raw_formats_skip_decode() {
        assert!(InputFormat::Jpeg.needs_decode());
        assert!(InputFormat::Png.needs_decode());
        assert!(!InputFormat::Rgb8.needs_decode());
        assert!(!InputFormat::Bgr8.needs_decode());
    }

    #[test]
    fn timing_breakdown_sums() {
        let t = TimingBreakdown {
            scheduler_queue_us: 100,
            scheduler_route_us: 50,
            network_to_node_us: 800,
            node_queue_us: 300,
            decode_us: 1_000,
            preprocess_us: 1_600,
            npu_input_us: 200,
            inference_us: 18_200,
            postprocess_us: 900,
            network_to_client_us: 700,
            end_to_end_us: 23_850,
        };
        assert_eq!(t.node_total_us(), 22_200);
        assert_eq!(t.scheduler_overhead_us(), 150);
        assert_eq!(t.network_total_us(), 1_500);

        let ratio = t.scheduler_overhead_ratio().unwrap();
        assert!(
            ratio < 0.05,
            "이 예시는 NFR-01의 5% 목표를 만족해야 한다: {ratio}"
        );
    }

    #[test]
    fn overhead_ratio_is_none_without_measurement() {
        assert!(
            TimingBreakdown::default()
                .scheduler_overhead_ratio()
                .is_none()
        );
    }
}
