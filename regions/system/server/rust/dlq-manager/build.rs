fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = std::path::PathBuf::from(&manifest_dir);

    let dlq_proto = manifest_path
        .join("../../../../../api/proto/k1s0/system/dlq/v1/dlq.proto")
        .canonicalize();
    let proto_include = manifest_path
        .join("../../../../../api/proto")
        .canonicalize();

    let (dlq_proto, proto_include) = match (dlq_proto, proto_include) {
        (Ok(p), Ok(i)) => (p, i),
        _ => {
            println!("cargo:warning=Proto file not found, skipping tonic codegen");
            return Ok(());
        }
    };

    let out_dir = manifest_path.join("src/proto");

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir(&out_dir)
        .compile_protos(&[&dlq_proto], &[&proto_include])
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
