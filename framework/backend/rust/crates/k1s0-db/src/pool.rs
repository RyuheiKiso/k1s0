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
        // パスワードを先に解決（self.password_file を参照するため）
        let password = self.resolve_password()?;

        let pool_config = PoolConfig {
            max_connections: self
                .max_connections
                .unwrap_or(crate::config::DEFAULT_MAX_CONNECTIONS),
            min_connections: self
                .min_connections
                .unwrap_or(crate::config::DEFAULT_MIN_CONNECTIONS),
            idle_timeout_secs: self
                .idle_timeout_secs
                .unwrap_or(crate::config::DEFAULT_IDLE_TIMEOUT_SECS),
            max_lifetime_secs: self
                .max_lifetime_secs
                .unwrap_or(crate::config::DEFAULT_MAX_LIFETIME_SECS),
        };

        let timeout_config = TimeoutConfig {
            connect_timeout_ms: self
                .connect_timeout_ms
                .unwrap_or(crate::config::DEFAULT_CONNECT_TIMEOUT_MS),
            query_timeout_ms: self
                .query_timeout_ms
                .unwrap_or(crate::config::DEFAULT_QUERY_TIMEOUT_MS),
        };

        let config = DbConfig {
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(5432),
            database: self
                .database
                .ok_or_else(|| DbError::config("database is required"))?,
            username: self
                .username
                .ok_or_else(|| DbError::config("username is required"))?,
            password_file: self.password_file,
            ssl_mode: self.ssl_mode.unwrap_or_default(),
            pool: pool_config,
            timeout: timeout_config,
        };

        config.validate()?;

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

/// バックプレッシャー付きプール設定
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackpressuredPoolConfig {
    /// 同時に接続取得を待機できる最大数
    #[serde(default = "default_max_waiting")]
    pub max_waiting: u32,
    /// 接続取得タイムアウト（ミリ秒）
    #[serde(default = "default_acquire_timeout_ms")]
    pub acquire_timeout_ms: u64,
}

fn default_max_waiting() -> u32 {
    100
}

fn default_acquire_timeout_ms() -> u64 {
    5_000
}

impl Default for BackpressuredPoolConfig {
    fn default() -> Self {
        Self {
            max_waiting: default_max_waiting(),
            acquire_timeout_ms: default_acquire_timeout_ms(),
        }
    }
}

impl BackpressuredPoolConfig {
    /// 同時待機数を設定
    pub fn with_max_waiting(mut self, max: u32) -> Self {
        self.max_waiting = max;
        self
    }

    /// 取得タイムアウトを設定（ミリ秒）
    pub fn with_acquire_timeout_ms(mut self, ms: u64) -> Self {
        self.acquire_timeout_ms = ms;
        self
    }

    /// 取得タイムアウトを Duration で取得
    pub fn acquire_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.acquire_timeout_ms)
    }
}

/// プールメトリクス
#[derive(Debug)]
pub struct PoolMetrics {
    /// アクティブな接続数
    pub active_connections: std::sync::atomic::AtomicU64,
    /// アイドルな接続数
    pub idle_connections: std::sync::atomic::AtomicU64,
    /// 現在の待機数
    pub waiting_count: std::sync::atomic::AtomicU64,
    /// 拒否された総数
    pub rejected_total: std::sync::atomic::AtomicU64,
    /// 接続取得にかかった時間（ナノ秒の累積）
    pub acquire_duration_nanos: std::sync::atomic::AtomicU64,
}

impl PoolMetrics {
    /// 新しいメトリクスを作成
    fn new() -> Self {
        Self {
            active_connections: std::sync::atomic::AtomicU64::new(0),
            idle_connections: std::sync::atomic::AtomicU64::new(0),
            waiting_count: std::sync::atomic::AtomicU64::new(0),
            rejected_total: std::sync::atomic::AtomicU64::new(0),
            acquire_duration_nanos: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// メトリクスのスナップショットを取得
    pub fn snapshot(&self) -> PoolMetricsSnapshot {
        use std::sync::atomic::Ordering;
        PoolMetricsSnapshot {
            active_connections: self.active_connections.load(Ordering::Relaxed),
            idle_connections: self.idle_connections.load(Ordering::Relaxed),
            waiting_count: self.waiting_count.load(Ordering::Relaxed),
            rejected_total: self.rejected_total.load(Ordering::Relaxed),
            acquire_duration_nanos: self.acquire_duration_nanos.load(Ordering::Relaxed),
        }
    }
}

impl Default for PoolMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// プールメトリクスのスナップショット
#[derive(Debug, Clone)]
pub struct PoolMetricsSnapshot {
    pub active_connections: u64,
    pub idle_connections: u64,
    pub waiting_count: u64,
    pub rejected_total: u64,
    pub acquire_duration_nanos: u64,
}

/// バックプレッシャー付きコネクションプールラッパー
///
/// 待機キューの長さを制限し、過負荷時に早期に拒否することで
/// カスケード障害を防止する。
pub struct BackpressuredPool {
    config: BackpressuredPoolConfig,
    metrics: std::sync::Arc<PoolMetrics>,
    /// 待機数を管理するセマフォ
    waiting_semaphore: std::sync::Arc<tokio::sync::Semaphore>,
}

/// バックプレッシャーガード
///
/// ドロップ時に待機カウントを減算する。
pub struct BackpressureGuard {
    metrics: std::sync::Arc<PoolMetrics>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    acquired: bool,
}

impl BackpressureGuard {
    /// 接続取得成功をマーク
    pub fn mark_acquired(&mut self) {
        if !self.acquired {
            self.acquired = true;
            self.metrics
                .active_connections
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

impl Drop for BackpressureGuard {
    fn drop(&mut self) {
        self.metrics
            .waiting_count
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        if self.acquired {
            self.metrics
                .active_connections
                .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

impl BackpressuredPool {
    /// 新しいバックプレッシャー付きプールを作成
    pub fn new(config: BackpressuredPoolConfig) -> Self {
        let semaphore = std::sync::Arc::new(
            tokio::sync::Semaphore::new(config.max_waiting as usize),
        );
        Self {
            config,
            metrics: std::sync::Arc::new(PoolMetrics::new()),
            waiting_semaphore: semaphore,
        }
    }

    /// 接続取得の許可を取得する
    ///
    /// 待機キューが満杯の場合はエラーを返す。
    /// 取得成功後は返された `BackpressureGuard` を保持し、
    /// 実際の接続取得後に `mark_acquired()` を呼ぶ。
    pub async fn acquire(&self) -> DbResult<BackpressureGuard> {
        let start = std::time::Instant::now();

        // 待機キューに空きがあるか試行
        let permit = match self.waiting_semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                self.metrics
                    .rejected_total
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return Err(DbError::pool_exhausted(self.config.max_waiting));
            }
        };

        self.metrics
            .waiting_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let elapsed = start.elapsed();
        self.metrics.acquire_duration_nanos.fetch_add(
            elapsed.as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );

        Ok(BackpressureGuard {
            metrics: self.metrics.clone(),
            _permit: permit,
            acquired: false,
        })
    }

    /// タイムアウト付きで接続取得の許可を取得する
    pub async fn acquire_timeout(&self) -> DbResult<BackpressureGuard> {
        let timeout = self.config.acquire_timeout();
        match tokio::time::timeout(timeout, self.acquire()).await {
            Ok(result) => result,
            Err(_) => Err(DbError::connection_timeout(self.config.acquire_timeout_ms)),
        }
    }

    /// メトリクスへの参照を取得
    pub fn metrics(&self) -> &PoolMetrics {
        &self.metrics
    }

    /// 設定への参照を取得
    pub fn config(&self) -> &BackpressuredPoolConfig {
        &self.config
    }
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

    #[test]
    fn test_backpressured_pool_config_default() {
        let config = BackpressuredPoolConfig::default();
        assert_eq!(config.max_waiting, 100);
        assert_eq!(config.acquire_timeout_ms, 5_000);
    }

    #[test]
    fn test_backpressured_pool_config_builder() {
        let config = BackpressuredPoolConfig::default()
            .with_max_waiting(50)
            .with_acquire_timeout_ms(10_000);
        assert_eq!(config.max_waiting, 50);
        assert_eq!(config.acquire_timeout_ms, 10_000);
        assert_eq!(
            config.acquire_timeout(),
            std::time::Duration::from_millis(10_000)
        );
    }

    #[test]
    fn test_pool_metrics_initial() {
        let metrics = PoolMetrics::new();
        let snap = metrics.snapshot();
        assert_eq!(snap.active_connections, 0);
        assert_eq!(snap.idle_connections, 0);
        assert_eq!(snap.waiting_count, 0);
        assert_eq!(snap.rejected_total, 0);
        assert_eq!(snap.acquire_duration_nanos, 0);
    }

    #[tokio::test]
    async fn test_backpressured_pool_acquire() {
        let config = BackpressuredPoolConfig::default().with_max_waiting(2);
        let pool = BackpressuredPool::new(config);

        let mut guard1 = pool.acquire().await.unwrap();
        guard1.mark_acquired();
        assert_eq!(pool.metrics().snapshot().waiting_count, 1);
        assert_eq!(pool.metrics().snapshot().active_connections, 1);

        let mut guard2 = pool.acquire().await.unwrap();
        guard2.mark_acquired();
        assert_eq!(pool.metrics().snapshot().waiting_count, 2);

        // 3つ目は拒否される
        let result = pool.acquire().await;
        assert!(result.is_err());
        assert_eq!(pool.metrics().snapshot().rejected_total, 1);

        // guard を解放するとスロットが空く
        drop(guard1);
        assert_eq!(pool.metrics().snapshot().waiting_count, 1);

        let _guard3 = pool.acquire().await.unwrap();
        assert_eq!(pool.metrics().snapshot().waiting_count, 2);
    }

    #[tokio::test]
    async fn test_backpressured_pool_rejected_when_full() {
        let config = BackpressuredPoolConfig::default().with_max_waiting(1);
        let pool = BackpressuredPool::new(config);

        let _guard = pool.acquire().await.unwrap();

        // キュー満杯で拒否
        let result = pool.acquire().await;
        assert!(result.is_err());
        assert_eq!(pool.metrics().snapshot().rejected_total, 1);

        // もう一度試行しても拒否
        let result = pool.acquire().await;
        assert!(result.is_err());
        assert_eq!(pool.metrics().snapshot().rejected_total, 2);
    }

    #[tokio::test]
    async fn test_backpressured_pool_guard_drop_cleans_up() {
        let config = BackpressuredPoolConfig::default().with_max_waiting(2);
        let pool = BackpressuredPool::new(config);

        {
            let mut guard = pool.acquire().await.unwrap();
            guard.mark_acquired();
            assert_eq!(pool.metrics().snapshot().active_connections, 1);
        }
        // guard ドロップ後
        assert_eq!(pool.metrics().snapshot().active_connections, 0);
        assert_eq!(pool.metrics().snapshot().waiting_count, 0);
    }
}
