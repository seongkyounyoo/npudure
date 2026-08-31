//! 모델 식별과 메타데이터.
//!
//! `docs/01-TECHSPEC.md` §13 참조.

use serde::{Deserialize, Serialize};

pub type ModelId = String;
pub type ModelVersion = String;

/// 모델 디렉터리의 `model.toml` 내용.
///
/// 세 노드가 동일한 모델을 로딩했는지 확인하는 기준이 되므로 `sha256`은 필수다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSpec {
    pub id: ModelId,
    pub version: ModelVersion,
    /// 백엔드 종류. `rknn` 또는 `mock`.
    pub backend: String,
    /// 모델 디렉터리 기준 상대 경로.
    pub model_file: String,
    pub input_width: u32,
    pub input_height: u32,
    pub input_channels: u32,
    pub input_format: crate::protocol::InputFormat,
    pub output_format: String,
    /// 모델 파일의 SHA-256. 노드 간 일치 검증에 사용한다.
    pub sha256: String,
    #[serde(default)]
    pub labels_file: Option<String>,
}

impl ModelSpec {
    /// 입력 텐서 한 장의 raw 바이트 크기.
    ///
    /// 네트워크 대역폭 산정과 버퍼 풀 크기 결정에 사용한다.
    /// `docs/01-TECHSPEC.md` §3.2의 네트워크 기준선 근거가 이 값에 기반한다.
    pub const fn raw_input_bytes(&self) -> u64 {
        (self.input_width as u64) * (self.input_height as u64) * (self.input_channels as u64)
    }
}

impl ModelSpec {
    /// 모델 디렉터리의 `model.toml` 을 읽는다.
    ///
    /// 설정 로딩과 같은 크레이트에 둔다. 노드가 따로 TOML 파서를 들이면
    /// 버전이 갈리고, 같은 파일을 두 곳에서 다르게 해석할 여지가 생긴다.
    pub fn from_path(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, crate::config::ConfigError> {
        let path = path.as_ref();
        let display = path.display().to_string();
        let text =
            std::fs::read_to_string(path).map_err(|source| crate::config::ConfigError::Io {
                path: display.clone(),
                source,
            })?;
        toml::from_str(&text).map_err(|source| crate::config::ConfigError::Parse {
            path: display,
            source,
        })
    }
}

/// 노드가 보고하는 모델 로딩 상태.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelState {
    Unloaded,
    Loading,
    Ready,
    Failed,
    Draining,
}

impl ModelState {
    /// 이 모델로 요청을 라우팅할 수 있는지 여부.
    pub const fn accepts_requests(self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// 노드 등록 시 스케줄러에 전달하는 모델 정보.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub id: ModelId,
    pub version: ModelVersion,
    pub state: ModelState,
    pub sha256: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::InputFormat;

    fn yolov8n() -> ModelSpec {
        ModelSpec {
            id: "yolov8n".into(),
            version: "1.0.0".into(),
            backend: "rknn".into(),
            model_file: "model.rknn".into(),
            input_width: 640,
            input_height: 640,
            input_channels: 3,
            input_format: InputFormat::Rgb8,
            output_format: "yolo-detections".into(),
            sha256: "0".repeat(64),
            labels_file: Some("labels.txt".into()),
        }
    }

    #[test]
    fn raw_input_size_matches_techspec_figure() {
        // TECHSPEC §3.2의 네트워크 기준선 근거에서 사용한 1.23MB 수치.
        assert_eq!(yolov8n().raw_input_bytes(), 1_228_800);
    }

    #[test]
    fn only_ready_models_accept_requests() {
        assert!(ModelState::Ready.accepts_requests());
        for state in [
            ModelState::Unloaded,
            ModelState::Loading,
            ModelState::Failed,
            ModelState::Draining,
        ] {
            assert!(!state.accepts_requests());
        }
    }
}
