//! 로컬 작업 큐와 워커 풀.
//!
//! RKNN Runtime 2.3.0 은 thread-safe 이므로 모델당 전용 스레드로 직렬화할
//! 필요가 없다(`docs/environment-matrix.md` §3.1). 워커가 같은 `LoadedModel` 을
//! 공유해 동시에 호출한다.
//!
//! `worker_count` 기본값은 8이다. RK3576 은 NPU 코어가 2개지만 8스레드에서
//! 처리량이 가장 높았다. 상세는 `docs/discuss.md` §4.

use std::sync::{
    Arc,
    atomic::{AtomicU32, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

use npuforge_common::{
    ErrorCode, NpuForgeError, Result,
    backend::{InferenceInput, InferenceOutput, LoadedModel},
    protocol::{InputFormat, TimingBreakdown},
};
use tokio::sync::{Semaphore, mpsc, oneshot};

/// 워커에 전달되는 작업 하나.
struct Job {
    input: InferenceInput,
    /// 큐에 들어온 시각. 큐 대기시간과 타임아웃 판정에 쓴다.
    enqueued_at: Instant,
    /// 이 시각을 넘겨 대기했으면 실행하지 않고 거절한다.
    deadline: Option<Instant>,
    reply: oneshot::Sender<Result<InferenceOutput>>,
}

/// 노드의 추론 실행기.
///
/// 큐 깊이와 in-flight 수를 추적해 스케줄러에 보고한다.
pub struct WorkerPool {
    tx: mpsc::Sender<Job>,
    queue_depth: Arc<AtomicU32>,
    in_flight: Arc<AtomicU32>,
    /// 최근 결과 집계. 오류율 계산에 쓴다.
    completed: Arc<AtomicU64>,
    failed: Arc<AtomicU64>,
    max_queue_depth: usize,
    worker_count: usize,
}

impl WorkerPool {
    /// 워커를 기동한다.
    ///
    /// `model` 은 모든 워커가 공유한다. 동시 호출이 안전함이 검증되어 있다.
    pub fn spawn(model: Arc<dyn LoadedModel>, worker_count: usize, max_queue_depth: usize) -> Self {
        let worker_count = worker_count.max(1);
        let max_queue_depth = max_queue_depth.max(1);

        let (tx, rx) = mpsc::channel::<Job>(max_queue_depth);
        let queue_depth = Arc::new(AtomicU32::new(0));
        let in_flight = Arc::new(AtomicU32::new(0));
        let completed = Arc::new(AtomicU64::new(0));
        let failed = Arc::new(AtomicU64::new(0));

        // 동시 실행 수를 워커 수로 제한한다. 채널 수신은 한 곳에서만 하고
        // 세마포어로 병렬도를 통제하는 편이 워커마다 채널을 나누는 것보다
        // 부하 분배가 고르다.
        let permits = Arc::new(Semaphore::new(worker_count));

        tokio::spawn(dispatch_loop(
            rx,
            model,
            permits,
            queue_depth.clone(),
            in_flight.clone(),
            completed.clone(),
            failed.clone(),
        ));

        Self {
            tx,
            queue_depth,
            in_flight,
            completed,
            failed,
            max_queue_depth,
            worker_count,
        }
    }

    /// 추론을 요청한다.
    ///
    /// 큐가 가득 차면 즉시 `NPF-1303 NODE_OVERLOADED` 로 거절한다.
    /// 스케줄러가 다른 노드로 재시도할 수 있게 하기 위함이며, 무한정
    /// 대기시키는 것보다 전체 지연이 짧다.
    pub async fn submit(
        &self,
        input: InferenceInput,
        queue_timeout: Option<Duration>,
    ) -> Result<InferenceOutput> {
        let (reply_tx, reply_rx) = oneshot::channel();
        let now = Instant::now();
        let job = Job {
            input,
            enqueued_at: now,
            deadline: queue_timeout.map(|d| now + d),
            reply: reply_tx,
        };

        self.queue_depth.fetch_add(1, Ordering::Relaxed);
        let send_result = self.tx.try_send(job);

        match send_result {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.queue_depth.fetch_sub(1, Ordering::Relaxed);
                self.failed.fetch_add(1, Ordering::Relaxed);
                return Err(NpuForgeError::new(
                    ErrorCode::NodeOverloaded,
                    format!(
                        "로컬 큐가 가득 찼습니다 (max_queue_depth={})",
                        self.max_queue_depth
                    ),
                ));
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.queue_depth.fetch_sub(1, Ordering::Relaxed);
                return Err(NpuForgeError::new(
                    ErrorCode::NodeUnavailable,
                    "워커 풀이 종료되었습니다",
                ));
            }
        }

        match reply_rx.await {
            Ok(result) => result,
            Err(_) => Err(NpuForgeError::internal("워커가 응답 없이 종료되었습니다")),
        }
    }

    pub fn queue_depth(&self) -> u32 {
        self.queue_depth.load(Ordering::Relaxed)
    }

    pub fn in_flight(&self) -> u32 {
        self.in_flight.load(Ordering::Relaxed)
    }

    pub fn worker_count(&self) -> usize {
        self.worker_count
    }

    /// 누적 오류율. 스케줄러의 `Degraded` 판정에 쓰인다.
    pub fn error_rate(&self) -> f64 {
        let ok = self.completed.load(Ordering::Relaxed);
        let ng = self.failed.load(Ordering::Relaxed);
        let total = ok + ng;
        if total == 0 {
            0.0
        } else {
            ng as f64 / total as f64
        }
    }

    pub fn stats(&self) -> (u64, u64) {
        (
            self.completed.load(Ordering::Relaxed),
            self.failed.load(Ordering::Relaxed),
        )
    }
}

#[allow(clippy::too_many_arguments)]
async fn dispatch_loop(
    mut rx: mpsc::Receiver<Job>,
    model: Arc<dyn LoadedModel>,
    permits: Arc<Semaphore>,
    queue_depth: Arc<AtomicU32>,
    in_flight: Arc<AtomicU32>,
    completed: Arc<AtomicU64>,
    failed: Arc<AtomicU64>,
) {
    while let Some(job) = rx.recv().await {
        queue_depth.fetch_sub(1, Ordering::Relaxed);

        // 워커가 빌 때까지 기다린다. 대기시간의 대부분이 여기서 발생한다.
        let permit = match permits.clone().acquire_owned().await {
            Ok(p) => p,
            Err(_) => break, // 세마포어가 닫혔다
        };

        // 대기시간은 permit 을 얻은 뒤에 확정한다.
        //
        // permit 획득 전에만 검사하면 채널에서 꺼낸 직후라 대기시간이 거의 0 이고,
        // 정작 워커를 기다리는 구간이 판정에서 빠진다. 그 상태로는 타임아웃이
        // 사실상 동작하지 않는다.
        let waited = job.enqueued_at.elapsed();

        // 이미 기한이 지난 요청은 실행하지 않는다.
        // 클라이언트가 포기했을 가능성이 높고, 실행해도 NPU 만 낭비된다.
        if let Some(deadline) = job.deadline
            && Instant::now() >= deadline
        {
            failed.fetch_add(1, Ordering::Relaxed);
            let _ = job.reply.send(Err(NpuForgeError::new(
                ErrorCode::NodeOverloaded,
                format!("로컬 큐 대기시간 초과 ({} ms)", waited.as_millis()),
            )));
            drop(permit);
            continue;
        }

        let model = model.clone();
        let in_flight = in_flight.clone();
        let completed = completed.clone();
        let failed = failed.clone();

        tokio::spawn(async move {
            in_flight.fetch_add(1, Ordering::Relaxed);
            let result = model.infer(job.input).await;
            in_flight.fetch_sub(1, Ordering::Relaxed);

            let result = match result {
                Ok(mut out) => {
                    completed.fetch_add(1, Ordering::Relaxed);
                    // 백엔드는 큐 대기시간을 모른다. 여기서 채운다.
                    out.timing.node_queue_us = waited.as_micros() as u64;
                    Ok(out)
                }
                Err(e) => {
                    failed.fetch_add(1, Ordering::Relaxed);
                    Err(e)
                }
            };

            let _ = job.reply.send(result);
            drop(permit);
        });
    }
}

/// 요청 payload 에서 백엔드 입력을 만든다.
pub fn make_input(payload: bytes::Bytes, format: InputFormat) -> InferenceInput {
    InferenceInput { payload, format }
}

/// 노드가 채우지 않는 스케줄러 구간을 0으로 둔 타이밍.
pub fn node_timing(out: &InferenceOutput) -> TimingBreakdown {
    out.timing
}

#[cfg(test)]
mod tests {
    use super::*;
    use npuforge_common::{config::MockBackendConfig, model::ModelSpec, protocol::InputFormat};
    use npuforge_mock_backend::MockModel;

    fn spec() -> ModelSpec {
        ModelSpec {
            id: "yolov8n".into(),
            version: "1.0.0".into(),
            backend: "mock".into(),
            model_file: "model.rknn".into(),
            input_width: 640,
            input_height: 640,
            input_channels: 3,
            input_format: InputFormat::Rgb8,
            output_format: "yolo-detections".into(),
            sha256: "0".repeat(64),
            labels_file: None,
        }
    }

    fn model(cfg: MockBackendConfig) -> Arc<dyn LoadedModel> {
        Arc::new(MockModel::with_seed(spec(), cfg, 42))
    }

    fn input() -> InferenceInput {
        InferenceInput {
            payload: bytes::Bytes::from_static(b"test"),
            format: InputFormat::Jpeg,
        }
    }

    #[tokio::test]
    async fn single_request_succeeds() {
        let pool = WorkerPool::spawn(
            model(MockBackendConfig {
                base_latency_ms: 1,
                jitter_ms: 0,
                ..Default::default()
            }),
            2,
            8,
        );
        let out = pool.submit(input(), None).await.unwrap();
        assert!(out.timing.inference_us > 0);
        assert_eq!(pool.stats().0, 1);
    }

    #[tokio::test]
    async fn queue_wait_time_is_recorded() {
        // 백엔드는 큐 대기시간을 모른다. 워커 풀이 채워야 한다.
        let pool = WorkerPool::spawn(
            model(MockBackendConfig {
                base_latency_ms: 30,
                jitter_ms: 0,
                ..Default::default()
            }),
            1,
            16,
        );
        // 워커 1개에 요청 4개 → 뒤쪽은 반드시 기다린다
        let pool = Arc::new(pool);
        let mut handles = Vec::new();
        for _ in 0..4 {
            let p = pool.clone();
            handles.push(tokio::spawn(async move { p.submit(input(), None).await }));
        }
        let mut waits = Vec::new();
        for h in handles {
            waits.push(h.await.unwrap().unwrap().timing.node_queue_us);
        }
        assert!(
            waits.iter().any(|&w| w > 0),
            "동시 요청 중 일부는 큐 대기시간이 잡혀야 한다: {waits:?}"
        );
    }

    #[tokio::test]
    async fn full_queue_rejects_immediately() {
        // 큐가 가득 차면 대기시키지 않고 거절해야 스케줄러가
        // 다른 노드로 재시도할 수 있다.
        let pool = WorkerPool::spawn(
            model(MockBackendConfig {
                base_latency_ms: 200,
                jitter_ms: 0,
                ..Default::default()
            }),
            1,
            2,
        );

        // 워커 1개, 큐 2칸에 16건을 한꺼번에 밀어넣는다.
        let pool = Arc::new(pool);
        let mut handles = Vec::new();
        for _ in 0..16 {
            let p = pool.clone();
            handles.push(tokio::spawn(async move { p.submit(input(), None).await }));
        }

        let mut overloaded = 0;
        for h in handles {
            if let Err(e) = h.await.unwrap()
                && e.code == ErrorCode::NodeOverloaded
            {
                overloaded += 1;
            }
        }
        assert!(
            overloaded > 0,
            "큐 포화 시 일부는 NODE_OVERLOADED 로 즉시 거절해야 한다"
        );
    }

    #[tokio::test]
    async fn queue_timeout_skips_execution() {
        // 이미 기한이 지난 요청은 NPU 를 쓰지 않고 버려야 한다.
        let pool = WorkerPool::spawn(
            model(MockBackendConfig {
                base_latency_ms: 50,
                jitter_ms: 0,
                ..Default::default()
            }),
            1,
            16,
        );
        // 워커 하나를 먼저 점유시켜 두 번째 요청이 반드시 큐에서 기다리게 한다.
        let pool = Arc::new(pool);
        let p1 = pool.clone();
        let blocker = tokio::spawn(async move { p1.submit(input(), None).await });
        tokio::time::sleep(Duration::from_millis(5)).await;

        // 기한 1ms. 앞 요청이 50ms 걸리므로 반드시 초과한다.
        let t = pool.submit(input(), Some(Duration::from_millis(1))).await;
        let _ = blocker.await;

        let err = t.unwrap_err();
        assert_eq!(err.code, ErrorCode::NodeOverloaded);
        assert!(
            err.message.contains("대기시간 초과"),
            "실제: {}",
            err.message
        );
    }

    #[tokio::test]
    async fn error_rate_tracks_failures() {
        let pool = WorkerPool::spawn(
            model(MockBackendConfig {
                base_latency_ms: 1,
                jitter_ms: 0,
                error_rate: 1.0,
                ..Default::default()
            }),
            2,
            8,
        );
        for _ in 0..5 {
            let _ = pool.submit(input(), None).await;
        }
        assert_eq!(pool.error_rate(), 1.0);
    }

    #[tokio::test]
    async fn concurrency_is_bounded_by_worker_count() {
        let pool = WorkerPool::spawn(
            model(MockBackendConfig {
                base_latency_ms: 40,
                jitter_ms: 0,
                ..Default::default()
            }),
            2,
            32,
        );
        let pool = Arc::new(pool);
        let start = Instant::now();
        let mut handles = Vec::new();
        for _ in 0..8 {
            let p = pool.clone();
            handles.push(tokio::spawn(async move { p.submit(input(), None).await }));
        }
        let mut results = Vec::new();
        for h in handles {
            results.push(h.await.unwrap());
        }
        let elapsed = start.elapsed();

        assert!(results.iter().all(|r| r.is_ok()));
        // 워커 2개로 8건 × 40ms 이면 최소 4배치 = 160ms 이상
        assert!(
            elapsed >= Duration::from_millis(120),
            "워커 수 제한이 동작해야 한다: {elapsed:?}"
        );
    }
}
