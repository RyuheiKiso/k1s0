use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub title: String,
    pub initiator_id: String,
    pub current_step_id: Option<String>,
    pub status: String,
    pub context: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl WorkflowInstance {
    pub fn new(
        id: String,
        workflow_id: String,
        workflow_name: String,
        title: String,
        initiator_id: String,
        first_step_id: Option<String>,
        context: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            workflow_id,
            workflow_name,
            title,
            initiator_id,
            current_step_id: first_step_id,
            status: "running".to_string(),
            context,
            started_at: now,
            completed_at: None,
            created_at: now,
        }
    }

    pub fn is_cancellable(&self) -> bool {
        matches!(self.status.as_str(), "pending" | "running")
    }

    pub fn cancel(&mut self) {
        self.status = "cancelled".to_string();
        self.completed_at = Some(Utc::now());
        self.current_step_id = None;
    }

    pub fn complete(&mut self) {
        self.status = "completed".to_string();
        self.completed_at = Some(Utc::now());
        self.current_step_id = None;
    }

    pub fn fail(&mut self) {
        self.status = "failed".to_string();
        self.completed_at = Some(Utc::now());
        self.current_step_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_instance() -> WorkflowInstance {
        WorkflowInstance::new(
            "inst_001".to_string(),
            "wf_001".to_string(),
            "purchase-approval".to_string(),
            "PC Purchase".to_string(),
            "user-001".to_string(),
            Some("step-1".to_string()),
            serde_json::json!({"item": "laptop"}),
        )
    }

    #[test]
    fn new_instance() {
        let inst = sample_instance();
        assert_eq!(inst.id, "inst_001");
        assert_eq!(inst.status, "running");
        assert_eq!(inst.current_step_id, Some("step-1".to_string()));
        assert!(inst.completed_at.is_none());
    }

    #[test]
    fn is_cancellable_running() {
        let inst = sample_instance();
        assert!(inst.is_cancellable());
    }

    #[test]
    fn is_not_cancellable_after_complete() {
        let mut inst = sample_instance();
        inst.complete();
        assert!(!inst.is_cancellable());
    }

    #[test]
    fn cancel_instance() {
        let mut inst = sample_instance();
        inst.cancel();
        assert_eq!(inst.status, "cancelled");
        assert!(inst.completed_at.is_some());
        assert!(inst.current_step_id.is_none());
    }

    #[test]
    fn complete_instance() {
        let mut inst = sample_instance();
        inst.complete();
        assert_eq!(inst.status, "completed");
        assert!(inst.completed_at.is_some());
    }

    #[test]
    fn fail_instance() {
        let mut inst = sample_instance();
        inst.fail();
        assert_eq!(inst.status, "failed");
        assert!(inst.completed_at.is_some());
    }
}
