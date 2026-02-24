pub mod dlq_handler;
pub mod error;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

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
    Router::new()
        // Health / Readiness / Metrics
        .route("/healthz", get(dlq_handler::healthz))
        .route("/readyz", get(dlq_handler::readyz))
        .route("/metrics", get(dlq_handler::metrics))
        // messages を先に定義（:topic との競合を避ける）
        .route(
            "/api/v1/dlq/messages/:id",
            get(dlq_handler::get_message).delete(dlq_handler::delete_message),
        )
        .route(
            "/api/v1/dlq/messages/:id/retry",
            post(dlq_handler::retry_message),
        )
        // topic-based endpoints
        .route("/api/v1/dlq/:topic", get(dlq_handler::list_messages))
        .route("/api/v1/dlq/:topic/retry-all", post(dlq_handler::retry_all))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
