use std::sync::Arc;

use crate::domain::entity::RateLimitRule;
use crate::domain::repository::RateLimitRepository;

/// ListRulesError はルール一覧取得に関するエラー。
#[derive(Debug, thiserror::Error)]
pub enum ListRulesError {
    #[error("internal error: {0}")]
    Internal(String),
}

/// ListRulesUseCase はルール一覧取得ユースケース。
pub struct ListRulesUseCase {
    repo: Arc<dyn RateLimitRepository>,
}

pub struct ListRulesInput {
    pub page: u32,
    pub page_size: u32,
    pub scope: Option<String>,
    pub enabled_only: bool,
}

#[derive(Debug)]
pub struct ListRulesOutput {
    pub rules: Vec<RateLimitRule>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

impl ListRulesUseCase {
    pub fn new(repo: Arc<dyn RateLimitRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &ListRulesInput) -> Result<ListRulesOutput, ListRulesError> {
        let page = input.page.max(1);
        let page_size = input.page_size.clamp(1, 200);
        let (rules, total_count) = self
            .repo
            .find_page(page, page_size, input.scope.clone(), input.enabled_only)
            .await
            .map_err(|e| ListRulesError::Internal(e.to_string()))?;
        let has_next = (page as u64 * page_size as u64) < total_count;
        Ok(ListRulesOutput {
            rules,
            total_count,
            page,
            page_size,
            has_next,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::Algorithm;
    use crate::domain::repository::rate_limit_repository::MockRateLimitRepository;

    #[tokio::test]
    async fn test_list_rules_success() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_page().returning(|_, _, _, _| {
            Ok((
                vec![
                    RateLimitRule::new(
                        "service".to_string(),
                        "pattern-1".to_string(),
                        100,
                        60,
                        Algorithm::TokenBucket,
                    ),
                    RateLimitRule::new(
                        "user".to_string(),
                        "pattern-2".to_string(),
                        200,
                        120,
                        Algorithm::FixedWindow,
                    ),
                ],
                2,
            ))
        });

        let uc = ListRulesUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&ListRulesInput {
                page: 1,
                page_size: 20,
                scope: None,
                enabled_only: false,
            })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().rules.len(), 2);
    }

    #[tokio::test]
    async fn test_list_rules_empty() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_page()
            .returning(|_, _, _, _| Ok((vec![], 0)));

        let uc = ListRulesUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&ListRulesInput {
                page: 1,
                page_size: 20,
                scope: None,
                enabled_only: false,
            })
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap().rules.is_empty());
    }

    #[tokio::test]
    async fn test_list_rules_internal_error() {
        let mut repo = MockRateLimitRepository::new();
        repo.expect_find_page()
            .returning(|_, _, _, _| Err(anyhow::anyhow!("db error")));

        let uc = ListRulesUseCase::new(Arc::new(repo));
        let result = uc
            .execute(&ListRulesInput {
                page: 1,
                page_size: 20,
                scope: None,
                enabled_only: false,
            })
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ListRulesError::Internal(msg) => assert!(msg.contains("db error")),
        }
    }
}
