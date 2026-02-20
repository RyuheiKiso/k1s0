fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_proto = "../../../../../api/proto/k1s0/system/config/v1/config.proto";
    let proto_include = "../../../../../api/proto";

    // proto ファイルが存在し、protoc が利用可能な場合のみコード生成を実行する。
    // CI/CD や buf generate 環境以外では手動型定義で代替するためスキップ可。
    if !std::path::Path::new(config_proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            config_proto
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[config_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for config proto");
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
