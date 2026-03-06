use crate::domain::entity::consistency_rule::{ConsistencyRule, CreateConsistencyRule};
use crate::domain::entity::rule_condition::RuleCondition;
use crate::domain::repository::consistency_rule_repository::ConsistencyRuleRepository;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct ManageRulesUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    rule_repo: Arc<dyn ConsistencyRuleRepository>,
}

impl ManageRulesUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        rule_repo: Arc<dyn ConsistencyRuleRepository>,
    ) -> Self {
        Self {
            table_repo,
            rule_repo,
        }
    }

    pub async fn list_rules(
        &self,
        table_name: Option<&str>,
        rule_type: Option<&str>,
        severity: Option<&str>,
    ) -> anyhow::Result<Vec<ConsistencyRule>> {
        let table_id = if let Some(name) = table_name {
            let table = self
                .table_repo
                .find_by_name(name)
                .await?
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

    pub async fn create_rule(
        &self,
        input: &serde_json::Value,
        created_by: &str,
    ) -> anyhow::Result<ConsistencyRule> {
        let create_input: CreateConsistencyRule = serde_json::from_value(input.clone())?;

        let table = self
            .table_repo
            .find_by_name(&create_input.source_table)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", create_input.source_table))?;

        let rule = ConsistencyRule {
            id: Uuid::new_v4(),
            name: create_input.name,
            description: create_input.description,
            rule_type: create_input.rule_type,
            severity: create_input.severity.unwrap_or_else(|| "error".to_string()),
            is_active: true,
            source_table_id: table.id,
            evaluation_timing: create_input
                .evaluation_timing
                .unwrap_or_else(|| "before_save".to_string()),
            error_message_template: create_input.error_message_template,
            zen_rule_json: create_input.zen_rule_json,
            created_by: created_by.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut conditions = Vec::new();
        for condition in create_input.conditions.unwrap_or_default() {
            let right_table_id = if let Some(right_table_name) = condition.right_table.as_deref() {
                Some(
                    self.table_repo
                        .find_by_name(right_table_name)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", right_table_name))?
                        .id,
                )
            } else {
                None
            };

            conditions.push(RuleCondition {
                id: Uuid::new_v4(),
                rule_id: rule.id,
                condition_order: condition.condition_order,
                left_table_id: table.id,
                left_column: condition.left_column,
                operator: condition.operator,
                right_table_id,
                right_column: condition.right_column,
                right_value: condition.right_value,
                logical_connector: condition.logical_connector,
                created_at: chrono::Utc::now(),
            });
        }

        self.rule_repo.create(&rule, &conditions).await
    }

    pub async fn update_rule(
        &self,
        id: Uuid,
        input: &serde_json::Value,
    ) -> anyhow::Result<ConsistencyRule> {
        let mut rule = self
            .rule_repo
            .find_by_id(id)
            .await?
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

        let updated = self.rule_repo.update(id, &rule).await?;

        if let Some(conditions_value) = input.get("conditions") {
            let source_table = self
                .table_repo
                .find_by_id(rule.source_table_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Source table not found"))?;
            let inputs: Vec<crate::domain::entity::consistency_rule::CreateRuleConditionInput> =
                serde_json::from_value(conditions_value.clone())?;
            let mut conditions = Vec::with_capacity(inputs.len());
            for condition in inputs {
                let right_table_id =
                    if let Some(right_table_name) = condition.right_table.as_deref() {
                        Some(
                            self.table_repo
                                .find_by_name(right_table_name)
                                .await?
                                .ok_or_else(|| {
                                    anyhow::anyhow!("Table '{}' not found", right_table_name)
                                })?
                                .id,
                        )
                    } else {
                        None
                    };

                conditions.push(RuleCondition {
                    id: Uuid::new_v4(),
                    rule_id: updated.id,
                    condition_order: condition.condition_order,
                    left_table_id: source_table.id,
                    left_column: condition.left_column,
                    operator: condition.operator,
                    right_table_id,
                    right_column: condition.right_column,
                    right_value: condition.right_value,
                    logical_connector: condition.logical_connector,
                    created_at: chrono::Utc::now(),
                });
            }
            self.rule_repo
                .replace_conditions(updated.id, &conditions)
                .await?;
        }

        Ok(updated)
    }

    pub async fn delete_rule(&self, id: Uuid) -> anyhow::Result<()> {
        self.rule_repo.delete(id).await
    }
}
