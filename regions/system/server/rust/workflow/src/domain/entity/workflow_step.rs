use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowStep {
    pub step_id: String,
    pub name: String,
    pub step_type: String,
    pub assignee_role: Option<String>,
    pub timeout_hours: Option<u32>,
    pub on_approve: Option<String>,
    pub on_reject: Option<String>,
}

impl WorkflowStep {
    pub fn new(
        step_id: String,
        name: String,
        step_type: String,
        assignee_role: Option<String>,
        timeout_hours: Option<u32>,
        on_approve: Option<String>,
        on_reject: Option<String>,
    ) -> Self {
        Self {
            step_id,
            name,
            step_type,
            assignee_role,
            timeout_hours,
            on_approve,
            on_reject,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_human_task_step() {
        let step = WorkflowStep::new(
            "step-1".to_string(),
            "Manager Approval".to_string(),
            "human_task".to_string(),
            Some("dept_manager".to_string()),
            Some(48),
            Some("step-2".to_string()),
            Some("end".to_string()),
        );
        assert_eq!(step.step_id, "step-1");
        assert_eq!(step.name, "Manager Approval");
        assert_eq!(step.step_type, "human_task");
        assert_eq!(step.assignee_role, Some("dept_manager".to_string()));
        assert_eq!(step.timeout_hours, Some(48));
        assert_eq!(step.on_approve, Some("step-2".to_string()));
        assert_eq!(step.on_reject, Some("end".to_string()));
    }

    #[test]
    fn new_automated_step() {
        let step = WorkflowStep::new(
            "step-auto".to_string(),
            "Auto Check".to_string(),
            "automated".to_string(),
            None,
            None,
            Some("step-2".to_string()),
            None,
        );
        assert_eq!(step.step_type, "automated");
        assert!(step.assignee_role.is_none());
        assert!(step.timeout_hours.is_none());
    }
}
