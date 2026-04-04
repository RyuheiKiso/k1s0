// event-store ビルドスクリプト。
// gRPC サーバーコードの生成または生成済みファイルの利用を行う。
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

    // event-store が使用するサービスとそのパッケージ名の対応表
    // (サブディレクトリパス, パッケージ名) のペアで管理する
    let services: &[(&str, &str)] = &[("eventstore/v1", "k1s0.system.eventstore.v1")];

    // 生成済みファイルが存在するか確認する
    let all_generated = services.iter().all(|(subdir, pkg)| {
        let rs_file = format!("{}/{}/{}.rs", gen_rust_base, subdir, pkg);
        Path::new(&rs_file).exists()
    });

    if all_generated {
        // 生成済みファイルを OUT_DIR にコピーして tonic::include_proto! から参照可能にする。
        // buf/validate.proto が BSR 依存のため Docker ビルドでは protoc コンパイルを
        // スキップし、buf generate で生成済みの .rs ファイルを直接使用する。
        println!(
            "cargo:warning=event-store: using pre-generated proto files from api/proto/gen/rust/"
        );

        for (subdir, pkg) in services {
            // prost-build 生成の .rs ファイルをコピー（メッセージ型定義）
            let src_rs = format!("{}/{}/{}.rs", gen_rust_base, subdir, pkg);
            let dst_rs = out_path.join(format!("{}.rs", pkg));
            fs::copy(&src_rs, &dst_rs).map_err(|e| {
                format!("Failed to copy {} -> {}: {}", src_rs, dst_rs.display(), e)
            })?;

            // tonic-build 生成の .tonic.rs ファイルをコピー（gRPC スタブ）
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
        // tonic-build を試みるが、buf/validate.proto が解決できない場合は警告のみで継続する。
        println!("cargo:warning=event-store: pre-generated proto files not found, falling back to tonic-build");

        let event_store_proto =
            "../../../../../api/proto/k1s0/system/eventstore/v1/event_store.proto";
        let proto_include = "../../../../../api/proto";

        if !Path::new(event_store_proto).exists() {
            println!(
                "cargo:warning=Proto file not found, skipping tonic codegen: {}",
                event_store_proto
            );
            return Ok(());
        }

        match tonic_build::configure()
            .build_server(true)
            .build_client(false)
            .out_dir("src/proto")
            .compile_protos(&[event_store_proto], &[proto_include])
        {
            Ok(()) => {
                println!("cargo:warning=tonic-build succeeded for event-store proto");
            }
            Err(e) => {
                println!(
                    "cargo:warning=tonic-build failed (protoc may not be installed): {}",
                    e
                );
            }
        }
    }

    Ok(())
}
