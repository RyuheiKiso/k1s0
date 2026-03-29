use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use super::AppState;
use crate::usecase::send_notification::SendNotificationError;
use crate::usecase::send_notification::SendNotificationInput;
use k1s0_server_common::error as codes;
use k1s0_server_common::ErrorResponse;

/// POST /api/v1/notifications - Send a notification
pub async fn send_notification(
    State(state): State<AppState>,
    Json(req): Json<SendNotificationRequest>,
) -> impl IntoResponse {
    if req.channel_id.trim().is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            codes::notification::invalid_id(),
            "channel_id is required",
        );
    }

    let channel_id = req.channel_id;
    let template_id = req.template_id;

    let input = SendNotificationInput {
        channel_id,
        template_id,
        recipient: req.recipient,
        subject: req.subject,
        body: req.body.unwrap_or_default(),
        template_variables: req.template_variables,
    };

    match state.send_notification_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "notification_id": output.log_id,
                "status": output.status,
                "created_at": output.created_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(SendNotificationError::ChannelNotFound(id)) => error_response(
            StatusCode::NOT_FOUND,
            codes::notification::channel_not_found(),
            format!("channel not found: {}", id),
        ),
        Err(SendNotificationError::TemplateNotFound(id)) => error_response(
            StatusCode::NOT_FOUND,
            codes::notification::template_not_found(),
            format!("template not found: {}", id),
        ),
        Err(SendNotificationError::ChannelDisabled(id)) => error_response(
            StatusCode::BAD_REQUEST,
            codes::notification::channel_disabled(),
            format!("channel disabled: {}", id),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            codes::notification::send_failed(),
            e.to_string(),
        ),
    }
}

/// GET /api/v1/notifications - List notification logs with pagination
pub async fn list_notifications(
    State(state): State<AppState>,
    Query(params): Query<ListNotificationsParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    // page_size を 1〜200 にクランプして異常値（0やオーバーフロー）を防ぐ（H-07 監査対応）
    let page_size = params.page_size.unwrap_or(20).clamp(1, 200);

    let channel_id = params.channel_id;

    match state
        .log_repo
        .find_all_paginated(page, page_size, channel_id, params.status)
        .await
    {
        Ok((logs, total_count)) => {
            let has_next = (page as u64 * page_size as u64) < total_count;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "notifications": logs,
                    "pagination": {
                        "total_count": total_count,
                        "page": page,
                        "page_size": page_size,
                        "has_next": has_next
                    }
                })),
            )
                .into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            codes::notification::list_failed(),
            e.to_string(),
        ),
    }
}

/// GET /api/v1/notifications/:id - Get a single notification log
pub async fn get_notification(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.log_repo.find_by_id(&id).await {
        Ok(Some(log)) => {
            let channel_type = match state.get_channel_uc.execute(&log.channel_id).await {
                Ok(channel) => Some(channel.channel_type),
                Err(_) => None,
            };

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "id": log.id,
                    "channel_id": log.channel_id,
                    "channel_type": channel_type,
                    "template_id": log.template_id,
                    "recipient": log.recipient,
                    "subject": log.subject,
                    "body": log.body,
                    "status": log.status,
                    "retry_count": log.retry_count,
                    "error_message": log.error_message,
                    "sent_at": log.sent_at,
                    "created_at": log.created_at,
                })),
            )
                .into_response()
        }
        Ok(None) => error_response(
            StatusCode::NOT_FOUND,
            codes::notification::not_found(),
            format!("notification not found: {}", id),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            codes::notification::get_failed(),
            e.to_string(),
        ),
    }
}

/// POST /api/v1/notifications/:id/retry
pub async fn retry_notification(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    use crate::usecase::retry_notification::RetryNotificationInput;

    let input = RetryNotificationInput {
        notification_id: id,
    };

    match state.retry_notification_uc.execute(&input).await {
        Ok(log) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "notification_id": log.id,
                "status": log.status,
                "message": "notification retried successfully"
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                error_response(
                    StatusCode::NOT_FOUND,
                    codes::notification::not_found(),
                    &msg,
                )
            } else if msg.contains("already sent") {
                error_response(
                    StatusCode::CONFLICT,
                    codes::notification::already_sent(),
                    &msg,
                )
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::notification::retry_failed(),
                    &msg,
                )
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

    if !is_valid_channel_type(&req.channel_type) {
        return error_response(
            StatusCode::BAD_REQUEST,
            codes::notification::validation_error(),
            format!(
                "invalid channel_type: {} (allowed: email, slack, webhook, sms, push)",
                req.channel_type
            ),
        );
    }

    // H-012 監査対応: tenant_id を指定してチャンネルを作成する
    // TODO: JWT クレームからテナント ID を取得する（ADR-0056 ロードマップ参照）
    // 現時点ではシステム共通チャンネル（tenant_id='system'）として作成する
    let input = CreateChannelInput {
        name: req.name,
        channel_type: req.channel_type,
        config: req.config,
        tenant_id: "system".to_string(),
        enabled: req.enabled,
    };

    match state.create_channel_uc.execute(&input).await {
        Ok(channel) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": channel.id.to_string(),
                "name": channel.name,
                "channel_type": channel.channel_type,
                "config": strip_sensitive_config(&channel.config),
                "enabled": channel.enabled,
                "created_at": channel.created_at.to_rfc3339(),
                "updated_at": channel.updated_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(crate::usecase::create_channel::CreateChannelError::Validation(msg)) => error_response(
            StatusCode::BAD_REQUEST,
            codes::notification::validation_error(),
            msg,
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            codes::notification::channel_create_failed(),
            e.to_string(),
        ),
    }
}

/// GET /api/v1/channels
pub async fn list_channels(
    State(state): State<AppState>,
    Query(params): Query<ListChannelsParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    // page_size を 1〜200 にクランプして異常値（0やオーバーフロー）を防ぐ（H-07 監査対応）
    let page_size = params.page_size.unwrap_or(20).clamp(1, 200);
    let enabled_only = params.enabled_only.unwrap_or(false);

    match state
        .list_channels_uc
        .execute_paginated(page, page_size, params.channel_type, enabled_only)
        .await
    {
        Ok((channels, total_count)) => {
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
            let has_next = (page as u64 * page_size as u64) < total_count;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "channels": items,
                    "pagination": {
                        "total_count": total_count,
                        "page": page,
                        "page_size": page_size,
                        "has_next": has_next
                    }
                })),
            )
                .into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            codes::notification::channel_list_failed(),
            e.to_string(),
        ),
    }
}

/// GET /api/v1/channels/:id
pub async fn get_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.get_channel_uc.execute(&id).await {
        Ok(channel) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "id": channel.id.to_string(),
                "name": channel.name,
                "channel_type": channel.channel_type,
                "config": strip_sensitive_config(&channel.config),
                "enabled": channel.enabled,
                "created_at": channel.created_at.to_rfc3339(),
                "updated_at": channel.updated_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                error_response(
                    StatusCode::NOT_FOUND,
                    codes::notification::channel_not_found(),
                    &msg,
                )
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::notification::channel_get_failed(),
                    &msg,
                )
            }
        }
    }
}

/// PUT /api/v1/channels/:id
pub async fn update_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
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
                error_response(
                    StatusCode::NOT_FOUND,
                    codes::notification::channel_not_found(),
                    &msg,
                )
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::notification::channel_update_failed(),
                    &msg,
                )
            }
        }
    }
}

/// DELETE /api/v1/channels/:id
pub async fn delete_channel(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.delete_channel_uc.execute(&id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(
                serde_json::json!({"success": true, "message": format!("channel {} deleted", id)}),
            ),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                error_response(
                    StatusCode::NOT_FOUND,
                    codes::notification::channel_not_found(),
                    &msg,
                )
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::notification::channel_delete_failed(),
                    &msg,
                )
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
        Err(crate::usecase::create_template::CreateTemplateError::Validation(msg)) => {
            error_response(
                StatusCode::BAD_REQUEST,
                codes::notification::validation_error(),
                msg,
            )
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            codes::notification::template_create_failed(),
            e.to_string(),
        ),
    }
}

/// GET /api/v1/templates
pub async fn list_templates(
    State(state): State<AppState>,
    Query(params): Query<ListTemplatesParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    // page_size を 1〜200 にクランプして異常値（0やオーバーフロー）を防ぐ（H-07 監査対応）
    let page_size = params.page_size.unwrap_or(20).clamp(1, 200);

    match state
        .list_templates_uc
        .execute_paginated(page, page_size, params.channel_type)
        .await
    {
        Ok((templates, total_count)) => {
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
            let has_next = (page as u64 * page_size as u64) < total_count;
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "templates": items,
                    "pagination": {
                        "total_count": total_count,
                        "page": page,
                        "page_size": page_size,
                        "has_next": has_next
                    }
                })),
            )
                .into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            codes::notification::template_list_failed(),
            e.to_string(),
        ),
    }
}

/// GET /api/v1/templates/:id
pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
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
                error_response(
                    StatusCode::NOT_FOUND,
                    codes::notification::template_not_found(),
                    &msg,
                )
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::notification::template_get_failed(),
                    &msg,
                )
            }
        }
    }
}

/// PUT /api/v1/templates/:id
pub async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
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
                error_response(
                    StatusCode::NOT_FOUND,
                    codes::notification::template_not_found(),
                    &msg,
                )
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::notification::template_update_failed(),
                    &msg,
                )
            }
        }
    }
}

/// DELETE /api/v1/templates/:id
pub async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.delete_template_uc.execute(&id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(
                serde_json::json!({"success": true, "message": format!("template {} deleted", id)}),
            ),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                error_response(
                    StatusCode::NOT_FOUND,
                    codes::notification::template_not_found(),
                    &msg,
                )
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    codes::notification::template_delete_failed(),
                    &msg,
                )
            }
        }
    }
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct SendNotificationRequest {
    pub channel_id: String,
    pub template_id: Option<String>,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub template_variables: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct ListNotificationsParams {
    pub channel_id: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ListChannelsParams {
    pub channel_type: Option<String>,
    pub enabled_only: Option<bool>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ListTemplatesParams {
    pub channel_type: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
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

fn error_response(
    status: StatusCode,
    code: impl Into<k1s0_server_common::ErrorCode>,
    message: impl Into<String>,
) -> Response {
    let err = ErrorResponse::new(code, message);
    (status, Json(err)).into_response()
}

fn strip_sensitive_config(config: &serde_json::Value) -> serde_json::Value {
    const SENSITIVE_KEYS: &[&str] = &[
        "password",
        "passwd",
        "secret",
        "token",
        "api_key",
        "apikey",
        "access_token",
        "refresh_token",
    ];

    match config {
        serde_json::Value::Object(map) => {
            let mut sanitized = serde_json::Map::new();
            for (key, value) in map {
                if SENSITIVE_KEYS
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(key.as_str()))
                {
                    continue;
                }
                sanitized.insert(key.clone(), strip_sensitive_config(value));
            }
            serde_json::Value::Object(sanitized)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(strip_sensitive_config).collect())
        }
        _ => config.clone(),
    }
}

fn is_valid_channel_type(channel_type: &str) -> bool {
    matches!(channel_type, "email" | "slack" | "webhook" | "sms" | "push")
}
