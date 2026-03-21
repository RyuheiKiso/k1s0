use crate::domain::entity::inventory_item::{InventoryFilter, InventoryItem};
use crate::domain::entity::inventory_reservation::InventoryReservation;
use crate::domain::entity::outbox::OutboxEvent;
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InventoryRepository: Send + Sync {
    /// 在庫アイテムをIDで取得する。
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<InventoryItem>>;

    /// product_id と warehouse_id で在庫アイテムを取得する。
    async fn find_by_product_and_warehouse(
        &self,
        product_id: &str,
        warehouse_id: &str,
    ) -> anyhow::Result<Option<InventoryItem>>;

    /// フィルター条件で在庫一覧を取得する。
    async fn find_all(&self, filter: &InventoryFilter) -> anyhow::Result<Vec<InventoryItem>>;

    /// フィルター条件に一致する在庫件数を取得する。
    async fn count(&self, filter: &InventoryFilter) -> anyhow::Result<i64>;

    /// 在庫を予約する（楽観ロック付き）。
    async fn reserve_stock(
        &self,
        id: Uuid,
        quantity: i32,
        expected_version: i32,
        order_id: &str,
    ) -> anyhow::Result<InventoryItem>;

    /// 予約済み在庫を解放する（楽観ロック付き）。
    async fn release_stock(
        &self,
        id: Uuid,
        quantity: i32,
        expected_version: i32,
        order_id: &str,
        reason: &str,
    ) -> anyhow::Result<InventoryItem>;

    /// 在庫数量を更新する（楽観ロック付き）。
    async fn update_stock(
        &self,
        id: Uuid,
        qty_available: i32,
        expected_version: i32,
    ) -> anyhow::Result<InventoryItem>;

    /// 在庫アイテムを作成する（存在しない場合）。
    async fn create(
        &self,
        product_id: &str,
        warehouse_id: &str,
        qty_available: i32,
    ) -> anyhow::Result<InventoryItem>;

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

    /// 注文IDに紐づく予約中（status='reserved'）の在庫予約レコードを取得する。
    /// Saga 補償トランザクションで解放対象を特定するために使用する。
    async fn find_reservations_by_order_id(
        &self,
        order_id: &str,
    ) -> anyhow::Result<Vec<InventoryReservation>>;

    /// order_id に紐づく全在庫予約を解放する（Saga 補償トランザクション）。
    /// fetch + update + outbox_events INSERT を単一トランザクション内で実行する。
    async fn compensate_order_reservations(
        &self,
        order_id: &str,
        reason: &str,
    ) -> anyhow::Result<Vec<InventoryItem>>;
}
