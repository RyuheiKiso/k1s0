/// マイグレーションツールの確認・インストール。
use std::process::Command;

use anyhow::{Context, Result};

use super::types::Language;

/// マイグレーションツールがインストールされているか確認する。
pub fn check_tool_installed(language: &Language) -> bool {
    match language {
        Language::Rust => Command::new("sqlx")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
        Language::Go => Command::new("migrate")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false),
    }
}

/// マイグレーションツールをインストールする。
///
/// # Errors
///
/// インストールに失敗した場合にエラーを返す。
pub fn install_tool(language: &Language) -> Result<()> {
    match language {
        Language::Rust => {
            println!("sqlx-cli をインストールしています...");
            let status = Command::new("cargo")
                .args([
                    "install",
                    "sqlx-cli",
                    "--no-default-features",
                    "--features",
                    "postgres",
                ])
                .status()
                .context("cargo install の実行に失敗しました")?;
            if !status.success() {
                anyhow::bail!("sqlx-cli のインストールに失敗しました");
            }
            println!("sqlx-cli のインストールが完了しました。");
        }
        Language::Go => {
            println!("golang-migrate をインストールしています...");
            let status = Command::new("go")
                .args([
                    "install",
                    "-tags",
                    "postgres",
                    "github.com/golang-migrate/migrate/v4/cmd/migrate@latest",
                ])
                .status()
                .context("go install の実行に失敗しました")?;
            if !status.success() {
                anyhow::bail!("golang-migrate のインストールに失敗しました");
            }
            println!("golang-migrate のインストールが完了しました。");
        }
    }
    Ok(())
}

/// ツール名を返す。
pub fn tool_name(language: &Language) -> &'static str {
    match language {
        Language::Rust => "sqlx-cli",
        Language::Go => "golang-migrate",
    }
}
