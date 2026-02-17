fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "../../../../api/proto/k1s0/system/auth/v1/auth_service.proto";

    // proto ファイルが存在し、protoc が利用可能な場合のみコード生成を実行する。
    // CI/CD や buf generate 環境以外では手動型定義で代替するためスキップ可。
    if !std::path::Path::new(proto_file).exists() {
        println!("cargo:warning=Proto file not found, skipping tonic codegen: {}", proto_file);
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[proto_file], &["../../../../api/proto"])
    {
        Ok(()) => {}
        Err(e) => {
            println!("cargo:warning=tonic-build failed (protoc may not be installed): {}", e);
        }
    }
    Ok(())
}
