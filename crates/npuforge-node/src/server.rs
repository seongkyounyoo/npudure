//! 노드가 제공하는 `NodeService` gRPC 서버.
//!
//! 스케줄러만 호출한다. 클라이언트는 스케줄러를 거친다.

use std::sync::Arc;
use std::time::Duration;

use npuforge_common::{
    ErrorCode, NodeId,
    backend::InferenceInput,
    model::{ModelDescriptor, ModelSpec, ModelState},
    protocol::InputFormat,
};
use npuforge_proto::v1::{
    self, HealthRequest, HealthResponse, ListModelsRequest, ListModelsResponse, NodeInferRequest,
    NodeInferResponse, WarmupRequest, WarmupResponse, node_service_server::NodeService,
};
use tonic::{Request, Response, Status};

use crate::{
    telemetry::{self, CpuSampler},
    worker::WorkerPool,
};

/// 노드 서비스 구현.
pub struct NodeServer {
    pub node_id: NodeId,
    pub pool: Arc<WorkerPool>,
    pub model: ModelSpec,
    pub temperature_path: Option<String>,
    cpu: parking_lot::Mutex<CpuSampler>,
}

impl NodeServer {
    pub fn new(
        node_id: NodeId,
        pool: Arc<WorkerPool>,
        model: ModelSpec,
        temperature_path: Option<String>,
    ) -> Self {
        Self {
            node_id,
            pool,
            model,
            temperature_path,
            cpu: parking_lot::Mutex::new(CpuSampler::new()),
        }
    }

    /// 현재 노드 상태를 proto 형식으로 만든다.
    pub fn health_snapshot(&self) -> v1::NodeHealth {
        let hw = telemetry::collect(self.temperature_path.as_deref());
        let cpu = self.cpu.lock().sample();

        v1::NodeHealth {
            queue_depth: self.pool.queue_depth(),
            in_flight: self.pool.in_flight(),
            error_rate: self.pool.error_rate(),
            has_temperature: hw.temperature_c.is_some(),
            temperature_c: hw.temperature_c.unwrap_or(0.0),
            has_cpu_percent: cpu.is_some(),
            cpu_percent: cpu.unwrap_or(0.0),
            has_memory_percent: hw.memory_percent.is_some(),
            memory_percent: hw.memory_percent.unwrap_or(0.0),
            has_npu_percent: false,
            npu_percent: 0.0,
            has_input_voltage: hw.input_voltage_v.is_some(),
            input_voltage_v: hw.input_voltage_v.unwrap_or(0.0),
            boot_id: hw.boot_id,
        }
    }

    pub fn model_descriptor(&self) -> ModelDescriptor {
        ModelDescriptor {
            id: self.model.id.clone(),
            version: self.model.version.clone(),
            state: ModelState::Ready,
            sha256: self.model.sha256.clone(),
        }
    }
}

#[tonic::async_trait]
impl NodeService for NodeServer {
    async fn infer(
        &self,
        request: Request<NodeInferRequest>,
    ) -> std::result::Result<Response<NodeInferResponse>, Status> {
        let req = request.into_inner();

        // 요청 검증. 실패는 재시도해도 같은 결과이므로 명확히 구분해 돌려준다.
        let format = match v1::InputFormat::try_from(req.input_format)
            .ok()
            .and_then(|f| InputFormat::try_from(f).ok())
        {
            Some(f) => f,
            None => {
                return Ok(Response::new(error_response(
                    &req.request_id,
                    ErrorCode::UnsupportedInputFormat,
                    "입력 형식이 지정되지 않았거나 지원하지 않습니다",
                )));
            }
        };

        if req.model_id != self.model.id {
            return Ok(Response::new(error_response(
                &req.request_id,
                ErrorCode::ModelNotFound,
                format!(
                    "이 노드는 '{}' 만 로딩했습니다 (요청: '{}')",
                    self.model.id, req.model_id
                ),
            )));
        }

        if req.payload.is_empty() {
            return Ok(Response::new(error_response(
                &req.request_id,
                ErrorCode::InvalidRequest,
                "payload 가 비어 있습니다",
            )));
        }

        let queue_timeout =
            (req.queue_timeout_ms > 0).then(|| Duration::from_millis(req.queue_timeout_ms));

        let input = InferenceInput {
            payload: bytes::Bytes::from(req.payload),
            format,
        };

        match self.pool.submit(input, queue_timeout).await {
            Ok(out) => Ok(Response::new(NodeInferResponse {
                request_id: req.request_id,
                result: out.result.to_vec(),
                result_format: out.result_format,
                timing: Some(out.timing.into()),
                error_code: ErrorCode::Ok.as_str().to_owned(),
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(error_response(
                &req.request_id,
                e.code,
                e.message,
            ))),
        }
    }

    async fn health(
        &self,
        _request: Request<HealthRequest>,
    ) -> std::result::Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse {
            health: Some(self.health_snapshot()),
            ready: true,
        }))
    }

    async fn list_models(
        &self,
        _request: Request<ListModelsRequest>,
    ) -> std::result::Result<Response<ListModelsResponse>, Status> {
        Ok(Response::new(ListModelsResponse {
            models: vec![self.model_descriptor().into()],
        }))
    }

    async fn warmup(
        &self,
        request: Request<WarmupRequest>,
    ) -> std::result::Result<Response<WarmupResponse>, Status> {
        let req = request.into_inner();
        if req.model_id != self.model.id {
            return Ok(Response::new(WarmupResponse {
                completed: false,
                runs_done: 0,
                error_code: ErrorCode::ModelNotFound.as_str().to_owned(),
                error_message: format!("모델 '{}' 을 찾을 수 없습니다", req.model_id),
            }));
        }

        // Warmup 은 실제 추론 크기의 더미 입력을 사용한다.
        // 결과는 벤치마크에서 제외한다(`01-TECHSPEC.md` §13.4).
        let size = self.model.raw_input_bytes() as usize;
        let dummy = bytes::Bytes::from(vec![0u8; size]);

        let runs = req.runs.max(1);
        let mut done = 0u32;
        for _ in 0..runs {
            let input = InferenceInput {
                payload: dummy.clone(),
                format: self.model.input_format,
            };
            match self.pool.submit(input, None).await {
                Ok(_) => done += 1,
                Err(e) => {
                    return Ok(Response::new(WarmupResponse {
                        completed: false,
                        runs_done: done,
                        error_code: e.code.as_str().to_owned(),
                        error_message: e.message,
                    }));
                }
            }
        }

        Ok(Response::new(WarmupResponse {
            completed: true,
            runs_done: done,
            error_code: ErrorCode::Ok.as_str().to_owned(),
            error_message: String::new(),
        }))
    }
}

fn error_response(
    request_id: &str,
    code: ErrorCode,
    message: impl Into<String>,
) -> NodeInferResponse {
    NodeInferResponse {
        request_id: request_id.to_owned(),
        result: Vec::new(),
        result_format: String::new(),
        timing: None,
        error_code: code.as_str().to_owned(),
        error_message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use npuforge_common::config::MockBackendConfig;
    use npuforge_mock_backend::MockModel;

    fn spec() -> ModelSpec {
        ModelSpec {
            id: "yolov8n".into(),
            version: "1.0.0".into(),
            backend: "mock".into(),
            model_file: "model.rknn".into(),
            input_width: 8,
            input_height: 8,
            input_channels: 3,
            input_format: InputFormat::Rgb8,
            output_format: "yolo-detections".into(),
            sha256: "0".repeat(64),
            labels_file: None,
        }
    }

    fn server(cfg: MockBackendConfig) -> NodeServer {
        let model = Arc::new(MockModel::with_seed(spec(), cfg, 7));
        let pool = Arc::new(WorkerPool::spawn(model, 2, 16));
        NodeServer::new("queen".into(), pool, spec(), None)
    }

    fn req(model_id: &str, format: i32, payload: Vec<u8>) -> Request<NodeInferRequest> {
        Request::new(NodeInferRequest {
            request_id: "req-1".into(),
            model_id: model_id.into(),
            payload,
            input_format: format,
            queue_timeout_ms: 0,
        })
    }

    fn fast() -> MockBackendConfig {
        MockBackendConfig {
            base_latency_ms: 1,
            jitter_ms: 0,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn infer_succeeds() {
        let s = server(fast());
        let r = s
            .infer(req(
                "yolov8n",
                v1::InputFormat::Jpeg as i32,
                b"data".to_vec(),
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.error_code, "NPF-0000");
        assert_eq!(r.request_id, "req-1");
        assert!(r.timing.is_some());
    }

    #[tokio::test]
    async fn unknown_model_is_rejected_without_retry_hint() {
        // 다른 노드로 재시도해도 같은 결과이므로 재시도 불가 코드여야 한다.
        let s = server(fast());
        let r = s
            .infer(req(
                "mobilenet",
                v1::InputFormat::Jpeg as i32,
                b"x".to_vec(),
            ))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.error_code, ErrorCode::ModelNotFound.as_str());
        assert!(!ErrorCode::ModelNotFound.is_retryable());
    }

    #[tokio::test]
    async fn unspecified_input_format_is_rejected() {
        // proto3 는 필드를 안 채우면 0 을 준다. 조용히 JPEG 로 처리하면 안 된다.
        let s = server(fast());
        let r = s
            .infer(req("yolov8n", 0, b"x".to_vec()))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.error_code, ErrorCode::UnsupportedInputFormat.as_str());
    }

    #[tokio::test]
    async fn empty_payload_is_rejected() {
        let s = server(fast());
        let r = s
            .infer(req("yolov8n", v1::InputFormat::Jpeg as i32, Vec::new()))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.error_code, ErrorCode::InvalidRequest.as_str());
    }

    #[tokio::test]
    async fn backend_failure_is_reported_as_retryable() {
        let s = server(MockBackendConfig {
            base_latency_ms: 1,
            jitter_ms: 0,
            error_rate: 1.0,
            ..Default::default()
        });
        let r = s
            .infer(req("yolov8n", v1::InputFormat::Jpeg as i32, b"x".to_vec()))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.error_code, ErrorCode::InferenceFailed.as_str());
        assert!(
            ErrorCode::InferenceFailed.is_retryable(),
            "다른 노드에서 재시도할 수 있어야 한다"
        );
    }

    #[tokio::test]
    async fn health_reports_queue_state() {
        let s = server(fast());
        let h = s
            .health(Request::new(HealthRequest {}))
            .await
            .unwrap()
            .into_inner();
        let health = h.health.unwrap();
        assert!(h.ready);
        assert_eq!(health.queue_depth, 0);
        // 센서가 없는 환경에서 0.0 을 실제 값으로 보고하면 안 된다
        if !health.has_temperature {
            assert_eq!(health.temperature_c, 0.0);
        }
    }

    #[tokio::test]
    async fn list_models_returns_loaded_model() {
        let s = server(fast());
        let r = s
            .list_models(Request::new(ListModelsRequest {}))
            .await
            .unwrap()
            .into_inner();
        assert_eq!(r.models.len(), 1);
        assert_eq!(r.models[0].id, "yolov8n");
        assert_eq!(r.models[0].state, v1::ModelState::Ready as i32);
    }

    #[tokio::test]
    async fn warmup_runs_requested_count() {
        let s = server(fast());
        let r = s
            .warmup(Request::new(WarmupRequest {
                model_id: "yolov8n".into(),
                runs: 3,
            }))
            .await
            .unwrap()
            .into_inner();
        assert!(r.completed);
        assert_eq!(r.runs_done, 3);
    }
}
