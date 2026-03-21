// Saga 正常/補償: Order イベントに応じた在庫操作ユースケース（C-003）。
// order.created → 注文明細ごとに ReserveStockUseCase を呼び出して在庫を確保する。
// order.cancelled → compensate_order_reservations を呼び出して注文に紐づく全予約を解放する（補償トランザクション）。
//
// 冪等性: compensate_order_reservations は inventory_reservations テーブルを参照するため、
// 既に解放済みの予約（status='released'）は対象外となり、二重解放が発生しない。

use crate::domain::repository::inventory_repository::InventoryRepository;
use crate::usecase::reserve_stock::ReserveStockUseCase;
use crate::proto::k1s0::event::service::order::v1::{OrderCancelledEvent, OrderCreatedEvent};
use std::sync::Arc;
use tracing::{info, warn};

/// HandleOrderEventUseCase は order イベントに応じた在庫操作を担う。
pub struct HandleOrderEventUseCase {
    reserve_stock_uc: Arc<ReserveStockUseCase>,
    /// Saga 補償トランザクションで order_id に紐づく全予約を解放するために使用する
    inventory_repo: Arc<dyn InventoryRepository>,
}

impl HandleOrderEventUseCase {
    pub fn new(inventory_repo: Arc<dyn InventoryRepository>) -> Self {
        Self {
            reserve_stock_uc: Arc::new(ReserveStockUseCase::new(inventory_repo.clone())),
            inventory_repo,
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

    /// order.cancelled イベントに応じて注文に紐づく全予約を解放する（Saga 補償トランザクション）。
    /// compensate_order_reservations を呼び出し、単一トランザクション内で
    /// 予約レコードの更新・在庫数量の復元・Outbox イベント挿入を一括実行する。
    pub async fn handle_cancelled(&self, event: &OrderCancelledEvent) -> anyhow::Result<()> {
        let released_items = self
            .inventory_repo
            .compensate_order_reservations(&event.order_id, &event.reason)
            .await?;

        if released_items.is_empty() {
            info!(
                order_id = %event.order_id,
                "no active reservations found for order, saga compensation is idempotent"
            );
        } else {
            info!(
                order_id = %event.order_id,
                released_count = released_items.len(),
                "saga compensation completed: stock reservations released"
            );
        }
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

    // order.cancelled イベントで予約が存在しない場合は冪等に Ok を返すことを確認する
    #[tokio::test]
    async fn test_handle_cancelled_no_reservations_is_idempotent() {
        let mut mock_repo = MockInventoryRepository::new();
        // 予約が存在しないケース: 空リストを返す
        mock_repo
            .expect_compensate_order_reservations()
            .times(1)
            .returning(|_, _| Ok(vec![]));

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

    // order.cancelled イベントで予約が存在する場合は補償処理が実行されることを確認する
    #[tokio::test]
    async fn test_handle_cancelled_releases_reservations() {
        let released_item = sample_inventory_item();
        let released_clone = released_item.clone();

        let mut mock_repo = MockInventoryRepository::new();
        // 1件の予約が解放されるケース
        mock_repo
            .expect_compensate_order_reservations()
            .times(1)
            .returning(move |_, _| Ok(vec![released_clone.clone()]));

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
