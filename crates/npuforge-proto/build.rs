//! gRPC 코드 생성.
//!
//! `protoc` 가 필요하다. 없으면 빌드가 실패하므로 안내 메시지를 남긴다.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/npuforge.proto");

    let result = tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/npuforge.proto"], &["proto"]);

    if let Err(e) = result {
        eprintln!();
        eprintln!("proto 컴파일에 실패했습니다: {e}");
        eprintln!();
        eprintln!("protoc 가 설치되어 있는지 확인하세요.");
        eprintln!("  Ubuntu/Debian : sudo apt install protobuf-compiler");
        eprintln!("  Rocky/RHEL    : sudo dnf install protobuf-compiler");
        eprintln!("  Windows       : winget install protobuf");
        eprintln!();
        return Err(e.into());
    }

    Ok(())
}
