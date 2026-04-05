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

/// CRIT-005 対応: トランザクション内で RLS セッション変数を設定するヘルパー。
/// set_config の第3引数 true は「トランザクションローカル」を意味し、トランザクション終了時にリセットされる。
async fn set_tenant_rls(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
) -> anyhow::Result<()> {
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[async_trait]
impl RateLimitRepository for RateLimitPostgresRepository {
    /// CRIT-005 対応: トランザクション内で RLS セッション変数を設定してからルールを作成する。
    async fn create(&self, rule: &RateLimitRule) -> anyhow::Result<RateLimitRule> {
        let mut tx = self.pool.begin().await?;
        set_tenant_rls(&mut tx, &rule.tenant_id).await?;

        let row = sqlx::query_as::<_, RuleRow>(
            r#"
            INSERT INTO ratelimit.rate_limit_rules
                (id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, tenant_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, tenant_id, created_at, updated_at
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
        .bind(&rule.tenant_id)
        .bind(rule.created_at)
        .bind(rule.updated_at)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        row.into_rule()
    }

    /// CRIT-005 対応: トランザクション内で RLS セッション変数を設定してから ID でルールを取得する。
    async fn find_by_id(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<RateLimitRule> {
        let mut tx = self.pool.begin().await?;
        set_tenant_rls(&mut tx, tenant_id).await?;

        let row = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, tenant_id, created_at, updated_at
            FROM ratelimit.rate_limit_rules
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;
        row.into_rule()
    }

    /// CRIT-005 対応: トランザクション内で RLS セッション変数を設定してから name でルールを取得する。
    async fn find_by_name(
        &self,
        name: &str,
        tenant_id: &str,
    ) -> anyhow::Result<Option<RateLimitRule>> {
        let mut tx = self.pool.begin().await?;
        set_tenant_rls(&mut tx, tenant_id).await?;

        let row = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, tenant_id, created_at, updated_at
            FROM ratelimit.rate_limit_rules
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;
        match row {
            Some(r) => Ok(Some(r.into_rule()?)),
            None => Ok(None),
        }
    }

    /// CRIT-005 対応: トランザクション内で RLS セッション変数を設定してから scope でルールを取得する。
    async fn find_by_scope(
        &self,
        scope: &str,
        tenant_id: &str,
    ) -> anyhow::Result<Vec<RateLimitRule>> {
        let mut tx = self.pool.begin().await?;
        set_tenant_rls(&mut tx, tenant_id).await?;

        let rows = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, tenant_id, created_at, updated_at
            FROM ratelimit.rate_limit_rules
            WHERE scope = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(scope)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        rows.into_iter().map(|r| r.into_rule()).collect()
    }

    /// CRIT-005 対応: トランザクション内で RLS セッション変数を設定してから全ルールを取得する。
    async fn find_all(&self, tenant_id: &str) -> anyhow::Result<Vec<RateLimitRule>> {
        let mut tx = self.pool.begin().await?;
        set_tenant_rls(&mut tx, tenant_id).await?;

        let rows = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, tenant_id, created_at, updated_at
            FROM ratelimit.rate_limit_rules
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        rows.into_iter().map(|r| r.into_rule()).collect()
    }

    /// CRIT-005 対応: トランザクション内で RLS セッション変数を設定してから条件付きページネーションでルールを取得する。
    async fn find_page(
        &self,
        page: u32,
        page_size: u32,
        scope: Option<String>,
        enabled_only: bool,
        tenant_id: &str,
    ) -> anyhow::Result<(Vec<RateLimitRule>, u64)> {
        let page = page.max(1);
        let page_size = page_size.clamp(1, 200);
        let offset = ((page - 1) * page_size) as i64;
        let scope_ref = scope.as_deref();

        let mut tx = self.pool.begin().await?;
        set_tenant_rls(&mut tx, tenant_id).await?;

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
        .fetch_one(&mut *tx)
        .await?;

        let rows = sqlx::query_as::<_, RuleRow>(
            r#"
            SELECT id, name, scope, identifier_pattern, limit_count, window_secs, algorithm, enabled, tenant_id, created_at, updated_at
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
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        let rules: Vec<RateLimitRule> = rows
            .into_iter()
            .map(|r| r.into_rule())
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok((rules, total.max(0) as u64))
    }

    /// CRIT-005 対応: トランザクション内で RLS セッション変数を設定してからルールを更新する。
    /// tenant_id はルールエンティティ内のフィールドから取得し、WHERE 句にも含める。
    async fn update(&self, rule: &RateLimitRule) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        set_tenant_rls(&mut tx, &rule.tenant_id).await?;

        sqlx::query(
            r#"
            UPDATE ratelimit.rate_limit_rules
            SET name = $1, scope = $2, identifier_pattern = $3, limit_count = $4, window_secs = $5, algorithm = $6, enabled = $7, updated_at = $8
            WHERE id = $9 AND tenant_id = $10
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
        .bind(&rule.tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// CRIT-005 対応: トランザクション内で RLS セッション変数を設定してからルールを削除する。削除された場合 true を返す。
    async fn delete(&self, id: &Uuid, tenant_id: &str) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;
        set_tenant_rls(&mut tx, tenant_id).await?;

        let result = sqlx::query(
            r#"DELETE FROM ratelimit.rate_limit_rules WHERE id = $1 AND tenant_id = $2"#,
        )
        .bind(id)
        .bind(tenant_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(result.rows_affected() > 0)
    }

    /// PostgreSQL リポジトリではRedis状態のリセットは行わない（state_storeが担当）。
    async fn reset_state(&self, _key: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

/// PostgreSQL の行を RateLimitRule エンティティに変換するための中間構造体。
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
    tenant_id: String,
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
            tenant_id: self.tenant_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
