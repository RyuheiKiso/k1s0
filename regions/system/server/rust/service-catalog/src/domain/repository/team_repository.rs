use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::team::Team;

/// TeamRepository はチーム情報の永続化トレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TeamRepository: Send + Sync {
    /// チーム一覧を取得する。
    async fn list(&self) -> anyhow::Result<Vec<Team>>;

    /// チーム ID でチームを取得する。
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Team>>;

    /// チームを作成する。
    async fn create(&self, team: &Team) -> anyhow::Result<Team>;

    /// チームを更新する。
    async fn update(&self, team: &Team) -> anyhow::Result<Team>;

    /// チームを削除する。
    async fn delete(&self, id: Uuid) -> anyhow::Result<bool>;
}
