use crate::domain::entity::outbox::OutboxEvent;
use crate::domain::entity::payment::{InitiatePayment, Payment, PaymentFilter};
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PaymentRepository: Send + Sync {
    /// 決済をIDで取得する。
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Payment>>;

    /// 注文IDで決済を取得する（冪等性チェック用）。
    async fn find_by_order_id(&self, order_id: &str) -> anyhow::Result<Option<Payment>>;

    /// フィルター条件で決済一覧を取得する。
    async fn find_all(&self, filter: &PaymentFilter) -> anyhow::Result<Vec<Payment>>;

    /// フィルター条件に一致する決済件数を取得する。
    async fn count(&self, filter: &PaymentFilter) -> anyhow::Result<i64>;

    /// 決済を開始する。
    async fn create(&self, input: &InitiatePayment) -> anyhow::Result<Payment>;

    /// 決済を完了する（楽観ロック付き）。
    async fn complete(
        &self,
        id: Uuid,
        transaction_id: &str,
        expected_version: i32,
    ) -> anyhow::Result<Payment>;

    /// 決済を失敗にする（楽観ロック付き）。
    async fn fail(
        &self,
        id: Uuid,
        error_code: &str,
        error_message: &str,
        expected_version: i32,
    ) -> anyhow::Result<Payment>;

    /// 決済を返金する（楽観ロック付き）。返金理由をOutboxイベントに記録する。
    async fn refund(
        &self,
        id: Uuid,
        expected_version: i32,
        reason: Option<String>,
    ) -> anyhow::Result<Payment>;

    /// Outbox イベントを挿入する。
    async fn insert_outbox_event(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        event_type: &str,
        payload: &serde_json::Value,
    ) -> anyhow::Result<()>;

    /// 未パブリッシュの Outbox イベントを取得する（mark は行わない）。
    /// FOR UPDATE SKIP LOCKED により並行ポーラー間の排他を保証する。
    /// at-least-once 配信のため、publish 成功後に mark_events_published を呼ぶこと。
    async fn fetch_unpublished_events(&self, limit: i64) -> anyhow::Result<Vec<OutboxEvent>>;

    /// 指定した ID のイベントをパブリッシュ済みとしてマークする。
    /// publish 成功後のみ呼び出すことで at-least-once セマンティクスを実現する。
    async fn mark_events_published(&self, ids: &[Uuid]) -> anyhow::Result<()>;
}
