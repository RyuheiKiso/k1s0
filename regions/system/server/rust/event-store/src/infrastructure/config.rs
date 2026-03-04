use serde::Deserialize;

/// Application configuration for event-store server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub event_store: EventStoreConfig,
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
    8099
}

fn default_grpc_port() -> u16 {
    50051
}

/// DatabaseConfig 縺ｯ繝・・繧ｿ繝吶・繧ｹ謗･邯壹・險ｭ螳壹ｒ陦ｨ縺呻ｼ・RL蠖｢蠑擾ｼ峨・
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_schema")]
    pub schema: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_connect_timeout_seconds")]
    pub connect_timeout_seconds: u64,
}

fn default_schema() -> String {
    "event_store".to_string()
}

fn default_max_connections() -> u32 {
    20
}

fn default_min_connections() -> u32 {
    5
}

fn default_connect_timeout_seconds() -> u64 {
    5
}

/// KafkaConfig 縺ｯ Kafka 繝悶Ο繝ｼ繧ｫ繝ｼ謗･邯壹・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    pub topic_published: String,
    #[serde(default = "default_producer_acks")]
    pub producer_acks: String,
    #[serde(default = "default_producer_retries")]
    pub producer_retries: u32,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

fn default_producer_acks() -> String {
    "all".to_string()
}

fn default_producer_retries() -> u32 {
    3
}

/// AuthConfig 縺ｯ JWT 隱崎ｨｼ險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwks_url: String,
    pub issuer: String,
    pub audience: String,
    #[serde(default = "default_jwks_cache_ttl")]
    pub jwks_cache_ttl_secs: u64,
}

fn default_jwks_cache_ttl() -> u64 {
    300
}

/// EventStoreConfig 縺ｯ繧､繝吶Φ繝医せ繝医い蝗ｺ譛峨・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct EventStoreConfig {
    #[serde(default = "default_max_events_per_append")]
    pub max_events_per_append: u32,
    #[serde(default = "default_max_page_size")]
    pub max_page_size: u32,
}

impl Default for EventStoreConfig {
    fn default() -> Self {
        Self {
            max_events_per_append: default_max_events_per_append(),
            max_page_size: default_max_page_size(),
        }
    }
}

fn default_max_events_per_append() -> u32 {
    100
}

fn default_max_page_size() -> u32 {
    200
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
    fn test_event_store_config_defaults() {
        let cfg = EventStoreConfig::default();
        assert_eq!(cfg.max_events_per_append, 100);
        assert_eq!(cfg.max_page_size, 200);
    }

    #[test]
    fn test_database_config_deserialization() {
        let yaml = r#"
url: "postgresql://app:@localhost:5432/k1s0_system"
schema: "event_store"
max_connections: 20
min_connections: 5
connect_timeout_seconds: 5
"#;
        let cfg: DatabaseConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.schema, "event_store");
        assert_eq!(cfg.max_connections, 20);
    }
}


