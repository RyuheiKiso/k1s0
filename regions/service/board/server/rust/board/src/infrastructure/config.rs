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
    #[serde(default = "default_jwks_ttl")]
    pub cache_ttl_secs: u64,
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
// H-005 監査対応: 非標準ポート (9320) から標準 gRPC ポート (50051) に統一し NetworkPolicy の許可ルールと整合させる
fn default_grpc_port() -> u16 { 50051 }
fn default_db_port() -> u16 { 5432 }
// DOCS-002 監査対応: ドキュメント（server.md）と config/default.yaml の "board_service" に統一する
fn default_schema() -> String { "board_service".to_string() }
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// nested 形式の AuthConfig が正しくデシリアライズできることを検証する（回帰テスト・H-005 監査対応）
    #[test]
    fn nested_auth_config_deserializes_correctly() {
        let yaml = r#"
app:
  name: "board"
  environment: "dev"
server:
  host: "0.0.0.0"
  port: 8320
  grpc_port: 50051
auth:
  jwt:
    issuer: "http://keycloak:8080/realms/k1s0"
    audience: "k1s0-api"
  jwks:
    url: "http://keycloak:8080/realms/k1s0/protocol/openid-connect/certs"
    cache_ttl_secs: 300
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let auth = config.auth.unwrap();
        assert_eq!(auth.jwt.issuer, "http://keycloak:8080/realms/k1s0");
        assert_eq!(auth.jwt.audience, "k1s0-api");
        let jwks = auth.jwks.unwrap();
        assert_eq!(jwks.url, "http://keycloak:8080/realms/k1s0/protocol/openid-connect/certs");
        assert_eq!(jwks.cache_ttl_secs, 300);
    }
}
