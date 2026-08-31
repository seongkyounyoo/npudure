//! TOML 설정 구조체.
//!
//! 스키마는 `docs/01-TECHSPEC.md` §16이 규범이다.
//! `configs/*.example.toml`이 이 구조체로 파싱되는지 테스트로 검증한다.

use serde::{Deserialize, Serialize};

use crate::{ModelId, NodeId, policy::SchedulingPolicyKind};

// ---------------------------------------------------------------------------
// Scheduler
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulerConfig {
    pub server: ServerConfig,
    #[serde(default)]
    pub scheduler: SchedulerSection,
    #[serde(default)]
    pub health: HealthConfig,
    #[serde(default)]
    pub thresholds: ThresholdConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub transport: SchedulerTransportConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub grpc_listen: String,
    pub http_listen: String,
    pub metrics_listen: String,
    /// 이 크기를 넘는 payload는 `NPF-1002`로 거부한다.
    #[serde(default = "default_max_payload")]
    pub max_payload_bytes: u64,
}

const fn default_max_payload() -> u64 {
    10 * 1024 * 1024
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct SchedulerSection {
    pub policy: SchedulingPolicyKind,
    pub request_timeout_ms: u64,
    pub node_rpc_timeout_ms: u64,
    /// 실시간 추론 특성상 긴 exponential backoff는 사용하지 않는다.
    /// `docs/01-TECHSPEC.md` §12.4 참조.
    pub max_retries: u32,
    pub retry_backoff_min_ms: u64,
    pub retry_backoff_max_ms: u64,
}

impl Default for SchedulerSection {
    fn default() -> Self {
        Self {
            policy: SchedulingPolicyKind::Ect,
            request_timeout_ms: 5_000,
            node_rpc_timeout_ms: 3_000,
            max_retries: 1,
            retry_backoff_min_ms: 10,
            retry_backoff_max_ms: 100,
        }
    }
}

/// 스케줄러 → 노드 HTTP/2 전송 설정.
///
/// S3.5 에서 −30% 손실이 전송 경로에 있다는 것까지 좁혔으나, 그 안에서
/// ①flow control ②커넥션/TCP ③직렬화 중 무엇인지는 갈리지 않았다.
/// 이 설정은 **①②를 코드 재작성 없이 A/B 로 가르기 위한 것**이다
/// (`docs/experiments/S3_5_TRANSPORT_PROFILE.md` §7).
///
/// 기본값은 기존 동작과 정확히 같다 — 커넥션 1개, window 는 h2 기본값.
/// 그래야 S2·S3 baseline 이 이 변경으로 흔들리지 않는다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct SchedulerTransportConfig {
    /// 노드당 gRPC 커넥션 수.
    ///
    /// HTTP/2 는 커넥션 하나에서 스트림을 다중화하므로 1 이 정상 설계다.
    /// 다만 커넥션 상태 기계·소켓이 직렬화 지점인지 확인하려면 늘려 봐야
    /// 한다. 요청마다 round-robin 으로 고른다.
    pub node_connections: usize,

    /// HTTP/2 스트림 window (byte). `None` 이면 h2 기본값(65,535).
    ///
    /// 이 노드로 보내는 1.23 MB 요청이 64 KB 마다 WINDOW_UPDATE 를 기다리는지
    /// 보려면 이 값을 키운다. 실제로 막는 쪽은 **수신자**가 광고한 window
    /// 이므로 요청 방향은 노드의 `[transport]`, 응답 방향은 여기가 정한다.
    pub h2_stream_window_bytes: Option<u32>,

    /// HTTP/2 커넥션 window (byte). `None` 이면 h2 기본값(65,535).
    ///
    /// 스트림 window 를 키워도 이것이 작으면 커넥션 위 모든 스트림이
    /// 같이 막힌다. 둘을 같이 올려야 의미가 있다.
    pub h2_connection_window_bytes: Option<u32>,
}

impl Default for SchedulerTransportConfig {
    fn default() -> Self {
        Self {
            node_connections: 1,
            h2_stream_window_bytes: None,
            h2_connection_window_bytes: None,
        }
    }
}

/// 노드 gRPC 서버의 HTTP/2 전송 설정.
///
/// 노드는 서버이므로 커넥션 수를 정하지 않는다. 대신 **요청 방향**
/// (스케줄러 → 노드, 1.23 MB)의 flow control 을 여기서 광고한다.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct NodeTransportConfig {
    /// `None` 이면 h2 기본값(65,535).
    pub h2_stream_window_bytes: Option<u32>,
    /// `None` 이면 h2 기본값(65,535).
    pub h2_connection_window_bytes: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct HealthConfig {
    pub heartbeat_interval_ms: u64,
    pub health_timeout_ms: u64,
    /// 연속 실패 횟수가 이 값에 도달하면 `Unreachable`.
    pub failure_threshold: u32,
    /// 연속 성공 횟수가 이 값에 도달하면 `Recovering` → `Healthy`.
    pub recovery_threshold: u32,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_ms: 2_000,
            health_timeout_ms: 1_000,
            failure_threshold: 3,
            recovery_threshold: 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct ThresholdConfig {
    pub busy_queue_depth: u32,
    pub degraded_error_rate: f64,
    pub degraded_temperature_c: f64,
    /// 이 온도를 넘으면 스케줄링에서 제외한다.
    pub disable_temperature_c: f64,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            busy_queue_depth: 8,
            degraded_error_rate: 0.10,
            degraded_temperature_c: 80.0,
            disable_temperature_c: 90.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct TelemetryConfig {
    pub json_log: bool,
    pub log_level: String,
    /// ⚠️ **아직 구현되지 않았다 (M2).** 이 값을 켜도 메트릭 엔드포인트가
    /// 뜨지 않는다. `metrics_listen` 포트에는 아무것도 바인드되지 않는다.
    /// 설정 스키마를 먼저 고정해 둔 것이며, 구현 전까지 기본값은 `false` 다.
    pub prometheus: bool,
    /// 노드 전용. 온도 파일 경로.
    pub temperature_path: Option<String>,
    pub heartbeat_interval_ms: Option<u64>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            json_log: true,
            log_level: "info".into(),
            prometheus: false,
            temperature_path: None,
            heartbeat_interval_ms: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    pub node: NodeSection,
    #[serde(default)]
    pub worker: WorkerConfig,
    pub backend: BackendConfig,
    pub models: ModelsConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub transport: NodeTransportConfig,
}

/// 노드 정체성과 주소.
///
/// 세 노드 간 차이는 이 섹션으로 제한된다. 실행 동시성은 `[worker]`로 분리한다.
/// `docs/01-TECHSPEC.md` §16.2 참조.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeSection {
    pub id: NodeId,
    pub scheduler_address: String,
    pub listen: String,
    pub advertise_address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct WorkerConfig {
    /// 동시 추론 워커 수.
    ///
    /// RKNN 백엔드는 이 수만큼 컨텍스트를 만들어 하나씩 점유한다.
    /// 컨텍스트를 공유하면 API 오류 없이 100% 틀린 결과가 나오므로
    /// 공유는 선택지가 아니다. `docs/environment-matrix.md` §3.1 정정 참조.
    ///
    /// RK3576 실측 최적값은 **8** 이다(4 대비 +27%). 기본값 1 은 백엔드를
    /// 모르는 상태에서의 안전값이며, 실장비 설정은 명시적으로 8 을 준다.
    pub worker_count: usize,
    pub max_queue_depth: usize,
    /// 로컬 큐에서 이 시간을 초과해 대기한 요청은 `NPF-1303`으로 거부한다.
    pub queue_timeout_ms: u64,
    /// 출력을 float32 로 역양자화해 반환할지 여부. RKNN 백엔드에만 해당한다.
    ///
    /// **기본값 `false` 는 네트워크 대역폭 때문이다.** 노드는 후처리를
    /// 하지 않고 원시 텐서를 반환하므로, `true` 면 출력이 입력의 3.96배가
    /// 되어 3노드 포화 시 스케줄러 RX 가 18.4 Gbps 에 이른다. 10G 로도
    /// 감당하지 못한다. `docs/02-HARDWARE-SETUP.md` §3.3.2.
    ///
    /// `false` 로 받은 데이터는 양자화된 정수다. 응답 blob(v2)이 텐서마다
    /// `scale` 과 `zero_point` 를 함께 실어 보내므로 받는 쪽이
    /// `real = (q - zero_point) × scale` 로 되돌린다.
    pub want_float: bool,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            worker_count: 1,
            max_queue_depth: 32,
            queue_timeout_ms: 3_000,
            want_float: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum BackendConfig {
    Rknn {
        #[serde(default = "default_rknn_library")]
        runtime_library: String,
    },
    Mock(MockBackendConfig),
}

fn default_rknn_library() -> String {
    "/usr/lib/librknnrt.so".into()
}

/// Mock Backend 동작 설정.
///
/// 실제 NanoPi 없이 스케줄링, 장애 감지, 복구를 검증하기 위한 것이다.
/// `docs/03-DEVELOPMENT-REQUIREMENTS.md` §4.1 참조.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MockBackendConfig {
    /// 기본 추론시간.
    pub base_latency_ms: u64,
    /// 추론시간 편차. 실제 결과는 base ± jitter 범위의 균등분포.
    pub jitter_ms: u64,
    /// 인위적 실패율. 0.0 ~ 1.0.
    pub error_rate: f64,
    /// 노드별 속도 차이를 만들기 위한 배수. 1.0이 기준.
    pub speed_factor: f64,
    /// 요청 N건마다 한 번 응답하지 않는다. 0이면 사용하지 않는다.
    pub hang_every_n: u64,
}

impl Default for MockBackendConfig {
    fn default() -> Self {
        Self {
            base_latency_ms: 20,
            jitter_ms: 5,
            error_rate: 0.0,
            speed_factor: 1.0,
            hang_every_n: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelsConfig {
    pub directory: String,
    #[serde(default)]
    pub preload: Vec<ModelId>,
    /// Warmup 결과는 벤치마크에서 제외한다.
    #[serde(default = "default_warmup_runs")]
    pub warmup_runs: u32,
}

const fn default_warmup_runs() -> u32 {
    3
}

// ---------------------------------------------------------------------------
// 로딩
// ---------------------------------------------------------------------------

/// 설정 파일 로딩 오류.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("설정 파일을 읽을 수 없습니다 ({path}): {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },

    #[error("설정 파일 형식이 잘못되었습니다 ({path}): {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },

    #[error("설정 값이 유효하지 않습니다: {0}")]
    Invalid(String),
}

macro_rules! impl_load {
    ($ty:ty) => {
        impl $ty {
            /// TOML 파일에서 설정을 읽는다.
            pub fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
                let path = path.as_ref();
                let display = path.display().to_string();
                let text = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
                    path: display.clone(),
                    source,
                })?;
                let parsed: Self = toml::from_str(&text).map_err(|source| ConfigError::Parse {
                    path: display,
                    source,
                })?;
                parsed.validate()?;
                Ok(parsed)
            }
        }
    };
}

impl_load!(SchedulerConfig);
impl_load!(NodeConfig);

/// HTTP/2 window 값 검증. RFC 7540 §6.9.1 상한은 2^31−1.
fn check_h2_window(label: &str, v: Option<u32>) -> Result<(), ConfigError> {
    const MAX: u32 = i32::MAX as u32; // 2^31 - 1
    match v {
        Some(0) => Err(ConfigError::Invalid(format!(
            "{label} 가 0 입니다. 생략하면 h2 기본값(65535)을 씁니다"
        ))),
        Some(n) if n > MAX => Err(ConfigError::Invalid(format!(
            "{label}({n}) 가 HTTP/2 상한 {MAX} 를 넘습니다"
        ))),
        _ => Ok(()),
    }
}

impl SchedulerConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        let t = &self.thresholds;
        if t.disable_temperature_c <= t.degraded_temperature_c {
            return Err(ConfigError::Invalid(format!(
                "disable_temperature_c({}) 는 degraded_temperature_c({}) 보다 커야 합니다",
                t.disable_temperature_c, t.degraded_temperature_c
            )));
        }
        if !(0.0..=1.0).contains(&t.degraded_error_rate) {
            return Err(ConfigError::Invalid(format!(
                "degraded_error_rate 는 0.0~1.0 범위여야 합니다: {}",
                t.degraded_error_rate
            )));
        }
        let s = &self.scheduler;
        if s.node_rpc_timeout_ms >= s.request_timeout_ms {
            return Err(ConfigError::Invalid(format!(
                "node_rpc_timeout_ms({}) 는 request_timeout_ms({}) 보다 작아야 재시도 여유가 남습니다",
                s.node_rpc_timeout_ms, s.request_timeout_ms
            )));
        }
        if s.retry_backoff_min_ms > s.retry_backoff_max_ms {
            return Err(ConfigError::Invalid(
                "retry_backoff_min_ms 가 retry_backoff_max_ms 보다 큽니다".into(),
            ));
        }
        let tr = &self.transport;
        if tr.node_connections == 0 {
            return Err(ConfigError::Invalid(
                "node_connections 는 1 이상이어야 합니다".into(),
            ));
        }
        check_h2_window("h2_stream_window_bytes", tr.h2_stream_window_bytes)?;
        check_h2_window("h2_connection_window_bytes", tr.h2_connection_window_bytes)?;
        Ok(())
    }
}

impl NodeConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.worker.worker_count == 0 {
            return Err(ConfigError::Invalid(
                "worker_count 는 1 이상이어야 합니다".into(),
            ));
        }
        if self.worker.max_queue_depth == 0 {
            return Err(ConfigError::Invalid(
                "max_queue_depth 는 1 이상이어야 합니다".into(),
            ));
        }
        if self.node.advertise_address.starts_with("0.0.0.0") {
            return Err(ConfigError::Invalid(
                "advertise_address 는 스케줄러가 실제로 접속할 주소여야 합니다. \
                 0.0.0.0 은 사용할 수 없습니다"
                    .into(),
            ));
        }
        if let BackendConfig::Mock(m) = &self.backend
            && !(0.0..=1.0).contains(&m.error_rate)
        {
            return Err(ConfigError::Invalid(format!(
                "error_rate 는 0.0~1.0 범위여야 합니다: {}",
                m.error_rate
            )));
        }
        check_h2_window(
            "h2_stream_window_bytes",
            self.transport.h2_stream_window_bytes,
        )?;
        check_h2_window(
            "h2_connection_window_bytes",
            self.transport.h2_connection_window_bytes,
        )?;
        Ok(())
    }

    /// Mock 백엔드를 사용하는 설정인지 여부.
    pub const fn is_mock(&self) -> bool {
        matches!(self.backend, BackendConfig::Mock(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// docs/01-TECHSPEC.md §16.1의 예제와 동일해야 한다.
    const SCHEDULER_EXAMPLE: &str = r#"
[server]
grpc_listen = "0.0.0.0:50051"
http_listen = "0.0.0.0:8080"
metrics_listen = "0.0.0.0:9090"
max_payload_bytes = 10485760

[scheduler]
policy = "ect"
request_timeout_ms = 5000
node_rpc_timeout_ms = 3000
max_retries = 1

[health]
heartbeat_interval_ms = 2000
health_timeout_ms = 1000
failure_threshold = 3
recovery_threshold = 3

[thresholds]
busy_queue_depth = 8
degraded_error_rate = 0.10
degraded_temperature_c = 80.0
disable_temperature_c = 90.0

[telemetry]
json_log = true
log_level = "info"
prometheus = false
"#;

    /// docs/01-TECHSPEC.md §16.2, docs/02-HARDWARE-SETUP.md §7.1의 예제.
    const NODE_EXAMPLE: &str = r#"
[node]
id = "king"
scheduler_address = "http://10.20.0.10:50051"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.21:51001"

[worker]
worker_count = 1
max_queue_depth = 32

[backend]
type = "rknn"
runtime_library = "/usr/lib/librknnrt.so"

[models]
directory = "/opt/npuforge/models"
preload = ["yolov8n"]
warmup_runs = 3

[telemetry]
heartbeat_interval_ms = 1000
temperature_path = "/sys/class/thermal/thermal_zone0/temp"
"#;

    #[test]
    fn parses_scheduler_example_from_techspec() {
        let cfg: SchedulerConfig = toml::from_str(SCHEDULER_EXAMPLE).unwrap();
        cfg.validate().unwrap();
        assert_eq!(cfg.scheduler.policy, SchedulingPolicyKind::Ect);
        assert_eq!(cfg.server.max_payload_bytes, 10_485_760);
        assert_eq!(cfg.health.failure_threshold, 3);
    }

    #[test]
    fn parses_node_example_from_techspec() {
        let cfg: NodeConfig = toml::from_str(NODE_EXAMPLE).unwrap();
        cfg.validate().unwrap();
        assert_eq!(cfg.node.id, "king");
        // worker_count 는 [worker] 아래에 있어야 한다. 이 위치가 문서와 어긋나면
        // 세 노드 설정 diff 검증이 깨진다.
        assert_eq!(cfg.worker.worker_count, 1);
        assert_eq!(cfg.worker.max_queue_depth, 32);
        assert!(matches!(cfg.backend, BackendConfig::Rknn { .. }));
        assert_eq!(cfg.models.warmup_runs, 3);
    }

    #[test]
    fn node_worker_keys_must_not_live_under_node_section() {
        // [node] 아래에 worker_count 를 두는 옛 스키마는 거부되어야 한다.
        let legacy = r#"
[node]
id = "king"
scheduler_address = "http://10.20.0.10:50051"
listen = "0.0.0.0:51001"
advertise_address = "10.20.0.21:51001"
worker_count = 1

[backend]
type = "rknn"

[models]
directory = "/opt/npuforge/models"
"#;
        let err = toml::from_str::<NodeConfig>(legacy).unwrap_err();
        assert!(err.to_string().contains("worker_count"), "실제 오류: {err}");
    }

    #[test]
    fn mock_backend_config_parses() {
        let text = r#"
[node]
id = "mock-01"
scheduler_address = "http://127.0.0.1:50051"
listen = "127.0.0.1:51001"
advertise_address = "127.0.0.1:51001"

[worker]
worker_count = 2
max_queue_depth = 16

[backend]
type = "mock"
base_latency_ms = 20
jitter_ms = 5
error_rate = 0.02

[models]
directory = "./models"
preload = ["yolov8n"]
"#;
        let cfg: NodeConfig = toml::from_str(text).unwrap();
        cfg.validate().unwrap();
        assert!(cfg.is_mock());
        let BackendConfig::Mock(m) = &cfg.backend else {
            panic!("mock 이어야 한다")
        };
        assert_eq!(m.base_latency_ms, 20);
        assert_eq!(m.error_rate, 0.02);
        // 지정하지 않은 값은 기본값을 쓴다.
        assert_eq!(m.speed_factor, 1.0);
    }

    #[test]
    fn rejects_advertise_address_wildcard() {
        // 0.0.0.0 을 advertise 하면 스케줄러가 노드에 접속할 수 없다.
        // 세 노드를 배포할 때 실수하기 쉬운 지점이라 파싱 단계에서 막는다.
        let text = NODE_EXAMPLE.replace(
            "advertise_address = \"10.20.0.21:51001\"",
            "advertise_address = \"0.0.0.0:51001\"",
        );
        let cfg: NodeConfig = toml::from_str(&text).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_rpc_timeout_larger_than_request_timeout() {
        // 노드 RPC 타임아웃이 전체 타임아웃보다 크면 재시도 기회가 없다.
        let text =
            SCHEDULER_EXAMPLE.replace("node_rpc_timeout_ms = 3000", "node_rpc_timeout_ms = 6000");
        let cfg: SchedulerConfig = toml::from_str(&text).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_inverted_temperature_thresholds() {
        let text = SCHEDULER_EXAMPLE.replace(
            "disable_temperature_c = 90.0",
            "disable_temperature_c = 70.0",
        );
        let cfg: SchedulerConfig = toml::from_str(&text).unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_legacy_policy_spelling() {
        let text = SCHEDULER_EXAMPLE.replace(r#"policy = "ect""#, r#"policy = "queue-aware""#);
        assert!(toml::from_str::<SchedulerConfig>(&text).is_err());
    }
}
