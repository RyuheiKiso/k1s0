use crate::domain::entity::scheduler_execution::SchedulerExecution;
use crate::domain::entity::scheduler_job::validate_cron;

pub struct SchedulerDomainService;

impl SchedulerDomainService {
    pub fn validate_cron_expression(expression: &str) -> bool {
        validate_cron(expression)
    }

    pub fn can_trigger(job_status: &str) -> bool {
        job_status == "active"
    }

    pub fn has_running_execution(executions: &[SchedulerExecution]) -> bool {
        executions
            .iter()
            .any(|execution| execution.status == "running")
    }
}
