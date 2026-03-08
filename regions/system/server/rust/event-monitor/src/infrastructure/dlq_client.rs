use async_trait::async_trait;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReplayRequest {
    pub correlation_ids: Vec<String>,
    pub from_step_index: i32,
    pub include_downstream: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct ReplayResponse {
    pub replay_id: String,
    pub status: String,
    pub total_events: i32,
    pub replayed_events: i32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ReplayPreviewResponse {
    pub total_events_to_replay: i32,
    pub affected_services: Vec<String>,
    pub dlq_messages_found: i32,
    pub estimated_duration_seconds: i32,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DlqManagerClient: Send + Sync {
    async fn preview_replay(&self, req: &ReplayRequest) -> anyhow::Result<ReplayPreviewResponse>;
    async fn execute_replay(&self, req: &ReplayRequest) -> anyhow::Result<ReplayResponse>;
}

pub struct NoopDlqClient;

#[async_trait]
impl DlqManagerClient for NoopDlqClient {
    async fn preview_replay(&self, _req: &ReplayRequest) -> anyhow::Result<ReplayPreviewResponse> {
        Ok(ReplayPreviewResponse {
            total_events_to_replay: 0,
            affected_services: vec![],
            dlq_messages_found: 0,
            estimated_duration_seconds: 0,
        })
    }

    async fn execute_replay(&self, _req: &ReplayRequest) -> anyhow::Result<ReplayResponse> {
        Ok(ReplayResponse {
            replay_id: uuid::Uuid::new_v4().to_string(),
            status: "noop".to_string(),
            total_events: 0,
            replayed_events: 0,
        })
    }
}
