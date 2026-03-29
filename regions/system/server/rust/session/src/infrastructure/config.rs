use serde::Deserialize;

/// AuthConfig 縺ｯ隱崎ｨｼ險ｭ螳壹ｒ陦ｨ縺吶・
/// AuthConfig は認証設定を保持する（nested 形式: jwt + jwks）。
#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// JWT トークンの検証に使用する issuer / audience 設定
    pub jwt: JwtConfig,
    /// JWKS エンドポイントの設定（オプション）
    #[serde(default)]
    pub jwks: Option<JwksConfig>,
}

/// JwtConfig は JWT トークン検証の issuer / audience を保持する。
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT 発行者（issuer）
    pub issuer: String,
    /// JWT 対象者（audience）
    pub audience: String,
}

/// JwksConfig は JWKS エンドポイントの URL とキャッシュ TTL を保持する。
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

/// Application configuration for session server.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    #[serde(default)]
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub redis: Option<RedisConfig>,
    #[serde(default)]
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub session: SessionConfig,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
    8102
}

fn default_grpc_port() -> u16 {
    50051
}

/// DatabaseConfig 縺ｯ PostgreSQL 繝・・繧ｿ繝吶・繧ｹ謗･邯壹・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 {
    10
}

/// RedisConfig 縺ｯ Redis 謗･邯壹・險ｭ螳壹ｒ陦ｨ縺吶・
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_connect_timeout_seconds")]
    pub connect_timeout_seconds: u64,
}

fn default_pool_size() -> u32 {
    10
}

fn default_connect_timeout_seconds() -> u64 {
    3
}

/// KafkaConfig 縺ｯ Kafka 繝悶Ο繝ｼ繧ｫ繝ｼ謗･邯壹・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    /// Consumer topic: 蜈ｨ繧ｻ繝・す繝ｧ繝ｳ螟ｱ蜉ｹ隕∵ｱ・
    pub topic_revoke_all: String,
    /// Producer topic: 繧ｻ繝・す繝ｧ繝ｳ菴懈・
    pub topic_created: String,
    /// Producer topic: 繧ｻ繝・す繝ｧ繝ｳ螟ｱ蜉ｹ
    pub topic_revoked: String,
    #[serde(default = "default_consumer_group")]
    pub consumer_group: String,
}

/// セキュリティデフォルト: 本番環境では SASL_SSL を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
}

fn default_consumer_group() -> String {
    "session-server-consumer".to_string()
}

/// SessionConfig 縺ｯ繧ｻ繝・す繝ｧ繝ｳ邂｡逅・崋譛峨・險ｭ螳壹ｒ陦ｨ縺吶・
/// 繝輔ぅ繝ｼ繝ｫ繝牙錐縺ｯ譌｢蟄倥・ main.rs 縺ｮ蜿ら・繝代ち繝ｼ繝ｳ縺ｫ蜷医ｏ縺帙※縺・ｋ縲・
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SessionConfig {
    #[serde(default = "default_ttl")]
    pub default_ttl_seconds: i64,
    #[serde(default = "default_max_ttl")]
    pub max_ttl_seconds: i64,
    #[serde(default = "default_max_devices_per_user")]
    pub max_devices_per_user: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_ttl_seconds: default_ttl(),
            max_ttl_seconds: default_max_ttl(),
            max_devices_per_user: default_max_devices_per_user(),
        }
    }
}

fn default_ttl() -> i64 {
    3600
}

fn default_max_ttl() -> i64 {
    86400
}

fn default_max_devices_per_user() -> u32 {
    10
}

#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// config.docker.yaml が Config にデシリアライズできることを検証する
    #[test]
    fn config_docker_yaml_deserializes_correctly() {
        let yaml = include_str!("../../config/config.docker.yaml");
        let _config: Config = serde_yaml::from_str(yaml)
            .expect("config.docker.yaml のデシリアライズに失敗しました");
    }

    #[test]
    fn test_session_config_defaults() {
        let cfg = SessionConfig::default();
        assert_eq!(cfg.default_ttl_seconds, 3600);
        assert_eq!(cfg.max_ttl_seconds, 86400);
        assert_eq!(cfg.max_devices_per_user, 10);
    }

    #[test]
    fn test_redis_config_deserialization() {
        let yaml = r#"
url: "redis://localhost:6379"
pool_size: 10
connect_timeout_seconds: 3
"#;
        let cfg: RedisConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.pool_size, 10);
        assert_eq!(cfg.connect_timeout_seconds, 3);
    }
}
