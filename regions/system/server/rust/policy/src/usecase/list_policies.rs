use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::policy::Policy;
use crate::domain::repository::PolicyRepository;

#[derive(Debug, Clone)]
pub struct ListPoliciesInput {
    pub page: u32,
    pub page_size: u32,
    pub bundle_id: Option<Uuid>,
    pub enabled_only: bool,
    /// テナント ID: CRIT-005 対応。テナントでフィルタリングする。
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct ListPoliciesOutput {
    pub policies: Vec<Policy>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListPoliciesError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListPoliciesUseCase {
    repo: Arc<dyn PolicyRepository>,
}

impl ListPoliciesUseCase {
    pub fn new(repo: Arc<dyn PolicyRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ListPoliciesInput,
    ) -> Result<ListPoliciesOutput, ListPoliciesError> {
        // M-004 監査対応: DoS 防止のためページサイズを 1〜100 に制限する（config サービスの模範実装に準拠）
        let page_size = input.page_size.clamp(1, 100);

        // CRIT-005 対応: tenant_id を渡して RLS セッション変数を設定する
        let (policies, total_count) = self
            .repo
            .find_all_paginated(
                input.page,
                page_size,
                input.bundle_id,
                input.enabled_only,
                &input.tenant_id,
            )
            .await
            .map_err(|e| ListPoliciesError::Internal(e.to_string()))?;

        let has_next = u64::from(input.page) * u64::from(page_size) < total_count;

        Ok(ListPoliciesOutput {
            policies,
            total_count,
            page: input.page,
            page_size,
            has_next,
        })
    }
}
