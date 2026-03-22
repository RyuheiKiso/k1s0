use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CliConfig {
    pub project_name: String,
    pub regions_root: String,
    pub docker_registry: String,
    pub go_module_base: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RuntimeConfig {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub grpc: Option<GrpcConfig>,
    pub database: Option<DatabaseConfig>,
    pub kafka: Option<KafkaConfig>,
    pub redis: Option<RedisConfig>,
    pub redis_session: Option<RedisSessionConfig>,
    pub observability: ObservabilityConfig,
    pub auth: AuthConfig,
}

impl RuntimeConfig {
    /// Validate runtime configuration values.
    ///
    /// # Errors
    ///
    /// Returns an error when a field contains an unsupported value.
    pub fn validate(&self) -> Result<(), String> {
        const VALID_TIERS: &[&str] = &["system", "business", "service"];
        const VALID_ENVIRONMENTS: &[&str] = &["dev", "staging", "prod"];
        const VALID_LOG_LEVELS: &[&str] = &["debug", "info", "warn", "error"];

        if !VALID_TIERS.contains(&self.app.tier.as_str()) {
            return Err(format!(
                "invalid app.tier: '{}' (allowed: {:?})",
                self.app.tier, VALID_TIERS
            ));
        }

        if !VALID_ENVIRONMENTS.contains(&self.app.environment.as_str()) {
            return Err(format!(
                "invalid app.environment: '{}' (allowed: {:?})",
                self.app.environment, VALID_ENVIRONMENTS
            ));
        }

        if self.server.port == 0 {
            return Err("invalid server.port: 0".to_string());
        }

        if !VALID_LOG_LEVELS.contains(&self.observability.log.level.as_str()) {
            return Err(format!(
                "invalid observability.log.level: '{}' (allowed: {:?})",
                self.observability.log.level, VALID_LOG_LEVELS
            ));
        }

        if let Some(ref kafka) = self.kafka {
            const VALID_SECURITY_PROTOCOLS: &[&str] = &["PLAINTEXT", "SASL_SSL"];
            if !VALID_SECURITY_PROTOCOLS.contains(&kafka.security_protocol.as_str()) {
                return Err(format!(
                    "invalid kafka.security_protocol: '{}' (allowed: {:?})",
                    kafka.security_protocol, VALID_SECURITY_PROTOCOLS
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub tier: String,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub read_timeout: String,
    pub write_timeout: String,
    pub shutdown_timeout: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            read_timeout: "30s".to_string(),
            write_timeout: "30s".to_string(),
            shutdown_timeout: "10s".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GrpcConfig {
    pub port: u16,
    pub max_recv_msg_size: Option<usize>,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self {
            port: 50051,
            max_recv_msg_size: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub ssl_mode: String,
    pub max_open_conns: u32,
    pub max_idle_conns: u32,
    pub conn_max_lifetime: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 5432,
            name: String::new(),
            user: String::new(),
            password: String::new(),
            ssl_mode: "disable".to_string(),
            max_open_conns: 25,
            max_idle_conns: 5,
            conn_max_lifetime: "5m".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub consumer_group: String,
    pub security_protocol: String,
    pub sasl: Option<KafkaSaslConfig>,
    pub tls: Option<KafkaTlsConfig>,
    pub topics: KafkaTopics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KafkaSaslConfig {
    pub mechanism: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KafkaTlsConfig {
    pub ca_cert_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KafkaTopics {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub db: u32,
    pub pool_size: u32,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 6379,
            password: String::new(),
            db: 0,
            pool_size: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RedisSessionConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
}

impl Default for RedisSessionConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 6380,
            password: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ObservabilityConfig {
    pub log: LogConfig,
    pub trace: TraceConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    pub level: String,
    pub format: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TraceConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub sample_rate: f64,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: String::new(),
            sample_rate: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub path: String,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/metrics".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub oidc: Option<OidcConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
    pub public_key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OidcConfig {
    pub discovery_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub jwks_uri: String,
    pub jwks_cache_ttl: Option<String>,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            regions_root: "regions".to_string(),
            docker_registry: "harbor.internal.example.com".to_string(),
            go_module_base: "github.com/org/k1s0".to_string(),
        }
    }
}

/// Load CLI configuration from YAML.
///
/// # Errors
///
/// Returns an error when the file cannot be read or parsed.
pub fn load_config(path: &str) -> anyhow::Result<CliConfig> {
    let config_path = Path::new(path);
    if !config_path.exists() {
        return Ok(CliConfig::default());
    }
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| anyhow::anyhow!("設定ファイルの読み込みに失敗しました: {e}"))?;
    let config: CliConfig = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("設定ファイルの解析に失敗しました: {e}"))?;
    Ok(config)
}

/// Load CLI configuration and optionally merge secrets from Vault.
///
/// # Errors
///
/// Returns an error when local configuration loading or Vault response parsing fails.
pub fn load_config_with_vault(path: &str) -> anyhow::Result<CliConfig> {
    let mut config = load_config(path)?;

    let vault_addr = std::env::var("K1S0_VAULT_ADDR")
        .or_else(|_| std::env::var("VAULT_ADDR"))
        .unwrap_or_default();
    let vault_path = std::env::var("K1S0_VAULT_PATH").unwrap_or_default();

    merge_vault_secrets(&mut config, &vault_addr, &vault_path)?;
    Ok(config)
}

fn vault_token() -> Option<String> {
    std::env::var("K1S0_VAULT_TOKEN")
        .ok()
        .filter(|token| !token.is_empty())
        .or_else(|| {
            std::env::var("VAULT_TOKEN")
                .ok()
                .filter(|token| !token.is_empty())
        })
}

fn apply_secret_overrides(
    base: &mut CliConfig,
    secrets: &serde_json::Map<String, serde_json::Value>,
) {
    if let Some(value) = secrets.get("project_name").and_then(|value| value.as_str()) {
        base.project_name = value.to_string();
    }
    if let Some(value) = secrets.get("regions_root").and_then(|value| value.as_str()) {
        base.regions_root = value.to_string();
    }
    if let Some(value) = secrets
        .get("docker_registry")
        .and_then(|value| value.as_str())
    {
        base.docker_registry = value.to_string();
    }
    if let Some(value) = secrets
        .get("go_module_base")
        .and_then(|value| value.as_str())
    {
        base.go_module_base = value.to_string();
    }
}

fn extract_vault_secret_data(
    body: &serde_json::Value,
) -> Option<&serde_json::Map<String, serde_json::Value>> {
    body.get("data")
        .and_then(|value| value.get("data").or(Some(value)))
        .and_then(|value| value.as_object())
}

/// Merge known CLI settings from a Vault KV response.
///
/// # Errors
///
/// Returns an error when the Vault response cannot be parsed or does not contain secret data.
pub fn merge_vault_secrets(
    base: &mut CliConfig,
    vault_addr: &str,
    vault_path: &str,
) -> anyhow::Result<()> {
    if vault_addr.is_empty() || vault_path.is_empty() {
        return Ok(());
    }

    let Some(token) = vault_token() else {
        eprintln!(
            "警告: Vault トークンが設定されていません。シークレットのマージをスキップします。addr={vault_addr} path={vault_path}"
        );
        return Ok(());
    };

    let endpoint = format!(
        "{}/v1/{}",
        vault_addr.trim_end_matches('/'),
        vault_path.trim_start_matches('/'),
    );

    let response = match ureq::get(&endpoint).header("X-Vault-Token", &token).call() {
        Ok(response) => response,
        Err(err) => {
            eprintln!(
                "警告: Vault への接続に失敗しました。シークレットのマージをスキップします。addr={vault_addr} path={vault_path} error={err}"
            );
            return Ok(());
        }
    };

    let body: serde_json::Value = response
        .into_body()
        .read_json()
        .map_err(|e| anyhow::anyhow!("Vault レスポンスの解析に失敗しました: {e}"))?;

    let secrets = extract_vault_secret_data(&body)
        .ok_or_else(|| anyhow::anyhow!("Vault レスポンスにシークレットデータが含まれていません"))?;

    apply_secret_overrides(base, secrets);
    Ok(())
}

/// Merge an override config file into the base config.
///
/// # Errors
///
/// Returns an error when the override file cannot be read or parsed.
pub fn merge_config(base: &mut CliConfig, override_path: &str) -> anyhow::Result<()> {
    let path = Path::new(override_path);
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("上書き設定ファイルの読み込みに失敗しました: {e}"))?;
    let override_config: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("上書き設定ファイルの解析に失敗しました: {e}"))?;

    if let serde_yaml::Value::Mapping(map) = override_config {
        if let Some(serde_yaml::Value::String(name)) =
            map.get(serde_yaml::Value::String("project_name".to_string()))
        {
            base.project_name.clone_from(name);
        }
        if let Some(serde_yaml::Value::String(root)) =
            map.get(serde_yaml::Value::String("regions_root".to_string()))
        {
            base.regions_root.clone_from(root);
        }
        if let Some(serde_yaml::Value::String(registry)) =
            map.get(serde_yaml::Value::String("docker_registry".to_string()))
        {
            base.docker_registry.clone_from(registry);
        }
        if let Some(serde_yaml::Value::String(go_base)) =
            map.get(serde_yaml::Value::String("go_module_base".to_string()))
        {
            base.go_module_base.clone_from(go_base);
        }
    }
    Ok(())
}

// テストコードでは unwrap() の使用を許可する
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Write;
    use std::sync::{Mutex, OnceLock};
    use tempfile::NamedTempFile;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.project_name, "");
        assert_eq!(config.regions_root, "regions");
        assert_eq!(config.docker_registry, "harbor.internal.example.com");
        assert_eq!(config.go_module_base, "github.com/org/k1s0");
    }

    #[test]
    fn test_load_config_nonexistent_returns_default() {
        let config = load_config("nonexistent.yaml").unwrap();
        assert_eq!(config.regions_root, "regions");
        assert_eq!(config.docker_registry, "harbor.internal.example.com");
    }

    #[test]
    fn test_load_config_from_yaml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "project_name: my-project\nregions_root: custom-regions\ndocker_registry: my-registry.io\ngo_module_base: github.com/myorg/myrepo"
        )
        .unwrap();
        let config = load_config(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.project_name, "my-project");
        assert_eq!(config.regions_root, "custom-regions");
        assert_eq!(config.docker_registry, "my-registry.io");
        assert_eq!(config.go_module_base, "github.com/myorg/myrepo");
    }

    #[test]
    fn test_load_config_partial_yaml_uses_defaults() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "project_name: partial-project").unwrap();
        let config = load_config(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.project_name, "partial-project");
        assert_eq!(config.regions_root, "regions");
        assert_eq!(config.docker_registry, "harbor.internal.example.com");
    }

    #[test]
    fn test_load_config_invalid_yaml_returns_error() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{{invalid yaml").unwrap();
        assert!(load_config(file.path().to_str().unwrap()).is_err());
    }

    #[test]
    fn test_merge_config_overrides_values() {
        let mut base = CliConfig {
            project_name: "base-project".to_string(),
            ..CliConfig::default()
        };

        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "project_name: overridden\ndocker_registry: custom-registry.io"
        )
        .unwrap();

        merge_config(&mut base, file.path().to_str().unwrap()).unwrap();
        assert_eq!(base.project_name, "overridden");
        assert_eq!(base.docker_registry, "custom-registry.io");
        assert_eq!(base.regions_root, "regions");
    }

    #[test]
    fn test_merge_config_nonexistent_file_noop() {
        let mut base = CliConfig {
            project_name: "original".to_string(),
            ..CliConfig::default()
        };
        merge_config(&mut base, "nonexistent.yaml").unwrap();
        assert_eq!(base.project_name, "original");
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = CliConfig {
            project_name: "test-project".to_string(),
            regions_root: "regions".to_string(),
            docker_registry: "harbor.internal.example.com".to_string(),
            go_module_base: "github.com/org/k1s0".to_string(),
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: CliConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.project_name, deserialized.project_name);
        assert_eq!(config.regions_root, deserialized.regions_root);
        assert_eq!(config.docker_registry, deserialized.docker_registry);
        assert_eq!(config.go_module_base, deserialized.go_module_base);
    }

    #[test]
    fn test_merge_vault_secrets_empty_addr_is_noop() {
        let mut base = CliConfig {
            project_name: "original".to_string(),
            ..CliConfig::default()
        };
        let result = merge_vault_secrets(&mut base, "", "secret/data/k1s0");
        assert!(result.is_ok());
        assert_eq!(base.project_name, "original");
    }

    #[test]
    fn test_merge_vault_secrets_empty_path_is_noop() {
        let mut base = CliConfig {
            project_name: "original".to_string(),
            ..CliConfig::default()
        };
        let result = merge_vault_secrets(&mut base, "https://vault.example.com", "");
        assert!(result.is_ok());
        assert_eq!(base.project_name, "original");
    }

    #[test]
    fn test_merge_vault_secrets_unreachable_warns() {
        let mut base = CliConfig::default();
        let result = merge_vault_secrets(
            &mut base,
            "https://vault.example.com",
            "secret/data/k1s0/service/task/database",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_vault_secrets_reads_kv_v2_payload() {
        let _guard = env_lock().lock().unwrap();
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/secret/data/k1s0")
            .match_header("x-vault-token", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "data": {
                            "project_name": "vault-project",
                            "regions_root": "vault-regions",
                            "docker_registry": "registry.internal",
                            "go_module_base": "github.com/example/k1s0"
                        }
                    }
                }"#,
            )
            .create();

        env::set_var("K1S0_VAULT_TOKEN", "test-token");
        let mut base = CliConfig::default();
        merge_vault_secrets(&mut base, &server.url(), "secret/data/k1s0").unwrap();
        env::remove_var("K1S0_VAULT_TOKEN");

        mock.assert();
        assert_eq!(base.project_name, "vault-project");
        assert_eq!(base.regions_root, "vault-regions");
        assert_eq!(base.docker_registry, "registry.internal");
        assert_eq!(base.go_module_base, "github.com/example/k1s0");
    }

    #[test]
    fn test_load_config_with_vault_applies_local_then_vault_override() {
        let _guard = env_lock().lock().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "project_name: local-project\nregions_root: local-regions\ndocker_registry: local-registry"
        )
        .unwrap();

        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/v1/secret/data/k1s0")
            .match_header("x-vault-token", "test-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                    "data": {
                        "data": {
                            "project_name": "vault-project",
                            "go_module_base": "github.com/example/k1s0"
                        }
                    }
                }"#,
            )
            .create();

        env::set_var("K1S0_VAULT_ADDR", server.url());
        env::set_var("K1S0_VAULT_PATH", "secret/data/k1s0");
        env::set_var("K1S0_VAULT_TOKEN", "test-token");

        let config = load_config_with_vault(file.path().to_str().unwrap()).unwrap();

        env::remove_var("K1S0_VAULT_ADDR");
        env::remove_var("K1S0_VAULT_PATH");
        env::remove_var("K1S0_VAULT_TOKEN");

        mock.assert();
        assert_eq!(config.project_name, "vault-project");
        assert_eq!(config.regions_root, "local-regions");
        assert_eq!(config.docker_registry, "local-registry");
        assert_eq!(config.go_module_base, "github.com/example/k1s0");
    }

    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert_eq!(config.app.name, "");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0");
        assert!(config.grpc.is_none());
        assert!(config.database.is_none());
        assert!(config.kafka.is_none());
        assert!(config.redis.is_none());
        assert!(config.redis_session.is_none());
        assert_eq!(config.observability.log.level, "info");
        assert_eq!(config.auth.jwt.issuer, "");
    }

    #[test]
    fn test_runtime_config_deserialize_minimal() {
        let yaml = "app:\n  name: test-service\nserver:\n  port: 9090\n";
        let config: RuntimeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.name, "test-service");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.server.host, "0.0.0.0");
        assert!(config.database.is_none());
    }

    #[test]
    fn test_runtime_config_deserialize_full() {
        let yaml = r#"
app:
  name: task-server
  version: "1.0.0"
  tier: service
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
grpc:
  port: 50051
  max_recv_msg_size: 4194304
database:
  host: localhost
  port: 5432
  name: task_db
  user: app
  password: ""
  ssl_mode: disable
kafka:
  brokers:
    - "kafka-0:9092"
  consumer_group: task-server.default
  security_protocol: PLAINTEXT
redis:
  host: localhost
  port: 6379
  db: 0
  pool_size: 10
redis_session:
  host: redis-session.k1s0-system.svc.cluster.local
  port: 6380
  password: ""
observability:
  log:
    level: info
    format: json
  trace:
    enabled: true
    sample_rate: 1.0
  metrics:
    enabled: true
    path: /metrics
auth:
  jwt:
    issuer: "https://auth.example.com"
    audience: k1s0-api
  oidc:
    discovery_url: "https://auth.example.com/.well-known/openid-configuration"
    client_id: k1s0-bff
    client_secret: ""
    redirect_uri: "https://app.example.com/callback"
    scopes:
      - openid
      - profile
    jwks_uri: "https://auth.example.com/certs"
"#;
        let config: RuntimeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.name, "task-server");
        assert_eq!(config.app.tier, "service");
        assert_eq!(config.server.port, 8080);
        let grpc = config.grpc.unwrap();
        assert_eq!(grpc.port, 50051);
        assert_eq!(grpc.max_recv_msg_size, Some(4_194_304));
        let db = config.database.unwrap();
        assert_eq!(db.name, "task_db");
        assert_eq!(db.ssl_mode, "disable");
        let kafka = config.kafka.unwrap();
        assert_eq!(kafka.brokers, vec!["kafka-0:9092"]);
        assert_eq!(kafka.security_protocol, "PLAINTEXT");
        let redis = config.redis.unwrap();
        assert_eq!(redis.port, 6379);
        let redis_session = config.redis_session.unwrap();
        assert_eq!(
            redis_session.host,
            "redis-session.k1s0-system.svc.cluster.local"
        );
        assert_eq!(redis_session.port, 6380);
        assert_eq!(config.observability.log.level, "info");
        assert!((config.observability.trace.sample_rate - 1.0_f64).abs() < f64::EPSILON);
        assert_eq!(config.auth.jwt.issuer, "https://auth.example.com");
        let oidc = config.auth.oidc.unwrap();
        assert_eq!(oidc.client_id, "k1s0-bff");
        assert_eq!(oidc.scopes, vec!["openid", "profile"]);
    }

    fn valid_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            app: AppConfig {
                name: "test-service".to_string(),
                version: "1.0.0".to_string(),
                tier: "system".to_string(),
                environment: "dev".to_string(),
            },
            server: ServerConfig::default(),
            grpc: None,
            database: None,
            kafka: Some(KafkaConfig {
                brokers: vec!["localhost:9092".to_string()],
                consumer_group: "test".to_string(),
                security_protocol: "PLAINTEXT".to_string(),
                sasl: None,
                tls: None,
                topics: KafkaTopics::default(),
            }),
            redis: None,
            redis_session: None,
            observability: ObservabilityConfig::default(),
            auth: AuthConfig::default(),
        }
    }

    #[test]
    fn test_runtime_config_validate_success() {
        assert!(valid_runtime_config().validate().is_ok());
    }

    #[test]
    fn test_runtime_config_validate_invalid_tier() {
        let mut config = valid_runtime_config();
        config.app.tier = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_runtime_config_validate_invalid_environment() {
        let mut config = valid_runtime_config();
        config.app.environment = "qa".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_runtime_config_validate_invalid_log_level() {
        let mut config = valid_runtime_config();
        config.observability.log.level = "trace".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_runtime_config_validate_invalid_kafka_protocol() {
        let mut config = valid_runtime_config();
        config.kafka.as_mut().unwrap().security_protocol = "SSL".to_string();
        assert!(config.validate().is_err());
    }
}
