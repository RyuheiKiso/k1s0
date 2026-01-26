//! ミドルウェア/インターセプタ基盤
//!
//! REST/gRPC のエラー変換をミドルウェアとして提供するための基盤。
//! サービスが独自変換を持たない方針をテンプレで固定。

use crate::application::AppError;
use crate::logging::{ErrorLog, GrpcErrorLog, HttpErrorLog, LogLevel};
use crate::metrics::ErrorMetricLabels;

/// エラーレスポンスのコンテンツタイプ
pub struct ContentType;

impl ContentType {
    /// RFC 7807 Problem Details
    pub const PROBLEM_JSON: &'static str = "application/problem+json";

    /// 通常の JSON
    pub const JSON: &'static str = "application/json";
}

/// HTTP エラーレスポンス
///
/// ミドルウェアが生成する完全な HTTP エラーレスポンス。
#[derive(Debug, Clone)]
pub struct HttpErrorResponse {
    /// HTTP ステータスコード
    pub status_code: u16,
    /// コンテンツタイプ
    pub content_type: &'static str,
    /// レスポンスボディ（JSON）
    pub body: String,
    /// ログ情報
    pub log: HttpErrorLog,
    /// メトリクスラベル
    pub metric_labels: ErrorMetricLabels,
}

impl HttpErrorResponse {
    /// AppError から HTTP エラーレスポンスを作成
    ///
    /// # Arguments
    ///
    /// * `app_error` - アプリケーションエラー
    /// * `instance` - リクエストパス（Problem Details の instance）
    pub fn from_app_error(app_error: &AppError, instance: &str) -> Self {
        let http_error = app_error.to_http_error();
        let problem = http_error.to_problem_details(instance);
        let body = problem.to_json().unwrap_or_else(|_| "{}".to_string());

        let base_log = ErrorLog::from_app_error(app_error).with_http_status(http_error.status_code());
        let log = HttpErrorLog::new(base_log).with_path(instance);

        let metric_labels = ErrorMetricLabels::from_app_error(app_error)
            .with_http_status(http_error.status_code());

        Self {
            status_code: http_error.status_code(),
            content_type: ContentType::PROBLEM_JSON,
            body,
            log,
            metric_labels,
        }
    }

    /// HTTP メソッドを設定
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.log = self.log.with_method(method);
        self
    }

    /// 推奨されるログレベルを取得
    pub fn log_level(&self) -> LogLevel {
        self.log.error.log_level()
    }
}

/// gRPC エラーレスポンス
///
/// インターセプタが生成する完全な gRPC エラーレスポンス。
#[derive(Debug, Clone)]
pub struct GrpcErrorResponse {
    /// gRPC ステータスコード（数値）
    pub status_code: i32,
    /// gRPC ステータス名
    pub status_name: String,
    /// エラーメッセージ
    pub message: String,
    /// metadata 用の詳細情報（JSON）
    pub details_json: String,
    /// ログ情報
    pub log: GrpcErrorLog,
    /// メトリクスラベル
    pub metric_labels: ErrorMetricLabels,
}

impl GrpcErrorResponse {
    /// AppError から gRPC エラーレスポンスを作成
    pub fn from_app_error(app_error: &AppError) -> Self {
        let grpc_error = app_error.to_grpc_error();
        let details_json = grpc_error
            .to_details_json()
            .unwrap_or_else(|_| "{}".to_string());

        let base_log =
            ErrorLog::from_app_error(app_error).with_grpc_status(grpc_error.status_code());
        let log = GrpcErrorLog::new(base_log);

        let metric_labels = ErrorMetricLabels::from_app_error(app_error)
            .with_grpc_status(grpc_error.status_code());

        Self {
            status_code: grpc_error.status_code().as_i32(),
            status_name: grpc_error.status_code().as_str().to_string(),
            message: grpc_error.message().to_string(),
            details_json,
            log,
            metric_labels,
        }
    }

    /// gRPC サービス名を設定
    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.log = self.log.with_service(service);
        self
    }

    /// gRPC メソッド名を設定
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.log = self.log.with_method(method);
        self
    }

    /// 推奨されるログレベルを取得
    pub fn log_level(&self) -> LogLevel {
        self.log.error.log_level()
    }
}

/// gRPC metadata キー
pub struct GrpcMetadataKeys;

impl GrpcMetadataKeys {
    /// エラーコード
    pub const ERROR_CODE: &'static str = "x-error-code";
    /// エラー詳細（JSON）
    pub const ERROR_DETAILS: &'static str = "x-error-details-bin";
    /// トレース ID
    pub const TRACE_ID: &'static str = "x-trace-id";
    /// リクエスト ID
    pub const REQUEST_ID: &'static str = "x-request-id";
}

/// エラー変換の設定
#[derive(Debug, Clone)]
pub struct ErrorMappingConfig {
    /// 内部エラーの詳細を公開するか（本番は false 推奨）
    pub expose_internal_errors: bool,
    /// スタックトレースを含めるか（開発環境のみ）
    pub include_stack_trace: bool,
    /// ヒントを含めるか
    pub include_hints: bool,
}

impl Default for ErrorMappingConfig {
    fn default() -> Self {
        Self {
            expose_internal_errors: false,
            include_stack_trace: false,
            include_hints: true,
        }
    }
}

impl ErrorMappingConfig {
    /// 開発環境用の設定
    pub fn development() -> Self {
        Self {
            expose_internal_errors: true,
            include_stack_trace: true,
            include_hints: true,
        }
    }

    /// 本番環境用の設定
    pub fn production() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomainError, ErrorCode};

    #[test]
    fn test_http_error_response() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"))
            .with_trace_id("trace-abc");

        let response = HttpErrorResponse::from_app_error(&app_err, "/api/users/user-123")
            .with_method("GET");

        assert_eq!(response.status_code, 404);
        assert_eq!(response.content_type, ContentType::PROBLEM_JSON);
        assert!(response.body.contains("USER_NOT_FOUND"));
        assert!(response.body.contains("trace-abc"));
        assert_eq!(response.log.method, Some("GET".to_string()));
    }

    #[test]
    fn test_grpc_error_response() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"))
            .with_trace_id("trace-abc");

        let response = GrpcErrorResponse::from_app_error(&app_err)
            .with_service("UserService")
            .with_method("GetUser");

        assert_eq!(response.status_code, 5); // NOT_FOUND
        assert_eq!(response.status_name, "NOT_FOUND");
        assert!(response.details_json.contains("USER_NOT_FOUND"));
        assert_eq!(response.log.service, Some("UserService".to_string()));
        assert_eq!(response.log.method, Some("GetUser".to_string()));
    }

    #[test]
    fn test_log_level() {
        let internal_err = AppError::from_domain(
            DomainError::internal("test"),
            ErrorCode::internal(),
        );
        let response = HttpErrorResponse::from_app_error(&internal_err, "/test");
        assert_eq!(response.log_level(), LogLevel::Error);

        let not_found_err = AppError::from_domain(
            DomainError::not_found("X", "1"),
            ErrorCode::not_found(),
        );
        let response = HttpErrorResponse::from_app_error(&not_found_err, "/test");
        assert_eq!(response.log_level(), LogLevel::Info);
    }

    #[test]
    fn test_error_mapping_config() {
        let dev = ErrorMappingConfig::development();
        assert!(dev.expose_internal_errors);
        assert!(dev.include_stack_trace);

        let prod = ErrorMappingConfig::production();
        assert!(!prod.expose_internal_errors);
        assert!(!prod.include_stack_trace);
    }

    #[test]
    fn test_content_type() {
        assert_eq!(ContentType::PROBLEM_JSON, "application/problem+json");
    }

    #[test]
    fn test_grpc_metadata_keys() {
        assert_eq!(GrpcMetadataKeys::ERROR_CODE, "x-error-code");
        assert_eq!(GrpcMetadataKeys::TRACE_ID, "x-trace-id");
    }
}
