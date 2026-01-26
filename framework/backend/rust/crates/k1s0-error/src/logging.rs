//! エラーログ出力
//!
//! 構造化ログへのエラー出力を統一する。
//! `error.kind` / `error.code` / status code を必ず出す仕組みを提供。

use serde::Serialize;

use crate::application::AppError;
use crate::presentation::GrpcStatusCode;

/// ログ出力用のエラー情報
///
/// 構造化ログ（JSON）に含めるエラー情報を統一する。
/// 運用が `error_code` と `trace_id` だけで一次判断できる状態を実現。
#[derive(Debug, Clone, Serialize)]
pub struct ErrorLog {
    /// エラーの種類（内部分類）
    #[serde(rename = "error.kind")]
    pub error_kind: String,

    /// エラーコード（運用識別子）
    #[serde(rename = "error.code")]
    pub error_code: String,

    /// エラーメッセージ
    #[serde(rename = "error.message")]
    pub message: String,

    /// HTTP ステータスコード（REST の場合）
    #[serde(rename = "http.status_code", skip_serializing_if = "Option::is_none")]
    pub http_status_code: Option<u16>,

    /// gRPC ステータスコード（gRPC の場合）
    #[serde(rename = "grpc.status_code", skip_serializing_if = "Option::is_none")]
    pub grpc_status_code: Option<i32>,

    /// gRPC ステータス名
    #[serde(rename = "grpc.status", skip_serializing_if = "Option::is_none")]
    pub grpc_status: Option<String>,

    /// トレース ID
    #[serde(rename = "trace.id", skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// リクエスト ID
    #[serde(rename = "request.id", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// リソースの種類
    #[serde(rename = "resource.type", skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,

    /// リソースの ID
    #[serde(rename = "resource.id", skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,

    /// リトライ可能かどうか
    #[serde(rename = "error.retryable")]
    pub retryable: bool,

    /// クライアント起因かどうか
    #[serde(rename = "error.client_error")]
    pub client_error: bool,
}

impl ErrorLog {
    /// AppError からログ情報を作成
    pub fn from_app_error(app_error: &AppError) -> Self {
        let domain_error = app_error.domain_error();

        Self {
            error_kind: app_error.kind().as_str().to_string(),
            error_code: app_error.error_code().to_string(),
            message: app_error.message().to_string(),
            http_status_code: None,
            grpc_status_code: None,
            grpc_status: None,
            trace_id: app_error.context().trace_id.clone(),
            request_id: app_error.context().request_id.clone(),
            resource_type: domain_error.resource_type().map(|s| s.to_string()),
            resource_id: domain_error.resource_id().map(|s| s.to_string()),
            retryable: app_error.is_retryable(),
            client_error: app_error.kind().is_client_error(),
        }
    }

    /// HTTP ステータスコードを設定
    pub fn with_http_status(mut self, status_code: u16) -> Self {
        self.http_status_code = Some(status_code);
        self
    }

    /// gRPC ステータスを設定
    pub fn with_grpc_status(mut self, status_code: GrpcStatusCode) -> Self {
        self.grpc_status_code = Some(status_code.as_i32());
        self.grpc_status = Some(status_code.as_str().to_string());
        self
    }

    /// JSON 文字列に変換
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// ログレベルを決定
    pub fn log_level(&self) -> LogLevel {
        match self.error_kind.as_str() {
            "INTERNAL" => LogLevel::Error,
            "DEPENDENCY_FAILURE" | "TRANSIENT" => LogLevel::Warn,
            _ => LogLevel::Info,
        }
    }
}

/// ログレベル
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// デバッグ
    Debug,
    /// 情報
    Info,
    /// 警告
    Warn,
    /// エラー
    Error,
}

impl LogLevel {
    /// 文字列表現を取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

/// エラーをログ出力可能にするトレイト
pub trait Loggable {
    /// ログ情報に変換
    fn to_error_log(&self) -> ErrorLog;

    /// 推奨されるログレベルを取得
    fn log_level(&self) -> LogLevel {
        self.to_error_log().log_level()
    }
}

impl Loggable for AppError {
    fn to_error_log(&self) -> ErrorLog {
        ErrorLog::from_app_error(self)
    }
}

/// HTTP レスポンス用のログ情報
#[derive(Debug, Clone, Serialize)]
pub struct HttpErrorLog {
    /// 基本のエラーログ
    #[serde(flatten)]
    pub error: ErrorLog,

    /// HTTP メソッド
    #[serde(rename = "http.method", skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// リクエストパス
    #[serde(rename = "http.path", skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl HttpErrorLog {
    /// 作成
    pub fn new(error: ErrorLog) -> Self {
        Self {
            error,
            method: None,
            path: None,
        }
    }

    /// HTTP メソッドを設定
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// リクエストパスを設定
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// gRPC レスポンス用のログ情報
#[derive(Debug, Clone, Serialize)]
pub struct GrpcErrorLog {
    /// 基本のエラーログ
    #[serde(flatten)]
    pub error: ErrorLog,

    /// gRPC サービス名
    #[serde(rename = "grpc.service", skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,

    /// gRPC メソッド名
    #[serde(rename = "grpc.method", skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
}

impl GrpcErrorLog {
    /// 作成
    pub fn new(error: ErrorLog) -> Self {
        Self {
            error,
            service: None,
            method: None,
        }
    }

    /// gRPC サービス名を設定
    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }

    /// gRPC メソッド名を設定
    pub fn with_method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomainError, ErrorCode};

    #[test]
    fn test_error_log_from_app_error() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"))
            .with_trace_id("trace-abc");

        let log = ErrorLog::from_app_error(&app_err);

        assert_eq!(log.error_kind, "NOT_FOUND");
        assert_eq!(log.error_code, "USER_NOT_FOUND");
        assert_eq!(log.trace_id, Some("trace-abc".to_string()));
        assert_eq!(log.resource_type, Some("User".to_string()));
        assert!(!log.retryable);
        assert!(log.client_error);
    }

    #[test]
    fn test_error_log_with_http_status() {
        let domain_err = DomainError::internal("test");
        let app_err = AppError::from_domain(domain_err, ErrorCode::internal());

        let log = ErrorLog::from_app_error(&app_err).with_http_status(500);

        assert_eq!(log.http_status_code, Some(500));
    }

    #[test]
    fn test_error_log_with_grpc_status() {
        let domain_err = DomainError::not_found("User", "123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::not_found());

        let log = ErrorLog::from_app_error(&app_err).with_grpc_status(GrpcStatusCode::NotFound);

        assert_eq!(log.grpc_status_code, Some(5));
        assert_eq!(log.grpc_status, Some("NOT_FOUND".to_string()));
    }

    #[test]
    fn test_error_log_to_json() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"))
            .with_trace_id("trace-abc");

        let log = ErrorLog::from_app_error(&app_err).with_http_status(404);
        let json = log.to_json().unwrap();

        assert!(json.contains("\"error.kind\":\"NOT_FOUND\""));
        assert!(json.contains("\"error.code\":\"USER_NOT_FOUND\""));
        assert!(json.contains("\"http.status_code\":404"));
        assert!(json.contains("\"trace.id\":\"trace-abc\""));
    }

    #[test]
    fn test_log_level() {
        let internal = ErrorLog::from_app_error(&AppError::from_domain(
            DomainError::internal("test"),
            ErrorCode::internal(),
        ));
        assert_eq!(internal.log_level(), LogLevel::Error);

        let not_found = ErrorLog::from_app_error(&AppError::from_domain(
            DomainError::not_found("X", "1"),
            ErrorCode::not_found(),
        ));
        assert_eq!(not_found.log_level(), LogLevel::Info);

        let transient = ErrorLog::from_app_error(&AppError::from_domain(
            DomainError::transient("test"),
            ErrorCode::transient(),
        ));
        assert_eq!(transient.log_level(), LogLevel::Warn);
    }

    #[test]
    fn test_loggable_trait() {
        let app_err = AppError::from_domain(
            DomainError::internal("test"),
            ErrorCode::internal(),
        );

        let log = app_err.to_error_log();
        assert_eq!(log.error_kind, "INTERNAL");

        let level = app_err.log_level();
        assert_eq!(level, LogLevel::Error);
    }

    #[test]
    fn test_http_error_log() {
        let domain_err = DomainError::not_found("User", "123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::not_found());
        let base_log = ErrorLog::from_app_error(&app_err).with_http_status(404);

        let http_log = HttpErrorLog::new(base_log)
            .with_method("GET")
            .with_path("/api/users/123");

        let json = serde_json::to_string(&http_log).unwrap();
        assert!(json.contains("\"http.method\":\"GET\""));
        assert!(json.contains("\"http.path\":\"/api/users/123\""));
    }

    #[test]
    fn test_grpc_error_log() {
        let domain_err = DomainError::not_found("User", "123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::not_found());
        let base_log =
            ErrorLog::from_app_error(&app_err).with_grpc_status(GrpcStatusCode::NotFound);

        let grpc_log = GrpcErrorLog::new(base_log)
            .with_service("UserService")
            .with_method("GetUser");

        let json = serde_json::to_string(&grpc_log).unwrap();
        assert!(json.contains("\"grpc.service\":\"UserService\""));
        assert!(json.contains("\"grpc.method\":\"GetUser\""));
    }
}
