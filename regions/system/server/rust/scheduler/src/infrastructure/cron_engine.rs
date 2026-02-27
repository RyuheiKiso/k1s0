use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;
use tokio_util::sync::CancellationToken;

use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::repository::{SchedulerExecutionRepository, SchedulerJobRepository};
use k1s0_distributed_lock::DistributedLock;

pub struct CronSchedulerEngine {
    job_repo: Arc<dyn SchedulerJobRepository>,
    execution_repo: Arc<dyn SchedulerExecutionRepository>,
    lock: Arc<dyn DistributedLock>,
    cancel_token: CancellationToken,
}

impl CronSchedulerEngine {
    pub fn new(
        job_repo: Arc<dyn SchedulerJobRepository>,
        execution_repo: Arc<dyn SchedulerExecutionRepository>,
        lock: Arc<dyn DistributedLock>,
    ) -> Self {
        Self {
            job_repo,
            execution_repo,
            lock,
            cancel_token: CancellationToken::new(),
        }
    }

    pub fn start(&self) -> tokio::task::JoinHandle<()> {
        let job_repo = self.job_repo.clone();
        let execution_repo = self.execution_repo.clone();
        let lock = self.lock.clone();
        let token = self.cancel_token.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = token.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_secs(60)) => {
                        if let Err(e) = Self::tick(&job_repo, &execution_repo, &lock).await {
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
        lock: &Arc<dyn DistributedLock>,
    ) -> anyhow::Result<()> {
        let jobs = job_repo.find_active_jobs().await?;
        let now = Utc::now();

        for job in jobs {
            if let Some(next_run) = Self::next_run_time(&job.cron_expression) {
                if next_run <= now {
                    let lock_key = format!("scheduler:job:{}", job.id);
                    if let Ok(guard) =
                        lock.acquire(&lock_key, Duration::from_secs(300)).await
                    {
                        let execution = SchedulerExecution::new(job.id);
                        let _ = execution_repo.create(&execution).await;
                        let _ = execution_repo
                            .update_status(
                                &execution.id,
                                "completed".to_string(),
                                None,
                            )
                            .await;
                        let _ = lock.release(guard).await;
                    }
                }
            }
        }
        Ok(())
    }

    fn next_run_time(cron_expr: &str) -> Option<chrono::DateTime<Utc>> {
        // cron crate expects 6-field (with seconds) or 7-field expressions
        // Standard 5-field cron: prepend "0 " for seconds
        let full_expr = if cron_expr.split_whitespace().count() == 5 {
            format!("0 {}", cron_expr)
        } else {
            cron_expr.to_string()
        };
        Schedule::from_str(&full_expr).ok()?.upcoming(Utc).next()
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
}
