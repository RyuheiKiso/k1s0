pub mod event_monitor_handler;
pub mod health;

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::adapter::middleware::auth::{auth_middleware, EventMonitorAuthState};
use crate::adapter::middleware::rbac::require_permission;
use crate::usecase::{
    CreateFlowUseCase, DeleteFlowUseCase, ExecuteReplayUseCase, GetFlowInstanceUseCase,
    GetFlowInstancesUseCase, GetFlowKpiUseCase, GetFlowUseCase, GetKpiSummaryUseCase,
    GetSloBurnRateUseCase, GetSloStatusUseCase, ListEventsUseCase, ListFlowsUseCase,
    PreviewReplayUseCase, TraceByCorrelationUseCase, UpdateFlowUseCase,
};

#[derive(Clone)]
pub struct AppState {
    pub list_events_uc: Arc<ListEventsUseCase>,
    pub trace_by_correlation_uc: Arc<TraceByCorrelationUseCase>,
    pub create_flow_uc: Arc<CreateFlowUseCase>,
    pub get_flow_uc: Arc<GetFlowUseCase>,
    pub update_flow_uc: Arc<UpdateFlowUseCase>,
    pub delete_flow_uc: Arc<DeleteFlowUseCase>,
    pub list_flows_uc: Arc<ListFlowsUseCase>,
    pub get_flow_instances_uc: Arc<GetFlowInstancesUseCase>,
    pub get_flow_instance_uc: Arc<GetFlowInstanceUseCase>,
    pub get_flow_kpi_uc: Arc<GetFlowKpiUseCase>,
    pub get_kpi_summary_uc: Arc<GetKpiSummaryUseCase>,
    pub get_slo_status_uc: Arc<GetSloStatusUseCase>,
    pub get_slo_burn_rate_uc: Arc<GetSloBurnRateUseCase>,
    pub preview_replay_uc: Arc<PreviewReplayUseCase>,
    pub execute_replay_uc: Arc<ExecuteReplayUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<EventMonitorAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: EventMonitorAuthState) -> Self {
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
        // read -> event_monitor/read (sys_auditor+)
        let read_routes = Router::new()
            .route("/api/v1/events", get(event_monitor_handler::list_events))
            .route(
                "/api/v1/events/trace/:correlation_id",
                get(event_monitor_handler::trace_by_correlation),
            )
            .route("/api/v1/flows", get(event_monitor_handler::list_flows))
            .route("/api/v1/flows/:id", get(event_monitor_handler::get_flow))
            .route(
                "/api/v1/flows/:id/instances",
                get(event_monitor_handler::get_flow_instances),
            )
            .route(
                "/api/v1/flows/:id/instances/:instance_id",
                get(event_monitor_handler::get_flow_instance),
            )
            .route(
                "/api/v1/flows/:id/kpi",
                get(event_monitor_handler::get_flow_kpi),
            )
            .route(
                "/api/v1/kpi/summary",
                get(event_monitor_handler::get_kpi_summary),
            )
            .route(
                "/api/v1/slo/status",
                get(event_monitor_handler::get_slo_status),
            )
            .route(
                "/api/v1/slo/:flow_id/burn-rate",
                get(event_monitor_handler::get_slo_burn_rate),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "event_monitor",
                "read",
            )));

        // write -> event_monitor/write (sys_operator+)
        let write_routes = Router::new()
            .route("/api/v1/flows", post(event_monitor_handler::create_flow))
            .route("/api/v1/flows/:id", put(event_monitor_handler::update_flow))
            .route(
                "/api/v1/replay/preview",
                post(event_monitor_handler::preview_replay),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "event_monitor",
                "write",
            )));

        // admin -> event_monitor/admin (sys_admin only)
        let admin_routes = Router::new()
            .route(
                "/api/v1/flows/:id",
                delete(event_monitor_handler::delete_flow),
            )
            .route(
                "/api/v1/replay/execute",
                post(event_monitor_handler::execute_replay),
            )
            .route_layer(axum::middleware::from_fn(require_permission(
                "event_monitor",
                "admin",
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
            .route("/api/v1/events", get(event_monitor_handler::list_events))
            .route(
                "/api/v1/events/trace/:correlation_id",
                get(event_monitor_handler::trace_by_correlation),
            )
            .route("/api/v1/flows", get(event_monitor_handler::list_flows))
            .route("/api/v1/flows/:id", get(event_monitor_handler::get_flow))
            .route("/api/v1/flows", post(event_monitor_handler::create_flow))
            .route("/api/v1/flows/:id", put(event_monitor_handler::update_flow))
            .route(
                "/api/v1/flows/:id",
                delete(event_monitor_handler::delete_flow),
            )
            .route(
                "/api/v1/flows/:id/instances",
                get(event_monitor_handler::get_flow_instances),
            )
            .route(
                "/api/v1/flows/:id/instances/:instance_id",
                get(event_monitor_handler::get_flow_instance),
            )
            .route(
                "/api/v1/flows/:id/kpi",
                get(event_monitor_handler::get_flow_kpi),
            )
            .route(
                "/api/v1/kpi/summary",
                get(event_monitor_handler::get_kpi_summary),
            )
            .route(
                "/api/v1/slo/status",
                get(event_monitor_handler::get_slo_status),
            )
            .route(
                "/api/v1/slo/:flow_id/burn-rate",
                get(event_monitor_handler::get_slo_burn_rate),
            )
            .route(
                "/api/v1/replay/preview",
                post(event_monitor_handler::preview_replay),
            )
            .route(
                "/api/v1/replay/execute",
                post(event_monitor_handler::execute_replay),
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
