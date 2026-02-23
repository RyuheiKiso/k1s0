use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerExecution {
    pub id: Uuid,
    pub job_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl SchedulerExecution {
    pub fn new(job_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_id,
            status: "running".to_string(),
            started_at: Utc::now(),
            completed_at: None,
            error_message: None,
        }
    }
}
