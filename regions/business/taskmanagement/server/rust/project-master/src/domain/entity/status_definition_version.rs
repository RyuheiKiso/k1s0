// ステータス定義バージョンエンティティ。
// ワークフロー変更の監査履歴を表現する（会計の MasterItemVersion に相当）。
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// ステータス定義バージョン（会計の MasterItemVersion に相当）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDefinitionVersion {
    pub id: Uuid,
    pub status_definition_id: Uuid,
    pub version_number: i32,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
    pub changed_by: String,
    pub change_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// バージョン一覧フィルタ
#[derive(Debug, Clone, Default)]
pub struct StatusDefinitionVersionFilter {
    pub status_definition_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
