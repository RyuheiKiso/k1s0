//! HTTP エラー表現
//!
//! RFC 7807 Problem Details 形式でエラーを表現する。

use serde::{Deserialize, Serialize};

use crate::application::AppError;
use crate::domain::ErrorKind;

/// HTTP エラー
///
/// application 層のエラーを HTTP レスポンス向けに変換したもの。
#[derive(Debug, Clone)]
pub struct HttpError {
    /// HTTP ステータスコード
    status_code: u16,
    /// エラーコード
    error_code: String,
    /// エラーメッセージ
    message: String,
    /// トレース ID
    trace_id: Option<String>,
    /// リクエスト ID
    request_id: Option<String>,
    /// ヒント
    hint: Option<String>,
    /// リソースの種類
    resource_type: Option<String>,
    /// リソースの ID
    resource_id: Option<String>,
}

impl HttpError {
    /// AppError から HttpError を作成
    pub fn from_app_error(app_error: &AppError) -> Self {
        let status_code = Self::kind_to_status_code(app_error.kind());
        let domain_error = app_error.domain_error();

        Self {
            status_code,
            error_code: app_error.error_code().to_string(),
            message: app_error.message().to_string(),
            trace_id: app_error.context().trace_id.clone(),
            request_id: app_error.context().request_id.clone(),
            hint: app_error.hint().map(|s| s.to_string()),
            resource_type: domain_error.resource_type().map(|s| s.to_string()),
            resource_id: domain_error.resource_id().map(|s| s.to_string()),
        }
    }

    /// エラーの種類を HTTP ステータスコードに変換
    fn kind_to_status_code(kind: ErrorKind) -> u16 {
        match kind {
            ErrorKind::InvalidInput | ErrorKind::InvariantViolation => 400,
            ErrorKind::Unauthorized => 401,
            ErrorKind::Forbidden => 403,
            ErrorKind::NotFound => 404,
            ErrorKind::Conflict => 409,
            ErrorKind::DependencyFailure => 502,
            ErrorKind::Transient => 503,
            ErrorKind::Internal => 500,
        }
    }

    /// HTTP ステータスコードを取得
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// エラーコードを取得
    pub fn error_code(&self) -> &str {
        &self.error_code
    }

    /// エラーメッセージを取得
    pub fn message(&self) -> &str {
        &self.message
    }

    /// トレース ID を取得
    pub fn trace_id(&self) -> Option<&str> {
        self.trace_id.as_deref()
    }

    /// リクエスト ID を取得
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// ヒントを取得
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    /// リソースの種類を取得
    pub fn resource_type(&self) -> Option<&str> {
        self.resource_type.as_deref()
    }

    /// リソースの ID を取得
    pub fn resource_id(&self) -> Option<&str> {
        self.resource_id.as_deref()
    }

    /// Problem Details 形式に変換
    pub fn to_problem_details(&self, instance: &str) -> ProblemDetails {
        ProblemDetails {
            type_uri: format!(
                "https://k1s0.io/errors/{}",
                self.error_code.to_lowercase().replace('_', "-")
            ),
            title: self.status_code_to_title(),
            status: self.status_code,
            detail: self.message.clone(),
            instance: instance.to_string(),
            error_code: self.error_code.clone(),
            trace_id: self.trace_id.clone(),
            request_id: self.request_id.clone(),
            hint: self.hint.clone(),
        }
    }

    /// ステータスコードからタイトルを取得
    fn status_code_to_title(&self) -> String {
        match self.status_code {
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            409 => "Conflict",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            _ => "Error",
        }
        .to_string()
    }
}

/// Problem Details (RFC 7807)
///
/// REST API のエラーレスポンスを表現する。
/// `application/problem+json` 形式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    /// エラータイプの URI
    #[serde(rename = "type")]
    pub type_uri: String,

    /// エラーのタイトル
    pub title: String,

    /// HTTP ステータスコード
    pub status: u16,

    /// エラーの詳細説明
    pub detail: String,

    /// 問題が発生したリソースのパス
    pub instance: String,

    /// エラーコード（k1s0 固有）
    pub error_code: String,

    /// トレース ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// リクエスト ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// ヒント
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl ProblemDetails {
    /// JSON 文字列に変換
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 整形された JSON 文字列に変換
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomainError, ErrorCode};

    #[test]
    fn test_from_app_error_not_found() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"));
        let http_err = HttpError::from_app_error(&app_err);

        assert_eq!(http_err.status_code(), 404);
        assert_eq!(http_err.error_code(), "USER_NOT_FOUND");
    }

    #[test]
    fn test_from_app_error_with_context() {
        let domain_err = DomainError::internal("test");
        let app_err = AppError::from_domain(domain_err, ErrorCode::internal())
            .with_trace_id("trace-123")
            .with_request_id("req-456");
        let http_err = HttpError::from_app_error(&app_err);

        assert_eq!(http_err.trace_id(), Some("trace-123"));
        assert_eq!(http_err.request_id(), Some("req-456"));
    }

    #[test]
    fn test_status_code_mapping() {
        let cases = vec![
            (ErrorKind::InvalidInput, 400),
            (ErrorKind::Unauthorized, 401),
            (ErrorKind::Forbidden, 403),
            (ErrorKind::NotFound, 404),
            (ErrorKind::Conflict, 409),
            (ErrorKind::Internal, 500),
            (ErrorKind::DependencyFailure, 502),
            (ErrorKind::Transient, 503),
        ];

        for (kind, expected_status) in cases {
            assert_eq!(HttpError::kind_to_status_code(kind), expected_status);
        }
    }

    #[test]
    fn test_to_problem_details() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"))
            .with_trace_id("trace-123");
        let http_err = HttpError::from_app_error(&app_err);
        let problem = http_err.to_problem_details("/api/users/user-123");

        assert_eq!(problem.status, 404);
        assert_eq!(problem.error_code, "USER_NOT_FOUND");
        assert_eq!(problem.instance, "/api/users/user-123");
        assert_eq!(problem.trace_id, Some("trace-123".to_string()));
        assert!(problem.type_uri.contains("user-not-found"));
    }

    #[test]
    fn test_problem_details_to_json() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"));
        let http_err = HttpError::from_app_error(&app_err);
        let problem = http_err.to_problem_details("/api/users");

        let json = problem.to_json().unwrap();
        assert!(json.contains("USER_NOT_FOUND"));
        assert!(json.contains("404"));
    }

    #[test]
    fn test_problem_details_deserialize() {
        let json = r#"{
            "type": "https://k1s0.io/errors/not-found",
            "title": "Not Found",
            "status": 404,
            "detail": "User 'user-123' が見つかりません",
            "instance": "/api/users",
            "error_code": "USER_NOT_FOUND"
        }"#;

        let problem: ProblemDetails = serde_json::from_str(json).unwrap();
        assert_eq!(problem.status, 404);
        assert_eq!(problem.error_code, "USER_NOT_FOUND");
    }
}
