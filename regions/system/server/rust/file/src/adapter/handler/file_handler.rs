use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use k1s0_server_common::error as codes;
use k1s0_server_common::ErrorResponse;

use super::AppState;
use crate::usecase::delete_file::DeleteFileInput;
use crate::usecase::generate_download_url::GenerateDownloadUrlInput;
use crate::usecase::generate_upload_url::GenerateUploadUrlInput;
use crate::usecase::get_file_metadata::GetFileMetadataInput;
use crate::usecase::list_files::ListFilesInput;

/// POST /api/v1/files - Generate upload URL (initiate upload)
pub async fn upload_file(
    State(state): State<AppState>,
    Json(req): Json<UploadFileRequest>,
) -> impl IntoResponse {
    let input = GenerateUploadUrlInput {
        name: req.filename,
        size_bytes: req.size_bytes,
        mime_type: req.content_type,
        tenant_id: req.tenant_id,
        owner_id: req.uploaded_by,
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
        Err(crate::usecase::generate_upload_url::GenerateUploadUrlError::SizeExceeded { actual, max }) => {
            let err = ErrorResponse::new(
                codes::file::size_exceeded(),
                &format!("file size exceeds limit: {} > {}", actual, max),
            );
            (StatusCode::PAYLOAD_TOO_LARGE, Json(err)).into_response()
        }
        Err(crate::usecase::generate_upload_url::GenerateUploadUrlError::Internal(msg)) => {
            let err = ErrorResponse::new(codes::file::upload_failed(), &msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/files/:id - Get file metadata and download URL
pub async fn get_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let metadata_input = GetFileMetadataInput {
        file_id: id.clone(),
    };

    match state.get_file_metadata_uc.execute(&metadata_input).await {
        Ok(file) => {
            if let Some(request_tenant_id) = tenant_id_from_headers(&headers) {
                if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                    &file.tenant_id,
                    request_tenant_id,
                ) {
                    let err =
                        ErrorResponse::new(codes::file::access_denied(), "access denied for tenant");
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
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new(codes::file::not_found(), &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new(codes::file::get_failed(), &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
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
        owner_id: params.owner_id,
        mime_type: params.mime_type,
        tag: params
            .tag
            .as_deref()
            .and_then(parse_tag_query),
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
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Some(request_tenant_id) = tenant_id_from_headers(&headers) {
        match state
            .get_file_metadata_uc
            .execute(&GetFileMetadataInput {
                file_id: id.clone(),
            })
            .await
        {
            Ok(file) => {
                if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                    &file.tenant_id,
                    request_tenant_id,
                ) {
                    let err =
                        ErrorResponse::new(codes::file::access_denied(), "access denied for tenant");
                    return (StatusCode::FORBIDDEN, Json(err)).into_response();
                }
            }
            Err(_) => {}
        }
    }

    let input = DeleteFileInput { file_id: id };

    match state.delete_file_uc.execute(&input).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new(codes::file::not_found(), &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new(codes::file::delete_failed(), &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/files/:id/complete
pub async fn complete_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<CompleteUploadRequest>,
) -> impl IntoResponse {
    use crate::usecase::complete_upload::CompleteUploadInput;

    if let Some(request_tenant_id) = tenant_id_from_headers(&headers) {
        match state
            .get_file_metadata_uc
            .execute(&GetFileMetadataInput {
                file_id: id.clone(),
            })
            .await
        {
            Ok(file) => {
                if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                    &file.tenant_id,
                    request_tenant_id,
                ) {
                    let err =
                        ErrorResponse::new(codes::file::access_denied(), "access denied for tenant");
                    return (StatusCode::FORBIDDEN, Json(err)).into_response();
                }
            }
            Err(_) => {}
        }
    }

    let input = CompleteUploadInput {
        file_id: id.clone(),
        checksum_sha256: req.checksum_sha256,
    };

    match state.complete_upload_uc.execute(&input).await {
        Ok(file) => (
            StatusCode::OK,
            Json(file_to_rest_detail(&file)),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new(codes::file::not_found(), &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else if msg.contains("already completed") {
                let err = ErrorResponse::new(codes::file::already_completed(), &msg);
                (StatusCode::CONFLICT, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new(codes::file::complete_failed(), &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// GET /api/v1/files/:id/download-url - Generate download URL
pub async fn download_url(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    if let Some(request_tenant_id) = tenant_id_from_headers(&headers) {
        match state
            .get_file_metadata_uc
            .execute(&GetFileMetadataInput {
                file_id: id.clone(),
            })
            .await
        {
            Ok(file) => {
                if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                    &file.tenant_id,
                    request_tenant_id,
                ) {
                    let err =
                        ErrorResponse::new(codes::file::access_denied(), "access denied for tenant");
                    return (StatusCode::FORBIDDEN, Json(err)).into_response();
                }
            }
            Err(_) => {}
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
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new(codes::file::not_found(), &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else if msg.contains("not available") {
                let err = ErrorResponse::new(codes::file::not_available(), &msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
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
    Path(id): Path<String>,
    Json(req): Json<UpdateFileTagsRequest>,
) -> impl IntoResponse {
    use crate::usecase::update_file_tags::UpdateFileTagsInput;

    if let Some(request_tenant_id) = tenant_id_from_headers(&headers) {
        match state
            .get_file_metadata_uc
            .execute(&GetFileMetadataInput {
                file_id: id.clone(),
            })
            .await
        {
            Ok(file) => {
                if !crate::domain::service::FileDomainService::can_access_tenant_resource(
                    &file.tenant_id,
                    request_tenant_id,
                ) {
                    let err =
                        ErrorResponse::new(codes::file::access_denied(), "access denied for tenant");
                    return (StatusCode::FORBIDDEN, Json(err)).into_response();
                }
            }
            Err(_) => {}
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
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new(codes::file::not_found(), &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new(codes::file::tags_update_failed(), &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

// --- Request / Response types ---

fn file_to_rest_summary(file: &crate::domain::entity::file::FileMetadata) -> serde_json::Value {
    serde_json::json!({
        "id": &file.id,
        "filename": &file.name,
        "size_bytes": file.size_bytes,
        "content_type": &file.mime_type,
        "tenant_id": &file.tenant_id,
        "uploaded_by": &file.owner_id,
        "tags": &file.tags,
        "storage_key": &file.storage_key,
        "status": &file.status,
        "created_at": file.created_at.to_rfc3339(),
        "updated_at": file.updated_at.to_rfc3339()
    })
}

fn file_to_rest_detail(file: &crate::domain::entity::file::FileMetadata) -> serde_json::Value {
    serde_json::json!({
        "id": &file.id,
        "filename": &file.name,
        "size_bytes": file.size_bytes,
        "content_type": &file.mime_type,
        "tenant_id": &file.tenant_id,
        "uploaded_by": &file.owner_id,
        "tags": &file.tags,
        "storage_key": &file.storage_key,
        "checksum_sha256": &file.checksum_sha256,
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
    #[serde(alias = "uploaded_by")]
    pub owner_id: Option<String>,
    #[serde(alias = "content_type")]
    pub mime_type: Option<String>,
    pub tag: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

fn parse_tag_query(raw: &str) -> Option<(String, String)> {
    let (key, value) = raw
        .split_once(':')
        .or_else(|| raw.split_once('='))?;
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

