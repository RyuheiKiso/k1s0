use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::policy::Policy;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PolicyRepository: Send + Sync {
    /// CRIT-005 対応: tenant_id を渡して RLS セッション変数を設定してからポリシーを取得する。
    async fn find_by_id(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<Option<Policy>>;
    /// CRIT-005 対応: tenant_id を渡して RLS セッション変数を設定してから全ポリシーを取得する。
    #[allow(dead_code)]
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<Policy>>;
    /// CRIT-005 対応: tenant_id を渡してページネーション付きでポリシーを取得する。
    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        bundle_id: Option<Uuid>,
        enabled_only: bool,
        tenant_id: &str,
    ) -> anyhow::Result<(Vec<Policy>, u64)>;
    /// CRIT-005 対応: policy.tenant_id で RLS 設定を行う。
    async fn create(&self, policy: &Policy) -> anyhow::Result<()>;
    /// CRIT-005 対応: policy.tenant_id で RLS 設定を行う。
    async fn update(&self, policy: &Policy) -> anyhow::Result<()>;
    /// CRIT-005 対応: tenant_id を渡して RLS セッション変数を設定してからポリシーを削除する。
    async fn delete(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<bool>;
    /// CRIT-005 対応: tenant_id を渡して名前で存在確認を行う。
    async fn exists_by_name(&self, name: &str, tenant_id: &str) -> anyhow::Result<bool>;
}
