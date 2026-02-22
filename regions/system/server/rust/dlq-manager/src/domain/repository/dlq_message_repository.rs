use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::DlqMessage;

/// DlqMessageRepository は DLQ メッセージ永続化のためのリポジトリトレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DlqMessageRepository: Send + Sync {
    /// IDで DLQ メッセージを検索する。
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<DlqMessage>>;

    /// トピック別に DLQ メッセージ一覧を取得する（ページネーション付き）。
    async fn find_by_topic(
        &self,
        topic: &str,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<(Vec<DlqMessage>, i64)>;

    /// DLQ メッセージを作成する。
    async fn create(&self, message: &DlqMessage) -> anyhow::Result<()>;

    /// DLQ メッセージを更新する。
    async fn update(&self, message: &DlqMessage) -> anyhow::Result<()>;

    /// DLQ メッセージを削除する。
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;

    /// トピック別のメッセージ件数を取得する。
    #[allow(dead_code)]
    async fn count_by_topic(&self, topic: &str) -> anyhow::Result<i64>;
}
