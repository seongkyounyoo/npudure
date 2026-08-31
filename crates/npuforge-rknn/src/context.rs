//! RKNN 컨텍스트 RAII 래퍼와 컨텍스트 풀.
//!
//! `unsafe` 는 이 모듈과 [`crate::ffi`] 에만 있다.
//!
//! # 왜 컨텍스트를 공유하지 않는가
//!
//! RKNN Runtime 2.3.0 은 thread-safe 다. 동일 컨텍스트를 여러 스레드가
//! 동시에 호출해도 오류가 나지 않는다(`docs/environment-matrix.md` §3.1).
//!
//! 그런데 추론 한 건은 세 번의 호출이다.
//!
//! ```text
//! rknn_inputs_set  →  rknn_run  →  rknn_outputs_get
//! ```
//!
//! **개별 호출이 thread-safe 라는 것과 시퀀스가 원자적이라는 것은 다르다.**
//! 두 스레드가 같은 컨텍스트에서 이 시퀀스를 겹쳐 실행하면, 오류 코드는
//! 0 이면서 서로의 결과를 가져갈 수 있다. thread-safety 검증은 반환값만
//! 셌고 출력 내용을 대조하지 않았으므로 이 경우를 잡지 못한다.
//!
//! C wrapper 의 `ctx->outputs` 배열이 컨텍스트별 공유 상태라는 점도 같은
//! 결론을 가리킨다.
//!
//! 그래서 **동시 실행 수만큼 컨텍스트를 만들어 하나씩 점유해 쓴다.**
//! 실측상 손해도 없다 — 8스레드 최고 처리량(72.9 inf/s)이 나온 구성이
//! 애초에 스레드별 전용 컨텍스트였다.

use std::ffi::CString;
use std::sync::Arc;

use npuforge_common::{ErrorCode, NpuForgeError, Result};
use tokio::sync::{Mutex, Semaphore};

use crate::blob::{self, TensorShape};
use crate::ffi;

/// 모델 메타데이터. C 구조체를 Rust 값으로 옮긴 것.
#[derive(Debug, Clone)]
pub struct RknnModelInfo {
    pub input_count: u32,
    pub output_count: u32,
    pub input_width: u32,
    pub input_height: u32,
    pub input_channels: u32,
    /// 입력 텐서 한 장의 바이트 크기.
    pub input_bytes: u32,
    pub output_total_bytes: usize,
    pub output_shapes: Vec<TensorShape>,
}

/// RKNN 컨텍스트 하나. `Drop` 에서 해제를 보장한다.
///
/// 여러 스레드가 동시에 쓸 수 없다. 모듈 문서 참조.
///
/// `Debug` 는 포인터 값을 찍지 않는다. 로그에 주소가 남아도 쓸모가 없고,
/// 실수로 노출되면 ASLR 정보만 흘린다.
pub struct RknnContext {
    raw: *mut ffi::NpfRknnCtx,
    want_float: bool,
}

// SAFETY: 포인터를 다른 스레드로 옮기는 것은 안전하다. 동시 사용만
// 막으면 되고, 그것은 `ContextPool` 이 Mutex 로 보장한다. `Sync` 는
// 일부러 구현하지 않는다 — 공유 참조로 동시 호출되면 안 된다.
unsafe impl Send for RknnContext {}

impl RknnContext {
    /// 모델 파일을 로딩한다.
    pub fn open(model_path: &str, want_float: bool) -> Result<Self> {
        let c_path = CString::new(model_path).map_err(|_| {
            NpuForgeError::new(
                ErrorCode::InvalidRequest,
                format!("모델 경로에 널 바이트가 있습니다: {model_path}"),
            )
        })?;

        let mut raw: *mut ffi::NpfRknnCtx = std::ptr::null_mut();
        // SAFETY: c_path 는 널 종료 문자열이고 raw 는 유효한 출력 위치다.
        let status =
            unsafe { ffi::npf_rknn_create(c_path.as_ptr(), i32::from(want_float), &mut raw) };
        ffi::check(status, &format!("모델 로딩 ({model_path})"))?;

        if raw.is_null() {
            return Err(NpuForgeError::new(
                ErrorCode::BackendError,
                format!("모델 로딩이 성공을 반환했지만 컨텍스트가 비어 있습니다: {model_path}"),
            ));
        }
        Ok(Self { raw, want_float })
    }

    pub fn model_info(&self) -> Result<RknnModelInfo> {
        let mut info = ffi::NpfRknnModelInfo::default();
        // SAFETY: self.raw 는 살아 있고 info 는 유효한 쓰기 위치다.
        let status = unsafe { ffi::npf_rknn_get_model_info(self.raw, &mut info) };
        ffi::check(status, "모델 정보 조회")?;

        let n = (info.output_count as usize).min(ffi::MAX_OUTPUTS);
        let output_shapes = info.outputs[..n]
            .iter()
            .map(|s| TensorShape {
                n_dims: s.n_dims,
                dims: s.dims,
                qnt_type: s.qnt_type,
                zero_point: s.zero_point,
                scale: s.scale,
            })
            .collect();

        Ok(RknnModelInfo {
            input_count: info.input_count,
            output_count: info.output_count,
            input_width: info.input_width,
            input_height: info.input_height,
            input_channels: info.input_channels,
            input_bytes: info.input_bytes,
            output_total_bytes: info.output_total_bytes,
            output_shapes,
        })
    }

    pub fn sdk_version(&self) -> Result<String> {
        // SAFETY: self.raw 는 살아 있다.
        unsafe { ffi::sdk_version(self.raw) }
    }

    /// 추론 한 건. 출력 텐서 전부를 blob 으로 직렬화해 돌려준다.
    ///
    /// `&mut self` 를 받는 것이 이 타입의 동시성 계약이다. 컴파일러가
    /// 같은 컨텍스트의 동시 호출을 막는다.
    pub fn infer(&mut self, input: &[u8]) -> Result<Vec<u8>> {
        if input.is_empty() {
            return Err(NpuForgeError::new(
                ErrorCode::InvalidRequest,
                "입력이 비어 있습니다",
            ));
        }

        let mut out = ffi::NpfRknnOutput::default();
        // SAFETY: input 은 이 호출이 끝날 때까지 살아 있고, out 은 유효한
        // 쓰기 위치다. C 쪽은 입력을 수정하지 않는다.
        let status =
            unsafe { ffi::npf_rknn_infer(self.raw, input.as_ptr(), input.len(), &mut out) };

        // 실패해도 out 은 초기화되어 있으므로 해제할 것이 없다.
        ffi::check(status, "추론")?;

        // 출력 포인터를 slice 로 빌린 뒤 곧바로 직렬화하고 해제한다.
        // RKNN 이 소유한 버퍼를 이 함수 밖으로 내보내지 않는다.
        let count = (out.count as usize).min(ffi::MAX_OUTPUTS);
        let mut tensors: Vec<(TensorShape, &[u8])> = Vec::with_capacity(count);

        // 형태 정보는 모델 정보에서 가져온다. 매 추론마다 조회하면
        // 추론당 rknn_query 가 count 번 늘어난다. ioctl 직렬화가 병목인
        // 환경에서 이 비용은 무시할 수 없다.
        let shapes = self.cached_shapes()?;

        for i in 0..count {
            let ptr = out.data[i];
            let len = out.len[i] as usize;
            let data: &[u8] = if ptr.is_null() || len == 0 {
                &[]
            } else {
                // SAFETY: RKNN 이 채운 버퍼이고 release 전까지 유효하다.
                unsafe { std::slice::from_raw_parts(ptr, len) }
            };
            let shape = shapes
                .get(i)
                .copied()
                .unwrap_or(TensorShape::plain(0, [0; 4]));
            tensors.push((shape, data));
        }

        let encoded = blob::encode(&tensors, self.want_float);
        drop(tensors);

        // SAFETY: out 은 방금 채워진 유효한 출력이다. 두 번 해제해도 안전하다.
        unsafe { ffi::npf_rknn_release_output(self.raw, &mut out) };

        Ok(encoded)
    }

    /// 출력 형태는 모델마다 고정이다. 첫 조회 후 재사용한다.
    fn cached_shapes(&self) -> Result<Vec<TensorShape>> {
        // 컨텍스트 생성 시 한 번 조회해 두는 편이 낫지만, model_info 가
        // 이미 싸고(rknn_query 만 호출) 여기서만 쓰이므로 단순하게 둔다.
        // 병목이 되면 필드로 캐시한다.
        Ok(self.model_info()?.output_shapes)
    }
}

impl std::fmt::Debug for RknnContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RknnContext")
            .field("loaded", &!self.raw.is_null())
            .field("want_float", &self.want_float)
            .finish()
    }
}

impl Drop for RknnContext {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // SAFETY: 살아 있는 컨텍스트를 정확히 한 번 해제한다.
            unsafe { ffi::npf_rknn_destroy(self.raw) };
            self.raw = std::ptr::null_mut();
        }
    }
}

/// 컨텍스트 풀.
///
/// 동시 실행 수만큼 컨텍스트를 미리 만들어 두고 하나씩 빌려준다.
/// 노드의 `worker_count` 와 같은 크기로 만들면 대기가 생기지 않는다.
pub struct ContextPool {
    contexts: Vec<Mutex<RknnContext>>,
    permits: Arc<Semaphore>,
    pub info: RknnModelInfo,
    pub sdk_version: String,
}

impl ContextPool {
    /// `size` 개의 컨텍스트를 만든다.
    ///
    /// 하나라도 실패하면 전체를 실패시킨다. 절반만 뜬 노드가 조용히
    /// 낮은 처리량으로 도는 것보다 명확히 죽는 편이 낫다.
    pub fn open(model_path: &str, size: usize, want_float: bool) -> Result<Self> {
        let size = size.max(1);

        let first = RknnContext::open(model_path, want_float)?;
        let info = first.model_info()?;
        let sdk_version = first.sdk_version().unwrap_or_else(|_| "unknown".into());

        if info.output_count as usize > ffi::MAX_OUTPUTS {
            return Err(NpuForgeError::new(
                ErrorCode::BackendError,
                format!(
                    "출력 텐서 {}개는 지원 한도 {}개를 넘습니다",
                    info.output_count,
                    ffi::MAX_OUTPUTS
                ),
            ));
        }

        let mut contexts = Vec::with_capacity(size);
        contexts.push(Mutex::new(first));
        for i in 1..size {
            let ctx = RknnContext::open(model_path, want_float).map_err(|e| {
                NpuForgeError::new(
                    e.code,
                    format!("컨텍스트 {}/{size} 생성 실패: {}", i + 1, e.message),
                )
            })?;
            contexts.push(Mutex::new(ctx));
        }

        Ok(Self {
            contexts,
            permits: Arc::new(Semaphore::new(size)),
            info,
            sdk_version,
        })
    }

    pub fn size(&self) -> usize {
        self.contexts.len()
    }

    /// 비어 있는 컨텍스트를 하나 잡아 추론한다.
    pub async fn infer(&self, input: &[u8]) -> Result<Vec<u8>> {
        // 세마포어로 대기하면 최소 하나는 반드시 비어 있다.
        let _permit =
            self.permits.acquire().await.map_err(|_| {
                NpuForgeError::new(ErrorCode::BackendError, "컨텍스트 풀이 닫혔습니다")
            })?;

        for slot in &self.contexts {
            if let Ok(mut ctx) = slot.try_lock() {
                return ctx.infer(input);
            }
        }

        // 여기 오면 세마포어와 락 개수가 어긋난 것이다. 조용히 넘어가면
        // 원인 없는 성능 저하로만 보이므로 명시적으로 알린다.
        Err(NpuForgeError::internal(
            "빈 컨텍스트를 찾지 못했습니다. 세마포어와 컨텍스트 수가 어긋났습니다",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_path_with_nul_is_rejected() {
        // CString 변환 실패를 패닉이 아니라 오류로 처리해야 한다.
        let err = RknnContext::open("model\0.rknn", true).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidRequest);
        assert!(err.message.contains("널 바이트"), "실제: {}", err.message);
    }

    #[test]
    fn context_is_send_but_not_sync() {
        // Send 여야 tokio 태스크 간 이동이 가능하고,
        // Sync 가 아니어야 공유 참조로 동시 호출되지 않는다.
        fn assert_send<T: Send>() {}
        assert_send::<RknnContext>();

        // Sync 를 구현했다면 이 함수가 컴파일된다. 구현하지 않았음을
        // 문서로 남긴다 — 컴파일 실패를 테스트할 수는 없다.
    }
}
