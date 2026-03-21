use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::Rule;
use crate::domain::repository::RuleRepository;

#[derive(Debug, thiserror::Error)]
pub enum GetRuleError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct GetRuleUseCase {
    repo: Arc<dyn RuleRepository>,
}

impl GetRuleUseCase {
    pub fn new(repo: Arc<dyn RuleRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: &Uuid) -> Result<Option<Rule>, GetRuleError> {
        self.repo
            .find_by_id(id)
            .await
            .map_err(|e| GetRuleError::Internal(e.to_string()))
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
            "sample".to_string(),
            "desc".to_string(),
            1,
            serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
            serde_json::json!({}),
        )
    }

    /// IDが存在する場合は Some(Rule) を返す
    #[tokio::test]
    async fn returns_some_when_found() {
        let rule = sample_rule();
        let id = rule.id;
        let mut mock = MockRuleRepository::new();
        mock.expect_find_by_id()
            .withf(move |i| *i == id)
            .returning(move |_| Ok(Some(sample_rule())));

        let uc = GetRuleUseCase::new(Arc::new(mock));
        let result = uc.execute(&id).await.unwrap();
        assert!(result.is_some());
    }

    /// IDが存在しない場合は None を返す
    #[tokio::test]
    async fn returns_none_when_not_found() {
        let id = Uuid::new_v4();
        let mut mock = MockRuleRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = GetRuleUseCase::new(Arc::new(mock));
        let result = uc.execute(&id).await.unwrap();
        assert!(result.is_none());
    }

    /// リポジトリがエラーを返す場合は Internal エラーになる
    #[tokio::test]
    async fn returns_internal_error_on_repo_failure() {
        let id = Uuid::new_v4();
        let mut mock = MockRuleRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = GetRuleUseCase::new(Arc::new(mock));
        let result = uc.execute(&id).await;
        assert!(matches!(result, Err(GetRuleError::Internal(_))));
    }
}
