use async_trait::async_trait;
use serde_json::Value;
use crate::domain::value_object::rule_result::RuleResult;
use crate::domain::entity::consistency_rule::ConsistencyRule;

#[async_trait]
pub trait RuleEngineService: Send + Sync {
    async fn evaluate_rule(&self, rule: &ConsistencyRule, record_data: &Value) -> anyhow::Result<RuleResult>;
}
