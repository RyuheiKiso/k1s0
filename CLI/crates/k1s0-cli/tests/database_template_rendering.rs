/// データベーステンプレートのレンダリング統合テスト。
///
/// 実際の CLI/templates/database/{postgresql,mysql,sqlite}/ テンプレートファイルを使用し、
/// テンプレートエンジンでレンダリングした結果が仕様書と一致することを検証する。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

// =========================================================================
// ヘルパー関数
// =========================================================================

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_database(db_type: &str) -> (TempDir, Vec<String>) {
    let tpl_dir = template_dir();
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new("main-db", "system", db_type, "database").build();

    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

    let names: Vec<String> = generated
        .iter()
        .map(|p| {
            p.strip_prefix(&output_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    (tmp, names)
}

fn read_output(tmp: &TempDir, path: &str) -> String {
    fs::read_to_string(tmp.path().join("output").join(path)).unwrap()
}

// =========================================================================
// PostgreSQL
// =========================================================================

#[test]
fn test_database_postgresql_file_list() {
    let (_, names) = render_database("postgresql");

    assert!(
        names.iter().any(|n| n == "001_init.up.sql"),
        "001_init.up.sql missing"
    );
    assert!(
        names.iter().any(|n| n == "001_init.down.sql"),
        "001_init.down.sql missing"
    );
}

#[test]
fn test_database_postgresql_up_migration_content() {
    let (tmp, _) = render_database("postgresql");
    let content = read_output(&tmp, "001_init.up.sql");

    assert!(content.contains("main-db"));
    assert!(content.contains("PostgreSQL"));
    assert!(content.contains("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\""));
    assert!(content.contains("CREATE SCHEMA IF NOT EXISTS main_db"));
    assert!(content.contains("CREATE TABLE main_db.examples"));
    assert!(content.contains("UUID PRIMARY KEY"));
    assert!(content.contains("TIMESTAMPTZ"));
    assert!(content.contains("CREATE OR REPLACE FUNCTION main_db.update_updated_at()"));
    assert!(content.contains("CREATE INDEX idx_examples_status"));
}

#[test]
fn test_database_postgresql_down_migration_content() {
    let (tmp, _) = render_database("postgresql");
    let content = read_output(&tmp, "001_init.down.sql");

    assert!(
        content.contains("DROP TRIGGER IF EXISTS trigger_update_updated_at ON main_db.examples")
    );
    assert!(content.contains("DROP FUNCTION IF EXISTS main_db.update_updated_at()"));
    assert!(content.contains("DROP TABLE IF EXISTS main_db.examples"));
    assert!(content.contains("DROP SCHEMA IF EXISTS main_db"));
    assert!(content.contains("DROP EXTENSION IF EXISTS \"uuid-ossp\""));
}

// =========================================================================
// MySQL
// =========================================================================

#[test]
fn test_database_mysql_file_list() {
    let (_, names) = render_database("mysql");

    assert!(
        names.iter().any(|n| n == "001_init.up.sql"),
        "001_init.up.sql missing"
    );
    assert!(
        names.iter().any(|n| n == "001_init.down.sql"),
        "001_init.down.sql missing"
    );
}

#[test]
fn test_database_mysql_up_migration_content() {
    let (tmp, _) = render_database("mysql");
    let content = read_output(&tmp, "001_init.up.sql");

    assert!(content.contains("main-db"));
    assert!(content.contains("MySQL"));
    assert!(content.contains("CREATE DATABASE IF NOT EXISTS main_db"));
    assert!(content.contains("CHARACTER SET utf8mb4"));
    assert!(content.contains("CREATE TABLE examples"));
    assert!(content.contains("CHAR(36) PRIMARY KEY"));
    assert!(content.contains("ENGINE=InnoDB"));
}

#[test]
fn test_database_mysql_down_migration_content() {
    let (tmp, _) = render_database("mysql");
    let content = read_output(&tmp, "001_init.down.sql");

    assert!(content.contains("DROP TABLE IF EXISTS examples"));
    assert!(content.contains("DROP DATABASE IF EXISTS main_db"));
}

// =========================================================================
// SQLite
// =========================================================================

#[test]
fn test_database_sqlite_file_list() {
    let (_, names) = render_database("sqlite");

    assert!(
        names.iter().any(|n| n == "001_init.up.sql"),
        "001_init.up.sql missing"
    );
    assert!(
        names.iter().any(|n| n == "001_init.down.sql"),
        "001_init.down.sql missing"
    );
}

#[test]
fn test_database_sqlite_up_migration_content() {
    let (tmp, _) = render_database("sqlite");
    let content = read_output(&tmp, "001_init.up.sql");

    assert!(content.contains("main-db"));
    assert!(content.contains("SQLite"));
    assert!(content.contains("CREATE TABLE IF NOT EXISTS examples"));
    assert!(content.contains("TEXT PRIMARY KEY"));
    assert!(content.contains("randomblob"));
    assert!(content.contains("CREATE TRIGGER IF NOT EXISTS trigger_update_updated_at"));
}

#[test]
fn test_database_sqlite_down_migration_content() {
    let (tmp, _) = render_database("sqlite");
    let content = read_output(&tmp, "001_init.down.sql");

    assert!(content.contains("DROP TRIGGER IF EXISTS trigger_update_updated_at"));
    assert!(content.contains("DROP INDEX IF EXISTS idx_examples_created_at"));
    assert!(content.contains("DROP INDEX IF EXISTS idx_examples_status"));
    assert!(content.contains("DROP TABLE IF EXISTS examples"));
}

// =========================================================================
// マイグレーション対称性テスト
// =========================================================================

#[test]
fn test_database_migration_symmetry_postgresql() {
    let (tmp, _) = render_database("postgresql");
    let up = read_output(&tmp, "001_init.up.sql");
    let down = read_output(&tmp, "001_init.down.sql");

    // up で CREATE したものが down で DROP されている
    assert!(up.contains("CREATE EXTENSION") && down.contains("DROP EXTENSION"));
    assert!(up.contains("CREATE SCHEMA") && down.contains("DROP SCHEMA"));
    assert!(up.contains("CREATE TABLE") && down.contains("DROP TABLE"));
    assert!(up.contains("CREATE OR REPLACE FUNCTION") && down.contains("DROP FUNCTION"));
    assert!(up.contains("CREATE TRIGGER") && down.contains("DROP TRIGGER"));
}

#[test]
fn test_database_migration_symmetry_mysql() {
    let (tmp, _) = render_database("mysql");
    let up = read_output(&tmp, "001_init.up.sql");
    let down = read_output(&tmp, "001_init.down.sql");

    // up で CREATE したものが down で DROP されている
    assert!(up.contains("CREATE DATABASE") && down.contains("DROP DATABASE"));
    assert!(up.contains("CREATE TABLE") && down.contains("DROP TABLE"));
}

#[test]
fn test_database_migration_symmetry_sqlite() {
    let (tmp, _) = render_database("sqlite");
    let up = read_output(&tmp, "001_init.up.sql");
    let down = read_output(&tmp, "001_init.down.sql");

    // up で CREATE したものが down で DROP されている
    assert!(up.contains("CREATE TABLE") && down.contains("DROP TABLE"));
    assert!(up.contains("CREATE TRIGGER") && down.contains("DROP TRIGGER"));
    assert!(up.contains("CREATE INDEX") && down.contains("DROP INDEX"));
}
