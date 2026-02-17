#[cfg(test)]
mod tests {
    use crate::logger::parse_log_level;
    use crate::TelemetryConfig;
    use crate::{trace_request, trace_grpc_call};

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
        };

        assert_eq!(cfg.service_name, "test-service");
        assert_eq!(cfg.version, "1.0.0");
        assert_eq!(cfg.tier, "system");
        assert_eq!(cfg.environment, "dev");
        assert!(cfg.trace_endpoint.is_none());
        assert_eq!(cfg.sample_rate, 1.0);
        assert_eq!(cfg.log_level, "debug");
    }

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
        };

        assert_eq!(cfg.service_name, "order-server");
        assert_eq!(cfg.trace_endpoint, Some("otel-collector:4317".to_string()));
        assert_eq!(cfg.sample_rate, 0.1);
    }

    #[test]
    fn test_parse_log_level_debug() {
        assert_eq!(parse_log_level("debug"), tracing::Level::DEBUG);
    }

    #[test]
    fn test_parse_log_level_info() {
        assert_eq!(parse_log_level("info"), tracing::Level::INFO);
    }

    #[test]
    fn test_parse_log_level_warn() {
        assert_eq!(parse_log_level("warn"), tracing::Level::WARN);
    }

    #[test]
    fn test_parse_log_level_error() {
        assert_eq!(parse_log_level("error"), tracing::Level::ERROR);
    }

    #[test]
    fn test_parse_log_level_unknown_defaults_to_info() {
        assert_eq!(parse_log_level("unknown"), tracing::Level::INFO);
        assert_eq!(parse_log_level(""), tracing::Level::INFO);
    }

    #[test]
    fn test_trace_request_macro() {
        let result = trace_request!("GET", "/health", { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_trace_grpc_call_macro() {
        let result = trace_grpc_call!("OrderService.CreateOrder", { "ok" });
        assert_eq!(result, "ok");
    }

    #[test]
    fn test_shutdown_does_not_panic() {
        crate::shutdown();
    }
}
