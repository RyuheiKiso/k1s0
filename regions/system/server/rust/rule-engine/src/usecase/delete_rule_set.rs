use std::sync::Arc;
use uuid::Uuid;

use crate::domain::repository::RuleSetRepository;
use crate::infrastructure::kafka_producer::{
    NoopRuleEventPublisher, RuleChangedEvent, RuleEventPublisher,
};

#[derive(Debug, thiserror::Error)]
pub enum DeleteRuleSetError {
    #[error("rule set not found: {0}")]
    NotFound(Uuid),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct DeleteRuleSetUseCase {
    repo: Arc<dyn RuleSetRepository>,
    event_publisher: Arc<dyn RuleEventPublisher>,
}

impl DeleteRuleSetUseCase {
    #[allow(dead_code)]
    pub fn new(repo: Arc<dyn RuleSetRepository>) -> Self {
        Self {
            repo,
            event_publisher: Arc::new(NoopRuleEventPublisher),
        }
    }

    pub fn with_publisher(
        repo: Arc<dyn RuleSetRepository>,
        event_publisher: Arc<dyn RuleEventPublisher>,
    ) -> Self {
        Self {
            repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<(), DeleteRuleSetError> {
        let rule_set = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| DeleteRuleSetError::Internal(e.to_string()))?
            .ok_or(DeleteRuleSetError::NotFound(*id))?;

        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DeleteRuleSetError::Internal(e.to_string()))?;

        if !deleted {
            return Err(DeleteRuleSetError::NotFound(*id));
        }

        if let Err(e) = self
            .event_publisher
            .publish_rule_changed(&RuleChangedEvent::rule_set_deleted(&rule_set))
            .await
        {
            tracing::warn!(error = %e, rule_set_id = %id, "failed to publish rule set deleted event");
        }

        Ok(())
    }
}
