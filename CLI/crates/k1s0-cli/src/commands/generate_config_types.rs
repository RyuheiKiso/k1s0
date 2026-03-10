use anyhow::Result;
use dialoguer::Input;
use std::fs;
use std::path::PathBuf;

use crate::prompt;
use k1s0_core::commands::generate::config_types::{
    build_push_request, write_generated_types_from_file,
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

    let content =
        fs::read_to_string(&schema_path).map_err(|e| anyhow::anyhow!("{schema_path}: {e}"))?;
    let schema: ConfigSchemaYaml = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("config-schema.yaml のパースエラー: {e}"))?;

    // Step 2: 生成ターゲット
    let Some(target_idx) = prompt::select_prompt(
        "生成ターゲットを選択してください",
        &["TypeScript", "Dart", "両方"],
    )?
    else {
        return Ok(());
    };

    let targets: Vec<&str> = match target_idx {
        0 => vec!["typescript"],
        1 => vec!["dart"],
        2 => vec!["typescript", "dart"],
        _ => unreachable!(),
    };

    // Step 3: 出力先ディレクトリ
    let default_output_dir = match target_idx {
        0 => "src/config/__generated__",
        1 => "lib/config/__generated__",
        _ => "generated/config",
    };
    let output_dir: String = Input::with_theme(&prompt::theme())
        .with_prompt("生成先ディレクトリ")
        .default(default_output_dir.to_string())
        .interact_text()?;

    // Step 4: config server に push するか
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
    println!("  出力先:   {}", output_dir);
    for target in &targets {
        match *target {
            "typescript" => println!(
                "  TypeScript → {}",
                PathBuf::from(&output_dir).join("config-types.ts").display()
            ),
            "dart" => println!(
                "  Dart       → {}",
                PathBuf::from(&output_dir).join("config_types.dart").display()
            ),
            _ => {}
        }
    }

    if prompt::confirm_prompt()? == prompt::ConfirmResult::Yes {
    } else {
        println!("キャンセルしました。");
        return Ok(());
    }

    // 型定義生成
    println!("\n型定義ファイルを生成中...");
    let generated = write_generated_types_from_file(
        std::path::Path::new(&schema_path),
        std::path::Path::new(&output_dir),
        &targets,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    for path in &generated {
        println!("  ✅ {}", path.display());
    }

    // push
    if let Some(ref url) = server_url {
        let token = std::env::var("K1S0_TOKEN").unwrap_or_default();
        let (method, req_url, _, _) =
            build_push_request(&schema, url, &token).map_err(|e| anyhow::anyhow!("{e}"))?;
        println!("  {method} {req_url}");
        println!(
            "  ⚠️  スキーマ push には K1S0_TOKEN 環境変数と HTTP クライアントの実装が必要です"
        );
    }

    println!("\n設定スキーマ型ファイルの生成が完了しました。");
    Ok(())
}
