/// CI統合チェック: マイグレーションファイルの整合性検証。
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;

use super::scanner::{scan_migrate_targets_at, scan_migration_files};
use super::types::{Direction, MigrateCiResult, MigrateTarget};

/// 単一サービスのマイグレーション整合性をチェックする。
pub fn check_migration_integrity(target: &MigrateTarget) -> MigrateCiResult {
    let mut result = MigrateCiResult::default();

    // ファイル走査
    let files = match scan_migration_files(&target.migrations_dir) {
        Ok(f) => f,
        Err(e) => {
            result
                .errors
                .push(format!("ファイル走査に失敗しました: {e}"));
            return result;
        }
    };

    if files.is_empty() {
        result
            .warnings
            .push("マイグレーションファイルがありません".to_string());
        return result;
    }

    // 1. ペア整合性チェック（up/down がペアで存在するか）
    let mut up_numbers: HashMap<u32, String> = HashMap::new();
    let mut down_numbers: HashMap<u32, String> = HashMap::new();
    for f in &files {
        match f.direction {
            Direction::Up => {
                up_numbers.insert(f.number, f.description.clone());
            }
            Direction::Down => {
                down_numbers.insert(f.number, f.description.clone());
            }
        }
    }

    for (num, desc) in &up_numbers {
        if !down_numbers.contains_key(num) {
            result.errors.push(format!(
                "down.sql が見つかりません: {num:03}_{desc}.down.sql"
            ));
        }
    }

    for (num, desc) in &down_numbers {
        if !up_numbers.contains_key(num) {
            result.errors.push(format!(
                "up.sql が見つかりません: {num:03}_{desc}.up.sql"
            ));
        }
    }

    // 2. 連番整合性チェック（ギャップや重複がないか）
    let mut all_numbers: Vec<u32> = up_numbers.keys().copied().collect();
    all_numbers.sort_unstable();

    if !all_numbers.is_empty() {
        // 1から始まるか確認
        if all_numbers[0] != 1 {
            result.warnings.push(format!(
                "連番が1から始まっていません（最初の番号: {}）",
                all_numbers[0]
            ));
        }

        // ギャップの検出
        for i in 1..all_numbers.len() {
            let expected = all_numbers[i - 1] + 1;
            if all_numbers[i] != expected {
                result.warnings.push(format!(
                    "連番にギャップがあります: {} の次が {} です（期待値: {}）",
                    all_numbers[i - 1],
                    all_numbers[i],
                    expected
                ));
            }
        }
    }

    // 3. down.sql 非空チェック
    for f in &files {
        if f.direction == Direction::Down {
            if let Ok(content) = fs::read_to_string(&f.path) {
                let trimmed = content
                    .lines()
                    .filter(|line| !line.trim().starts_with("--") && !line.trim().is_empty())
                    .collect::<Vec<_>>()
                    .join("");
                if trimmed.is_empty() {
                    result.warnings.push(format!(
                        "down.sql にSQL文がありません: {:03}_{}.down.sql",
                        f.number, f.description
                    ));
                }
            }
        }
    }

    // 4. SQL構文の基本チェック（セミコロンの存在確認）
    for f in &files {
        if let Ok(content) = fs::read_to_string(&f.path) {
            let sql_lines: Vec<&str> = content
                .lines()
                .filter(|line| !line.trim().starts_with("--") && !line.trim().is_empty())
                .collect();
            if !sql_lines.is_empty() && !content.contains(';') {
                result.errors.push(format!(
                    "SQL文にセミコロンがありません: {}",
                    f.path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                ));
            }
        }
    }

    result
}

/// 全マイグレーション対象の整合性をチェックする。
///
/// # Errors
///
/// 走査に失敗した場合にエラーを返す。
pub fn check_all_migrations(base_dir: &Path) -> Result<Vec<(String, MigrateCiResult)>> {
    let targets = scan_migrate_targets_at(base_dir);
    let mut results = Vec::new();

    for target in &targets {
        let result = check_migration_integrity(target);
        results.push((target.service_name.clone(), result));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::migrate::types::Language;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_target(migrations_dir: PathBuf) -> MigrateTarget {
        MigrateTarget {
            service_name: "test".to_string(),
            tier: "system".to_string(),
            language: Language::Rust,
            migrations_dir,
            db_name: "test_db".to_string(),
        }
    }

    #[test]
    fn test_check_integrity_valid() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        fs::write(
            migrations.join("001_create_users.up.sql"),
            "CREATE TABLE users (id INT);\n",
        )
        .unwrap();
        fs::write(
            migrations.join("001_create_users.down.sql"),
            "DROP TABLE users;\n",
        )
        .unwrap();
        fs::write(
            migrations.join("002_add_email.up.sql"),
            "ALTER TABLE users ADD COLUMN email VARCHAR;\n",
        )
        .unwrap();
        fs::write(
            migrations.join("002_add_email.down.sql"),
            "ALTER TABLE users DROP COLUMN email;\n",
        )
        .unwrap();

        let target = make_target(migrations);
        let result = check_migration_integrity(&target);
        assert!(!result.has_errors(), "エラー: {:?}", result.errors);
        assert!(!result.has_warnings(), "警告: {:?}", result.warnings);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_integrity_missing_down() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        fs::write(
            migrations.join("001_create_users.up.sql"),
            "CREATE TABLE users (id INT);\n",
        )
        .unwrap();
        // down.sql が無い

        let target = make_target(migrations);
        let result = check_migration_integrity(&target);
        assert!(result.has_errors());
        assert!(result.errors[0].contains("down.sql が見つかりません"));
    }

    #[test]
    fn test_check_integrity_missing_up() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        // up.sql が無い
        fs::write(
            migrations.join("001_create_users.down.sql"),
            "DROP TABLE users;\n",
        )
        .unwrap();

        let target = make_target(migrations);
        let result = check_migration_integrity(&target);
        assert!(result.has_errors());
        assert!(result.errors[0].contains("up.sql が見つかりません"));
    }

    #[test]
    fn test_check_integrity_gap_in_sequence() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        fs::write(
            migrations.join("001_create_users.up.sql"),
            "CREATE TABLE users;\n",
        )
        .unwrap();
        fs::write(
            migrations.join("001_create_users.down.sql"),
            "DROP TABLE users;\n",
        )
        .unwrap();
        // 002 がスキップされている
        fs::write(
            migrations.join("003_add_email.up.sql"),
            "ALTER TABLE users ADD email;\n",
        )
        .unwrap();
        fs::write(
            migrations.join("003_add_email.down.sql"),
            "ALTER TABLE users DROP email;\n",
        )
        .unwrap();

        let target = make_target(migrations);
        let result = check_migration_integrity(&target);
        assert!(result.has_warnings());
        assert!(result.warnings.iter().any(|w| w.contains("ギャップ")));
    }

    #[test]
    fn test_check_integrity_not_starting_from_1() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        fs::write(
            migrations.join("002_something.up.sql"),
            "CREATE TABLE t;\n",
        )
        .unwrap();
        fs::write(
            migrations.join("002_something.down.sql"),
            "DROP TABLE t;\n",
        )
        .unwrap();

        let target = make_target(migrations);
        let result = check_migration_integrity(&target);
        assert!(result.has_warnings());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("1から始まっていません")));
    }

    #[test]
    fn test_check_integrity_empty_down() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        fs::write(
            migrations.join("001_create_users.up.sql"),
            "CREATE TABLE users;\n",
        )
        .unwrap();
        fs::write(
            migrations.join("001_create_users.down.sql"),
            "-- ロールバック: create_users\n-- 作成日: 2026-01-01\n\n-- TODO: ここに処理を記述\n",
        )
        .unwrap();

        let target = make_target(migrations);
        let result = check_migration_integrity(&target);
        assert!(result.has_warnings());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("SQL文がありません")));
    }

    #[test]
    fn test_check_integrity_no_semicolon() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        fs::write(
            migrations.join("001_create_users.up.sql"),
            "CREATE TABLE users (id INT)\n",
        )
        .unwrap();
        fs::write(
            migrations.join("001_create_users.down.sql"),
            "DROP TABLE users;\n",
        )
        .unwrap();

        let target = make_target(migrations);
        let result = check_migration_integrity(&target);
        assert!(result.has_errors());
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("セミコロンがありません")));
    }

    #[test]
    fn test_check_integrity_empty_migrations() {
        let tmp = TempDir::new().unwrap();
        let migrations = tmp.path().join("migrations");
        fs::create_dir_all(&migrations).unwrap();

        let target = make_target(migrations);
        let result = check_migration_integrity(&target);
        assert!(!result.has_errors());
        assert!(result.has_warnings());
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("ファイルがありません")));
    }

    #[test]
    fn test_check_all_migrations() {
        let tmp = TempDir::new().unwrap();

        // サービスをセットアップ
        let service_dir = tmp.path().join("regions/system/server/rust/auth");
        fs::create_dir_all(&service_dir).unwrap();
        fs::write(service_dir.join("Cargo.toml"), "[package]").unwrap();
        let migrations = service_dir.join("migrations");
        fs::create_dir_all(&migrations).unwrap();
        fs::write(
            migrations.join("001_init.up.sql"),
            "CREATE TABLE t;\n",
        )
        .unwrap();
        fs::write(
            migrations.join("001_init.down.sql"),
            "DROP TABLE t;\n",
        )
        .unwrap();

        let results = check_all_migrations(tmp.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "auth");
        assert!(results[0].1.is_ok());
    }
}
