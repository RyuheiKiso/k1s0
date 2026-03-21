use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PolicyEvaluation {
    pub id: Uuid,
    pub policy_id: Option<Uuid>,
    pub package_path: String,
    pub input: serde_json::Value,
    pub allowed: bool,
    pub reason: Option<String>,
    pub decision_id: String,
    pub cached: bool,
    pub evaluated_at: DateTime<Utc>,
}

impl PolicyEvaluation {
    pub fn new(
        policy_id: Option<Uuid>,
        package_path: String,
        input: serde_json::Value,
        allowed: bool,
        reason: Option<String>,
        decision_id: String,
        cached: bool,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            policy_id,
            package_path,
            input,
            allowed,
            reason,
            decision_id,
            cached,
            evaluated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PolicyEvaluation::new が allowed=true の評価結果を生成する
    #[test]
    fn new_allowed() {
        let eval = PolicyEvaluation::new(
            Some(Uuid::new_v4()),
            "data.rbac.allow".to_string(),
            serde_json::json!({"user": "alice", "action": "read"}),
            true,
            None,
            "dec-001".to_string(),
            false,
        );
        assert!(eval.allowed);
        assert!(!eval.cached);
        assert_eq!(eval.decision_id, "dec-001");
    }

    /// PolicyEvaluation::new が denied の評価結果を生成する
    #[test]
    fn new_denied_with_reason() {
        let eval = PolicyEvaluation::new(
            None,
            "data.rbac.allow".to_string(),
            serde_json::json!({}),
            false,
            Some("insufficient permissions".to_string()),
            "dec-002".to_string(),
            true,
        );
        assert!(!eval.allowed);
        assert!(eval.cached);
        assert_eq!(eval.reason.as_deref(), Some("insufficient permissions"));
    }
}
