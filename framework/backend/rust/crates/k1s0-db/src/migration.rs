//! マイグレーション
//!
//! マイグレーション実行の入口を固定し、サービスごとの分岐を防ぐ。

use std::path::{Path, PathBuf};

use crate::error::{DbError, DbResult};

/// マイグレーションの方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationDirection {
    /// 適用（up）
    Up,
    /// ロールバック（down）
    Down,
}

impl MigrationDirection {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

/// マイグレーション設定
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// マイグレーションファイルのディレクトリ
    pub migrations_dir: PathBuf,
    /// マイグレーションテーブル名
    pub table_name: String,
    /// ドライランモード（実際には適用しない）
    pub dry_run: bool,
}

impl MigrationConfig {
    /// 新しい設定を作成
    pub fn new(migrations_dir: impl Into<PathBuf>) -> Self {
        Self {
            migrations_dir: migrations_dir.into(),
            table_name: "_k1s0_migrations".to_string(),
            dry_run: false,
        }
    }

    /// テーブル名を設定
    pub fn with_table_name(mut self, name: impl Into<String>) -> Self {
        self.table_name = name.into();
        self
    }

    /// ドライランモードを設定
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> DbResult<()> {
        if !self.migrations_dir.exists() {
            return Err(DbError::migration(format!(
                "migrations directory not found: {}",
                self.migrations_dir.display()
            )));
        }
        if self.table_name.is_empty() {
            return Err(DbError::config("migration table name cannot be empty"));
        }
        Ok(())
    }
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self::new("migrations")
    }
}

/// マイグレーション情報
#[derive(Debug, Clone)]
pub struct Migration {
    /// バージョン（ファイル名のプレフィックス）
    pub version: String,
    /// 名前（ファイル名から抽出）
    pub name: String,
    /// up スクリプトのパス
    pub up_path: PathBuf,
    /// down スクリプトのパス（存在する場合）
    pub down_path: Option<PathBuf>,
}

impl Migration {
    /// ファイル名からマイグレーション情報を作成
    ///
    /// 形式: `{version}_{name}.sql` (例: `0001_create_users.sql`)
    pub fn from_filename(path: &Path) -> DbResult<Self> {
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| DbError::migration("invalid migration filename"))?;

        let parts: Vec<&str> = filename.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err(DbError::migration(format!(
                "invalid migration filename format: {} (expected: {{version}}_{{name}}.sql)",
                filename
            )));
        }

        let version = parts[0].to_string();
        let name = parts[1].to_string();

        Ok(Self {
            version,
            name,
            up_path: path.to_path_buf(),
            down_path: None,
        })
    }

    /// down スクリプトのパスを設定
    pub fn with_down_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.down_path = Some(path.into());
        self
    }

    /// 表示名を取得
    pub fn display_name(&self) -> String {
        format!("{}_{}", self.version, self.name)
    }
}

/// 適用済みマイグレーション
#[derive(Debug, Clone)]
pub struct AppliedMigration {
    /// バージョン
    pub version: String,
    /// 適用日時（Unix タイムスタンプ）
    pub applied_at: i64,
    /// チェックサム（ファイルハッシュ）
    pub checksum: Option<String>,
}

/// マイグレーション実行結果
#[derive(Debug, Clone)]
pub struct MigrationResult {
    /// 適用されたマイグレーション数
    pub applied_count: usize,
    /// スキップされたマイグレーション数
    pub skipped_count: usize,
    /// 適用されたマイグレーションのバージョンリスト
    pub applied_versions: Vec<String>,
}

impl MigrationResult {
    /// 新しい結果を作成
    pub fn new() -> Self {
        Self {
            applied_count: 0,
            skipped_count: 0,
            applied_versions: Vec::new(),
        }
    }

    /// 適用を記録
    pub fn record_applied(&mut self, version: String) {
        self.applied_count += 1;
        self.applied_versions.push(version);
    }

    /// スキップを記録
    pub fn record_skipped(&mut self) {
        self.skipped_count += 1;
    }

    /// 変更があったかどうか
    pub fn has_changes(&self) -> bool {
        self.applied_count > 0
    }
}

impl Default for MigrationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// マイグレーションランナートレイト
///
/// 実際のマイグレーション実行は infrastructure 層で実装する。
pub trait MigrationRunner: Send + Sync {
    /// マイグレーションを適用
    fn migrate(&self, config: &MigrationConfig) -> impl std::future::Future<Output = DbResult<MigrationResult>> + Send;

    /// マイグレーションをロールバック
    fn rollback(&self, config: &MigrationConfig, steps: usize) -> impl std::future::Future<Output = DbResult<MigrationResult>> + Send;

    /// 適用済みマイグレーションを取得
    fn get_applied(&self, config: &MigrationConfig) -> impl std::future::Future<Output = DbResult<Vec<AppliedMigration>>> + Send;

    /// 保留中のマイグレーションを取得
    fn get_pending(&self, config: &MigrationConfig) -> impl std::future::Future<Output = DbResult<Vec<Migration>>> + Send;
}

/// マイグレーションファイルを読み込むユーティリティ
pub fn load_migrations(migrations_dir: &Path) -> DbResult<Vec<Migration>> {
    if !migrations_dir.exists() {
        return Err(DbError::migration(format!(
            "migrations directory not found: {}",
            migrations_dir.display()
        )));
    }

    let mut migrations = Vec::new();
    let entries = std::fs::read_dir(migrations_dir).map_err(|e| {
        DbError::migration(format!(
            "failed to read migrations directory: {}",
            e
        ))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            DbError::migration(format!("failed to read directory entry: {}", e))
        })?;

        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "sql" {
                    let migration = Migration::from_filename(&path)?;
                    migrations.push(migration);
                }
            }
        }
    }

    // バージョンでソート
    migrations.sort_by(|a, b| a.version.cmp(&b.version));

    Ok(migrations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_migration_direction() {
        assert_eq!(MigrationDirection::Up.as_str(), "up");
        assert_eq!(MigrationDirection::Down.as_str(), "down");
    }

    #[test]
    fn test_migration_config() {
        let config = MigrationConfig::new("./migrations")
            .with_table_name("my_migrations")
            .dry_run(true);

        assert_eq!(config.table_name, "my_migrations");
        assert!(config.dry_run);
    }

    #[test]
    fn test_migration_from_filename() {
        let path = Path::new("migrations/0001_create_users.sql");
        let migration = Migration::from_filename(path).unwrap();

        assert_eq!(migration.version, "0001");
        assert_eq!(migration.name, "create_users");
        assert_eq!(migration.display_name(), "0001_create_users");
    }

    #[test]
    fn test_migration_from_filename_invalid() {
        // アンダースコアなし
        let path = Path::new("migrations/0001.sql");
        assert!(Migration::from_filename(path).is_err());
    }

    #[test]
    fn test_migration_result() {
        let mut result = MigrationResult::new();
        assert!(!result.has_changes());

        result.record_applied("0001".to_string());
        result.record_applied("0002".to_string());
        result.record_skipped();

        assert!(result.has_changes());
        assert_eq!(result.applied_count, 2);
        assert_eq!(result.skipped_count, 1);
        assert_eq!(result.applied_versions, vec!["0001", "0002"]);
    }

    #[test]
    fn test_load_migrations() {
        let temp_dir = TempDir::new().unwrap();
        let migrations_dir = temp_dir.path().join("migrations");
        fs::create_dir(&migrations_dir).unwrap();

        // マイグレーションファイルを作成
        fs::write(
            migrations_dir.join("0001_create_users.sql"),
            "CREATE TABLE users (id INT);",
        ).unwrap();
        fs::write(
            migrations_dir.join("0002_add_email.sql"),
            "ALTER TABLE users ADD email VARCHAR;",
        ).unwrap();

        let migrations = load_migrations(&migrations_dir).unwrap();
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].version, "0001");
        assert_eq!(migrations[1].version, "0002");
    }

    #[test]
    fn test_load_migrations_not_found() {
        let result = load_migrations(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
