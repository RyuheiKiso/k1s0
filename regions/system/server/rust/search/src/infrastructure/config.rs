use serde::Deserialize;

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

/// Application configuration for search server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub opensearch: Option<OpenSearchConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
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

fn default_version() -> String {
    "0.1.0".to_string()
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
    8094
}

fn default_grpc_port() -> u16 {
    50051
}

/// OpenSearchConfig 縺ｯ OpenSearch 謗･邯壹・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct OpenSearchConfig {
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_index_prefix")]
    pub index_prefix: String,
}

fn default_index_prefix() -> String {
    "k1s0-".to_string()
}

/// KafkaConfig 縺ｯ Kafka 繝悶Ο繝ｼ繧ｫ繝ｼ謗･邯壹・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    #[serde(default = "default_consumer_group")]
    pub consumer_group: String,
    /// Consumer topic: 繧､繝ｳ繝・ャ繧ｯ繧ｹ逋ｻ骭ｲ隕∵ｱ・
    pub topic: String,
}

/// セキュリティデフォルト: 本番環境では SASL_SSL を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
}

fn default_consumer_group() -> String {
    "search-server-consumer".to_string()
}

/// CacheConfig 縺ｯ繧､繝ｳ繝｡繝｢繝ｪ繧ｭ繝｣繝・す繝･縺ｮ險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_max_entries")]
    pub max_entries: u64,
    #[serde(default = "default_ttl_seconds")]
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: default_max_entries(),
            ttl_seconds: default_ttl_seconds(),
        }
    }
}

fn default_max_entries() -> u64 {
    1000
}

fn default_ttl_seconds() -> u64 {
    30
}

#[derive(Debug, Clone, Default, Deserialize)]
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
    fn test_cache_config_defaults() {
        let cache = CacheConfig::default();
        assert_eq!(cache.max_entries, 1000);
        assert_eq!(cache.ttl_seconds, 30);
    }

    #[test]
    fn test_opensearch_config_deserialization() {
        let yaml = r#"
url: "https://opensearch:9200"
username: "app"
password: ""
index_prefix: "k1s0-"
"#;
        let cfg: OpenSearchConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.index_prefix, "k1s0-");
        assert_eq!(cfg.url, "https://opensearch:9200");
    }
}
