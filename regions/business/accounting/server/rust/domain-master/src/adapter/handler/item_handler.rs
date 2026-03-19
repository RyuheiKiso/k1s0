use crate::adapter::handler::error::from_anyhow;
use crate::adapter::handler::AppState;
use crate::domain::entity::master_item::{CreateMasterItem, UpdateMasterItem};
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
pub struct ListItemsQuery {
    pub active_only: Option<bool>,
}

pub async fn list_items(
    State(state): State<AppState>,
    Path(category_code): Path<String>,
    Query(query): Query<ListItemsQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    let items = state
        .manage_items_uc
        .list_items(&category_code, query.active_only.unwrap_or(false))
        .await
        .map_err(from_anyhow)?;
    Ok(Json(serde_json::json!({ "items": items })))
}

pub async fn get_item(
    State(state): State<AppState>,
    Path((category_code, item_code)): Path<(String, String)>,
) -> Result<impl IntoResponse, ServiceError> {
    let item = state
        .manage_items_uc
        .get_item(&category_code, &item_code)
        .await
        .map_err(from_anyhow)?
        .ok_or_else(|| ServiceError::NotFound {
            code: k1s0_server_common::ErrorCode::new("BIZ_DOMAINMASTER_ITEM_NOT_FOUND"),
            message: format!(
                "Item '{}' not found in category '{}'",
                item_code, category_code
            ),
        })?;
    Ok(Json(item))
}

pub async fn create_item(
    State(state): State<AppState>,
    Path(category_code): Path<String>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<CreateMasterItem>,
) -> Result<impl IntoResponse, ServiceError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let item = state
        .manage_items_uc
        .create_item(&category_code, &input, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn update_item(
    State(state): State<AppState>,
    Path((category_code, item_code)): Path<(String, String)>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<UpdateMasterItem>,
) -> Result<impl IntoResponse, ServiceError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let item = state
        .manage_items_uc
        .update_item(&category_code, &item_code, &input, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok(Json(item))
}

pub async fn delete_item(
    State(state): State<AppState>,
    Path((category_code, item_code)): Path<(String, String)>,
    claims: Option<Extension<Claims>>,
) -> Result<StatusCode, ServiceError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    state
        .manage_items_uc
        .delete_item(&category_code, &item_code, &actor)
        .await
        .map_err(from_anyhow)?;
    Ok(StatusCode::NO_CONTENT)
}
