use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::entity::consistency_rule::ConsistencyRule;
use crate::domain::entity::rule_condition::RuleCondition;
use crate::domain::repository::consistency_rule_repository::ConsistencyRuleRepository;

pub struct ConsistencyRulePostgresRepository {
    pool: PgPool,
}

impl ConsistencyRulePostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConsistencyRuleRepository for ConsistencyRulePostgresRepository {
    async fn find_all(&self, _table_id: Option<Uuid>, _rule_type: Option<&str>, _severity: Option<&str>) -> anyhow::Result<Vec<ConsistencyRule>> {
        todo!()
    }
    async fn find_by_id(&self, _id: Uuid) -> anyhow::Result<Option<ConsistencyRule>> {
        todo!()
    }
    async fn find_by_table_id(&self, _table_id: Uuid, _timing: Option<&str>) -> anyhow::Result<Vec<ConsistencyRule>> {
        todo!()
    }
    async fn create(&self, _rule: &ConsistencyRule, _conditions: &[RuleCondition]) -> anyhow::Result<ConsistencyRule> {
        todo!()
    }
    async fn update(&self, _id: Uuid, _rule: &ConsistencyRule) -> anyhow::Result<ConsistencyRule> {
        todo!()
    }
    async fn delete(&self, _id: Uuid) -> anyhow::Result<()> {
        todo!()
    }
    async fn find_conditions_by_rule_id(&self, _rule_id: Uuid) -> anyhow::Result<Vec<RuleCondition>> {
        todo!()
    }
}
