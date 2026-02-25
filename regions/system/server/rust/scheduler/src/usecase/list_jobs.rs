use std::sync::Arc;

use crate::domain::entity::scheduler_job::SchedulerJob;
use crate::domain::repository::SchedulerJobRepository;

pub struct ListJobsInput {
    pub status: Option<String>,
    pub page: u32,
    pub page_size: u32,
}

pub struct ListJobsOutput {
    pub jobs: Vec<SchedulerJob>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

pub struct ListJobsUseCase {
    repo: Arc<dyn SchedulerJobRepository>,
}

impl ListJobsUseCase {
    pub fn new(repo: Arc<dyn SchedulerJobRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, input: &ListJobsInput) -> anyhow::Result<ListJobsOutput> {
        let all_jobs = self.repo.find_all().await?;
        let filtered: Vec<_> = all_jobs
            .into_iter()
            .filter(|j| {
                if let Some(ref s) = input.status {
                    &j.status == s
                } else {
                    true
                }
            })
            .collect();
        let total_count = filtered.len() as u64;
        let start = ((input.page - 1) * input.page_size) as usize;
        let jobs: Vec<_> = filtered
            .into_iter()
            .skip(start)
            .take(input.page_size as usize)
            .collect();
        let has_next = (start + input.page_size as usize) < total_count as usize;
        Ok(ListJobsOutput {
            jobs,
            total_count,
            page: input.page,
            page_size: input.page_size,
            has_next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::scheduler_job::SchedulerJob;
    use crate::domain::repository::scheduler_job_repository::MockSchedulerJobRepository;

    fn make_job(name: &str, status: &str) -> SchedulerJob {
        let mut job = SchedulerJob::new(
            name.to_string(),
            "* * * * *".to_string(),
            serde_json::json!({}),
        );
        job.status = status.to_string();
        job
    }

    #[tokio::test]
    async fn list_all_no_filter() {
        let mut mock = MockSchedulerJobRepository::new();
        mock.expect_find_all().returning(|| {
            Ok(vec![
                make_job("job-1", "active"),
                make_job("job-2", "paused"),
            ])
        });

        let uc = ListJobsUseCase::new(Arc::new(mock));
        let input = ListJobsInput {
            status: None,
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 2);
        assert_eq!(output.jobs.len(), 2);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn list_with_status_filter() {
        let mut mock = MockSchedulerJobRepository::new();
        mock.expect_find_all().returning(|| {
            Ok(vec![
                make_job("job-1", "active"),
                make_job("job-2", "paused"),
                make_job("job-3", "active"),
            ])
        });

        let uc = ListJobsUseCase::new(Arc::new(mock));
        let input = ListJobsInput {
            status: Some("active".to_string()),
            page: 1,
            page_size: 20,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 2);
        assert_eq!(output.jobs.len(), 2);
        assert!(!output.has_next);
    }

    #[tokio::test]
    async fn pagination() {
        let mut mock = MockSchedulerJobRepository::new();
        mock.expect_find_all().returning(|| {
            Ok((0..5)
                .map(|i| make_job(&format!("job-{}", i), "active"))
                .collect())
        });

        let uc = ListJobsUseCase::new(Arc::new(mock));
        let input = ListJobsInput {
            status: None,
            page: 1,
            page_size: 3,
        };
        let output = uc.execute(&input).await.unwrap();
        assert_eq!(output.total_count, 5);
        assert_eq!(output.jobs.len(), 3);
        assert!(output.has_next);
    }
}
