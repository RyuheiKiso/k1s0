use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entity::api_key::ApiKey;
use crate::domain::error::AuthError;
use crate::domain::repository::api_key_repository::ApiKeyRepository;

/// `ApiKeyPostgresRepository` は `ApiKeyRepository` の `PostgreSQL` 実装。
pub struct ApiKeyPostgresRepository {
    pool: PgPool,
    metrics: Option<Arc<k1s0_telemetry::metrics::Metrics>>,
}

impl ApiKeyPostgresRepository {
    #[allow(dead_code)]
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            metrics: None,
        }
    }

    #[must_use]
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

        // SEC-CRIT-002 修正: set_config の第3引数 true は SET LOCAL（トランザクションスコープのみ有効）を意味する。
        // トランザクション外で実行すると接続プール返却時に即座に無効化され RLS バイパスが発生するため、
        // pool.begin() でトランザクションを開始し &mut *tx に対して set_config と INSERT を実行する。
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&api_key.tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r"
            INSERT INTO auth.api_keys (
                id, tenant_id, name, key_hash, prefix, scopes,
                expires_at, revoked, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ",
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
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("create", "api_keys", start.elapsed().as_secs_f64());
        }
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ApiKey>> {
        let start = std::time::Instant::now();

        // CRITICAL-RUST-001 監査対応: auth.api_key_find_by_id は SECURITY DEFINER 関数（migration 022）。
        // FORCE ROW LEVEL SECURITY が有効な auth.api_keys テーブルに対して
        // 管理操作（テナント ID 不明の内部 API）では RLS バイパスが必要なため
        // 関数オーナー（DB オーナー）権限で実行する。
        let row = sqlx::query_as::<_, ApiKeyRow>(
            r"
            SELECT id, tenant_id, name, key_hash, prefix, scopes,
                   expires_at, revoked, created_at, updated_at
            FROM auth.api_key_find_by_id($1)
            ",
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

        // CRITICAL-RUST-001 監査対応: auth.api_key_find_by_prefix は SECURITY DEFINER 関数（migration 022）。
        // 認証ブートストラップフロー（テナント ID 不明の API キー検証）では
        // FORCE ROW LEVEL SECURITY を持つテーブルに対して RLS バイパスが必要。
        let row = sqlx::query_as::<_, ApiKeyRow>(
            r"
            SELECT id, tenant_id, name, key_hash, prefix, scopes,
                   expires_at, revoked, created_at, updated_at
            FROM auth.api_key_find_by_prefix($1)
            ",
        )
        .bind(prefix)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("find_by_prefix", "api_keys", start.elapsed().as_secs_f64());
        }
        Ok(row.map(Into::into))
    }

    async fn list_by_tenant(&self, tenant_id: &str) -> anyhow::Result<Vec<ApiKey>> {
        let start = std::time::Instant::now();

        // SEC-CRIT-002 修正: set_config の SET LOCAL はトランザクション外では即座に無効化されるため、
        // トランザクション内で set_config と SELECT を実行して RLS が正しく機能することを保証する。
        // list_by_tenant は WHERE 句でも tenant_id フィルタを行うが、RLS との二重防衛として設定する。
        let mut tx = self.pool.begin().await?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let rows = sqlx::query_as::<_, ApiKeyRow>(
            r"
            SELECT id, tenant_id, name, key_hash, prefix, scopes,
                   expires_at, revoked, created_at, updated_at
            FROM auth.api_keys
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            ",
        )
        .bind(tenant_id)
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("list_by_tenant", "api_keys", start.elapsed().as_secs_f64());
        }
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn revoke(&self, id: Uuid) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        // CRITICAL-RUST-001 監査対応: auth.api_key_revoke は SECURITY DEFINER 関数（migration 022）。
        // FORCE ROW LEVEL SECURITY が有効なため直接 UPDATE では RLS に遮断される。
        // 関数がオーナー権限で実行されることで RLS をバイパスして失効処理を実行する。
        // 更新された行の id を返す: 0 行 → キーが存在しない（NotFound）。
        let updated_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM auth.api_key_revoke($1)")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("revoke", "api_keys", start.elapsed().as_secs_f64());
        }

        if updated_id.is_none() {
            // M-10 対応: 型安全なドメインエラーを使用して適切な HTTP ステータスコードに変換する
            return Err(AuthError::NotFound(format!("api key が見つかりません: {id}")).into());
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        // CRITICAL-RUST-001 監査対応: auth.api_key_delete は SECURITY DEFINER 関数（migration 022）。
        // FORCE ROW LEVEL SECURITY が有効なため直接 DELETE では RLS に遮断される。
        // 削除された行の id を返す: 0 行 → キーが存在しない（NotFound）。
        let deleted_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM auth.api_key_delete($1)")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(ref m) = self.metrics {
            m.record_db_query_duration("delete", "api_keys", start.elapsed().as_secs_f64());
        }

        if deleted_id.is_none() {
            // M-10 対応: 型安全なドメインエラーを使用して適切な HTTP ステータスコードに変換する
            return Err(AuthError::NotFound(format!("api key が見つかりません: {id}")).into());
        }
        Ok(())
    }
}

/// `ApiKeyRow` は DB から取得した行を表す中間構造体。
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
