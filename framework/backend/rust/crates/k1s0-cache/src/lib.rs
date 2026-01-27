//! k1s0-cache: Redis キャッシュクライアント
//!
//! このクレートは、k1s0 フレームワークにおけるキャッシュ機能の
//! 標準化されたインターフェースを提供する。
//!
//! ## 機能
//!
//! - **接続プール**: bb8 による Redis コネクションプール
//! - **基本操作**: get, set, delete, exists, incr, decr
//! - **バルク操作**: mget, mset, mdel
//! - **データ構造**: Hash, List, Set
//! - **キャッシュパターン**: Cache-Aside, TTL リフレッシュ
//! - **メトリクス**: ヒット率、操作数の計測
//! - **ヘルスチェック**: k1s0-health との統合
//!
//! ## 設計原則
//!
//! 1. **型安全**: serde による自動シリアライズ/デシリアライズ
//! 2. **障害耐性**: キャッシュ障害時のフォールバック
//! 3. **計測**: 全操作のメトリクス収集
//! 4. **プレフィックス**: 名前空間によるキー分離
//!
//! ## 使用例
//!
//! ### 基本的な使い方
//!
//! ```rust,ignore
//! use k1s0_cache::{CacheConfig, CacheClient, CacheOperations};
//! use std::time::Duration;
//!
//! // 設定の作成
//! let config = CacheConfig::builder()
//!     .host("localhost")
//!     .port(6379)
//!     .key_prefix("myapp")
//!     .build()?;
//!
//! // クライアントの作成
//! let client = CacheClient::new(config).await?;
//!
//! // 値の設定と取得
//! client.set("user:123", &user, Some(Duration::from_secs(3600))).await?;
//! let user: Option<User> = client.get("user:123").await?;
//! ```
//!
//! ### Cache-Aside パターン
//!
//! ```rust,ignore
//! use k1s0_cache::patterns::{CacheAside, CacheAsideConfig};
//!
//! let cache_aside = CacheAside::new(cache_client.into(), CacheAsideConfig::default());
//!
//! // キャッシュまたはDBから取得
//! let user = cache_aside.get_or_load(
//!     &format!("user:{}", user_id),
//!     || async { db.find_user(user_id).await },
//! ).await?;
//! ```
//!
//! ### 環境変数からの設定
//!
//! ```rust,ignore
//! use k1s0_cache::config::from_env;
//!
//! // REDIS_HOST, REDIS_PORT, REDIS_PASSWORD などから設定を読み込み
//! let config = from_env().build()?;
//! ```

pub mod config;
pub mod error;
pub mod metrics;
pub mod operations;
pub mod patterns;
pub mod pool;

#[cfg(feature = "redis")]
pub mod client;

#[cfg(feature = "health")]
pub mod health;

// 主要な型の再エクスポート
pub use config::{
    CacheConfig, CacheConfigBuilder, PoolConfig, TimeoutConfig, TtlConfig,
    DEFAULT_CONNECT_TIMEOUT_MS, DEFAULT_IDLE_TIMEOUT_SECS, DEFAULT_MAX_CONNECTIONS,
    DEFAULT_MIN_CONNECTIONS, DEFAULT_OPERATION_TIMEOUT_MS, DEFAULT_REDIS_PORT, DEFAULT_TTL_SECS,
};
pub use error::{CacheError, CacheResult};
pub use metrics::{CacheMetrics, CacheOperation, MetricsSnapshot, OperationTimer};
pub use operations::{
    CacheOperations, CacheOperationsExt, HashOperations, ListOperations, SetOperations,
};
pub use patterns::{CacheAside, CacheAsideConfig, TtlRefresh, TtlRefreshConfig};
pub use pool::PoolStatus;

// Redis 固有の型（feature = "redis" 時のみ）
#[cfg(feature = "redis")]
pub use client::CacheClient;
#[cfg(feature = "redis")]
pub use pool::{RedisPool, RedisPoolBuilder};

// ヘルスチェック（feature = "health" 時のみ）
#[cfg(all(feature = "redis", feature = "health"))]
pub use health::CacheHealthChecker;
#[cfg(feature = "health")]
pub use health::CacheHealthStatus;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, DEFAULT_REDIS_PORT);
    }

    #[test]
    fn test_cache_error_codes() {
        let err = CacheError::connection("test");
        assert_eq!(err.error_code(), "CACHE_CONNECTION_ERROR");
        assert!(err.is_retryable());
    }

    #[test]
    fn test_cache_metrics() {
        let metrics = CacheMetrics::new();
        assert_eq!(metrics.hits(), 0);
        assert_eq!(metrics.hit_rate(), 0.0);

        metrics.record_hit(CacheOperation::Get);
        assert_eq!(metrics.hits(), 1);
    }
}
