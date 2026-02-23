fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vault_proto = "../../../../../api/proto/k1s0/system/vault/v1/vault.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(vault_proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            vault_proto
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[vault_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for vault proto");
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
