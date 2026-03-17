/// マイグレーション実行モジュール。
///
/// dev up 時に各サービスの migrations/ ディレクトリを検出し、
/// `migrate apply`（sqlx-cli / golang-migrate）経由でマイグレーションを実行する。
///
/// 設計書: docs/cli/dev/ローカル開発環境設計.md
///   - Rust サービス → sqlx migrate run
///   - Go サービス   → migrate -path ... up
use anyhow::Result;
use std::path::Path;

use crate::commands::migrate::apply::execute_migrate_up;
use crate::commands::migrate::types::{
    DbConnection, Language, MigrateRange, MigrateTarget, MigrateUpConfig,
};

use super::types::PortAssignments;

/// dev up 時のマイグレーションを実行する。
///
/// 各サービスの migrations/ ディレクトリを検出し、
/// 言語に応じた正規のマイグレーションツール（sqlx-cli / golang-migrate）で
/// マイグレーションを適用する。ポート情報から接続文字列を構築するため、
/// state.json の保存前でも正しいポートで接続できる。
///
/// # Errors
///
/// マイグレーションツールの実行に失敗した場合にエラーを返す。
pub fn run_dev_migrations(service_paths: &[String], ports: &PortAssignments) -> Result<()> {
    for service_path in service_paths {
        let path = Path::new(service_path);
        let migrations_dir = path.join("migrations");

        if !migrations_dir.is_dir() {
            continue;
        }

        let service_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        println!("  マイグレーション実行中: {service_name}");

        let language = detect_service_language(path);
        let db_name = detect_db_name(path).unwrap_or_else(|| format!("{service_name}_db"));

        // state.json はまだ保存されていないため、LocalDev ではなく
        // ポート情報から直接接続文字列を構築する
        let conn_url = format!(
            "postgresql://app:password@localhost:{}/{db_name}?sslmode=disable",
            ports.postgres
        );

        let config = MigrateUpConfig {
            target: MigrateTarget {
                service_name: service_name.to_string(),
                tier: detect_tier(path),
                language,
                migrations_dir,
                db_name,
            },
            range: MigrateRange::All,
            connection: DbConnection::Custom(conn_url),
        };

        execute_migrate_up(&config)?;
    }

    Ok(())
}

/// サービスディレクトリから言語を検出する。
/// Cargo.toml があれば Rust、go.mod があれば Go、デフォルトは Rust。
fn detect_service_language(path: &Path) -> Language {
    if path.join("go.mod").exists() {
        Language::Go
    } else {
        // Cargo.toml の有無に関わらず Rust をデフォルトとする
        Language::Rust
    }
}

/// config/config.yaml からデータベース名を検出する。
fn detect_db_name(path: &Path) -> Option<String> {
    let config_path = path.join("config").join("config.yaml");
    let content = std::fs::read_to_string(config_path).ok()?;
    let config: serde_yaml::Value = serde_yaml::from_str(&content).ok()?;
    config
        .get("database")?
        .get("name")?
        .as_str()
        .map(String::from)
}

/// パスからティアを抽出する。
fn detect_tier(path: &Path) -> String {
    let path_str = path.to_string_lossy().replace('\\', "/");
    let parts: Vec<&str> = path_str.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "regions" && i + 1 < parts.len() {
            let tier = parts[i + 1];
            if tier == "system" || tier == "business" || tier == "service" {
                return tier.to_string();
            }
        }
    }
    "unknown".to_string()
}

/// 指定ディレクトリにマイグレーションファイルが存在するか確認する。
pub fn has_migrations(service_path: &Path) -> bool {
    let migrations_dir = service_path.join("migrations");
    if !migrations_dir.is_dir() {
        return false;
    }
    std::fs::read_dir(&migrations_dir)
        .map(|entries| {
            entries
                .filter_map(std::result::Result::ok)
                .any(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("sql"))
        })
        .unwrap_or(false)
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// migrations/ がないサービスはスキップされる。
    #[test]
    fn test_has_migrations_no_dir() {
        let tmp = TempDir::new().unwrap();
        assert!(!has_migrations(tmp.path()));
    }

    /// migrations/ に SQL ファイルがある場合は true を返す。
    #[test]
    fn test_has_migrations_with_sql() {
        let tmp = TempDir::new().unwrap();
        let migrations_dir = tmp.path().join("migrations");
        std::fs::create_dir_all(&migrations_dir).unwrap();
        std::fs::write(
            migrations_dir.join("001_init.sql"),
            "CREATE TABLE test (id INT);",
        )
        .unwrap();

        assert!(has_migrations(tmp.path()));
    }

    /// migrations/ に SQL 以外のファイルしかない場合は false を返す。
    #[test]
    fn test_has_migrations_no_sql() {
        let tmp = TempDir::new().unwrap();
        let migrations_dir = tmp.path().join("migrations");
        std::fs::create_dir_all(&migrations_dir).unwrap();
        std::fs::write(migrations_dir.join("README.md"), "# Migrations").unwrap();

        assert!(!has_migrations(tmp.path()));
    }

    /// Cargo.toml があるディレクトリは Rust として検出する。
    #[test]
    fn test_detect_service_language_rust() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]").unwrap();
        assert_eq!(detect_service_language(tmp.path()), Language::Rust);
    }

    /// go.mod があるディレクトリは Go として検出する。
    #[test]
    fn test_detect_service_language_go() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("go.mod"), "module test").unwrap();
        assert_eq!(detect_service_language(tmp.path()), Language::Go);
    }

    /// 言語判定ファイルがない場合は Rust をデフォルトとする。
    #[test]
    fn test_detect_service_language_default() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_service_language(tmp.path()), Language::Rust);
    }

    /// config/config.yaml からデータベース名を検出する。
    #[test]
    fn test_detect_db_name() {
        let tmp = TempDir::new().unwrap();
        let config_dir = tmp.path().join("config");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.yaml"),
            "database:\n  name: order_db\n",
        )
        .unwrap();

        assert_eq!(detect_db_name(tmp.path()), Some("order_db".to_string()));
    }

    /// config.yaml がない場合は None を返す。
    #[test]
    fn test_detect_db_name_missing() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(detect_db_name(tmp.path()), None);
    }
}
