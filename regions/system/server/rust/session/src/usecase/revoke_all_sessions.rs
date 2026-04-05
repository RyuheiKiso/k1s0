use std::sync::Arc;

use crate::adapter::repository::session_metadata_postgres::SessionMetadataRepository;
use crate::domain::repository::SessionRepository;
use crate::error::SessionError;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct RevokeAllSessionsInput {
    pub user_id: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RevokeAllSessionsOutput {
    pub count: u32,
}

pub struct RevokeAllSessionsUseCase {
    repo: Arc<dyn SessionRepository>,
    metadata_repo: Arc<dyn SessionMetadataRepository>,
}

impl RevokeAllSessionsUseCase {
    pub fn new(
        repo: Arc<dyn SessionRepository>,
        metadata_repo: Arc<dyn SessionMetadataRepository>,
    ) -> Self {
        Self {
            repo,
            metadata_repo,
        }
    }

    pub async fn execute(
        &self,
        input: &RevokeAllSessionsInput,
    ) -> Result<RevokeAllSessionsOutput, SessionError> {
        let sessions = self.repo.find_by_user_id(&input.user_id).await?;
        let mut count = 0u32;

        for mut session in sessions {
            if !session.revoked {
                session.revoke();
                self.repo.save(&session).await?;

                // セッションの無効化をメタデータに反映する。
                // tenant_id をセッションエンティティから取得して mark_revoked に渡し、RLS を正しく適用する。
                // メタデータ更新は補助的な処理のため、失敗してもセッション処理は継続する。
                // ただしサイレント無視は監査ログの欠落を検知できないため、警告ログを記録する。
                if let Ok(session_uuid) = uuid::Uuid::parse_str(&session.id) {
                    if let Err(e) = self
                        .metadata_repo
                        .mark_revoked(session_uuid, &session.tenant_id)
                        .await
                    {
                        tracing::warn!(
                            error = %e,
                            "セッションメタデータの保存に失敗しました。監査ログが欠落する可能性があります"
                        );
                    }
                }

                count += 1;
            }
        }

        Ok(RevokeAllSessionsOutput { count })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::adapter::repository::session_metadata_postgres::NoopSessionMetadataRepository;
    use crate::domain::entity::session::Session;
    use crate::domain::repository::session_repository::MockSessionRepository;
    use chrono::{Duration, Utc};
    use std::collections::HashMap;

    fn make_session(id: &str, revoked: bool) -> Session {
        Session {
            id: id.to_string(),
            user_id: "user-1".to_string(),
            tenant_id: "tenant-a".to_string(),
            device_id: format!("device-{}", id),
            device_name: Some("device".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: Some("ua".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            token: format!("tok-{}", id),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            last_accessed_at: None,
            revoked,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| {
            Ok(vec![
                make_session("s1", false),
                make_session("s2", false),
                make_session("s3", true),
            ])
        });
        mock.expect_save().returning(|_| Ok(()));

        let uc =
            RevokeAllSessionsUseCase::new(Arc::new(mock), Arc::new(NoopSessionMetadataRepository));
        let result = uc
            .execute(&RevokeAllSessionsInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(result.count, 2);
    }

    #[tokio::test]
    async fn metadata_mark_revoked_error_does_not_fail_usecase() {
        // メタデータリポジトリがエラーを返してもユースケースが成功することを確認する
        use crate::adapter::repository::session_metadata_postgres::MockSessionMetadataRepository;

        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id()
            .returning(|_| Ok(vec![make_session("s1", false), make_session("s2", false)]));
        mock.expect_save().returning(|_| Ok(()));

        // mark_revoked が常にエラーを返すモックを設定する（session_id と tenant_id の2引数）
        let mut mock_meta = MockSessionMetadataRepository::new();
        mock_meta
            .expect_mark_revoked()
            .returning(|_, _| Err(anyhow::anyhow!("db connection error")));

        let uc = RevokeAllSessionsUseCase::new(Arc::new(mock), Arc::new(mock_meta));
        // メタデータ更新失敗でもユースケース全体は成功し、対象セッション数が返る
        let result = uc
            .execute(&RevokeAllSessionsInput {
                user_id: "user-1".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(result.count, 2);
    }

    #[tokio::test]
    async fn no_sessions() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| Ok(vec![]));

        let uc =
            RevokeAllSessionsUseCase::new(Arc::new(mock), Arc::new(NoopSessionMetadataRepository));
        let result = uc
            .execute(&RevokeAllSessionsInput {
                user_id: "user-2".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(result.count, 0);
    }
}
