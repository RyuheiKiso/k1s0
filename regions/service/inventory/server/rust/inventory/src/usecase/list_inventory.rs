use crate::domain::entity::inventory_item::{InventoryFilter, InventoryItem};
use crate::domain::repository::inventory_repository::InventoryRepository;
use std::sync::Arc;

pub struct ListInventoryUseCase {
    inventory_repo: Arc<dyn InventoryRepository>,
}

impl ListInventoryUseCase {
    pub fn new(inventory_repo: Arc<dyn InventoryRepository>) -> Self {
        Self { inventory_repo }
    }

    pub async fn execute(
        &self,
        filter: &InventoryFilter,
    ) -> anyhow::Result<(Vec<InventoryItem>, i64)> {
        let items = self.inventory_repo.find_all(filter).await?;
        let total = self.inventory_repo.count(filter).await?;
        Ok((items, total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::inventory_repository::MockInventoryRepository;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_list_inventory_success() {
        let items = vec![InventoryItem {
            id: Uuid::new_v4(),
            product_id: "PROD-001".to_string(),
            warehouse_id: "WH-EAST".to_string(),
            qty_available: 100,
            qty_reserved: 10,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];
        let items_clone = items.clone();

        let mut mock_repo = MockInventoryRepository::new();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok(items_clone.clone()));
        mock_repo.expect_count().times(1).returning(|_| Ok(1));

        let uc = ListInventoryUseCase::new(Arc::new(mock_repo));
        let filter = InventoryFilter::default();
        let result = uc.execute(&filter).await;
        assert!(result.is_ok());
        let (found_items, total) = result.unwrap();
        assert_eq!(found_items.len(), 1);
        assert_eq!(total, 1);
    }

    #[tokio::test]
    async fn test_list_inventory_empty() {
        let mut mock_repo = MockInventoryRepository::new();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(|_| Ok(vec![]));
        mock_repo.expect_count().times(1).returning(|_| Ok(0));

        let uc = ListInventoryUseCase::new(Arc::new(mock_repo));
        let filter = InventoryFilter::default();
        let result = uc.execute(&filter).await;
        assert!(result.is_ok());
        let (found_items, total) = result.unwrap();
        assert_eq!(found_items.len(), 0);
        assert_eq!(total, 0);
    }
}
