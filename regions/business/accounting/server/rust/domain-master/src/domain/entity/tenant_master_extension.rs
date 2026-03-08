use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMasterExtension {
    pub id: Uuid,
    pub tenant_id: String,
    pub item_id: Uuid,
    pub display_name_override: Option<String>,
    pub attributes_override: Option<serde_json::Value>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertTenantMasterExtension {
    pub display_name_override: Option<String>,
    pub attributes_override: Option<serde_json::Value>,
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMergedItem {
    pub base_item: super::master_item::MasterItem,
    pub extension: Option<TenantMasterExtension>,
    pub effective_display_name: String,
    pub effective_attributes: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_tenant_master_extension_construction() {
        let ext = TenantMasterExtension {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            item_id: Uuid::new_v4(),
            display_name_override: Some("Custom Name".to_string()),
            attributes_override: Some(serde_json::json!({"custom_field": "value"})),
            is_enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(ext.tenant_id, "tenant-1");
        assert!(ext.is_enabled);
        assert!(ext.display_name_override.is_some());
    }

    #[test]
    fn test_tenant_master_extension_serialization_roundtrip() {
        let ext = TenantMasterExtension {
            id: Uuid::new_v4(),
            tenant_id: "tenant-2".to_string(),
            item_id: Uuid::new_v4(),
            display_name_override: None,
            attributes_override: None,
            is_enabled: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&ext).unwrap();
        let deserialized: TenantMasterExtension = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.tenant_id, "tenant-2");
        assert!(!deserialized.is_enabled);
    }

    #[test]
    fn test_upsert_tenant_master_extension_dto() {
        let dto = UpsertTenantMasterExtension {
            display_name_override: Some("Override Name".to_string()),
            attributes_override: Some(serde_json::json!({"key": "val"})),
            is_enabled: Some(true),
        };
        assert_eq!(dto.display_name_override.as_deref(), Some("Override Name"));
        assert_eq!(dto.is_enabled, Some(true));
    }

    #[test]
    fn test_upsert_tenant_master_extension_dto_minimal() {
        let dto = UpsertTenantMasterExtension {
            display_name_override: None,
            attributes_override: None,
            is_enabled: None,
        };
        assert!(dto.display_name_override.is_none());
        assert!(dto.is_enabled.is_none());
    }
}
