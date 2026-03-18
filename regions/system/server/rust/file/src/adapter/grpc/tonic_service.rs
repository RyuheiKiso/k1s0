use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::PaginationResult as ProtoPaginationResult;
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
    pub fn new(inner: Arc<FileGrpcService>) -> Self {
        Self { inner }
    }
}

fn domain_to_proto(file: &crate::domain::entity::file::FileMetadata) -> ProtoFileMetadata {
    ProtoFileMetadata {
        id: file.id.clone(),
        filename: file.filename.clone(),
        content_type: file.content_type.clone(),
        size_bytes: file.size_bytes as i64,
        tenant_id: file.tenant_id.clone(),
        uploaded_by: file.uploaded_by.clone(),
        status: file.status.clone(),
        created_at: file.created_at.to_rfc3339(),
        updated_at: file.updated_at.to_rfc3339(),
        tags: file.tags.clone(),
        storage_key: file.storage_key.clone(),
        checksum_sha256: file.checksum_sha256.clone(),
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
        let page = if inner.page <= 0 {
            1
        } else {
            inner.page as u32
        };
        let page_size = if inner.page_size <= 0 {
            20
        } else {
            inner.page_size as u32
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
        let has_next = ((page * page_size) as u64) < total;
        let total_count = total as i64;
        Ok(Response::new(ListFilesResponse {
            files: files.iter().map(domain_to_proto).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count,
                page: page as i32,
                page_size: page_size as i32,
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
            expires_in_seconds: expires_in_seconds as i32,
        }))
    }

    async fn delete_file(
        &self,
        request: Request<DeleteFileRequest>,
    ) -> Result<Response<DeleteFileResponse>, Status> {
        let inner = request.into_inner();
        self.inner
            .delete_file(inner.id)
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
