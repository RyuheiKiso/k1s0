use crate::domain::entity::inventory_item::InventoryItem;
use crate::domain::error::InventoryError;
use crate::domain::repository::inventory_repository::InventoryRepository;
use crate::domain::service::inventory_service::InventoryDomainService;
use std::sync::Arc;

pub struct ReserveStockUseCase {
    inventory_repo: Arc<dyn InventoryRepository>,
}

impl ReserveStockUseCase {
    pub fn new(inventory_repo: Arc<dyn InventoryRepository>) -> Self {
        Self { inventory_repo }
    }

    /// 最大リトライ回数（楽観ロック競合時）。高負荷環境でも十分な再試行回数を確保する（H-006）
    const MAX_RETRIES: u32 = 5;

    pub async fn execute(
        &self,
        product_id: &str,
        warehouse_id: &str,
        quantity: i32,
        order_id: &str,
    ) -> anyhow::Result<InventoryItem> {
        // 楽観ロック競合時にリトライする（指数バックオフ付き）
        for attempt in 0..Self::MAX_RETRIES {
            let item = self
                .inventory_repo
                .find_by_product_and_warehouse(product_id, warehouse_id)
                .await?
                .ok_or_else(|| {
                    InventoryError::NotFound(format!(
                        "product_id={}, warehouse_id={}",
                        product_id, warehouse_id
                    ))
                })?;

            // ドメインバリデーション
            InventoryDomainService::validate_reserve(&item, quantity)?;

            // 予約実行（Outbox イベントも同一トランザクション内で挿入される）
            match self
                .inventory_repo
                .reserve_stock(item.id, quantity, item.version, order_id)
                .await
            {
                Ok(updated) => return Ok(updated),
                Err(e) => {
                    let is_version_conflict = e
                        .downcast_ref::<InventoryError>()
                        .map(|ie| matches!(ie, InventoryError::VersionConflict(_)))
                        .unwrap_or(false);

                    if is_version_conflict && attempt < Self::MAX_RETRIES - 1 {
                        // 指数バックオフ + ジッター: Thundering Herd を防止する（H-006）
                        // ジッターにはシステム時刻の下位ビットを使用する（暗号学的乱数不要）
                        let base_ms = (attempt + 1) as u64 * (attempt + 1) as u64 * 10;
                        let jitter_range = (base_ms / 2).max(1);
                        let jitter_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.subsec_nanos() as u64 % jitter_range)
                            .unwrap_or(0);
                        let backoff_ms = base_ms + jitter_ms;
                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                        tracing::warn!(
                            attempt = attempt + 1,
                            max_retries = Self::MAX_RETRIES,
                            backoff_ms,
                            product_id,
                            warehouse_id,
                            "楽観ロック競合のためリトライします"
                        );
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        // ここには到達しない（最終試行では必ず return Err(e) が実行される）
        Err(anyhow::anyhow!(
            "reserve_stock: {} 回のリトライ後も成功しませんでした (product_id={}, warehouse_id={})",
            Self::MAX_RETRIES,
            product_id,
            warehouse_id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::inventory_repository::MockInventoryRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_item(qty_available: i32, qty_reserved: i32) -> InventoryItem {
        InventoryItem {
            id: Uuid::new_v4(),
            product_id: "PROD-001".to_string(),
            warehouse_id: "WH-EAST".to_string(),
            qty_available,
            qty_reserved,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_reserve_stock_success() {
        let item = sample_item(100, 0);
        let item_id = item.id;
        let item_clone = item.clone();
        let mut reserved_item = item.clone();
        reserved_item.qty_available = 90;
        reserved_item.qty_reserved = 10;
        reserved_item.version = 2;
        let reserved_clone = reserved_item.clone();

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

        let uc = ReserveStockUseCase::new(Arc::new(mock_repo));
        let result = uc.execute("PROD-001", "WH-EAST", 10, "ORD-001").await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.qty_available, 90);
        assert_eq!(updated.qty_reserved, 10);
    }

    #[tokio::test]
    async fn test_reserve_stock_insufficient() {
        let item = sample_item(5, 0);
        let item_clone = item.clone();

        let mut mock_repo = MockInventoryRepository::new();

        mock_repo
            .expect_find_by_product_and_warehouse()
            .times(1)
            .returning(move |_, _| Ok(Some(item_clone.clone())));

        let uc = ReserveStockUseCase::new(Arc::new(mock_repo));
        let result = uc.execute("PROD-001", "WH-EAST", 10, "ORD-001").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("insufficient stock"));
    }

    #[tokio::test]
    async fn test_reserve_stock_not_found() {
        let mut mock_repo = MockInventoryRepository::new();

        mock_repo
            .expect_find_by_product_and_warehouse()
            .times(1)
            .returning(|_, _| Ok(None));

        let uc = ReserveStockUseCase::new(Arc::new(mock_repo));
        let result = uc.execute("PROD-999", "WH-EAST", 10, "ORD-001").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
