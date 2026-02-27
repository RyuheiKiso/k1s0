pub mod file_handler;
pub mod health;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::{auth_middleware, FileAuthState};
use crate::adapter::middleware::rbac::require_permission;
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
    pub auth_state: Option<FileAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: FileAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    // 認証不要のエンドポイント
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler));

    // 認証が設定されている場合は RBAC 付きルーティング、そうでなければオープンアクセス
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> files/read
        let read_routes = Router::new()
            .route("/api/v1/files", get(file_handler::list_files))
            .route("/api/v1/files/:id", get(file_handler::get_file))
            .route(
                "/api/v1/files/:id/download-url",
                get(file_handler::download_url),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "files", "read",
            )));

        // POST/complete/tags -> files/write
        let write_routes = Router::new()
            .route("/api/v1/files", post(file_handler::upload_file))
            .route(
                "/api/v1/files/:id/complete",
                post(file_handler::complete_upload),
            )
            .route(
                "/api/v1/files/:id/tags",
                put(file_handler::update_file_tags),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "files", "write",
            )));

        // DELETE -> files/admin
        let admin_routes = Router::new()
            .route("/api/v1/files/:id", delete(file_handler::delete_file))
            .route_layer(axum::middleware::from_fn(require_permission(
                "files", "admin",
            )));

        // 認証ミドルウェアを全 API ルートに適用
        Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(admin_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        // 認証なし（dev モード / テスト）: 従来どおり
        Router::new()
            .route("/api/v1/files", get(file_handler::list_files))
            .route("/api/v1/files", post(file_handler::upload_file))
            .route("/api/v1/files/:id", get(file_handler::get_file))
            .route("/api/v1/files/:id", delete(file_handler::delete_file))
            .route(
                "/api/v1/files/:id/complete",
                post(file_handler::complete_upload),
            )
            .route(
                "/api/v1/files/:id/download-url",
                get(file_handler::download_url),
            )
            .route(
                "/api/v1/files/:id/tags",
                put(file_handler::update_file_tags),
            )
    };

    public_routes
        .merge(api_routes)
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
