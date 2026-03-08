use crate::adapter::handler::error::AppError;
use crate::adapter::handler::{actor_from_claims, publish_change_event, AppState};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use k1s0_auth::Claims;

pub async fn list_relationships(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let relationships = state.manage_relationships_uc.list_relationships().await?;
    Ok(Json(serde_json::to_value(relationships).unwrap()))
}

pub async fn create_relationship(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(input): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let relationship = state
        .manage_relationships_uc
        .create_relationship(&input, &actor, None)
        .await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "relationship",
            "resource_id": relationship.id,
            "action": "created",
            "actor": actor,
            "after": relationship.clone(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    )
    .await;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(relationship).unwrap()),
    ))
}

pub async fn update_relationship(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(input): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let relationship = state
        .manage_relationships_uc
        .update_relationship(id, &input)
        .await?;
    Ok(Json(serde_json::to_value(relationship).unwrap()))
}

pub async fn delete_relationship(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, AppError> {
    state
        .manage_relationships_uc
        .delete_relationship(id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_related_records(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, AppError> {
    let related = state
        .manage_relationships_uc
        .get_related_records(&name, &id, None)
        .await?;
    Ok(Json(related))
}
