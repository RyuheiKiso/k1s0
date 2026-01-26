//! データベースプールビルダー
//!
//! データベースコネクションプールの構築を簡略化するビルダーパターンを提供する。
//!
//! # 使用例
//!
//! ```ignore
//! use k1s0_db::pool::DbPoolBuilder;
//!
//! let pool = DbPoolBuilder::new()
//!     .host("localhost")
//!     .port(5432)
//!     .database("myapp")
//!     .username("app_user")
//!     .password("secret")
//!     .max_connections(20)
//!     .min_connections(5)
//!     .connect_timeout_ms(5000)
//!     .build()
//!     .await?;
//! ```

use crate::config::{DbConfig, PoolConfig, SslMode, TimeoutConfig};
use crate::error::{DbError, DbResult};

/// データベースプールビルダー
///
/// コネクションプールの設定をビルダーパターンで構築する。
#[derive(Debug, Default)]
pub struct DbPoolBuilder {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    password_file: Option<String>,
    ssl_mode: Option<SslMode>,
    max_connections: Option<u32>,
    min_connections: Option<u32>,
    connect_timeout_ms: Option<u64>,
    query_timeout_ms: Option<u64>,
    idle_timeout_secs: Option<u64>,
    max_lifetime_secs: Option<u64>,
}

impl DbPoolBuilder {
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

    /// SSL モードを設定
    pub fn ssl_mode(mut self, mode: SslMode) -> Self {
        self.ssl_mode = Some(mode);
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

    /// クエリタイムアウト（ミリ秒）を設定
    pub fn query_timeout_ms(mut self, timeout: u64) -> Self {
        self.query_timeout_ms = Some(timeout);
        self
    }

    /// アイドルタイムアウト（秒）を設定
    pub fn idle_timeout_secs(mut self, timeout: u64) -> Self {
        self.idle_timeout_secs = Some(timeout);
        self
    }

    /// 最大ライフタイム（秒）を設定
    pub fn max_lifetime_secs(mut self, lifetime: u64) -> Self {
        self.max_lifetime_secs = Some(lifetime);
        self
    }

    /// 設定をビルド
    pub fn build_config(self) -> DbResult<(DbConfig, String)> {
        let pool_config = PoolConfig {
            max_connections: self.max_connections.unwrap_or(crate::config::DEFAULT_MAX_CONNECTIONS),
            min_connections: self.min_connections.unwrap_or(crate::config::DEFAULT_MIN_CONNECTIONS),
            idle_timeout_secs: self.idle_timeout_secs.unwrap_or(crate::config::DEFAULT_IDLE_TIMEOUT_SECS),
            max_lifetime_secs: self.max_lifetime_secs.unwrap_or(crate::config::DEFAULT_MAX_LIFETIME_SECS),
        };

        let timeout_config = TimeoutConfig {
            connect_timeout_ms: self.connect_timeout_ms.unwrap_or(crate::config::DEFAULT_CONNECT_TIMEOUT_MS),
            query_timeout_ms: self.query_timeout_ms.unwrap_or(crate::config::DEFAULT_QUERY_TIMEOUT_MS),
        };

        let config = DbConfig {
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(5432),
            database: self.database.ok_or_else(|| DbError::config("database is required"))?,
            username: self.username.ok_or_else(|| DbError::config("username is required"))?,
            password_file: self.password_file,
            ssl_mode: self.ssl_mode.unwrap_or_default(),
            pool: pool_config,
            timeout: timeout_config,
        };

        config.validate()?;

        let password = self.resolve_password()?;

        Ok((config, password))
    }

    /// パスワードを解決
    fn resolve_password(&self) -> DbResult<String> {
        if let Some(ref password) = self.password {
            return Ok(password.clone());
        }

        if let Some(ref file) = self.password_file {
            return std::fs::read_to_string(file)
                .map(|s| s.trim().to_string())
                .map_err(|e| DbError::config(format!("failed to read password file: {}", e)));
        }

        Err(DbError::config("password or password_file is required"))
    }

    /// PostgreSQL プールをビルド
    #[cfg(feature = "postgres")]
    pub async fn build(self) -> DbResult<crate::postgres::PostgresPool> {
        let (config, password) = self.build_config()?;
        crate::postgres::PostgresPool::new(&config, &password).await
    }
}

/// 環境変数から DbPoolBuilder を作成
///
/// 以下の環境変数を読み取る：
/// - `DB_HOST` - データベースホスト
/// - `DB_PORT` - データベースポート
/// - `DB_NAME` - データベース名
/// - `DB_USER` - ユーザー名
/// - `DB_PASSWORD` - パスワード
/// - `DB_PASSWORD_FILE` - パスワードファイル
/// - `DB_SSL_MODE` - SSL モード
/// - `DB_MAX_CONNECTIONS` - 最大接続数
/// - `DB_MIN_CONNECTIONS` - 最小接続数
pub fn from_env() -> DbPoolBuilder {
    from_env_with_prefix("DB")
}

/// プレフィックス付きで環境変数から DbPoolBuilder を作成
pub fn from_env_with_prefix(prefix: &str) -> DbPoolBuilder {
    let mut builder = DbPoolBuilder::new();

    if let Ok(host) = std::env::var(format!("{}_HOST", prefix)) {
        builder = builder.host(host);
    }

    if let Ok(port) = std::env::var(format!("{}_PORT", prefix)) {
        if let Ok(port) = port.parse() {
            builder = builder.port(port);
        }
    }

    if let Ok(database) = std::env::var(format!("{}_NAME", prefix)) {
        builder = builder.database(database);
    }

    if let Ok(username) = std::env::var(format!("{}_USER", prefix)) {
        builder = builder.username(username);
    }

    if let Ok(password) = std::env::var(format!("{}_PASSWORD", prefix)) {
        builder = builder.password(password);
    }

    if let Ok(password_file) = std::env::var(format!("{}_PASSWORD_FILE", prefix)) {
        builder = builder.password_file(password_file);
    }

    if let Ok(ssl_mode) = std::env::var(format!("{}_SSL_MODE", prefix)) {
        let mode = match ssl_mode.to_lowercase().as_str() {
            "disable" => SslMode::Disable,
            "prefer" => SslMode::Prefer,
            "require" => SslMode::Require,
            "verify-ca" => SslMode::VerifyCa,
            "verify-full" => SslMode::VerifyFull,
            _ => SslMode::Prefer,
        };
        builder = builder.ssl_mode(mode);
    }

    if let Ok(max) = std::env::var(format!("{}_MAX_CONNECTIONS", prefix)) {
        if let Ok(max) = max.parse() {
            builder = builder.max_connections(max);
        }
    }

    if let Ok(min) = std::env::var(format!("{}_MIN_CONNECTIONS", prefix)) {
        if let Ok(min) = min.parse() {
            builder = builder.min_connections(min);
        }
    }

    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = DbPoolBuilder::new();
        assert!(builder.host.is_none());
        assert!(builder.database.is_none());
    }

    #[test]
    fn test_builder_chain() {
        let builder = DbPoolBuilder::new()
            .host("localhost")
            .port(5432)
            .database("mydb")
            .username("myuser")
            .password("secret")
            .max_connections(20);

        assert_eq!(builder.host, Some("localhost".to_string()));
        assert_eq!(builder.port, Some(5432));
        assert_eq!(builder.database, Some("mydb".to_string()));
        assert_eq!(builder.max_connections, Some(20));
    }

    #[test]
    fn test_build_config_missing_database() {
        let builder = DbPoolBuilder::new()
            .host("localhost")
            .username("user")
            .password("pass");

        let result = builder.build_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_config_missing_username() {
        let builder = DbPoolBuilder::new()
            .host("localhost")
            .database("db")
            .password("pass");

        let result = builder.build_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_config_missing_password() {
        let builder = DbPoolBuilder::new()
            .host("localhost")
            .database("db")
            .username("user");

        let result = builder.build_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_build_config_success() {
        let builder = DbPoolBuilder::new()
            .host("localhost")
            .database("db")
            .username("user")
            .password("secret");

        let result = builder.build_config();
        assert!(result.is_ok());

        let (config, password) = result.unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.database, "db");
        assert_eq!(config.username, "user");
        assert_eq!(password, "secret");
    }
}
