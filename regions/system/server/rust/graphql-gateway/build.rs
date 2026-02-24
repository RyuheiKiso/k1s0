fn main() -> Result<(), Box<dyn std::error::Error>> {
    // すべての proto ファイルを api/proto/k1s0/system/ から参照する（統一された canonical 位置）
    let tenant_proto = "../../../../../api/proto/k1s0/system/tenant/v1/tenant.proto";
    let featureflag_proto =
        "../../../../../api/proto/k1s0/system/featureflag/v1/featureflag.proto";
    let config_proto = "../../../../../api/proto/k1s0/system/config/v1/config.proto";
    let api_common_proto = "../../../../../api/proto/k1s0/system/common/v1/types.proto";
    let api_proto_include = "../../../../../api/proto";

    let protos_exist = std::path::Path::new(tenant_proto).exists()
        && std::path::Path::new(featureflag_proto).exists()
        && std::path::Path::new(config_proto).exists()
        && std::path::Path::new(api_common_proto).exists();

    if protos_exist {
        match tonic_build::configure()
            .build_server(false)
            .build_client(true)
            .compile_protos(
                &[tenant_proto, featureflag_proto, config_proto, api_common_proto],
                &[api_proto_include],
            )
        {
            Ok(()) => {
                println!("cargo:warning=tonic-build succeeded for graphql-gateway protos");
            }
            Err(e) => {
                println!(
                    "cargo:warning=tonic-build failed for graphql-gateway protos (protoc may not be installed): {}",
                    e
                );
            }
        }
    } else {
        println!(
            "cargo:warning=API proto files not found, skipping codegen for graphql-gateway"
        );
    }

    Ok(())
}
