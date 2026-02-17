use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub grpc: Option<GrpcConfig>,
    pub database: Option<DatabaseConfig>,
    pub kafka: Option<KafkaConfig>,
    pub redis: Option<RedisConfig>,
    pub redis_session: Option<RedisConfig>,
    pub observability: ObservabilityConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub tier: String,
    pub environment: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub read_timeout: Option<String>,
    pub write_timeout: Option<String>,
    pub shutdown_timeout: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrpcConfig {
    pub port: u16,
    pub max_recv_msg_size: Option<usize>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub ssl_mode: Option<String>,
    pub max_open_conns: Option<u32>,
    pub max_idle_conns: Option<u32>,
    pub conn_max_lifetime: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub consumer_group: String,
    pub security_protocol: String,
    pub sasl: Option<KafkaSaslConfig>,
    pub tls: Option<KafkaTlsConfig>,
    pub topics: KafkaTopics,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaSaslConfig {
    pub mechanism: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaTlsConfig {
    pub ca_cert_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaTopics {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub db: Option<u8>,
    pub pool_size: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ObservabilityConfig {
    pub log: LogConfig,
    pub trace: TraceConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TraceConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub sample_rate: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub oidc: Option<OidcConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
    pub public_key_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OidcConfig {
    pub discovery_url: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub jwks_uri: String,
    pub jwks_cache_ttl: Option<String>,
}
