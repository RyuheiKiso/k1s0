use std::sync::Arc;

use crate::adapter::repository::session_metadata_postgres::SessionMetadataRepository;
use crate::domain::repository::SessionRepository;
use crate::error::SessionError;
use crate::infrastructure::kafka_producer::SessionEventPublisher;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RevokeSessionInput {
    pub id: String,
}

pub struct RevokeSessionUseCase {
    repo: Arc<dyn SessionRepository>,
    metadata_repo: Arc<dyn SessionMetadataRepository>,
    event_publisher: Arc<dyn SessionEventPublisher>,
}

impl RevokeSessionUseCase {
    pub fn new(
        repo: Arc<dyn SessionRepository>,
        metadata_repo: Arc<dyn SessionMetadataRepository>,
        event_publisher: Arc<dyn SessionEventPublisher>,
    ) -> Self {
        Self {
            repo,
            metadata_repo,
            event_publisher,
        }
    }

    pub async fn execute(&self, input: &RevokeSessionInput) -> Result<(), SessionError> {
        let mut session = self
            .repo
            .find_by_id(&input.id)
            .await?
            .ok_or_else(|| SessionError::NotFound(input.id.clone()))?;

        if session.revoked {
            return Err(SessionError::AlreadyRevoked(input.id.clone()));
        }

        session.revoke();
        self.repo.save(&session).await?;
        self.event_publisher
            .publish_session_revoked(&session.id, &session.user_id)
            .await
            .map_err(|e| SessionError::Internal(e.to_string()))?;

        // セッションの無効化をメタデータに反映する
        // メタデータ更新は補助的な処理のため、失敗してもセッション処理は継続する。
        // ただしサイレント無視は監査ログの欠落を検知できないため、警告ログを記録する。
        if let Ok(session_uuid) = uuid::Uuid::parse_str(&session.id) {
            if let Err(e) = self.metadata_repo.mark_revoked(session_uuid).await {
                tracing::warn!(
                    error = %e,
                    "セッションメタデータの保存に失敗しました。監査ログが欠落する可能性があります"
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::repository::session_metadata_postgres::NoopSessionMetadataRepository;
    use crate::domain::entity::session::Session;
    use crate::domain::repository::session_repository::MockSessionRepository;
    use crate::infrastructure::kafka_producer::MockSessionEventPublisher;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    fn make_session() -> Session {
        Session {
            id: "sess-1".to_string(),
            user_id: "user-1".to_string(),
            device_id: "device-1".to_string(),
            device_name: Some("device".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: Some("ua".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            token: "tok-1".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            last_accessed_at: None,
            revoked: false,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(Some(make_session())));
        mock.expect_save().returning(|_| Ok(()));
        let mut mock_publisher = MockSessionEventPublisher::new();
        mock_publisher
            .expect_publish_session_revoked()
            .withf(|session_id, user_id| session_id == "sess-1" && user_id == "user-1")
            .returning(|_, _| Ok(()));

        let uc = RevokeSessionUseCase::new(
            Arc::new(mock),
            Arc::new(NoopSessionMetadataRepository),
            Arc::new(mock_publisher),
        );
        let result = uc
            .execute(&RevokeSessionInput {
                id: "sess-1".to_string(),
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn metadata_mark_revoked_error_does_not_fail_usecase() {
        // メタデータリポジトリがエラーを返してもユースケースが成功することを確認する
        use crate::adapter::repository::session_metadata_postgres::MockSessionMetadataRepository;

        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Ok(Some(make_session())));
        mock.expect_save().returning(|_| Ok(()));
        let mut mock_publisher = MockSessionEventPublisher::new();
        mock_publisher
            .expect_publish_session_revoked()
            .returning(|_, _| Ok(()));

        // mark_revoked が常にエラーを返すモックを設定する
        let mut mock_meta = MockSessionMetadataRepository::new();
        mock_meta
            .expect_mark_revoked()
            .returning(|_| Err(anyhow::anyhow!("db connection error")));

        let uc = RevokeSessionUseCase::new(
            Arc::new(mock),
            Arc::new(mock_meta),
            Arc::new(mock_publisher),
        );
        // メタデータ更新失敗でもユースケース全体は成功する
        let result = uc
            .execute(&RevokeSessionInput {
                id: "sess-1".to_string(),
            })
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = RevokeSessionUseCase::new(
            Arc::new(mock),
            Arc::new(NoopSessionMetadataRepository),
            Arc::new(crate::infrastructure::kafka_producer::NoopSessionEventPublisher),
        );
        let result = uc
            .execute(&RevokeSessionInput {
                id: "missing".to_string(),
            })
            .await;
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[tokio::test]
    async fn already_revoked() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_id().returning(|_| {
            let mut s = make_session();
            s.revoked = true;
            Ok(Some(s))
        });

        let uc = RevokeSessionUseCase::new(
            Arc::new(mock),
            Arc::new(NoopSessionMetadataRepository),
            Arc::new(crate::infrastructure::kafka_producer::NoopSessionEventPublisher),
        );
        let result = uc
            .execute(&RevokeSessionInput {
                id: "sess-1".to_string(),
            })
            .await;
        assert!(matches!(result, Err(SessionError::AlreadyRevoked(_))));
    }
}
