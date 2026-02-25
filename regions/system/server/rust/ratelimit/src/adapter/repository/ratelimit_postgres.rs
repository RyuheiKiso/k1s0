use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::{Algorithm, RateLimitRule};
use crate::domain::repository::RateLimitRepository;

/// RateLimitPostgresRepository は PostgreSQL ベースのルールリポジトリ。
pub struct RateLimitPostgresRepository {
    pool: PgPool,
}

impl RateLimitPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RateLimitRepository for RateLimitPostgresRepository {
    async fn create(&self, rule: &RateLimitRule) -> anyhow::Result<RateLimitRule> {
        let row = sqlx::query_as::<_, RuleRow>(
            r#"
            INSERT INTO ratelimit.rate_limit_rules (id, name, key, limit_count, window_secs, algorithm, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, name, key, limit_count, window_secs, algorithm, enabled, created_at, updated_at
            "#,
        )
        .bind(rule.id)
        .bind(&rule.name)
        .bind(&rule.key)
        .bind(rule.limit)
        .bind(rule.window_secs)
        .bind(rule.algorithm.as_str())
        .bind(rule.enabled)
        .bind(rule.created_at)
        .bind(rule.updated_at)
        .fetch_one(&self.pool)
        .await?;

        row.into_rule()
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<RateLimitRule> {
        let row = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, key, limit_count, window_secs, algorithm, enabled, created_at, updated_at
            FROM ratelimit.rate_limit_rules
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        row.into_rule()
    }

    async fn find_by_name(&self, name: &str) -> anyhow::Result<Option<RateLimitRule>> {
        let row = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, key, limit_count, window_secs, algorithm, enabled, created_at, updated_at
            FROM ratelimit.rate_limit_rules
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(r.into_rule()?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> anyhow::Result<Vec<RateLimitRule>> {
        let rows = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, key, limit_count, window_secs, algorithm, enabled, created_at, updated_at
            FROM ratelimit.rate_limit_rules
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.into_rule()).collect()
    }

    async fn update(&self, rule: &RateLimitRule) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE ratelimit.rate_limit_rules
            SET name = $1, key = $2, limit_count = $3, window_secs = $4, algorithm = $5, enabled = $6, updated_at = $7
            WHERE id = $8
            "#,
        )
        .bind(&rule.name)
        .bind(&rule.key)
        .bind(rule.limit)
        .bind(rule.window_secs)
        .bind(rule.algorithm.as_str())
        .bind(rule.enabled)
        .bind(rule.updated_at)
        .bind(rule.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM ratelimit.rate_limit_rules WHERE id = $1"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[derive(sqlx::FromRow)]
struct RuleRow {
    id: Uuid,
    name: String,
    key: String,
    limit_count: i64,
    window_secs: i64,
    algorithm: String,
    enabled: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl RuleRow {
    fn into_rule(self) -> anyhow::Result<RateLimitRule> {
        let algorithm = Algorithm::from_str(&self.algorithm)
            .map_err(|e| anyhow::anyhow!("invalid algorithm in DB: {}", e))?;

        Ok(RateLimitRule {
            id: self.id,
            name: self.name,
            key: self.key,
            limit: self.limit_count,
            window_secs: self.window_secs,
            algorithm,
            enabled: self.enabled,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
