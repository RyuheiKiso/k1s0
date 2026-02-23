use std::sync::Arc;

use crate::domain::entity::quota::QuotaPolicy;
use crate::domain::repository::QuotaPolicyRepository;

#[derive(Debug, Clone)]
pub struct ListQuotaPoliciesInput {
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct ListQuotaPoliciesOutput {
    pub quotas: Vec<QuotaPolicy>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListQuotaPoliciesError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListQuotaPoliciesUseCase {
    repo: Arc<dyn QuotaPolicyRepository>,
}

impl ListQuotaPoliciesUseCase {
    pub fn new(repo: Arc<dyn QuotaPolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ListQuotaPoliciesInput,
    ) -> Result<ListQuotaPoliciesOutput, ListQuotaPoliciesError> {
        let (quotas, total_count) = self
            .repo
            .find_all(input.page, input.page_size)
            .await
            .map_err(|e| ListQuotaPoliciesError::Internal(e.to_string()))?;

        let has_next = (input.page as u64 * input.page_size as u64) < total_count;

        Ok(ListQuotaPoliciesOutput {
            quotas,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::quota::{Period, SubjectType};
    use crate::domain::repository::quota_repository::MockQuotaPolicyRepository;

    fn sample_policy(name: &str) -> QuotaPolicy {
        QuotaPolicy::new(
            name.to_string(),
            SubjectType::Tenant,
            "tenant-1".to_string(),
            1000,
            Period::Daily,
            true,
            None,
        )
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockQuotaPolicyRepository::new();
        let policies = vec![sample_policy("p1"), sample_policy("p2")];
        let return_policies = policies.clone();

        mock.expect_find_all()
            .withf(|page, page_size| *page == 1 && *page_size == 20)
            .returning(move |_, _| Ok((return_policies.clone(), 2)));

        let uc = ListQuotaPoliciesUseCase::new(Arc::new(mock));
        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.quotas.len(), 2);
        assert_eq!(output.total_count, 2);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn success_with_pagination() {
        let mut mock = MockQuotaPolicyRepository::new();
        let policies = vec![sample_policy("p1")];
        let return_policies = policies.clone();

        mock.expect_find_all()
            .returning(move |_, _| Ok((return_policies.clone(), 25)));

        let uc = ListQuotaPoliciesUseCase::new(Arc::new(mock));
        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 10,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.has_next);
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockQuotaPolicyRepository::new();
        mock.expect_find_all()
            .returning(|_, _| Err(anyhow::anyhow!("db error")));

        let uc = ListQuotaPoliciesUseCase::new(Arc::new(mock));
        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ListQuotaPoliciesError::Internal(msg) => assert!(msg.contains("db error")),
        }
    }
}
