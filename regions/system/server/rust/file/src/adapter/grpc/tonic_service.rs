use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp,
};
use crate::proto::k1s0::system::file::v1::{
    file_service_server::FileService, CompleteUploadRequest, CompleteUploadResponse,
    DeleteFileRequest, DeleteFileResponse, FileMetadata as ProtoFileMetadata,
    GenerateDownloadUrlRequest, GenerateDownloadUrlResponse, GenerateUploadUrlRequest,
    GenerateUploadUrlResponse, GetFileMetadataRequest, GetFileMetadataResponse, ListFilesRequest,
    ListFilesResponse, UpdateFileTagsRequest, UpdateFileTagsResponse,
};

use super::file_grpc::FileGrpcService;

pub struct FileServiceTonic {
    inner: Arc<FileGrpcService>,
}

impl FileServiceTonic {
    #[must_use]
    pub fn new(inner: Arc<FileGrpcService>) -> Self {
        Self { inner }
    }
}

/// テナント分離対応: domain entity → proto メッセージ変換
/// migration 003 で追加した `tenant_id` フィールドを proto メッセージに含める
fn domain_to_proto(file: &crate::domain::entity::file::FileMetadata) -> ProtoFileMetadata {
    ProtoFileMetadata {
        id: file.id.clone(),
        filename: file.filename.clone(),
        content_type: file.content_type.clone(),
        // LOW-008: 安全な型変換（ファイルサイズは i64::MAX を超えない前提）
        size_bytes: i64::try_from(file.size_bytes).unwrap_or(i64::MAX),
        // テナント分離: エンティティの tenant_id フィールドを proto に渡す
        tenant_id: file.tenant_id.clone(),
        uploaded_by: file.uploaded_by.clone(),
        status: file.status.clone(),
        // DateTime<Utc> を Timestamp（seconds/nanos）へ変換
        created_at: Some(Timestamp {
            seconds: file.created_at.timestamp(),
            // LOW-008: 安全な型変換（subsec_nanos は 0..999_999_999 の範囲で i32 に収まる）
            nanos: i32::try_from(file.created_at.timestamp_subsec_nanos()).unwrap_or(0),
        }),
        updated_at: Some(Timestamp {
            seconds: file.updated_at.timestamp(),
            // LOW-008: 安全な型変換（subsec_nanos は 0..999_999_999 の範囲で i32 に収まる）
            nanos: i32::try_from(file.updated_at.timestamp_subsec_nanos()).unwrap_or(0),
        }),
        tags: file.tags.clone(),
        storage_key: file.storage_path.clone(),
        checksum_sha256: file.checksum.clone(),
    }
}

#[async_trait::async_trait]
impl FileService for FileServiceTonic {
    async fn get_file_metadata(
        &self,
        request: Request<GetFileMetadataRequest>,
    ) -> Result<Response<GetFileMetadataResponse>, Status> {
        let inner = request.into_inner();
        let file = self
            .inner
            .get_file_metadata(inner.id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(GetFileMetadataResponse {
            metadata: Some(domain_to_proto(&file)),
        }))
    }

    async fn list_files(
        &self,
        request: Request<ListFilesRequest>,
    ) -> Result<Response<ListFilesResponse>, Status> {
        let inner = request.into_inner();
        // ページネーションパラメータを共通Paginationサブメッセージから取得
        let pagination = inner.pagination.unwrap_or_default();
        // LOW-008: 安全な型変換（負の場合はデフォルト値を使用、プロトコルの不変条件）
        let page = if pagination.page <= 0 {
            1
        } else {
            u32::try_from(pagination.page).unwrap_or(0)
        };
        let page_size = if pagination.page_size <= 0 {
            20
        } else {
            u32::try_from(pagination.page_size).unwrap_or(0)
        };
        let (files, total) = self
            .inner
            .list_files(
                inner.tenant_id,
                inner.uploaded_by,
                inner.mime_type,
                inner.tag,
                page,
                page_size,
            )
            .await
            .map_err(Into::<Status>::into)?;
        let has_next = u64::from(page * page_size) < total;
        // LOW-008: 安全な型変換（u64 → i64: ファイル総数は i64::MAX を超えない前提）
        let total_count = i64::try_from(total).unwrap_or(i64::MAX);
        Ok(Response::new(ListFilesResponse {
            files: files.iter().map(domain_to_proto).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count,
                // LOW-008: 安全な型変換（page/page_size は正の値でありi32範囲内）
                page: i32::try_from(page).unwrap_or(i32::MAX),
                page_size: i32::try_from(page_size).unwrap_or(i32::MAX),
                has_next,
            }),
        }))
    }

    async fn generate_upload_url(
        &self,
        request: Request<GenerateUploadUrlRequest>,
    ) -> Result<Response<GenerateUploadUrlResponse>, Status> {
        let inner = request.into_inner();
        let (file_id, upload_url, expires_in_seconds) = self
            .inner
            .generate_upload_url(
                inner.filename,
                inner.content_type,
                inner.tenant_id,
                inner.uploaded_by,
                inner.tags,
                inner.expires_in_seconds,
                inner.size_bytes,
            )
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(GenerateUploadUrlResponse {
            file_id,
            upload_url,
            expires_in_seconds,
        }))
    }

    async fn complete_upload(
        &self,
        request: Request<CompleteUploadRequest>,
    ) -> Result<Response<CompleteUploadResponse>, Status> {
        let inner = request.into_inner();
        let file = self
            .inner
            .complete_upload(inner.file_id, inner.checksum_sha256)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(CompleteUploadResponse {
            metadata: Some(domain_to_proto(&file)),
        }))
    }

    async fn generate_download_url(
        &self,
        request: Request<GenerateDownloadUrlRequest>,
    ) -> Result<Response<GenerateDownloadUrlResponse>, Status> {
        let inner = request.into_inner();
        let (download_url, expires_in_seconds) = self
            .inner
            .generate_download_url(inner.id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(GenerateDownloadUrlResponse {
            download_url,
            // LOW-008: 安全な型変換（有効期限秒数は i32 範囲内が前提）
            expires_in_seconds: i32::try_from(expires_in_seconds).unwrap_or(i32::MAX),
        }))
    }

    async fn delete_file(
        &self,
        request: Request<DeleteFileRequest>,
    ) -> Result<Response<DeleteFileResponse>, Status> {
        let inner = request.into_inner();
        // CRIT-01 監査対応: gRPC はサービス間内部通信であり JWT により認可済みのため、
        // tenant_id は空文字列（チェックなし）・expected_uploader は None（所有者チェックなし）とする
        // 将来的には gRPC メタデータから tenant_id を取得することを推奨する
        self.inner
            .delete_file(inner.id, String::new(), None)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(DeleteFileResponse {}))
    }

    async fn update_file_tags(
        &self,
        request: Request<UpdateFileTagsRequest>,
    ) -> Result<Response<UpdateFileTagsResponse>, Status> {
        let inner = request.into_inner();
        let file = self
            .inner
            .update_file_tags(inner.id, inner.tags)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(UpdateFileTagsResponse {
            metadata: Some(domain_to_proto(&file)),
        }))
    }
}
