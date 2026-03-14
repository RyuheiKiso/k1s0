use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::rule::Rule;
use crate::domain::repository::RuleRepository;

pub struct RulePostgresRepository {
    pool: Arc<PgPool>,
}

impl RulePostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RuleRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    condition: serde_json::Value,
    action: serde_json::Value,
    priority: i32,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<RuleRow> for Rule {
    fn from(r: RuleRow) -> Self {
        Rule {
            id: r.id,
            name: r.name,
            description: r.description.unwrap_or_default(),
            priority: r.priority,
            when_condition: r.condition,
            then_result: r.action,
            enabled: r.status == "active",
            version: 1, // DB schema does not have a version column
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl RuleRepository for RulePostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Rule>> {
        let row: Option<RuleRow> = sqlx::query_as(
            "SELECT id, name, description, condition, action, priority, status, \
                    created_at, updated_at \
             FROM rule_engine.rules WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(Into::into))
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Rule>> {
        let rows: Vec<RuleRow> = sqlx::query_as(
            "SELECT id, name, description, condition, action, priority, status, \
                    created_at, updated_at \
             FROM rule_engine.rules ORDER BY created_at DESC",
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        _rule_set_id: Option<Uuid>,
        _domain: Option<String>,
    ) -> anyhow::Result<(Vec<Rule>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let rows: Vec<RuleRow> = sqlx::query_as(
            "SELECT id, name, description, condition, action, priority, status, \
                    created_at, updated_at \
             FROM rule_engine.rules \
             ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rule_engine.rules")
            .fetch_one(self.pool.as_ref())
            .await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    async fn create(&self, rule: &Rule) -> anyhow::Result<()> {
        let status = if rule.enabled { "active" } else { "inactive" };

        sqlx::query(
            "INSERT INTO rule_engine.rules \
             (id, name, description, condition, action, priority, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(rule.id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&rule.when_condition)
        .bind(&rule.then_result)
        .bind(rule.priority)
        .bind(status)
        .bind(rule.created_at)
        .bind(rule.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn update(&self, rule: &Rule) -> anyhow::Result<()> {
        let status = if rule.enabled { "active" } else { "inactive" };

        sqlx::query(
            "UPDATE rule_engine.rules \
             SET name = $2, description = $3, condition = $4, action = $5, \
                 priority = $6, status = $7, updated_at = $8 \
             WHERE id = $1",
        )
        .bind(rule.id)
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&rule.when_condition)
        .bind(&rule.then_result)
        .bind(rule.priority)
        .bind(status)
        .bind(rule.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM rule_engine.rules WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM rule_engine.rules WHERE name = $1")
                .bind(name)
                .fetch_one(self.pool.as_ref())
                .await?;

        Ok(count.0 > 0)
    }

    async fn find_by_ids(&self, ids: &[Uuid]) -> anyhow::Result<Vec<Rule>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows: Vec<RuleRow> = sqlx::query_as(
            "SELECT id, name, description, condition, action, priority, status, \
                    created_at, updated_at \
             FROM rule_engine.rules WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
