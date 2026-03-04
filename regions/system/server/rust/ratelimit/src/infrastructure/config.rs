use serde::Deserialize;

use super::database::DatabaseConfig;

/// RedisConfig 縺ｯ Redis 謗･邯夊ｨｭ螳壹・
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
    4
}

fn default_redis_timeout_ms() -> u64 {
    2000
}

/// AuthConfig 縺ｯ JWT 隱崎ｨｼ險ｭ螳壹ｒ陦ｨ縺吶・
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

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_environment() -> String {
    "dev".to_string()
}

#[derive(Debug, Clone, Deserialize)]
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

fn default_fail_open() -> bool {
    true
}

fn default_limit() -> u32 {
    100
}

fn default_window_seconds() -> u32 {
    60
}


#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default = "default_otlp_endpoint")]
    pub otlp_endpoint: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_log_format")]
    pub log_format: String,
    #[serde(default = "default_metrics_enabled")]
    pub metrics_enabled: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: default_otlp_endpoint(),
            log_level: default_log_level(),
            log_format: default_log_format(),
            metrics_enabled: default_metrics_enabled(),
        }
    }
}

fn default_otlp_endpoint() -> String {
    "http://otel-collector.observability:4317".to_string()
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
#[cfg(test)]
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
        assert!(cfg.ratelimit.fail_open);
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
        assert!(cfg.ratelimit.fail_open);
        assert_eq!(cfg.ratelimit.default_limit, 100);
        assert_eq!(cfg.ratelimit.default_window_seconds, 60);
    }
}


