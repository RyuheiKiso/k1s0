pub mod file_handler;
pub mod health;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::usecase::{
    CompleteUploadUseCase, DeleteFileUseCase, GenerateDownloadUrlUseCase,
    GenerateUploadUrlUseCase, GetFileMetadataUseCase, ListFilesUseCase, UpdateFileTagsUseCase,
};

/// Shared application state for REST handlers.
#[derive(Clone)]
pub struct AppState {
    pub list_files_uc: Arc<ListFilesUseCase>,
    pub generate_upload_url_uc: Arc<GenerateUploadUrlUseCase>,
    pub complete_upload_uc: Arc<CompleteUploadUseCase>,
    pub get_file_metadata_uc: Arc<GetFileMetadataUseCase>,
    pub generate_download_url_uc: Arc<GenerateDownloadUrlUseCase>,
    pub delete_file_uc: Arc<DeleteFileUseCase>,
    pub update_file_tags_uc: Arc<UpdateFileTagsUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler))
        .route("/api/v1/files", get(file_handler::list_files))
        .route("/api/v1/files", post(file_handler::upload_file))
        .route("/api/v1/files/:id", get(file_handler::get_file))
        .route("/api/v1/files/:id", delete(file_handler::delete_file))
        .route(
            "/api/v1/files/:id/complete",
            post(file_handler::complete_upload),
        )
        .route(
            "/api/v1/files/:id/tags",
            put(file_handler::update_file_tags),
        )
        .with_state(state)
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
