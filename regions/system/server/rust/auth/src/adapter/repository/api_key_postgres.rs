use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::api_key::ApiKey;
use crate::domain::repository::api_key_repository::ApiKeyRepository;

/// ApiKeyPostgresRepository は ApiKeyRepository の PostgreSQL 実装。
pub struct ApiKeyPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl ApiKeyPostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    pub fn with_metrics(pool: PgPool, metrics: Arc<k1s0_telemetry::metrics::Metrics>) -> Self {
        Self {
            pool,
            metrics: Some(metrics),
        }
    }
}

#[async_trait]
impl ApiKeyRepository for ApiKeyPostgresRepository {
    async fn create(&self, api_key: &ApiKey) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        let scopes_json = serde_json::to_value(&api_key.scopes)?;

        sqlx::query(
            r#"
            INSERT INTO auth.api_keys (
                id, tenant_id, name, key_hash, prefix, scopes,
                expires_at, revoked, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(api_key.id)
        .bind(&api_key.tenant_id)
        .bind(&api_key.name)
        .bind(&api_key.key_hash)
        .bind(&api_key.prefix)
        .bind(&scopes_json)
        .bind(api_key.expires_at)
        .bind(api_key.revoked)
        .bind(api_key.created_at)
        .bind(api_key.updated_at)
        .execute(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("create", "api_keys", start.elapsed().as_secs_f64());
        }
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ApiKey>> {
        let start = std::time::Instant::now();

        let row = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            SELECT id, tenant_id, name, key_hash, prefix, scopes,
                   expires_at, revoked, created_at, updated_at
            FROM auth.api_keys
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("find_by_id", "api_keys", start.elapsed().as_secs_f64());
        }
        Ok(row.map(Into::into))
    }

    async fn find_by_prefix(&self, prefix: &str) -> anyhow::Result<Option<ApiKey>> {
        let start = std::time::Instant::now();

        let row = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            SELECT id, tenant_id, name, key_hash, prefix, scopes,
                   expires_at, revoked, created_at, updated_at
            FROM auth.api_keys
            WHERE prefix = $1
            "#,
        )
        .bind(prefix)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "find_by_prefix",
                "api_keys",
                start.elapsed().as_secs_f64(),
            );
        }
        Ok(row.map(Into::into))
    }

    async fn list_by_tenant(&self, tenant_id: &str) -> anyhow::Result<Vec<ApiKey>> {
        let start = std::time::Instant::now();

        let rows = sqlx::query_as::<_, ApiKeyRow>(
            r#"
            SELECT id, tenant_id, name, key_hash, prefix, scopes,
                   expires_at, revoked, created_at, updated_at
            FROM auth.api_keys
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration(
                "list_by_tenant",
                "api_keys",
                start.elapsed().as_secs_f64(),
            );
        }
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn revoke(&self, id: Uuid) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        let result = sqlx::query(
            r#"
            UPDATE auth.api_keys
            SET revoked = true, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("revoke", "api_keys", start.elapsed().as_secs_f64());
        }

        if result.rows_affected() == 0 {
            anyhow::bail!("api key not found: {}", id);
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        let result = sqlx::query("DELETE FROM auth.api_keys WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("delete", "api_keys", start.elapsed().as_secs_f64());
        }

        if result.rows_affected() == 0 {
            anyhow::bail!("api key not found: {}", id);
        }
        Ok(())
    }
}

/// ApiKeyRow は DB から取得した行を表す中間構造体。
#[derive(Debug, Clone, sqlx::FromRow)]
struct ApiKeyRow {
    pub id: Uuid,
    pub tenant_id: String,
    pub name: String,
    pub key_hash: String,
    pub prefix: String,
    pub scopes: serde_json::Value,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ApiKeyRow> for ApiKey {
    fn from(row: ApiKeyRow) -> Self {
        let scopes: Vec<String> = serde_json::from_value(row.scopes).unwrap_or_default();
        ApiKey {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            key_hash: row.key_hash,
            prefix: row.prefix,
            scopes,
            expires_at: row.expires_at,
            revoked: row.revoked,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::api_key_repository::MockApiKeyRepository;

    #[tokio::test]
    async fn test_mock_create_and_find() {
        let mut mock = MockApiKeyRepository::new();
        mock.expect_create().returning(|_| Ok(()));

        let now = chrono::Utc::now();
        let key = ApiKey {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            name: "Test".to_string(),
            key_hash: "hash".to_string(),
            prefix: "k1s0_test".to_string(),
            scopes: vec!["read".to_string()],
            expires_at: None,
            revoked: false,
            created_at: now,
            updated_at: now,
        };

        assert!(mock.create(&key).await.is_ok());
    }

    #[test]
    fn test_api_key_row_to_api_key() {
        let now = chrono::Utc::now();
        let row = ApiKeyRow {
            id: Uuid::new_v4(),
            tenant_id: "tenant-1".to_string(),
            name: "Row Key".to_string(),
            key_hash: "hash".to_string(),
            prefix: "k1s0_row".to_string(),
            scopes: serde_json::json!(["read", "write"]),
            expires_at: None,
            revoked: false,
            created_at: now,
            updated_at: now,
        };

        let key: ApiKey = row.into();
        assert_eq!(key.name, "Row Key");
        assert_eq!(key.scopes, vec!["read", "write"]);
    }
}
