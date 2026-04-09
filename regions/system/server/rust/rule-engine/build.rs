// HIGH-001 監査対応: build.rs の unnecessary_wraps は Result 戻り値の慣用的パターンとして許容する
#![allow(clippy::unnecessary_wraps)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rule_engine_proto = "../../../../../api/proto/k1s0/system/ruleengine/v1/rule_engine.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(rule_engine_proto).exists() {
        println!("cargo:warning=Proto file not found, skipping tonic codegen: {rule_engine_proto}");
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[rule_engine_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for rule_engine proto");
        }
        Err(e) => {
            println!("cargo:warning=tonic-build failed (protoc may not be installed): {e}");
        }
    }
    Ok(())
}
