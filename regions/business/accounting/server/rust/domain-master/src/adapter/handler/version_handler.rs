use crate::adapter::handler::error::from_anyhow;
use crate::adapter::handler::AppState;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use k1s0_server_common::ServiceError;

pub async fn list_versions(
    State(state): State<AppState>,
    Path((category_code, item_code)): Path<(String, String)>,
) -> Result<impl IntoResponse, ServiceError> {
    let versions = state
        .get_item_versions_uc
        .list_versions(&category_code, &item_code)
        .await
        .map_err(from_anyhow)?;
    Ok(Json(serde_json::json!({ "versions": versions })))
}
