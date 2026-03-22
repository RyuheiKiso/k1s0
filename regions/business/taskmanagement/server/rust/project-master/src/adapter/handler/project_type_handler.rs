// プロジェクトタイプ REST ハンドラ。
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{error::map_domain_error, AppState};
use crate::domain::entity::project_type::{
    CreateProjectType, ProjectType, UpdateProjectType,
};

#[derive(Debug, Deserialize)]
pub struct CreateProjectTypeRequest {
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub default_workflow: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ProjectTypeResponse {
    pub id: String,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
}

impl From<ProjectType> for ProjectTypeResponse {
    fn from(pt: ProjectType) -> Self {
        Self {
            id: pt.id.to_string(),
            code: pt.code,
            display_name: pt.display_name,
            description: pt.description,
            is_active: pt.is_active,
            sort_order: pt.sort_order,
            created_by: pt.created_by,
        }
    }
}

/// プロジェクトタイプ一覧を取得する
pub async fn list_project_types(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    use crate::domain::entity::project_type::ProjectTypeFilter;
    let filter = ProjectTypeFilter::default();
    state
        .manage_project_types_uc
        .list(&filter)
        .await
        .map(|(items, _total)| {
            let resp: Vec<ProjectTypeResponse> = items.into_iter().map(Into::into).collect();
            Json(resp)
        })
        .map_err(map_domain_error)
}

/// プロジェクトタイプを取得する
pub async fn get_project_type(
    State(state): State<AppState>,
    Path(project_type_id): Path<Uuid>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state
        .manage_project_types_uc
        .get(project_type_id)
        .await
        .map_err(map_domain_error)?
        .map(|pt| Json(ProjectTypeResponse::from(pt)))
        .ok_or_else(|| map_domain_error(anyhow::anyhow!("not found")))
}

/// プロジェクトタイプを作成する
pub async fn create_project_type(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectTypeRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let input = CreateProjectType {
        code: body.code,
        display_name: body.display_name,
        description: body.description,
        default_workflow: body.default_workflow,
        is_active: body.is_active,
        sort_order: body.sort_order,
    };
    state
        .manage_project_types_uc
        .create(&input, "system")
        .await
        .map(|pt| (StatusCode::CREATED, Json(ProjectTypeResponse::from(pt))))
        .map_err(map_domain_error)
}

/// プロジェクトタイプを更新する
pub async fn update_project_type(
    State(state): State<AppState>,
    Path(project_type_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let input = UpdateProjectType {
        display_name: body.get("display_name").and_then(|v| v.as_str()).map(String::from),
        description: body.get("description").and_then(|v| v.as_str()).map(String::from),
        default_workflow: body.get("default_workflow").cloned(),
        is_active: body.get("is_active").and_then(|v| v.as_bool()),
        sort_order: body.get("sort_order").and_then(|v| v.as_i64()).map(|v| v as i32),
    };
    state
        .manage_project_types_uc
        .update(project_type_id, &input, "system")
        .await
        .map(|pt| Json(ProjectTypeResponse::from(pt)))
        .map_err(map_domain_error)
}

/// プロジェクトタイプを削除する
pub async fn delete_project_type(
    State(state): State<AppState>,
    Path(project_type_id): Path<Uuid>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    state
        .manage_project_types_uc
        .delete(project_type_id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(map_domain_error)
}
