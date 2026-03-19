#![allow(clippy::unwrap_used)]
use k1s0_migration::{
    InMemoryMigrationRunner, MigrationConfig, MigrationError, MigrationFile, MigrationReport,
    MigrationRunner, MigrationStatus, PendingMigration,
};
use std::path::PathBuf;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_config() -> MigrationConfig {
    MigrationConfig::new(PathBuf::from("."), "memory://".to_string())
}

fn create_runner() -> InMemoryMigrationRunner {
    InMemoryMigrationRunner::from_migrations(
        test_config(),
        vec![
            (
                "20240101000001".to_string(),
                "create_users".to_string(),
                "CREATE TABLE users (id INT PRIMARY KEY);".to_string(),
            ),
            (
                "20240101000002".to_string(),
                "add_email".to_string(),
                "ALTER TABLE users ADD COLUMN email TEXT;".to_string(),
            ),
            (
                "20240201000001".to_string(),
                "create_orders".to_string(),
                "CREATE TABLE orders (id INT PRIMARY KEY);".to_string(),
            ),
        ],
        vec![
            (
                "20240101000001".to_string(),
                "create_users".to_string(),
                "DROP TABLE users;".to_string(),
            ),
            (
                "20240101000002".to_string(),
                "add_email".to_string(),
                "ALTER TABLE users DROP COLUMN email;".to_string(),
            ),
            (
                "20240201000001".to_string(),
                "create_orders".to_string(),
                "DROP TABLE orders;".to_string(),
            ),
        ],
    )
}

fn create_single_migration_runner() -> InMemoryMigrationRunner {
    InMemoryMigrationRunner::from_migrations(
        test_config(),
        vec![(
            "20240301000001".to_string(),
            "create_products".to_string(),
            "CREATE TABLE products (id INT);".to_string(),
        )],
        vec![(
            "20240301000001".to_string(),
            "create_products".to_string(),
            "DROP TABLE products;".to_string(),
        )],
    )
}

// ===========================================================================
// MigrationConfig tests
// ===========================================================================

// MigrationConfig のデフォルトテーブル名が "_migrations" であることを確認する。
#[test]
fn config_default_table_name() {
    let config = MigrationConfig::new(PathBuf::from("./migrations"), "postgres://test".to_string());
    assert_eq!(config.table_name, "_migrations");
}

// with_table_name でカスタムテーブル名を設定できることを確認する。
#[test]
fn config_custom_table_name() {
    let config = MigrationConfig::new(PathBuf::from("./migrations"), "postgres://test".to_string())
        .with_table_name("schema_versions");
    assert_eq!(config.table_name, "schema_versions");
}

// MigrationConfig がマイグレーションディレクトリと DB URL を正しく保持することを確認する。
#[test]
fn config_stores_paths() {
    let config = MigrationConfig::new(
        PathBuf::from("/app/migrations"),
        "postgres://host/db".to_string(),
    );
    assert_eq!(config.migrations_dir, PathBuf::from("/app/migrations"));
    assert_eq!(config.database_url, "postgres://host/db");
}

// ===========================================================================
// MigrationFile::parse_filename tests
// ===========================================================================

// UP マイグレーションのファイル名を parse_filename が正しく解析することを確認する。
#[test]
fn parse_filename_up_migration() {
    let result = MigrationFile::parse_filename("20240101000001_create_users.up.sql");
    assert!(result.is_some());
    let (version, name, _dir) = result.unwrap();
    assert_eq!(version, "20240101000001");
    assert_eq!(name, "create_users");
}

// DOWN マイグレーションのファイル名を parse_filename が正しく解析することを確認する。
#[test]
fn parse_filename_down_migration() {
    let result = MigrationFile::parse_filename("20240101000001_create_users.down.sql");
    assert!(result.is_some());
    let (version, name, _dir) = result.unwrap();
    assert_eq!(version, "20240101000001");
    assert_eq!(name, "create_users");
}

// up/down の方向指定がないファイル名は parse_filename が None を返すことを確認する。
#[test]
fn parse_filename_invalid_no_direction() {
    assert!(MigrationFile::parse_filename("20240101000001_create_users.sql").is_none());
}

// アンダースコアがないファイル名は parse_filename が None を返すことを確認する。
#[test]
fn parse_filename_invalid_no_underscore() {
    assert!(MigrationFile::parse_filename("nodirection.up.sql").is_none());
}

// バージョン部分が空のファイル名は parse_filename が None を返すことを確認する。
#[test]
fn parse_filename_invalid_empty_version() {
    assert!(MigrationFile::parse_filename("_.up.sql").is_none());
}

// .sql 拡張子でないファイル名は parse_filename が None を返すことを確認する。
#[test]
fn parse_filename_invalid_not_sql() {
    assert!(MigrationFile::parse_filename("20240101_create.up.txt").is_none());
}

// ===========================================================================
// MigrationFile::checksum tests
// ===========================================================================

// 同じ内容のチェックサムが常に同じ値を返す（決定論的）ことを確認する。
#[test]
fn checksum_deterministic() {
    let content = "CREATE TABLE users (id SERIAL PRIMARY KEY);";
    let c1 = MigrationFile::checksum(content);
    let c2 = MigrationFile::checksum(content);
    assert_eq!(c1, c2);
}

// 異なる内容のチェックサムが異なる値を返すことを確認する。
#[test]
fn checksum_differs_for_different_content() {
    let c1 = MigrationFile::checksum("CREATE TABLE users;");
    let c2 = MigrationFile::checksum("CREATE TABLE orders;");
    assert_ne!(c1, c2);
}

// チェックサムが 64 文字の16進数文字列（SHA-256）であることを確認する。
#[test]
fn checksum_is_hex_string() {
    let c = MigrationFile::checksum("SELECT 1;");
    assert!(c.chars().all(|ch| ch.is_ascii_hexdigit()));
    // SHA-256 hex output is 64 characters
    assert_eq!(c.len(), 64);
}

// ===========================================================================
// Model struct tests
// ===========================================================================

// MigrationReport が適用件数・経過時間・エラーリストを正しく保持することを確認する。
#[test]
fn migration_report_fields() {
    let report = MigrationReport {
        applied_count: 3,
        elapsed: Duration::from_millis(150),
        errors: vec!["oops".to_string()],
    };
    assert_eq!(report.applied_count, 3);
    assert_eq!(report.elapsed, Duration::from_millis(150));
    assert_eq!(report.errors.len(), 1);
}

// applied_at が Some の MigrationStatus が適用済みを表すことを確認する。
#[test]
fn migration_status_applied() {
    let status = MigrationStatus {
        version: "20240101000001".to_string(),
        name: "create_users".to_string(),
        applied_at: Some(chrono::Utc::now()),
        checksum: "abc".to_string(),
    };
    assert!(status.applied_at.is_some());
}

// applied_at が None の MigrationStatus が未適用を表すことを確認する。
#[test]
fn migration_status_not_applied() {
    let status = MigrationStatus {
        version: "20240101000001".to_string(),
        name: "create_users".to_string(),
        applied_at: None,
        checksum: "abc".to_string(),
    };
    assert!(status.applied_at.is_none());
}

// PendingMigration がバージョンと名前を正しく保持することを確認する。
#[test]
fn pending_migration_fields() {
    let p = PendingMigration {
        version: "20240101000001".to_string(),
        name: "create_users".to_string(),
    };
    assert_eq!(p.version, "20240101000001");
    assert_eq!(p.name, "create_users");
}

// ===========================================================================
// MigrationError tests
// ===========================================================================

// DirectoryNotFound エラーの表示文字列にパスが含まれることを確認する。
#[test]
fn error_directory_not_found() {
    let err = MigrationError::DirectoryNotFound("/nonexistent".to_string());
    assert!(err.to_string().contains("/nonexistent"));
}

// ConnectionFailed エラーの表示文字列にメッセージが含まれることを確認する。
#[test]
fn error_connection_failed() {
    let err = MigrationError::ConnectionFailed("refused".to_string());
    assert!(err.to_string().contains("refused"));
}

// MigrationFailed エラーの表示文字列にバージョンとメッセージが含まれることを確認する。
#[test]
fn error_migration_failed() {
    let err = MigrationError::MigrationFailed {
        version: "001".to_string(),
        message: "syntax error".to_string(),
    };
    assert!(err.to_string().contains("001"));
    assert!(err.to_string().contains("syntax error"));
}

// ChecksumMismatch エラーの表示文字列に期待値と実際値が含まれることを確認する。
#[test]
fn error_checksum_mismatch() {
    let err = MigrationError::ChecksumMismatch {
        version: "001".to_string(),
        expected: "aaa".to_string(),
        actual: "bbb".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("aaa"));
    assert!(msg.contains("bbb"));
}

// ParseError の表示文字列にエラー内容が含まれることを確認する。
#[test]
fn error_parse_error() {
    let err = MigrationError::ParseError("bad format".to_string());
    assert!(err.to_string().contains("bad format"));
}

// ===========================================================================
// InMemoryMigrationRunner — run_up
// ===========================================================================

// run_up が全ての未適用マイグレーションを適用し適用件数を返すことを確認する。
#[tokio::test]
async fn run_up_applies_all_pending() {
    let runner = create_runner();
    let report = runner.run_up().await.unwrap();
    assert_eq!(report.applied_count, 3);
    assert!(report.errors.is_empty());
}

// run_up を 2 回実行しても 2 回目は 0 件適用で冪等に動作することを確認する。
#[tokio::test]
async fn run_up_idempotent() {
    let runner = create_runner();
    runner.run_up().await.unwrap();
    let report = runner.run_up().await.unwrap();
    assert_eq!(report.applied_count, 0);
}

// 単一のマイグレーションを持つランナーで run_up が 1 件適用することを確認する。
#[tokio::test]
async fn run_up_single_migration() {
    let runner = create_single_migration_runner();
    let report = runner.run_up().await.unwrap();
    assert_eq!(report.applied_count, 1);
}

// run_up のレポートに経過時間フィールドが含まれることを確認する。
#[tokio::test]
async fn run_up_report_has_elapsed_time() {
    let runner = create_runner();
    let report = runner.run_up().await.unwrap();
    // elapsed should be non-negative (it always is, but worth checking the field exists)
    assert!(report.elapsed >= Duration::ZERO);
}

// ===========================================================================
// InMemoryMigrationRunner — run_down
// ===========================================================================

// run_down(1) が最新のマイグレーション 1 件をロールバックすることを確認する。
#[tokio::test]
async fn run_down_single_step() {
    let runner = create_runner();
    runner.run_up().await.unwrap();

    let report = runner.run_down(1).await.unwrap();
    assert_eq!(report.applied_count, 1);

    let pending = runner.pending().await.unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].version, "20240201000001");
}

// run_down(2) が最新の 2 件のマイグレーションをロールバックすることを確認する。
#[tokio::test]
async fn run_down_multiple_steps() {
    let runner = create_runner();
    runner.run_up().await.unwrap();

    let report = runner.run_down(2).await.unwrap();
    assert_eq!(report.applied_count, 2);

    let pending = runner.pending().await.unwrap();
    assert_eq!(pending.len(), 2);
}

// run_down(3) が全 3 件のマイグレーションをロールバックすることを確認する。
#[tokio::test]
async fn run_down_all_steps() {
    let runner = create_runner();
    runner.run_up().await.unwrap();

    let report = runner.run_down(3).await.unwrap();
    assert_eq!(report.applied_count, 3);

    let pending = runner.pending().await.unwrap();
    assert_eq!(pending.len(), 3);
}

// run_down に適用済み件数を超えるステップ数を指定しても全件ロールバックで上限処理されることを確認する。
#[tokio::test]
async fn run_down_more_than_applied_caps() {
    let runner = create_runner();
    runner.run_up().await.unwrap();

    let report = runner.run_down(100).await.unwrap();
    assert_eq!(report.applied_count, 3);
}

// 未適用状態で run_down を実行しても 0 件処理で正常終了することを確認する。
#[tokio::test]
async fn run_down_on_empty_noop() {
    let runner = create_runner();
    let report = runner.run_down(5).await.unwrap();
    assert_eq!(report.applied_count, 0);
}

// run_down 後に run_up を実行するとロールバックされたマイグレーションが再適用されることを確認する。
#[tokio::test]
async fn run_up_after_down_reapplies() {
    let runner = create_runner();
    runner.run_up().await.unwrap();
    runner.run_down(1).await.unwrap();

    let report = runner.run_up().await.unwrap();
    assert_eq!(report.applied_count, 1);

    let pending = runner.pending().await.unwrap();
    assert!(pending.is_empty());
}

// ===========================================================================
// InMemoryMigrationRunner — status
// ===========================================================================

// 初期状態では全マイグレーションが未適用（applied_at = None）であることを確認する。
#[tokio::test]
async fn status_all_pending() {
    let runner = create_runner();
    let statuses = runner.status().await.unwrap();
    assert_eq!(statuses.len(), 3);
    for s in &statuses {
        assert!(s.applied_at.is_none());
    }
}

// run_up 後に全マイグレーションが適用済み（applied_at = Some）であることを確認する。
#[tokio::test]
async fn status_all_applied() {
    let runner = create_runner();
    runner.run_up().await.unwrap();
    let statuses = runner.status().await.unwrap();
    assert_eq!(statuses.len(), 3);
    for s in &statuses {
        assert!(s.applied_at.is_some());
    }
}

// run_down(1) 後に 2 件が適用済み・1 件が未適用になることを status で確認する。
#[tokio::test]
async fn status_partially_applied() {
    let runner = create_runner();
    runner.run_up().await.unwrap();
    runner.run_down(1).await.unwrap();

    let statuses = runner.status().await.unwrap();
    assert_eq!(statuses.len(), 3);
    let applied_count = statuses.iter().filter(|s| s.applied_at.is_some()).count();
    assert_eq!(applied_count, 2);
}

// status の結果がバージョン昇順に並んでいることを確認する。
#[tokio::test]
async fn status_ordered_by_version() {
    let runner = create_runner();
    let statuses = runner.status().await.unwrap();
    let versions: Vec<&str> = statuses.iter().map(|s| s.version.as_str()).collect();
    assert_eq!(
        versions,
        vec!["20240101000001", "20240101000002", "20240201000001"]
    );
}

// status の各エントリに 64 文字の SHA-256 チェックサムが含まれることを確認する。
#[tokio::test]
async fn status_includes_checksum() {
    let runner = create_runner();
    let statuses = runner.status().await.unwrap();
    for s in &statuses {
        assert!(!s.checksum.is_empty());
        assert_eq!(s.checksum.len(), 64); // SHA-256 hex
    }
}

// ===========================================================================
// InMemoryMigrationRunner — pending
// ===========================================================================

// 初期状態で pending が全マイグレーションをバージョン昇順で返すことを確認する。
#[tokio::test]
async fn pending_returns_all_initially() {
    let runner = create_runner();
    let pending = runner.pending().await.unwrap();
    assert_eq!(pending.len(), 3);
    assert_eq!(pending[0].version, "20240101000001");
    assert_eq!(pending[0].name, "create_users");
}

// run_up 後は pending が空になることを確認する。
#[tokio::test]
async fn pending_empty_after_run_up() {
    let runner = create_runner();
    runner.run_up().await.unwrap();
    let pending = runner.pending().await.unwrap();
    assert!(pending.is_empty());
}

// run_down(2) 後に pending が 2 件を返すことを確認する。
#[tokio::test]
async fn pending_reflects_partial_rollback() {
    let runner = create_runner();
    runner.run_up().await.unwrap();
    runner.run_down(2).await.unwrap();

    let pending = runner.pending().await.unwrap();
    assert_eq!(pending.len(), 2);
}

// ===========================================================================
// InMemoryMigrationRunner — constructor error
// ===========================================================================

// 存在しないディレクトリを指定して InMemoryMigrationRunner::new するとエラーになることを確認する。
#[test]
fn new_with_nonexistent_directory_fails() {
    let config = MigrationConfig::new(
        PathBuf::from("/absolutely/nonexistent/path"),
        "memory://".to_string(),
    );
    let result = InMemoryMigrationRunner::new(config);
    assert!(result.is_err());
    match result.unwrap_err() {
        MigrationError::DirectoryNotFound(path) => {
            assert!(path.contains("nonexistent"));
        }
        other => panic!("expected DirectoryNotFound, got: {:?}", other),
    }
}

// ===========================================================================
// from_migrations with no migrations
// ===========================================================================

// マイグレーションが空の場合に run_up が 0 件適用で正常終了することを確認する。
#[tokio::test]
async fn empty_migrations_run_up_noop() {
    let runner = InMemoryMigrationRunner::from_migrations(test_config(), vec![], vec![]);
    let report = runner.run_up().await.unwrap();
    assert_eq!(report.applied_count, 0);
}

// マイグレーションが空の場合に status が空のリストを返すことを確認する。
#[tokio::test]
async fn empty_migrations_status_empty() {
    let runner = InMemoryMigrationRunner::from_migrations(test_config(), vec![], vec![]);
    let statuses = runner.status().await.unwrap();
    assert!(statuses.is_empty());
}

// マイグレーションが空の場合に pending が空のリストを返すことを確認する。
#[tokio::test]
async fn empty_migrations_pending_empty() {
    let runner = InMemoryMigrationRunner::from_migrations(test_config(), vec![], vec![]);
    let pending = runner.pending().await.unwrap();
    assert!(pending.is_empty());
}
