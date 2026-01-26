//! k1s0-error
//!
//! Clean Architecture に基づいたエラー表現の統一ライブラリ。
//!
//! # 設計方針
//!
//! - **domain 層**: transport 非依存のエラー型（HTTP/gRPC を意識しない）
//! - **application 層**: `error_code` を付与し、運用で識別可能にする
//! - **presentation 層**: REST（problem+json）/ gRPC（status + metadata）へ変換
//!
//! # エラー分類
//!
//! | 分類 | 説明 | HTTP | gRPC |
//! |------|------|------|------|
//! | InvalidInput | 入力不備 | 400 | INVALID_ARGUMENT |
//! | NotFound | リソースが見つからない | 404 | NOT_FOUND |
//! | Conflict | 競合（重複等） | 409 | ALREADY_EXISTS |
//! | Unauthorized | 認証エラー | 401 | UNAUTHENTICATED |
//! | Forbidden | 認可エラー | 403 | PERMISSION_DENIED |
//! | DependencyFailure | 依存障害 | 502 | UNAVAILABLE |
//! | Transient | 一時障害 | 503 | UNAVAILABLE |
//! | Internal | 内部エラー | 500 | INTERNAL |
//!
//! # 使用例
//!
//! ```
//! use k1s0_error::{DomainError, AppError, ErrorCode, ErrorKind};
//!
//! // domain 層: transport 非依存
//! let domain_err = DomainError::not_found("User", "user-123");
//!
//! // application 層: error_code 付与
//! let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"));
//!
//! // presentation 層: REST/gRPC 変換
//! let http_err = app_err.to_http_error();
//! let grpc_err = app_err.to_grpc_error();
//! ```

mod application;
mod code;
mod context;
mod domain;
pub mod logging;
pub mod metrics;
pub mod middleware;
mod presentation;

pub use application::AppError;
pub use code::{ErrorCode, GrpcCode, HttpStatus};
pub use context::ErrorContext;
pub use domain::{DomainError, ErrorKind};
pub use logging::{ErrorLog, GrpcErrorLog, HttpErrorLog, LogLevel, Loggable};
pub use metrics::{ErrorCounter, ErrorMetricLabels, ErrorMetricNames, Measurable};
pub use middleware::{
    ContentType, ErrorMappingConfig, GrpcErrorResponse, GrpcMetadataKeys, HttpErrorResponse,
};
pub use presentation::{GrpcError, GrpcStatusCode, HttpError, ProblemDetails};

/// エラー結果の型エイリアス（domain 層用）
pub type DomainResult<T> = Result<T, DomainError>;

/// エラー結果の型エイリアス（application 層用）
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_flow() {
        // domain 層: transport 非依存
        let domain_err = DomainError::not_found("User", "user-123");
        assert_eq!(domain_err.kind(), ErrorKind::NotFound);

        // application 層: error_code 付与
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("USER_NOT_FOUND"));
        assert_eq!(app_err.error_code().as_str(), "USER_NOT_FOUND");

        // presentation 層: HTTP 変換
        let http_err = app_err.to_http_error();
        assert_eq!(http_err.status_code(), 404);

        // presentation 層: gRPC 変換
        let grpc_err = app_err.to_grpc_error();
        assert_eq!(grpc_err.status_code(), GrpcStatusCode::NotFound);
    }

    #[test]
    fn test_with_context() {
        let domain_err = DomainError::internal("データベース接続エラー");
        let app_err = AppError::from_domain(domain_err, ErrorCode::new("DB_CONNECTION_ERROR"))
            .with_trace_id("trace-abc123")
            .with_request_id("req-xyz789");

        let http_err = app_err.to_http_error();
        assert!(http_err.trace_id().is_some());
        assert!(http_err.request_id().is_some());
    }
}
