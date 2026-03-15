use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::event_record::EventRecord;
use crate::domain::entity::flow_definition::FlowDefinition;
use crate::usecase::create_flow::{CreateFlowError, CreateFlowInput, CreateFlowUseCase};
use crate::usecase::delete_flow::{DeleteFlowError, DeleteFlowUseCase};
use crate::usecase::execute_replay::{ExecuteReplayInput, ExecuteReplayUseCase};
use crate::usecase::get_flow::{GetFlowError, GetFlowUseCase};
use crate::usecase::get_flow_kpi::{GetFlowKpiError, GetFlowKpiUseCase};
use crate::usecase::get_kpi_summary::GetKpiSummaryUseCase;
use crate::usecase::get_slo_burn_rate::{GetSloBurnRateError, GetSloBurnRateUseCase};
use crate::usecase::get_slo_status::GetSloStatusUseCase;
use crate::usecase::list_events::{ListEventsInput, ListEventsUseCase};
use crate::usecase::list_flows::{ListFlowsInput, ListFlowsUseCase};
use crate::usecase::preview_replay::{
    PreviewReplayError, PreviewReplayInput, PreviewReplayUseCase,
};
use crate::usecase::trace_by_correlation::{TraceByCorrelationError, TraceByCorrelationUseCase};
use crate::usecase::update_flow::{UpdateFlowError, UpdateFlowInput, UpdateFlowUseCase};

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub struct EventMonitorGrpcService {
    list_events_uc: Arc<ListEventsUseCase>,
    trace_by_correlation_uc: Arc<TraceByCorrelationUseCase>,
    create_flow_uc: Arc<CreateFlowUseCase>,
    get_flow_uc: Arc<GetFlowUseCase>,
    update_flow_uc: Arc<UpdateFlowUseCase>,
    delete_flow_uc: Arc<DeleteFlowUseCase>,
    list_flows_uc: Arc<ListFlowsUseCase>,
    get_flow_kpi_uc: Arc<GetFlowKpiUseCase>,
    get_kpi_summary_uc: Arc<GetKpiSummaryUseCase>,
    get_slo_status_uc: Arc<GetSloStatusUseCase>,
    get_slo_burn_rate_uc: Arc<GetSloBurnRateUseCase>,
    preview_replay_uc: Arc<PreviewReplayUseCase>,
    execute_replay_uc: Arc<ExecuteReplayUseCase>,
}

impl EventMonitorGrpcService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        list_events_uc: Arc<ListEventsUseCase>,
        trace_by_correlation_uc: Arc<TraceByCorrelationUseCase>,
        create_flow_uc: Arc<CreateFlowUseCase>,
        get_flow_uc: Arc<GetFlowUseCase>,
        update_flow_uc: Arc<UpdateFlowUseCase>,
        delete_flow_uc: Arc<DeleteFlowUseCase>,
        list_flows_uc: Arc<ListFlowsUseCase>,
        get_flow_kpi_uc: Arc<GetFlowKpiUseCase>,
        get_kpi_summary_uc: Arc<GetKpiSummaryUseCase>,
        get_slo_status_uc: Arc<GetSloStatusUseCase>,
        get_slo_burn_rate_uc: Arc<GetSloBurnRateUseCase>,
        preview_replay_uc: Arc<PreviewReplayUseCase>,
        execute_replay_uc: Arc<ExecuteReplayUseCase>,
    ) -> Self {
        Self {
            list_events_uc,
            trace_by_correlation_uc,
            create_flow_uc,
            get_flow_uc,
            update_flow_uc,
            delete_flow_uc,
            list_flows_uc,
            get_flow_kpi_uc,
            get_kpi_summary_uc,
            get_slo_status_uc,
            get_slo_burn_rate_uc,
            preview_replay_uc,
            execute_replay_uc,
        }
    }

    pub async fn list_events(
        &self,
        page: i32,
        page_size: i32,
        domain: Option<String>,
        event_type: Option<String>,
        source: Option<String>,
        status: Option<String>,
    ) -> Result<(Vec<EventRecord>, u64, bool), GrpcError> {
        let page = if page <= 0 { 1 } else { page as u32 };
        let page_size = if page_size <= 0 { 20 } else { page_size as u32 };

        let output = self
            .list_events_uc
            .execute(&ListEventsInput {
                page,
                page_size,
                domain,
                event_type,
                source,
                from: None,
                to: None,
                status,
            })
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;

        Ok((output.events, output.total_count, output.has_next))
    }

    pub async fn trace_by_correlation(
        &self,
        correlation_id: &str,
    ) -> Result<crate::usecase::trace_by_correlation::TraceOutput, GrpcError> {
        self.trace_by_correlation_uc
            .execute(correlation_id)
            .await
            .map_err(|e| match e {
                TraceByCorrelationError::NotFound(id) => GrpcError::NotFound(id),
                TraceByCorrelationError::Internal(msg) => GrpcError::Internal(msg),
            })
    }

    pub async fn create_flow(&self, input: CreateFlowInput) -> Result<FlowDefinition, GrpcError> {
        self.create_flow_uc
            .execute(&input)
            .await
            .map_err(|e| match e {
                CreateFlowError::AlreadyExists(name) => GrpcError::AlreadyExists(name),
                CreateFlowError::Validation(msg) => GrpcError::InvalidArgument(msg),
                CreateFlowError::Internal(msg) => GrpcError::Internal(msg),
            })
    }

    pub async fn get_flow(&self, id: &str) -> Result<FlowDefinition, GrpcError> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid flow id: {}", id)))?;
        self.get_flow_uc.execute(&uuid).await.map_err(|e| match e {
            GetFlowError::NotFound(id) => GrpcError::NotFound(id),
            GetFlowError::Internal(msg) => GrpcError::Internal(msg),
        })
    }

    pub async fn update_flow(&self, input: UpdateFlowInput) -> Result<FlowDefinition, GrpcError> {
        self.update_flow_uc
            .execute(&input)
            .await
            .map_err(|e| match e {
                UpdateFlowError::NotFound(id) => GrpcError::NotFound(id),
                UpdateFlowError::Internal(msg) => GrpcError::Internal(msg),
            })
    }

    pub async fn delete_flow(&self, id: &str) -> Result<(), GrpcError> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid flow id: {}", id)))?;
        self.delete_flow_uc
            .execute(&uuid)
            .await
            .map_err(|e| match e {
                DeleteFlowError::NotFound(id) => GrpcError::NotFound(id),
                DeleteFlowError::Internal(msg) => GrpcError::Internal(msg),
            })
    }

    pub async fn list_flows(
        &self,
        page: i32,
        page_size: i32,
        domain: Option<String>,
    ) -> Result<(Vec<FlowDefinition>, u64, bool), GrpcError> {
        let page = if page <= 0 { 1 } else { page as u32 };
        let page_size = if page_size <= 0 { 20 } else { page_size as u32 };

        let output = self
            .list_flows_uc
            .execute(&ListFlowsInput {
                page,
                page_size,
                domain,
            })
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;

        Ok((output.flows, output.total_count, output.has_next))
    }

    pub async fn get_flow_kpi(
        &self,
        flow_id: &str,
        period: Option<String>,
    ) -> Result<crate::usecase::get_flow_kpi::GetFlowKpiOutput, GrpcError> {
        let uuid = Uuid::parse_str(flow_id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid flow id: {}", flow_id)))?;
        let period = period.as_deref().unwrap_or("1h");
        self.get_flow_kpi_uc
            .execute(&uuid, period)
            .await
            .map_err(|e| match e {
                GetFlowKpiError::NotFound(id) => GrpcError::NotFound(id),
                GetFlowKpiError::Internal(msg) => GrpcError::Internal(msg),
            })
    }

    pub async fn get_kpi_summary(
        &self,
        period: Option<String>,
    ) -> Result<crate::usecase::get_kpi_summary::GetKpiSummaryOutput, GrpcError> {
        let period = period.as_deref().unwrap_or("24h");
        self.get_kpi_summary_uc
            .execute(period)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))
    }

    pub async fn get_slo_status(
        &self,
    ) -> Result<Vec<crate::usecase::get_slo_status::SloFlowStatusItem>, GrpcError> {
        self.get_slo_status_uc
            .execute()
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))
    }

    pub async fn get_slo_burn_rate(
        &self,
        flow_id: &str,
    ) -> Result<crate::usecase::get_slo_burn_rate::GetSloBurnRateOutput, GrpcError> {
        let uuid = Uuid::parse_str(flow_id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid flow id: {}", flow_id)))?;
        self.get_slo_burn_rate_uc
            .execute(&uuid)
            .await
            .map_err(|e| match e {
                GetSloBurnRateError::NotFound(id) => GrpcError::NotFound(id),
                GetSloBurnRateError::Internal(msg) => GrpcError::Internal(msg),
            })
    }

    pub async fn preview_replay(
        &self,
        input: PreviewReplayInput,
    ) -> Result<crate::usecase::preview_replay::PreviewReplayOutput, GrpcError> {
        self.preview_replay_uc
            .execute(&input)
            .await
            .map_err(|e| match e {
                PreviewReplayError::NotFound(id) => GrpcError::NotFound(id),
                PreviewReplayError::Internal(msg) => GrpcError::Internal(msg),
            })
    }

    pub async fn execute_replay(
        &self,
        input: ExecuteReplayInput,
    ) -> Result<crate::usecase::execute_replay::ExecuteReplayOutput, GrpcError> {
        self.execute_replay_uc
            .execute(&input)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))
    }
}
