use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum ProvisioningStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl ProvisioningStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ProvisioningStatus::Pending => "pending",
            ProvisioningStatus::Running => "running",
            ProvisioningStatus::Completed => "completed",
            ProvisioningStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProvisioningJob {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub status: ProvisioningStatus,
    pub current_step: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provisioning_job_creation() {
        let now = Utc::now();
        let job = ProvisioningJob {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            status: ProvisioningStatus::Pending,
            current_step: None,
            error_message: None,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(job.status, ProvisioningStatus::Pending);
        assert!(job.current_step.is_none());
    }

    #[test]
    fn test_provisioning_status_as_str() {
        assert_eq!(ProvisioningStatus::Pending.as_str(), "pending");
        assert_eq!(ProvisioningStatus::Running.as_str(), "running");
        assert_eq!(ProvisioningStatus::Completed.as_str(), "completed");
        assert_eq!(ProvisioningStatus::Failed.as_str(), "failed");
    }
}
