use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// SagaStatus はSagaの状態を表す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// SagaState はSagaの状態DTOを表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// SagaStepLog はSagaステップのログDTOを表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStepLog {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub step_index: i32,
    pub step_name: String,
    pub action: String,
    pub status: String,
    pub request_payload: Option<serde_json::Value>,
    pub response_payload: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// StartSagaRequest はSaga開始リクエスト。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSagaRequest {
    pub workflow_name: String,
    #[serde(default)]
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
    pub initiated_by: Option<String>,
}

/// StartSagaResponse はSaga開始レスポンス。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSagaResponse {
    pub saga_id: String,
    pub status: String,
}
