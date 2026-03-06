use std::sync::Arc;

use crate::domain::entity::event_record::EventRecord;
use crate::domain::entity::flow_instance::FlowInstance;
use crate::domain::repository::{EventRecordRepository, FlowDefinitionRepository, FlowInstanceRepository};

#[derive(Debug, thiserror::Error)]
pub enum TraceByCorrelationError {
    #[error("correlation not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug)]
pub struct TraceOutput {
    pub correlation_id: String,
    pub events: Vec<EventRecord>,
    pub flow_instance: Option<FlowInstance>,
    pub flow_name: Option<String>,
}

pub struct TraceByCorrelationUseCase {
    event_repo: Arc<dyn EventRecordRepository>,
    flow_def_repo: Arc<dyn FlowDefinitionRepository>,
    flow_inst_repo: Arc<dyn FlowInstanceRepository>,
}

impl TraceByCorrelationUseCase {
    pub fn new(
        event_repo: Arc<dyn EventRecordRepository>,
        flow_def_repo: Arc<dyn FlowDefinitionRepository>,
        flow_inst_repo: Arc<dyn FlowInstanceRepository>,
    ) -> Self {
        Self {
            event_repo,
            flow_def_repo,
            flow_inst_repo,
        }
    }

    pub async fn execute(
        &self,
        correlation_id: &str,
    ) -> Result<TraceOutput, TraceByCorrelationError> {
        let events = self
            .event_repo
            .find_by_correlation_id(correlation_id.to_string())
            .await
            .map_err(|e| TraceByCorrelationError::Internal(e.to_string()))?;

        if events.is_empty() {
            return Err(TraceByCorrelationError::NotFound(
                correlation_id.to_string(),
            ));
        }

        let flow_instance = self
            .flow_inst_repo
            .find_by_correlation_id(correlation_id.to_string())
            .await
            .map_err(|e| TraceByCorrelationError::Internal(e.to_string()))?;

        let flow_name = if let Some(ref instance) = flow_instance {
            let flow = self
                .flow_def_repo
                .find_by_id(&instance.flow_id)
                .await
                .map_err(|e| TraceByCorrelationError::Internal(e.to_string()))?;
            flow.map(|f| f.name)
        } else {
            None
        };

        Ok(TraceOutput {
            correlation_id: correlation_id.to_string(),
            events,
            flow_instance,
            flow_name,
        })
    }
}
