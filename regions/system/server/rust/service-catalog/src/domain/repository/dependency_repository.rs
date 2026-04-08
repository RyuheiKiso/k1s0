use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::dependency::Dependency;

/// `DependencyRepository` はサービス間依存関係の永続化トレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DependencyRepository: Send + Sync {
    /// 指定サービスの依存関係一覧を取得する。
    async fn list_by_service(&self, service_id: Uuid) -> anyhow::Result<Vec<Dependency>>;

    /// 指定サービスの依存関係を一括設定する（既存を置換）。
    async fn set_dependencies(&self, service_id: Uuid, deps: Vec<Dependency>)
        -> anyhow::Result<()>;

    /// 全依存関係を取得する（サイクル検出用）。
    async fn get_all_dependencies(&self) -> anyhow::Result<Vec<Dependency>>;
}
