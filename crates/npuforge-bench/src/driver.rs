//! 부하 발생.
//!
//! # 열린 모델(open loop)이 아니라 닫힌 모델(closed loop)이다
//!
//! 동시성 N 을 고정하고, 각 워커가 응답을 받은 뒤 다음 요청을 보낸다.
//! 목표 RPS 를 정해 놓고 응답과 무관하게 쏘는 방식(open loop)이 아니다.
//!
//! 이 선택은 측정 대상에 맞춘 것이다. 우리가 알고 싶은 것은 "N개의
//! 동시 클라이언트가 있을 때 시스템이 얼마나 처리하는가"이지 "초당 X건을
//! 넣으면 큐가 어떻게 쌓이는가"가 아니다. 후자는 노드 큐 길이가 유한해서
//! 금방 거절로 끝난다.
//!
//! 다만 닫힌 모델은 **coordinated omission** 에 취약하다. 시스템이 느려지면
//! 클라이언트도 덩달아 천천히 보내므로 지연 분포가 낙관적으로 나온다.
//! 이 한계를 결과에 명시한다. 절대 지연이 아니라 **구성 간 비교**에
//! 쓰는 것이 이 도구의 용도다.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use npuforge_proto::v1::{self, InferRequest, scheduler_service_client::SchedulerServiceClient};
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::stats::{Sample, StageTimings};

/// 부하 조건.
#[derive(Debug, Clone)]
pub struct LoadSpec {
    pub scheduler_address: String,
    pub model_id: String,
    pub concurrency: usize,
    /// 부하 지속 시간. `requests` 와 함께 주면 먼저 도달하는 쪽에서 멈춘다.
    pub duration: Option<Duration>,
    /// 총 요청 수.
    pub requests: Option<usize>,
    /// 벤치마크에서 제외할 예열 요청 수(워커당).
    pub warmup_per_worker: usize,
    pub input_format: v1::InputFormat,
    /// 요청 페이로드. 순환해 사용한다.
    pub payloads: Arc<Vec<bytes::Bytes>>,
    pub request_timeout: Duration,
}

impl LoadSpec {
    /// 종료 조건이 하나라도 있어야 한다. 없으면 영원히 돈다.
    pub fn validate(&self) -> Result<(), String> {
        if self.duration.is_none() && self.requests.is_none() {
            return Err("--duration 또는 --requests 중 하나는 필요합니다".into());
        }
        if self.concurrency == 0 {
            return Err("--concurrency 는 1 이상이어야 합니다".into());
        }
        if self.payloads.is_empty() {
            return Err("입력 페이로드가 비어 있습니다".into());
        }
        Ok(())
    }
}

/// 부하 실행 결과.
pub struct LoadResult {
    pub samples: Vec<Sample>,
    /// 예열을 제외한 실제 부하 구간(초).
    pub duration_s: f64,
    /// 예열로 버린 요청 수.
    pub warmup_discarded: usize,
}

/// 부하를 발생시킨다.
pub async fn run(spec: &LoadSpec) -> Result<LoadResult, String> {
    spec.validate()?;

    let endpoint = Channel::from_shared(spec.scheduler_address.clone())
        .map_err(|e| format!("스케줄러 주소 '{}' 형식 오류: {e}", spec.scheduler_address))?
        .connect_timeout(Duration::from_secs(5))
        .timeout(spec.request_timeout);

    // 워커마다 채널을 따로 연다. 하나를 공유하면 HTTP/2 스트림 제한과
    // 단일 커넥션의 head-of-line blocking 이 클라이언트 쪽 병목이 되어,
    // 측정 대상이 아닌 것을 측정하게 된다.
    let mut clients = Vec::with_capacity(spec.concurrency);
    for i in 0..spec.concurrency {
        let ch = endpoint
            .clone()
            .connect()
            .await
            .map_err(|e| format!("워커 {i} 연결 실패: {e}"))?;
        clients.push(SchedulerServiceClient::new(ch));
    }

    let stop = Arc::new(AtomicBool::new(false));
    let issued = Arc::new(AtomicU64::new(0));
    let collected: Arc<Mutex<Vec<Sample>>> = Arc::new(Mutex::new(Vec::new()));

    // --- 예열 ---
    // 첫 추론은 지연이 크게 튄다. 벤치마크에서 제외한다.
    // `docs/01-TECHSPEC.md` §13.4.
    let mut warmup_discarded = 0usize;
    if spec.warmup_per_worker > 0 {
        let mut tasks = Vec::new();
        for (w, client) in clients.iter().enumerate() {
            let mut c = client.clone();
            let s = spec.clone();
            tasks.push(tokio::spawn(async move {
                let mut n = 0;
                for i in 0..s.warmup_per_worker {
                    let payload = s.payloads[(w + i) % s.payloads.len()].clone();
                    let _ = c
                        .infer(make_request(&s, &payload, format!("warmup-{w}-{i}")))
                        .await;
                    n += 1;
                }
                n
            }));
        }
        for t in tasks {
            warmup_discarded += t.await.unwrap_or(0);
        }
    }

    // --- 본 측정 ---
    let started = Instant::now();
    let deadline = spec.duration.map(|d| started + d);

    let mut tasks = Vec::new();
    for (w, client) in clients.into_iter().enumerate() {
        let mut c = client;
        let s = spec.clone();
        let stop = Arc::clone(&stop);
        let issued = Arc::clone(&issued);
        let collected = Arc::clone(&collected);

        tasks.push(tokio::spawn(async move {
            let mut local: Vec<Sample> = Vec::new();
            let mut i = 0usize;

            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                if let Some(d) = deadline
                    && Instant::now() >= d
                {
                    break;
                }
                if let Some(limit) = s.requests {
                    // fetch_add 로 선점한다. 초과분은 되돌리지 않는다 —
                    // 되돌리면 다른 워커가 그 자리를 다시 가져가 총량이
                    // limit 을 넘을 수 있다.
                    if issued.fetch_add(1, Ordering::Relaxed) >= limit as u64 {
                        break;
                    }
                }

                let payload = s.payloads[(w + i) % s.payloads.len()].clone();
                let req = make_request(&s, &payload, format!("bench-{w}-{i}"));

                let t0 = Instant::now();
                let outcome = c.infer(req).await;
                let latency_us = t0.elapsed().as_micros() as u64;

                let sample = match outcome {
                    Ok(resp) => {
                        let r = resp.into_inner();
                        // TimingBreakdown 전체를 담는다. inference_us 는 여기서
                        // 유도한다(StageTimings 는 Copy).
                        let stages = r.timing.map(|t| StageTimings {
                            scheduler_queue_us: t.scheduler_queue_us,
                            scheduler_route_us: t.scheduler_route_us,
                            network_to_node_us: t.network_to_node_us,
                            node_queue_us: t.node_queue_us,
                            decode_us: t.decode_us,
                            preprocess_us: t.preprocess_us,
                            npu_input_us: t.npu_input_us,
                            inference_us: t.inference_us,
                            postprocess_us: t.postprocess_us,
                            network_to_client_us: t.network_to_client_us,
                            end_to_end_us: t.end_to_end_us,
                        });
                        Sample {
                            latency_us,
                            node_id: r.node_id,
                            error_code: r.error_code,
                            attempts: r.attempts,
                            inference_us: stages.map(|s| s.inference_us).unwrap_or(0),
                            stages,
                        }
                    }
                    Err(status) => Sample {
                        latency_us,
                        node_id: String::new(),
                        // 전송 실패는 스케줄러에 닿지도 못한 것이다.
                        // 노드 장애와 구분해야 원인 추적이 된다.
                        error_code: format!("TRANSPORT-{:?}", status.code()),
                        attempts: 0,
                        inference_us: 0,
                        stages: None,
                    },
                };
                local.push(sample);
                i += 1;
            }

            collected.lock().await.extend(local);
        }));
    }

    for t in tasks {
        let _ = t.await;
    }
    let duration_s = started.elapsed().as_secs_f64();

    let samples = Arc::try_unwrap(collected)
        .map_err(|_| "표본 수집기 참조가 남아 있습니다".to_owned())?
        .into_inner();

    Ok(LoadResult {
        samples,
        duration_s,
        warmup_discarded,
    })
}

fn make_request(spec: &LoadSpec, payload: &bytes::Bytes, id: String) -> InferRequest {
    InferRequest {
        request_id: id,
        model_id: spec.model_id.clone(),
        payload: payload.to_vec(),
        input_format: spec.input_format as i32,
        priority: 0,
        deadline_unix_ms: 0,
        metadata: Default::default(),
    }
}

/// 결정적 입력을 만든다.
///
/// 무작위 데이터를 쓰면 run 마다 내용이 달라져, 압축이나 캐시가 개입할 때
/// 비교가 흔들린다. 시드로 고정한다.
pub fn synthetic_payloads(count: usize, bytes_each: usize, seed: u64) -> Vec<bytes::Bytes> {
    (0..count)
        .map(|i| {
            let base = seed.wrapping_add(i as u64).wrapping_mul(2_654_435_761);
            let data: Vec<u8> = (0..bytes_each)
                .map(|k| (base.wrapping_add(k as u64) % 251) as u8)
                .collect();
            bytes::Bytes::from(data)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> LoadSpec {
        LoadSpec {
            scheduler_address: "http://127.0.0.1:50051".into(),
            model_id: "yolov8n".into(),
            concurrency: 4,
            duration: Some(Duration::from_secs(10)),
            requests: None,
            warmup_per_worker: 3,
            input_format: v1::InputFormat::Rgb8,
            payloads: Arc::new(synthetic_payloads(4, 128, 7)),
            request_timeout: Duration::from_secs(5),
        }
    }

    #[test]
    fn spec_without_stop_condition_is_rejected() {
        // 종료 조건이 없으면 영원히 돈다. 야간 무인 실행에서 치명적이다.
        let mut s = spec();
        s.duration = None;
        s.requests = None;
        assert!(s.validate().unwrap_err().contains("--duration"));
    }

    #[test]
    fn zero_concurrency_is_rejected() {
        let mut s = spec();
        s.concurrency = 0;
        assert!(s.validate().is_err());
    }

    #[test]
    fn empty_payloads_are_rejected() {
        let mut s = spec();
        s.payloads = Arc::new(Vec::new());
        assert!(s.validate().is_err());
    }

    #[test]
    fn either_stop_condition_is_enough() {
        let mut s = spec();
        s.duration = None;
        s.requests = Some(100);
        assert!(s.validate().is_ok());
    }

    #[test]
    fn synthetic_payloads_are_deterministic() {
        // 같은 시드는 같은 바이트를 내야 run 간 비교가 성립한다.
        let a = synthetic_payloads(3, 64, 42);
        let b = synthetic_payloads(3, 64, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn synthetic_payloads_differ_between_slots() {
        // 전부 같은 내용이면 캐시 효과가 개입할 수 있다.
        let p = synthetic_payloads(3, 64, 42);
        assert_ne!(p[0], p[1]);
        assert_ne!(p[1], p[2]);
    }

    #[test]
    fn synthetic_payloads_have_exact_size() {
        // 모델 입력 크기와 정확히 맞아야 한다. 다르면 노드가 거부한다.
        let p = synthetic_payloads(2, 640 * 640 * 3, 1);
        assert!(p.iter().all(|b| b.len() == 640 * 640 * 3));
    }

    #[test]
    fn different_seeds_give_different_data() {
        assert_ne!(synthetic_payloads(1, 64, 1), synthetic_payloads(1, 64, 2));
    }
}
