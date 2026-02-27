pub mod dlq_handler;
pub mod error;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::adapter::middleware::auth::{auth_middleware, DlqAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::usecase::{
    DeleteMessageUseCase, GetMessageUseCase, ListMessagesUseCase, RetryAllUseCase,
    RetryMessageUseCase,
};

// Re-export shared error types for backward compatibility within the crate.
pub use k1s0_server_common::{ErrorBody, ErrorResponse};

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub list_messages_uc: Arc<ListMessagesUseCase>,
    pub get_message_uc: Arc<GetMessageUseCase>,
    pub retry_message_uc: Arc<RetryMessageUseCase>,
    pub delete_message_uc: Arc<DeleteMessageUseCase>,
    pub retry_all_uc: Arc<RetryAllUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<DlqAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: DlqAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        dlq_handler::healthz,
        dlq_handler::readyz,
        dlq_handler::metrics,
        dlq_handler::list_messages,
        dlq_handler::get_message,
        dlq_handler::retry_message,
        dlq_handler::delete_message,
        dlq_handler::retry_all,
    ),
    components(schemas(
        dlq_handler::DlqMessageResponse,
        dlq_handler::ListMessagesResponse,
        dlq_handler::PaginationResponse,
        dlq_handler::RetryMessageResponse,
        dlq_handler::RetryAllResponse,
        dlq_handler::DeleteMessageResponse,
    )),
    security(("bearer_auth" = [])),
)]
struct ApiDoc;

/// REST API ルーターを構築する。
pub fn router(state: AppState) -> Router {
    // 認証不要のエンドポイント
    let public_routes = Router::new()
        .route("/healthz", get(dlq_handler::healthz))
        .route("/readyz", get(dlq_handler::readyz))
        .route("/metrics", get(dlq_handler::metrics));

    // 認証が設定されている場合は RBAC 付きルーティング、そうでなければオープンアクセス
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> dlq/read
        let read_routes = Router::new()
            .route(
                "/api/v1/dlq/messages/:id",
                get(dlq_handler::get_message),
            )
            .route("/api/v1/dlq/:topic", get(dlq_handler::list_messages))
            .route_layer(axum::middleware::from_fn(require_permission(
                "dlq", "read",
            )));

        // POST retry -> dlq/write
        let write_routes = Router::new()
            .route(
                "/api/v1/dlq/messages/:id/retry",
                post(dlq_handler::retry_message),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "dlq", "write",
            )));

        // DELETE / retry-all -> dlq/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/dlq/messages/:id",
                axum::routing::delete(dlq_handler::delete_message),
            )
            .route(
                "/api/v1/dlq/:topic/retry-all",
                post(dlq_handler::retry_all),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "dlq", "admin",
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
            .route(
                "/api/v1/dlq/messages/:id",
                get(dlq_handler::get_message).delete(dlq_handler::delete_message),
            )
            .route(
                "/api/v1/dlq/messages/:id/retry",
                post(dlq_handler::retry_message),
            )
            .route("/api/v1/dlq/:topic", get(dlq_handler::list_messages))
            .route("/api/v1/dlq/:topic/retry-all", post(dlq_handler::retry_all))
    };

    public_routes
        .merge(api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
