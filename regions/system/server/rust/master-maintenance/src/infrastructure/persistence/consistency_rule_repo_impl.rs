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
    async fn find_all(&self, table_id: Option<Uuid>, rule_type: Option<&str>, severity: Option<&str>) -> anyhow::Result<Vec<ConsistencyRule>> {
        let mut query = String::from("SELECT * FROM master_maintenance.consistency_rules WHERE 1=1");
        let mut param_idx = 1u32;
        let mut bind_values: Vec<String> = Vec::new();

        if let Some(tid) = table_id {
            query.push_str(&format!(" AND source_table_id = ${}", param_idx));
            bind_values.push(tid.to_string());
            param_idx += 1;
        }
        if let Some(rt) = rule_type {
            query.push_str(&format!(" AND rule_type = ${}", param_idx));
            bind_values.push(rt.to_string());
            param_idx += 1;
        }
        if let Some(sev) = severity {
            query.push_str(&format!(" AND severity = ${}", param_idx));
            bind_values.push(sev.to_string());
        }
        query.push_str(" ORDER BY name");

        let mut q = sqlx::query_as::<_, ConsistencyRuleRow>(&query);
        for val in &bind_values {
            // Try to parse as UUID first, then bind as string
            if let Ok(uuid) = Uuid::parse_str(val) {
                q = q.bind(uuid);
            } else {
                q = q.bind(val.clone());
            }
        }

        let rows = q.fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ConsistencyRule>> {
        let row = sqlx::query_as::<_, ConsistencyRuleRow>(
            "SELECT * FROM master_maintenance.consistency_rules WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_table_id(&self, table_id: Uuid, timing: Option<&str>) -> anyhow::Result<Vec<ConsistencyRule>> {
        let rows = if let Some(t) = timing {
            sqlx::query_as::<_, ConsistencyRuleRow>(
                "SELECT * FROM master_maintenance.consistency_rules WHERE source_table_id = $1 AND evaluation_timing = $2 AND is_active = true ORDER BY name"
            )
            .bind(table_id)
            .bind(t)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ConsistencyRuleRow>(
                "SELECT * FROM master_maintenance.consistency_rules WHERE source_table_id = $1 AND is_active = true ORDER BY name"
            )
            .bind(table_id)
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn create(&self, rule: &ConsistencyRule, conditions: &[RuleCondition]) -> anyhow::Result<ConsistencyRule> {
        let mut tx = self.pool.begin().await?;

        let rule_row = sqlx::query_as::<_, ConsistencyRuleRow>(
            r#"INSERT INTO master_maintenance.consistency_rules
               (id, name, description, rule_type, severity, is_active, source_table_id,
                evaluation_timing, error_message_template, zen_rule_json, created_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
               RETURNING *"#
        )
        .bind(rule.id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&rule.rule_type)
        .bind(&rule.severity)
        .bind(rule.is_active)
        .bind(rule.source_table_id)
        .bind(&rule.evaluation_timing)
        .bind(&rule.error_message_template)
        .bind(&rule.zen_rule_json)
        .bind(&rule.created_by)
        .fetch_one(&mut *tx)
        .await?;

        for cond in conditions {
            sqlx::query(
                r#"INSERT INTO master_maintenance.rule_conditions
                   (id, rule_id, condition_order, left_table_id, left_column, operator,
                    right_table_id, right_column, right_value, logical_connector)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#
            )
            .bind(cond.id)
            .bind(cond.rule_id)
            .bind(cond.condition_order)
            .bind(cond.left_table_id)
            .bind(&cond.left_column)
            .bind(&cond.operator)
            .bind(cond.right_table_id)
            .bind(&cond.right_column)
            .bind(&cond.right_value)
            .bind(&cond.logical_connector)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(rule_row.into())
    }

    async fn update(&self, id: Uuid, rule: &ConsistencyRule) -> anyhow::Result<ConsistencyRule> {
        let row = sqlx::query_as::<_, ConsistencyRuleRow>(
            r#"UPDATE master_maintenance.consistency_rules SET
               name = $2,
               description = $3,
               rule_type = $4,
               severity = $5,
               is_active = $6,
               evaluation_timing = $7,
               error_message_template = $8,
               zen_rule_json = $9,
               updated_at = now()
               WHERE id = $1 RETURNING *"#
        )
        .bind(id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&rule.rule_type)
        .bind(&rule.severity)
        .bind(rule.is_active)
        .bind(&rule.evaluation_timing)
        .bind(&rule.error_message_template)
        .bind(&rule.zen_rule_json)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.into())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM master_maintenance.rule_conditions WHERE rule_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM master_maintenance.consistency_rules WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn find_conditions_by_rule_id(&self, rule_id: Uuid) -> anyhow::Result<Vec<RuleCondition>> {
        let rows = sqlx::query_as::<_, RuleConditionRow>(
            "SELECT * FROM master_maintenance.rule_conditions WHERE rule_id = $1 ORDER BY condition_order"
        )
        .bind(rule_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct ConsistencyRuleRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    rule_type: String,
    severity: String,
    is_active: bool,
    source_table_id: Uuid,
    evaluation_timing: String,
    error_message_template: String,
    zen_rule_json: Option<serde_json::Value>,
    created_by: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ConsistencyRuleRow> for ConsistencyRule {
    fn from(row: ConsistencyRuleRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            rule_type: row.rule_type,
            severity: row.severity,
            is_active: row.is_active,
            source_table_id: row.source_table_id,
            evaluation_timing: row.evaluation_timing,
            error_message_template: row.error_message_template,
            zen_rule_json: row.zen_rule_json,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct RuleConditionRow {
    id: Uuid,
    rule_id: Uuid,
    condition_order: i32,
    left_table_id: Uuid,
    left_column: String,
    operator: String,
    right_table_id: Option<Uuid>,
    right_column: Option<String>,
    right_value: Option<String>,
    logical_connector: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<RuleConditionRow> for RuleCondition {
    fn from(row: RuleConditionRow) -> Self {
        Self {
            id: row.id,
            rule_id: row.rule_id,
            condition_order: row.condition_order,
            left_table_id: row.left_table_id,
            left_column: row.left_column,
            operator: row.operator,
            right_table_id: row.right_table_id,
            right_column: row.right_column,
            right_value: row.right_value,
            logical_connector: row.logical_connector,
            created_at: row.created_at,
        }
    }
}
