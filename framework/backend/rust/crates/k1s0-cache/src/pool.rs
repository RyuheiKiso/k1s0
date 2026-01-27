//! Redis コネクションプール
//!
//! bb8 を使用した Redis コネクションプール。

#[cfg(feature = "redis")]
use bb8::Pool;
#[cfg(feature = "redis")]
use bb8_redis::RedisConnectionManager;
#[cfg(feature = "redis")]
use tracing::{debug, info};

#[cfg(feature = "redis")]
use crate::config::CacheConfig;
#[cfg(feature = "redis")]
use crate::error::{CacheError, CacheResult};

/// プールの状態
#[derive(Debug, Clone)]
pub struct PoolStatus {
    /// 現在の接続数
    pub connections: u32,
    /// アイドル接続数
    pub idle_connections: u32,
    /// 最大接続数
    pub max_connections: u32,
}

/// Redis コネクションプール
#[cfg(feature = "redis")]
pub struct RedisPool {
    pool: Pool<RedisConnectionManager>,
    max_connections: u32,
}

#[cfg(feature = "redis")]
impl RedisPool {
    /// 新しいプールを作成
    pub async fn new(config: &CacheConfig) -> CacheResult<Self> {
        let url = config.connection_url();

        let manager = RedisConnectionManager::new(url.clone())
            .map_err(|e| CacheError::connection_with_source("Failed to create connection manager", e))?;

        let pool = Pool::builder()
            .max_size(config.pool.max_connections)
            .min_idle(Some(config.pool.min_connections))
            .connection_timeout(config.timeout.connect_timeout())
            .idle_timeout(Some(config.pool.idle_timeout()))
            .build(manager)
            .await
            .map_err(|e| CacheError::connection_with_source("Failed to create pool", e))?;

        info!(
            url = %config.connection_url_redacted(),
            max_connections = config.pool.max_connections,
            min_connections = config.pool.min_connections,
            "Redis connection pool created"
        );

        Ok(Self {
            pool,
            max_connections: config.pool.max_connections,
        })
    }

    /// 接続を取得
    pub async fn get(
        &self,
    ) -> CacheResult<bb8::PooledConnection<'_, RedisConnectionManager>> {
        self.pool.get().await.map_err(|e| {
            match e {
                bb8::RunError::User(redis_err) => {
                    CacheError::connection(redis_err.to_string())
                }
                bb8::RunError::TimedOut => {
                    CacheError::pool_exhausted(self.max_connections)
                }
            }
        })
    }

    /// プールの状態を取得
    pub fn status(&self) -> PoolStatus {
        let state = self.pool.state();
        PoolStatus {
            connections: state.connections,
            idle_connections: state.idle_connections,
            max_connections: self.max_connections,
        }
    }

    /// ヘルスチェック
    pub async fn health_check(&self) -> CacheResult<()> {
        let mut conn = self.get().await?;
        let pong: String = redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| CacheError::connection(e.to_string()))?;

        if pong != "PONG" {
            return Err(CacheError::connection("Unexpected PING response"));
        }

        debug!("Redis health check passed");
        Ok(())
    }
}

/// Redis プールビルダー
#[cfg(feature = "redis")]
#[derive(Debug, Default)]
pub struct RedisPoolBuilder {
    host: Option<String>,
    port: Option<u16>,
    database: Option<u8>,
    password: Option<String>,
    use_tls: Option<bool>,
    key_prefix: Option<String>,
    max_connections: Option<u32>,
    min_connections: Option<u32>,
    connect_timeout_ms: Option<u64>,
    operation_timeout_ms: Option<u64>,
}

#[cfg(feature = "redis")]
impl RedisPoolBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// ホストを設定
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// ポートを設定
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// データベース番号を設定
    pub fn database(mut self, database: u8) -> Self {
        self.database = Some(database);
        self
    }

    /// パスワードを設定
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// TLS を設定
    pub fn use_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = Some(use_tls);
        self
    }

    /// キープレフィックスを設定
    pub fn key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// 最大接続数を設定
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    /// 最小接続数を設定
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = Some(min);
        self
    }

    /// 接続タイムアウト（ミリ秒）を設定
    pub fn connect_timeout_ms(mut self, timeout: u64) -> Self {
        self.connect_timeout_ms = Some(timeout);
        self
    }

    /// 操作タイムアウト（ミリ秒）を設定
    pub fn operation_timeout_ms(mut self, timeout: u64) -> Self {
        self.operation_timeout_ms = Some(timeout);
        self
    }

    /// 設定をビルドしてプールを作成
    pub async fn build(self) -> CacheResult<crate::client::CacheClient> {
        let config = self.build_config()?;
        crate::client::CacheClient::new(config).await
    }

    /// 設定のみをビルド
    pub fn build_config(self) -> CacheResult<CacheConfig> {
        use crate::config::{PoolConfig, TimeoutConfig, TtlConfig};

        let pool_config = PoolConfig {
            max_connections: self.max_connections.unwrap_or(crate::config::DEFAULT_MAX_CONNECTIONS),
            min_connections: self.min_connections.unwrap_or(crate::config::DEFAULT_MIN_CONNECTIONS),
            idle_timeout_secs: crate::config::DEFAULT_IDLE_TIMEOUT_SECS,
        };

        let timeout_config = TimeoutConfig {
            connect_timeout_ms: self.connect_timeout_ms.unwrap_or(crate::config::DEFAULT_CONNECT_TIMEOUT_MS),
            operation_timeout_ms: self.operation_timeout_ms.unwrap_or(crate::config::DEFAULT_OPERATION_TIMEOUT_MS),
        };

        let config = CacheConfig {
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(crate::config::DEFAULT_REDIS_PORT),
            database: self.database.unwrap_or(0),
            password: self.password,
            password_file: None,
            use_tls: self.use_tls.unwrap_or(false),
            key_prefix: self.key_prefix,
            pool: pool_config,
            timeout: timeout_config,
            default_ttl: TtlConfig::default(),
        };

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_status() {
        let status = PoolStatus {
            connections: 5,
            idle_connections: 3,
            max_connections: 10,
        };
        assert_eq!(status.connections, 5);
        assert_eq!(status.idle_connections, 3);
        assert_eq!(status.max_connections, 10);
    }

    #[cfg(feature = "redis")]
    #[test]
    fn test_redis_pool_builder() {
        let builder = RedisPoolBuilder::new()
            .host("localhost")
            .port(6379)
            .database(1)
            .password("secret")
            .max_connections(20);

        assert_eq!(builder.host, Some("localhost".to_string()));
        assert_eq!(builder.port, Some(6379));
        assert_eq!(builder.database, Some(1));
        assert_eq!(builder.max_connections, Some(20));
    }

    #[cfg(feature = "redis")]
    #[test]
    fn test_build_config() {
        let config = RedisPoolBuilder::new()
            .host("redis.example.com")
            .port(6380)
            .database(2)
            .max_connections(50)
            .build_config()
            .unwrap();

        assert_eq!(config.host, "redis.example.com");
        assert_eq!(config.port, 6380);
        assert_eq!(config.database, 2);
        assert_eq!(config.pool.max_connections, 50);
    }
}
