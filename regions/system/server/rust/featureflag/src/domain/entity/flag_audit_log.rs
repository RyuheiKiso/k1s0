use chrono::{DateTime, Utc};
use uuid::Uuid;

/// FlagAuditLog はフィーチャーフラグの変更監査ログエンティティ。
/// STATIC-CRITICAL-001 監査対応: tenant_id でテナントスコープの監査ログを実現する。
/// HIGH-005 対応: migration 006 で tenant_id を UUID → TEXT に変更したため String 型を使用する。
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlagAuditLog {
    pub id: Uuid,
    /// テナント識別子: migration 006 で TEXT 型に変更済み。
    pub tenant_id: String,
    pub flag_id: Uuid,
    pub flag_key: String,
    pub action: String,
    pub before_json: Option<serde_json::Value>,
    pub after_json: Option<serde_json::Value>,
    pub changed_by: String,
    pub trace_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl FlagAuditLog {
    /// HIGH-005 対応: tenant_id は String 型（DB の TEXT 型に対応）。
    pub fn new(
        tenant_id: String,
        flag_id: Uuid,
        flag_key: String,
        action: String,
        before_json: Option<serde_json::Value>,
        after_json: Option<serde_json::Value>,
        changed_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            flag_id,
            flag_key,
            action,
            before_json,
            after_json,
            changed_by,
            trace_id: None,
            created_at: Utc::now(),
        }
    }
}
