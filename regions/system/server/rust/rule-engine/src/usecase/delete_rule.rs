use std::sync::Arc;
use uuid::Uuid;

use crate::domain::repository::RuleRepository;
use crate::infrastructure::kafka_producer::{
    NoopRuleEventPublisher, RuleChangedEvent, RuleEventPublisher,
};

#[derive(Debug, thiserror::Error)]
pub enum DeleteRuleError {
    #[error("rule not found: {0}")]
    NotFound(Uuid),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteRuleUseCase {
    repo: Arc<dyn RuleRepository>,
    event_publisher: Arc<dyn RuleEventPublisher>,
}

impl DeleteRuleUseCase {
    #[allow(dead_code)]
    pub fn new(repo: Arc<dyn RuleRepository>) -> Self {
        Self {
            repo,
            event_publisher: Arc::new(NoopRuleEventPublisher),
        }
    }

    pub fn with_publisher(
        repo: Arc<dyn RuleRepository>,
        event_publisher: Arc<dyn RuleEventPublisher>,
    ) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeleteRuleError> {
        let rule = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| DeleteRuleError::Internal(e.to_string()))?
            .ok_or(DeleteRuleError::NotFound(*id))?;

        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteRuleError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteRuleError::NotFound(*id));
        }

        if let Err(e) = self
            .event_publisher
            .publish_rule_changed(&RuleChangedEvent::rule_deleted(&rule))
            .await
        {
            tracing::warn!(error = %e, rule_id = %rule.id, "failed to publish rule deleted event");
        }

        Ok(())
    }
}
