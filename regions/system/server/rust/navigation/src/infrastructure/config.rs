use serde::Deserialize;

/// Application configuration for navigation server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default = "default_navigation_path")]
    pub navigation_path: String,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
}

fn default_navigation_path() -> String {
    "config/navigation.yaml".to_string()
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
    8095
}

fn default_grpc_port() -> u16 {
    50051
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load() {
        let yaml = r#"
app:
  name: k1s0-navigation-server
  version: "0.1.0"
  environment: dev
server:
  host: "0.0.0.0"
  port: 8095
  grpc_port: 50051
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.app.name, "k1s0-navigation-server");
        assert_eq!(cfg.server.port, 8095);
        assert_eq!(cfg.server.grpc_port, 50051);
        assert_eq!(cfg.navigation_path, "config/navigation.yaml");
    }

    #[test]
    fn test_config_defaults() {
        let yaml = r#"
app:
  name: test
server: {}
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.server.host, "0.0.0.0");
        assert_eq!(cfg.server.port, 8095);
        assert_eq!(cfg.server.grpc_port, 50051);
        assert_eq!(cfg.app.version, "0.1.0");
        assert_eq!(cfg.app.environment, "dev");
        assert!(cfg.auth.is_none());
    }
}
