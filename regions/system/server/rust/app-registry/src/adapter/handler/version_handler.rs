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
use crate::usecase::version_selection::normalize_arch;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct VersionListResponse {
    pub versions: Vec<AppVersion>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct CreateVersionResponse {
    pub id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    get,
    path = "/api/v1/apps/{id}/versions",
    params(("id" = String, Path, description = "App ID")),
    responses(
        (status = 200, description = "Version list", body = VersionListResponse),
        (status = 404, description = "App not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_versions(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.list_versions_uc.execute(&id).await {
        Ok(versions) => (StatusCode::OK, Json(VersionListResponse { versions })).into_response(),
        Err(crate::usecase::list_versions::ListVersionsError::AppNotFound(_)) => {
            let err =
                ErrorResponse::new("SYS_APPS_APP_NOT_FOUND", "The specified app was not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", e.to_string());
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
    pub storage_key: String,
    pub release_notes: Option<String>,
    #[serde(default)]
    pub mandatory: bool,
    /// STATIC-CRITICAL-002: Cosign 署名（base64 エンコード）。
    /// cosign が有効な場合、検証に使用する。省略可（開発環境のみ）。
    pub cosign_signature: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/apps/{id}/versions",
    params(("id" = String, Path, description = "App ID")),
    request_body = CreateVersionRequest,
    responses(
        (status = 201, description = "Version created", body = CreateVersionResponse),
        (status = 400, description = "Bad request"),
        (status = 422, description = "Signature verification failed"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_version(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CreateVersionRequest>,
) -> impl IntoResponse {
    // STATIC-CRITICAL-002: 署名が提供された場合は Cosign で検証する
    if let Some(ref signature) = req.cosign_signature {
        match state
            .cosign_verifier
            .verify(&req.checksum_sha256, signature)
            .await
        {
            Ok(true) => {
                tracing::info!(
                    app_id = %id,
                    version = %req.version,
                    "Cosign 署名検証成功"
                );
            }
            Ok(false) => {
                tracing::warn!(
                    app_id = %id,
                    version = %req.version,
                    "Cosign 署名検証失敗: 無効な署名"
                );
                let err = ErrorResponse::new(
                    "SYS_APPS_SIGNATURE_INVALID",
                    "Cosign 署名の検証に失敗しました。署名が正しいか確認してください。",
                );
                return (StatusCode::UNPROCESSABLE_ENTITY, Json(err)).into_response();
            }
            Err(e) => {
                tracing::error!(
                    app_id = %id,
                    version = %req.version,
                    error = %e,
                    "Cosign 署名検証中にエラーが発生しました"
                );
                let err = ErrorResponse::new(
                    "SYS_APPS_SIGNATURE_VERIFY_ERROR",
                    format!("署名検証中にエラーが発生しました: {}", e),
                );
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response();
            }
        }
    }

    let version = AppVersion {
        id: uuid::Uuid::new_v4(),
        app_id: id,
        version: req.version,
        platform: req.platform,
        arch: normalize_arch(&req.arch),
        size_bytes: req.size_bytes,
        checksum_sha256: req.checksum_sha256,
        storage_key: req.storage_key,
        release_notes: req.release_notes,
        mandatory: req.mandatory,
        cosign_signature: req.cosign_signature,
        published_at: chrono::Utc::now(),
        created_at: chrono::Utc::now(),
    };

    match state.create_version_uc.execute(&version).await {
        Ok(created) => (
            StatusCode::CREATED,
            Json(CreateVersionResponse {
                id: created.id,
                created_at: created.created_at,
            }),
        )
            .into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_CREATE_VERSION_FAILED", e.to_string());
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

/// バージョン削除のクエリパラメータ。
#[derive(Debug, Deserialize)]
pub struct DeleteVersionQuery {
    pub platform: Option<String>,
    pub arch: Option<String>,
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
    let platform = match params.platform {
        Some(platform) => match platform.parse::<Platform>() {
            Ok(platform) => Some(platform),
            Err(_) => {
                let err = ErrorResponse::new(
                    "SYS_APPS_INVALID_PLATFORM",
                    "Invalid platform. Use: windows, linux, macos",
                );
                return (StatusCode::BAD_REQUEST, Json(err)).into_response();
            }
        },
        None => None,
    };
    let arch = params.arch.map(|arch| normalize_arch(&arch));

    match state
        .delete_version_uc
        .execute(&id, &version, platform.as_ref(), arch.as_deref())
        .await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(crate::usecase::delete_version::DeleteVersionError::AppNotFound(_)) => {
            let err =
                ErrorResponse::new("SYS_APPS_APP_NOT_FOUND", "The specified app was not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::delete_version::DeleteVersionError::VersionNotFound(_, _)) => {
            let err = ErrorResponse::new(
                "SYS_APPS_VERSION_NOT_FOUND",
                "The specified version was not found",
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::delete_version::DeleteVersionError::AmbiguousVersion(_, _)) => {
            let err = ErrorResponse::new(
                "SYS_APPS_CREATE_VERSION_FAILED",
                "Multiple platform-specific releases matched the requested version. Specify platform and arch.",
            );
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_APPS_INTERNAL_ERROR", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
