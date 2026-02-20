fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Proto codegen will be enabled when proto files are ready
    // tonic_build::configure()
    //     .build_server(true)
    //     .build_client(true)
    //     .compile(
    //         &["../../../../proto/v1/saga.proto"],
    //         &["../../../../proto/v1"],
    //     )?;
    Ok(())
}
