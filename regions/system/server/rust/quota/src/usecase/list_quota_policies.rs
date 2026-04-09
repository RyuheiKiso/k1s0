use std::sync::Arc;

use crate::domain::entity::quota::QuotaPolicy;
use crate::domain::repository::QuotaPolicyRepository;

/// CRITICAL-RUST-001 監査対応: `tenant_id` を追加して RLS テナント分離を有効にする。
#[derive(Debug, Clone)]
pub struct ListQuotaPoliciesInput {
    pub page: u32,
    pub page_size: u32,
    pub subject_type: Option<String>,
    pub subject_id: Option<String>,
    pub enabled_only: Option<bool>,
    pub tenant_id: String,
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
        let has_filters = input.subject_type.is_some()
            || input.subject_id.is_some()
            || input.enabled_only.unwrap_or(false);

        if !has_filters {
            let (quotas, total_count) = self
                .repo
                .find_all(input.page, input.page_size, &input.tenant_id)
                .await
                .map_err(|e| ListQuotaPoliciesError::Internal(e.to_string()))?;

            let has_next = (u64::from(input.page) * u64::from(input.page_size)) < total_count;

            return Ok(ListQuotaPoliciesOutput {
                quotas,
                total_count,
                page: input.page,
                page_size: input.page_size,
                has_next,
            });
        }

        let mut all_filtered = Vec::new();
        let mut fetch_page = 1;
        let fetch_page_size = 100;
        let mut fetched: u64 = 0;

        loop {
            let (items, total_count) = self
                .repo
                .find_all(fetch_page, fetch_page_size, &input.tenant_id)
                .await
                .map_err(|e| ListQuotaPoliciesError::Internal(e.to_string()))?;
            // LOW-008: 安全な型変換（オーバーフロー防止）
            fetched += u64::try_from(items.len()).unwrap_or(u64::MAX);

            all_filtered.extend(items.into_iter().filter(|policy| {
                if let Some(ref subject_type) = input.subject_type {
                    if policy.subject_type.as_str() != subject_type {
                        return false;
                    }
                }
                if let Some(ref subject_id) = input.subject_id {
                    if &policy.subject_id != subject_id {
                        return false;
                    }
                }
                if input.enabled_only.unwrap_or(false) && !policy.enabled {
                    return false;
                }
                true
            }));

            if fetched >= total_count {
                break;
            }
            fetch_page += 1;
        }

        // LOW-008: 安全な型変換（オーバーフロー防止）
        let total_count = u64::try_from(all_filtered.len()).unwrap_or(u64::MAX);
        let start = usize::try_from(input.page.saturating_sub(1) * input.page_size).unwrap_or(0);
        let take_count = usize::try_from(input.page_size).unwrap_or(usize::MAX);
        let quotas = all_filtered
            .into_iter()
            .skip(start)
            .take(take_count)
            .collect();
        let has_next = (u64::from(input.page) * u64::from(input.page_size)) < total_count;

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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::quota::{Period, SubjectType};
    use crate::domain::repository::quota_repository::MockQuotaPolicyRepository;

    // テスト用ポリシーサンプルを生成するヘルパー関数（テナントIDを先頭引数に追加）
    fn sample_policy(name: &str) -> QuotaPolicy {
        QuotaPolicy::new(
            "test-tenant".to_string(),
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
            .withf(|page, page_size, _tenant_id| *page == 1 && *page_size == 20)
            .returning(move |_, _, _| Ok((return_policies.clone(), 2)));

        let uc = ListQuotaPoliciesUseCase::new(Arc::new(mock));
        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 20,
            subject_type: None,
            subject_id: None,
            enabled_only: None,
            tenant_id: "tenant-1".to_string(),
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
            .returning(move |_, _, _| Ok((return_policies.clone(), 25)));

        let uc = ListQuotaPoliciesUseCase::new(Arc::new(mock));
        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 10,
            subject_type: None,
            subject_id: None,
            enabled_only: None,
            tenant_id: "tenant-1".to_string(),
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
            .returning(|_, _, _| Err(anyhow::anyhow!("db error")));

        let uc = ListQuotaPoliciesUseCase::new(Arc::new(mock));
        let input = ListQuotaPoliciesInput {
            page: 1,
            page_size: 20,
            subject_type: None,
            subject_id: None,
            enabled_only: None,
            tenant_id: "tenant-1".to_string(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ListQuotaPoliciesError::Internal(msg) => assert!(msg.contains("db error")),
        }
    }
}
