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

/// CRITICAL-RUST-001 監査対応: 全メソッドに tenant_id を追加し、
/// PostgreSQL RLS の set_config('app.current_tenant_id', ...) を確実に呼び出す。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QuotaPolicyRepository: Send + Sync {
    async fn find_by_id(&self, id: &str, tenant_id: &str) -> anyhow::Result<Option<QuotaPolicy>>;
    async fn find_all(
        &self,
        page: u32,
        page_size: u32,
        tenant_id: &str,
    ) -> anyhow::Result<(Vec<QuotaPolicy>, u64)>;
    async fn create(&self, policy: &QuotaPolicy) -> anyhow::Result<()>;
    async fn update(&self, policy: &QuotaPolicy) -> anyhow::Result<()>;
    async fn delete(&self, id: &str, tenant_id: &str) -> anyhow::Result<bool>;
}

/// CRITICAL-RUST-001 監査対応: 全メソッドに tenant_id を追加し、
/// PostgreSQL RLS の set_config('app.current_tenant_id', ...) を確実に呼び出す。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait QuotaUsageRepository: Send + Sync {
    async fn get_usage(&self, quota_id: &str, tenant_id: &str) -> anyhow::Result<Option<u64>>;
    #[allow(dead_code)]
    async fn increment(&self, quota_id: &str, amount: u64, tenant_id: &str) -> anyhow::Result<u64>;
    async fn reset(&self, quota_id: &str, tenant_id: &str) -> anyhow::Result<()>;

    /// リミットを超えない場合のみアトミックに増分する。
    /// 戻り値の `allowed` が false の場合、カウンターは変更されない。
    async fn check_and_increment(
        &self,
        quota_id: &str,
        amount: u64,
        limit: u64,
        tenant_id: &str,
    ) -> anyhow::Result<CheckAndIncrementResult>;
}
