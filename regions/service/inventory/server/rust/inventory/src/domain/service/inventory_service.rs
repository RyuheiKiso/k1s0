use crate::domain::entity::inventory_item::InventoryItem;
use crate::domain::error::InventoryError;

/// 在庫ドメインサービス — ドメインルールのバリデーションを担当。
pub struct InventoryDomainService;

impl InventoryDomainService {
    /// 在庫予約のバリデーション。
    pub fn validate_reserve(
        item: &InventoryItem,
        quantity: i32,
    ) -> Result<(), InventoryError> {
        if quantity <= 0 {
            return Err(InventoryError::ValidationFailed(
                "quantity must be greater than zero".to_string(),
            ));
        }
        if item.qty_available < quantity {
            return Err(InventoryError::InsufficientStock {
                available: item.qty_available,
                requested: quantity,
            });
        }
        Ok(())
    }

    /// 在庫解放のバリデーション。
    pub fn validate_release(
        item: &InventoryItem,
        quantity: i32,
    ) -> Result<(), InventoryError> {
        if quantity <= 0 {
            return Err(InventoryError::ValidationFailed(
                "quantity must be greater than zero".to_string(),
            ));
        }
        if item.qty_reserved < quantity {
            return Err(InventoryError::InsufficientReserved {
                reserved: item.qty_reserved,
                requested: quantity,
            });
        }
        Ok(())
    }

    /// 在庫更新のバリデーション。
    pub fn validate_update_stock(qty_available: i32) -> Result<(), InventoryError> {
        if qty_available < 0 {
            return Err(InventoryError::ValidationFailed(
                "qty_available must not be negative".to_string(),
            ));
        }
        Ok(())
    }

    /// 在庫作成の入力バリデーション。
    pub fn validate_create(
        product_id: &str,
        warehouse_id: &str,
        qty_available: i32,
    ) -> Result<(), InventoryError> {
        if product_id.trim().is_empty() {
            return Err(InventoryError::ValidationFailed(
                "product_id must not be empty".to_string(),
            ));
        }
        if warehouse_id.trim().is_empty() {
            return Err(InventoryError::ValidationFailed(
                "warehouse_id must not be empty".to_string(),
            ));
        }
        if qty_available < 0 {
            return Err(InventoryError::ValidationFailed(
                "qty_available must not be negative".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_validate_reserve_success() {
        let item = sample_item(100, 10);
        assert!(InventoryDomainService::validate_reserve(&item, 50).is_ok());
    }

    #[test]
    fn test_validate_reserve_insufficient() {
        let item = sample_item(10, 0);
        let result = InventoryDomainService::validate_reserve(&item, 20);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("insufficient stock"));
    }

    #[test]
    fn test_validate_reserve_zero_quantity() {
        let item = sample_item(100, 0);
        let result = InventoryDomainService::validate_reserve(&item, 0);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("quantity must be greater than zero"));
    }

    #[test]
    fn test_validate_release_success() {
        let item = sample_item(90, 10);
        assert!(InventoryDomainService::validate_release(&item, 5).is_ok());
    }

    #[test]
    fn test_validate_release_insufficient() {
        let item = sample_item(90, 5);
        let result = InventoryDomainService::validate_release(&item, 10);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("insufficient reserved"));
    }

    #[test]
    fn test_validate_release_zero_quantity() {
        let item = sample_item(90, 10);
        let result = InventoryDomainService::validate_release(&item, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_update_stock_success() {
        assert!(InventoryDomainService::validate_update_stock(100).is_ok());
        assert!(InventoryDomainService::validate_update_stock(0).is_ok());
    }

    #[test]
    fn test_validate_update_stock_negative() {
        let result = InventoryDomainService::validate_update_stock(-1);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_create_success() {
        assert!(
            InventoryDomainService::validate_create("PROD-001", "WH-EAST", 100).is_ok()
        );
    }

    #[test]
    fn test_validate_create_empty_product_id() {
        let result = InventoryDomainService::validate_create("", "WH-EAST", 100);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("product_id"));
    }

    #[test]
    fn test_validate_create_empty_warehouse_id() {
        let result = InventoryDomainService::validate_create("PROD-001", "", 100);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("warehouse_id"));
    }

    #[test]
    fn test_validate_create_negative_qty() {
        let result = InventoryDomainService::validate_create("PROD-001", "WH-EAST", -1);
        assert!(result.is_err());
    }
}
