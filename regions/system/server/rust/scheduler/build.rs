fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scheduler_proto = "../../../../../api/proto/k1s0/system/scheduler/v1/scheduler.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(scheduler_proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            scheduler_proto
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[scheduler_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for scheduler proto");
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
