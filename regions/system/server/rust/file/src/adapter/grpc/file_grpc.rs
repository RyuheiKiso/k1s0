use std::sync::Arc;

use crate::usecase::complete_upload::{CompleteUploadInput, CompleteUploadUseCase};
use crate::usecase::delete_file::{DeleteFileInput, DeleteFileUseCase};
use crate::usecase::generate_download_url::{GenerateDownloadUrlInput, GenerateDownloadUrlUseCase};
use crate::usecase::generate_upload_url::{GenerateUploadUrlInput, GenerateUploadUrlUseCase};
use crate::usecase::get_file_metadata::{GetFileMetadataInput, GetFileMetadataUseCase};
use crate::usecase::list_files::{ListFilesInput, ListFilesUseCase};
use crate::usecase::update_file_tags::{UpdateFileTagsInput, UpdateFileTagsUseCase};

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("already exists: {0}")]
    AlreadyExists(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<GrpcError> for tonic::Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => tonic::Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => tonic::Status::invalid_argument(msg),
            GrpcError::AlreadyExists(msg) => tonic::Status::already_exists(msg),
            GrpcError::Internal(msg) => tonic::Status::internal(msg),
        }
    }
}

pub struct FileGrpcService {
    get_file_metadata_uc: Arc<GetFileMetadataUseCase>,
    list_files_uc: Arc<ListFilesUseCase>,
    generate_upload_url_uc: Arc<GenerateUploadUrlUseCase>,
    complete_upload_uc: Arc<CompleteUploadUseCase>,
    generate_download_url_uc: Arc<GenerateDownloadUrlUseCase>,
    delete_file_uc: Arc<DeleteFileUseCase>,
    update_file_tags_uc: Arc<UpdateFileTagsUseCase>,
}

impl FileGrpcService {
    pub fn new(
        get_file_metadata_uc: Arc<GetFileMetadataUseCase>,
        list_files_uc: Arc<ListFilesUseCase>,
        generate_upload_url_uc: Arc<GenerateUploadUrlUseCase>,
        complete_upload_uc: Arc<CompleteUploadUseCase>,
        generate_download_url_uc: Arc<GenerateDownloadUrlUseCase>,
        delete_file_uc: Arc<DeleteFileUseCase>,
        update_file_tags_uc: Arc<UpdateFileTagsUseCase>,
    ) -> Self {
        Self {
            get_file_metadata_uc,
            list_files_uc,
            generate_upload_url_uc,
            complete_upload_uc,
            generate_download_url_uc,
            delete_file_uc,
            update_file_tags_uc,
        }
    }

    pub async fn get_file_metadata(
        &self,
        id: String,
    ) -> Result<crate::domain::entity::file::FileMetadata, GrpcError> {
        if id.is_empty() {
            return Err(GrpcError::InvalidArgument("id is required".to_string()));
        }
        let input = GetFileMetadataInput { file_id: id };
        self.get_file_metadata_uc
            .execute(&input)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("not found") {
                    GrpcError::NotFound(msg)
                } else {
                    GrpcError::Internal(msg)
                }
            })
    }

    pub async fn list_files(
        &self,
        tenant_id: String,
        uploaded_by: Option<String>,
        mime_type: Option<String>,
        tag: Option<String>,
        page: u32,
        page_size: u32,
    ) -> Result<(Vec<crate::domain::entity::file::FileMetadata>, u64), GrpcError> {
        let input = ListFilesInput {
            tenant_id: if tenant_id.is_empty() {
                None
            } else {
                Some(tenant_id)
            },
            owner_id: uploaded_by.filter(|v| !v.is_empty()),
            mime_type: mime_type.filter(|v| !v.is_empty()),
            tag: tag.as_deref().and_then(parse_tag_filter),
            page: if page == 0 { 1 } else { page },
            page_size: if page_size == 0 { 20 } else { page_size },
        };
        let output = self
            .list_files_uc
            .execute(&input)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;
        Ok((output.files, output.total_count))
    }

    pub async fn generate_upload_url(
        &self,
        filename: String,
        content_type: String,
        tenant_id: String,
        uploaded_by: String,
        tags: std::collections::HashMap<String, String>,
        expires_in_seconds: Option<u32>,
        size_bytes: i64,
    ) -> Result<(String, String, u32), GrpcError> {
        if filename.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "filename is required".to_string(),
            ));
        }
        let expires_in_seconds = expires_in_seconds.unwrap_or(3600).max(1);
        if size_bytes <= 0 {
            return Err(GrpcError::InvalidArgument(
                "size_bytes must be greater than zero".to_string(),
            ));
        }
        let input = GenerateUploadUrlInput {
            name: filename,
            size_bytes: size_bytes as u64,
            mime_type: content_type,
            tenant_id,
            owner_id: uploaded_by,
            tags,
            expires_in_seconds,
        };
        let output = self
            .generate_upload_url_uc
            .execute(&input)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("validation") {
                    GrpcError::InvalidArgument(msg)
                } else {
                    GrpcError::Internal(msg)
                }
            })?;
        Ok((output.file_id, output.upload_url, output.expires_in_seconds))
    }

    pub async fn complete_upload(
        &self,
        file_id: String,
        checksum_sha256: Option<String>,
    ) -> Result<crate::domain::entity::file::FileMetadata, GrpcError> {
        if file_id.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "file_id is required".to_string(),
            ));
        }
        let input = CompleteUploadInput {
            file_id,
            checksum_sha256,
        };
        self.complete_upload_uc.execute(&input).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GrpcError::NotFound(msg)
            } else if msg.contains("already completed") {
                GrpcError::AlreadyExists(msg)
            } else {
                GrpcError::Internal(msg)
            }
        })
    }

    pub async fn generate_download_url(&self, id: String) -> Result<(String, u32), GrpcError> {
        if id.is_empty() {
            return Err(GrpcError::InvalidArgument("id is required".to_string()));
        }
        let expires_in_seconds = 3600;
        let input = GenerateDownloadUrlInput {
            file_id: id,
            expires_in_seconds,
        };
        let output = self
            .generate_download_url_uc
            .execute(&input)
            .await
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("not found") {
                    GrpcError::NotFound(msg)
                } else if msg.contains("not available") {
                    GrpcError::InvalidArgument(msg)
                } else {
                    GrpcError::Internal(msg)
                }
            })?;
        Ok((output.download_url, output.expires_in_seconds))
    }

    pub async fn delete_file(&self, id: String) -> Result<(), GrpcError> {
        if id.is_empty() {
            return Err(GrpcError::InvalidArgument("id is required".to_string()));
        }
        let input = DeleteFileInput { file_id: id };
        self.delete_file_uc.execute(&input).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GrpcError::NotFound(msg)
            } else {
                GrpcError::Internal(msg)
            }
        })?;
        Ok(())
    }

    pub async fn update_file_tags(
        &self,
        id: String,
        tags: std::collections::HashMap<String, String>,
    ) -> Result<crate::domain::entity::file::FileMetadata, GrpcError> {
        if id.is_empty() {
            return Err(GrpcError::InvalidArgument("id is required".to_string()));
        }
        let input = UpdateFileTagsInput { file_id: id, tags };
        self.update_file_tags_uc.execute(&input).await.map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                GrpcError::NotFound(msg)
            } else {
                GrpcError::Internal(msg)
            }
        })
    }
}

fn parse_tag_filter(raw: &str) -> Option<(String, String)> {
    let (key, value) = raw.split_once(':').or_else(|| raw.split_once('='))?;
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() || value.is_empty() {
        None
    } else {
        Some((key.to_string(), value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("file not found".to_string());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("id is required".to_string());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    #[test]
    fn test_grpc_error_already_exists_to_status() {
        let err = GrpcError::AlreadyExists("already completed".to_string());
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::AlreadyExists);
    }
}
