// タスクエンティティ。
// Open→InProgress→Review→Done/Cancelled のステータスマシンを持つ。
use crate::domain::error::TaskError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// 文字列パースエラー型（thiserror ベースで型安全なエラー分類を実現する）
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid value: {0}")]
    InvalidValue(String),
}

/// タスクステータス（ステータスマシン）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Open,
    InProgress,
    Review,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Review => "review",
            Self::Done => "done",
            Self::Cancelled => "cancelled",
        }
    }

    /// ステータス遷移が有効かどうかを検証する
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Open, Self::InProgress)
                | (Self::Open, Self::Cancelled)
                | (Self::InProgress, Self::Review)
                | (Self::InProgress, Self::Cancelled)
                | (Self::Review, Self::Done)
                | (Self::Review, Self::InProgress)
                | (Self::Review, Self::Cancelled)
        )
    }
}

// TaskStatus の文字列パース実装（型安全な ParseError を使用する）
impl std::str::FromStr for TaskStatus {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "in_progress" => Ok(Self::InProgress),
            "review" => Ok(Self::Review),
            "done" => Ok(Self::Done),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(ParseError::InvalidValue(format!("invalid task status: '{}'", s))),
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// タスクの優先度
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

// TaskPriority の文字列パース実装（型安全な ParseError を使用する）
impl std::str::FromStr for TaskPriority {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => Err(ParseError::InvalidValue(format!("invalid task priority: '{}'", s))),
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// タスクエンティティ
/// proto の Task メッセージと一致するよう reporter_id と labels フィールドを保持する。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assignee_id: Option<String>,
    // タスクを報告したユーザーの ID（proto の reporter_id フィールドに対応）
    pub reporter_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    // タスクに付与されたラベル一覧（proto の labels フィールドに対応）
    pub labels: Vec<String>,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    /// ステータス遷移を検証し、遷移後のステータスを返す
    pub fn transition_to(&self, next: TaskStatus) -> Result<TaskStatus, TaskError> {
        if !self.status.can_transition_to(&next) {
            return Err(TaskError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: next.to_string(),
            });
        }
        Ok(next)
    }
}

/// タスクチェックリスト項目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskChecklistItem {
    pub id: Uuid,
    pub task_id: Uuid,
    pub title: String,
    pub is_completed: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// タスク作成 DTO
/// proto の CreateTaskRequest フィールドに対応し、reporter_id と labels を含む。
/// validator クレートを使いスキーマレベルのバリデーションを定義する（SM-3 監査対応）
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTask {
    pub project_id: Uuid,
    // タイトルは1〜500文字の範囲で必須入力
    #[validate(length(min = 1, max = 500, message = "タイトルは1〜500文字で指定してください"))]
    pub title: String,
    // 説明は最大5000文字
    #[validate(length(max = 5000, message = "説明は5000文字以内で指定してください"))]
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub assignee_id: Option<String>,
    // タスクを報告したユーザーの ID（DB の reporter_id NOT NULL カラムに対応。ハンドラーで設定する）
    pub reporter_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    // タスクに付与するラベル一覧（proto の labels フィールドに対応、最大20件）
    #[validate(length(max = 20, message = "ラベルは最大20件まで指定できます"))]
    pub labels: Vec<String>,
    pub checklist: Vec<CreateChecklistItem>,
}


/// チェックリスト項目作成 DTO
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateChecklistItem {
    // チェックリスト項目のタイトルは1〜200文字
    #[validate(length(min = 1, max = 200, message = "チェックリスト項目は1〜200文字で指定してください"))]
    pub title: String,
    pub sort_order: i32,
}

/// タスク更新 DTO（REST PUT /tasks/{id} 専用）
/// 未設定フィールドは変更しない（部分更新）。
/// validator クレートを使いスキーマレベルのバリデーションを定義する（SM-3 監査対応）
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateTask {
    // タイトルは1〜500文字（指定する場合のみバリデーション）
    #[validate(length(min = 1, max = 500, message = "タイトルは1〜500文字で指定してください"))]
    pub title: Option<String>,
    // 説明は最大5000文字
    #[validate(length(max = 5000, message = "説明は5000文字以内で指定してください"))]
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub assignee_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    // ラベル一覧を指定した場合は全置換する（None の場合は変更しない）、最大20件
    #[validate(length(max = 20, message = "ラベルは最大20件まで指定できます"))]
    pub labels: Option<Vec<String>>,
}

/// チェックリスト項目追加 DTO（REST POST /tasks/{id}/checklist 専用）
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddChecklistItem {
    // チェックリスト項目のタイトルは1〜200文字
    #[validate(length(min = 1, max = 200, message = "チェックリスト項目は1〜200文字で指定してください"))]
    pub title: String,
    pub sort_order: i32,
}

/// チェックリスト項目更新 DTO（REST PUT /tasks/{id}/checklist/{item_id} 専用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChecklistItem {
    pub title: Option<String>,
    pub is_completed: Option<bool>,
    pub sort_order: Option<i32>,
}

/// タスクステータス更新 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskStatus {
    pub status: TaskStatus,
    pub expected_version: i32,
}

/// タスク一覧フィルター
#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub project_id: Option<Uuid>,
    pub assignee_id: Option<String>,
    pub status: Option<TaskStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(test)]
// テストコード内の .unwrap() 呼び出しを許容する（テスト失敗時にパニックで意図を明示するため）
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(TaskStatus::Open.can_transition_to(&TaskStatus::InProgress));
        assert!(TaskStatus::Open.can_transition_to(&TaskStatus::Cancelled));
        assert!(TaskStatus::InProgress.can_transition_to(&TaskStatus::Review));
        assert!(TaskStatus::Review.can_transition_to(&TaskStatus::Done));
        assert!(TaskStatus::Review.can_transition_to(&TaskStatus::InProgress));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!TaskStatus::Open.can_transition_to(&TaskStatus::Done));
        assert!(!TaskStatus::Done.can_transition_to(&TaskStatus::Open));
        assert!(!TaskStatus::Cancelled.can_transition_to(&TaskStatus::Open));
    }

    #[test]
    fn test_status_roundtrip() {
        for status in &["open", "in_progress", "review", "done", "cancelled"] {
            let parsed: TaskStatus = status.parse().unwrap();
            assert_eq!(parsed.as_str(), *status);
        }
    }
}
