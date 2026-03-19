#![allow(clippy::unwrap_used)]
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;

use k1s0_scheduler_server::domain::entity::scheduler_execution::SchedulerExecution;
use k1s0_scheduler_server::domain::entity::scheduler_job::SchedulerJob;
use k1s0_scheduler_server::domain::repository::{
    SchedulerExecutionRepository, SchedulerJobRepository,
};
use k1s0_scheduler_server::infrastructure::job_executor::JobExecutor;
use k1s0_scheduler_server::infrastructure::kafka_producer::SchedulerEventPublisher;

// ---------------------------------------------------------------------------
// In-memory stub: SchedulerJobRepository
// ---------------------------------------------------------------------------

struct StubJobRepository {
    jobs: RwLock<Vec<SchedulerJob>>,
    should_fail: bool,
}

impl StubJobRepository {
    fn new() -> Self {
        Self {
            jobs: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_jobs(jobs: Vec<SchedulerJob>) -> Self {
        Self {
            jobs: RwLock::new(jobs),
            should_fail: false,
        }
    }

    fn failing() -> Self {
        Self {
            jobs: RwLock::new(Vec::new()),
            should_fail: true,
        }
    }
}

#[async_trait]
impl SchedulerJobRepository for StubJobRepository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<SchedulerJob>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let jobs = self.jobs.read().await;
        Ok(jobs.iter().find(|j| j.id == id).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<SchedulerJob>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        Ok(self.jobs.read().await.clone())
    }

    async fn create(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        self.jobs.write().await.push(job.clone());
        Ok(())
    }

    async fn update(&self, job: &SchedulerJob) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut jobs = self.jobs.write().await;
        if let Some(existing) = jobs.iter_mut().find(|j| j.id == job.id) {
            *existing = job.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<bool> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut jobs = self.jobs.write().await;
        let len_before = jobs.len();
        jobs.retain(|j| j.id != id);
        Ok(jobs.len() < len_before)
    }

    async fn find_active_jobs(&self) -> anyhow::Result<Vec<SchedulerJob>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let jobs = self.jobs.read().await;
        Ok(jobs
            .iter()
            .filter(|j| j.status == "active")
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: SchedulerExecutionRepository
// ---------------------------------------------------------------------------

struct StubExecutionRepository {
    executions: RwLock<Vec<SchedulerExecution>>,
    should_fail: bool,
}

impl StubExecutionRepository {
    fn new() -> Self {
        Self {
            executions: RwLock::new(Vec::new()),
            should_fail: false,
        }
    }

    fn with_executions(executions: Vec<SchedulerExecution>) -> Self {
        Self {
            executions: RwLock::new(executions),
            should_fail: false,
        }
    }
}

#[async_trait]
impl SchedulerExecutionRepository for StubExecutionRepository {
    async fn create(&self, execution: &SchedulerExecution) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        self.executions.write().await.push(execution.clone());
        Ok(())
    }

    async fn find_by_job_id(&self, job_id: &str) -> anyhow::Result<Vec<SchedulerExecution>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let execs = self.executions.read().await;
        Ok(execs
            .iter()
            .filter(|e| e.job_id == job_id)
            .cloned()
            .collect())
    }

    async fn update_status(
        &self,
        id: &str,
        status: String,
        error_message: Option<String>,
    ) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let mut execs = self.executions.write().await;
        if let Some(exec) = execs.iter_mut().find(|e| e.id == id) {
            exec.status = status;
            exec.error_message = error_message;
            exec.finished_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<SchedulerExecution>> {
        if self.should_fail {
            anyhow::bail!("db error");
        }
        let execs = self.executions.read().await;
        Ok(execs.iter().find(|e| e.id == id).cloned())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: SchedulerEventPublisher
// ---------------------------------------------------------------------------

struct StubEventPublisher {
    should_fail: bool,
}

impl StubEventPublisher {
    fn new() -> Self {
        Self { should_fail: false }
    }

    fn failing() -> Self {
        Self { should_fail: true }
    }
}

#[async_trait]
impl SchedulerEventPublisher for StubEventPublisher {
    async fn publish_job_executed(
        &self,
        _job: &SchedulerJob,
        _execution: &SchedulerExecution,
    ) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("kafka unavailable");
        }
        Ok(())
    }

    async fn publish_job_created(&self, _job: &SchedulerJob) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("kafka unavailable");
        }
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// In-memory stub: JobExecutor
// ---------------------------------------------------------------------------

struct StubJobExecutor {
    should_fail: bool,
}

impl StubJobExecutor {
    fn new() -> Self {
        Self { should_fail: false }
    }

    fn failing() -> Self {
        Self { should_fail: true }
    }
}

#[async_trait]
impl JobExecutor for StubJobExecutor {
    async fn execute(&self, _job: &SchedulerJob) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("target execution failed");
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_job(name: &str, status: &str) -> SchedulerJob {
    let mut job = SchedulerJob::new(
        name.to_string(),
        "*/5 * * * *".to_string(),
        serde_json::json!({"task": name}),
    );
    job.status = status.to_string();
    job
}

fn make_execution(job_id: &str, status: &str) -> SchedulerExecution {
    let mut exec = SchedulerExecution::new(job_id.to_string());
    exec.status = status.to_string();
    if status != "running" {
        exec.finished_at = Some(Utc::now());
    }
    exec
}

// ===========================================================================
// CreateJobUseCase tests
// ===========================================================================

mod create_job {
    use super::*;
    use k1s0_scheduler_server::usecase::create_job::{
        CreateJobError, CreateJobInput, CreateJobUseCase,
    };

    fn default_input() -> CreateJobInput {
        CreateJobInput {
            name: "daily-backup".to_string(),
            description: Some("Run daily backup".to_string()),
            cron_expression: "0 2 * * *".to_string(),
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: Some("scheduler.jobs.backup".to_string()),
            payload: serde_json::json!({"task": "backup"}),
        }
    }

    #[tokio::test]
    async fn success_creates_job_with_correct_fields() {
        let repo = Arc::new(StubJobRepository::new());
        let publisher = Arc::new(StubEventPublisher::new());
        let uc = CreateJobUseCase::new(repo.clone(), publisher);

        let input = default_input();
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let job = result.unwrap();
        assert_eq!(job.name, "daily-backup");
        assert_eq!(job.description.as_deref(), Some("Run daily backup"));
        assert_eq!(job.cron_expression, "0 2 * * *");
        assert_eq!(job.timezone, "UTC");
        assert_eq!(job.target_type, "kafka");
        assert_eq!(job.target.as_deref(), Some("scheduler.jobs.backup"));
        assert_eq!(job.status, "active");
        assert!(job.next_run_at.is_some());
        assert!(job.id.starts_with("job_"));

        // Verify persisted in repository
        let jobs = repo.jobs.read().await;
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].id, job.id);
    }

    #[tokio::test]
    async fn success_with_no_description() {
        let repo = Arc::new(StubJobRepository::new());
        let publisher = Arc::new(StubEventPublisher::new());
        let uc = CreateJobUseCase::new(repo, publisher);

        let mut input = default_input();
        input.description = None;
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert!(result.unwrap().description.is_none());
    }

    #[tokio::test]
    async fn invalid_cron_expression_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let publisher = Arc::new(StubEventPublisher::new());
        let uc = CreateJobUseCase::new(repo, publisher);

        let mut input = default_input();
        input.cron_expression = "invalid cron".to_string();
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(CreateJobError::InvalidCron(ref e)) if e == "invalid cron"));
    }

    #[tokio::test]
    async fn empty_cron_expression_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let publisher = Arc::new(StubEventPublisher::new());
        let uc = CreateJobUseCase::new(repo, publisher);

        let mut input = default_input();
        input.cron_expression = "".to_string();
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(CreateJobError::InvalidCron(_))));
    }

    #[tokio::test]
    async fn invalid_timezone_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let publisher = Arc::new(StubEventPublisher::new());
        let uc = CreateJobUseCase::new(repo, publisher);

        let mut input = default_input();
        input.timezone = "Invalid/Zone".to_string();
        let result = uc.execute(&input).await;
        assert!(
            matches!(result, Err(CreateJobError::InvalidTimezone(ref tz)) if tz == "Invalid/Zone")
        );
    }

    #[tokio::test]
    async fn valid_non_utc_timezone_succeeds() {
        let repo = Arc::new(StubJobRepository::new());
        let publisher = Arc::new(StubEventPublisher::new());
        let uc = CreateJobUseCase::new(repo, publisher);

        let mut input = default_input();
        input.timezone = "Asia/Tokyo".to_string();
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().timezone, "Asia/Tokyo");
    }

    #[tokio::test]
    async fn repository_error_returns_internal_error() {
        let repo = Arc::new(StubJobRepository::failing());
        let publisher = Arc::new(StubEventPublisher::new());
        let uc = CreateJobUseCase::new(repo, publisher);

        let input = default_input();
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(CreateJobError::Internal(_))));
    }

    #[tokio::test]
    async fn publisher_failure_does_not_fail_create() {
        let repo = Arc::new(StubJobRepository::new());
        let publisher = Arc::new(StubEventPublisher::failing());
        let uc = CreateJobUseCase::new(repo.clone(), publisher);

        let input = default_input();
        let result = uc.execute(&input).await;
        assert!(
            result.is_ok(),
            "create should succeed even if publisher fails"
        );

        let jobs = repo.jobs.read().await;
        assert_eq!(jobs.len(), 1);
    }
}

// ===========================================================================
// GetJobUseCase tests
// ===========================================================================

mod get_job {
    use super::*;
    use k1s0_scheduler_server::usecase::get_job::{GetJobError, GetJobUseCase};

    #[tokio::test]
    async fn found_returns_job() {
        let job = make_job("test-job", "active");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let uc = GetJobUseCase::new(repo);

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "test-job");
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let uc = GetJobUseCase::new(repo);

        let result = uc.execute("nonexistent").await;
        assert!(matches!(result, Err(GetJobError::NotFound(ref id)) if id == "nonexistent"));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let repo = Arc::new(StubJobRepository::failing());
        let uc = GetJobUseCase::new(repo);

        let result = uc.execute("any-id").await;
        assert!(matches!(result, Err(GetJobError::Internal(_))));
    }
}

// ===========================================================================
// ListJobsUseCase tests
// ===========================================================================

mod list_jobs {
    use super::*;
    use k1s0_scheduler_server::usecase::list_jobs::{ListJobsInput, ListJobsUseCase};

    #[tokio::test]
    async fn returns_all_jobs_without_filter() {
        let jobs = vec![
            make_job("job-1", "active"),
            make_job("job-2", "paused"),
            make_job("job-3", "active"),
        ];
        let repo = Arc::new(StubJobRepository::with_jobs(jobs));
        let uc = ListJobsUseCase::new(repo);

        let input = ListJobsInput {
            status: None,
            name_prefix: None,
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 3);
        assert_eq!(output.jobs.len(), 3);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn filters_by_status() {
        let jobs = vec![
            make_job("job-active-1", "active"),
            make_job("job-paused", "paused"),
            make_job("job-active-2", "active"),
        ];
        let repo = Arc::new(StubJobRepository::with_jobs(jobs));
        let uc = ListJobsUseCase::new(repo);

        let input = ListJobsInput {
            status: Some("active".to_string()),
            name_prefix: None,
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 2);
        assert!(output.jobs.iter().all(|j| j.status == "active"));
    }

    #[tokio::test]
    async fn filters_by_name_prefix() {
        let jobs = vec![
            make_job("workflow-check", "active"),
            make_job("daily-report", "active"),
            make_job("workflow-notify", "active"),
        ];
        let repo = Arc::new(StubJobRepository::with_jobs(jobs));
        let uc = ListJobsUseCase::new(repo);

        let input = ListJobsInput {
            status: None,
            name_prefix: Some("workflow-".to_string()),
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 2);
        assert!(output.jobs.iter().all(|j| j.name.starts_with("workflow-")));
    }

    #[tokio::test]
    async fn filters_by_status_and_name_prefix() {
        let jobs = vec![
            make_job("workflow-check", "active"),
            make_job("workflow-notify", "paused"),
            make_job("daily-report", "active"),
        ];
        let repo = Arc::new(StubJobRepository::with_jobs(jobs));
        let uc = ListJobsUseCase::new(repo);

        let input = ListJobsInput {
            status: Some("active".to_string()),
            name_prefix: Some("workflow-".to_string()),
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 1);
        assert_eq!(output.jobs[0].name, "workflow-check");
    }

    #[tokio::test]
    async fn pagination_first_page() {
        let jobs: Vec<_> = (0..7)
            .map(|i| make_job(&format!("job-{}", i), "active"))
            .collect();
        let repo = Arc::new(StubJobRepository::with_jobs(jobs));
        let uc = ListJobsUseCase::new(repo);

        let input = ListJobsInput {
            status: None,
            name_prefix: None,
            page: 1,
            page_size: 3,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 7);
        assert_eq!(output.jobs.len(), 3);
        assert!(output.has_next);
        assert_eq!(output.page, 1);
        assert_eq!(output.page_size, 3);
    }

    #[tokio::test]
    async fn pagination_last_page() {
        let jobs: Vec<_> = (0..7)
            .map(|i| make_job(&format!("job-{}", i), "active"))
            .collect();
        let repo = Arc::new(StubJobRepository::with_jobs(jobs));
        let uc = ListJobsUseCase::new(repo);

        let input = ListJobsInput {
            status: None,
            name_prefix: None,
            page: 3,
            page_size: 3,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 7);
        assert_eq!(output.jobs.len(), 1);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn empty_result() {
        let repo = Arc::new(StubJobRepository::new());
        let uc = ListJobsUseCase::new(repo);

        let input = ListJobsInput {
            status: None,
            name_prefix: None,
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 0);
        assert!(output.jobs.is_empty());
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn repository_error_propagates() {
        let repo = Arc::new(StubJobRepository::failing());
        let uc = ListJobsUseCase::new(repo);

        let input = ListJobsInput {
            status: None,
            name_prefix: None,
            page: 1,
            page_size: 20,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
    }
}

// ===========================================================================
// UpdateJobUseCase tests
// ===========================================================================

mod update_job {
    use super::*;
    use k1s0_scheduler_server::usecase::update_job::{
        UpdateJobError, UpdateJobInput, UpdateJobUseCase,
    };

    #[tokio::test]
    async fn success_updates_all_fields() {
        let job = make_job("original-job", "active");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let uc = UpdateJobUseCase::new(repo.clone());

        let input = UpdateJobInput {
            id: job_id.clone(),
            name: "updated-job".to_string(),
            description: Some("Updated description".to_string()),
            cron_expression: "0 12 * * *".to_string(),
            timezone: "America/New_York".to_string(),
            target_type: "http".to_string(),
            target: Some("https://example.com/webhook".to_string()),
            payload: serde_json::json!({"task": "updated"}),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name, "updated-job");
        assert_eq!(updated.description.as_deref(), Some("Updated description"));
        assert_eq!(updated.cron_expression, "0 12 * * *");
        assert_eq!(updated.timezone, "America/New_York");
        assert_eq!(updated.target_type, "http");
        assert_eq!(
            updated.target.as_deref(),
            Some("https://example.com/webhook")
        );
        assert!(updated.next_run_at.is_some());

        // Verify persisted
        let jobs = repo.jobs.read().await;
        assert_eq!(jobs[0].name, "updated-job");
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let uc = UpdateJobUseCase::new(repo);

        let input = UpdateJobInput {
            id: "nonexistent".to_string(),
            name: "test".to_string(),
            description: None,
            cron_expression: "* * * * *".to_string(),
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            payload: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(UpdateJobError::NotFound(ref id)) if id == "nonexistent"));
    }

    #[tokio::test]
    async fn invalid_cron_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let uc = UpdateJobUseCase::new(repo);

        let input = UpdateJobInput {
            id: "some-id".to_string(),
            name: "test".to_string(),
            description: None,
            cron_expression: "not-a-cron".to_string(),
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            payload: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(UpdateJobError::InvalidCron(ref e)) if e == "not-a-cron"));
    }

    #[tokio::test]
    async fn invalid_timezone_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let uc = UpdateJobUseCase::new(repo);

        let input = UpdateJobInput {
            id: "some-id".to_string(),
            name: "test".to_string(),
            description: None,
            cron_expression: "* * * * *".to_string(),
            timezone: "Bad/Zone".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            payload: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(UpdateJobError::InvalidTimezone(ref tz)) if tz == "Bad/Zone"));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let repo = Arc::new(StubJobRepository::failing());
        let uc = UpdateJobUseCase::new(repo);

        let input = UpdateJobInput {
            id: "any-id".to_string(),
            name: "test".to_string(),
            description: None,
            cron_expression: "* * * * *".to_string(),
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            payload: serde_json::json!({}),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(UpdateJobError::Internal(_))));
    }
}

// ===========================================================================
// DeleteJobUseCase tests
// ===========================================================================

mod delete_job {
    use super::*;
    use k1s0_scheduler_server::usecase::delete_job::{DeleteJobError, DeleteJobUseCase};

    #[tokio::test]
    async fn success_deletes_job() {
        let job = make_job("to-delete", "active");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let uc = DeleteJobUseCase::new(repo.clone(), exec_repo);

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());

        let jobs = repo.jobs.read().await;
        assert!(jobs.is_empty());
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let uc = DeleteJobUseCase::new(repo, exec_repo);

        let result = uc.execute("nonexistent").await;
        assert!(matches!(result, Err(DeleteJobError::NotFound(ref id)) if id == "nonexistent"));
    }

    #[tokio::test]
    async fn job_with_running_execution_cannot_be_deleted() {
        let job = make_job("running-job", "active");
        let job_id = job.id.clone();
        let running_exec = make_execution(&job_id, "running");
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let exec_repo = Arc::new(StubExecutionRepository::with_executions(vec![running_exec]));
        let uc = DeleteJobUseCase::new(repo, exec_repo);

        let result = uc.execute(&job_id).await;
        assert!(matches!(result, Err(DeleteJobError::JobRunning(ref id)) if id == &job_id));
    }

    #[tokio::test]
    async fn job_with_completed_execution_can_be_deleted() {
        let job = make_job("completed-job", "active");
        let job_id = job.id.clone();
        let completed_exec = make_execution(&job_id, "succeeded");
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let exec_repo = Arc::new(StubExecutionRepository::with_executions(vec![
            completed_exec,
        ]));
        let uc = DeleteJobUseCase::new(repo.clone(), exec_repo);

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());

        let jobs = repo.jobs.read().await;
        assert!(jobs.is_empty());
    }

    #[tokio::test]
    async fn job_with_failed_execution_can_be_deleted() {
        let job = make_job("failed-job", "active");
        let job_id = job.id.clone();
        let failed_exec = make_execution(&job_id, "failed");
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let exec_repo = Arc::new(StubExecutionRepository::with_executions(vec![failed_exec]));
        let uc = DeleteJobUseCase::new(repo.clone(), exec_repo);

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let repo = Arc::new(StubJobRepository::failing());
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let uc = DeleteJobUseCase::new(repo, exec_repo);

        let result = uc.execute("any-id").await;
        assert!(
            matches!(result, Err(DeleteJobError::Internal(ref msg)) if msg.contains("db error"))
        );
    }
}

// ===========================================================================
// PauseJobUseCase tests
// ===========================================================================

mod pause_job {
    use super::*;
    use k1s0_scheduler_server::usecase::pause_job::{PauseJobError, PauseJobUseCase};

    #[tokio::test]
    async fn success_pauses_active_job() {
        let job = make_job("active-job", "active");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let uc = PauseJobUseCase::new(repo.clone());

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());

        let paused = result.unwrap();
        assert_eq!(paused.status, "paused");

        // Verify persisted
        let jobs = repo.jobs.read().await;
        assert_eq!(jobs[0].status, "paused");
    }

    #[tokio::test]
    async fn pausing_already_paused_job_succeeds() {
        let job = make_job("paused-job", "paused");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let uc = PauseJobUseCase::new(repo);

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "paused");
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let uc = PauseJobUseCase::new(repo);

        let result = uc.execute("nonexistent").await;
        assert!(matches!(result, Err(PauseJobError::NotFound(ref id)) if id == "nonexistent"));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let repo = Arc::new(StubJobRepository::failing());
        let uc = PauseJobUseCase::new(repo);

        let result = uc.execute("any-id").await;
        assert!(matches!(result, Err(PauseJobError::Internal(_))));
    }
}

// ===========================================================================
// ResumeJobUseCase tests
// ===========================================================================

mod resume_job {
    use super::*;
    use k1s0_scheduler_server::usecase::resume_job::{ResumeJobError, ResumeJobUseCase};

    #[tokio::test]
    async fn success_resumes_paused_job() {
        let job = make_job("paused-job", "paused");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let uc = ResumeJobUseCase::new(repo.clone());

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());

        let resumed = result.unwrap();
        assert_eq!(resumed.status, "active");

        // Verify persisted
        let jobs = repo.jobs.read().await;
        assert_eq!(jobs[0].status, "active");
    }

    #[tokio::test]
    async fn resuming_already_active_job_succeeds() {
        let job = make_job("active-job", "active");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let uc = ResumeJobUseCase::new(repo);

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, "active");
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let uc = ResumeJobUseCase::new(repo);

        let result = uc.execute("nonexistent").await;
        assert!(matches!(result, Err(ResumeJobError::NotFound(ref id)) if id == "nonexistent"));
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let repo = Arc::new(StubJobRepository::failing());
        let uc = ResumeJobUseCase::new(repo);

        let result = uc.execute("any-id").await;
        assert!(matches!(result, Err(ResumeJobError::Internal(_))));
    }
}

// ===========================================================================
// ListExecutionsUseCase tests
// ===========================================================================

mod list_executions {
    use super::*;
    use k1s0_scheduler_server::usecase::list_executions::{
        ListExecutionsError, ListExecutionsUseCase,
    };

    #[tokio::test]
    async fn success_returns_executions_for_job() {
        let job = make_job("exec-job", "active");
        let job_id = job.id.clone();
        let exec1 = make_execution(&job_id, "succeeded");
        let exec2 = make_execution(&job_id, "failed");
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let exec_repo = Arc::new(StubExecutionRepository::with_executions(vec![exec1, exec2]));
        let uc = ListExecutionsUseCase::new(repo, exec_repo);

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn returns_empty_list_for_job_with_no_executions() {
        let job = make_job("no-exec-job", "active");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let uc = ListExecutionsUseCase::new(repo, exec_repo);

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn not_found_when_job_does_not_exist() {
        let repo = Arc::new(StubJobRepository::new());
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let uc = ListExecutionsUseCase::new(repo, exec_repo);

        let result = uc.execute("nonexistent").await;
        assert!(
            matches!(result, Err(ListExecutionsError::NotFound(ref id)) if id == "nonexistent")
        );
    }

    #[tokio::test]
    async fn only_returns_executions_for_requested_job() {
        let job_a = make_job("job-a", "active");
        let job_b = make_job("job-b", "active");
        let job_a_id = job_a.id.clone();
        let job_b_id = job_b.id.clone();
        let exec_a = make_execution(&job_a_id, "succeeded");
        let exec_b = make_execution(&job_b_id, "succeeded");
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job_a, job_b]));
        let exec_repo = Arc::new(StubExecutionRepository::with_executions(vec![
            exec_a, exec_b,
        ]));
        let uc = ListExecutionsUseCase::new(repo, exec_repo);

        let result = uc.execute(&job_a_id).await;
        assert!(result.is_ok());
        let execs = result.unwrap();
        assert_eq!(execs.len(), 1);
        assert_eq!(execs[0].job_id, job_a_id);
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let repo = Arc::new(StubJobRepository::failing());
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let uc = ListExecutionsUseCase::new(repo, exec_repo);

        let result = uc.execute("any-id").await;
        assert!(matches!(result, Err(ListExecutionsError::Internal(_))));
    }
}

// ===========================================================================
// TriggerJobUseCase tests
// ===========================================================================

mod trigger_job {
    use super::*;
    use k1s0_scheduler_server::usecase::trigger_job::{TriggerJobError, TriggerJobUseCase};

    #[tokio::test]
    async fn success_triggers_active_job() {
        let job = make_job("trigger-me", "active");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let executor = Arc::new(StubJobExecutor::new());
        let publisher = Arc::new(StubEventPublisher::new());
        let uc = TriggerJobUseCase::with_dependencies(
            repo.clone(),
            exec_repo.clone(),
            executor,
            publisher,
        );

        let result = uc.execute(&job_id).await;
        assert!(result.is_ok());

        let execution = result.unwrap();
        assert_eq!(execution.job_id, job_id);
        assert_eq!(execution.status, "succeeded");
        assert_eq!(execution.triggered_by, "manual");

        // Verify job was updated with last_run_at
        let jobs = repo.jobs.read().await;
        assert!(jobs[0].last_run_at.is_some());

        // Verify execution was persisted
        let execs = exec_repo.executions.read().await;
        assert_eq!(execs.len(), 1);
    }

    #[tokio::test]
    async fn not_found_returns_error() {
        let repo = Arc::new(StubJobRepository::new());
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let uc = TriggerJobUseCase::new(repo, exec_repo);

        let result = uc.execute("nonexistent").await;
        assert!(matches!(result, Err(TriggerJobError::NotFound(ref id)) if id == "nonexistent"));
    }

    #[tokio::test]
    async fn paused_job_returns_not_active() {
        let job = make_job("paused-job", "paused");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let uc = TriggerJobUseCase::new(repo, exec_repo);

        let result = uc.execute(&job_id).await;
        assert!(matches!(result, Err(TriggerJobError::NotActive(ref id)) if id == &job_id));
    }

    #[tokio::test]
    async fn executor_failure_returns_internal_error() {
        let job = make_job("fail-exec", "active");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let executor = Arc::new(StubJobExecutor::failing());
        let publisher = Arc::new(StubEventPublisher::new());
        let uc = TriggerJobUseCase::with_dependencies(repo, exec_repo, executor, publisher);

        let result = uc.execute(&job_id).await;
        assert!(
            matches!(result, Err(TriggerJobError::Internal(ref msg)) if msg.contains("target execution failed"))
        );
    }

    #[tokio::test]
    async fn publisher_failure_does_not_fail_trigger() {
        let job = make_job("pub-fail", "active");
        let job_id = job.id.clone();
        let repo = Arc::new(StubJobRepository::with_jobs(vec![job]));
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let executor = Arc::new(StubJobExecutor::new());
        let publisher = Arc::new(StubEventPublisher::failing());
        let uc = TriggerJobUseCase::with_dependencies(repo, exec_repo, executor, publisher);

        let result = uc.execute(&job_id).await;
        assert!(
            result.is_ok(),
            "trigger should succeed even if publisher fails"
        );
    }

    #[tokio::test]
    async fn repository_error_returns_internal() {
        let repo = Arc::new(StubJobRepository::failing());
        let exec_repo = Arc::new(StubExecutionRepository::new());
        let uc = TriggerJobUseCase::new(repo, exec_repo);

        let result = uc.execute("any-id").await;
        assert!(matches!(result, Err(TriggerJobError::Internal(_))));
    }
}

// ===========================================================================
// Pause-then-Resume lifecycle test
// ===========================================================================

mod lifecycle {
    use super::*;
    use k1s0_scheduler_server::usecase::create_job::{CreateJobInput, CreateJobUseCase};
    use k1s0_scheduler_server::usecase::delete_job::DeleteJobUseCase;
    use k1s0_scheduler_server::usecase::get_job::GetJobUseCase;
    use k1s0_scheduler_server::usecase::pause_job::PauseJobUseCase;
    use k1s0_scheduler_server::usecase::resume_job::ResumeJobUseCase;
    use k1s0_scheduler_server::usecase::trigger_job::TriggerJobUseCase;

    #[tokio::test]
    async fn full_job_lifecycle_create_pause_resume_trigger_delete() {
        let repo: Arc<StubJobRepository> = Arc::new(StubJobRepository::new());
        let exec_repo: Arc<StubExecutionRepository> = Arc::new(StubExecutionRepository::new());
        let publisher: Arc<StubEventPublisher> = Arc::new(StubEventPublisher::new());
        let executor: Arc<StubJobExecutor> = Arc::new(StubJobExecutor::new());

        // 1. Create a job
        let create_uc = CreateJobUseCase::new(repo.clone(), publisher.clone());
        let input = CreateJobInput {
            name: "lifecycle-test".to_string(),
            description: Some("Full lifecycle test".to_string()),
            cron_expression: "0 * * * *".to_string(),
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: Some("test-topic".to_string()),
            payload: serde_json::json!({"test": true}),
        };
        let job = create_uc.execute(&input).await.unwrap();
        let job_id = job.id.clone();
        assert_eq!(job.status, "active");

        // 2. Get the job
        let get_uc = GetJobUseCase::new(repo.clone());
        let retrieved = get_uc.execute(&job_id).await.unwrap();
        assert_eq!(retrieved.name, "lifecycle-test");

        // 3. Pause the job
        let pause_uc = PauseJobUseCase::new(repo.clone());
        let paused = pause_uc.execute(&job_id).await.unwrap();
        assert_eq!(paused.status, "paused");

        // 4. Trigger should fail when paused
        let trigger_uc = TriggerJobUseCase::with_dependencies(
            repo.clone(),
            exec_repo.clone(),
            executor.clone(),
            publisher.clone(),
        );
        let trigger_result = trigger_uc.execute(&job_id).await;
        assert!(trigger_result.is_err());

        // 5. Resume the job
        let resume_uc = ResumeJobUseCase::new(repo.clone());
        let resumed = resume_uc.execute(&job_id).await.unwrap();
        assert_eq!(resumed.status, "active");

        // 6. Trigger should succeed when active
        let trigger_result = trigger_uc.execute(&job_id).await;
        assert!(trigger_result.is_ok());

        // 7. Delete the job
        let delete_uc = DeleteJobUseCase::new(repo.clone(), exec_repo.clone());
        let delete_result = delete_uc.execute(&job_id).await;
        assert!(delete_result.is_ok());

        // 8. Verify deleted
        let get_result = get_uc.execute(&job_id).await;
        assert!(get_result.is_err());
    }
}
