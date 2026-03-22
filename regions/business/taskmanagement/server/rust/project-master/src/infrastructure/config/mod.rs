// サーバー設定モジュール。YAML ファイルから設定を読み込む。
pub mod auth_config;

use serde::Deserialize;

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

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    #[serde(default = "default_environment")]
    pub environment: String,
}

fn default_environment() -> String { "development".to_string() }

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ObservabilityConfig {}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
}

fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8210 }
fn default_grpc_port() -> u16 { 9210 }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            grpc_port: default_grpc_port(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    #[serde(default = "default_db_port")]
    pub port: u16,
    pub name: String,
    #[serde(default = "default_schema")]
    pub schema: String,
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_max_idle_conns")]
    pub max_idle_conns: u32,
    #[serde(default = "default_conn_max_lifetime")]
    pub conn_max_lifetime: u64,
}

fn default_db_port() -> u16 { 5432 }
fn default_schema() -> String { "project_master".to_string() }
fn default_ssl_mode() -> String { "prefer".to_string() }
fn default_max_connections() -> u32 { 25 }
fn default_max_idle_conns() -> u32 { 5 }
fn default_conn_max_lifetime() -> u64 { 300 }

impl DatabaseConfig {
    pub fn connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}&options=-c search_path%3D{}",
            self.user, self.password, self.host, self.port, self.name, self.ssl_mode, self.schema
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub project_type_changed_topic: String,
    pub status_definition_changed_topic: String,
    pub tenant_extension_changed_topic: String,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
}

fn default_security_protocol() -> String { "PLAINTEXT".to_string() }

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwks_url: String,
    pub issuer: String,
    pub audience: String,
    #[serde(default = "default_jwks_cache_ttl")]
    pub jwks_cache_ttl_secs: u64,
}

fn default_jwks_cache_ttl() -> u64 { 300 }
