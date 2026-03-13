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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::dlq_client::{MockDlqManagerClient, ReplayResponse};

    #[tokio::test]
    async fn success() {
        let mut dlq_mock = MockDlqManagerClient::new();
        dlq_mock.expect_execute_replay().returning(|_| {
            Ok(ReplayResponse {
                replay_id: "replay-001".to_string(),
                status: "completed".to_string(),
                total_events: 5,
                replayed_events: 5,
            })
        });

        let uc = ExecuteReplayUseCase::new(Arc::new(dlq_mock));
        let input = ExecuteReplayInput {
            correlation_ids: vec!["corr-1".to_string()],
            from_step_index: 0,
            include_downstream: true,
            dry_run: false,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.replay_id, "replay-001");
        assert_eq!(output.status, "completed");
        assert_eq!(output.total_events, 5);
        assert_eq!(output.replayed_events, 5);
    }

    #[tokio::test]
    async fn in_progress() {
        let dlq_mock = MockDlqManagerClient::new();
        let uc = ExecuteReplayUseCase::new(Arc::new(dlq_mock));

        // Pre-insert a correlation_id into active_replays
        {
            let mut active = uc.active_replays.write().await;
            active.insert("corr-1".to_string());
        }

        let input = ExecuteReplayInput {
            correlation_ids: vec!["corr-1".to_string()],
            from_step_index: 0,
            include_downstream: false,
            dry_run: false,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(ExecuteReplayError::ReplayInProgress(_))));
    }

    #[tokio::test]
    async fn dlq_failure() {
        let mut dlq_mock = MockDlqManagerClient::new();
        dlq_mock
            .expect_execute_replay()
            .returning(|_| Err(anyhow::anyhow!("dlq unavailable")));

        let uc = ExecuteReplayUseCase::new(Arc::new(dlq_mock));
        let input = ExecuteReplayInput {
            correlation_ids: vec!["corr-1".to_string()],
            from_step_index: 0,
            include_downstream: false,
            dry_run: false,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(ExecuteReplayError::Failed(_))));
    }
}
