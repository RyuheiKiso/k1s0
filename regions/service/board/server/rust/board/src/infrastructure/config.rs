// ボードサーバー設定。
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
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let cfg: Self = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_env")]
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
    // パスワードは Secret<String> で保持し、Debug トレイトでは [REDACTED] と表示される
    // パスワードは必須項目のため serde(default) を設定しない（Secret<String> は Default 未実装）
    pub password: Secret<String>,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default = "default_max_conn")]
    pub max_connections: u32,
    #[serde(default = "default_idle_conn")]
    pub max_idle_conns: u32,
    #[serde(default = "default_lifetime")]
    pub conn_max_lifetime: u64,
}

impl DatabaseConfig {
    pub fn connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}&options=-c%20search_path%3D{}",
            // expose_secret() でパスワードを取り出す。戻り値の URL はログに出力しないこと。
            self.user, self.password.expose_secret(), self.host, self.port, self.name, self.ssl_mode, self.schema
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub board_column_updated_topic: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwks_url: String,
    pub issuer: String,
    pub audience: String,
    #[serde(default = "default_jwks_ttl")]
    pub jwks_cache_ttl_secs: u64,
}

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
    #[serde(default = "default_log_fmt")]
    pub format: String,
}
impl Default for LogConfig {
    fn default() -> Self { Self { level: default_log_level(), format: default_log_fmt() } }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TraceConfig {
    #[serde(default = "default_trace_enabled")]
    pub enabled: bool,
    #[serde(default = "default_trace_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_sample_rate")]
    pub sample_rate: f64,
}
impl Default for TraceConfig {
    fn default() -> Self {
        Self { enabled: true, endpoint: default_trace_endpoint(), sample_rate: 1.0 }
    }
}

fn default_version() -> String { "0.1.0".to_string() }
fn default_env() -> String { "development".to_string() }
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8320 }
fn default_grpc_port() -> u16 { 9320 }
fn default_db_port() -> u16 { 5432 }
fn default_schema() -> String { "board".to_string() }
fn default_ssl_mode() -> String { "disable".to_string() }
fn default_max_conn() -> u32 { 25 }
fn default_idle_conn() -> u32 { 5 }
fn default_lifetime() -> u64 { 300 }
fn default_jwks_ttl() -> u64 { 300 }
fn default_log_level() -> String { "info".to_string() }
fn default_log_fmt() -> String { "json".to_string() }
fn default_trace_enabled() -> bool { true }
fn default_trace_endpoint() -> String { k1s0_server_common::DEFAULT_OTEL_ENDPOINT.to_string() }
// デフォルトのサンプリングレート（1.0 = 全トレースを収集する）
fn default_sample_rate() -> f64 { 1.0 }
