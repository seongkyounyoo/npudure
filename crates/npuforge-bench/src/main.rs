//! NPUDure 벤치마크 CLI.
//!
//! ```bash
//! npuforge-bench \
//!   --scheduler http://192.168.123.100:50051 \
//!   --model yolov8n \
//!   --concurrency 8 \
//!   --duration 60 \
//!   --input-size 640x640x3 \
//!   --out results/s2-3node
//! ```

use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use npuforge_bench::{
    driver::{self, LoadSpec},
    report::{Conditions, Report, default_caveats, render},
    stats::summarize,
    validity::{NodeSnapshot, ValidityPolicy, judge},
};
use npuforge_proto::v1::{
    self, ListNodesRequest, scheduler_service_client::SchedulerServiceClient,
};

const BENCH_VERSION: &str = env!("CARGO_PKG_VERSION");

struct Args {
    scheduler: String,
    model: String,
    concurrency: usize,
    duration: Option<u64>,
    requests: Option<usize>,
    input_size: (u32, u32, u32),
    warmup: usize,
    seed: u64,
    payload_slots: usize,
    out: Option<String>,
    timeout_ms: u64,
    max_error_rate: f64,
    min_samples: usize,
    policy: Option<String>,
}

fn usage() -> &'static str {
    "\
사용법: npuforge-bench [옵션]

필수
  --scheduler <url>       스케줄러 gRPC 주소 (http://host:port)
  --model <id>            모델 ID
  --duration <초> 또는 --requests <n>   둘 중 하나는 필요

선택
  --concurrency <n>       동시 요청 수 (기본 8)
  --input-size WxHxC      입력 크기 (기본 640x640x3)
  --warmup <n>            워커당 예열 요청 수 (기본 5, 집계 제외)
  --seed <n>              입력 생성 시드 (기본 42). 같은 시드는 같은 바이트
  --payload-slots <n>     순환할 입력 개수 (기본 16)
  --timeout-ms <n>        요청 타임아웃 (기본 10000)
  --out <dir>             결과 JSON 저장 디렉터리
  --policy <name>         결과에 기록할 스케줄러 정책 이름
  --max-error-rate <f>    이 값을 넘으면 run 을 무효로 표시 (기본 0.01)
  --min-samples <n>       성공 표본이 이보다 적으면 무효 (기본 100)

주의
  닫힌 모델 부하다. coordinated omission 이 있으므로 절대 지연을 SLA 로
  인용하지 말고 구성 간 비교에 쓴다.
"
}

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        scheduler: String::new(),
        model: String::new(),
        concurrency: 8,
        duration: None,
        requests: None,
        input_size: (640, 640, 3),
        warmup: 5,
        seed: 42,
        payload_slots: 16,
        out: None,
        timeout_ms: 10_000,
        max_error_rate: 0.01,
        min_samples: 100,
        policy: None,
    };

    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut next = |name: &str| -> Result<String, String> {
            it.next()
                .ok_or_else(|| format!("{name} 에 값이 필요합니다"))
        };
        match arg.as_str() {
            "--scheduler" => a.scheduler = next("--scheduler")?,
            "--model" => a.model = next("--model")?,
            "--concurrency" => a.concurrency = parse(&next("--concurrency")?, "--concurrency")?,
            "--duration" => a.duration = Some(parse(&next("--duration")?, "--duration")?),
            "--requests" => a.requests = Some(parse(&next("--requests")?, "--requests")?),
            "--warmup" => a.warmup = parse(&next("--warmup")?, "--warmup")?,
            "--seed" => a.seed = parse(&next("--seed")?, "--seed")?,
            "--payload-slots" => {
                a.payload_slots = parse(&next("--payload-slots")?, "--payload-slots")?
            }
            "--timeout-ms" => a.timeout_ms = parse(&next("--timeout-ms")?, "--timeout-ms")?,
            "--max-error-rate" => {
                a.max_error_rate = parse(&next("--max-error-rate")?, "--max-error-rate")?
            }
            "--min-samples" => a.min_samples = parse(&next("--min-samples")?, "--min-samples")?,
            "--out" => a.out = Some(next("--out")?),
            "--policy" => a.policy = Some(next("--policy")?),
            "--input-size" => a.input_size = parse_size(&next("--input-size")?)?,
            "-h" | "--help" => return Err(String::new()),
            other => return Err(format!("알 수 없는 옵션: {other}")),
        }
    }

    if a.scheduler.is_empty() {
        return Err("--scheduler 가 필요합니다".into());
    }
    if a.model.is_empty() {
        return Err("--model 이 필요합니다".into());
    }
    if a.payload_slots == 0 {
        return Err("--payload-slots 는 1 이상이어야 합니다".into());
    }
    Ok(a)
}

fn parse<T: std::str::FromStr>(s: &str, name: &str) -> Result<T, String> {
    s.parse()
        .map_err(|_| format!("{name} 값을 해석할 수 없습니다: {s}"))
}

fn parse_size(s: &str) -> Result<(u32, u32, u32), String> {
    let parts: Vec<&str> = s.split(['x', 'X']).collect();
    if parts.len() != 3 {
        return Err(format!("--input-size 는 WxHxC 형식입니다: {s}"));
    }
    Ok((
        parse(parts[0], "width")?,
        parse(parts[1], "height")?,
        parse(parts[2], "channels")?,
    ))
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            if !msg.is_empty() {
                eprintln!("오류: {msg}\n");
            }
            eprint!("{}", usage());
            return ExitCode::from(2);
        }
    };

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("tokio 런타임 생성 실패: {e}");
            return ExitCode::FAILURE;
        }
    };

    match rt.block_on(run(args)) {
        Ok(valid) => {
            // 무효 run 은 종료 코드로도 알린다. 야간 자동 실행에서
            // 스크립트가 판단할 수 있어야 한다.
            if valid {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(3)
            }
        }
        Err(e) => {
            eprintln!("실패: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Args) -> Result<bool, String> {
    let (w, h, c) = args.input_size;
    let payload_bytes = (w as usize) * (h as usize) * (c as usize);

    let spec = LoadSpec {
        scheduler_address: args.scheduler.clone(),
        model_id: args.model.clone(),
        concurrency: args.concurrency,
        duration: args.duration.map(Duration::from_secs),
        requests: args.requests,
        warmup_per_worker: args.warmup,
        input_format: v1::InputFormat::Rgb8,
        payloads: Arc::new(driver::synthetic_payloads(
            args.payload_slots,
            payload_bytes,
            args.seed,
        )),
        request_timeout: Duration::from_millis(args.timeout_ms),
    };
    spec.validate()?;

    let mut probe = SchedulerServiceClient::connect(args.scheduler.clone())
        .await
        .map_err(|e| format!("스케줄러에 연결할 수 없습니다: {e}"))?;

    let (before, policy) = snapshot_nodes(&mut probe).await?;
    if before.is_empty() {
        return Err(
            "등록된 노드를 찾지 못했습니다. 노드가 스케줄러에 등록되었는지 확인하세요".into(),
        );
    }
    tracing::info!(
        nodes = before.len(),
        concurrency = args.concurrency,
        "부하 시작"
    );

    let result = driver::run(&spec).await?;
    let (after, _) = snapshot_nodes(&mut probe).await?;

    let summary = summarize(&result.samples, result.duration_s);
    let verdict = judge(
        &before,
        &after,
        summary.error_rate,
        summary.succeeded,
        ValidityPolicy {
            max_error_rate: args.max_error_rate,
            min_samples: args.min_samples,
        },
    );

    // run_id 와 node_count 는 **실제로 요청을 처리한 노드 수**(active)로 붙인다.
    //
    // `before.len()`(측정 시작 시 스케줄러 등록 노드)을 쓰면 틀린다. 노드를
    // 중지해도 스케줄러 등록이 남아 있는 동안은 활성 노드보다 크게 찍히기
    // 때문이다. 실제로 1/2노드로 측정한 run 이 파일명에 `-n3` 으로 붙는
    // 사고가 있었다(`results/scaling-20260820/README.md` §5).
    //
    // 실제 참여 노드는 성공 응답의 `node_id` 로 집계된 `per_node` 다.
    // 등록 노드 스냅샷은 `nodes_before` / `nodes_after` 에 따로 남는다.
    let active_node_count = summary.per_node.len();
    let run_id = format!(
        "{}-c{}-n{}",
        args.model, args.concurrency, active_node_count
    );
    let report = Report {
        format_version: 1,
        run_id: run_id.clone(),
        started_at: unix_time_iso(),
        conditions: Conditions {
            scheduler_address: args.scheduler,
            model_id: args.model,
            concurrency: args.concurrency,
            duration_s: args.duration.map(|d| d as f64),
            requests: args.requests,
            payload_bytes,
            input_format: "rgb8".into(),
            payload_seed: args.seed,
            bench_version: BENCH_VERSION.into(),
            // 스케줄러가 보고한 값을 우선한다. 손으로 적으면 틀리고,
            // 틀린 정책 이름이 붙은 결과는 정책 비교 실험을 망친다.
            scheduler_policy: policy.or(args.policy),
            // 등록 노드가 아니라 **실제 요청을 처리한 노드 수**다.
            node_count: active_node_count,
        },
        summary,
        verdict,
        nodes_before: before,
        nodes_after: after,
        warmup_discarded: result.warmup_discarded,
        caveats: default_caveats(),
    };

    print!("{}", render(&report));

    if let Some(dir) = &args.out {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("결과 디렉터리를 만들 수 없습니다 ({dir}): {e}"))?;
        let path = format!("{dir}/{run_id}.json");
        let json =
            serde_json::to_string_pretty(&report).map_err(|e| format!("결과 직렬화 실패: {e}"))?;
        std::fs::write(&path, json).map_err(|e| format!("결과를 쓸 수 없습니다 ({path}): {e}"))?;
        println!("\n저장: {path}");
    }

    Ok(report.verdict.valid)
}

/// 노드 상태를 훑는다. **읽기 전용이다.**
///
/// 하트비트로 대신하면 스케줄러가 그것을 실제 관측값으로 기록해 노드
/// 상태를 덮어쓴다. 측정 직전에 온도·큐 깊이를 0 으로 만들어 버린다.
async fn snapshot_nodes(
    client: &mut SchedulerServiceClient<tonic::transport::Channel>,
) -> Result<(Vec<NodeSnapshot>, Option<String>), String> {
    let resp = client
        .list_nodes(ListNodesRequest {})
        .await
        .map_err(|e| format!("노드 목록 조회 실패: {e}"))?
        .into_inner();

    let nodes = resp
        .nodes
        .into_iter()
        .map(|n| {
            let id = n
                .descriptor
                .map(|d| d.node_id)
                .unwrap_or_else(|| "unknown".into());
            let h = n.health.unwrap_or_default();
            NodeSnapshot {
                node_id: id,
                boot_id: h.boot_id,
                temperature_c: h.has_temperature.then_some(h.temperature_c),
                input_voltage_v: h.has_input_voltage.then_some(h.input_voltage_v),
            }
        })
        .collect();

    let policy = (!resp.policy.is_empty()).then_some(resp.policy);
    Ok((nodes, policy))
}

fn unix_time_iso() -> String {
    // 외부 크레이트 없이 대략적인 ISO-8601 을 만든다. 정확한 시각이
    // 필요하면 NTP 동기화된 노드의 로그와 대조한다.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}
