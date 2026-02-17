mod commands;
mod config;
mod prompt;
mod template;

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select};

/// メインメニューの選択肢
const MENU_ITEMS: &[&str] = &[
    "プロジェクト初期化",
    "ひな形生成",
    "ビルド",
    "テスト実行",
    "デプロイ",
    "終了",
];

fn main() {
    // Ctrl+C でパニックせずに終了するためのハンドラ
    ctrlc_handler();

    // D-09: 起動時に設定ファイルを読み込む
    let cli_config = match config::load_config("k1s0.yaml") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("設定ファイルの読み込みに失敗しました: {}", e);
            eprintln!("デフォルト設定を使用します。");
            config::CliConfig::default()
        }
    };

    loop {
        match show_main_menu(&cli_config) {
            Ok(should_exit) => {
                if should_exit {
                    println!("終了します。");
                    break;
                }
            }
            Err(e) => {
                let msg = format!("{}", e);
                if msg.contains("interrupted") {
                    // メインメニューで Ctrl+C → 終了
                    println!("\n終了します。");
                    break;
                }
                // その他のエラーはメインメニューに戻る
                eprintln!("エラーが発生しました: {}", e);
                continue;
            }
        }
    }
}

/// Ctrl+C のグローバルハンドラを設定する。
/// dialoguer が Ctrl+C を処理するため、ここでは最低限のフォールバックのみ。
fn ctrlc_handler() {
    let _ = ctrlc::set_handler(|| {
        // dialoguer の interact_opt が None を返すので、
        // ここでは何もしない（二重終了を防ぐ）。
    });
}

/// メインメニューを表示し、選択されたコマンドを実行する。
/// 終了が選択された場合は Ok(true) を返す。
fn show_main_menu(_cli_config: &config::CliConfig) -> Result<bool> {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("操作を選択してください")
        .items(MENU_ITEMS)
        .default(0)
        .interact_opt()?;

    match selection {
        // Ctrl+C が押された場合 (None) → 終了
        None => Ok(true),
        Some(index) => {
            match index {
                0 => {
                    if let Err(e) = commands::init::run() {
                        eprintln!("初期化エラー: {}", e);
                    }
                }
                1 => {
                    if let Err(e) = commands::generate::run() {
                        eprintln!("ひな形生成エラー: {}", e);
                    }
                }
                2 => {
                    if let Err(e) = commands::build::run() {
                        eprintln!("ビルドエラー: {}", e);
                    }
                }
                3 => {
                    if let Err(e) = commands::test_cmd::run() {
                        eprintln!("テスト実行エラー: {}", e);
                    }
                }
                4 => {
                    if let Err(e) = commands::deploy::run() {
                        eprintln!("デプロイエラー: {}", e);
                    }
                }
                5 => return Ok(true),
                _ => unreachable!(),
            }
            Ok(false)
        }
    }
}
