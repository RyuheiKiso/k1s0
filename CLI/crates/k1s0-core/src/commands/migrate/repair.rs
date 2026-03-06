/// マイグレーションの修復。
use std::process::Command;

use anyhow::{Context, Result};

use super::types::{DbConnection, Language, MigrateTarget, RepairOperation};
use super::apply::resolve_connection_string;

/// マイグレーション修復を実行する。
///
/// # Errors
///
/// ツールの実行に失敗した場合にエラーを返す。
pub fn execute_repair(
    target: &MigrateTarget,
    operation: &RepairOperation,
    connection: &DbConnection,
) -> Result<()> {
    let conn_str = resolve_connection_string(connection, &target.db_name)?;

    match target.language {
        Language::Rust => execute_sqlx_repair(target, operation, &conn_str),
        Language::Go => execute_golang_migrate_repair(target, operation, &conn_str),
    }
}

/// sqlx での修復操作。
///
/// sqlx-cli にはダーティクリアやバージョン強制設定の直接コマンドがないため、
/// `_sqlx_migrations` テーブルを直接操作する SQL を実行する。
fn execute_sqlx_repair(
    target: &MigrateTarget,
    operation: &RepairOperation,
    conn_str: &str,
) -> Result<()> {
    let sql = match operation {
        RepairOperation::ClearDirty => {
            "UPDATE _sqlx_migrations SET success = true WHERE success = false;".to_string()
        }
        RepairOperation::ForceVersion(version) => {
            format!(
                "DELETE FROM _sqlx_migrations WHERE version > {};",
                version
            )
        }
    };

    println!("修復SQL: {}", sql);
    println!("対象: {} ({})", target.service_name, conn_str);

    let dir_str = target
        .migrations_dir
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or(".");

    let status = Command::new("sqlx")
        .args(["database", "run", "--database-url", conn_str, "-e", &sql])
        .current_dir(dir_str)
        .status()
        .context("sqlx コマンドの実行に失敗しました")?;

    if !status.success() {
        anyhow::bail!("修復操作が失敗しました（終了コード: {:?}）", status.code());
    }

    println!("修復が完了しました。");
    Ok(())
}

/// golang-migrate での修復操作。
fn execute_golang_migrate_repair(
    target: &MigrateTarget,
    operation: &RepairOperation,
    conn_str: &str,
) -> Result<()> {
    let dir_str = target
        .migrations_dir
        .to_str()
        .context("マイグレーションディレクトリのパスが不正です")?;

    match operation {
        RepairOperation::ClearDirty => {
            println!("ダーティフラグをクリアします: {}", target.service_name);
            let status = Command::new("migrate")
                .args(["-path", dir_str, "-database", conn_str, "force", "0"])
                .status()
                .context("migrate コマンドの実行に失敗しました")?;

            if !status.success() {
                anyhow::bail!(
                    "ダーティクリアが失敗しました（終了コード: {:?}）",
                    status.code()
                );
            }
        }
        RepairOperation::ForceVersion(version) => {
            let version_str = version.to_string();
            println!(
                "バージョンを {} に強制設定します: {}",
                version, target.service_name
            );
            let status = Command::new("migrate")
                .args([
                    "-path",
                    dir_str,
                    "-database",
                    conn_str,
                    "force",
                    &version_str,
                ])
                .status()
                .context("migrate コマンドの実行に失敗しました")?;

            if !status.success() {
                anyhow::bail!(
                    "バージョン強制設定が失敗しました（終了コード: {:?}）",
                    status.code()
                );
            }
        }
    }

    println!("修復が完了しました。");
    Ok(())
}
