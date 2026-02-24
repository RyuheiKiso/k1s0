use serde::Deserialize;

/// Application configuration for featureflag server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub cache: CacheConfig,
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
    8087
}

fn default_grpc_port() -> u16 {
    9090
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
    pub topic: String,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

/// CacheConfig はインメモリキャッシュの設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_max_entries")]
    pub max_entries: u64,
    #[serde(default = "default_ttl_seconds")]
    pub ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: default_max_entries(),
            ttl_seconds: default_ttl_seconds(),
        }
    }
}

fn default_max_entries() -> u64 {
    10000
}

fn default_ttl_seconds() -> u64 {
    60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_connection_url() {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "k1s0_system".to_string(),
            user: "app".to_string(),
            password: "secret".to_string(),
            ssl_mode: "disable".to_string(),
            max_open_conns: 25,
            max_idle_conns: 5,
            conn_max_lifetime: "5m".to_string(),
        };
        assert_eq!(
            config.connection_url(),
            "postgres://app:secret@localhost:5432/k1s0_system?sslmode=disable"
        );
    }

    #[test]
    fn test_cache_config_defaults() {
        let cache = CacheConfig::default();
        assert_eq!(cache.max_entries, 10000);
        assert_eq!(cache.ttl_seconds, 60);
    }
}
