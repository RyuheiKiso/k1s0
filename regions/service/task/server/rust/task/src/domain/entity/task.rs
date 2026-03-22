// タスクエンティティ。
// Open→InProgress→Review→Done/Cancelled のステータスマシンを持つ。
use crate::domain::error::TaskError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

impl std::str::FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(Self::Open),
            "in_progress" => Ok(Self::InProgress),
            "review" => Ok(Self::Review),
            "done" => Ok(Self::Done),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("invalid task status: '{}'", s)),
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

impl std::str::FromStr for TaskPriority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => Err(format!("invalid task priority: '{}'", s)),
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// タスクエンティティ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assignee_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTask {
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub priority: TaskPriority,
    pub assignee_id: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub checklist: Vec<CreateChecklistItem>,
}

impl CreateTask {
    /// 入力バリデーション
    pub fn validate(&self) -> Result<(), TaskError> {
        if self.title.trim().is_empty() {
            return Err(TaskError::ValidationFailed("title must not be empty".to_string()));
        }
        Ok(())
    }
}

/// チェックリスト項目作成 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChecklistItem {
    pub title: String,
    pub sort_order: i32,
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
