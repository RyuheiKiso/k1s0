use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::feature_flag::FeatureFlag;

/// FeatureFlagRepository はフィーチャーフラグのリポジトリインターフェース。
/// STATIC-CRITICAL-001 監査対応: 全メソッドに tenant_id パラメータを追加し、
/// テナント間のデータ分離を API 契約レベルで強制する。
/// HIGH-005 対応: tenant_id は &str 型（migration 006 で DB の TEXT 型に変更済み）。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait FeatureFlagRepository: Send + Sync {
    async fn find_by_key(&self, tenant_id: &str, flag_key: &str) -> anyhow::Result<FeatureFlag>;
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<FeatureFlag>>;
    async fn create(&self, tenant_id: &str, flag: &FeatureFlag) -> anyhow::Result<()>;
    async fn update(&self, tenant_id: &str, flag: &FeatureFlag) -> anyhow::Result<()>;
    async fn delete(&self, tenant_id: &str, id: &Uuid) -> anyhow::Result<bool>;
    async fn exists_by_key(&self, tenant_id: &str, flag_key: &str) -> anyhow::Result<bool>;
}
