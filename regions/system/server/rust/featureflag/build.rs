fn main() -> Result<(), Box<dyn std::error::Error>> {
    let featureflag_proto = "../../../../../api/proto/k1s0/system/featureflag/v1/featureflag.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(featureflag_proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            featureflag_proto
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[featureflag_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for featureflag proto");
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
