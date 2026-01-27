//! データベースヘルスチェック
//!
//! k1s0-health との統合によるデータベースヘルスチェック機能を提供する。
//!
//! # 機能
//!
//! - データベース接続のヘルスチェック
//! - 接続プールの状態監視
//! - カスタムヘルスチェッククエリ
//!
//! # 使用例
//!
//! ```rust,ignore
//! use k1s0_db::health::{DbHealthChecker, DbHealthStatus};
//!
//! let checker = DbHealthChecker::new(pool.clone());
//! let status = checker.check().await;
//!
//! if status.is_healthy() {
//!     println!("Database is healthy");
//! }
//! ```

use crate::error::DbError;
use std::time::{Duration, Instant};

/// データベースヘルス状態
#[derive(Debug, Clone)]
pub struct DbHealthStatus {
    /// ヘルシーかどうか
    pub healthy: bool,
    /// レイテンシ（ミリ秒）
    pub latency_ms: u64,
    /// プール使用中接続数
    pub pool_in_use: Option<usize>,
    /// プールアイドル接続数
    pub pool_idle: Option<usize>,
    /// プール最大接続数
    pub pool_max: Option<u32>,
    /// エラーメッセージ（ある場合）
    pub error: Option<String>,
    /// 追加情報
    pub details: Option<String>,
}

impl DbHealthStatus {
    /// ヘルシーな状態を作成
    pub fn healthy(latency_ms: u64) -> Self {
        Self {
            healthy: true,
            latency_ms,
            pool_in_use: None,
            pool_idle: None,
            pool_max: None,
            error: None,
            details: None,
        }
    }

    /// アンヘルシーな状態を作成
    pub fn unhealthy(error: impl Into<String>) -> Self {
        Self {
            healthy: false,
            latency_ms: 0,
            pool_in_use: None,
            pool_idle: None,
            pool_max: None,
            error: Some(error.into()),
            details: None,
        }
    }

    /// ヘルシーかどうか
    pub fn is_healthy(&self) -> bool {
        self.healthy
    }

    /// プール状態を設定
    pub fn with_pool_status(mut self, in_use: usize, idle: usize, max: u32) -> Self {
        self.pool_in_use = Some(in_use);
        self.pool_idle = Some(idle);
        self.pool_max = Some(max);
        self
    }

    /// 追加情報を設定
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// プール使用率を取得（0.0 - 1.0）
    pub fn pool_utilization(&self) -> Option<f64> {
        match (self.pool_in_use, self.pool_max) {
            (Some(in_use), Some(max)) if max > 0 => Some(in_use as f64 / max as f64),
            _ => None,
        }
    }
}

/// ヘルスチェック設定
#[derive(Debug, Clone)]
pub struct DbHealthConfig {
    /// ヘルスチェッククエリ
    pub query: String,
    /// タイムアウト
    pub timeout: Duration,
    /// ヘルシーと見なす最大レイテンシ
    pub max_latency: Duration,
    /// ヘルシーと見なす最大プール使用率
    pub max_pool_utilization: f64,
}

impl Default for DbHealthConfig {
    fn default() -> Self {
        Self {
            query: "SELECT 1".to_string(),
            timeout: Duration::from_secs(5),
            max_latency: Duration::from_secs(1),
            max_pool_utilization: 0.9,
        }
    }
}

impl DbHealthConfig {
    /// 新しい設定を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// ヘルスチェッククエリを設定
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    /// タイムアウトを設定
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 最大レイテンシを設定
    pub fn with_max_latency(mut self, latency: Duration) -> Self {
        self.max_latency = latency;
        self
    }

    /// 最大プール使用率を設定
    pub fn with_max_pool_utilization(mut self, utilization: f64) -> Self {
        self.max_pool_utilization = utilization.clamp(0.0, 1.0);
        self
    }
}

/// データベースヘルスチェッカー（PostgreSQL用）
#[cfg(feature = "postgres")]
pub struct DbHealthChecker {
    pool: std::sync::Arc<crate::postgres::PgPool>,
    config: DbHealthConfig,
}

#[cfg(feature = "postgres")]
impl DbHealthChecker {
    /// 新しいヘルスチェッカーを作成
    pub fn new(pool: std::sync::Arc<crate::postgres::PgPool>) -> Self {
        Self {
            pool,
            config: DbHealthConfig::default(),
        }
    }

    /// 設定を指定して作成
    pub fn with_config(pool: std::sync::Arc<crate::postgres::PgPool>, config: DbHealthConfig) -> Self {
        Self { pool, config }
    }

    /// ヘルスチェックを実行
    pub async fn check(&self) -> DbHealthStatus {
        let start = Instant::now();

        // タイムアウト付きでクエリを実行
        let result = tokio::time::timeout(
            self.config.timeout,
            sqlx::query(&self.config.query).execute(&*self.pool),
        )
        .await;

        let latency = start.elapsed();
        let latency_ms = latency.as_millis() as u64;

        match result {
            Ok(Ok(_)) => {
                let mut status = DbHealthStatus::healthy(latency_ms);

                // プール状態を追加
                let pool_size = self.pool.size();
                let pool_idle = self.pool.num_idle();
                // max_connectionsの取得はプールの設定に依存するため、仮の値を使用
                let pool_max = pool_size; // 実際のmax_connectionsは設定から取得する必要がある
                status = status.with_pool_status(
                    pool_size as usize - pool_idle,
                    pool_idle,
                    pool_max,
                );

                // レイテンシチェック
                if latency > self.config.max_latency {
                    status.details = Some(format!(
                        "High latency: {}ms (threshold: {}ms)",
                        latency_ms,
                        self.config.max_latency.as_millis()
                    ));
                }

                status
            }
            Ok(Err(e)) => DbHealthStatus::unhealthy(format!("Query failed: {}", e)),
            Err(_) => DbHealthStatus::unhealthy("Health check timed out"),
        }
    }

    /// 接続性のみをチェック（高速）
    pub async fn ping(&self) -> bool {
        sqlx::query("SELECT 1")
            .execute(&*self.pool)
            .await
            .is_ok()
    }
}

/// ヘルスチェックトレイト
///
/// 異なるデータベース実装に対応するための抽象化。
#[async_trait::async_trait]
pub trait HealthCheckable: Send + Sync {
    /// ヘルスチェックを実行
    async fn health_check(&self) -> Result<DbHealthStatus, DbError>;

    /// 簡易ping
    async fn ping(&self) -> bool;
}

#[cfg(feature = "postgres")]
#[async_trait::async_trait]
impl HealthCheckable for crate::postgres::PostgresPool {
    async fn health_check(&self) -> Result<DbHealthStatus, DbError> {
        let start = Instant::now();

        sqlx::query("SELECT 1")
            .execute(self.inner())
            .await
            .map_err(|e| DbError::connection(e.to_string()))?;

        let latency_ms = start.elapsed().as_millis() as u64;
        let status = self.status();

        Ok(DbHealthStatus::healthy(latency_ms).with_pool_status(
            status.in_use(),
            status.idle,
            status.max_size,
        ))
    }

    async fn ping(&self) -> bool {
        sqlx::query("SELECT 1")
            .execute(self.inner())
            .await
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_healthy() {
        let status = DbHealthStatus::healthy(50);
        assert!(status.is_healthy());
        assert_eq!(status.latency_ms, 50);
        assert!(status.error.is_none());
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = DbHealthStatus::unhealthy("Connection failed");
        assert!(!status.is_healthy());
        assert!(status.error.is_some());
    }

    #[test]
    fn test_health_status_with_pool() {
        let status = DbHealthStatus::healthy(50).with_pool_status(5, 3, 10);

        assert_eq!(status.pool_in_use, Some(5));
        assert_eq!(status.pool_idle, Some(3));
        assert_eq!(status.pool_max, Some(10));
        assert!((status.pool_utilization().unwrap() - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_health_config() {
        let config = DbHealthConfig::new()
            .with_query("SELECT NOW()")
            .with_timeout(Duration::from_secs(3))
            .with_max_latency(Duration::from_millis(500))
            .with_max_pool_utilization(0.8);

        assert_eq!(config.query, "SELECT NOW()");
        assert_eq!(config.timeout, Duration::from_secs(3));
        assert_eq!(config.max_latency, Duration::from_millis(500));
        assert!((config.max_pool_utilization - 0.8).abs() < 0.001);
    }
}
