pub mod audit_handler;
pub mod auth_handler;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};

use crate::adapter::middleware::auth::auth_middleware;
use crate::adapter::middleware::rbac::rbac_middleware;
use crate::domain::repository::{AuditLogRepository, UserRepository};
use crate::infrastructure::TokenVerifier;
use crate::usecase::{
    CheckPermissionUseCase, GetUserUseCase, ListUsersUseCase,
    RecordAuditLogUseCase, SearchAuditLogsUseCase, ValidateTokenUseCase,
};

/// AppState はアプリケーション全体の共有状態を表す。
#[derive(Clone)]
pub struct AppState {
    pub validate_token_uc: Arc<ValidateTokenUseCase>,
    pub get_user_uc: Arc<GetUserUseCase>,
    pub list_users_uc: Arc<ListUsersUseCase>,
    pub record_audit_log_uc: Arc<RecordAuditLogUseCase>,
    pub search_audit_logs_uc: Arc<SearchAuditLogsUseCase>,
    pub check_permission_uc: Arc<CheckPermissionUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub db_pool: Option<sqlx::PgPool>,
    pub keycloak_url: Option<String>,
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
            list_users_uc: Arc::new(ListUsersUseCase::new(user_repo)),
            record_audit_log_uc: Arc::new(RecordAuditLogUseCase::new(audit_repo.clone())),
            search_audit_logs_uc: Arc::new(SearchAuditLogsUseCase::new(audit_repo)),
            check_permission_uc: Arc::new(CheckPermissionUseCase::new()),
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-auth-server")),
            db_pool,
            keycloak_url,
        }
    }
}

/// REST API ルーターを構築する。
pub fn router(state: AppState) -> Router {
    // 認証が必要なエンドポイント (auth_middleware を適用)
    let protected = Router::new()
        // User endpoints (sys_auditor以上)
        .route("/api/v1/users", get(auth_handler::list_users))
        .route("/api/v1/users/:id", get(auth_handler::get_user))
        .route("/api/v1/users/:id/roles", get(auth_handler::get_user_roles))
        // Auth endpoints (sys_operator以上)
        .route(
            "/api/v1/auth/token/introspect",
            post(auth_handler::introspect_token),
        )
        .route(
            "/api/v1/auth/permissions/check",
            post(auth_handler::check_permission),
        )
        // Audit log endpoints (sys_auditor以上: GET, sys_operator以上: POST)
        .route(
            "/api/v1/audit/logs",
            post(audit_handler::record_audit_log)
                .get(audit_handler::search_audit_logs),
        )
        // rbac_middleware: sys_auditor以上のロールを持つユーザーのみ通過 (auth_middlewareの後に実行)
        .route_layer(middleware::from_fn_with_state(state.clone(), rbac_middleware))
        // auth_middleware: Bearerトークンを検証しClaimsをextensionに格納 (最初に実行)
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // 公開エンドポイント (認証不要)
    let public = Router::new()
        // Health / Readiness / Metrics
        .route("/healthz", get(auth_handler::healthz))
        .route("/readyz", get(auth_handler::readyz))
        .route("/metrics", get(auth_handler::metrics))
        // トークン検証は認証サービス自身なので公開
        .route(
            "/api/v1/auth/token/validate",
            post(auth_handler::validate_token),
        );

    Router::new()
        .merge(protected)
        .merge(public)
        .with_state(state)
}

/// ErrorResponse は統一エラーレスポンス。
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, serde::Serialize)]
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
