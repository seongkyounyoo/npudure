//! NPUForge 노드 에이전트.
//!
//! 설정을 읽어 백엔드와 모델을 준비하고, `NodeService` gRPC 서버를 띄운 뒤
//! 스케줄러에 등록해 하트비트를 보낸다.

use std::process::ExitCode;
use std::sync::Arc;

use npuforge_common::{
    backend::InferenceBackend,
    config::{BackendConfig, NodeConfig},
    model::ModelSpec,
};
use npuforge_node::{
    NodeServer, WorkerPool, models,
    registry_client::{self, BackoffPolicy, GrpcLink},
    telemetry,
};
use npuforge_proto::v1::{self, node_service_server::NodeServiceServer};

fn main() -> ExitCode {
    let Some(config_path) = parse_args() else {
        eprintln!("사용법: npuforge-node --config <path>");
        return ExitCode::from(2);
    };

    let config = match NodeConfig::from_path(&config_path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("설정 로딩 실패: {err}");
            return ExitCode::FAILURE;
        }
    };

    init_tracing(&config);

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(err) => {
            tracing::error!(error = %err, "tokio 런타임을 만들 수 없습니다");
            return ExitCode::FAILURE;
        }
    };

    match runtime.block_on(run(config)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            tracing::error!("{message}");
            ExitCode::FAILURE
        }
    }
}

async fn run(config: NodeConfig) -> Result<(), String> {
    let backend = build_backend(&config)?;
    let runtime_version = backend
        .runtime_version()
        .map_err(|e| format!("런타임 버전을 조회할 수 없습니다: {e}"))?;

    // v0.1 은 노드당 모델 하나다. `docs/01-TECHSPEC.md` §13.2.
    let model_id = config
        .models
        .preload
        .first()
        .ok_or("models.preload 가 비어 있습니다. v0.1 은 노드당 모델 하나를 요구합니다")?
        .clone();

    let (mut spec, model_file) = models::load_spec(&config.models.directory, &model_id)?;

    // Mock 백엔드는 모델 파일을 읽지 않으므로 해시 검증 대상이 아니다.
    // 실장비에서는 반드시 검증한다 — 노드마다 다른 파일을 로딩한 채 측정하면
    // 정책 비교 결과가 통째로 무의미해진다.
    if !matches!(config.backend, BackendConfig::Mock(_)) {
        models::verify_sha256(&model_file, &spec.sha256)?;
    }

    // `load_spec` 이 `directory`/`model_id` 기준으로 해석한 절대경로를 백엔드에
    // 넘긴다. `spec.model_file` 은 `model.toml` 의 원본 값(상대경로일 수 있다)
    // 이라 그대로 넘기면 백엔드가 CWD 기준으로 파일을 찾아 로딩에 실패한다.
    // 해시 검증은 `model_file` 로 이미 절대경로를 썼는데 로딩은 상대경로를 쓰던
    // 불일치를 바로잡는다.
    spec.model_file = model_file.to_string_lossy().into_owned();

    let model = backend
        .load_model(&spec)
        .await
        .map_err(|e| format!("모델 '{model_id}' 로딩 실패: {e}"))?;
    let model: Arc<dyn npuforge_common::backend::LoadedModel> = Arc::from(model);

    let pool = Arc::new(WorkerPool::spawn(
        model,
        config.worker.worker_count,
        config.worker.max_queue_depth,
    ));

    let server = Arc::new(NodeServer::new(
        config.node.id.clone(),
        Arc::clone(&pool),
        spec.clone(),
        config.telemetry.temperature_path.clone(),
    ));

    tracing::info!(
        event = "node_started",
        node_id = %config.node.id,
        version = env!("CARGO_PKG_VERSION"),
        backend = backend.backend_name(),
        runtime_version = %runtime_version,
        worker_count = config.worker.worker_count,
        max_queue_depth = config.worker.max_queue_depth,
        listen = %config.node.listen,
        advertise_address = %config.node.advertise_address,
        model = %model_id,
        boot_id = %telemetry::read_boot_id(),
        "NPUForge 노드 시작"
    );

    // Warmup. 첫 추론은 지연시간이 크게 튀므로 벤치마크에서 제외한다.
    // `docs/01-TECHSPEC.md` §13.4.
    if config.models.warmup_runs > 0 {
        warmup(&pool, &spec, config.models.warmup_runs).await;
    }

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // 스케줄러 등록 + 하트비트
    let descriptor = build_descriptor(&config, &spec, &runtime_version);
    let scheduler_address = config.node.scheduler_address.clone();
    let health_server = Arc::clone(&server);
    let registry_task = tokio::spawn(async move {
        let mut link = match GrpcLink::connect_lazy(&scheduler_address) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("{e}");
                return;
            }
        };
        registry_client::run_loop(
            &mut link,
            descriptor,
            String::new(),
            || health_server.health_snapshot(),
            BackoffPolicy::default(),
            shutdown_rx,
        )
        .await;
    });

    // gRPC 서버
    let addr = config
        .node
        .listen
        .parse()
        .map_err(|e| format!("listen 주소 '{}' 형식 오류: {e}", config.node.listen))?;

    // 여기서 정하는 것은 **노드가 수신하는 방향**, 즉 스케줄러 → 노드
    // 요청(1.23 MB)의 HTTP/2 flow-control window 다. 응답 방향은 스케줄러가
    // 광고한다. 둘 다 기본값이면 h2 기본 65,535 byte 로 동작한다.
    let mut builder = tonic::transport::Server::builder();
    if let Some(v) = config.transport.h2_stream_window_bytes {
        builder = builder.initial_stream_window_size(Some(v));
    }
    if let Some(v) = config.transport.h2_connection_window_bytes {
        builder = builder.initial_connection_window_size(Some(v));
    }
    tracing::info!(
        h2_stream_window = ?config.transport.h2_stream_window_bytes,
        h2_connection_window = ?config.transport.h2_connection_window_bytes,
        "전송 설정"
    );

    let serve = builder
        .add_service(NodeServiceServer::from_arc(server))
        .serve_with_shutdown(addr, async {
            wait_for_shutdown().await;
            let _ = shutdown_tx.send(true);
        });

    serve.await.map_err(|e| format!("gRPC 서버 오류: {e}"))?;

    // 등록 해제가 끝날 때까지 기다린다. 스케줄러가 즉시 이 노드를 후보에서
    // 빼야 종료 중 요청이 실패로 잡히지 않는다.
    let _ = tokio::time::timeout(std::time::Duration::from_secs(3), registry_task).await;

    tracing::info!(event = "node_stopped", node_id = %config.node.id, "노드 종료");
    Ok(())
}

async fn warmup(pool: &WorkerPool, spec: &ModelSpec, runs: u32) {
    let dummy = bytes::Bytes::from(vec![0u8; spec.raw_input_bytes() as usize]);
    let started = std::time::Instant::now();
    let mut done = 0u32;
    for _ in 0..runs {
        let input = npuforge_common::backend::InferenceInput {
            payload: dummy.clone(),
            format: spec.input_format,
        };
        match pool.submit(input, None).await {
            Ok(_) => done += 1,
            Err(e) => {
                tracing::warn!(error = %e.message, "warmup 추론 실패");
                break;
            }
        }
    }
    tracing::info!(
        event = "warmup_done",
        runs = done,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "warmup 완료. 이 결과는 벤치마크에서 제외한다"
    );
}

fn build_descriptor(
    config: &NodeConfig,
    spec: &ModelSpec,
    runtime_version: &str,
) -> v1::NodeDescriptor {
    v1::NodeDescriptor {
        node_id: config.node.id.clone(),
        hostname: hostname(),
        address: normalize_address(&config.node.advertise_address),
        device_type: std::env::var("NPUFORGE_DEVICE_TYPE").unwrap_or_else(|_| "unknown".into()),
        npu_type: std::env::var("NPUFORGE_NPU_TYPE").unwrap_or_else(|_| "unknown".into()),
        npu_core_count: std::env::var("NPUFORGE_NPU_CORES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        memory_bytes: 0,
        agent_version: env!("CARGO_PKG_VERSION").to_owned(),
        runtime_version: runtime_version.to_owned(),
        models: vec![v1::ModelDescriptor {
            id: spec.id.clone(),
            version: spec.version.clone(),
            state: v1::ModelState::Ready as i32,
            sha256: spec.sha256.clone(),
        }],
    }
}

/// 설정의 `advertise_address` 는 `listen` 과 같은 `호스트:포트` 형식이다.
/// gRPC 클라이언트는 scheme 이 있는 URL 을 요구하므로 여기서 붙인다.
///
/// 설정 파일 두 곳에 서로 다른 형식을 쓰게 하면 반드시 한쪽을 틀린다.
fn normalize_address(address: &str) -> String {
    if address.starts_with("http://") || address.starts_with("https://") {
        address.to_owned()
    } else {
        format!("http://{address}")
    }
}

fn hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_owned())
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

async fn wait_for_shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = match signal(SignalKind::terminate()) {
            Ok(s) => s,
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
                return;
            }
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

/// 설정과 빌드 조합을 검사한 뒤 백엔드를 만든다.
///
/// Mock 전용으로 빌드한 바이너리를 실제 노드에 배포하는 사고를 여기서 잡는다.
/// 세 노드에 동일 바이너리를 배포하는 구조라 이 실수는 조용히 퍼지기 쉽다.
fn build_backend(config: &NodeConfig) -> Result<Arc<dyn InferenceBackend>, String> {
    match &config.backend {
        BackendConfig::Mock(mock) => Ok(Arc::new(npuforge_mock_backend::MockBackend::new(
            mock.clone(),
        ))),
        BackendConfig::Rknn { runtime_library } => {
            if !npuforge_rknn::is_rknn_enabled() {
                return Err(format!(
                    "설정은 RKNN 백엔드를 요구하지만 이 바이너리는 RKNN 지원 없이 빌드되었습니다. \
                     `cargo build-node` 로 다시 빌드하세요. (runtime_library={runtime_library})"
                ));
            }
            // 컨텍스트 수를 worker_count 와 맞춘다.
            //
            // 적으면 워커가 컨텍스트를 기다리느라 놀고, 많으면 NPU 메모리를
            // 낭비한다. 공유는 애초에 선택지가 아니다 — 컨텍스트를 공유하면
            // API 오류 없이 100% 틀린 결과가 나온다.
            // `docs/environment-matrix.md` §3.1 정정 참조.
            Ok(Arc::new(
                npuforge_rknn::RknnBackend::new(runtime_library.clone())
                    .with_context_count(config.worker.worker_count)
                    .with_want_float(config.worker.want_float),
            ))
        }
    }
}

fn parse_args() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" | "-c" => return args.next(),
            _ => continue,
        }
    }
    None
}

fn init_tracing(config: &NodeConfig) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.telemetry.log_level));

    if config.telemetry.json_log {
        fmt()
            .json()
            .with_env_filter(filter)
            .with_current_span(false)
            .init();
    } else {
        fmt().with_env_filter(filter).init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use npuforge_common::config::{MockBackendConfig, ModelsConfig, NodeSection, WorkerConfig};

    fn config(backend: BackendConfig) -> NodeConfig {
        NodeConfig {
            node: NodeSection {
                id: "mock-01".into(),
                scheduler_address: "http://127.0.0.1:50051".into(),
                listen: "127.0.0.1:51001".into(),
                advertise_address: "127.0.0.1:51001".into(),
            },
            worker: WorkerConfig::default(),
            backend,
            models: ModelsConfig {
                directory: "./examples/models".into(),
                preload: vec!["yolov8n".into()],
                warmup_runs: 3,
            },
            telemetry: Default::default(),
            transport: Default::default(),
        }
    }

    #[test]
    fn mock_backend_builds() {
        let backend =
            build_backend(&config(BackendConfig::Mock(MockBackendConfig::default()))).unwrap();
        assert_eq!(backend.backend_name(), "mock");
    }

    #[test]
    #[cfg(not(feature = "rknn"))]
    fn rknn_config_on_mock_build_fails_with_actionable_message() {
        // 개발 PC에서 만든 바이너리를 실장비에 올렸을 때 원인이 바로 보여야 한다.
        let result = build_backend(&config(BackendConfig::Rknn {
            runtime_library: "/usr/lib/librknnrt.so".into(),
        }));
        let Err(err) = result else {
            panic!("RKNN 지원 없는 빌드에서는 실패해야 한다");
        };
        assert!(err.contains("RKNN 지원 없이 빌드"), "실제 메시지: {err}");
        assert!(
            err.contains("cargo build-node"),
            "재빌드 방법을 알려줘야 한다"
        );
    }

    #[test]
    fn advertise_address_gets_a_scheme() {
        // 설정은 listen 과 같은 host:port 형식을 쓰고, 등록할 때 URL 로 만든다.
        assert_eq!(
            normalize_address("192.168.123.22:50052"),
            "http://192.168.123.22:50052"
        );
    }

    #[test]
    fn explicit_scheme_is_preserved() {
        assert_eq!(
            normalize_address("https://king.local:50052"),
            "https://king.local:50052"
        );
    }
}
