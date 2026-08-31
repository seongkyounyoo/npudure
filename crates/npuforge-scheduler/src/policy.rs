//! 스케줄링 정책.
//!
//! 세 정책은 동일한 후보 필터를 거친 뒤 서로 다른 기준으로 노드를 고른다.
//! 필터를 정책마다 따로 두면 정책 비교 실험이 공정하지 않게 되므로
//! [`candidates`] 한 곳에서만 수행한다.
//!
//! `docs/01-TECHSPEC.md` §10 참조.

use std::sync::atomic::{AtomicUsize, Ordering};

use npuforge_common::{
    NodeId,
    policy::SchedulingPolicyKind,
    protocol::{InferenceTask, NodeState},
};

/// 스케줄링 시점의 노드 상태 사본.
///
/// 정책은 레지스트리 잠금을 들고 있지 않은 상태에서 동작해야 하므로
/// 스냅샷을 받는다.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeSnapshot {
    pub node_id: NodeId,
    pub state: NodeState,
    /// 노드가 요청한 모델을 Ready 상태로 보유하고 있는지 여부.
    pub has_model: bool,
    pub queue_depth: u32,
    pub in_flight: u32,
    /// **스케줄러가 이 노드로 보냈지만 아직 끝나지 않은 요청 수.**
    ///
    /// `queue_depth`·`in_flight` 는 하트비트(1초) 관측값이라 dispatch-time
    /// 결정에 쓸 수 없다. 초당 수백 건이 같은 고정 스냅샷을 보고 같은 노드를
    /// 고르는 herding 이 생긴다(S0-C 에서 처리량 55~58% 붕괴).
    ///
    /// 이 값은 디스패치마다 즉시 갱신된다. **부하 인지 정책은 이것을 1차
    /// 신호로 써야 한다.**
    ///
    /// 하트비트 값과 더하지 않는다 — 중앙 스케줄러 하나가 모든 요청을
    /// 보내므로(ADR-003) 두 값은 **같은 요청을 세는 것**이고, 더하면
    /// 이중 계산이 된다.
    pub local_in_flight: u32,
    /// 이동 평균 추론시간. 샘플이 없으면 `None`.
    pub ewma_inference_ms: Option<f64>,
    /// 이동 평균 네트워크 왕복시간. 샘플이 없으면 `None`.
    pub ewma_network_ms: Option<f64>,
    /// 최근 윈도우 오류율. 0.0 ~ 1.0.
    pub error_rate: f64,
    pub temperature_c: Option<f64>,
}

impl NodeSnapshot {
    /// 테스트와 예제용 최소 생성자.
    pub fn healthy(node_id: impl Into<NodeId>) -> Self {
        Self {
            node_id: node_id.into(),
            state: NodeState::Healthy,
            has_model: true,
            queue_depth: 0,
            in_flight: 0,
            local_in_flight: 0,
            ewma_inference_ms: None,
            ewma_network_ms: None,
            error_rate: 0.0,
            temperature_c: None,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ScheduleError {
    /// 후보가 하나도 없다. `NPF-1201`로 변환한다.
    #[error("요청을 처리할 수 있는 노드가 없습니다")]
    NoAvailableNode,
}

/// 스케줄링 정책이 동작할 때 참조하는 임계치.
///
/// `docs/01-TECHSPEC.md` §9.2의 설정값에서 온다.
#[derive(Debug, Clone, Copy)]
pub struct PolicyThresholds {
    /// 이 온도 이상이면 스케줄링에서 제외한다.
    pub disable_temperature_c: f64,
    /// 이 온도부터 ECT 점수에 패널티를 준다.
    pub degraded_temperature_c: f64,
}

impl Default for PolicyThresholds {
    fn default() -> Self {
        Self {
            disable_temperature_c: 90.0,
            degraded_temperature_c: 80.0,
        }
    }
}

/// 후보 노드를 거른다.
///
/// 모든 정책이 이 필터를 공유한다. 정책별로 필터가 다르면 S3 정책 비교 실험이
/// 정책의 차이가 아니라 필터의 차이를 측정하게 된다.
pub fn candidates<'a>(
    task: &InferenceTask,
    nodes: &'a [NodeSnapshot],
    thresholds: PolicyThresholds,
) -> Vec<&'a NodeSnapshot> {
    let _ = task;
    nodes
        .iter()
        .filter(|n| n.state.is_schedulable())
        .filter(|n| n.has_model)
        .filter(|n| match n.temperature_c {
            // 온도 임계치를 넘은 노드는 상태와 무관하게 제외한다.
            // 계속 요청을 보내면 thermal throttling이 심해져 클러스터 전체
            // 처리량이 떨어진다.
            Some(t) => t < thresholds.disable_temperature_c,
            None => true,
        })
        .collect()
}

/// 노드 선택 정책.
pub trait SchedulingPolicy: Send + Sync {
    fn kind(&self) -> SchedulingPolicyKind;

    /// 후보 중 하나를 고른다.
    ///
    /// `candidates`는 비어 있지 않음이 보장된다.
    fn choose(&self, task: &InferenceTask, candidates: &[&NodeSnapshot]) -> NodeId;

    /// 필터링과 선택을 함께 수행하는 진입점.
    fn select_node(
        &self,
        task: &InferenceTask,
        nodes: &[NodeSnapshot],
        thresholds: PolicyThresholds,
    ) -> Result<NodeId, ScheduleError> {
        let candidates = candidates(task, nodes, thresholds);
        if candidates.is_empty() {
            return Err(ScheduleError::NoAvailableNode);
        }
        Ok(self.choose(task, &candidates))
    }
}

/// 설정에서 읽은 식별자로 정책 구현을 만든다.
pub fn policy_for(kind: SchedulingPolicyKind) -> Box<dyn SchedulingPolicy> {
    match kind {
        SchedulingPolicyKind::RoundRobin => Box::new(RoundRobin::default()),
        SchedulingPolicyKind::LeastQueue => Box::new(LeastQueue),
        SchedulingPolicyKind::Ect => Box::new(Ect::default()),
    }
}

// ---------------------------------------------------------------------------
// Round Robin
// ---------------------------------------------------------------------------

/// 순차 분배. 비교 기준으로만 사용한다.
///
/// 노드별 처리속도와 큐 상태를 반영하지 못하므로 노드 성능이 다르면
/// 느린 노드가 병목이 된다. 이 한계를 보여주는 것이 S3 실험의 목적이다.
#[derive(Debug, Default)]
pub struct RoundRobin {
    counter: AtomicUsize,
}

impl SchedulingPolicy for RoundRobin {
    fn kind(&self) -> SchedulingPolicyKind {
        SchedulingPolicyKind::RoundRobin
    }

    fn choose(&self, _task: &InferenceTask, candidates: &[&NodeSnapshot]) -> NodeId {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        candidates[n % candidates.len()].node_id.clone()
    }
}

// ---------------------------------------------------------------------------
// Least Queue
// ---------------------------------------------------------------------------

/// 가장 짧은 큐를 가진 노드를 선택한다.
///
/// 동률이면 in-flight, 평균 추론시간, Node ID 순으로 결정한다.
/// Node ID까지 내려가는 이유는 동률에서 순서가 흔들리면 실험이 재현되지 않기 때문이다.
#[derive(Debug)]
pub struct LeastQueue;

impl SchedulingPolicy for LeastQueue {
    fn kind(&self) -> SchedulingPolicyKind {
        SchedulingPolicyKind::LeastQueue
    }

    fn choose(&self, _task: &InferenceTask, candidates: &[&NodeSnapshot]) -> NodeId {
        candidates
            .iter()
            .min_by(|a, b| {
                // 1차 키는 **디스패치마다 갱신되는** 로컬 미완료 수다.
                // 하트비트 값(queue_depth/in_flight)을 1차로 쓰면 갱신 주기
                // 사이에 들어온 수백 건이 전부 같은 노드로 몰린다(S0-C).
                a.local_in_flight
                    .cmp(&b.local_in_flight)
                    .then_with(|| a.queue_depth.cmp(&b.queue_depth))
                    .then_with(|| a.in_flight.cmp(&b.in_flight))
                    .then_with(|| {
                        let ai = a.ewma_inference_ms.unwrap_or(f64::MAX);
                        let bi = b.ewma_inference_ms.unwrap_or(f64::MAX);
                        ai.partial_cmp(&bi).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| a.node_id.cmp(&b.node_id))
            })
            .expect("candidates가 비어 있지 않음이 보장된다")
            .node_id
            .clone()
    }
}

// ---------------------------------------------------------------------------
// Estimated Completion Time
// ---------------------------------------------------------------------------

/// ECT 점수 계산에 쓰는 계수.
#[derive(Debug, Clone, Copy)]
pub struct EctParams {
    /// 오류율 1.0에 해당하는 패널티(ms).
    ///
    /// 오류가 나면 재시도 비용이 추가로 들기 때문에 점수에 반영한다.
    pub error_penalty_ms: f64,
    /// degraded 온도에서 disable 온도까지 갔을 때 가해지는 최대 패널티(ms).
    pub thermal_penalty_ms: f64,
    /// EWMA 샘플이 하나도 없을 때 사용하는 추론시간 가정(ms).
    pub cold_start_inference_ms: f64,
}

impl Default for EctParams {
    fn default() -> Self {
        Self {
            error_penalty_ms: 200.0,
            thermal_penalty_ms: 100.0,
            cold_start_inference_ms: 50.0,
        }
    }
}

/// 예상 완료시간 기반 정책. 권장 기본값이다.
///
/// ```text
/// score = ((queue_depth + in_flight + 1) × EWMA_inference
///          + EWMA_network
///          + thermal_penalty
///          + error_penalty)
///         / load_factor
/// ```
///
/// `+ 1`은 지금 배정하려는 요청 자신이다. ECT는 "이 요청이 언제 끝나는가"를
/// 추정하는 것이므로 자기 추론시간을 포함해야 하고, 큐가 빈 노드의 점수가
/// 0이 되어 `load_factor`가 무력화되는 것도 막는다.
///
/// `load_factor`로 나눠, 복구 중이거나 성능이 떨어진 노드가 정상 노드보다
/// 덜 선택되게 한다.
#[derive(Debug, Default)]
pub struct Ect {
    pub params: EctParams,
    pub thresholds: PolicyThresholds,
}

impl Ect {
    /// 노드 하나의 점수. 낮을수록 좋다.
    pub fn score(&self, node: &NodeSnapshot, fallback_inference_ms: f64) -> f64 {
        let inference = node.ewma_inference_ms.unwrap_or(fallback_inference_ms);
        let network = node.ewma_network_ms.unwrap_or(0.0);

        // 지금 배정하려는 요청 자신을 포함해 센다.
        //
        // 대기열만 세면 큐가 빈 노드의 점수가 0이 되어, 뒤에서 곱하는
        // load_factor가 아무 효과를 내지 못한다. 복구 중인 유휴 노드가
        // 항상 선택되는 문제가 여기서 나왔다.
        //
        // 의미상으로도 이쪽이 맞다. ECT는 "이 요청이 언제 끝나는가"이므로
        // 자기 자신의 추론시간이 포함되어야 한다.
        //
        // **`local_in_flight` 를 쓴다.** 예전에는 `queue_depth + in_flight`
        // (둘 다 하트비트 관측값)를 썼는데, 1초짜리 stale 스냅샷을
        // dispatch-time 상태로 취급한 것이라 herding 이 생겼다(S0-C).
        // 하트비트 값을 더하지 않는 이유는 `NodeSnapshot::local_in_flight`
        // 주석 참조 — 같은 요청을 두 번 세게 된다.
        let pending = f64::from(node.local_in_flight) + 1.0;

        let thermal = match node.temperature_c {
            Some(t) if t > self.thresholds.degraded_temperature_c => {
                let span =
                    self.thresholds.disable_temperature_c - self.thresholds.degraded_temperature_c;
                let ratio = if span > 0.0 {
                    ((t - self.thresholds.degraded_temperature_c) / span).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                ratio * self.params.thermal_penalty_ms
            }
            _ => 0.0,
        };

        let error = node.error_rate.clamp(0.0, 1.0) * self.params.error_penalty_ms;

        let base = pending * inference + network + thermal + error;

        // load_factor가 낮은 노드는 같은 점수여도 덜 매력적으로 만든다.
        // Recovering(0.25) 노드는 사실상 큐가 비어 있어도 정상 노드와
        // 경쟁하지 않게 된다. PRD FR-07의 "제한된 요청만 할당" 요구를
        // 별도 카운터 없이 점수 하나로 구현하기 위한 것이다.
        let factor = node.state.load_factor();
        if factor <= 0.0 {
            f64::MAX
        } else {
            base / factor
        }
    }

    /// EWMA 샘플이 없는 노드에 적용할 추론시간 가정.
    ///
    /// 0으로 두면 신규 노드가 무한히 빠른 것으로 평가되어 요청이 몰리고,
    /// 크게 잡으면 신규 노드가 영원히 선택되지 않는다.
    /// 샘플이 있는 노드들의 평균을 쓰는 것이 두 실패를 모두 피한다.
    fn fallback_inference_ms(&self, candidates: &[&NodeSnapshot]) -> f64 {
        let known: Vec<f64> = candidates
            .iter()
            .filter_map(|n| n.ewma_inference_ms)
            .collect();
        if known.is_empty() {
            self.params.cold_start_inference_ms
        } else {
            known.iter().sum::<f64>() / known.len() as f64
        }
    }
}

impl SchedulingPolicy for Ect {
    fn kind(&self) -> SchedulingPolicyKind {
        SchedulingPolicyKind::Ect
    }

    fn choose(&self, _task: &InferenceTask, candidates: &[&NodeSnapshot]) -> NodeId {
        let fallback = self.fallback_inference_ms(candidates);
        candidates
            .iter()
            .min_by(|a, b| {
                let sa = self.score(a, fallback);
                let sb = self.score(b, fallback);
                sa.partial_cmp(&sb)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    // 동점에서 순서가 흔들리면 실험이 재현되지 않는다.
                    .then_with(|| a.node_id.cmp(&b.node_id))
            })
            .expect("candidates가 비어 있지 않음이 보장된다")
            .node_id
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use npuforge_common::protocol::{InputFormat, priority};
    use std::time::Instant;

    fn task() -> InferenceTask {
        InferenceTask {
            request_id: uuid::Uuid::new_v4(),
            model_id: "yolov8n".into(),
            payload: bytes::Bytes::new(),
            input_format: InputFormat::Jpeg,
            priority: priority::NORMAL,
            deadline: None,
            created_at: Instant::now(),
            attempt: 0,
            max_attempts: 2,
        }
    }

    fn three_healthy() -> Vec<NodeSnapshot> {
        vec![
            NodeSnapshot::healthy("king"),
            NodeSnapshot::healthy("queen"),
            NodeSnapshot::healthy("jack"),
        ]
    }

    // -- 후보 필터 ----------------------------------------------------------

    #[test]
    fn drained_and_disabled_nodes_are_excluded() {
        // 벤치마크에서 노드 수를 바꾸는 방식이 이 동작에 의존한다.
        let mut nodes = three_healthy();
        nodes[1].state = NodeState::Draining;
        nodes[2].state = NodeState::Disabled;

        let c = candidates(&task(), &nodes, PolicyThresholds::default());
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].node_id, "king");
    }

    #[test]
    fn nodes_without_the_model_are_excluded() {
        let mut nodes = three_healthy();
        nodes[0].has_model = false;
        let c = candidates(&task(), &nodes, PolicyThresholds::default());
        assert_eq!(c.len(), 2);
        assert!(c.iter().all(|n| n.node_id != "king"));
    }

    #[test]
    fn overheated_nodes_are_excluded() {
        let mut nodes = three_healthy();
        nodes[0].temperature_c = Some(91.0);
        nodes[1].temperature_c = Some(85.0);
        nodes[2].temperature_c = None;

        let c = candidates(&task(), &nodes, PolicyThresholds::default());
        assert_eq!(c.len(), 2, "90도를 넘은 노드만 빠져야 한다");
        assert!(c.iter().all(|n| n.node_id != "king"));
    }

    #[test]
    fn all_policies_share_the_same_filter() {
        // 정책마다 필터가 다르면 S3 실험이 정책이 아니라 필터를 비교하게 된다.
        let mut nodes = three_healthy();
        nodes[0].state = NodeState::Disabled;
        nodes[1].has_model = false;

        for kind in SchedulingPolicyKind::ALL {
            let policy = policy_for(kind);
            let chosen = policy
                .select_node(&task(), &nodes, PolicyThresholds::default())
                .unwrap();
            assert_eq!(chosen, "jack", "{kind} 정책이 다른 후보를 골랐다");
        }
    }

    #[test]
    fn no_candidates_is_an_error() {
        let nodes = vec![NodeSnapshot {
            state: NodeState::Unreachable,
            ..NodeSnapshot::healthy("king")
        }];
        for kind in SchedulingPolicyKind::ALL {
            let err = policy_for(kind)
                .select_node(&task(), &nodes, PolicyThresholds::default())
                .unwrap_err();
            assert_eq!(err, ScheduleError::NoAvailableNode);
        }
    }

    // -- Round Robin --------------------------------------------------------

    #[test]
    fn round_robin_distributes_evenly() {
        let nodes = three_healthy();
        let policy = RoundRobin::default();
        let mut counts = std::collections::HashMap::new();
        for _ in 0..300 {
            let id = policy
                .select_node(&task(), &nodes, PolicyThresholds::default())
                .unwrap();
            *counts.entry(id).or_insert(0) += 1;
        }
        for node in &nodes {
            assert_eq!(counts[&node.node_id], 100);
        }
    }

    #[test]
    fn round_robin_ignores_load() {
        // 이것이 Round Robin의 한계이며 S3 실험이 보여주려는 지점이다.
        let mut nodes = three_healthy();
        nodes[0].queue_depth = 100;
        let policy = RoundRobin::default();

        let mut hit_busy = 0;
        for _ in 0..30 {
            if policy
                .select_node(&task(), &nodes, PolicyThresholds::default())
                .unwrap()
                == "king"
            {
                hit_busy += 1;
            }
        }
        assert_eq!(hit_busy, 10, "큐가 100이어도 1/3을 받는다");
    }

    // -- Least Queue --------------------------------------------------------

    #[test]
    fn least_queue_picks_shortest_queue() {
        let mut nodes = three_healthy();
        nodes[0].queue_depth = 5;
        nodes[1].queue_depth = 1;
        nodes[2].queue_depth = 3;

        let chosen = LeastQueue
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        assert_eq!(chosen, "queen");
    }

    #[test]
    fn least_queue_breaks_ties_deterministically() {
        // 동률 처리가 흔들리면 같은 조건의 반복 실험 결과가 달라진다.
        let nodes = three_healthy();
        let first = LeastQueue
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        for _ in 0..50 {
            let again = LeastQueue
                .select_node(&task(), &nodes, PolicyThresholds::default())
                .unwrap();
            assert_eq!(first, again);
        }
        // 사전순이므로 jack < king < queen. 등록 순서가 아니라 ID 순이다.
        assert_eq!(first, "jack", "동률이면 Node ID 사전순");
    }

    #[test]
    fn least_queue_uses_in_flight_as_first_tiebreak() {
        let mut nodes = three_healthy();
        nodes[0].in_flight = 4;
        nodes[1].in_flight = 1;
        nodes[2].in_flight = 2;

        let chosen = LeastQueue
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        assert_eq!(chosen, "queen");
    }

    // -- ECT ----------------------------------------------------------------

    #[test]
    fn ect_prefers_faster_node_at_equal_queue() {
        // Least Queue가 구분하지 못하는 상황. ECT의 존재 이유다.
        let mut nodes = three_healthy();
        for n in &mut nodes {
            n.queue_depth = 4;
        }
        nodes[0].ewma_inference_ms = Some(40.0);
        nodes[1].ewma_inference_ms = Some(10.0);
        nodes[2].ewma_inference_ms = Some(25.0);

        let chosen = Ect::default()
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        assert_eq!(chosen, "queen");
    }

    #[test]
    fn ect_uses_local_in_flight_not_heartbeat() {
        // 하트비트 값(queue_depth/in_flight)은 최대 1초 stale 이라
        // dispatch-time 결정에 쓸 수 없다. ECT 는 로컬 미완료 수를 봐야 한다.
        // 이 테스트가 그것을 고정한다 — 예전 구현은 하트비트 합을 썼고,
        // 초당 수백 건이 같은 고정 스냅샷을 보고 몰렸다 (S0-C, 처리량 −55%).
        let mut nodes = three_healthy();
        for n in &mut nodes {
            n.ewma_inference_ms = Some(20.0);
        }
        // 하트비트 값은 king 이 가장 한가해 보이게 둔다.
        nodes[0].queue_depth = 0;
        nodes[0].in_flight = 0;
        nodes[1].queue_depth = 9;
        nodes[1].in_flight = 9;
        nodes[2].queue_depth = 9;
        nodes[2].in_flight = 9;
        // 그러나 스케줄러가 실제로 보낸 미완료는 king 이 가장 많다.
        nodes[0].local_in_flight = 8;
        nodes[1].local_in_flight = 1;
        nodes[2].local_in_flight = 5;

        let chosen = Ect::default()
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        assert_eq!(
            chosen, "queen",
            "하트비트로는 king 이 한가해 보이지만 로컬 미완료가 가장 적은 것은 queen"
        );
    }

    #[test]
    fn least_queue_uses_local_in_flight_first() {
        let mut nodes = three_healthy();
        // 하트비트 기준으로는 king 이 가장 한가하다.
        nodes[0].queue_depth = 0;
        nodes[1].queue_depth = 5;
        nodes[2].queue_depth = 5;
        // 실제 미완료는 jack 이 가장 적다.
        nodes[0].local_in_flight = 7;
        nodes[1].local_in_flight = 6;
        nodes[2].local_in_flight = 2;

        let chosen = LeastQueue
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        assert_eq!(chosen, "jack", "1차 키는 로컬 미완료 수여야 한다");
    }

    #[test]
    fn ect_penalizes_hot_nodes() {
        let mut nodes = vec![NodeSnapshot::healthy("cool"), NodeSnapshot::healthy("hot")];
        for n in &mut nodes {
            n.queue_depth = 1;
            n.ewma_inference_ms = Some(20.0);
        }
        nodes[0].temperature_c = Some(60.0);
        nodes[1].temperature_c = Some(88.0); // degraded는 넘었지만 disable 미만

        let ect = Ect::default();
        assert!(ect.score(&nodes[1], 20.0) > ect.score(&nodes[0], 20.0));
        assert_eq!(
            ect.select_node(&task(), &nodes, PolicyThresholds::default())
                .unwrap(),
            "cool"
        );
    }

    #[test]
    fn ect_penalizes_error_prone_nodes() {
        let mut nodes = vec![
            NodeSnapshot::healthy("clean"),
            NodeSnapshot::healthy("flaky"),
        ];
        for n in &mut nodes {
            n.queue_depth = 1;
            n.ewma_inference_ms = Some(20.0);
        }
        nodes[1].error_rate = 0.5;

        let chosen = Ect::default()
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        assert_eq!(chosen, "clean");
    }

    #[test]
    fn ect_accounts_for_network_time() {
        let mut nodes = vec![NodeSnapshot::healthy("near"), NodeSnapshot::healthy("far")];
        for n in &mut nodes {
            n.queue_depth = 1;
            n.ewma_inference_ms = Some(20.0);
        }
        nodes[0].ewma_network_ms = Some(1.0);
        nodes[1].ewma_network_ms = Some(50.0);

        let chosen = Ect::default()
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        assert_eq!(chosen, "near");
    }

    #[test]
    fn recovering_node_loses_to_healthy_even_when_idle() {
        // FR-07: 복구 직후에는 제한된 요청만 할당해야 한다.
        // 복구 노드는 큐가 비어 있으므로 점수만 보면 항상 이긴다.
        let mut busy = NodeSnapshot::healthy("healthy");
        busy.queue_depth = 2;
        busy.ewma_inference_ms = Some(20.0);

        let mut recovering = NodeSnapshot::healthy("recovering");
        recovering.state = NodeState::Recovering;
        recovering.queue_depth = 0;
        recovering.ewma_inference_ms = Some(20.0);

        let nodes = vec![busy, recovering];
        let chosen = Ect::default()
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        assert_eq!(chosen, "healthy", "복구 중인 노드에 요청이 몰리면 안 된다");
    }

    #[test]
    fn recovering_node_is_used_when_it_is_the_only_candidate() {
        let mut recovering = NodeSnapshot::healthy("recovering");
        recovering.state = NodeState::Recovering;
        let nodes = vec![recovering];

        let chosen = Ect::default()
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        assert_eq!(chosen, "recovering");
    }

    #[test]
    fn new_node_without_samples_is_not_treated_as_infinitely_fast() {
        // EWMA 초기값을 0으로 두면 신규 노드가 모든 요청을 빨아들인다.
        let mut known = NodeSnapshot::healthy("known");
        known.queue_depth = 1;
        known.ewma_inference_ms = Some(20.0);

        let mut fresh = NodeSnapshot::healthy("fresh");
        fresh.queue_depth = 1;
        fresh.ewma_inference_ms = None;

        let ect = Ect::default();
        let cands = vec![&known, &fresh];
        let fallback = ect.fallback_inference_ms(&cands);
        assert_eq!(fallback, 20.0, "샘플이 있는 노드들의 평균을 써야 한다");
        assert_eq!(ect.score(&fresh, fallback), ect.score(&known, fallback));
    }

    #[test]
    fn cold_cluster_falls_back_to_configured_assumption() {
        let a = NodeSnapshot::healthy("a");
        let b = NodeSnapshot::healthy("b");
        let ect = Ect::default();
        let cands = vec![&a, &b];
        assert_eq!(
            ect.fallback_inference_ms(&cands),
            ect.params.cold_start_inference_ms
        );
    }

    #[test]
    fn ect_is_deterministic_for_identical_nodes() {
        let nodes = three_healthy();
        let ect = Ect::default();
        let first = ect
            .select_node(&task(), &nodes, PolicyThresholds::default())
            .unwrap();
        for _ in 0..50 {
            assert_eq!(
                first,
                ect.select_node(&task(), &nodes, PolicyThresholds::default())
                    .unwrap()
            );
        }
    }

    #[test]
    fn policy_kind_matches_constructor() {
        for kind in SchedulingPolicyKind::ALL {
            assert_eq!(policy_for(kind).kind(), kind);
        }
    }
}
