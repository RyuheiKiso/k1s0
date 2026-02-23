use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTask {
    pub id: String,
    pub instance_id: String,
    pub step_id: String,
    pub step_name: String,
    pub assignee_id: Option<String>,
    pub status: String,
    pub due_at: Option<DateTime<Utc>>,
    pub comment: Option<String>,
    pub actor_id: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkflowTask {
    pub fn new(
        id: String,
        instance_id: String,
        step_id: String,
        step_name: String,
        assignee_id: Option<String>,
        due_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        let status = if assignee_id.is_some() {
            "assigned".to_string()
        } else {
            "pending".to_string()
        };
        Self {
            id,
            instance_id,
            step_id,
            step_name,
            assignee_id,
            status,
            due_at,
            comment: None,
            actor_id: None,
            decided_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_reassignable(&self) -> bool {
        matches!(self.status.as_str(), "pending" | "assigned")
    }

    pub fn is_decidable(&self) -> bool {
        matches!(self.status.as_str(), "pending" | "assigned")
    }

    pub fn approve(&mut self, actor_id: String, comment: Option<String>) {
        self.status = "approved".to_string();
        self.actor_id = Some(actor_id);
        self.comment = comment;
        self.decided_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn reject(&mut self, actor_id: String, comment: Option<String>) {
        self.status = "rejected".to_string();
        self.actor_id = Some(actor_id);
        self.comment = comment;
        self.decided_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn reassign(&mut self, new_assignee_id: String) {
        self.assignee_id = Some(new_assignee_id);
        self.status = "assigned".to_string();
        self.updated_at = Utc::now();
    }

    pub fn is_overdue(&self) -> bool {
        match self.due_at {
            Some(due) => Utc::now() > due && self.is_decidable(),
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_task() -> WorkflowTask {
        WorkflowTask::new(
            "task_001".to_string(),
            "inst_001".to_string(),
            "step-1".to_string(),
            "Manager Approval".to_string(),
            Some("user-002".to_string()),
            Some(Utc::now() + chrono::Duration::hours(48)),
        )
    }

    #[test]
    fn new_task_with_assignee() {
        let task = sample_task();
        assert_eq!(task.id, "task_001");
        assert_eq!(task.status, "assigned");
        assert!(task.assignee_id.is_some());
    }

    #[test]
    fn new_task_without_assignee() {
        let task = WorkflowTask::new(
            "task_002".to_string(),
            "inst_001".to_string(),
            "step-1".to_string(),
            "Approval".to_string(),
            None,
            None,
        );
        assert_eq!(task.status, "pending");
    }

    #[test]
    fn is_reassignable() {
        let task = sample_task();
        assert!(task.is_reassignable());
    }

    #[test]
    fn is_not_reassignable_after_approve() {
        let mut task = sample_task();
        task.approve("actor".to_string(), None);
        assert!(!task.is_reassignable());
    }

    #[test]
    fn approve_task() {
        let mut task = sample_task();
        task.approve("user-002".to_string(), Some("OK".to_string()));
        assert_eq!(task.status, "approved");
        assert_eq!(task.actor_id, Some("user-002".to_string()));
        assert_eq!(task.comment, Some("OK".to_string()));
        assert!(task.decided_at.is_some());
    }

    #[test]
    fn reject_task() {
        let mut task = sample_task();
        task.reject("user-002".to_string(), Some("Too expensive".to_string()));
        assert_eq!(task.status, "rejected");
        assert_eq!(task.comment, Some("Too expensive".to_string()));
    }

    #[test]
    fn reassign_task() {
        let mut task = sample_task();
        task.reassign("user-003".to_string());
        assert_eq!(task.assignee_id, Some("user-003".to_string()));
        assert_eq!(task.status, "assigned");
    }

    #[test]
    fn is_overdue_not_yet() {
        let task = sample_task();
        assert!(!task.is_overdue());
    }

    #[test]
    fn is_overdue_past_due() {
        let mut task = sample_task();
        task.due_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(task.is_overdue());
    }

    #[test]
    fn is_overdue_no_due_date() {
        let mut task = sample_task();
        task.due_at = None;
        assert!(!task.is_overdue());
    }
}
