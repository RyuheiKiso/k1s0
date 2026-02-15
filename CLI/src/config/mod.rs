use serde::{Deserialize, Serialize};
use std::path::Path;

/// CLI全体の設定を保持する構造体。
///
/// プロジェクトルートの設定ファイル (k1s0.yaml) から読み込む。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CliConfig {
    /// プロジェクト名
    pub project_name: String,
    /// リージョンのルートパス
    pub regions_root: String,
    /// Docker レジストリ
    pub docker_registry: String,
    /// Go モジュールのベースパス
    pub go_module_base: String,
}

// ============================================================================
// ランタイム設定スキーマ (config設計.md 準拠)
// ============================================================================

/// ランタイムサービス設定 (config/config.yaml のスキーマ)。
///
/// config設計.md の「Rust での読み込み実装」セクションに準拠。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RuntimeConfig {
    /// アプリケーション設定
    pub app: AppConfig,
    /// HTTP サーバー設定
    pub server: ServerConfig,
    /// gRPC 設定 (gRPC 有効時のみ)
    pub grpc: Option<GrpcConfig>,
    /// データベース設定 (DB 有効時のみ)
    pub database: Option<DatabaseConfig>,
    /// Kafka 設定 (Kafka 有効時のみ)
    pub kafka: Option<KafkaConfig>,
    /// Redis 設定 (Redis 有効時のみ)
    pub redis: Option<RedisConfig>,
    /// 可観測性設定
    pub observability: ObservabilityConfig,
    /// 認証設定
    pub auth: AuthConfig,
}

/// アプリケーション基本設定。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub tier: String,
    pub environment: String,
}

/// HTTP サーバー設定。
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

/// gRPC 設定。
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

/// データベース設定。
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

/// Kafka 設定。
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

/// Kafka SASL 設定。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KafkaSaslConfig {
    pub mechanism: String,
    pub username: String,
    pub password: String,
}

/// Kafka TLS 設定。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KafkaTlsConfig {
    pub ca_cert_path: Option<String>,
}

/// Kafka トピック設定。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KafkaTopics {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
}

/// Redis 設定。
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

/// 可観測性設定。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ObservabilityConfig {
    pub log: LogConfig,
    pub trace: TraceConfig,
    pub metrics: MetricsConfig,
}

/// ログ設定。
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

/// トレース設定。
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

/// メトリクス設定。
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

/// 認証設定。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AuthConfig {
    pub jwt: JwtConfig,
    pub oidc: Option<OidcConfig>,
}

/// JWT 設定。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
    pub public_key_path: Option<String>,
}

/// OIDC 設定。
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

/// 設定ファイルを読み込む。
///
/// 指定されたパスから YAML 形式の設定ファイルを読み込む。
/// ファイルが存在しない場合はデフォルト値を返す。
pub fn load_config(path: &str) -> anyhow::Result<CliConfig> {
    let config_path = Path::new(path);
    if !config_path.exists() {
        return Ok(CliConfig::default());
    }
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| anyhow::anyhow!("設定ファイルの読み込みに失敗: {}", e))?;
    let config: CliConfig = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("設定ファイルのパースに失敗: {}", e))?;
    Ok(config)
}

/// Vault からシークレットをマージする (第3段階)。
///
/// config設計.md のマージ順序:
///   1. config.yaml (デフォルト値) -- 最低優先
///   2. config.{environment}.yaml で上書き
///   3. Vault から注入されたシークレットで上書き -- 最高優先
///
/// # Arguments
/// * `_base` - マージ先の設定
/// * `vault_addr` - Vault サーバーのアドレス
/// * `vault_path` - Vault 上のシークレットパス
///
/// # Returns
/// 成功時は `Ok(())`、Vault 未到達時は警告ログを出力して `Ok(())` を返す。
///
/// TODO: Vault統合 -- 実際の Vault 通信を実装する
pub fn merge_vault_secrets(
    _base: &mut CliConfig,
    vault_addr: &str,
    vault_path: &str,
) -> anyhow::Result<()> {
    if vault_addr.is_empty() || vault_path.is_empty() {
        // Vault 未設定時は何もしない (no-op)
        return Ok(());
    }

    // TODO: Vault統合 -- 以下の処理を実装する:
    //   1. vault_addr に接続
    //   2. vault_path からシークレットを取得
    //   3. base の該当フィールドに上書きマージ
    //   4. ConfigMap と Vault で同一キーが存在する場合は Vault を優先し警告ログを出力

    // Vault 未到達時は警告ログを出力
    eprintln!(
        "WARN: Vault ({}) にアクセスできません。シークレットのマージをスキップします。path={}",
        vault_addr, vault_path
    );

    Ok(())
}

/// 環境別設定をマージする。
///
/// ベース設定に環境別設定を上書きマージする。
/// config設計.md のマージ順序: config.yaml < config.{env}.yaml < Vault
pub fn merge_config(base: &mut CliConfig, override_path: &str) -> anyhow::Result<()> {
    let path = Path::new(override_path);
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("環境別設定の読み込みに失敗: {}", e))?;
    let override_config: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("環境別設定のパースに失敗: {}", e))?;

    if let serde_yaml::Value::Mapping(map) = override_config {
        if let Some(serde_yaml::Value::String(name)) = map.get(&serde_yaml::Value::String("project_name".to_string())) {
            base.project_name = name.clone();
        }
        if let Some(serde_yaml::Value::String(root)) = map.get(&serde_yaml::Value::String("regions_root".to_string())) {
            base.regions_root = root.clone();
        }
        if let Some(serde_yaml::Value::String(registry)) = map.get(&serde_yaml::Value::String("docker_registry".to_string())) {
            base.docker_registry = registry.clone();
        }
        if let Some(serde_yaml::Value::String(go_base)) = map.get(&serde_yaml::Value::String("go_module_base".to_string())) {
            base.go_module_base = go_base.clone();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

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
        let mut base = CliConfig::default();
        base.project_name = "base-project".to_string();

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "project_name: overridden\ndocker_registry: custom-registry.io").unwrap();

        merge_config(&mut base, file.path().to_str().unwrap()).unwrap();
        assert_eq!(base.project_name, "overridden");
        assert_eq!(base.docker_registry, "custom-registry.io");
        assert_eq!(base.regions_root, "regions"); // not overridden
    }

    #[test]
    fn test_merge_config_nonexistent_file_noop() {
        let mut base = CliConfig::default();
        base.project_name = "original".to_string();
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

    // --- Vault 統合スタブ ---

    #[test]
    fn test_merge_vault_secrets_empty_addr_is_noop() {
        let mut base = CliConfig::default();
        base.project_name = "original".to_string();
        let result = merge_vault_secrets(&mut base, "", "secret/data/k1s0");
        assert!(result.is_ok());
        assert_eq!(base.project_name, "original");
    }

    #[test]
    fn test_merge_vault_secrets_empty_path_is_noop() {
        let mut base = CliConfig::default();
        base.project_name = "original".to_string();
        let result = merge_vault_secrets(&mut base, "https://vault.example.com", "");
        assert!(result.is_ok());
        assert_eq!(base.project_name, "original");
    }

    #[test]
    fn test_merge_vault_secrets_unreachable_warns() {
        // Vault 未到達時は警告ログを出力するが、エラーにはしない
        let mut base = CliConfig::default();
        let result = merge_vault_secrets(
            &mut base,
            "https://vault.example.com",
            "secret/data/k1s0/service/order/database",
        );
        assert!(result.is_ok());
    }

    // --- ランタイム設定スキーマ ---

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
        assert_eq!(config.observability.log.level, "info");
        assert_eq!(config.auth.jwt.issuer, "");
    }

    #[test]
    fn test_runtime_config_deserialize_minimal() {
        let yaml = "app:\n  name: test-service\nserver:\n  port: 9090\n";
        let config: RuntimeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.app.name, "test-service");
        assert_eq!(config.server.port, 9090);
        // 未指定フィールドはデフォルト
        assert_eq!(config.server.host, "0.0.0.0");
        assert!(config.database.is_none());
    }

    #[test]
    fn test_runtime_config_deserialize_full() {
        let yaml = r#"
app:
  name: order-server
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
  name: order_db
  user: app
  password: ""
  ssl_mode: disable
kafka:
  brokers:
    - "kafka-0:9092"
  consumer_group: order-server.default
  security_protocol: PLAINTEXT
redis:
  host: localhost
  port: 6379
  db: 0
  pool_size: 10
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
        assert_eq!(config.app.name, "order-server");
        assert_eq!(config.app.tier, "service");
        assert_eq!(config.server.port, 8080);
        let grpc = config.grpc.unwrap();
        assert_eq!(grpc.port, 50051);
        assert_eq!(grpc.max_recv_msg_size, Some(4_194_304));
        let db = config.database.unwrap();
        assert_eq!(db.name, "order_db");
        assert_eq!(db.ssl_mode, "disable");
        let kafka = config.kafka.unwrap();
        assert_eq!(kafka.brokers, vec!["kafka-0:9092"]);
        assert_eq!(kafka.security_protocol, "PLAINTEXT");
        let redis = config.redis.unwrap();
        assert_eq!(redis.port, 6379);
        assert_eq!(config.observability.log.level, "info");
        assert_eq!(config.observability.trace.sample_rate, 1.0);
        assert_eq!(config.auth.jwt.issuer, "https://auth.example.com");
        let oidc = config.auth.oidc.unwrap();
        assert_eq!(oidc.client_id, "k1s0-bff");
        assert_eq!(oidc.scopes, vec!["openid", "profile"]);
    }

    #[test]
    fn test_runtime_config_serialization_roundtrip() {
        let config = RuntimeConfig {
            app: AppConfig {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                tier: "service".to_string(),
                environment: "dev".to_string(),
            },
            server: ServerConfig::default(),
            grpc: Some(GrpcConfig::default()),
            database: Some(DatabaseConfig::default()),
            kafka: None,
            redis: None,
            observability: ObservabilityConfig::default(),
            auth: AuthConfig::default(),
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: RuntimeConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.app.name, deserialized.app.name);
        assert_eq!(config.server.port, deserialized.server.port);
        assert!(deserialized.grpc.is_some());
        assert!(deserialized.database.is_some());
        assert!(deserialized.kafka.is_none());
    }
}
