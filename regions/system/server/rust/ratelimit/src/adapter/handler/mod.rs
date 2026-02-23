pub mod ratelimit_handler;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::usecase::{CheckRateLimitUseCase, CreateRuleUseCase, GetRuleUseCase};

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub check_uc: Arc<CheckRateLimitUseCase>,
    pub create_uc: Arc<CreateRuleUseCase>,
    pub get_uc: Arc<GetRuleUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub db_pool: Option<sqlx::PgPool>,
}

impl AppState {
    pub fn new(
        check_uc: Arc<CheckRateLimitUseCase>,
        create_uc: Arc<CreateRuleUseCase>,
        get_uc: Arc<GetRuleUseCase>,
        db_pool: Option<sqlx::PgPool>,
    ) -> Self {
        Self {
            check_uc,
            create_uc,
            get_uc,
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
                "k1s0-ratelimit-server",
            )),
            db_pool,
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        ratelimit_handler::healthz,
        ratelimit_handler::readyz,
        ratelimit_handler::metrics,
        ratelimit_handler::check_rate_limit,
        ratelimit_handler::create_rule,
        ratelimit_handler::get_rule,
    ),
    components(schemas(
        ratelimit_handler::CheckRateLimitRequest,
        ratelimit_handler::CheckRateLimitResponse,
        ratelimit_handler::CreateRuleRequest,
        ratelimit_handler::RuleResponse,
        ErrorResponse,
        ErrorBody,
    )),
)]
struct ApiDoc;

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    let api_routes = Router::new()
        .route(
            "/api/v1/ratelimit/check",
            post(ratelimit_handler::check_rate_limit),
        )
        .route(
            "/api/v1/ratelimit/rules",
            post(ratelimit_handler::create_rule),
        )
        .route(
            "/api/v1/ratelimit/rules/:id",
            get(ratelimit_handler::get_rule),
        );

    let public = Router::new()
        .route("/healthz", get(ratelimit_handler::healthz))
        .route("/readyz", get(ratelimit_handler::readyz))
        .route("/metrics", get(ratelimit_handler::metrics));

    Router::new()
        .merge(api_routes)
        .merge(public)
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
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
            },
        }
    }
}
