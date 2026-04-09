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

// ユースケースフィールドの命名規則として _uc サフィックスを使用する（アーキテクチャ上の意図的な設計）
#[allow(clippy::struct_field_names)]
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
    #[must_use]
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

    /// ファイルメタデータを取得する。ユースケースエラー型で型ベースにGrpcErrorへ変換する。
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
                use crate::usecase::get_file_metadata::GetFileMetadataError;
                match e {
                    GetFileMetadataError::NotFound(msg) => GrpcError::NotFound(msg),
                    GetFileMetadataError::Internal(msg) => GrpcError::Internal(msg),
                }
            })
    }

    pub async fn list_files(
        &self,
        tenant_id: String,
        uploaded_by: Option<String>,
        content_type: Option<String>,
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
            uploaded_by: uploaded_by.filter(|v| !v.is_empty()),
            content_type: content_type.filter(|v| !v.is_empty()),
            tag: tag.as_deref().and_then(parse_tag_filter),
            page: if page == 0 { 1 } else { page },
            page_size: if page_size == 0 { 20 } else { page_size },
        };
        // ファイル一覧取得。ユースケースエラー型で型ベースにGrpcErrorへ変換する。
        let output = self.list_files_uc.execute(&input).await.map_err(|e| {
            use crate::usecase::list_files::ListFilesError;
            match e {
                ListFilesError::Internal(msg) => GrpcError::Internal(msg),
            }
        })?;
        Ok((output.files, output.total_count))
    }

    #[allow(clippy::too_many_arguments)]
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
            filename,
            // LOW-008: 安全な型変換（オーバーフロー防止）
            size_bytes: u64::try_from(size_bytes).unwrap_or(0),
            content_type,
            tenant_id,
            uploaded_by,
            tags,
            expires_in_seconds,
        };
        // アップロードURL生成。ユースケースエラー型で型ベースにGrpcErrorへ変換する。
        let output = self
            .generate_upload_url_uc
            .execute(&input)
            .await
            .map_err(|e| {
                use crate::usecase::generate_upload_url::GenerateUploadUrlError;
                match e {
                    GenerateUploadUrlError::Validation(msg) => GrpcError::InvalidArgument(msg),
                    GenerateUploadUrlError::SizeExceeded { actual, max } => {
                        GrpcError::InvalidArgument(format!(
                            "file size exceeded: actual={actual}, max={max}"
                        ))
                    }
                    GenerateUploadUrlError::Internal(msg) => GrpcError::Internal(msg),
                }
            })?;
        Ok((output.file_id, output.upload_url, output.expires_in_seconds))
    }

    /// C-01 監査対応: `checksum_sha256` → checksum にリネーム
    pub async fn complete_upload(
        &self,
        file_id: String,
        checksum: Option<String>,
    ) -> Result<crate::domain::entity::file::FileMetadata, GrpcError> {
        if file_id.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "file_id is required".to_string(),
            ));
        }
        let input = CompleteUploadInput { file_id, checksum };
        // アップロード完了。ユースケースエラー型で型ベースにGrpcErrorへ変換する。
        self.complete_upload_uc.execute(&input).await.map_err(|e| {
            use crate::usecase::complete_upload::CompleteUploadError;
            match e {
                CompleteUploadError::NotFound(msg) => GrpcError::NotFound(msg),
                CompleteUploadError::AlreadyCompleted(msg) => GrpcError::AlreadyExists(msg),
                CompleteUploadError::Internal(msg) => GrpcError::Internal(msg),
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
        // ダウンロードURL生成。ユースケースエラー型で型ベースにGrpcErrorへ変換する。
        let output = self
            .generate_download_url_uc
            .execute(&input)
            .await
            .map_err(|e| {
                use crate::usecase::generate_download_url::GenerateDownloadUrlError;
                match e {
                    GenerateDownloadUrlError::NotFound(msg) => GrpcError::NotFound(msg),
                    GenerateDownloadUrlError::NotAvailable(msg) => GrpcError::InvalidArgument(msg),
                    GenerateDownloadUrlError::Internal(msg) => GrpcError::Internal(msg),
                }
            })?;
        Ok((output.download_url, output.expires_in_seconds))
    }

    /// ファイルを削除する。ユースケースエラー型で型ベースにGrpcErrorへ変換する。
    /// CRIT-01 監査対応: `tenant_id` と `expected_uploader` を DELETE 条件に追加してアトミックな認可チェックを実現する。
    /// gRPC 呼び出し元はリクエストメタデータから取得したテナントID・認証済みユーザーIDを渡す責任を持つ。
    pub async fn delete_file(
        &self,
        id: String,
        tenant_id: String,
        expected_uploader: Option<String>,
    ) -> Result<(), GrpcError> {
        if id.is_empty() {
            return Err(GrpcError::InvalidArgument("id is required".to_string()));
        }
        let input = DeleteFileInput {
            file_id: id,
            tenant_id,
            expected_uploader,
        };
        self.delete_file_uc.execute(&input).await.map_err(|e| {
            use crate::usecase::delete_file::DeleteFileError;
            match e {
                DeleteFileError::NotFound(msg) => GrpcError::NotFound(msg),
                DeleteFileError::Internal(msg) => GrpcError::Internal(msg),
            }
        })?;
        Ok(())
    }

    /// ファイルタグを更新する。ユースケースエラー型で型ベースにGrpcErrorへ変換する。
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
            use crate::usecase::update_file_tags::UpdateFileTagsError;
            match e {
                UpdateFileTagsError::NotFound(msg) => GrpcError::NotFound(msg),
                UpdateFileTagsError::Internal(msg) => GrpcError::Internal(msg),
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
