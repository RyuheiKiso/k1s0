use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::rule::Rule;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RuleRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Rule>>;
    async fn find_all(&self) -> anyhow::Result<Vec<Rule>>;
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        rule_set_id: Option<Uuid>,
        domain: Option<String>,
    ) -> anyhow::Result<(Vec<Rule>, u64)>;
    async fn create(&self, rule: &Rule) -> anyhow::Result<()>;
    async fn update(&self, rule: &Rule) -> anyhow::Result<()>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;
    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool>;
    async fn find_by_ids(&self, ids: &[Uuid]) -> anyhow::Result<Vec<Rule>>;
}
