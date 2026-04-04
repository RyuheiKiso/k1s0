// サーバー設定モジュール。YAML ファイルから設定を読み込む。
// CRIT-004 監査対応: secrecy クレートを使用してデータベースパスワードを Secret<String> で保持し、Debug 出力への漏洩を防ぐ。
// BSL-MED-003 監査対応: テレメトリ設定を config から読み込めるよう ObservabilityConfig を拡充する。
pub mod auth_config;

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
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    #[serde(default = "default_environment")]
    pub environment: String,
}

fn default_environment() -> String { "development".to_string() }

// BSL-MED-003 監査対応: ログ・トレース設定を外部化するため ObservabilityConfig を拡充する
// activity/board と同じ構造に統一することで設定管理の一貫性を保つ
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub trace: TraceConfig,
}

// ログ設定（レベルとフォーマットを外部から制御する）
#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self { level: default_log_level(), format: default_log_format() }
    }
}

// トレース設定（OTLP エンドポイントへの送信を制御する）
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
        Self {
            enabled: default_trace_enabled(),
            endpoint: default_trace_endpoint(),
            sample_rate: default_sample_rate(),
        }
    }
}

fn default_log_level() -> String { "info".to_string() }
fn default_log_format() -> String { "json".to_string() }
fn default_trace_enabled() -> bool { true }
fn default_trace_endpoint() -> String { k1s0_server_common::DEFAULT_OTEL_ENDPOINT.to_string() }
fn default_sample_rate() -> f64 { 1.0 }

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
}

fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8210 }
// H-005 監査対応: 非標準ポート (9210) から標準 gRPC ポート (50051) に統一し NetworkPolicy の許可ルールと整合させる
fn default_grpc_port() -> u16 { 50051 }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            grpc_port: default_grpc_port(),
        }
    }
}

// CRIT-004 監査対応: DatabaseConfig の Debug derive を削除し、手動実装でパスワードをマスクする。
// Secret<String> は Debug で [REDACTED] と表示されるため、derive(Debug) でも安全だが、
// master-maintenance との一貫性を保つため Clone + Deserialize のみを derive する。
#[derive(Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    #[serde(default = "default_db_port")]
    pub port: u16,
    pub name: String,
    #[serde(default = "default_schema")]
    pub schema: String,
    pub user: String,
    // パスワードは Secret<String> で保持し、誤って Debug 出力に漏れないようにする（CRIT-004 監査対応）
    // Secret<String> は Default 未実装のため serde(default) を設定しない（必須項目）
    pub password: Secret<String>,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_max_idle_conns")]
    pub max_idle_conns: u32,
    #[serde(default = "default_conn_max_lifetime")]
    pub conn_max_lifetime: u64,
}

// パスワードフィールドをマスクして Debug 出力に漏洩しないようにする（CRIT-004 監査対応）
impl std::fmt::Debug for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("name", &self.name)
            .field("schema", &self.schema)
            .field("user", &self.user)
            .field("password", &"***")
            .field("ssl_mode", &self.ssl_mode)
            .field("max_connections", &self.max_connections)
            .field("max_idle_conns", &self.max_idle_conns)
            .field("conn_max_lifetime", &self.conn_max_lifetime)
            .finish()
    }
}

fn default_db_port() -> u16 { 5432 }
fn default_schema() -> String { "project_master".to_string() }
fn default_ssl_mode() -> String { "prefer".to_string() }
fn default_max_connections() -> u32 { 25 }
fn default_max_idle_conns() -> u32 { 5 }
fn default_conn_max_lifetime() -> u64 { 300 }

impl DatabaseConfig {
    // データベース接続 URL を Secret<String> として返す（パスワード漏洩防止: CRIT-004 監査対応）
    pub fn connection_url(&self) -> Secret<String> {
        Secret::new(format!(
            "postgresql://{}:{}@{}:{}/{}?sslmode={}&options=-c search_path%3D{}",
            self.user,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name,
            self.ssl_mode,
            self.schema
        ))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub project_type_changed_topic: String,
    pub status_definition_changed_topic: String,
    pub tenant_extension_changed_topic: String,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
}

fn default_security_protocol() -> String { "PLAINTEXT".to_string() }

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
fn default_jwks_cache_ttl() -> u64 { 300 }

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// nested 形式の AuthConfig が正しくデシリアライズできることを検証する（回帰テスト・H-005 監査対応）
    #[test]
    fn nested_auth_config_deserializes_correctly() {
        let yaml = r#"
app:
  name: "project-master"
  environment: "dev"
server:
  host: "0.0.0.0"
  port: 8210
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
