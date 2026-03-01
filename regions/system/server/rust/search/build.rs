fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "../../../../../api/proto/k1s0/system/search/v1/search.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(proto_file).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            proto_file
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[proto_file], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for search proto");
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
