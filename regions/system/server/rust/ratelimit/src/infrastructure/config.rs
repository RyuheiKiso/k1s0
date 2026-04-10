use serde::Deserialize;

use super::database::DatabaseConfig;

/// `RedisConfig` 縺ｯ Redis 謗･邯夊ｨｭ螳壹・
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_redis_url")]
    pub url: String,
    #[serde(default = "default_redis_pool_size")]
    pub pool_size: usize,
    #[serde(default = "default_redis_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_redis_url() -> String {
    "redis://127.0.0.1:6379".to_string()
}

fn default_redis_pool_size() -> usize {
    20
}

fn default_redis_timeout_ms() -> u64 {
    100
}

/// `AuthConfig` 縺ｯ JWT 隱崎ｨｼ險ｭ螳壹ｒ陦ｨ縺吶・
/// `AuthConfig` は認証設定を保持する（nested 形式: jwt + jwks）。
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// JWT トークンの検証に使用する issuer / audience 設定
    pub jwt: JwtConfig,
    /// JWKS エンドポイントの設定（オプション）
    #[serde(default)]
    pub jwks: Option<JwksConfig>,
}

/// `JwtConfig` は JWT トークン検証の issuer / audience を保持する。
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT 発行者（issuer）
    pub issuer: String,
    /// JWT 対象者（audience）
    pub audience: String,
}

/// `JwksConfig` は JWKS エンドポイントの URL とキャッシュ TTL を保持する。
#[derive(Debug, Clone, Deserialize)]
pub struct JwksConfig {
    /// JWKS エンドポイント URL
    pub url: String,
    /// JWKS キャッシュ TTL（秒）。デフォルト 300 秒。
    #[serde(default = "default_jwks_cache_ttl_secs")]
    pub cache_ttl_secs: u64,
}

/// JWKS キャッシュ TTL のデフォルト値（300 秒）
fn default_jwks_cache_ttl_secs() -> u64 {
    300
}

/// Application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub redis: Option<RedisConfig>,
    #[serde(default)]
    pub ratelimit: RatelimitConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_environment")]
    pub environment: String,
}

// Cargo.toml の package.version からバージョンを取得する（M-16 監査対応: ハードコード解消）
fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_environment() -> String {
    "dev".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_grpc_port() -> u16 {
    50051
}

#[derive(Debug, Clone, Deserialize)]
pub struct RatelimitConfig {
    #[serde(default = "default_fail_open")]
    pub fail_open: bool,
    #[serde(default = "default_limit")]
    pub default_limit: u32,
    #[serde(default = "default_window_seconds")]
    pub default_window_seconds: u32,
}

impl Default for RatelimitConfig {
    fn default() -> Self {
        Self {
            fail_open: default_fail_open(),
            default_limit: default_limit(),
            default_window_seconds: default_window_seconds(),
        }
    }
}

/// デフォルトは fail-closed（ADR-0048）。Redis 障害時もレートリミットを維持しセキュリティを確保する。
fn default_fail_open() -> bool {
    false
}

fn default_limit() -> u32 {
    100
}

fn default_window_seconds() -> u32 {
    60
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
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
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
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
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
app:
  name: "ratelimit-server"
  version: "0.1.0"
  environment: "dev"
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: "localhost"
  port: 5432
  name: "ratelimit_db"
  user: "dev"
  password: "dev"
redis:
  url: "redis://localhost:6379"
  pool_size: 4
  timeout_ms: 2000
ratelimit:
  fail_open: true
  default_limit: 100
  default_window_seconds: 60
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.app.name, "ratelimit-server");
        assert_eq!(cfg.server.port, 8080);
        assert_eq!(cfg.server.grpc_port, 50051);
        assert!(cfg.database.is_some());
        assert!(cfg.redis.is_some());
        let redis = cfg.redis.unwrap();
        assert_eq!(redis.url, "redis://localhost:6379");
        assert_eq!(redis.pool_size, 4);
        assert_eq!(redis.timeout_ms, 2000);
        // YAML で明示的に fail_open: true を指定した場合は true になることを確認する
        assert!(cfg.ratelimit.fail_open);
    }

    /// config.docker.yaml が Config にデシリアライズできることを検証する
    #[test]
    fn config_docker_yaml_deserializes_correctly() {
        let yaml = include_str!("../../config/config.docker.yaml");
        let _config: Config =
            serde_yaml::from_str(yaml).expect("config.docker.yaml のデシリアライズに失敗しました");
    }

    #[test]
    fn test_config_minimal() {
        let yaml = r#"
app:
  name: "ratelimit-server"
server:
  port: 8080
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.app.name, "ratelimit-server");
        assert!(cfg.database.is_none());
        assert!(cfg.redis.is_none());
        // デフォルトは fail-closed（ADR-0048）。YAML 未指定時は false になることを確認する。
        assert!(!cfg.ratelimit.fail_open);
        assert_eq!(cfg.ratelimit.default_limit, 100);
        assert_eq!(cfg.ratelimit.default_window_seconds, 60);
    }
}
