use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterItem {
    pub id: Uuid,
    pub category_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub parent_item_id: Option<Uuid>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMasterItem {
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub parent_item_id: Option<Uuid>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMasterItem {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub parent_item_id: Option<Uuid>,
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_item() -> MasterItem {
        MasterItem {
            id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            code: "JPY".to_string(),
            display_name: "Japanese Yen".to_string(),
            description: Some("Japanese Yen currency".to_string()),
            attributes: Some(serde_json::json!({"symbol": "¥", "decimals": 0})),
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_master_item_construction() {
        let item = sample_item();
        assert_eq!(item.code, "JPY");
        assert_eq!(item.display_name, "Japanese Yen");
        assert!(item.is_active);
        assert!(item.parent_item_id.is_none());
        assert!(item.attributes.is_some());
    }

    #[test]
    fn test_master_item_serialization_roundtrip() {
        let item = sample_item();
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: MasterItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, item.code);
        assert_eq!(deserialized.display_name, item.display_name);
        assert_eq!(deserialized.category_id, item.category_id);
    }

    #[test]
    fn test_create_master_item_dto() {
        let dto = CreateMasterItem {
            code: "USD".to_string(),
            display_name: "US Dollar".to_string(),
            description: Some("United States Dollar".to_string()),
            attributes: Some(serde_json::json!({"symbol": "$"})),
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: Some(true),
            sort_order: Some(2),
        };
        assert_eq!(dto.code, "USD");
        assert!(dto.attributes.is_some());
    }

    #[test]
    fn test_create_master_item_dto_minimal() {
        let dto = CreateMasterItem {
            code: "EUR".to_string(),
            display_name: "Euro".to_string(),
            description: None,
            attributes: None,
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: None,
            sort_order: None,
        };
        assert!(dto.attributes.is_none());
        assert!(dto.is_active.is_none());
    }

    #[test]
    fn test_update_master_item_dto() {
        let dto = UpdateMasterItem {
            display_name: Some("Updated Dollar".to_string()),
            description: None,
            attributes: Some(serde_json::json!({"symbol": "$", "decimals": 2})),
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: Some(false),
            sort_order: None,
        };
        assert_eq!(dto.display_name.as_deref(), Some("Updated Dollar"));
        assert_eq!(dto.is_active, Some(false));
    }
}
