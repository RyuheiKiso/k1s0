fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_store_proto =
        "../../../../../api/proto/k1s0/system/eventstore/v1/event_store.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(event_store_proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            event_store_proto
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[event_store_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for event-store proto");
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
