//! 실장비 통합 테스트. RKNN Runtime 과 실제 NPU 가 있어야 한다.
//!
//! 개발 PC 에서는 `rknn` feature 가 꺼져 있어 통째로 컴파일되지 않는다.
//! 보드에서 실행한다.
//!
//! ```bash
//! RKNN_SDK_PATH=/usr/include NPUFORGE_TEST_MODEL=/path/to/yolov8n-fp16.rknn \
//!   cargo test --release -p npuforge-rknn --features rknn --test real_device
//! ```
//!
//! `NPUFORGE_TEST_MODEL` 이 없으면 조용히 건너뛴다. CI 에서 실패하지
//! 않으면서 보드에서는 실제로 돌게 하려는 것이다.

#![cfg(feature = "rknn")]

use npuforge_common::{
    backend::{InferenceBackend, InferenceInput, LoadedModel},
    model::ModelSpec,
    protocol::InputFormat,
};
use npuforge_rknn::{RknnBackend, blob};

/// 모델 경로. 없으면 테스트를 건너뛴다.
fn model_path() -> Option<String> {
    std::env::var("NPUFORGE_TEST_MODEL").ok()
}

fn spec(path: &str) -> ModelSpec {
    ModelSpec {
        id: "yolov8n".into(),
        version: "1.0.0".into(),
        backend: "rknn".into(),
        model_file: path.into(),
        input_width: 640,
        input_height: 640,
        input_channels: 3,
        input_format: InputFormat::Rgb8,
        output_format: "yolo-detections".into(),
        sha256: "0".repeat(64),
        labels_file: None,
    }
}

fn dummy_input(len: usize) -> InferenceInput {
    // 결정적 패턴. 두 번 추론하면 같은 결과가 나와야 한다.
    let data: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
    InferenceInput {
        payload: bytes::Bytes::from(data),
        format: InputFormat::Rgb8,
    }
}

#[tokio::test]
async fn loads_model_and_reports_nine_outputs() {
    let Some(path) = model_path() else { return };

    let backend = RknnBackend::new("/usr/lib/librknnrt.so").with_context_count(2);
    let model = backend.load_model(&spec(&path)).await.expect("모델 로딩");

    assert!(model.model_info().state.accepts_requests());
    assert!(
        model.model_info().supports_concurrent_infer,
        "컨텍스트 풀이 직렬화를 처리하므로 동시 호출이 가능해야 한다"
    );

    let out = model.infer(dummy_input(640 * 640 * 3)).await.expect("추론");

    assert_eq!(out.result_format, blob::RESULT_FORMAT);
    let decoded = blob::decode(&out.result).expect("blob 해석");
    assert_eq!(
        decoded.tensors.len(),
        9,
        "YOLOv8n 은 출력 9개다. 하나만 반환하면 후처리가 불가능하다"
    );
    assert!(
        !decoded.float_output,
        "기본값은 원본 dtype 이다. 네트워크 대역폭 때문이다"
    );
    assert!(out.timing.inference_us > 0);

    // 모든 텐서가 실제 데이터를 담고 있어야 한다
    for (i, (shape, data)) in decoded.tensors.iter().enumerate() {
        assert!(!data.is_empty(), "텐서 {i} 가 비었다");
        assert!(shape.n_dims > 0, "텐서 {i} 의 형태가 없다");
    }
}

#[tokio::test]
async fn quantized_output_carries_dequantization_params() {
    // want_float=0 으로 int8 을 보내면서 scale·zero_point 를 빼먹으면
    // 받는 쪽이 해석할 방법이 없다. 데이터가 그냥 못 읽는 바이트열이 된다.
    let Some(path) = model_path() else { return };

    let backend = RknnBackend::new("/usr/lib/librknnrt.so"); // 기본 want_float=false
    let model = backend.load_model(&spec(&path)).await.expect("모델 로딩");
    let out = model.infer(dummy_input(640 * 640 * 3)).await.expect("추론");
    let decoded = blob::decode(&out.result).expect("blob 해석");

    assert!(!decoded.float_output);
    let quantized = decoded
        .tensors
        .iter()
        .filter(|(sh, _)| sh.needs_dequant())
        .count();
    assert!(
        quantized > 0,
        "INT8 모델인데 역양자화 파라미터가 붙은 텐서가 하나도 없다"
    );
    for (i, (sh, _)) in decoded.tensors.iter().enumerate() {
        if sh.needs_dequant() {
            assert!(
                sh.scale > 0.0,
                "텐서 {i} 의 scale 이 0 이하다: {}",
                sh.scale
            );
        }
    }
}

#[tokio::test]
async fn dequantized_output_matches_float_output() {
    // 이것이 want_float=0 전환의 핵심 검증이다.
    // 같은 입력에 대해
    //   (a) want_float=1 로 받은 float32
    //   (b) want_float=0 으로 받은 int8 을 직접 역양자화한 값
    // 이 일치해야 한다. 다르면 파라미터가 틀렸거나 공식이 틀린 것이다.
    let Some(path) = model_path() else { return };
    let input = || dummy_input(640 * 640 * 3);

    let f32_out = RknnBackend::new("/usr/lib/librknnrt.so")
        .with_want_float(true)
        .load_model(&spec(&path))
        .await
        .expect("f32 모델")
        .infer(input())
        .await
        .expect("f32 추론");

    let i8_out = RknnBackend::new("/usr/lib/librknnrt.so")
        .with_want_float(false)
        .load_model(&spec(&path))
        .await
        .expect("int8 모델")
        .infer(input())
        .await
        .expect("int8 추론");

    // 네트워크 관점의 이득부터 확인한다
    assert!(
        i8_out.result.len() * 3 < f32_out.result.len(),
        "int8 출력이 float32 의 절반 이하여야 한다: {} vs {}",
        i8_out.result.len(),
        f32_out.result.len()
    );

    let f = blob::decode(&f32_out.result).expect("f32 blob");
    let q = blob::decode(&i8_out.result).expect("int8 blob");
    assert_eq!(f.tensors.len(), q.tensors.len());

    let mut checked = 0usize;
    let mut worst = 0.0_f32;
    for (i, ((_, fdata), (qsh, qdata))) in f.tensors.iter().zip(q.tensors.iter()).enumerate() {
        if !qsh.needs_dequant() {
            continue;
        }
        assert_eq!(fdata.len(), qdata.len() * 4, "텐서 {i} 크기 관계");

        for (k, qb) in qdata.iter().enumerate() {
            let expected = f32::from_le_bytes([
                fdata[k * 4],
                fdata[k * 4 + 1],
                fdata[k * 4 + 2],
                fdata[k * 4 + 3],
            ]);
            let got = qsh.dequantize(*qb as i8 as i32);
            let diff = (expected - got).abs();
            if diff > worst {
                worst = diff;
            }
            assert!(
                diff <= qsh.scale.abs() * 0.5 + 1e-6,
                "텐서 {i} 원소 {k}: float32 {expected} vs 역양자화 {got} (scale {})",
                qsh.scale
            );
        }
        checked += 1;
    }
    assert!(checked > 0, "역양자화를 검증한 텐서가 없다");
    println!("역양자화 검증: 텐서 {checked}개, 최대 오차 {worst:e}");
}

#[tokio::test]
async fn same_input_gives_same_output() {
    // 결정적이어야 벤치마크 결과를 신뢰할 수 있다.
    let Some(path) = model_path() else { return };

    let backend = RknnBackend::new("/usr/lib/librknnrt.so");
    let model = backend.load_model(&spec(&path)).await.expect("모델 로딩");

    let a = model
        .infer(dummy_input(640 * 640 * 3))
        .await
        .expect("추론 1");
    let b = model
        .infer(dummy_input(640 * 640 * 3))
        .await
        .expect("추론 2");

    assert_eq!(a.result, b.result, "같은 입력은 같은 출력을 내야 한다");
}

#[tokio::test]
async fn wrong_input_size_is_rejected() {
    // RKNN 은 크기가 다르면 조용히 다른 메모리를 읽는다. 막아야 한다.
    let Some(path) = model_path() else { return };

    let backend = RknnBackend::new("/usr/lib/librknnrt.so");
    let model = backend.load_model(&spec(&path)).await.expect("모델 로딩");

    let err = model
        .infer(dummy_input(1024))
        .await
        .expect_err("크기가 틀린 입력은 거부되어야 한다");
    assert!(
        err.code.is_client_error(),
        "요청 문제로 분류되어야 한다: {err}"
    );
}

#[tokio::test]
async fn jpeg_input_is_rejected_with_a_clear_message() {
    let Some(path) = model_path() else { return };

    let backend = RknnBackend::new("/usr/lib/librknnrt.so");
    let model = backend.load_model(&spec(&path)).await.expect("모델 로딩");

    let err = model
        .infer(InferenceInput {
            payload: bytes::Bytes::from_static(b"\xff\xd8\xff"),
            format: InputFormat::Jpeg,
        })
        .await
        .expect_err("JPEG 은 아직 지원하지 않는다");
    assert!(err.message.contains("RGB8"), "실제: {}", err.message);
}

#[tokio::test]
async fn spec_mismatch_is_caught_at_load_time() {
    // model.toml 의 입력 크기가 틀리면 로딩에서 막아야 한다.
    // 통과시키면 노드가 잘못된 크기를 받아들이고 RKNN 이 다른 것을 읽는다.
    let Some(path) = model_path() else { return };

    let mut bad = spec(&path);
    bad.input_width = 320;

    let backend = RknnBackend::new("/usr/lib/librknnrt.so");
    // Box<dyn LoadedModel> 은 Debug 가 아니라 expect_err 를 쓸 수 없다.
    let Err(err) = backend.load_model(&bad).await else {
        panic!("크기가 어긋난 spec 은 거부되어야 한다");
    };
    assert!(err.message.contains("입력 크기"), "실제: {}", err.message);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_inference_does_not_mix_results() {
    // 이 테스트가 컨텍스트 풀의 존재 이유다.
    //
    // 컨텍스트를 공유하면 inputs_set → run → outputs_get 시퀀스가 겹쳐
    // 오류 없이 다른 요청의 결과를 가져갈 수 있다. 서로 다른 입력을
    // 동시에 던지고, 각자가 단독 실행 때와 같은 결과를 받는지 본다.
    let Some(path) = model_path() else { return };

    let backend = RknnBackend::new("/usr/lib/librknnrt.so").with_context_count(4);
    let model: std::sync::Arc<dyn LoadedModel> =
        std::sync::Arc::from(backend.load_model(&spec(&path)).await.expect("모델 로딩"));

    // 기준: 각 입력을 단독으로 추론
    let mut inputs = Vec::new();
    let mut baseline = Vec::new();
    for seed in 0..4u8 {
        let data: Vec<u8> = (0..640 * 640 * 3)
            .map(|i| ((i + seed as usize * 37) % 251) as u8)
            .collect();
        let payload = bytes::Bytes::from(data);
        let out = model
            .infer(InferenceInput {
                payload: payload.clone(),
                format: InputFormat::Rgb8,
            })
            .await
            .expect("기준 추론");
        inputs.push(payload);
        baseline.push(out.result);
    }

    // 같은 입력들을 동시에
    let mut tasks = Vec::new();
    for (i, payload) in inputs.iter().enumerate() {
        let m = std::sync::Arc::clone(&model);
        let p = payload.clone();
        tasks.push(tokio::spawn(async move {
            let out = m
                .infer(InferenceInput {
                    payload: p,
                    format: InputFormat::Rgb8,
                })
                .await
                .expect("동시 추론");
            (i, out.result)
        }));
    }

    for t in tasks {
        let (i, result) = t.await.expect("태스크");
        assert_eq!(
            result, baseline[i],
            "입력 {i} 의 결과가 단독 실행과 다르다. 컨텍스트가 섞였다"
        );
    }
}
