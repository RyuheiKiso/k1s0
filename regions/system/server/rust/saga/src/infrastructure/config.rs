use std::collections::HashMap;

use serde::Deserialize;

use crate::infrastructure::database::DatabaseConfig;
use crate::infrastructure::kafka_producer::KafkaConfig;

/// Config はアプリケーション全体の設定。
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub services: HashMap<String, ServiceEndpoint>,
    #[serde(default)]
    pub saga: SagaConfig,
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

/// ServiceEndpoint は外部サービスのエンドポイント。
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceEndpoint {
    pub host: String,
    pub port: u16,
}

/// SagaConfig はSaga固有の設定。
#[derive(Debug, Clone, Deserialize)]
pub struct SagaConfig {
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    #[serde(default = "default_workflow_dir")]
    pub workflow_dir: String,
}

fn default_max_concurrent() -> usize {
    100
}

fn default_workflow_dir() -> String {
    "workflows".to_string()
}

impl Default for SagaConfig {
    fn default() -> Self {
        Self {
            max_concurrent: default_max_concurrent(),
            workflow_dir: default_workflow_dir(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
app:
  name: "saga-server"
  version: "0.1.0"
  environment: "dev"
server:
  host: "0.0.0.0"
  port: 8080
services:
  inventory-service:
    host: "localhost"
    port: 50051
saga:
  max_concurrent: 50
  workflow_dir: "workflows"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.name, "saga-server");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.services.len(), 1);
        assert_eq!(config.saga.max_concurrent, 50);
    }

    #[test]
    fn test_config_defaults() {
        let yaml = r#"
app:
  name: "saga-server"
server: {}
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.version, "0.1.0");
        assert_eq!(config.app.environment, "dev");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.saga.max_concurrent, 100);
        assert_eq!(config.saga.workflow_dir, "workflows");
    }
}
