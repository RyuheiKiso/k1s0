//! Presenter レイヤー — ドメインエンティティを API レスポンス形式に変換する。

use crate::domain::entity::master_category::MasterCategory;
use crate::domain::entity::master_item::MasterItem;
use crate::domain::entity::master_item_version::MasterItemVersion;
use crate::domain::entity::tenant_master_extension::{TenantMasterExtension, TenantMergedItem};
use serde::Serialize;

/// カテゴリ API レスポンス。
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: String,
    pub code: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_schema: Option<serde_json::Value>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// マスタ項目 API レスポンス。
#[derive(Debug, Serialize)]
pub struct ItemResponse {
    pub id: String,
    pub category_id: String,
    pub code: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_item_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_until: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

/// バージョン履歴 API レスポンス。
#[derive(Debug, Serialize)]
pub struct VersionResponse {
    pub id: String,
    pub item_id: String,
    pub version_number: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_data: Option<serde_json::Value>,
    pub changed_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_reason: Option<String>,
    pub created_at: String,
}

/// テナント拡張 API レスポンス。
#[derive(Debug, Serialize)]
pub struct TenantExtensionResponse {
    pub id: String,
    pub tenant_id: String,
    pub item_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name_override: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes_override: Option<serde_json::Value>,
    pub is_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// テナント別マージ済みアイテム API レスポンス。
#[derive(Debug, Serialize)]
pub struct MergedItemResponse {
    pub base_item: ItemResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<TenantExtensionResponse>,
    pub effective_display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_attributes: Option<serde_json::Value>,
}

// ── 変換実装 ──

impl From<&MasterCategory> for CategoryResponse {
    fn from(c: &MasterCategory) -> Self {
        Self {
            id: c.id.to_string(),
            code: c.code.clone(),
            display_name: c.display_name.clone(),
            description: c.description.clone(),
            validation_schema: c.validation_schema.clone(),
            is_active: c.is_active,
            sort_order: c.sort_order,
            created_by: c.created_by.clone(),
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        }
    }
}

impl From<&MasterItem> for ItemResponse {
    fn from(i: &MasterItem) -> Self {
        Self {
            id: i.id.to_string(),
            category_id: i.category_id.to_string(),
            code: i.code.clone(),
            display_name: i.display_name.clone(),
            description: i.description.clone(),
            attributes: i.attributes.clone(),
            parent_item_id: i.parent_item_id.map(|id| id.to_string()),
            effective_from: i.effective_from.map(|d| d.to_rfc3339()),
            effective_until: i.effective_until.map(|d| d.to_rfc3339()),
            is_active: i.is_active,
            sort_order: i.sort_order,
            created_by: i.created_by.clone(),
            created_at: i.created_at.to_rfc3339(),
            updated_at: i.updated_at.to_rfc3339(),
        }
    }
}

impl From<&MasterItemVersion> for VersionResponse {
    fn from(v: &MasterItemVersion) -> Self {
        Self {
            id: v.id.to_string(),
            item_id: v.item_id.to_string(),
            version_number: v.version_number,
            before_data: v.before_data.clone(),
            after_data: v.after_data.clone(),
            changed_by: v.changed_by.clone(),
            change_reason: v.change_reason.clone(),
            created_at: v.created_at.to_rfc3339(),
        }
    }
}

impl From<&TenantMasterExtension> for TenantExtensionResponse {
    fn from(e: &TenantMasterExtension) -> Self {
        Self {
            id: e.id.to_string(),
            tenant_id: e.tenant_id.clone(),
            item_id: e.item_id.to_string(),
            display_name_override: e.display_name_override.clone(),
            attributes_override: e.attributes_override.clone(),
            is_enabled: e.is_enabled,
            created_at: e.created_at.to_rfc3339(),
            updated_at: e.updated_at.to_rfc3339(),
        }
    }
}

impl From<&TenantMergedItem> for MergedItemResponse {
    fn from(m: &TenantMergedItem) -> Self {
        Self {
            base_item: ItemResponse::from(&m.base_item),
            extension: m.extension.as_ref().map(TenantExtensionResponse::from),
            effective_display_name: m.effective_display_name.clone(),
            effective_attributes: m.effective_attributes.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_category_response_from_entity() {
        let cat = MasterCategory {
            id: Uuid::new_v4(),
            code: "ACCT".to_string(),
            display_name: "Account Titles".to_string(),
            description: Some("勘定科目".to_string()),
            validation_schema: None,
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = CategoryResponse::from(&cat);
        assert_eq!(resp.code, "ACCT");
        assert_eq!(resp.display_name, "Account Titles");
        assert!(resp.is_active);
    }

    #[test]
    fn test_item_response_from_entity() {
        let item = MasterItem {
            id: Uuid::new_v4(),
            category_id: Uuid::new_v4(),
            code: "1100".to_string(),
            display_name: "現金".to_string(),
            description: None,
            attributes: Some(serde_json::json!({"account_type": "asset"})),
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: true,
            sort_order: 0,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = ItemResponse::from(&item);
        assert_eq!(resp.code, "1100");
        assert!(resp.parent_item_id.is_none());
    }

    #[test]
    fn test_version_response_from_entity() {
        let ver = MasterItemVersion {
            id: Uuid::new_v4(),
            item_id: Uuid::new_v4(),
            version_number: 1,
            before_data: None,
            after_data: Some(serde_json::json!({"code": "1100"})),
            changed_by: "admin".to_string(),
            change_reason: Some("Initial".to_string()),
            created_at: Utc::now(),
        };
        let resp = VersionResponse::from(&ver);
        assert_eq!(resp.version_number, 1);
        assert!(resp.before_data.is_none());
    }

    #[test]
    fn test_tenant_extension_response_from_entity() {
        let ext = TenantMasterExtension {
            id: Uuid::new_v4(),
            tenant_id: "t-001".to_string(),
            item_id: Uuid::new_v4(),
            display_name_override: Some("Custom".to_string()),
            attributes_override: None,
            is_enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resp = TenantExtensionResponse::from(&ext);
        assert_eq!(resp.tenant_id, "t-001");
        assert!(resp.is_enabled);
    }
}
