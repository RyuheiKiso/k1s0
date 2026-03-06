use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum FlowInstanceStatus {
    InProgress,
    Completed,
    Failed,
    Timeout,
}

impl FlowInstanceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Timeout => "timeout",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "timeout" => Self::Timeout,
            _ => Self::InProgress,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlowInstance {
    pub id: Uuid,
    pub flow_id: Uuid,
    pub correlation_id: String,
    pub status: FlowInstanceStatus,
    pub current_step_index: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
}

impl FlowInstance {
    pub fn new(flow_id: Uuid, correlation_id: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            flow_id,
            correlation_id,
            status: FlowInstanceStatus::InProgress,
            current_step_index: 0,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
        }
    }
}
