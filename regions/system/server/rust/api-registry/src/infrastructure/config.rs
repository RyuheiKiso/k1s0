// secrecy クレートを使用してデータベースパスワードを Secret<String> で保持し、Debug 出力への漏洩を防ぐ。
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub validator: Option<ValidatorConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_environment")]
    pub environment: String,
}

// Cargo.toml の package.version からバージョンを取得する（M-16 監査対応: ハードコード解消）
fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
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
    8101
}
fn default_grpc_port() -> u16 {
    50051
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    // パスワードは Secret<String> で保持し、Debug トレイトでは [REDACTED] と表示される
    pub password: Secret<String>,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default = "default_max_open_conns")]
    pub max_open_conns: u32,
    #[serde(default = "default_max_idle_conns")]
    pub max_idle_conns: u32,
    #[serde(default = "default_conn_max_lifetime_secs")]
    pub conn_max_lifetime: u64,
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
fn default_conn_max_lifetime_secs() -> u64 {
    300
}

impl DatabaseConfig {
    // expose_secret() でパスワードを取り出す。戻り値の URL はログに出力しないこと。
    pub fn connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}",
            self.user,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name,
            self.ssl_mode
        )
    }
}

/// 認証設定（JWT検証とJWKS取得を管理する）
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// JWT トークン検証設定
    pub jwt: JwtConfig,
    /// JWKS エンドポイント設定（省略時は JWKS 検証をスキップする）
    #[serde(default)]
    pub jwks: Option<JwksConfig>,
}

/// JWT トークン検証設定
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT 発行者 URL（Keycloak realm URL）
    pub issuer: String,
    /// JWT 受信者識別子
    pub audience: String,
}

/// JWKS エンドポイント設定
#[derive(Debug, Clone, Deserialize)]
pub struct JwksConfig {
    /// JWKS エンドポイント URL
    pub url: String,
    /// JWKS キャッシュ有効期限（秒）
    #[serde(default = "default_jwks_cache_ttl")]
    pub cache_ttl_secs: u64,
}

/// JWKS キャッシュのデフォルト有効期限（300秒）
fn default_jwks_cache_ttl() -> u64 {
    300
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(alias = "schema_updated_topic")]
    pub topic: String,
    #[serde(default = "default_kafka_security_protocol")]
    pub security_protocol: String,
}

/// セキュリティデフォルト: 本番環境では SASL_SSL を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_kafka_security_protocol() -> String {
    "SASL_SSL".to_string()
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            brokers: vec!["localhost:9092".to_string()],
            topic: "k1s0.system.apiregistry.schema_updated.v1".to_string(),
            security_protocol: default_kafka_security_protocol(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorConfig {
    #[serde(alias = "openapi_validator_path")]
    pub openapi_spec_validator_path: String,
    pub buf_path: String,
    #[serde(alias = "timeout_secs")]
    pub timeout_seconds: u64,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            openapi_spec_validator_path: "openapi-spec-validator".to_string(),
            buf_path: "buf".to_string(),
            timeout_seconds: 10,
        }
    }
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

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub trace: TraceConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
}
#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}
impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct TraceConfig {
    #[serde(default = "default_trace_enabled")]
    pub enabled: bool,
    #[serde(default = "default_trace_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_trace_sample_rate")]
    pub sample_rate: f64,
}
impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            enabled: default_trace_enabled(),
            endpoint: default_trace_endpoint(),
            sample_rate: default_trace_sample_rate(),
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    #[serde(default = "default_metrics_enabled")]
    pub enabled: bool,
    #[serde(default = "default_metrics_path")]
    pub path: String,
}
impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: default_metrics_enabled(),
            path: default_metrics_path(),
        }
    }
}
fn default_trace_enabled() -> bool {
    true
}
fn default_trace_endpoint() -> String {
    "http://otel-collector.observability:4317".to_string()
}
fn default_trace_sample_rate() -> f64 {
    1.0
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "json".to_string()
}
fn default_metrics_enabled() -> bool {
    true
}
fn default_metrics_path() -> String {
    "/metrics".to_string()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_missing_file() {
        let result = Config::load("nonexistent.yaml");
        assert!(result.is_err());
    }

    // Secret::new() で平文パスワードをラップしてテスト用の DatabaseConfig を構築する
    #[test]
    fn test_database_config_connection_url() {
        let db = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "testdb".to_string(),
            user: "user".to_string(),
            password: Secret::new("pass".to_string()),
            ssl_mode: "disable".to_string(),
            max_open_conns: 10,
            max_idle_conns: 5,
            conn_max_lifetime: 300,
        };
        assert_eq!(
            db.connection_url(),
            "postgresql://user:pass@localhost:5432/testdb?sslmode=disable"
        );
    }

    /// config.docker.yaml が正しくデシリアライズできることを検証する（回帰テスト・H-005 監査対応）
    #[test]
    fn config_docker_yaml_deserializes_correctly() {
        let yaml = include_str!("../../config/config.docker.yaml");
        let _config: Config =
            serde_yaml::from_str(yaml).expect("config.docker.yaml のデシリアライズに失敗しました");
    }
}
