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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWipLimitRequest {
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
}
