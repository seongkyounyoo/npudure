//! 결과 저장과 사람이 읽는 요약.
//!
//! 결과 파일에는 **측정값만이 아니라 측정 조건도 함께** 넣는다. 3개월 뒤
//! 두 파일을 비교할 때 조건이 달랐는지 알 방법이 없으면 그 데이터는
//! 쓸모가 없다.

use serde::{Deserialize, Serialize};

use crate::stats::RunSummary;
use crate::validity::{NodeSnapshot, Verdict};

/// 결과 파일 전체.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// 결과 형식 버전. 나중에 스키마가 바뀌면 이 값으로 구분한다.
    pub format_version: u32,
    pub run_id: String,
    /// ISO-8601. 호출자가 채운다 — 라이브러리는 시계를 읽지 않는다.
    pub started_at: String,
    pub conditions: Conditions,
    pub summary: RunSummary,
    pub verdict: Verdict,
    /// 측정 **전** 스케줄러에 **등록된** 노드 스냅샷(온도·전압·boot_id).
    /// 실제로 요청을 처리한 노드가 아니다 — 그건 `summary.per_node` 다.
    pub nodes_before: Vec<NodeSnapshot>,
    /// 측정 **후** 등록 노드 스냅샷. 온도 상승분으로 실제 부하를 받은
    /// 노드를 사후 확인할 수 있다.
    pub nodes_after: Vec<NodeSnapshot>,
    /// 예열로 버린 요청 수.
    pub warmup_discarded: usize,
    /// 해석에 필요한 주의사항. 결과만 떼어 봐도 알 수 있어야 한다.
    pub caveats: Vec<String>,
}

/// 측정 조건. 이것 없이는 결과를 비교할 수 없다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conditions {
    pub scheduler_address: String,
    pub model_id: String,
    pub concurrency: usize,
    pub duration_s: Option<f64>,
    pub requests: Option<usize>,
    pub payload_bytes: usize,
    pub input_format: String,
    pub payload_seed: u64,
    pub bench_version: String,
    /// 스케줄러 정책. 정책 비교 실험에서 필수다.
    pub scheduler_policy: Option<String>,
    /// **실제로 요청을 처리한 노드 수**(active). 확장 효율 실험에서 필수다.
    /// 등록 노드가 아니라 `summary.per_node.len()` 다 — 노드를 중지해도
    /// 스케줄러 등록은 남으므로 둘이 다를 수 있다.
    pub node_count: usize,
}

/// 모든 run 에 붙는 기본 주의사항.
pub fn default_caveats() -> Vec<String> {
    vec![
        "닫힌 모델(closed loop) 부하다. coordinated omission 이 있어 \
         지연이 낙관적으로 나온다. 절대 지연이 아니라 구성 간 비교에 쓸 것."
            .to_owned(),
        "지연 백분위는 nearest-rank 다. 보간하지 않는다.".to_owned(),
        "예열 요청은 집계에서 제외했다.".to_owned(),
    ]
}

/// 사람이 읽는 요약.
pub fn render(report: &Report) -> String {
    let mut out = String::new();
    let c = &report.conditions;
    let s = &report.summary;

    out.push_str(&format!("run  : {}\n", report.run_id));
    out.push_str(&format!("시각 : {}\n", report.started_at));
    out.push_str(&format!(
        "조건 : 동시성 {} · 노드 {}대 · 모델 {} · {} bytes/요청\n",
        c.concurrency, c.node_count, c.model_id, c.payload_bytes
    ));
    if let Some(p) = &c.scheduler_policy {
        out.push_str(&format!("정책 : {p}\n"));
    }
    out.push('\n');

    // 무효면 가장 먼저, 눈에 띄게. 숫자를 먼저 보여주면 사람은 그것부터 믿는다.
    if !report.verdict.valid {
        out.push_str("!!!!!! 이 run 은 무효다 !!!!!!\n");
        for r in &report.verdict.reasons {
            out.push_str(&format!("  - {r}\n"));
        }
        out.push_str("아래 수치를 인용하지 말 것.\n\n");
    }

    out.push_str(&format!(
        "요청 : {} (성공 {} / 실패 {}, 오류율 {:.3}%)\n",
        s.total_requests,
        s.succeeded,
        s.failed,
        s.error_rate * 100.0
    ));
    out.push_str(&format!(
        "처리량: {:.1} inf/s  ({:.1}초 동안)\n",
        s.throughput, s.duration_s
    ));
    if s.retried > 0 {
        out.push_str(&format!("재시도: {}건\n", s.retried));
    }
    out.push('\n');

    match &s.latency {
        Some(l) => {
            out.push_str("지연 (왕복, ms)\n");
            out.push_str(&format!(
                "  min {:.1}  p50 {:.1}  p90 {:.1}  p95 {:.1}  p99 {:.1}  max {:.1}  평균 {:.1}\n",
                ms(l.min),
                ms(l.p50),
                ms(l.p90),
                ms(l.p95),
                ms(l.p99),
                ms(l.max),
                ms(l.mean)
            ));
        }
        None => out.push_str("지연 : 성공한 요청이 없다\n"),
    }

    if let Some(n) = &s.node_inference {
        out.push_str(&format!(
            "  (노드 보고 추론시간: p50 {:.1} ms  p99 {:.1} ms)\n",
            ms(n.p50),
            ms(n.p99)
        ));
    }
    out.push('\n');

    if !s.per_node.is_empty() {
        out.push_str("노드별 분배\n");
        for n in &s.per_node {
            out.push_str(&format!(
                "  {:<8} {:>6}건 {:>6.1}%   p50 {:>7.1} ms  p99 {:>7.1} ms\n",
                n.node_id,
                n.count,
                n.share * 100.0,
                ms(n.latency.p50),
                ms(n.latency.p99)
            ));
        }
        out.push('\n');
    }

    // TimingBreakdown 단계 분해. 27% 노드당 오버헤드가 어디서 나오는지 본다.
    // bench 와 스케줄러가 같은 호스트면 client→scheduler 는 loopback 이라
    // scheduler_queue 부터가 사실상 첫 단계다.
    if let Some(b) = &s.stage_breakdown {
        out.push_str("단계 분해 (p50 / p99, ms) — TimingBreakdown\n");
        let row = |name: &str, ls: &Option<crate::stats::LatencySummary>| {
            ls.as_ref()
                .map(|l| format!("  {name:<22} {:>7.2} / {:>7.2}\n", ms(l.p50), ms(l.p99)))
        };
        for line in [
            row("scheduler_queue", &b.scheduler_queue),
            row("scheduler_route", &b.scheduler_route),
            row("network_to_node", &b.network_to_node),
            row("node_queue", &b.node_queue),
            row("decode", &b.decode),
            row("preprocess", &b.preprocess),
            row("npu_input", &b.npu_input),
            row("inference (NPU)", &b.inference),
            row("postprocess", &b.postprocess),
            row("network_to_client", &b.network_to_client),
            row("end_to_end", &b.end_to_end),
        ]
        .into_iter()
        .flatten()
        {
            out.push_str(&line);
        }
        out.push('\n');
    }

    if !s.errors_by_code.is_empty() {
        out.push_str("오류\n");
        for (code, count) in &s.errors_by_code {
            out.push_str(&format!("  {code:<20} {count}건\n"));
        }
        out.push('\n');
    }

    // 노드 상태 변화. 온도·전압은 열 실험과 대조할 때 쓴다.
    if !report.nodes_after.is_empty() {
        out.push_str("노드 상태 (run 종료 시점)\n");
        for n in &report.nodes_after {
            let t = n
                .temperature_c
                .map(|v| format!("{v:.1}°C"))
                .unwrap_or_else(|| "온도 없음".into());
            let v = n
                .input_voltage_v
                .map(|v| format!("{v:.3}V"))
                .unwrap_or_else(|| "전압 없음".into());
            out.push_str(&format!("  {:<8} {t:<10} {v}\n", n.node_id));
        }
    }

    out
}

fn ms(micros: u64) -> f64 {
    micros as f64 / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::{Sample, summarize};
    use crate::validity::{ValidityPolicy, judge};

    fn node(id: &str, boot: &str) -> NodeSnapshot {
        NodeSnapshot {
            node_id: id.into(),
            boot_id: boot.into(),
            temperature_c: Some(72.5),
            input_voltage_v: Some(5.07),
        }
    }

    fn samples(n: usize) -> Vec<Sample> {
        (0..n)
            .map(|i| Sample {
                latency_us: 100_000 + (i as u64 % 50) * 1000,
                node_id: ["king", "queen", "jack"][i % 3].into(),
                error_code: "NPF-0000".into(),
                attempts: 1,
                inference_us: 50_000,
                stages: None,
            })
            .collect()
    }

    fn report(nodes_before: Vec<NodeSnapshot>, nodes_after: Vec<NodeSnapshot>, n: usize) -> Report {
        let s = summarize(&samples(n), 10.0);
        let verdict = judge(
            &nodes_before,
            &nodes_after,
            s.error_rate,
            s.succeeded,
            ValidityPolicy::default(),
        );
        Report {
            format_version: 1,
            run_id: "test-run".into(),
            started_at: "2026-08-11T12:00:00Z".into(),
            conditions: Conditions {
                scheduler_address: "http://127.0.0.1:50051".into(),
                model_id: "yolov8n".into(),
                concurrency: 8,
                duration_s: Some(10.0),
                requests: None,
                payload_bytes: 640 * 640 * 3,
                input_format: "rgb8".into(),
                payload_seed: 42,
                bench_version: "0.1.0".into(),
                scheduler_policy: Some("round-robin".into()),
                node_count: nodes_after.len(),
            },
            summary: s,
            verdict,
            nodes_before,
            nodes_after,
            warmup_discarded: 24,
            caveats: default_caveats(),
        }
    }

    #[test]
    fn valid_run_renders_numbers() {
        let n = vec![node("king", "a"), node("queen", "b"), node("jack", "c")];
        let r = report(n.clone(), n, 300);
        let text = render(&r);
        assert!(text.contains("처리량"), "{text}");
        assert!(text.contains("p99"), "{text}");
        assert!(!text.contains("무효"), "{text}");
        // 노드별 분배가 보여야 스케줄링 정책을 평가할 수 있다
        assert!(text.contains("king"), "{text}");
    }

    #[test]
    fn invalid_run_says_so_before_the_numbers() {
        // 숫자를 먼저 보여주면 사람은 그것부터 믿는다.
        let before = vec![node("king", "a")];
        let after = vec![node("king", "REBOOTED")];
        let r = report(before, after, 300);
        let text = render(&r);

        let warn = text.find("무효").expect("무효 표시가 있어야 한다");
        let num = text.find("처리량").expect("처리량이 있어야 한다");
        assert!(warn < num, "경고가 숫자보다 먼저 나와야 한다:\n{text}");
        assert!(text.contains("인용하지 말 것"), "{text}");
    }

    #[test]
    fn too_few_samples_is_flagged() {
        // 100건 미만이면 p99 는 최댓값이다. 그대로 인용하면 안 된다.
        let n = vec![node("king", "a")];
        let r = report(n.clone(), n, 10);
        assert!(!r.verdict.valid);
        assert!(render(&r).contains("무효"));
    }

    #[test]
    fn report_round_trips_through_json() {
        // 결과 파일을 나중에 다시 읽어 비교한다.
        let n = vec![node("king", "a")];
        let r = report(n.clone(), n, 300);
        let json = serde_json::to_string_pretty(&r).unwrap();
        let back: Report = serde_json::from_str(&json).unwrap();
        assert_eq!(back.run_id, r.run_id);
        assert_eq!(back.summary.succeeded, r.summary.succeeded);
        assert_eq!(back.verdict.valid, r.verdict.valid);
        assert_eq!(back.conditions.concurrency, 8);
    }

    #[test]
    fn conditions_are_stored_with_the_numbers() {
        // 조건 없는 측정값은 3개월 뒤에 쓸모가 없다.
        let n = vec![node("king", "a")];
        let json = serde_json::to_string(&report(n.clone(), n, 300)).unwrap();
        for key in [
            "concurrency",
            "model_id",
            "payload_bytes",
            "payload_seed",
            "scheduler_policy",
            "node_count",
            "bench_version",
        ] {
            assert!(json.contains(key), "{key} 가 결과에 없다");
        }
    }

    #[test]
    fn caveats_travel_with_the_report() {
        // 결과만 떼어 봐도 한계를 알 수 있어야 한다.
        let n = vec![node("king", "a")];
        let r = report(n.clone(), n, 300);
        assert!(
            r.caveats.iter().any(|c| c.contains("coordinated omission")),
            "{:?}",
            r.caveats
        );
    }

    #[test]
    fn zero_success_renders_without_panicking() {
        let s = summarize(
            &[Sample {
                latency_us: 1000,
                node_id: String::new(),
                error_code: "NPF-1302".into(),
                attempts: 2,
                inference_us: 0,
                stages: None,
            }],
            1.0,
        );
        let mut r = report(vec![node("king", "a")], vec![node("king", "a")], 300);
        r.summary = s;
        let text = render(&r);
        assert!(text.contains("성공한 요청이 없다"), "{text}");
    }
}
