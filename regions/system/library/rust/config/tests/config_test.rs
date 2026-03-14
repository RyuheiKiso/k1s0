//! External integration tests for k1s0-config.
//!
//! These tests complement the inline unit tests in src/tests.rs and src/vault_test.rs.
//! Focus areas:
//! - merge_yaml edge cases (deep nesting, array replacement, type coercion)
//! - Validation boundary conditions not covered inline (database, redis, grpc constraints)
//! - ConfigError Display formatting
//! - End-to-end load + merge + validate workflows

use k1s0_config::*;
use std::io::Write;
use tempfile::NamedTempFile;

// ---------------------------------------------------------------------------
// Helper: write YAML to a temp file and return the path holder
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

fn valid_config() -> Config {
    let f = yaml_file(minimal_yaml());
    load(f.path().to_str().unwrap(), None).unwrap()
}

// ===========================================================================
// merge_yaml tests — complementary to inline tests
// ===========================================================================

// 深くネストされたマッピングが正しくマージされることを確認する。
#[test]
fn merge_yaml_deep_nested_mapping() {
    let mut base: serde_yaml::Value = serde_yaml::from_str(
        r#"
a:
  b:
    c: 1
    d: 2
"#,
    )
    .unwrap();
    let overlay: serde_yaml::Value = serde_yaml::from_str(
        r#"
a:
  b:
    c: 99
    e: 3
"#,
    )
    .unwrap();
    merge_yaml(&mut base, &overlay);
    assert_eq!(base["a"]["b"]["c"], serde_yaml::Value::Number(99.into()));
    assert_eq!(base["a"]["b"]["d"], serde_yaml::Value::Number(2.into()));
    assert_eq!(base["a"]["b"]["e"], serde_yaml::Value::Number(3.into()));
}

// スカラー値をマッピングで上書きできることを確認する。
#[test]
fn merge_yaml_replaces_scalar_with_mapping() {
    let mut base: serde_yaml::Value =
        serde_yaml::from_str("key: simple_string").unwrap();
    let overlay: serde_yaml::Value =
        serde_yaml::from_str("key:\n  nested: value").unwrap();
    merge_yaml(&mut base, &overlay);
    assert_eq!(
        base["key"]["nested"],
        serde_yaml::Value::String("value".into())
    );
}

// シーケンスはオーバーレイで完全に置き換わることを確認する。
#[test]
fn merge_yaml_replaces_sequence_entirely() {
    let mut base: serde_yaml::Value =
        serde_yaml::from_str("items:\n  - a\n  - b").unwrap();
    let overlay: serde_yaml::Value =
        serde_yaml::from_str("items:\n  - x").unwrap();
    merge_yaml(&mut base, &overlay);
    let items = base["items"].as_sequence().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0], serde_yaml::Value::String("x".into()));
}

// オーバーレイに新しいトップレベルキーを追加できることを確認する。
#[test]
fn merge_yaml_adds_new_top_level_key() {
    let mut base: serde_yaml::Value = serde_yaml::from_str("a: 1").unwrap();
    let overlay: serde_yaml::Value = serde_yaml::from_str("b: 2").unwrap();
    merge_yaml(&mut base, &overlay);
    assert_eq!(base["a"], serde_yaml::Value::Number(1.into()));
    assert_eq!(base["b"], serde_yaml::Value::Number(2.into()));
}

// null のオーバーレイ値でベースの値が null に置き換わることを確認する。
#[test]
fn merge_yaml_null_overlay_replaces_value() {
    let mut base: serde_yaml::Value = serde_yaml::from_str("key: value").unwrap();
    let overlay: serde_yaml::Value = serde_yaml::from_str("key: null").unwrap();
    merge_yaml(&mut base, &overlay);
    assert!(base["key"].is_null());
}

// ===========================================================================
// ConfigError display tests
// ===========================================================================

// ファイル読み込みエラーの表示文字列に適切なメッセージが含まれることを確認する。
#[test]
fn config_error_read_file_display() {
    let result = load("/nonexistent/path/config.yaml", None);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("failed to read file"), "got: {msg}");
}

// YAML パースエラーの表示文字列に適切なメッセージが含まれることを確認する。
#[test]
fn config_error_parse_yaml_display() {
    let f = yaml_file("invalid: [yaml: broken");
    let result = load(f.path().to_str().unwrap(), None);
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("failed to parse YAML"), "got: {msg}");
}

// ===========================================================================
// Validation: database constraints
// ===========================================================================

// データベースポートが 0 の場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_database_zero_port_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: localhost
  port: 0
  name: test_db
  user: app
  password: secret
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
    assert!(err.contains("database.port"), "got: {err}");
}

// max_idle_conns が max_open_conns を超える場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_database_max_idle_exceeds_max_open_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: localhost
  port: 5432
  name: test_db
  user: app
  password: secret
  max_open_conns: 5
  max_idle_conns: 10
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
    assert!(
        err.contains("max_idle_conns"),
        "got: {err}"
    );
}

// 無効な ssl_mode を指定した場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_database_invalid_ssl_mode_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: localhost
  port: 5432
  name: test_db
  user: app
  password: secret
  ssl_mode: "prefer"
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
    assert!(err.contains("database.ssl_mode"), "got: {err}");
}

// 有効なデータベース設定でバリデーションが成功することを確認する。
#[test]
fn validate_database_valid_config_accepted() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
database:
  host: localhost
  port: 5432
  name: test_db
  user: app
  password: secret
  ssl_mode: require
  max_open_conns: 25
  max_idle_conns: 5
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

// ===========================================================================
// Validation: gRPC constraints
// ===========================================================================

// gRPC ポートが 0 の場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_grpc_zero_port_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
grpc:
  port: 0
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
    assert!(err.contains("grpc.port"), "got: {err}");
}

// gRPC の max_recv_msg_size が 0 の場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_grpc_zero_max_recv_msg_size_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
grpc:
  port: 50051
  max_recv_msg_size: 0
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
    assert!(err.contains("max_recv_msg_size"), "got: {err}");
}

// ===========================================================================
// Validation: redis constraints
// ===========================================================================

// Redis の pool_size が 0 の場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_redis_zero_pool_size_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
redis:
  host: localhost
  port: 6379
  pool_size: 0
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
    assert!(err.contains("redis.pool_size"), "got: {err}");
}

// Redis セッションの pool_size が 0 の場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_redis_session_zero_pool_size_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
redis_session:
  host: localhost
  port: 6380
  pool_size: 0
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
    assert!(err.contains("redis_session.pool_size"), "got: {err}");
}

// ===========================================================================
// Validation: observability trace/metrics edge cases
// ===========================================================================

// sample_rate が 0〜1 の範囲外の場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_trace_sample_rate_out_of_range_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log: { level: info, format: json }
  trace:
    enabled: false
    sample_rate: 1.5
  metrics: { enabled: false }
auth:
  jwt: { issuer: x, audience: x }
"#,
    );
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    let err = validate(&cfg).unwrap_err().to_string();
    assert!(err.contains("sample_rate"), "got: {err}");
}

// sample_rate が 0.0 の場合にバリデーションが成功することを確認する。
#[test]
fn validate_trace_sample_rate_zero_accepted() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log: { level: info, format: json }
  trace:
    enabled: false
    sample_rate: 0.0
  metrics: { enabled: false }
auth:
  jwt: { issuer: x, audience: x }
"#,
    );
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    assert!(validate(&cfg).is_ok());
}

// メトリクスパスが "/" で始まらない場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_metrics_path_must_start_with_slash() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log: { level: info, format: json }
  trace: { enabled: false }
  metrics:
    enabled: true
    path: "metrics"
auth:
  jwt: { issuer: x, audience: x }
"#,
    );
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    let err = validate(&cfg).unwrap_err().to_string();
    assert!(err.contains("must start with '/'"), "got: {err}");
}

// 無効なログフォーマットを指定した場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_invalid_log_format_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log: { level: info, format: xml }
  trace: { enabled: false }
  metrics: { enabled: false }
auth:
  jwt: { issuer: x, audience: x }
"#,
    );
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    let err = validate(&cfg).unwrap_err().to_string();
    assert!(err.contains("observability.log.format"), "got: {err}");
}

// ===========================================================================
// Validation: kafka edge cases
// ===========================================================================

// Kafka ブローカーリストに空文字が含まれる場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_kafka_empty_broker_string_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
kafka:
  brokers: ["  "]
  consumer_group: grp
  security_protocol: PLAINTEXT
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
    assert!(err.contains("brokers must not contain empty"), "got: {err}");
}

// Kafka パブリッシュトピックリストに空文字が含まれる場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_kafka_empty_topic_name_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
kafka:
  brokers: [localhost:9092]
  consumer_group: grp
  security_protocol: PLAINTEXT
  topics:
    publish: ["  "]
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
    assert!(err.contains("publish must not contain empty"), "got: {err}");
}

// 無効な SASL メカニズムを指定した場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_kafka_invalid_sasl_mechanism_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
kafka:
  brokers: [localhost:9092]
  consumer_group: grp
  security_protocol: SASL_SSL
  sasl:
    mechanism: OAUTHBEARER
    username: u
    password: p
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
    assert!(err.contains("kafka.sasl.mechanism"), "got: {err}");
}

// ===========================================================================
// End-to-end: load + env overlay + validate
// ===========================================================================

// ベース設定のロード・オーバーレイ適用・バリデーションが一連で動作することを確認する。
#[test]
fn end_to_end_load_overlay_validate() {
    let base = yaml_file(minimal_yaml());
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
    let cfg = load(
        base.path().to_str().unwrap(),
        Some(overlay.path().to_str().unwrap()),
    )
    .unwrap();
    assert_eq!(cfg.app.environment, "staging");
    assert_eq!(cfg.server.port, 9090);
    assert_eq!(cfg.app.name, "test-server"); // preserved from base
    assert!(validate(&cfg).is_ok());
}

// 全ての有効な tier 値で設定のロードとバリデーションが成功することを確認する。
#[test]
fn load_with_all_tiers() {
    for tier in &["system", "business", "service"] {
        let yaml = format!(
            r#"
app:
  name: svc
  version: "1.0"
  tier: {tier}
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log: {{ level: info, format: json }}
  trace: {{ enabled: false }}
  metrics: {{ enabled: false }}
auth:
  jwt: {{ issuer: x, audience: x }}
"#
        );
        let f = yaml_file(&yaml);
        let cfg = load(f.path().to_str().unwrap(), None).unwrap();
        assert!(validate(&cfg).is_ok(), "tier={tier} should pass validation");
    }
}

// 全ての有効な environment 値で設定のロードとバリデーションが成功することを確認する。
#[test]
fn load_with_all_environments() {
    for env in &["dev", "staging", "prod"] {
        let yaml = format!(
            r#"
app:
  name: svc
  version: "1.0"
  tier: system
  environment: {env}
server:
  host: "0.0.0.0"
  port: 8080
observability:
  log: {{ level: info, format: json }}
  trace: {{ enabled: false }}
  metrics: {{ enabled: false }}
auth:
  jwt: {{ issuer: x, audience: x }}
"#
        );
        let f = yaml_file(&yaml);
        let cfg = load(f.path().to_str().unwrap(), None).unwrap();
        assert!(
            validate(&cfg).is_ok(),
            "environment={env} should pass validation"
        );
    }
}

// ===========================================================================
// Validation: OIDC edge cases
// ===========================================================================

// OIDC スコープリストに空文字が含まれる場合にバリデーションエラーが返されることを確認する。
#[test]
fn validate_oidc_empty_scope_value_rejected() {
    let f = yaml_file(
        r#"
app:
  name: test
  version: "1.0"
  tier: system
  environment: dev
server:
  host: "0.0.0.0"
  port: 8080
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
    scopes: ["openid", "  "]
    jwks_uri: http://localhost/jwks
"#,
    );
    let cfg = load(f.path().to_str().unwrap(), None).unwrap();
    let err = validate(&cfg).unwrap_err().to_string();
    assert!(
        err.contains("scopes must not contain empty"),
        "got: {err}"
    );
}

// ===========================================================================
// Config Clone and Debug
// ===========================================================================

// 設定構造体が Clone と Debug トレイトを正しく実装していることを確認する。
#[test]
fn config_is_cloneable_and_debuggable() {
    let cfg = valid_config();
    let cloned = cfg.clone();
    assert_eq!(cloned.app.name, cfg.app.name);
    // Debug must not panic
    let debug_str = format!("{:?}", cfg);
    assert!(debug_str.contains("test-server"));
}
