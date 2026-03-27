use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::Rule;
use crate::domain::repository::RuleRepository;
use crate::domain::service::condition_parser::ConditionParser;
use crate::infrastructure::kafka_producer::{
    NoopRuleEventPublisher, RuleChangedEvent, RuleEventPublisher,
};

#[derive(Debug, Clone)]
pub struct UpdateRuleInput {
    pub id: Uuid,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub when_condition: Option<serde_json::Value>,
    pub then_result: Option<serde_json::Value>,
    pub enabled: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateRuleError {
    #[error("rule not found: {0}")]
    NotFound(Uuid),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("invalid condition: {0}")]
    InvalidCondition(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateRuleUseCase {
    repo: Arc<dyn RuleRepository>,
    event_publisher: Arc<dyn RuleEventPublisher>,
}

impl UpdateRuleUseCase {
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

    pub async fn execute(&self, input: &UpdateRuleInput) -> Result<Rule, UpdateRuleError> {
        let mut rule = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateRuleError::Internal(e.to_string()))?
            .ok_or(UpdateRuleError::NotFound(input.id))?;

        if let Some(ref desc) = input.description {
            rule.description = desc.clone();
        }
        if let Some(priority) = input.priority {
            if !(1..=1000).contains(&priority) {
                return Err(UpdateRuleError::Validation(
                    "priority must be between 1 and 1000".to_string(),
                ));
            }
            rule.priority = priority;
        }
        if let Some(ref when) = input.when_condition {
            ConditionParser::parse(when).map_err(UpdateRuleError::InvalidCondition)?;
            rule.when_condition = when.clone();
        }
        if let Some(ref then) = input.then_result {
            rule.then_result = then.clone();
        }
        if let Some(enabled) = input.enabled {
            rule.enabled = enabled;
        }

        rule.version += 1;
        rule.updated_at = chrono::Utc::now();

        self.repo
            .update(&rule)
            .await
            .map_err(|e| UpdateRuleError::Internal(e.to_string()))?;

        if let Err(e) = self
            .event_publisher
            .publish_rule_changed(&RuleChangedEvent::rule_updated(&rule))
            .await
        {
            tracing::warn!(error = %e, rule_id = %rule.id, "failed to publish rule updated event");
        }

        Ok(rule)
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
            "original".to_string(),
            "original desc".to_string(),
            10,
            serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
            serde_json::json!({"action": "allow"}),
        )
    }

    /// 正常なUpdate入力でルールが更新される
    #[tokio::test]
    async fn success_updates_rule() {
        let rule = sample_rule();
        let id = rule.id;
        let original_version = rule.version;
        let mut mock = MockRuleRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(sample_rule())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = UpdateRuleUseCase::new(Arc::new(mock));
        let input = UpdateRuleInput {
            id,
            description: Some("new description".to_string()),
            priority: Some(20),
            when_condition: None,
            then_result: None,
            enabled: None,
        };
        let result = uc.execute(&input).await.unwrap();
        assert_eq!(result.description, "new description");
        assert_eq!(result.priority, 20);
        assert_eq!(result.version, original_version + 1);
    }

    /// ルールが存在しない場合は NotFound を返す
    #[tokio::test]
    async fn not_found() {
        let mut mock = MockRuleRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdateRuleUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&UpdateRuleInput {
                id: Uuid::new_v4(),
                description: None,
                priority: None,
                when_condition: None,
                then_result: None,
                enabled: None,
            })
            .await;
        assert!(matches!(result, Err(UpdateRuleError::NotFound(_))));
    }

    /// 優先度が範囲外の場合は Validation エラーを返す
    #[tokio::test]
    async fn invalid_priority() {
        let mut mock = MockRuleRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(sample_rule())));

        let uc = UpdateRuleUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&UpdateRuleInput {
                id: Uuid::new_v4(),
                description: None,
                priority: Some(9999),
                when_condition: None,
                then_result: None,
                enabled: None,
            })
            .await;
        assert!(matches!(result, Err(UpdateRuleError::Validation(_))));
    }

    /// 不正な条件式の場合は InvalidCondition エラーを返す
    #[tokio::test]
    async fn invalid_condition() {
        let mut mock = MockRuleRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(sample_rule())));

        let uc = UpdateRuleUseCase::new(Arc::new(mock));
        let result = uc
            .execute(&UpdateRuleInput {
                id: Uuid::new_v4(),
                description: None,
                priority: None,
                when_condition: Some(
                    serde_json::json!({"field": "x", "operator": "unknown_op", "value": 1}),
                ),
                then_result: None,
                enabled: None,
            })
            .await;
        assert!(matches!(result, Err(UpdateRuleError::InvalidCondition(_))));
    }
}
