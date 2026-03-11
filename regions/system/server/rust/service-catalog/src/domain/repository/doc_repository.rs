use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::service_doc::ServiceDoc;

/// DocRepository はサービスドキュメントの永続化トレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DocRepository: Send + Sync {
    /// 指定サービスのドキュメント一覧を取得する。
    async fn list_by_service(&self, service_id: Uuid) -> anyhow::Result<Vec<ServiceDoc>>;

    /// 指定サービスのドキュメントを一括設定する（既存を置換）。
    async fn set_docs(&self, service_id: Uuid, docs: Vec<ServiceDoc>) -> anyhow::Result<()>;
}
