pub mod client;
pub mod error;
pub mod job;

pub use client::SchedulerClient;
pub use error::SchedulerError;
pub use job::{
    Job, JobCompletedEvent, JobExecution, JobFilter, JobRequest, JobStatus, Schedule,
};

#[cfg(feature = "mock")]
pub use client::MockSchedulerClient;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;

    #[test]
    fn test_schedule_cron_variant() {
        let schedule = Schedule::Cron("0 2 * * *".to_string());
        assert!(matches!(schedule, Schedule::Cron(_)));
    }

    #[test]
    fn test_schedule_one_shot_variant() {
        let at = Utc::now();
        let schedule = Schedule::OneShot(at);
        assert!(matches!(schedule, Schedule::OneShot(_)));
    }

    #[test]
    fn test_schedule_interval_variant() {
        let schedule = Schedule::Interval(Duration::from_secs(600));
        assert!(matches!(schedule, Schedule::Interval(_)));
    }

    #[test]
    fn test_job_status_pending() {
        let status = JobStatus::Pending;
        assert!(matches!(status, JobStatus::Pending));
    }

    #[test]
    fn test_job_status_all_variants() {
        let statuses = vec![
            JobStatus::Pending,
            JobStatus::Running,
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Paused,
            JobStatus::Cancelled,
        ];
        assert_eq!(statuses.len(), 6);
    }

    #[test]
    fn test_scheduler_error_job_not_found() {
        let err = SchedulerError::JobNotFound("job-999".to_string());
        assert!(matches!(err, SchedulerError::JobNotFound(_)));
        assert!(err.to_string().contains("job-999"));
    }

    #[test]
    fn test_scheduler_error_invalid_schedule() {
        let err = SchedulerError::InvalidSchedule("bad cron".to_string());
        assert!(matches!(err, SchedulerError::InvalidSchedule(_)));
    }

    #[test]
    fn test_scheduler_error_timeout() {
        let err = SchedulerError::Timeout;
        assert!(matches!(err, SchedulerError::Timeout));
    }

    #[test]
    fn test_job_filter_builder() {
        let filter = JobFilter::new()
            .status(JobStatus::Running)
            .name_prefix("daily");
        assert!(matches!(filter.status, Some(JobStatus::Running)));
        assert_eq!(filter.name_prefix.as_deref(), Some("daily"));
    }

    #[test]
    fn test_job_request_creation() {
        let req = JobRequest {
            name: "test".to_string(),
            schedule: Schedule::Cron("* * * * *".to_string()),
            payload: serde_json::json!({"key": "value"}),
            max_retries: 3,
            timeout_secs: 60,
        };
        assert_eq!(req.name, "test");
        assert_eq!(req.max_retries, 3);
    }

    #[test]
    fn test_job_completed_event() {
        let event = JobCompletedEvent {
            job_id: "job-1".to_string(),
            execution_id: "exec-1".to_string(),
            completed_at: Utc::now(),
            result: "success".to_string(),
        };
        assert_eq!(event.job_id, "job-1");
        assert_eq!(event.result, "success");
    }

    #[test]
    fn test_job_execution() {
        let exec = JobExecution {
            id: "exec-1".to_string(),
            job_id: "job-1".to_string(),
            started_at: Utc::now(),
            finished_at: Some(Utc::now()),
            result: "success".to_string(),
            error: None,
        };
        assert_eq!(exec.id, "exec-1");
        assert!(exec.error.is_none());
    }
}
