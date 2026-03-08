use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::policy::Policy;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PolicyRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Policy>>;
    #[allow(dead_code)]
    async fn find_all(&self) -> anyhow::Result<Vec<Policy>>;
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        bundle_id: Option<Uuid>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<Policy>, u64)>;
    async fn create(&self, policy: &Policy) -> anyhow::Result<()>;
    async fn update(&self, policy: &Policy) -> anyhow::Result<()>;
    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool>;
    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool>;
}
