//! HTTP ミドルウェア
//!
//! HTTP リクエストの観測性を提供する。

use crate::config::ObservabilityConfig;
use crate::context::RequestContext;
use crate::log_fields::{LogEntry, LogLevel};
use crate::logging::RequestLog;
use crate::metrics::MetricLabels;

/// HTTP リクエスト情報
#[derive(Debug, Clone)]
pub struct HttpRequestInfo {
    /// HTTP メソッド
    pub method: String,
    /// リクエストパス
    pub path: String,
    /// ホスト
    pub host: Option<String>,
    /// User-Agent
    pub user_agent: Option<String>,
    /// リクエストサイズ（バイト）
    pub content_length: Option<u64>,
}

impl HttpRequestInfo {
    /// 新しいリクエスト情報を作成
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            host: None,
            user_agent: None,
            content_length: None,
        }
    }

    /// ホストを設定
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// User-Agent を設定
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Content-Length を設定
    pub fn with_content_length(mut self, length: u64) -> Self {
        self.content_length = Some(length);
        self
    }
}

/// HTTP レスポンス情報
#[derive(Debug, Clone)]
pub struct HttpResponseInfo {
    /// ステータスコード
    pub status_code: u16,
    /// レスポンスサイズ（バイト）
    pub content_length: Option<u64>,
    /// エラーの種類（エラーの場合）
    pub error_kind: Option<String>,
    /// エラーコード（エラーの場合）
    pub error_code: Option<String>,
}

impl HttpResponseInfo {
    /// 新しいレスポンス情報を作成
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            content_length: None,
            error_kind: None,
            error_code: None,
        }
    }

    /// 成功かどうか
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 400
    }

    /// クライアントエラーかどうか
    pub fn is_client_error(&self) -> bool {
        self.status_code >= 400 && self.status_code < 500
    }

    /// サーバーエラーかどうか
    pub fn is_server_error(&self) -> bool {
        self.status_code >= 500
    }

    /// Content-Length を設定
    pub fn with_content_length(mut self, length: u64) -> Self {
        self.content_length = Some(length);
        self
    }

    /// エラー情報を設定
    pub fn with_error(mut self, kind: impl Into<String>, code: impl Into<String>) -> Self {
        self.error_kind = Some(kind.into());
        self.error_code = Some(code.into());
        self
    }
}

/// HTTP 観測性
///
/// HTTP リクエストのログ、メトリクス、トレースを統合する。
#[derive(Debug, Clone)]
pub struct HttpObservability {
    service_name: String,
    service_env: String,
}

impl HttpObservability {
    /// ObservabilityConfig から作成
    pub fn from_config(config: &ObservabilityConfig) -> Self {
        Self {
            service_name: config.service_name().to_string(),
            service_env: config.env().to_string(),
        }
    }

    /// リクエストコンテキストを作成または取得
    ///
    /// traceparent ヘッダがあれば引き継ぎ、なければ新規作成。
    pub fn extract_or_create_context(&self, traceparent: Option<&str>) -> RequestContext {
        traceparent
            .and_then(RequestContext::from_traceparent)
            .unwrap_or_else(RequestContext::new)
    }

    /// リクエスト完了時の観測性出力を生成
    pub fn on_request_complete(
        &self,
        ctx: &RequestContext,
        request: &HttpRequestInfo,
        response: &HttpResponseInfo,
        latency_ms: f64,
    ) -> HttpObservabilityOutput {
        // ログレベルの決定
        let log_level = if response.is_server_error() {
            LogLevel::Error
        } else if response.is_client_error() {
            LogLevel::Warn
        } else {
            LogLevel::Info
        };

        // ログメッセージ
        let message = format!(
            "{} {} {} {:.2}ms",
            request.method, request.path, response.status_code, latency_ms
        );

        // ログエントリ
        let mut entry = LogEntry::new(log_level, &message);
        entry.service_name = Some(self.service_name.clone());
        entry.service_env = Some(self.service_env.clone());
        entry.trace_id = Some(ctx.trace_id().to_string());
        entry.span_id = Some(ctx.span_id().to_string());
        entry.request_id = Some(ctx.request_id().to_string());

        let log = RequestLog::new(entry)
            .with_http(&request.method, &request.path, response.status_code)
            .with_latency(latency_ms);

        // メトリクスラベル
        let labels = MetricLabels::new()
            .service(&self.service_name)
            .env(&self.service_env)
            .http(&request.method, &request.path, response.status_code);

        HttpObservabilityOutput {
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

/// HTTP 観測性出力
#[derive(Debug)]
pub struct HttpObservabilityOutput {
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
    pub request: HttpRequestInfo,
    /// レスポンス情報
    pub response: HttpResponseInfo,
}

impl HttpObservabilityOutput {
    /// JSON ログを出力
    pub fn log_json(&self) -> Result<String, serde_json::Error> {
        self.log.to_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_request_info() {
        let info = HttpRequestInfo::new("GET", "/api/users")
            .with_host("localhost")
            .with_content_length(100);

        assert_eq!(info.method, "GET");
        assert_eq!(info.path, "/api/users");
        assert_eq!(info.host, Some("localhost".to_string()));
        assert_eq!(info.content_length, Some(100));
    }

    #[test]
    fn test_http_response_info() {
        let success = HttpResponseInfo::new(200);
        assert!(success.is_success());
        assert!(!success.is_client_error());
        assert!(!success.is_server_error());

        let client_error = HttpResponseInfo::new(404);
        assert!(!client_error.is_success());
        assert!(client_error.is_client_error());

        let server_error = HttpResponseInfo::new(500);
        assert!(server_error.is_server_error());
    }

    #[test]
    fn test_http_observability() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let obs = HttpObservability::from_config(&config);
        let ctx = RequestContext::new();
        let request = HttpRequestInfo::new("GET", "/api/users");
        let response = HttpResponseInfo::new(200);

        let output = obs.on_request_complete(&ctx, &request, &response, 42.5);

        assert!(output.success);
        assert_eq!(output.log_level, LogLevel::Info);
        assert_eq!(output.latency_ms, 42.5);
    }

    #[test]
    fn test_http_observability_error() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let obs = HttpObservability::from_config(&config);
        let ctx = RequestContext::new();
        let request = HttpRequestInfo::new("GET", "/api/users/123");
        let response = HttpResponseInfo::new(500)
            .with_error("INTERNAL", "INTERNAL_ERROR");

        let output = obs.on_request_complete(&ctx, &request, &response, 100.0);

        assert!(!output.success);
        assert_eq!(output.log_level, LogLevel::Error);
    }

    #[test]
    fn test_extract_or_create_context() {
        let config = ObservabilityConfig::builder()
            .service_name("test")
            .env("dev")
            .build()
            .unwrap();

        let obs = HttpObservability::from_config(&config);

        // traceparent なし
        let ctx1 = obs.extract_or_create_context(None);
        assert!(!ctx1.trace_id().is_empty());

        // traceparent あり
        let traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let ctx2 = obs.extract_or_create_context(Some(traceparent));
        assert_eq!(ctx2.trace_id(), "0af7651916cd43dd8448eb211c80319c");
    }

    #[test]
    fn test_log_json() {
        let config = ObservabilityConfig::builder()
            .service_name("test-service")
            .env("dev")
            .build()
            .unwrap();

        let obs = HttpObservability::from_config(&config);
        let ctx = RequestContext::new();
        let request = HttpRequestInfo::new("GET", "/api/users");
        let response = HttpResponseInfo::new(200);

        let output = obs.on_request_complete(&ctx, &request, &response, 42.5);
        let json = output.log_json().unwrap();

        assert!(json.contains("test-service"));
        assert!(json.contains("GET"));
        assert!(json.contains("/api/users"));
        assert!(json.contains("200"));
    }
}
