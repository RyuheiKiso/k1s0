use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::entity::scheduler_job::validate_cron;

pub struct SchedulerDomainService;

impl SchedulerDomainService {
    #[must_use] 
    pub fn validate_cron_expression(expression: &str) -> bool {
        validate_cron(expression)
    }

    #[must_use] 
    pub fn can_trigger(job_status: &str) -> bool {
        job_status == "active"
    }

    #[must_use] 
    pub fn has_running_execution(executions: &[SchedulerExecution]) -> bool {
        executions
            .iter()
            .any(|execution| execution.status == "running")
    }
}
