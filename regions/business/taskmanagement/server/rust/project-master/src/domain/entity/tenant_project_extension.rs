// テナントプロジェクト拡張エンティティ。
// テナント毎のカスタマイズを表現する（会計の TenantMasterExtension に相当）。
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// テナントプロジェクト拡張
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantProjectExtension {
    pub id: Uuid,
    pub tenant_id: String,
    pub status_definition_id: Uuid,
    pub display_name_override: Option<String>,
    pub attributes_override: Option<serde_json::Value>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// テナント拡張 upsert DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertTenantExtension {
    pub tenant_id: String,
    pub status_definition_id: Uuid,
    pub display_name_override: Option<String>,
    pub attributes_override: Option<serde_json::Value>,
    pub is_enabled: Option<bool>,
}

/// テナントマージステータス（テナント拡張をマージしたビュー）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMergedStatus {
    pub base_status: StatusDefinition,
    pub extension: Option<TenantProjectExtension>,
    pub effective_display_name: String,
    pub effective_attributes: Option<serde_json::Value>,
}

use super::status_definition::StatusDefinition;
