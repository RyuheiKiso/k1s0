use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::access_log::SecretAccessLog;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AccessLogRepository: Send + Sync {
    async fn record(&self, log: &SecretAccessLog) -> anyhow::Result<()>;
    /// LOW-12 監査対応: keyset ページネーション。OFFSET による全行スキャンを回避し、
    /// 大量の監査ログでも一定のクエリ性能を保証する。
    /// - after_id=None のとき最初のページ（created_at 降順の先頭）を返す。
    /// - after_id=Some(id) のとき、その id のレコードより古いレコードを返す。
    /// 戻り値はログリストと次ページカーソル（次ページが存在する場合は最後のアイテムの id、ない場合は None）。
    async fn list(
        &self,
        after_id: Option<Uuid>,
        limit: u32,
    ) -> anyhow::Result<(Vec<SecretAccessLog>, Option<Uuid>)>;
}
