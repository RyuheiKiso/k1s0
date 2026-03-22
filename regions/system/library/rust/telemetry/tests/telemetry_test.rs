//! External integration tests for k1s0-telemetry.
//!
//! Inline tests (src/tests.rs) already cover:
//! - TelemetryConfig creation with/without trace endpoint
//! - parse_log_level variants
//! - trace_request! / trace_grpc_call! macros
//! - Metrics initialization, recording, gather
//! - TelemetryMiddleware / GrpcInterceptor creation and recording
//! - shutdown
//!
//! These external tests focus on:
//! - ErrorSeverity classification edge cases (mixed keywords, chaining)
//! - Metrics RED pattern end-to-end (record multiple + gather + verify output)
//! - Metrics DB/Kafka/Cache recording
//! - TelemetryConfig field boundary values
#![allow(clippy::unwrap_used)]

use k1s0_telemetry::error_classifier::{classify_error, ErrorSeverity};
use k1s0_telemetry::logger::parse_log_level;
use k1s0_telemetry::metrics::Metrics;
use k1s0_telemetry::TelemetryConfig;

// ---------------------------------------------------------------------------
// Helper error type for error classification tests
// ---------------------------------------------------------------------------
#[derive(Debug)]
struct SimpleError(String);

impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SimpleError {}

// ===========================================================================
// ErrorSeverity classification — complementary edge cases
// ===========================================================================

// transient と permanent 両方のキーワードを含む場合に最初にマッチした種別が返ることを確認する。
#[test]
fn classify_mixed_transient_and_permanent_keywords_first_match_wins() {
    // "connection" appears first -> Transient
    let err = SimpleError("connection error: resource not found".to_string());
    assert_eq!(classify_error(&err), ErrorSeverity::Transient);
}

// 長いメッセージに timeout キーワードが含まれる場合に Transient と分類されることを確認する。
#[test]
fn classify_transient_timeout_in_longer_message() {
    let err = SimpleError("operation failed with timeout after waiting 30 seconds".to_string());
    assert_eq!(classify_error(&err), ErrorSeverity::Transient);
}

// 長いメッセージに unauthorized キーワードが含まれる場合に Permanent と分類されることを確認する。
#[test]
fn classify_permanent_unauthorized_in_longer_message() {
    let err = SimpleError("user attempted unauthorized access to admin resource".to_string());
    assert_eq!(classify_error(&err), ErrorSeverity::Permanent);
}

// 分類キーワードを含まないエラーが Unknown と分類されることを確認する。
#[test]
fn classify_unknown_for_generic_error() {
    let err = SimpleError("internal server error".to_string());
    assert_eq!(classify_error(&err), ErrorSeverity::Unknown);
}

// 空のエラーメッセージが Unknown と分類されることを確認する。
#[test]
fn classify_empty_error_message() {
    let err = SimpleError(String::new());
    assert_eq!(classify_error(&err), ErrorSeverity::Unknown);
}

// ErrorSeverity が Debug/Clone/PartialEq を正しく実装していることを確認する。
#[test]
fn error_severity_debug_and_clone() {
    let s = ErrorSeverity::Transient;
    let cloned = s;
    assert_eq!(format!("{:?}", cloned), "Transient");

    assert_eq!(ErrorSeverity::Permanent, ErrorSeverity::Permanent);
    assert_ne!(ErrorSeverity::Transient, ErrorSeverity::Permanent);
}

// ===========================================================================
// TelemetryConfig — boundary values
// ===========================================================================

// sample_rate に 0.0 を設定した TelemetryConfig が正しく保持されることを確認する。
#[test]
fn telemetry_config_zero_sample_rate() {
    let cfg = TelemetryConfig {
        service_name: "svc".to_string(),
        version: "0.1.0".to_string(),
        tier: "system".to_string(),
        environment: "dev".to_string(),
        trace_endpoint: None,
        sample_rate: 0.0,
        log_level: "info".to_string(),
        log_format: "json".to_string(),
    };
    assert_eq!(cfg.sample_rate, 0.0);
}

// log_format に "text" を設定した TelemetryConfig が正しく保持されることを確認する。
#[test]
fn telemetry_config_text_log_format() {
    let cfg = TelemetryConfig {
        service_name: "svc".to_string(),
        version: "1.0.0".to_string(),
        tier: "business".to_string(),
        environment: "staging".to_string(),
        trace_endpoint: Some("localhost:4317".to_string()),
        sample_rate: 0.5,
        log_level: "warn".to_string(),
        log_format: "text".to_string(),
    };
    assert_eq!(cfg.log_format, "text");
    assert_eq!(cfg.sample_rate, 0.5);
}

// ===========================================================================
// parse_log_level — boundary (not duplicating inline tests)
// ===========================================================================

// parse_log_level が大文字入力を受け付けず INFO にフォールバックすることを確認する。
#[test]
fn parse_log_level_case_sensitive() {
    // Uppercase not matched — falls back to INFO
    assert_eq!(parse_log_level("DEBUG"), tracing::Level::INFO);
    assert_eq!(parse_log_level("WARN"), tracing::Level::INFO);
}

// ===========================================================================
// Metrics — RED pattern end-to-end
// ===========================================================================

// HTTP の RED パターン（Rate・Error・Duration）がメトリクスに正しく記録されることを確認する。
#[test]
fn metrics_http_red_pattern() {
    let m = Metrics::new("red-test");

    // Rate: multiple requests
    m.record_http_request("GET", "/orders", "200");
    m.record_http_request("GET", "/orders", "200");
    m.record_http_request("POST", "/orders", "201");

    // Errors
    m.record_http_request("GET", "/orders", "500");
    m.record_http_request("POST", "/orders", "503");

    // Duration
    m.record_http_duration("GET", "/orders", 0.012);
    m.record_http_duration("POST", "/orders", 0.250);

    let output = m.gather_metrics();
    assert!(output.contains("http_requests_total"));
    assert!(output.contains("http_request_duration_seconds"));
    assert!(output.contains("200"));
    assert!(output.contains("500"));
}

// gRPC の RED パターンがメトリクスに正しく記録されることを確認する。
#[test]
fn metrics_grpc_red_pattern() {
    let m = Metrics::new("grpc-red-test");

    m.record_grpc_request("UserService", "GetUser", "OK");
    m.record_grpc_request("UserService", "GetUser", "NOT_FOUND");
    m.record_grpc_request("UserService", "CreateUser", "INTERNAL");

    m.record_grpc_duration("UserService", "GetUser", 0.005);
    m.record_grpc_duration("UserService", "CreateUser", 0.100);

    let output = m.gather_metrics();
    assert!(output.contains("grpc_server_handled_total"));
    assert!(output.contains("grpc_server_handling_seconds"));
    assert!(output.contains("OK"));
    assert!(output.contains("NOT_FOUND"));
}

// ===========================================================================
// Metrics — DB, Kafka, Cache (not covered in inline tests)
// ===========================================================================

// DB クエリのレイテンシが db_query_duration_seconds メトリクスに記録されることを確認する。
#[test]
fn metrics_db_query_duration() {
    let m = Metrics::new("db-test");
    m.record_db_query_duration("find_user", "users", 0.003);
    m.record_db_query_duration("list_orders", "orders", 0.045);

    let output = m.gather_metrics();
    assert!(output.contains("db_query_duration_seconds"));
}

// Kafka のメッセージ送受信カウンタがメトリクスに記録されることを確認する。
#[test]
fn metrics_kafka_produce_consume() {
    let m = Metrics::new("kafka-test");
    m.record_kafka_message_produced("orders.created");
    m.record_kafka_message_produced("orders.created");
    m.record_kafka_message_consumed("tasks.completed", "task-server.default");

    let output = m.gather_metrics();
    assert!(output.contains("kafka_messages_produced_total"));
    assert!(output.contains("kafka_messages_consumed_total"));
}

// キャッシュヒット・ミスのカウンタがメトリクスに記録されることを確認する。
#[test]
fn metrics_cache_hit_miss() {
    let m = Metrics::new("cache-test");
    m.record_cache_hit("user-cache");
    m.record_cache_hit("user-cache");
    m.record_cache_miss("user-cache");

    let output = m.gather_metrics();
    assert!(output.contains("cache_hits_total"));
    assert!(output.contains("cache_misses_total"));
}

// 観測値なしで gather_metrics を呼び出してもパニックしないことを確認する。
#[test]
fn metrics_gather_without_observations_is_valid() {
    let m = Metrics::new("empty-test");
    let output = m.gather_metrics();
    // Prometheus only emits metrics that have been observed.
    // With no observations, the output may be empty but must not panic.
    assert!(output.is_empty() || output.contains("# HELP") || output.contains("# TYPE"));
}

// ===========================================================================
// Metrics — multiple services don't interfere
// ===========================================================================

// 異なるサービス名の Metrics インスタンスが互いに干渉しないことを確認する。
#[test]
fn metrics_separate_registries() {
    let m1 = Metrics::new("service-a");
    let m2 = Metrics::new("service-b");

    m1.record_http_request("GET", "/a", "200");
    m2.record_http_request("POST", "/b", "201");

    let out1 = m1.gather_metrics();
    let out2 = m2.gather_metrics();

    assert!(out1.contains("service-a"));
    assert!(out2.contains("service-b"));
}
