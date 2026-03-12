use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::health::HealthStatus;

/// HealthRepository はサービスヘルスステータスの永続化トレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HealthRepository: Send + Sync {
    /// 指定サービスの最新ヘルスステータスを取得する。
    async fn get_latest(&self, service_id: Uuid) -> anyhow::Result<Option<HealthStatus>>;

    /// ヘルスステータスを upsert する。
    async fn upsert(&self, health: &HealthStatus) -> anyhow::Result<()>;

    /// 全サービスの最新ヘルスステータス一覧を取得する。
    async fn list_all_latest(&self) -> anyhow::Result<Vec<HealthStatus>>;
}
