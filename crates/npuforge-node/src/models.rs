//! 모델 디렉터리 로딩.
//!
//! 레이아웃은 `docs/01-TECHSPEC.md` §13.1 을 따른다.
//!
//! ```text
//! models/
//!   yolov8n/
//!     model.toml     ← ModelSpec
//!     yolov8n.rknn
//! ```
//!
//! `sha256` 은 세 노드가 같은 모델 파일을 쓰는지 확인하는 유일한 근거다.
//! 노드마다 다른 파일을 로딩한 채 벤치마크를 돌리면 정책 비교 결과가 통째로
//! 무의미해지므로, 실제 백엔드에서는 로딩 전에 반드시 검증한다.

use std::path::{Path, PathBuf};

use npuforge_common::model::ModelSpec;

/// 모델 하나를 디스크에서 읽는다.
pub fn load_spec(directory: &str, model_id: &str) -> Result<(ModelSpec, PathBuf), String> {
    let dir = Path::new(directory).join(model_id);
    let manifest = dir.join("model.toml");
    let spec = ModelSpec::from_path(&manifest).map_err(|e| e.to_string())?;

    if spec.id != model_id {
        return Err(format!(
            "{} 의 id 가 '{}' 인데 디렉터리 이름은 '{model_id}' 입니다",
            manifest.display(),
            spec.id
        ));
    }

    let model_file = dir.join(&spec.model_file);
    Ok((spec, model_file))
}

/// 모델 파일의 SHA-256 이 `model.toml` 의 선언과 같은지 확인한다.
///
/// 파일이 없거나 다르면 오류다. "일단 뜨고 나중에 확인"은 하지 않는다.
/// 잘못된 모델로 수집한 측정치는 되돌릴 수 없다.
pub fn verify_sha256(model_file: &Path, expected: &str) -> Result<(), String> {
    let bytes = std::fs::read(model_file)
        .map_err(|e| format!("{} 를 읽을 수 없습니다: {e}", model_file.display()))?;
    let actual = sha256_hex(&bytes);
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!(
            "{} 의 SHA-256 불일치\n  선언: {expected}\n  실제: {actual}\n\
             세 노드가 같은 모델 파일을 쓰는지 확인하세요",
            model_file.display()
        ));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use npuforge_common::protocol::InputFormat;

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("npuforge_models_{name}"));
        std::fs::remove_dir_all(&d).ok();
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn write_model(root: &Path, id: &str, declared_id: &str, sha: &str) -> PathBuf {
        let dir = root.join(id);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("model.bin"), b"weights").unwrap();
        std::fs::write(
            dir.join("model.toml"),
            format!(
                r#"
id = "{declared_id}"
version = "1.0.0"
backend = "mock"
model_file = "model.bin"
input_width = 640
input_height = 640
input_channels = 3
input_format = "rgb8"
output_format = "yolo-detections"
sha256 = "{sha}"
"#
            ),
        )
        .unwrap();
        dir
    }

    #[test]
    fn loads_spec_and_resolves_model_path() {
        let root = temp_dir("load");
        write_model(&root, "yolov8n", "yolov8n", &sha256_hex(b"weights"));
        let (spec, file) = load_spec(root.to_str().unwrap(), "yolov8n").unwrap();
        assert_eq!(spec.id, "yolov8n");
        assert_eq!(spec.input_format, InputFormat::Rgb8);
        assert_eq!(spec.raw_input_bytes(), 640 * 640 * 3);
        assert!(file.ends_with("model.bin"));
    }

    #[test]
    fn id_mismatch_is_rejected() {
        // 디렉터리를 복사해 이름만 바꾸는 실수를 잡는다.
        let root = temp_dir("mismatch");
        write_model(&root, "yolov8s", "yolov8n", &sha256_hex(b"weights"));
        let err = load_spec(root.to_str().unwrap(), "yolov8s").unwrap_err();
        assert!(err.contains("yolov8n"), "실제 메시지: {err}");
    }

    #[test]
    fn sha256_match_passes() {
        let root = temp_dir("sha_ok");
        let sha = sha256_hex(b"weights");
        let dir = write_model(&root, "yolov8n", "yolov8n", &sha);
        verify_sha256(&dir.join("model.bin"), &sha).unwrap();
    }

    #[test]
    fn sha256_mismatch_reports_both_values() {
        // 노드마다 다른 모델을 로딩한 채 벤치마크를 돌리면 결과가 무의미해진다.
        let root = temp_dir("sha_bad");
        let dir = write_model(&root, "yolov8n", "yolov8n", &"0".repeat(64));
        let err = verify_sha256(&dir.join("model.bin"), &"0".repeat(64)).unwrap_err();
        assert!(err.contains("선언"), "실제 메시지: {err}");
        assert!(
            err.contains(&sha256_hex(b"weights")),
            "실제 값도 보여야 한다"
        );
    }

    #[test]
    fn missing_manifest_names_the_path() {
        let root = temp_dir("missing");
        let err = load_spec(root.to_str().unwrap(), "nonexistent").unwrap_err();
        assert!(err.contains("model.toml"), "실제 메시지: {err}");
    }

    #[test]
    fn sha256_matches_known_vector() {
        // 빈 입력의 SHA-256. 구현이 바뀌어도 값이 흔들리지 않게 고정한다.
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
