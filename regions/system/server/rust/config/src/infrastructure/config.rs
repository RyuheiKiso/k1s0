use serde::Deserialize;

use super::database::DatabaseConfig;
use super::kafka_producer::KafkaConfig;

/// AuthConfig は認証設定を表す。
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

/// Config はアプリケーション全体の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
}

/// AppConfig はアプリケーション基本設定を表す。
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

/// ServerConfig はサーバー設定を表す。
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
    8082
}

impl Config {
    /// YAML ファイルから設定を読み込む。
    pub fn from_yaml(content: &str) -> anyhow::Result<Self> {
        let config: Config = serde_yaml::from_str(content)?;
        Ok(config)
    }

    /// 設定ファイルパスから設定を読み込む。
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_yaml() {
        let yaml = r#"
app:
  name: "k1s0-config-server"
  version: "0.1.0"
  environment: "dev"
server:
  host: "0.0.0.0"
  port: 8082
"#;
        let config = Config::from_yaml(yaml).unwrap();
        assert_eq!(config.app.name, "k1s0-config-server");
        assert_eq!(config.app.version, "0.1.0");
        assert_eq!(config.app.environment, "dev");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8082);
    }

    #[test]
    fn test_config_defaults() {
        let yaml = r#"
app:
  name: "k1s0-config-server"
server: {}
"#;
        let config = Config::from_yaml(yaml).unwrap();
        assert_eq!(config.app.version, "0.1.0");
        assert_eq!(config.app.environment, "dev");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8082);
        assert!(config.database.is_none());
        assert!(config.kafka.is_none());
    }

    #[test]
    fn test_config_with_database() {
        let yaml = r#"
app:
  name: "k1s0-config-server"
server:
  port: 8082
database:
  host: "localhost"
  port: 5432
  name: "k1s0_config"
  user: "app"
  password: "pass"
"#;
        let config = Config::from_yaml(yaml).unwrap();
        assert!(config.database.is_some());

        let db = config.database.unwrap();
        assert_eq!(db.host, "localhost");
        assert_eq!(db.name, "k1s0_config");
    }

    #[test]
    fn test_config_with_kafka() {
        let yaml = r#"
app:
  name: "k1s0-config-server"
server:
  port: 8082
kafka:
  brokers:
    - "localhost:9092"
  topics:
    publish:
      - "k1s0.system.config.changed.v1"
    subscribe: []
"#;
        let config = Config::from_yaml(yaml).unwrap();
        assert!(config.kafka.is_some());

        let kafka = config.kafka.unwrap();
        assert_eq!(kafka.brokers.len(), 1);
        assert_eq!(kafka.topics.publish.len(), 1);
    }

    #[test]
    fn test_config_full() {
        let yaml = r#"
app:
  name: "k1s0-config-server"
  version: "0.1.0"
  environment: "prod"
server:
  host: "0.0.0.0"
  port: 8082
database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_config"
  user: "app"
  password: ""
  ssl_mode: "require"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
    - "kafka-1.messaging.svc.cluster.local:9092"
  consumer_group: "config-server.default"
  security_protocol: "SASL_SSL"
  sasl:
    mechanism: "SCRAM-SHA-512"
    username: ""
    password: ""
  topics:
    publish:
      - "k1s0.system.config.changed.v1"
    subscribe: []
"#;
        let config = Config::from_yaml(yaml).unwrap();
        assert_eq!(config.app.environment, "prod");
        assert!(config.database.is_some());
        assert!(config.kafka.is_some());

        let kafka = config.kafka.unwrap();
        assert_eq!(kafka.brokers.len(), 2);
        assert_eq!(kafka.security_protocol, "SASL_SSL");
        assert_eq!(kafka.sasl.mechanism, "SCRAM-SHA-512");
    }
}
