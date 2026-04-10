// HIGH-001 監査対応: build.rs の unnecessary_wraps は Result 戻り値の慣用的パターンとして許容する
#![allow(clippy::unnecessary_wraps)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ai_agent_proto = "../../../../../api/proto/k1s0/system/ai_agent/v1/ai_agent.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(ai_agent_proto).exists() {
        println!("cargo:warning=Proto file not found, skipping tonic codegen: {ai_agent_proto}");
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[ai_agent_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for ai_agent proto");
        }
        Err(e) => {
            println!("cargo:warning=tonic-build failed (protoc may not be installed): {e}");
        }
    }
    Ok(())
}
