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
            Json(serde_json::to_value(&err).unwrap()),
        )
    }

    pub fn bad_request(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::bad_request(), message);
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(&err).unwrap()),
        )
    }

    pub fn conflict(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::conflict(), message);
        (
            StatusCode::CONFLICT,
            Json(serde_json::to_value(&err).unwrap()),
        )
    }

    pub fn unauthorized(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::unauthorized(), message);
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::to_value(&err).unwrap()),
        )
    }

    pub fn internal(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ErrorResponse::new(codes::api_registry::internal_error(), message);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(&err).unwrap()),
        )
    }
}

/// Convert a ServiceError into the (StatusCode, Json<Value>) tuple format
/// used by the api-registry handlers.
pub fn service_error_to_response(err: ServiceError) -> (StatusCode, Json<serde_json::Value>) {
    let status = match &err {
        ServiceError::NotFound { .. } => StatusCode::NOT_FOUND,
        ServiceError::BadRequest { .. } => StatusCode::BAD_REQUEST,
        ServiceError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
        ServiceError::Forbidden { .. } => StatusCode::FORBIDDEN,
        ServiceError::Conflict { .. } => StatusCode::CONFLICT,
        ServiceError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let body = err.to_error_response();
    (status, Json(serde_json::to_value(&body).unwrap()))
}
