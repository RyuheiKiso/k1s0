use serde::Deserialize;

/// Application configuration for notification server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub notification: NotificationConfig,
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
    8092
}

fn default_grpc_port() -> u16 {
    50051
}

/// DatabaseConfig はデータベース接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default = "default_max_open_conns")]
    pub max_open_conns: u32,
    #[serde(default = "default_max_idle_conns")]
    pub max_idle_conns: u32,
    #[serde(default = "default_conn_max_lifetime")]
    pub conn_max_lifetime: String,
}

fn default_ssl_mode() -> String {
    "disable".to_string()
}

fn default_max_open_conns() -> u32 {
    25
}

fn default_max_idle_conns() -> u32 {
    5
}

fn default_conn_max_lifetime() -> String {
    "5m".to_string()
}

impl DatabaseConfig {
    /// PostgreSQL 接続 URL を生成する。
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, self.ssl_mode
        )
    }
}

/// KafkaConfig は Kafka ブローカー接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    /// Consumer topic: 通知リクエスト受信
    pub topic_requested: String,
    /// Producer topic: 通知配信完了
    pub topic_delivered: String,
    #[serde(default = "default_consumer_group")]
    pub consumer_group: String,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

fn default_consumer_group() -> String {
    "notification-server-consumer".to_string()
}

/// AuthConfig は JWT 認証の設定を表す。
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

/// NotificationConfig は通知サービス固有の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_retry_max_attempts")]
    pub retry_max_attempts: u32,
    #[serde(default = "default_retry_initial_delay_secs")]
    pub retry_initial_delay_secs: u64,
    #[serde(default = "default_retry_max_delay_secs")]
    pub retry_max_delay_secs: u64,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            retry_max_attempts: default_retry_max_attempts(),
            retry_initial_delay_secs: default_retry_initial_delay_secs(),
            retry_max_delay_secs: default_retry_max_delay_secs(),
        }
    }
}

fn default_retry_max_attempts() -> u32 {
    5
}

fn default_retry_initial_delay_secs() -> u64 {
    1
}

fn default_retry_max_delay_secs() -> u64 {
    60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_config_defaults() {
        let cfg = NotificationConfig::default();
        assert_eq!(cfg.retry_max_attempts, 5);
        assert_eq!(cfg.retry_initial_delay_secs, 1);
        assert_eq!(cfg.retry_max_delay_secs, 60);
    }

    #[test]
    fn test_database_connection_url() {
        let cfg = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "k1s0_system".to_string(),
            user: "app".to_string(),
            password: "pass".to_string(),
            ssl_mode: "disable".to_string(),
            max_open_conns: 25,
            max_idle_conns: 5,
            conn_max_lifetime: "5m".to_string(),
        };
        assert_eq!(
            cfg.connection_url(),
            "postgres://app:pass@localhost:5432/k1s0_system?sslmode=disable"
        );
    }
}
