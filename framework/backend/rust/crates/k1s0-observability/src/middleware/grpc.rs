//! gRPC インターセプタ
//!
//! gRPC リクエストの観測性を提供する。

use crate::config::ObservabilityConfig;
use crate::context::RequestContext;
use crate::log_fields::{LogEntry, LogLevel};
use crate::logging::RequestLog;
use crate::metrics::MetricLabels;

/// gRPC リクエスト情報
#[derive(Debug, Clone)]
pub struct GrpcRequestInfo {
    /// gRPC サービス名
    pub service: String,
    /// gRPC メソッド名
    pub method: String,
    /// メタデータ（ヘッダ）
    pub metadata: Vec<(String, String)>,
}

impl GrpcRequestInfo {
    /// 新しいリクエスト情報を作成
    pub fn new(service: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            method: method.into(),
            metadata: Vec::new(),
        }
    }

    /// フルメソッド名を取得（`/service/method` 形式）
    pub fn full_method(&self) -> String {
        format!("/{}/{}", self.service, self.method)
    }

    /// メタデータを追加
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.push((key.into(), value.into()));
        self
    }

    /// メタデータから値を取得
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

/// gRPC レスポンス情報
#[derive(Debug, Clone)]
pub struct GrpcResponseInfo {
    /// gRPC ステータスコード（0 = OK）
    pub status_code: i32,
    /// ステータス名
    pub status_name: String,
    /// エラーの種類（エラーの場合）
    pub error_kind: Option<String>,
    /// エラーコード（エラーの場合）
    pub error_code: Option<String>,
}

impl GrpcResponseInfo {
    /// 新しいレスポンス情報を作成
    pub fn new(status_code: i32, status_name: impl Into<String>) -> Self {
        Self {
            status_code,
            status_name: status_name.into(),
            error_kind: None,
            error_code: None,
        }
    }

    /// 成功レスポンスを作成
    pub fn ok() -> Self {
        Self::new(0, "OK")
    }

    /// 成功かどうか
    pub fn is_success(&self) -> bool {
        self.status_code == 0
    }

    /// エラー情報を設定
    pub fn with_error(mut self, kind: impl Into<String>, code: impl Into<String>) -> Self {
        self.error_kind = Some(kind.into());
        self.error_code = Some(code.into());
        self
    }

    /// gRPC ステータスコードからレスポンスを作成
    pub fn from_status_code(code: i32) -> Self {
        let name = match code {
            0 => "OK",
            1 => "CANCELLED",
            2 => "UNKNOWN",
            3 => "INVALID_ARGUMENT",
            4 => "DEADLINE_EXCEEDED",
            5 => "NOT_FOUND",
            6 => "ALREADY_EXISTS",
            7 => "PERMISSION_DENIED",
            8 => "RESOURCE_EXHAUSTED",
            9 => "FAILED_PRECONDITION",
            10 => "ABORTED",
            11 => "OUT_OF_RANGE",
            12 => "UNIMPLEMENTED",
            13 => "INTERNAL",
            14 => "UNAVAILABLE",
            15 => "DATA_LOSS",
            16 => "UNAUTHENTICATED",
            _ => "UNKNOWN",
        };
        Self::new(code, name)
    }
}

/// gRPC 観測性
///
/// gRPC リクエストのログ、メトリクス、トレースを統合する。
#[derive(Debug, Clone)]
pub struct GrpcObservability {
    service_name: String,
    service_env: String,
}

impl GrpcObservability {
    /// ObservabilityConfig から作成
    pub fn from_config(config: &ObservabilityConfig) -> Self {
        Self {
            service_name: config.service_name().to_string(),
            service_env: config.env().to_string(),
        }
    }

    /// リクエストコンテキストを作成または取得
    ///
    /// metadata から traceparent を取得できれば引き継ぎ、なければ新規作成。
    pub fn extract_or_create_context(&self, request: &GrpcRequestInfo) -> RequestContext {
        request
            .get_metadata("traceparent")
            .and_then(RequestContext::from_traceparent)
            .unwrap_or_else(RequestContext::new)
    }

    /// リクエスト完了時の観測性出力を生成
    pub fn on_request_complete(
        &self,
        ctx: &RequestContext,
        request: &GrpcRequestInfo,
        response: &GrpcResponseInfo,
        latency_ms: f64,
    ) -> GrpcObservabilityOutput {
        // ログレベルの決定
        let log_level = if response.is_success() {
            LogLevel::Info
        } else if response.status_code == 3 || response.status_code == 5 {
            // INVALID_ARGUMENT, NOT_FOUND はクライアント起因
            LogLevel::Warn
        } else {
            LogLevel::Error
        };

        // ログメッセージ
        let message = format!(
            "{} {} {:.2}ms",
            request.full_method(),
            response.status_name,
            latency_ms
        );

        // ログエントリ
        let mut entry = LogEntry::new(log_level, &message);
        entry.service_name = Some(self.service_name.clone());
        entry.service_env = Some(self.service_env.clone());
        entry.trace_id = Some(ctx.trace_id().to_string());
        entry.span_id = Some(ctx.span_id().to_string());
        entry.request_id = Some(ctx.request_id().to_string());

        let log = RequestLog::new(entry)
            .with_grpc(&request.service, &request.method, response.status_code)
            .with_latency(latency_ms);

        // メトリクスラベル
        let labels = MetricLabels::new()
            .service(&self.service_name)
            .env(&self.service_env)
            .grpc(&request.service, &request.method, &response.status_name);

        GrpcObservabilityOutput {
            log,
            labels,
            log_level,
            latency_ms,
            success: response.is_success(),
            request: request.clone(),
            response: response.clone(),
        }
    }
}

/// gRPC 観測性出力
#[derive(Debug)]
pub struct GrpcObservabilityOutput {
    /// ログ
    pub log: RequestLog,
    /// メトリクスラベル
    pub labels: MetricLabels,
    /// ログレベル
    pub log_level: LogLevel,
    /// レイテンシ
    pub latency_ms: f64,
    /// 成功かどうか
    pub success: bool,
    /// リクエスト情報
    pub request: GrpcRequestInfo,
    /// レスポンス情報
    pub response: GrpcResponseInfo,
}

impl GrpcObservabilityOutput {
    /// JSON ログを出力
    pub fn log_json(&self) -> Result<String, serde_json::Error> {
        self.log.to_json()
    }
}

/// gRPC メタデータキー
pub struct GrpcMetadataKeys;

impl GrpcMetadataKeys {
    /// traceparent（W3C Trace Context）
    pub const TRACEPARENT: &'static str = "traceparent";
    /// tracestate（W3C Trace Context）
    pub const TRACESTATE: &'static str = "tracestate";
    /// リクエスト ID
    pub const X_REQUEST_ID: &'static str = "x-request-id";
    /// エラーコード
    pub const X_ERROR_CODE: &'static str = "x-error-code";
    /// エラー詳細
    pub const X_ERROR_DETAILS_BIN: &'static str = "x-error-details-bin";
    /// テナント ID
    pub const X_TENANT_ID: &'static str = "x-tenant-id";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_request_info() {
        let info = GrpcRequestInfo::new("UserService", "GetUser")
            .with_metadata("traceparent", "00-abc-def-01");

        assert_eq!(info.service, "UserService");
        assert_eq!(info.method, "GetUser");
        assert_eq!(info.full_method(), "/UserService/GetUser");
        assert_eq!(info.get_metadata("traceparent"), Some("00-abc-def-01"));
    }

    #[test]
    fn test_grpc_response_info() {
        let ok = GrpcResponseInfo::ok();
        assert!(ok.is_success());
        assert_eq!(ok.status_code, 0);

        let not_found = GrpcResponseInfo::from_status_code(5);
        assert!(!not_found.is_success());
        assert_eq!(not_found.status_name, "NOT_FOUND");
    }

    #[test]
    fn test_grpc_observability() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let obs = GrpcObservability::from_config(&config);
        let ctx = RequestContext::new();
        let request = GrpcRequestInfo::new("UserService", "GetUser");
        let response = GrpcResponseInfo::ok();

        let output = obs.on_request_complete(&ctx, &request, &response, 15.5);

        assert!(output.success);
        assert_eq!(output.log_level, LogLevel::Info);
        assert_eq!(output.latency_ms, 15.5);
    }

    #[test]
    fn test_grpc_observability_error() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let obs = GrpcObservability::from_config(&config);
        let ctx = RequestContext::new();
        let request = GrpcRequestInfo::new("UserService", "GetUser");
        let response = GrpcResponseInfo::from_status_code(13) // INTERNAL
            .with_error("INTERNAL", "INTERNAL_ERROR");

        let output = obs.on_request_complete(&ctx, &request, &response, 100.0);

        assert!(!output.success);
        assert_eq!(output.log_level, LogLevel::Error);
    }

    #[test]
    fn test_grpc_observability_client_error() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let obs = GrpcObservability::from_config(&config);
        let ctx = RequestContext::new();
        let request = GrpcRequestInfo::new("UserService", "GetUser");
        let response = GrpcResponseInfo::from_status_code(5); // NOT_FOUND

        let output = obs.on_request_complete(&ctx, &request, &response, 50.0);

        assert!(!output.success);
        assert_eq!(output.log_level, LogLevel::Warn);
    }

    #[test]
    fn test_log_json() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let obs = GrpcObservability::from_config(&config);
        let ctx = RequestContext::new();
        let request = GrpcRequestInfo::new("UserService", "GetUser");
        let response = GrpcResponseInfo::ok();

        let output = obs.on_request_complete(&ctx, &request, &response, 15.5);
        let json = output.log_json().unwrap();

        assert!(json.contains("test-service"));
        assert!(json.contains("UserService"));
        assert!(json.contains("GetUser"));
    }

    #[test]
    fn test_grpc_metadata_keys() {
        assert_eq!(GrpcMetadataKeys::TRACEPARENT, "traceparent");
        assert_eq!(GrpcMetadataKeys::X_REQUEST_ID, "x-request-id");
    }
}
