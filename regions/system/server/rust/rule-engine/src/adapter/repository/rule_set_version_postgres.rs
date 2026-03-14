use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::rule::RuleSetVersion;
use crate::domain::repository::RuleSetVersionRepository;

pub struct RuleSetVersionPostgresRepository {
    pool: Arc<PgPool>,
}

impl RuleSetVersionPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RuleSetVersionRow {
    id: Uuid,
    rule_set_id: Uuid,
    version: i32,
    rules: serde_json::Value,
    #[allow(dead_code)]
    status: String,
    published_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    #[allow(dead_code)]
    updated_at: DateTime<Utc>,
}

impl From<RuleSetVersionRow> for RuleSetVersion {
    fn from(r: RuleSetVersionRow) -> Self {
        // The DB `rules` column is a JSONB array of UUID strings.
        let rule_ids_snapshot: Vec<Uuid> = serde_json::from_value(r.rules).unwrap_or_default();

        RuleSetVersion {
            id: r.id,
            rule_set_id: r.rule_set_id,
            version: r.version as u32,
            rule_ids_snapshot,
            default_result_snapshot: serde_json::Value::Null, // not in DB schema
            published_at: r.published_at.unwrap_or(r.created_at),
            published_by: String::new(), // not in DB schema
        }
    }
}

#[async_trait]
impl RuleSetVersionRepository for RuleSetVersionPostgresRepository {
    async fn find_by_rule_set_id_and_version(
        &self,
        rule_set_id: &Uuid,
        version: u32,
    ) -> anyhow::Result<Option<RuleSetVersion>> {
        let row: Option<RuleSetVersionRow> = sqlx::query_as(
            "SELECT id, rule_set_id, version, rules, status, published_at, \
                    created_at, updated_at \
             FROM rule_engine.rule_set_versions \
             WHERE rule_set_id = $1 AND version = $2",
        )
        .bind(rule_set_id)
        .bind(version as i32)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(Into::into))
    }

    async fn find_latest_by_rule_set_id(
        &self,
        rule_set_id: &Uuid,
    ) -> anyhow::Result<Option<RuleSetVersion>> {
        let row: Option<RuleSetVersionRow> = sqlx::query_as(
            "SELECT id, rule_set_id, version, rules, status, published_at, \
                    created_at, updated_at \
             FROM rule_engine.rule_set_versions \
             WHERE rule_set_id = $1 \
             ORDER BY version DESC LIMIT 1",
        )
        .bind(rule_set_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(row.map(Into::into))
    }

    async fn create(&self, version: &RuleSetVersion) -> anyhow::Result<()> {
        let rules_json = serde_json::to_value(&version.rule_ids_snapshot)?;

        sqlx::query(
            "INSERT INTO rule_engine.rule_set_versions \
             (id, rule_set_id, version, rules, status, published_at, created_at) \
             VALUES ($1, $2, $3, $4, 'published', $5, $6)",
        )
        .bind(version.id)
        .bind(version.rule_set_id)
        .bind(version.version as i32)
        .bind(&rules_json)
        .bind(version.published_at)
        .bind(version.published_at) // created_at = published_at
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}
