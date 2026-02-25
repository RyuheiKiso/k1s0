use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// SessionMetadata はセッションのメタデータ（デバイス情報、IP、UA）を表す。
/// 監査ログ・セッション一覧表示用に PostgreSQL に永続化される。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionMetadata {
    pub id: Uuid,
    pub user_id: Uuid,
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

/// SaveSessionMetadataInput はメタデータ保存用の入力パラメータ。
pub struct SaveSessionMetadataInput {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
}

/// SessionMetadataRepository はセッションメタデータの永続化トレイト。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SessionMetadataRepository: Send + Sync {
    async fn save_metadata(&self, input: &SaveSessionMetadataInput) -> anyhow::Result<()>;
    async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<SessionMetadata>>;
    async fn mark_revoked(&self, session_id: Uuid) -> anyhow::Result<()>;
}

/// SessionMetadataPostgresRepository は PostgreSQL を使ったメタデータリポジトリ。
pub struct SessionMetadataPostgresRepository {
    pool: Arc<PgPool>,
}

impl SessionMetadataPostgresRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SessionMetadataRow {
    id: Uuid,
    user_id: Uuid,
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
    async fn save_metadata(&self, input: &SaveSessionMetadataInput) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO session.user_sessions \
             (id, user_id, device_id, device_name, device_type, ip_address, user_agent, expires_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO UPDATE SET \
                 last_accessed_at = NOW()",
        )
        .bind(input.session_id)
        .bind(input.user_id)
        .bind(&input.device_id)
        .bind(&input.device_name)
        .bind(&input.device_type)
        .bind(&input.ip_address)
        .bind(&input.user_agent)
        .bind(input.expires_at)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<SessionMetadata>> {
        let rows: Vec<SessionMetadataRow> = sqlx::query_as(
            "SELECT id, user_id, device_id, device_name, device_type, \
                    ip_address, user_agent, created_at, expires_at, \
                    last_accessed_at, revoked \
             FROM session.user_sessions \
             WHERE user_id = $1 \
             ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn mark_revoked(&self, session_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE session.user_sessions SET revoked = true WHERE id = $1",
        )
        .bind(session_id)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }
}

/// NoopSessionMetadataRepository は何もしないフォールバック実装。
/// PostgreSQL が設定されていない場合に使用する。
pub struct NoopSessionMetadataRepository;

#[async_trait]
impl SessionMetadataRepository for NoopSessionMetadataRepository {
    async fn save_metadata(&self, _input: &SaveSessionMetadataInput) -> anyhow::Result<()> {
        tracing::debug!("noop: session metadata save skipped");
        Ok(())
    }

    async fn list_by_user(&self, _user_id: Uuid) -> anyhow::Result<Vec<SessionMetadata>> {
        tracing::debug!("noop: session metadata list skipped");
        Ok(Vec::new())
    }

    async fn mark_revoked(&self, _session_id: Uuid) -> anyhow::Result<()> {
        tracing::debug!("noop: session metadata mark_revoked skipped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_metadata_from_row() {
        let now = Utc::now();
        let row = SessionMetadataRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
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
            device_id: None,
            device_name: None,
            device_type: None,
            ip_address: None,
            user_agent: None,
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };

        assert!(repo.save_metadata(&input).await.is_ok());
        assert!(repo.list_by_user(Uuid::new_v4()).await.unwrap().is_empty());
        assert!(repo.mark_revoked(Uuid::new_v4()).await.is_ok());
    }
}
