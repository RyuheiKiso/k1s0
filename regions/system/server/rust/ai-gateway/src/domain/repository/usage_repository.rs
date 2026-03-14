// 使用量リポジトリのトレイト定義。
// API使用量レコードの保存と検索を抽象化する。

use async_trait::async_trait;

use crate::domain::entity::usage_record::UsageRecord;

/// 使用量リポジトリのインターフェース。
/// 使用量レコードの永続化とテナント別検索を提供する。
#[async_trait]
pub trait UsageRepository: Send + Sync {
    /// 使用量レコードを保存する。
    async fn save(&self, record: &UsageRecord) -> anyhow::Result<()>;

    /// 指定テナントの期間内使用量レコードを取得する。
    /// start, end はISO 8601形式の日時文字列。
    async fn find_by_tenant(
        &self,
        tenant_id: &str,
        start: &str,
        end: &str,
    ) -> Vec<UsageRecord>;
}
