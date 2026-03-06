use crate::adapter::presenter::response::{ErrorDetail, ErrorResponse};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug)]
pub struct AppError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AppError {
    pub fn not_found(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn bad_request(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn conflict(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn unauthorized(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn forbidden(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn internal(code: &str, message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            error: ErrorDetail {
                code: self.code,
                message: self.message,
                request_id: None,
                details: self.details,
            },
        };
        (self.status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        let msg = err.to_string();
        let lower = msg.to_ascii_lowercase();

        if lower.contains("not found") {
            if lower.contains("table") {
                return Self::not_found("SYS_MM_TABLE_NOT_FOUND", &msg);
            }
            if lower.contains("record") {
                return Self::not_found("SYS_MM_RECORD_NOT_FOUND", &msg);
            }
            if lower.contains("rule") {
                return Self::not_found("SYS_MM_RULE_NOT_FOUND", &msg);
            }
            if lower.contains("display config") {
                return Self::not_found("SYS_MM_DISPLAY_CONFIG_NOT_FOUND", &msg);
            }
            if lower.contains("import job") {
                return Self::not_found("SYS_MM_IMPORT_JOB_NOT_FOUND", &msg);
            }
            if lower.contains("relationship") {
                return Self::not_found("SYS_MM_RELATIONSHIP_NOT_FOUND", &msg);
            }
            return Self::not_found("SYS_MM_NOT_FOUND", &msg);
        }
        if lower.contains("delete not allowed")
            || (lower.contains("delete") && lower.contains("not allowed"))
        {
            return Self::forbidden("SYS_AUTH_PERMISSION_DENIED", &msg);
        }
        if lower.contains("duplicate table")
            || (lower.contains("table") && lower.contains("already exists"))
        {
            return Self::conflict("SYS_MM_DUPLICATE_TABLE", &msg);
        }
        if lower.contains("duplicate column")
            || (lower.contains("column") && lower.contains("already exists"))
        {
            return Self::conflict("SYS_MM_DUPLICATE_COLUMN", &msg);
        }
        if lower.contains("invalid rule") {
            return Self::bad_request("SYS_MM_INVALID_RULE", &msg);
        }
        if lower.contains("import") && lower.contains("fail") {
            return Self::bad_request("SYS_MM_IMPORT_FAILED", &msg);
        }
        if lower.contains("sql") && lower.contains("build") {
            return Self::internal("SYS_MM_INTERNAL_ERROR", &msg);
        }
        if lower.contains("validation") {
            return Self::bad_request("SYS_MM_VALIDATION_ERROR", &msg);
        }

        Self::internal("SYS_MM_INTERNAL_ERROR", &msg)
    }
}
