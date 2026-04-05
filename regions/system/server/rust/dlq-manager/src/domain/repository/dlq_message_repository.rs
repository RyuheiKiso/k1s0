use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::DlqMessage;

/// DlqMessageRepository は DLQ メッセージ永続化のためのリポジトリトレイト。
/// CRIT-005 対応: 各メソッドに tenant_id を追加してテナント分離を実現する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DlqMessageRepository: Send + Sync {
    /// CRIT-005 対応: tenant_id を渡して RLS を有効にし ID で DLQ メッセージを検索する。
    async fn find_by_id(&self, id: Uuid, tenant_id: &str) -> anyhow::Result<Option<DlqMessage>>;

    /// CRIT-005 対応: tenant_id を渡して RLS を有効にしトピック別に DLQ メッセージ一覧を取得する（ページネーション付き）。
    async fn find_by_topic(
        &self,
        topic: &str,
        page: i32,
        page_size: i32,
        tenant_id: &str,
    ) -> anyhow::Result<(Vec<DlqMessage>, i64)>;

    /// DLQ メッセージを作成する。
    async fn create(&self, message: &DlqMessage) -> anyhow::Result<()>;

    /// DLQ メッセージを更新する。
    async fn update(&self, message: &DlqMessage) -> anyhow::Result<()>;

    /// CRIT-005 対応: tenant_id を渡して RLS を有効にし DLQ メッセージを削除する。
    async fn delete(&self, id: Uuid, tenant_id: &str) -> anyhow::Result<()>;

    /// CRIT-005 対応: tenant_id を渡して RLS を有効にしトピック別のメッセージ件数を取得する。
    #[allow(dead_code)]
    async fn count_by_topic(&self, topic: &str, tenant_id: &str) -> anyhow::Result<i64>;
}
