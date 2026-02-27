pub mod error;
pub mod saga_handler;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::adapter::middleware::auth::{auth_middleware, SagaAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::usecase::{
    CancelSagaUseCase, ExecuteSagaUseCase, GetSagaUseCase, ListSagasUseCase,
    ListWorkflowsUseCase, RegisterWorkflowUseCase, StartSagaUseCase,
};

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub start_saga_uc: Arc<StartSagaUseCase>,
    pub get_saga_uc: Arc<GetSagaUseCase>,
    pub list_sagas_uc: Arc<ListSagasUseCase>,
    pub cancel_saga_uc: Arc<CancelSagaUseCase>,
    pub execute_saga_uc: Arc<ExecuteSagaUseCase>,
    pub register_workflow_uc: Arc<RegisterWorkflowUseCase>,
    pub list_workflows_uc: Arc<ListWorkflowsUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<SagaAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: SagaAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
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
        saga_handler::compensate_saga,
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
        saga_handler::CompensateSagaResponse,
        ErrorResponse,
        ErrorBody,
    )),
    security(("bearer_auth" = [])),
)]
struct ApiDoc;

/// REST API ルーターを構築する。
pub fn router(state: AppState) -> Router {
    // 認証不要のエンドポイント
    let public_routes = Router::new()
        .route("/healthz", get(saga_handler::healthz))
        .route("/readyz", get(saga_handler::readyz))
        .route("/metrics", get(saga_handler::metrics));

    // 認証が設定されている場合は RBAC 付きルーティング、そうでなければオープンアクセス
    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET -> sagas/read
        let read_routes = Router::new()
            .route("/api/v1/sagas", get(saga_handler::list_sagas))
            .route("/api/v1/sagas/:saga_id", get(saga_handler::get_saga))
            .route("/api/v1/workflows", get(saga_handler::list_workflows))
            .route_layer(axum::middleware::from_fn(require_permission(
                "sagas", "read",
            )));

        // POST/cancel/compensate -> sagas/write
        let write_routes = Router::new()
            .route("/api/v1/sagas", post(saga_handler::start_saga))
            .route(
                "/api/v1/sagas/:saga_id/cancel",
                post(saga_handler::cancel_saga),
            )
            .route(
                "/api/v1/sagas/:saga_id/compensate",
                post(saga_handler::compensate_saga),
            )
            .route("/api/v1/workflows", post(saga_handler::register_workflow))
            .route_layer(axum::middleware::from_fn(require_permission(
                "sagas", "write",
            )));

        // 認証ミドルウェアを全 API ルートに適用
        Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        // 認証なし（dev モード / テスト）: 従来どおり
        Router::new()
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
                post(saga_handler::compensate_saga),
            )
            .route(
                "/api/v1/workflows",
                post(saga_handler::register_workflow).get(saga_handler::list_workflows),
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
