use anyhow::Result;
use dialoguer::{Input, MultiSelect};
use std::fs;

use crate::prompt;
use k1s0_core::commands::generate::config_types::{
    build_push_request, generate_dart_types, generate_typescript_types,
};
use k1s0_core::commands::validate::config_schema::ConfigSchemaYaml;

/// 設定スキーマ型ファイル生成コマンドを実行する。
///
/// config-schema.yaml を読み込み、TypeScript / Dart の型定義ファイルを生成する。
/// オプションで config server にスキーマを push する。
///
/// # Errors
///
/// プロンプトの入出力・ファイル操作・スキーマパースに失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- 設定スキーマ型ファイル生成 ---\n");

    // Step 1: config-schema.yaml のパス
    let schema_path: String = Input::with_theme(&prompt::theme())
        .with_prompt("config-schema.yaml のパス")
        .default("config-schema.yaml".to_string())
        .interact_text()?;

    let content = fs::read_to_string(&schema_path)
        .map_err(|e| anyhow::anyhow!("{}: {e}", schema_path))?;
    let schema: ConfigSchemaYaml = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("config-schema.yaml のパースエラー: {e}"))?;

    // Step 2: 対象フレームワーク（複数選択）
    let fw_items = &["React (TypeScript)", "Flutter (Dart)"];
    let Some(fw_selection) = MultiSelect::with_theme(&prompt::theme())
        .with_prompt("対象フレームワーク（スペースで選択、Enter で確定）")
        .items(fw_items)
        .defaults(&[true, true])
        .interact_opt()?
    else {
        return Ok(());
    };

    if fw_selection.is_empty() {
        println!("フレームワークが選択されていません。");
        return Ok(());
    }

    // Step 3: config server に push するか
    let push = match prompt::yes_no_prompt("config server に push しますか？")? {
        Some(v) => v,
        None => return Ok(()),
    };

    let server_url = if push {
        let url: String = Input::with_theme(&prompt::theme())
            .with_prompt("config server URL")
            .default("http://localhost:8080".to_string())
            .interact_text()?;
        Some(url)
    } else {
        None
    };

    // 確認
    println!("\n[確認] 以下の内容で実行します。よろしいですか？");
    println!("  スキーマ: {} ({})", schema_path, schema.service);
    if let Some(ref url) = server_url {
        println!("  push:     {url}");
    }
    if fw_selection.contains(&0) {
        println!("  React  → src/config/__generated__/config-types.ts");
    }
    if fw_selection.contains(&1) {
        println!("  Flutter → lib/config/__generated__/config_types.dart");
    }

    match prompt::confirm_prompt()? {
        prompt::ConfirmResult::Yes => {}
        _ => {
            println!("キャンセルしました。");
            return Ok(());
        }
    }

    // 型定義生成
    println!("\n型定義ファイルを生成中...");
    if fw_selection.contains(&0) {
        let ts = generate_typescript_types(&schema);
        let out_path = "src/config/__generated__/config-types.ts";
        fs::create_dir_all("src/config/__generated__")?;
        fs::write(out_path, ts)?;
        println!("  ✅ {out_path}");
    }
    if fw_selection.contains(&1) {
        let dart = generate_dart_types(&schema);
        let out_path = "lib/config/__generated__/config_types.dart";
        fs::create_dir_all("lib/config/__generated__")?;
        fs::write(out_path, dart)?;
        println!("  ✅ {out_path}");
    }

    // push
    if let Some(ref url) = server_url {
        let token = std::env::var("K1S0_TOKEN").unwrap_or_default();
        let (method, req_url, _, _) =
            build_push_request(&schema, url, &token).map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("  {method} {req_url}");
        println!("  ⚠️  スキーマ push には K1S0_TOKEN 環境変数と HTTP クライアントの実装が必要です");
    }

    println!("\n設定スキーマ型ファイルの生成が完了しました。");
    Ok(())
}
