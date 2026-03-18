use serde::Deserialize;

use crate::infrastructure::database::DatabaseConfig;
use crate::infrastructure::kafka::KafkaConfig;

/// Config 縺ｯ繧｢繝励Μ繧ｱ繝ｼ繧ｷ繝ｧ繝ｳ蜈ｨ菴薙・險ｭ螳壹・
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

/// AppConfig 縺ｯ繧｢繝励Μ繧ｱ繝ｼ繧ｷ繝ｧ繝ｳ險ｭ螳壹・
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

/// ServerConfig 縺ｯ繧ｵ繝ｼ繝舌・險ｭ螳壹・
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
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
app:
  name: "dlq-manager"
  version: "0.1.0"
  environment: "dev"
server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 50051
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.name, "dlq-manager");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.grpc_port, 50051);
        assert!(config.database.is_none());
        assert!(config.kafka.is_none());
    }

    #[test]
    fn test_config_defaults() {
        let yaml = r#"
app:
  name: "dlq-manager"
server: {}
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.version, "0.1.0");
        assert_eq!(config.app.environment, "dev");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.grpc_port, 50051);
    }

    #[test]
    fn test_config_with_database() {
        let yaml = r#"
app:
  name: "dlq-manager"
server:
  port: 9090
database:
  host: "localhost"
  port: 5432
  name: "k1s0_system"
  user: "app"
  password: "secret"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.database.is_some());
        let db = config.database.unwrap();
        assert_eq!(db.host, "localhost");
        assert_eq!(db.name, "k1s0_system");
    }

    #[test]
    fn test_config_with_kafka() {
        let yaml = r#"
app:
  name: "dlq-manager"
server:
  port: 8080
kafka:
  brokers:
    - "localhost:9092"
  consumer_group: "dlq-manager.default"
  dlq_topic_pattern: "*.v1.dlq"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.kafka.is_some());
        let kafka = config.kafka.unwrap();
        assert_eq!(kafka.brokers.len(), 1);
        assert_eq!(kafka.dlq_topic_pattern, "*.v1.dlq");
    }
}
