use serde::Deserialize;

use super::database::DatabaseConfig;
use super::kafka_producer::KafkaConfig;

/// `AuthConfig` 縺ｯ隱崎ｨｼ險ｭ螳壹ｒ陦ｨ縺吶・
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

/// Config 縺ｯ繧｢繝励Μ繧ｱ繝ｼ繧ｷ繝ｧ繝ｳ蜈ｨ菴薙・險ｭ螳壹ｒ陦ｨ縺吶・
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
    pub kafka: Option<KafkaConfig>,
    #[serde(default)]
    pub config_server: ConfigServerConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ConfigServerConfig {
    #[serde(default)]
    pub cache: ConfigServerCacheConfig,
    #[serde(default)]
    pub audit: ConfigServerAuditConfig,
    #[serde(default)]
    pub namespace: ConfigServerNamespaceConfig,
    /// STATIC-HIGH-002: 設定値の AES-256-GCM 暗号化設定
    #[serde(default)]
    pub encryption: EncryptionConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServerCacheConfig {
    #[serde(default = "default_cache_max_entries")]
    pub max_entries: usize,
    #[serde(default = "default_cache_ttl")]
    pub ttl: String,
    #[serde(default = "default_cache_refresh_on_miss")]
    pub refresh_on_miss: bool,
}

impl Default for ConfigServerCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: default_cache_max_entries(),
            ttl: default_cache_ttl(),
            refresh_on_miss: default_cache_refresh_on_miss(),
        }
    }
}

fn default_cache_max_entries() -> usize {
    10_000
}

fn default_cache_ttl() -> String {
    "60s".to_string()
}

fn default_cache_refresh_on_miss() -> bool {
    true
}

impl ConfigServerCacheConfig {
    pub fn ttl_seconds(&self) -> anyhow::Result<u64> {
        parse_duration_seconds(&self.ttl)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServerAuditConfig {
    #[serde(default = "default_audit_retention_days")]
    pub retention_days: u32,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub kafka_enabled: bool,
    #[serde(default = "default_audit_kafka_topic")]
    pub kafka_topic: String,
}

impl Default for ConfigServerAuditConfig {
    fn default() -> Self {
        Self {
            retention_days: default_audit_retention_days(),
            enabled: true,
            kafka_enabled: false,
            kafka_topic: default_audit_kafka_topic(),
        }
    }
}

fn default_audit_retention_days() -> u32 {
    365
}

fn default_audit_kafka_topic() -> String {
    "k1s0.system.config.changed.v1".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigServerNamespaceConfig {
    #[serde(default = "default_namespace_default_prefix")]
    pub default_prefix: String,
    #[serde(default = "default_namespace_allowed_tiers")]
    pub allowed_tiers: Vec<String>,
    #[serde(default = "default_namespace_max_depth")]
    pub max_depth: usize,
}

impl Default for ConfigServerNamespaceConfig {
    fn default() -> Self {
        Self {
            default_prefix: default_namespace_default_prefix(),
            allowed_tiers: default_namespace_allowed_tiers(),
            max_depth: default_namespace_max_depth(),
        }
    }
}

fn default_namespace_default_prefix() -> String {
    "system".to_string()
}

fn default_namespace_allowed_tiers() -> Vec<String> {
    vec![
        "system".to_string(),
        "business".to_string(),
        "service".to_string(),
    ]
}

fn default_namespace_max_depth() -> usize {
    4
}

/// STATIC-HIGH-002: 設定値暗号化設定。
/// enabled が true `の場合、sensitive_namespaces` にマッチする設定値を AES-256-GCM で暗号化する。
/// 暗号化鍵は環境変数 `CONFIG_ENCRYPTION_KEY（base64` エンコード 32 バイト）または `key_base64` から取得する。
#[derive(Debug, Clone, Deserialize)]
pub struct EncryptionConfig {
    /// 暗号化を有効にするフラグ。本番環境では true を推奨
    #[serde(default)]
    pub enabled: bool,
    /// 暗号化対象の namespace プレフィックスリスト
    #[serde(default = "default_sensitive_namespaces")]
    pub sensitive_namespaces: Vec<String>,
    /// 暗号化鍵（base64 エンコード 32 バイト）。環境変数 `CONFIG_ENCRYPTION_KEY` でも指定可能
    #[serde(default)]
    pub key_base64: String,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sensitive_namespaces: default_sensitive_namespaces(),
            key_base64: String::new(),
        }
    }
}

fn default_sensitive_namespaces() -> Vec<String> {
    vec![
        "system.auth".to_string(),
        "system.database".to_string(),
    ]
}

/// `AppConfig` 縺ｯ繧｢繝励Μ繧ｱ繝ｼ繧ｷ繝ｧ繝ｳ蝓ｺ譛ｬ險ｭ螳壹ｒ陦ｨ縺吶・
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

/// `ServerConfig` 縺ｯ繧ｵ繝ｼ繝舌・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "default_read_timeout")]
    pub read_timeout: String,
    #[serde(default = "default_write_timeout")]
    pub write_timeout: String,
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout: String,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8082
}

fn default_grpc_port() -> u16 {
    50051
}

fn default_read_timeout() -> String {
    "10s".to_string()
}

fn default_write_timeout() -> String {
    "30s".to_string()
}

fn default_shutdown_timeout() -> String {
    "15s".to_string()
}

impl Config {
    /// YAML 繝輔ぃ繧､繝ｫ縺九ｉ險ｭ螳壹ｒ隱ｭ縺ｿ霎ｼ繧縲・
    pub fn from_yaml(content: &str) -> anyhow::Result<Self> {
        let config: Config = serde_yaml::from_str(content)?;
        Ok(config)
    }

    /// 險ｭ螳壹ヵ繧｡繧､繝ｫ繝代せ縺九ｉ險ｭ螳壹ｒ隱ｭ縺ｿ霎ｼ繧縲・
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }
}

fn parse_duration_seconds(input: &str) -> anyhow::Result<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(anyhow::anyhow!("duration must not be empty"));
    }

    if let Some(raw) = trimmed.strip_suffix("ms") {
        let millis = raw.trim().parse::<u64>()?;
        return Ok((millis / 1000).max(1));
    }
    if let Some(raw) = trimmed.strip_suffix('s') {
        return Ok(raw.trim().parse::<u64>()?);
    }
    if let Some(raw) = trimmed.strip_suffix('m') {
        return Ok(raw.trim().parse::<u64>()? * 60);
    }
    if let Some(raw) = trimmed.strip_suffix('h') {
        return Ok(raw.trim().parse::<u64>()? * 60 * 60);
    }

    Ok(trimmed.parse::<u64>()?)
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
#[allow(clippy::unwrap_used)]
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
    fn test_config_from_yaml() {
        let yaml = r#"
app:
  name: "k1s0-config-server"
  version: "0.1.0"
  environment: "dev"
server:
  host: "0.0.0.0"
  port: 8082
"#;
        let config = Config::from_yaml(yaml).unwrap();
        assert_eq!(config.app.name, "k1s0-config-server");
        assert_eq!(config.app.version, "0.1.0");
        assert_eq!(config.app.environment, "dev");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8082);
        assert_eq!(config.server.read_timeout, "10s");
        assert_eq!(config.server.write_timeout, "30s");
        assert_eq!(config.server.shutdown_timeout, "15s");
        assert_eq!(config.server.grpc_port, 50051);
    }

    #[test]
    fn test_config_defaults() {
        let yaml = r#"
app:
  name: "k1s0-config-server"
server: {}
"#;
        let config = Config::from_yaml(yaml).unwrap();
        assert_eq!(config.app.version, "0.1.0");
        assert_eq!(config.app.environment, "dev");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8082);
        assert_eq!(config.server.grpc_port, 50051);
        assert!(config.database.is_none());
        assert!(config.kafka.is_none());
    }

    #[test]
    fn test_cache_ttl_parse() {
        let cache = ConfigServerCacheConfig {
            max_entries: 100,
            ttl: "60s".to_string(),
            refresh_on_miss: true,
        };
        assert_eq!(cache.ttl_seconds().unwrap(), 60);

        let cache = ConfigServerCacheConfig {
            max_entries: 100,
            ttl: "5m".to_string(),
            refresh_on_miss: true,
        };
        assert_eq!(cache.ttl_seconds().unwrap(), 300);
    }

    #[test]
    fn test_config_with_database() {
        let yaml = r#"
app:
  name: "k1s0-config-server"
server:
  port: 8082
database:
  host: "localhost"
  port: 5432
  name: "k1s0_config"
  user: "app"
  password: "pass"
"#;
        let config = Config::from_yaml(yaml).unwrap();
        assert!(config.database.is_some());

        let db = config.database.unwrap();
        assert_eq!(db.host, "localhost");
        assert_eq!(db.name, "k1s0_config");
    }

    #[test]
    fn test_config_with_kafka() {
        let yaml = r#"
app:
  name: "k1s0-config-server"
server:
  port: 8082
kafka:
  brokers:
    - "localhost:9092"
  topics:
    publish:
      - "k1s0.system.config.changed.v1"
    subscribe: []
"#;
        let config = Config::from_yaml(yaml).unwrap();
        assert!(config.kafka.is_some());

        let kafka = config.kafka.unwrap();
        assert_eq!(kafka.brokers.len(), 1);
        assert_eq!(kafka.topics.publish.len(), 1);
    }

    #[test]
    fn test_config_full() {
        let yaml = r#"
app:
  name: "k1s0-config-server"
  version: "0.1.0"
  environment: "prod"
server:
  host: "0.0.0.0"
  port: 8082
database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_config"
  user: "app"
  password: ""
  ssl_mode: "require"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
    - "kafka-1.messaging.svc.cluster.local:9092"
  consumer_group: "config-server.default"
  security_protocol: "SASL_SSL"
  sasl:
    mechanism: "SCRAM-SHA-512"
    username: ""
    password: ""
  topics:
    publish:
      - "k1s0.system.config.changed.v1"
    subscribe: []
"#;
        let config = Config::from_yaml(yaml).unwrap();
        assert_eq!(config.app.environment, "prod");
        assert!(config.database.is_some());
        assert!(config.kafka.is_some());

        let kafka = config.kafka.unwrap();
        assert_eq!(kafka.brokers.len(), 2);
        assert_eq!(kafka.security_protocol, "SASL_SSL");
        assert_eq!(kafka.sasl.mechanism, "SCRAM-SHA-512");
    }
}
