use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::scorecard::Scorecard;

/// `ScorecardRepository` はサービススコアカードの永続化トレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ScorecardRepository: Send + Sync {
    /// 指定サービスのスコアカードを取得する。
    async fn get(&self, service_id: Uuid) -> anyhow::Result<Option<Scorecard>>;

    /// スコアカードを upsert する。
    #[allow(dead_code)]
    async fn upsert(&self, scorecard: &Scorecard) -> anyhow::Result<()>;
}
