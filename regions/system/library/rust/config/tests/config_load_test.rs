//! Integration tests for config loading, merging, and validation workflows.
//!
//! Complements config_test.rs by focusing on:
//! - merge_yaml behaviour with real Config structs
//! - Multi-layer overlay scenarios
//! - Validation of valid full configs with all optional sections
//! - Edge cases in env overlay + vault secret merging combined

use k1s0_config::*;
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn yaml_file(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "{}", content).unwrap();
    f
}

fn minimal_yaml() -> &'static str {
    r#"
app:
  name: test-server
  version: "1.0.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log:
    level: debug
    format: json
  trace:
    enabled: false
  metrics:
    enabled: false
auth:
  jwt:
    issuer: "http://localhost:8180/realms/k1s0"
    audience: "k1s0-api"
"#
}

// ===========================================================================
// Load: basic scenarios
// ===========================================================================

// 最小構成の設定ファイルが正しくロードされることを確認する。
#[test]
fn load_minimal_config_succeeds() {
    let f = yaml_file(minimal_yaml());
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    assert_eq!(cfg.app.name, "test-server");
    assert_eq!(cfg.app.version, "1.0.0");
    assert_eq!(cfg.app.tier, "system");
    assert_eq!(cfg.app.environment, "dev");
    assert_eq!(cfg.server.host, "0.0.0.0");
    assert_eq!(cfg.server.port, 8080);
    assert!(cfg.database.is_none());
    assert!(cfg.kafka.is_none());
    assert!(cfg.redis.is_none());
    assert!(cfg.grpc.is_none());
}

// 存在しないファイルをロードした場合に読み込みエラーが返されることを確認する。
#[test]
fn load_nonexistent_file_returns_read_error() {
    let result = load("/nonexistent/does-not-exist.yaml", None);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("failed to read file"), "got: {msg}");
}

// 存在しない環境オーバーレイファイルを指定した場合に読み込みエラーが返されることを確認する。
#[test]
fn load_nonexistent_env_file_returns_read_error() {
    let base = yaml_file(minimal_yaml());
    let result = load(base.path().to_str().unwrap(), Some("/nonexistent/env.yaml"));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("failed to read file"), "got: {msg}");
}

// 不正な YAML ファイルをロードした場合にパースエラーが返されることを確認する。
#[test]
fn load_malformed_yaml_returns_parse_error() {
    let f = yaml_file("not: [valid: yaml: {{");
    let result = load(f.path().to_str().unwrap(), None);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("failed to parse YAML"), "got: {msg}");
}

// ===========================================================================
// Load with env overlay: merge behaviour
// ===========================================================================

// 環境オーバーレイがスカラーフィールドを正しく上書きすることを確認する。
#[test]
fn env_overlay_overrides_scalar_fields() {
    let base = yaml_file(minimal_yaml());
    let env = yaml_file(
        r#"
app:
  environment: prod
server:
  port: 443
observability:
  log:
    level: warn
"#,
    );
    let cfg = load(
        base.path().to_str().unwrap(),
        Some(env.path().to_str().unwrap()),
    )
    .unwrap();
    assert_eq!(cfg.app.environment, "prod");
    assert_eq!(cfg.server.port, 443);
    assert_eq!(cfg.observability.log.level, "warn");
}

// 環境オーバーレイで言及されていないフィールドがベース値のまま保持されることを確認する。
#[test]
fn env_overlay_preserves_unmentioned_fields() {
    let base = yaml_file(minimal_yaml());
    let env = yaml_file(
        r#"
app:
  environment: staging
"#,
    );
    let cfg = load(
        base.path().to_str().unwrap(),
        Some(env.path().to_str().unwrap()),
    )
    .unwrap();
    assert_eq!(cfg.app.environment, "staging");
    // Unmentioned fields from base remain
    assert_eq!(cfg.app.name, "test-server");
    assert_eq!(cfg.app.version, "1.0.0");
    assert_eq!(cfg.server.port, 8080);
    assert_eq!(cfg.auth.jwt.issuer, "http://localhost:8180/realms/k1s0");
}

// 環境オーバーレイでベースにないオプションセクションを追加できることを確認する。
#[test]
fn env_overlay_adds_optional_section() {
    let base = yaml_file(minimal_yaml());
    let env = yaml_file(
        r#"
database:
  host: "db.example.com"
  port: 5432
  name: "mydb"
  user: "app"
  password: "secret"
"#,
    );
    let cfg = load(
        base.path().to_str().unwrap(),
        Some(env.path().to_str().unwrap()),
    )
    .unwrap();
    assert!(cfg.database.is_some());
    let db = cfg.database.unwrap();
    assert_eq!(db.host, "db.example.com");
    assert_eq!(db.port, 5432);
    assert_eq!(db.name, "mydb");
}

// ===========================================================================
// merge_yaml: direct function tests
// ===========================================================================

// オーバーレイでベースのマッピングキーが保持されることを確認する。
#[test]
fn merge_yaml_preserves_base_mapping_keys() {
    let mut base: serde_yaml::Value = serde_yaml::from_str(
        r#"
app:
  name: original
  version: "1.0"
  extra: keep-me
"#,
    )
    .unwrap();
    let overlay: serde_yaml::Value = serde_yaml::from_str(
        r#"
app:
  version: "2.0"
"#,
    )
    .unwrap();
    merge_yaml(&mut base, &overlay);
    assert_eq!(
        base["app"]["name"],
        serde_yaml::Value::String("original".into())
    );
    assert_eq!(
        base["app"]["version"],
        serde_yaml::Value::String("2.0".into())
    );
    assert_eq!(
        base["app"]["extra"],
        serde_yaml::Value::String("keep-me".into())
    );
}

// 空のオーバーレイを適用してもベース設定が変更されないことを確認する。
#[test]
fn merge_yaml_empty_overlay_leaves_base_unchanged() {
    let mut base: serde_yaml::Value = serde_yaml::from_str("key: value").unwrap();
    let original = base.clone();
    let overlay: serde_yaml::Value = serde_yaml::from_str("{}").unwrap();
    merge_yaml(&mut base, &overlay);
    assert_eq!(base, original);
}

// オーバーレイによって配列が完全に置き換わることを確認する。
#[test]
fn merge_yaml_replaces_array_entirely() {
    let mut base: serde_yaml::Value = serde_yaml::from_str("items:\n  - a\n  - b\n  - c").unwrap();
    let overlay: serde_yaml::Value = serde_yaml::from_str("items:\n  - x").unwrap();
    merge_yaml(&mut base, &overlay);
    let items = base["items"].as_sequence().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], serde_yaml::Value::String("x".into()));
}

// 深くネストされた YAML 構造が正しくマージされることを確認する。
#[test]
fn merge_yaml_deeply_nested() {
    let mut base: serde_yaml::Value = serde_yaml::from_str(
        r#"
a:
  b:
    c:
      d: old
      e: keep
"#,
    )
    .unwrap();
    let overlay: serde_yaml::Value = serde_yaml::from_str(
        r#"
a:
  b:
    c:
      d: new
      f: added
"#,
    )
    .unwrap();
    merge_yaml(&mut base, &overlay);
    assert_eq!(
        base["a"]["b"]["c"]["d"],
        serde_yaml::Value::String("new".into())
    );
    assert_eq!(
        base["a"]["b"]["c"]["e"],
        serde_yaml::Value::String("keep".into())
    );
    assert_eq!(
        base["a"]["b"]["c"]["f"],
        serde_yaml::Value::String("added".into())
    );
}

// ===========================================================================
// Validate: valid full config with all optional sections
// ===========================================================================

// 全オプションセクションを含む完全な設定でバリデーションが成功することを確認する。
#[test]
fn validate_full_config_with_all_sections() {
    let f = yaml_file(
        r#"
app:
  name: order-server
  version: "2.0.0"
  tier: business
  environment: staging
server:
  host: "0.0.0.0"
  port: 8080
  read_timeout: "30s"
  write_timeout: "30s"
  shutdown_timeout: "10s"
grpc:
  port: 50051
  max_recv_msg_size: 4194304
database:
  host: "db.example.com"
  port: 5432
  name: "order_db"
  user: "app"
  password: "secret"
  ssl_mode: "require"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"
kafka:
  brokers:
    - "kafka-0:9092"
    - "kafka-1:9092"
  consumer_group: "order-server.default"
  security_protocol: "PLAINTEXT"
  topics:
    publish:
      - "k1s0.business.order.created.v1"
    subscribe:
      - "k1s0.business.payment.completed.v1"
redis:
  host: "redis.example.com"
  port: 6379
  db: 0
  pool_size: 10
redis_session:
  host: "redis-session.example.com"
  port: 6380
  pool_size: 5
observability:
  log:
    level: info
    format: json
  trace:
    enabled: true
    endpoint: "otel-collector:4317"
    sample_rate: 0.5
  metrics:
    enabled: true
    path: "/metrics"
auth:
  jwt:
    issuer: "http://keycloak:8180/realms/k1s0"
    audience: "k1s0-api"
  oidc:
    discovery_url: "http://keycloak:8180/realms/k1s0/.well-known/openid-configuration"
    client_id: "k1s0-bff"
    redirect_uri: "http://localhost:3000/callback"
    scopes:
      - "openid"
      - "profile"
    jwks_uri: "http://keycloak:8180/realms/k1s0/protocol/openid-connect/certs"
    jwks_cache_ttl: "10m"
"#,
    );
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    assert!(validate(&cfg).is_ok());
}

// ===========================================================================
// Validate: boundary value tests
// ===========================================================================

// sample_rate の境界値 (0.0 および 1.0) でバリデーションが成功することを確認する。
#[test]
fn validate_trace_sample_rate_at_boundaries() {
    // 0.0 is valid
    let f = yaml_file(
        r#"
app: { name: t, version: "1", tier: system, environment: dev }
server: { host: "0.0.0.0", port: 8080 }
observability:
  log: { level: info, format: json }
  trace: { enabled: false, sample_rate: 0.0 }
  metrics: { enabled: false }
auth:
  jwt: { issuer: x, audience: x }
"#,
    );
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    assert!(validate(&cfg).is_ok());

    // 1.0 is valid
    let f2 = yaml_file(
        r#"
app: { name: t, version: "1", tier: system, environment: dev }
server: { host: "0.0.0.0", port: 8080 }
observability:
  log: { level: info, format: json }
  trace: { enabled: false, sample_rate: 1.0 }
  metrics: { enabled: false }
auth:
  jwt: { issuer: x, audience: x }
"#,
    );
    let cfg2 = load(f2.path().to_str().unwrap(), None).unwrap();
    assert!(validate(&cfg2).is_ok());
}

// sample_rate が負の場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_trace_sample_rate_negative_rejected() {
    let f = yaml_file(
        r#"
app: { name: t, version: "1", tier: system, environment: dev }
server: { host: "0.0.0.0", port: 8080 }
observability:
  log: { level: info, format: json }
  trace: { enabled: false, sample_rate: -0.1 }
  metrics: { enabled: false }
auth:
  jwt: { issuer: x, audience: x }
"#,
    );
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    assert!(validate(&cfg).is_err());
}

// ===========================================================================
// Validate: kafka with SASL_SSL and all fields
// ===========================================================================

// SASL_SSL かつ有効な SASL 設定の場合にバリデーションが成功することを確認する。
#[test]
fn validate_kafka_sasl_ssl_with_valid_sasl() {
    let f = yaml_file(
        r#"
app: { name: t, version: "1", tier: system, environment: dev }
server: { host: "0.0.0.0", port: 8080 }
kafka:
  brokers: ["kafka:9092"]
  consumer_group: grp
  security_protocol: SASL_SSL
  sasl:
    mechanism: SCRAM-SHA-512
    username: user
    password: pass
  tls:
    ca_cert_path: /etc/ssl/ca.pem
  topics:
    publish: [t1]
    subscribe: []
observability:
  log: { level: info, format: json }
  trace: { enabled: false }
  metrics: { enabled: false }
auth:
  jwt: { issuer: x, audience: x }
"#,
    );
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    assert!(validate(&cfg).is_ok());
}

// TLS の ca_cert_path が空白の場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_kafka_empty_tls_ca_cert_path_rejected() {
    let f = yaml_file(
        r#"
app: { name: t, version: "1", tier: system, environment: dev }
server: { host: "0.0.0.0", port: 8080 }
kafka:
  brokers: ["kafka:9092"]
  consumer_group: grp
  security_protocol: SASL_SSL
  sasl:
    mechanism: SCRAM-SHA-512
    username: user
    password: pass
  tls:
    ca_cert_path: "  "
  topics:
    publish: [t1]
    subscribe: []
observability:
  log: { level: info, format: json }
  trace: { enabled: false }
  metrics: { enabled: false }
auth:
  jwt: { issuer: x, audience: x }
"#,
    );
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    let err = validate(&cfg).unwrap_err().to_string();
    assert!(err.contains("ca_cert_path"), "got: {err}");
}

// ===========================================================================
// End-to-end: load + overlay + vault merge + validate
// ===========================================================================

// ロード・オーバーレイ適用・Vault シークレットマージ・バリデーションが一連で動作することを確認する。
#[test]
fn end_to_end_load_overlay_vault_validate() {
    let base = yaml_file(
        r#"
app:
  name: svc
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: localhost
  port: 5432
  name: mydb
  user: app
  password: placeholder
observability:
  log: { level: debug, format: json }
  trace: { enabled: false }
  metrics: { enabled: false }
auth:
  jwt: { issuer: "http://localhost", audience: test }
"#,
    );
    let overlay = yaml_file(
        r#"
app:
  environment: staging
server:
  port: 9090
observability:
  log:
    level: info
"#,
    );

    let mut cfg = load(
        base.path().to_str().unwrap(),
        Some(overlay.path().to_str().unwrap()),
    )
    .unwrap();

    // Verify overlay was applied
    assert_eq!(cfg.app.environment, "staging");
    assert_eq!(cfg.server.port, 9090);
    assert_eq!(cfg.observability.log.level, "info");
    // Verify base preserved
    assert_eq!(cfg.app.name, "svc");
    assert_eq!(cfg.database.as_ref().unwrap().host, "localhost");

    // Apply vault secrets
    let secrets = HashMap::from([("database.password".into(), "vault-secret-pw".into())]);
    merge_vault_secrets(&mut cfg, &secrets);
    assert_eq!(cfg.database.as_ref().unwrap().password, "vault-secret-pw");

    // Validate final config
    assert!(validate(&cfg).is_ok());
}

// 全種類のシークレットが一度にマージされることを確認する。
#[test]
fn vault_merge_with_all_secret_types() {
    let f = yaml_file(
        r#"
app: { name: svc, version: "1.0", tier: system, environment: dev }
server: { host: "0.0.0.0", port: 8080 }
database:
  host: localhost
  port: 5432
  name: db
  user: app
  password: ""
redis:
  host: localhost
  port: 6379
redis_session:
  host: localhost
  port: 6380
kafka:
  brokers: ["localhost:9092"]
  consumer_group: grp
  security_protocol: SASL_SSL
  sasl:
    mechanism: SCRAM-SHA-512
    username: ""
    password: ""
  topics:
    publish: [t1]
    subscribe: []
observability:
  log: { level: info, format: json }
  trace: { enabled: false }
  metrics: { enabled: false }
auth:
  jwt: { issuer: x, audience: x }
  oidc:
    discovery_url: http://localhost/.well-known
    client_id: c
    redirect_uri: http://localhost/cb
    scopes: [openid]
    jwks_uri: http://localhost/jwks
"#,
    );

    let mut cfg = load(f.path().to_str().unwrap(), None).unwrap();
    let secrets = HashMap::from([
        ("database.password".into(), "db-secret".into()),
        ("redis.password".into(), "redis-secret".into()),
        ("redis_session.password".into(), "session-secret".into()),
        ("kafka.sasl.username".into(), "kafka-user".into()),
        ("kafka.sasl.password".into(), "kafka-pass".into()),
        ("auth.oidc.client_secret".into(), "oidc-secret".into()),
    ]);
    merge_vault_secrets(&mut cfg, &secrets);

    assert_eq!(cfg.database.as_ref().unwrap().password, "db-secret");
    assert_eq!(
        cfg.redis.as_ref().unwrap().password,
        Some("redis-secret".into())
    );
    assert_eq!(
        cfg.redis_session.as_ref().unwrap().password,
        Some("session-secret".into())
    );
    let sasl = cfg.kafka.as_ref().unwrap().sasl.as_ref().unwrap();
    assert_eq!(sasl.username, "kafka-user");
    assert_eq!(sasl.password, "kafka-pass");
    assert_eq!(
        cfg.auth.oidc.as_ref().unwrap().client_secret,
        Some("oidc-secret".into())
    );
}

// ===========================================================================
// ConfigError: Debug formatting
// ===========================================================================

// バリデーションエラーの表示文字列に適切なメッセージが含まれることを確認する。
#[test]
fn config_error_validation_display() {
    let f = yaml_file(minimal_yaml());
    let mut cfg = load(f.path().to_str().unwrap(), None).unwrap();
    cfg.app.name = String::new();
    let err = validate(&cfg).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("validation error"), "got: {msg}");
    assert!(msg.contains("app.name"), "got: {msg}");
}

// 設定エラーが Debug トレイトを実装していることを確認する。
#[test]
fn config_error_is_debuggable() {
    let f = yaml_file(minimal_yaml());
    let mut cfg = load(f.path().to_str().unwrap(), None).unwrap();
    cfg.app.name = String::new();
    let err = validate(&cfg).unwrap_err();
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("Validation"));
}
