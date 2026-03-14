use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use crate::client::SchedulerClient;
use crate::error::SchedulerError;
use crate::job::{Job, JobExecution, JobFilter, JobRequest, JobStatus, Schedule};

#[derive(Debug, Default)]
struct MemoryState {
    jobs: HashMap<String, Job>,
    seq: u64,
}

/// InMemorySchedulerClient is an in-memory SchedulerClient implementation
/// for unit tests and local development.
#[derive(Debug, Default)]
pub struct InMemorySchedulerClient {
    state: Mutex<MemoryState>,
}

impl InMemorySchedulerClient {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(MemoryState::default()),
        }
    }

    /// Returns a snapshot of currently registered jobs.
    pub async fn jobs(&self) -> HashMap<String, Job> {
        self.state.lock().await.jobs.clone()
    }
}

fn next_run_at(schedule: &Schedule) -> Option<DateTime<Utc>> {
    match schedule {
        Schedule::OneShot(at) => Some(*at),
        Schedule::Interval(interval) => chrono::Duration::from_std(*interval)
            .ok()
            .map(|d| Utc::now() + d),
        Schedule::Cron(_) => None,
    }
}

#[async_trait]
impl SchedulerClient for InMemorySchedulerClient {
    async fn create_job(&self, req: JobRequest) -> Result<Job, SchedulerError> {
        let mut state = self.state.lock().await;
        state.seq += 1;
        let id = format!("job-{:03}", state.seq);
        let job = Job {
            id: id.clone(),
            name: req.name,
            schedule: req.schedule.clone(),
            status: JobStatus::Pending,
            payload: req.payload,
            max_retries: req.max_retries,
            timeout_secs: req.timeout_secs,
            created_at: Utc::now(),
            next_run_at: next_run_at(&req.schedule),
        };
        state.jobs.insert(id, job.clone());
        Ok(job)
    }

    async fn cancel_job(&self, job_id: &str) -> Result<(), SchedulerError> {
        let mut state = self.state.lock().await;
        let Some(job) = state.jobs.get_mut(job_id) else {
            return Err(SchedulerError::JobNotFound(job_id.to_string()));
        };
        job.status = JobStatus::Cancelled;
        Ok(())
    }

    async fn pause_job(&self, job_id: &str) -> Result<(), SchedulerError> {
        let mut state = self.state.lock().await;
        let Some(job) = state.jobs.get_mut(job_id) else {
            return Err(SchedulerError::JobNotFound(job_id.to_string()));
        };
        job.status = JobStatus::Paused;
        Ok(())
    }

    async fn resume_job(&self, job_id: &str) -> Result<(), SchedulerError> {
        let mut state = self.state.lock().await;
        let Some(job) = state.jobs.get_mut(job_id) else {
            return Err(SchedulerError::JobNotFound(job_id.to_string()));
        };
        job.status = JobStatus::Pending;
        Ok(())
    }

    async fn get_job(&self, job_id: &str) -> Result<Job, SchedulerError> {
        let state = self.state.lock().await;
        state
            .jobs
            .get(job_id)
            .cloned()
            .ok_or_else(|| SchedulerError::JobNotFound(job_id.to_string()))
    }

    async fn list_jobs(&self, filter: JobFilter) -> Result<Vec<Job>, SchedulerError> {
        let state = self.state.lock().await;
        let mut jobs: Vec<Job> = state
            .jobs
            .values()
            .filter(|job| {
                if let Some(status) = filter.status.clone() {
                    if job.status != status {
                        return false;
                    }
                }
                if let Some(prefix) = filter.name_prefix.as_ref() {
                    if !job.name.starts_with(prefix) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        jobs.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(jobs)
    }

    async fn get_executions(&self, job_id: &str) -> Result<Vec<JobExecution>, SchedulerError> {
        let state = self.state.lock().await;
        if !state.jobs.contains_key(job_id) {
            return Err(SchedulerError::JobNotFound(job_id.to_string()));
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::Schedule;
    use serde_json::json;
    use std::time::Duration;

    // ジョブを作成しスナップショットで正しく確認できることを確認する。
    #[tokio::test]
    async fn test_create_and_snapshot_jobs() {
        let client = InMemorySchedulerClient::new();
        let job = client
            .create_job(JobRequest {
                name: "daily-report".to_string(),
                schedule: Schedule::Interval(Duration::from_secs(60)),
                payload: json!({"type": "report"}),
                max_retries: 3,
                timeout_secs: 30,
            })
            .await
            .unwrap();

        assert_eq!(job.id, "job-001");
        assert_eq!(job.status, JobStatus::Pending);

        let jobs = client.jobs().await;
        assert_eq!(jobs.len(), 1);
        assert!(jobs.contains_key("job-001"));
    }

    // ジョブを一時停止・再開・キャンセルした際にステータスが正しく変化することを確認する。
    #[tokio::test]
    async fn test_pause_resume_cancel() {
        let client = InMemorySchedulerClient::new();
        let created = client
            .create_job(JobRequest {
                name: "job".to_string(),
                schedule: Schedule::Cron("*/5 * * * *".to_string()),
                payload: json!({}),
                max_retries: 1,
                timeout_secs: 10,
            })
            .await
            .unwrap();

        client.pause_job(&created.id).await.unwrap();
        assert!(matches!(
            client.get_job(&created.id).await.unwrap().status,
            JobStatus::Paused
        ));

        client.resume_job(&created.id).await.unwrap();
        assert!(matches!(
            client.get_job(&created.id).await.unwrap().status,
            JobStatus::Pending
        ));

        client.cancel_job(&created.id).await.unwrap();
        assert!(matches!(
            client.get_job(&created.id).await.unwrap().status,
            JobStatus::Cancelled
        ));
    }
}
