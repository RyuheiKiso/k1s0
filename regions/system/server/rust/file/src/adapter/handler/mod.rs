pub mod file_handler;
pub mod health;
pub mod storage_handler;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::{auth_middleware, AuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::usecase::{
    CompleteUploadUseCase, DeleteFileUseCase, GenerateDownloadUrlUseCase, GenerateUploadUrlUseCase,
    GetFileMetadataUseCase, ListFilesUseCase, UpdateFileTagsUseCase,
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
    pub auth_state: Option<AuthState>,
    /// DB 接続確認用のコネクションプール（CRITICAL-003 対応: /readyz で SELECT 1 チェックに使用）
    pub db_pool: Option<sqlx::PgPool>,
    /// STATIC-HIGH-003 監査対応: ローカルFSストレージのルートパス。
    /// /internal/storage/ ハンドラーがファイルを提供するために使用する。
    /// S3 バックエンド使用時は None。
    pub storage_root_path: Option<std::path::PathBuf>,
}

impl AppState {
    #[must_use]
    pub fn with_auth(mut self, auth_state: AuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    // 認証不要のエンドポイント
    // STATIC-HIGH-003 監査対応: /internal/storage/ はローカルFS用のファイル配信エンドポイント。
    // Content-Disposition: attachment ヘッダーでブラウザの自動実行を防止する。
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler))
        .route(
            "/internal/storage/{*key}",
            get(storage_handler::serve_internal_storage),
        );

    // 認証が設定されている場合は RBAC 付きルーティング、そうでなければオープンアクセス
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> files/read
        let read_routes = Router::new()
            .route("/api/v1/files", get(file_handler::list_files))
            .route("/api/v1/files/{id}", get(file_handler::get_file))
            .route(
                "/api/v1/files/{id}/download-url",
                get(file_handler::download_url),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "files", "read",
            )));

        // POST/complete/tags/DELETE -> files/write (DELETE has owner/admin check in handler)
        let write_routes = Router::new()
            .route("/api/v1/files", post(file_handler::upload_file))
            .route(
                "/api/v1/files/{id}/complete",
                post(file_handler::complete_upload),
            )
            .route(
                "/api/v1/files/{id}/tags",
                put(file_handler::update_file_tags),
            )
            .route("/api/v1/files/{id}", delete(file_handler::delete_file))
            .route_layer(axum::middleware::from_fn(require_permission(
                "files", "write",
            )));

        let admin_routes = Router::new()
            .route(
                "/api/v1/files/admin/{id}",
                delete(file_handler::delete_file_admin),
            )
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
            .route("/api/v1/files/{id}", get(file_handler::get_file))
            .route("/api/v1/files/{id}", delete(file_handler::delete_file))
            .route(
                "/api/v1/files/admin/{id}",
                delete(file_handler::delete_file_admin),
            )
            .route(
                "/api/v1/files/{id}/complete",
                post(file_handler::complete_upload),
            )
            .route(
                "/api/v1/files/{id}/download-url",
                get(file_handler::download_url),
            )
            .route(
                "/api/v1/files/{id}/tags",
                put(file_handler::update_file_tags),
            )
    };

    public_routes.merge(api_routes).with_state(state)
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
