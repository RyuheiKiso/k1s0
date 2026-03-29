use serde::Deserialize;
use std::fs;

/// graphql-gateway のルート設定。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub graphql: GraphQLConfig,
    pub auth: AuthConfig,
    pub backends: BackendsConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    /// レート制限設定（ratelimit gRPC サービスと連携してリクエストレートを制御）
    #[serde(default)]
    pub ratelimit: RateLimitConfig,
    /// サーキットブレーカー設定（外部 gRPC 呼び出しの障害伝播防止）
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig,
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
#[allow(dead_code)]
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
    8080
}

#[derive(Debug, Clone, Deserialize)]
pub struct GraphQLConfig {
    /// スキーマイントロスペクションを有効化するか（development のみ true 推奨）
    #[serde(default)]
    pub introspection: bool,
    /// GraphQL Playground を有効化するか（development のみ true 推奨）
    #[serde(default)]
    pub playground: bool,
    /// クエリネスト深度の上限
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
    /// クエリ複雑度の上限
    #[serde(default = "default_max_complexity")]
    pub max_complexity: u32,
    /// クエリ実行タイムアウト（秒）
    #[serde(default = "default_query_timeout_seconds")]
    pub query_timeout_seconds: u32,
}

fn default_max_depth() -> u32 {
    10
}

fn default_max_complexity() -> u32 {
    1000
}

fn default_query_timeout_seconds() -> u32 {
    30
}

impl Default for GraphQLConfig {
    fn default() -> Self {
        Self {
            introspection: false,
            playground: false,
            max_depth: default_max_depth(),
            max_complexity: default_max_complexity(),
            query_timeout_seconds: default_query_timeout_seconds(),
        }
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
    #[serde(default = "default_jwks_cache_ttl_secs")]
    pub cache_ttl_secs: u64,
}

/// JWKS キャッシュのデフォルト有効期限（300秒）
fn default_jwks_cache_ttl_secs() -> u64 {
    300
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt: JwtConfig {
                issuer: String::new(),
                audience: String::new(),
            },
            jwks: None,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BackendsConfig {
    #[serde(default)]
    pub tenant: BackendConfig,
    #[serde(default)]
    pub featureflag: BackendConfig,
    #[serde(default)]
    pub config: BackendConfig,
    #[serde(default)]
    pub navigation: BackendConfig,
    #[serde(default)]
    pub service_catalog: BackendConfig,
    #[serde(default)]
    pub auth: BackendConfig,
    #[serde(default)]
    pub session: BackendConfig,
    #[serde(default)]
    pub vault: BackendConfig,
    #[serde(default)]
    pub scheduler: BackendConfig,
    #[serde(default)]
    pub notification: BackendConfig,
    #[serde(default)]
    pub workflow: BackendConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfig {
    /// gRPC エンドポイント（例: "http://tenant-server.k1s0-system.svc.cluster.local:50051"）
    #[serde(default = "default_backend_address")]
    pub address: String,
    /// リクエストタイムアウト（ミリ秒）
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_backend_address() -> String {
    "http://localhost:50051".to_string()
}

fn default_timeout_ms() -> u64 {
    3000
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            address: default_backend_address(),
            timeout_ms: default_timeout_ms(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// レート制限の設定。ratelimit gRPC サービスと連携してリクエストレートを制御する。
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    /// レート制限を有効化するか
    #[serde(default)]
    pub enabled: bool,
    /// ratelimit サービスの URL（例: "http://ratelimit-server:50051"）
    #[serde(default = "default_ratelimit_server_url")]
    pub server_url: String,
    /// レート制限のスコープ（キープレフィックスとして使用）
    #[serde(default = "default_ratelimit_scope")]
    pub scope: String,
    /// ウィンドウ内の最大リクエスト数
    #[serde(default = "default_requests_per_window")]
    pub requests_per_window: u32,
    /// レート制限ウィンドウの秒数
    #[serde(default = "default_window_secs")]
    pub window_secs: u64,
}

fn default_ratelimit_server_url() -> String {
    "http://localhost:50051".to_string()
}

fn default_ratelimit_scope() -> String {
    "graphql-gateway".to_string()
}

fn default_requests_per_window() -> u32 {
    100
}

fn default_window_secs() -> u64 {
    60
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server_url: default_ratelimit_server_url(),
            scope: default_ratelimit_scope(),
            requests_per_window: default_requests_per_window(),
            window_secs: default_window_secs(),
        }
    }
}

/// サーキットブレーカーの設定。外部 gRPC サービス呼び出し時の障害伝播を防止する。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CircuitBreakerConfig {
    /// オープン状態に遷移する連続失敗回数の閾値
    #[serde(default = "default_cb_failure_threshold")]
    pub failure_threshold: u32,
    /// ハーフオープンからクローズに遷移する連続成功回数の閾値
    #[serde(default = "default_cb_success_threshold")]
    pub success_threshold: u32,
    /// オープン状態の持続時間（秒）。この時間経過後にハーフオープンに遷移する。
    #[serde(default = "default_cb_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_cb_failure_threshold() -> u32 {
    5
}

fn default_cb_success_threshold() -> u32 {
    3
}

fn default_cb_timeout_secs() -> u64 {
    30
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: default_cb_failure_threshold(),
            success_threshold: default_cb_success_threshold(),
            timeout_secs: default_cb_timeout_secs(),
        }
    }
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| path.to_owned());
        let content = fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("failed to read config file {}: {}", path, e))?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.app.name.is_empty() {
            anyhow::bail!("app.name is required");
        }
        if self.server.port == 0 {
            anyhow::bail!("server.port must be > 0");
        }
        // MED-7 監査対応: issuer/audience が空文字で設定されている場合は起動を拒否する。
        // 空文字は設定ミスの可能性が高く、JWT 検証を無効化するリスクがある。
        if self.auth.jwt.issuer.is_empty() {
            anyhow::bail!("auth.jwt.issuer must not be empty string");
        }
        if self.auth.jwt.audience.is_empty() {
            anyhow::bail!("auth.jwt.audience must not be empty string");
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// config.docker.yaml が正しくデシリアライズできることを検証する（回帰テスト・H-005 監査対応）
    #[test]
    fn config_docker_yaml_deserializes_correctly() {
        let yaml = include_str!("../../config/config.docker.yaml");
        let _config: Config = serde_yaml::from_str(yaml)
            .expect("config.docker.yaml のデシリアライズに失敗しました");
    }
}
