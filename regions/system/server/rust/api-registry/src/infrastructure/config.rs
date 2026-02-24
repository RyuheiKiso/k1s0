use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
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
    8101
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default = "default_max_open_conns")]
    pub max_open_conns: u32,
}

fn default_ssl_mode() -> String {
    "disable".to_string()
}
fn default_max_open_conns() -> u32 {
    25
}

impl DatabaseConfig {
    pub fn connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, self.ssl_mode
        )
    }
}

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

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path, e))?;
        let config: Self = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file: {}", e))?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_missing_file() {
        let result = Config::load("nonexistent.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_database_config_connection_url() {
        let db = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "testdb".to_string(),
            user: "user".to_string(),
            password: "pass".to_string(),
            ssl_mode: "disable".to_string(),
            max_open_conns: 10,
        };
        assert_eq!(
            db.connection_url(),
            "postgresql://user:pass@localhost:5432/testdb?sslmode=disable"
        );
    }
}
