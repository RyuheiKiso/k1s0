//! k1s0 サービス共通の proto コード生成ユーティリティ
//!
//! 各サービスの build.rs で同一パターンの proto コンパイルロジックを
//! 共通化し、横展開事故を防止する。

/// サービス proto とイベント proto を単一呼び出しでコンパイルする
///
/// 各サービスの build.rs から呼び出され、サービス定義とイベント定義の
/// proto ファイルを tonic-build で一括コンパイルする。
/// 存在しない proto ファイルはスキップし、警告を出力する。
///
/// # Arguments
/// * `service_name` - サービス名（例: "payment", "order", "inventory"）
/// * `proto_root` - proto ルートディレクトリへのパス（import 解決用）
/// * `out_dir` - 生成コードの出力先ディレクトリ
pub fn compile_service_protos(
    service_name: &str,
    proto_root: &str,
    out_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // サービス定義用 proto ファイルパス
    let service_proto = format!("{proto_root}/k1s0/service/{service_name}/v1/{service_name}.proto");
    // イベント定義用 proto ファイルパス
    let event_proto =
        format!("{proto_root}/k1s0/event/service/{service_name}/v1/{service_name}_events.proto");

    // コンパイル対象の proto ファイルを収集（存在するもののみ）
    let mut protos = Vec::new();
    for path in [&service_proto, &event_proto] {
        if std::path::Path::new(path).exists() {
            protos.push(path.clone());
        } else {
            println!("cargo:warning=Proto file not found, skipping: {path}");
        }
    }

    // proto ファイルが見つからない場合はコード生成をスキップ
    if protos.is_empty() {
        println!("cargo:warning=No proto files found, skipping tonic codegen");
        return Ok(());
    }

    // 全 proto を単一呼び出しでコンパイル（共通型の上書きを防止）
    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir(out_dir)
        .compile_protos(&protos, &[proto_root])
    {
        Ok(()) => println!("cargo:warning=tonic-build succeeded for {service_name}"),
        Err(e) => println!(
            "cargo:warning=tonic-build failed for {service_name} (protoc may not be installed): {e}"
        ),
    }

    Ok(())
}
