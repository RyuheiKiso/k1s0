use std::sync::Arc;

use crate::domain::entity::rule::RuleSet;
use crate::domain::repository::RuleSetRepository;

#[derive(Debug, Clone)]
pub struct ListRuleSetsInput {
    pub page: u32,
    pub page_size: u32,
    pub domain: Option<String>,
}

#[derive(Debug)]
pub struct ListRuleSetsOutput {
    pub rule_sets: Vec<RuleSet>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListRuleSetsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListRuleSetsUseCase {
    repo: Arc<dyn RuleSetRepository>,
}

impl ListRuleSetsUseCase {
    pub fn new(repo: Arc<dyn RuleSetRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ListRuleSetsInput,
    ) -> Result<ListRuleSetsOutput, ListRuleSetsError> {
        // M-002 監査対応: DoS 防止のためページサイズを 1〜100 に制限する（config サービスの模範実装に準拠）
        let page_size = input.page_size.clamp(1, 100);

        let (rule_sets, total_count) = self
            .repo
            .find_all_paginated(input.page, page_size, input.domain.clone())
            .await
            .map_err(|e| ListRuleSetsError::Internal(e.to_string()))?;

        let has_next = (u64::from(input.page) * u64::from(page_size)) < total_count;

        Ok(ListRuleSetsOutput {
            rule_sets,
            total_count,
            page: input.page,
            page_size,
            has_next,
        })
    }
}
