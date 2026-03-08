use std::sync::Arc;

use crate::domain::entity::rule::Rule;
use crate::domain::repository::RuleRepository;
use crate::domain::service::condition_parser::ConditionParser;
use crate::infrastructure::kafka_producer::{
    NoopRuleEventPublisher, RuleChangedEvent, RuleEventPublisher,
};

#[derive(Debug, Clone)]
pub struct CreateRuleInput {
    pub name: String,
    pub description: String,
    pub priority: i32,
    pub when_condition: serde_json::Value,
    pub then_result: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateRuleError {
    #[error("rule already exists: {0}")]
    AlreadyExists(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("invalid condition: {0}")]
    InvalidCondition(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateRuleUseCase {
    repo: Arc<dyn RuleRepository>,
    event_publisher: Arc<dyn RuleEventPublisher>,
}

impl CreateRuleUseCase {
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

    pub async fn execute(&self, input: &CreateRuleInput) -> Result<Rule, CreateRuleError> {
        if input.name.is_empty() {
            return Err(CreateRuleError::Validation("name is required".to_string()));
        }
        if input.priority < 1 || input.priority > 1000 {
            return Err(CreateRuleError::Validation(
                "priority must be between 1 and 1000".to_string(),
            ));
        }

        ConditionParser::parse(&input.when_condition)
            .map_err(|e| CreateRuleError::InvalidCondition(e))?;

        let exists = self
            .repo
            .exists_by_name(&input.name)
            .await
            .map_err(|e| CreateRuleError::Internal(e.to_string()))?;

        if exists {
            return Err(CreateRuleError::AlreadyExists(input.name.clone()));
        }

        let rule = Rule::new(
            input.name.clone(),
            input.description.clone(),
            input.priority,
            input.when_condition.clone(),
            input.then_result.clone(),
        );

        self.repo
            .create(&rule)
            .await
            .map_err(|e| CreateRuleError::Internal(e.to_string()))?;

        if let Err(e) = self
            .event_publisher
            .publish_rule_changed(&RuleChangedEvent::rule_created(&rule))
            .await
        {
            tracing::warn!(error = %e, rule_id = %rule.id, "failed to publish rule created event");
        }

        Ok(rule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::rule_repository::MockRuleRepository;

    #[tokio::test]
    async fn success() {
        let mut mock = MockRuleRepository::new();
        mock.expect_exists_by_name()
            .withf(|name| name == "test-rule")
            .returning(|_| Ok(false));
        mock.expect_create().returning(|_| Ok(()));

        let uc = CreateRuleUseCase::new(Arc::new(mock));
        let input = CreateRuleInput {
            name: "test-rule".to_string(),
            description: "Test rule".to_string(),
            priority: 10,
            when_condition: serde_json::json!({"field": "status", "operator": "eq", "value": "active"}),
            then_result: serde_json::json!({"action": "approve"}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        let rule = result.unwrap();
        assert_eq!(rule.name, "test-rule");
        assert_eq!(rule.priority, 10);
    }

    #[tokio::test]
    async fn already_exists() {
        let mut mock = MockRuleRepository::new();
        mock.expect_exists_by_name().returning(|_| Ok(true));

        let uc = CreateRuleUseCase::new(Arc::new(mock));
        let input = CreateRuleInput {
            name: "existing".to_string(),
            description: "".to_string(),
            priority: 10,
            when_condition: serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
            then_result: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(CreateRuleError::AlreadyExists(_))));
    }

    #[tokio::test]
    async fn invalid_priority() {
        let mock = MockRuleRepository::new();
        let uc = CreateRuleUseCase::new(Arc::new(mock));
        let input = CreateRuleInput {
            name: "test".to_string(),
            description: "".to_string(),
            priority: 0,
            when_condition: serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
            then_result: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(CreateRuleError::Validation(_))));
    }

    #[tokio::test]
    async fn invalid_condition() {
        let mock = MockRuleRepository::new();
        let uc = CreateRuleUseCase::new(Arc::new(mock));
        let input = CreateRuleInput {
            name: "test".to_string(),
            description: "".to_string(),
            priority: 10,
            when_condition: serde_json::json!({"field": "x", "operator": "between", "value": 1}),
            then_result: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(CreateRuleError::InvalidCondition(_))));
    }
}
