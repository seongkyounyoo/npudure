//! 오류 코드와 공통 오류 타입.
//!
//! 오류 코드는 외부 API의 일부이며 안정적으로 유지한다.
//! 코드 문자열을 변경하거나 재사용하지 않는다. `docs/01-TECHSPEC.md` §21 참조.

use serde::{Deserialize, Serialize};

/// NPUForge 오류 코드.
///
/// 외부 API 응답과 메트릭 레이블에 사용한다.
/// 새 코드는 추가만 하고, 기존 코드의 의미를 바꾸지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    Ok,

    // 1000번대: 요청 자체의 문제. 재시도 불가.
    InvalidRequest,
    PayloadTooLarge,
    UnsupportedInputFormat,
    /// 노드 등록 토큰 불일치.
    Unauthorized,

    // 1100번대: 모델. 재시도 불가.
    ModelNotFound,
    ModelVersionMismatch,

    // 1200번대: 스케줄링.
    NoAvailableNode,
    DeadlineUnsatisfiable,
    /// 스케줄러가 정한 요청 처리 시간 상한을 넘겼다. 재시도 전체를 포함한 시간이다.
    RequestTimeout,

    // 1300번대: 노드. 재시도 가능.
    NodeTimeout,
    NodeUnavailable,
    NodeOverloaded,

    // 1400번대: 백엔드.
    BackendError,
    InferenceFailed,

    // 1500번대: 내부.
    InternalError,
}

impl ErrorCode {
    /// 모든 코드. 문자열 역변환과 테스트에 쓴다.
    pub const ALL: [Self; 16] = [
        Self::Ok,
        Self::InvalidRequest,
        Self::PayloadTooLarge,
        Self::UnsupportedInputFormat,
        Self::Unauthorized,
        Self::ModelNotFound,
        Self::ModelVersionMismatch,
        Self::NoAvailableNode,
        Self::DeadlineUnsatisfiable,
        Self::RequestTimeout,
        Self::NodeTimeout,
        Self::NodeUnavailable,
        Self::NodeOverloaded,
        Self::BackendError,
        Self::InferenceFailed,
        Self::InternalError,
    ];

    /// `NPF-xxxx` 형식의 안정적인 문자열 표현.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "NPF-0000",
            Self::InvalidRequest => "NPF-1001",
            Self::PayloadTooLarge => "NPF-1002",
            Self::UnsupportedInputFormat => "NPF-1003",
            Self::Unauthorized => "NPF-1004",
            Self::ModelNotFound => "NPF-1101",
            Self::ModelVersionMismatch => "NPF-1102",
            Self::NoAvailableNode => "NPF-1201",
            Self::DeadlineUnsatisfiable => "NPF-1202",
            Self::RequestTimeout => "NPF-1203",
            Self::NodeTimeout => "NPF-1301",
            Self::NodeUnavailable => "NPF-1302",
            Self::NodeOverloaded => "NPF-1303",
            Self::BackendError => "NPF-1401",
            Self::InferenceFailed => "NPF-1402",
            Self::InternalError => "NPF-1501",
        }
    }

    /// `NPF-xxxx` 문자열을 다시 열거형으로 되돌린다.
    ///
    /// 노드가 보낸 코드를 스케줄러가 재시도 판정에 쓰려면 필요하다.
    /// 모르는 코드는 `None` 이다. 호출자가 보수적인 기본값을 정한다.
    pub fn from_str_code(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|c| c.as_str() == s)
    }

    /// 다른 노드에서 재시도할 가치가 있는 오류인지 판단한다.
    ///
    /// 추론 요청은 side effect가 없으므로 노드 측 일시 장애는 재시도한다.
    /// 요청 자체가 잘못된 경우는 어느 노드로 보내도 같은 결과이므로 재시도하지 않는다.
    /// `docs/01-TECHSPEC.md` §12.2, §12.3 참조.
    ///
    /// `InferenceFailed`를 재시도 대상에 포함한 근거:
    /// 입력 검증을 통과한 요청의 추론 실패는 NPU 일시 오류, 메모리 부족,
    /// thermal 이상처럼 노드에 국한된 원인일 가능성이 높다. 모델 자체가 깨져
    /// 모든 노드에서 실패하는 경우에도 기본 `max_retries = 1` 때문에 부하는
    /// 최대 2배로 제한되고, 오류율 메트릭에 즉시 드러난다.
    /// 장애 노드를 우회해 서비스를 지속하는 것이 이 프로젝트의 목표이므로
    /// 이 방향의 오탐을 감수한다.
    pub const fn is_retryable(self) -> bool {
        matches!(
            self,
            Self::NodeTimeout
                | Self::NodeUnavailable
                | Self::NodeOverloaded
                | Self::BackendError
                | Self::InferenceFailed
        )
    }

    /// 클라이언트 잘못인지 여부. 메트릭 분류와 로그 레벨 결정에 사용한다.
    pub const fn is_client_error(self) -> bool {
        matches!(
            self,
            Self::InvalidRequest
                | Self::PayloadTooLarge
                | Self::UnsupportedInputFormat
                | Self::Unauthorized
                | Self::ModelNotFound
                | Self::ModelVersionMismatch
        )
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// NPUForge 공통 오류.
#[derive(Debug, thiserror::Error)]
#[error("{code}: {message}")]
pub struct NpuForgeError {
    pub code: ErrorCode,
    pub message: String,
}

impl NpuForgeError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalError, message)
    }

    pub fn is_retryable(&self) -> bool {
        self.code.is_retryable()
    }
}

pub type Result<T, E = NpuForgeError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_are_unique() {
        // 오류 코드 문자열이 중복되면 외부 API 소비자가 구분할 수 없다.
        // ALL 을 순회하므로 코드를 추가하고 문자열을 빠뜨리면 여기서 잡힌다.
        let mut seen = std::collections::HashSet::new();
        for code in ErrorCode::ALL {
            assert!(
                seen.insert(code.as_str()),
                "중복된 오류 코드: {}",
                code.as_str()
            );
        }
        assert_eq!(seen.len(), ErrorCode::ALL.len());
    }

    #[test]
    fn string_codes_round_trip() {
        // 노드가 보낸 문자열을 스케줄러가 재시도 판정에 쓴다.
        for code in ErrorCode::ALL {
            assert_eq!(ErrorCode::from_str_code(code.as_str()), Some(code));
        }
    }

    #[test]
    fn unknown_string_code_is_none() {
        // 상위 버전 노드가 모르는 코드를 보내도 패닉하지 않아야 한다.
        assert_eq!(ErrorCode::from_str_code("NPF-9999"), None);
        assert_eq!(ErrorCode::from_str_code(""), None);
    }

    #[test]
    fn client_errors_are_never_retryable() {
        // 잘못된 요청을 다른 노드로 재시도하면 부하만 늘고 결과는 같다.
        for code in [
            ErrorCode::InvalidRequest,
            ErrorCode::PayloadTooLarge,
            ErrorCode::UnsupportedInputFormat,
            ErrorCode::Unauthorized,
            ErrorCode::ModelNotFound,
            ErrorCode::ModelVersionMismatch,
        ] {
            assert!(code.is_client_error());
            assert!(
                !code.is_retryable(),
                "{code} 는 재시도 대상이 아니어야 한다"
            );
        }
    }

    #[test]
    fn node_failures_are_retryable() {
        for code in [
            ErrorCode::NodeTimeout,
            ErrorCode::NodeUnavailable,
            ErrorCode::NodeOverloaded,
        ] {
            assert!(code.is_retryable(), "{code} 는 재시도 대상이어야 한다");
        }
    }
}
