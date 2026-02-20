use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// StepAction はステップの実行アクションを表す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepAction {
    Execute,
    Compensate,
}

impl std::fmt::Display for StepAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Execute => write!(f, "EXECUTE"),
            Self::Compensate => write!(f, "COMPENSATE"),
        }
    }
}

/// StepStatus はステップの実行結果を表す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepStatus {
    Success,
    Failed,
    Timeout,
    Skipped,
}

impl std::fmt::Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "SUCCESS"),
            Self::Failed => write!(f, "FAILED"),
            Self::Timeout => write!(f, "TIMEOUT"),
            Self::Skipped => write!(f, "SKIPPED"),
        }
    }
}

/// SagaStepLog はSagaステップの実行ログを表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStepLog {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub step_index: i32,
    pub step_name: String,
    pub action: StepAction,
    pub status: StepStatus,
    pub request_payload: Option<serde_json::Value>,
    pub response_payload: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl SagaStepLog {
    /// 実行ログを作成する。
    pub fn new_execute(
        saga_id: Uuid,
        step_index: i32,
        step_name: String,
        request_payload: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            saga_id,
            step_index,
            step_name,
            action: StepAction::Execute,
            status: StepStatus::Failed, // default, overwritten on completion
            request_payload,
            response_payload: None,
            error_message: None,
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    /// 補償ログを作成する。
    pub fn new_compensate(
        saga_id: Uuid,
        step_index: i32,
        step_name: String,
        request_payload: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            saga_id,
            step_index,
            step_name,
            action: StepAction::Compensate,
            status: StepStatus::Failed,
            request_payload,
            response_payload: None,
            error_message: None,
            started_at: Utc::now(),
            completed_at: None,
        }
    }

    /// 成功にマークする。
    pub fn mark_success(&mut self, response: Option<serde_json::Value>) {
        self.status = StepStatus::Success;
        self.response_payload = response;
        self.completed_at = Some(Utc::now());
    }

    /// 失敗にマークする。
    pub fn mark_failed(&mut self, error: String) {
        self.status = StepStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
    }

    /// タイムアウトにマークする。
    pub fn mark_timeout(&mut self) {
        self.status = StepStatus::Timeout;
        self.error_message = Some("step timed out".to_string());
        self.completed_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_execute() {
        let saga_id = Uuid::new_v4();
        let log = SagaStepLog::new_execute(
            saga_id,
            0,
            "reserve-inventory".to_string(),
            Some(serde_json::json!({"item_id": "abc"})),
        );
        assert_eq!(log.saga_id, saga_id);
        assert_eq!(log.step_index, 0);
        assert_eq!(log.action, StepAction::Execute);
        assert_eq!(log.status, StepStatus::Failed); // default before completion
        assert!(log.completed_at.is_none());
    }

    #[test]
    fn test_new_compensate() {
        let saga_id = Uuid::new_v4();
        let log = SagaStepLog::new_compensate(saga_id, 1, "release-inventory".to_string(), None);
        assert_eq!(log.action, StepAction::Compensate);
    }

    #[test]
    fn test_mark_success() {
        let mut log = SagaStepLog::new_execute(Uuid::new_v4(), 0, "step".to_string(), None);
        log.mark_success(Some(serde_json::json!({"ok": true})));
        assert_eq!(log.status, StepStatus::Success);
        assert!(log.completed_at.is_some());
        assert!(log.response_payload.is_some());
    }

    #[test]
    fn test_mark_failed() {
        let mut log = SagaStepLog::new_execute(Uuid::new_v4(), 0, "step".to_string(), None);
        log.mark_failed("connection refused".to_string());
        assert_eq!(log.status, StepStatus::Failed);
        assert_eq!(log.error_message.as_deref(), Some("connection refused"));
        assert!(log.completed_at.is_some());
    }

    #[test]
    fn test_mark_timeout() {
        let mut log = SagaStepLog::new_execute(Uuid::new_v4(), 0, "step".to_string(), None);
        log.mark_timeout();
        assert_eq!(log.status, StepStatus::Timeout);
        assert!(log.completed_at.is_some());
    }

    #[test]
    fn test_action_display() {
        assert_eq!(StepAction::Execute.to_string(), "EXECUTE");
        assert_eq!(StepAction::Compensate.to_string(), "COMPENSATE");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(StepStatus::Success.to_string(), "SUCCESS");
        assert_eq!(StepStatus::Failed.to_string(), "FAILED");
        assert_eq!(StepStatus::Timeout.to_string(), "TIMEOUT");
        assert_eq!(StepStatus::Skipped.to_string(), "SKIPPED");
    }
}
