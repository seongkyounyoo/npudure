//! NPUForge 스케줄러 실행 파일.

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use npuforge_common::config::SchedulerConfig;
use npuforge_proto::v1::scheduler_service_server::SchedulerServiceServer;
use npuforge_scheduler::{GrpcNodePool, Registry, SchedulerServer, policy_for};

fn main() -> ExitCode {
    let Some(config_path) = parse_args() else {
        eprintln!("사용법: npuforge-scheduler --config <path>");
        return ExitCode::from(2);
    };

    let config = match SchedulerConfig::from_path(&config_path) {
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

async fn run(config: SchedulerConfig) -> Result<(), String> {
    let policy = policy_for(config.scheduler.policy);
    let policy_kind = policy.kind();
    let registry = Arc::new(Registry::new(
        config.health.clone(),
        config.thresholds.clone(),
    ));

    let nodes = Arc::new(GrpcNodePool::with_transport(
        Duration::from_millis(config.scheduler.node_rpc_timeout_ms),
        config.server.max_payload_bytes as usize,
        config.transport.clone(),
    ));
    tracing::info!(
        node_connections = config.transport.node_connections,
        h2_stream_window = ?config.transport.h2_stream_window_bytes,
        h2_connection_window = ?config.transport.h2_connection_window_bytes,
        "전송 설정"
    );

    let addr = config
        .server
        .grpc_listen
        .parse()
        .map_err(|e| format!("grpc_listen '{}' 형식 오류: {e}", config.server.grpc_listen))?;

    let max_payload = config.server.max_payload_bytes as usize;
    let config = Arc::new(config);

    let server = SchedulerServer::new(
        Arc::clone(&registry),
        Arc::from(policy),
        nodes,
        Arc::clone(&config),
        std::env::var("NPUFORGE_NODE_TOKEN").unwrap_or_default(),
    );

    tracing::info!(
        event = "scheduler_started",
        version = env!("CARGO_PKG_VERSION"),
        policy = %policy_kind,
        grpc_listen = %config.server.grpc_listen,
        max_retries = config.scheduler.max_retries,
        request_timeout_ms = config.scheduler.request_timeout_ms,
        "NPUForge 스케줄러 시작"
    );

    tonic::transport::Server::builder()
        .add_service(
            SchedulerServiceServer::new(server)
                .max_decoding_message_size(max_payload)
                .max_encoding_message_size(max_payload),
        )
        .serve_with_shutdown(addr, wait_for_shutdown())
        .await
        .map_err(|e| format!("gRPC 서버 오류: {e}"))?;

    tracing::info!(
        event = "scheduler_stopped",
        registered_nodes = registry.len(),
        "스케줄러 종료"
    );
    Ok(())
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

fn init_tracing(config: &SchedulerConfig) {
    use tracing_subscriber::{EnvFilter, fmt};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.telemetry.log_level));

    // 구조화 로그를 기본으로 한다. docs/01-TECHSPEC.md §18.1 참조.
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
