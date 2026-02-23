fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto = "../../../../proto/v1/ratelimit.proto";
    let include = "../../../../proto";
    if !std::path::Path::new(proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            proto
        );
        return Ok(());
    }
    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[proto], &[include])
    {
        Ok(()) => println!("cargo:warning=tonic-build succeeded for ratelimit proto"),
        Err(e) => println!("cargo:warning=tonic-build failed: {}", e),
    }
    Ok(())
}
