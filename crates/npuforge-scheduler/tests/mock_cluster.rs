//! 로컬 3노드 Mock 클러스터 통합 테스트.
//!
//! 실제 gRPC 를 타고 스케줄러 ↔ 노드가 붙는다. 프로세스만 하나일 뿐
//! 전송 경로는 실장비와 같다. NanoPi 없이도 다음을 검증한다.
//!
//!   - 노드 등록과 하트비트가 실제로 연결을 맺는가
//!   - 요청이 세 노드에 분산되는가
//!   - 노드 한 대가 죽으면 다른 노드로 넘어가는가 (이 프로젝트의 핵심 주장)
//!
//! `docs/01-TECHSPEC.md` §26 (M2 완료 기준) 참조.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use npuforge_common::{
    config::{MockBackendConfig, SchedulerConfig},
    model::ModelSpec,
    policy::SchedulingPolicyKind,
    protocol::InputFormat,
};
use npuforge_mock_backend::MockModel;
use npuforge_node::{NodeServer, WorkerPool};
use npuforge_proto::v1::{
    self, InferRequest, RegisterNodeRequest, node_service_server::NodeServiceServer,
    scheduler_service_client::SchedulerServiceClient,
    scheduler_service_server::SchedulerServiceServer,
};
use npuforge_scheduler::{GrpcNodePool, Registry, SchedulerServer, policy_for};
use tokio::net::TcpListener;

fn spec() -> ModelSpec {
    ModelSpec {
        id: "yolov8n".into(),
        version: "1.0.0".into(),
        backend: "mock".into(),
        model_file: "yolov8n.rknn".into(),
        input_width: 32,
        input_height: 32,
        input_channels: 3,
        input_format: InputFormat::Rgb8,
        output_format: "yolo-detections".into(),
        sha256: "0".repeat(64),
        labels_file: None,
    }
}

/// 실행 중인 Mock 노드 하나.
struct RunningNode {
    node_id: String,
    address: String,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

impl RunningNode {
    /// 노드를 멈춘다. 이후 스케줄러의 호출은 전송 오류가 된다.
    fn kill(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

/// 임의 포트에 Mock 노드를 띄운다.
async fn start_node(node_id: &str, latency_ms: u64) -> RunningNode {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let cfg = MockBackendConfig {
        base_latency_ms: latency_ms,
        jitter_ms: 0,
        ..Default::default()
    };
    let model = Arc::new(MockModel::with_seed(spec(), cfg, 42));
    let pool = Arc::new(WorkerPool::spawn(model, 2, 32));
    let server = Arc::new(NodeServer::new(node_id.into(), pool, spec(), None));

    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(NodeServiceServer::from_arc(server))
            .serve_with_incoming_shutdown(
                tokio_stream::wrappers::TcpListenerStream::new(listener),
                async {
                    let _ = rx.await;
                },
            )
            .await
            .ok();
    });

    RunningNode {
        node_id: node_id.into(),
        address: format!("http://{addr}"),
        shutdown: Some(tx),
    }
}

/// 스케줄러를 임의 포트에 띄우고 (주소, 레지스트리) 를 돌려준다.
async fn start_scheduler(
    policy: SchedulingPolicyKind,
    max_retries: u32,
) -> (String, Arc<Registry>) {
    let mut config = SchedulerConfig::from_path("../../configs/scheduler.example.toml").unwrap();
    config.scheduler.policy = policy;
    config.scheduler.max_retries = max_retries;
    config.scheduler.retry_backoff_min_ms = 1;
    config.scheduler.retry_backoff_max_ms = 5;
    config.scheduler.node_rpc_timeout_ms = 2_000;
    config.scheduler.request_timeout_ms = 10_000;

    let registry = Arc::new(Registry::new(
        config.health.clone(),
        config.thresholds.clone(),
    ));
    let nodes = Arc::new(GrpcNodePool::new(
        Duration::from_millis(config.scheduler.node_rpc_timeout_ms),
        config.server.max_payload_bytes as usize,
    ));
    let server = SchedulerServer::new(
        Arc::clone(&registry),
        Arc::from(policy_for(config.scheduler.policy)),
        nodes,
        Arc::new(config),
        String::new(),
    );

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(SchedulerServiceServer::new(server))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .ok();
    });

    (format!("http://{addr}"), registry)
}

fn descriptor(node: &RunningNode) -> v1::NodeDescriptor {
    v1::NodeDescriptor {
        node_id: node.node_id.clone(),
        hostname: node.node_id.clone(),
        address: node.address.clone(),
        device_type: "mock".into(),
        npu_type: "mock".into(),
        npu_core_count: 2,
        memory_bytes: 0,
        agent_version: "0.1.0".into(),
        runtime_version: "mock-1".into(),
        models: vec![v1::ModelDescriptor {
            id: "yolov8n".into(),
            version: "1.0.0".into(),
            state: v1::ModelState::Ready as i32,
            sha256: "0".repeat(64),
        }],
    }
}

fn infer_req(n: usize) -> InferRequest {
    InferRequest {
        request_id: format!("req-{n}"),
        model_id: "yolov8n".into(),
        payload: vec![7u8; 32 * 32 * 3],
        input_format: v1::InputFormat::Rgb8 as i32,
        priority: 0,
        deadline_unix_ms: 0,
        metadata: Default::default(),
    }
}

/// 3노드 클러스터를 구성하고 등록 + 첫 하트비트까지 끝낸다.
async fn cluster(
    policy: SchedulingPolicyKind,
    max_retries: u32,
) -> (
    SchedulerServiceClient<tonic::transport::Channel>,
    Vec<RunningNode>,
) {
    let (sched_addr, _registry) = start_scheduler(policy, max_retries).await;
    let nodes = vec![
        start_node("king", 5).await,
        start_node("queen", 5).await,
        start_node("jack", 5).await,
    ];

    let mut client = SchedulerServiceClient::connect(sched_addr).await.unwrap();
    for n in &nodes {
        let resp = client
            .register_node(RegisterNodeRequest {
                descriptor: Some(descriptor(n)),
                token: String::new(),
            })
            .await
            .unwrap()
            .into_inner();
        assert!(resp.accepted, "노드 '{}' 등록이 거부되었다", n.node_id);

        // 등록 직후 상태는 Registering 이라 스케줄 대상이 아니다.
        // 실제 노드도 첫 하트비트를 보내야 Healthy 가 된다.
        client
            .heartbeat(v1::HeartbeatRequest {
                node_id: n.node_id.clone(),
                health: Some(v1::NodeHealth {
                    boot_id: format!("boot-{}", n.node_id),
                    ..Default::default()
                }),
            })
            .await
            .unwrap();
    }
    (client, nodes)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn requests_are_distributed_across_three_nodes() {
    let (mut client, _nodes) = cluster(SchedulingPolicyKind::RoundRobin, 1).await;

    let mut seen = HashSet::new();
    for i in 0..9 {
        let r = client.infer(infer_req(i)).await.unwrap().into_inner();
        assert_eq!(r.error_code, "NPF-0000", "{}", r.error_message);
        assert_eq!(r.request_id, format!("req-{i}"));
        assert!(!r.result.is_empty(), "결과가 비어 있다");
        seen.insert(r.node_id);
    }
    assert_eq!(
        seen.len(),
        3,
        "round-robin 은 세 노드를 모두 써야 한다. 실제: {seen:?}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn request_survives_a_dead_node() {
    // 이 프로젝트의 핵심 주장. 노드 한 대가 죽어도 요청은 성공해야 한다.
    let (mut client, mut nodes) = cluster(SchedulingPolicyKind::RoundRobin, 2).await;

    nodes[0].kill();
    // 서버 종료가 반영될 때까지 짧게 기다린다
    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut succeeded = 0;
    let mut used_dead_node = false;
    for i in 0..6 {
        let r = client.infer(infer_req(i)).await.unwrap().into_inner();
        if r.error_code == "NPF-0000" {
            succeeded += 1;
            if r.node_id == "king" {
                used_dead_node = true;
            }
        }
    }
    assert_eq!(succeeded, 6, "죽은 노드를 우회해 모두 성공해야 한다");
    assert!(!used_dead_node, "죽은 노드가 결과를 반환할 수는 없다");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn timing_breakdown_is_populated_end_to_end() {
    // 노드가 측정한 구간과 스케줄러가 측정한 구간이 모두 채워져야
    // "어디서 시간이 갔는가"를 발표에서 설명할 수 있다.
    let (mut client, _nodes) = cluster(SchedulingPolicyKind::LeastQueue, 1).await;

    let r = client.infer(infer_req(0)).await.unwrap().into_inner();
    let t = r.timing.expect("타이밍이 있어야 한다");

    assert!(t.inference_us > 0, "노드가 측정한 추론시간");
    assert!(t.end_to_end_us > 0, "스케줄러가 측정한 전체 시간");
    assert!(
        t.end_to_end_us >= t.inference_us,
        "전체 시간이 추론시간보다 짧을 수 없다: e2e={} inference={}",
        t.end_to_end_us,
        t.inference_us
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn unknown_model_is_rejected_without_calling_nodes() {
    let (mut client, _nodes) = cluster(SchedulingPolicyKind::LeastQueue, 1).await;

    let mut req = infer_req(0);
    req.model_id = "mobilenet".into();
    let r = client.infer(req).await.unwrap().into_inner();

    assert_eq!(r.error_code, "NPF-1201", "{}", r.error_message);
    assert!(r.node_id.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn all_nodes_down_reports_node_unavailable() {
    let (mut client, mut nodes) = cluster(SchedulingPolicyKind::RoundRobin, 2).await;
    for n in nodes.iter_mut() {
        n.kill();
    }
    tokio::time::sleep(Duration::from_millis(200)).await;

    let r = client.infer(infer_req(0)).await.unwrap().into_inner();
    assert_eq!(
        r.error_code, "NPF-1302",
        "노드 전멸은 노드 장애로 보고해야 한다: {}",
        r.error_message
    );
    // 어느 노드를 시도했는지 메시지에 남아야 원인 추적이 가능하다
    assert!(
        r.error_message.contains("시도한 노드") || r.attempts > 1,
        "실제 메시지: {}",
        r.error_message
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn least_queue_does_not_pile_onto_one_node() {
    // 노드 성능이 다를 때 least-queue 가 한 노드에 쏠리지 않는지 본다.
    //
    // **이 테스트는 예전에 버그 덕분에 통과하고 있었다.** 정책이 보던
    // queue_depth/in_flight 는 하트비트 값이라 부하 중에도 계속 0 이었고,
    // `min_by` 는 값이 모두 같으면 **첫 후보**를 돌려준다. 그래서 12건이
    // 전부 "fast" 로 몰렸고 `fast > 6` 이 통과했다 — 부하 분산이 아니라
    // herding 이었다(S0-C 에서 실장비 처리량 −55%로 드러났다).
    //
    // 로컬 미완료 카운터로 고친 뒤에는 12건 동시 버스트에서 정확히 반씩
    // 나뉜다. 그것이 **count 기반 least-outstanding 의 올바른 동작**이다.
    // 서비스 속도 차이를 반영하는 것은 ECT 의 몫이다(아래 테스트).
    let (sched_addr, _registry) = start_scheduler(SchedulingPolicyKind::LeastQueue, 1).await;
    let nodes = vec![start_node("fast", 2).await, start_node("slow", 60).await];

    let mut client = SchedulerServiceClient::connect(sched_addr).await.unwrap();
    for n in &nodes {
        client
            .register_node(RegisterNodeRequest {
                descriptor: Some(descriptor(n)),
                token: String::new(),
            })
            .await
            .unwrap();
        client
            .heartbeat(v1::HeartbeatRequest {
                node_id: n.node_id.clone(),
                health: Some(v1::NodeHealth::default()),
            })
            .await
            .unwrap();
    }

    // 동시에 던져야 큐 깊이 차이가 생긴다. 순차로 보내면 항상 큐가 0 이라
    // least-queue 와 round-robin 이 구분되지 않는다.
    let mut tasks = Vec::new();
    for i in 0..12 {
        let mut c = client.clone();
        tasks.push(tokio::spawn(async move {
            c.infer(infer_req(i)).await.unwrap().into_inner().node_id
        }));
    }
    let mut fast = 0;
    for t in tasks {
        if t.await.unwrap() == "fast" {
            fast += 1;
        }
    }
    // count 기반 정책은 서비스 속도를 모른다. 보장하는 것은 "한쪽으로
    // 쏠리지 않는다" 까지다. 쏠림(0~2 또는 10~12)이면 herding 회귀다.
    assert!(
        (4..=8).contains(&fast),
        "한 노드로 쏠리면 안 된다 (count 기반은 균등 분배가 정상). fast={fast}/12"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ect_prefers_the_faster_node_after_warmup() {
    // 서비스 속도 차이를 반영하는 것은 ECT 다. 단 EWMA 가 채워져야 한다 —
    // 응답이 한 번도 안 돌아온 상태에서는 두 노드 점수가 같아 구분이 안 된다.
    let (sched_addr, _registry) = start_scheduler(SchedulingPolicyKind::Ect, 1).await;
    let nodes = vec![start_node("fast", 2).await, start_node("slow", 60).await];

    let mut client = SchedulerServiceClient::connect(sched_addr).await.unwrap();
    for n in &nodes {
        client
            .register_node(RegisterNodeRequest {
                descriptor: Some(descriptor(n)),
                token: String::new(),
            })
            .await
            .unwrap();
        client
            .heartbeat(v1::HeartbeatRequest {
                node_id: n.node_id.clone(),
                health: Some(v1::NodeHealth::default()),
            })
            .await
            .unwrap();
    }

    // 예열: 순차로 보내 두 노드 모두 EWMA 를 채운다. 순차라 매 요청마다
    // 미완료가 0 이므로 두 노드가 번갈아 뽑히고, 각자의 실제 지연이 기록된다.
    for i in 0..6 {
        let _ = client.infer(infer_req(1000 + i)).await.unwrap();
    }

    let mut tasks = Vec::new();
    for i in 0..12 {
        let mut c = client.clone();
        tasks.push(tokio::spawn(async move {
            c.infer(infer_req(i)).await.unwrap().into_inner().node_id
        }));
    }
    let mut fast = 0;
    for t in tasks {
        if t.await.unwrap() == "fast" {
            fast += 1;
        }
    }
    assert!(
        fast > 6,
        "EWMA 가 채워진 뒤에는 ECT 가 빠른 노드를 더 쓴다. fast={fast}/12"
    );
}
