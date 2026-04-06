// ステータス定義 REST ハンドラ。
// CRITICAL-BIZ-003 対応: Claims から actor_id を取得して "system" ハードコードを排除する。
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension,
    Json,
};
use k1s0_auth::Claims;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{error::map_domain_error, AppState};
use crate::domain::entity::status_definition::{
    CreateStatusDefinition, StatusDefinition, StatusDefinitionFilter, StatusTransition,
    UpdateStatusDefinition,
};

#[derive(Debug, Deserialize)]
pub struct CreateStatusDefinitionRequest {
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    // serde が JSON 配列を Vec<StatusTransition> へ直接デシリアライズする
    pub allowed_transitions: Option<Vec<StatusTransition>>,
    pub is_initial: Option<bool>,
    pub is_terminal: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct StatusDefinitionResponse {
    pub id: String,
    pub project_type_id: String,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub is_initial: bool,
    pub is_terminal: bool,
    pub sort_order: i32,
    pub created_by: String,
}

impl From<StatusDefinition> for StatusDefinitionResponse {
    fn from(s: StatusDefinition) -> Self {
        Self {
            id: s.id.to_string(),
            project_type_id: s.project_type_id.to_string(),
            code: s.code,
            display_name: s.display_name,
            description: s.description,
            color: s.color,
            is_initial: s.is_initial,
            is_terminal: s.is_terminal,
            sort_order: s.sort_order,
            created_by: s.created_by,
        }
    }
}

pub async fn list_status_definitions(
    State(state): State<AppState>,
    Path(project_type_id): Path<Uuid>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let filter = StatusDefinitionFilter {
        project_type_id: Some(project_type_id),
        ..Default::default()
    };
    state
        .manage_status_definitions_uc
        .list(&filter)
        .await
        .map(|(items, _)| {
            let resp: Vec<StatusDefinitionResponse> = items.into_iter().map(Into::into).collect();
            Json(resp)
        })
        .map_err(map_domain_error)
}

pub async fn get_status_definition(
    State(state): State<AppState>,
    Path((_project_type_id, status_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state
        .manage_status_definitions_uc
        .get(status_id)
        .await
        .map_err(map_domain_error)?
        .map(|s| Json(StatusDefinitionResponse::from(s)))
        .ok_or_else(|| map_domain_error(anyhow::anyhow!("not found")))
}

pub async fn create_status_definition(
    State(state): State<AppState>,
    // CRITICAL-BIZ-003 対応: Claims から actor_id を取得して "system" ハードコードを排除する
    claims: Option<Extension<Claims>>,
    Path(project_type_id): Path<Uuid>,
    Json(body): Json<CreateStatusDefinitionRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // JWT の preferred_username > email > sub の優先順位でアクター ID を解決する
    let actor = k1s0_auth::claims::actor_from_claims(claims.as_ref().map(|c| &c.0));
    let input = CreateStatusDefinition {
        project_type_id,
        code: body.code,
        display_name: body.display_name,
        description: body.description,
        color: body.color,
        allowed_transitions: body.allowed_transitions,
        is_initial: body.is_initial,
        is_terminal: body.is_terminal,
        sort_order: body.sort_order,
    };
    state
        .manage_status_definitions_uc
        .create(&input, &actor)
        .await
        .map(|s| (StatusCode::CREATED, Json(StatusDefinitionResponse::from(s))))
        .map_err(map_domain_error)
}

pub async fn update_status_definition(
    State(state): State<AppState>,
    // CRITICAL-BIZ-003 対応: Claims から actor_id を取得して "system" ハードコードを排除する
    claims: Option<Extension<Claims>>,
    Path((_project_type_id, status_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // JWT の preferred_username > email > sub の優先順位でアクター ID を解決する
    let actor = k1s0_auth::claims::actor_from_claims(claims.as_ref().map(|c| &c.0));
    let input = UpdateStatusDefinition {
        display_name: body.get("display_name").and_then(|v| v.as_str()).map(String::from),
        description: body.get("description").and_then(|v| v.as_str()).map(String::from),
        color: body.get("color").and_then(|v| v.as_str()).map(String::from),
        // JSON ボディの allowed_transitions を Vec<StatusTransition> へデシリアライズする
        // 解析失敗時は None として扱い、既存値を保持する
        allowed_transitions: body.get("allowed_transitions")
            .and_then(|v| serde_json::from_value(v.clone()).ok()),
        is_initial: body.get("is_initial").and_then(|v| v.as_bool()),
        is_terminal: body.get("is_terminal").and_then(|v| v.as_bool()),
        sort_order: body.get("sort_order").and_then(|v| v.as_i64()).map(|v| v as i32),
        change_reason: body.get("change_reason").and_then(|v| v.as_str()).map(String::from),
    };
    state
        .manage_status_definitions_uc
        .update(status_id, &input, &actor)
        .await
        .map(|s| Json(StatusDefinitionResponse::from(s)))
        .map_err(map_domain_error)
}

pub async fn delete_status_definition(
    State(state): State<AppState>,
    Path((_project_type_id, status_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state
        .manage_status_definitions_uc
        .delete(status_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(map_domain_error)
}
