use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum AccessAction {
    Read,
    Write,
    Delete,
    List,
}

#[derive(Debug, Clone)]
pub struct SecretAccessLog {
    pub id: Uuid,
    pub path: String,
    pub action: AccessAction,
    pub subject: Option<String>,
    pub tenant_id: Option<String>,
    pub ip_address: Option<String>,
    pub trace_id: Option<String>,
    pub success: bool,
    pub error_msg: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl SecretAccessLog {
    pub fn new(
        path: String,
        action: AccessAction,
        subject: Option<String>,
        success: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            path,
            action,
            subject,
            tenant_id: None,
            ip_address: None,
            trace_id: None,
            success,
            error_msg: None,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_access_log() {
        let log = SecretAccessLog::new(
            "app/db/password".to_string(),
            AccessAction::Read,
            Some("user-1".to_string()),
            true,
        );

        assert_eq!(log.path, "app/db/password");
        assert_eq!(log.action, AccessAction::Read);
        assert_eq!(log.subject, Some("user-1".to_string()));
        assert!(log.success);
        assert!(log.error_msg.is_none());
        assert!(log.tenant_id.is_none());
    }
}
