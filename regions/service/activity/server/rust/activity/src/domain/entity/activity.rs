// アクティビティエンティティ。承認フロー (Active → Submitted → Approved/Rejected) を持つ。
// payment の冪等性パターンを踏襲する。
use crate::domain::error::ActivityError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// アクティビティステータス（承認フロー）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityStatus {
    Active,
    Submitted,
    Approved,
    Rejected,
}

impl ActivityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Submitted => "submitted",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Active, Self::Submitted)
                | (Self::Submitted, Self::Approved)
                | (Self::Submitted, Self::Rejected)
                | (Self::Rejected, Self::Active)
        )
    }
}

impl std::str::FromStr for ActivityStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "submitted" => Ok(Self::Submitted),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            _ => Err(format!("invalid activity status: '{}'", s)),
        }
    }
}

impl std::fmt::Display for ActivityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// アクティビティ種別
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Comment,
    TimeEntry,
    StatusChange,
    Assignment,
}

impl ActivityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::TimeEntry => "time_entry",
            Self::StatusChange => "status_change",
            Self::Assignment => "assignment",
        }
    }
}

impl std::str::FromStr for ActivityType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "comment" => Ok(Self::Comment),
            "time_entry" => Ok(Self::TimeEntry),
            "status_change" => Ok(Self::StatusChange),
            "assignment" => Ok(Self::Assignment),
            _ => Err(format!("invalid activity type: '{}'", s)),
        }
    }
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// アクティビティエンティティ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: Uuid,
    pub task_id: Uuid,
    pub actor_id: String,
    pub activity_type: ActivityType,
    pub content: Option<String>,
    pub duration_minutes: Option<i32>,
    pub status: ActivityStatus,
    pub idempotency_key: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Activity {
    /// ステータス遷移を検証する
    pub fn transition_to(&self, next: ActivityStatus) -> Result<ActivityStatus, ActivityError> {
        if !self.status.can_transition_to(&next) {
            return Err(ActivityError::InvalidStatusTransition {
                from: self.status.to_string(),
                to: next.to_string(),
            });
        }
        Ok(next)
    }
}

/// アクティビティ作成 DTO（冪等性キー付き）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActivity {
    pub task_id: Uuid,
    pub activity_type: ActivityType,
    pub content: Option<String>,
    pub duration_minutes: Option<i32>,
    pub idempotency_key: Option<String>,
}

impl CreateActivity {
    pub fn validate(&self) -> Result<(), ActivityError> {
        if self.activity_type == ActivityType::TimeEntry && self.duration_minutes.is_none() {
            return Err(ActivityError::ValidationFailed(
                "duration_minutes is required for TimeEntry".to_string(),
            ));
        }
        Ok(())
    }
}

/// アクティビティ一覧フィルター
#[derive(Debug, Clone, Default)]
pub struct ActivityFilter {
    pub task_id: Option<Uuid>,
    pub actor_id: Option<String>,
    pub status: Option<ActivityStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(ActivityStatus::Active.can_transition_to(&ActivityStatus::Submitted));
        assert!(ActivityStatus::Submitted.can_transition_to(&ActivityStatus::Approved));
        assert!(ActivityStatus::Submitted.can_transition_to(&ActivityStatus::Rejected));
        assert!(ActivityStatus::Rejected.can_transition_to(&ActivityStatus::Active));
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(!ActivityStatus::Active.can_transition_to(&ActivityStatus::Approved));
        assert!(!ActivityStatus::Approved.can_transition_to(&ActivityStatus::Active));
    }

    #[test]
    fn test_status_roundtrip() {
        for s in &["active", "submitted", "approved", "rejected"] {
            let parsed: ActivityStatus = s.parse().unwrap();
            assert_eq!(parsed.as_str(), *s);
        }
    }

    #[test]
    fn test_validate_time_entry_requires_duration() {
        let input = CreateActivity {
            task_id: Uuid::new_v4(),
            activity_type: ActivityType::TimeEntry,
            content: None,
            duration_minutes: None,
            idempotency_key: None,
        };
        assert!(input.validate().is_err());
    }

    #[test]
    fn test_validate_comment_no_duration() {
        let input = CreateActivity {
            task_id: Uuid::new_v4(),
            activity_type: ActivityType::Comment,
            content: Some("Nice work!".to_string()),
            duration_minutes: None,
            idempotency_key: None,
        };
        assert!(input.validate().is_ok());
    }
}
