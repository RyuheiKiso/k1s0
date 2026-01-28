//! fw_m_setting テーブルの PostgreSQL 実装
//!
//! このモジュールは `k1s0_config::db::DbSettingRepository` トレイトの
//! PostgreSQL 実装を提供する。
//!
//! # テーブル定義
//!
//! ```sql
//! CREATE TABLE fw_m_setting (
//!     key VARCHAR(255) PRIMARY KEY,
//!     value TEXT NOT NULL,
//!     updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
//! );
//!
//! -- インデックス（プレフィックス検索用）
//! CREATE INDEX idx_fw_m_setting_key_prefix ON fw_m_setting (key varchar_pattern_ops);
//! ```
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_db::setting::PostgresSettingRepository;
//! use k1s0_config::db::{DbSettingRepository, DbConfigLoader};
//!
//! // PostgreSQL プールから作成
//! let setting_repo = PostgresSettingRepository::new(pool.clone());
//!
//! // 単独で使用
//! let entries = setting_repo.get_all().await?;
//! let entry = setting_repo.get("http.timeout_ms").await?;
//!
//! // DbConfigLoader と組み合わせて使用
//! let db_loader = DbConfigLoader::new(yaml_loader, Box::new(setting_repo));
//! let config: AppConfig = db_loader.load().await?;
//! ```

#[cfg(all(feature = "postgres", feature = "config"))]
pub use postgres_setting_impl::*;

#[cfg(all(feature = "postgres", feature = "config"))]
mod postgres_setting_impl {
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use k1s0_config::db::{DbSettingError, DbSettingRepository, SettingEntry};
    use sqlx::FromRow;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::RwLock;

    use crate::postgres::PgPool;

    /// fw_m_setting テーブルの行
    #[derive(Debug, Clone, FromRow)]
    struct SettingRow {
        key: String,
        value: String,
        updated_at: DateTime<Utc>,
    }

    impl From<SettingRow> for SettingEntry {
        fn from(row: SettingRow) -> Self {
            SettingEntry::with_updated_at(row.key, row.value, row.updated_at)
        }
    }

    /// PostgreSQL ベースの設定リポジトリ
    ///
    /// `fw_m_setting` テーブルからの設定取得を実装する。
    #[derive(Clone)]
    pub struct PostgresSettingRepository {
        pool: Arc<PgPool>,
        table_name: String,
        cache: Arc<RwLock<Option<CachedSettings>>>,
        cache_ttl: Duration,
    }

    /// キャッシュされた設定
    #[derive(Debug, Clone)]
    struct CachedSettings {
        entries: Vec<SettingEntry>,
        cached_at: std::time::Instant,
    }

    impl PostgresSettingRepository {
        /// 新しいリポジトリを作成
        pub fn new(pool: Arc<PgPool>) -> Self {
            Self {
                pool,
                table_name: "fw_m_setting".to_string(),
                cache: Arc::new(RwLock::new(None)),
                cache_ttl: Duration::from_secs(60), // デフォルト: 1分
            }
        }

        /// テーブル名をカスタマイズ
        pub fn with_table_name(mut self, name: impl Into<String>) -> Self {
            self.table_name = name.into();
            self
        }

        /// キャッシュ TTL を設定
        pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
            self.cache_ttl = ttl;
            self
        }

        /// キャッシュを無効化
        pub async fn invalidate_cache(&self) {
            let mut cache = self.cache.write().await;
            *cache = None;
        }

        /// キャッシュから取得（有効期限内なら）
        async fn get_from_cache(&self) -> Option<Vec<SettingEntry>> {
            let cache = self.cache.read().await;
            if let Some(cached) = &*cache {
                if cached.cached_at.elapsed() < self.cache_ttl {
                    return Some(cached.entries.clone());
                }
            }
            None
        }

        /// キャッシュを更新
        async fn update_cache(&self, entries: Vec<SettingEntry>) {
            let mut cache = self.cache.write().await;
            *cache = Some(CachedSettings {
                entries,
                cached_at: std::time::Instant::now(),
            });
        }

        /// エラーを変換
        fn convert_error(e: sqlx::Error) -> DbSettingError {
            match e {
                sqlx::Error::PoolTimedOut => {
                    DbSettingError::connection("Database connection pool timeout")
                }
                sqlx::Error::PoolClosed => {
                    DbSettingError::connection("Database connection pool closed")
                }
                sqlx::Error::Io(_) => {
                    DbSettingError::retryable(format!("Database I/O error: {}", e))
                }
                _ => DbSettingError::query(format!("Database query error: {}", e)),
            }
        }
    }

    #[async_trait]
    impl DbSettingRepository for PostgresSettingRepository {
        async fn get_all(&self) -> Result<Vec<SettingEntry>, DbSettingError> {
            // キャッシュをチェック
            if let Some(cached) = self.get_from_cache().await {
                return Ok(cached);
            }

            // クエリ実行
            let query = format!(
                "SELECT key, value, updated_at FROM {} ORDER BY key",
                self.table_name
            );

            let rows: Vec<SettingRow> = sqlx::query_as(&query)
                .fetch_all(&*self.pool)
                .await
                .map_err(Self::convert_error)?;

            let entries: Vec<SettingEntry> = rows.into_iter().map(Into::into).collect();

            // キャッシュを更新
            self.update_cache(entries.clone()).await;

            Ok(entries)
        }

        async fn get(&self, key: &str) -> Result<Option<SettingEntry>, DbSettingError> {
            let query = format!(
                "SELECT key, value, updated_at FROM {} WHERE key = $1",
                self.table_name
            );

            let row: Option<SettingRow> = sqlx::query_as(&query)
                .bind(key)
                .fetch_optional(&*self.pool)
                .await
                .map_err(Self::convert_error)?;

            Ok(row.map(Into::into))
        }

        async fn get_by_prefix(&self, prefix: &str) -> Result<Vec<SettingEntry>, DbSettingError> {
            let query = format!(
                "SELECT key, value, updated_at FROM {} WHERE key LIKE $1 ORDER BY key",
                self.table_name
            );

            let pattern = format!("{}%", prefix);
            let rows: Vec<SettingRow> = sqlx::query_as(&query)
                .bind(&pattern)
                .fetch_all(&*self.pool)
                .await
                .map_err(Self::convert_error)?;

            Ok(rows.into_iter().map(Into::into).collect())
        }

        async fn health_check(&self) -> Result<(), DbSettingError> {
            let query = format!("SELECT 1 FROM {} LIMIT 1", self.table_name);

            sqlx::query(&query)
                .fetch_optional(&*self.pool)
                .await
                .map_err(Self::convert_error)?;

            Ok(())
        }
    }

    /// 設定リポジトリビルダー
    pub struct PostgresSettingRepositoryBuilder {
        pool: Arc<PgPool>,
        table_name: Option<String>,
        cache_ttl: Option<Duration>,
    }

    impl PostgresSettingRepositoryBuilder {
        /// 新しいビルダーを作成
        pub fn new(pool: Arc<PgPool>) -> Self {
            Self {
                pool,
                table_name: None,
                cache_ttl: None,
            }
        }

        /// テーブル名を設定
        pub fn table_name(mut self, name: impl Into<String>) -> Self {
            self.table_name = Some(name.into());
            self
        }

        /// キャッシュ TTL を設定
        pub fn cache_ttl(mut self, ttl: Duration) -> Self {
            self.cache_ttl = Some(ttl);
            self
        }

        /// リポジトリを構築
        pub fn build(self) -> PostgresSettingRepository {
            let mut repo = PostgresSettingRepository::new(self.pool);

            if let Some(name) = self.table_name {
                repo = repo.with_table_name(name);
            }

            if let Some(ttl) = self.cache_ttl {
                repo = repo.with_cache_ttl(ttl);
            }

            repo
        }
    }

    /// 設定の書き込み操作（管理用）
    ///
    /// 通常のアプリケーションでは読み取りのみを行い、
    /// 書き込みは管理ツールまたはマイグレーションで行う。
    pub struct PostgresSettingWriter {
        pool: Arc<PgPool>,
        table_name: String,
    }

    impl PostgresSettingWriter {
        /// 新しいライターを作成
        pub fn new(pool: Arc<PgPool>) -> Self {
            Self {
                pool,
                table_name: "fw_m_setting".to_string(),
            }
        }

        /// テーブル名をカスタマイズ
        pub fn with_table_name(mut self, name: impl Into<String>) -> Self {
            self.table_name = name.into();
            self
        }

        /// 設定を挿入または更新（UPSERT）
        pub async fn upsert(&self, key: &str, value: &str) -> Result<(), DbSettingError> {
            let query = format!(
                r#"
                INSERT INTO {} (key, value, updated_at)
                VALUES ($1, $2, CURRENT_TIMESTAMP)
                ON CONFLICT (key) DO UPDATE SET
                    value = EXCLUDED.value,
                    updated_at = CURRENT_TIMESTAMP
                "#,
                self.table_name
            );

            sqlx::query(&query)
                .bind(key)
                .bind(value)
                .execute(&*self.pool)
                .await
                .map_err(PostgresSettingRepository::convert_error)?;

            Ok(())
        }

        /// 設定を削除
        pub async fn delete(&self, key: &str) -> Result<bool, DbSettingError> {
            let query = format!("DELETE FROM {} WHERE key = $1", self.table_name);

            let result = sqlx::query(&query)
                .bind(key)
                .execute(&*self.pool)
                .await
                .map_err(PostgresSettingRepository::convert_error)?;

            Ok(result.rows_affected() > 0)
        }

        /// プレフィックスで削除
        pub async fn delete_by_prefix(&self, prefix: &str) -> Result<u64, DbSettingError> {
            let query = format!("DELETE FROM {} WHERE key LIKE $1", self.table_name);
            let pattern = format!("{}%", prefix);

            let result = sqlx::query(&query)
                .bind(&pattern)
                .execute(&*self.pool)
                .await
                .map_err(PostgresSettingRepository::convert_error)?;

            Ok(result.rows_affected())
        }

        /// 複数の設定を一括挿入
        pub async fn bulk_upsert(&self, entries: &[(&str, &str)]) -> Result<(), DbSettingError> {
            if entries.is_empty() {
                return Ok(());
            }

            // トランザクションで実行
            let mut tx = self.pool.begin().await.map_err(|e| {
                DbSettingError::query(format!("Failed to begin transaction: {}", e))
            })?;

            for (key, value) in entries {
                let query = format!(
                    r#"
                    INSERT INTO {} (key, value, updated_at)
                    VALUES ($1, $2, CURRENT_TIMESTAMP)
                    ON CONFLICT (key) DO UPDATE SET
                        value = EXCLUDED.value,
                        updated_at = CURRENT_TIMESTAMP
                    "#,
                    self.table_name
                );

                sqlx::query(&query)
                    .bind(key)
                    .bind(value)
                    .execute(&mut *tx)
                    .await
                    .map_err(PostgresSettingRepository::convert_error)?;
            }

            tx.commit().await.map_err(|e| {
                DbSettingError::query(format!("Failed to commit transaction: {}", e))
            })?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_setting_row_conversion() {
            let row = SettingRow {
                key: "http.timeout_ms".to_string(),
                value: "5000".to_string(),
                updated_at: Utc::now(),
            };

            let entry: SettingEntry = row.clone().into();
            assert_eq!(entry.key, "http.timeout_ms");
            assert_eq!(entry.value, "5000");
        }

        #[test]
        fn test_builder() {
            // プールなしではテストできないが、ビルダーのAPIは確認できる
            // 実際のテストはインテグレーションテストで行う
        }
    }
}

/// マイグレーション SQL
///
/// fw_m_setting テーブルを作成するマイグレーション SQL。
pub const MIGRATION_SQL: &str = r#"
-- fw_m_setting テーブルの作成
CREATE TABLE IF NOT EXISTS fw_m_setting (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- プレフィックス検索用インデックス
CREATE INDEX IF NOT EXISTS idx_fw_m_setting_key_prefix
ON fw_m_setting (key varchar_pattern_ops);

-- 更新日時自動更新トリガー
CREATE OR REPLACE FUNCTION update_fw_m_setting_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_fw_m_setting_updated_at ON fw_m_setting;
CREATE TRIGGER trg_fw_m_setting_updated_at
    BEFORE UPDATE ON fw_m_setting
    FOR EACH ROW
    EXECUTE FUNCTION update_fw_m_setting_updated_at();

-- コメント
COMMENT ON TABLE fw_m_setting IS 'k1s0 framework settings table';
COMMENT ON COLUMN fw_m_setting.key IS 'Setting key in format: category.name (e.g., http.timeout_ms)';
COMMENT ON COLUMN fw_m_setting.value IS 'Setting value as JSON or plain text';
COMMENT ON COLUMN fw_m_setting.updated_at IS 'Last update timestamp';
"#;

/// マイグレーションのロールバック SQL
pub const ROLLBACK_SQL: &str = r#"
DROP TRIGGER IF EXISTS trg_fw_m_setting_updated_at ON fw_m_setting;
DROP FUNCTION IF EXISTS update_fw_m_setting_updated_at();
DROP INDEX IF EXISTS idx_fw_m_setting_key_prefix;
DROP TABLE IF EXISTS fw_m_setting;
"#;
