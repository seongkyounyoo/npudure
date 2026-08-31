//! proto 타입과 `npuforge-common` 타입 사이의 변환.
//!
//! 두 방향 모두 제공하며, 왕복 변환이 값을 보존하는지 테스트로 검증한다.
//! 필드를 하나 빠뜨리면 조용히 값이 사라지므로 이 테스트가 안전장치다.

use npuforge_common::{
    model::{ModelDescriptor, ModelState},
    protocol::{InputFormat, NodeDescriptor, NodeState, TimingBreakdown},
    telemetry::NodeHealth,
};

use crate::v1;

// ---------------------------------------------------------------------------
// InputFormat
// ---------------------------------------------------------------------------

impl From<InputFormat> for v1::InputFormat {
    fn from(f: InputFormat) -> Self {
        match f {
            InputFormat::Jpeg => v1::InputFormat::Jpeg,
            InputFormat::Png => v1::InputFormat::Png,
            InputFormat::Rgb8 => v1::InputFormat::Rgb8,
            InputFormat::Bgr8 => v1::InputFormat::Bgr8,
        }
    }
}

impl TryFrom<v1::InputFormat> for InputFormat {
    type Error = UnknownEnum;

    fn try_from(f: v1::InputFormat) -> Result<Self, Self::Error> {
        match f {
            v1::InputFormat::Jpeg => Ok(InputFormat::Jpeg),
            v1::InputFormat::Png => Ok(InputFormat::Png),
            v1::InputFormat::Rgb8 => Ok(InputFormat::Rgb8),
            v1::InputFormat::Bgr8 => Ok(InputFormat::Bgr8),
            v1::InputFormat::Unspecified => Err(UnknownEnum("InputFormat", 0)),
        }
    }
}

/// 지정되지 않았거나 알 수 없는 열거값.
///
/// proto3 의 열거형은 0 을 기본값으로 강제하므로, 클라이언트가 필드를
/// 채우지 않으면 `UNSPECIFIED` 가 온다. 이를 오류로 처리해 조용한 오동작을 막는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownEnum(pub &'static str, pub i32);

impl std::fmt::Display for UnknownEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} 열거값이 지정되지 않았거나 알 수 없습니다: {}",
            self.0, self.1
        )
    }
}

impl std::error::Error for UnknownEnum {}

// ---------------------------------------------------------------------------
// NodeState
// ---------------------------------------------------------------------------

impl From<NodeState> for v1::NodeState {
    fn from(s: NodeState) -> Self {
        match s {
            NodeState::Registering => v1::NodeState::Registering,
            NodeState::Healthy => v1::NodeState::Healthy,
            NodeState::Busy => v1::NodeState::Busy,
            NodeState::Degraded => v1::NodeState::Degraded,
            NodeState::Unreachable => v1::NodeState::Unreachable,
            NodeState::Recovering => v1::NodeState::Recovering,
            NodeState::Draining => v1::NodeState::Draining,
            NodeState::Disabled => v1::NodeState::Disabled,
        }
    }
}

impl TryFrom<v1::NodeState> for NodeState {
    type Error = UnknownEnum;

    fn try_from(s: v1::NodeState) -> Result<Self, Self::Error> {
        match s {
            v1::NodeState::Registering => Ok(NodeState::Registering),
            v1::NodeState::Healthy => Ok(NodeState::Healthy),
            v1::NodeState::Busy => Ok(NodeState::Busy),
            v1::NodeState::Degraded => Ok(NodeState::Degraded),
            v1::NodeState::Unreachable => Ok(NodeState::Unreachable),
            v1::NodeState::Recovering => Ok(NodeState::Recovering),
            v1::NodeState::Draining => Ok(NodeState::Draining),
            v1::NodeState::Disabled => Ok(NodeState::Disabled),
            v1::NodeState::Unspecified => Err(UnknownEnum("NodeState", 0)),
        }
    }
}

// ---------------------------------------------------------------------------
// ModelState
// ---------------------------------------------------------------------------

impl From<ModelState> for v1::ModelState {
    fn from(s: ModelState) -> Self {
        match s {
            ModelState::Unloaded => v1::ModelState::Unloaded,
            ModelState::Loading => v1::ModelState::Loading,
            ModelState::Ready => v1::ModelState::Ready,
            ModelState::Failed => v1::ModelState::Failed,
            ModelState::Draining => v1::ModelState::Draining,
        }
    }
}

impl TryFrom<v1::ModelState> for ModelState {
    type Error = UnknownEnum;

    fn try_from(s: v1::ModelState) -> Result<Self, Self::Error> {
        match s {
            v1::ModelState::Unloaded => Ok(ModelState::Unloaded),
            v1::ModelState::Loading => Ok(ModelState::Loading),
            v1::ModelState::Ready => Ok(ModelState::Ready),
            v1::ModelState::Failed => Ok(ModelState::Failed),
            v1::ModelState::Draining => Ok(ModelState::Draining),
            v1::ModelState::Unspecified => Err(UnknownEnum("ModelState", 0)),
        }
    }
}

// ---------------------------------------------------------------------------
// Timing
// ---------------------------------------------------------------------------

impl From<TimingBreakdown> for v1::Timing {
    fn from(t: TimingBreakdown) -> Self {
        v1::Timing {
            scheduler_queue_us: t.scheduler_queue_us,
            scheduler_route_us: t.scheduler_route_us,
            network_to_node_us: t.network_to_node_us,
            node_queue_us: t.node_queue_us,
            decode_us: t.decode_us,
            preprocess_us: t.preprocess_us,
            npu_input_us: t.npu_input_us,
            inference_us: t.inference_us,
            postprocess_us: t.postprocess_us,
            network_to_client_us: t.network_to_client_us,
            end_to_end_us: t.end_to_end_us,
        }
    }
}

impl From<v1::Timing> for TimingBreakdown {
    fn from(t: v1::Timing) -> Self {
        TimingBreakdown {
            scheduler_queue_us: t.scheduler_queue_us,
            scheduler_route_us: t.scheduler_route_us,
            network_to_node_us: t.network_to_node_us,
            node_queue_us: t.node_queue_us,
            decode_us: t.decode_us,
            preprocess_us: t.preprocess_us,
            npu_input_us: t.npu_input_us,
            inference_us: t.inference_us,
            postprocess_us: t.postprocess_us,
            network_to_client_us: t.network_to_client_us,
            end_to_end_us: t.end_to_end_us,
        }
    }
}

// ---------------------------------------------------------------------------
// ModelDescriptor
// ---------------------------------------------------------------------------

impl From<ModelDescriptor> for v1::ModelDescriptor {
    fn from(m: ModelDescriptor) -> Self {
        v1::ModelDescriptor {
            id: m.id,
            version: m.version,
            state: v1::ModelState::from(m.state) as i32,
            sha256: m.sha256,
        }
    }
}

impl TryFrom<v1::ModelDescriptor> for ModelDescriptor {
    type Error = UnknownEnum;

    fn try_from(m: v1::ModelDescriptor) -> Result<Self, Self::Error> {
        Ok(ModelDescriptor {
            id: m.id,
            version: m.version,
            state: v1::ModelState::try_from(m.state)
                .map_err(|_| UnknownEnum("ModelState", m.state))?
                .try_into()?,
            sha256: m.sha256,
        })
    }
}

// ---------------------------------------------------------------------------
// NodeDescriptor
// ---------------------------------------------------------------------------

impl From<NodeDescriptor> for v1::NodeDescriptor {
    fn from(d: NodeDescriptor) -> Self {
        v1::NodeDescriptor {
            node_id: d.node_id,
            hostname: d.hostname,
            address: d.address,
            device_type: d.device_type,
            npu_type: d.npu_type,
            npu_core_count: d.npu_core_count,
            memory_bytes: d.memory_bytes,
            agent_version: d.agent_version,
            runtime_version: d.runtime_version,
            models: d.models.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<v1::NodeDescriptor> for NodeDescriptor {
    type Error = UnknownEnum;

    fn try_from(d: v1::NodeDescriptor) -> Result<Self, Self::Error> {
        Ok(NodeDescriptor {
            node_id: d.node_id,
            hostname: d.hostname,
            address: d.address,
            device_type: d.device_type,
            npu_type: d.npu_type,
            npu_core_count: d.npu_core_count,
            memory_bytes: d.memory_bytes,
            agent_version: d.agent_version,
            runtime_version: d.runtime_version,
            models: d
                .models
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

// ---------------------------------------------------------------------------
// NodeHealth
// ---------------------------------------------------------------------------

/// proto 의 `NodeHealth` 는 `Option` 을 표현할 수 없어 `has_*` 플래그를 쓴다.
///
/// `boot_id` 와 `input_voltage_v` 는 `npuforge-common` 의 `NodeHealth` 에 없다.
/// 노드 에이전트가 채우고 스케줄러가 별도로 다루므로 여기서는 통과시킨다.
impl From<NodeHealth> for v1::NodeHealth {
    fn from(h: NodeHealth) -> Self {
        v1::NodeHealth {
            queue_depth: h.queue_depth,
            in_flight: h.in_flight,
            error_rate: h.error_rate,
            has_temperature: h.temperature_c.is_some(),
            temperature_c: h.temperature_c.unwrap_or(0.0),
            has_cpu_percent: h.cpu_percent.is_some(),
            cpu_percent: h.cpu_percent.unwrap_or(0.0),
            has_memory_percent: h.memory_percent.is_some(),
            memory_percent: h.memory_percent.unwrap_or(0.0),
            has_npu_percent: h.npu_percent.is_some(),
            npu_percent: h.npu_percent.unwrap_or(0.0),
            has_input_voltage: false,
            input_voltage_v: 0.0,
            boot_id: String::new(),
        }
    }
}

impl From<v1::NodeHealth> for NodeHealth {
    fn from(h: v1::NodeHealth) -> Self {
        NodeHealth {
            queue_depth: h.queue_depth,
            in_flight: h.in_flight,
            error_rate: h.error_rate,
            temperature_c: h.has_temperature.then_some(h.temperature_c),
            cpu_percent: h.has_cpu_percent.then_some(h.cpu_percent),
            memory_percent: h.has_memory_percent.then_some(h.memory_percent),
            npu_percent: h.has_npu_percent.then_some(h.npu_percent),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_format_round_trips() {
        for f in [
            InputFormat::Jpeg,
            InputFormat::Png,
            InputFormat::Rgb8,
            InputFormat::Bgr8,
        ] {
            let back: InputFormat = v1::InputFormat::from(f).try_into().unwrap();
            assert_eq!(f, back);
        }
    }

    #[test]
    fn unspecified_input_format_is_an_error() {
        // proto3 는 필드를 채우지 않으면 0(UNSPECIFIED)을 준다.
        // 이를 조용히 기본값으로 처리하면 잘못된 형식으로 추론하게 된다.
        assert!(InputFormat::try_from(v1::InputFormat::Unspecified).is_err());
    }

    #[test]
    fn node_state_round_trips_all_variants() {
        for s in [
            NodeState::Registering,
            NodeState::Healthy,
            NodeState::Busy,
            NodeState::Degraded,
            NodeState::Unreachable,
            NodeState::Recovering,
            NodeState::Draining,
            NodeState::Disabled,
        ] {
            let back: NodeState = v1::NodeState::from(s).try_into().unwrap();
            assert_eq!(s, back, "{s:?} 왕복 변환 실패");
        }
    }

    #[test]
    fn model_state_round_trips_all_variants() {
        for s in [
            ModelState::Unloaded,
            ModelState::Loading,
            ModelState::Ready,
            ModelState::Failed,
            ModelState::Draining,
        ] {
            let back: ModelState = v1::ModelState::from(s).try_into().unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn timing_round_trips_every_field() {
        // 필드를 하나 빠뜨리면 값이 조용히 0이 된다.
        // 모든 필드에 서로 다른 값을 넣어 누락을 잡는다.
        let t = TimingBreakdown {
            scheduler_queue_us: 1,
            scheduler_route_us: 2,
            network_to_node_us: 3,
            node_queue_us: 4,
            decode_us: 5,
            preprocess_us: 6,
            npu_input_us: 7,
            inference_us: 8,
            postprocess_us: 9,
            network_to_client_us: 10,
            end_to_end_us: 11,
        };
        let back: TimingBreakdown = v1::Timing::from(t).into();
        assert_eq!(t, back);
    }

    #[test]
    fn node_descriptor_round_trips_with_models() {
        let d = NodeDescriptor {
            node_id: "queen".into(),
            hostname: "queen".into(),
            address: "10.20.0.22:51001".into(),
            device_type: "nanopi-r76s".into(),
            npu_type: "rk3576".into(),
            npu_core_count: 2,
            memory_bytes: 4_094_296_064,
            agent_version: "0.1.0".into(),
            runtime_version: "2.3.0".into(),
            models: vec![ModelDescriptor {
                id: "yolov8n".into(),
                version: "1.0.0".into(),
                state: ModelState::Ready,
                sha256: "459602ea".repeat(8),
            }],
        };
        let back: NodeDescriptor = v1::NodeDescriptor::from(d.clone()).try_into().unwrap();
        assert_eq!(d, back);
    }

    #[test]
    fn node_health_preserves_missing_sensors() {
        // 센서가 없는 것과 값이 0인 것은 다르다.
        // has_* 플래그가 그 구분을 유지하는지 확인한다.
        let h = NodeHealth {
            queue_depth: 3,
            in_flight: 2,
            error_rate: 0.05,
            temperature_c: Some(72.5),
            cpu_percent: None,
            memory_percent: Some(0.0), // 0 이지만 존재한다
            npu_percent: None,
        };
        let back: NodeHealth = v1::NodeHealth::from(h).into();
        assert_eq!(h.temperature_c, back.temperature_c);
        assert_eq!(
            h.cpu_percent, back.cpu_percent,
            "없는 센서가 0.0 으로 바뀌면 안 된다"
        );
        assert_eq!(
            h.memory_percent, back.memory_percent,
            "0.0 이 None 으로 바뀌면 안 된다"
        );
        assert_eq!(h.npu_percent, back.npu_percent);
        assert_eq!(h.queue_depth, back.queue_depth);
        assert_eq!(h.error_rate, back.error_rate);
    }
}
