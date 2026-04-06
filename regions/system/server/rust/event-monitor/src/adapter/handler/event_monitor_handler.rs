use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::usecase::create_flow::CreateFlowInput;
use crate::usecase::get_flow_instances::GetFlowInstancesInput;
use crate::usecase::list_events::ListEventsInput;
use crate::usecase::list_flows::ListFlowsInput;
use crate::usecase::update_flow::UpdateFlowInput;

// --- List Events ---

#[derive(Debug, Deserialize)]
pub struct ListEventsParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub domain: Option<String>,
    pub event_type: Option<String>,
    pub source: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub status: Option<String>,
}

pub async fn list_events(
    State(state): State<AppState>,
    Query(params): Query<ListEventsParams>,
) -> impl IntoResponse {
    let from = params
        .from
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let to = params
        .to
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let input = ListEventsInput {
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
        domain: params.domain,
        event_type: params.event_type,
        source: params.source,
        from,
        to,
        status: params.status,
    };

    match state.list_events_uc.execute(&input).await {
        Ok(output) => {
            let events: Vec<EventResponse> =
                output.events.into_iter().map(EventResponse::from).collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "events": events,
                    "pagination": {
                        "total_count": output.total_count,
                        "page": output.page,
                        "page_size": output.page_size,
                        "has_next": output.has_next
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Trace by Correlation ---

pub async fn trace_by_correlation(
    State(state): State<AppState>,
    Path(correlation_id): Path<String>,
) -> impl IntoResponse {
    match state.trace_by_correlation_uc.execute(&correlation_id).await {
        Ok(output) => {
            let mut events_json: Vec<serde_json::Value> = Vec::new();
            let mut prev_ts: Option<chrono::DateTime<chrono::Utc>> = None;

            for (i, event) in output.events.iter().enumerate() {
                let duration_from_prev = if let Some(prev) = prev_ts {
                    (event.timestamp - prev).num_milliseconds()
                } else {
                    0
                };
                prev_ts = Some(event.timestamp);

                events_json.push(serde_json::json!({
                    "id": event.id.to_string(),
                    "event_type": event.event_type,
                    "source": event.source,
                    "timestamp": event.timestamp.to_rfc3339(),
                    "step_index": event.flow_step_index.unwrap_or(i as i32),
                    "status": event.status,
                    "duration_from_previous_ms": duration_from_prev
                }));
            }

            let flow_json = if let Some(ref instance) = output.flow_instance {
                let elapsed = (chrono::Utc::now() - instance.started_at).num_seconds();
                serde_json::json!({
                    "id": instance.flow_id.to_string(),
                    "name": output.flow_name.clone().unwrap_or_default(),
                    "status": instance.status.as_str(),
                    "started_at": instance.started_at.to_rfc3339(),
                    "elapsed_seconds": elapsed
                })
            } else {
                serde_json::Value::Null
            };

            let pending_steps_json: Vec<serde_json::Value> = output
                .pending_steps
                .iter()
                .map(|ps| {
                    serde_json::json!({
                        "event_type": ps.event_type,
                        "source": ps.source,
                        "step_index": ps.step_index,
                        "timeout_seconds": ps.timeout_seconds,
                        "waiting_since_seconds": ps.waiting_since_seconds
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "correlation_id": output.correlation_id,
                    "flow": flow_json,
                    "events": events_json,
                    "pending_steps": pending_steps_json
                })),
            )
                .into_response()
        }
        Err(crate::usecase::trace_by_correlation::TraceByCorrelationError::NotFound(id)) => {
            let err = ErrorResponse::new(
                "SYS_EVMON_CORRELATION_NOT_FOUND",
                &format!("no events found for correlation_id: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Flows CRUD ---

#[derive(Debug, Deserialize)]
pub struct ListFlowsParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub domain: Option<String>,
}

pub async fn list_flows(
    State(state): State<AppState>,
    Query(params): Query<ListFlowsParams>,
) -> impl IntoResponse {
    let input = ListFlowsInput {
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
        domain: params.domain,
    };

    match state.list_flows_uc.execute(&input).await {
        Ok(output) => {
            let flows: Vec<FlowResponse> =
                output.flows.into_iter().map(FlowResponse::from).collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "flows": flows,
                    "pagination": {
                        "total_count": output.total_count,
                        "page": output.page,
                        "page_size": output.page_size,
                        "has_next": output.has_next
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn get_flow(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.get_flow_uc.execute(&id).await {
        Ok(flow) => {
            let resp = FlowResponse::from(flow);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(crate::usecase::get_flow::GetFlowError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_EVMON_FLOW_NOT_FOUND",
                &format!("flow not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateFlowRequest {
    pub name: String,
    pub description: String,
    pub domain: String,
    pub steps: Vec<crate::domain::entity::flow_definition::FlowStep>,
    pub slo: crate::domain::entity::flow_definition::FlowSlo,
}

pub async fn create_flow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateFlowRequest>,
) -> impl IntoResponse {
    // HTTP ヘッダーから tenant_id を取得する。未設定の場合は "system" を使用する。
    let tenant_id = headers
        .get("x-tenant-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("system")
        .to_string();

    let input = CreateFlowInput {
        tenant_id,
        name: req.name,
        description: req.description,
        domain: req.domain,
        steps: req.steps,
        slo: req.slo,
    };

    match state.create_flow_uc.execute(&input).await {
        Ok(flow) => {
            let resp = FlowResponse::from(flow);
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(crate::usecase::create_flow::CreateFlowError::AlreadyExists(name)) => {
            let err = ErrorResponse::new(
                "SYS_EVMON_ALREADY_EXISTS",
                &format!("flow already exists: {}", name),
            );
            (StatusCode::CONFLICT, Json(err)).into_response()
        }
        Err(crate::usecase::create_flow::CreateFlowError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_EVMON_VALIDATION_ERROR", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlowRequest {
    pub description: Option<String>,
    pub steps: Option<Vec<crate::domain::entity::flow_definition::FlowStep>>,
    pub slo: Option<crate::domain::entity::flow_definition::FlowSlo>,
    pub enabled: Option<bool>,
}

pub async fn update_flow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateFlowRequest>,
) -> impl IntoResponse {
    let input = UpdateFlowInput {
        id,
        description: req.description,
        steps: req.steps,
        slo: req.slo,
        enabled: req.enabled,
    };

    match state.update_flow_uc.execute(&input).await {
        Ok(flow) => {
            let resp = FlowResponse::from(flow);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(crate::usecase::update_flow::UpdateFlowError::NotFound(id)) => {
            let err = ErrorResponse::new(
                "SYS_EVMON_FLOW_NOT_FOUND",
                &format!("flow not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn delete_flow(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.delete_flow_uc.execute(&id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("flow {} deleted", id)
            })),
        )
            .into_response(),
        Err(crate::usecase::delete_flow::DeleteFlowError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_EVMON_FLOW_NOT_FOUND",
                &format!("flow not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Flow Instances ---

#[derive(Debug, Deserialize)]
pub struct FlowInstancesParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn get_flow_instances(
    State(state): State<AppState>,
    Path(flow_id): Path<Uuid>,
    Query(params): Query<FlowInstancesParams>,
) -> impl IntoResponse {
    let input = GetFlowInstancesInput {
        flow_id,
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
    };

    match state.get_flow_instances_uc.execute(&input).await {
        Ok(output) => {
            let instances: Vec<FlowInstanceResponse> = output
                .instances
                .into_iter()
                .map(FlowInstanceResponse::from)
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "instances": instances,
                    "pagination": {
                        "total_count": output.total_count,
                        "page": output.page,
                        "page_size": output.page_size,
                        "has_next": output.has_next
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn get_flow_instance(
    State(state): State<AppState>,
    Path((_flow_id, instance_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match state.get_flow_instance_uc.execute(&instance_id).await {
        Ok(instance) => {
            let resp = FlowInstanceResponse::from(instance);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(crate::usecase::get_flow_instance::GetFlowInstanceError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_EVMON_NOT_FOUND",
                &format!("instance not found: {}", instance_id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- KPI ---

#[derive(Debug, Deserialize)]
pub struct KpiParams {
    pub period: Option<String>,
}

pub async fn get_flow_kpi(
    State(state): State<AppState>,
    Path(flow_id): Path<Uuid>,
    Query(params): Query<KpiParams>,
) -> impl IntoResponse {
    let period = params.period.as_deref().unwrap_or("1h");

    match state.get_flow_kpi_uc.execute(&flow_id, period).await {
        Ok(output) => {
            let bottleneck = output
                .kpi
                .bottleneck_step
                .as_ref()
                .map(|b| {
                    serde_json::json!({
                        "event_type": b.event_type,
                        "step_index": b.step_index,
                        "avg_duration_seconds": b.avg_duration_seconds,
                        "timeout_rate": b.timeout_rate
                    })
                })
                .unwrap_or(serde_json::Value::Null);

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "flow_id": output.flow_id.to_string(),
                    "flow_name": output.flow_name,
                    "period": output.period,
                    "kpi": {
                        "total_started": output.kpi.total_started,
                        "total_completed": output.kpi.total_completed,
                        "total_failed": output.kpi.total_failed,
                        "total_in_progress": output.kpi.total_in_progress,
                        "completion_rate": output.kpi.completion_rate,
                        "avg_duration_seconds": output.kpi.avg_duration_seconds,
                        "p50_duration_seconds": output.kpi.p50_duration_seconds,
                        "p95_duration_seconds": output.kpi.p95_duration_seconds,
                        "p99_duration_seconds": output.kpi.p99_duration_seconds,
                        "bottleneck_step": bottleneck
                    },
                    "slo_status": {
                        "target_completion_seconds": output.slo_status.target_completion_seconds,
                        "target_success_rate": output.slo_status.target_success_rate,
                        "current_success_rate": output.slo_status.current_success_rate,
                        "is_violated": output.slo_status.is_violated,
                        "burn_rate": output.slo_status.burn_rate,
                        "estimated_budget_exhaustion_hours": output.slo_status.estimated_budget_exhaustion_hours
                    }
                })),
            )
                .into_response()
        }
        Err(crate::usecase::get_flow_kpi::GetFlowKpiError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_EVMON_FLOW_NOT_FOUND",
                &format!("flow not found: {}", flow_id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn get_kpi_summary(
    State(state): State<AppState>,
    Query(params): Query<KpiParams>,
) -> impl IntoResponse {
    let period = params.period.as_deref().unwrap_or("24h");

    match state.get_kpi_summary_uc.execute(period).await {
        Ok(output) => {
            let flows: Vec<serde_json::Value> = output
                .flows
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "flow_id": f.flow_id,
                        "flow_name": f.flow_name,
                        "domain": f.domain,
                        "total_started": f.total_started,
                        "completion_rate": f.completion_rate,
                        "avg_duration_seconds": f.avg_duration_seconds,
                        "slo_violated": f.slo_violated
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "period": output.period,
                    "flows": flows,
                    "summary": {
                        "total_flows": output.total_flows,
                        "flows_with_slo_violation": output.flows_with_slo_violation,
                        "overall_completion_rate": output.overall_completion_rate
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- SLO ---

pub async fn get_slo_status(State(state): State<AppState>) -> impl IntoResponse {
    match state.get_slo_status_uc.execute().await {
        Ok(items) => {
            let flows: Vec<serde_json::Value> = items
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "flow_id": f.flow_id,
                        "flow_name": f.flow_name,
                        "is_violated": f.is_violated,
                        "burn_rate": f.burn_rate,
                        "error_budget_remaining": f.error_budget_remaining
                    })
                })
                .collect();

            (StatusCode::OK, Json(serde_json::json!({ "flows": flows }))).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

pub async fn get_slo_burn_rate(
    State(state): State<AppState>,
    Path(flow_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.get_slo_burn_rate_uc.execute(&flow_id).await {
        Ok(output) => {
            let windows: Vec<serde_json::Value> = output
                .windows
                .iter()
                .map(|w| {
                    serde_json::json!({
                        "window": w.window,
                        "burn_rate": w.burn_rate,
                        "error_budget_remaining": w.error_budget_remaining
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "flow_id": output.flow_id,
                    "flow_name": output.flow_name,
                    "windows": windows,
                    "alert_status": output.alert_status,
                    "alert_fired_at": output.alert_fired_at.map(|t| t.to_rfc3339())
                })),
            )
                .into_response()
        }
        Err(crate::usecase::get_slo_burn_rate::GetSloBurnRateError::NotFound(_)) => {
            let err = ErrorResponse::new(
                "SYS_EVMON_FLOW_NOT_FOUND",
                &format!("flow not found: {}", flow_id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Replay ---

#[derive(Debug, Deserialize)]
pub struct PreviewReplayRequest {
    pub correlation_ids: Vec<String>,
    pub from_step_index: i32,
    pub include_downstream: bool,
}

pub async fn preview_replay(
    State(state): State<AppState>,
    Json(req): Json<PreviewReplayRequest>,
) -> impl IntoResponse {
    let input = crate::usecase::preview_replay::PreviewReplayInput {
        correlation_ids: req.correlation_ids,
        from_step_index: req.from_step_index,
        include_downstream: req.include_downstream,
    };

    match state.preview_replay_uc.execute(&input).await {
        Ok(output) => {
            let affected: Vec<serde_json::Value> = output
                .affected_flows
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "correlation_id": f.correlation_id,
                        "flow_name": f.flow_name,
                        "replay_from_step": f.replay_from_step,
                        "events_to_replay": f.events_to_replay
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "preview": {
                        "total_events_to_replay": output.total_events_to_replay,
                        "affected_services": output.affected_services,
                        "affected_flows": affected,
                        "dlq_messages_found": output.dlq_messages_found,
                        "estimated_duration_seconds": output.estimated_duration_seconds
                    }
                })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_INTERNAL_ERROR", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ExecuteReplayRequest {
    pub correlation_ids: Vec<String>,
    pub from_step_index: i32,
    pub include_downstream: bool,
    #[serde(default)]
    pub dry_run: bool,
}

pub async fn execute_replay(
    State(state): State<AppState>,
    Json(req): Json<ExecuteReplayRequest>,
) -> impl IntoResponse {
    let input = crate::usecase::execute_replay::ExecuteReplayInput {
        correlation_ids: req.correlation_ids,
        from_step_index: req.from_step_index,
        include_downstream: req.include_downstream,
        dry_run: req.dry_run,
    };

    match state.execute_replay_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "replay_id": output.replay_id,
                "status": output.status,
                "total_events": output.total_events,
                "replayed_events": output.replayed_events,
                "started_at": output.started_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(crate::usecase::execute_replay::ExecuteReplayError::ReplayInProgress(ids)) => {
            let err = ErrorResponse::new(
                "SYS_EVMON_REPLAY_IN_PROGRESS",
                &format!("replay already in progress for: {}", ids),
            );
            (StatusCode::CONFLICT, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_EVMON_REPLAY_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Response types ---

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: String,
    pub correlation_id: String,
    pub event_type: String,
    pub source: String,
    pub domain: String,
    pub trace_id: String,
    pub timestamp: String,
    pub flow_id: Option<String>,
    pub flow_step_index: Option<i32>,
    pub status: String,
}

impl From<crate::domain::entity::event_record::EventRecord> for EventResponse {
    fn from(e: crate::domain::entity::event_record::EventRecord) -> Self {
        Self {
            id: e.id.to_string(),
            correlation_id: e.correlation_id,
            event_type: e.event_type,
            source: e.source,
            domain: e.domain,
            trace_id: e.trace_id,
            timestamp: e.timestamp.to_rfc3339(),
            flow_id: e.flow_id.map(|id| id.to_string()),
            flow_step_index: e.flow_step_index,
            status: e.status,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FlowResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub steps: Vec<crate::domain::entity::flow_definition::FlowStep>,
    pub slo: crate::domain::entity::flow_definition::FlowSlo,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::entity::flow_definition::FlowDefinition> for FlowResponse {
    fn from(f: crate::domain::entity::flow_definition::FlowDefinition) -> Self {
        Self {
            id: f.id.to_string(),
            name: f.name,
            description: f.description,
            domain: f.domain,
            steps: f.steps,
            slo: f.slo,
            enabled: f.enabled,
            created_at: f.created_at.to_rfc3339(),
            updated_at: f.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FlowInstanceResponse {
    pub id: String,
    pub flow_id: String,
    pub correlation_id: String,
    pub status: String,
    pub current_step_index: i32,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<i64>,
}

impl From<crate::domain::entity::flow_instance::FlowInstance> for FlowInstanceResponse {
    fn from(i: crate::domain::entity::flow_instance::FlowInstance) -> Self {
        Self {
            id: i.id.to_string(),
            flow_id: i.flow_id.to_string(),
            correlation_id: i.correlation_id,
            status: i.status.as_str().to_string(),
            current_step_index: i.current_step_index,
            started_at: i.started_at.to_rfc3339(),
            completed_at: i.completed_at.map(|t| t.to_rfc3339()),
            duration_ms: i.duration_ms,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub field: String,
    pub message: String,
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
