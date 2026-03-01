use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    pub rule_id: String,
    pub rule_name: String,
    pub passed: bool,
    pub message: Option<String>,
    pub severity: String,
    pub affected_record_ids: Vec<String>,
}

impl RuleResult {
    pub fn pass() -> Self {
        Self {
            rule_id: String::new(),
            rule_name: String::new(),
            passed: true,
            message: None,
            severity: "info".to_string(),
            affected_record_ids: vec![],
        }
    }

    pub fn fail(message: String) -> Self {
        Self {
            rule_id: String::new(),
            rule_name: String::new(),
            passed: false,
            message: Some(message),
            severity: "error".to_string(),
            affected_record_ids: vec![],
        }
    }

    pub fn warning(message: String) -> Self {
        Self {
            rule_id: String::new(),
            rule_name: String::new(),
            passed: true,
            message: Some(message),
            severity: "warning".to_string(),
            affected_record_ids: vec![],
        }
    }

    /// ルール情報を設定して返す
    pub fn with_rule_info(mut self, rule_id: String, rule_name: String) -> Self {
        self.rule_id = rule_id;
        self.rule_name = rule_name;
        self
    }
}
