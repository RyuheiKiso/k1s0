//! PostgreSQL 統合
//!
//! sqlx を使用した PostgreSQL データベースアクセスを提供する。
//!
//! # 機能
//!
//! - コネクションプールの管理
//! - トランザクション実行
//! - Unit of Work パターンの実装
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_db::postgres::{PgPool, create_pool};
//! use k1s0_db::DbConfig;
//!
//! // プールの作成
//! let config = DbConfig::builder()
//!     .host("localhost")
//!     .database("myapp")
//!     .username("app_user")
//!     .password_file("/run/secrets/db_password")
//!     .build()?;
//!
//! let pool = create_pool(&config, "secret").await?;
//!
//! // クエリの実行
//! let users = sqlx::query_as!(User, "SELECT * FROM users")
//!     .fetch_all(&*pool)
//!     .await?;
//! ```

#[cfg(feature = "postgres")]
pub use postgres_impl::*;

#[cfg(feature = "postgres")]
mod postgres_impl {
    use crate::config::{DbConfig, PoolConfig, TimeoutConfig};
    use crate::error::{DbError, DbResult};
    use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
    use sqlx::{Pool, Postgres};
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;

    /// PostgreSQL コネクションプール
    pub type PgPool = Pool<Postgres>;

    /// PostgreSQL コネクションプールのラッパー
    ///
    /// プールのライフサイクル管理と追加機能を提供する。
    #[derive(Clone)]
    pub struct PostgresPool {
        pool: Arc<PgPool>,
        config: DbConfig,
    }

    impl PostgresPool {
        /// 新しいプールを作成
        pub async fn new(config: &DbConfig, password: &str) -> DbResult<Self> {
            let pool = create_pool(config, password).await?;
            Ok(Self {
                pool: Arc::new(pool),
                config: config.clone(),
            })
        }

        /// 内部プールへの参照を取得
        pub fn inner(&self) -> &PgPool {
            &self.pool
        }

        /// 設定を取得
        pub fn config(&self) -> &DbConfig {
            &self.config
        }

        /// プールの状態を取得
        pub fn status(&self) -> PoolStatus {
            PoolStatus {
                size: self.pool.size(),
                idle: self.pool.num_idle(),
                max_size: self.config.pool.max_connections,
            }
        }

        /// ヘルスチェック
        pub async fn health_check(&self) -> DbResult<()> {
            sqlx::query("SELECT 1")
                .execute(&*self.pool)
                .await
                .map_err(|e| DbError::connection(e.to_string()))?;
            Ok(())
        }

        /// プールを閉じる
        pub async fn close(&self) {
            self.pool.close().await;
        }

        /// プールが閉じられたかどうか
        pub fn is_closed(&self) -> bool {
            self.pool.is_closed()
        }
    }

    impl std::ops::Deref for PostgresPool {
        type Target = PgPool;

        fn deref(&self) -> &Self::Target {
            &self.pool
        }
    }

    /// プール状態
    #[derive(Debug, Clone)]
    pub struct PoolStatus {
        /// 現在のコネクション数
        pub size: u32,
        /// アイドルコネクション数
        pub idle: usize,
        /// 最大コネクション数
        pub max_size: u32,
    }

    impl PoolStatus {
        /// 使用中のコネクション数
        pub fn in_use(&self) -> usize {
            self.size as usize - self.idle
        }

        /// 使用率（0.0 - 1.0）
        pub fn utilization(&self) -> f64 {
            if self.max_size == 0 {
                0.0
            } else {
                self.in_use() as f64 / self.max_size as f64
            }
        }
    }

    /// コネクションプールを作成
    ///
    /// # 引数
    ///
    /// * `config` - データベース設定
    /// * `password` - データベースパスワード
    ///
    /// # 戻り値
    ///
    /// * `Ok(PgPool)` - 作成されたプール
    /// * `Err(DbError)` - 作成に失敗
    pub async fn create_pool(config: &DbConfig, password: &str) -> DbResult<PgPool> {
        config.validate()?;

        let connect_options = create_connect_options(config, password)?;
        let pool_options = create_pool_options(&config.pool, &config.timeout);

        pool_options
            .connect_with(connect_options)
            .await
            .map_err(|e| DbError::connection(format!("failed to create pool: {}", e)))
    }

    /// 接続オプションを作成
    fn create_connect_options(config: &DbConfig, password: &str) -> DbResult<PgConnectOptions> {
        let ssl_mode = match config.ssl_mode {
            crate::config::SslMode::Disable => PgSslMode::Disable,
            crate::config::SslMode::Prefer => PgSslMode::Prefer,
            crate::config::SslMode::Require => PgSslMode::Require,
            crate::config::SslMode::VerifyCa => PgSslMode::VerifyCa,
            crate::config::SslMode::VerifyFull => PgSslMode::VerifyFull,
        };

        let options = PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .database(&config.database)
            .username(&config.username)
            .password(password)
            .ssl_mode(ssl_mode);

        Ok(options)
    }

    /// プールオプションを作成
    fn create_pool_options(pool: &PoolConfig, timeout: &TimeoutConfig) -> PgPoolOptions {
        PgPoolOptions::new()
            .max_connections(pool.max_connections)
            .min_connections(pool.min_connections)
            .acquire_timeout(Duration::from_millis(timeout.connect_timeout_ms))
            .idle_timeout(Some(Duration::from_secs(pool.idle_timeout_secs)))
            .max_lifetime(Some(Duration::from_secs(pool.max_lifetime_secs)))
    }

    /// 接続文字列からプールを作成
    ///
    /// 完全な接続文字列を使用してプールを作成する。
    pub async fn create_pool_from_url(url: &str, pool_config: &PoolConfig) -> DbResult<PgPool> {
        let connect_options = PgConnectOptions::from_str(url)
            .map_err(|e| DbError::config(format!("invalid connection string: {}", e)))?;

        PgPoolOptions::new()
            .max_connections(pool_config.max_connections)
            .min_connections(pool_config.min_connections)
            .idle_timeout(Some(Duration::from_secs(pool_config.idle_timeout_secs)))
            .max_lifetime(Some(Duration::from_secs(pool_config.max_lifetime_secs)))
            .connect_with(connect_options)
            .await
            .map_err(|e| DbError::connection(format!("failed to create pool: {}", e)))
    }

    /// トランザクションガード
    ///
    /// トランザクションのライフサイクルを管理する。
    /// ドロップ時にコミットされていなければロールバックする。
    pub struct TransactionGuard<'a> {
        tx: Option<sqlx::Transaction<'a, Postgres>>,
        committed: bool,
    }

    impl<'a> TransactionGuard<'a> {
        /// 新しいトランザクションガードを作成
        pub fn new(tx: sqlx::Transaction<'a, Postgres>) -> Self {
            Self {
                tx: Some(tx),
                committed: false,
            }
        }

        /// トランザクションへの参照を取得
        pub fn as_ref(&mut self) -> &mut sqlx::Transaction<'a, Postgres> {
            self.tx.as_mut().expect("transaction already consumed")
        }

        /// コミット
        pub async fn commit(mut self) -> DbResult<()> {
            if let Some(tx) = self.tx.take() {
                tx.commit()
                    .await
                    .map_err(|e| DbError::transaction(format!("commit failed: {}", e)))?;
                self.committed = true;
            }
            Ok(())
        }

        /// ロールバック
        pub async fn rollback(mut self) -> DbResult<()> {
            if let Some(tx) = self.tx.take() {
                tx.rollback()
                    .await
                    .map_err(|e| DbError::transaction(format!("rollback failed: {}", e)))?;
            }
            Ok(())
        }
    }

    impl Drop for TransactionGuard<'_> {
        fn drop(&mut self) {
            if self.tx.is_some() && !self.committed {
                tracing::warn!("transaction dropped without commit, will be rolled back");
            }
        }
    }

    /// トランザクションを開始
    pub async fn begin_transaction(pool: &PgPool) -> DbResult<TransactionGuard<'_>> {
        let tx = pool
            .begin()
            .await
            .map_err(|e| DbError::transaction(format!("begin failed: {}", e)))?;
        Ok(TransactionGuard::new(tx))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_pool_status() {
            let status = PoolStatus {
                size: 5,
                idle: 2,
                max_size: 10,
            };

            assert_eq!(status.in_use(), 3);
            assert!((status.utilization() - 0.3).abs() < 0.001);
        }

        #[test]
        fn test_pool_status_zero_max() {
            let status = PoolStatus {
                size: 0,
                idle: 0,
                max_size: 0,
            };

            assert_eq!(status.utilization(), 0.0);
        }
    }
}
