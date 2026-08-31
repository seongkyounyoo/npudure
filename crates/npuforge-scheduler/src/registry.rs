//! 노드 레지스트리와 상태 머신.
//!
//! 상태 전이는 `docs/01-TECHSPEC.md` §9.1을 따른다.
//!
//! ```text
//! Registering ─ 등록 성공 ─→ Healthy ─ 수동 drain ─→ Draining ─ 큐 비움 ─→ Disabled
//!                              │  부하 높음 → Busy
//!                              │  오류/온도 → Degraded
//!                              │  연속 실패 → Unreachable ─ 헬스 성공 ─→ Recovering
//!                              └←──────────── 연속 성공 ────────────────┘
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use npuforge_common::{
    ModelId, NodeId,
    config::{HealthConfig, ThresholdConfig},
    model::ModelState,
    protocol::{NodeDescriptor, NodeState},
    telemetry::{Ewma, NodeHealth},
};
use parking_lot::{Mutex, RwLock};

use crate::policy::NodeSnapshot;

/// 스케줄러가 보관하는 노드 하나의 상태.
#[derive(Debug)]
pub struct NodeRecord {
    pub descriptor: NodeDescriptor,
    pub state: NodeState,
    pub health: NodeHealth,
    pub consecutive_health_failures: u32,
    pub consecutive_health_successes: u32,
    pub ewma_inference_ms: Ewma,
    pub ewma_network_ms: Ewma,
    /// 마지막으로 관측한 부팅 식별자.
    ///
    /// 값이 바뀌면 노드가 재부팅한 것이다. 실측에서 어댑터 용량 부족으로
    /// 보드가 벤치마크 도중 리셋된 적이 있고, 그 구간의 측정은 무효다.
    /// `docs/board-worklog.md` §2.17 참조.
    pub boot_id: String,
    /// 관측한 재부팅 횟수. 벤치마크 run 유효성 판정에 쓴다.
    pub reboot_count: u32,
    /// 전원 입력 전압. 강하는 리셋의 선행 지표다.
    pub input_voltage_v: Option<f64>,
    /// 운영자가 지정한 상태. 헬스체크로 덮어쓰지 않는다.
    operator_state: Option<NodeState>,
    /// **스케줄러가 이 노드로 보냈지만 아직 끝나지 않은 요청 수.**
    ///
    /// `health.in_flight` 와 의미가 다르다. 둘을 반드시 구분해야 한다.
    ///
    /// ```text
    /// health.in_flight   노드가 마지막 하트비트에 실어 보낸 관측값
    ///                    → authoritative 하지만 최대 1초 stale
    /// local_in_flight    이 스케줄러가 디스패치한 미완료 요청
    ///                    → 즉시 갱신, dispatch-time 결정에 쓸 수 있는 유일한 값
    /// ```
    ///
    /// 왜 필요한가 (S0-C): 정책이 하트비트 값만 보면, 갱신 주기(1초) 사이에
    /// 디스패치되는 수백 건이 **전부 같은 고정 스냅샷**을 보고 같은 노드를
    /// 고른다. 초당 380건 부하에서 처리량이 55~58% 무너졌다.
    /// 제어 루프의 샘플링 주기(1000ms)가 의사결정 주기(~3ms)보다 수백 배
    /// 길어서 생기는 문제다.
    ///
    /// **하트비트가 와도 이 값을 덮어쓰지 않는다.** 덮어쓰면 1초마다 다시
    /// 튄다.
    local_in_flight: Arc<AtomicU32>,
}

/// 하트비트 한 번에 실려 오는 관측값.
///
/// `NodeHealth` 에 없는 하드웨어 항목을 함께 전달한다.
#[derive(Debug, Clone, Default)]
pub struct HeartbeatObservation {
    pub health: NodeHealth,
    pub boot_id: String,
    pub input_voltage_v: Option<f64>,
}

/// 읽기 전용 노드 상태 뷰.
///
/// `NodeRecord` 를 그대로 노출하지 않는다. 내부 필드(`operator_state`)를
/// 밖에서 만질 수 있으면 상태 머신의 불변식이 깨진다.
#[derive(Debug, Clone)]
pub struct NodeStatusView {
    pub descriptor: NodeDescriptor,
    pub state: NodeState,
    pub health: NodeHealth,
    pub boot_id: String,
    pub reboot_count: u32,
    pub input_voltage_v: Option<f64>,
    pub ewma_inference_ms: Option<f64>,
    pub ewma_network_ms: Option<f64>,
}

/// 하트비트 처리 결과.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeartbeatOutcome {
    /// 등록되지 않은 노드다. 노드에 재등록을 요구해야 한다.
    pub must_reregister: bool,
    /// 직전 관측 이후 노드가 재부팅했다.
    pub rebooted: bool,
}

impl NodeRecord {
    pub fn new(descriptor: NodeDescriptor) -> Self {
        Self {
            descriptor,
            state: NodeState::Registering,
            health: NodeHealth::default(),
            consecutive_health_failures: 0,
            consecutive_health_successes: 0,
            ewma_inference_ms: Ewma::default(),
            ewma_network_ms: Ewma::default(),
            boot_id: String::new(),
            reboot_count: 0,
            input_voltage_v: None,
            operator_state: None,
            local_in_flight: Arc::new(AtomicU32::new(0)),
        }
    }

    /// 이 노드가 해당 모델을 Ready 상태로 보유하고 있는지 여부.
    pub fn has_model(&self, model_id: &ModelId) -> bool {
        self.descriptor
            .models
            .iter()
            .any(|m| &m.id == model_id && m.state == ModelState::Ready)
    }

    pub fn snapshot(&self, model_id: &ModelId) -> NodeSnapshot {
        NodeSnapshot {
            node_id: self.descriptor.node_id.clone(),
            state: self.state,
            has_model: self.has_model(model_id),
            queue_depth: self.health.queue_depth,
            in_flight: self.health.in_flight,
            local_in_flight: self.local_in_flight.load(Ordering::Acquire),
            ewma_inference_ms: self.ewma_inference_ms.get(),
            ewma_network_ms: self.ewma_network_ms.get(),
            error_rate: self.health.error_rate,
            temperature_c: self.health.temperature_c,
        }
    }

    /// 헬스체크 성공을 반영한다.
    fn on_health_success(&mut self, health: NodeHealth, cfg: &HealthConfig, th: &ThresholdConfig) {
        self.health = health;
        self.consecutive_health_failures = 0;
        self.consecutive_health_successes = self.consecutive_health_successes.saturating_add(1);

        // 운영자가 drain/disable을 걸어 둔 노드는 헬스체크가 성공해도
        // 자동으로 돌아오지 않는다. 벤치마크 중 노드 수가 임의로 바뀌면
        // 실험이 무효가 되기 때문이다.
        if let Some(forced) = self.operator_state {
            self.state = forced;
            return;
        }

        self.state = match self.state {
            // 복구 중에는 연속 성공 횟수를 채워야 정상으로 올라간다.
            NodeState::Unreachable | NodeState::Recovering => {
                if self.consecutive_health_successes >= cfg.recovery_threshold {
                    Self::classify(&self.health, th)
                } else {
                    NodeState::Recovering
                }
            }
            NodeState::Registering => Self::classify(&self.health, th),
            _ => Self::classify(&self.health, th),
        };
    }

    /// 헬스체크 실패를 반영한다.
    fn on_health_failure(&mut self, cfg: &HealthConfig) {
        self.consecutive_health_successes = 0;
        self.consecutive_health_failures = self.consecutive_health_failures.saturating_add(1);

        if let Some(forced) = self.operator_state {
            self.state = forced;
            return;
        }

        if self.consecutive_health_failures >= cfg.failure_threshold {
            self.state = NodeState::Unreachable;
        }
    }

    /// 헬스 지표로부터 정상 계열 상태를 판정한다.
    fn classify(health: &NodeHealth, th: &ThresholdConfig) -> NodeState {
        if let Some(t) = health.temperature_c
            && t >= th.degraded_temperature_c
        {
            return NodeState::Degraded;
        }
        if health.error_rate > th.degraded_error_rate {
            return NodeState::Degraded;
        }
        if health.queue_depth >= th.busy_queue_depth {
            return NodeState::Busy;
        }
        NodeState::Healthy
    }
}

/// 디스패치 예약. 살아 있는 동안 해당 노드의 `local_in_flight` 를 1 점유한다.
///
/// **어떤 종료 경로에서도 반드시 해제되어야 한다** — 성공, 오류, 타임아웃,
/// 취소, 재시도(다른 노드로 이동), future drop. 성공 응답에서만 감소시키면
/// 나머지 경로에서 카운터가 영원히 남아 그 노드가 과부하로 보인다.
/// 그래서 Drop 으로 묶는다.
#[derive(Debug)]
pub struct Reservation {
    counter: Arc<AtomicU32>,
}

impl Drop for Reservation {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::AcqRel);
    }
}

/// 노드 레지스트리.
///
/// v0.1은 메모리 기반이다. 영구 저장은 벤치마크 결과와 이벤트 로그에만 적용한다.
/// `docs/01-TECHSPEC.md` §6.3 참조.
#[derive(Debug, Default)]
pub struct Registry {
    nodes: RwLock<HashMap<NodeId, NodeRecord>>,
    /// 선택과 예약을 하나의 원자적 연산으로 묶는다.
    ///
    /// 이것이 없으면 동시에 들어온 요청들이 **모두 예약 전 스냅샷**을 보고
    /// 같은 노드를 고른다 — 하트비트 stale 문제를 고쳐도 herding 이 형태만
    /// 바꿔 남는다. `snapshot → choose → reserve` 가 한 임계구역이어야 한다.
    ///
    /// 직렬화 비용은 무시할 만하다. 선택 자체가 ~4 µs 이고 380 req/s 에서
    /// 점유율이 0.2% 수준이다.
    select_lock: Mutex<()>,
    health_config: HealthConfig,
    thresholds: ThresholdConfig,
}

impl Registry {
    pub fn new(health_config: HealthConfig, thresholds: ThresholdConfig) -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            select_lock: Mutex::new(()),
            health_config,
            thresholds,
        }
    }

    /// 노드를 등록하거나 기존 등록을 갱신한다.
    ///
    /// 재등록은 노드 재시작을 의미하므로 카운터를 초기화한다.
    /// 운영자가 걸어 둔 drain/disable은 유지한다.
    pub fn register(&self, descriptor: NodeDescriptor) {
        let mut nodes = self.nodes.write();
        let node_id = descriptor.node_id.clone();
        match nodes.get_mut(&node_id) {
            Some(existing) => {
                let operator_state = existing.operator_state;
                let mut record = NodeRecord::new(descriptor);
                record.operator_state = operator_state;
                if let Some(forced) = operator_state {
                    record.state = forced;
                }
                *existing = record;
            }
            None => {
                nodes.insert(node_id, NodeRecord::new(descriptor));
            }
        }
    }

    pub fn deregister(&self, node_id: &NodeId) -> bool {
        self.nodes.write().remove(node_id).is_some()
    }

    pub fn record_health_success(&self, node_id: &NodeId, health: NodeHealth) {
        if let Some(record) = self.nodes.write().get_mut(node_id) {
            record.on_health_success(health, &self.health_config, &self.thresholds);
        }
    }

    /// 하트비트를 반영한다.
    ///
    /// 등록되지 않은 노드의 하트비트는 무시하고 재등록을 요구한다. 스케줄러가
    /// 재시작하면 레지스트리가 비므로, 노드가 계속 하트비트만 보내면 영원히
    /// 클러스터에 복귀하지 못한다.
    pub fn record_heartbeat(
        &self,
        node_id: &NodeId,
        obs: HeartbeatObservation,
    ) -> HeartbeatOutcome {
        let mut nodes = self.nodes.write();
        let Some(record) = nodes.get_mut(node_id) else {
            return HeartbeatOutcome {
                must_reregister: true,
                rebooted: false,
            };
        };

        // 첫 관측은 재부팅이 아니다. 비교 대상이 없다.
        let rebooted =
            !record.boot_id.is_empty() && !obs.boot_id.is_empty() && record.boot_id != obs.boot_id;
        if rebooted {
            record.reboot_count = record.reboot_count.saturating_add(1);
        }
        if !obs.boot_id.is_empty() {
            record.boot_id = obs.boot_id;
        }
        record.input_voltage_v = obs.input_voltage_v;
        record.on_health_success(obs.health, &self.health_config, &self.thresholds);

        HeartbeatOutcome {
            must_reregister: false,
            rebooted,
        }
    }

    /// 스케줄러가 접속할 노드 주소.
    pub fn address_of(&self, node_id: &NodeId) -> Option<String> {
        self.nodes
            .read()
            .get(node_id)
            .map(|r| r.descriptor.address.clone())
    }

    /// 노드가 관측된 재부팅 횟수. run 무효화 판정에 쓴다.
    pub fn reboot_count(&self, node_id: &NodeId) -> Option<u32> {
        self.nodes.read().get(node_id).map(|r| r.reboot_count)
    }

    pub fn record_health_failure(&self, node_id: &NodeId) {
        if let Some(record) = self.nodes.write().get_mut(node_id) {
            record.on_health_failure(&self.health_config);
        }
    }

    /// 추론 완료 결과를 EWMA에 반영한다.
    pub fn record_inference(&self, node_id: &NodeId, inference_ms: f64, network_ms: f64) {
        if let Some(record) = self.nodes.write().get_mut(node_id) {
            record.ewma_inference_ms.update(inference_ms);
            record.ewma_network_ms.update(network_ms);
        }
    }

    /// 운영자 drain. 신규 요청을 받지 않는다.
    pub fn drain(&self, node_id: &NodeId) -> bool {
        self.set_operator_state(node_id, Some(NodeState::Draining))
    }

    /// 운영자 disable. 즉시 제외한다.
    pub fn disable(&self, node_id: &NodeId) -> bool {
        self.set_operator_state(node_id, Some(NodeState::Disabled))
    }

    /// 운영자 지정을 해제하고 헬스체크 결과에 따르게 한다.
    pub fn enable(&self, node_id: &NodeId) -> bool {
        if !self.set_operator_state(node_id, None) {
            return false;
        }
        if let Some(record) = self.nodes.write().get_mut(node_id) {
            // 다시 관측할 때까지는 복구 중으로 둔다.
            record.state = NodeState::Recovering;
            record.consecutive_health_successes = 0;
        }
        true
    }

    fn set_operator_state(&self, node_id: &NodeId, state: Option<NodeState>) -> bool {
        match self.nodes.write().get_mut(node_id) {
            Some(record) => {
                record.operator_state = state;
                if let Some(forced) = state {
                    record.state = forced;
                }
                true
            }
            None => false,
        }
    }

    /// 스케줄링에 넘길 스냅샷 목록.
    /// 후보를 고르고 **같은 임계구역 안에서** 예약까지 마친다.
    ///
    /// 반환된 [`Reservation`] 이 살아 있는 동안 그 노드의 `local_in_flight`
    /// 가 1 증가해 있다. 다음 요청은 그 값을 즉시 본다 — 이것이 herding 을
    /// 막는 핵심이다.
    ///
    /// `exclude` 는 이미 시도해 실패한 노드다(재시도 경로).
    pub fn select_and_reserve<F>(
        &self,
        model_id: &ModelId,
        exclude: &[NodeId],
        choose: F,
    ) -> Result<(NodeId, Reservation), crate::policy::ScheduleError>
    where
        F: FnOnce(&[NodeSnapshot]) -> Result<NodeId, crate::policy::ScheduleError>,
    {
        // 선택과 예약 사이에 다른 요청이 끼어들면 둘 다 같은 노드를 고른다.
        let _guard = self.select_lock.lock();

        let snaps: Vec<NodeSnapshot> = self
            .snapshots(model_id)
            .into_iter()
            .filter(|s| !exclude.contains(&s.node_id))
            .collect();

        let node_id = choose(&snaps)?;

        let nodes = self.nodes.read();
        let Some(rec) = nodes.get(&node_id) else {
            // 선택과 조회 사이에 등록이 해제됐다. 호출부가 다음 후보로 넘어간다.
            return Err(crate::policy::ScheduleError::NoAvailableNode);
        };
        let counter = Arc::clone(&rec.local_in_flight);
        counter.fetch_add(1, Ordering::AcqRel);
        Ok((node_id, Reservation { counter }))
    }

    /// 테스트·관측용. 현재 스케줄러 로컬 미완료 수.
    pub fn local_in_flight(&self, node_id: &NodeId) -> Option<u32> {
        self.nodes
            .read()
            .get(node_id)
            .map(|r| r.local_in_flight.load(Ordering::Acquire))
    }

    pub fn snapshots(&self, model_id: &ModelId) -> Vec<NodeSnapshot> {
        self.nodes
            .read()
            .values()
            .map(|r| r.snapshot(model_id))
            .collect()
    }

    /// 모든 노드의 상태를 읽는다. **레지스트리를 바꾸지 않는다.**
    ///
    /// 벤치마크 도구가 run 전후로 호출한다. 하트비트로 대신하면 스케줄러가
    /// 그 값을 실제 관측으로 기록해 노드 상태를 덮어쓴다 — 측정 직전에
    /// 상태를 오염시키게 된다.
    ///
    /// 노드 ID 순으로 정렬해 돌려준다. 순서가 run 마다 바뀌면 결과 파일을
    /// diff 하기 어렵다.
    pub fn list(&self) -> Vec<NodeStatusView> {
        let mut out: Vec<NodeStatusView> = self
            .nodes
            .read()
            .values()
            .map(|r| NodeStatusView {
                descriptor: r.descriptor.clone(),
                state: r.state,
                health: r.health,
                boot_id: r.boot_id.clone(),
                reboot_count: r.reboot_count,
                input_voltage_v: r.input_voltage_v,
                ewma_inference_ms: r.ewma_inference_ms.get(),
                ewma_network_ms: r.ewma_network_ms.get(),
            })
            .collect();
        out.sort_by(|a, b| a.descriptor.node_id.cmp(&b.descriptor.node_id));
        out
    }

    pub fn state_of(&self, node_id: &NodeId) -> Option<NodeState> {
        self.nodes.read().get(node_id).map(|r| r.state)
    }

    pub fn len(&self) -> usize {
        self.nodes.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 상태별 노드 수. 대시보드와 메트릭에 사용한다.
    pub fn count_by_state(&self) -> HashMap<NodeState, usize> {
        let mut counts = HashMap::new();
        for record in self.nodes.read().values() {
            *counts.entry(record.state).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use npuforge_common::model::ModelDescriptor;

    fn descriptor(id: &str) -> NodeDescriptor {
        NodeDescriptor {
            node_id: id.into(),
            hostname: format!("npuforge-{id}"),
            address: "10.20.0.21:51001".into(),
            device_type: "nanopi-r76s".into(),
            npu_type: "rk3576".into(),
            npu_core_count: 3,
            memory_bytes: 8 * 1024 * 1024 * 1024,
            agent_version: "0.1.0".into(),
            runtime_version: "mock-1.0.0".into(),
            models: vec![ModelDescriptor {
                id: "yolov8n".into(),
                version: "1.0.0".into(),
                state: ModelState::Ready,
                sha256: "0".repeat(64),
            }],
        }
    }

    fn registry() -> Registry {
        Registry::new(HealthConfig::default(), ThresholdConfig::default())
    }

    fn ok_health() -> NodeHealth {
        NodeHealth {
            queue_depth: 0,
            in_flight: 0,
            error_rate: 0.0,
            ..Default::default()
        }
    }

    #[test]
    fn registration_starts_in_registering_state() {
        let reg = registry();
        reg.register(descriptor("king"));
        assert_eq!(
            reg.state_of(&"king".to_string()),
            Some(NodeState::Registering)
        );
        // 등록 직후에는 스케줄링 후보가 아니다.
        assert!(!NodeState::Registering.is_schedulable());
    }

    #[test]
    fn first_successful_health_check_makes_node_healthy() {
        let reg = registry();
        reg.register(descriptor("king"));
        reg.record_health_success(&"king".to_string(), ok_health());
        assert_eq!(reg.state_of(&"king".to_string()), Some(NodeState::Healthy));
    }

    #[test]
    fn three_consecutive_failures_mark_unreachable() {
        let reg = registry();
        reg.register(descriptor("king"));
        reg.record_health_success(&"king".to_string(), ok_health());

        let id = "king".to_string();
        reg.record_health_failure(&id);
        assert_eq!(
            reg.state_of(&id),
            Some(NodeState::Healthy),
            "1회 실패로는 빼지 않는다"
        );
        reg.record_health_failure(&id);
        assert_eq!(
            reg.state_of(&id),
            Some(NodeState::Healthy),
            "2회 실패로도 빼지 않는다"
        );
        reg.record_health_failure(&id);
        assert_eq!(
            reg.state_of(&id),
            Some(NodeState::Unreachable),
            "3회에서 제외한다"
        );
    }

    #[test]
    fn recovery_requires_three_consecutive_successes() {
        let reg = registry();
        let id = "king".to_string();
        reg.register(descriptor("king"));
        for _ in 0..3 {
            reg.record_health_failure(&id);
        }
        assert_eq!(reg.state_of(&id), Some(NodeState::Unreachable));

        reg.record_health_success(&id, ok_health());
        assert_eq!(
            reg.state_of(&id),
            Some(NodeState::Recovering),
            "1회 성공은 복구 중"
        );
        reg.record_health_success(&id, ok_health());
        assert_eq!(
            reg.state_of(&id),
            Some(NodeState::Recovering),
            "2회 성공도 복구 중"
        );
        reg.record_health_success(&id, ok_health());
        assert_eq!(
            reg.state_of(&id),
            Some(NodeState::Healthy),
            "3회에서 정상 복귀"
        );
    }

    #[test]
    fn a_single_failure_during_recovery_resets_the_counter() {
        let reg = registry();
        let id = "king".to_string();
        reg.register(descriptor("king"));
        for _ in 0..3 {
            reg.record_health_failure(&id);
        }
        reg.record_health_success(&id, ok_health());
        reg.record_health_success(&id, ok_health());
        reg.record_health_failure(&id);
        reg.record_health_success(&id, ok_health());
        assert_eq!(
            reg.state_of(&id),
            Some(NodeState::Recovering),
            "중간에 실패했으면 다시 3회를 채워야 한다"
        );
    }

    #[test]
    fn high_queue_depth_marks_busy() {
        let reg = registry();
        let id = "king".to_string();
        reg.register(descriptor("king"));
        reg.record_health_success(
            &id,
            NodeHealth {
                queue_depth: 8,
                ..ok_health()
            },
        );
        assert_eq!(reg.state_of(&id), Some(NodeState::Busy));
        // Busy는 여전히 요청을 받는다.
        assert!(NodeState::Busy.is_schedulable());
    }

    #[test]
    fn high_error_rate_marks_degraded() {
        let reg = registry();
        let id = "king".to_string();
        reg.register(descriptor("king"));
        reg.record_health_success(
            &id,
            NodeHealth {
                error_rate: 0.2,
                ..ok_health()
            },
        );
        assert_eq!(reg.state_of(&id), Some(NodeState::Degraded));
    }

    #[test]
    fn high_temperature_marks_degraded() {
        let reg = registry();
        let id = "king".to_string();
        reg.register(descriptor("king"));
        reg.record_health_success(
            &id,
            NodeHealth {
                temperature_c: Some(82.0),
                ..ok_health()
            },
        );
        assert_eq!(reg.state_of(&id), Some(NodeState::Degraded));
    }

    // -- 운영자 제어 --------------------------------------------------------

    #[test]
    fn drained_node_stays_drained_through_healthy_checks() {
        // 이것이 공식 벤치마크에서 노드 수를 바꾸는 방식이다.
        // 헬스체크가 성공한다고 노드가 되돌아오면 실험이 무효가 된다.
        let reg = registry();
        let id = "king".to_string();
        reg.register(descriptor("king"));
        reg.record_health_success(&id, ok_health());
        assert!(reg.drain(&id));
        assert_eq!(reg.state_of(&id), Some(NodeState::Draining));

        for _ in 0..10 {
            reg.record_health_success(&id, ok_health());
        }
        assert_eq!(
            reg.state_of(&id),
            Some(NodeState::Draining),
            "헬스 성공으로 복귀하면 안 된다"
        );
    }

    #[test]
    fn disabled_node_survives_reregistration() {
        // 노드 프로세스가 재시작해도 운영자 지정은 유지되어야 한다.
        let reg = registry();
        let id = "king".to_string();
        reg.register(descriptor("king"));
        reg.disable(&id);

        reg.register(descriptor("king"));
        assert_eq!(reg.state_of(&id), Some(NodeState::Disabled));
    }

    #[test]
    fn enable_returns_node_through_recovering() {
        let reg = registry();
        let id = "king".to_string();
        reg.register(descriptor("king"));
        reg.record_health_success(&id, ok_health());
        reg.disable(&id);
        assert!(reg.enable(&id));

        // 곧바로 Healthy로 올리지 않는다. 실제 상태를 다시 관측해야 한다.
        assert_eq!(reg.state_of(&id), Some(NodeState::Recovering));
        for _ in 0..3 {
            reg.record_health_success(&id, ok_health());
        }
        assert_eq!(reg.state_of(&id), Some(NodeState::Healthy));
    }

    #[test]
    fn operator_commands_on_unknown_node_report_failure() {
        let reg = registry();
        let id = "nope".to_string();
        assert!(!reg.drain(&id));
        assert!(!reg.disable(&id));
        assert!(!reg.enable(&id));
    }

    // -- 스냅샷 -------------------------------------------------------------

    #[test]
    fn snapshot_reports_model_availability() {
        let reg = registry();
        reg.register(descriptor("king"));
        let snaps = reg.snapshots(&"yolov8n".to_string());
        assert!(snaps[0].has_model);

        let snaps = reg.snapshots(&"mobilenet_v3".to_string());
        assert!(
            !snaps[0].has_model,
            "보유하지 않은 모델은 후보에서 빠져야 한다"
        );
    }

    #[test]
    fn snapshot_carries_ewma_values() {
        let reg = registry();
        let id = "king".to_string();
        reg.register(descriptor("king"));
        assert_eq!(
            reg.snapshots(&"yolov8n".to_string())[0].ewma_inference_ms,
            None
        );

        reg.record_inference(&id, 25.0, 2.0);
        let snap = &reg.snapshots(&"yolov8n".to_string())[0];
        assert_eq!(snap.ewma_inference_ms, Some(25.0));
        assert_eq!(snap.ewma_network_ms, Some(2.0));
    }

    #[test]
    fn counts_by_state() {
        let reg = registry();
        for id in ["king", "queen", "jack"] {
            reg.register(descriptor(id));
            reg.record_health_success(&id.to_string(), ok_health());
        }
        reg.disable(&"jack".to_string());

        let counts = reg.count_by_state();
        assert_eq!(counts[&NodeState::Healthy], 2);
        assert_eq!(counts[&NodeState::Disabled], 1);
        assert_eq!(reg.len(), 3);
    }

    #[test]
    fn deregister_removes_node() {
        let reg = registry();
        reg.register(descriptor("king"));
        assert!(reg.deregister(&"king".to_string()));
        assert!(reg.is_empty());
        assert!(!reg.deregister(&"king".to_string()));
    }
}
