pub mod health;
pub mod rule_handler;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::{auth_middleware, AuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::usecase::{
    CreateRuleSetUseCase, CreateRuleUseCase, DeleteRuleSetUseCase, DeleteRuleUseCase,
    EvaluateUseCase, GetRuleSetUseCase, GetRuleUseCase, ListEvaluationLogsUseCase,
    ListRuleSetsUseCase, ListRulesUseCase, PublishRuleSetUseCase, RollbackRuleSetUseCase,
    UpdateRuleSetUseCase, UpdateRuleUseCase,
};

#[derive(Clone)]
pub struct AppState {
    pub create_rule_uc: Arc<CreateRuleUseCase>,
    pub get_rule_uc: Arc<GetRuleUseCase>,
    pub list_rules_uc: Arc<ListRulesUseCase>,
    pub update_rule_uc: Arc<UpdateRuleUseCase>,
    pub delete_rule_uc: Arc<DeleteRuleUseCase>,
    pub create_rule_set_uc: Arc<CreateRuleSetUseCase>,
    pub get_rule_set_uc: Arc<GetRuleSetUseCase>,
    pub list_rule_sets_uc: Arc<ListRuleSetsUseCase>,
    pub update_rule_set_uc: Arc<UpdateRuleSetUseCase>,
    pub delete_rule_set_uc: Arc<DeleteRuleSetUseCase>,
    pub publish_rule_set_uc: Arc<PublishRuleSetUseCase>,
    pub rollback_rule_set_uc: Arc<RollbackRuleSetUseCase>,
    pub evaluate_uc: Arc<EvaluateUseCase>,
    pub list_evaluation_logs_uc: Arc<ListEvaluationLogsUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<AuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: AuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(metrics_handler));

    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // rules/read -> sys_auditor+
        let read_routes = Router::new()
            .route("/api/v1/rules", get(rule_handler::list_rules))
            .route("/api/v1/rules/{id}", get(rule_handler::get_rule))
            .route("/api/v1/rule-sets", get(rule_handler::list_rule_sets))
            .route("/api/v1/rule-sets/{id}", get(rule_handler::get_rule_set))
            .route(
                "/api/v1/evaluation-logs",
                get(rule_handler::list_evaluation_logs),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "rules", "read",
            )));

        // rules/write -> sys_operator+
        let write_routes = Router::new()
            .route("/api/v1/rules", post(rule_handler::create_rule))
            .route("/api/v1/rules/{id}", put(rule_handler::update_rule))
            .route("/api/v1/rule-sets", post(rule_handler::create_rule_set))
            .route("/api/v1/rule-sets/{id}", put(rule_handler::update_rule_set))
            .route("/api/v1/evaluate", post(rule_handler::evaluate))
            .route(
                "/api/v1/evaluate/dry-run",
                post(rule_handler::evaluate_dry_run),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "rules", "write",
            )));

        // rules/admin -> sys_admin only
        let admin_routes = Router::new()
            .route("/api/v1/rules/{id}", delete(rule_handler::delete_rule))
            .route(
                "/api/v1/rule-sets/{id}",
                delete(rule_handler::delete_rule_set),
            )
            .route(
                "/api/v1/rule-sets/{id}/publish",
                post(rule_handler::publish_rule_set),
            )
            .route(
                "/api/v1/rule-sets/{id}/rollback",
                post(rule_handler::rollback_rule_set),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "rules", "admin",
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
            .route("/api/v1/rules", get(rule_handler::list_rules))
            .route("/api/v1/rules", post(rule_handler::create_rule))
            .route("/api/v1/rules/{id}", get(rule_handler::get_rule))
            .route("/api/v1/rules/{id}", put(rule_handler::update_rule))
            .route("/api/v1/rules/{id}", delete(rule_handler::delete_rule))
            .route("/api/v1/rule-sets", get(rule_handler::list_rule_sets))
            .route("/api/v1/rule-sets", post(rule_handler::create_rule_set))
            .route("/api/v1/rule-sets/{id}", get(rule_handler::get_rule_set))
            .route("/api/v1/rule-sets/{id}", put(rule_handler::update_rule_set))
            .route(
                "/api/v1/rule-sets/{id}",
                delete(rule_handler::delete_rule_set),
            )
            .route(
                "/api/v1/rule-sets/{id}/publish",
                post(rule_handler::publish_rule_set),
            )
            .route(
                "/api/v1/rule-sets/{id}/rollback",
                post(rule_handler::rollback_rule_set),
            )
            .route("/api/v1/evaluate", post(rule_handler::evaluate))
            .route(
                "/api/v1/evaluate/dry-run",
                post(rule_handler::evaluate_dry_run),
            )
            .route(
                "/api/v1/evaluation-logs",
                get(rule_handler::list_evaluation_logs),
            )
    };

    Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .with_state(state)
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}
