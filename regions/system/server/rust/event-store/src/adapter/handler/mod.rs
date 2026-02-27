pub mod error;
pub mod event_handler;
pub mod health;

use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::extract::State;
use axum::routing::{delete, get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::adapter::middleware::auth::{auth_middleware, EventStoreAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::domain::repository::{EventRepository, EventStreamRepository};
use crate::infrastructure::kafka::EventPublisher;
use crate::usecase::{
    AppendEventsUseCase, CreateSnapshotUseCase, DeleteStreamUseCase, GetLatestSnapshotUseCase,
    ReadEventsUseCase,
};

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub append_events_uc: Arc<AppendEventsUseCase>,
    pub read_events_uc: Arc<ReadEventsUseCase>,
    pub create_snapshot_uc: Arc<CreateSnapshotUseCase>,
    pub get_latest_snapshot_uc: Arc<GetLatestSnapshotUseCase>,
    pub delete_stream_uc: Arc<DeleteStreamUseCase>,
    pub stream_repo: Arc<dyn EventStreamRepository>,
    pub event_repo: Arc<dyn EventRepository>,
    pub event_publisher: Arc<dyn EventPublisher>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<EventStoreAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: EventStoreAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        event_handler::append_events,
        event_handler::read_events,
        event_handler::list_events,
        event_handler::list_streams,
        event_handler::delete_stream,
        event_handler::get_snapshot,
        event_handler::create_snapshot,
    ),
    components(schemas(
        event_handler::AppendEventsResponse,
        event_handler::ReadEventsResponse,
        event_handler::ListEventsResponse,
        event_handler::ListStreamsResponse,
        event_handler::StoredEventResponse,
        event_handler::MetadataResponse,
        event_handler::StreamResponse,
        event_handler::SnapshotResponse,
        event_handler::PaginationResponse,
        ErrorResponse,
        ErrorBody,
    )),
)]
struct ApiDoc;

/// REST API ルーターを構築する。
pub fn router(state: AppState) -> Router {
    // 認証不要のエンドポイント
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler));

    // 認証が設定されている場合は RBAC 付きルーティング、そうでなければオープンアクセス
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> events/read
        let read_routes = Router::new()
            .route(
                "/api/v1/events",
                get(event_handler::list_events),
            )
            .route(
                "/api/v1/events/:stream_id",
                get(event_handler::read_events),
            )
            .route("/api/v1/streams", get(event_handler::list_streams))
            .route(
                "/api/v1/streams/:stream_id/snapshot",
                get(event_handler::get_snapshot),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "events", "read",
            )));

        // POST events/snapshot -> events/write
        let write_routes = Router::new()
            .route(
                "/api/v1/events",
                post(event_handler::append_events),
            )
            .route(
                "/api/v1/streams/:stream_id/snapshot",
                post(event_handler::create_snapshot),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "events", "write",
            )));

        // DELETE stream -> events/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/streams/:stream_id",
                delete(event_handler::delete_stream),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "events", "admin",
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
                "/api/v1/events",
                post(event_handler::append_events).get(event_handler::list_events),
            )
            .route(
                "/api/v1/events/:stream_id",
                get(event_handler::read_events),
            )
            .route("/api/v1/streams", get(event_handler::list_streams))
            .route(
                "/api/v1/streams/:stream_id",
                delete(event_handler::delete_stream),
            )
            .route(
                "/api/v1/streams/:stream_id/snapshot",
                get(event_handler::get_snapshot).post(event_handler::create_snapshot),
            )
    };

    public_routes
        .merge(api_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}

/// ErrorResponse は統一エラーレスポンス。
#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub details: Vec<String>,
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                details: vec![],
            },
        }
    }
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
