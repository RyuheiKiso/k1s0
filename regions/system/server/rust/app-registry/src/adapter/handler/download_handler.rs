use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;

use super::{AppState, ErrorResponse};
use crate::domain::entity::claims::Claims;
use crate::domain::entity::platform::Platform;
use crate::domain::entity::version::AppVersion;
use crate::usecase::generate_download_url::DownloadUrlResult;

/// プラットフォーム・アーキテクチャのクエリパラメータ。
#[derive(Debug, Deserialize)]
pub struct PlatformQuery {
    pub platform: String,
    pub arch: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/apps/{id}/latest",
    params(
        ("id" = String, Path, description = "App ID"),
        ("platform" = String, Query, description = "Platform: windows, linux, macos"),
        ("arch" = String, Query, description = "Architecture: amd64, arm64"),
    ),
    responses(
        (status = 200, description = "Latest version", body = AppVersion),
        (status = 404, description = "No version found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_latest(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<PlatformQuery>,
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
        .get_latest_uc
        .execute(&id, &platform, &params.arch)
        .await
    {
        Ok(version) => {
            (StatusCode::OK, Json(serde_json::to_value(version).unwrap())).into_response()
        }
        Err(crate::usecase::get_latest::GetLatestError::NotFound(_, _, _)) => {
            let err = ErrorResponse::new(
                "SYS_APPS_VERSION_NOT_FOUND",
                "No version found for the specified platform and architecture",
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/apps/{id}/versions/{version}/download",
    params(
        ("id" = String, Path, description = "App ID"),
        ("version" = String, Path, description = "Version string"),
        ("platform" = String, Query, description = "Platform: windows, linux, macos"),
        ("arch" = String, Query, description = "Architecture: amd64, arm64"),
    ),
    responses(
        (status = 200, description = "Download URL generated", body = DownloadUrlResult),
        (status = 404, description = "Version not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn download_version(
    State(state): State<AppState>,
    Path((id, version)): Path<(String, String)>,
    Query(params): Query<PlatformQuery>,
    Extension(claims): Extension<Claims>,
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
        .generate_download_url_uc
        .execute(&id, &version, &platform, &params.arch, &claims.sub)
        .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(crate::usecase::generate_download_url::GenerateDownloadUrlError::NotFound(
            _,
            _,
            _,
            _,
        )) => {
            let err = ErrorResponse::new(
                "SYS_APPS_VERSION_NOT_FOUND",
                "Version not found for the specified platform and architecture",
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
