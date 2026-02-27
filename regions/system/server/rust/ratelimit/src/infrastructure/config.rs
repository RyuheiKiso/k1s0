use serde::Deserialize;

use super::database::DatabaseConfig;

/// RedisConfig は Redis 接続設定。
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_redis_url")]
    pub url: String,
}

fn default_redis_url() -> String {
    "redis://127.0.0.1:6379".to_string()
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

/// Application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub redis: Option<RedisConfig>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
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
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
app:
  name: "ratelimit-server"
  version: "0.1.0"
  environment: "dev"
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: "localhost"
  port: 5432
  name: "ratelimit_db"
  user: "dev"
  password: "dev"
redis:
  url: "redis://localhost:6379"
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.app.name, "ratelimit-server");
        assert_eq!(cfg.server.port, 8080);
        assert!(cfg.database.is_some());
        assert!(cfg.redis.is_some());
        assert_eq!(cfg.redis.unwrap().url, "redis://localhost:6379");
    }

    #[test]
    fn test_config_minimal() {
        let yaml = r#"
app:
  name: "ratelimit-server"
server:
  port: 8080
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.app.name, "ratelimit-server");
        assert!(cfg.database.is_none());
        assert!(cfg.redis.is_none());
    }
}
