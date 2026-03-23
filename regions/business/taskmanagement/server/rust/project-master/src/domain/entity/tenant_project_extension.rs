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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    // TenantProjectExtensionエンティティが正しくフィールドを保持することを確認する
    // 前提: テナントIDとステータス定義IDを持つ拡張エントリを表現する
    // 期待: 各フィールドが正確に格納されている
    #[test]
    fn test_tenant_project_extension_fields() {
        let id = Uuid::new_v4();
        let sd_id = Uuid::new_v4();
        let now = Utc::now();
        let ext = TenantProjectExtension {
            id,
            tenant_id: "tenant-001".to_string(),
            status_definition_id: sd_id,
            display_name_override: Some("カスタム表示名".to_string()),
            attributes_override: None,
            is_enabled: true,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(ext.id, id);
        assert_eq!(ext.tenant_id, "tenant-001");
        assert_eq!(ext.status_definition_id, sd_id);
        assert!(ext.display_name_override.is_some());
        assert!(ext.is_enabled);
    }

    // TenantProjectExtensionが無効化されている場合も正しく保持されることを確認する
    // 前提: is_enabled=false の拡張を表現する
    // 期待: is_enabled が false となる
    #[test]
    fn test_tenant_project_extension_disabled() {
        let now = Utc::now();
        let ext = TenantProjectExtension {
            id: Uuid::new_v4(),
            tenant_id: "tenant-002".to_string(),
            status_definition_id: Uuid::new_v4(),
            display_name_override: None,
            attributes_override: None,
            is_enabled: false,
            created_at: now,
            updated_at: now,
        };
        assert!(!ext.is_enabled);
        assert!(ext.display_name_override.is_none());
    }

    // UpsertTenantExtension DTOが正しく構築されることを確認する
    // 前提: is_enabled を省略する
    // 期待: is_enabled が None となる
    #[test]
    fn test_upsert_tenant_extension_optional_fields() {
        let sd_id = Uuid::new_v4();
        let input = UpsertTenantExtension {
            tenant_id: "tenant-003".to_string(),
            status_definition_id: sd_id,
            display_name_override: Some("上書き名".to_string()),
            attributes_override: None,
            is_enabled: None,
        };
        assert_eq!(input.tenant_id, "tenant-003");
        assert_eq!(input.status_definition_id, sd_id);
        assert!(input.is_enabled.is_none());
    }

    // TenantMergedStatusが有効な表示名を保持することを確認する
    // 前提: ベースステータスと拡張を組み合わせたビューを表現する
    // 期待: effective_display_name が正確に設定されている
    #[test]
    fn test_tenant_merged_status_effective_display_name() {
        let now = Utc::now();
        let base = StatusDefinition {
            id: Uuid::new_v4(),
            project_type_id: Uuid::new_v4(),
            code: "OPEN".to_string(),
            display_name: "オープン".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: true,
            is_terminal: false,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: now,
            updated_at: now,
        };
        let merged = TenantMergedStatus {
            base_status: base,
            extension: None,
            effective_display_name: "テナント専用オープン".to_string(),
            effective_attributes: None,
        };
        assert_eq!(merged.effective_display_name, "テナント専用オープン");
        assert!(merged.extension.is_none());
    }
}
