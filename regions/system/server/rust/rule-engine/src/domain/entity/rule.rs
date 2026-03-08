use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub priority: i32,
    pub when_condition: serde_json::Value,
    pub then_result: serde_json::Value,
    pub enabled: bool,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Rule {
    pub fn new(
        name: String,
        description: String,
        priority: i32,
        when_condition: serde_json::Value,
        then_result: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            priority,
            when_condition,
            then_result,
            enabled: true,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuleSet {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub evaluation_mode: EvaluationMode,
    pub default_result: serde_json::Value,
    pub rule_ids: Vec<Uuid>,
    pub current_version: u32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RuleSet {
    pub fn new(
        name: String,
        description: String,
        domain: String,
        evaluation_mode: EvaluationMode,
        default_result: serde_json::Value,
        rule_ids: Vec<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            domain,
            evaluation_mode,
            default_result,
            rule_ids,
            current_version: 0,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationMode {
    FirstMatch,
    AllMatch,
}

impl EvaluationMode {
    pub fn as_str(&self) -> &str {
        match self {
            EvaluationMode::FirstMatch => "first_match",
            EvaluationMode::AllMatch => "all_match",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "first_match" => Some(EvaluationMode::FirstMatch),
            "all_match" => Some(EvaluationMode::AllMatch),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RuleSetVersion {
    pub id: Uuid,
    pub rule_set_id: Uuid,
    pub version: u32,
    pub rule_ids_snapshot: Vec<Uuid>,
    pub default_result_snapshot: serde_json::Value,
    pub published_at: DateTime<Utc>,
    pub published_by: String,
}

impl RuleSetVersion {
    pub fn new(
        rule_set_id: Uuid,
        version: u32,
        rule_ids_snapshot: Vec<Uuid>,
        default_result_snapshot: serde_json::Value,
        published_by: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule_set_id,
            version,
            rule_ids_snapshot,
            default_result_snapshot,
            published_at: Utc::now(),
            published_by,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvaluationLog {
    pub id: Uuid,
    pub rule_set_name: String,
    pub rule_set_version: u32,
    pub matched_rule_id: Option<Uuid>,
    pub input_hash: String,
    pub result: serde_json::Value,
    pub context: serde_json::Value,
    pub evaluated_at: DateTime<Utc>,
}
