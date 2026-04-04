// ステータス定義エンティティ。
// Open/In Progress/Review/Done 等の共通ステータス定義を表現する（会計の MasterItem に相当）。
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// M-013/M-036 監査対応: allowed_transitions を型付き構造体に変更してバリデーションを追加する
// serde_json::Value を使うと遷移先コードの存在チェックやロールバリデーションが実行時まで検出できないため、
// コンパイル時に型安全性を保証する構造体を定義する。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct StatusTransition {
    /// 遷移先のステータスコード
    pub to_status: String,
    /// 遷移に必要な権限（省略可能）
    pub required_role: Option<String>,
    /// 遷移条件（省略可能）
    pub condition: Option<String>,
}

/// ステータス定義（会計の MasterItem に相当）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDefinition {
    pub id: Uuid,
    pub project_type_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub allowed_transitions: Option<Vec<StatusTransition>>,
    pub is_initial: bool,
    pub is_terminal: bool,
    pub sort_order: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// ステータス定義作成 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStatusDefinition {
    pub project_type_id: Uuid,
    pub code: String,
    pub display_name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub allowed_transitions: Option<Vec<StatusTransition>>,
    pub is_initial: Option<bool>,
    pub is_terminal: Option<bool>,
    pub sort_order: Option<i32>,
}

/// ステータス定義更新 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatusDefinition {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub allowed_transitions: Option<Vec<StatusTransition>>,
    pub is_initial: Option<bool>,
    pub is_terminal: Option<bool>,
    pub sort_order: Option<i32>,
    pub change_reason: Option<String>,
}

/// ステータス定義一覧フィルタ
#[derive(Debug, Clone, Default)]
pub struct StatusDefinitionFilter {
    pub project_type_id: Option<Uuid>,
    pub active_only: bool,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // StatusDefinitionエンティティが正しくフィールドを保持することを確認する
    // 前提: is_initial=true, is_terminal=false の初期ステータスを表現する
    // 期待: 各フィールドが格納され、状態フラグが正確に設定されている
    #[test]
    fn test_status_definition_fields() {
        let id = Uuid::new_v4();
        let project_type_id = Uuid::new_v4();
        let now = chrono::Utc::now();
        let sd = StatusDefinition {
            id,
            project_type_id,
            code: "OPEN".to_string(),
            display_name: "オープン".to_string(),
            description: Some("初期ステータス".to_string()),
            color: Some("#00FF00".to_string()),
            allowed_transitions: None,
            is_initial: true,
            is_terminal: false,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: now,
            updated_at: now,
        };
        assert_eq!(sd.id, id);
        assert_eq!(sd.project_type_id, project_type_id);
        assert_eq!(sd.code, "OPEN");
        assert!(sd.is_initial);
        assert!(!sd.is_terminal);
    }

    // 終端ステータス（is_terminal=true）が正しく設定されることを確認する
    // 前提: DONE ステータスを表現する
    // 期待: is_terminal=true, is_initial=false となる
    #[test]
    fn test_status_definition_terminal_status() {
        let now = chrono::Utc::now();
        let sd = StatusDefinition {
            id: Uuid::new_v4(),
            project_type_id: Uuid::new_v4(),
            code: "DONE".to_string(),
            display_name: "完了".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: false,
            is_terminal: true,
            sort_order: 99,
            created_by: "admin".to_string(),
            created_at: now,
            updated_at: now,
        };
        assert!(!sd.is_initial);
        assert!(sd.is_terminal);
    }

    // CreateStatusDefinition DTOが正しく構築されることを確認する
    // 前提: is_initial と is_terminal を省略する
    // 期待: 省略可能フィールドが None となる
    #[test]
    fn test_create_status_definition_optional_fields() {
        let project_type_id = Uuid::new_v4();
        let input = CreateStatusDefinition {
            project_type_id,
            code: "IN_PROGRESS".to_string(),
            display_name: "進行中".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: None,
            is_terminal: None,
            sort_order: None,
        };
        assert_eq!(input.project_type_id, project_type_id);
        assert!(input.is_initial.is_none());
        assert!(input.is_terminal.is_none());
    }

    // StatusDefinitionFilterのデフォルト値が正しいことを確認する
    // 前提: Default トレイトを使用する
    // 期待: project_type_id=None, active_only=false
    #[test]
    fn test_status_definition_filter_default() {
        let filter = StatusDefinitionFilter::default();
        assert!(filter.project_type_id.is_none());
        assert!(!filter.active_only);
        assert!(filter.limit.is_none());
        assert!(filter.offset.is_none());
    }

    // StatusDefinitionFilterにproject_type_idでフィルタリングできることを確認する
    // 前提: 特定のproject_type_idを指定する
    // 期待: フィルタが正しく設定されている
    #[test]
    fn test_status_definition_filter_with_project_type() {
        let pt_id = Uuid::new_v4();
        let filter = StatusDefinitionFilter {
            project_type_id: Some(pt_id),
            active_only: true,
            limit: Some(20),
            offset: Some(10),
        };
        assert_eq!(filter.project_type_id, Some(pt_id));
        assert!(filter.active_only);
        assert_eq!(filter.limit, Some(20));
    }
}
