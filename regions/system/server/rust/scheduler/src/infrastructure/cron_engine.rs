use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use tokio_util::sync::CancellationToken;

use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::repository::{SchedulerExecutionRepository, SchedulerJobRepository};
use crate::infrastructure::job_executor::JobExecutor;
use k1s0_distributed_lock::DistributedLock;

pub struct CronSchedulerEngine {
    job_repo: Arc<dyn SchedulerJobRepository>,
    execution_repo: Arc<dyn SchedulerExecutionRepository>,
    executor: Arc<dyn JobExecutor>,
    lock: Arc<dyn DistributedLock>,
    cancel_token: CancellationToken,
}

impl CronSchedulerEngine {
    pub fn new(
        job_repo: Arc<dyn SchedulerJobRepository>,
        execution_repo: Arc<dyn SchedulerExecutionRepository>,
        executor: Arc<dyn JobExecutor>,
        lock: Arc<dyn DistributedLock>,
    ) -> Self {
        Self {
            job_repo,
            execution_repo,
            executor,
            lock,
            cancel_token: CancellationToken::new(),
        }
    }

    #[must_use]
    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let job_repo = self.job_repo.clone();
        let execution_repo = self.execution_repo.clone();
        let executor = self.executor.clone();
        let lock = self.lock.clone();
        let token = self.cancel_token.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    () = token.cancelled() => break,
                    () = tokio::time::sleep(Duration::from_secs(60)) => {
                        if let Err(e) = Self::tick(&job_repo, &execution_repo, &executor, &lock).await {
                            tracing::error!("cron tick error: {}", e);
                        }
                    }
                }
            }
        })
    }

    async fn tick(
        job_repo: &Arc<dyn SchedulerJobRepository>,
        execution_repo: &Arc<dyn SchedulerExecutionRepository>,
        executor: &Arc<dyn JobExecutor>,
        lock: &Arc<dyn DistributedLock>,
    ) -> anyhow::Result<()> {
        let jobs = job_repo.find_active_jobs().await?;
        let now = Utc::now();

        for mut job in jobs {
            let scheduled_at = Self::due_at(&job, now);
            job.next_run_at = job.next_run_at();

            if let Some(scheduled_at) = scheduled_at {
                if scheduled_at <= now {
                    let lock_key = format!("scheduler:job:{}", job.id);
                    if let Ok(guard) = lock.acquire(&lock_key, Duration::from_secs(300)).await {
                        let execution = SchedulerExecution::new(job.id.clone());
                        let _ = execution_repo.create(&execution).await;

                        let result = executor.execute(&job).await;
                        let (status, error_message) = match result {
                            Ok(()) => ("succeeded".to_string(), None),
                            Err(err) => ("failed".to_string(), Some(err.to_string())),
                        };

                        job.last_run_at = Some(now);
                        job.next_run_at = job.next_run_at();
                        job.updated_at = now;
                        let _ = job_repo.update(&job).await;

                        let _ = execution_repo
                            .update_status(&execution.id, status, error_message)
                            .await;
                        let _ = lock.release(guard).await;
                    }
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn next_run_time(cron_expr: &str) -> Option<chrono::DateTime<Utc>> {
        Self::next_run_time_in_timezone(cron_expr, "UTC")
    }

    #[allow(dead_code)]
    fn next_run_time_in_timezone(cron_expr: &str, timezone: &str) -> Option<chrono::DateTime<Utc>> {
        // cron crate expects 6-field (with seconds) or 7-field expressions
        // Standard 5-field cron: prepend "0 " for seconds
        let full_expr = if cron_expr.split_whitespace().count() == 5 {
            format!("0 {cron_expr}")
        } else {
            cron_expr.to_string()
        };
        let tz = crate::domain::entity::scheduler_job::parse_timezone(timezone)?;
        Schedule::from_str(&full_expr)
            .ok()?
            .upcoming(tz)
            .next()
            .map(|dt| dt.with_timezone(&Utc))
    }

    fn due_at(
        job: &crate::domain::entity::scheduler_job::SchedulerJob,
        now: chrono::DateTime<Utc>,
    ) -> Option<chrono::DateTime<Utc>> {
        let full_expr = if job.cron_expression.split_whitespace().count() == 5 {
            format!("0 {}", job.cron_expression)
        } else {
            job.cron_expression.clone()
        };
        let schedule = Schedule::from_str(&full_expr).ok()?;
        let tz = crate::domain::entity::scheduler_job::parse_timezone(&job.timezone)?;
        let last_anchor = job.last_run_at.or(job
            .created_at
            .checked_sub_signed(chrono::Duration::seconds(1)))?;
        let last_anchor = last_anchor.with_timezone(&tz);
        let next_due = schedule.after(&last_anchor).next()?.with_timezone(&Utc);
        (next_due <= now).then_some(next_due)
    }

    pub fn stop(&self) {
        self.cancel_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_run_time_valid() {
        // cron クレートは6フィールド（秒含む）のため "0 * * * * *" 形式
        let result = CronSchedulerEngine::next_run_time("0 * * * * *");
        assert!(result.is_some());
    }

    #[test]
    fn test_next_run_time_invalid() {
        let result = CronSchedulerEngine::next_run_time("invalid cron");
        assert!(result.is_none());
    }

    #[test]
    fn test_due_at_after_last_run() {
        let mut job = crate::domain::entity::scheduler_job::SchedulerJob::new(
            "test".to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        let now = Utc::now();
        job.created_at = now - chrono::Duration::minutes(5);
        job.last_run_at = Some(now - chrono::Duration::minutes(2));
        assert!(CronSchedulerEngine::due_at(&job, now).is_some());
    }
}
