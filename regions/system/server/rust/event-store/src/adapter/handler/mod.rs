pub mod error;
pub mod event_handler;
pub mod health;

use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::extract::State;
use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::domain::repository::{EventRepository, EventStreamRepository};
use crate::infrastructure::kafka::EventPublisher;
use crate::usecase::{
    AppendEventsUseCase, CreateSnapshotUseCase, GetLatestSnapshotUseCase, ReadEventsUseCase,
};

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub append_events_uc: Arc<AppendEventsUseCase>,
    pub read_events_uc: Arc<ReadEventsUseCase>,
    pub create_snapshot_uc: Arc<CreateSnapshotUseCase>,
    pub get_latest_snapshot_uc: Arc<GetLatestSnapshotUseCase>,
    pub stream_repo: Arc<dyn EventStreamRepository>,
    pub event_repo: Arc<dyn EventRepository>,
    pub event_publisher: Arc<dyn EventPublisher>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        event_handler::append_events,
        event_handler::read_events,
        event_handler::list_events,
        event_handler::list_streams,
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
    Router::new()
        // Health / Readiness / Metrics
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler))
        // Events
        .route(
            "/api/v1/events",
            post(event_handler::append_events).get(event_handler::list_events),
        )
        .route(
            "/api/v1/events/:stream_id",
            get(event_handler::read_events),
        )
        // Streams
        .route("/api/v1/streams", get(event_handler::list_streams))
        .route(
            "/api/v1/streams/:stream_id/snapshot",
            get(event_handler::get_snapshot).post(event_handler::create_snapshot),
        )
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
