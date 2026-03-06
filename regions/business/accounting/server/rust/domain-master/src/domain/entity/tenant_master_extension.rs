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
