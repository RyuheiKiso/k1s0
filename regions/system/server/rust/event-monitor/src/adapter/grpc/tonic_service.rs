use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};
use crate::proto::k1s0::system::event_monitor::v1::{
    event_monitor_service_server::EventMonitorService,
    CreateFlowRequest as ProtoCreateFlowRequest, CreateFlowResponse as ProtoCreateFlowResponse,
    DeleteFlowRequest as ProtoDeleteFlowRequest, DeleteFlowResponse as ProtoDeleteFlowResponse,
    EventRecord as ProtoEventRecord, ExecuteReplayRequest as ProtoExecuteReplayRequest,
    ExecuteReplayResponse as ProtoExecuteReplayResponse, FlowDefinition as ProtoFlowDefinition,
    FlowKpi as ProtoFlowKpi, FlowKpiSummary as ProtoFlowKpiSummary,
    FlowSlo as ProtoFlowSlo, FlowStep as ProtoFlowStep, FlowSummary as ProtoFlowSummary,
    GetFlowKpiRequest as ProtoGetFlowKpiRequest, GetFlowKpiResponse as ProtoGetFlowKpiResponse,
    GetFlowRequest as ProtoGetFlowRequest, GetFlowResponse as ProtoGetFlowResponse,
    GetKpiSummaryRequest as ProtoGetKpiSummaryRequest,
    GetKpiSummaryResponse as ProtoGetKpiSummaryResponse,
    GetSloBurnRateRequest as ProtoGetSloBurnRateRequest,
    GetSloBurnRateResponse as ProtoGetSloBurnRateResponse,
    GetSloStatusRequest as ProtoGetSloStatusRequest,
    GetSloStatusResponse as ProtoGetSloStatusResponse,
    ListEventsRequest as ProtoListEventsRequest, ListEventsResponse as ProtoListEventsResponse,
    ListFlowsRequest as ProtoListFlowsRequest, ListFlowsResponse as ProtoListFlowsResponse,
    PreviewReplayRequest as ProtoPreviewReplayRequest,
    PreviewReplayResponse as ProtoPreviewReplayResponse,
    TraceByCorrelationRequest as ProtoTraceByCorrelationRequest,
    TraceByCorrelationResponse as ProtoTraceByCorrelationResponse,
    UpdateFlowRequest as ProtoUpdateFlowRequest, UpdateFlowResponse as ProtoUpdateFlowResponse,
    BottleneckStep as ProtoBottleneckStep, BurnRateWindow as ProtoBurnRateWindow,
    ReplayFlowPreview as ProtoReplayFlowPreview, SloFlowStatus as ProtoSloFlowStatus,
    SloStatus as ProtoSloStatus, TraceEvent as ProtoTraceEvent,
};

use super::event_monitor_grpc::{EventMonitorGrpcService, GrpcError};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

fn to_proto_timestamp(dt: chrono::DateTime<chrono::Utc>) -> Option<ProtoTimestamp> {
    Some(ProtoTimestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

fn to_proto_flow(flow: crate::domain::entity::flow_definition::FlowDefinition) -> ProtoFlowDefinition {
    ProtoFlowDefinition {
        id: flow.id.to_string(),
        name: flow.name,
        description: flow.description,
        domain: flow.domain,
        steps: flow
            .steps
            .into_iter()
            .map(|s| ProtoFlowStep {
                event_type: s.event_type,
                source: s.source,
                timeout_seconds: s.timeout_seconds,
                description: s.description,
            })
            .collect(),
        slo: Some(ProtoFlowSlo {
            target_completion_seconds: flow.slo.target_completion_seconds,
            target_success_rate: flow.slo.target_success_rate,
            alert_on_violation: flow.slo.alert_on_violation,
        }),
        enabled: flow.enabled,
        created_at: to_proto_timestamp(flow.created_at),
        updated_at: to_proto_timestamp(flow.updated_at),
    }
}

fn to_proto_event(event: crate::domain::entity::event_record::EventRecord) -> ProtoEventRecord {
    ProtoEventRecord {
        id: event.id.to_string(),
        correlation_id: event.correlation_id,
        event_type: event.event_type,
        source: event.source,
        domain: event.domain,
        trace_id: event.trace_id,
        timestamp: to_proto_timestamp(event.timestamp),
        flow_id: event.flow_id.map(|id| id.to_string()),
        flow_step_index: event.flow_step_index,
        status: event.status,
    }
}

pub struct EventMonitorServiceTonic {
    inner: Arc<EventMonitorGrpcService>,
}

impl EventMonitorServiceTonic {
    pub fn new(inner: Arc<EventMonitorGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl EventMonitorService for EventMonitorServiceTonic {
    async fn list_events(
        &self,
        request: Request<ProtoListEventsRequest>,
    ) -> Result<Response<ProtoListEventsResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner
            .pagination
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, 20));

        let (events, total_count, has_next) = self
            .inner
            .list_events(page, page_size, inner.domain, inner.event_type, inner.source, inner.status)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoListEventsResponse {
            events: events.into_iter().map(to_proto_event).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: total_count.min(i32::MAX as u64) as i32,
                page,
                page_size,
                has_next,
            }),
        }))
    }

    async fn trace_by_correlation(
        &self,
        request: Request<ProtoTraceByCorrelationRequest>,
    ) -> Result<Response<ProtoTraceByCorrelationResponse>, Status> {
        let inner = request.into_inner();
        let output = self
            .inner
            .trace_by_correlation(&inner.correlation_id)
            .await
            .map_err(Into::<Status>::into)?;

        let flow = output.flow_instance.as_ref().map(|inst| {
            let elapsed = (chrono::Utc::now() - inst.started_at).num_seconds();
            ProtoFlowSummary {
                id: inst.flow_id.to_string(),
                name: output.flow_name.clone().unwrap_or_default(),
                status: inst.status.as_str().to_string(),
                started_at: to_proto_timestamp(inst.started_at),
                elapsed_seconds: elapsed,
            }
        });

        let mut prev_ts: Option<chrono::DateTime<chrono::Utc>> = None;
        let events: Vec<ProtoTraceEvent> = output
            .events
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let duration = if let Some(prev) = prev_ts {
                    (e.timestamp - prev).num_milliseconds()
                } else {
                    0
                };
                prev_ts = Some(e.timestamp);
                ProtoTraceEvent {
                    id: e.id.to_string(),
                    event_type: e.event_type.clone(),
                    source: e.source.clone(),
                    timestamp: to_proto_timestamp(e.timestamp),
                    step_index: e.flow_step_index.unwrap_or(i as i32),
                    status: e.status.clone(),
                    duration_from_previous_ms: duration,
                }
            })
            .collect();

        Ok(Response::new(ProtoTraceByCorrelationResponse {
            correlation_id: output.correlation_id,
            flow,
            events,
            pending_steps: vec![],
        }))
    }

    async fn list_flows(
        &self,
        request: Request<ProtoListFlowsRequest>,
    ) -> Result<Response<ProtoListFlowsResponse>, Status> {
        let inner = request.into_inner();
        let (page, page_size) = inner
            .pagination
            .map(|p| (p.page, p.page_size))
            .unwrap_or((1, 20));

        let (flows, total_count, has_next) = self
            .inner
            .list_flows(page, page_size, inner.domain)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoListFlowsResponse {
            flows: flows.into_iter().map(to_proto_flow).collect(),
            pagination: Some(ProtoPaginationResult {
                total_count: total_count.min(i32::MAX as u64) as i32,
                page,
                page_size,
                has_next,
            }),
        }))
    }

    async fn get_flow(
        &self,
        request: Request<ProtoGetFlowRequest>,
    ) -> Result<Response<ProtoGetFlowResponse>, Status> {
        let inner = request.into_inner();
        let flow = self
            .inner
            .get_flow(&inner.id)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetFlowResponse {
            flow: Some(to_proto_flow(flow)),
        }))
    }

    async fn create_flow(
        &self,
        request: Request<ProtoCreateFlowRequest>,
    ) -> Result<Response<ProtoCreateFlowResponse>, Status> {
        let inner = request.into_inner();
        let input = crate::usecase::create_flow::CreateFlowInput {
            name: inner.name,
            description: inner.description,
            domain: inner.domain,
            steps: inner
                .steps
                .into_iter()
                .map(|s| crate::domain::entity::flow_definition::FlowStep {
                    event_type: s.event_type,
                    source: s.source,
                    timeout_seconds: s.timeout_seconds,
                    description: s.description,
                })
                .collect(),
            slo: inner
                .slo
                .map(|s| crate::domain::entity::flow_definition::FlowSlo {
                    target_completion_seconds: s.target_completion_seconds,
                    target_success_rate: s.target_success_rate,
                    alert_on_violation: s.alert_on_violation,
                })
                .unwrap_or(crate::domain::entity::flow_definition::FlowSlo {
                    target_completion_seconds: 300,
                    target_success_rate: 0.995,
                    alert_on_violation: true,
                }),
        };

        let flow = self
            .inner
            .create_flow(input)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCreateFlowResponse {
            flow: Some(to_proto_flow(flow)),
        }))
    }

    async fn update_flow(
        &self,
        request: Request<ProtoUpdateFlowRequest>,
    ) -> Result<Response<ProtoUpdateFlowResponse>, Status> {
        let inner = request.into_inner();
        let id = uuid::Uuid::parse_str(&inner.id)
            .map_err(|_| Status::invalid_argument(format!("invalid flow id: {}", inner.id)))?;

        let input = crate::usecase::update_flow::UpdateFlowInput {
            id,
            description: inner.description,
            steps: if inner.steps.is_empty() {
                None
            } else {
                Some(
                    inner
                        .steps
                        .into_iter()
                        .map(|s| crate::domain::entity::flow_definition::FlowStep {
                            event_type: s.event_type,
                            source: s.source,
                            timeout_seconds: s.timeout_seconds,
                            description: s.description,
                        })
                        .collect(),
                )
            },
            slo: inner
                .slo
                .map(|s| crate::domain::entity::flow_definition::FlowSlo {
                    target_completion_seconds: s.target_completion_seconds,
                    target_success_rate: s.target_success_rate,
                    alert_on_violation: s.alert_on_violation,
                }),
            enabled: inner.enabled,
        };

        let flow = self
            .inner
            .update_flow(input)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoUpdateFlowResponse {
            flow: Some(to_proto_flow(flow)),
        }))
    }

    async fn delete_flow(
        &self,
        request: Request<ProtoDeleteFlowRequest>,
    ) -> Result<Response<ProtoDeleteFlowResponse>, Status> {
        let inner = request.into_inner();
        self.inner
            .delete_flow(&inner.id)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoDeleteFlowResponse {
            success: true,
            message: format!("flow {} deleted", inner.id),
        }))
    }

    async fn get_flow_kpi(
        &self,
        request: Request<ProtoGetFlowKpiRequest>,
    ) -> Result<Response<ProtoGetFlowKpiResponse>, Status> {
        let inner = request.into_inner();
        let output = self
            .inner
            .get_flow_kpi(&inner.flow_id, inner.period)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetFlowKpiResponse {
            flow_id: output.flow_id.to_string(),
            flow_name: output.flow_name,
            period: output.period,
            kpi: Some(ProtoFlowKpi {
                total_started: output.kpi.total_started,
                total_completed: output.kpi.total_completed,
                total_failed: output.kpi.total_failed,
                total_in_progress: output.kpi.total_in_progress,
                completion_rate: output.kpi.completion_rate,
                avg_duration_seconds: output.kpi.avg_duration_seconds,
                p50_duration_seconds: output.kpi.p50_duration_seconds,
                p95_duration_seconds: output.kpi.p95_duration_seconds,
                p99_duration_seconds: output.kpi.p99_duration_seconds,
                bottleneck_step: output.kpi.bottleneck_step.map(|b| ProtoBottleneckStep {
                    event_type: b.event_type,
                    step_index: b.step_index,
                    avg_duration_seconds: b.avg_duration_seconds,
                    timeout_rate: b.timeout_rate,
                }),
            }),
            slo_status: Some(ProtoSloStatus {
                target_completion_seconds: output.slo_status.target_completion_seconds,
                target_success_rate: output.slo_status.target_success_rate,
                current_success_rate: output.slo_status.current_success_rate,
                is_violated: output.slo_status.is_violated,
                burn_rate: output.slo_status.burn_rate,
                estimated_budget_exhaustion_hours: output.slo_status.estimated_budget_exhaustion_hours,
            }),
        }))
    }

    async fn get_kpi_summary(
        &self,
        request: Request<ProtoGetKpiSummaryRequest>,
    ) -> Result<Response<ProtoGetKpiSummaryResponse>, Status> {
        let inner = request.into_inner();
        let output = self
            .inner
            .get_kpi_summary(inner.period)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetKpiSummaryResponse {
            period: output.period,
            flows: output
                .flows
                .into_iter()
                .map(|f| ProtoFlowKpiSummary {
                    flow_id: f.flow_id,
                    flow_name: f.flow_name,
                    domain: f.domain,
                    total_started: f.total_started,
                    completion_rate: f.completion_rate,
                    avg_duration_seconds: f.avg_duration_seconds,
                    slo_violated: f.slo_violated,
                })
                .collect(),
            total_flows: output.total_flows,
            flows_with_slo_violation: output.flows_with_slo_violation,
            overall_completion_rate: output.overall_completion_rate,
        }))
    }

    async fn get_slo_status(
        &self,
        _request: Request<ProtoGetSloStatusRequest>,
    ) -> Result<Response<ProtoGetSloStatusResponse>, Status> {
        let items = self
            .inner
            .get_slo_status()
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetSloStatusResponse {
            flows: items
                .into_iter()
                .map(|f| ProtoSloFlowStatus {
                    flow_id: f.flow_id,
                    flow_name: f.flow_name,
                    is_violated: f.is_violated,
                    burn_rate: f.burn_rate,
                    error_budget_remaining: f.error_budget_remaining,
                })
                .collect(),
        }))
    }

    async fn get_slo_burn_rate(
        &self,
        request: Request<ProtoGetSloBurnRateRequest>,
    ) -> Result<Response<ProtoGetSloBurnRateResponse>, Status> {
        let inner = request.into_inner();
        let output = self
            .inner
            .get_slo_burn_rate(&inner.flow_id)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetSloBurnRateResponse {
            flow_id: output.flow_id,
            flow_name: output.flow_name,
            windows: output
                .windows
                .into_iter()
                .map(|w| ProtoBurnRateWindow {
                    window: w.window,
                    burn_rate: w.burn_rate,
                    error_budget_remaining: w.error_budget_remaining,
                })
                .collect(),
            alert_status: output.alert_status,
            alert_fired_at: output.alert_fired_at.and_then(|t| to_proto_timestamp(t)),
        }))
    }

    async fn preview_replay(
        &self,
        request: Request<ProtoPreviewReplayRequest>,
    ) -> Result<Response<ProtoPreviewReplayResponse>, Status> {
        let inner = request.into_inner();
        let input = crate::usecase::preview_replay::PreviewReplayInput {
            correlation_ids: inner.correlation_ids,
            from_step_index: inner.from_step_index,
            include_downstream: inner.include_downstream,
        };

        let output = self
            .inner
            .preview_replay(input)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoPreviewReplayResponse {
            total_events_to_replay: output.total_events_to_replay,
            affected_services: output.affected_services,
            affected_flows: output
                .affected_flows
                .into_iter()
                .map(|f| ProtoReplayFlowPreview {
                    correlation_id: f.correlation_id,
                    flow_name: f.flow_name,
                    replay_from_step: f.replay_from_step,
                    events_to_replay: f.events_to_replay,
                })
                .collect(),
            dlq_messages_found: output.dlq_messages_found,
            estimated_duration_seconds: output.estimated_duration_seconds,
        }))
    }

    async fn execute_replay(
        &self,
        request: Request<ProtoExecuteReplayRequest>,
    ) -> Result<Response<ProtoExecuteReplayResponse>, Status> {
        let inner = request.into_inner();
        let input = crate::usecase::execute_replay::ExecuteReplayInput {
            correlation_ids: inner.correlation_ids,
            from_step_index: inner.from_step_index,
            include_downstream: inner.include_downstream,
            dry_run: inner.dry_run,
        };

        let output = self
            .inner
            .execute_replay(input)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoExecuteReplayResponse {
            replay_id: output.replay_id,
            status: output.status,
            total_events: output.total_events,
            replayed_events: output.replayed_events,
            started_at: to_proto_timestamp(output.started_at),
        }))
    }
}
