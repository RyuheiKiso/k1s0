use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub graphql: GraphQLConfig,
    pub auth: AuthConfig,
    pub backends: BackendsConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
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
}

fn default_max_depth() -> u32 {
    10
}

fn default_max_complexity() -> u32 {
    1000
}

impl Default for GraphQLConfig {
    fn default() -> Self {
        Self {
            introspection: false,
            playground: false,
            max_depth: default_max_depth(),
            max_complexity: default_max_complexity(),
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

#[derive(Debug, Clone, Deserialize)]
pub struct BackendsConfig {
    #[serde(default)]
    pub tenant: BackendConfig,
    #[serde(default)]
    pub featureflag: BackendConfig,
    #[serde(default)]
    pub config: BackendConfig,
}

impl Default for BackendsConfig {
    fn default() -> Self {
        Self {
            tenant: BackendConfig::default(),
            featureflag: BackendConfig::default(),
            config: BackendConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfig {
    /// gRPC エンドポイント（例: "http://tenant-server.k1s0-system.svc.cluster.local:9090"）
    #[serde(default = "default_backend_address")]
    pub address: String,
    /// リクエストタイムアウト（ミリ秒）
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_backend_address() -> String {
    "http://localhost:9090".to_string()
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

#[derive(Debug, Clone, Deserialize, Default)]
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

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "text".to_string()
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TraceConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub sample_rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_metrics_path")]
    pub path: String,
}

fn default_metrics_path() -> String {
    "/metrics".to_string()
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            path: default_metrics_path(),
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
