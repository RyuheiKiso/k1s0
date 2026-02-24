use axum::{http::StatusCode, Json};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub request_id: String,
}

impl ApiError {
    pub fn not_found(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ApiError {
            code: "SYS_APIREG_NOT_FOUND".to_string(),
            message: message.into(),
            request_id: uuid::Uuid::new_v4().to_string(),
        };
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": err })),
        )
    }

    pub fn bad_request(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ApiError {
            code: "SYS_APIREG_BAD_REQUEST".to_string(),
            message: message.into(),
            request_id: uuid::Uuid::new_v4().to_string(),
        };
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": err })),
        )
    }

    pub fn conflict(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ApiError {
            code: "SYS_APIREG_CONFLICT".to_string(),
            message: message.into(),
            request_id: uuid::Uuid::new_v4().to_string(),
        };
        (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": err })),
        )
    }

    pub fn internal(message: impl Into<String>) -> (StatusCode, Json<serde_json::Value>) {
        let err = ApiError {
            code: "SYS_APIREG_INTERNAL".to_string(),
            message: message.into(),
            request_id: uuid::Uuid::new_v4().to_string(),
        };
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": err })),
        )
    }
}
