use serde::Deserialize;

/// Application configuration for quota server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub redis: Option<RedisConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub quota: QuotaConfig,
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
    8097
}

fn default_grpc_port() -> u16 {
    50051
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
    "quota".to_string()
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    2
}

fn default_connect_timeout_seconds() -> u64 {
    5
}

/// RedisConfig は Redis 接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_key_prefix")]
    pub key_prefix: String,
    #[serde(default = "default_connect_timeout_seconds")]
    pub connect_timeout_seconds: u64,
}

fn default_pool_size() -> u32 {
    10
}

fn default_key_prefix() -> String {
    "quota:".to_string()
}

/// KafkaConfig は Kafka ブローカー接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    pub topic_exceeded: String,
    pub topic_threshold: String,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

/// AuthConfig は JWT 認証設定を表す。
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

/// QuotaConfig はクォータ管理固有の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct QuotaConfig {
    #[serde(default)]
    pub reset_schedule: ResetScheduleConfig,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            reset_schedule: ResetScheduleConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResetScheduleConfig {
    #[serde(default = "default_daily_cron")]
    pub daily: String,
    #[serde(default = "default_monthly_cron")]
    pub monthly: String,
}

impl Default for ResetScheduleConfig {
    fn default() -> Self {
        Self {
            daily: default_daily_cron(),
            monthly: default_monthly_cron(),
        }
    }
}

fn default_daily_cron() -> String {
    "0 0 * * *".to_string()
}

fn default_monthly_cron() -> String {
    "0 0 1 * *".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_schedule_defaults() {
        let cfg = ResetScheduleConfig::default();
        assert_eq!(cfg.daily, "0 0 * * *");
        assert_eq!(cfg.monthly, "0 0 1 * *");
    }

    #[test]
    fn test_redis_config_deserialization() {
        let yaml = r#"
url: "redis://localhost:6379"
pool_size: 10
key_prefix: "quota:"
connect_timeout_seconds: 3
"#;
        let cfg: RedisConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.pool_size, 10);
        assert_eq!(cfg.key_prefix, "quota:");
    }
}
