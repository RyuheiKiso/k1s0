fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Proto directory for regions/system/proto/v1/ (uses google.protobuf.Timestamp)
    // tenant.proto imports "v1/common.proto" so include path must be the parent of v1/
    let tenant_proto = "../../../proto/v1/tenant.proto";
    let common_proto = "../../../proto/v1/common.proto";
    let system_proto_include = "../../../proto";

    // Proto directory for api/proto/ (uses k1s0.system.common.v1.Timestamp)
    let featureflag_proto =
        "../../../../../api/proto/k1s0/system/featureflag/v1/featureflag.proto";
    let config_proto = "../../../../../api/proto/k1s0/system/config/v1/config.proto";
    let api_common_proto = "../../../../../api/proto/k1s0/system/common/v1/types.proto";
    let api_proto_include = "../../../../../api/proto";

    // Compile tenant + system common proto together so that Pagination type is available
    let tenant_exists = std::path::Path::new(tenant_proto).exists();
    let common_exists = std::path::Path::new(common_proto).exists();

    if tenant_exists && common_exists {
        match tonic_build::configure()
            .build_server(false)
            .build_client(true)
            .compile_protos(&[tenant_proto, common_proto], &[system_proto_include])
        {
            Ok(()) => {
                println!("cargo:warning=tonic-build succeeded for tenant proto");
            }
            Err(e) => {
                println!(
                    "cargo:warning=tonic-build failed for tenant proto (protoc may not be installed): {}",
                    e
                );
            }
        }
    } else {
        println!(
            "cargo:warning=Tenant proto files not found, skipping codegen"
        );
    }

    // Compile featureflag, config and api common protos together
    let api_protos_exist = std::path::Path::new(featureflag_proto).exists()
        && std::path::Path::new(config_proto).exists()
        && std::path::Path::new(api_common_proto).exists();

    if api_protos_exist {
        match tonic_build::configure()
            .build_server(false)
            .build_client(true)
            .compile_protos(
                &[featureflag_proto, config_proto, api_common_proto],
                &[api_proto_include],
            )
        {
            Ok(()) => {
                println!("cargo:warning=tonic-build succeeded for featureflag/config protos");
            }
            Err(e) => {
                println!(
                    "cargo:warning=tonic-build failed for featureflag/config protos (protoc may not be installed): {}",
                    e
                );
            }
        }
    } else {
        println!(
            "cargo:warning=API proto files not found, skipping codegen: featureflag/config"
        );
    }

    Ok(())
}
