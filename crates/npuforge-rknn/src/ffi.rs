//! `native/rknn_wrapper.h` 에 대응하는 FFI 선언.
//!
//! 이 모듈은 `rknn` feature가 켜졌을 때만 컴파일된다.
//! Raw pointer는 이 모듈 밖으로 나가지 않는다.

use npuforge_common::{ErrorCode, NpuForgeError, Result};
use std::os::raw::{c_char, c_int};

/// `NPF_RKNN_MAX_DIMS` 와 같아야 한다.
pub const MAX_DIMS: usize = 4;
/// `NPF_RKNN_MAX_OUTPUTS` 와 같아야 한다.
pub const MAX_OUTPUTS: usize = 32;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NpfRknnTensorShape {
    pub n_dims: u32,
    pub dims: [u32; MAX_DIMS],
    pub size: u32,
    /// 0 없음 / 1 DFP / 2 affine asymmetric
    pub qnt_type: u32,
    pub zero_point: i32,
    pub scale: f32,
    /// `rknn_tensor_type`. want_float=1 이면 FLOAT32 로 덮여 온다.
    pub dtype: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NpfRknnModelInfo {
    pub input_count: u32,
    pub output_count: u32,
    pub input_width: u32,
    pub input_height: u32,
    pub input_channels: u32,
    pub input_bytes: u32,
    pub output_total_bytes: usize,
    pub outputs: [NpfRknnTensorShape; MAX_OUTPUTS],
}

impl Default for NpfRknnModelInfo {
    fn default() -> Self {
        Self {
            input_count: 0,
            output_count: 0,
            input_width: 0,
            input_height: 0,
            input_channels: 0,
            input_bytes: 0,
            output_total_bytes: 0,
            outputs: [NpfRknnTensorShape::default(); MAX_OUTPUTS],
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct NpfRknnOutput {
    pub count: u32,
    pub data: [*mut u8; MAX_OUTPUTS],
    pub len: [u32; MAX_OUTPUTS],
    pub held: c_int,
}

impl Default for NpfRknnOutput {
    fn default() -> Self {
        Self {
            count: 0,
            data: [std::ptr::null_mut(); MAX_OUTPUTS],
            len: [0; MAX_OUTPUTS],
            held: 0,
        }
    }
}

/// 불투명 컨텍스트. 내부 구조를 Rust에서 알지 못한다.
#[repr(C)]
pub struct NpfRknnCtx {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn npf_rknn_create(
        model_path: *const c_char,
        want_float: c_int,
        out_ctx: *mut *mut NpfRknnCtx,
    ) -> c_int;
    pub fn npf_rknn_destroy(ctx: *mut NpfRknnCtx);
    pub fn npf_rknn_get_model_info(ctx: *mut NpfRknnCtx, out: *mut NpfRknnModelInfo) -> c_int;
    pub fn npf_rknn_infer(
        ctx: *mut NpfRknnCtx,
        input: *const u8,
        input_len: usize,
        out: *mut NpfRknnOutput,
    ) -> c_int;
    pub fn npf_rknn_release_output(ctx: *mut NpfRknnCtx, output: *mut NpfRknnOutput);
    pub fn npf_rknn_get_runtime_version(buf: *mut c_char, buf_len: usize) -> c_int;
    pub fn npf_rknn_get_sdk_version(
        ctx: *mut NpfRknnCtx,
        buf: *mut c_char,
        buf_len: usize,
    ) -> c_int;
}

/// C wrapper 상태 코드를 NPUDure 오류로 변환한다.
pub fn check(status: c_int, context: &str) -> Result<()> {
    if status == 0 {
        return Ok(());
    }
    let reason = match status {
        -1 => "잘못된 인자",
        -2 => "모델 로딩 실패",
        -3 => "모델 정보 조회 실패",
        -4 => "입력 설정 실패",
        -5 => "추론 실행 실패",
        -6 => "출력 조회 실패",
        -7 => "메모리 할당 실패",
        -8 => "런타임 오류",
        -9 => "출력 텐서가 너무 많음",
        _ => "알 수 없는 오류",
    };
    let code = match status {
        -1 => ErrorCode::InvalidRequest,
        -2 | -3 | -9 => ErrorCode::BackendError,
        _ => ErrorCode::InferenceFailed,
    };
    Err(NpuForgeError::new(
        code,
        format!("{context}: {reason} (status={status})"),
    ))
}

/// 널 종료 C 문자열 버퍼를 Rust 문자열로 만든다.
fn buf_to_string(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

/// RKNPU 드라이버 버전. 컨텍스트 없이 조회할 수 있다.
pub fn runtime_version() -> Result<String> {
    let mut buf = [0_u8; 512];
    // SAFETY: buf 는 유효한 쓰기 가능 메모리이고 길이를 함께 전달한다.
    // C 쪽은 snprintf 로 널 종료를 보장한다.
    let status = unsafe { npf_rknn_get_runtime_version(buf.as_mut_ptr().cast(), buf.len()) };
    check(status, "runtime_version")?;
    Ok(buf_to_string(&buf))
}

/// 컨텍스트가 있을 때의 정확한 SDK 버전.
///
/// # Safety
/// `ctx` 는 살아 있는 유효한 컨텍스트여야 한다.
pub unsafe fn sdk_version(ctx: *mut NpfRknnCtx) -> Result<String> {
    let mut buf = [0_u8; 512];
    let status = unsafe { npf_rknn_get_sdk_version(ctx, buf.as_mut_ptr().cast(), buf.len()) };
    check(status, "sdk_version")?;
    Ok(buf_to_string(&buf))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_terminated_buffer_is_trimmed() {
        let mut buf = [0_u8; 16];
        buf[..5].copy_from_slice(b"0.9.8");
        assert_eq!(buf_to_string(&buf), "0.9.8");
    }

    #[test]
    fn buffer_without_null_is_still_read() {
        // C 쪽이 널을 못 넣는 경우에도 패닉하지 않아야 한다.
        let buf = *b"0123456789abcdef";
        assert_eq!(buf_to_string(&buf), "0123456789abcdef");
    }

    #[test]
    fn model_load_failure_is_a_backend_error_not_a_client_error() {
        // 모델 로딩은 노드 기동 시점에만 일어나고 요청 경로에 없다.
        // 따라서 재시도 여부는 의미가 없고, 클라이언트 잘못으로 분류되지
        // 않는 것이 중요하다. 잘못 분류하면 메트릭에서 4xx 로 집계된다.
        let err = check(-2, "create").unwrap_err();
        assert_eq!(err.code, ErrorCode::BackendError);
        assert!(!err.code.is_client_error());
    }

    #[test]
    fn invalid_argument_is_a_client_error() {
        // 입력 크기 불일치 등은 요청 자체의 문제다. 재시도해도 같다.
        let err = check(-1, "infer").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidRequest);
        assert!(err.code.is_client_error());
        assert!(!err.code.is_retryable());
    }

    #[test]
    fn inference_failure_is_retryable() {
        // NPU 일시 오류일 수 있으므로 다른 노드에서 재시도할 값어치가 있다.
        let err = check(-5, "infer").unwrap_err();
        assert_eq!(err.code, ErrorCode::InferenceFailed);
        assert!(err.code.is_retryable());
    }

    #[test]
    fn too_many_outputs_is_a_backend_error() {
        let err = check(-9, "create").unwrap_err();
        assert_eq!(err.code, ErrorCode::BackendError);
        assert!(err.message.contains("출력 텐서"));
    }

    #[test]
    fn unknown_status_still_produces_an_error() {
        let err = check(-999, "x").unwrap_err();
        assert!(err.message.contains("-999"), "실제: {}", err.message);
    }
}
