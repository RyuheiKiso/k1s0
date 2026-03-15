#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::logger::parse_log_level;
    use crate::TelemetryConfig;
    use crate::{trace_grpc_call, trace_request};

    // TelemetryConfig が指定した全フィールドで正しく生成されることを確認する。
    #[test]
    fn test_telemetry_config_creation() {
        let cfg = TelemetryConfig {
            service_name: "test-service".to_string(),
            version: "1.0.0".to_string(),
            tier: "system".to_string(),
            environment: "dev".to_string(),
            trace_endpoint: None,
            sample_rate: 1.0,
            log_level: "debug".to_string(),
            log_format: "json".to_string(),
        };

        assert_eq!(cfg.service_name, "test-service");
        assert_eq!(cfg.version, "1.0.0");
        assert_eq!(cfg.tier, "system");
        assert_eq!(cfg.environment, "dev");
        assert!(cfg.trace_endpoint.is_none());
        assert_eq!(cfg.sample_rate, 1.0);
        assert_eq!(cfg.log_level, "debug");
        assert_eq!(cfg.log_format, "json");
    }

    // トレースエンドポイントを含む TelemetryConfig が正しく生成されることを確認する。
    #[test]
    fn test_telemetry_config_with_trace_endpoint() {
        let cfg = TelemetryConfig {
            service_name: "order-server".to_string(),
            version: "2.0.0".to_string(),
            tier: "service".to_string(),
            environment: "prod".to_string(),
            trace_endpoint: Some("otel-collector:4317".to_string()),
            sample_rate: 0.1,
            log_level: "warn".to_string(),
            log_format: "text".to_string(),
        };

        assert_eq!(cfg.service_name, "order-server");
        assert_eq!(cfg.trace_endpoint, Some("otel-collector:4317".to_string()));
        assert_eq!(cfg.sample_rate, 0.1);
    }

    // "debug" 文字列が DEBUG ログレベルに変換されることを確認する。
    #[test]
    fn test_parse_log_level_debug() {
        assert_eq!(parse_log_level("debug"), tracing::Level::DEBUG);
    }

    // "info" 文字列が INFO ログレベルに変換されることを確認する。
    #[test]
    fn test_parse_log_level_info() {
        assert_eq!(parse_log_level("info"), tracing::Level::INFO);
    }

    // "warn" 文字列が WARN ログレベルに変換されることを確認する。
    #[test]
    fn test_parse_log_level_warn() {
        assert_eq!(parse_log_level("warn"), tracing::Level::WARN);
    }

    // "error" 文字列が ERROR ログレベルに変換されることを確認する。
    #[test]
    fn test_parse_log_level_error() {
        assert_eq!(parse_log_level("error"), tracing::Level::ERROR);
    }

    // 未知の文字列や空文字列が INFO ログレベルにフォールバックすることを確認する。
    #[test]
    fn test_parse_log_level_unknown_defaults_to_info() {
        assert_eq!(parse_log_level("unknown"), tracing::Level::INFO);
        assert_eq!(parse_log_level(""), tracing::Level::INFO);
    }

    // trace_request! マクロがブロックの戻り値をそのまま返すことを確認する。
    #[test]
    fn test_trace_request_macro() {
        let result = trace_request!("GET", "/health", { 42 });
        assert_eq!(result, 42);
    }

    // trace_grpc_call! マクロがブロックの戻り値をそのまま返すことを確認する。
    #[test]
    fn test_trace_grpc_call_macro() {
        let result = trace_grpc_call!("OrderService.CreateOrder", { "ok" });
        assert_eq!(result, "ok");
    }

    // shutdown() がパニックなしに実行できることを確認する。
    #[test]
    fn test_shutdown_does_not_panic() {
        crate::shutdown();
    }

    // --- metrics tests ---

    use crate::metrics::Metrics;

    // Metrics が正常に初期化されすべてのフィールドが Some になることを確認する。
    #[test]
    fn test_metrics_initialization() {
        let m = Metrics::new("test-service");
        // Metrics が正常に初期化されることを確認
        assert!(m.http_requests_total.is_some());
        assert!(m.http_request_duration.is_some());
        assert!(m.grpc_handled_total.is_some());
        assert!(m.grpc_handling_duration.is_some());
    }

    // record_http_request がパニックなしに HTTP リクエストカウンタを記録することを確認する。
    #[test]
    fn test_record_http_request() {
        let m = Metrics::new("test-http");
        // パニックせずにカウンタが増加すること
        m.record_http_request("GET", "/api/v1/orders", "200");
        m.record_http_request("POST", "/api/v1/orders", "201");
        m.record_http_request("GET", "/api/v1/orders", "500");
    }

    // record_grpc_request がパニックなしに gRPC リクエストカウンタを記録することを確認する。
    #[test]
    fn test_record_grpc_request() {
        let m = Metrics::new("test-grpc");
        // パニックせずに gRPC カウンタが増加すること
        m.record_grpc_request("OrderService", "CreateOrder", "OK");
        m.record_grpc_request("OrderService", "GetOrder", "NOT_FOUND");
    }

    // record_http_duration がパニックなしに HTTP レイテンシヒストグラムを記録することを確認する。
    #[test]
    fn test_record_http_duration() {
        let m = Metrics::new("test-duration");
        // パニックせずにヒストグラムが記録されること
        m.record_http_duration("GET", "/api/v1/orders", 0.05);
        m.record_http_duration("POST", "/api/v1/orders", 1.2);
    }

    // record_grpc_duration がパニックなしに gRPC レイテンシヒストグラムを記録することを確認する。
    #[test]
    fn test_record_grpc_duration() {
        let m = Metrics::new("test-grpc-duration");
        m.record_grpc_duration("OrderService", "CreateOrder", 0.01);
        m.record_grpc_duration("OrderService", "GetOrder", 0.5);
    }

    // エラーステータスの HTTP/gRPC リクエスト記録がパニックなしに動作することを確認する。
    #[test]
    fn test_record_error_counter() {
        let m = Metrics::new("test-errors");
        // エラーステータスの HTTP リクエストを記録
        m.record_http_request("GET", "/api/v1/orders", "500");
        m.record_http_request("POST", "/api/v1/orders", "503");
        // エラーコードの gRPC リクエストを記録
        m.record_grpc_request("OrderService", "CreateOrder", "INTERNAL");
        m.record_grpc_request("OrderService", "CreateOrder", "UNAVAILABLE");
    }

    // gather_metrics が Prometheus テキスト形式を含む文字列を返すことを確認する。
    #[test]
    fn test_metrics_handler_returns_text() {
        let _m = Metrics::new("test-handler");
        _m.record_http_request("GET", "/health", "200");
        let output = _m.gather_metrics();
        // Prometheus テキストフォーマットが含まれることを確認
        assert!(output.contains("http_requests_total"));
    }

    // --- middleware tests ---

    use crate::middleware::{GrpcInterceptor, TelemetryMiddleware};
    use std::sync::Arc;

    // TelemetryMiddleware が Metrics を受け取り正常に生成されることを確認する。
    #[test]
    fn test_telemetry_middleware_creation() {
        let metrics = Arc::new(Metrics::new("test-mw"));
        let mw = TelemetryMiddleware::new(metrics);
        // 正常に生成されることを確認
        assert!(mw.metrics.http_requests_total.is_some());
    }

    // TelemetryMiddleware 経由でのメトリクス記録が gather_metrics に反映されることを確認する。
    #[test]
    fn test_telemetry_middleware_record_request() {
        let metrics = Arc::new(Metrics::new("test-mw-record"));
        let mw = TelemetryMiddleware::new(metrics);
        // middleware 経由でメトリクス記録ができること
        mw.on_request("GET", "/api/v1/orders");
        mw.on_response("GET", "/api/v1/orders", 200, 0.05);

        let output = mw.metrics.gather_metrics();
        assert!(output.contains("http_requests_total"));
        assert!(output.contains("http_request_duration_seconds"));
    }

    // エラーステータス (500) を on_response に渡した場合にメトリクスに記録されることを確認する。
    #[test]
    fn test_telemetry_middleware_error_status() {
        let metrics = Arc::new(Metrics::new("test-mw-err"));
        let mw = TelemetryMiddleware::new(metrics);
        mw.on_response("POST", "/api/v1/orders", 500, 1.2);

        let output = mw.metrics.gather_metrics();
        assert!(output.contains("500"));
    }

    // GrpcInterceptor が正常に生成されることを確認する。
    #[test]
    fn test_grpc_interceptor_creation() {
        let metrics = Arc::new(Metrics::new("test-grpc-int"));
        let interceptor = GrpcInterceptor::new(metrics);
        assert!(interceptor.metrics.grpc_handled_total.is_some());
    }

    // GrpcInterceptor の on_response が gRPC メトリクスを gather_metrics に反映することを確認する。
    #[test]
    fn test_grpc_interceptor_record_call() {
        let metrics = Arc::new(Metrics::new("test-grpc-call"));
        let interceptor = GrpcInterceptor::new(metrics);
        interceptor.on_response("OrderService", "CreateOrder", "OK", 0.01);

        let output = interceptor.metrics.gather_metrics();
        assert!(output.contains("grpc_server_handled_total"));
        assert!(output.contains("grpc_server_handling_seconds"));
    }

    // gRPC エラーレスポンスが INTERNAL ステータスとしてメトリクスに記録されることを確認する。
    #[test]
    fn test_grpc_interceptor_error_call() {
        let metrics = Arc::new(Metrics::new("test-grpc-err"));
        let interceptor = GrpcInterceptor::new(metrics);
        interceptor.on_response("OrderService", "CreateOrder", "INTERNAL", 0.5);

        let output = interceptor.metrics.gather_metrics();
        assert!(output.contains("INTERNAL"));
    }
}
