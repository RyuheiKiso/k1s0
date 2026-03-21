use serde::Deserialize;

// アプリケーション設定のルート構造体。startup.rs から Config::load() で使用される。
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
    /// DLQ Manager gRPC クライアントの接続設定。startup.rs で GrpcDlqClient 生成に使用する。
    #[serde(default)]
    pub dlq_manager: Option<DlqManagerConfig>,
    #[serde(default)]
    pub cache: CacheConfig,
    /// タイムアウト検出・クリーンアップ機能で使用予定。YAML から読み込むため Deserialize に必要。
    #[serde(default)]
    #[allow(dead_code)]
    pub scheduler: Option<SchedulerConfig>,
    /// アラート通知機能で使用予定。YAML から読み込むため Deserialize に必要。
    #[serde(default)]
    #[allow(dead_code)]
    pub notification: Option<NotificationConfig>,
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

fn default_version() -> String {
    "0.1.0".to_string()
}

fn default_environment() -> String {
    "dev".to_string()
}

// サーバー設定。startup.rs でリスニングアドレス・ポートの取得に使用される。
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// startup.rs では現在 0.0.0.0 をハードコードしているが、将来的にはこの値を参照する。
    #[serde(default = "default_host")]
    #[allow(dead_code)]
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
    8112
}

fn default_grpc_port() -> u16 {
    50051
}

// データベース設定。database.rs の connect() で接続 URL 構築に使用される。
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    #[serde(default = "default_max_open_conns")]
    pub max_open_conns: u32,
    /// コネクションプール設定。sqlx の idle 接続数上限として使用予定。
    #[serde(default = "default_max_idle_conns")]
    #[allow(dead_code)]
    pub max_idle_conns: u32,
    /// コネクション最大生存期間。sqlx の max_lifetime として使用予定。
    #[serde(default = "default_conn_max_lifetime")]
    #[allow(dead_code)]
    pub conn_max_lifetime: String,
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

fn default_conn_max_lifetime() -> String {
    "5m".to_string()
}

impl DatabaseConfig {
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, self.ssl_mode
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwks_url: String,
    pub issuer: String,
    pub audience: String,
    #[serde(default = "default_jwks_cache_ttl_secs")]
    pub jwks_cache_ttl_secs: u64,
}

fn default_jwks_cache_ttl_secs() -> u64 {
    3600
}

// Kafka設定。kafka_consumer.rs でコンシューマー構成に使用される。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_consumer_group")]
    pub consumer_group: String,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    #[serde(default = "default_event_topic_pattern")]
    pub event_topic_pattern: String,
    #[serde(default)]
    pub exclude_pattern: Option<String>,
}

fn default_consumer_group() -> String {
    "event-monitor.default".to_string()
}

/// セキュリティデフォルト: 本番環境では SASL_SSL を強制する。
/// 開発環境では config.dev.yaml / config.docker.yaml で明示的に PLAINTEXT を指定すること。
fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
}

fn default_event_topic_pattern() -> String {
    "k1s0.*.*.*.v1".to_string()
}

// DLQ Manager 設定。DLQ クライアント接続に使用される。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DlqManagerConfig {
    pub grpc_endpoint: String,
    #[serde(default = "default_dlq_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_dlq_timeout_ms() -> u64 {
    5000
}

/// キャッシュ設定。KPI キャッシュとフロー定義キャッシュの両方を管理する。
#[derive(Debug, Clone, Deserialize)]
pub struct CacheConfig {
    /// KPI キャッシュの最大エントリ数
    #[serde(default = "default_max_entries")]
    pub kpi_max_entries: u64,
    /// KPI キャッシュの TTL（秒）
    #[serde(default = "default_ttl_seconds")]
    pub kpi_ttl_seconds: u64,
    /// フロー定義キャッシュの最大エントリ数
    #[serde(default = "default_flow_def_max_entries")]
    pub flow_def_max_entries: u64,
    /// フロー定義キャッシュの TTL（秒）。短すぎると DB 負荷が増え、長すぎると変更反映が遅れる。
    #[serde(default = "default_flow_def_ttl_seconds")]
    pub flow_def_ttl_seconds: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            kpi_max_entries: default_max_entries(),
            kpi_ttl_seconds: default_ttl_seconds(),
            flow_def_max_entries: default_flow_def_max_entries(),
            flow_def_ttl_seconds: default_flow_def_ttl_seconds(),
        }
    }
}

/// フロー定義キャッシュのデフォルト最大エントリ数
fn default_flow_def_max_entries() -> u64 {
    100
}

/// フロー定義キャッシュのデフォルト TTL（秒）
fn default_flow_def_ttl_seconds() -> u64 {
    60
}

fn default_max_entries() -> u64 {
    10000
}

fn default_ttl_seconds() -> u64 {
    30
}

// スケジューラ設定。タイムアウト検出やクリーンアップ間隔の制御に使用される。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SchedulerConfig {
    #[serde(default = "default_timeout_check_interval")]
    pub timeout_check_interval_seconds: u64,
    #[serde(default = "default_cleanup_interval")]
    pub cleanup_interval_seconds: u64,
}

fn default_timeout_check_interval() -> u64 {
    60
}

fn default_cleanup_interval() -> u64 {
    3600
}

// 通知設定。通知エンドポイントの接続構成に使用される。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct NotificationConfig {
    pub endpoint: String,
    #[serde(default = "default_notification_timeout")]
    pub timeout_seconds: u64,
}

fn default_notification_timeout() -> u64 {
    10
}

// オブザーバビリティ設定。startup.rs でテレメトリ初期化に使用される。
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

// メトリクス設定。ObservabilityConfig の一部として使用される。
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // CacheConfig のデフォルト値が KPI・フロー定義キャッシュの両方で正しいことを確認する。
    #[test]
    fn test_cache_config_defaults() {
        let cache = CacheConfig::default();
        assert_eq!(cache.kpi_max_entries, 10000);
        assert_eq!(cache.kpi_ttl_seconds, 30);
        assert_eq!(cache.flow_def_max_entries, 100);
        assert_eq!(cache.flow_def_ttl_seconds, 60);
    }

    #[test]
    fn test_scheduler_config_defaults() {
        let yaml = "timeout_check_interval_seconds: 120\ncleanup_interval_seconds: 7200";
        let cfg: SchedulerConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.timeout_check_interval_seconds, 120);
        assert_eq!(cfg.cleanup_interval_seconds, 7200);
    }

    #[test]
    fn test_notification_config() {
        let yaml = "endpoint: http://notification:8080\ntimeout_seconds: 5";
        let cfg: NotificationConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.endpoint, "http://notification:8080");
        assert_eq!(cfg.timeout_seconds, 5);
    }

    #[test]
    fn test_database_connection_url() {
        let cfg = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "k1s0_system".to_string(),
            user: "app".to_string(),
            password: "pass".to_string(),
            ssl_mode: "disable".to_string(),
            max_open_conns: 25,
            max_idle_conns: 5,
            conn_max_lifetime: "5m".to_string(),
        };
        assert_eq!(
            cfg.connection_url(),
            "postgres://app:pass@localhost:5432/k1s0_system?sslmode=disable"
        );
    }
}
