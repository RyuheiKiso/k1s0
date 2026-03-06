fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_monitor_proto =
        "../../../../../api/proto/k1s0/system/eventmonitor/v1/event_monitor.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(event_monitor_proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            event_monitor_proto
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[event_monitor_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for event_monitor proto");
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
