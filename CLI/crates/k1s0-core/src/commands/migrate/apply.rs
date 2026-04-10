/// マイグレーションの適用・ロールバック。
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use super::types::{DbConnection, Language, MigrateDownConfig, MigrateRange, MigrateUpConfig};

/// ローカル開発環境のデフォルトポート。
const LOCAL_DEV_DEFAULT_PORT: u16 = 5432;

/// マイグレーションを適用する（up）。
///
/// # Errors
///
/// ツールの実行に失敗した場合にエラーを返す。
pub fn execute_migrate_up(config: &MigrateUpConfig) -> Result<()> {
    let conn_str = resolve_connection_string(&config.connection, &config.target.db_name)?;
    let migrations_dir = &config.target.migrations_dir;

    match config.target.language {
        Language::Rust => {
            execute_sqlx_up(migrations_dir, &conn_str, &config.range)?;
        }
        Language::Go => {
            execute_golang_migrate_up(migrations_dir, &conn_str, &config.range)?;
        }
    }

    Ok(())
}

/// マイグレーションをロールバックする（down）。
///
/// # Errors
///
/// ツールの実行に失敗した場合にエラーを返す。
pub fn execute_migrate_down(config: &MigrateDownConfig) -> Result<()> {
    let conn_str = resolve_connection_string(&config.connection, &config.target.db_name)?;
    let migrations_dir = &config.target.migrations_dir;

    match config.target.language {
        Language::Rust => {
            execute_sqlx_down(migrations_dir, &conn_str, &config.range)?;
        }
        Language::Go => {
            execute_golang_migrate_down(migrations_dir, &conn_str, &config.range)?;
        }
    }

    Ok(())
}

/// DB接続文字列を解決する。
///
/// # Errors
///
/// state.json の読み込みに失敗した場合にエラーを返す。
pub fn resolve_connection_string(connection: &DbConnection, db_name: &str) -> Result<String> {
    match connection {
        DbConnection::LocalDev => {
            let port = read_local_dev_port().unwrap_or(LOCAL_DEV_DEFAULT_PORT);
            // CLI-001 修正: デフォルトパスワードを削除し環境変数を必須化する。
            // apply.rs は本番マイグレーションパスにも到達しうるため、
            // フォールバックとして "password" をハードコードすることは危険。
            let dev_password = std::env::var("K1S0_DEV_DB_PASSWORD")
                .context("K1S0_DEV_DB_PASSWORD must be set for local dev migration")?;
            Ok(format!(
                "postgresql://app:{dev_password}@localhost:{port}/{db_name}?sslmode=disable"
            ))
        }
        DbConnection::Custom(url) => Ok(url.clone()),
    }
}

/// `.k1s0-dev/state.json` からローカル開発用のポート番号を読む。
fn read_local_dev_port() -> Option<u16> {
    let state_path = Path::new(".k1s0-dev/state.json");
    if !state_path.exists() {
        return None;
    }
    let content = fs::read_to_string(state_path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    value
        .get("dependencies")
        .and_then(|d| d.get("postgres"))
        .and_then(|p| p.get("port"))
        .and_then(tera::Value::as_u64)
        .and_then(|p| u16::try_from(p).ok())
}

/// sqlx migrate run を実行する。
fn execute_sqlx_up(migrations_dir: &Path, conn_str: &str, range: &MigrateRange) -> Result<()> {
    let dir_str = migrations_dir
        .to_str()
        .context("マイグレーションディレクトリのパスが不正です")?;

    let mut args = vec![
        "migrate",
        "run",
        "--source",
        dir_str,
        "--database-url",
        conn_str,
    ];

    let version_str;
    match range {
        MigrateRange::All => {}
        MigrateRange::UpTo(n) => {
            version_str = n.to_string();
            args.push("--target-version");
            args.push(&version_str);
        }
        MigrateRange::Steps(_) => {
            anyhow::bail!("sqlx migrate run does not support step-based execution");
        }
    }

    println!("実行: sqlx {}", args.join(" "));
    let status = Command::new("sqlx").args(&args).status().context(
        "sqlx コマンドの実行に失敗しました。sqlx-cli がインストールされているか確認してください。",
    )?;

    if !status.success() {
        anyhow::bail!(
            "sqlx migrate run が失敗しました（終了コード: {:?}）",
            status.code()
        );
    }

    Ok(())
}

/// sqlx migrate revert を実行する。
fn execute_sqlx_down(migrations_dir: &Path, conn_str: &str, range: &MigrateRange) -> Result<()> {
    match range {
        MigrateRange::All => execute_sqlx_revert_to_version(migrations_dir, conn_str, 0),
        MigrateRange::UpTo(version) => {
            execute_sqlx_revert_to_version(migrations_dir, conn_str, *version)
        }
        MigrateRange::Steps(steps) => {
            for _ in 0..*steps {
                execute_sqlx_revert_latest(migrations_dir, conn_str)?;
            }
            Ok(())
        }
    }
}

fn execute_sqlx_revert_latest(migrations_dir: &Path, conn_str: &str) -> Result<()> {
    let dir_str = migrations_dir
        .to_str()
        .context("マイグレーションディレクトリのパスが不正です")?;

    let args = vec![
        "migrate",
        "revert",
        "--source",
        dir_str,
        "--database-url",
        conn_str,
    ];

    println!("実行: sqlx {}", args.join(" "));
    let status = Command::new("sqlx").args(&args).status().context(
        "sqlx コマンドの実行に失敗しました。sqlx-cli がインストールされているか確認してください。",
    )?;

    if !status.success() {
        anyhow::bail!(
            "sqlx migrate revert が失敗しました（終了コード: {:?}）",
            status.code()
        );
    }

    Ok(())
}

fn execute_sqlx_revert_to_version(
    migrations_dir: &Path,
    conn_str: &str,
    version: u32,
) -> Result<()> {
    let dir_str = migrations_dir
        .to_str()
        .context("マイグレーションディレクトリのパスが不正です")?;
    let version_str = version.to_string();
    let args = vec![
        "migrate",
        "revert",
        "--source",
        dir_str,
        "--database-url",
        conn_str,
        "--target-version",
        &version_str,
    ];

    println!("実行: sqlx {}", args.join(" "));
    let status = Command::new("sqlx").args(&args).status().context(
        "sqlx コマンドの実行に失敗しました。sqlx-cli がインストールされているか確認してください。",
    )?;

    if !status.success() {
        anyhow::bail!(
            "sqlx migrate revert が失敗しました（終了コード: {:?}）",
            status.code()
        );
    }

    Ok(())
}

/// golang-migrate の up を実行する。
fn execute_golang_migrate_up(
    migrations_dir: &Path,
    conn_str: &str,
    range: &MigrateRange,
) -> Result<()> {
    let dir_str = migrations_dir
        .to_str()
        .context("マイグレーションディレクトリのパスが不正です")?;

    let mut args = vec!["-path", dir_str, "-database", conn_str];
    let value_str;
    match range {
        MigrateRange::All => {
            args.push("up");
        }
        MigrateRange::UpTo(version) => {
            value_str = version.to_string();
            args.push("goto");
            args.push(&value_str);
        }
        MigrateRange::Steps(steps) => {
            value_str = steps.to_string();
            args.push("up");
            args.push(&value_str);
        }
    }

    println!("実行: migrate {}", args.join(" "));
    let status = Command::new("migrate")
        .args(&args)
        .status()
        .context("migrate コマンドの実行に失敗しました。golang-migrate がインストールされているか確認してください。")?;

    if !status.success() {
        anyhow::bail!(
            "migrate up が失敗しました（終了コード: {:?}）",
            status.code()
        );
    }

    Ok(())
}

/// golang-migrate の down を実行する。
fn execute_golang_migrate_down(
    migrations_dir: &Path,
    conn_str: &str,
    range: &MigrateRange,
) -> Result<()> {
    let dir_str = migrations_dir
        .to_str()
        .context("マイグレーションディレクトリのパスが不正です")?;

    let mut args = vec!["-path", dir_str, "-database", conn_str];
    let value_str;
    match range {
        MigrateRange::All => {
            args.push("down");
            args.push("-all");
        }
        MigrateRange::UpTo(version) => {
            value_str = version.to_string();
            args.push("goto");
            args.push(&value_str);
        }
        MigrateRange::Steps(steps) => {
            value_str = steps.to_string();
            args.push("down");
            args.push(&value_str);
        }
    }

    println!("実行: migrate {}", args.join(" "));
    let status = Command::new("migrate")
        .args(&args)
        .status()
        .context("migrate コマンドの実行に失敗しました。golang-migrate がインストールされているか確認してください。")?;

    if !status.success() {
        anyhow::bail!(
            "migrate down が失敗しました（終了コード: {:?}）",
            status.code()
        );
    }

    Ok(())
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_connection_string_local_dev() {
        // MED-007 監査対応: テスト内でenv変数を設定し、CI環境での未設定による失敗を防ぐ
        // SAFETY: テスト専用ファンクションであり、並列テスト実行による競合は #[test] の制約下で許容する
        unsafe {
            std::env::set_var("K1S0_DEV_DB_PASSWORD", "test-dev-password");
        }
        let conn = resolve_connection_string(&DbConnection::LocalDev, "auth_db").unwrap();
        // ポートはstate.jsonが無い場合デフォルトの5432
        assert!(conn.contains("localhost"));
        assert!(conn.contains("auth_db"));
        assert!(conn.contains("sslmode=disable"));
    }

    #[test]
    fn test_resolve_connection_string_custom() {
        let custom_url = "postgresql://user:pass@remote:5433/mydb".to_string();
        let conn = resolve_connection_string(&DbConnection::Custom(custom_url.clone()), "ignored")
            .unwrap();
        assert_eq!(conn, custom_url);
    }
}
