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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // StatusDefinitionVersionエンティティが正しくフィールドを保持することを確認する
    // 前提: 変更前後のデータを持つバージョンレコードを表現する
    // 期待: version_number, before_data, after_data が正確に格納されている
    #[test]
    fn test_status_definition_version_fields() {
        let id = Uuid::new_v4();
        let status_def_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let before = serde_json::json!({"display_name": "旧表示名"});
        let after = serde_json::json!({"display_name": "新表示名"});
        let version = StatusDefinitionVersion {
            id,
            status_definition_id: status_def_id,
            version_number: 2,
            before_data: Some(before.clone()),
            after_data: Some(after.clone()),
            changed_by: "user1".to_string(),
            change_reason: Some("表示名変更".to_string()),
            created_at: now,
        };
        assert_eq!(version.id, id);
        assert_eq!(version.status_definition_id, status_def_id);
        assert_eq!(version.version_number, 2);
        assert!(version.before_data.is_some());
        assert!(version.after_data.is_some());
        assert_eq!(version.changed_by, "user1");
    }

    // StatusDefinitionVersionで変更前後データがNoneの場合も正しく動作することを確認する
    // 前提: 初回作成バージョン（before_dataがない）を表現する
    // 期待: before_data=None, after_dataは存在する
    #[test]
    fn test_status_definition_version_initial_creation() {
        let now = chrono::Utc::now();
        let version = StatusDefinitionVersion {
            id: Uuid::new_v4(),
            status_definition_id: Uuid::new_v4(),
            version_number: 1,
            before_data: None,
            after_data: Some(serde_json::json!({"code": "OPEN"})),
            changed_by: "admin".to_string(),
            change_reason: None,
            created_at: now,
        };
        assert_eq!(version.version_number, 1);
        assert!(version.before_data.is_none());
        assert!(version.after_data.is_some());
        assert!(version.change_reason.is_none());
    }

    // StatusDefinitionVersionFilterのデフォルト値が正しいことを確認する
    // 前提: Default トレイトを使用する
    // 期待: 全フィールドが None となる
    #[test]
    fn test_version_filter_default() {
        let filter = StatusDefinitionVersionFilter::default();
        assert!(filter.status_definition_id.is_none());
        assert!(filter.limit.is_none());
        assert!(filter.offset.is_none());
    }

    // StatusDefinitionVersionFilterにstatus_definition_idを指定できることを確認する
    // 前提: 特定のstatus_definition_idを指定する
    // 期待: フィルタが正しく設定されている
    #[test]
    fn test_version_filter_with_status_definition_id() {
        let sd_id = Uuid::new_v4();
        let filter = StatusDefinitionVersionFilter {
            status_definition_id: Some(sd_id),
            limit: Some(5),
            offset: Some(0),
        };
        assert_eq!(filter.status_definition_id, Some(sd_id));
        assert_eq!(filter.limit, Some(5));
    }
}
