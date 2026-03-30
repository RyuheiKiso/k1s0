use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use k1s0_server_common::error as codes;
use k1s0_server_common::ErrorResponse;
use serde::Deserialize;
use std::collections::HashMap;

use super::AppState;
use crate::usecase::delete_file::DeleteFileInput;
use crate::usecase::generate_download_url::GenerateDownloadUrlInput;
use crate::usecase::generate_upload_url::GenerateUploadUrlInput;
use crate::usecase::get_file_metadata::GetFileMetadataInput;
use crate::usecase::list_files::ListFilesInput;

fn is_storage_error_message(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    [
        "storage",
        "s3",
        "bucket",
        "object store",
        "presign",
        "upload url",
        "download url",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

/// POST /api/v1/files - Generate upload URL (initiate upload)
pub async fn upload_file(
    State(state): State<AppState>,
    Json(req): Json<UploadFileRequest>,
) -> impl IntoResponse {
    let input = GenerateUploadUrlInput {
        filename: req.filename,
        size_bytes: req.size_bytes,
        content_type: req.content_type,
        tenant_id: req.tenant_id,
        uploaded_by: req.uploaded_by,
        tags: req.tags.unwrap_or_default(),
        expires_in_seconds: req.expires_in_seconds.unwrap_or(3600),
    };

    match state.generate_upload_url_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "file_id": output.file_id,
                "upload_url": output.upload_url,
                "expires_in_seconds": output.expires_in_seconds
            })),
        )
            .into_response(),
        Err(crate::usecase::generate_upload_url::GenerateUploadUrlError::Validation(msg)) => {
            let err = ErrorResponse::new(codes::file::validation(), &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(crate::usecase::generate_upload_url::GenerateUploadUrlError::SizeExceeded {
            actual,
            max,
        }) => {
            let err = ErrorResponse::new(
                codes::file::size_exceeded(),
                format!("file size exceeds limit: {} > {}", actual, max),
            );
            (StatusCode::PAYLOAD_TOO_LARGE, Json(err)).into_response()
        }
        Err(crate::usecase::generate_upload_url::GenerateUploadUrlError::Internal(msg)) => {
            if is_storage_error_message(&msg) {
                let err = ErrorResponse::new(codes::file::storage_error(), &msg);
                (StatusCode::BAD_GATEWAY, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new(codes::file::upload_failed(), &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// GET /api/v1/files/:id - Get file metadata and download URL
pub async fn get_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // MED-11 監査対応: claims が None（認証なし）の場合は 401 を返す（防御的プログラミング）
    let claims = match claims {
        Some(axum::extract::Extension(c)) => c,
        None => {
            let err = ErrorResponse::new(codes::file::access_denied(), "authentication required");
            return (StatusCode::UNAUTHORIZED, Json(err)).into_response();
        }
    };

    // MED-11 監査対応: sys_admin ロールの場合はテナント照合をスキップする
    let is_admin = claims.realm_roles().iter().any(|role| role == "sys_admin");

    let metadata_input = GetFileMetadataInput {
        file_id: id.clone(),
    };

    match state.get_file_metadata_uc.execute(&metadata_input).await {
        Ok(file) => {
            if let Some(request_tenant_id) = tenant_id_from_headers(&headers) {
                // MED-11 監査対応: X-Tenant-ID ヘッダーと JWT Claims のテナント ID が一致しない場合は 403 を返す
                if !is_admin && claims.tenant_id() != request_tenant_id {
                    let err = ErrorResponse::new(
                        codes::file::access_denied(),
                        "tenant id mismatch between header and token",
                    );
                    return (StatusCode::FORBIDDEN, Json(err)).into_response();
                }

                // storage_path のプレフィックス（テナントID）とリクエストヘッダーのテナントIDを比較してアクセス制御を行う
                // FileMetadata から tenant_id フィールドが削除されたため、storage_path から取得する
                let resource_tenant_id =
                    crate::domain::service::FileDomainService::tenant_id_from_storage_path(
                        &file.storage_path,
                    )
                    .unwrap_or("");
                if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                    resource_tenant_id,
                    request_tenant_id,
                ) {
                    let err = ErrorResponse::new(
                        codes::file::access_denied(),
                        "access denied for tenant",
                    );
                    return (StatusCode::FORBIDDEN, Json(err)).into_response();
                }
            }
            // Also generate a download URL if file is available
            let download_url = if file.status == "available" {
                let dl_input = GenerateDownloadUrlInput {
                    file_id: id,
                    expires_in_seconds: 3600,
                };
                state
                    .generate_download_url_uc
                    .execute(&dl_input)
                    .await
                    .ok()
                    .map(|o| o.download_url)
            } else {
                None
            };

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "file": file_to_rest_detail(&file),
                    "download_url": download_url
                })),
            )
                .into_response()
        }
        // MED-02 監査対応: 文字列マッチングをやめ、型安全なエラー型で HTTP ステータスを決定する
        Err(crate::usecase::get_file_metadata::GetFileMetadataError::NotFound(msg)) => {
            let err = ErrorResponse::new(codes::file::not_found(), &msg);
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::get_file_metadata::GetFileMetadataError::Internal(msg)) => {
            let err = ErrorResponse::new(codes::file::get_failed(), &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/files - List files
pub async fn list_files(
    State(state): State<AppState>,
    Query(params): Query<ListFilesParams>,
) -> impl IntoResponse {
    let input = ListFilesInput {
        tenant_id: params.tenant_id,
        uploaded_by: params.uploaded_by,
        content_type: params.content_type,
        tag: params.tag.as_deref().and_then(parse_tag_query),
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
    };

    match state.list_files_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "files": output.files.iter().map(file_to_rest_summary).collect::<Vec<_>>(),
                "total_count": output.total_count,
                "page": output.page,
                "page_size": output.page_size,
                "has_next": output.has_next
            })),
        )
            .into_response(),
        Err(e) => {
            let err = ErrorResponse::new(codes::file::list_failed(), e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// DELETE /api/v1/files/:id
pub async fn delete_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // CRIT-02 監査対応: claims が None（認証なし）の場合は 401 を返す（防御的プログラミング）
    // このエンドポイントは認証が必須であり、JWTミドルウェアが何らかの理由でスキップされた場合も保護する
    let claims = match claims {
        Some(axum::extract::Extension(c)) => c,
        None => {
            let err = ErrorResponse::new(codes::file::access_denied(), "authentication required");
            return (StatusCode::UNAUTHORIZED, Json(err)).into_response();
        }
    };

    // テナントIDをヘッダーから取得する（CRIT-01 でテナント条件付き DELETE に使用）
    let request_tenant_id = tenant_id_from_headers(&headers).unwrap_or("").to_string();

    if let Ok(file) = state
        .get_file_metadata_uc
        .execute(&GetFileMetadataInput {
            file_id: id.clone(),
        })
        .await
    {
        if !request_tenant_id.is_empty() {
            // storage_path のプレフィックス（テナントID）とリクエストヘッダーのテナントIDを比較してアクセス制御を行う
            // FileMetadata から tenant_id フィールドが削除されたため、storage_path から取得する
            let resource_tenant_id =
                crate::domain::service::FileDomainService::tenant_id_from_storage_path(
                    &file.storage_path,
                )
                .unwrap_or("");
            if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                resource_tenant_id,
                &request_tenant_id,
            ) {
                let err =
                    ErrorResponse::new(codes::file::access_denied(), "access denied for tenant");
                return (StatusCode::FORBIDDEN, Json(err)).into_response();
            }
        }

        let is_admin = claims.realm_roles().iter().any(|role| role == "sys_admin");
        if !is_admin && file.uploaded_by != claims.sub {
            let err = ErrorResponse::new(
                codes::file::access_denied(),
                "only the file owner or sys_admin can delete this file",
            );
            return (StatusCode::FORBIDDEN, Json(err)).into_response();
        }
    }

    // CRIT-02 監査対応: claims.sub（認証済みユーザーID）を所有者確認に使用する
    // CRIT-01 監査対応: テナントIDと所有者IDを DELETE 条件に追加してアトミックな認可チェックを実現する
    let is_admin = claims.realm_roles().iter().any(|role| role == "sys_admin");
    let input = DeleteFileInput {
        file_id: id,
        tenant_id: request_tenant_id,
        // sys_admin は全ファイルを削除可能なため所有者チェックをスキップする
        expected_uploader: if is_admin {
            None
        } else {
            Some(claims.sub.clone())
        },
    };

    // MED-02 監査対応: 文字列マッチングをやめ、型安全なエラー型で HTTP ステータスを決定する
    match state.delete_file_uc.execute(&input).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(crate::usecase::delete_file::DeleteFileError::NotFound(msg)) => {
            let err = ErrorResponse::new(codes::file::not_found(), &msg);
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::delete_file::DeleteFileError::Internal(msg)) => {
            let err = ErrorResponse::new(codes::file::delete_failed(), &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// DELETE /api/v1/files/admin/:id
pub async fn delete_file_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // テナントIDをヘッダーから取得する（CRIT-01 でテナント条件付き DELETE に使用）
    let request_tenant_id = tenant_id_from_headers(&headers).unwrap_or("").to_string();

    if let Ok(file) = state
        .get_file_metadata_uc
        .execute(&GetFileMetadataInput {
            file_id: id.clone(),
        })
        .await
    {
        if !request_tenant_id.is_empty() {
            // storage_path のプレフィックス（テナントID）とリクエストヘッダーのテナントIDを比較してアクセス制御を行う
            // FileMetadata から tenant_id フィールドが削除されたため、storage_path から取得する
            let resource_tenant_id =
                crate::domain::service::FileDomainService::tenant_id_from_storage_path(
                    &file.storage_path,
                )
                .unwrap_or("");
            if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                resource_tenant_id,
                &request_tenant_id,
            ) {
                let err =
                    ErrorResponse::new(codes::file::access_denied(), "access denied for tenant");
                return (StatusCode::FORBIDDEN, Json(err)).into_response();
            }
        }
    }

    // CRIT-01 監査対応: テナントIDを DELETE 条件に追加してアトミックな認可チェックを実現する
    // admin エンドポイントは所有者チェックをスキップするため expected_uploader は None
    let input = DeleteFileInput {
        file_id: id,
        tenant_id: request_tenant_id,
        expected_uploader: None,
    };

    // MED-02 監査対応: 文字列マッチングをやめ、型安全なエラー型で HTTP ステータスを決定する
    match state.delete_file_uc.execute(&input).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(crate::usecase::delete_file::DeleteFileError::NotFound(msg)) => {
            let err = ErrorResponse::new(codes::file::not_found(), &msg);
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::delete_file::DeleteFileError::Internal(msg)) => {
            let err = ErrorResponse::new(codes::file::delete_failed(), &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/files/:id/complete
pub async fn complete_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path(id): Path<String>,
    Json(req): Json<CompleteUploadRequest>,
) -> impl IntoResponse {
    use crate::usecase::complete_upload::CompleteUploadInput;

    // MED-11 監査対応: claims が None（認証なし）の場合は 401 を返す（防御的プログラミング）
    let claims = match claims {
        Some(axum::extract::Extension(c)) => c,
        None => {
            let err = ErrorResponse::new(codes::file::access_denied(), "authentication required");
            return (StatusCode::UNAUTHORIZED, Json(err)).into_response();
        }
    };

    // MED-11 監査対応: sys_admin ロールの場合はテナント照合をスキップする
    let is_admin = claims.realm_roles().iter().any(|role| role == "sys_admin");

    if let Some(request_tenant_id) = tenant_id_from_headers(&headers) {
        // MED-11 監査対応: X-Tenant-ID ヘッダーと JWT Claims のテナント ID が一致しない場合は 403 を返す
        if !is_admin && claims.tenant_id() != request_tenant_id {
            let err = ErrorResponse::new(
                codes::file::access_denied(),
                "tenant id mismatch between header and token",
            );
            return (StatusCode::FORBIDDEN, Json(err)).into_response();
        }

        if let Ok(file) = state
            .get_file_metadata_uc
            .execute(&GetFileMetadataInput {
                file_id: id.clone(),
            })
            .await
        {
            // storage_path のプレフィックス（テナントID）とリクエストヘッダーのテナントIDを比較してアクセス制御を行う
            // FileMetadata から tenant_id フィールドが削除されたため、storage_path から取得する
            let resource_tenant_id =
                crate::domain::service::FileDomainService::tenant_id_from_storage_path(
                    &file.storage_path,
                )
                .unwrap_or("");
            if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                resource_tenant_id,
                request_tenant_id,
            ) {
                let err =
                    ErrorResponse::new(codes::file::access_denied(), "access denied for tenant");
                return (StatusCode::FORBIDDEN, Json(err)).into_response();
            }
        }
    }

    // C-01 監査対応: checksum_sha256 → checksum にリネーム
    let input = CompleteUploadInput {
        file_id: id.clone(),
        checksum: req.checksum_sha256,
    };

    match state.complete_upload_uc.execute(&input).await {
        Ok(file) => (StatusCode::OK, Json(file_to_rest_detail(&file))).into_response(),
        // MED-02 監査対応: 文字列マッチングをやめ、型安全なエラー型で HTTP ステータスを決定する
        Err(crate::usecase::complete_upload::CompleteUploadError::NotFound(msg)) => {
            let err = ErrorResponse::new(codes::file::not_found(), &msg);
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::complete_upload::CompleteUploadError::AlreadyCompleted(msg)) => {
            let err = ErrorResponse::new(codes::file::already_completed(), &msg);
            (StatusCode::CONFLICT, Json(err)).into_response()
        }
        Err(crate::usecase::complete_upload::CompleteUploadError::Internal(msg)) => {
            let err = ErrorResponse::new(codes::file::complete_failed(), &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/files/:id/download-url - Generate download URL
pub async fn download_url(
    State(state): State<AppState>,
    headers: HeaderMap,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // MED-11 監査対応: claims が None（認証なし）の場合は 401 を返す（防御的プログラミング）
    let claims = match claims {
        Some(axum::extract::Extension(c)) => c,
        None => {
            let err = ErrorResponse::new(codes::file::access_denied(), "authentication required");
            return (StatusCode::UNAUTHORIZED, Json(err)).into_response();
        }
    };

    // MED-11 監査対応: sys_admin ロールの場合はテナント照合をスキップする
    let is_admin = claims.realm_roles().iter().any(|role| role == "sys_admin");

    if let Some(request_tenant_id) = tenant_id_from_headers(&headers) {
        // MED-11 監査対応: X-Tenant-ID ヘッダーと JWT Claims のテナント ID が一致しない場合は 403 を返す
        if !is_admin && claims.tenant_id() != request_tenant_id {
            let err = ErrorResponse::new(
                codes::file::access_denied(),
                "tenant id mismatch between header and token",
            );
            return (StatusCode::FORBIDDEN, Json(err)).into_response();
        }

        if let Ok(file) = state
            .get_file_metadata_uc
            .execute(&GetFileMetadataInput {
                file_id: id.clone(),
            })
            .await
        {
            // storage_path のプレフィックス（テナントID）とリクエストヘッダーのテナントIDを比較してアクセス制御を行う
            // FileMetadata から tenant_id フィールドが削除されたため、storage_path から取得する
            let resource_tenant_id =
                crate::domain::service::FileDomainService::tenant_id_from_storage_path(
                    &file.storage_path,
                )
                .unwrap_or("");
            if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                resource_tenant_id,
                request_tenant_id,
            ) {
                let err =
                    ErrorResponse::new(codes::file::access_denied(), "access denied for tenant");
                return (StatusCode::FORBIDDEN, Json(err)).into_response();
            }
        }
    }

    let input = GenerateDownloadUrlInput {
        file_id: id,
        expires_in_seconds: 3600,
    };

    match state.generate_download_url_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "file_id": output.file_id,
                "download_url": output.download_url,
                "expires_in_seconds": output.expires_in_seconds
            })),
        )
            .into_response(),
        // MED-02 監査対応: 文字列マッチングをやめ、型安全なエラー型で HTTP ステータスを決定する
        // GenerateDownloadUrlError::Internal はストレージエラーを含む可能性があるため
        // is_storage_error_message による判定を Internal バリアント内で維持する
        Err(crate::usecase::generate_download_url::GenerateDownloadUrlError::NotFound(msg)) => {
            let err = ErrorResponse::new(codes::file::not_found(), &msg);
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::generate_download_url::GenerateDownloadUrlError::NotAvailable(msg)) => {
            let err = ErrorResponse::new(codes::file::not_available(), &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(crate::usecase::generate_download_url::GenerateDownloadUrlError::Internal(msg)) => {
            if is_storage_error_message(&msg) {
                let err = ErrorResponse::new(codes::file::storage_error(), &msg);
                (StatusCode::BAD_GATEWAY, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new(codes::file::download_url_failed(), &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// PUT /api/v1/files/:id/tags
pub async fn update_file_tags(
    State(state): State<AppState>,
    headers: HeaderMap,
    claims: Option<axum::extract::Extension<k1s0_auth::Claims>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateFileTagsRequest>,
) -> impl IntoResponse {
    use crate::usecase::update_file_tags::UpdateFileTagsInput;

    // MED-11 監査対応: claims が None（認証なし）の場合は 401 を返す（防御的プログラミング）
    let claims = match claims {
        Some(axum::extract::Extension(c)) => c,
        None => {
            let err = ErrorResponse::new(codes::file::access_denied(), "authentication required");
            return (StatusCode::UNAUTHORIZED, Json(err)).into_response();
        }
    };

    // MED-11 監査対応: sys_admin ロールの場合はテナント照合をスキップする
    let is_admin = claims.realm_roles().iter().any(|role| role == "sys_admin");

    if let Some(request_tenant_id) = tenant_id_from_headers(&headers) {
        // MED-11 監査対応: X-Tenant-ID ヘッダーと JWT Claims のテナント ID が一致しない場合は 403 を返す
        if !is_admin && claims.tenant_id() != request_tenant_id {
            let err = ErrorResponse::new(
                codes::file::access_denied(),
                "tenant id mismatch between header and token",
            );
            return (StatusCode::FORBIDDEN, Json(err)).into_response();
        }

        if let Ok(file) = state
            .get_file_metadata_uc
            .execute(&GetFileMetadataInput {
                file_id: id.clone(),
            })
            .await
        {
            // storage_path のプレフィックス（テナントID）とリクエストヘッダーのテナントIDを比較してアクセス制御を行う
            // FileMetadata から tenant_id フィールドが削除されたため、storage_path から取得する
            let resource_tenant_id =
                crate::domain::service::FileDomainService::tenant_id_from_storage_path(
                    &file.storage_path,
                )
                .unwrap_or("");
            if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                resource_tenant_id,
                request_tenant_id,
            ) {
                let err =
                    ErrorResponse::new(codes::file::access_denied(), "access denied for tenant");
                return (StatusCode::FORBIDDEN, Json(err)).into_response();
            }
        }
    }

    let input = UpdateFileTagsInput {
        file_id: id.clone(),
        tags: req.tags,
    };

    match state.update_file_tags_uc.execute(&input).await {
        Ok(file) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "file_id": file.id,
                "tags": file.tags,
                "message": "tags updated"
            })),
        )
            .into_response(),
        // MED-02 監査対応: 文字列マッチングをやめ、型安全なエラー型で HTTP ステータスを決定する
        Err(crate::usecase::update_file_tags::UpdateFileTagsError::NotFound(msg)) => {
            let err = ErrorResponse::new(codes::file::not_found(), &msg);
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::update_file_tags::UpdateFileTagsError::Internal(msg)) => {
            let err = ErrorResponse::new(codes::file::tags_update_failed(), &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Request / Response types ---

/// C-01 監査対応: REST レスポンスのフィールド名を DB カラム名に合わせる
fn file_to_rest_summary(file: &crate::domain::entity::file::FileMetadata) -> serde_json::Value {
    serde_json::json!({
        "id": &file.id,
        "filename": &file.filename,
        "size_bytes": file.size_bytes,
        "content_type": &file.content_type,
        "uploaded_by": &file.uploaded_by,
        "tags": &file.tags,
        "storage_path": &file.storage_path,
        "status": &file.status,
        "created_at": file.created_at.to_rfc3339(),
        "updated_at": file.updated_at.to_rfc3339()
    })
}

fn file_to_rest_detail(file: &crate::domain::entity::file::FileMetadata) -> serde_json::Value {
    serde_json::json!({
        "id": &file.id,
        "filename": &file.filename,
        "size_bytes": file.size_bytes,
        "content_type": &file.content_type,
        "uploaded_by": &file.uploaded_by,
        "tags": &file.tags,
        "storage_path": &file.storage_path,
        "checksum": &file.checksum,
        "status": &file.status,
        "created_at": file.created_at.to_rfc3339(),
        "updated_at": file.updated_at.to_rfc3339()
    })
}

#[derive(Debug, Deserialize)]
pub struct UploadFileRequest {
    #[serde(alias = "name")]
    pub filename: String,
    #[serde(alias = "size")]
    pub size_bytes: u64,
    #[serde(alias = "mime_type")]
    pub content_type: String,
    pub tenant_id: String,
    #[serde(alias = "owner_id")]
    pub uploaded_by: String,
    pub tags: Option<HashMap<String, String>>,
    pub expires_in_seconds: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteUploadRequest {
    pub checksum_sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFileTagsRequest {
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct ListFilesParams {
    pub tenant_id: Option<String>,
    #[serde(alias = "owner_id", alias = "uploaded_by")]
    pub uploaded_by: Option<String>,
    #[serde(alias = "mime_type", alias = "content_type")]
    pub content_type: Option<String>,
    pub tag: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

fn parse_tag_query(raw: &str) -> Option<(String, String)> {
    let (key, value) = raw.split_once(':').or_else(|| raw.split_once('='))?;
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() || value.is_empty() {
        None
    } else {
        Some((key.to_string(), value.to_string()))
    }
}

fn tenant_id_from_headers(headers: &HeaderMap) -> Option<&str> {
    headers.get("x-tenant-id").and_then(|h| h.to_str().ok())
}
