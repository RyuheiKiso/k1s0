use serde::Deserialize;

/// Application configuration for event-store server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
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
    9090
}

/// DatabaseConfig はデータベース接続の設定を表す（URL形式）。
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

/// KafkaConfig は Kafka ブローカー接続の設定を表す。
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

/// AuthConfig は JWT 認証設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwks_url: String,
}

/// EventStoreConfig はイベントストア固有の設定を表す。
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
