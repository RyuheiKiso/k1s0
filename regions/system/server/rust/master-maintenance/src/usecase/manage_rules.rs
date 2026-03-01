use std::sync::Arc;
use uuid::Uuid;
use crate::domain::entity::consistency_rule::{ConsistencyRule, CreateConsistencyRule};
use crate::domain::entity::rule_condition::RuleCondition;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::repository::consistency_rule_repository::ConsistencyRuleRepository;

pub struct ManageRulesUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    rule_repo: Arc<dyn ConsistencyRuleRepository>,
}

impl ManageRulesUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        rule_repo: Arc<dyn ConsistencyRuleRepository>,
    ) -> Self {
        Self { table_repo, rule_repo }
    }

    pub async fn list_rules(
        &self,
        table_name: Option<&str>,
        rule_type: Option<&str>,
        severity: Option<&str>,
    ) -> anyhow::Result<Vec<ConsistencyRule>> {
        let table_id = if let Some(name) = table_name {
            let table = self.table_repo.find_by_name(name).await?
                .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", name))?;
            Some(table.id)
        } else {
            None
        };
        self.rule_repo.find_all(table_id, rule_type, severity).await
    }

    pub async fn get_rule(&self, id: Uuid) -> anyhow::Result<Option<ConsistencyRule>> {
        self.rule_repo.find_by_id(id).await
    }

    pub async fn create_rule(&self, input: &serde_json::Value, created_by: &str) -> anyhow::Result<ConsistencyRule> {
        let create_input: CreateConsistencyRule = serde_json::from_value(input.clone())?;

        let table = self.table_repo.find_by_name(&create_input.source_table).await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", create_input.source_table))?;

        let rule = ConsistencyRule {
            id: Uuid::new_v4(),
            name: create_input.name,
            description: create_input.description,
            rule_type: create_input.rule_type,
            severity: create_input.severity.unwrap_or_else(|| "error".to_string()),
            is_active: true,
            source_table_id: table.id,
            evaluation_timing: create_input.evaluation_timing.unwrap_or_else(|| "on_save".to_string()),
            error_message_template: create_input.error_message_template,
            zen_rule_json: create_input.zen_rule_json,
            created_by: created_by.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let conditions: Vec<RuleCondition> = create_input.conditions
            .unwrap_or_default()
            .iter()
            .map(|c| RuleCondition {
                id: Uuid::new_v4(),
                rule_id: rule.id,
                condition_order: c.condition_order,
                left_table_id: table.id,
                left_column: c.left_column.clone(),
                operator: c.operator.clone(),
                right_table_id: None,
                right_column: c.right_column.clone(),
                right_value: c.right_value.clone(),
                logical_connector: c.logical_connector.clone(),
                created_at: chrono::Utc::now(),
            })
            .collect();

        self.rule_repo.create(&rule, &conditions).await
    }

    pub async fn update_rule(&self, id: Uuid, input: &serde_json::Value) -> anyhow::Result<ConsistencyRule> {
        let mut rule = self.rule_repo.find_by_id(id).await?
            .ok_or_else(|| anyhow::anyhow!("Rule not found"))?;

        if let Some(name) = input.get("name").and_then(|v| v.as_str()) {
            rule.name = name.to_string();
        }
        if let Some(desc) = input.get("description").and_then(|v| v.as_str()) {
            rule.description = Some(desc.to_string());
        }
        if let Some(severity) = input.get("severity").and_then(|v| v.as_str()) {
            rule.severity = severity.to_string();
        }
        if let Some(active) = input.get("is_active").and_then(|v| v.as_bool()) {
            rule.is_active = active;
        }
        if let Some(timing) = input.get("evaluation_timing").and_then(|v| v.as_str()) {
            rule.evaluation_timing = timing.to_string();
        }
        if let Some(tmpl) = input.get("error_message_template").and_then(|v| v.as_str()) {
            rule.error_message_template = tmpl.to_string();
        }
        if let Some(zen) = input.get("zen_rule_json") {
            rule.zen_rule_json = Some(zen.clone());
        }
        rule.updated_at = chrono::Utc::now();

        self.rule_repo.update(id, &rule).await
    }

    pub async fn delete_rule(&self, id: Uuid) -> anyhow::Result<()> {
        self.rule_repo.delete(id).await
    }
}
