pub mod audit_handler;
pub mod auth_handler;
pub mod navigation_handler;

use std::path::PathBuf;
use std::sync::Arc;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::adapter::middleware::auth::auth_middleware;
use crate::adapter::middleware::rbac::make_rbac_middleware;
use crate::domain::repository::{AuditLogRepository, UserRepository};
use crate::infrastructure::TokenVerifier;
use crate::usecase::{
    CheckPermissionUseCase, GetUserRolesUseCase, GetUserUseCase, ListUsersUseCase,
    RecordAuditLogUseCase, SearchAuditLogsUseCase, ValidateTokenUseCase,
};

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub validate_token_uc: Arc<ValidateTokenUseCase>,
    pub get_user_uc: Arc<GetUserUseCase>,
    pub get_user_roles_uc: Arc<GetUserRolesUseCase>,
    pub list_users_uc: Arc<ListUsersUseCase>,
    pub record_audit_log_uc: Arc<RecordAuditLogUseCase>,
    pub search_audit_logs_uc: Arc<SearchAuditLogsUseCase>,
    pub check_permission_uc: Arc<CheckPermissionUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub db_pool: Option<sqlx::PgPool>,
    pub keycloak_url: Option<String>,
    pub navigation_config_path: Option<PathBuf>,
}

impl AppState {
    pub fn new(
        token_verifier: Arc<dyn TokenVerifier>,
        user_repo: Arc<dyn UserRepository>,
        audit_repo: Arc<dyn AuditLogRepository>,
        expected_issuer: String,
        expected_audience: String,
        db_pool: Option<sqlx::PgPool>,
        keycloak_url: Option<String>,
    ) -> Self {
        Self {
            validate_token_uc: Arc::new(ValidateTokenUseCase::new(
                token_verifier,
                expected_issuer,
                expected_audience,
            )),
            get_user_uc: Arc::new(GetUserUseCase::new(user_repo.clone())),
            get_user_roles_uc: Arc::new(GetUserRolesUseCase::new(user_repo.clone())),
            list_users_uc: Arc::new(ListUsersUseCase::new(user_repo)),
            record_audit_log_uc: Arc::new(RecordAuditLogUseCase::new(audit_repo.clone())),
            search_audit_logs_uc: Arc::new(SearchAuditLogsUseCase::new(audit_repo)),
            check_permission_uc: Arc::new(CheckPermissionUseCase::new()),
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-auth-server")),
            db_pool,
            keycloak_url,
            navigation_config_path: None,
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handler::healthz,
        auth_handler::readyz,
        auth_handler::metrics,
        auth_handler::validate_token,
        auth_handler::introspect_token,
        auth_handler::get_user,
        auth_handler::list_users,
        auth_handler::check_permission,
        auth_handler::get_user_roles,
        audit_handler::record_audit_log,
        audit_handler::search_audit_logs,
        navigation_handler::get_navigation,
    ),
    components(schemas(
        crate::domain::entity::claims::Claims,
        crate::domain::entity::claims::RealmAccess,
        crate::domain::entity::claims::ResourceAccess,
        crate::domain::entity::user::User,
        crate::domain::entity::user::Role,
        crate::domain::entity::user::UserRoles,
        crate::domain::entity::user::Pagination,
        crate::domain::entity::user::UserListResult,
        crate::domain::entity::audit_log::AuditLog,
        crate::domain::entity::audit_log::CreateAuditLogRequest,
        crate::domain::entity::audit_log::AuditLogSearchResult,
        crate::domain::entity::audit_log::CreateAuditLogResponse,
        auth_handler::ValidateTokenRequest,
        auth_handler::IntrospectTokenRequest,
        auth_handler::CheckPermissionRequest,
        ErrorResponse,
        ErrorBody,
    )),
    security(("bearer_auth" = [])),
)]
struct ApiDoc;

/// Build the REST API router.
pub fn router(state: AppState) -> Router {
    // User endpoints: require "users" / "read" permission
    let user_routes = Router::new()
        .route("/api/v1/users", get(auth_handler::list_users))
        .route("/api/v1/users/:id", get(auth_handler::get_user))
        .route("/api/v1/users/:id/roles", get(auth_handler::get_user_roles))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("users", "read"),
        ));

    // Permission check endpoint: require "auth_config" / "read" permission
    let permission_routes = Router::new()
        .route(
            "/api/v1/auth/permissions/check",
            post(auth_handler::check_permission),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("auth_config", "read"),
        ));

    // Audit log endpoints: GET requires "audit_logs" / "read", POST requires "audit_logs" / "write"
    let audit_read_routes = Router::new()
        .route("/api/v1/audit/logs", get(audit_handler::search_audit_logs))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("audit_logs", "read"),
        ));

    let audit_write_routes = Router::new()
        .route("/api/v1/audit/logs", post(audit_handler::record_audit_log))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("audit_logs", "write"),
        ));

    // Protected routes share auth_middleware for Bearer token validation
    let protected = Router::new()
        .merge(user_routes)
        .merge(permission_routes)
        .merge(audit_read_routes)
        .merge(audit_write_routes)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Public endpoints (no auth required)
    let public = Router::new()
        // Health / Readiness / Metrics
        .route("/healthz", get(auth_handler::healthz))
        .route("/readyz", get(auth_handler::readyz))
        .route("/metrics", get(auth_handler::metrics))
        // Token validate/introspect are public (RFC 7662)
        .route(
            "/api/v1/auth/token/validate",
            post(auth_handler::validate_token),
        )
        .route(
            "/api/v1/auth/token/introspect",
            post(auth_handler::introspect_token),
        )
        // Navigation config (public)
        .route(
            "/api/v1/navigation",
            get(navigation_handler::get_navigation),
        );

    Router::new()
        .merge(protected)
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
