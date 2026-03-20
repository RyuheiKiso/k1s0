use std::collections::HashMap;

use serde::Deserialize;

use crate::infrastructure::database::DatabaseConfig;
use crate::infrastructure::kafka_producer::KafkaConfig;

/// 可観測性設定は server-common から共通型を使用する。
pub use k1s0_server_common::ObservabilityConfig;

/// Config はアプリケーション全体の設定。
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
    pub services: HashMap<String, ServiceEndpoint>,
    #[serde(default)]
    pub saga: SagaConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
}

/// AuthConfig は JWT 検証設定を表す。
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

/// ServiceEndpoint は外部サービスのエンドポイント。
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceEndpoint {
    pub host: String,
    pub port: u16,
}

/// SagaConfig は Saga 固有の設定。
#[allow(dead_code)]
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
#[allow(clippy::unwrap_used)]
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
  grpc_port: 50051
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
        assert_eq!(config.server.grpc_port, 50051);
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
        assert_eq!(config.server.grpc_port, 50051);
        assert_eq!(config.saga.max_concurrent, 100);
        assert_eq!(config.saga.workflow_dir, "workflows");
    }
}
