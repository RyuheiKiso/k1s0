use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::file::v1::{
    file_service_server::FileService, CompleteUploadRequest, CompleteUploadResponse,
    DeleteFileRequest, DeleteFileResponse, FileMetadata as ProtoFileMetadata,
    FileMetadataResponse, GenerateDownloadUrlRequest, GenerateDownloadUrlResponse,
    GenerateUploadUrlRequest, GenerateUploadUrlResponse, GetFileMetadataRequest,
    ListFilesRequest, ListFilesResponse,
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
        filename: file.name.clone(),
        content_type: file.mime_type.clone(),
        size: file.size_bytes as i64,
        tenant_id: file.tenant_id.clone(),
        uploaded_by: file.owner_id.clone(),
        status: file.status.clone(),
        created_at: file.created_at.to_rfc3339(),
        updated_at: file.updated_at.to_rfc3339(),
        tags: file.tags.clone(),
    }
}

#[async_trait::async_trait]
impl FileService for FileServiceTonic {
    async fn get_file_metadata(
        &self,
        request: Request<GetFileMetadataRequest>,
    ) -> Result<Response<FileMetadataResponse>, Status> {
        let inner = request.into_inner();
        let file = self
            .inner
            .get_file_metadata(inner.id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(FileMetadataResponse {
            metadata: Some(domain_to_proto(&file)),
        }))
    }

    async fn list_files(
        &self,
        request: Request<ListFilesRequest>,
    ) -> Result<Response<ListFilesResponse>, Status> {
        let inner = request.into_inner();
        let (files, total) = self
            .inner
            .list_files(inner.tenant_id, inner.page as u32, inner.page_size as u32)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(ListFilesResponse {
            files: files.iter().map(domain_to_proto).collect(),
            total: total as i32,
        }))
    }

    async fn generate_upload_url(
        &self,
        request: Request<GenerateUploadUrlRequest>,
    ) -> Result<Response<GenerateUploadUrlResponse>, Status> {
        let inner = request.into_inner();
        let (file_id, upload_url) = self
            .inner
            .generate_upload_url(
                inner.filename,
                inner.content_type,
                inner.tenant_id,
                inner.uploaded_by,
                inner.tags,
            )
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(GenerateUploadUrlResponse {
            file_id,
            upload_url,
        }))
    }

    async fn complete_upload(
        &self,
        request: Request<CompleteUploadRequest>,
    ) -> Result<Response<CompleteUploadResponse>, Status> {
        let inner = request.into_inner();
        let file = self
            .inner
            .complete_upload(inner.file_id)
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
        let download_url = self
            .inner
            .generate_download_url(inner.id)
            .await
            .map_err(Into::<Status>::into)?;
        Ok(Response::new(GenerateDownloadUrlResponse { download_url }))
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
}
