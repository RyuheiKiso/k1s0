use axum::{http::StatusCode, Json};

use k1s0_server_common::error as codes;
use k1s0_server_common::{ErrorResponse, ServiceError};

/// ApiError provides convenience constructors for api-registry error responses.
///
/// Error codes follow the `SYS_APIREG_*` pattern via k1s0-server-common.
pub struct ApiError;

impl ApiError {
    pub fn not_found(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::not_found(), message);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }

    pub fn schema_not_found(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::schema_not_found(), message);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }

    pub fn version_not_found(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::version_not_found(), message);
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }

    pub fn bad_request(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::bad_request(), message);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }

    pub fn conflict(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::conflict(), message);
        (
            StatusCode::CONFLICT,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }

    pub fn already_exists(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::already_exists(), message);
        (
            StatusCode::CONFLICT,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }

    pub fn cannot_delete_latest(
        message: impl Into<String>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::cannot_delete_latest(), message);
        (
            StatusCode::CONFLICT,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }

    pub fn unprocessable_entity(
        message: impl Into<String>,
    ) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::schema_invalid(), message);
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }

    pub fn validator_error(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::validator_error(), message);
        (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }

    pub fn unauthorized(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::unauthorized(), message);
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }

    pub fn internal(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::internal_error(), message);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(&err).expect("ErrorResponseのJSON変換に失敗")),
        )
    }
}

/// ServiceErrorを(StatusCode, Json<Value>)タプル形式に変換する
/// api-registryハンドラで使用するヘルパー関数。
pub fn service_error_to_response(err: ServiceError) -> (StatusCode, Json<serde_json::Value>) {
    let status = match &err {
        ServiceError::NotFound { .. } => StatusCode::NOT_FOUND,
        ServiceError::BadRequest { .. } => StatusCode::BAD_REQUEST,
        ServiceError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
        ServiceError::Forbidden { .. } => StatusCode::FORBIDDEN,
        ServiceError::Conflict { .. } => StatusCode::CONFLICT,
        ServiceError::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        ServiceError::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
        ServiceError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        ServiceError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
    };
    let body = err.to_error_response();
    (status, Json(serde_json::to_value(&body).expect("ServiceErrorのJSON変換に失敗")))
}
