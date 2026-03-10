use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use super::{AppState, ErrorResponse};
use crate::domain::entity::platform::Platform;
use crate::domain::entity::version::AppVersion;

#[utoipa::path(
    get,
    path = "/api/v1/apps/{id}/versions",
    params(("id" = String, Path, description = "App ID")),
    responses(
        (status = 200, description = "Version list", body = Vec<AppVersion>),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_versions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.list_versions_uc.execute(&id).await {
        Ok(versions) => {
            (StatusCode::OK, Json(serde_json::to_value(versions).unwrap())).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// バージョン作成のリクエストボディ。
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateVersionRequest {
    pub version: String,
    pub platform: Platform,
    pub arch: String,
    pub size_bytes: Option<i64>,
    pub checksum_sha256: String,
    pub s3_key: String,
    pub release_notes: Option<String>,
    #[serde(default)]
    pub mandatory: bool,
}

#[utoipa::path(
    post,
    path = "/api/v1/apps/{id}/versions",
    params(("id" = String, Path, description = "App ID")),
    request_body = CreateVersionRequest,
    responses(
        (status = 201, description = "Version created", body = AppVersion),
        (status = 400, description = "Bad request"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_version(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateVersionRequest>,
) -> impl IntoResponse {
    let version = AppVersion {
        id: uuid::Uuid::new_v4(),
        app_id: id,
        version: req.version,
        platform: req.platform,
        arch: req.arch,
        size_bytes: req.size_bytes,
        checksum_sha256: req.checksum_sha256,
        s3_key: req.s3_key,
        release_notes: req.release_notes,
        mandatory: req.mandatory,
        published_at: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
    };

    match state.create_version_uc.execute(&version).await {
        Ok(created) => {
            (StatusCode::CREATED, Json(serde_json::to_value(created).unwrap())).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_CREATE_VERSION_FAILED", &e.to_string());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

/// バージョン削除のクエリパラメータ。
#[derive(Debug, Deserialize)]
pub struct DeleteVersionQuery {
    pub platform: String,
    pub arch: String,
}

#[utoipa::path(
    delete,
    path = "/api/v1/apps/{id}/versions/{version}",
    params(
        ("id" = String, Path, description = "App ID"),
        ("version" = String, Path, description = "Version string"),
    ),
    responses(
        (status = 204, description = "Version deleted"),
        (status = 404, description = "Version not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_version(
    State(state): State<AppState>,
    Path((id, version)): Path<(String, String)>,
    Query(params): Query<DeleteVersionQuery>,
) -> impl IntoResponse {
    let platform: Platform = match params.platform.parse() {
        Ok(p) => p,
        Err(_) => {
            let err = ErrorResponse::new(
                "SYS_APPS_INVALID_PLATFORM",
                "Invalid platform. Use: windows, linux, macos",
            );
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    };

    match state
        .delete_version_uc
        .execute(&id, &version, &platform, &params.arch)
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_VERSION_NOT_FOUND", &e.to_string());
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
    }
}
