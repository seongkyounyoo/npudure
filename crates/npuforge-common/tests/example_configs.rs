//! 저장소에 들어 있는 실제 설정 파일이 코드로 파싱되는지 검증한다.
//!
//! 단위 테스트는 문자열 리터럴을 파싱하므로 `configs/` 아래 파일이 따로
//! 낡아도 잡히지 않는다. 이 테스트가 그 간극을 막는다.

use npuforge_common::config::{BackendConfig, NodeConfig, SchedulerConfig};

fn repo_root() -> std::path::PathBuf {
    // crates/npuforge-common -> 저장소 루트
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("저장소 루트를 찾을 수 없습니다")
}

#[test]
fn scheduler_example_config_loads() {
    let path = repo_root().join("configs/scheduler.example.toml");
    let config = SchedulerConfig::from_path(&path)
        .unwrap_or_else(|e| panic!("{} 로딩 실패: {e}", path.display()));

    assert_eq!(config.scheduler.policy.as_str(), "ect");
    assert!(config.scheduler.node_rpc_timeout_ms < config.scheduler.request_timeout_ms);
}

#[test]
fn node_example_config_loads() {
    let path = repo_root().join("configs/node.example.toml");
    let config = NodeConfig::from_path(&path)
        .unwrap_or_else(|e| panic!("{} 로딩 실패: {e}", path.display()));

    assert!(matches!(config.backend, BackendConfig::Rknn { .. }));
    // 실측으로 확정한 값이다. 4 대비 +27%, 8에서 아직 꺾이지 않았다.
    // `docs/environment-matrix.md` §3.1, `docs/discuss.md` §4.
    //
    // 백엔드가 이 수만큼 RKNN 컨텍스트를 만들어 하나씩 점유한다.
    // 컨텍스트를 공유하면 API 오류 없이 100% 틀린 결과가 나오므로,
    // 이 값이 바뀌면 컨텍스트 수도 함께 바뀌어야 한다.
    assert_eq!(config.worker.worker_count, 8);
}

#[test]
fn mock_cluster_configs_load_and_are_distinct() {
    let root = repo_root();
    let mut configs = Vec::new();
    for name in ["node-01", "node-02", "node-03"] {
        let path = root.join(format!("configs/mock/{name}.toml"));
        let config = NodeConfig::from_path(&path)
            .unwrap_or_else(|e| panic!("{} 로딩 실패: {e}", path.display()));
        assert!(config.is_mock(), "{name} 은 mock 백엔드여야 한다");
        configs.push(config);
    }

    // 로컬 3노드가 같은 포트를 쓰면 클러스터가 뜨지 않는다.
    let listens: std::collections::HashSet<_> =
        configs.iter().map(|c| c.node.listen.clone()).collect();
    assert_eq!(listens.len(), 3, "세 노드의 listen 주소가 겹칩니다");

    let ids: std::collections::HashSet<_> = configs.iter().map(|c| c.node.id.clone()).collect();
    assert_eq!(ids.len(), 3, "세 노드의 id가 겹칩니다");

    // 정책 비교가 의미를 가지려면 노드 성능이 달라야 한다.
    let speeds: Vec<f64> = configs
        .iter()
        .map(|c| match &c.backend {
            BackendConfig::Mock(m) => m.speed_factor,
            _ => unreachable!(),
        })
        .collect();
    assert!(
        speeds.iter().any(|s| (*s - speeds[0]).abs() > f64::EPSILON),
        "노드 속도가 모두 같으면 Round Robin과 ECT의 차이가 드러나지 않는다"
    );
}
