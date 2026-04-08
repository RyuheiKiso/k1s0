use chrono::{DateTime, Duration, Utc};

pub struct WorkflowDomainService;

impl WorkflowDomainService {
    #[must_use] 
    pub fn can_start_workflow(enabled: bool) -> bool {
        enabled
    }

    #[must_use] 
    pub fn task_due_at(timeout_hours: Option<u32>) -> Option<DateTime<Utc>> {
        timeout_hours.map(|hours| Utc::now() + Duration::hours(i64::from(hours)))
    }

    #[must_use] 
    pub fn next_step_on_approve(
        definition: &crate::domain::entity::workflow_definition::WorkflowDefinition,
        step_id: &str,
    ) -> Option<String> {
        definition
            .find_step(step_id)
            .and_then(|step| step.on_approve.clone())
    }

    #[must_use] 
    pub fn next_step_on_reject(
        definition: &crate::domain::entity::workflow_definition::WorkflowDefinition,
        step_id: &str,
    ) -> Option<String> {
        definition
            .find_step(step_id)
            .and_then(|step| step.on_reject.clone())
    }

    #[must_use] 
    pub fn is_terminal_step(next_step_id: Option<&str>) -> bool {
        matches!(next_step_id, Some("end") | None)
    }
}
