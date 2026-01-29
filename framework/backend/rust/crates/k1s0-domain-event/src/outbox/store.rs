use async_trait::async_trait;
use uuid::Uuid;

use super::entry::OutboxEntry;
use crate::error::OutboxError;

/// Outbox の永続化 trait。
///
/// データベース実装は `outbox` feature を有効にして提供する。
#[async_trait]
pub trait OutboxStore: Send + Sync {
    /// エントリを保存する。
    async fn insert(&self, entry: &OutboxEntry) -> Result<(), OutboxError>;

    /// 未処理エントリを最大 `limit` 件取得する。
    async fn fetch_pending(&self, limit: i64) -> Result<Vec<OutboxEntry>, OutboxError>;

    /// エントリを発行済みとしてマークする。
    async fn mark_published(&self, id: Uuid) -> Result<(), OutboxError>;

    /// エントリを失敗としてマークし、エラーメッセージを記録する。
    async fn mark_failed(&self, id: Uuid, error: &str) -> Result<(), OutboxError>;
}
