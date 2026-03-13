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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::event_record::EventRecord;
    use crate::domain::entity::flow_definition::{FlowDefinition, FlowSlo, FlowStep};
    use crate::domain::repository::event_record_repository::MockEventRecordRepository;
    use crate::domain::repository::flow_definition_repository::MockFlowDefinitionRepository;
    use crate::infrastructure::dlq_client::{MockDlqManagerClient, ReplayPreviewResponse};
    use chrono::Utc;
    use uuid::Uuid;

    fn make_event_with_flow(corr_id: &str, flow_id: Uuid, step_index: i32) -> EventRecord {
        let mut event = EventRecord::new(
            corr_id.to_string(),
            "OrderCreated".to_string(),
            "order-service".to_string(),
            "service.order".to_string(),
            "trace-123".to_string(),
            Utc::now(),
        );
        event.flow_id = Some(flow_id);
        event.flow_step_index = Some(step_index);
        event
    }

    fn make_flow() -> FlowDefinition {
        FlowDefinition::new(
            "order_flow".to_string(),
            "test".to_string(),
            "service.order".to_string(),
            vec![FlowStep {
                event_type: "OrderCreated".to_string(),
                source: "order-service".to_string(),
                source_filter: None,
                timeout_seconds: 30,
                description: String::new(),
            }],
            FlowSlo {
                target_completion_seconds: 120,
                target_success_rate: 0.99,
                alert_on_violation: true,
            },
        )
    }

    #[tokio::test]
    async fn success() {
        let flow = make_flow();
        let flow_id = flow.id;
        let flow_clone = flow.clone();

        let mut event_mock = MockEventRecordRepository::new();
        event_mock
            .expect_find_by_correlation_id()
            .returning(move |_| {
                Ok(vec![
                    make_event_with_flow("corr-1", flow_id, 0),
                    make_event_with_flow("corr-1", flow_id, 1),
                ])
            });

        let mut def_mock = MockFlowDefinitionRepository::new();
        def_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(flow_clone.clone())));

        let mut dlq_mock = MockDlqManagerClient::new();
        dlq_mock.expect_preview_replay().returning(|_| {
            Ok(ReplayPreviewResponse {
                total_events_to_replay: 2,
                affected_services: vec!["order-service".to_string()],
                dlq_messages_found: 1,
                estimated_duration_seconds: 10,
            })
        });

        let uc = PreviewReplayUseCase::new(
            Arc::new(event_mock),
            Arc::new(def_mock),
            Arc::new(dlq_mock),
        );
        let input = PreviewReplayInput {
            correlation_ids: vec!["corr-1".to_string()],
            from_step_index: 0,
            include_downstream: true,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.total_events_to_replay, 2);
        assert_eq!(output.dlq_messages_found, 1);
        assert_eq!(output.affected_flows.len(), 1);
        assert_eq!(output.affected_flows[0].flow_name, "order_flow");
    }

    #[tokio::test]
    async fn no_events() {
        let mut event_mock = MockEventRecordRepository::new();
        event_mock
            .expect_find_by_correlation_id()
            .returning(|_| Ok(vec![]));

        let def_mock = MockFlowDefinitionRepository::new();

        let mut dlq_mock = MockDlqManagerClient::new();
        dlq_mock.expect_preview_replay().returning(|_| {
            Ok(ReplayPreviewResponse {
                total_events_to_replay: 0,
                affected_services: vec![],
                dlq_messages_found: 0,
                estimated_duration_seconds: 0,
            })
        });

        let uc = PreviewReplayUseCase::new(
            Arc::new(event_mock),
            Arc::new(def_mock),
            Arc::new(dlq_mock),
        );
        let input = PreviewReplayInput {
            correlation_ids: vec!["corr-missing".to_string()],
            from_step_index: 0,
            include_downstream: false,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.total_events_to_replay, 0);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut event_mock = MockEventRecordRepository::new();
        event_mock
            .expect_find_by_correlation_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let def_mock = MockFlowDefinitionRepository::new();
        let dlq_mock = MockDlqManagerClient::new();

        let uc = PreviewReplayUseCase::new(
            Arc::new(event_mock),
            Arc::new(def_mock),
            Arc::new(dlq_mock),
        );
        let input = PreviewReplayInput {
            correlation_ids: vec!["corr-1".to_string()],
            from_step_index: 0,
            include_downstream: false,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(PreviewReplayError::Internal(_))));
    }
}
