//! `SchedulerService` gRPC 서버.
//!
//! 클라이언트 추론 요청과 노드 등록/하트비트를 모두 받는다.
//! `docs/01-TECHSPEC.md` §7 참조.

use std::sync::Arc;
use std::time::{Duration, Instant};

use npuforge_common::{
    ErrorCode, ModelId, NodeId, RequestId,
    config::SchedulerConfig,
    protocol::{InferenceTask, InputFormat, TimingBreakdown},
};
use npuforge_proto::v1::{
    self, DeregisterNodeRequest, DeregisterNodeResponse, HeartbeatRequest, HeartbeatResponse,
    InferRequest, InferResponse, ListNodesRequest, ListNodesResponse, RegisterNodeRequest,
    RegisterNodeResponse, scheduler_service_server::SchedulerService,
};
use tonic::{Request, Response, Status};

use crate::{
    node_client::{RetryBackoff, SharedNodeLink},
    policy::{PolicyThresholds, ScheduleError, SchedulingPolicy},
    registry::{HeartbeatObservation, Registry},
};

pub struct SchedulerServer {
    pub registry: Arc<Registry>,
    pub policy: Arc<dyn SchedulingPolicy>,
    pub nodes: SharedNodeLink,
    pub config: Arc<SchedulerConfig>,
    /// 노드 등록 토큰. 비어 있으면 검증하지 않는다.
    pub token: String,
}

impl SchedulerServer {
    pub fn new(
        registry: Arc<Registry>,
        policy: Arc<dyn SchedulingPolicy>,
        nodes: SharedNodeLink,
        config: Arc<SchedulerConfig>,
        token: String,
    ) -> Self {
        Self {
            registry,
            policy,
            nodes,
            config,
            token,
        }
    }

    fn thresholds(&self) -> PolicyThresholds {
        PolicyThresholds {
            disable_temperature_c: self.config.thresholds.disable_temperature_c,
            degraded_temperature_c: self.config.thresholds.degraded_temperature_c,
        }
    }

    fn backoff(&self) -> RetryBackoff {
        RetryBackoff {
            min: Duration::from_millis(self.config.scheduler.retry_backoff_min_ms),
            max: Duration::from_millis(self.config.scheduler.retry_backoff_max_ms),
        }
    }

    /// 요청 하나를 처리한다. 재시도를 포함한다.
    ///
    /// `request_id` 는 클라이언트가 보낸 문자열이다. 응답과 노드 요청에
    /// 그대로 실어 클라이언트가 자기 로그와 대조할 수 있게 한다.
    /// 내부 `task.request_id`(UUID)와 다를 수 있다.
    async fn dispatch(&self, mut task: InferenceTask, request_id: String) -> InferResponse {
        // 이미 시도한 노드는 다시 고르지 않는다. 같은 노드가 연속으로 뽑히면
        // 재시도가 사실상 무의미해진다(특히 round-robin 외 정책에서).
        let started = Instant::now();
        let mut tried: Vec<NodeId> = Vec::new();
        let mut last_error = (ErrorCode::NoAvailableNode, String::new());

        loop {
            let route_start = Instant::now();
            // 선택과 예약을 한 임계구역에서 처리한다. 예약을 나중에 하면
            // 동시에 들어온 요청들이 모두 예약 전 상태를 보고 같은 노드를
            // 고른다 — 하트비트 stale 을 고쳐도 herding 이 남는다.
            //
            // `_reservation` 은 이 반복(iteration)이 끝날 때 drop 되며,
            // 그때 local_in_flight 가 감소한다. 성공 return, 오류 return,
            // 재시도 continue — **모든 종료 경로가 자동으로 해제된다.**
            // 이름을 `_` 하나로 두면 즉시 drop 되므로 반드시 이름을 준다.
            let picked = self
                .registry
                .select_and_reserve(&task.model_id, &tried, |snaps| {
                    self.policy.select_node(&task, snaps, self.thresholds())
                });

            let (node_id, _reservation) = match picked {
                Ok(v) => v,
                Err(ScheduleError::NoAvailableNode) => {
                    // 첫 시도부터 후보가 없으면 그 사실을, 재시도 중이면
                    // 직전 노드의 실패 이유를 돌려준다. 후자가 더 유용하다.
                    let (code, msg) = if tried.is_empty() {
                        (
                            ErrorCode::NoAvailableNode,
                            format!("모델 '{}' 을 처리할 노드가 없습니다", task.model_id),
                        )
                    } else {
                        (
                            last_error.0,
                            format!(
                                "{} (시도한 노드: {})",
                                last_error.1,
                                tried
                                    .iter()
                                    .map(|n| n.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        )
                    };
                    return error_response(&request_id, code, msg, task.attempt);
                }
            };
            let route_us = route_start.elapsed().as_micros() as u64;

            let Some(address) = self.registry.address_of(&node_id) else {
                // 선택과 주소 조회 사이에 등록이 해제되었다. 다음 후보로 넘어간다.
                tried.push(node_id);
                continue;
            };

            let node_req = v1::NodeInferRequest {
                request_id: request_id.clone(),
                model_id: task.model_id.to_string(),
                payload: task.payload.to_vec(),
                input_format: v1::InputFormat::from(task.input_format) as i32,
                queue_timeout_ms: self.config.scheduler.node_rpc_timeout_ms,
            };

            let call_start = Instant::now();
            let outcome = self.nodes.infer_boxed(&node_id, &address, node_req).await;
            let call_elapsed = call_start.elapsed();

            match outcome {
                Ok(resp) if resp.error_code == ErrorCode::Ok.as_str() => {
                    let node_timing: TimingBreakdown =
                        resp.timing.map(Into::into).unwrap_or_default();
                    // 네트워크 시간 = 왕복 전체 − 노드가 보고한 내부 처리시간.
                    // 서로 다른 장치의 절대 시각을 빼지 않는다는 원칙을 지키려면
                    // 이 방법뿐이다. `docs/02-HARDWARE-SETUP.md` §10.1 참조.
                    let node_internal_us = node_timing.node_total_us();
                    let network_us =
                        (call_elapsed.as_micros() as u64).saturating_sub(node_internal_us);

                    self.registry.record_inference(
                        &node_id,
                        node_timing.inference_us as f64 / 1000.0,
                        network_us as f64 / 1000.0,
                    );

                    let mut timing = node_timing;
                    timing.scheduler_route_us = route_us;
                    timing.network_to_node_us = network_us / 2;
                    timing.network_to_client_us = network_us / 2;
                    timing.end_to_end_us = started.elapsed().as_micros() as u64;

                    return InferResponse {
                        request_id,
                        node_id: node_id.to_string(),
                        result: resp.result,
                        result_format: resp.result_format,
                        timing: Some(timing.into()),
                        error_code: ErrorCode::Ok.as_str().to_owned(),
                        error_message: String::new(),
                        attempts: task.attempt + 1,
                    };
                }
                Ok(resp) => {
                    // 노드가 애플리케이션 오류를 돌려줬다.
                    let code = ErrorCode::from_str_code(&resp.error_code)
                        .unwrap_or(ErrorCode::InferenceFailed);
                    last_error = (code, resp.error_message);
                    if !code.is_retryable() {
                        return error_response(
                            &request_id,
                            code,
                            last_error.1.clone(),
                            task.attempt,
                        );
                    }
                }
                Err(transport) => {
                    // 전송 실패는 노드 장애로 본다. 헬스 카운터에 반영해야
                    // 다음 요청부터 이 노드를 피할 수 있다.
                    self.registry.record_health_failure(&node_id);
                    last_error = (ErrorCode::NodeUnavailable, transport);
                }
            }

            tried.push(node_id);
            if !task.can_retry() {
                return error_response(&request_id, last_error.0, last_error.1, task.attempt + 1);
            }
            // 데드라인이 이미 지났으면 재시도해도 버려진다.
            if let Some(deadline) = task.deadline
                && Instant::now() >= deadline
            {
                return error_response(
                    &request_id,
                    ErrorCode::DeadlineUnsatisfiable,
                    "재시도 전에 데드라인을 초과했습니다".to_owned(),
                    task.attempt + 1,
                );
            }

            let delay = self.backoff().delay(task.attempt);
            task.attempt += 1;
            tokio::time::sleep(delay).await;
        }
    }
}

#[tonic::async_trait]
impl SchedulerService for SchedulerServer {
    async fn infer(
        &self,
        request: Request<InferRequest>,
    ) -> std::result::Result<Response<InferResponse>, Status> {
        let req = request.into_inner();

        let request_id = if req.request_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            req.request_id
        };

        let Some(input_format) = v1::InputFormat::try_from(req.input_format)
            .ok()
            .and_then(|f| InputFormat::try_from(f).ok())
        else {
            return Ok(Response::new(error_response(
                &request_id,
                ErrorCode::UnsupportedInputFormat,
                "input_format 이 지정되지 않았거나 지원하지 않습니다".to_owned(),
                0,
            )));
        };

        if req.payload.is_empty() {
            return Ok(Response::new(error_response(
                &request_id,
                ErrorCode::InvalidRequest,
                "payload 가 비어 있습니다".to_owned(),
                0,
            )));
        }

        let max = self.config.server.max_payload_bytes;
        if req.payload.len() as u64 > max {
            return Ok(Response::new(error_response(
                &request_id,
                ErrorCode::PayloadTooLarge,
                format!(
                    "payload {} 바이트가 상한 {max} 을 넘습니다",
                    req.payload.len()
                ),
                0,
            )));
        }

        let now = Instant::now();
        // 클라이언트가 준 절대 시각(unix ms)을 스케줄러의 monotonic clock 으로
        // 옮긴다. 장치 간 시계 차이는 NTP 동기화 오차만큼 남는다.
        let deadline = (req.deadline_unix_ms > 0)
            .then(|| unix_ms_to_instant(req.deadline_unix_ms, now))
            .flatten();

        let task = InferenceTask {
            // 외부 request_id 는 클라이언트가 정하는 임의 문자열이고,
            // 내부 `RequestId` 는 UUID 다. 클라이언트가 UUID 를 줬으면 그대로
            // 쓰고, 아니면 내부용으로 새로 만든다. 응답과 노드 요청에는
            // 클라이언트가 보낸 문자열을 그대로 돌려준다.
            request_id: RequestId::parse_str(&request_id).unwrap_or_else(|_| RequestId::new_v4()),
            model_id: ModelId::from(req.model_id),
            payload: bytes::Bytes::from(req.payload),
            input_format,
            priority: req.priority,
            deadline,
            created_at: now,
            attempt: 0,
            max_attempts: self.config.scheduler.max_retries + 1,
        };

        let timeout = Duration::from_millis(self.config.scheduler.request_timeout_ms);
        match tokio::time::timeout(timeout, self.dispatch(task, request_id.clone())).await {
            Ok(resp) => Ok(Response::new(resp)),
            Err(_) => Ok(Response::new(error_response(
                &request_id,
                ErrorCode::RequestTimeout,
                format!("요청이 {} ms 안에 완료되지 않았습니다", timeout.as_millis()),
                0,
            ))),
        }
    }

    async fn register_node(
        &self,
        request: Request<RegisterNodeRequest>,
    ) -> std::result::Result<Response<RegisterNodeResponse>, Status> {
        let req = request.into_inner();

        if !self.token.is_empty() && req.token != self.token {
            return Ok(Response::new(RegisterNodeResponse {
                accepted: false,
                error_code: ErrorCode::Unauthorized.as_str().to_owned(),
                error_message: "등록 토큰이 일치하지 않습니다".into(),
                heartbeat_interval_ms: 0,
            }));
        }

        let Some(descriptor) = req.descriptor else {
            return Ok(Response::new(RegisterNodeResponse {
                accepted: false,
                error_code: ErrorCode::InvalidRequest.as_str().to_owned(),
                error_message: "descriptor 가 없습니다".into(),
                heartbeat_interval_ms: 0,
            }));
        };

        let descriptor = match npuforge_common::protocol::NodeDescriptor::try_from(descriptor) {
            Ok(d) => d,
            Err(e) => {
                return Ok(Response::new(RegisterNodeResponse {
                    accepted: false,
                    error_code: ErrorCode::InvalidRequest.as_str().to_owned(),
                    error_message: e.to_string(),
                    heartbeat_interval_ms: 0,
                }));
            }
        };

        if descriptor.address.is_empty() {
            // 주소가 없으면 스케줄러가 이 노드를 호출할 방법이 없다.
            return Ok(Response::new(RegisterNodeResponse {
                accepted: false,
                error_code: ErrorCode::InvalidRequest.as_str().to_owned(),
                error_message: "advertise_address 가 비어 있습니다".into(),
                heartbeat_interval_ms: 0,
            }));
        }

        tracing::info!(
            event = "node_registered",
            node_id = %descriptor.node_id,
            address = %descriptor.address,
            agent_version = %descriptor.agent_version,
            runtime_version = %descriptor.runtime_version,
            models = descriptor.models.len(),
            "노드 등록"
        );
        self.registry.register(descriptor);

        Ok(Response::new(RegisterNodeResponse {
            accepted: true,
            error_code: ErrorCode::Ok.as_str().to_owned(),
            error_message: String::new(),
            heartbeat_interval_ms: self.config.health.heartbeat_interval_ms as u32,
        }))
    }

    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> std::result::Result<Response<HeartbeatResponse>, Status> {
        let req = request.into_inner();
        let node_id = NodeId::from(req.node_id);
        let health = req.health.unwrap_or_default();

        let obs = HeartbeatObservation {
            boot_id: health.boot_id.clone(),
            input_voltage_v: health.has_input_voltage.then_some(health.input_voltage_v),
            health: health.into(),
        };

        let outcome = self.registry.record_heartbeat(&node_id, obs);

        if outcome.rebooted {
            // 벤치마크 중이라면 이 시점 이후의 결과는 별도 run 으로 취급해야 한다.
            tracing::warn!(
                event = "node_rebooted",
                node_id = %node_id,
                reboot_count = self.registry.reboot_count(&node_id).unwrap_or(0),
                "노드 재부팅이 관측되었습니다. 진행 중인 벤치마크 run 은 무효입니다"
            );
        }
        if outcome.must_reregister {
            tracing::info!(
                node_id = %node_id,
                "등록되지 않은 노드의 하트비트. 재등록을 요구한다"
            );
        }

        let state = self
            .registry
            .state_of(&node_id)
            .map(|s| v1::NodeState::from(s) as i32)
            .unwrap_or(v1::NodeState::Unspecified as i32);

        Ok(Response::new(HeartbeatResponse {
            accepted: !outcome.must_reregister,
            state,
            must_reregister: outcome.must_reregister,
        }))
    }

    async fn list_nodes(
        &self,
        _request: Request<ListNodesRequest>,
    ) -> std::result::Result<Response<ListNodesResponse>, Status> {
        // 읽기 전용이다. 레지스트리를 바꾸지 않는다.
        let nodes = self
            .registry
            .list()
            .into_iter()
            .map(|v| {
                let mut health = v1::NodeHealth::from(v.health);
                // NodeHealth 에 없는 항목을 여기서 채운다.
                health.boot_id = v.boot_id;
                health.has_input_voltage = v.input_voltage_v.is_some();
                health.input_voltage_v = v.input_voltage_v.unwrap_or(0.0);

                v1::NodeStatus {
                    descriptor: Some(v.descriptor.into()),
                    state: v1::NodeState::from(v.state) as i32,
                    health: Some(health),
                    reboot_count: v.reboot_count,
                    has_ewma_inference_ms: v.ewma_inference_ms.is_some(),
                    ewma_inference_ms: v.ewma_inference_ms.unwrap_or(0.0),
                    has_ewma_network_ms: v.ewma_network_ms.is_some(),
                    ewma_network_ms: v.ewma_network_ms.unwrap_or(0.0),
                }
            })
            .collect();

        Ok(Response::new(ListNodesResponse {
            nodes,
            policy: self.policy.kind().to_string(),
        }))
    }

    async fn deregister_node(
        &self,
        request: Request<DeregisterNodeRequest>,
    ) -> std::result::Result<Response<DeregisterNodeResponse>, Status> {
        let req = request.into_inner();
        let node_id = NodeId::from(req.node_id);
        let removed = self.registry.deregister(&node_id);
        if removed {
            tracing::info!(
                event = "node_deregistered",
                node_id = %node_id,
                reason = %req.reason,
                "노드 등록 해제"
            );
        }
        Ok(Response::new(DeregisterNodeResponse { accepted: removed }))
    }
}

fn error_response(
    request_id: &str,
    code: ErrorCode,
    message: String,
    attempts: u32,
) -> InferResponse {
    InferResponse {
        request_id: request_id.to_owned(),
        node_id: String::new(),
        result: Vec::new(),
        result_format: String::new(),
        timing: None,
        error_code: code.as_str().to_owned(),
        error_message: message,
        attempts,
    }
}

/// unix ms 데드라인을 스케줄러의 monotonic 시각으로 옮긴다.
///
/// 이미 지난 데드라인은 `None` 이 아니라 `now` 로 만든다. 그래야 요청이
/// "데드라인 초과"로 즉시 거부되고, 조용히 데드라인 없이 처리되지 않는다.
fn unix_ms_to_instant(deadline_unix_ms: u64, now: Instant) -> Option<Instant> {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;
    Some(if deadline_unix_ms > now_ms {
        now + Duration::from_millis(deadline_unix_ms - now_ms)
    } else {
        now
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use npuforge_common::{
        model::{ModelDescriptor, ModelState},
        policy::SchedulingPolicyKind,
        protocol::NodeDescriptor,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// 노드 호출을 흉내낸다. 노드별 응답 스크립트를 미리 넣어 둔다.
    #[derive(Default)]
    struct FakeNodes {
        script: Mutex<HashMap<NodeId, Vec<Result<v1::NodeInferResponse, String>>>>,
        calls: Mutex<Vec<NodeId>>,
    }

    impl FakeNodes {
        fn with(pairs: &[(&str, &[Outcome])]) -> Arc<Self> {
            let mut m = HashMap::new();
            for (k, v) in pairs {
                m.insert(NodeId::from(*k), v.iter().map(Outcome::build).collect());
            }
            Arc::new(Self {
                script: Mutex::new(m),
                calls: Mutex::new(Vec::new()),
            })
        }
        fn calls(&self) -> Vec<NodeId> {
            self.calls.lock().unwrap().clone()
        }
    }

    /// 테스트 스크립트에 넣는 노드 응답.
    #[derive(Debug, Clone, Copy)]
    enum Outcome {
        Ok,
        /// 노드가 애플리케이션 오류를 반환했다.
        AppError(ErrorCode),
        /// 전송 실패. 노드 다운이나 네트워크 단절.
        Transport,
    }

    impl Outcome {
        fn build(&self) -> Result<v1::NodeInferResponse, String> {
            match self {
                Outcome::Ok => Ok(v1::NodeInferResponse {
                    request_id: String::new(),
                    result: b"det".to_vec(),
                    result_format: "yolo-detections".into(),
                    timing: Some(v1::Timing {
                        inference_us: 13_000,
                        ..Default::default()
                    }),
                    error_code: ErrorCode::Ok.as_str().to_owned(),
                    error_message: String::new(),
                }),
                Outcome::AppError(code) => Ok(v1::NodeInferResponse {
                    request_id: String::new(),
                    result: Vec::new(),
                    result_format: String::new(),
                    timing: None,
                    error_code: code.as_str().to_owned(),
                    error_message: "테스트 오류".into(),
                }),
                Outcome::Transport => Err("transport error".into()),
            }
        }
    }

    impl crate::node_client::NodeLink for FakeNodes {
        async fn infer(
            &self,
            node_id: &NodeId,
            _address: &str,
            _req: v1::NodeInferRequest,
        ) -> Result<v1::NodeInferResponse, String> {
            self.calls.lock().unwrap().push(node_id.clone());
            let mut script = self.script.lock().unwrap();
            match script.get_mut(node_id) {
                Some(v) if !v.is_empty() => v.remove(0),
                _ => Err("스크립트에 없는 노드".into()),
            }
        }
    }

    fn config(max_retries: u32) -> Arc<SchedulerConfig> {
        let mut c = SchedulerConfig::from_path("../../configs/scheduler.example.toml")
            .expect("예제 설정이 로딩되어야 한다");
        c.scheduler.policy = SchedulingPolicyKind::LeastQueue;
        c.scheduler.max_retries = max_retries;
        // 테스트가 백오프만큼 기다리지 않게 한다
        c.scheduler.retry_backoff_min_ms = 1;
        c.scheduler.retry_backoff_max_ms = 2;
        Arc::new(c)
    }

    fn descriptor(node_id: &str) -> NodeDescriptor {
        NodeDescriptor {
            node_id: node_id.into(),
            hostname: node_id.into(),
            address: format!("http://{node_id}.local:50052"),
            device_type: "nanopi-r76s".into(),
            npu_type: "rk3576".into(),
            npu_core_count: 2,
            memory_bytes: 4 << 30,
            agent_version: "0.1.0".into(),
            runtime_version: "2.3.0".into(),
            models: vec![ModelDescriptor {
                id: "yolov8n".into(),
                version: "1.0.0".into(),
                state: ModelState::Ready,
                sha256: "0".repeat(64),
            }],
        }
    }

    /// 노드들을 등록하고 Healthy 상태까지 올린 스케줄러를 만든다.
    fn server(nodes: Arc<FakeNodes>, node_ids: &[&str], max_retries: u32) -> SchedulerServer {
        let cfg = config(max_retries);
        let registry = Arc::new(Registry::new(cfg.health.clone(), cfg.thresholds.clone()));
        for id in node_ids {
            registry.register(descriptor(id));
            // 등록 직후는 Registering 이라 스케줄 대상이 아니다.
            // 하트비트를 한 번 받아야 Healthy 로 올라간다.
            registry.record_heartbeat(
                &NodeId::from(*id),
                HeartbeatObservation {
                    boot_id: format!("boot-{id}"),
                    ..Default::default()
                },
            );
        }
        SchedulerServer::new(
            registry,
            Arc::from(crate::policy_for(cfg.scheduler.policy)),
            nodes,
            cfg,
            String::new(),
        )
    }

    fn infer_req(model: &str) -> Request<InferRequest> {
        Request::new(InferRequest {
            request_id: "client-req-1".into(),
            model_id: model.into(),
            payload: b"jpeg-bytes".to_vec(),
            input_format: v1::InputFormat::Jpeg as i32,
            priority: 0,
            deadline_unix_ms: 0,
            metadata: HashMap::new(),
        })
    }

    #[tokio::test]
    async fn successful_request_reports_node_and_timing() {
        let nodes = FakeNodes::with(&[("king", &[Outcome::Ok])]);
        let s = server(Arc::clone(&nodes), &["king"], 1);
        let r = s.infer(infer_req("yolov8n")).await.unwrap().into_inner();

        assert_eq!(r.error_code, ErrorCode::Ok.as_str());
        assert_eq!(r.node_id, "king");
        assert_eq!(r.attempts, 1);
        // 클라이언트가 준 request_id 를 그대로 돌려줘야 로그 대조가 가능하다
        assert_eq!(r.request_id, "client-req-1");
        let t = r.timing.expect("타이밍이 있어야 한다");
        assert_eq!(t.inference_us, 13_000, "노드가 보고한 값을 보존해야 한다");
        assert!(t.end_to_end_us > 0);
    }

    #[tokio::test]
    async fn transport_failure_retries_on_a_different_node() {
        // king 이 죽었을 때 queen 으로 넘어가는지. 이 프로젝트의 핵심 시나리오다.
        let nodes = FakeNodes::with(&[("king", &[Outcome::Transport]), ("queen", &[Outcome::Ok])]);
        let s = server(Arc::clone(&nodes), &["king", "queen"], 1);
        let r = s.infer(infer_req("yolov8n")).await.unwrap().into_inner();

        assert_eq!(r.error_code, ErrorCode::Ok.as_str());
        assert_eq!(r.attempts, 2);
        let calls = nodes.calls();
        assert_eq!(calls.len(), 2);
        assert_ne!(calls[0], calls[1], "같은 노드로 재시도하면 안 된다");
    }

    #[tokio::test]
    async fn non_retryable_error_is_returned_immediately() {
        // 잘못된 요청은 다른 노드로 보내도 같은 결과다. 부하만 늘린다.
        let nodes = FakeNodes::with(&[
            ("king", &[Outcome::AppError(ErrorCode::ModelNotFound)]),
            ("queen", &[Outcome::Ok]),
        ]);
        let s = server(Arc::clone(&nodes), &["king", "queen"], 1);
        let r = s.infer(infer_req("yolov8n")).await.unwrap().into_inner();

        assert_eq!(r.error_code, ErrorCode::ModelNotFound.as_str());
        assert_eq!(nodes.calls().len(), 1, "재시도하면 안 된다");
    }

    #[tokio::test]
    async fn retries_are_bounded_by_max_retries() {
        // max_retries = 1 이면 최초 1회 + 재시도 1회 = 총 2회.
        let nodes = FakeNodes::with(&[
            ("king", &[Outcome::Transport]),
            ("queen", &[Outcome::Transport]),
            ("jack", &[Outcome::Transport]),
        ]);
        let s = server(Arc::clone(&nodes), &["king", "queen", "jack"], 1);
        let r = s.infer(infer_req("yolov8n")).await.unwrap().into_inner();

        assert_eq!(r.error_code, ErrorCode::NodeUnavailable.as_str());
        assert_eq!(
            nodes.calls().len(),
            2,
            "노드가 3대여도 재시도 상한을 지킨다"
        );
        assert_eq!(r.attempts, 2);
    }

    #[tokio::test]
    async fn no_node_has_the_model() {
        let nodes = FakeNodes::with(&[("king", &[Outcome::Ok])]);
        let s = server(Arc::clone(&nodes), &["king"], 1);
        let r = s.infer(infer_req("mobilenet")).await.unwrap().into_inner();

        assert_eq!(r.error_code, ErrorCode::NoAvailableNode.as_str());
        assert!(nodes.calls().is_empty(), "노드를 호출하면 안 된다");
    }

    #[tokio::test]
    async fn unspecified_input_format_is_rejected() {
        let nodes = FakeNodes::with(&[("king", &[Outcome::Ok])]);
        let s = server(Arc::clone(&nodes), &["king"], 1);
        let mut req = infer_req("yolov8n");
        req.get_mut().input_format = 0;
        let r = s.infer(req).await.unwrap().into_inner();
        assert_eq!(r.error_code, ErrorCode::UnsupportedInputFormat.as_str());
    }

    #[tokio::test]
    async fn empty_request_id_is_generated() {
        let nodes = FakeNodes::with(&[("king", &[Outcome::Ok])]);
        let s = server(Arc::clone(&nodes), &["king"], 1);
        let mut req = infer_req("yolov8n");
        req.get_mut().request_id = String::new();
        let r = s.infer(req).await.unwrap().into_inner();
        assert!(!r.request_id.is_empty(), "스케줄러가 ID 를 만들어야 한다");
        assert!(uuid::Uuid::parse_str(&r.request_id).is_ok());
    }

    #[tokio::test]
    async fn payload_over_limit_is_rejected() {
        let nodes = FakeNodes::with(&[("king", &[Outcome::Ok])]);
        let s = server(Arc::clone(&nodes), &["king"], 1);
        let limit = s.config.server.max_payload_bytes as usize;
        let mut req = infer_req("yolov8n");
        req.get_mut().payload = vec![0u8; limit + 1];
        let r = s.infer(req).await.unwrap().into_inner();
        assert_eq!(r.error_code, ErrorCode::PayloadTooLarge.as_str());
        assert!(nodes.calls().is_empty());
    }

    // --- 노드 등록 / 하트비트 ---

    #[tokio::test]
    async fn register_requires_address() {
        // 주소가 없으면 스케줄러가 이 노드를 호출할 방법이 없다.
        let s = server(FakeNodes::with(&[]), &[], 1);
        let mut d = v1::NodeDescriptor::from(descriptor("king"));
        d.address = String::new();
        let r = s
            .register_node(Request::new(RegisterNodeRequest {
                descriptor: Some(d),
                token: String::new(),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(!r.accepted);
        assert_eq!(r.error_code, ErrorCode::InvalidRequest.as_str());
    }

    #[tokio::test]
    async fn register_rejects_wrong_token() {
        let mut s = server(FakeNodes::with(&[]), &[], 1);
        s.token = "secret".into();
        let r = s
            .register_node(Request::new(RegisterNodeRequest {
                descriptor: Some(descriptor("king").into()),
                token: "wrong".into(),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(!r.accepted);
        assert_eq!(r.error_code, ErrorCode::Unauthorized.as_str());
        assert!(s.registry.is_empty());
    }

    #[tokio::test]
    async fn register_returns_heartbeat_interval() {
        let s = server(FakeNodes::with(&[]), &[], 1);
        let r = s
            .register_node(Request::new(RegisterNodeRequest {
                descriptor: Some(descriptor("king").into()),
                token: String::new(),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(r.accepted);
        assert_eq!(
            r.heartbeat_interval_ms as u64, s.config.health.heartbeat_interval_ms,
            "노드가 스케줄러 설정을 따르게 한다"
        );
    }

    #[tokio::test]
    async fn heartbeat_from_unknown_node_demands_reregistration() {
        // 스케줄러가 재시작하면 레지스트리가 빈다. 노드가 하트비트만 계속
        // 보내면 영원히 복귀하지 못하므로 재등록을 요구해야 한다.
        let s = server(FakeNodes::with(&[]), &[], 1);
        let r = s
            .heartbeat(Request::new(HeartbeatRequest {
                node_id: "king".into(),
                health: Some(v1::NodeHealth::default()),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(!r.accepted);
        assert!(r.must_reregister);
    }

    #[tokio::test]
    async fn heartbeat_detects_reboot_by_boot_id() {
        // 어댑터 용량 부족으로 보드가 벤치마크 도중 리셋된 적이 있다.
        // 그 구간의 측정은 무효이므로 재부팅을 감지해야 한다.
        let s = server(FakeNodes::with(&[]), &["king"], 1);
        let king = NodeId::from("king");
        assert_eq!(s.registry.reboot_count(&king), Some(0));

        let r = s
            .heartbeat(Request::new(HeartbeatRequest {
                node_id: "king".into(),
                health: Some(v1::NodeHealth {
                    boot_id: "boot-DIFFERENT".into(),
                    ..Default::default()
                }),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(r.accepted);
        assert_eq!(s.registry.reboot_count(&king), Some(1));
    }

    #[tokio::test]
    async fn heartbeat_with_same_boot_id_is_not_a_reboot() {
        let s = server(FakeNodes::with(&[]), &["king"], 1);
        let king = NodeId::from("king");
        s.heartbeat(Request::new(HeartbeatRequest {
            node_id: "king".into(),
            health: Some(v1::NodeHealth {
                boot_id: "boot-king".into(),
                ..Default::default()
            }),
        }))
        .await
        .unwrap();
        assert_eq!(s.registry.reboot_count(&king), Some(0));
    }

    #[tokio::test]
    async fn deregister_removes_node() {
        let s = server(FakeNodes::with(&[]), &["king"], 1);
        let r = s
            .deregister_node(Request::new(DeregisterNodeRequest {
                node_id: "king".into(),
                reason: "종료".into(),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(r.accepted);
        assert!(s.registry.is_empty());
    }

    #[tokio::test]
    async fn deregister_unknown_node_is_not_accepted() {
        let s = server(FakeNodes::with(&[]), &[], 1);
        let r = s
            .deregister_node(Request::new(DeregisterNodeRequest {
                node_id: "ghost".into(),
                reason: String::new(),
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(!r.accepted);
    }
}
