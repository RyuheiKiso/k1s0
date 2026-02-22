use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// SagaStatus はSagaの状態を表す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SagaStatus {
    Started,
    Running,
    Completed,
    Compensating,
    Failed,
    Cancelled,
}

impl std::fmt::Display for SagaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Started => write!(f, "STARTED"),
            Self::Running => write!(f, "RUNNING"),
            Self::Completed => write!(f, "COMPLETED"),
            Self::Compensating => write!(f, "COMPENSATING"),
            Self::Failed => write!(f, "FAILED"),
            Self::Cancelled => write!(f, "CANCELLED"),
        }
    }
}

impl SagaStatus {
    pub fn from_str_value(s: &str) -> anyhow::Result<Self> {
        match s {
            "STARTED" => Ok(Self::Started),
            "RUNNING" => Ok(Self::Running),
            "COMPLETED" => Ok(Self::Completed),
            "COMPENSATING" => Ok(Self::Compensating),
            "FAILED" => Ok(Self::Failed),
            "CANCELLED" => Ok(Self::Cancelled),
            _ => anyhow::bail!("invalid saga status: {}", s),
        }
    }
}

/// SagaState はSagaの状態を表す。
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct SagaState {
    pub saga_id: Uuid,
    pub workflow_name: String,
    pub current_step: i32,
    pub status: SagaStatus,
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
    pub initiated_by: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SagaState {
    /// 新しいSagaStateを作成する。
    pub fn new(
        workflow_name: String,
        payload: serde_json::Value,
        correlation_id: Option<String>,
        initiated_by: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            saga_id: Uuid::new_v4(),
            workflow_name,
            current_step: 0,
            status: SagaStatus::Started,
            payload,
            correlation_id,
            initiated_by,
            error_message: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// ステップを進める。
    pub fn advance_step(&mut self) {
        self.current_step += 1;
        self.status = SagaStatus::Running;
        self.updated_at = Utc::now();
    }

    /// Sagaを完了する。
    pub fn complete(&mut self) {
        self.status = SagaStatus::Completed;
        self.error_message = None;
        self.updated_at = Utc::now();
    }

    /// 補償処理を開始する。
    pub fn start_compensation(&mut self, error: String) {
        self.status = SagaStatus::Compensating;
        self.error_message = Some(error);
        self.updated_at = Utc::now();
    }

    /// Sagaを失敗にする。
    pub fn fail(&mut self, error: String) {
        self.status = SagaStatus::Failed;
        self.error_message = Some(error);
        self.updated_at = Utc::now();
    }

    /// Sagaをキャンセルする。
    pub fn cancel(&mut self) {
        self.status = SagaStatus::Cancelled;
        self.updated_at = Utc::now();
    }

    /// 終端状態かどうかを返す。
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            SagaStatus::Completed | SagaStatus::Failed | SagaStatus::Cancelled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_saga() -> SagaState {
        SagaState::new(
            "order-fulfillment".to_string(),
            serde_json::json!({"order_id": "123"}),
            Some("corr-001".to_string()),
            Some("user-1".to_string()),
        )
    }

    #[test]
    fn test_new_saga_state() {
        let saga = make_saga();
        assert_eq!(saga.status, SagaStatus::Started);
        assert_eq!(saga.current_step, 0);
        assert_eq!(saga.workflow_name, "order-fulfillment");
        assert!(saga.error_message.is_none());
        assert!(!saga.is_terminal());
    }

    #[test]
    fn test_advance_step() {
        let mut saga = make_saga();
        saga.advance_step();
        assert_eq!(saga.current_step, 1);
        assert_eq!(saga.status, SagaStatus::Running);
    }

    #[test]
    fn test_complete() {
        let mut saga = make_saga();
        saga.advance_step();
        saga.complete();
        assert_eq!(saga.status, SagaStatus::Completed);
        assert!(saga.is_terminal());
    }

    #[test]
    fn test_start_compensation() {
        let mut saga = make_saga();
        saga.advance_step();
        saga.start_compensation("step failed".to_string());
        assert_eq!(saga.status, SagaStatus::Compensating);
        assert_eq!(saga.error_message.as_deref(), Some("step failed"));
        assert!(!saga.is_terminal());
    }

    #[test]
    fn test_fail() {
        let mut saga = make_saga();
        saga.fail("unrecoverable error".to_string());
        assert_eq!(saga.status, SagaStatus::Failed);
        assert!(saga.is_terminal());
    }

    #[test]
    fn test_cancel() {
        let mut saga = make_saga();
        saga.cancel();
        assert_eq!(saga.status, SagaStatus::Cancelled);
        assert!(saga.is_terminal());
    }

    #[test]
    fn test_status_display() {
        assert_eq!(SagaStatus::Started.to_string(), "STARTED");
        assert_eq!(SagaStatus::Running.to_string(), "RUNNING");
        assert_eq!(SagaStatus::Completed.to_string(), "COMPLETED");
        assert_eq!(SagaStatus::Compensating.to_string(), "COMPENSATING");
        assert_eq!(SagaStatus::Failed.to_string(), "FAILED");
        assert_eq!(SagaStatus::Cancelled.to_string(), "CANCELLED");
    }

    #[test]
    fn test_status_from_str() {
        assert_eq!(
            SagaStatus::from_str_value("STARTED").unwrap(),
            SagaStatus::Started
        );
        assert_eq!(
            SagaStatus::from_str_value("RUNNING").unwrap(),
            SagaStatus::Running
        );
        assert!(SagaStatus::from_str_value("INVALID").is_err());
    }

    #[test]
    fn test_terminal_states() {
        assert!(!SagaStatus::Started.eq(&SagaStatus::Completed)); // not equal
        let mut saga = make_saga();
        assert!(!saga.is_terminal()); // Started
        saga.status = SagaStatus::Running;
        assert!(!saga.is_terminal());
        saga.status = SagaStatus::Compensating;
        assert!(!saga.is_terminal());
        saga.status = SagaStatus::Completed;
        assert!(saga.is_terminal());
        saga.status = SagaStatus::Failed;
        assert!(saga.is_terminal());
        saga.status = SagaStatus::Cancelled;
        assert!(saga.is_terminal());
    }
}
