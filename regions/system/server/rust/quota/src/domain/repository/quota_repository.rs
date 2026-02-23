use async_trait::async_trait;

use crate::domain::entity::quota::QuotaPolicy;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QuotaPolicyRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<QuotaPolicy>>;
    async fn find_all(&self, page: u32, page_size: u32) -> anyhow::Result<(Vec<QuotaPolicy>, u64)>;
    async fn create(&self, policy: &QuotaPolicy) -> anyhow::Result<()>;
    async fn update(&self, policy: &QuotaPolicy) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<bool>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QuotaUsageRepository: Send + Sync {
    async fn get_usage(&self, quota_id: &str) -> anyhow::Result<Option<u64>>;
    async fn increment(&self, quota_id: &str, amount: u64) -> anyhow::Result<u64>;
    async fn reset(&self, quota_id: &str) -> anyhow::Result<()>;
}
