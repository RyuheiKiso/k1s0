use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterCategory {
    pub id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub validation_schema: Option<serde_json::Value>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMasterCategory {
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub validation_schema: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMasterCategory {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub validation_schema: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn sample_category() -> MasterCategory {
        MasterCategory {
            id: Uuid::new_v4(),
            code: "CURRENCY".to_string(),
            display_name: "Currency".to_string(),
            description: Some("Currency master".to_string()),
            validation_schema: Some(serde_json::json!({
                "required": ["symbol"],
                "properties": {
                    "symbol": { "type": "string", "maxLength": 3 }
                }
            })),
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_master_category_construction() {
        let cat = sample_category();
        assert_eq!(cat.code, "CURRENCY");
        assert_eq!(cat.display_name, "Currency");
        assert!(cat.is_active);
        assert_eq!(cat.sort_order, 1);
        assert!(cat.description.is_some());
        assert!(cat.validation_schema.is_some());
    }

    #[test]
    fn test_master_category_serialization_roundtrip() {
        let cat = sample_category();
        let json = serde_json::to_string(&cat).unwrap();
        let deserialized: MasterCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.code, cat.code);
        assert_eq!(deserialized.display_name, cat.display_name);
    }

    #[test]
    fn test_create_master_category_dto() {
        let dto = CreateMasterCategory {
            code: "TAX_RATE".to_string(),
            display_name: "Tax Rate".to_string(),
            description: None,
            validation_schema: None,
            is_active: Some(true),
            sort_order: Some(10),
        };
        assert_eq!(dto.code, "TAX_RATE");
        assert_eq!(dto.is_active, Some(true));
        assert_eq!(dto.sort_order, Some(10));
    }

    #[test]
    fn test_create_master_category_dto_with_defaults() {
        let dto = CreateMasterCategory {
            code: "UNIT".to_string(),
            display_name: "Unit".to_string(),
            description: None,
            validation_schema: None,
            is_active: None,
            sort_order: None,
        };
        assert!(dto.is_active.is_none());
        assert!(dto.sort_order.is_none());
    }

    #[test]
    fn test_update_master_category_dto() {
        let dto = UpdateMasterCategory {
            display_name: Some("Updated Name".to_string()),
            description: Some("Updated desc".to_string()),
            validation_schema: None,
            is_active: Some(false),
            sort_order: Some(99),
        };
        assert_eq!(dto.display_name.as_deref(), Some("Updated Name"));
        assert_eq!(dto.is_active, Some(false));
    }

    #[test]
    fn test_update_master_category_dto_partial() {
        let dto = UpdateMasterCategory {
            display_name: None,
            description: None,
            validation_schema: None,
            is_active: None,
            sort_order: None,
        };
        assert!(dto.display_name.is_none());
        assert!(dto.is_active.is_none());
    }
}
