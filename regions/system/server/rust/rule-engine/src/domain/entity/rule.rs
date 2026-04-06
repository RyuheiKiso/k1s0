use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Rule {
    pub id: Uuid,
    /// CRITICAL-RUST-001 監査対応: テナント分離のために追加したテナント識別子。
    /// RLS ポリシーの app.current_tenant_id セッション変数と対応する（migration 003 対応）。
    pub tenant_id: String,
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
        tenant_id: String,
        name: String,
        description: String,
        priority: i32,
        when_condition: serde_json::Value,
        then_result: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            tenant_id,
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
    /// CRITICAL-RUST-001 監査対応: テナント分離のために追加したテナント識別子（migration 003 対応）。
    pub tenant_id: String,
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
        tenant_id: String,
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
            tenant_id,
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

    #[allow(clippy::should_implement_trait)]
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// Rule::new がデフォルト値（enabled=true, version=1）で生成される
    #[test]
    fn rule_new_defaults() {
        let rule = Rule::new(
            "tenant-1".to_string(),
            "my-rule".to_string(),
            "description".to_string(),
            50,
            serde_json::json!({"field": "x", "operator": "eq", "value": "y"}),
            serde_json::json!({"action": "allow"}),
        );
        assert_eq!(rule.tenant_id, "tenant-1");
        assert_eq!(rule.name, "my-rule");
        assert_eq!(rule.priority, 50);
        assert!(rule.enabled);
        assert_eq!(rule.version, 1);
    }

    /// RuleSet::new が current_version=0 で生成される
    #[test]
    fn rule_set_new_defaults() {
        let rs = RuleSet::new(
            "tenant-1".to_string(),
            "pricing".to_string(),
            "Pricing rules".to_string(),
            "sales".to_string(),
            EvaluationMode::FirstMatch,
            serde_json::json!({}),
            vec![],
        );
        assert_eq!(rs.tenant_id, "tenant-1");
        assert_eq!(rs.name, "pricing");
        assert_eq!(rs.domain, "sales");
        assert_eq!(rs.current_version, 0);
        assert!(rs.enabled);
        assert!(rs.rule_ids.is_empty());
    }

    /// EvaluationMode::as_str が snake_case 文字列を返す
    #[test]
    fn evaluation_mode_as_str() {
        assert_eq!(EvaluationMode::FirstMatch.as_str(), "first_match");
        assert_eq!(EvaluationMode::AllMatch.as_str(), "all_match");
    }

    /// EvaluationMode::from_str が正しいバリアントを返す
    #[test]
    fn evaluation_mode_from_str_valid() {
        assert_eq!(
            EvaluationMode::from_str("first_match"),
            Some(EvaluationMode::FirstMatch)
        );
        assert_eq!(
            EvaluationMode::from_str("all_match"),
            Some(EvaluationMode::AllMatch)
        );
    }

    /// EvaluationMode::from_str が不明な文字列に None を返す
    #[test]
    fn evaluation_mode_from_str_unknown() {
        assert!(EvaluationMode::from_str("random").is_none());
        assert!(EvaluationMode::from_str("").is_none());
    }

    /// EvaluationMode が serde_json でシリアライズ・デシリアライズできる
    #[test]
    fn evaluation_mode_serde_roundtrip() {
        let mode = EvaluationMode::AllMatch;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"all_match\"");
        let decoded: EvaluationMode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, EvaluationMode::AllMatch);
    }
}

#[derive(Debug, Clone)]
pub struct EvaluationLog {
    pub id: Uuid,
    /// CRITICAL-RUST-001 監査対応: テナント分離のために追加したテナント識別子（migration 003 対応）。
    pub tenant_id: String,
    pub rule_set_name: String,
    pub rule_set_version: u32,
    pub matched_rule_id: Option<Uuid>,
    pub input_hash: String,
    pub result: serde_json::Value,
    pub context: serde_json::Value,
    pub evaluated_at: DateTime<Utc>,
}
