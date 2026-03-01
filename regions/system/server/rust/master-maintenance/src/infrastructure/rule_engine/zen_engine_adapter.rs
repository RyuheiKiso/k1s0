use async_trait::async_trait;
use serde_json::Value;
use crate::domain::entity::consistency_rule::ConsistencyRule;
use crate::domain::value_object::rule_result::RuleResult;
use crate::domain::service::rule_engine_service::RuleEngineService;

pub struct ZenEngineAdapter;

impl ZenEngineAdapter {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl RuleEngineService for ZenEngineAdapter {
    async fn evaluate_rule(&self, rule: &ConsistencyRule, _record_data: &Value) -> anyhow::Result<RuleResult> {
        match rule.rule_type.as_str() {
            "custom" => {
                if let Some(ref zen_json) = rule.zen_rule_json {
                    // TODO: ZEN Engine evaluation
                    let _ = zen_json;
                    Ok(RuleResult::pass())
                } else {
                    Ok(RuleResult::fail("No ZEN rule definition found".to_string()))
                }
            }
            _ => {
                // Non-custom rules are evaluated by the use case layer
                Ok(RuleResult::pass())
            }
        }
    }
}
