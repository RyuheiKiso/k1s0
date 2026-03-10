/// マイグレーションファイルの新規作成。
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Local;

use super::scanner::{next_sequence_number, scan_migration_files, validate_migration_name};
use super::types::MigrateCreateConfig;

/// マイグレーションファイル（up/down ペア）を作成する。
///
/// # 戻り値
///
/// 作成された (up.sql, down.sql) のパスを返す。
///
/// # Errors
///
/// ファイル書き込みに失敗した場合にエラーを返す。
pub fn create_migration(config: &MigrateCreateConfig) -> Result<(PathBuf, PathBuf)> {
    // 名前のバリデーション
    validate_migration_name(&config.migration_name)
        .map_err(|e| anyhow::anyhow!("マイグレーション名が不正です: {e}"))?;

    // migrations/ ディレクトリが存在しない場合は作成
    if !config.target.migrations_dir.exists() {
        fs::create_dir_all(&config.target.migrations_dir).with_context(|| {
            format!(
                "migrations ディレクトリの作成に失敗しました: {}",
                config.target.migrations_dir.display()
            )
        })?;
    }

    // 既存ファイルを走査して次の連番を取得
    let existing_files = scan_migration_files(&config.target.migrations_dir)?;
    let seq = next_sequence_number(&existing_files);

    let date = Local::now().format("%Y-%m-%d").to_string();
    let prefix = format!("{:03}_{}", seq, config.migration_name);

    // up.sql
    let up_path = config
        .target
        .migrations_dir
        .join(format!("{prefix}.up.sql"));
    let up_content = format!(
        "-- migrations/{prefix}.up.sql\n\
         -- マイグレーション: {name}\n\
         -- 作成日: {date}\n\
         \n\
         -- TODO: ここにスキーマ変更を記述してください\n",
        prefix = prefix,
        name = config.migration_name,
        date = date,
    );
    fs::write(&up_path, &up_content)
        .with_context(|| format!("up.sql の書き込みに失敗しました: {}", up_path.display()))?;

    // down.sql
    let down_path = config
        .target
        .migrations_dir
        .join(format!("{prefix}.down.sql"));
    let down_content = format!(
        "-- migrations/{prefix}.down.sql\n\
         -- ロールバック: {name}\n\
         -- 作成日: {date}\n\
         \n\
         -- TODO: ここに up.sql の変更を元に戻す処理を記述してください\n",
        prefix = prefix,
        name = config.migration_name,
        date = date,
    );
    fs::write(&down_path, &down_content)
        .with_context(|| format!("down.sql の書き込みに失敗しました: {}", down_path.display()))?;

    Ok((up_path, down_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::migrate::types::{Language, MigrateTarget};
    use tempfile::TempDir;

    fn make_target(tmp: &TempDir) -> MigrateTarget {
        let migrations_dir = tmp.path().join("migrations");
        MigrateTarget {
            service_name: "auth".to_string(),
            tier: "system".to_string(),
            language: Language::Rust,
            migrations_dir,
            db_name: "auth_db".to_string(),
        }
    }

    #[test]
    fn test_create_migration_first() {
        let tmp = TempDir::new().unwrap();
        let target = make_target(&tmp);

        let config = MigrateCreateConfig {
            target,
            migration_name: "create_users".to_string(),
        };

        let (up, down) = create_migration(&config).unwrap();

        assert!(up.exists());
        assert!(down.exists());

        let up_name = up.file_name().unwrap().to_str().unwrap();
        let down_name = down.file_name().unwrap().to_str().unwrap();
        assert_eq!(up_name, "001_create_users.up.sql");
        assert_eq!(down_name, "001_create_users.down.sql");

        // up.sql の内容を確認
        let up_content = fs::read_to_string(&up).unwrap();
        assert!(up_content.contains("マイグレーション: create_users"));
        assert!(up_content.contains("ここにスキーマ変更を記述してください"));

        // down.sql の内容を確認
        let down_content = fs::read_to_string(&down).unwrap();
        assert!(down_content.contains("ロールバック: create_users"));
        assert!(down_content.contains("up.sql の変更を元に戻す処理を記述してください"));
    }

    #[test]
    fn test_create_migration_sequential() {
        let tmp = TempDir::new().unwrap();
        let migrations_dir = tmp.path().join("migrations");
        fs::create_dir_all(&migrations_dir).unwrap();

        // 既存のマイグレーションファイルを配置
        fs::write(
            migrations_dir.join("001_create_users.up.sql"),
            "CREATE TABLE users;",
        )
        .unwrap();
        fs::write(
            migrations_dir.join("001_create_users.down.sql"),
            "DROP TABLE users;",
        )
        .unwrap();

        let target = MigrateTarget {
            service_name: "auth".to_string(),
            tier: "system".to_string(),
            language: Language::Rust,
            migrations_dir: migrations_dir.clone(),
            db_name: "auth_db".to_string(),
        };

        let config = MigrateCreateConfig {
            target,
            migration_name: "add_email".to_string(),
        };

        let (up, down) = create_migration(&config).unwrap();
        let up_name = up.file_name().unwrap().to_str().unwrap();
        let down_name = down.file_name().unwrap().to_str().unwrap();
        assert_eq!(up_name, "002_add_email.up.sql");
        assert_eq!(down_name, "002_add_email.down.sql");
    }

    #[test]
    fn test_create_migration_invalid_name() {
        let tmp = TempDir::new().unwrap();
        let target = make_target(&tmp);

        let config = MigrateCreateConfig {
            target,
            migration_name: "Invalid-Name".to_string(),
        };

        let result = create_migration(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("マイグレーション名が不正です"));
    }

    #[test]
    fn test_create_migration_creates_dir() {
        let tmp = TempDir::new().unwrap();
        let target = make_target(&tmp);

        // migrations/ ディレクトリが存在しないことを確認
        assert!(!target.migrations_dir.exists());

        let config = MigrateCreateConfig {
            target,
            migration_name: "init".to_string(),
        };

        let (up, _) = create_migration(&config).unwrap();
        assert!(up.exists());
    }
}
