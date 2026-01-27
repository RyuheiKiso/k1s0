//! テスト支援ユーティリティ
//!
//! データベーステストのためのユーティリティを提供する。
//!
//! # 機能
//!
//! - **TestDb**: テスト用データベースの自動作成・破棄
//! - **TestTransaction**: テスト用トランザクション（自動ロールバック）
//! - **Fixtures**: テストデータの投入
//!
//! # 使用例
//!
//! ```rust,ignore
//! use k1s0_db::testing::{TestDb, TestTransaction};
//!
//! #[tokio::test]
//! async fn test_user_creation() {
//!     // テスト用データベースを作成
//!     let test_db = TestDb::new("postgres://localhost/test_db_template").await.unwrap();
//!
//!     // テスト用トランザクションを開始
//!     let tx = TestTransaction::begin(&test_db.pool()).await.unwrap();
//!
//!     // テストコード...
//!
//!     // tx がドロップされると自動的にロールバック
//! }
//! ```

use crate::error::{DbError, DbResult};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{debug, info};

/// テスト用データベース名のカウンター
static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

/// テスト用データベース名を生成
pub fn generate_test_db_name(prefix: &str) -> String {
    let counter = TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{}_{}_{}_{}", prefix, std::process::id(), timestamp, counter)
}

/// テスト用データベース設定
#[derive(Debug, Clone)]
pub struct TestDbConfig {
    /// テンプレートデータベース名
    pub template_db: Option<String>,
    /// テストデータベース名のプレフィックス
    pub db_name_prefix: String,
    /// 終了時にデータベースを削除するかどうか
    pub drop_on_close: bool,
    /// マイグレーションを実行するかどうか
    pub run_migrations: bool,
    /// マイグレーションディレクトリ
    pub migrations_path: Option<String>,
}

impl Default for TestDbConfig {
    fn default() -> Self {
        Self {
            template_db: None,
            db_name_prefix: "test_db".to_string(),
            drop_on_close: true,
            run_migrations: false,
            migrations_path: None,
        }
    }
}

impl TestDbConfig {
    /// 新しい設定を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// テンプレートデータベースを設定
    pub fn with_template(mut self, template: impl Into<String>) -> Self {
        self.template_db = Some(template.into());
        self
    }

    /// データベース名プレフィックスを設定
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.db_name_prefix = prefix.into();
        self
    }

    /// 終了時の削除を無効化
    pub fn keep_db_on_close(mut self) -> Self {
        self.drop_on_close = false;
        self
    }

    /// マイグレーションを有効化
    pub fn with_migrations(mut self, path: impl Into<String>) -> Self {
        self.run_migrations = true;
        self.migrations_path = Some(path.into());
        self
    }
}

/// テストトランザクション
///
/// テスト用のトランザクションラッパー。
/// ドロップ時に自動的にロールバックする。
#[cfg(feature = "postgres")]
pub struct TestTransaction<'a> {
    tx: Option<sqlx::Transaction<'a, sqlx::Postgres>>,
}

#[cfg(feature = "postgres")]
impl<'a> TestTransaction<'a> {
    /// テストトランザクションを開始
    pub async fn begin(pool: &'a crate::postgres::PgPool) -> DbResult<Self> {
        let tx = pool
            .begin()
            .await
            .map_err(|e| DbError::transaction(format!("Failed to begin transaction: {}", e)))?;

        debug!("Test transaction started");
        Ok(Self { tx: Some(tx) })
    }

    /// トランザクションへの参照を取得
    pub fn as_ref(&mut self) -> &mut sqlx::Transaction<'a, sqlx::Postgres> {
        self.tx.as_mut().expect("Transaction already consumed")
    }

    /// 明示的にロールバック（通常は不要 - ドロップ時に自動実行）
    pub async fn rollback(mut self) -> DbResult<()> {
        if let Some(tx) = self.tx.take() {
            tx.rollback()
                .await
                .map_err(|e| DbError::transaction(format!("Failed to rollback: {}", e)))?;
            debug!("Test transaction rolled back");
        }
        Ok(())
    }

    /// テスト目的でコミット（通常のテストでは使用しない）
    pub async fn commit(mut self) -> DbResult<()> {
        if let Some(tx) = self.tx.take() {
            tx.commit()
                .await
                .map_err(|e| DbError::transaction(format!("Failed to commit: {}", e)))?;
            debug!("Test transaction committed");
        }
        Ok(())
    }
}

#[cfg(feature = "postgres")]
impl Drop for TestTransaction<'_> {
    fn drop(&mut self) {
        if self.tx.is_some() {
            debug!("Test transaction dropped without explicit rollback/commit - will be rolled back");
        }
    }
}

/// フィクスチャデータ
#[derive(Debug, Clone)]
pub struct Fixture {
    /// テーブル名
    pub table: String,
    /// カラム名
    pub columns: Vec<String>,
    /// データ行
    pub rows: Vec<Vec<String>>,
}

impl Fixture {
    /// 新しいフィクスチャを作成
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// カラムを設定
    pub fn columns(mut self, columns: &[&str]) -> Self {
        self.columns = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// データ行を追加
    pub fn row(mut self, values: &[&str]) -> Self {
        self.rows.push(values.iter().map(|s| s.to_string()).collect());
        self
    }

    /// INSERT文を生成
    pub fn to_insert_sql(&self) -> String {
        if self.rows.is_empty() {
            return String::new();
        }

        let columns_str = self.columns.join(", ");
        let values_parts: Vec<String> = self
            .rows
            .iter()
            .map(|row| {
                let values: Vec<String> = row.iter().map(|v| format!("'{}'", v)).collect();
                format!("({})", values.join(", "))
            })
            .collect();

        format!(
            "INSERT INTO {} ({}) VALUES {}",
            self.table,
            columns_str,
            values_parts.join(", ")
        )
    }
}

/// フィクスチャローダー
#[cfg(feature = "postgres")]
pub struct FixtureLoader<'a> {
    pool: &'a crate::postgres::PgPool,
}

#[cfg(feature = "postgres")]
impl<'a> FixtureLoader<'a> {
    /// 新しいローダーを作成
    pub fn new(pool: &'a crate::postgres::PgPool) -> Self {
        Self { pool }
    }

    /// フィクスチャをロード
    pub async fn load(&self, fixture: &Fixture) -> DbResult<()> {
        let sql = fixture.to_insert_sql();
        if sql.is_empty() {
            return Ok(());
        }

        sqlx::query(&sql)
            .execute(self.pool)
            .await
            .map_err(|e| DbError::query(format!("Failed to load fixture: {}", e)))?;

        info!(table = %fixture.table, rows = fixture.rows.len(), "Fixture loaded");
        Ok(())
    }

    /// 複数のフィクスチャをロード
    pub async fn load_all(&self, fixtures: &[Fixture]) -> DbResult<()> {
        for fixture in fixtures {
            self.load(fixture).await?;
        }
        Ok(())
    }

    /// テーブルをクリア
    pub async fn truncate(&self, table: &str) -> DbResult<()> {
        let sql = format!("TRUNCATE TABLE {} CASCADE", table);
        sqlx::query(&sql)
            .execute(self.pool)
            .await
            .map_err(|e| DbError::query(format!("Failed to truncate table: {}", e)))?;

        debug!(table = %table, "Table truncated");
        Ok(())
    }

    /// 複数のテーブルをクリア
    pub async fn truncate_all(&self, tables: &[&str]) -> DbResult<()> {
        let tables_str = tables.join(", ");
        let sql = format!("TRUNCATE TABLE {} CASCADE", tables_str);
        sqlx::query(&sql)
            .execute(self.pool)
            .await
            .map_err(|e| DbError::query(format!("Failed to truncate tables: {}", e)))?;

        debug!(tables = %tables_str, "Tables truncated");
        Ok(())
    }
}

/// テストアサーションヘルパー
pub mod assert {
    use super::*;

    /// レコード数をアサート
    #[cfg(feature = "postgres")]
    pub async fn record_count(
        pool: &crate::postgres::PgPool,
        table: &str,
        expected: i64,
    ) -> DbResult<()> {
        let sql = format!("SELECT COUNT(*) as count FROM {}", table);
        let row: (i64,) = sqlx::query_as(&sql)
            .fetch_one(pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        assert_eq!(
            row.0, expected,
            "Expected {} records in {}, found {}",
            expected, table, row.0
        );
        Ok(())
    }

    /// レコードが存在することをアサート
    #[cfg(feature = "postgres")]
    pub async fn record_exists(
        pool: &crate::postgres::PgPool,
        table: &str,
        id_column: &str,
        id: &str,
    ) -> DbResult<()> {
        let sql = format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)",
            table, id_column
        );
        let row: (bool,) = sqlx::query_as(&sql)
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        assert!(
            row.0,
            "Expected record with {} = {} to exist in {}",
            id_column, id, table
        );
        Ok(())
    }

    /// レコードが存在しないことをアサート
    #[cfg(feature = "postgres")]
    pub async fn record_not_exists(
        pool: &crate::postgres::PgPool,
        table: &str,
        id_column: &str,
        id: &str,
    ) -> DbResult<()> {
        let sql = format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)",
            table, id_column
        );
        let row: (bool,) = sqlx::query_as(&sql)
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| DbError::query(e.to_string()))?;

        assert!(
            !row.0,
            "Expected record with {} = {} to not exist in {}",
            id_column, id, table
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_db_name() {
        let name1 = generate_test_db_name("test");
        let name2 = generate_test_db_name("test");

        assert!(name1.starts_with("test_"));
        assert_ne!(name1, name2);
    }

    #[test]
    fn test_fixture_to_sql() {
        let fixture = Fixture::new("users")
            .columns(&["id", "name", "email"])
            .row(&["1", "Alice", "alice@example.com"])
            .row(&["2", "Bob", "bob@example.com"]);

        let sql = fixture.to_insert_sql();

        assert!(sql.contains("INSERT INTO users"));
        assert!(sql.contains("(id, name, email)"));
        assert!(sql.contains("('1', 'Alice', 'alice@example.com')"));
        assert!(sql.contains("('2', 'Bob', 'bob@example.com')"));
    }

    #[test]
    fn test_fixture_empty() {
        let fixture = Fixture::new("users").columns(&["id", "name"]);

        let sql = fixture.to_insert_sql();
        assert!(sql.is_empty());
    }

    #[test]
    fn test_test_db_config() {
        let config = TestDbConfig::new()
            .with_template("template_db")
            .with_prefix("my_test")
            .with_migrations("./migrations")
            .keep_db_on_close();

        assert_eq!(config.template_db, Some("template_db".to_string()));
        assert_eq!(config.db_name_prefix, "my_test");
        assert!(config.run_migrations);
        assert!(!config.drop_on_close);
    }
}
