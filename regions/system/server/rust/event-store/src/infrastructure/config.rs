use serde::Deserialize;

/// 可観測性設定は server-common から共通型を使用する。
pub use k1s0_server_common::ObservabilityConfig;

/// Application configuration for event-store server.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
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

// Cargo.toml の package.version からバージョンを取得する（M-16 監査対応: ハードコード解消）
fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
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

/// DatabaseConfig はデータベース接続設定を表す（URL形式）。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
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

/// KafkaConfig は Kafka ブローカー接続設定を表す。
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

/// セキュリティデフォルト: 本番環境では SASL_SSL を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
}

fn default_producer_acks() -> String {
    "all".to_string()
}

fn default_producer_retries() -> u32 {
    3
}

/// 認証設定（JWT検証とJWKS取得を管理する）
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// JWT トークン検証設定
    pub jwt: JwtConfig,
    /// JWKS エンドポイント設定（省略時は JWKS 検証をスキップする）
    #[serde(default)]
    pub jwks: Option<JwksConfig>,
}

/// JWT トークン検証設定
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT 発行者 URL（Keycloak realm URL）
    pub issuer: String,
    /// JWT 受信者識別子
    pub audience: String,
}

/// JWKS エンドポイント設定
#[derive(Debug, Clone, Deserialize)]
pub struct JwksConfig {
    /// JWKS エンドポイント URL
    pub url: String,
    /// JWKS キャッシュ有効期限（秒）
    #[serde(default = "default_jwks_cache_ttl")]
    pub cache_ttl_secs: u64,
}

/// JWKS キャッシュのデフォルト有効期限（300秒）
fn default_jwks_cache_ttl() -> u64 {
    300
}

/// EventStoreConfig はイベントストア固有の設定を表す。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
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
#[allow(clippy::unwrap_used)]
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

    /// config.docker.yaml が正しくデシリアライズできることを検証する（回帰テスト・H-005 監査対応）
    #[test]
    fn config_docker_yaml_deserializes_correctly() {
        let yaml = include_str!("../../config/config.docker.yaml");
        let _config: Config = serde_yaml::from_str(yaml)
            .expect("config.docker.yaml のデシリアライズに失敗しました");
    }
}
