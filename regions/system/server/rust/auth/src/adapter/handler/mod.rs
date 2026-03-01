pub mod api_key_handler;
pub mod audit_handler;
pub mod auth_handler;
pub mod jwks_handler;
pub mod navigation_handler;

use std::path::PathBuf;
use std::sync::Arc;

use axum::middleware;
use axum::routing::{delete, get, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub use k1s0_server_common::{ErrorBody, ErrorResponse};

use crate::adapter::middleware::auth::auth_middleware;
use crate::adapter::middleware::rbac::make_rbac_middleware;
use crate::domain::repository::{AuditLogRepository, UserRepository};
use crate::infrastructure::TokenVerifier;
use crate::usecase::{
    CheckPermissionUseCase, CreateApiKeyUseCase, GetApiKeyUseCase, GetUserRolesUseCase,
    GetUserUseCase, ListApiKeysUseCase, ListUsersUseCase, RecordAuditLogUseCase,
    RevokeApiKeyUseCase, SearchAuditLogsUseCase, ValidateTokenUseCase,
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
    pub create_api_key_uc: Arc<CreateApiKeyUseCase>,
    pub get_api_key_uc: Arc<GetApiKeyUseCase>,
    pub list_api_keys_uc: Arc<ListApiKeysUseCase>,
    pub revoke_api_key_uc: Arc<RevokeApiKeyUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub db_pool: Option<sqlx::PgPool>,
    pub keycloak_url: Option<String>,
    pub jwks_provider: Option<crate::infrastructure::jwks_provider::JwksProvider>,
    pub navigation_config_path: Option<PathBuf>,
}

impl AppState {
    pub fn new(
        token_verifier: Arc<dyn TokenVerifier>,
        user_repo: Arc<dyn UserRepository>,
        audit_repo: Arc<dyn AuditLogRepository>,
        api_key_repo: Arc<dyn crate::domain::repository::ApiKeyRepository>,
        expected_issuer: String,
        expected_audience: String,
        db_pool: Option<sqlx::PgPool>,
        keycloak_url: Option<String>,
        jwks_provider: Option<crate::infrastructure::jwks_provider::JwksProvider>,
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
            create_api_key_uc: Arc::new(CreateApiKeyUseCase::new(api_key_repo.clone())),
            get_api_key_uc: Arc::new(GetApiKeyUseCase::new(api_key_repo.clone())),
            list_api_keys_uc: Arc::new(ListApiKeysUseCase::new(api_key_repo.clone())),
            revoke_api_key_uc: Arc::new(RevokeApiKeyUseCase::new(api_key_repo)),
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-auth-server")),
            db_pool,
            keycloak_url,
            jwks_provider,
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
        jwks_handler::jwks,
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

    // API key endpoints: require "api_keys" permission
    let api_key_write_routes = Router::new()
        .route("/api/v1/api-keys", post(api_key_handler::create_api_key))
        .route(
            "/api/v1/api-keys/:id/revoke",
            delete(api_key_handler::revoke_api_key),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("api_keys", "write"),
        ));

    let api_key_read_routes = Router::new()
        .route("/api/v1/api-keys", get(api_key_handler::list_api_keys))
        .route("/api/v1/api-keys/:id", get(api_key_handler::get_api_key))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            make_rbac_middleware("api_keys", "read"),
        ));

    // Protected routes share auth_middleware for Bearer token validation
    let protected = Router::new()
        .merge(user_routes)
        .merge(permission_routes)
        .merge(audit_read_routes)
        .merge(audit_write_routes)
        .merge(api_key_read_routes)
        .merge(api_key_write_routes)
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
        // JWKS endpoint (public)
        .route("/jwks", get(jwks_handler::jwks))
        .route("/.well-known/jwks.json", get(jwks_handler::jwks))
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

// ErrorResponse / ErrorBody は k1s0-server-common から re-export。
// REST-API設計.md D-007 の統一 JSON スキーマに従う。
