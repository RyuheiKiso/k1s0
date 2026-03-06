use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::rule::RuleSet;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait RuleSetRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<RuleSet>>;
    async fn find_all(&self) -> anyhow::Result<Vec<RuleSet>>;
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        domain: Option<String>,
    ) -> anyhow::Result<(Vec<RuleSet>, u64)>;
    async fn find_by_domain_and_name(
        &self,
        domain: &str,
        name: &str,
    ) -> anyhow::Result<Option<RuleSet>>;
    async fn create(&self, rule_set: &RuleSet) -> anyhow::Result<()>;
    async fn update(&self, rule_set: &RuleSet) -> anyhow::Result<()>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;
    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool>;
}
