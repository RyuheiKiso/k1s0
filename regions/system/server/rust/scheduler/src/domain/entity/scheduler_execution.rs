use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerExecution {
    pub id: String,
    pub job_id: String,
    pub status: String,
    pub triggered_by: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl SchedulerExecution {
    #[must_use] 
    pub fn new(job_id: String) -> Self {
        Self {
            id: format!("exec_{}", uuid::Uuid::new_v4().simple()),
            job_id,
            status: "running".to_string(),
            triggered_by: "scheduler".to_string(),
            started_at: Utc::now(),
            finished_at: None,
            error_message: None,
        }
    }
}
