//! 지연 분포와 처리량 집계.
//!
//! 하드웨어 없이 검증할 수 있는 부분이다. 통계가 틀리면 모든 결론이
//! 틀리므로 여기부터 테스트로 고정한다.
//!
//! # 백분위 정의
//!
//! **nearest-rank 방식을 쓴다.** `p95` 는 "정렬했을 때 95% 지점 이상에
//! 있는 첫 표본"이다. 보간하지 않는다.
//!
//! 보간 방식(선형 보간 등)은 표본이 적을 때 실제로 관측되지 않은 값을
//! 만들어낸다. 벤치마크 결과에 "실제로는 아무도 겪지 않은 지연"이 적히면
//! 곤란하다. 정의를 문서에 박아 두고 바꾸지 않는다.

use serde::{Deserialize, Serialize};

/// 요청 하나의 결과.
#[derive(Debug, Clone)]
pub struct Sample {
    /// 클라이언트가 잰 왕복 시간(µs).
    pub latency_us: u64,
    /// 처리한 노드. 실패면 비어 있다.
    pub node_id: String,
    /// `NPF-0000` 이면 성공.
    pub error_code: String,
    /// 스케줄러가 시도한 횟수.
    pub attempts: u32,
    /// 노드가 보고한 추론 시간(µs). 없으면 0.
    pub inference_us: u64,
    /// 응답의 `TimingBreakdown` 전체 단계. 없으면 `None`(전송 실패 등).
    /// 27% 오버헤드가 어느 단계에서 나오는지 분해하는 데 쓴다.
    pub stages: Option<StageTimings>,
}

/// 응답의 `Timing`(proto) 11단계를 그대로 담는다. 전부 µs.
///
/// 단계 값 0 은 유효하다 — bench 와 스케줄러가 같은 호스트면
/// `network_to_client` 같은 loopback 구간이 실제로 0 에 가깝다. 그래서
/// 집계에서 0 을 버리지 않는다(미보고는 `stages = None` 으로 구분).
#[derive(Debug, Clone, Copy)]
pub struct StageTimings {
    pub scheduler_queue_us: u64,
    pub scheduler_route_us: u64,
    pub network_to_node_us: u64,
    pub node_queue_us: u64,
    pub decode_us: u64,
    pub preprocess_us: u64,
    pub npu_input_us: u64,
    pub inference_us: u64,
    pub postprocess_us: u64,
    pub network_to_client_us: u64,
    pub end_to_end_us: u64,
}

impl Sample {
    pub fn is_success(&self) -> bool {
        self.error_code == "NPF-0000"
    }
}

/// 지연 분포 요약. 모든 값은 µs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatencySummary {
    pub count: usize,
    pub min: u64,
    pub p50: u64,
    pub p90: u64,
    pub p95: u64,
    pub p99: u64,
    pub max: u64,
    pub mean: u64,
}

impl LatencySummary {
    /// 표본이 없으면 `None`. 0 으로 채운 요약을 돌려주면 "지연 0"으로
    /// 오독되고, 실패한 run 이 성공한 것처럼 집계된다.
    pub fn from_micros(mut values: Vec<u64>) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        values.sort_unstable();
        let n = values.len();
        let sum: u128 = values.iter().map(|&v| u128::from(v)).sum();

        Some(Self {
            count: n,
            min: values[0],
            p50: percentile(&values, 50.0),
            p90: percentile(&values, 90.0),
            p95: percentile(&values, 95.0),
            p99: percentile(&values, 99.0),
            max: values[n - 1],
            mean: (sum / n as u128) as u64,
        })
    }
}

/// nearest-rank 백분위. `sorted` 는 오름차순이고 비어 있지 않아야 한다.
///
/// 순위는 `ceil(p/100 × n)` 이며 1-기반이다. 즉 `p100` 은 최댓값,
/// `p0` 이하는 최솟값이다.
pub fn percentile(sorted: &[u64], p: f64) -> u64 {
    debug_assert!(!sorted.is_empty());
    debug_assert!(
        sorted.windows(2).all(|w| w[0] <= w[1]),
        "정렬되어 있어야 한다"
    );

    let n = sorted.len();
    if p <= 0.0 {
        return sorted[0];
    }
    if p >= 100.0 {
        return sorted[n - 1];
    }
    let rank = (p / 100.0 * n as f64).ceil() as usize;
    sorted[rank.saturating_sub(1).min(n - 1)]
}

/// 노드 한 대의 몫.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeShare {
    pub node_id: String,
    pub count: usize,
    /// 성공 요청 중 비율. 0.0 ~ 1.0.
    pub share: f64,
    pub latency: LatencySummary,
}

/// run 하나의 집계 결과.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub total_requests: usize,
    pub succeeded: usize,
    pub failed: usize,
    /// 0.0 ~ 1.0.
    pub error_rate: f64,
    /// 실제 부하 구간 길이(초).
    pub duration_s: f64,
    /// 성공 요청 기준 처리량.
    pub throughput: f64,
    /// 성공 요청의 왕복 지연.
    pub latency: Option<LatencySummary>,
    /// 노드가 보고한 추론 시간. 왕복 지연에서 네트워크·큐를 뺀 값에 해당한다.
    pub node_inference: Option<LatencySummary>,
    /// 노드별 분배.
    pub per_node: Vec<NodeShare>,
    /// 오류 코드별 건수. 무엇이 실패했는지 알아야 한다.
    pub errors_by_code: Vec<(String, usize)>,
    /// 스케줄러가 재시도한 요청 수.
    pub retried: usize,
    /// `TimingBreakdown` 단계별 지연 분포. 응답에 timing 이 하나도 없으면
    /// `None`. 27% 노드당 오버헤드를 단계로 쪼개는 데 쓴다.
    pub stage_breakdown: Option<StageBreakdown>,
}

/// `TimingBreakdown` 11단계 각각의 지연 분포. 각 값은 성공 표본 기준.
///
/// 필드명은 proto `Timing` 과 1:1 이다. 한 단계라도 표본이 있으면 그
/// 단계는 `Some`, 없으면 `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageBreakdown {
    pub scheduler_queue: Option<LatencySummary>,
    pub scheduler_route: Option<LatencySummary>,
    pub network_to_node: Option<LatencySummary>,
    pub node_queue: Option<LatencySummary>,
    pub decode: Option<LatencySummary>,
    pub preprocess: Option<LatencySummary>,
    pub npu_input: Option<LatencySummary>,
    pub inference: Option<LatencySummary>,
    pub postprocess: Option<LatencySummary>,
    pub network_to_client: Option<LatencySummary>,
    pub end_to_end: Option<LatencySummary>,
}

/// 표본들을 집계한다.
pub fn summarize(samples: &[Sample], duration_s: f64) -> RunSummary {
    let total = samples.len();
    let ok: Vec<&Sample> = samples.iter().filter(|s| s.is_success()).collect();
    let succeeded = ok.len();
    let failed = total - succeeded;

    let latency = LatencySummary::from_micros(ok.iter().map(|s| s.latency_us).collect());
    let node_inference = LatencySummary::from_micros(
        ok.iter()
            .map(|s| s.inference_us)
            .filter(|&v| v > 0)
            .collect(),
    );

    // 노드별. 이름순으로 정렬해 run 간 비교가 쉽도록 한다.
    let mut node_ids: Vec<&str> = ok.iter().map(|s| s.node_id.as_str()).collect();
    node_ids.sort_unstable();
    node_ids.dedup();

    let per_node = node_ids
        .into_iter()
        .filter_map(|id| {
            let mine: Vec<u64> = ok
                .iter()
                .filter(|s| s.node_id == id)
                .map(|s| s.latency_us)
                .collect();
            let count = mine.len();
            LatencySummary::from_micros(mine).map(|latency| NodeShare {
                node_id: id.to_owned(),
                count,
                share: if succeeded == 0 {
                    0.0
                } else {
                    count as f64 / succeeded as f64
                },
                latency,
            })
        })
        .collect();

    let mut codes: Vec<(String, usize)> = Vec::new();
    for s in samples.iter().filter(|s| !s.is_success()) {
        match codes.iter_mut().find(|(c, _)| c == &s.error_code) {
            Some((_, n)) => *n += 1,
            None => codes.push((s.error_code.clone(), 1)),
        }
    }
    // 건수 내림차순. 같으면 코드순.
    codes.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    // 단계별 분해. timing 을 보고한 성공 표본만 쓴다. 0µs 는 유효값이므로
    // 버리지 않는다(미보고는 애초에 stages=None 이라 여기서 빠진다).
    let stage_breakdown = if ok.iter().any(|s| s.stages.is_some()) {
        let stage = |f: fn(&StageTimings) -> u64| {
            LatencySummary::from_micros(
                ok.iter().filter_map(|s| s.stages.as_ref()).map(f).collect(),
            )
        };
        Some(StageBreakdown {
            scheduler_queue: stage(|t| t.scheduler_queue_us),
            scheduler_route: stage(|t| t.scheduler_route_us),
            network_to_node: stage(|t| t.network_to_node_us),
            node_queue: stage(|t| t.node_queue_us),
            decode: stage(|t| t.decode_us),
            preprocess: stage(|t| t.preprocess_us),
            npu_input: stage(|t| t.npu_input_us),
            inference: stage(|t| t.inference_us),
            postprocess: stage(|t| t.postprocess_us),
            network_to_client: stage(|t| t.network_to_client_us),
            end_to_end: stage(|t| t.end_to_end_us),
        })
    } else {
        None
    };

    RunSummary {
        total_requests: total,
        succeeded,
        failed,
        error_rate: if total == 0 {
            0.0
        } else {
            failed as f64 / total as f64
        },
        duration_s,
        throughput: if duration_s > 0.0 {
            succeeded as f64 / duration_s
        } else {
            0.0
        },
        latency,
        node_inference,
        per_node,
        errors_by_code: codes,
        retried: samples.iter().filter(|s| s.attempts > 1).count(),
        stage_breakdown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(latency_us: u64, node: &str, code: &str) -> Sample {
        Sample {
            latency_us,
            node_id: node.into(),
            error_code: code.into(),
            attempts: 1,
            inference_us: latency_us / 2,
            stages: None,
        }
    }

    fn stage(inference_us: u64, net_to_client_us: u64) -> StageTimings {
        StageTimings {
            scheduler_queue_us: 1,
            scheduler_route_us: 2,
            network_to_node_us: 3,
            node_queue_us: 4,
            decode_us: 5,
            preprocess_us: 6,
            npu_input_us: 7,
            inference_us,
            postprocess_us: 9,
            network_to_client_us: net_to_client_us,
            end_to_end_us: 100,
        }
    }

    #[test]
    fn stage_breakdown_aggregates_only_samples_with_timing() {
        let mut a = sample(200, "king", "NPF-0000");
        a.stages = Some(stage(40, 0)); // 0µs 구간(loopback)도 유효
        let mut b = sample(300, "queen", "NPF-0000");
        b.stages = Some(stage(60, 0));
        // timing 이 없는 성공 표본은 단계 분해에서 빠진다.
        let c = sample(250, "jack", "NPF-0000");

        let s = summarize(&[a, b, c], 1.0);
        let sb = s.stage_breakdown.expect("timing 표본이 있으면 Some");
        let inf = sb.inference.expect("inference 단계");
        // a,b 만 stages 를 가지므로 표본 2개, 0µs 는 버리지 않는다.
        assert_eq!(inf.count, 2);
        assert_eq!(inf.min, 40);
        assert_eq!(inf.max, 60);
        // network_to_client 가 전부 0 이어도 단계는 살아 있어야 한다
        // (0 을 버리면 loopback 구간이 통째로 사라진다).
        let n2c = sb.network_to_client.expect("network_to_client 단계");
        assert_eq!(n2c.count, 2);
        assert_eq!(n2c.max, 0);
    }

    #[test]
    fn stage_breakdown_is_none_without_any_timing() {
        let s = summarize(&[sample(100, "king", "NPF-0000")], 1.0);
        assert!(s.stage_breakdown.is_none());
    }

    #[test]
    fn percentile_uses_nearest_rank_not_interpolation() {
        // 1..=10 에서 p95 는 10 이다. 보간하면 9.55 같은 값이 나오는데,
        // 그런 지연은 아무도 겪지 않았다.
        let v: Vec<u64> = (1..=10).collect();
        assert_eq!(percentile(&v, 50.0), 5);
        assert_eq!(percentile(&v, 90.0), 9);
        assert_eq!(percentile(&v, 95.0), 10);
        assert_eq!(percentile(&v, 99.0), 10);
        assert_eq!(percentile(&v, 100.0), 10);
    }

    #[test]
    fn percentile_of_single_sample_is_that_sample() {
        let v = vec![42];
        for p in [0.0, 50.0, 95.0, 99.0, 100.0] {
            assert_eq!(percentile(&v, p), 42, "p{p}");
        }
    }

    #[test]
    fn percentile_clamps_out_of_range() {
        let v = vec![1, 2, 3];
        assert_eq!(percentile(&v, -5.0), 1);
        assert_eq!(percentile(&v, 150.0), 3);
    }

    #[test]
    fn p99_of_100_samples_is_the_99th() {
        // 100개 중 하나만 튀면 p99 가 그것을 잡아야 한다.
        let mut v: Vec<u64> = vec![10; 99];
        v.push(9999);
        v.sort_unstable();
        assert_eq!(percentile(&v, 99.0), 10);
        assert_eq!(percentile(&v, 100.0), 9999);
        // 두 개가 튀면 p99 가 반응한다
        let mut v2: Vec<u64> = vec![10; 98];
        v2.extend([9999, 9999]);
        v2.sort_unstable();
        assert_eq!(percentile(&v2, 99.0), 9999);
    }

    #[test]
    fn empty_latency_is_none_not_zero() {
        // 0 으로 채운 요약을 돌려주면 "지연 0"으로 오독된다.
        assert!(LatencySummary::from_micros(Vec::new()).is_none());
    }

    #[test]
    fn all_failed_run_reports_no_latency() {
        let s = summarize(
            &[sample(100, "", "NPF-1302"), sample(200, "", "NPF-1302")],
            1.0,
        );
        assert_eq!(s.succeeded, 0);
        assert_eq!(s.failed, 2);
        assert_eq!(s.error_rate, 1.0);
        assert_eq!(s.throughput, 0.0);
        assert!(s.latency.is_none(), "성공이 없으면 지연 통계도 없다");
        assert!(s.per_node.is_empty());
    }

    #[test]
    fn per_node_shares_sum_to_one() {
        let samples = vec![
            sample(100, "king", "NPF-0000"),
            sample(120, "king", "NPF-0000"),
            sample(110, "queen", "NPF-0000"),
            sample(130, "jack", "NPF-0000"),
        ];
        let s = summarize(&samples, 1.0);
        assert_eq!(s.per_node.len(), 3);
        let total: f64 = s.per_node.iter().map(|n| n.share).sum();
        assert!((total - 1.0).abs() < 1e-9, "합이 1 이어야 한다: {total}");
        // 이름순 정렬로 run 간 비교가 쉬워야 한다
        let names: Vec<&str> = s.per_node.iter().map(|n| n.node_id.as_str()).collect();
        assert_eq!(names, ["jack", "king", "queen"]);
    }

    #[test]
    fn failed_requests_do_not_count_toward_node_share() {
        // 실패를 노드 몫에 넣으면 죽은 노드가 "많이 처리한" 것처럼 보인다.
        let samples = vec![
            sample(100, "king", "NPF-0000"),
            sample(50, "", "NPF-1302"),
            sample(50, "", "NPF-1302"),
        ];
        let s = summarize(&samples, 1.0);
        assert_eq!(s.per_node.len(), 1);
        assert_eq!(s.per_node[0].node_id, "king");
        assert_eq!(s.per_node[0].share, 1.0);
    }

    #[test]
    fn throughput_counts_only_successes() {
        // 실패도 처리량에 넣으면 노드가 다 죽었을 때 처리량이 가장 높아진다.
        let samples = vec![sample(100, "king", "NPF-0000"), sample(100, "", "NPF-1302")];
        let s = summarize(&samples, 2.0);
        assert_eq!(s.throughput, 0.5);
    }

    #[test]
    fn errors_are_grouped_and_sorted_by_count() {
        let samples = vec![
            sample(1, "", "NPF-1302"),
            sample(1, "", "NPF-1201"),
            sample(1, "", "NPF-1302"),
            sample(1, "", "NPF-1302"),
        ];
        let s = summarize(&samples, 1.0);
        assert_eq!(
            s.errors_by_code,
            vec![("NPF-1302".to_owned(), 3), ("NPF-1201".to_owned(), 1)]
        );
    }

    #[test]
    fn retries_are_counted() {
        let mut a = sample(100, "king", "NPF-0000");
        a.attempts = 2;
        let b = sample(100, "queen", "NPF-0000");
        let s = summarize(&[a, b], 1.0);
        assert_eq!(s.retried, 1);
    }

    #[test]
    fn zero_duration_does_not_divide_by_zero() {
        let s = summarize(&[sample(100, "king", "NPF-0000")], 0.0);
        assert_eq!(s.throughput, 0.0);
        assert!(s.throughput.is_finite());
    }

    #[test]
    fn empty_run_is_summarized_without_panicking() {
        let s = summarize(&[], 1.0);
        assert_eq!(s.total_requests, 0);
        assert_eq!(s.error_rate, 0.0);
        assert!(s.latency.is_none());
    }

    #[test]
    fn node_inference_ignores_missing_values() {
        // 노드가 타이밍을 안 채우면 0 이 온다. 0 을 통계에 넣으면
        // p50 이 0 으로 내려가 "추론이 즉시 끝났다"로 읽힌다.
        let mut a = sample(100, "king", "NPF-0000");
        a.inference_us = 0;
        let b = sample(200, "queen", "NPF-0000");
        let s = summarize(&[a, b], 1.0);
        let ni = s.node_inference.expect("하나는 값이 있다");
        assert_eq!(ni.count, 1);
        assert_eq!(ni.min, 100);
    }
}
