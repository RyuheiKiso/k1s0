use serde::Deserialize;

/// Application configuration for search server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub opensearch: Option<OpenSearchConfig>,
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
    8094
}

fn default_grpc_port() -> u16 {
    9090
}

/// OpenSearchConfig は OpenSearch 接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct OpenSearchConfig {
    pub url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_index_prefix")]
    pub index_prefix: String,
}

fn default_index_prefix() -> String {
    "k1s0-".to_string()
}

/// KafkaConfig は Kafka ブローカー接続の設定を表す。
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    #[serde(default = "default_consumer_group")]
    pub consumer_group: String,
    /// Consumer topic: インデックス登録要求
    pub topic: String,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

fn default_consumer_group() -> String {
    "search-server-consumer".to_string()
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
    1000
}

fn default_ttl_seconds() -> u64 {
    30
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_defaults() {
        let cache = CacheConfig::default();
        assert_eq!(cache.max_entries, 1000);
        assert_eq!(cache.ttl_seconds, 30);
    }

    #[test]
    fn test_opensearch_config_deserialization() {
        let yaml = r#"
url: "https://opensearch:9200"
username: "app"
password: ""
index_prefix: "k1s0-"
"#;
        let cfg: OpenSearchConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.index_prefix, "k1s0-");
        assert_eq!(cfg.url, "https://opensearch:9200");
    }
}
