use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// `SessionMetadata` はセッションのメタデータ（デバイス情報、IP、UA）を表す。
/// 監査ログ・セッション一覧表示用に `PostgreSQL` に永続化される。
/// `tenant_id` は RLS `ポリシー（app.current_tenant_id）によるテナント分離に使用する`。
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionMetadata {
    pub id: Uuid,
    pub user_id: Uuid,
    /// テナント識別子。RLS ポリシーで `app.current_tenant_id` と照合される。
    pub tenant_id: String,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub revoked: bool,
}

/// `SaveSessionMetadataInput` はメタデータ保存用の入力パラメータ。
/// `tenant_id` は RLS ポリシーと INSERT カラム両方で必須となる。
pub struct SaveSessionMetadataInput {
    pub session_id: Uuid,
    pub user_id: Uuid,
    /// テナント識別子。RLS ポリシーが使用する `app.current_tenant_id` の設定に使用する。
    pub tenant_id: String,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
}

/// `SessionMetadataRepository` はセッションメタデータの永続化トレイト。
/// 全メソッドは RLS ポリシー適用のため `tenant_id` を受け取る。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SessionMetadataRepository: Send + Sync {
    async fn save_metadata(&self, input: &SaveSessionMetadataInput) -> anyhow::Result<()>;
    #[allow(dead_code)]
    /// テナントに属するユーザーセッション一覧を取得する。
    async fn list_by_user(
        &self,
        user_id: Uuid,
        tenant_id: &str,
    ) -> anyhow::Result<Vec<SessionMetadata>>;
    /// セッションを失効済みに更新する。RLS のため `tenant_id` を渡す。
    async fn mark_revoked(&self, session_id: Uuid, tenant_id: &str) -> anyhow::Result<()>;
    async fn health_check(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// `SessionMetadataPostgresRepository` は `PostgreSQL` を使ったメタデータリポジトリ。
pub struct SessionMetadataPostgresRepository {
    pool: Arc<PgPool>,
}

impl SessionMetadataPostgresRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// `PostgreSQL` の `user_sessions` テーブル行を表す内部構造体。
/// `tenant_id` は RLS で使用するカラムであり、SELECT 結果にも含まれる。
#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct SessionMetadataRow {
    id: Uuid,
    user_id: Uuid,
    tenant_id: String,
    device_id: Option<String>,
    device_name: Option<String>,
    device_type: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_accessed_at: DateTime<Utc>,
    revoked: bool,
}

impl From<SessionMetadataRow> for SessionMetadata {
    fn from(r: SessionMetadataRow) -> Self {
        SessionMetadata {
            id: r.id,
            user_id: r.user_id,
            tenant_id: r.tenant_id,
            device_id: r.device_id,
            device_name: r.device_name,
            device_type: r.device_type,
            ip_address: r.ip_address,
            user_agent: r.user_agent,
            created_at: r.created_at,
            expires_at: r.expires_at,
            last_accessed_at: r.last_accessed_at,
            revoked: r.revoked,
        }
    }
}

#[async_trait]
impl SessionMetadataRepository for SessionMetadataPostgresRepository {
    /// セッションメタデータを `PostgreSQL` に保存する。
    /// トランザクション内で `set_config` を呼び出し、RLS ポリシーが正しいテナントに適用されるようにする。
    async fn save_metadata(&self, input: &SaveSessionMetadataInput) -> anyhow::Result<()> {
        // トランザクションを開始し、set_config → INSERT をアトミックに実行する
        let mut tx = self.pool.begin().await?;

        // RLS ポリシー適用のため現在テナントを設定する（true = トランザクションスコープ）
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(&input.tenant_id)
            .execute(&mut *tx)
            .await?;

        // tenant_id カラムを含めてメタデータを挿入する。ON CONFLICT 時は last_accessed_at のみ更新する
        sqlx::query(
            "INSERT INTO session.user_sessions \
             (id, user_id, tenant_id, device_id, device_name, device_type, ip_address, user_agent, expires_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             ON CONFLICT (id) DO UPDATE SET \
                 last_accessed_at = NOW()",
        )
        .bind(input.session_id)
        .bind(input.user_id)
        .bind(&input.tenant_id)
        .bind(&input.device_id)
        .bind(&input.device_name)
        .bind(&input.device_type)
        .bind(&input.ip_address)
        .bind(&input.user_agent)
        .bind(input.expires_at)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// テナントに属するユーザーのセッション一覧を取得する。
    /// `set_config` で RLS ポリシーを適用し、クロステナントアクセスを防ぐ。
    async fn list_by_user(
        &self,
        user_id: Uuid,
        tenant_id: &str,
    ) -> anyhow::Result<Vec<SessionMetadata>> {
        // トランザクション内で set_config → SELECT をアトミックに実行する
        let mut tx = self.pool.begin().await?;

        // RLS ポリシー適用のため現在テナントを設定する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        let rows: Vec<SessionMetadataRow> = sqlx::query_as(
            "SELECT id, user_id, tenant_id, device_id, device_name, device_type, \
                    ip_address, user_agent, created_at, expires_at, \
                    last_accessed_at, revoked \
             FROM session.user_sessions \
             WHERE user_id = $1 \
             ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// セッションを失効済みに更新する。
    /// `set_config` で RLS ポリシーを適用し、他テナントのセッションを変更できないようにする。
    async fn mark_revoked(&self, session_id: Uuid, tenant_id: &str) -> anyhow::Result<()> {
        // トランザクション内で set_config → UPDATE をアトミックに実行する
        let mut tx = self.pool.begin().await?;

        // RLS ポリシー適用のため現在テナントを設定する
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query("UPDATE session.user_sessions SET revoked = true WHERE id = $1")
            .bind(session_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1")
            .execute(self.pool.as_ref())
            .await
            .map(|_| ())
            .map_err(|e| anyhow::anyhow!("postgres health check failed: {e}"))
    }
}

/// `NoopSessionMetadataRepository` は何もしないフォールバック実装。
/// `PostgreSQL` が設定されていない場合（dev/test 環境）に使用する。
pub struct NoopSessionMetadataRepository;

#[async_trait]
impl SessionMetadataRepository for NoopSessionMetadataRepository {
    async fn save_metadata(&self, _input: &SaveSessionMetadataInput) -> anyhow::Result<()> {
        tracing::debug!("noop: session metadata save skipped");
        Ok(())
    }

    /// Noop 実装のため `tenant_id` 引数は無視してから空リストを返す。
    async fn list_by_user(
        &self,
        _user_id: Uuid,
        _tenant_id: &str,
    ) -> anyhow::Result<Vec<SessionMetadata>> {
        tracing::debug!("noop: session metadata list skipped");
        Ok(Vec::new())
    }

    /// Noop 実装のため `tenant_id` 引数は無視して成功を返す。
    async fn mark_revoked(&self, _session_id: Uuid, _tenant_id: &str) -> anyhow::Result<()> {
        tracing::debug!("noop: session metadata mark_revoked skipped");
        Ok(())
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_session_metadata_from_row() {
        let now = Utc::now();
        let row = SessionMetadataRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            tenant_id: "tenant-a".to_string(),
            device_id: Some("device-1".to_string()),
            device_name: Some("iPhone 15".to_string()),
            device_type: Some("mobile".to_string()),
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            created_at: now,
            expires_at: now + chrono::Duration::hours(1),
            last_accessed_at: now,
            revoked: false,
        };

        let metadata: SessionMetadata = row.into();
        assert_eq!(metadata.tenant_id, "tenant-a");
        assert_eq!(metadata.device_name.as_deref(), Some("iPhone 15"));
        assert_eq!(metadata.device_type.as_deref(), Some("mobile"));
        assert_eq!(metadata.ip_address.as_deref(), Some("192.168.1.1"));
        assert!(!metadata.revoked);
    }

    #[test]
    fn test_session_metadata_serialization() {
        let metadata = SessionMetadata {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            tenant_id: "tenant-b".to_string(),
            device_id: None,
            device_name: Some("Chrome".to_string()),
            device_type: Some("browser".to_string()),
            ip_address: Some("10.0.0.1".to_string()),
            user_agent: None,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            last_accessed_at: Utc::now(),
            revoked: false,
        };

        let json = serde_json::to_value(&metadata).unwrap();
        assert_eq!(json["tenant_id"], "tenant-b");
        assert_eq!(json["device_name"], "Chrome");
        assert_eq!(json["device_type"], "browser");
        assert_eq!(json["revoked"], false);
    }

    #[tokio::test]
    async fn test_noop_metadata_repository() {
        let repo = NoopSessionMetadataRepository;

        let input = SaveSessionMetadataInput {
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            tenant_id: "tenant-a".to_string(),
            device_id: None,
            device_name: None,
            device_type: None,
            ip_address: None,
            user_agent: None,
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };

        assert!(repo.save_metadata(&input).await.is_ok());
        assert!(repo
            .list_by_user(Uuid::new_v4(), "tenant-a")
            .await
            .unwrap()
            .is_empty());
        assert!(repo.mark_revoked(Uuid::new_v4(), "tenant-a").await.is_ok());
    }
}
