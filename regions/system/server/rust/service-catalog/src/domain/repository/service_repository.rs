use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::service::{Service, ServiceLifecycle, ServiceTier};

/// ServiceListFilters はサービス一覧のフィルタ条件を表す。
#[derive(Debug, Clone, Default)]
pub struct ServiceListFilters {
    pub team_id: Option<Uuid>,
    pub tier: Option<ServiceTier>,
    pub lifecycle: Option<ServiceLifecycle>,
    pub tag: Option<String>,
}

/// ServiceRepository はサービス情報の永続化トレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ServiceRepository: Send + Sync {
    /// フィルタ条件を指定してサービス一覧を取得する。
    async fn list(&self, filters: ServiceListFilters) -> anyhow::Result<Vec<Service>>;

    /// サービス ID でサービスを取得する。
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Service>>;

    /// 新規サービスを登録する。
    async fn create(&self, service: &Service) -> anyhow::Result<Service>;

    /// サービスを更新する。
    async fn update(&self, service: &Service) -> anyhow::Result<Service>;

    /// サービスを削除する。
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;

    /// クエリ文字列、タグ、ティアでサービスを検索する。
    async fn search(
        &self,
        query: Option<String>,
        tags: Option<Vec<String>>,
        tier: Option<ServiceTier>,
    ) -> anyhow::Result<Vec<Service>>;
}
