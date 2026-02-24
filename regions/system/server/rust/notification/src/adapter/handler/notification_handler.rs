use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::usecase::send_notification::SendNotificationInput;

/// POST /api/v1/notifications - Send a notification
pub async fn send_notification(
    State(state): State<AppState>,
    Json(req): Json<SendNotificationRequest>,
) -> impl IntoResponse {
    let channel_id = match Uuid::parse_str(&req.channel_id) {
        Ok(id) => id,
        Err(_) => {
            let err = ErrorResponse::new("SYS_NOTIF_INVALID_ID", "invalid channel_id format");
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    };

    let input = SendNotificationInput {
        channel_id,
        recipient: req.recipient,
        subject: req.subject,
        body: req.body,
    };

    match state.send_notification_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "log_id": output.log_id.to_string(),
                "status": output.status
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_NOTIF_CHANNEL_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else if msg.contains("disabled") {
                let err = ErrorResponse::new("SYS_NOTIF_CHANNEL_DISABLED", &msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_NOTIF_SEND_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// GET /api/v1/notifications - List notification logs (by channel_id query param)
pub async fn list_notifications(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<ListNotificationsParams>,
) -> impl IntoResponse {
    if let Some(channel_id_str) = params.channel_id {
        let channel_id = match Uuid::parse_str(&channel_id_str) {
            Ok(id) => id,
            Err(_) => {
                let err = ErrorResponse::new("SYS_NOTIF_INVALID_ID", "invalid channel_id format");
                return (StatusCode::BAD_REQUEST, Json(serde_json::to_value(err).unwrap()))
                    .into_response();
            }
        };

        match state.log_repo.find_by_channel_id(&channel_id).await {
            Ok(logs) => (StatusCode::OK, Json(serde_json::json!({ "notifications": logs })))
                .into_response(),
            Err(e) => {
                let err = ErrorResponse::new("SYS_NOTIF_LIST_FAILED", &e.to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::to_value(err).unwrap()))
                    .into_response()
            }
        }
    } else {
        // No filter -- return empty for now (no find_all on log repo)
        (
            StatusCode::OK,
            Json(serde_json::json!({ "notifications": [], "message": "provide channel_id query parameter to filter" })),
        )
            .into_response()
    }
}

/// GET /api/v1/notifications/:id - Get a single notification log
pub async fn get_notification(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.log_repo.find_by_id(&id).await {
        Ok(Some(log)) => (StatusCode::OK, Json(serde_json::to_value(log).unwrap())).into_response(),
        Ok(None) => {
            let err = ErrorResponse::new(
                "SYS_NOTIF_NOT_FOUND",
                &format!("notification not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_NOTIF_GET_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct SendNotificationRequest {
    pub channel_id: String,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct ListNotificationsParams {
    pub channel_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
            },
        }
    }
}
