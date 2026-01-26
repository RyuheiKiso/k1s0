//! Application 層エラー
//!
//! domain 層のエラーに `error_code` を付与し、presentation 層への橋渡しを行う。

use std::fmt;

use crate::code::ErrorCode;
use crate::context::ErrorContext;
use crate::domain::{DomainError, ErrorKind};
use crate::presentation::{GrpcError, HttpError};

/// Application 層エラー
///
/// domain 層のエラーに `error_code` と相関情報を付与する。
/// presentation 層で REST/gRPC 表現へ変換可能。
#[derive(Debug, Clone)]
pub struct AppError {
    /// 元のドメインエラー
    domain_error: DomainError,
    /// エラーコード（運用識別子）
    error_code: ErrorCode,
    /// 相関情報
    context: ErrorContext,
    /// 追加のヒント（ユーザー向けのアクション提案）
    hint: Option<String>,
}

impl AppError {
    /// ドメインエラーから AppError を作成
    ///
    /// # Arguments
    ///
    /// * `domain_error` - 元のドメインエラー
    /// * `error_code` - 付与するエラーコード
    pub fn from_domain(domain_error: DomainError, error_code: ErrorCode) -> Self {
        Self {
            domain_error,
            error_code,
            context: ErrorContext::new(),
            hint: None,
        }
    }

    /// エラーの種類に基づいてデフォルトのエラーコードを自動付与
    pub fn from_domain_auto(domain_error: DomainError) -> Self {
        let error_code = match domain_error.kind() {
            ErrorKind::InvalidInput => ErrorCode::validation_error(),
            ErrorKind::NotFound => ErrorCode::not_found(),
            ErrorKind::Conflict => ErrorCode::conflict(),
            ErrorKind::Unauthorized => ErrorCode::unauthenticated(),
            ErrorKind::Forbidden => ErrorCode::permission_denied(),
            ErrorKind::DependencyFailure => ErrorCode::dependency_failure(),
            ErrorKind::Transient => ErrorCode::transient(),
            ErrorKind::InvariantViolation => ErrorCode::validation_error(),
            ErrorKind::Internal => ErrorCode::internal(),
        };
        Self::from_domain(domain_error, error_code)
    }

    /// トレース ID を設定
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.context = self.context.with_trace_id(trace_id);
        self
    }

    /// リクエスト ID を設定
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.context = self.context.with_request_id(request_id);
        self
    }

    /// コンテキストを設定
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = self.context.merge(&context);
        self
    }

    /// ヒントを設定
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// エラーの種類を取得
    pub fn kind(&self) -> ErrorKind {
        self.domain_error.kind()
    }

    /// エラーコードを取得
    pub fn error_code(&self) -> &ErrorCode {
        &self.error_code
    }

    /// エラーメッセージを取得
    pub fn message(&self) -> &str {
        self.domain_error.message()
    }

    /// ドメインエラーへの参照を取得
    pub fn domain_error(&self) -> &DomainError {
        &self.domain_error
    }

    /// コンテキストへの参照を取得
    pub fn context(&self) -> &ErrorContext {
        &self.context
    }

    /// ヒントを取得
    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    /// リトライ可能かどうか
    pub fn is_retryable(&self) -> bool {
        self.domain_error.is_retryable()
    }

    /// HTTP エラーに変換
    pub fn to_http_error(&self) -> HttpError {
        HttpError::from_app_error(self)
    }

    /// gRPC エラーに変換
    pub fn to_grpc_error(&self) -> GrpcError {
        GrpcError::from_app_error(self)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.error_code, self.domain_error)
    }
}

impl std::error::Error for AppError {}

impl From<DomainError> for AppError {
    fn from(domain_error: DomainError) -> Self {
        Self::from_domain_auto(domain_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_domain() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"));

        assert_eq!(app_err.kind(), ErrorKind::NotFound);
        assert_eq!(app_err.error_code().as_str(), "USER_NOT_FOUND");
    }

    #[test]
    fn test_from_domain_auto() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain_auto(domain_err);

        assert_eq!(app_err.error_code().as_str(), "NOT_FOUND");
    }

    #[test]
    fn test_from_domain_auto_all_kinds() {
        let cases = vec![
            (DomainError::invalid_input("test"), "VALIDATION_ERROR"),
            (DomainError::not_found("X", "1"), "NOT_FOUND"),
            (DomainError::conflict("test"), "CONFLICT"),
            (DomainError::unauthorized("test"), "UNAUTHENTICATED"),
            (DomainError::forbidden("test"), "PERMISSION_DENIED"),
            (
                DomainError::dependency_failure("db", "err"),
                "DEPENDENCY_FAILURE",
            ),
            (DomainError::transient("test"), "TRANSIENT_ERROR"),
            (DomainError::invariant_violation("test"), "VALIDATION_ERROR"),
            (DomainError::internal("test"), "INTERNAL_ERROR"),
        ];

        for (domain_err, expected_code) in cases {
            let app_err = AppError::from_domain_auto(domain_err);
            assert_eq!(app_err.error_code().as_str(), expected_code);
        }
    }

    #[test]
    fn test_with_trace_id() {
        let domain_err = DomainError::internal("test");
        let app_err = AppError::from_domain_auto(domain_err).with_trace_id("trace-123");

        assert_eq!(
            app_err.context().trace_id,
            Some("trace-123".to_string())
        );
    }

    #[test]
    fn test_with_request_id() {
        let domain_err = DomainError::internal("test");
        let app_err = AppError::from_domain_auto(domain_err).with_request_id("req-456");

        assert_eq!(
            app_err.context().request_id,
            Some("req-456".to_string())
        );
    }

    #[test]
    fn test_with_context() {
        let domain_err = DomainError::internal("test");
        let ctx = ErrorContext::new()
            .with_trace_id("trace-123")
            .with_tenant_id("tenant-456");

        let app_err = AppError::from_domain_auto(domain_err).with_context(ctx);

        assert_eq!(
            app_err.context().trace_id,
            Some("trace-123".to_string())
        );
        assert_eq!(
            app_err.context().tenant_id,
            Some("tenant-456".to_string())
        );
    }

    #[test]
    fn test_with_hint() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain_auto(domain_err)
            .with_hint("ユーザー ID を確認してください");

        assert_eq!(app_err.hint(), Some("ユーザー ID を確認してください"));
    }

    #[test]
    fn test_is_retryable() {
        let transient_err = AppError::from_domain_auto(DomainError::transient("test"));
        assert!(transient_err.is_retryable());

        let not_found_err = AppError::from_domain_auto(DomainError::not_found("X", "1"));
        assert!(!not_found_err.is_retryable());
    }

    #[test]
    fn test_display() {
        let domain_err = DomainError::not_found("User", "user-123");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"));

        let display = format!("{}", app_err);
        assert!(display.contains("USER_NOT_FOUND"));
        assert!(display.contains("User"));
    }

    #[test]
    fn test_from_domain_error() {
        let domain_err = DomainError::internal("test");
        let app_err: AppError = domain_err.into();
        assert_eq!(app_err.error_code().as_str(), "INTERNAL_ERROR");
    }
}
