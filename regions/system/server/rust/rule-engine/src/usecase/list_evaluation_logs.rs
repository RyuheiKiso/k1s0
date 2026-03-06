use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::domain::entity::rule::EvaluationLog;
use crate::domain::repository::EvaluationLogRepository;

#[derive(Debug, Clone)]
pub struct ListEvaluationLogsInput {
    pub page: u32,
    pub page_size: u32,
    pub rule_set_name: Option<String>,
    pub domain: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct ListEvaluationLogsOutput {
    pub logs: Vec<EvaluationLog>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ListEvaluationLogsError {
    #[error("internal error: {0}")]
    Internal(String),
}

pub struct ListEvaluationLogsUseCase {
    repo: Arc<dyn EvaluationLogRepository>,
}

impl ListEvaluationLogsUseCase {
    pub fn new(repo: Arc<dyn EvaluationLogRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &ListEvaluationLogsInput,
    ) -> Result<ListEvaluationLogsOutput, ListEvaluationLogsError> {
        let (logs, total_count) = self
            .repo
            .find_all_paginated(
                input.page,
                input.page_size,
                input.rule_set_name.clone(),
                input.domain.clone(),
                input.from,
                input.to,
            )
            .await
            .map_err(|e| ListEvaluationLogsError::Internal(e.to_string()))?;

        let has_next = (input.page as u64 * input.page_size as u64) < total_count;

        Ok(ListEvaluationLogsOutput {
            logs,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
        })
    }
}
