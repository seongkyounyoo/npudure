//! 스케줄러 → 노드 gRPC 클라이언트.
//!
//! 노드마다 채널을 만들어 재사용한다. HTTP/2 는 하나의 커넥션에서 스트림을
//! 다중화하므로 요청마다 연결하면 매 요청에 TCP+HTTP/2 핸드셰이크가 붙는다.
//! 2.5GbE 링크에서 이 비용은 추론시간(약 13ms)에 비해 무시할 수 없다.
//!
//! **노드당 채널 수는 설정으로 정한다(기본 1 — 기존 동작과 동일).**
//! S3.5 는 −30% 손실이 이 전송 경로에 있다는 것까지 좁혔지만, 그 안에서
//! flow control 때문인지 커넥션이 직렬화 지점이라서인지는 가르지 못했다.
//! `node_connections` 와 h2 window 를 A/B 로 흔들어 그것을 가른다
//! (`docs/experiments/S3_5_TRANSPORT_PROFILE.md` §7).
//!
//! 전송은 [`NodeLink`] 로 추상화한다. 재시도 로직은 실제 노드 없이 테스트한다.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use npuforge_common::{NodeId, config::SchedulerTransportConfig};
use npuforge_proto::v1::{
    NodeInferRequest, NodeInferResponse, node_service_client::NodeServiceClient,
};
use parking_lot::Mutex;
use tonic::transport::Channel;

/// 노드 한 대에 대한 호출.
#[allow(async_fn_in_trait)]
pub trait NodeLink: Send + Sync {
    fn infer(
        &self,
        node_id: &NodeId,
        address: &str,
        req: NodeInferRequest,
    ) -> impl std::future::Future<Output = Result<NodeInferResponse, String>> + Send;
}

/// tonic 채널 풀. 노드당 채널 `n` 개를 만들어 요청마다 돌려 쓴다.
#[derive(Debug, Default)]
pub struct GrpcNodePool {
    channels: Mutex<HashMap<NodeId, Vec<Channel>>>,
    /// 노드당 다음에 쓸 채널 인덱스. round-robin.
    next: Mutex<HashMap<NodeId, usize>>,
    rpc_timeout: Duration,
    max_payload_bytes: usize,
    transport: SchedulerTransportConfig,
}

impl GrpcNodePool {
    /// 기존 호출부 호환용. 전송 설정은 기본값(커넥션 1, h2 기본 window).
    pub fn new(rpc_timeout: Duration, max_payload_bytes: usize) -> Self {
        Self::with_transport(
            rpc_timeout,
            max_payload_bytes,
            SchedulerTransportConfig::default(),
        )
    }

    pub fn with_transport(
        rpc_timeout: Duration,
        max_payload_bytes: usize,
        transport: SchedulerTransportConfig,
    ) -> Self {
        Self {
            channels: Mutex::new(HashMap::new()),
            next: Mutex::new(HashMap::new()),
            rpc_timeout,
            max_payload_bytes,
            transport,
        }
    }

    /// 노드용 채널을 가져오거나 만든다. 요청마다 round-robin 으로 고른다.
    ///
    /// lazy 연결이므로 노드가 아직 안 떠 있어도 여기서 실패하지 않는다.
    /// 실제 실패는 호출 시점에 드러나고, 그 편이 재시도 판정에 유리하다.
    fn channel(&self, node_id: &NodeId, address: &str) -> Result<Channel, String> {
        if let Some(chs) = self.channels.lock().get(node_id) {
            return Ok(self.pick(node_id, chs));
        }
        // tonic 은 scheme 없는 주소도 파싱은 통과시키고 연결 시점에야 실패한다.
        // 노드 설정은 손으로 쓰는 TOML 이라 `http://` 누락이 흔하다.
        // 여기서 막지 않으면 "노드가 응답하지 않는다"로만 보인다.
        if !address.starts_with("http://") && !address.starts_with("https://") {
            return Err(format!(
                "노드 '{node_id}' 주소 '{address}' 에 scheme 이 없습니다.                  advertise_address 는 'http://호스트:포트' 형식이어야 합니다"
            ));
        }

        let n = self.transport.node_connections.max(1);
        let mut chs = Vec::with_capacity(n);
        for _ in 0..n {
            chs.push(self.connect_one(node_id, address)?);
        }
        let picked = self.pick(node_id, &chs);
        self.channels.lock().insert(node_id.clone(), chs);
        Ok(picked)
    }

    /// 채널 하나를 만든다. 전송 설정을 여기서 적용한다.
    fn connect_one(&self, node_id: &NodeId, address: &str) -> Result<Channel, String> {
        let mut endpoint = Channel::from_shared(address.to_owned())
            .map_err(|e| format!("노드 '{node_id}' 주소 '{address}' 형식 오류: {e}"))?
            .connect_timeout(Duration::from_secs(2))
            .timeout(self.rpc_timeout)
            // 노드가 죽었을 때 커넥션이 좀비로 남지 않게 한다
            .tcp_keepalive(Some(Duration::from_secs(10)))
            .http2_keep_alive_interval(Duration::from_secs(5))
            .keep_alive_timeout(Duration::from_secs(5));

        // 여기서 정하는 것은 **이쪽이 수신하는 방향**, 즉 노드 → 스케줄러
        // 응답(1.218 MB)의 window 다. 요청 방향은 노드가 광고한다.
        if let Some(v) = self.transport.h2_stream_window_bytes {
            endpoint = endpoint.initial_stream_window_size(v);
        }
        if let Some(v) = self.transport.h2_connection_window_bytes {
            endpoint = endpoint.initial_connection_window_size(v);
        }

        Ok(endpoint.connect_lazy())
    }

    /// round-robin 선택. 커넥션이 1개면 그것을 그대로 준다.
    fn pick(&self, node_id: &NodeId, chs: &[Channel]) -> Channel {
        if chs.len() == 1 {
            return chs[0].clone();
        }
        let mut next = self.next.lock();
        let i = next.entry(node_id.clone()).or_insert(0);
        let ch = chs[*i % chs.len()].clone();
        *i = i.wrapping_add(1);
        ch
    }

    /// 노드가 등록 해제되거나 주소가 바뀌면 채널을 버린다.
    pub fn evict(&self, node_id: &NodeId) {
        self.channels.lock().remove(node_id);
        self.next.lock().remove(node_id);
    }
}

impl NodeLink for GrpcNodePool {
    async fn infer(
        &self,
        node_id: &NodeId,
        address: &str,
        req: NodeInferRequest,
    ) -> Result<NodeInferResponse, String> {
        let channel = self.channel(node_id, address)?;
        let mut client = NodeServiceClient::new(channel)
            .max_decoding_message_size(self.max_payload_bytes)
            .max_encoding_message_size(self.max_payload_bytes);
        client
            .infer(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| format!("{}: {}", e.code(), e.message()))
    }
}

/// 재시도 백오프. 스케줄러 설정의 `retry_backoff_*_ms` 에서 온다.
#[derive(Debug, Clone, Copy)]
pub struct RetryBackoff {
    pub min: Duration,
    pub max: Duration,
}

impl RetryBackoff {
    /// `attempt` 는 0부터. 지수 증가 후 상한에서 포화한다.
    ///
    /// 지터는 넣지 않는다. 벤치마크에서 재현 가능해야 하고, 노드가 3대뿐이라
    /// thundering herd 문제가 실질적으로 없다.
    pub fn delay(&self, attempt: u32) -> Duration {
        let factor = 1u32.checked_shl(attempt).unwrap_or(u32::MAX);
        self.min.saturating_mul(factor).min(self.max)
    }
}

pub type SharedNodeLink = Arc<dyn ErasedNodeLink>;

/// `NodeLink` 를 dyn 으로 쓰기 위한 object-safe 래퍼.
///
/// `impl Future` 를 반환하는 trait 은 그대로는 dyn 이 되지 않는다.
pub trait ErasedNodeLink: Send + Sync {
    fn infer_boxed<'a>(
        &'a self,
        node_id: &'a NodeId,
        address: &'a str,
        req: NodeInferRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<NodeInferResponse, String>> + Send + 'a>,
    >;
}

impl<T: NodeLink> ErasedNodeLink for T {
    fn infer_boxed<'a>(
        &'a self,
        node_id: &'a NodeId,
        address: &'a str,
        req: NodeInferRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<NodeInferResponse, String>> + Send + 'a>,
    > {
        Box::pin(self.infer(node_id, address, req))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_and_saturates() {
        let b = RetryBackoff {
            min: Duration::from_millis(10),
            max: Duration::from_millis(100),
        };
        assert_eq!(b.delay(0), Duration::from_millis(10));
        assert_eq!(b.delay(1), Duration::from_millis(20));
        assert_eq!(b.delay(3), Duration::from_millis(80));
        assert_eq!(b.delay(4), Duration::from_millis(100));
        // 큰 attempt 에서 시프트 오버플로로 패닉하면 안 된다
        assert_eq!(b.delay(64), Duration::from_millis(100));
        assert_eq!(b.delay(u32::MAX), Duration::from_millis(100));
    }

    // connect_lazy 는 tokio 리액터를 요구한다.
    #[tokio::test]
    async fn channel_is_reused_per_node() {
        let pool = GrpcNodePool::new(Duration::from_secs(1), 16 * 1024 * 1024);
        let id: NodeId = "king".into();
        pool.channel(&id, "http://192.168.123.22:50052").unwrap();
        pool.channel(&id, "http://192.168.123.22:50052").unwrap();
        assert_eq!(
            pool.channels.lock().len(),
            1,
            "노드당 채널 하나를 재사용한다"
        );
    }

    #[tokio::test]
    async fn evict_drops_channel() {
        let pool = GrpcNodePool::new(Duration::from_secs(1), 16 * 1024 * 1024);
        let id: NodeId = "queen".into();
        pool.channel(&id, "http://192.168.123.16:50052").unwrap();
        pool.evict(&id);
        assert!(pool.channels.lock().is_empty());
    }

    #[tokio::test]
    async fn address_without_scheme_is_rejected_up_front() {
        // tonic 은 이 주소를 파싱만 통과시키고 채널을 만들어 버린다.
        // 그러면 첫 추론 요청에서야 실패하고, 로그에는 "노드 무응답"으로만
        // 남아 원인이 오타라는 것을 알기 어렵다.
        let pool = GrpcNodePool::new(Duration::from_secs(1), 16 * 1024 * 1024);
        let err = pool
            .channel(&"jack".into(), "192.168.123.33:50052")
            .unwrap_err();
        assert!(
            err.contains("jack"),
            "어느 노드인지 알 수 있어야 한다: {err}"
        );
        assert!(
            err.contains("scheme"),
            "무엇이 잘못됐는지 알려야 한다: {err}"
        );
        assert!(
            pool.channels.lock().is_empty(),
            "실패한 주소를 캐시하면 설정을 고쳐도 계속 실패한다"
        );
    }
}
