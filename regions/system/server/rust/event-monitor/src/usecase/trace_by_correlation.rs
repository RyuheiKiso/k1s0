use std::sync::Arc;

use crate::domain::entity::event_record::EventRecord;
use crate::domain::entity::flow_definition::FlowStep;
use crate::domain::entity::flow_instance::FlowInstance;
use crate::domain::repository::{EventRecordRepository, FlowDefinitionRepository, FlowInstanceRepository};

#[derive(Debug, thiserror::Error)]
pub enum TraceByCorrelationError {
    #[error("correlation not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone)]
pub struct PendingStepInfo {
    pub event_type: String,
    pub source: String,
    pub step_index: i32,
    pub timeout_seconds: i32,
    pub waiting_since_seconds: i64,
}

#[derive(Debug)]
pub struct TraceOutput {
    pub correlation_id: String,
    pub events: Vec<EventRecord>,
    pub flow_instance: Option<FlowInstance>,
    pub flow_name: Option<String>,
    pub pending_steps: Vec<PendingStepInfo>,
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

    fn compute_pending_steps(
        instance: &FlowInstance,
        steps: Option<&Vec<FlowStep>>,
    ) -> Vec<PendingStepInfo> {
        let steps = match steps {
            Some(s) => s,
            None => return vec![],
        };

        // Only compute pending steps for in-progress flows
        if instance.status != crate::domain::entity::flow_instance::FlowInstanceStatus::InProgress {
            return vec![];
        }

        let current_idx = instance.current_step_index as usize;
        let elapsed_since_start = (chrono::Utc::now() - instance.started_at).num_seconds();

        steps
            .iter()
            .enumerate()
            .skip(current_idx + 1)
            .map(|(i, step)| PendingStepInfo {
                event_type: step.event_type.clone(),
                source: step.source_filter.clone().unwrap_or_else(|| step.source.clone()),
                step_index: i as i32,
                timeout_seconds: step.timeout_seconds,
                waiting_since_seconds: elapsed_since_start,
            })
            .collect()
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

        let (flow_name, pending_steps) = if let Some(ref instance) = flow_instance {
            let flow = self
                .flow_def_repo
                .find_by_id(&instance.flow_id)
                .await
                .map_err(|e| TraceByCorrelationError::Internal(e.to_string()))?;
            let name = flow.as_ref().map(|f| f.name.clone());
            let pending = Self::compute_pending_steps(instance, flow.as_ref().map(|f| &f.steps));
            (name, pending)
        } else {
            (None, vec![])
        };

        Ok(TraceOutput {
            correlation_id: correlation_id.to_string(),
            events,
            flow_instance,
            flow_name,
            pending_steps,
        })
    }
}
