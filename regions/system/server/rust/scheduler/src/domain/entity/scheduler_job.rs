use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerJob {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub cron_expression: String,
    pub timezone: String,
    pub target_type: String,
    pub target: Option<String>,
    pub payload: serde_json::Value,
    pub status: String,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SchedulerJob {
    pub fn new(name: String, cron_expression: String, payload: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            cron_expression,
            timezone: "UTC".to_string(),
            target_type: "kafka".to_string(),
            target: None,
            payload,
            status: "active".to_string(),
            next_run_at: None,
            last_run_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

pub fn validate_cron(expr: &str) -> bool {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    parts.len() == 5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_cron() {
        assert!(validate_cron("* * * * *"));
        assert!(validate_cron("0 12 * * 1"));
        assert!(validate_cron("30 6 1 1 *"));
    }

    #[test]
    fn invalid_cron() {
        assert!(!validate_cron("* * *"));
        assert!(!validate_cron("* * * * * *"));
        assert!(!validate_cron(""));
    }
}
