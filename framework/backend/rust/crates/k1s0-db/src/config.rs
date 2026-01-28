//! データベース設定
//!
//! 接続プール、タイムアウト、リトライなどの設定を定義する。

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{DbError, DbResult};

/// デフォルトの最大接続数
pub const DEFAULT_MAX_CONNECTIONS: u32 = 10;

/// デフォルトの最小接続数
pub const DEFAULT_MIN_CONNECTIONS: u32 = 1;

/// デフォルトの接続タイムアウト（ミリ秒）
pub const DEFAULT_CONNECT_TIMEOUT_MS: u64 = 5_000;

/// デフォルトのクエリタイムアウト（ミリ秒）
pub const DEFAULT_QUERY_TIMEOUT_MS: u64 = 30_000;

/// デフォルトのアイドルタイムアウト（秒）
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 600;

/// デフォルトの最大ライフタイム（秒）
pub const DEFAULT_MAX_LIFETIME_SECS: u64 = 1800;

/// 最小タイムアウト（ミリ秒）
pub const MIN_TIMEOUT_MS: u64 = 100;

/// 最大タイムアウト（ミリ秒）
pub const MAX_TIMEOUT_MS: u64 = 300_000;

/// データベース接続設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    /// データベースホスト
    pub host: String,
    /// データベースポート
    #[serde(default = "default_port")]
    pub port: u16,
    /// データベース名
    pub database: String,
    /// ユーザー名
    pub username: String,
    /// パスワードファイル参照（`*_file` パターン）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_file: Option<String>,
    /// SSL モード
    #[serde(default)]
    pub ssl_mode: SslMode,
    /// プール設定
    #[serde(default)]
    pub pool: PoolConfig,
    /// タイムアウト設定
    #[serde(default)]
    pub timeout: TimeoutConfig,
}

fn default_port() -> u16 {
    5432
}

impl DbConfig {
    /// 新しい設定を作成
    pub fn new(
        host: impl Into<String>,
        database: impl Into<String>,
        username: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            port: default_port(),
            database: database.into(),
            username: username.into(),
            password_file: None,
            ssl_mode: SslMode::default(),
            pool: PoolConfig::default(),
            timeout: TimeoutConfig::default(),
        }
    }

    /// ビルダーを作成
    pub fn builder() -> DbConfigBuilder {
        DbConfigBuilder::default()
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> DbResult<()> {
        if self.host.is_empty() {
            return Err(DbError::config("host is required"));
        }
        if self.database.is_empty() {
            return Err(DbError::config("database is required"));
        }
        if self.username.is_empty() {
            return Err(DbError::config("username is required"));
        }

        self.pool.validate()?;
        self.timeout.validate()?;

        Ok(())
    }

    /// 接続文字列を生成（パスワードなし）
    pub fn connection_string_without_password(&self) -> String {
        format!(
            "postgres://{}@{}:{}/{}?sslmode={}",
            self.username,
            self.host,
            self.port,
            self.database,
            self.ssl_mode.as_str()
        )
    }

    /// 接続文字列を生成（パスワード付き）
    pub fn connection_string(&self, password: &str) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.username,
            password,
            self.host,
            self.port,
            self.database,
            self.ssl_mode.as_str()
        )
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self::new("localhost", "k1s0", "k1s0")
    }
}

/// データベース設定ビルダー
#[derive(Debug, Default)]
pub struct DbConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password_file: Option<String>,
    ssl_mode: Option<SslMode>,
    pool: Option<PoolConfig>,
    timeout: Option<TimeoutConfig>,
}

impl DbConfigBuilder {
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

    /// データベース名を設定
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// ユーザー名を設定
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// パスワードファイルを設定
    pub fn password_file(mut self, file: impl Into<String>) -> Self {
        self.password_file = Some(file.into());
        self
    }

    /// SSL モードを設定
    pub fn ssl_mode(mut self, mode: SslMode) -> Self {
        self.ssl_mode = Some(mode);
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

    /// 設定をビルド
    pub fn build(self) -> DbResult<DbConfig> {
        let config = DbConfig {
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or_else(default_port),
            database: self
                .database
                .ok_or_else(|| DbError::config("database is required"))?,
            username: self
                .username
                .ok_or_else(|| DbError::config("username is required"))?,
            password_file: self.password_file,
            ssl_mode: self.ssl_mode.unwrap_or_default(),
            pool: self.pool.unwrap_or_default(),
            timeout: self.timeout.unwrap_or_default(),
        };

        config.validate()?;
        Ok(config)
    }
}

/// SSL モード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SslMode {
    /// SSL を無効化
    Disable,
    /// SSL が利用可能なら使用
    Prefer,
    /// SSL を必須
    Require,
    /// CA 証明書を検証
    VerifyCa,
    /// CA 証明書とホスト名を検証
    VerifyFull,
}

impl SslMode {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disable => "disable",
            Self::Prefer => "prefer",
            Self::Require => "require",
            Self::VerifyCa => "verify-ca",
            Self::VerifyFull => "verify-full",
        }
    }
}

impl Default for SslMode {
    fn default() -> Self {
        Self::Prefer
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
    /// 最大ライフタイム（秒）
    #[serde(default = "default_max_lifetime_secs")]
    pub max_lifetime_secs: u64,
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

fn default_max_lifetime_secs() -> u64 {
    DEFAULT_MAX_LIFETIME_SECS
}

impl PoolConfig {
    /// 新しいプール設定を作成
    pub fn new(max_connections: u32) -> Self {
        Self {
            max_connections,
            min_connections: DEFAULT_MIN_CONNECTIONS,
            idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
            max_lifetime_secs: DEFAULT_MAX_LIFETIME_SECS,
        }
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> DbResult<()> {
        if self.max_connections == 0 {
            return Err(DbError::config("max_connections must be > 0"));
        }
        if self.min_connections > self.max_connections {
            return Err(DbError::config(
                "min_connections cannot exceed max_connections",
            ));
        }
        Ok(())
    }

    /// アイドルタイムアウトを取得
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_secs)
    }

    /// 最大ライフタイムを取得
    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_secs)
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
    /// クエリタイムアウト（ミリ秒）
    #[serde(default = "default_query_timeout_ms")]
    pub query_timeout_ms: u64,
}

fn default_connect_timeout_ms() -> u64 {
    DEFAULT_CONNECT_TIMEOUT_MS
}

fn default_query_timeout_ms() -> u64 {
    DEFAULT_QUERY_TIMEOUT_MS
}

impl TimeoutConfig {
    /// 新しいタイムアウト設定を作成
    pub fn new(connect_timeout_ms: u64, query_timeout_ms: u64) -> Self {
        Self {
            connect_timeout_ms,
            query_timeout_ms,
        }
    }

    /// 設定をバリデーション
    pub fn validate(&self) -> DbResult<()> {
        if self.connect_timeout_ms < MIN_TIMEOUT_MS {
            return Err(DbError::config(format!(
                "connect_timeout_ms {} is below minimum {}",
                self.connect_timeout_ms, MIN_TIMEOUT_MS
            )));
        }
        if self.connect_timeout_ms > MAX_TIMEOUT_MS {
            return Err(DbError::config(format!(
                "connect_timeout_ms {} exceeds maximum {}",
                self.connect_timeout_ms, MAX_TIMEOUT_MS
            )));
        }
        if self.query_timeout_ms < MIN_TIMEOUT_MS {
            return Err(DbError::config(format!(
                "query_timeout_ms {} is below minimum {}",
                self.query_timeout_ms, MIN_TIMEOUT_MS
            )));
        }
        if self.query_timeout_ms > MAX_TIMEOUT_MS {
            return Err(DbError::config(format!(
                "query_timeout_ms {} exceeds maximum {}",
                self.query_timeout_ms, MAX_TIMEOUT_MS
            )));
        }
        Ok(())
    }

    /// 接続タイムアウトを取得
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_millis(self.connect_timeout_ms)
    }

    /// クエリタイムアウトを取得
    pub fn query_timeout(&self) -> Duration {
        Duration::from_millis(self.query_timeout_ms)
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self::new(DEFAULT_CONNECT_TIMEOUT_MS, DEFAULT_QUERY_TIMEOUT_MS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_config_default() {
        let config = DbConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "k1s0");
    }

    #[test]
    fn test_db_config_builder() {
        let config = DbConfig::builder()
            .host("db.example.com")
            .port(5433)
            .database("mydb")
            .username("myuser")
            .ssl_mode(SslMode::Require)
            .build()
            .unwrap();

        assert_eq!(config.host, "db.example.com");
        assert_eq!(config.port, 5433);
        assert_eq!(config.database, "mydb");
        assert_eq!(config.ssl_mode, SslMode::Require);
    }

    #[test]
    fn test_db_config_validation() {
        // database 必須
        let result = DbConfig::builder()
            .host("localhost")
            .username("user")
            .build();
        assert!(result.is_err());

        // username 必須
        let result = DbConfig::builder().host("localhost").database("db").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_connection_string() {
        let config = DbConfig::new("localhost", "mydb", "myuser");
        let conn_str = config.connection_string("secret");
        assert!(conn_str.contains("localhost"));
        assert!(conn_str.contains("mydb"));
        assert!(conn_str.contains("myuser"));
        assert!(conn_str.contains("secret"));
    }

    #[test]
    fn test_ssl_mode() {
        assert_eq!(SslMode::Disable.as_str(), "disable");
        assert_eq!(SslMode::Prefer.as_str(), "prefer");
        assert_eq!(SslMode::Require.as_str(), "require");
        assert_eq!(SslMode::VerifyCa.as_str(), "verify-ca");
        assert_eq!(SslMode::VerifyFull.as_str(), "verify-full");
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
        assert_eq!(config.query_timeout_ms, DEFAULT_QUERY_TIMEOUT_MS);
    }

    #[test]
    fn test_timeout_config_validation() {
        // 下限未満
        let config = TimeoutConfig::new(50, DEFAULT_QUERY_TIMEOUT_MS);
        assert!(config.validate().is_err());

        // 上限超過
        let config = TimeoutConfig::new(DEFAULT_CONNECT_TIMEOUT_MS, 400_000);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_timeout_durations() {
        let config = TimeoutConfig::new(5000, 30000);
        assert_eq!(config.connect_timeout(), Duration::from_millis(5000));
        assert_eq!(config.query_timeout(), Duration::from_millis(30000));
    }
}
