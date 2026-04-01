use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::feature_flag::FeatureFlag;

/// FeatureFlagRepository はフィーチャーフラグのリポジトリインターフェース。
/// STATIC-CRITICAL-001 監査対応: 全メソッドに tenant_id パラメータを追加し、
/// テナント間のデータ分離を API 契約レベルで強制する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FeatureFlagRepository: Send + Sync {
    async fn find_by_key(&self, tenant_id: Uuid, flag_key: &str) -> anyhow::Result<FeatureFlag>;
    async fn find_all(&self, tenant_id: Uuid) -> anyhow::Result<Vec<FeatureFlag>>;
    async fn create(&self, tenant_id: Uuid, flag: &FeatureFlag) -> anyhow::Result<()>;
    async fn update(&self, tenant_id: Uuid, flag: &FeatureFlag) -> anyhow::Result<()>;
    async fn delete(&self, tenant_id: Uuid, id: &Uuid) -> anyhow::Result<bool>;
    async fn exists_by_key(&self, tenant_id: Uuid, flag_key: &str) -> anyhow::Result<bool>;
}
