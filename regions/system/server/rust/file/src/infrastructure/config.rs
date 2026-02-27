use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub storage: Option<StorageConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
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
    8098
}

fn default_grpc_port() -> u16 {
    50058
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
    3600
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_backend")]
    pub backend: String,
    pub bucket: Option<String>,
    pub region: Option<String>,
    pub endpoint: Option<String>,
}

fn default_backend() -> String {
    "memory".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    #[serde(default = "default_topic")]
    pub topic: String,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

fn default_topic() -> String {
    "k1s0.system.file.events.v1".to_string()
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
    fn test_config_deserialization() {
        let yaml = r#"
app:
  name: "file-server"
  version: "0.1.0"
  environment: "dev"
server:
  host: "0.0.0.0"
  port: 8098
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.name, "file-server");
        assert_eq!(config.server.port, 8098);
        assert!(config.storage.is_none());
        assert!(config.kafka.is_none());
    }

    #[test]
    fn test_config_defaults() {
        let yaml = r#"
app:
  name: "file-server"
server: {}
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.version, "0.1.0");
        assert_eq!(config.app.environment, "dev");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8098);
        assert_eq!(config.server.grpc_port, 50058);
    }

    #[test]
    fn test_config_with_storage() {
        let yaml = r#"
app:
  name: "file-server"
server:
  port: 8098
storage:
  backend: "s3"
  bucket: "k1s0-files"
  region: "ap-northeast-1"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.storage.is_some());
        let storage = config.storage.unwrap();
        assert_eq!(storage.backend, "s3");
        assert_eq!(storage.bucket.unwrap(), "k1s0-files");
    }

    #[test]
    fn test_config_with_kafka() {
        let yaml = r#"
app:
  name: "file-server"
server:
  port: 8098
kafka:
  brokers:
    - "localhost:9092"
  topic: "k1s0.system.file.events.v1"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(config.kafka.is_some());
        let kafka = config.kafka.unwrap();
        assert_eq!(kafka.brokers.len(), 1);
        assert_eq!(kafka.topic, "k1s0.system.file.events.v1");
    }
}
