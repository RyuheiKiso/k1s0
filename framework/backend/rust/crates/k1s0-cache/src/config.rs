//! キャッシュ設定
//!
//! Redis 接続設定、プール設定、タイムアウト設定を定義する。

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{CacheError, CacheResult};

/// デフォルトの最大接続数
pub const DEFAULT_MAX_CONNECTIONS: u32 = 10;

/// デフォルトの最小接続数
pub const DEFAULT_MIN_CONNECTIONS: u32 = 1;

/// デフォルトの接続タイムアウト（ミリ秒）
pub const DEFAULT_CONNECT_TIMEOUT_MS: u64 = 5_000;

/// デフォルトの操作タイムアウト（ミリ秒）
pub const DEFAULT_OPERATION_TIMEOUT_MS: u64 = 1_000;

/// デフォルトのアイドルタイムアウト（秒）
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300;

/// デフォルトのTTL（秒）
pub const DEFAULT_TTL_SECS: u64 = 3600;

/// 最小タイムアウト（ミリ秒）
pub const MIN_TIMEOUT_MS: u64 = 10;

/// 最大タイムアウト（ミリ秒）
pub const MAX_TIMEOUT_MS: u64 = 60_000;

/// デフォルトの Redis ポート
pub const DEFAULT_REDIS_PORT: u16 = 6379;

/// キャッシュ接続設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Redis ホスト
    pub host: String,
    /// Redis ポート
    #[serde(default = "default_port")]
    pub port: u16,
    /// Redis データベース番号
    #[serde(default)]
    pub database: u8,
    /// パスワード（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// パスワードファイル参照
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_file: Option<String>,
    /// TLS を使用するか
    #[serde(default)]
    pub use_tls: bool,
    /// キープレフィックス
    #[serde(default)]
    pub key_prefix: Option<String>,
    /// プール設定
    #[serde(default)]
    pub pool: PoolConfig,
    /// タイムアウト設定
    #[serde(default)]
    pub timeout: TimeoutConfig,
    /// デフォルト TTL 設定
    #[serde(default)]
    pub default_ttl: TtlConfig,
}

fn default_port() -> u16 {
    DEFAULT_REDIS_PORT
}

impl CacheConfig {
    /// 新しい設定を作成
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: DEFAULT_REDIS_PORT,
            database: 0,
            password: None,
            password_file: None,
            use_tls: false,
            key_prefix: None,
            pool: PoolConfig::default(),
            timeout: TimeoutConfig::default(),
            default_ttl: TtlConfig::default(),
        }
    }

    /// ビルダーを作成
    pub fn builder() -> CacheConfigBuilder {
        CacheConfigBuilder::default()
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> CacheResult<()> {
        if self.host.is_empty() {
            return Err(CacheError::config("host is required"));
        }

        self.pool.validate()?;
        self.timeout.validate()?;

        Ok(())
    }

    /// 接続 URL を生成
    pub fn connection_url(&self) -> String {
        let scheme = if self.use_tls { "rediss" } else { "redis" };

        match (&self.password, self.database) {
            (Some(pass), db) if db > 0 => {
                format!("{}://:{}@{}:{}/{}", scheme, pass, self.host, self.port, db)
            }
            (Some(pass), _) => {
                format!("{}://:{}@{}:{}", scheme, pass, self.host, self.port)
            }
            (None, db) if db > 0 => {
                format!("{}://{}:{}/{}", scheme, self.host, self.port, db)
            }
            (None, _) => {
                format!("{}://{}:{}", scheme, self.host, self.port)
            }
        }
    }

    /// 接続 URL を生成（パスワードなし、ログ用）
    pub fn connection_url_redacted(&self) -> String {
        let scheme = if self.use_tls { "rediss" } else { "redis" };

        match (&self.password, self.database) {
            (Some(_), db) if db > 0 => {
                format!("{}://:***@{}:{}/{}", scheme, self.host, self.port, db)
            }
            (Some(_), _) => {
                format!("{}://:***@{}:{}", scheme, self.host, self.port)
            }
            (None, db) if db > 0 => {
                format!("{}://{}:{}/{}", scheme, self.host, self.port, db)
            }
            (None, _) => {
                format!("{}://{}:{}", scheme, self.host, self.port)
            }
        }
    }

    /// キーにプレフィックスを付与
    pub fn prefixed_key(&self, key: &str) -> String {
        match &self.key_prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new("localhost")
    }
}

/// キャッシュ設定ビルダー
#[derive(Debug, Default)]
pub struct CacheConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    database: Option<u8>,
    password: Option<String>,
    password_file: Option<String>,
    use_tls: Option<bool>,
    key_prefix: Option<String>,
    pool: Option<PoolConfig>,
    timeout: Option<TimeoutConfig>,
    default_ttl: Option<TtlConfig>,
}

impl CacheConfigBuilder {
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

    /// パスワードファイルを設定
    pub fn password_file(mut self, file: impl Into<String>) -> Self {
        self.password_file = Some(file.into());
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

    /// プール設定を設定
    pub fn pool(mut self, pool: PoolConfig) -> Self {
        self.pool = Some(pool);
        self
    }

    /// タイムアウト設定を設定
    pub fn timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// デフォルト TTL 設定を設定
    pub fn default_ttl(mut self, ttl: TtlConfig) -> Self {
        self.default_ttl = Some(ttl);
        self
    }

    /// 設定をビルド
    pub fn build(self) -> CacheResult<CacheConfig> {
        let config = CacheConfig {
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(DEFAULT_REDIS_PORT),
            database: self.database.unwrap_or(0),
            password: self.password,
            password_file: self.password_file,
            use_tls: self.use_tls.unwrap_or(false),
            key_prefix: self.key_prefix,
            pool: self.pool.unwrap_or_default(),
            timeout: self.timeout.unwrap_or_default(),
            default_ttl: self.default_ttl.unwrap_or_default(),
        };

        config.validate()?;
        Ok(config)
    }
}

/// コネクションプール設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// 最大接続数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// 最小接続数
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    /// アイドルタイムアウト（秒）
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
}

fn default_max_connections() -> u32 {
    DEFAULT_MAX_CONNECTIONS
}

fn default_min_connections() -> u32 {
    DEFAULT_MIN_CONNECTIONS
}

fn default_idle_timeout_secs() -> u64 {
    DEFAULT_IDLE_TIMEOUT_SECS
}

impl PoolConfig {
    /// 新しいプール設定を作成
    pub fn new(max_connections: u32) -> Self {
        Self {
            max_connections,
            min_connections: DEFAULT_MIN_CONNECTIONS,
            idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
        }
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> CacheResult<()> {
        if self.max_connections == 0 {
            return Err(CacheError::config("max_connections must be > 0"));
        }
        if self.min_connections > self.max_connections {
            return Err(CacheError::config(
                "min_connections cannot exceed max_connections",
            ));
        }
        Ok(())
    }

    /// アイドルタイムアウトを取得
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_CONNECTIONS)
    }
}

/// タイムアウト設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// 接続タイムアウト（ミリ秒）
    #[serde(default = "default_connect_timeout_ms")]
    pub connect_timeout_ms: u64,
    /// 操作タイムアウト（ミリ秒）
    #[serde(default = "default_operation_timeout_ms")]
    pub operation_timeout_ms: u64,
}

fn default_connect_timeout_ms() -> u64 {
    DEFAULT_CONNECT_TIMEOUT_MS
}

fn default_operation_timeout_ms() -> u64 {
    DEFAULT_OPERATION_TIMEOUT_MS
}

impl TimeoutConfig {
    /// 新しいタイムアウト設定を作成
    pub fn new(connect_timeout_ms: u64, operation_timeout_ms: u64) -> Self {
        Self {
            connect_timeout_ms,
            operation_timeout_ms,
        }
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> CacheResult<()> {
        if self.connect_timeout_ms < MIN_TIMEOUT_MS {
            return Err(CacheError::config(format!(
                "connect_timeout_ms {} is below minimum {}",
                self.connect_timeout_ms, MIN_TIMEOUT_MS
            )));
        }
        if self.connect_timeout_ms > MAX_TIMEOUT_MS {
            return Err(CacheError::config(format!(
                "connect_timeout_ms {} exceeds maximum {}",
                self.connect_timeout_ms, MAX_TIMEOUT_MS
            )));
        }
        if self.operation_timeout_ms < MIN_TIMEOUT_MS {
            return Err(CacheError::config(format!(
                "operation_timeout_ms {} is below minimum {}",
                self.operation_timeout_ms, MIN_TIMEOUT_MS
            )));
        }
        if self.operation_timeout_ms > MAX_TIMEOUT_MS {
            return Err(CacheError::config(format!(
                "operation_timeout_ms {} exceeds maximum {}",
                self.operation_timeout_ms, MAX_TIMEOUT_MS
            )));
        }
        Ok(())
    }

    /// 接続タイムアウトを取得
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_millis(self.connect_timeout_ms)
    }

    /// 操作タイムアウトを取得
    pub fn operation_timeout(&self) -> Duration {
        Duration::from_millis(self.operation_timeout_ms)
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self::new(DEFAULT_CONNECT_TIMEOUT_MS, DEFAULT_OPERATION_TIMEOUT_MS)
    }
}

/// デフォルト TTL 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtlConfig {
    /// デフォルト TTL（秒）
    #[serde(default = "default_ttl_secs")]
    pub default_secs: u64,
    /// 最大 TTL（秒）
    #[serde(default = "default_max_ttl_secs")]
    pub max_secs: u64,
}

fn default_ttl_secs() -> u64 {
    DEFAULT_TTL_SECS
}

fn default_max_ttl_secs() -> u64 {
    86400 // 24 hours
}

impl TtlConfig {
    /// デフォルト TTL を Duration で取得
    pub fn default_ttl(&self) -> Duration {
        Duration::from_secs(self.default_secs)
    }

    /// 最大 TTL を Duration で取得
    pub fn max_ttl(&self) -> Duration {
        Duration::from_secs(self.max_secs)
    }

    /// TTL を制限内に収める
    pub fn clamp_ttl(&self, ttl: Duration) -> Duration {
        let max = self.max_ttl();
        if ttl > max {
            max
        } else {
            ttl
        }
    }
}

impl Default for TtlConfig {
    fn default() -> Self {
        Self {
            default_secs: DEFAULT_TTL_SECS,
            max_secs: 86400,
        }
    }
}

/// 環境変数からキャッシュ設定を作成
pub fn from_env() -> CacheConfigBuilder {
    from_env_with_prefix("REDIS")
}

/// プレフィックス付きで環境変数からキャッシュ設定を作成
pub fn from_env_with_prefix(prefix: &str) -> CacheConfigBuilder {
    let mut builder = CacheConfigBuilder::default();

    if let Ok(host) = std::env::var(format!("{}_HOST", prefix)) {
        builder = builder.host(host);
    }

    if let Ok(port) = std::env::var(format!("{}_PORT", prefix)) {
        if let Ok(port) = port.parse() {
            builder = builder.port(port);
        }
    }

    if let Ok(database) = std::env::var(format!("{}_DATABASE", prefix)) {
        if let Ok(database) = database.parse() {
            builder = builder.database(database);
        }
    }

    if let Ok(password) = std::env::var(format!("{}_PASSWORD", prefix)) {
        builder = builder.password(password);
    }

    if let Ok(password_file) = std::env::var(format!("{}_PASSWORD_FILE", prefix)) {
        builder = builder.password_file(password_file);
    }

    if let Ok(use_tls) = std::env::var(format!("{}_TLS", prefix)) {
        let use_tls = matches!(use_tls.to_lowercase().as_str(), "true" | "1" | "yes");
        builder = builder.use_tls(use_tls);
    }

    if let Ok(prefix_val) = std::env::var(format!("{}_KEY_PREFIX", prefix)) {
        builder = builder.key_prefix(prefix_val);
    }

    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, DEFAULT_REDIS_PORT);
        assert_eq!(config.database, 0);
        assert!(!config.use_tls);
    }

    #[test]
    fn test_cache_config_builder() {
        let config = CacheConfig::builder()
            .host("redis.example.com")
            .port(6380)
            .database(1)
            .password("secret")
            .use_tls(true)
            .key_prefix("myapp")
            .build()
            .unwrap();

        assert_eq!(config.host, "redis.example.com");
        assert_eq!(config.port, 6380);
        assert_eq!(config.database, 1);
        assert_eq!(config.password, Some("secret".to_string()));
        assert!(config.use_tls);
        assert_eq!(config.key_prefix, Some("myapp".to_string()));
    }

    #[test]
    fn test_connection_url() {
        let config = CacheConfig::new("localhost");
        assert_eq!(config.connection_url(), "redis://localhost:6379");

        let config = CacheConfig::builder()
            .host("localhost")
            .password("pass")
            .build()
            .unwrap();
        assert_eq!(config.connection_url(), "redis://:pass@localhost:6379");

        let config = CacheConfig::builder()
            .host("localhost")
            .database(1)
            .build()
            .unwrap();
        assert_eq!(config.connection_url(), "redis://localhost:6379/1");

        let config = CacheConfig::builder()
            .host("localhost")
            .use_tls(true)
            .build()
            .unwrap();
        assert_eq!(config.connection_url(), "rediss://localhost:6379");
    }

    #[test]
    fn test_connection_url_redacted() {
        let config = CacheConfig::builder()
            .host("localhost")
            .password("supersecret")
            .build()
            .unwrap();
        assert_eq!(config.connection_url_redacted(), "redis://:***@localhost:6379");
        assert!(!config.connection_url_redacted().contains("supersecret"));
    }

    #[test]
    fn test_prefixed_key() {
        let config = CacheConfig::builder()
            .host("localhost")
            .key_prefix("myapp")
            .build()
            .unwrap();
        assert_eq!(config.prefixed_key("user:123"), "myapp:user:123");

        let config = CacheConfig::new("localhost");
        assert_eq!(config.prefixed_key("user:123"), "user:123");
    }

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, DEFAULT_MAX_CONNECTIONS);
        assert_eq!(config.min_connections, DEFAULT_MIN_CONNECTIONS);
    }

    #[test]
    fn test_pool_config_validation() {
        let mut config = PoolConfig::default();
        config.max_connections = 0;
        assert!(config.validate().is_err());

        config.max_connections = 10;
        config.min_connections = 20;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.connect_timeout_ms, DEFAULT_CONNECT_TIMEOUT_MS);
        assert_eq!(config.operation_timeout_ms, DEFAULT_OPERATION_TIMEOUT_MS);
    }

    #[test]
    fn test_timeout_config_validation() {
        // 下限未満
        let config = TimeoutConfig::new(5, DEFAULT_OPERATION_TIMEOUT_MS);
        assert!(config.validate().is_err());

        // 上限超過
        let config = TimeoutConfig::new(DEFAULT_CONNECT_TIMEOUT_MS, 100_000);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ttl_config() {
        let config = TtlConfig::default();
        assert_eq!(config.default_ttl(), Duration::from_secs(DEFAULT_TTL_SECS));
        assert_eq!(config.max_ttl(), Duration::from_secs(86400));

        // clamp test
        let short_ttl = Duration::from_secs(100);
        assert_eq!(config.clamp_ttl(short_ttl), short_ttl);

        let long_ttl = Duration::from_secs(100_000);
        assert_eq!(config.clamp_ttl(long_ttl), Duration::from_secs(86400));
    }
}
