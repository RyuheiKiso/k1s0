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
            INSERT INTO ratelimit.rate_limit_rules
                (id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, created_at, updated_at
            "#,
        )
        .bind(rule.id)
        .bind(&rule.name)
        .bind(&rule.scope)
        .bind(&rule.identifier_pattern)
        .bind(i64::from(rule.limit))
        .bind(i64::from(rule.window_seconds))
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
            SELECT id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, created_at, updated_at
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
            SELECT id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, created_at, updated_at
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

    async fn find_by_scope(&self, scope: &str) -> anyhow::Result<Vec<RateLimitRule>> {
        let rows = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, created_at, updated_at
            FROM ratelimit.rate_limit_rules
            WHERE scope = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(scope)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.into_rule()).collect()
    }

    async fn find_all(&self) -> anyhow::Result<Vec<RateLimitRule>> {
        let rows = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, created_at, updated_at
            FROM ratelimit.rate_limit_rules
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.into_rule()).collect()
    }

    async fn find_page(
        &self,
        page: u32,
        page_size: u32,
        scope: Option<String>,
        enabled_only: bool,
    ) -> anyhow::Result<(Vec<RateLimitRule>, u64)> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 200);
        let offset = ((page - 1) * page_size) as i64;
        let scope_ref = scope.as_deref();

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM ratelimit.rate_limit_rules
            WHERE ($1::text IS NULL OR scope = $1)
              AND ($2::bool = FALSE OR enabled = TRUE)
            "#,
        )
        .bind(scope_ref)
        .bind(enabled_only)
        .fetch_one(&self.pool)
        .await?;

        let rows = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, created_at, updated_at
            FROM ratelimit.rate_limit_rules
            WHERE ($1::text IS NULL OR scope = $1)
              AND ($2::bool = FALSE OR enabled = TRUE)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(scope_ref)
        .bind(enabled_only)
        .bind(page_size as i64)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let rules: Vec<RateLimitRule> = rows
            .into_iter()
            .map(|r| r.into_rule())
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok((rules, total.max(0) as u64))
    }

    async fn update(&self, rule: &RateLimitRule) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE ratelimit.rate_limit_rules
            SET name = $1, scope = $2, identifier_pattern = $3, limit_count = $4, window_secs = $5, algorithm = $6, enabled = $7, updated_at = $8
            WHERE id = $9
            "#,
        )
        .bind(&rule.name)
        .bind(&rule.scope)
        .bind(&rule.identifier_pattern)
        .bind(i64::from(rule.limit))
        .bind(i64::from(rule.window_seconds))
        .bind(rule.algorithm.as_str())
        .bind(rule.enabled)
        .bind(rule.updated_at)
        .bind(rule.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query(r#"DELETE FROM ratelimit.rate_limit_rules WHERE id = $1"#)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn reset_state(&self, _key: &str) -> anyhow::Result<()> {
        // PostgreSQL リポジトリではRedis状態のリセットは行わない（state_storeが担当）
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct RuleRow {
    id: Uuid,
    name: String,
    scope: String,
    identifier_pattern: String,
    limit_count: i64,
    #[sqlx(rename = "window_secs")]
    window_seconds: i64,
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
            scope: self.scope,
            identifier_pattern: self.identifier_pattern,
            limit: u32::try_from(self.limit_count)
                .map_err(|_| anyhow::anyhow!("invalid limit_count in DB: {}", self.limit_count))?,
            window_seconds: u32::try_from(self.window_seconds).map_err(|_| {
                anyhow::anyhow!("invalid window_secs in DB: {}", self.window_seconds)
            })?,
            algorithm,
            enabled: self.enabled,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
