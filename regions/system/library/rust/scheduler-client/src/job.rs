use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    Cron(String),
    OneShot(DateTime<Utc>),
    #[serde(with = "duration_secs")]
    Interval(Duration),
}

mod duration_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(d.as_secs())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let secs = u64::deserialize(d)?;
        Ok(Duration::from_secs(secs))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequest {
    pub name: String,
    pub schedule: Schedule,
    pub payload: serde_json::Value,
    pub max_retries: u32,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub schedule: Schedule,
    pub status: JobStatus,
    pub payload: serde_json::Value,
    pub max_retries: u32,
    pub timeout_secs: u64,
    pub created_at: DateTime<Utc>,
    pub next_run_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct JobFilter {
    pub status: Option<JobStatus>,
    pub name_prefix: Option<String>,
}

impl JobFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: JobStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn name_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.name_prefix = Some(prefix.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecution {
    pub id: String,
    pub job_id: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub result: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCompletedEvent {
    pub job_id: String,
    pub execution_id: String,
    pub completed_at: DateTime<Utc>,
    pub result: String,
}
