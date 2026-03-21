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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::rule::Rule;
    use crate::domain::repository::rule_repository::MockRuleRepository;

    fn sample_rule() -> Rule {
        Rule::new(
            "to-delete".to_string(),
            "desc".to_string(),
            1,
            serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
            serde_json::json!({}),
        )
    }

    /// ルールが存在する場合は正常に削除される
    #[tokio::test]
    async fn success() {
        let rule = sample_rule();
        let id = rule.id;
        let mut mock = MockRuleRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(sample_rule())));
        mock.expect_delete().returning(|_| Ok(true));

        let uc = DeleteRuleUseCase::new(Arc::new(mock));
        let result = uc.execute(&id).await;
        assert!(result.is_ok());
    }

    /// ルールが存在しない場合は NotFound エラーを返す
    #[tokio::test]
    async fn not_found_when_missing() {
        let id = Uuid::new_v4();
        let mut mock = MockRuleRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = DeleteRuleUseCase::new(Arc::new(mock));
        let result = uc.execute(&id).await;
        assert!(matches!(result, Err(DeleteRuleError::NotFound(_))));
    }

    /// delete が false を返す場合は NotFound エラーを返す
    #[tokio::test]
    async fn not_found_when_delete_returns_false() {
        let mut mock = MockRuleRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(sample_rule())));
        mock.expect_delete().returning(|_| Ok(false));

        let uc = DeleteRuleUseCase::new(Arc::new(mock));
        let result = uc.execute(&Uuid::new_v4()).await;
        assert!(matches!(result, Err(DeleteRuleError::NotFound(_))));
    }
}
