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
