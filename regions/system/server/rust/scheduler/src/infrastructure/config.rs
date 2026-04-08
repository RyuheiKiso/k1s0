// secrecy クレートを使用してデータベースパスワードを Secret<String> で保持し、Debug 出力への漏洩を防ぐ。
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

/// `AuthConfig` 縺ｯ JWT 隱崎ｨｼ險ｭ螳壹ｒ陦ｨ縺吶・
/// `AuthConfig` は認証設定を保持する（nested 形式: jwt + jwks）。
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// JWT トークンの検証に使用する issuer / audience 設定
    pub jwt: JwtConfig,
    /// JWKS エンドポイントの設定（オプション）
    #[serde(default)]
    pub jwks: Option<JwksConfig>,
}

/// `JwtConfig` は JWT トークン検証の issuer / audience を保持する。
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT 発行者（issuer）
    pub issuer: String,
    /// JWT 対象者（audience）
    pub audience: String,
}

/// `JwksConfig` は JWKS エンドポイントの URL とキャッシュ TTL を保持する。
#[derive(Debug, Clone, Deserialize)]
pub struct JwksConfig {
    /// JWKS エンドポイント URL
    pub url: String,
    /// JWKS キャッシュ TTL（秒）。デフォルト 300 秒。
    #[serde(default = "default_jwks_cache_ttl_secs")]
    pub cache_ttl_secs: u64,
}

/// JWKS キャッシュ TTL のデフォルト値（300 秒）
fn default_jwks_cache_ttl_secs() -> u64 {
    300
}

/// Application configuration for scheduler server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
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

// Cargo.toml の package.version からバージョンを取得する（M-16 監査対応: ハードコード解消）
fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_environment() -> String {
    "dev".to_string()
}

/// `ServerConfig` `はサーバーのバインドアドレス設定。host/port/grpc_port` は startup.rs で使用する。
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

/// `DatabaseConfig` 縺ｯ繝・・繧ｿ繝吶・繧ｹ謗･邯壹・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    // パスワードは Secret<String> で保持し、Debug トレイトでは [REDACTED] と表示される
    // パスワードは必須項目のため serde(default) を設定しない（Secret<String> は Default 未実装）
    pub password: Secret<String>,
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
    /// `PostgreSQL` 謗･邯・URL 繧堤函謌舌☆繧九・
    #[must_use] 
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            // expose_secret() でパスワードを取り出す
            self.user,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name,
            self.ssl_mode
        )
    }
}

/// `KafkaConfig` 縺ｯ Kafka 繝悶Ο繝ｼ繧ｫ繝ｼ謗･邯壹・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    #[serde(default)]
    pub topics: KafkaTopics,
}

/// セキュリティデフォルト: 本番環境では `SASL_SSL` を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaTopics {
    #[serde(default = "default_topic_executed")]
    pub executed: String,
    #[serde(default = "default_topic_created")]
    pub created: String,
}

impl Default for KafkaTopics {
    fn default() -> Self {
        Self {
            executed: default_topic_executed(),
            created: default_topic_created(),
        }
    }
}

fn default_topic_executed() -> String {
    "k1s0.system.scheduler.executed.v1".to_string()
}

fn default_topic_created() -> String {
    "k1s0.system.scheduler.created.v1".to_string()
}

/// `SchedulerConfig` 縺ｯ繧ｹ繧ｱ繧ｸ繝･繝ｼ繝ｩ繝ｼ蝗ｺ譛峨・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub trace: TraceConfig,
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
#[cfg(test)]
mod tests {
    use super::*;

    /// config.docker.yaml が Config にデシリアライズできることを検証する
    #[test]
    fn config_docker_yaml_deserializes_correctly() {
        let yaml = include_str!("../../config/config.docker.yaml");
        let _config: Config =
            serde_yaml::from_str(yaml).expect("config.docker.yaml のデシリアライズに失敗しました");
    }

    #[test]
    fn test_database_connection_url() {
        let cfg = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "k1s0_system".to_string(),
            user: "app".to_string(),
            password: Secret::new("pass".to_string()),
            ssl_mode: "disable".to_string(),
            max_open_conns: 25,
        };
        assert_eq!(
            cfg.connection_url(),
            "postgres://app:pass@localhost:5432/k1s0_system?sslmode=disable"
        );
    }
}
