use std::sync::Arc;
use uuid::Uuid;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::consistency_rule_repository::ConsistencyRuleRepository;
use crate::domain::repository::dynamic_record_repository::DynamicRecordRepository;
use crate::domain::service::rule_engine_service::RuleEngineService;
use crate::domain::value_object::rule_result::RuleResult;

pub struct CheckConsistencyUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    rule_repo: Arc<dyn ConsistencyRuleRepository>,
    record_repo: Arc<dyn DynamicRecordRepository>,
    rule_engine: Arc<dyn RuleEngineService>,
}

impl CheckConsistencyUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        rule_repo: Arc<dyn ConsistencyRuleRepository>,
        record_repo: Arc<dyn DynamicRecordRepository>,
        rule_engine: Arc<dyn RuleEngineService>,
    ) -> Self {
        Self { table_repo, column_repo, rule_repo, record_repo, rule_engine }
    }

    pub async fn execute_rule(&self, rule_id: Uuid) -> anyhow::Result<Vec<RuleResult>> {
        let rule = self.rule_repo.find_by_id(rule_id).await?
            .ok_or_else(|| anyhow::anyhow!("Rule not found"))?;

        let table_id = rule.source_table_id;
        let table_defs = self.table_repo.find_all(None, false).await?;
        let table = table_defs.into_iter().find(|t| t.id == table_id)
            .ok_or_else(|| anyhow::anyhow!("Source table not found for rule"))?;

        let columns = self.column_repo.find_by_table_id(table.id).await?;
        let (records, _) = self.record_repo.find_all(&table, &columns, 1, 10000, None, None, None).await?;

        let mut results = Vec::new();
        for record in &records {
            let result = self.rule_engine.evaluate_rule(&rule, record).await?;
            if !result.passed {
                results.push(result);
            }
        }

        if results.is_empty() {
            results.push(RuleResult::pass());
        }

        Ok(results)
    }

    pub async fn check_all_rules(&self, table_name: &str) -> anyhow::Result<Vec<RuleResult>> {
        let table = self.table_repo.find_by_name(table_name).await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;

        let rules = self.rule_repo.find_by_table_id(table.id, None).await?;
        let columns = self.column_repo.find_by_table_id(table.id).await?;
        let (records, _) = self.record_repo.find_all(&table, &columns, 1, 10000, None, None, None).await?;

        let mut all_results = Vec::new();
        for rule in &rules {
            if !rule.is_active {
                continue;
            }
            for record in &records {
                let result = self.rule_engine.evaluate_rule(rule, record).await?;
                if !result.passed {
                    all_results.push(result);
                }
            }
        }

        if all_results.is_empty() {
            all_results.push(RuleResult::pass());
        }

        Ok(all_results)
    }
}
