use crate::domain::entity::consistency_rule::ConsistencyRule;
use crate::domain::service::rule_engine_service::RuleEngineService;
use crate::domain::value_object::rule_result::RuleResult;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use zen_engine::model::DecisionContent;
use zen_engine::DecisionEngine;

pub struct ZenEngineAdapter;

impl Default for ZenEngineAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ZenEngineAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RuleEngineService for ZenEngineAdapter {
    /// カスタムルールを評価する。
    ///
    /// ZEN エンジンの Future は Send ではないため、spawn_blocking で
    /// 専用スレッド上で同期版を実行することでネストランタイムを回避する。
    async fn evaluate_rule(
        &self,
        rule: &ConsistencyRule,
        record_data: &Value,
    ) -> anyhow::Result<RuleResult> {
        match rule.rule_type.as_str() {
            "custom" => {
                let rule = rule.clone();
                let record_data = record_data.clone();
                tokio::task::spawn_blocking(move || {
                    Self::evaluate_custom_rule(&rule, &record_data)
                })
                .await?
            }
            _ => {
                // カスタムルール以外はユースケース層で評価される
                Ok(RuleResult::pass())
            }
        }
    }
}

impl ZenEngineAdapter {
    fn result_with_severity(
        rule: &ConsistencyRule,
        passed: bool,
        message: Option<String>,
    ) -> RuleResult {
        RuleResult {
            rule_id: String::new(),
            rule_name: String::new(),
            passed,
            message,
            severity: rule.severity.clone(),
            affected_record_ids: vec![],
        }
    }

    fn parse_decision_content(rule: &ConsistencyRule) -> anyhow::Result<Arc<DecisionContent>> {
        let zen_json = rule
            .zen_rule_json
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No ZEN rule definition found"))?;
        let content: DecisionContent = serde_json::from_value(zen_json)?;
        Ok(Arc::new(content))
    }

    fn message_from_result(result: &Value, default_message: &str) -> Option<String> {
        result
            .get("_message")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .or_else(|| (!default_message.is_empty()).then(|| default_message.to_string()))
    }

    /// ZEN エンジンでカスタムルールを同期評価する。
    ///
    /// ZEN の evaluate Future は !Send のため、spawn_blocking 上で
    /// 一時的なランタイムを使って実行する。
    fn evaluate_custom_rule(
        rule: &ConsistencyRule,
        record_data: &Value,
    ) -> anyhow::Result<RuleResult> {
        let engine = DecisionEngine::default();
        let decision = engine.create_decision(Self::parse_decision_content(rule)?);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let response = runtime.block_on(async { decision.evaluate(record_data).await })?;
        let result = response.result;

        Self::map_rule_result(rule, &result)
    }

    /// ZEN エンジンの結果を RuleResult にマッピングする。
    fn map_rule_result(rule: &ConsistencyRule, result: &Value) -> anyhow::Result<RuleResult> {
        match result.get("_result").and_then(|v| v.as_str()) {
            Some("fail") => Ok(Self::result_with_severity(
                rule,
                false,
                Self::message_from_result(result, &rule.error_message_template),
            )),
            Some("warning") => Ok(Self::result_with_severity(
                rule,
                true,
                Self::message_from_result(result, &rule.error_message_template),
            )),
            Some("pass") | None => Ok(Self::result_with_severity(rule, true, None)),
            Some(other) => Ok(Self::result_with_severity(
                rule,
                other.eq_ignore_ascii_case("pass"),
                Self::message_from_result(result, &rule.error_message_template),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    fn custom_rule(zen_rule_json: Option<Value>) -> ConsistencyRule {
        ConsistencyRule {
            id: Uuid::new_v4(),
            name: "budget-check".to_string(),
            description: None,
            rule_type: "custom".to_string(),
            severity: "error".to_string(),
            is_active: true,
            source_table_id: Uuid::new_v4(),
            evaluation_timing: "before_save".to_string(),
            error_message_template: "rule failed".to_string(),
            zen_rule_json,
            created_by: "tester".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn message_from_result_prefers_message_field() {
        let result = ZenEngineAdapter::message_from_result(
            &json!({
                "_result": "fail",
                "_message": "budget must be non-negative"
            }),
            "fallback",
        );

        assert_eq!(result.as_deref(), Some("budget must be non-negative"));
    }

    #[test]
    fn custom_rule_requires_zen_definition() {
        let rule = custom_rule(None);

        let err = ZenEngineAdapter::evaluate_custom_rule(&rule, &json!({ "budget": 10 }))
            .expect_err("expected missing ZEN definition to fail");

        assert!(err.to_string().contains("No ZEN rule definition found"));
    }
}
