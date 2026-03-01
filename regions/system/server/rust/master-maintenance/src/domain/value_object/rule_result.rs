use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub passed: bool,
    pub message: Option<String>,
    pub severity: String,
    pub affected_record_ids: Vec<String>,
}

impl RuleResult {
    pub fn pass() -> Self {
        Self {
            passed: true,
            message: None,
            severity: "info".to_string(),
            affected_record_ids: vec![],
        }
    }

    pub fn fail(message: String) -> Self {
        Self {
            passed: false,
            message: Some(message),
            severity: "error".to_string(),
            affected_record_ids: vec![],
        }
    }

    pub fn warning(message: String) -> Self {
        Self {
            passed: true,
            message: Some(message),
            severity: "warning".to_string(),
            affected_record_ids: vec![],
        }
    }
}
