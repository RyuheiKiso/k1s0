use crate::domain::entity::inventory_item::InventoryItem;
use crate::domain::error::InventoryError;
use crate::domain::repository::inventory_repository::InventoryRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetInventoryUseCase {
    inventory_repo: Arc<dyn InventoryRepository>,
}

impl GetInventoryUseCase {
    pub fn new(inventory_repo: Arc<dyn InventoryRepository>) -> Self {
        Self { inventory_repo }
    }

    pub async fn execute(&self, inventory_id: Uuid) -> anyhow::Result<InventoryItem> {
        let item = self
            .inventory_repo
            .find_by_id(inventory_id)
            .await?
            .ok_or_else(|| InventoryError::NotFound(inventory_id.to_string()))?;

        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::inventory_repository::MockInventoryRepository;
    use chrono::Utc;

    #[tokio::test]
    async fn test_get_inventory_success() {
        let inventory_id = Uuid::new_v4();
        let item = InventoryItem {
            id: inventory_id,
            product_id: "PROD-001".to_string(),
            warehouse_id: "WH-EAST".to_string(),
            qty_available: 100,
            qty_reserved: 10,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let item_clone = item.clone();

        let mut mock_repo = MockInventoryRepository::new();
        mock_repo
            .expect_find_by_id()
            .withf(move |id| *id == inventory_id)
            .times(1)
            .returning(move |_| Ok(Some(item_clone.clone())));

        let uc = GetInventoryUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(inventory_id).await;
        assert!(result.is_ok());
        let found_item = result.unwrap();
        assert_eq!(found_item.id, inventory_id);
        assert_eq!(found_item.qty_available, 100);
    }

    #[tokio::test]
    async fn test_get_inventory_not_found() {
        let inventory_id = Uuid::new_v4();
        let mut mock_repo = MockInventoryRepository::new();
        mock_repo
            .expect_find_by_id()
            .times(1)
            .returning(|_| Ok(None));

        let uc = GetInventoryUseCase::new(Arc::new(mock_repo));
        let result = uc.execute(inventory_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
