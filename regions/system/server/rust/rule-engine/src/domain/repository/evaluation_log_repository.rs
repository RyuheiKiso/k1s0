use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::entity::rule::EvaluationLog;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait EvaluationLogRepository: Send + Sync {
    async fn create(&self, log: &EvaluationLog) -> anyhow::Result<()>;
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        rule_set_name: Option<String>,
        domain: Option<String>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> anyhow::Result<(Vec<EvaluationLog>, u64)>;
}
