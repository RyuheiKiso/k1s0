// ボードカラムエンティティ。プロジェクト×ステータスコードのWIP管理。
use crate::domain::error::BoardError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardColumn {
    pub id: Uuid,
    pub project_id: Uuid,
    pub status_code: String,
    pub wip_limit: i32,
    pub task_count: i32,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BoardColumn {
    /// WIP 制限に達しているか確認する（在庫チェックに相当）
    pub fn check_wip_limit(&self) -> Result<(), BoardError> {
        if self.wip_limit > 0 && self.task_count >= self.wip_limit {
            return Err(BoardError::WipLimitExceeded {
                column_id: self.id.to_string(),
                current: self.task_count,
                limit: self.wip_limit,
            });
        }
        Ok(())
    }
}

/// カラムへのタスク追加リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementColumnRequest {
    pub task_id: Uuid,
    pub project_id: Uuid,
    pub status_code: String,
}

/// カラムからのタスク削除リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecrementColumnRequest {
    pub task_id: Uuid,
    pub project_id: Uuid,
    pub status_code: String,
    pub reason: Option<String>,
}

/// WIP 制限更新リクエスト
/// column_id はパスパラメータ（PUT /api/v1/board-columns/{id}）から取得するため、
/// JSON ボディのデシリアライズ対象から除外する。ハンドラーで req.column_id = column_id にて上書きする。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWipLimitRequest {
    // パスパラメータから設定されるため JSON デシリアライズをスキップする
    #[serde(skip_deserializing, default)]
    pub column_id: Uuid,
    pub wip_limit: i32,
    pub expected_version: i32,
}

/// カラム一覧フィルター
#[derive(Debug, Clone, Default)]
pub struct BoardColumnFilter {
    pub project_id: Option<Uuid>,
    pub status_code: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_column(wip_limit: i32, task_count: i32) -> BoardColumn {
        BoardColumn {
            id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            status_code: "in_progress".to_string(),
            wip_limit,
            task_count,
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_wip_limit_not_exceeded() {
        let col = sample_column(5, 4);
        assert!(col.check_wip_limit().is_ok());
    }

    #[test]
    fn test_wip_limit_exceeded() {
        let col = sample_column(5, 5);
        assert!(col.check_wip_limit().is_err());
    }

    #[test]
    fn test_wip_limit_zero_means_unlimited() {
        let col = sample_column(0, 999);
        assert!(col.check_wip_limit().is_ok());
    }

    // 境界値: task_count が wip_limit - 1 の場合は制限未到達
    #[test]
    fn test_wip_limit_one_below_limit() {
        let col = sample_column(5, 4);
        assert!(col.check_wip_limit().is_ok());
    }

    // 境界値: task_count が wip_limit と等しい場合は制限到達
    #[test]
    fn test_wip_limit_exactly_at_limit() {
        let col = sample_column(3, 3);
        let result = col.check_wip_limit();
        assert!(result.is_err());
        if let Err(BoardError::WipLimitExceeded { current, limit, .. }) = result {
            assert_eq!(current, 3);
            assert_eq!(limit, 3);
        } else {
            panic!("expected WipLimitExceeded");
        }
    }

    // 境界値: task_count が wip_limit を超過している場合
    #[test]
    fn test_wip_limit_exceeds_limit() {
        let col = sample_column(3, 10);
        let result = col.check_wip_limit();
        assert!(result.is_err());
        if let Err(BoardError::WipLimitExceeded { current, limit, .. }) = result {
            assert_eq!(current, 10);
            assert_eq!(limit, 3);
        } else {
            panic!("expected WipLimitExceeded");
        }
    }

    // wip_limit=1 かつ task_count=0 の場合は許容される
    #[test]
    fn test_wip_limit_one_with_zero_tasks() {
        let col = sample_column(1, 0);
        assert!(col.check_wip_limit().is_ok());
    }

    // wip_limit=1 かつ task_count=1 の場合は制限到達
    #[test]
    fn test_wip_limit_one_with_one_task() {
        let col = sample_column(1, 1);
        assert!(col.check_wip_limit().is_err());
    }

    // BoardColumn の Clone が正しくディープコピーされることを確認する
    #[test]
    fn test_board_column_clone() {
        let original = sample_column(5, 3);
        let cloned = original.clone();
        assert_eq!(original.id, cloned.id);
        assert_eq!(original.wip_limit, cloned.wip_limit);
        assert_eq!(original.task_count, cloned.task_count);
    }

    // エラーメッセージに column_id / current / limit が含まれることを確認する
    #[test]
    fn test_wip_limit_exceeded_error_message() {
        let col = sample_column(5, 5);
        let result = col.check_wip_limit();
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("5"), "expected current count in error: {msg}");
        assert!(msg.contains(col.id.to_string().as_str()), "expected column_id in error: {msg}");
    }

    // IncrementColumnRequest の構築が正常にできることを確認する
    #[test]
    fn test_increment_column_request_construction() {
        let task_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let req = IncrementColumnRequest {
            task_id,
            project_id,
            status_code: "in_progress".to_string(),
        };
        assert_eq!(req.task_id, task_id);
        assert_eq!(req.project_id, project_id);
        assert_eq!(req.status_code, "in_progress");
    }

    // DecrementColumnRequest の reason フィールドが None/Some 両方に対応することを確認する
    #[test]
    fn test_decrement_column_request_with_and_without_reason() {
        let req_no_reason = DecrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            status_code: "done".to_string(),
            reason: None,
        };
        assert!(req_no_reason.reason.is_none());

        let req_with_reason = DecrementColumnRequest {
            task_id: Uuid::new_v4(),
            project_id: Uuid::new_v4(),
            status_code: "cancelled".to_string(),
            reason: Some("task cancelled by user".to_string()),
        };
        assert_eq!(req_with_reason.reason.as_deref(), Some("task cancelled by user"));
    }

    // UpdateWipLimitRequest の構築が正常にできることを確認する
    #[test]
    fn test_update_wip_limit_request_construction() {
        let column_id = Uuid::new_v4();
        let req = UpdateWipLimitRequest {
            column_id,
            wip_limit: 10,
            expected_version: 5,
        };
        assert_eq!(req.column_id, column_id);
        assert_eq!(req.wip_limit, 10);
        assert_eq!(req.expected_version, 5);
    }

    // BoardColumnFilter の default が全フィールド None であることを確認する
    #[test]
    fn test_board_column_filter_default() {
        let filter = BoardColumnFilter::default();
        assert!(filter.project_id.is_none());
        assert!(filter.status_code.is_none());
        assert!(filter.limit.is_none());
        assert!(filter.offset.is_none());
    }

    // BoardColumnFilter に値を設定できることを確認する
    #[test]
    fn test_board_column_filter_with_values() {
        let pid = Uuid::new_v4();
        let filter = BoardColumnFilter {
            project_id: Some(pid),
            status_code: Some("open".to_string()),
            limit: Some(20),
            offset: Some(40),
        };
        assert_eq!(filter.project_id, Some(pid));
        assert_eq!(filter.status_code.as_deref(), Some("open"));
        assert_eq!(filter.limit, Some(20));
        assert_eq!(filter.offset, Some(40));
    }

    // BoardColumn の Debug トレイト出力が正常に動作することを確認する
    #[test]
    fn test_board_column_debug() {
        let col = sample_column(5, 2);
        let debug_str = format!("{col:?}");
        assert!(debug_str.contains("BoardColumn"));
    }

    // WIP 制限 = task_count + 1 の場合は制限未到達であることを確認する（境界直下）
    #[test]
    fn test_wip_limit_one_above_task_count() {
        let col = sample_column(10, 9);
        assert!(col.check_wip_limit().is_ok());
    }

    // 複数のステータスコードを持つ BoardColumnFilter のテスト
    #[test]
    fn test_board_column_filter_different_status_codes() {
        for code in &["open", "in_progress", "review", "done", "cancelled"] {
            let filter = BoardColumnFilter {
                project_id: None,
                status_code: Some(code.to_string()),
                limit: None,
                offset: None,
            };
            assert_eq!(filter.status_code.as_deref(), Some(*code));
        }
    }
}
