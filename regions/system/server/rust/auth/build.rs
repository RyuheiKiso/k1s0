fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth_proto = "../../proto/v1/auth.proto";
    let common_proto = "../../proto/v1/common.proto";
    let proto_include = "../../proto";

    // proto ファイルが存在し、protoc が利用可能な場合のみコード生成を実行する。
    // CI/CD や buf generate 環境以外では手動型定義で代替するためスキップ可。
    if !std::path::Path::new(auth_proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            auth_proto
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[auth_proto, common_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for auth proto");
        }
        Err(e) => {
            println!(
                "cargo:warning=tonic-build failed (protoc may not be installed): {}",
                e
            );
        }
    }
    Ok(())
}
