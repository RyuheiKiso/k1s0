/// マイグレーション状態の確認。
use anyhow::Result;

use super::scanner::scan_migration_files;
use super::types::{DbConnection, Direction, MigrateTarget, MigrationStatus};

/// マイグレーション状態を取得する。
///
/// ファイルシステム上のマイグレーションファイルを走査し、
/// 各マイグレーションの状態を返す。
///
/// 注: 実際のDB適用状態の確認には、マイグレーションツールの出力を
/// パースする必要があるが、現時点ではファイルベースの情報のみ返す。
///
/// # Errors
///
/// ファイル走査に失敗した場合にエラーを返す。
pub fn get_migration_status(
    target: &MigrateTarget,
    _connection: &DbConnection,
) -> Result<Vec<MigrationStatus>> {
    let files = scan_migration_files(&target.migrations_dir)?;

    // up ファイルのみを使って状態一覧を構築
    let statuses: Vec<MigrationStatus> = files
        .iter()
        .filter(|f| f.direction == Direction::Up)
        .map(|f| MigrationStatus {
            number: f.number,
            description: f.description.clone(),
            applied: false, // ファイルベースでは適用状態は不明
            applied_at: None,
        })
        .collect();

    Ok(statuses)
}

/// 全対象のマイグレーション状態を表示する。
///
/// # Errors
///
/// 状態取得に失敗した場合にエラーを返す。
pub fn get_all_migration_status(targets: &[MigrateTarget]) -> Result<()> {
    for target in targets {
        println!("\n=== {} ({}) ===", target.service_name, target.tier);
        let statuses = get_migration_status(target, &DbConnection::LocalDev)?;
        if statuses.is_empty() {
            println!("  マイグレーションファイルはありません。");
        } else {
            for status in &statuses {
                let mark = if status.applied { "[x]" } else { "[ ]" };
                let at = status.applied_at.as_deref().unwrap_or("-");
                println!(
                    "  {} {:03}_{} (適用日時: {})",
                    mark, status.number, status.description, at
                );
            }
        }
    }
    Ok(())
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::commands::migrate::types::Language;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_get_migration_status_empty() {
        let tmp = TempDir::new().unwrap();
        let migrations_dir = tmp.path().join("migrations");
        fs::create_dir_all(&migrations_dir).unwrap();

        let target = MigrateTarget {
            service_name: "auth".to_string(),
            tier: "system".to_string(),
            language: Language::Rust,
            migrations_dir,
            db_name: "auth_db".to_string(),
        };

        let statuses = get_migration_status(&target, &DbConnection::LocalDev).unwrap();
        assert!(statuses.is_empty());
    }

    #[test]
    fn test_get_migration_status_with_files() {
        let tmp = TempDir::new().unwrap();
        let migrations_dir = tmp.path().join("migrations");
        fs::create_dir_all(&migrations_dir).unwrap();

        fs::write(
            migrations_dir.join("001_create_users.up.sql"),
            "CREATE TABLE;",
        )
        .unwrap();
        fs::write(
            migrations_dir.join("001_create_users.down.sql"),
            "DROP TABLE;",
        )
        .unwrap();
        fs::write(migrations_dir.join("002_add_email.up.sql"), "ALTER TABLE;").unwrap();
        fs::write(
            migrations_dir.join("002_add_email.down.sql"),
            "ALTER TABLE;",
        )
        .unwrap();

        let target = MigrateTarget {
            service_name: "auth".to_string(),
            tier: "system".to_string(),
            language: Language::Rust,
            migrations_dir,
            db_name: "auth_db".to_string(),
        };

        let statuses = get_migration_status(&target, &DbConnection::LocalDev).unwrap();
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0].number, 1);
        assert_eq!(statuses[0].description, "create_users");
        assert!(!statuses[0].applied);
        assert_eq!(statuses[1].number, 2);
        assert_eq!(statuses[1].description, "add_email");
    }

    #[test]
    fn test_get_migration_status_nonexistent() {
        let target = MigrateTarget {
            service_name: "test".to_string(),
            tier: "system".to_string(),
            language: Language::Rust,
            migrations_dir: PathBuf::from("/nonexistent/migrations"),
            db_name: "test_db".to_string(),
        };

        let statuses = get_migration_status(&target, &DbConnection::LocalDev).unwrap();
        assert!(statuses.is_empty());
    }
}
