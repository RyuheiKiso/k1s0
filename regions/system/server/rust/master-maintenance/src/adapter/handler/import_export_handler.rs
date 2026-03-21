use axum::{
    extract::{Extension, Multipart, Path, Query, State},
    http::header::{CONTENT_DISPOSITION, CONTENT_TYPE},
    http::HeaderMap,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use k1s0_auth::Claims;
use serde::Deserialize;

use crate::adapter::handler::error::AppError;
use crate::adapter::handler::table_handler::DomainScopeQuery;
use crate::adapter::handler::{actor_from_claims, publish_change_event, AppState};

/// レコードインポートハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn import_records(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(ds_query): Query<DomainScopeQuery>,
    Json(data): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Claims が存在しない（未認証）場合は 401 を返す（P0-2 対応）。
    let claims_ext = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims_ext.0));
    let job = state
        .import_export_uc
        .import_records(&name, &data, &actor, ds_query.domain_scope.as_deref())
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
    Ok((StatusCode::CREATED, Json(job)))
}

/// ファイルインポートハンドラー。Claims が存在しない場合は 401 Unauthorized を返す。
pub async fn import_records_file(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(ds_query): Query<DomainScopeQuery>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // Claims が存在しない（未認証）場合は 401 を返す（P0-2 対応）。
    let claims_ext = claims
        .ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let actor = actor_from_claims(Some(&claims_ext.0));
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
        .import_records_from_file(
            &name,
            &file_name,
            &file_content,
            &actor,
            ds_query.domain_scope.as_deref(),
        )
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
    Ok((StatusCode::CREATED, Json(job)))
}

/// エクスポートハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn export_records(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(name): Path<String>,
    Query(query): Query<ExportQuery>,
) -> Result<impl IntoResponse, AppError> {
    // read 操作も認証が必要（P0-2 対応）。
    let _guard = claims.ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let result = state
        .import_export_uc
        .export_records(
            &name,
            query.format.as_deref(),
            query.domain_scope.as_deref(),
        )
        .await?;

    if let Some(file) = result.file {
        let mut headers = HeaderMap::new();
        // Content-Type ヘッダーを設定する
        headers.insert(
            CONTENT_TYPE,
            file.content_type.parse().map_err(|_| {
                AppError::internal(
                    "SYS_MM_EXPORT_HEADER_INVALID",
                    &format!("invalid content_type header value: {}", file.content_type),
                )
            })?,
        );
        // Content-Disposition ヘッダーを設定する
        headers.insert(
            CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", file.file_name)
                .parse()
                .map_err(|_| {
                    AppError::internal(
                        "SYS_MM_EXPORT_HEADER_INVALID",
                        &format!(
                            "invalid content_disposition header value: {}",
                            file.file_name
                        ),
                    )
                })?,
        );
        Ok((StatusCode::OK, headers, file.bytes).into_response())
    } else {
        Ok(Json(result.as_json()).into_response())
    }
}

/// インポートジョブ取得ハンドラー。read 操作も認証必須（P0-2 対応）。
pub async fn get_import_job(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // read 操作も認証が必要（P0-2 対応）。
    let _guard = claims.ok_or_else(|| AppError::unauthorized("SYS_MM_AUTH_REQUIRED", "authentication required"))?;
    let job = state
        .import_export_uc
        .get_import_job(id)
        .await?
        .ok_or_else(|| {
            AppError::not_found("SYS_MM_IMPORT_JOB_NOT_FOUND", "Import job not found")
        })?;
    Ok(Json(job))
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
    pub domain_scope: Option<String>,
}
