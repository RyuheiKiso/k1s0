use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub event_type: String,
    pub source: String,
    pub timeout_seconds: i32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowSlo {
    pub target_completion_seconds: i32,
    pub target_success_rate: f64,
    pub alert_on_violation: bool,
}

#[derive(Debug, Clone)]
pub struct FlowDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub steps: Vec<FlowStep>,
    pub slo: FlowSlo,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FlowDefinition {
    pub fn new(
        name: String,
        description: String,
        domain: String,
        steps: Vec<FlowStep>,
        slo: FlowSlo,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            domain,
            steps,
            slo,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}
