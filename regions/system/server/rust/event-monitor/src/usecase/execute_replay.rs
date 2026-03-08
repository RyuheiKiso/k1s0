use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    #[error("replay in progress for correlation_ids: {0}")]
    ReplayInProgress(String),

    #[error("replay failed: {0}")]
    Failed(String),

    #[allow(dead_code)]
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ExecuteReplayUseCase {
    dlq_client: Arc<dyn DlqManagerClient>,
    active_replays: Arc<RwLock<HashSet<String>>>,
}

impl ExecuteReplayUseCase {
    pub fn new(dlq_client: Arc<dyn DlqManagerClient>) -> Self {
        Self {
            dlq_client,
            active_replays: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn execute(
        &self,
        input: &ExecuteReplayInput,
    ) -> Result<ExecuteReplayOutput, ExecuteReplayError> {
        // Check for in-progress replays
        {
            let active = self.active_replays.read().await;
            let in_progress: Vec<&String> = input
                .correlation_ids
                .iter()
                .filter(|id| active.contains(*id))
                .collect();
            if !in_progress.is_empty() {
                return Err(ExecuteReplayError::ReplayInProgress(
                    in_progress.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "),
                ));
            }
        }

        // Mark as active
        {
            let mut active = self.active_replays.write().await;
            for id in &input.correlation_ids {
                active.insert(id.clone());
            }
        }

        let result = self
            .dlq_client
            .execute_replay(&ReplayRequest {
                correlation_ids: input.correlation_ids.clone(),
                from_step_index: input.from_step_index,
                include_downstream: input.include_downstream,
                dry_run: input.dry_run,
            })
            .await;

        // Remove from active on completion (success or failure)
        {
            let mut active = self.active_replays.write().await;
            for id in &input.correlation_ids {
                active.remove(id);
            }
        }

        let resp = result.map_err(|e| ExecuteReplayError::Failed(e.to_string()))?;

        Ok(ExecuteReplayOutput {
            replay_id: resp.replay_id,
            status: resp.status,
            total_events: resp.total_events,
            replayed_events: resp.replayed_events,
            started_at: chrono::Utc::now(),
        })
    }
}
