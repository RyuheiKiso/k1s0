use crate::domain::entity::inventory_item::InventoryItem;
use crate::domain::error::InventoryError;
use crate::domain::repository::inventory_repository::InventoryRepository;
use crate::domain::service::inventory_service::InventoryDomainService;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateStockUseCase {
    inventory_repo: Arc<dyn InventoryRepository>,
}

impl UpdateStockUseCase {
    pub fn new(inventory_repo: Arc<dyn InventoryRepository>) -> Self {
        Self { inventory_repo }
    }

    pub async fn execute(
        &self,
        inventory_id: Uuid,
        qty_available: i32,
        expected_version: i32,
    ) -> anyhow::Result<InventoryItem> {
        // 存在確認
        let _existing = self
            .inventory_repo
            .find_by_id(inventory_id)
            .await?
            .ok_or_else(|| InventoryError::NotFound(inventory_id.to_string()))?;

        // ドメインバリデーション
        InventoryDomainService::validate_update_stock(qty_available)?;

        // 更新実行（楽観ロック付き）
        let updated = self
            .inventory_repo
            .update_stock(inventory_id, qty_available, expected_version)
            .await?;

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::inventory_repository::MockInventoryRepository;
    use chrono::Utc;

    fn sample_item() -> InventoryItem {
        InventoryItem {
            id: Uuid::new_v4(),
            product_id: "PROD-001".to_string(),
            warehouse_id: "WH-EAST".to_string(),
            qty_available: 100,
            qty_reserved: 10,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_update_stock_success() {
        let item = sample_item();
        let item_id = item.id;
        let item_clone = item.clone();
        let mut updated_item = item.clone();
        updated_item.qty_available = 200;
        updated_item.version = 2;
        let updated_clone = updated_item.clone();

        let mut mock_repo = MockInventoryRepository::new();

        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == item_id)
            .times(1)
            .returning(move |_| Ok(Some(item_clone.clone())));

        mock_repo
            .expect_update_stock()
            .times(1)
            .returning(move |_, _, _| Ok(updated_clone.clone()));

        let uc = UpdateStockUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(item_id, 200, 1).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().qty_available, 200);
    }

    #[tokio::test]
    async fn test_update_stock_not_found() {
        let item_id = Uuid::new_v4();
        let mut mock_repo = MockInventoryRepository::new();

        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let uc = UpdateStockUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(item_id, 200, 1).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_update_stock_negative_qty() {
        let item = sample_item();
        let item_id = item.id;
        let item_clone = item.clone();

        let mut mock_repo = MockInventoryRepository::new();

        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(move |_| Ok(Some(item_clone.clone())));

        let uc = UpdateStockUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(item_id, -10, 1).await;
        assert!(result.is_err());
    }
}
