use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use k1s0_auth::Claims;

use crate::adapter::handler::error::AppError;
use crate::adapter::handler::{actor_from_claims, publish_change_event, AppState};

pub async fn import_records(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Json(data): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let job = state
        .import_export_uc
        .import_records(&name, &data, &actor)
        .await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "import_job",
            "resource_id": job.id,
            "resource_name": name,
            "action": "started",
            "actor": actor,
            "after": job.clone(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }),
    )
    .await;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(job).unwrap()),
    ))
}

pub async fn export_records(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = state.import_export_uc.export_records(&name).await?;
    Ok(Json(result))
}

pub async fn get_import_job(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let job = state
        .import_export_uc
        .get_import_job(id)
        .await?
        .ok_or_else(|| {
            AppError::not_found("SYS_MM_IMPORT_JOB_NOT_FOUND", "Import job not found")
        })?;
    Ok(Json(serde_json::to_value(job).unwrap()))
}
