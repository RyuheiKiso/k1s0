//! Presenter レイヤー — ドメインエンティティを API レスポンス形式に変換する。

use crate::domain::entity::inventory_item::InventoryItem;
use serde::Serialize;

/// 在庫詳細 API レスポンス。
#[derive(Debug, Serialize)]
pub struct InventoryDetailResponse {
    pub id: String,
    pub product_id: String,
    pub warehouse_id: String,
    pub qty_available: i32,
    pub qty_reserved: i32,
    pub version: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// 在庫一覧 API レスポンス。
#[derive(Debug, Serialize)]
pub struct InventoryListResponse {
    pub items: Vec<InventorySummaryResponse>,
    pub total: i64,
}

/// 在庫サマリ（一覧表示用）。
#[derive(Debug, Serialize)]
pub struct InventorySummaryResponse {
    pub id: String,
    pub product_id: String,
    pub warehouse_id: String,
    pub qty_available: i32,
    pub qty_reserved: i32,
    pub version: i32,
    pub updated_at: String,
}

impl InventoryDetailResponse {
    pub fn from_entity(item: &InventoryItem) -> Self {
        Self {
            id: item.id.to_string(),
            product_id: item.product_id.clone(),
            warehouse_id: item.warehouse_id.clone(),
            qty_available: item.qty_available,
            qty_reserved: item.qty_reserved,
            version: item.version,
            created_at: item.created_at.to_rfc3339(),
            updated_at: item.updated_at.to_rfc3339(),
        }
    }
}

impl From<&InventoryItem> for InventorySummaryResponse {
    fn from(item: &InventoryItem) -> Self {
        Self {
            id: item.id.to_string(),
            product_id: item.product_id.clone(),
            warehouse_id: item.warehouse_id.clone(),
            qty_available: item.qty_available,
            qty_reserved: item.qty_reserved,
            version: item.version,
            updated_at: item.updated_at.to_rfc3339(),
        }
    }
}

impl InventoryListResponse {
    pub fn from_entities(items: &[InventoryItem], total: i64) -> Self {
        Self {
            items: items.iter().map(InventorySummaryResponse::from).collect(),
            total,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

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

    #[test]
    fn test_inventory_detail_response() {
        let item = sample_item();
        let resp = InventoryDetailResponse::from_entity(&item);
        assert_eq!(resp.product_id, "PROD-001");
        assert_eq!(resp.qty_available, 100);
        assert_eq!(resp.qty_reserved, 10);
    }

    #[test]
    fn test_inventory_list_response() {
        let items = vec![sample_item()];
        let resp = InventoryListResponse::from_entities(&items, 1);
        assert_eq!(resp.total, 1);
        assert_eq!(resp.items.len(), 1);
    }

    #[test]
    fn test_inventory_summary_response() {
        let item = sample_item();
        let resp = InventorySummaryResponse::from(&item);
        assert_eq!(resp.warehouse_id, "WH-EAST");
        assert_eq!(resp.qty_available, 100);
    }
}
