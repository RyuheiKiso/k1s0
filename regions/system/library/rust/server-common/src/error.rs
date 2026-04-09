//! Structured error code system for k1s0 system tier servers.
//!
//! Error codes follow the pattern: `SYS_{SERVICE}_{ERROR}`
//!
//! Examples:
//! - `SYS_CONFIG_KEY_NOT_FOUND`
//! - `SYS_AUTH_UNAUTHORIZED`
//! - `SYS_DLQ_PROCESS_FAILED`
//! - `SYS_APIREG_NOT_FOUND`
//!
//! Each error includes a machine-readable code, human-readable message,
//! a unique request ID for tracing, and optional structured details.

use serde::Serialize;

/// `ErrorCode` represents a structured error code for the system tier.
///
/// Error codes follow the `SYS_{SERVICE}_{ERROR}` naming convention.
/// Services define their own codes using these constants or custom strings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "utoipa", schema(value_type = String, example = "SYS_AUTH_TOKEN_EXPIRED"))]
pub struct ErrorCode(String);

impl ErrorCode {
    /// Create a new error code from a string.
    ///
    /// Codes should follow the pattern `SYS_{SERVICE}_{ERROR}`.
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// Create a standard "not found" error code for a service.
    #[must_use]
    pub fn not_found(service: &str) -> Self {
        Self(format!("SYS_{}_NOT_FOUND", service.to_uppercase()))
    }

    /// Create a standard "validation failed" error code for a service.
    #[must_use]
    pub fn validation(service: &str) -> Self {
        Self(format!("SYS_{}_VALIDATION_FAILED", service.to_uppercase()))
    }

    /// Create a standard "internal error" error code for a service.
    #[must_use]
    pub fn internal(service: &str) -> Self {
        Self(format!("SYS_{}_INTERNAL_ERROR", service.to_uppercase()))
    }

    /// Create a standard "unauthorized" error code for a service.
    #[must_use]
    pub fn unauthorized(service: &str) -> Self {
        Self(format!("SYS_{}_UNAUTHORIZED", service.to_uppercase()))
    }

    /// Create a standard "forbidden" error code for a service.
    #[must_use]
    pub fn forbidden(service: &str) -> Self {
        Self(format!("SYS_{}_PERMISSION_DENIED", service.to_uppercase()))
    }

    /// Create a standard "conflict" error code for a service.
    #[must_use]
    pub fn conflict(service: &str) -> Self {
        Self(format!("SYS_{}_CONFLICT", service.to_uppercase()))
    }

    /// Create a standard "unprocessable entity" error code for a service.
    #[must_use]
    pub fn unprocessable(service: &str) -> Self {
        Self(format!(
            "SYS_{}_BUSINESS_RULE_VIOLATION",
            service.to_uppercase()
        ))
    }

    /// Create a standard "rate exceeded" error code for a service.
    #[must_use]
    pub fn rate_exceeded(service: &str) -> Self {
        Self(format!("SYS_{}_RATE_EXCEEDED", service.to_uppercase()))
    }

    /// Create a standard "service unavailable" error code for a service.
    #[must_use]
    pub fn service_unavailable(service: &str) -> Self {
        Self(format!(
            "SYS_{}_SERVICE_UNAVAILABLE",
            service.to_uppercase()
        ))
    }

    /// Create a standard "not found" error code for a business tier service.
    #[must_use]
    pub fn biz_not_found(service: &str) -> Self {
        Self(format!("BIZ_{}_NOT_FOUND", service.to_uppercase()))
    }

    /// Create a standard "validation failed" error code for a business tier service.
    #[must_use]
    pub fn biz_validation(service: &str) -> Self {
        Self(format!("BIZ_{}_VALIDATION_FAILED", service.to_uppercase()))
    }

    /// Create a standard "not found" error code for a service tier service.
    #[must_use]
    pub fn svc_not_found(service: &str) -> Self {
        Self(format!("SVC_{}_NOT_FOUND", service.to_uppercase()))
    }

    /// Create a standard "validation failed" error code for a service tier service.
    #[must_use]
    pub fn svc_validation(service: &str) -> Self {
        Self(format!("SVC_{}_VALIDATION_FAILED", service.to_uppercase()))
    }

    /// Return the error code string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for ErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl From<&str> for ErrorCode {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ErrorCode {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// `ErrorDetail` provides additional context for an error field.
///
/// Follows the REST-API設計.md D-007 specification:
/// `{ "field": "quantity", "reason": "invalid_type", "message": "..." }`
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ErrorDetail {
    pub field: String,
    pub reason: String,
    pub message: String,
}

impl ErrorDetail {
    pub fn new(
        field: impl Into<String>,
        reason: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            reason: reason.into(),
            message: message.into(),
        }
    }
}

/// `ErrorBody` is the structured error payload.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<ErrorDetail>,
}

/// `ErrorResponse` wraps `ErrorBody` in an `{ "error": ... }` envelope.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

impl ErrorResponse {
    /// Create a new error response with code and message.
    pub fn new(code: impl Into<ErrorCode>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorBody {
                code: code.into(),
                message: message.into(),
                request_id: default_request_id(),
                details: vec![],
            },
        }
    }

    /// Create a new error response with code, message, and details.
    pub fn with_details(
        code: impl Into<ErrorCode>,
        message: impl Into<String>,
        details: Vec<ErrorDetail>,
    ) -> Self {
        Self {
            error: ErrorBody {
                code: code.into(),
                message: message.into(),
                request_id: default_request_id(),
                details,
            },
        }
    }

    /// Override `request_id` when a correlation ID is already available.
    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.error.request_id = request_id.into();
        self
    }
}

fn default_request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// `ServiceError` is a high-level error type that maps to HTTP status codes.
///
/// Each variant carries a structured error code and message.
/// When the `axum` feature is enabled, `ServiceError` implements `IntoResponse`.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// 404 Not Found
    #[error("{message}")]
    NotFound { code: ErrorCode, message: String },

    /// 400 Bad Request
    #[error("{message}")]
    BadRequest {
        code: ErrorCode,
        message: String,
        details: Vec<ErrorDetail>,
    },

    /// 401 Unauthorized
    #[error("{message}")]
    Unauthorized { code: ErrorCode, message: String },

    /// 403 Forbidden
    #[error("{message}")]
    Forbidden { code: ErrorCode, message: String },

    /// 409 Conflict
    #[error("{message}")]
    Conflict {
        code: ErrorCode,
        message: String,
        details: Vec<ErrorDetail>,
    },

    /// 422 Unprocessable Entity (business rule violation)
    #[error("{message}")]
    UnprocessableEntity {
        code: ErrorCode,
        message: String,
        details: Vec<ErrorDetail>,
    },

    /// 429 Too Many Requests (rate limit exceeded)
    #[error("{message}")]
    TooManyRequests { code: ErrorCode, message: String },

    /// 500 Internal Server Error
    #[error("{message}")]
    Internal { code: ErrorCode, message: String },

    /// 503 Service Unavailable
    #[error("{message}")]
    ServiceUnavailable { code: ErrorCode, message: String },
}

impl ServiceError {
    /// Create a `NotFound` error for a service.
    pub fn not_found(service: &str, message: impl Into<String>) -> Self {
        Self::NotFound {
            code: ErrorCode::not_found(service),
            message: message.into(),
        }
    }

    /// Create a `BadRequest` error for a service.
    pub fn bad_request(service: &str, message: impl Into<String>) -> Self {
        Self::BadRequest {
            code: ErrorCode::validation(service),
            message: message.into(),
            details: vec![],
        }
    }

    /// Create a `BadRequest` error with field-level details.
    pub fn bad_request_with_details(
        service: &str,
        message: impl Into<String>,
        details: Vec<ErrorDetail>,
    ) -> Self {
        Self::BadRequest {
            code: ErrorCode::validation(service),
            message: message.into(),
            details,
        }
    }

    /// Create an Unauthorized error for a service.
    pub fn unauthorized(service: &str, message: impl Into<String>) -> Self {
        Self::Unauthorized {
            code: ErrorCode::unauthorized(service),
            message: message.into(),
        }
    }

    /// Create a Forbidden error for a service.
    pub fn forbidden(service: &str, message: impl Into<String>) -> Self {
        Self::Forbidden {
            code: ErrorCode::forbidden(service),
            message: message.into(),
        }
    }

    /// Create a Conflict error for a service.
    pub fn conflict(service: &str, message: impl Into<String>) -> Self {
        Self::Conflict {
            code: ErrorCode::conflict(service),
            message: message.into(),
            details: vec![],
        }
    }

    /// Create an `UnprocessableEntity` error for a service (business rule violation).
    pub fn unprocessable_entity(service: &str, message: impl Into<String>) -> Self {
        Self::UnprocessableEntity {
            code: ErrorCode::unprocessable(service),
            message: message.into(),
            details: vec![],
        }
    }

    /// Create a `TooManyRequests` error for a service (rate limit exceeded).
    pub fn too_many_requests(service: &str, message: impl Into<String>) -> Self {
        Self::TooManyRequests {
            code: ErrorCode::rate_exceeded(service),
            message: message.into(),
        }
    }

    /// Create an Internal error for a service.
    pub fn internal(service: &str, message: impl Into<String>) -> Self {
        Self::Internal {
            code: ErrorCode::internal(service),
            message: message.into(),
        }
    }

    /// Create a `ServiceUnavailable` error for a service.
    pub fn service_unavailable(service: &str, message: impl Into<String>) -> Self {
        Self::ServiceUnavailable {
            code: ErrorCode::service_unavailable(service),
            message: message.into(),
        }
    }

    /// Convert to an `ErrorResponse`.
    #[must_use]
    pub fn to_error_response(&self) -> ErrorResponse {
        match self {
            ServiceError::NotFound { code, message }
            | ServiceError::Unauthorized { code, message }
            | ServiceError::Forbidden { code, message }
            | ServiceError::TooManyRequests { code, message }
            | ServiceError::Internal { code, message }
            | ServiceError::ServiceUnavailable { code, message } => {
                ErrorResponse::new(code.clone(), message.clone())
            }
            ServiceError::BadRequest {
                code,
                message,
                details,
            }
            | ServiceError::Conflict {
                code,
                message,
                details,
            }
            | ServiceError::UnprocessableEntity {
                code,
                message,
                details,
            } => ErrorResponse::with_details(code.clone(), message.clone(), details.clone()),
        }
    }
}

// --- axum integration ---

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for ServiceError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            ServiceError::NotFound { .. } => axum::http::StatusCode::NOT_FOUND,
            ServiceError::BadRequest { .. } => axum::http::StatusCode::BAD_REQUEST,
            ServiceError::Unauthorized { .. } => axum::http::StatusCode::UNAUTHORIZED,
            ServiceError::Forbidden { .. } => axum::http::StatusCode::FORBIDDEN,
            ServiceError::Conflict { .. } => axum::http::StatusCode::CONFLICT,
            ServiceError::UnprocessableEntity { .. } => {
                axum::http::StatusCode::UNPROCESSABLE_ENTITY
            }
            ServiceError::TooManyRequests { .. } => axum::http::StatusCode::TOO_MANY_REQUESTS,
            ServiceError::Internal { .. } => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::ServiceUnavailable { .. } => axum::http::StatusCode::SERVICE_UNAVAILABLE,
        };

        let body = self.to_error_response();
        (status, axum::Json(body)).into_response()
    }
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        // Default to 500 if used standalone -- callers should use ServiceError for proper status codes.
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(self),
        )
            .into_response()
    }
}

// --- tonic (gRPC) integration ---

/// ドメインエラーを gRPC ステータスコードにマッピングするトレイト。
///
/// 各サーバーの `DomainError` に実装することで、gRPC ハンドラーから
/// 統一的に `tonic::Status` へ変換できる。
/// 典型的な実装: `DomainError` -> `ServiceError` -> `tonic::Status` の変換チェーン。
#[cfg(feature = "grpc-auth")]
pub trait IntoGrpcStatus {
    /// ドメインエラーを gRPC ステータスに変換する。
    fn into_grpc_status(self) -> tonic::Status;
}

/// `ServiceError` から `tonic::Status` への変換実装。
/// HTTP ステータスコードと gRPC ステータスコードの標準的な対応関係に従う。
#[cfg(feature = "grpc-auth")]
impl From<ServiceError> for tonic::Status {
    fn from(err: ServiceError) -> Self {
        let msg = err.to_string();
        match err {
            ServiceError::NotFound { .. } => tonic::Status::not_found(msg),
            ServiceError::BadRequest { .. } => tonic::Status::invalid_argument(msg),
            ServiceError::Unauthorized { .. } => tonic::Status::unauthenticated(msg),
            ServiceError::Forbidden { .. } => tonic::Status::permission_denied(msg),
            ServiceError::Conflict { .. } => tonic::Status::already_exists(msg),
            ServiceError::UnprocessableEntity { .. } => tonic::Status::failed_precondition(msg),
            ServiceError::TooManyRequests { .. } => tonic::Status::resource_exhausted(msg),
            ServiceError::Internal { .. } => tonic::Status::internal(msg),
            ServiceError::ServiceUnavailable { .. } => tonic::Status::unavailable(msg),
        }
    }
}

/// `ServiceError` は `IntoGrpcStatus` を直接実装する。
#[cfg(feature = "grpc-auth")]
impl IntoGrpcStatus for ServiceError {
    fn into_grpc_status(self) -> tonic::Status {
        self.into()
    }
}

/// `anyhow::Error` からドメインエラー型をダウンキャストして `tonic::Status` に変換するヘルパー。
///
/// ユースケースが `anyhow::Result` を返す場合に、gRPC ハンドラーで使用する。
/// ダウンキャストに成功した場合は `IntoGrpcStatus` 経由で変換し、
/// 失敗した場合は internal エラーとして扱う。
///
/// # 使用例
/// ```ignore
/// use k1s0_server_common::error::map_anyhow_to_grpc_status;
/// use crate::domain::error::MyDomainError;
///
/// let status = map_anyhow_to_grpc_status::<MyDomainError>(anyhow_err);
/// ```
#[cfg(feature = "grpc-auth")]
#[must_use]
pub fn map_anyhow_to_grpc_status<E>(err: anyhow::Error) -> tonic::Status
where
    E: std::error::Error + Send + Sync + 'static + Into<ServiceError>,
{
    match err.downcast::<E>() {
        Ok(domain_err) => {
            let service_err: ServiceError = domain_err.into();
            service_err.into()
        }
        Err(err) => tonic::Status::internal(err.to_string()),
    }
}

// --- Well-known error codes for system tier services ---

/// Well-known error codes for the Auth service.
pub mod auth {
    use super::ErrorCode;

    #[must_use]
    pub fn missing_claims() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_MISSING_CLAIMS")
    }

    #[must_use]
    pub fn permission_denied() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_PERMISSION_DENIED")
    }

    #[must_use]
    pub fn unauthorized() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_UNAUTHORIZED")
    }

    #[must_use]
    pub fn token_expired() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_TOKEN_EXPIRED")
    }

    #[must_use]
    pub fn invalid_token() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_INVALID_TOKEN")
    }

    #[must_use]
    pub fn jwks_fetch_failed() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_JWKS_FETCH_FAILED")
    }

    #[must_use]
    pub fn audit_validation() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_AUDIT_VALIDATION")
    }
}

/// Well-known error codes for the Config service.
pub mod config {
    use super::ErrorCode;

    #[must_use]
    pub fn key_not_found() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_KEY_NOT_FOUND")
    }

    #[must_use]
    pub fn service_not_found() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_SERVICE_NOT_FOUND")
    }

    #[must_use]
    pub fn schema_not_found() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_SCHEMA_NOT_FOUND")
    }

    #[must_use]
    pub fn version_conflict() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_VERSION_CONFLICT")
    }

    #[must_use]
    pub fn validation_failed() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_VALIDATION_FAILED")
    }

    #[must_use]
    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_INTERNAL_ERROR")
    }
}

/// Well-known error codes for the DLQ Manager service.
pub mod dlq {
    use super::ErrorCode;

    #[must_use]
    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_DLQ_NOT_FOUND")
    }

    #[must_use]
    pub fn validation_error() -> ErrorCode {
        ErrorCode::new("SYS_DLQ_VALIDATION_ERROR")
    }

    #[must_use]
    pub fn conflict() -> ErrorCode {
        ErrorCode::new("SYS_DLQ_CONFLICT")
    }

    #[must_use]
    pub fn process_failed() -> ErrorCode {
        ErrorCode::new("SYS_DLQ_PROCESS_FAILED")
    }

    #[must_use]
    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_DLQ_INTERNAL_ERROR")
    }
}

/// Well-known error codes for the Tenant service.
pub mod tenant {
    use super::ErrorCode;

    #[must_use]
    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_NOT_FOUND")
    }

    #[must_use]
    pub fn name_conflict() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_NAME_CONFLICT")
    }

    #[must_use]
    pub fn invalid_status() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_INVALID_STATUS")
    }

    #[must_use]
    pub fn invalid_input() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_INVALID_INPUT")
    }

    #[must_use]
    pub fn validation_error() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_VALIDATION_ERROR")
    }

    #[must_use]
    pub fn member_conflict() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_MEMBER_CONFLICT")
    }

    #[must_use]
    pub fn member_not_found() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_MEMBER_NOT_FOUND")
    }

    #[must_use]
    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_INTERNAL_ERROR")
    }
}

/// Well-known error codes for the Session service.
pub mod session {
    use super::ErrorCode;

    #[must_use]
    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_NOT_FOUND")
    }

    #[must_use]
    pub fn expired() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_EXPIRED")
    }

    #[must_use]
    pub fn already_revoked() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_ALREADY_REVOKED")
    }

    #[must_use]
    pub fn validation_error() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_VALIDATION_ERROR")
    }

    #[must_use]
    pub fn max_devices_exceeded() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_MAX_DEVICES_EXCEEDED")
    }

    #[must_use]
    pub fn forbidden() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_FORBIDDEN")
    }

    #[must_use]
    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_INTERNAL_ERROR")
    }
}

/// Well-known error codes for the API Registry service.
pub mod api_registry {
    use super::ErrorCode;

    #[must_use]
    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_NOT_FOUND")
    }

    #[must_use]
    pub fn bad_request() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_VALIDATION_ERROR")
    }

    #[must_use]
    pub fn conflict() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_CONFLICT")
    }

    #[must_use]
    pub fn unauthorized() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_UNAUTHORIZED")
    }

    #[must_use]
    pub fn schema_invalid() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_SCHEMA_INVALID")
    }

    #[must_use]
    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_INTERNAL_ERROR")
    }

    #[must_use]
    pub fn validator_error() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_VALIDATOR_ERROR")
    }

    #[must_use]
    pub fn schema_not_found() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_SCHEMA_NOT_FOUND")
    }

    #[must_use]
    pub fn version_not_found() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_VERSION_NOT_FOUND")
    }

    #[must_use]
    pub fn cannot_delete_latest() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_CANNOT_DELETE_LATEST")
    }

    #[must_use]
    pub fn already_exists() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_ALREADY_EXISTS")
    }
}

/// Well-known error codes for Event Store service.
pub mod event_store {
    use super::ErrorCode;

    #[must_use]
    pub fn stream_not_found() -> ErrorCode {
        ErrorCode::new("SYS_EVSTORE_STREAM_NOT_FOUND")
    }

    #[must_use]
    pub fn event_not_found() -> ErrorCode {
        ErrorCode::new("SYS_EVSTORE_EVENT_NOT_FOUND")
    }

    #[must_use]
    pub fn snapshot_not_found() -> ErrorCode {
        ErrorCode::new("SYS_EVSTORE_SNAPSHOT_NOT_FOUND")
    }

    #[must_use]
    pub fn version_conflict() -> ErrorCode {
        ErrorCode::new("SYS_EVSTORE_VERSION_CONFLICT")
    }

    #[must_use]
    pub fn stream_already_exists() -> ErrorCode {
        ErrorCode::new("SYS_EVSTORE_STREAM_ALREADY_EXISTS")
    }
}

/// Well-known error codes for File service.
pub mod file {
    use super::ErrorCode;

    #[must_use]
    pub fn validation() -> ErrorCode {
        ErrorCode::new("SYS_FILE_VALIDATION")
    }

    #[must_use]
    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_FILE_NOT_FOUND")
    }

    #[must_use]
    pub fn already_completed() -> ErrorCode {
        ErrorCode::new("SYS_FILE_ALREADY_COMPLETED")
    }

    #[must_use]
    pub fn not_available() -> ErrorCode {
        ErrorCode::new("SYS_FILE_NOT_AVAILABLE")
    }

    #[must_use]
    pub fn access_denied() -> ErrorCode {
        ErrorCode::new("SYS_FILE_ACCESS_DENIED")
    }

    #[must_use]
    pub fn storage_error() -> ErrorCode {
        ErrorCode::new("SYS_FILE_STORAGE_ERROR")
    }

    #[must_use]
    pub fn size_exceeded() -> ErrorCode {
        ErrorCode::new("SYS_FILE_SIZE_EXCEEDED")
    }

    #[must_use]
    pub fn upload_failed() -> ErrorCode {
        ErrorCode::new("SYS_FILE_UPLOAD_FAILED")
    }

    #[must_use]
    pub fn get_failed() -> ErrorCode {
        ErrorCode::new("SYS_FILE_GET_FAILED")
    }

    #[must_use]
    pub fn list_failed() -> ErrorCode {
        ErrorCode::new("SYS_FILE_LIST_FAILED")
    }

    #[must_use]
    pub fn delete_failed() -> ErrorCode {
        ErrorCode::new("SYS_FILE_DELETE_FAILED")
    }

    #[must_use]
    pub fn complete_failed() -> ErrorCode {
        ErrorCode::new("SYS_FILE_COMPLETE_FAILED")
    }

    #[must_use]
    pub fn download_url_failed() -> ErrorCode {
        ErrorCode::new("SYS_FILE_DOWNLOAD_URL_FAILED")
    }

    #[must_use]
    pub fn tags_update_failed() -> ErrorCode {
        ErrorCode::new("SYS_FILE_TAGS_UPDATE_FAILED")
    }
}

/// Well-known error codes for Scheduler service.
pub mod scheduler {
    use super::ErrorCode;

    #[must_use]
    pub fn already_exists() -> ErrorCode {
        ErrorCode::new("SYS_SCHED_ALREADY_EXISTS")
    }
}

/// Well-known error codes for API Response normalization in Notification service.
pub mod notification {
    use super::ErrorCode;

    #[must_use]
    pub fn invalid_id() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_INVALID_ID")
    }

    #[must_use]
    pub fn validation_error() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_VALIDATION_ERROR")
    }

    #[must_use]
    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_NOT_FOUND")
    }

    #[must_use]
    pub fn channel_not_found() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_CHANNEL_NOT_FOUND")
    }

    #[must_use]
    pub fn template_not_found() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_TEMPLATE_NOT_FOUND")
    }

    #[must_use]
    pub fn already_sent() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_ALREADY_SENT")
    }

    #[must_use]
    pub fn channel_disabled() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_CHANNEL_DISABLED")
    }

    #[must_use]
    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_INTERNAL_ERROR")
    }

    #[must_use]
    pub fn send_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_SEND_FAILED")
    }

    #[must_use]
    pub fn list_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_LIST_FAILED")
    }

    #[must_use]
    pub fn get_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_GET_FAILED")
    }

    #[must_use]
    pub fn retry_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_RETRY_FAILED")
    }

    #[must_use]
    pub fn channel_create_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_CHANNEL_CREATE_FAILED")
    }

    #[must_use]
    pub fn channel_list_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_CHANNEL_LIST_FAILED")
    }

    #[must_use]
    pub fn channel_get_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_CHANNEL_GET_FAILED")
    }

    #[must_use]
    pub fn channel_update_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_CHANNEL_UPDATE_FAILED")
    }

    #[must_use]
    pub fn channel_delete_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_CHANNEL_DELETE_FAILED")
    }

    #[must_use]
    pub fn template_create_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_TEMPLATE_CREATE_FAILED")
    }

    #[must_use]
    pub fn template_list_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_TEMPLATE_LIST_FAILED")
    }

    #[must_use]
    pub fn template_get_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_TEMPLATE_GET_FAILED")
    }

    #[must_use]
    pub fn template_update_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_TEMPLATE_UPDATE_FAILED")
    }

    #[must_use]
    pub fn template_delete_failed() -> ErrorCode {
        ErrorCode::new("SYS_NOTIFY_TEMPLATE_DELETE_FAILED")
    }
}

/// Well-known error codes for the Task service (service tier).
pub mod task {
    use super::ErrorCode;

    #[must_use]
    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SVC_TASK_NOT_FOUND")
    }

    #[must_use]
    pub fn validation_failed() -> ErrorCode {
        ErrorCode::new("SVC_TASK_VALIDATION_FAILED")
    }

    #[must_use]
    pub fn invalid_status_transition() -> ErrorCode {
        ErrorCode::new("SVC_TASK_INVALID_STATUS_TRANSITION")
    }

    #[must_use]
    pub fn version_conflict() -> ErrorCode {
        ErrorCode::new("SVC_TASK_VERSION_CONFLICT")
    }

    #[must_use]
    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SVC_TASK_INTERNAL_ERROR")
    }
}

/// Well-known error codes for Feature Flag service.
pub mod featureflag {
    use super::ErrorCode;

    #[must_use]
    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_FF_INTERNAL_ERROR")
    }

    #[must_use]
    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_FF_NOT_FOUND")
    }

    #[must_use]
    pub fn already_exists() -> ErrorCode {
        ErrorCode::new("SYS_FF_ALREADY_EXISTS")
    }

    #[must_use]
    pub fn list_failed() -> ErrorCode {
        ErrorCode::new("SYS_FF_LIST_FAILED")
    }

    #[must_use]
    pub fn get_failed() -> ErrorCode {
        ErrorCode::new("SYS_FF_GET_FAILED")
    }

    #[must_use]
    pub fn create_failed() -> ErrorCode {
        ErrorCode::new("SYS_FF_CREATE_FAILED")
    }

    #[must_use]
    pub fn update_failed() -> ErrorCode {
        ErrorCode::new("SYS_FF_UPDATE_FAILED")
    }

    #[must_use]
    pub fn delete_failed() -> ErrorCode {
        ErrorCode::new("SYS_FF_DELETE_FAILED")
    }

    #[must_use]
    pub fn evaluate_failed() -> ErrorCode {
        ErrorCode::new("SYS_FF_EVALUATE_FAILED")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // not_found エラーコードが正しい文字列を生成することを確認する。
    #[test]
    fn test_error_code_not_found() {
        let code = ErrorCode::not_found("CONFIG");
        assert_eq!(code.as_str(), "SYS_CONFIG_NOT_FOUND");
    }

    // validation エラーコードが正しい文字列を生成することを確認する。
    #[test]
    fn test_error_code_validation() {
        let code = ErrorCode::validation("DLQ");
        assert_eq!(code.as_str(), "SYS_DLQ_VALIDATION_FAILED");
    }

    // BIZ_ および SVC_ プレフィックスのエラーコードヘルパーが正しい文字列を生成することを確認する。
    #[test]
    fn test_error_code_biz_and_svc_helpers() {
        assert_eq!(
            ErrorCode::biz_not_found("ORDER").as_str(),
            "BIZ_ORDER_NOT_FOUND"
        );
        assert_eq!(
            ErrorCode::biz_validation("ORDER").as_str(),
            "BIZ_ORDER_VALIDATION_FAILED"
        );
        assert_eq!(
            ErrorCode::svc_not_found("PAYMENT").as_str(),
            "SVC_PAYMENT_NOT_FOUND"
        );
        assert_eq!(
            ErrorCode::svc_validation("PAYMENT").as_str(),
            "SVC_PAYMENT_VALIDATION_FAILED"
        );
    }

    // &str から ErrorCode への変換が正しく機能することを確認する。
    #[test]
    fn test_error_code_from_str() {
        let code = ErrorCode::from("SYS_AUTH_MISSING_CLAIMS");
        assert_eq!(code.as_str(), "SYS_AUTH_MISSING_CLAIMS");
    }

    // ErrorResponse::new でコード・メッセージ・request_id が正しく設定されることを確認する。
    #[test]
    fn test_error_response_new() {
        let resp = ErrorResponse::new("SYS_CONFIG_KEY_NOT_FOUND", "config key not found");
        assert_eq!(resp.error.code.as_str(), "SYS_CONFIG_KEY_NOT_FOUND");
        assert_eq!(resp.error.message, "config key not found");
        assert!(!resp.error.request_id.is_empty());
        assert!(resp.error.details.is_empty());
    }

    // ErrorResponse::with_details で詳細情報が正しく設定されることを確認する。
    #[test]
    fn test_error_response_with_details() {
        let details = vec![
            ErrorDetail::new("namespace", "required", "must not be empty"),
            ErrorDetail::new("key", "format", "invalid format"),
        ];
        let resp = ErrorResponse::with_details(
            "SYS_CONFIG_VALIDATION_FAILED",
            "validation failed",
            details,
        );
        assert_eq!(resp.error.details.len(), 2);
        assert_eq!(resp.error.details[0].field, "namespace");
        assert_eq!(resp.error.details[0].reason, "required");
        assert_eq!(resp.error.details[0].message, "must not be empty");
    }

    // ServiceError::not_found の ErrorResponse 変換でコードが正しいことを確認する。
    #[test]
    fn test_service_error_not_found() {
        let err = ServiceError::not_found("CONFIG", "key 'system.auth/jwt_secret' not found");
        let resp = err.to_error_response();
        assert_eq!(resp.error.code.as_str(), "SYS_CONFIG_NOT_FOUND");
    }

    // ServiceError::bad_request_with_details の ErrorResponse 変換で詳細が含まれることを確認する。
    #[test]
    fn test_service_error_bad_request_with_details() {
        let details = vec![ErrorDetail::new("page", "range", "must be >= 1")];
        let err = ServiceError::bad_request_with_details("CONFIG", "validation failed", details);
        let resp = err.to_error_response();
        assert_eq!(resp.error.code.as_str(), "SYS_CONFIG_VALIDATION_FAILED");
        assert_eq!(resp.error.details.len(), 1);
        assert_eq!(resp.error.details[0].reason, "range");
        assert_eq!(resp.error.details[0].message, "must be >= 1");
    }

    // Auth サービスの既知エラーコードが正しい文字列を返すことを確認する。
    #[test]
    fn test_well_known_auth_codes() {
        assert_eq!(auth::missing_claims().as_str(), "SYS_AUTH_MISSING_CLAIMS");
        assert_eq!(
            auth::permission_denied().as_str(),
            "SYS_AUTH_PERMISSION_DENIED"
        );
        assert_eq!(auth::unauthorized().as_str(), "SYS_AUTH_UNAUTHORIZED");
        assert_eq!(
            auth::audit_validation().as_str(),
            "SYS_AUTH_AUDIT_VALIDATION"
        );
    }

    // Config サービスの既知エラーコードが正しい文字列を返すことを確認する。
    #[test]
    fn test_well_known_config_codes() {
        assert_eq!(config::key_not_found().as_str(), "SYS_CONFIG_KEY_NOT_FOUND");
        assert_eq!(
            config::version_conflict().as_str(),
            "SYS_CONFIG_VERSION_CONFLICT"
        );
    }

    // DLQ サービスの既知エラーコードが正しい文字列を返すことを確認する。
    #[test]
    fn test_well_known_dlq_codes() {
        assert_eq!(dlq::not_found().as_str(), "SYS_DLQ_NOT_FOUND");
        assert_eq!(dlq::process_failed().as_str(), "SYS_DLQ_PROCESS_FAILED");
    }

    // API Registry サービスの既知エラーコードが正しい文字列を返すことを確認する。
    #[test]
    fn test_well_known_api_registry_codes() {
        assert_eq!(api_registry::not_found().as_str(), "SYS_APIREG_NOT_FOUND");
        assert_eq!(api_registry::conflict().as_str(), "SYS_APIREG_CONFLICT");
        assert_eq!(
            api_registry::validator_error().as_str(),
            "SYS_APIREG_VALIDATOR_ERROR"
        );
    }

    // ErrorResponse の JSON シリアライズで envelope 構造と各フィールドが正しいことを確認する。
    #[test]
    fn test_error_response_serialization() {
        let resp = ErrorResponse::new("SYS_CONFIG_KEY_NOT_FOUND", "not found");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["error"]["code"], "SYS_CONFIG_KEY_NOT_FOUND");
        assert_eq!(json["error"]["message"], "not found");
        assert!(json["error"]["request_id"].is_string());
        // details should be omitted when empty
        assert!(json["error"].get("details").is_none());
    }

    // 詳細情報付き ErrorResponse の JSON シリアライズで details が正しく出力されることを確認する。
    #[test]
    fn test_error_response_with_details_serialization() {
        let details = vec![ErrorDetail::new("field1", "invalid", "error1")];
        let resp =
            ErrorResponse::with_details("SYS_CONFIG_VALIDATION_FAILED", "validation", details);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["error"]["details"][0]["field"], "field1");
        assert_eq!(json["error"]["details"][0]["reason"], "invalid");
        assert_eq!(json["error"]["details"][0]["message"], "error1");
        assert_eq!(json["error"]["details"][0]["message"], "error1");
    }

    // unprocessable_entity・too_many_requests・service_unavailable の ErrorResponse 変換が正しいことを確認する。
    #[test]
    fn test_new_service_error_variants() {
        let err = ServiceError::unprocessable_entity("ACCT", "ledger is closed");
        let resp = err.to_error_response();
        assert_eq!(resp.error.code.as_str(), "SYS_ACCT_BUSINESS_RULE_VIOLATION");

        let err = ServiceError::too_many_requests("RATE", "rate limit exceeded");
        let resp = err.to_error_response();
        assert_eq!(resp.error.code.as_str(), "SYS_RATE_RATE_EXCEEDED");

        let err = ServiceError::service_unavailable("AUTH", "service unavailable");
        let resp = err.to_error_response();
        assert_eq!(resp.error.code.as_str(), "SYS_AUTH_SERVICE_UNAVAILABLE");
    }

    // Tenant サービスの既知エラーコードが正しい文字列を返すことを確認する。
    #[test]
    fn test_well_known_tenant_codes() {
        assert_eq!(tenant::not_found().as_str(), "SYS_TENANT_NOT_FOUND");
        assert_eq!(tenant::name_conflict().as_str(), "SYS_TENANT_NAME_CONFLICT");
    }

    // Session サービスの既知エラーコードが正しい文字列を返すことを確認する。
    #[test]
    fn test_well_known_session_codes() {
        assert_eq!(session::not_found().as_str(), "SYS_SESSION_NOT_FOUND");
        assert_eq!(session::expired().as_str(), "SYS_SESSION_EXPIRED");
        assert_eq!(
            session::max_devices_exceeded().as_str(),
            "SYS_SESSION_MAX_DEVICES_EXCEEDED"
        );
    }
}

/// gRPC 変換のテストモジュール。
/// tonic feature が有効な場合のみコンパイルされる。
#[cfg(all(test, feature = "grpc-auth"))]
mod grpc_tests {
    use super::*;

    // ServiceError::NotFound が tonic::Status::not_found に変換されることを確認する。
    #[test]
    fn test_service_error_not_found_to_grpc_status() {
        let err = ServiceError::not_found("TEST", "item not found");
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("item not found"));
    }

    // ServiceError::BadRequest が tonic::Status::invalid_argument に変換されることを確認する。
    #[test]
    fn test_service_error_bad_request_to_grpc_status() {
        let err = ServiceError::bad_request("TEST", "invalid input");
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    // ServiceError::Unauthorized が tonic::Status::unauthenticated に変換されることを確認する。
    #[test]
    fn test_service_error_unauthorized_to_grpc_status() {
        let err = ServiceError::unauthorized("TEST", "not authenticated");
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    // ServiceError::Forbidden が tonic::Status::permission_denied に変換されることを確認する。
    #[test]
    fn test_service_error_forbidden_to_grpc_status() {
        let err = ServiceError::forbidden("TEST", "access denied");
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::PermissionDenied);
    }

    // ServiceError::Conflict が tonic::Status::already_exists に変換されることを確認する。
    #[test]
    fn test_service_error_conflict_to_grpc_status() {
        let err = ServiceError::conflict("TEST", "already exists");
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::AlreadyExists);
    }

    // ServiceError::UnprocessableEntity が tonic::Status::failed_precondition に変換されることを確認する。
    #[test]
    fn test_service_error_unprocessable_to_grpc_status() {
        let err = ServiceError::unprocessable_entity("TEST", "business rule violation");
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::FailedPrecondition);
    }

    // ServiceError::TooManyRequests が tonic::Status::resource_exhausted に変換されることを確認する。
    #[test]
    fn test_service_error_too_many_requests_to_grpc_status() {
        let err = ServiceError::too_many_requests("TEST", "rate limit exceeded");
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::ResourceExhausted);
    }

    // ServiceError::Internal が tonic::Status::internal に変換されることを確認する。
    #[test]
    fn test_service_error_internal_to_grpc_status() {
        let err = ServiceError::internal("TEST", "internal error");
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    // ServiceError::ServiceUnavailable が tonic::Status::unavailable に変換されることを確認する。
    #[test]
    fn test_service_error_unavailable_to_grpc_status() {
        let err = ServiceError::service_unavailable("TEST", "service unavailable");
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unavailable);
    }

    // IntoGrpcStatus トレイトが ServiceError で正しく動作することを確認する。
    #[test]
    fn test_into_grpc_status_trait() {
        let err = ServiceError::not_found("TEST", "not found via trait");
        let status = err.into_grpc_status();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    // map_anyhow_to_grpc_status がドメインエラーのダウンキャストに成功する場合を確認する。
    #[test]
    fn test_map_anyhow_to_grpc_status_with_domain_error() {
        /// テスト用ドメインエラー
        #[derive(Debug, thiserror::Error)]
        enum TestDomainError {
            #[error("test not found: {0}")]
            NotFound(String),
        }

        impl From<TestDomainError> for ServiceError {
            fn from(err: TestDomainError) -> Self {
                match err {
                    TestDomainError::NotFound(msg) => ServiceError::NotFound {
                        code: ErrorCode::new("TEST_NOT_FOUND"),
                        message: msg,
                    },
                }
            }
        }

        let anyhow_err: anyhow::Error = TestDomainError::NotFound("item-1".to_string()).into();
        let status = map_anyhow_to_grpc_status::<TestDomainError>(anyhow_err);
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("item-1"));
    }

    // map_anyhow_to_grpc_status がダウンキャスト失敗時に internal を返すことを確認する。
    #[test]
    fn test_map_anyhow_to_grpc_status_fallback_to_internal() {
        /// テスト用ドメインエラー（ダウンキャスト対象ではない）
        #[derive(Debug, thiserror::Error)]
        enum UnrelatedError {
            #[error("unrelated: {0}")]
            Other(String),
        }

        impl From<UnrelatedError> for ServiceError {
            fn from(err: UnrelatedError) -> Self {
                match err {
                    UnrelatedError::Other(msg) => ServiceError::Internal {
                        code: ErrorCode::new("TEST_INTERNAL"),
                        message: msg,
                    },
                }
            }
        }

        let anyhow_err = anyhow::anyhow!("unknown error");
        let status = map_anyhow_to_grpc_status::<UnrelatedError>(anyhow_err);
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("unknown error"));
    }
}
