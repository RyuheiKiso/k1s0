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

/// `AuthConfig` は JWT 認証設定を表す。
/// config.docker.yaml の nested 形式（auth.jwt / auth.jwks）に対応する。
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// JWT トークン検証に必要な issuer / audience の設定
    pub jwt: JwtConfig,
    /// JWKS エンドポイントの設定（省略時は JWKS 検証を行わない）
    #[serde(default)]
    pub jwks: Option<JwksConfig>,
}

/// `JwtConfig` は JWT トークンの issuer と audience を保持する。
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT の発行者（issuer）URL
    pub issuer: String,
    /// JWT の想定オーディエンス
    pub audience: String,
}

/// `JwksConfig` は JWKS エンドポイントとキャッシュ TTL を保持する。
#[derive(Debug, Clone, Deserialize)]
pub struct JwksConfig {
    /// JWKS エンドポイント URL
    pub url: String,
    /// JWKS キャッシュの TTL（秒）。デフォルトは 300 秒。
    #[serde(default = "default_jwks_cache_ttl")]
    pub cache_ttl_secs: u64,
}

/// JWKS キャッシュ TTL のデフォルト値（300 秒）
fn default_jwks_cache_ttl() -> u64 {
    300
}

/// `AppConfig` はアプリケーション設定。
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

/// `ServerConfig` はサーバー設定。
/// `host/port/grpc_port` は startup.rs でバインドアドレス構築に使用する。
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

/// `ServiceEndpoint` は外部サービスのエンドポイント。
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceEndpoint {
    pub host: String,
    pub port: u16,
}

/// `SagaConfig` は Saga 固有の設定。
/// `workflow_dir` は startup.rs で `WorkflowLoader` の初期化に使用する。
#[derive(Debug, Clone, Deserialize)]
pub struct SagaConfig {
    #[serde(default = "default_workflow_dir")]
    pub workflow_dir: String,
}

fn default_workflow_dir() -> String {
    "workflows".to_string()
}

impl Default for SagaConfig {
    fn default() -> Self {
        Self {
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
  board-server:
    host: "localhost"
    port: 50051
saga:
  workflow_dir: "workflows"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.name, "saga-server");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.grpc_port, 50051);
        assert_eq!(config.services.len(), 1);
        assert_eq!(config.saga.workflow_dir, "workflows");
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
        assert_eq!(config.saga.workflow_dir, "workflows");
    }

    /// config.docker.yaml が正しくデシリアライズできることを検証する（回帰テスト）
    #[test]
    fn config_docker_yaml_deserializes_correctly() {
        let yaml = include_str!("../../config/config.docker.yaml");
        let config: Config =
            serde_yaml::from_str(yaml).expect("config.docker.yaml のデシリアライズに失敗しました");
        // AuthConfig が正しく読み込まれていることを確認する
        assert!(config.auth.is_some(), "auth 設定が読み込まれていません");
        let auth = config.auth.unwrap();
        assert!(!auth.jwt.issuer.is_empty(), "issuer が空です");
    }
}
