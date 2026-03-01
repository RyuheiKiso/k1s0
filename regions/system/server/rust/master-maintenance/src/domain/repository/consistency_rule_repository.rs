use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::entity::consistency_rule::ConsistencyRule;
use crate::domain::entity::rule_condition::RuleCondition;

#[async_trait]
pub trait ConsistencyRuleRepository: Send + Sync {
    async fn find_all(&self, table_id: Option<Uuid>, rule_type: Option<&str>, severity: Option<&str>) -> anyhow::Result<Vec<ConsistencyRule>>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ConsistencyRule>>;
    async fn find_by_table_id(&self, table_id: Uuid, timing: Option<&str>) -> anyhow::Result<Vec<ConsistencyRule>>;
    async fn create(&self, rule: &ConsistencyRule, conditions: &[RuleCondition]) -> anyhow::Result<ConsistencyRule>;
    async fn update(&self, id: Uuid, rule: &ConsistencyRule) -> anyhow::Result<ConsistencyRule>;
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
    async fn find_conditions_by_rule_id(&self, rule_id: Uuid) -> anyhow::Result<Vec<RuleCondition>>;
}
