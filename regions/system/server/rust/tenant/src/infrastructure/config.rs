// secrecy クレートを使用してデータベースパスワードを Secret<String> で保持し、Debug 出力への漏洩を防ぐ。
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

use super::kafka_producer::KafkaConfig;
use super::keycloak_admin::KeycloakAdminConfig;

/// アプリケーション全体の設定。
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub keycloak: Option<KeycloakAdminConfig>,
    /// テナント検索結果のインメモリキャッシュ設定
    #[serde(default)]
    pub cache: CacheConfig,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwks_url: String,
    pub issuer: String,
    pub audience: String,
    #[serde(default = "default_jwks_cache_ttl_secs")]
    pub jwks_cache_ttl_secs: u64,
}

fn default_jwks_cache_ttl_secs() -> u64 {
    3600
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_environment")]
    pub environment: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_environment() -> String {
    "dev".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
}

fn default_http_port() -> u16 {
    8080
}

fn default_grpc_port() -> u16 {
    50051
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub trace: TraceConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TraceConfig {
    #[serde(default = "default_trace_enabled")]
    pub enabled: bool,
    #[serde(default = "default_trace_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_trace_sample_rate")]
    pub sample_rate: f64,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            enabled: default_trace_enabled(),
            endpoint: default_trace_endpoint(),
            sample_rate: default_trace_sample_rate(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    #[serde(default = "default_metrics_enabled")]
    pub enabled: bool,
    #[serde(default = "default_metrics_path")]
    pub path: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: default_metrics_enabled(),
            path: default_metrics_path(),
        }
    }
}

fn default_trace_enabled() -> bool {
    true
}

fn default_trace_endpoint() -> String {
    "http://otel-collector.observability:4317".to_string()
}

fn default_trace_sample_rate() -> f64 {
    1.0
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "json".to_string()
}

fn default_metrics_enabled() -> bool {
    true
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_host")]
    pub host: String,
    #[serde(default = "default_db_port")]
    pub port: u16,
    #[serde(default = "default_db_name")]
    pub name: String,
    #[serde(default = "default_db_user")]
    pub user: String,
    // パスワードは Secret<String> で保持し、Debug トレイトでは [REDACTED] と表示される
    // パスワードは必須項目のため serde(default) を設定しない（Secret<String> は Default 未実装）
    pub password: Secret<String>,
    #[serde(default = "default_db_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_max_idle_conns")]
    pub max_idle_conns: u32,
    #[serde(default = "default_conn_max_lifetime")]
    pub conn_max_lifetime: String,
}

impl DatabaseConfig {
    pub fn connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            // expose_secret() でパスワードを取り出す。戻り値の URL はログに出力しないこと。
            self.user, self.password.expose_secret(), self.host, self.port, self.name, self.ssl_mode
        )
    }
}

fn default_db_host() -> String {
    "localhost".to_string()
}

fn default_db_port() -> u16 {
    5432
}

fn default_db_name() -> String {
    "k1s0_system".to_string()
}

fn default_db_user() -> String {
    "app".to_string()
}

fn default_db_ssl_mode() -> String {
    "disable".to_string()
}

pub fn default_max_connections() -> u32 {
    25
}

pub fn default_max_idle_conns() -> u32 {
    5
}

pub fn default_conn_max_lifetime() -> String {
    "5m".to_string()
}

/// テナントキャッシュの設定。find_by_id / find_by_name の結果をインメモリにキャッシュする。
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    /// キャッシュエントリの有効期限（秒）。デフォルト60秒。
    #[serde(default = "default_cache_ttl_secs")]
    pub ttl_secs: u64,
    /// キャッシュの最大エントリ数。デフォルト100。
    #[serde(default = "default_cache_max_entries")]
    pub max_entries: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl_secs: default_cache_ttl_secs(),
            max_entries: default_cache_max_entries(),
        }
    }
}

/// キャッシュ TTL のデフォルト値（60秒）
fn default_cache_ttl_secs() -> u64 {
    60
}

/// キャッシュ最大エントリ数のデフォルト値（100件）
fn default_cache_max_entries() -> u64 {
    100
}

pub fn parse_pool_duration(value: &str) -> Option<std::time::Duration> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(v) = trimmed.strip_suffix("ms") {
        return v.parse::<u64>().ok().map(std::time::Duration::from_millis);
    }
    if let Some(v) = trimmed.strip_suffix('s') {
        return v.parse::<u64>().ok().map(std::time::Duration::from_secs);
    }
    if let Some(v) = trimmed.strip_suffix('m') {
        return v
            .parse::<u64>()
            .ok()
            .map(|mins| std::time::Duration::from_secs(mins * 60));
    }
    if let Some(v) = trimmed.strip_suffix('h') {
        return v
            .parse::<u64>()
            .ok()
            .map(|hours| std::time::Duration::from_secs(hours * 3600));
    }
    trimmed
        .parse::<u64>()
        .ok()
        .map(std::time::Duration::from_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_connection_url() {
        let cfg = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "k1s0_system".to_string(),
            user: "app".to_string(),
            password: Secret::new("pass".to_string()),
            ssl_mode: "disable".to_string(),
            max_connections: 25,
            max_idle_conns: 5,
            conn_max_lifetime: "5m".to_string(),
        };
        assert_eq!(
            cfg.connection_url(),
            "postgresql://app:pass@localhost:5432/k1s0_system?sslmode=disable"
        );
    }

    #[test]
    fn test_parse_pool_duration() {
        assert_eq!(
            parse_pool_duration("5m"),
            Some(std::time::Duration::from_secs(300))
        );
        assert_eq!(
            parse_pool_duration("30s"),
            Some(std::time::Duration::from_secs(30))
        );
        assert_eq!(
            parse_pool_duration("1h"),
            Some(std::time::Duration::from_secs(3600))
        );
        assert_eq!(parse_pool_duration(""), None);
    }
}
