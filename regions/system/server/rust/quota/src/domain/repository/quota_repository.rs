use async_trait::async_trait;

use crate::domain::entity::quota::QuotaPolicy;

/// アトミック check-and-increment の結果。
#[derive(Debug, Clone, PartialEq)]
pub struct CheckAndIncrementResult {
    /// 操作後の使用量
    pub used: u64,
    /// リミット内で増分が許可されたか
    pub allowed: bool,
}

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

    /// リミットを超えない場合のみアトミックに増分する。
    /// 戻り値の `allowed` が false の場合、カウンターは変更されない。
    async fn check_and_increment(
        &self,
        quota_id: &str,
        amount: u64,
        limit: u64,
    ) -> anyhow::Result<CheckAndIncrementResult>;
}
