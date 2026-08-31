//! 스케줄러 등록 및 하트비트 클라이언트.
//!
//! 노드는 기동 직후 스케줄러에 등록하고, 그 뒤 주기적으로 하트비트를 보낸다.
//! 스케줄러가 먼저 떠 있으리라 가정하지 않는다 — 실측 환경에서 세 보드는
//! 전원 인가 순서가 매번 다르고, 어댑터 용량 부족으로 리부팅되기도 했다.
//! 따라서 등록은 성공할 때까지 백오프하며 재시도한다.
//!
//! 전송 계층은 [`SchedulerLink`] 로 분리해 두었다. 재시도·백오프·재등록 판정은
//! 실제 gRPC 연결 없이 테스트한다.

use std::time::Duration;

use npuforge_proto::v1::{
    self, DeregisterNodeRequest, HeartbeatRequest, RegisterNodeRequest,
    scheduler_service_client::SchedulerServiceClient,
};
use tokio::sync::watch;
use tonic::transport::Channel;

/// 등록·하트비트 백오프 정책.
#[derive(Debug, Clone)]
pub struct BackoffPolicy {
    pub initial: Duration,
    pub max: Duration,
    pub multiplier: u32,
}

impl Default for BackoffPolicy {
    fn default() -> Self {
        Self {
            initial: Duration::from_millis(500),
            max: Duration::from_secs(15),
            multiplier: 2,
        }
    }
}

impl BackoffPolicy {
    /// `attempt` 는 0부터 시작한다. 상한에서 포화한다.
    pub fn delay(&self, attempt: u32) -> Duration {
        let mut d = self.initial;
        for _ in 0..attempt {
            match d.checked_mul(self.multiplier) {
                Some(next) if next < self.max => d = next,
                _ => return self.max,
            }
        }
        d
    }
}

/// 스케줄러 호출을 추상화한다. 테스트에서 대체 구현을 넣는다.
#[allow(async_fn_in_trait)]
pub trait SchedulerLink {
    /// 등록 결과. `Ok(interval)` 은 스케줄러가 원하는 하트비트 주기(ms).
    async fn register(&mut self, req: RegisterNodeRequest) -> Result<u64, String>;
    /// `Ok(must_reregister)`.
    async fn heartbeat(&mut self, req: HeartbeatRequest) -> Result<bool, String>;
    async fn deregister(&mut self, req: DeregisterNodeRequest) -> Result<(), String>;
}

/// 스케줄러가 주기를 지정하지 않았을 때 쓰는 기본값.
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 1_000;

/// 하트비트 주기의 하한. 스케줄러가 0 이나 비상식적으로 작은 값을 주더라도
/// 노드가 자기 자신을 하트비트로 마비시키지 않게 한다.
pub const MIN_HEARTBEAT_INTERVAL_MS: u64 = 100;

pub fn clamp_heartbeat_interval(ms: u64) -> Duration {
    if ms == 0 {
        return Duration::from_millis(DEFAULT_HEARTBEAT_INTERVAL_MS);
    }
    Duration::from_millis(ms.max(MIN_HEARTBEAT_INTERVAL_MS))
}

/// 등록 → 하트비트 루프를 수행한다.
///
/// - 등록 실패는 백오프하며 무한 재시도한다.
/// - 하트비트 실패는 곧바로 재등록으로 전환한다. 스케줄러가 재시작해
///   노드를 잊어버린 경우와 일시적 네트워크 오류를 구분할 수 없기 때문에,
///   더 비싼 쪽(재등록)을 택한다. 등록은 멱등이다.
/// - `must_reregister` 를 받으면 즉시 재등록한다.
///
/// `shutdown` 이 `true` 로 바뀌면 등록 해제를 시도하고 반환한다.
pub async fn run_loop<L, H>(
    link: &mut L,
    descriptor: v1::NodeDescriptor,
    token: String,
    mut health_fn: H,
    backoff: BackoffPolicy,
    mut shutdown: watch::Receiver<bool>,
) where
    L: SchedulerLink,
    H: FnMut() -> v1::NodeHealth,
{
    let node_id = descriptor.node_id.clone();

    'outer: loop {
        // --- 등록 ---
        let mut attempt = 0u32;
        let interval = loop {
            if *shutdown.borrow() {
                return;
            }
            let req = RegisterNodeRequest {
                descriptor: Some(descriptor.clone()),
                token: token.clone(),
            };
            match link.register(req).await {
                Ok(ms) => {
                    tracing::info!(node_id = %node_id, interval_ms = ms, "스케줄러 등록 완료");
                    break clamp_heartbeat_interval(ms);
                }
                Err(e) => {
                    let delay = backoff.delay(attempt);
                    tracing::warn!(
                        node_id = %node_id,
                        attempt,
                        retry_in_ms = delay.as_millis() as u64,
                        error = %e,
                        "스케줄러 등록 실패, 재시도"
                    );
                    attempt = attempt.saturating_add(1);
                    if sleep_or_shutdown(delay, &mut shutdown).await {
                        return;
                    }
                }
            }
        };

        // --- 하트비트 ---
        loop {
            if sleep_or_shutdown(interval, &mut shutdown).await {
                break 'outer;
            }
            let req = HeartbeatRequest {
                node_id: node_id.to_string(),
                health: Some(health_fn()),
            };
            match link.heartbeat(req).await {
                Ok(false) => {}
                Ok(true) => {
                    tracing::info!(node_id = %node_id, "스케줄러가 재등록을 요구했다");
                    continue 'outer;
                }
                Err(e) => {
                    tracing::warn!(node_id = %node_id, error = %e, "하트비트 실패, 재등록한다");
                    continue 'outer;
                }
            }
        }
    }

    // --- 종료 ---
    let req = DeregisterNodeRequest {
        node_id: node_id.to_string(),
        reason: "노드 종료".into(),
    };
    if let Err(e) = link.deregister(req).await {
        // 등록 해제 실패는 치명적이지 않다. 스케줄러가 하트비트 부재로 판정한다.
        tracing::warn!(node_id = %node_id, error = %e, "등록 해제 실패");
    }
}

/// `true` 를 반환하면 종료 신호를 받은 것이다.
async fn sleep_or_shutdown(d: Duration, shutdown: &mut watch::Receiver<bool>) -> bool {
    if *shutdown.borrow() {
        return true;
    }
    tokio::select! {
        _ = tokio::time::sleep(d) => false,
        r = shutdown.changed() => r.is_err() || *shutdown.borrow(),
    }
}

/// tonic 기반 실제 전송 구현.
pub struct GrpcLink {
    client: SchedulerServiceClient<Channel>,
}

impl GrpcLink {
    /// 스케줄러에 연결한다. 스케줄러가 아직 안 떠 있어도 실패하지 않도록
    /// lazy 연결을 쓴다. 실제 접속은 첫 호출에서 일어난다.
    pub fn connect_lazy(address: &str) -> Result<Self, String> {
        let endpoint = Channel::from_shared(address.to_owned())
            .map_err(|e| format!("스케줄러 주소 '{address}' 형식이 잘못되었습니다: {e}"))?
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(5));
        Ok(Self {
            client: SchedulerServiceClient::new(endpoint.connect_lazy()),
        })
    }
}

impl SchedulerLink for GrpcLink {
    async fn register(&mut self, req: RegisterNodeRequest) -> Result<u64, String> {
        let resp = self
            .client
            .register_node(req)
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        if !resp.accepted {
            return Err(format!("{} {}", resp.error_code, resp.error_message));
        }
        Ok(resp.heartbeat_interval_ms as u64)
    }

    async fn heartbeat(&mut self, req: HeartbeatRequest) -> Result<bool, String> {
        let resp = self
            .client
            .heartbeat(req)
            .await
            .map_err(|e| e.to_string())?
            .into_inner();
        if !resp.accepted {
            // 거부는 곧 재등록 대상이다
            return Ok(true);
        }
        Ok(resp.must_reregister)
    }

    async fn deregister(&mut self, req: DeregisterNodeRequest) -> Result<(), String> {
        self.client
            .deregister_node(req)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Default)]
    struct Calls {
        register: AtomicU32,
        heartbeat: AtomicU32,
        deregister: AtomicU32,
    }

    /// 스크립트대로 응답하는 테스트용 링크.
    struct FakeLink {
        calls: Arc<Calls>,
        /// 앞에서부터 소비한다. 비면 성공으로 간주한다.
        register_script: Vec<Result<u64, String>>,
        heartbeat_script: Vec<Result<bool, String>>,
        done: watch::Sender<bool>,
        /// 하트비트 스크립트를 다 쓰면 종료 신호를 보낸다
        stop_when_script_empty: bool,
    }

    impl SchedulerLink for FakeLink {
        async fn register(&mut self, _req: RegisterNodeRequest) -> Result<u64, String> {
            self.calls.register.fetch_add(1, Ordering::SeqCst);
            if self.register_script.is_empty() {
                Ok(0)
            } else {
                self.register_script.remove(0)
            }
        }

        async fn heartbeat(&mut self, _req: HeartbeatRequest) -> Result<bool, String> {
            self.calls.heartbeat.fetch_add(1, Ordering::SeqCst);
            if self.heartbeat_script.is_empty() {
                if self.stop_when_script_empty {
                    let _ = self.done.send(true);
                }
                return Ok(false);
            }
            let r = self.heartbeat_script.remove(0);
            if self.heartbeat_script.is_empty() && self.stop_when_script_empty {
                let _ = self.done.send(true);
            }
            r
        }

        async fn deregister(&mut self, _req: DeregisterNodeRequest) -> Result<(), String> {
            self.calls.deregister.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn descriptor() -> v1::NodeDescriptor {
        v1::NodeDescriptor {
            node_id: "king".into(),
            hostname: "king".into(),
            address: "http://192.168.123.22:50052".into(),
            device_type: "nanopi-r76s".into(),
            npu_type: "rk3576".into(),
            npu_core_count: 2,
            memory_bytes: 4 * 1024 * 1024 * 1024,
            agent_version: "0.1.0".into(),
            runtime_version: "2.3.0".into(),
            models: Vec::new(),
        }
    }

    fn fast_backoff() -> BackoffPolicy {
        BackoffPolicy {
            initial: Duration::from_millis(1),
            max: Duration::from_millis(4),
            multiplier: 2,
        }
    }

    async fn run(
        register_script: Vec<Result<u64, String>>,
        heartbeat_script: Vec<Result<bool, String>>,
    ) -> Arc<Calls> {
        let calls = Arc::new(Calls::default());
        let (tx, rx) = watch::channel(false);
        let mut link = FakeLink {
            calls: Arc::clone(&calls),
            register_script,
            heartbeat_script,
            done: tx,
            stop_when_script_empty: true,
        };
        // 하트비트 주기를 짧게 만들기 위해 등록 응답으로 최소값을 준다.
        tokio::time::timeout(
            Duration::from_secs(5),
            run_loop(
                &mut link,
                descriptor(),
                String::new(),
                v1::NodeHealth::default,
                fast_backoff(),
                rx,
            ),
        )
        .await
        .expect("루프가 종료 신호에 반응해야 한다");
        calls
    }

    #[tokio::test]
    async fn backoff_grows_and_saturates() {
        let p = BackoffPolicy {
            initial: Duration::from_millis(500),
            max: Duration::from_secs(15),
            multiplier: 2,
        };
        assert_eq!(p.delay(0), Duration::from_millis(500));
        assert_eq!(p.delay(1), Duration::from_secs(1));
        assert_eq!(p.delay(4), Duration::from_secs(8));
        // 상한에서 포화한다. 오버플로로 패닉하지 않는다.
        assert_eq!(p.delay(5), Duration::from_secs(15));
        assert_eq!(p.delay(u32::MAX), Duration::from_secs(15));
    }

    #[test]
    fn heartbeat_interval_is_clamped() {
        // 스케줄러가 0 을 주면 기본값을 쓴다 (proto3 미설정 필드)
        assert_eq!(
            clamp_heartbeat_interval(0),
            Duration::from_millis(DEFAULT_HEARTBEAT_INTERVAL_MS)
        );
        // 비상식적으로 작은 값은 하한으로 올린다
        assert_eq!(
            clamp_heartbeat_interval(1),
            Duration::from_millis(MIN_HEARTBEAT_INTERVAL_MS)
        );
        assert_eq!(clamp_heartbeat_interval(2_000), Duration::from_secs(2));
    }

    #[tokio::test]
    async fn register_retries_until_scheduler_is_up() {
        // 스케줄러가 늦게 뜨는 상황. 노드가 포기하면 안 된다.
        let calls = run(
            vec![
                Err("connection refused".into()),
                Err("connection refused".into()),
                Ok(MIN_HEARTBEAT_INTERVAL_MS),
            ],
            vec![Ok(false)],
        )
        .await;
        assert_eq!(calls.register.load(Ordering::SeqCst), 3);
        assert!(calls.heartbeat.load(Ordering::SeqCst) >= 1);
    }

    #[tokio::test]
    async fn must_reregister_triggers_new_registration() {
        let calls = run(
            vec![Ok(MIN_HEARTBEAT_INTERVAL_MS), Ok(MIN_HEARTBEAT_INTERVAL_MS)],
            vec![Ok(true), Ok(false)],
        )
        .await;
        assert_eq!(
            calls.register.load(Ordering::SeqCst),
            2,
            "must_reregister 를 받으면 다시 등록해야 한다"
        );
    }

    #[tokio::test]
    async fn heartbeat_failure_falls_back_to_registration() {
        // 스케줄러 재시작과 일시적 네트워크 오류를 구분할 수 없으므로 재등록한다.
        let calls = run(
            vec![Ok(MIN_HEARTBEAT_INTERVAL_MS), Ok(MIN_HEARTBEAT_INTERVAL_MS)],
            vec![Err("transport error".into()), Ok(false)],
        )
        .await;
        assert_eq!(calls.register.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn shutdown_deregisters() {
        let calls = run(vec![Ok(MIN_HEARTBEAT_INTERVAL_MS)], vec![Ok(false)]).await;
        assert_eq!(
            calls.deregister.load(Ordering::SeqCst),
            1,
            "정상 종료 시 등록을 해제해야 한다"
        );
    }

    #[tokio::test]
    async fn shutdown_before_registration_does_not_deregister() {
        // 등록도 못 한 노드가 등록 해제를 보내면 스케줄러 로그만 더럽힌다.
        let calls = Arc::new(Calls::default());
        let (tx, rx) = watch::channel(true); // 처음부터 종료 상태
        let mut link = FakeLink {
            calls: Arc::clone(&calls),
            register_script: vec![Err("down".into())],
            heartbeat_script: Vec::new(),
            done: tx,
            stop_when_script_empty: false,
        };
        tokio::time::timeout(
            Duration::from_secs(2),
            run_loop(
                &mut link,
                descriptor(),
                String::new(),
                v1::NodeHealth::default,
                fast_backoff(),
                rx,
            ),
        )
        .await
        .unwrap();
        assert_eq!(calls.register.load(Ordering::SeqCst), 0);
        assert_eq!(calls.deregister.load(Ordering::SeqCst), 0);
    }
}
