use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 在庫アイテムエンティティ。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: Uuid,
    pub product_id: String,
    pub warehouse_id: String,
    pub qty_available: i32,
    pub qty_reserved: i32,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 在庫一覧フィルター。
#[derive(Debug, Clone, Default)]
pub struct InventoryFilter {
    pub product_id: Option<String>,
    pub warehouse_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_item_creation() {
        let item = InventoryItem {
            id: Uuid::new_v4(),
            product_id: "PROD-001".to_string(),
            warehouse_id: "WH-EAST".to_string(),
            qty_available: 100,
            qty_reserved: 10,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(item.qty_available, 100);
        assert_eq!(item.qty_reserved, 10);
    }

    #[test]
    fn test_inventory_filter_default() {
        let filter = InventoryFilter::default();
        assert!(filter.product_id.is_none());
        assert!(filter.warehouse_id.is_none());
        assert!(filter.limit.is_none());
        assert!(filter.offset.is_none());
    }

    #[test]
    fn test_inventory_item_serialization_roundtrip() {
        let item = InventoryItem {
            id: Uuid::new_v4(),
            product_id: "PROD-001".to_string(),
            warehouse_id: "WH-EAST".to_string(),
            qty_available: 50,
            qty_reserved: 5,
            version: 2,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: InventoryItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.product_id, "PROD-001");
        assert_eq!(deserialized.qty_available, 50);
    }
}
