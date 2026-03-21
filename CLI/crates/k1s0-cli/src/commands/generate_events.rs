use anyhow::Result;
use dialoguer::Input;
use std::path::PathBuf;

use crate::prompt;
use k1s0_core::commands::generate_events::{
    execute_event_codegen, format_generation_summary, parse_events_yaml,
};

/// イベントコード生成コマンドを実行する。
///
/// events.yaml を読み込み、Producer / Consumer / Proto / Outbox / Schema Registry
/// の各コードを一括生成する。
///
/// # Errors
///
/// プロンプトの入出力・ファイル操作・バリデーションに失敗した場合にエラーを返す。
pub fn run() -> Result<()> {
    println!("\n--- イベントコード生成 ---\n");

    // Step 1: events.yaml のパス
    let events_path: String = Input::with_theme(&prompt::theme())
        .with_prompt("events.yaml のパス")
        .default("events.yaml".to_string())
        .interact_text()?;

    // Step 2: パース & バリデーション
    let config = match parse_events_yaml(&events_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\nevents.yaml のバリデーションエラー:\n  {e}");
            return Ok(());
        }
    };

    // Step 3: 生成サマリー表示
    println!("\n[生成サマリー]");
    println!("{}", format_generation_summary(&config));

    // Step 4: 確認
    if prompt::confirm_prompt()? != prompt::ConfirmResult::Yes {
        println!("キャンセルしました。");
        return Ok(());
    }

    // Step 5: テンプレートディレクトリの解決
    let template_base = resolve_template_dir()?;

    // Step 6: 出力先ディレクトリの決定 (events.yaml のあるディレクトリ)
    let events_path = PathBuf::from(&events_path);
    let output_dir = match events_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    };

    // Step 7: コード生成実行
    println!("\nコードを生成中...");
    let generated = execute_event_codegen(&config, &output_dir, &template_base)?;

    // Step 8: 結果表示
    println!("\n生成されたファイル:");
    for path in &generated {
        println!("  ✅ {}", path.display());
    }
    println!(
        "\nイベントコードの生成が完了しました。({} ファイル)",
        generated.len()
    );

    Ok(())
}

/// テンプレートディレクトリを解決する。
/// CLI バイナリの実行ディレクトリまたは Cargo マニフェストディレクトリから探す。
fn resolve_template_dir() -> Result<PathBuf> {
    // CARGO_MANIFEST_DIR (開発中) → テンプレートはクレートルートの templates/events/
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(manifest_dir).join("templates").join("events");
        if path.exists() {
            return Ok(path);
        }
    }

    // 実行ファイルの隣の templates/events/
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let path = exe_dir.join("templates").join("events");
            if path.exists() {
                return Ok(path);
            }
        }
    }

    // カレントディレクトリの templates/events/
    let path = PathBuf::from("templates/events");
    if path.exists() {
        return Ok(path);
    }

    // ワークスペースルートから実行していない可能性を含め、具体的な解決策を案内する
    anyhow::bail!(
        "テンプレートディレクトリが見つかりません。\n\
         以下を確認してください:\n\
         - k1s0 プロジェクトのルートディレクトリで実行していること\n\
         - CLI/crates/k1s0-cli/templates/events/ ディレクトリが存在すること"
    );
}
