pub mod error;
pub mod saga_handler;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::usecase::{
    CancelSagaUseCase, GetSagaUseCase, ListSagasUseCase, ListWorkflowsUseCase,
    RegisterWorkflowUseCase, StartSagaUseCase,
};

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub start_saga_uc: Arc<StartSagaUseCase>,
    pub get_saga_uc: Arc<GetSagaUseCase>,
    pub list_sagas_uc: Arc<ListSagasUseCase>,
    pub cancel_saga_uc: Arc<CancelSagaUseCase>,
    pub register_workflow_uc: Arc<RegisterWorkflowUseCase>,
    pub list_workflows_uc: Arc<ListWorkflowsUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        saga_handler::healthz,
        saga_handler::readyz,
        saga_handler::metrics,
        saga_handler::start_saga,
        saga_handler::list_sagas,
        saga_handler::get_saga,
        saga_handler::cancel_saga,
        saga_handler::register_workflow,
        saga_handler::list_workflows,
    ),
    components(schemas(
        saga_handler::StartSagaRequest,
        saga_handler::StartSagaResponse,
        saga_handler::SagaResponse,
        saga_handler::SagaDetailResponse,
        saga_handler::StepLogResponse,
        saga_handler::ListSagasResponse,
        saga_handler::PaginationResponse,
        saga_handler::RegisterWorkflowRequest,
        saga_handler::RegisterWorkflowResponse,
        saga_handler::WorkflowSummaryResponse,
        saga_handler::ListWorkflowsResponse,
        saga_handler::CancelSagaResponse,
        ErrorResponse,
        ErrorBody,
    )),
    security(("bearer_auth" = [])),
)]
struct ApiDoc;

/// REST API ルーターを構築する。
pub fn router(state: AppState) -> Router {
    Router::new()
        // Health / Readiness / Metrics
        .route("/healthz", get(saga_handler::healthz))
        .route("/readyz", get(saga_handler::readyz))
        .route("/metrics", get(saga_handler::metrics))
        // Saga endpoints
        .route(
            "/api/v1/sagas",
            post(saga_handler::start_saga).get(saga_handler::list_sagas),
        )
        .route("/api/v1/sagas/:saga_id", get(saga_handler::get_saga))
        .route(
            "/api/v1/sagas/:saga_id/cancel",
            post(saga_handler::cancel_saga),
        )
        .route(
            "/api/v1/sagas/:saga_id/compensate",
            post(saga_handler::cancel_saga),
        )
        // Workflow endpoints
        .route(
            "/api/v1/workflows",
            post(saga_handler::register_workflow).get(saga_handler::list_workflows),
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
