use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub rule_engine: Option<RuleEngineConfig>,
    #[serde(default)]
    pub import: Option<ImportConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_environment")]
    pub environment: String,
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

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    #[serde(default = "default_db_port")]
    pub port: u16,
    pub name: String,
    #[serde(default = "default_schema")]
    pub schema: String,
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
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
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub topic: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwks_url: String,
    pub issuer: String,
    pub audience: String,
    #[serde(default = "default_jwks_cache_ttl")]
    pub jwks_cache_ttl_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuleEngineConfig {
    #[serde(default = "default_max_rules")]
    pub max_rules_per_table: usize,
    #[serde(default = "default_eval_timeout")]
    pub evaluation_timeout_ms: u64,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportConfig {
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: usize,
    #[serde(default = "default_max_rows")]
    pub max_rows_per_import: usize,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_version() -> String { "0.1.0".to_string() }
fn default_environment() -> String { "development".to_string() }
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8110 }
fn default_grpc_port() -> u16 { 50051 }
fn default_db_port() -> u16 { 5432 }
fn default_schema() -> String { "master_maintenance".to_string() }
fn default_ssl_mode() -> String { "disable".to_string() }
fn default_max_connections() -> u32 { 25 }
fn default_jwks_cache_ttl() -> u64 { 300 }
fn default_max_rules() -> usize { 100 }
fn default_eval_timeout() -> u64 { 5000 }
fn default_cache_ttl() -> u64 { 300 }
fn default_max_file_size() -> usize { 50 }
fn default_max_rows() -> usize { 100000 }
fn default_batch_size() -> usize { 500 }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
app:
  name: test-server
server:
  host: "0.0.0.0"
  port: 8110
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.name, "test-server");
        assert_eq!(config.server.port, 8110);
        assert!(config.database.is_none());
    }

    #[test]
    fn test_database_connection_url() {
        let db = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "k1s0".to_string(),
            schema: "master_maintenance".to_string(),
            user: "k1s0".to_string(),
            password: "secret".to_string(),
            ssl_mode: "disable".to_string(),
            max_connections: 25,
        };
        assert_eq!(
            db.connection_url(),
            "postgresql://k1s0:secret@localhost:5432/k1s0?sslmode=disable"
        );
    }
}
