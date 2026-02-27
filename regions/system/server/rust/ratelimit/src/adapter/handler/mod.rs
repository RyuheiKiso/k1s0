pub mod ratelimit_handler;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::adapter::middleware::auth::{auth_middleware, RatelimitAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::usecase::{
    CheckRateLimitUseCase, CreateRuleUseCase, DeleteRuleUseCase, GetRuleUseCase, GetUsageUseCase,
    ListRulesUseCase, ResetRateLimitUseCase, UpdateRuleUseCase,
};

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub check_uc: Arc<CheckRateLimitUseCase>,
    pub create_uc: Arc<CreateRuleUseCase>,
    pub get_uc: Arc<GetRuleUseCase>,
    pub list_uc: Arc<ListRulesUseCase>,
    pub update_uc: Arc<UpdateRuleUseCase>,
    pub delete_uc: Arc<DeleteRuleUseCase>,
    pub get_usage_uc: Arc<GetUsageUseCase>,
    pub reset_uc: Arc<ResetRateLimitUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub db_pool: Option<sqlx::PgPool>,
    pub auth_state: Option<RatelimitAuthState>,
}

impl AppState {
    pub fn new(
        check_uc: Arc<CheckRateLimitUseCase>,
        create_uc: Arc<CreateRuleUseCase>,
        get_uc: Arc<GetRuleUseCase>,
        list_uc: Arc<ListRulesUseCase>,
        update_uc: Arc<UpdateRuleUseCase>,
        delete_uc: Arc<DeleteRuleUseCase>,
        get_usage_uc: Arc<GetUsageUseCase>,
        reset_uc: Arc<ResetRateLimitUseCase>,
        db_pool: Option<sqlx::PgPool>,
    ) -> Self {
        Self {
            check_uc,
            create_uc,
            get_uc,
            list_uc,
            update_uc,
            delete_uc,
            get_usage_uc,
            reset_uc,
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new(
                "k1s0-ratelimit-server",
            )),
            db_pool,
            auth_state: None,
        }
    }

    pub fn with_auth(mut self, auth_state: RatelimitAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        ratelimit_handler::healthz,
        ratelimit_handler::readyz,
        ratelimit_handler::metrics,
        ratelimit_handler::check_rate_limit,
        ratelimit_handler::reset_rate_limit,
        ratelimit_handler::create_rule,
        ratelimit_handler::get_rule,
        ratelimit_handler::list_rules,
        ratelimit_handler::update_rule,
        ratelimit_handler::delete_rule,
        ratelimit_handler::get_usage,
    ),
    components(schemas(
        ratelimit_handler::CheckRateLimitRequest,
        ratelimit_handler::CheckRateLimitResponse,
        ratelimit_handler::ResetRateLimitRequest,
        ratelimit_handler::CreateRuleRequest,
        ratelimit_handler::UpdateRuleRequest,
        ratelimit_handler::RuleResponse,
        ratelimit_handler::UsageResponse,
        ErrorResponse,
        ErrorBody,
    )),
)]
struct ApiDoc;

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(ratelimit_handler::healthz))
        .route("/readyz", get(ratelimit_handler::readyz))
        .route("/metrics", get(ratelimit_handler::metrics));

    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // GET rules/usage -> ratelimit/read
        let read_routes = Router::new()
            .route(
                "/api/v1/ratelimit/rules",
                get(ratelimit_handler::list_rules),
            )
            .route(
                "/api/v1/ratelimit/rules/:id",
                get(ratelimit_handler::get_rule),
            )
            .route(
                "/api/v1/ratelimit/usage",
                get(ratelimit_handler::get_usage),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "ratelimit", "read",
            )));

        // POST check/rules -> ratelimit/write
        let write_routes = Router::new()
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
                axum::routing::put(ratelimit_handler::update_rule),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "ratelimit", "write",
            )));

        // DELETE rules/reset -> ratelimit/admin
        let admin_routes = Router::new()
            .route(
                "/api/v1/ratelimit/rules/:id",
                axum::routing::delete(ratelimit_handler::delete_rule),
            )
            .route(
                "/api/v1/ratelimit/reset",
                post(ratelimit_handler::reset_rate_limit),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "ratelimit", "admin",
            )));

        Router::new()
            .merge(read_routes)
            .merge(write_routes)
            .merge(admin_routes)
            .layer(axum::middleware::from_fn_with_state(
                auth_state.clone(),
                auth_middleware,
            ))
    } else {
        Router::new()
            .route(
                "/api/v1/ratelimit/check",
                post(ratelimit_handler::check_rate_limit),
            )
            .route(
                "/api/v1/ratelimit/reset",
                post(ratelimit_handler::reset_rate_limit),
            )
            .route(
                "/api/v1/ratelimit/rules",
                get(ratelimit_handler::list_rules).post(ratelimit_handler::create_rule),
            )
            .route(
                "/api/v1/ratelimit/rules/:id",
                get(ratelimit_handler::get_rule)
                    .put(ratelimit_handler::update_rule)
                    .delete(ratelimit_handler::delete_rule),
            )
            .route(
                "/api/v1/ratelimit/usage",
                get(ratelimit_handler::get_usage),
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
