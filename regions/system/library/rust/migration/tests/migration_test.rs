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

#[test]
fn config_default_table_name() {
    let config = MigrationConfig::new(PathBuf::from("./migrations"), "postgres://test".to_string());
    assert_eq!(config.table_name, "_migrations");
}

#[test]
fn config_custom_table_name() {
    let config = MigrationConfig::new(PathBuf::from("./migrations"), "postgres://test".to_string())
        .with_table_name("schema_versions");
    assert_eq!(config.table_name, "schema_versions");
}

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

#[test]
fn parse_filename_up_migration() {
    let result = MigrationFile::parse_filename("20240101000001_create_users.up.sql");
    assert!(result.is_some());
    let (version, name, _dir) = result.unwrap();
    assert_eq!(version, "20240101000001");
    assert_eq!(name, "create_users");
}

#[test]
fn parse_filename_down_migration() {
    let result = MigrationFile::parse_filename("20240101000001_create_users.down.sql");
    assert!(result.is_some());
    let (version, name, _dir) = result.unwrap();
    assert_eq!(version, "20240101000001");
    assert_eq!(name, "create_users");
}

#[test]
fn parse_filename_invalid_no_direction() {
    assert!(MigrationFile::parse_filename("20240101000001_create_users.sql").is_none());
}

#[test]
fn parse_filename_invalid_no_underscore() {
    assert!(MigrationFile::parse_filename("nodirection.up.sql").is_none());
}

#[test]
fn parse_filename_invalid_empty_version() {
    assert!(MigrationFile::parse_filename("_.up.sql").is_none());
}

#[test]
fn parse_filename_invalid_not_sql() {
    assert!(MigrationFile::parse_filename("20240101_create.up.txt").is_none());
}

// ===========================================================================
// MigrationFile::checksum tests
// ===========================================================================

#[test]
fn checksum_deterministic() {
    let content = "CREATE TABLE users (id SERIAL PRIMARY KEY);";
    let c1 = MigrationFile::checksum(content);
    let c2 = MigrationFile::checksum(content);
    assert_eq!(c1, c2);
}

#[test]
fn checksum_differs_for_different_content() {
    let c1 = MigrationFile::checksum("CREATE TABLE users;");
    let c2 = MigrationFile::checksum("CREATE TABLE orders;");
    assert_ne!(c1, c2);
}

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

#[test]
fn error_directory_not_found() {
    let err = MigrationError::DirectoryNotFound("/nonexistent".to_string());
    assert!(err.to_string().contains("/nonexistent"));
}

#[test]
fn error_connection_failed() {
    let err = MigrationError::ConnectionFailed("refused".to_string());
    assert!(err.to_string().contains("refused"));
}

#[test]
fn error_migration_failed() {
    let err = MigrationError::MigrationFailed {
        version: "001".to_string(),
        message: "syntax error".to_string(),
    };
    assert!(err.to_string().contains("001"));
    assert!(err.to_string().contains("syntax error"));
}

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

#[test]
fn error_parse_error() {
    let err = MigrationError::ParseError("bad format".to_string());
    assert!(err.to_string().contains("bad format"));
}

// ===========================================================================
// InMemoryMigrationRunner — run_up
// ===========================================================================

#[tokio::test]
async fn run_up_applies_all_pending() {
    let runner = create_runner();
    let report = runner.run_up().await.unwrap();
    assert_eq!(report.applied_count, 3);
    assert!(report.errors.is_empty());
}

#[tokio::test]
async fn run_up_idempotent() {
    let runner = create_runner();
    runner.run_up().await.unwrap();
    let report = runner.run_up().await.unwrap();
    assert_eq!(report.applied_count, 0);
}

#[tokio::test]
async fn run_up_single_migration() {
    let runner = create_single_migration_runner();
    let report = runner.run_up().await.unwrap();
    assert_eq!(report.applied_count, 1);
}

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

#[tokio::test]
async fn run_down_multiple_steps() {
    let runner = create_runner();
    runner.run_up().await.unwrap();

    let report = runner.run_down(2).await.unwrap();
    assert_eq!(report.applied_count, 2);

    let pending = runner.pending().await.unwrap();
    assert_eq!(pending.len(), 2);
}

#[tokio::test]
async fn run_down_all_steps() {
    let runner = create_runner();
    runner.run_up().await.unwrap();

    let report = runner.run_down(3).await.unwrap();
    assert_eq!(report.applied_count, 3);

    let pending = runner.pending().await.unwrap();
    assert_eq!(pending.len(), 3);
}

#[tokio::test]
async fn run_down_more_than_applied_caps() {
    let runner = create_runner();
    runner.run_up().await.unwrap();

    let report = runner.run_down(100).await.unwrap();
    assert_eq!(report.applied_count, 3);
}

#[tokio::test]
async fn run_down_on_empty_noop() {
    let runner = create_runner();
    let report = runner.run_down(5).await.unwrap();
    assert_eq!(report.applied_count, 0);
}

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

#[tokio::test]
async fn status_all_pending() {
    let runner = create_runner();
    let statuses = runner.status().await.unwrap();
    assert_eq!(statuses.len(), 3);
    for s in &statuses {
        assert!(s.applied_at.is_none());
    }
}

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

#[tokio::test]
async fn pending_returns_all_initially() {
    let runner = create_runner();
    let pending = runner.pending().await.unwrap();
    assert_eq!(pending.len(), 3);
    assert_eq!(pending[0].version, "20240101000001");
    assert_eq!(pending[0].name, "create_users");
}

#[tokio::test]
async fn pending_empty_after_run_up() {
    let runner = create_runner();
    runner.run_up().await.unwrap();
    let pending = runner.pending().await.unwrap();
    assert!(pending.is_empty());
}

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

#[tokio::test]
async fn empty_migrations_run_up_noop() {
    let runner = InMemoryMigrationRunner::from_migrations(test_config(), vec![], vec![]);
    let report = runner.run_up().await.unwrap();
    assert_eq!(report.applied_count, 0);
}

#[tokio::test]
async fn empty_migrations_status_empty() {
    let runner = InMemoryMigrationRunner::from_migrations(test_config(), vec![], vec![]);
    let statuses = runner.status().await.unwrap();
    assert!(statuses.is_empty());
}

#[tokio::test]
async fn empty_migrations_pending_empty() {
    let runner = InMemoryMigrationRunner::from_migrations(test_config(), vec![], vec![]);
    let pending = runner.pending().await.unwrap();
    assert!(pending.is_empty());
}
