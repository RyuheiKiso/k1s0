use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::Claims;
use serde::Deserialize;

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
        .import_records(&name, &data, &actor, None)
        .await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "import_job",
            "resource_id": job.id,
            "resource_name": name,
            "action": "completed",
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

pub async fn import_records_file(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let actor = actor_from_claims(claims.as_ref().map(|Extension(claims)| claims));
    let mut file_name: Option<String> = None;
    let mut file_content: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| AppError::bad_request("SYS_MM_IMPORT_MULTIPART_INVALID", &err.to_string()))?
    {
        if field.name() == Some("file") {
            file_name = field.file_name().map(ToString::to_string);
            file_content = Some(
                field
                    .bytes()
                    .await
                    .map_err(|err| {
                        AppError::bad_request("SYS_MM_IMPORT_MULTIPART_INVALID", &err.to_string())
                    })?
                    .to_vec(),
            );
            break;
        }
    }

    let file_name = file_name.ok_or_else(|| {
        AppError::bad_request("SYS_MM_IMPORT_FILE_REQUIRED", "file field is required")
    })?;
    let file_content = file_content.ok_or_else(|| {
        AppError::bad_request("SYS_MM_IMPORT_FILE_REQUIRED", "file field is required")
    })?;

    let job = state
        .import_export_uc
        .import_records_from_file(&name, &file_name, &file_content, &actor, None)
        .await?;
    publish_change_event(
        &state,
        serde_json::json!({
            "event_type": "MASTER_MAINTENANCE_DATA_CHANGED",
            "resource_type": "import_job",
            "resource_id": job.id,
            "resource_name": name,
            "action": "completed",
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
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, AppError> {
    let result = state
        .import_export_uc
        .export_records(&name, query.format.as_deref(), None)
        .await?;

    if matches!(query.format.as_deref(), Some("csv")) {
        let content = result
            .get("content")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string();
        Ok((
            StatusCode::OK,
            [("content-type", "text/csv; charset=utf-8")],
            content,
        )
            .into_response())
    } else {
        Ok(Json(result).into_response())
    }
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

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}
