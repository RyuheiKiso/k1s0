use serde::Deserialize;

use crate::infrastructure::database::DatabaseConfig;
use crate::infrastructure::kafka::KafkaConfig;

/// Config はアプリケーション全体の設定。
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
}

/// AppConfig はアプリケーション設定。
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

/// ServerConfig はサーバー設定。
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
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
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.name, "dlq-manager");
        assert_eq!(config.server.port, 8080);
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
  name: "k1s0_dlq"
  user: "app"
  password: "secret"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.database.is_some());
        let db = config.database.unwrap();
        assert_eq!(db.host, "localhost");
        assert_eq!(db.name, "k1s0_dlq");
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
  dlq_topic_pattern: "*.dlq.v1"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.kafka.is_some());
        let kafka = config.kafka.unwrap();
        assert_eq!(kafka.brokers.len(), 1);
        assert_eq!(kafka.dlq_topic_pattern, "*.dlq.v1");
    }
}
