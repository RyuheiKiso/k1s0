fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dlq_proto = "../../../../../api/proto/k1s0/system/dlq/v1/dlq.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(dlq_proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            dlq_proto
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[dlq_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for dlq proto");
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
