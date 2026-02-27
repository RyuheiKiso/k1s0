use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        name: req.name,
        size_bytes: req.size_bytes,
        mime_type: req.mime_type,
        tenant_id: req.tenant_id,
        owner_id: req.owner_id,
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
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("validation") {
                let err = ErrorResponse::new("SYS_FILE_VALIDATION", &msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_FILE_UPLOAD_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// GET /api/v1/files/:id - Get file metadata and download URL
pub async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let metadata_input = GetFileMetadataInput {
        file_id: id.clone(),
    };

    match state.get_file_metadata_uc.execute(&metadata_input).await {
        Ok(file) => {
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
                    "file": file,
                    "download_url": download_url
                })),
            )
                .into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_FILE_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_FILE_GET_FAILED", &msg);
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
        tag: None,
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
    };

    match state.list_files_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "files": output.files,
                "total_count": output.total_count,
                "page": output.page,
                "page_size": output.page_size,
                "has_next": output.has_next
            })),
        )
            .into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_FILE_LIST_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// DELETE /api/v1/files/:id
pub async fn delete_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = DeleteFileInput { file_id: id };

    match state.delete_file_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": output.success,
                "message": output.message
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_FILE_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_FILE_DELETE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// POST /api/v1/files/:id/complete
pub async fn complete_upload(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<CompleteUploadRequest>,
) -> impl IntoResponse {
    use crate::usecase::complete_upload::CompleteUploadInput;

    let input = CompleteUploadInput {
        file_id: id.clone(),
        checksum_sha256: req.checksum_sha256,
    };

    match state.complete_upload_uc.execute(&input).await {
        Ok(file) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "file_id": file.id,
                "status": file.status,
                "message": "upload completed"
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_FILE_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else if msg.contains("already completed") {
                let err = ErrorResponse::new("SYS_FILE_ALREADY_COMPLETED", &msg);
                (StatusCode::CONFLICT, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_FILE_COMPLETE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// GET /api/v1/files/:id/download-url - Generate download URL
pub async fn download_url(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
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
                let err = ErrorResponse::new("SYS_FILE_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else if msg.contains("not available") {
                let err = ErrorResponse::new("SYS_FILE_NOT_AVAILABLE", &msg);
                (StatusCode::BAD_REQUEST, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_FILE_DOWNLOAD_URL_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// PUT /api/v1/files/:id/tags
pub async fn update_file_tags(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateFileTagsRequest>,
) -> impl IntoResponse {
    use crate::usecase::update_file_tags::UpdateFileTagsInput;

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
                let err = ErrorResponse::new("SYS_FILE_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_FILE_TAGS_UPDATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct UploadFileRequest {
    pub name: String,
    pub size_bytes: u64,
    pub mime_type: String,
    pub tenant_id: String,
    pub owner_id: String,
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
    pub owner_id: Option<String>,
    pub mime_type: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
            },
        }
    }
}
