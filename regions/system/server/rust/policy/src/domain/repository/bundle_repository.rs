use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::policy_bundle::PolicyBundle;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PolicyBundleRepository: Send + Sync {
    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからバンドルを取得する。
    async fn find_by_id(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<Option<PolicyBundle>>;
    /// CRIT-005 対応: `tenant_id` を渡して全バンドルを取得する。
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<PolicyBundle>>;
    /// CRIT-005 対応: `bundle.tenant_id` で RLS 設定を行う。
    async fn create(&self, bundle: &PolicyBundle) -> anyhow::Result<()>;
    /// CRIT-005 対応: `tenant_id` を渡して RLS セッション変数を設定してからバンドルを削除する。
    #[allow(dead_code)]
    async fn delete(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<bool>;
}
