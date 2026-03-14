fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ai_gateway_proto =
        "../../../../../api/proto/k1s0/system/ai_gateway/v1/ai_gateway.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(ai_gateway_proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            ai_gateway_proto
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[ai_gateway_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for ai_gateway proto");
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
