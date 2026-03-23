// プロジェクトタイプエンティティ。
// ソフトウェア開発・マーケティング等のテンプレート定義を表現する。
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// プロジェクトタイプ（会計の MasterCategory に相当）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectType {
    pub id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub default_workflow: Option<serde_json::Value>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// プロジェクトタイプ作成 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectType {
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub default_workflow: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

/// プロジェクトタイプ更新 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectType {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub default_workflow: Option<serde_json::Value>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i32>,
}

/// プロジェクトタイプ一覧フィルタ
#[derive(Debug, Clone, Default)]
pub struct ProjectTypeFilter {
    pub active_only: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ProjectTypeエンティティが正しくフィールドを保持することを確認する
    // 前提: 有効なUUIDと文字列を渡す
    // 期待: 各フィールドが正確に格納されている
    #[test]
    fn test_project_type_fields() {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let pt = ProjectType {
            id,
            code: "SOFTWARE".to_string(),
            display_name: "ソフトウェア開発".to_string(),
            description: Some("説明文".to_string()),
            default_workflow: None,
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: now,
            updated_at: now,
        };
        assert_eq!(pt.id, id);
        assert_eq!(pt.code, "SOFTWARE");
        assert_eq!(pt.display_name, "ソフトウェア開発");
        assert!(pt.description.is_some());
        assert!(pt.is_active);
        assert_eq!(pt.sort_order, 1);
    }

    // CreateProjectType DTOが省略可能フィールドのデフォルト値（None）を取ることを確認する
    // 前提: is_active と sort_order を指定しない
    // 期待: 省略可能フィールドが None となる
    #[test]
    fn test_create_project_type_optional_fields() {
        let input = CreateProjectType {
            code: "MARKETING".to_string(),
            display_name: "マーケティング".to_string(),
            description: None,
            default_workflow: None,
            is_active: None,
            sort_order: None,
        };
        assert_eq!(input.code, "MARKETING");
        assert!(input.is_active.is_none());
        assert!(input.sort_order.is_none());
    }

    // UpdateProjectType DTOが全フィールド省略可能であることを確認する
    // 前提: 全フィールドを指定しない
    // 期待: 全て None となる
    #[test]
    fn test_update_project_type_all_none() {
        let input = UpdateProjectType {
            display_name: None,
            description: None,
            default_workflow: None,
            is_active: None,
            sort_order: None,
        };
        assert!(input.display_name.is_none());
        assert!(input.is_active.is_none());
    }

    // ProjectTypeFilterのデフォルト値が正しいことを確認する
    // 前提: Default トレイトを使用する
    // 期待: active_only=false, limit/offset が None
    #[test]
    fn test_project_type_filter_default() {
        let filter = ProjectTypeFilter::default();
        assert!(!filter.active_only);
        assert!(filter.limit.is_none());
        assert!(filter.offset.is_none());
    }

    // ProjectTypeFilterにactive_only=trueを設定できることを確認する
    // 前提: active_onlyにtrueを指定する
    // 期待: フィルタがアクティブのみに限定される
    #[test]
    fn test_project_type_filter_active_only() {
        let filter = ProjectTypeFilter {
            active_only: true,
            limit: Some(10),
            offset: Some(0),
        };
        assert!(filter.active_only);
        assert_eq!(filter.limit, Some(10));
        assert_eq!(filter.offset, Some(0));
    }
}
