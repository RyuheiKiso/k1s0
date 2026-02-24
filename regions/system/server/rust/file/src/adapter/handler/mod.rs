pub mod file_handler;
pub mod health;

use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::Router;

use crate::usecase::{
    DeleteFileUseCase, GenerateDownloadUrlUseCase, GenerateUploadUrlUseCase,
    GetFileMetadataUseCase, ListFilesUseCase,
};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub list_files_uc: Arc<ListFilesUseCase>,
    pub generate_upload_url_uc: Arc<GenerateUploadUrlUseCase>,
    pub get_file_metadata_uc: Arc<GetFileMetadataUseCase>,
    pub generate_download_url_uc: Arc<GenerateDownloadUrlUseCase>,
    pub delete_file_uc: Arc<DeleteFileUseCase>,
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/api/v1/files", get(file_handler::list_files))
        .route("/api/v1/files", post(file_handler::upload_file))
        .route("/api/v1/files/:id", get(file_handler::get_file))
        .route("/api/v1/files/:id", delete(file_handler::delete_file))
        .with_state(state)
}
