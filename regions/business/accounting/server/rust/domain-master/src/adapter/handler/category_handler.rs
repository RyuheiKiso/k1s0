use crate::adapter::handler::error::from_anyhow;
use crate::adapter::handler::AppState;
use crate::domain::entity::master_category::{CreateMasterCategory, UpdateMasterCategory};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::actor_from_claims;
use k1s0_auth::Claims;
use k1s0_server_common::ServiceError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListCategoriesQuery {
    pub active_only: Option<bool>,
}

pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let postgres_ok = state
        .manage_categories_uc
        .list_categories(false)
        .await
        .is_ok();
    let status = if postgres_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(serde_json::json!({
            "status": if postgres_ok { "ready" } else { "not_ready" },
            "checks": {
                "postgres": if postgres_ok { "ok" } else { "error" }
            }
        })),
    )
}

pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

pub async fn list_categories(
    State(state): State<AppState>,
    Query(query): Query<ListCategoriesQuery>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let categories = state
        .manage_categories_uc
        .list_categories(query.active_only.unwrap_or(false))
        .await
        .map_err(from_anyhow)?;
    Ok(Json(serde_json::json!({ "categories": categories })))
}

pub async fn get_category(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let category = state
        .manage_categories_uc
        .get_category(&code)
        .await
        .map_err(from_anyhow)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_CATEGORY_NOT_FOUND"),
            message: format!("Category '{}' not found", code),
        })?;
    Ok(Json(serde_json::to_value(category).unwrap()))
}

pub async fn create_category(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<CreateMasterCategory>,
) -> Result<(StatusCode, Json<serde_json::Value>), ServiceError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let category = state
        .manage_categories_uc
        .create_category(&input, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(category).unwrap()),
    ))
}

pub async fn update_category(
    State(state): State<AppState>,
    Path(code): Path<String>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<UpdateMasterCategory>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let category = state
        .manage_categories_uc
        .update_category(&code, &actor, &input)
        .await
        .map_err(from_anyhow)?;
    Ok(Json(serde_json::to_value(category).unwrap()))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(code): Path<String>,
    claims: Option<Extension<Claims>>,
) -> Result<StatusCode, ServiceError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    state
        .manage_categories_uc
        .delete_category(&code, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok(StatusCode::NO_CONTENT)
}
