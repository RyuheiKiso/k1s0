use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::workflow_step::WorkflowStep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub enabled: bool,
    pub steps: Vec<WorkflowStep>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkflowDefinition {
    pub fn new(
        id: String,
        name: String,
        description: String,
        enabled: bool,
        steps: Vec<WorkflowStep>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            description,
            version: 1,
            enabled,
            steps,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn find_step(&self, step_id: &str) -> Option<&WorkflowStep> {
        self.steps.iter().find(|s| s.step_id == step_id)
    }

    pub fn first_step(&self) -> Option<&WorkflowStep> {
        self.steps.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_steps() -> Vec<WorkflowStep> {
        vec![
            WorkflowStep::new(
                "step-1".to_string(),
                "Approval".to_string(),
                "human_task".to_string(),
                Some("manager".to_string()),
                Some(48),
                Some("step-2".to_string()),
                Some("end".to_string()),
            ),
            WorkflowStep::new(
                "step-2".to_string(),
                "Finance".to_string(),
                "human_task".to_string(),
                Some("finance".to_string()),
                Some(72),
                Some("end".to_string()),
                Some("step-1".to_string()),
            ),
        ]
    }

    #[test]
    fn new_definition() {
        let def = WorkflowDefinition::new(
            "wf_001".to_string(),
            "purchase-approval".to_string(),
            "Purchase approval flow".to_string(),
            true,
            sample_steps(),
        );
        assert_eq!(def.id, "wf_001");
        assert_eq!(def.name, "purchase-approval");
        assert_eq!(def.version, 1);
        assert!(def.enabled);
        assert_eq!(def.step_count(), 2);
    }

    #[test]
    fn find_step_exists() {
        let def = WorkflowDefinition::new(
            "wf_001".to_string(),
            "test".to_string(),
            "".to_string(),
            true,
            sample_steps(),
        );
        let step = def.find_step("step-2");
        assert!(step.is_some());
        assert_eq!(step.unwrap().name, "Finance");
    }

    #[test]
    fn find_step_not_exists() {
        let def = WorkflowDefinition::new(
            "wf_001".to_string(),
            "test".to_string(),
            "".to_string(),
            true,
            sample_steps(),
        );
        assert!(def.find_step("nonexistent").is_none());
    }

    #[test]
    fn first_step() {
        let def = WorkflowDefinition::new(
            "wf_001".to_string(),
            "test".to_string(),
            "".to_string(),
            true,
            sample_steps(),
        );
        let first = def.first_step().unwrap();
        assert_eq!(first.step_id, "step-1");
    }

    #[test]
    fn first_step_empty() {
        let def = WorkflowDefinition::new(
            "wf_001".to_string(),
            "test".to_string(),
            "".to_string(),
            true,
            vec![],
        );
        assert!(def.first_step().is_none());
    }
}
