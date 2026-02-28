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

/// ErrorCode represents a structured error code for the system tier.
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
    pub fn not_found(service: &str) -> Self {
        Self(format!("SYS_{}_NOT_FOUND", service.to_uppercase()))
    }

    /// Create a standard "validation failed" error code for a service.
    pub fn validation(service: &str) -> Self {
        Self(format!("SYS_{}_VALIDATION_FAILED", service.to_uppercase()))
    }

    /// Create a standard "internal error" error code for a service.
    pub fn internal(service: &str) -> Self {
        Self(format!("SYS_{}_INTERNAL_ERROR", service.to_uppercase()))
    }

    /// Create a standard "unauthorized" error code for a service.
    pub fn unauthorized(service: &str) -> Self {
        Self(format!("SYS_{}_UNAUTHORIZED", service.to_uppercase()))
    }

    /// Create a standard "forbidden" error code for a service.
    pub fn forbidden(service: &str) -> Self {
        Self(format!("SYS_{}_PERMISSION_DENIED", service.to_uppercase()))
    }

    /// Create a standard "conflict" error code for a service.
    pub fn conflict(service: &str) -> Self {
        Self(format!("SYS_{}_CONFLICT", service.to_uppercase()))
    }

    /// Create a standard "unprocessable entity" error code for a service.
    pub fn unprocessable(service: &str) -> Self {
        Self(format!(
            "SYS_{}_BUSINESS_RULE_VIOLATION",
            service.to_uppercase()
        ))
    }

    /// Create a standard "rate exceeded" error code for a service.
    pub fn rate_exceeded(service: &str) -> Self {
        Self(format!("SYS_{}_RATE_EXCEEDED", service.to_uppercase()))
    }

    /// Create a standard "service unavailable" error code for a service.
    pub fn service_unavailable(service: &str) -> Self {
        Self(format!(
            "SYS_{}_SERVICE_UNAVAILABLE",
            service.to_uppercase()
        ))
    }

    /// Return the error code string.
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

/// ErrorDetail provides additional context for an error field.
///
/// Follows the REST-API設計.md D-007 specification:
/// `{ "field": "quantity", "reason": "must_be_positive", "message": "..." }`
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

/// ErrorBody is the structured error payload.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<ErrorDetail>,
}

/// ErrorResponse wraps ErrorBody in an `{ "error": ... }` envelope.
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
                request_id: uuid::Uuid::new_v4().to_string(),
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
                request_id: uuid::Uuid::new_v4().to_string(),
                details,
            },
        }
    }
}

/// ServiceError is a high-level error type that maps to HTTP status codes.
///
/// Each variant carries a structured error code and message.
/// When the `axum` feature is enabled, ServiceError implements `IntoResponse`.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// 404 Not Found
    #[error("{message}")]
    NotFound {
        code: ErrorCode,
        message: String,
    },

    /// 400 Bad Request
    #[error("{message}")]
    BadRequest {
        code: ErrorCode,
        message: String,
        details: Vec<ErrorDetail>,
    },

    /// 401 Unauthorized
    #[error("{message}")]
    Unauthorized {
        code: ErrorCode,
        message: String,
    },

    /// 403 Forbidden
    #[error("{message}")]
    Forbidden {
        code: ErrorCode,
        message: String,
    },

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
    TooManyRequests {
        code: ErrorCode,
        message: String,
    },

    /// 500 Internal Server Error
    #[error("{message}")]
    Internal {
        code: ErrorCode,
        message: String,
    },

    /// 503 Service Unavailable
    #[error("{message}")]
    ServiceUnavailable {
        code: ErrorCode,
        message: String,
    },
}

impl ServiceError {
    /// Create a NotFound error for a service.
    pub fn not_found(service: &str, message: impl Into<String>) -> Self {
        Self::NotFound {
            code: ErrorCode::not_found(service),
            message: message.into(),
        }
    }

    /// Create a BadRequest error for a service.
    pub fn bad_request(service: &str, message: impl Into<String>) -> Self {
        Self::BadRequest {
            code: ErrorCode::validation(service),
            message: message.into(),
            details: vec![],
        }
    }

    /// Create a BadRequest error with field-level details.
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

    /// Create an UnprocessableEntity error for a service (business rule violation).
    pub fn unprocessable_entity(service: &str, message: impl Into<String>) -> Self {
        Self::UnprocessableEntity {
            code: ErrorCode::unprocessable(service),
            message: message.into(),
            details: vec![],
        }
    }

    /// Create a TooManyRequests error for a service (rate limit exceeded).
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

    /// Create a ServiceUnavailable error for a service.
    pub fn service_unavailable(service: &str, message: impl Into<String>) -> Self {
        Self::ServiceUnavailable {
            code: ErrorCode::service_unavailable(service),
            message: message.into(),
        }
    }

    /// Convert to an ErrorResponse.
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
            ServiceError::ServiceUnavailable { .. } => {
                axum::http::StatusCode::SERVICE_UNAVAILABLE
            }
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

// --- Well-known error codes for system tier services ---

/// Well-known error codes for the Auth service.
pub mod auth {
    use super::ErrorCode;

    pub fn missing_claims() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_MISSING_CLAIMS")
    }

    pub fn permission_denied() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_PERMISSION_DENIED")
    }

    pub fn unauthorized() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_UNAUTHORIZED")
    }

    pub fn token_expired() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_TOKEN_EXPIRED")
    }

    pub fn invalid_token() -> ErrorCode {
        ErrorCode::new("SYS_AUTH_INVALID_TOKEN")
    }
}

/// Well-known error codes for the Config service.
pub mod config {
    use super::ErrorCode;

    pub fn key_not_found() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_KEY_NOT_FOUND")
    }

    pub fn service_not_found() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_SERVICE_NOT_FOUND")
    }

    pub fn schema_not_found() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_SCHEMA_NOT_FOUND")
    }

    pub fn version_conflict() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_VERSION_CONFLICT")
    }

    pub fn validation_failed() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_VALIDATION_FAILED")
    }

    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_CONFIG_INTERNAL_ERROR")
    }
}

/// Well-known error codes for the DLQ Manager service.
pub mod dlq {
    use super::ErrorCode;

    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_DLQ_NOT_FOUND")
    }

    pub fn validation_error() -> ErrorCode {
        ErrorCode::new("SYS_DLQ_VALIDATION_ERROR")
    }

    pub fn conflict() -> ErrorCode {
        ErrorCode::new("SYS_DLQ_CONFLICT")
    }

    pub fn process_failed() -> ErrorCode {
        ErrorCode::new("SYS_DLQ_PROCESS_FAILED")
    }

    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_DLQ_INTERNAL_ERROR")
    }
}

/// Well-known error codes for the Tenant service.
pub mod tenant {
    use super::ErrorCode;

    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_NOT_FOUND")
    }

    pub fn name_conflict() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_NAME_CONFLICT")
    }

    pub fn invalid_status() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_INVALID_STATUS")
    }

    pub fn invalid_input() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_INVALID_INPUT")
    }

    pub fn member_conflict() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_MEMBER_CONFLICT")
    }

    pub fn member_not_found() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_MEMBER_NOT_FOUND")
    }

    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_TENANT_INTERNAL_ERROR")
    }
}

/// Well-known error codes for the Session service.
pub mod session {
    use super::ErrorCode;

    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_NOT_FOUND")
    }

    pub fn expired() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_EXPIRED")
    }

    pub fn revoked() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_REVOKED")
    }

    pub fn invalid_input() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_INVALID_INPUT")
    }

    pub fn too_many_sessions() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_TOO_MANY")
    }

    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_SESSION_INTERNAL_ERROR")
    }
}

/// Well-known error codes for the API Registry service.
pub mod api_registry {
    use super::ErrorCode;

    pub fn not_found() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_NOT_FOUND")
    }

    pub fn bad_request() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_BAD_REQUEST")
    }

    pub fn conflict() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_CONFLICT")
    }

    pub fn unauthorized() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_UNAUTHORIZED")
    }

    pub fn schema_invalid() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_SCHEMA_INVALID")
    }

    pub fn internal_error() -> ErrorCode {
        ErrorCode::new("SYS_APIREG_INTERNAL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_not_found() {
        let code = ErrorCode::not_found("CONFIG");
        assert_eq!(code.as_str(), "SYS_CONFIG_NOT_FOUND");
    }

    #[test]
    fn test_error_code_validation() {
        let code = ErrorCode::validation("DLQ");
        assert_eq!(code.as_str(), "SYS_DLQ_VALIDATION_FAILED");
    }

    #[test]
    fn test_error_code_from_str() {
        let code = ErrorCode::from("SYS_AUTH_MISSING_CLAIMS");
        assert_eq!(code.as_str(), "SYS_AUTH_MISSING_CLAIMS");
    }

    #[test]
    fn test_error_response_new() {
        let resp = ErrorResponse::new("SYS_CONFIG_KEY_NOT_FOUND", "config key not found");
        assert_eq!(resp.error.code.as_str(), "SYS_CONFIG_KEY_NOT_FOUND");
        assert_eq!(resp.error.message, "config key not found");
        assert!(!resp.error.request_id.is_empty());
        assert!(resp.error.details.is_empty());
    }

    #[test]
    fn test_error_response_with_details() {
        let details = vec![
            ErrorDetail::new("namespace", "required", "must not be empty"),
            ErrorDetail::new("key", "invalid_format", "invalid format"),
        ];
        let resp = ErrorResponse::with_details(
            "SYS_CONFIG_VALIDATION_FAILED",
            "validation failed",
            details,
        );
        assert_eq!(resp.error.details.len(), 2);
        assert_eq!(resp.error.details[0].field, "namespace");
        assert_eq!(resp.error.details[0].reason, "required");
    }

    #[test]
    fn test_service_error_not_found() {
        let err = ServiceError::not_found("CONFIG", "key 'system.auth/jwt_secret' not found");
        let resp = err.to_error_response();
        assert_eq!(resp.error.code.as_str(), "SYS_CONFIG_NOT_FOUND");
    }

    #[test]
    fn test_service_error_bad_request_with_details() {
        let details = vec![ErrorDetail::new("page", "must_be_positive", "must be >= 1")];
        let err =
            ServiceError::bad_request_with_details("CONFIG", "validation failed", details);
        let resp = err.to_error_response();
        assert_eq!(resp.error.code.as_str(), "SYS_CONFIG_VALIDATION_FAILED");
        assert_eq!(resp.error.details.len(), 1);
        assert_eq!(resp.error.details[0].reason, "must_be_positive");
    }

    #[test]
    fn test_well_known_auth_codes() {
        assert_eq!(auth::missing_claims().as_str(), "SYS_AUTH_MISSING_CLAIMS");
        assert_eq!(
            auth::permission_denied().as_str(),
            "SYS_AUTH_PERMISSION_DENIED"
        );
        assert_eq!(auth::unauthorized().as_str(), "SYS_AUTH_UNAUTHORIZED");
    }

    #[test]
    fn test_well_known_config_codes() {
        assert_eq!(
            config::key_not_found().as_str(),
            "SYS_CONFIG_KEY_NOT_FOUND"
        );
        assert_eq!(
            config::version_conflict().as_str(),
            "SYS_CONFIG_VERSION_CONFLICT"
        );
    }

    #[test]
    fn test_well_known_dlq_codes() {
        assert_eq!(dlq::not_found().as_str(), "SYS_DLQ_NOT_FOUND");
        assert_eq!(dlq::process_failed().as_str(), "SYS_DLQ_PROCESS_FAILED");
    }

    #[test]
    fn test_well_known_api_registry_codes() {
        assert_eq!(api_registry::not_found().as_str(), "SYS_APIREG_NOT_FOUND");
        assert_eq!(api_registry::conflict().as_str(), "SYS_APIREG_CONFLICT");
    }

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

    #[test]
    fn test_error_response_with_details_serialization() {
        let details = vec![ErrorDetail::new("field1", "invalid", "error1")];
        let resp =
            ErrorResponse::with_details("SYS_CONFIG_VALIDATION_FAILED", "validation", details);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["error"]["details"][0]["field"], "field1");
        assert_eq!(json["error"]["details"][0]["reason"], "invalid");
        assert_eq!(json["error"]["details"][0]["message"], "error1");
    }

    #[test]
    fn test_new_service_error_variants() {
        let err = ServiceError::unprocessable_entity("ACCT", "ledger is closed");
        let resp = err.to_error_response();
        assert_eq!(
            resp.error.code.as_str(),
            "SYS_ACCT_BUSINESS_RULE_VIOLATION"
        );

        let err = ServiceError::too_many_requests("RATE", "rate limit exceeded");
        let resp = err.to_error_response();
        assert_eq!(resp.error.code.as_str(), "SYS_RATE_RATE_EXCEEDED");

        let err = ServiceError::service_unavailable("AUTH", "service unavailable");
        let resp = err.to_error_response();
        assert_eq!(
            resp.error.code.as_str(),
            "SYS_AUTH_SERVICE_UNAVAILABLE"
        );
    }

    #[test]
    fn test_well_known_tenant_codes() {
        assert_eq!(
            tenant::not_found().as_str(),
            "SYS_TENANT_NOT_FOUND"
        );
        assert_eq!(
            tenant::name_conflict().as_str(),
            "SYS_TENANT_NAME_CONFLICT"
        );
    }

    #[test]
    fn test_well_known_session_codes() {
        assert_eq!(
            session::not_found().as_str(),
            "SYS_SESSION_NOT_FOUND"
        );
        assert_eq!(session::expired().as_str(), "SYS_SESSION_EXPIRED");
        assert_eq!(
            session::too_many_sessions().as_str(),
            "SYS_SESSION_TOO_MANY"
        );
    }
}
