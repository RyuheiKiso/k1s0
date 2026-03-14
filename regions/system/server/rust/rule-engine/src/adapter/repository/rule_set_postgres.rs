use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::rule::{EvaluationMode, RuleSet};
use crate::domain::repository::RuleSetRepository;

pub struct RuleSetPostgresRepository {
    pool: Arc<PgPool>,
}

impl RuleSetPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RuleSetRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<RuleSetRow> for RuleSet {
    fn from(r: RuleSetRow) -> Self {
        RuleSet {
            id: r.id,
            name: r.name,
            description: r.description.unwrap_or_default(),
            domain: String::new(),                        // not in DB schema
            evaluation_mode: EvaluationMode::FirstMatch,  // not in DB schema
            default_result: serde_json::Value::Null,      // not in DB schema
            rule_ids: Vec::new(),                          // loaded separately via versions
            current_version: 0,                            // not in DB schema
            enabled: r.status == "active",
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[async_trait]
impl RuleSetRepository for RuleSetPostgresRepository {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<RuleSet>> {
        let row: Option<RuleSetRow> = sqlx::query_as(
            "SELECT id, name, description, status, created_at, updated_at \
             FROM rule_engine.rule_sets WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(Into::into))
    }

    async fn find_all(&self) -> anyhow::Result<Vec<RuleSet>> {
        let rows: Vec<RuleSetRow> = sqlx::query_as(
            "SELECT id, name, description, status, created_at, updated_at \
             FROM rule_engine.rule_sets ORDER BY created_at DESC",
        )
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_all_paginated(
        &self,
        page: u32,
        page_size: u32,
        _domain: Option<String>,
    ) -> anyhow::Result<(Vec<RuleSet>, u64)> {
        let offset = (page.saturating_sub(1) * page_size) as i64;
        let limit = page_size as i64;

        let rows: Vec<RuleSetRow> = sqlx::query_as(
            "SELECT id, name, description, status, created_at, updated_at \
             FROM rule_engine.rule_sets \
             ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM rule_engine.rule_sets")
            .fetch_one(self.pool.as_ref())
            .await?;

        Ok((rows.into_iter().map(Into::into).collect(), count.0 as u64))
    }

    async fn find_by_domain_and_name(
        &self,
        _domain: &str,
        name: &str,
    ) -> anyhow::Result<Option<RuleSet>> {
        // DB schema does not have a domain column, so we match by name only.
        let row: Option<RuleSetRow> = sqlx::query_as(
            "SELECT id, name, description, status, created_at, updated_at \
             FROM rule_engine.rule_sets WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(Into::into))
    }

    async fn create(&self, rule_set: &RuleSet) -> anyhow::Result<()> {
        let status = if rule_set.enabled {
            "active"
        } else {
            "inactive"
        };

        sqlx::query(
            "INSERT INTO rule_engine.rule_sets \
             (id, name, description, status, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(rule_set.id)
        .bind(&rule_set.name)
        .bind(&rule_set.description)
        .bind(status)
        .bind(rule_set.created_at)
        .bind(rule_set.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn update(&self, rule_set: &RuleSet) -> anyhow::Result<()> {
        let status = if rule_set.enabled {
            "active"
        } else {
            "inactive"
        };

        sqlx::query(
            "UPDATE rule_engine.rule_sets \
             SET name = $2, description = $3, status = $4, updated_at = $5 \
             WHERE id = $1",
        )
        .bind(rule_set.id)
        .bind(&rule_set.name)
        .bind(&rule_set.description)
        .bind(status)
        .bind(rule_set.updated_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM rule_engine.rule_sets WHERE id = $1")
            .bind(id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn exists_by_name(&self, name: &str) -> anyhow::Result<bool> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM rule_engine.rule_sets WHERE name = $1")
                .bind(name)
                .fetch_one(self.pool.as_ref())
                .await?;

        Ok(count.0 > 0)
    }
}
