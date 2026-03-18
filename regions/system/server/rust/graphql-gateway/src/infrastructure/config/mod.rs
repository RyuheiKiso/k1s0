use serde::Deserialize;
use std::fs;

/// graphql-gateway のルート設定。
#[derive(Debug, Clone, Deserialize)]
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

fn default_version() -> String {
    "0.1.0".to_string()
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

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// JWKS エンドポイント URL
    #[serde(default = "default_jwks_url")]
    pub jwks_url: String,
}

fn default_jwks_url() -> String {
    "http://auth-server.k1s0-system.svc.cluster.local/jwks".to_string()
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwks_url: default_jwks_url(),
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

/// サーキットブレーカーの設定。外部 gRPC サービス呼び出し時の障害伝播を防止する。
#[derive(Debug, Clone, Deserialize)]
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
        Ok(())
    }
}
