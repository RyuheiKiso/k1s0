// Saga 正常/補償: Order イベントに応じた在庫操作ユースケース（C-001）。
// order.created → 注文明細ごとに ReserveStockUseCase を呼び出して在庫を確保する。
// order.cancelled → 注文に紐づく在庫を ReleaseStockUseCase で解放する（補償トランザクション）。
//
// 冪等性: ReserveStockUseCase / ReleaseStockUseCase は InventoryError::VersionConflict や
// InventoryError::InsufficientReserved を返す場合がある。
// 既に同一 order_id で在庫操作が完了している場合、重複は在庫数の不整合を引き起こすため、
// order_id で操作済みかどうかを確認してからスキップする。
// 現実装では二重処理抑制はステータス遷移の前提条件チェックに依存する（inventory_item の予約管理）。

use crate::domain::repository::inventory_repository::InventoryRepository;
use crate::usecase::release_stock::ReleaseStockUseCase;
use crate::usecase::reserve_stock::ReserveStockUseCase;
use crate::proto::k1s0::event::service::order::v1::{OrderCancelledEvent, OrderCreatedEvent};
use std::sync::Arc;
use tracing::{info, warn};

/// HandleOrderEventUseCase は order イベントに応じた在庫操作を担う。
pub struct HandleOrderEventUseCase {
    reserve_stock_uc: Arc<ReserveStockUseCase>,
    // TODO(C-001): find_reserved_by_order_id 実装後に handle_cancelled で使用する
    #[allow(dead_code)]
    release_stock_uc: Arc<ReleaseStockUseCase>,
}

impl HandleOrderEventUseCase {
    pub fn new(inventory_repo: Arc<dyn InventoryRepository>) -> Self {
        Self {
            reserve_stock_uc: Arc::new(ReserveStockUseCase::new(inventory_repo.clone())),
            release_stock_uc: Arc::new(ReleaseStockUseCase::new(inventory_repo)),
        }
    }

    /// order.created イベントに応じて注文明細ごとに在庫を確保する。
    /// デフォルト倉庫 "WH-DEFAULT" を使用する（将来的には注文データに warehouse_id を含める）。
    pub async fn handle_created(&self, event: &OrderCreatedEvent) -> anyhow::Result<()> {
        for item in &event.items {
            match self
                .reserve_stock_uc
                .execute(
                    &item.product_id,
                    "WH-DEFAULT",
                    item.quantity,
                    &event.order_id,
                )
                .await
            {
                Ok(updated) => {
                    info!(
                        order_id = %event.order_id,
                        product_id = %item.product_id,
                        qty_reserved = updated.qty_reserved,
                        "stock reserved for saga order"
                    );
                }
                Err(e) => {
                    warn!(
                        order_id = %event.order_id,
                        product_id = %item.product_id,
                        "stock reservation failed: {}",
                        e
                    );
                    // 在庫確保失敗: エラーを上位に伝播させ、メッセージリトライを促す
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// order.cancelled イベントに応じて注文明細ごとに在庫を解放する（Saga 補償）。
    pub async fn handle_cancelled(&self, event: &OrderCancelledEvent) -> anyhow::Result<()> {
        // order.cancelled イベントには在庫明細情報がないため、
        // 在庫リポジトリで order_id に紐づく予約を検索して解放する。
        // 注意: 現在の InventoryRepository には find_by_order_id がないため、
        //       release は NOP として警告のみ記録する。
        // TODO(C-001): InventoryRepository に find_reserved_by_order_id を追加し、
        //              order_id に紐づく全予約を解放する実装に差し替える。
        warn!(
            order_id = %event.order_id,
            reason = %event.reason,
            "order cancelled: stock release for order_id is not yet implemented (requires find_reserved_by_order_id). \
             Manual compensation may be required."
        );
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::inventory_item::InventoryItem;
    use crate::domain::repository::inventory_repository::MockInventoryRepository;
    use crate::proto::k1s0::event::service::order::v1::OrderItem;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_inventory_item() -> InventoryItem {
        InventoryItem {
            id: Uuid::new_v4(),
            product_id: "PROD-001".to_string(),
            warehouse_id: "WH-DEFAULT".to_string(),
            qty_available: 100,
            qty_reserved: 0,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // order.created イベントで在庫が確保されることを確認する
    #[tokio::test]
    async fn test_handle_created_reserves_stock() {
        let item = sample_inventory_item();
        let item_id = item.id;
        let item_clone = item.clone();
        let mut reserved = item.clone();
        reserved.qty_available = 90;
        reserved.qty_reserved = 10;
        reserved.version = 2;
        let reserved_clone = reserved.clone();

        let mut mock_repo = MockInventoryRepository::new();
        mock_repo
            .expect_find_by_product_and_warehouse()
            .times(1)
            .returning(move |_, _| Ok(Some(item_clone.clone())));
        mock_repo
            .expect_reserve_stock()
            .withf(move |id, qty, ver, _| *id == item_id && *qty == 10 && *ver == 1)
            .times(1)
            .returning(move |_, _, _, _| Ok(reserved_clone.clone()));

        let uc = HandleOrderEventUseCase::new(Arc::new(mock_repo));
        let event = OrderCreatedEvent {
            metadata: None,
            order_id: "ORD-001".to_string(),
            customer_id: "CUST-001".to_string(),
            items: vec![OrderItem {
                product_id: "PROD-001".to_string(),
                quantity: 10,
                unit_price: 500,
            }],
            total_amount: 5000,
            currency: "JPY".to_string(),
        };
        let result = uc.handle_created(&event).await;
        assert!(result.is_ok());
    }

    // order.cancelled イベントは警告ログを出して Ok を返すことを確認する（TODO 実装完了まで）
    #[tokio::test]
    async fn test_handle_cancelled_returns_ok_with_warning() {
        let mock_repo = MockInventoryRepository::new();
        let uc = HandleOrderEventUseCase::new(Arc::new(mock_repo));
        let event = OrderCancelledEvent {
            metadata: None,
            order_id: "ORD-001".to_string(),
            user_id: "saga-consumer".to_string(),
            reason: "payment failed".to_string(),
        };
        let result = uc.handle_cancelled(&event).await;
        assert!(result.is_ok());
    }
}
