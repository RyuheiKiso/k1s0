use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::rule::Rule;
use crate::domain::repository::RuleRepository;

#[derive(Debug, Clone)]
pub struct ListRulesInput {
    pub page: u32,
    pub page_size: u32,
    pub rule_set_id: Option<Uuid>,
    pub domain: Option<String>,
}

#[derive(Debug)]
pub struct ListRulesOutput {
    pub rules: Vec<Rule>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListRulesError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListRulesUseCase {
    repo: Arc<dyn RuleRepository>,
}

impl ListRulesUseCase {
    pub fn new(repo: Arc<dyn RuleRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &ListRulesInput) -> Result<ListRulesOutput, ListRulesError> {
        let (rules, total_count) = self
            .repo
            .find_all_paginated(
                input.page,
                input.page_size,
                input.rule_set_id,
                input.domain.clone(),
            )
            .await
            .map_err(|e| ListRulesError::Internal(e.to_string()))?;

        let has_next = (input.page as u64 * input.page_size as u64) < total_count;

        Ok(ListRulesOutput {
            rules,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::rule::Rule;
    use crate::domain::repository::rule_repository::MockRuleRepository;

    fn sample_rule(name: &str) -> Rule {
        Rule::new(
            name.to_string(),
            "desc".to_string(),
            1,
            serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
            serde_json::json!({}),
        )
    }

    /// ルールが存在する場合にページネーション付きで返される
    #[tokio::test]
    async fn returns_rules_with_pagination() {
        let mut mock = MockRuleRepository::new();
        mock.expect_find_all_paginated()
            .returning(|_, _, _, _| Ok((vec![sample_rule("r1"), sample_rule("r2")], 5)));

        let uc = ListRulesUseCase::new(Arc::new(mock));
        let input = ListRulesInput {
            page: 1,
            page_size: 2,
            rule_set_id: None,
            domain: None,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.rules.len(), 2);
        assert_eq!(output.total_count, 5);
        assert_eq!(output.page, 1);
        assert_eq!(output.page_size, 2);
    }

    /// has_next が正しく計算される（次ページあり）
    #[tokio::test]
    async fn has_next_true_when_more_items() {
        let mut mock = MockRuleRepository::new();
        mock.expect_find_all_paginated()
            .returning(|_, _, _, _| Ok((vec![sample_rule("r1")], 10)));

        let uc = ListRulesUseCase::new(Arc::new(mock));
        let input = ListRulesInput {
            page: 1,
            page_size: 5,
            rule_set_id: None,
            domain: None,
        };
        let output = uc.execute(&input).await.unwrap();
        // page=1, page_size=5 -> 5 < 10 -> has_next=true
        assert!(output.has_next);
    }

    /// has_next が正しく計算される（次ページなし）
    #[tokio::test]
    async fn has_next_false_when_last_page() {
        let mut mock = MockRuleRepository::new();
        mock.expect_find_all_paginated()
            .returning(|_, _, _, _| Ok((vec![sample_rule("r1")], 3)));

        let uc = ListRulesUseCase::new(Arc::new(mock));
        let input = ListRulesInput {
            page: 1,
            page_size: 5,
            rule_set_id: None,
            domain: None,
        };
        let output = uc.execute(&input).await.unwrap();
        // page=1, page_size=5 -> 5 >= 3 -> has_next=false
        assert!(!output.has_next);
    }
}
