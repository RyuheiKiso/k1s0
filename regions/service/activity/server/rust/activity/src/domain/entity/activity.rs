// アクティビティエンティティ。承認フロー (Active → Submitted → Approved/Rejected) を持つ。
// payment の冪等性パターンを踏襲する。
// H-021 監査対応: validator クレートを使用して CreateActivity のフィールドバリデーションを追加する。
use crate::domain::error::ActivityError;
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

// ActivityStatus の文字列パース実装（型安全な ParseError を使用する）
impl std::str::FromStr for ActivityStatus {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "submitted" => Ok(Self::Submitted),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            _ => Err(ParseError::InvalidValue(format!("invalid activity status: '{}'", s))),
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

// ActivityType の文字列パース実装（型安全な ParseError を使用する）
impl std::str::FromStr for ActivityType {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "comment" => Ok(Self::Comment),
            "time_entry" => Ok(Self::TimeEntry),
            "status_change" => Ok(Self::StatusChange),
            "assignment" => Ok(Self::Assignment),
            _ => Err(ParseError::InvalidValue(format!("invalid activity type: '{}'", s))),
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
// H-021 監査対応: Validate derive を追加し content フィールドに最大長バリデーションを付与する。
// content は任意項目（Option<String>）だが、入力された場合は 10000 文字以内に制限する。
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateActivity {
    pub task_id: Uuid,
    pub activity_type: ActivityType,
    // コメント・作業メモ等のテキスト。最大 10000 文字（XSS・ペイロード肥大化防止）
    #[validate(length(max = 10000, message = "content must be 10000 characters or fewer"))]
    pub content: Option<String>,
    pub duration_minutes: Option<i32>,
    pub idempotency_key: Option<String>,
}

impl CreateActivity {
    // ドメインルールの検証に加え、validator クレートが生成したフィールドバリデーションを実行する。
    // H-021 監査対応: Validate::validate() を呼び出して content の最大長チェックを行う。
    pub fn validate(&self) -> Result<(), ActivityError> {
        // validator クレートが生成したバリデーション（content 最大長等）を先に実行する
        Validate::validate(self).map_err(|e| {
            ActivityError::ValidationFailed(e.to_string())
        })?;
        // ドメイン固有ルール: TimeEntry の場合は duration_minutes が必須
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
// テストコード内の .unwrap() 呼び出しを許容する（テスト失敗時にパニックで意図を明示するため）
#[allow(clippy::unwrap_used)]
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

    // ActivityType の文字列変換が全バリアントで正常に動作することを検証する
    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_activity_type_roundtrip() {
        let variants = [
            (ActivityType::Comment, "comment"),
            (ActivityType::TimeEntry, "time_entry"),
            (ActivityType::StatusChange, "status_change"),
            (ActivityType::Assignment, "assignment"),
        ];
        for (activity_type, s) in &variants {
            assert_eq!(activity_type.as_str(), *s);
            let parsed: ActivityType = s.parse().unwrap();
            assert_eq!(parsed, *activity_type);
            assert_eq!(format!("{}", activity_type), *s);
        }
    }

    // 無効な文字列から ActivityType への変換がエラーを返すことを検証する
    #[test]
    fn test_activity_type_invalid_input() {
        let result: Result<ActivityType, _> = "invalid_type".parse();
        assert!(result.is_err());
        let result: Result<ActivityType, _> = "".parse();
        assert!(result.is_err());
        // 大文字は無効（大文字小文字を区別する）
        let result: Result<ActivityType, _> = "Comment".parse();
        assert!(result.is_err());
    }

    // ActivityStatus::can_transition_to() が全ての状態遷移の組み合わせで正しく動作することを検証する
    #[test]
    fn test_activity_status_transition_matrix() {
        // 有効な遷移: Active→Submitted
        assert!(ActivityStatus::Active.can_transition_to(&ActivityStatus::Submitted));
        // 有効な遷移: Submitted→Approved
        assert!(ActivityStatus::Submitted.can_transition_to(&ActivityStatus::Approved));
        // 有効な遷移: Submitted→Rejected
        assert!(ActivityStatus::Submitted.can_transition_to(&ActivityStatus::Rejected));
        // 有効な遷移: Rejected→Active（差し戻し後の再提出を許可）
        assert!(ActivityStatus::Rejected.can_transition_to(&ActivityStatus::Active));

        // 無効な遷移: Active から Approved/Rejected へは直接遷移不可
        assert!(!ActivityStatus::Active.can_transition_to(&ActivityStatus::Approved));
        assert!(!ActivityStatus::Active.can_transition_to(&ActivityStatus::Rejected));
        // 無効な遷移: Approved は終端状態
        assert!(!ActivityStatus::Approved.can_transition_to(&ActivityStatus::Active));
        assert!(!ActivityStatus::Approved.can_transition_to(&ActivityStatus::Submitted));
        assert!(!ActivityStatus::Approved.can_transition_to(&ActivityStatus::Rejected));
        // 無効な遷移: Rejected から直接 Submitted/Approved へは遷移不可
        assert!(!ActivityStatus::Rejected.can_transition_to(&ActivityStatus::Submitted));
        assert!(!ActivityStatus::Rejected.can_transition_to(&ActivityStatus::Approved));
        // 無効な遷移: Submitted から Active への逆戻りは不可
        assert!(!ActivityStatus::Submitted.can_transition_to(&ActivityStatus::Active));
    }

    // ActivityError の Display メッセージが期待通りの形式であることを検証する
    #[test]
    fn test_activity_error_display() {
        // NotFound エラーには ID が含まれること
        let err = ActivityError::NotFound("activity-456".to_string());
        assert!(err.to_string().contains("activity-456"));

        // InvalidStatusTransition エラーには from/to のステータス文字列が含まれること
        let err = ActivityError::InvalidStatusTransition {
            from: ActivityStatus::Approved.to_string(),
            to: ActivityStatus::Active.to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("approved"));
        assert!(msg.contains("active"));

        // ValidationFailed エラーには検証メッセージが含まれること
        let err = ActivityError::ValidationFailed("duration_minutes is required".to_string());
        assert!(err.to_string().contains("duration_minutes is required"));

        // DuplicateIdempotencyKey エラーには冪等性キーが含まれること
        let err = ActivityError::DuplicateIdempotencyKey("key-xyz".to_string());
        assert!(err.to_string().contains("key-xyz"));
    }
}
