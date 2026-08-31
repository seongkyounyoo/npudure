//! Mock 추론 백엔드.
//!
//! 실제 NanoPi 3대가 항상 켜져 있어야 개발할 수 있는 구조를 피하기 위한 것이다.
//! 스케줄링 정책, 장애 감지, 자동 복구, 재시도, 대시보드, CI 통합 테스트를
//! 하드웨어 없이 검증한다.
//!
//! `docs/03-DEVELOPMENT-REQUIREMENTS.md` §4.1 참조.
//!
//! # 결정성
//!
//! 테스트에서는 [`MockBackend::with_seed`]를 사용한다. 같은 시드는 같은 지연시간과
//! 같은 실패 시점을 만들어내므로 스케줄러 동작을 재현 가능하게 검증할 수 있다.

use std::sync::{
    Mutex,
    atomic::{AtomicU64, Ordering},
};

use npuforge_common::{
    ErrorCode, NpuForgeError, Result,
    backend::{InferenceBackend, InferenceInput, InferenceOutput, LoadedModel, LoadedModelInfo},
    config::MockBackendConfig,
    model::{ModelSpec, ModelState},
    protocol::TimingBreakdown,
};
use rand::{Rng, SeedableRng, rngs::StdRng};

/// Mock 백엔드가 보고하는 런타임 버전.
pub const MOCK_RUNTIME_VERSION: &str = "mock-1.0.0";

/// 응답하지 않는 요청을 흉내 낼 때 사용하는 대기시간.
///
/// 스케줄러의 `node_rpc_timeout_ms`(기본 3초)보다 충분히 길어야
/// 타임아웃 경로가 실제로 동작하는지 검증할 수 있다.
const HANG_DURATION: std::time::Duration = std::time::Duration::from_secs(600);

pub struct MockBackend {
    config: MockBackendConfig,
    /// 로딩되는 모델의 시드를 파생시키는 RNG.
    ///
    /// 백엔드에 준 시드가 모델까지 전파되어야 결정성 계약이 성립한다.
    seeder: Mutex<StdRng>,
}

impl MockBackend {
    /// 엔트로피 기반 RNG로 생성한다. 실행할 때마다 결과가 달라진다.
    pub fn new(config: MockBackendConfig) -> Self {
        Self {
            config,
            seeder: Mutex::new(StdRng::from_os_rng()),
        }
    }

    /// 고정 시드로 생성한다. 테스트와 재현 가능한 시나리오에 사용한다.
    ///
    /// 이 백엔드가 로딩하는 모델의 시드도 여기서 파생되므로, 같은 시드로 만든
    /// 두 백엔드는 같은 순서의 지연시간과 실패를 만들어낸다.
    pub fn with_seed(config: MockBackendConfig, seed: u64) -> Self {
        Self {
            config,
            seeder: Mutex::new(StdRng::seed_from_u64(seed)),
        }
    }

    fn next_model_seed(&self) -> u64 {
        self.seeder
            .lock()
            .expect("mock seeder lock poisoned")
            .random()
    }
}

struct Draw {
    latency_ms: u64,
    fails: bool,
}

/// 이번 요청의 지연시간과 실패 여부를 한 번에 뽑는다.
///
/// RNG 잠금을 await 지점 너머로 들고 가지 않기 위해 동기 구간에서 모두 결정한다.
fn draw(config: &MockBackendConfig, rng: &Mutex<StdRng>) -> Draw {
    let mut rng = rng.lock().expect("mock RNG lock poisoned");

    let base = config.base_latency_ms;
    let jitter = config.jitter_ms;
    let sampled = if jitter == 0 {
        base
    } else {
        // base ± jitter 균등분포. base가 jitter보다 작을 수 있으므로 하한을 0으로 막는다.
        rng.random_range(base.saturating_sub(jitter)..=base.saturating_add(jitter))
    };

    // speed_factor가 클수록 빠른 노드다. 노드별 성능 편차를 만들어
    // Least Queue와 ECT가 Round Robin보다 나은지 검증하는 데 사용한다.
    let factor = if config.speed_factor > 0.0 {
        config.speed_factor
    } else {
        1.0
    };
    let latency_ms = (sampled as f64 / factor).round() as u64;

    let fails = config.error_rate > 0.0 && rng.random_bool(config.error_rate.clamp(0.0, 1.0));

    Draw { latency_ms, fails }
}

#[async_trait::async_trait]
impl InferenceBackend for MockBackend {
    async fn load_model(&self, spec: &ModelSpec) -> Result<Box<dyn LoadedModel>> {
        Ok(Box::new(MockModel::with_seed(
            spec.clone(),
            self.config.clone(),
            self.next_model_seed(),
        )))
    }

    fn backend_name(&self) -> &'static str {
        "mock"
    }

    fn runtime_version(&self) -> Result<String> {
        Ok(MOCK_RUNTIME_VERSION.to_owned())
    }
}

pub struct MockModel {
    info: LoadedModelInfo,
    config: MockBackendConfig,
    rng: Mutex<StdRng>,
    counter: AtomicU64,
}

impl MockModel {
    /// 테스트에서 백엔드를 거치지 않고 모델을 직접 만들 때 사용한다.
    pub fn with_seed(spec: ModelSpec, config: MockBackendConfig, seed: u64) -> Self {
        Self {
            info: LoadedModelInfo {
                spec,
                state: ModelState::Ready,
                // Mock은 동시 호출에 제약이 없다. RKNN 백엔드는 실장비 검증
                // 결과에 따라 false가 될 수 있다.
                supports_concurrent_infer: true,
            },
            config,
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
            counter: AtomicU64::new(0),
        }
    }
}

#[async_trait::async_trait]
impl LoadedModel for MockModel {
    async fn infer(&self, input: InferenceInput) -> Result<InferenceOutput> {
        let seq = self.counter.fetch_add(1, Ordering::Relaxed) + 1;

        // 응답하지 않는 노드를 흉내 낸다. 스케줄러의 타임아웃과 재시도 경로가
        // 실제로 동작하는지 확인하는 용도다.
        if self.config.hang_every_n > 0 && seq % self.config.hang_every_n == 0 {
            tracing::warn!(seq, "mock backend: 의도적 무응답");
            tokio::time::sleep(HANG_DURATION).await;
        }

        let Draw { latency_ms, fails } = draw(&self.config, &self.rng);

        let started = std::time::Instant::now();
        tokio::time::sleep(std::time::Duration::from_millis(latency_ms)).await;
        let elapsed_us = started.elapsed().as_micros() as u64;

        if fails {
            return Err(NpuForgeError::new(
                ErrorCode::InferenceFailed,
                format!("mock backend: 의도적 실패 (seq={seq})"),
            ));
        }

        // 실제 백엔드와 같은 모양의 단계별 시간을 채운다. 대시보드와 통계 코드가
        // 하드웨어 없이도 현실적인 입력으로 검증되도록 하기 위한 것이다.
        let decode_us = if input.format.needs_decode() {
            elapsed_us / 10
        } else {
            0
        };
        let preprocess_us = elapsed_us / 10;
        let npu_input_us = elapsed_us / 20;
        let postprocess_us = elapsed_us / 20;
        let inference_us =
            elapsed_us.saturating_sub(decode_us + preprocess_us + npu_input_us + postprocess_us);

        Ok(InferenceOutput {
            result: mock_result(&input),
            result_format: self.info.spec.output_format.clone(),
            timing: TimingBreakdown {
                decode_us,
                preprocess_us,
                npu_input_us,
                inference_us,
                postprocess_us,
                ..Default::default()
            },
        })
    }

    fn model_info(&self) -> &LoadedModelInfo {
        &self.info
    }
}

/// 입력 길이에서 유도한 결정적 더미 결과.
///
/// 테스트가 결과를 검증할 수 있도록 입력에 의존하게 만든다.
fn mock_result(input: &InferenceInput) -> bytes::Bytes {
    let payload = format!("{{\"mock\":true,\"input_bytes\":{}}}", input.payload.len());
    bytes::Bytes::from(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use npuforge_common::protocol::InputFormat;

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

    fn input() -> InferenceInput {
        InferenceInput {
            payload: bytes::Bytes::from_static(b"0123456789"),
            format: InputFormat::Jpeg,
        }
    }

    #[tokio::test]
    async fn loads_model_and_infers() {
        let backend = MockBackend::with_seed(
            MockBackendConfig {
                base_latency_ms: 1,
                jitter_ms: 0,
                ..Default::default()
            },
            42,
        );
        assert_eq!(backend.backend_name(), "mock");
        assert_eq!(backend.runtime_version().unwrap(), MOCK_RUNTIME_VERSION);

        let model = backend.load_model(&spec()).await.unwrap();
        assert!(model.model_info().state.accepts_requests());

        let out = model.infer(input()).await.unwrap();
        assert_eq!(out.result_format, "yolo-detections");
        assert!(out.timing.inference_us > 0);
    }

    #[tokio::test]
    async fn jpeg_input_reports_decode_time_raw_does_not() {
        let config = MockBackendConfig {
            base_latency_ms: 20,
            jitter_ms: 0,
            ..Default::default()
        };
        let model = MockModel::with_seed(spec(), config, 1);

        let jpeg = model
            .infer(InferenceInput {
                format: InputFormat::Jpeg,
                ..input()
            })
            .await
            .unwrap();
        assert!(
            jpeg.timing.decode_us > 0,
            "JPEG 입력은 디코딩 시간이 잡혀야 한다"
        );

        let raw = model
            .infer(InferenceInput {
                format: InputFormat::Rgb8,
                ..input()
            })
            .await
            .unwrap();
        assert_eq!(raw.timing.decode_us, 0, "raw 입력은 디코딩 단계가 없다");
    }

    #[tokio::test]
    async fn error_rate_one_always_fails() {
        let config = MockBackendConfig {
            base_latency_ms: 1,
            jitter_ms: 0,
            error_rate: 1.0,
            ..Default::default()
        };
        let model = MockModel::with_seed(spec(), config, 7);
        let err = model.infer(input()).await.unwrap_err();
        assert_eq!(err.code, ErrorCode::InferenceFailed);
        // 추론 실패는 다른 노드에서 재시도할 가치가 있다.
        assert!(err.is_retryable());
    }

    #[tokio::test]
    async fn error_rate_zero_never_fails() {
        let config = MockBackendConfig {
            base_latency_ms: 1,
            jitter_ms: 0,
            error_rate: 0.0,
            ..Default::default()
        };
        let model = MockModel::with_seed(spec(), config, 7);
        for _ in 0..50 {
            assert!(model.infer(input()).await.is_ok());
        }
    }

    #[tokio::test]
    async fn speed_factor_makes_a_faster_node() {
        // 노드별 속도 차이를 만들 수 있어야 Least Queue와 ECT가
        // Round Robin보다 나은지 검증할 수 있다.
        let slow = MockModel::with_seed(
            spec(),
            MockBackendConfig {
                base_latency_ms: 100,
                jitter_ms: 0,
                speed_factor: 1.0,
                ..Default::default()
            },
            1,
        );
        let fast = MockModel::with_seed(
            spec(),
            MockBackendConfig {
                base_latency_ms: 100,
                jitter_ms: 0,
                speed_factor: 4.0,
                ..Default::default()
            },
            1,
        );

        let t0 = std::time::Instant::now();
        slow.infer(input()).await.unwrap();
        let slow_elapsed = t0.elapsed();

        let t1 = std::time::Instant::now();
        fast.infer(input()).await.unwrap();
        let fast_elapsed = t1.elapsed();

        assert!(
            fast_elapsed < slow_elapsed,
            "speed_factor=4.0 노드가 더 빨라야 한다: fast={fast_elapsed:?} slow={slow_elapsed:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn hang_every_n_blocks_past_rpc_timeout() {
        // 스케줄러의 기본 node_rpc_timeout_ms(3초)보다 오래 잡고 있어야
        // 타임아웃 경로를 검증할 수 있다.
        let config = MockBackendConfig {
            base_latency_ms: 1,
            jitter_ms: 0,
            hang_every_n: 2,
            ..Default::default()
        };
        let model = MockModel::with_seed(spec(), config, 3);

        model.infer(input()).await.unwrap(); // seq=1, 정상

        let hung = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            model.infer(input()), // seq=2, 무응답
        )
        .await;
        assert!(hung.is_err(), "두 번째 요청은 응답하지 않아야 한다");
    }

    #[tokio::test]
    async fn same_seed_produces_same_sequence() {
        let config = MockBackendConfig {
            base_latency_ms: 20,
            jitter_ms: 10,
            error_rate: 0.5,
            ..Default::default()
        };
        let a = MockModel::with_seed(spec(), config.clone(), 12345);
        let b = MockModel::with_seed(spec(), config, 12345);

        let mut outcomes_a = Vec::new();
        let mut outcomes_b = Vec::new();
        for _ in 0..20 {
            outcomes_a.push(a.infer(input()).await.is_ok());
            outcomes_b.push(b.infer(input()).await.is_ok());
        }
        assert_eq!(
            outcomes_a, outcomes_b,
            "같은 시드는 같은 실패 시점을 만들어야 한다"
        );
        assert!(
            outcomes_a.iter().any(|ok| !ok),
            "error_rate=0.5 이면 실패가 섞여야 한다"
        );
    }

    #[tokio::test]
    async fn backend_seed_propagates_to_loaded_models() {
        // with_seed 를 쓰고도 모델이 엔트로피 RNG를 쓰면 결정성 계약이 깨진다.
        // 벤치마크 재현성이 이 성질에 의존하므로 테스트로 고정한다.
        let config = MockBackendConfig {
            base_latency_ms: 20,
            jitter_ms: 10,
            error_rate: 0.5,
            ..Default::default()
        };

        let mut runs = Vec::new();
        for _ in 0..2 {
            let backend = MockBackend::with_seed(config.clone(), 999);
            let model = backend.load_model(&spec()).await.unwrap();
            let mut outcomes = Vec::new();
            for _ in 0..20 {
                outcomes.push(model.infer(input()).await.is_ok());
            }
            runs.push(outcomes);
        }
        assert_eq!(
            runs[0], runs[1],
            "같은 시드의 백엔드는 같은 모델 동작을 만들어야 한다"
        );
    }

    #[tokio::test]
    async fn result_depends_on_input() {
        let model = MockModel::with_seed(
            spec(),
            MockBackendConfig {
                base_latency_ms: 1,
                jitter_ms: 0,
                ..Default::default()
            },
            1,
        );
        let out = model
            .infer(InferenceInput {
                payload: bytes::Bytes::from_static(b"abc"),
                format: InputFormat::Rgb8,
            })
            .await
            .unwrap();
        assert_eq!(
            out.result,
            bytes::Bytes::from_static(br#"{"mock":true,"input_bytes":3}"#)
        );
    }
}
