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
    #[must_use]
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

    #[must_use]
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

    #[must_use]
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
    #[must_use]
    pub fn with_rule_info(mut self, rule_id: String, rule_name: String) -> Self {
        self.rule_id = rule_id;
        self.rule_name = rule_name;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// pass() はpassed=trueかつseverity=infoで生成される
    #[test]
    fn pass_creates_passed_result() {
        let r = RuleResult::pass();
        assert!(r.passed);
        assert_eq!(r.severity, "info");
        assert!(r.message.is_none());
        assert!(r.affected_record_ids.is_empty());
    }

    /// fail() はpassed=falseかつseverity=errorで生成される
    #[test]
    fn fail_creates_failed_result() {
        let r = RuleResult::fail("name is required".to_string());
        assert!(!r.passed);
        assert_eq!(r.severity, "error");
        assert_eq!(r.message, Some("name is required".to_string()));
    }

    /// warning() はpassed=trueかつseverity=warningで生成される
    #[test]
    fn warning_creates_warning_result() {
        let r = RuleResult::warning("possible duplicate".to_string());
        assert!(r.passed);
        assert_eq!(r.severity, "warning");
        assert!(r.message.is_some());
    }

    /// with_rule_info() でrule_idとrule_nameが設定される
    #[test]
    fn with_rule_info_sets_ids() {
        let r = RuleResult::pass().with_rule_info("r-001".to_string(), "my-rule".to_string());
        assert_eq!(r.rule_id, "r-001");
        assert_eq!(r.rule_name, "my-rule");
        assert!(r.passed);
    }

    /// fail() のメッセージがNone以外のことを確認する
    #[test]
    fn fail_message_is_some() {
        let r = RuleResult::fail(String::new());
        assert!(r.message.is_some());
    }
}
