use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::service::{Service, ServiceLifecycle, ServiceTier};

/// `ServiceListFilters` はサービス一覧のフィルタ条件を表す。
#[derive(Debug, Clone, Default)]
pub struct ServiceListFilters {
    pub team_id: Option<Uuid>,
    pub tier: Option<ServiceTier>,
    pub lifecycle: Option<ServiceLifecycle>,
    pub tag: Option<String>,
}

/// `ServiceRepository` はサービス情報の永続化トレイト。
/// CRIT-004 監査対応: RLS テナント分離のため全メソッドに `tenant_id` を追加する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ServiceRepository: Send + Sync {
    /// テナントスコープでフィルタ条件を指定してサービス一覧を取得する。
    async fn list(
        &self,
        tenant_id: &str,
        filters: ServiceListFilters,
    ) -> anyhow::Result<Vec<Service>>;

    /// テナントスコープでサービス ID でサービスを取得する。
    async fn find_by_id(&self, tenant_id: &str, id: Uuid) -> anyhow::Result<Option<Service>>;

    /// テナントスコープで新規サービスを登録する。
    async fn create(&self, tenant_id: &str, service: &Service) -> anyhow::Result<Service>;

    /// テナントスコープでサービスを更新する。
    async fn update(&self, tenant_id: &str, service: &Service) -> anyhow::Result<Service>;

    /// テナントスコープでサービスを削除する。
    async fn delete(&self, tenant_id: &str, id: Uuid) -> anyhow::Result<()>;

    /// テナントスコープでクエリ文字列、タグ、ティアでサービスを検索する。
    async fn search(
        &self,
        tenant_id: &str,
        query: Option<String>,
        tags: Option<Vec<String>>,
        tier: Option<ServiceTier>,
    ) -> anyhow::Result<Vec<Service>>;
}
