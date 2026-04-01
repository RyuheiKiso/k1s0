// graphql-gateway ビルドスクリプト。
// gRPC クライアントコードの生成または生成済みファイルの利用を行う。
//
// 戦略:
//   api/proto/gen/rust/ に buf generate で生成済みの .rs ファイルが存在する場合は
//   それを OUT_DIR にコピーして使用する（buf/validate.proto が BSR 依存のため
//   protoc 単体では解決できない問題を回避）。
//   生成済みファイルが存在しない場合のみ tonic-build によるオンデマンド生成を試みる。
//
// このアプローチにより Docker ビルド時に buf/validate.proto が不要となる。

use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // OUT_DIR は cargo が設定するビルド出力ディレクトリ
    let out_dir = std::env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    // api/proto/gen/rust 配下の生成済みファイルパス（リポジトリルートからの相対パス）
    let gen_rust_base = "../../../../../api/proto/gen/rust/k1s0/system";

    // graphql-gateway が使用するサービスとそのパッケージ名の対応表
    // (サブディレクトリパス, パッケージ名) のペアで管理する
    let services: &[(&str, &str)] = &[
        ("common/v1", "k1s0.system.common.v1"),
        ("tenant/v1", "k1s0.system.tenant.v1"),
        ("featureflag/v1", "k1s0.system.featureflag.v1"),
        ("config/v1", "k1s0.system.config.v1"),
        ("navigation/v1", "k1s0.system.navigation.v1"),
        ("servicecatalog/v1", "k1s0.system.servicecatalog.v1"),
        ("auth/v1", "k1s0.system.auth.v1"),
        ("session/v1", "k1s0.system.session.v1"),
        ("vault/v1", "k1s0.system.vault.v1"),
        ("scheduler/v1", "k1s0.system.scheduler.v1"),
        ("notification/v1", "k1s0.system.notification.v1"),
        ("workflow/v1", "k1s0.system.workflow.v1"),
    ];

    // 全ての生成済みファイルが揃っているか確認する
    // 1つでも欠けている場合はフォールバックとして tonic-build を試みる
    let all_generated = services.iter().all(|(subdir, pkg)| {
        let rs_file = format!("{}/{}/{}.rs", gen_rust_base, subdir, pkg);
        Path::new(&rs_file).exists()
    });

    if all_generated {
        // 生成済みファイルを OUT_DIR にコピーして tonic::include_proto! から参照可能にする。
        // buf/validate.proto が BSR 依存のため Docker ビルドでは protoc コンパイルを
        // スキップし、buf generate で生成済みの .rs ファイルを直接使用する。
        println!("cargo:warning=graphql-gateway: using pre-generated proto files from api/proto/gen/rust/");

        for (subdir, pkg) in services {
            // prost-build 生成の .rs ファイルをコピー（メッセージ型定義）
            let src_rs = format!("{}/{}/{}.rs", gen_rust_base, subdir, pkg);
            let dst_rs = out_path.join(format!("{}.rs", pkg));
            fs::copy(&src_rs, &dst_rs).map_err(|e| {
                format!("Failed to copy {} -> {}: {}", src_rs, dst_rs.display(), e)
            })?;

            // tonic-build 生成の .tonic.rs ファイルをコピー（gRPC スタブ）
            // common/v1 は gRPC サービスを持たないためスキップ
            let src_tonic = format!("{}/{}/{}.tonic.rs", gen_rust_base, subdir, pkg);
            if Path::new(&src_tonic).exists() {
                // tonic::include_proto! は {pkg}.rs を読み込んだ後に
                // 同ファイル内で include!("{pkg}.tonic.rs") を参照する設計のため、
                // .tonic.rs も同じ OUT_DIR に配置する必要がある
                let dst_tonic = out_path.join(format!("{}.tonic.rs", pkg));
                fs::copy(&src_tonic, &dst_tonic).map_err(|e| {
                    format!(
                        "Failed to copy {} -> {}: {}",
                        src_tonic,
                        dst_tonic.display(),
                        e
                    )
                })?;
            }
        }
    } else {
        // 生成済みファイルが存在しない場合のフォールバック。
        // ローカル開発環境で protoc + buf/validate が利用可能な場合にのみ成功する。
        // Docker ビルドでは api/proto/gen/rust/ の生成済みファイルが必須。
        println!("cargo:warning=graphql-gateway: pre-generated files not found, falling back to tonic-build");
        println!("cargo:warning=NOTE: Docker build requires pre-generated files in api/proto/gen/rust/");

        let tenant_proto = "../../../../../api/proto/k1s0/system/tenant/v1/tenant.proto";
        let featureflag_proto =
            "../../../../../api/proto/k1s0/system/featureflag/v1/featureflag.proto";
        let config_proto = "../../../../../api/proto/k1s0/system/config/v1/config.proto";
        let navigation_proto =
            "../../../../../api/proto/k1s0/system/navigation/v1/navigation.proto";
        let service_catalog_proto =
            "../../../../../api/proto/k1s0/system/servicecatalog/v1/service_catalog.proto";
        let api_common_proto = "../../../../../api/proto/k1s0/system/common/v1/types.proto";
        let auth_proto = "../../../../../api/proto/k1s0/system/auth/v1/auth.proto";
        let session_proto = "../../../../../api/proto/k1s0/system/session/v1/session.proto";
        let vault_proto = "../../../../../api/proto/k1s0/system/vault/v1/vault.proto";
        let scheduler_proto =
            "../../../../../api/proto/k1s0/system/scheduler/v1/scheduler.proto";
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
            // 注意: buf/validate.proto が BSR 依存のため、protoc + libprotobuf-dev のみでは
            // コンパイルに失敗する可能性がある。その場合は api/proto/gen/rust/ を参照のこと。
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
                        "cargo:warning=tonic-build failed (buf/validate.proto requires BSR access): {}",
                        e
                    );
                    println!("cargo:warning=Run 'buf generate' in api/proto/ to regenerate api/proto/gen/rust/");
                }
            }
        } else {
            println!(
                "cargo:warning=API proto files not found, skipping codegen for graphql-gateway"
            );
        }
    }

    Ok(())
}
