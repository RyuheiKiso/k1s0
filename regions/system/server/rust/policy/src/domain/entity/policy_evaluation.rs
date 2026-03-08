use chrono::{DateTime, Utc};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, Clone)]
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
