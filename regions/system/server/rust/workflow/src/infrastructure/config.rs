use serde::Deserialize;

/// AuthConfig 縺ｯ隱崎ｨｼ險ｭ螳壹ｒ陦ｨ縺吶・
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

/// Application configuration for workflow server.
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
    pub scheduler: Option<SchedulerClientConfig>,
    #[serde(default)]
    pub overdue_check: OverdueCheckConfig,
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
    8080
}

fn default_grpc_port() -> u16 {
    50051
}

/// DatabaseConfig 縺ｯ繝・・繧ｿ繝吶・繧ｹ謗･邯壹・險ｭ螳壹ｒ陦ｨ縺吶・
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
    #[serde(default = "default_max_idle_conns")]
    pub max_idle_conns: u32,
    #[serde(default = "default_conn_max_lifetime")]
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
    /// PostgreSQL 謗･邯・URL 繧堤函謌舌☆繧九・
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, self.ssl_mode
        )
    }
}

/// KafkaConfig 縺ｯ Kafka 繝悶Ο繝ｼ繧ｫ繝ｼ謗･邯壹・險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    /// Producer topic: 繝ｯ繝ｼ繧ｯ繝輔Ο繝ｼ繝ｻ繧ｿ繧ｹ繧ｯ迥ｶ諷句､牙喧
    pub state_topic: String,
    /// Producer topic: 騾夂衍萓晞ｼ
    pub notification_topic: String,
}

fn default_security_protocol() -> String {
    "PLAINTEXT".to_string()
}

/// SchedulerClientConfig 縺ｯ scheduler-server 騾｣謳ｺ縺ｮ險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct SchedulerClientConfig {
    pub internal_endpoint: String,
}

/// OverdueCheckConfig 縺ｯ繧ｿ繧ｹ繧ｯ譛滓律雜・℃繝√ぉ繝・け縺ｮ險ｭ螳壹ｒ陦ｨ縺吶・
#[derive(Debug, Clone, Deserialize)]
pub struct OverdueCheckConfig {
    #[serde(default = "default_cron_expression")]
    pub cron_expression: String,
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

impl Default for OverdueCheckConfig {
    fn default() -> Self {
        Self {
            cron_expression: default_cron_expression(),
            timezone: default_timezone(),
        }
    }
}

fn default_cron_expression() -> String {
    "*/15 * * * *".to_string()
}

fn default_timezone() -> String {
    "UTC".to_string()
}


#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default = "default_otlp_endpoint")]
    pub otlp_endpoint: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_log_format")]
    pub log_format: String,
    #[serde(default = "default_metrics_enabled")]
    pub metrics_enabled: bool,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            otlp_endpoint: default_otlp_endpoint(),
            log_level: default_log_level(),
            log_format: default_log_format(),
            metrics_enabled: default_metrics_enabled(),
        }
    }
}

fn default_otlp_endpoint() -> String {
    "http://otel-collector.observability:4317".to_string()
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overdue_check_defaults() {
        let cfg = OverdueCheckConfig::default();
        assert_eq!(cfg.cron_expression, "*/15 * * * *");
        assert_eq!(cfg.timezone, "UTC");
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


