use std::sync::Arc;

use crate::domain::entity::event_record::EventRecord;
use crate::domain::entity::flow_definition::FlowStep;
use crate::domain::entity::flow_instance::FlowInstance;
use crate::domain::repository::{
    EventRecordRepository, FlowDefinitionRepository, FlowInstanceRepository,
};

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
                source: step
                    .source_filter
                    .clone()
                    .unwrap_or_else(|| step.source.clone()),
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::event_record::EventRecord;
    use crate::domain::entity::flow_definition::{FlowDefinition, FlowSlo, FlowStep};
    use crate::domain::repository::event_record_repository::MockEventRecordRepository;
    use crate::domain::repository::flow_definition_repository::MockFlowDefinitionRepository;
    use crate::domain::repository::flow_instance_repository::MockFlowInstanceRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_event(corr_id: &str) -> EventRecord {
        EventRecord::new(
            corr_id.to_string(),
            "OrderCreated".to_string(),
            "order-service".to_string(),
            "service.order".to_string(),
            "trace-123".to_string(),
            Utc::now(),
        )
    }

    fn make_flow_with_steps(flow_id: Uuid) -> FlowDefinition {
        let mut flow = FlowDefinition::new(
            "order_flow".to_string(),
            "test".to_string(),
            "service.order".to_string(),
            vec![
                FlowStep {
                    event_type: "OrderCreated".to_string(),
                    source: "order-service".to_string(),
                    source_filter: None,
                    timeout_seconds: 30,
                    description: "step 0".to_string(),
                },
                FlowStep {
                    event_type: "PaymentProcessed".to_string(),
                    source: "payment-service".to_string(),
                    source_filter: None,
                    timeout_seconds: 60,
                    description: "step 1".to_string(),
                },
                FlowStep {
                    event_type: "OrderShipped".to_string(),
                    source: "shipping-service".to_string(),
                    source_filter: None,
                    timeout_seconds: 120,
                    description: "step 2".to_string(),
                },
            ],
            FlowSlo {
                target_completion_seconds: 300,
                target_success_rate: 0.99,
                alert_on_violation: true,
            },
        );
        flow.id = flow_id;
        flow
    }

    #[tokio::test]
    async fn with_flow() {
        let flow_id = Uuid::new_v4();
        let instance = FlowInstance::new(flow_id, "corr-123".to_string());
        // instance is InProgress at step 0, so steps 1 and 2 are pending

        let mut event_mock = MockEventRecordRepository::new();
        event_mock
            .expect_find_by_correlation_id()
            .returning(|_| Ok(vec![make_event("corr-123")]));

        let instance_clone = instance.clone();
        let mut inst_mock = MockFlowInstanceRepository::new();
        inst_mock
            .expect_find_by_correlation_id()
            .returning(move |_| Ok(Some(instance_clone.clone())));

        let mut def_mock = MockFlowDefinitionRepository::new();
        def_mock
            .expect_find_by_id()
            .returning(move |_| Ok(Some(make_flow_with_steps(flow_id))));

        let uc = TraceByCorrelationUseCase::new(
            Arc::new(event_mock),
            Arc::new(def_mock),
            Arc::new(inst_mock),
        );
        let result = uc.execute("corr-123").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.correlation_id, "corr-123");
        assert_eq!(output.events.len(), 1);
        assert!(output.flow_instance.is_some());
        assert_eq!(output.flow_name, Some("order_flow".to_string()));
        assert_eq!(output.pending_steps.len(), 2);
        assert_eq!(output.pending_steps[0].event_type, "PaymentProcessed");
        assert_eq!(output.pending_steps[1].event_type, "OrderShipped");
    }

    #[tokio::test]
    async fn without_flow() {
        let mut event_mock = MockEventRecordRepository::new();
        event_mock
            .expect_find_by_correlation_id()
            .returning(|_| Ok(vec![make_event("corr-456")]));

        let mut inst_mock = MockFlowInstanceRepository::new();
        inst_mock
            .expect_find_by_correlation_id()
            .returning(|_| Ok(None));

        let def_mock = MockFlowDefinitionRepository::new();

        let uc = TraceByCorrelationUseCase::new(
            Arc::new(event_mock),
            Arc::new(def_mock),
            Arc::new(inst_mock),
        );
        let result = uc.execute("corr-456").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.flow_instance.is_none());
        assert!(output.flow_name.is_none());
        assert!(output.pending_steps.is_empty());
    }

    #[tokio::test]
    async fn not_found() {
        let mut event_mock = MockEventRecordRepository::new();
        event_mock
            .expect_find_by_correlation_id()
            .returning(|_| Ok(vec![]));

        let inst_mock = MockFlowInstanceRepository::new();
        let def_mock = MockFlowDefinitionRepository::new();

        let uc = TraceByCorrelationUseCase::new(
            Arc::new(event_mock),
            Arc::new(def_mock),
            Arc::new(inst_mock),
        );
        let result = uc.execute("corr-missing").await;
        assert!(matches!(result, Err(TraceByCorrelationError::NotFound(_))));
    }
}
