use serde::Deserialize;

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

/// Application configuration for session server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub redis: Option<RedisConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub session: SessionConfig,
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
    8102
}

fn default_grpc_port() -> u16 {
    9090
}

/// DatabaseConfig は PostgreSQL データベース接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 {
    10
}

/// RedisConfig は Redis 接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_connect_timeout_seconds")]
    pub connect_timeout_seconds: u64,
}

fn default_pool_size() -> u32 {
    10
}

fn default_connect_timeout_seconds() -> u64 {
    3
}

/// KafkaConfig は Kafka ブローカー接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    /// Consumer topic: 全セッション失効要求
    pub topic_revoke_all: String,
    /// Producer topic: セッション作成
    pub topic_created: String,
    /// Producer topic: セッション失効
    pub topic_revoked: String,
    #[serde(default = "default_consumer_group")]
    pub consumer_group: String,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

fn default_consumer_group() -> String {
    "session-server-consumer".to_string()
}

/// SessionConfig はセッション管理固有の設定を表す。
/// フィールド名は既存の main.rs の参照パターンに合わせている。
#[derive(Debug, Clone, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_ttl")]
    pub default_ttl_seconds: i64,
    #[serde(default = "default_max_ttl")]
    pub max_ttl_seconds: i64,
    #[serde(default = "default_max_devices_per_user")]
    pub max_devices_per_user: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_ttl_seconds: default_ttl(),
            max_ttl_seconds: default_max_ttl(),
            max_devices_per_user: default_max_devices_per_user(),
        }
    }
}

fn default_ttl() -> i64 {
    3600
}

fn default_max_ttl() -> i64 {
    86400
}

fn default_max_devices_per_user() -> u32 {
    10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_defaults() {
        let cfg = SessionConfig::default();
        assert_eq!(cfg.default_ttl_seconds, 3600);
        assert_eq!(cfg.max_ttl_seconds, 86400);
        assert_eq!(cfg.max_devices_per_user, 10);
    }

    #[test]
    fn test_redis_config_deserialization() {
        let yaml = r#"
url: "redis://localhost:6379"
pool_size: 10
connect_timeout_seconds: 3
"#;
        let cfg: RedisConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.pool_size, 10);
        assert_eq!(cfg.connect_timeout_seconds, 3);
    }
}
