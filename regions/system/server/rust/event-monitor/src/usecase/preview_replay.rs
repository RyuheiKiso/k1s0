use std::sync::Arc;

use crate::domain::repository::{EventRecordRepository, FlowDefinitionRepository};
use crate::infrastructure::dlq_client::{DlqManagerClient, ReplayRequest};

#[derive(Debug, Clone)]
pub struct PreviewReplayInput {
    pub correlation_ids: Vec<String>,
    pub from_step_index: i32,
    pub include_downstream: bool,
}

#[derive(Debug)]
pub struct ReplayFlowPreview {
    pub correlation_id: String,
    pub flow_name: String,
    pub replay_from_step: i32,
    pub events_to_replay: i32,
}

#[derive(Debug)]
pub struct PreviewReplayOutput {
    pub total_events_to_replay: i32,
    pub affected_services: Vec<String>,
    pub affected_flows: Vec<ReplayFlowPreview>,
    pub dlq_messages_found: i32,
    pub estimated_duration_seconds: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum PreviewReplayError {
    #[allow(dead_code)]
    #[error("correlation not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct PreviewReplayUseCase {
    event_repo: Arc<dyn EventRecordRepository>,
    flow_def_repo: Arc<dyn FlowDefinitionRepository>,
    dlq_client: Arc<dyn DlqManagerClient>,
}

impl PreviewReplayUseCase {
    pub fn new(
        event_repo: Arc<dyn EventRecordRepository>,
        flow_def_repo: Arc<dyn FlowDefinitionRepository>,
        dlq_client: Arc<dyn DlqManagerClient>,
    ) -> Self {
        Self {
            event_repo,
            flow_def_repo,
            dlq_client,
        }
    }

    pub async fn execute(
        &self,
        input: &PreviewReplayInput,
    ) -> Result<PreviewReplayOutput, PreviewReplayError> {
        let mut affected_flows = Vec::new();
        let mut affected_services = Vec::new();
        let mut total_events = 0i32;

        for corr_id in &input.correlation_ids {
            let events = self
                .event_repo
                .find_by_correlation_id(corr_id.clone())
                .await
                .map_err(|e| PreviewReplayError::Internal(e.to_string()))?;

            if events.is_empty() {
                continue;
            }

            let replay_events: Vec<_> = events
                .iter()
                .filter(|e| {
                    e.flow_step_index
                        .map(|idx| idx >= input.from_step_index)
                        .unwrap_or(false)
                })
                .collect();

            let events_count = replay_events.len() as i32;
            total_events += events_count;

            for e in &replay_events {
                if !affected_services.contains(&e.source) {
                    affected_services.push(e.source.clone());
                }
            }

            let flow_name = if let Some(flow_id) = events.first().and_then(|e| e.flow_id) {
                self.flow_def_repo
                    .find_by_id(&flow_id)
                    .await
                    .map_err(|e| PreviewReplayError::Internal(e.to_string()))?
                    .map(|f| f.name)
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                "unknown".to_string()
            };

            affected_flows.push(ReplayFlowPreview {
                correlation_id: corr_id.clone(),
                flow_name,
                replay_from_step: input.from_step_index,
                events_to_replay: events_count,
            });
        }

        let dlq_preview = self
            .dlq_client
            .preview_replay(&ReplayRequest {
                correlation_ids: input.correlation_ids.clone(),
                from_step_index: input.from_step_index,
                include_downstream: input.include_downstream,
                dry_run: true,
            })
            .await
            .map_err(|e| PreviewReplayError::Internal(e.to_string()))?;

        Ok(PreviewReplayOutput {
            total_events_to_replay: total_events,
            affected_services,
            affected_flows,
            dlq_messages_found: dlq_preview.dlq_messages_found,
            estimated_duration_seconds: dlq_preview.estimated_duration_seconds,
        })
    }
}
