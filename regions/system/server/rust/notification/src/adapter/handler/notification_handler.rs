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

/// POST /api/v1/notifications/:id/retry
pub async fn retry_notification(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    use crate::usecase::retry_notification::RetryNotificationInput;

    let input = RetryNotificationInput {
        notification_id: id,
    };

    match state.retry_notification_uc.execute(&input).await {
        Ok(log) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "log_id": log.id.to_string(),
                "status": log.status,
                "message": "notification retried successfully"
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_NOTIF_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else if msg.contains("already sent") {
                let err = ErrorResponse::new("SYS_NOTIF_ALREADY_SENT", &msg);
                (StatusCode::CONFLICT, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_NOTIF_RETRY_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/channels
pub async fn create_channel(
    State(state): State<AppState>,
    Json(req): Json<CreateChannelRequest>,
) -> impl IntoResponse {
    use crate::usecase::create_channel::CreateChannelInput;

    let input = CreateChannelInput {
        name: req.name,
        channel_type: req.channel_type,
        config: req.config,
        enabled: req.enabled,
    };

    match state.create_channel_uc.execute(&input).await {
        Ok(channel) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": channel.id.to_string(),
                "name": channel.name,
                "channel_type": channel.channel_type,
                "enabled": channel.enabled,
                "created_at": channel.created_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_NOTIF_CHANNEL_CREATE_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/channels
pub async fn list_channels(State(state): State<AppState>) -> impl IntoResponse {
    match state.list_channels_uc.execute().await {
        Ok(channels) => {
            let items: Vec<serde_json::Value> = channels
                .into_iter()
                .map(|ch| {
                    serde_json::json!({
                        "id": ch.id.to_string(),
                        "name": ch.name,
                        "channel_type": ch.channel_type,
                        "enabled": ch.enabled,
                        "created_at": ch.created_at.to_rfc3339()
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({ "channels": items })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_NOTIF_CHANNEL_LIST_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/channels/:id
pub async fn get_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.get_channel_uc.execute(&id).await {
        Ok(channel) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": channel.id.to_string(),
                "name": channel.name,
                "channel_type": channel.channel_type,
                "config": channel.config,
                "enabled": channel.enabled,
                "created_at": channel.created_at.to_rfc3339(),
                "updated_at": channel.updated_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_NOTIF_CHANNEL_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_NOTIF_CHANNEL_GET_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// PUT /api/v1/channels/:id
pub async fn update_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateChannelRequest>,
) -> impl IntoResponse {
    use crate::usecase::update_channel::UpdateChannelInput;

    let input = UpdateChannelInput {
        id,
        name: req.name,
        enabled: req.enabled,
        config: req.config,
    };

    match state.update_channel_uc.execute(&input).await {
        Ok(channel) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": channel.id.to_string(),
                "name": channel.name,
                "channel_type": channel.channel_type,
                "enabled": channel.enabled,
                "updated_at": channel.updated_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_NOTIF_CHANNEL_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_NOTIF_CHANNEL_UPDATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// DELETE /api/v1/channels/:id
pub async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.delete_channel_uc.execute(&id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"success": true, "message": format!("channel {} deleted", id)})),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_NOTIF_CHANNEL_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_NOTIF_CHANNEL_DELETE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/templates
pub async fn create_template(
    State(state): State<AppState>,
    Json(req): Json<CreateTemplateRequest>,
) -> impl IntoResponse {
    use crate::usecase::create_template::CreateTemplateInput;

    let input = CreateTemplateInput {
        name: req.name,
        channel_type: req.channel_type,
        subject_template: req.subject_template,
        body_template: req.body_template,
    };

    match state.create_template_uc.execute(&input).await {
        Ok(template) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": template.id.to_string(),
                "name": template.name,
                "channel_type": template.channel_type,
                "created_at": template.created_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_NOTIF_TEMPLATE_CREATE_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/templates
pub async fn list_templates(State(state): State<AppState>) -> impl IntoResponse {
    match state.list_templates_uc.execute().await {
        Ok(templates) => {
            let items: Vec<serde_json::Value> = templates
                .into_iter()
                .map(|t| {
                    serde_json::json!({
                        "id": t.id.to_string(),
                        "name": t.name,
                        "channel_type": t.channel_type,
                        "created_at": t.created_at.to_rfc3339()
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({ "templates": items })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_NOTIF_TEMPLATE_LIST_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/templates/:id
pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.get_template_uc.execute(&id).await {
        Ok(template) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": template.id.to_string(),
                "name": template.name,
                "channel_type": template.channel_type,
                "subject_template": template.subject_template,
                "body_template": template.body_template,
                "created_at": template.created_at.to_rfc3339(),
                "updated_at": template.updated_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_NOTIF_TEMPLATE_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_NOTIF_TEMPLATE_GET_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// PUT /api/v1/templates/:id
pub async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTemplateRequest>,
) -> impl IntoResponse {
    use crate::usecase::update_template::UpdateTemplateInput;

    let input = UpdateTemplateInput {
        id,
        name: req.name,
        subject_template: req.subject_template,
        body_template: req.body_template,
    };

    match state.update_template_uc.execute(&input).await {
        Ok(template) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": template.id.to_string(),
                "name": template.name,
                "channel_type": template.channel_type,
                "updated_at": template.updated_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_NOTIF_TEMPLATE_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_NOTIF_TEMPLATE_UPDATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// DELETE /api/v1/templates/:id
pub async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.delete_template_uc.execute(&id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({"success": true, "message": format!("template {} deleted", id)})),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_NOTIF_TEMPLATE_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_NOTIF_TEMPLATE_DELETE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
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

#[derive(Debug, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    pub channel_type: String,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub channel_type: String,
    pub subject_template: Option<String>,
    pub body_template: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub subject_template: Option<String>,
    pub body_template: Option<String>,
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
