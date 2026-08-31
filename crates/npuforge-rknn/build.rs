//! RKNN C Wrapper 빌드.
//!
//! `rknn` feature가 켜져 있고 대상이 Linux일 때만 C 코드를 컴파일하고
//! `librknnrt.so`에 링크한다. 그 외의 경우 이 크레이트는 아무것도 하지 않으므로
//! Windows/x86 개발 PC에서도 `cargo build --workspace`가 통과한다.

fn main() {
    println!("cargo:rerun-if-changed=native/rknn_wrapper.c");
    println!("cargo:rerun-if-changed=native/rknn_wrapper.h");
    println!("cargo:rerun-if-env-changed=RKNN_SDK_PATH");

    if std::env::var_os("CARGO_FEATURE_RKNN").is_none() {
        return;
    }

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "linux" {
        panic!(
            "npuforge-rknn 의 `rknn` feature는 Linux 대상에서만 사용할 수 있습니다. \
             현재 대상: {target_os}. 개발 PC에서는 feature를 끄고 빌드하세요."
        );
    }

    // RKNN SDK는 저장소에 포함하지 않는다. 사용자가 공식 경로에서 설치한 뒤
    // 헤더 위치를 RKNN_SDK_PATH로 알려준다.
    // docs/03-DEVELOPMENT-REQUIREMENTS.md §5.1 참조.
    let sdk_path = std::env::var("RKNN_SDK_PATH").unwrap_or_else(|_| {
        panic!(
            "RKNN_SDK_PATH 환경변수가 필요합니다. \
             rknn_api.h 가 있는 디렉터리를 지정하세요. \
             예: RKNN_SDK_PATH=/usr/include/rknn"
        )
    });

    cc::Build::new()
        .file("native/rknn_wrapper.c")
        .include("native")
        .include(&sdk_path)
        .warnings(true)
        .compile("npf_rknn_wrapper");

    println!("cargo:rustc-link-lib=dylib=rknnrt");
    if let Ok(lib_dir) = std::env::var("RKNN_LIB_PATH") {
        println!("cargo:rustc-link-search=native={lib_dir}");
    }
}
