use std::sync::Arc;

use crate::infrastructure::dlq_client::{DlqManagerClient, ReplayRequest};

#[derive(Debug, Clone)]
pub struct ExecuteReplayInput {
    pub correlation_ids: Vec<String>,
    pub from_step_index: i32,
    pub include_downstream: bool,
    pub dry_run: bool,
}

#[derive(Debug)]
pub struct ExecuteReplayOutput {
    pub replay_id: String,
    pub status: String,
    pub total_events: i32,
    pub replayed_events: i32,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecuteReplayError {
    #[error("replay failed: {0}")]
    Failed(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ExecuteReplayUseCase {
    dlq_client: Arc<dyn DlqManagerClient>,
}

impl ExecuteReplayUseCase {
    pub fn new(dlq_client: Arc<dyn DlqManagerClient>) -> Self {
        Self { dlq_client }
    }

    pub async fn execute(
        &self,
        input: &ExecuteReplayInput,
    ) -> Result<ExecuteReplayOutput, ExecuteReplayError> {
        let resp = self
            .dlq_client
            .execute_replay(&ReplayRequest {
                correlation_ids: input.correlation_ids.clone(),
                from_step_index: input.from_step_index,
                include_downstream: input.include_downstream,
                dry_run: input.dry_run,
            })
            .await
            .map_err(|e| ExecuteReplayError::Failed(e.to_string()))?;

        Ok(ExecuteReplayOutput {
            replay_id: resp.replay_id,
            status: resp.status,
            total_events: resp.total_events,
            replayed_events: resp.replayed_events,
            started_at: chrono::Utc::now(),
        })
    }
}
