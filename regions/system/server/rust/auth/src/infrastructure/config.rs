use serde::Deserialize;

use super::database::DatabaseConfig;
use super::kafka_producer::KafkaConfig;
use super::keycloak_client::KeycloakConfig;

/// Application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub keycloak: Option<KeycloakConfig>,
    #[serde(default)]
    pub permission_cache: PermissionCacheConfig,
    #[serde(default)]
    pub audit: AuditConfig,
    #[serde(default)]
    pub keycloak_admin: KeycloakAdminConfig,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
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

#[allow(dead_code)]
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
pub struct AuthConfig {
    pub jwt: JwtConfig,
    #[serde(default)]
    pub jwks: Option<JwksConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwksConfig {
    pub url: String,
    #[serde(default = "default_cache_ttl_secs")]
    pub cache_ttl_secs: u64,
}

pub fn default_cache_ttl_secs() -> u64 {
    600
}

/// Permission cache configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct PermissionCacheConfig {
    #[serde(default = "default_permission_cache_ttl_secs")]
    pub ttl_secs: u64,
    #[serde(default = "default_refresh_on_miss")]
    pub refresh_on_miss: bool,
}

impl Default for PermissionCacheConfig {
    fn default() -> Self {
        Self {
            ttl_secs: default_permission_cache_ttl_secs(),
            refresh_on_miss: default_refresh_on_miss(),
        }
    }
}

fn default_permission_cache_ttl_secs() -> u64 {
    300
}

fn default_refresh_on_miss() -> bool {
    true
}

/// Audit configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct AuditConfig {
    #[serde(default)]
    pub kafka_enabled: bool,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            kafka_enabled: false,
            retention_days: default_retention_days(),
        }
    }
}

fn default_retention_days() -> u32 {
    365
}

/// Keycloak admin client configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct KeycloakAdminConfig {
    #[serde(default = "default_keycloak_admin_token_cache_ttl_secs")]
    pub token_cache_ttl_secs: u64,
    #[serde(default = "default_keycloak_admin_refresh_interval_secs")]
    pub refresh_interval_secs: u64,
}

impl Default for KeycloakAdminConfig {
    fn default() -> Self {
        Self {
            token_cache_ttl_secs: default_keycloak_admin_token_cache_ttl_secs(),
            refresh_interval_secs: default_keycloak_admin_refresh_interval_secs(),
        }
    }
}

fn default_keycloak_admin_token_cache_ttl_secs() -> u64 {
    300
}

fn default_keycloak_admin_refresh_interval_secs() -> u64 {
    300
}

pub fn parse_pool_duration(input: &str) -> Option<std::time::Duration> {
    let value = input.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(raw) = value.strip_suffix("ms") {
        return raw
            .trim()
            .parse::<u64>()
            .ok()
            .map(std::time::Duration::from_millis);
    }
    if let Some(raw) = value.strip_suffix('s') {
        return raw
            .trim()
            .parse::<u64>()
            .ok()
            .map(std::time::Duration::from_secs);
    }
    if let Some(raw) = value.strip_suffix('m') {
        return raw
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| std::time::Duration::from_secs(v * 60));
    }
    if let Some(raw) = value.strip_suffix('h') {
        return raw
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| std::time::Duration::from_secs(v * 60 * 60));
    }
    value
        .parse::<u64>()
        .ok()
        .map(std::time::Duration::from_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_cache_config_defaults() {
        let cfg = PermissionCacheConfig::default();
        assert_eq!(cfg.ttl_secs, 300);
        assert!(cfg.refresh_on_miss);
    }

    #[test]
    fn test_audit_config_defaults() {
        let cfg = AuditConfig::default();
        assert!(!cfg.kafka_enabled);
        assert_eq!(cfg.retention_days, 365);
    }

    #[test]
    fn test_keycloak_admin_config_defaults() {
        let cfg = KeycloakAdminConfig::default();
        assert_eq!(cfg.token_cache_ttl_secs, 300);
        assert_eq!(cfg.refresh_interval_secs, 300);
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
            parse_pool_duration("100ms"),
            Some(std::time::Duration::from_millis(100))
        );
        assert_eq!(
            parse_pool_duration("1h"),
            Some(std::time::Duration::from_secs(3600))
        );
        assert_eq!(parse_pool_duration(""), None);
    }
}
