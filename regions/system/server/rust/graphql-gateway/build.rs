// graphql-gateway ビルドスクリプト。
// proto ファイルから gRPC クライアントコードを生成する。
// tonic-build v0.12 の configure() API は v0.14 で削除されたため、
// tonic_build::Builder::new() を使用して同等の設定を行う。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // すべての proto ファイルを api/proto/k1s0/system/ から参照する（統一された canonical 位置）
    let tenant_proto = "../../../../../api/proto/k1s0/system/tenant/v1/tenant.proto";
    let featureflag_proto = "../../../../../api/proto/k1s0/system/featureflag/v1/featureflag.proto";
    let config_proto = "../../../../../api/proto/k1s0/system/config/v1/config.proto";
    let navigation_proto = "../../../../../api/proto/k1s0/system/navigation/v1/navigation.proto";
    let service_catalog_proto =
        "../../../../../api/proto/k1s0/system/servicecatalog/v1/service_catalog.proto";
    let api_common_proto = "../../../../../api/proto/k1s0/system/common/v1/types.proto";
    let auth_proto = "../../../../../api/proto/k1s0/system/auth/v1/auth.proto";
    let session_proto = "../../../../../api/proto/k1s0/system/session/v1/session.proto";
    let vault_proto = "../../../../../api/proto/k1s0/system/vault/v1/vault.proto";
    let scheduler_proto = "../../../../../api/proto/k1s0/system/scheduler/v1/scheduler.proto";
    let notification_proto =
        "../../../../../api/proto/k1s0/system/notification/v1/notification.proto";
    let workflow_proto = "../../../../../api/proto/k1s0/system/workflow/v1/workflow.proto";
    let api_proto_include = "../../../../../api/proto";

    let protos_exist = std::path::Path::new(tenant_proto).exists()
        && std::path::Path::new(featureflag_proto).exists()
        && std::path::Path::new(config_proto).exists()
        && std::path::Path::new(navigation_proto).exists()
        && std::path::Path::new(service_catalog_proto).exists()
        && std::path::Path::new(api_common_proto).exists()
        && std::path::Path::new(auth_proto).exists()
        && std::path::Path::new(session_proto).exists()
        && std::path::Path::new(vault_proto).exists()
        && std::path::Path::new(scheduler_proto).exists()
        && std::path::Path::new(notification_proto).exists()
        && std::path::Path::new(workflow_proto).exists();

    if protos_exist {
        // tonic-build v0.14 の Builder API を使用してクライアントコードのみを生成する。
        // build_server(false) でサーバースタブを生成しない（gateway はクライアントとしてのみ動作）。
        match tonic_build::configure()
            .build_server(false)
            .build_client(true)
            .compile_protos(
                &[
                    tenant_proto,
                    featureflag_proto,
                    config_proto,
                    navigation_proto,
                    service_catalog_proto,
                    api_common_proto,
                    auth_proto,
                    session_proto,
                    vault_proto,
                    scheduler_proto,
                    notification_proto,
                    workflow_proto,
                ],
                &[api_proto_include],
            ) {
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
        println!("cargo:warning=API proto files not found, skipping codegen for graphql-gateway");
    }

    Ok(())
}
