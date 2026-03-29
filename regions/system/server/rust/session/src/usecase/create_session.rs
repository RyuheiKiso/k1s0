use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use chrono::{Duration, Utc};
use rand::RngCore;
use uuid::Uuid;

use crate::adapter::repository::session_metadata_postgres::{
    SaveSessionMetadataInput, SessionMetadataRepository,
};
use crate::domain::entity::session::Session;
use crate::domain::repository::SessionRepository;
use crate::domain::service::SessionDomainService;
use crate::error::SessionError;
use crate::infrastructure::kafka_producer::SessionEventPublisher;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateSessionInput {
    pub user_id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub ttl_seconds: Option<i64>,
    pub max_devices: Option<u32>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CreateSessionOutput {
    pub session: Session,
}

pub struct CreateSessionUseCase {
    repo: Arc<dyn SessionRepository>,
    metadata_repo: Arc<dyn SessionMetadataRepository>,
    event_publisher: Arc<dyn SessionEventPublisher>,
    default_ttl: i64,
    max_ttl: i64,
}

impl CreateSessionUseCase {
    pub fn new(
        repo: Arc<dyn SessionRepository>,
        metadata_repo: Arc<dyn SessionMetadataRepository>,
        event_publisher: Arc<dyn SessionEventPublisher>,
        default_ttl: i64,
        max_ttl: i64,
    ) -> Self {
        Self {
            repo,
            metadata_repo,
            event_publisher,
            default_ttl,
            max_ttl,
        }
    }

    pub async fn execute(
        &self,
        input: &CreateSessionInput,
    ) -> Result<CreateSessionOutput, SessionError> {
        let ttl = input.ttl_seconds.unwrap_or(self.default_ttl);
        SessionDomainService::validate_create_request(&input.device_id, ttl, self.max_ttl)?;

        let max_devices = input.max_devices.unwrap_or(10).max(1) as usize;
        let mut existing = self.repo.find_by_user_id(&input.user_id).await?;
        existing.sort_by_key(|s| s.created_at);

        let valid_sessions: Vec<Session> = existing.into_iter().filter(|s| s.is_valid()).collect();
        let revoke_count =
            SessionDomainService::compute_revoke_count(valid_sessions.len(), max_devices);
        if revoke_count > 0 {
            for mut old in valid_sessions.into_iter().take(revoke_count) {
                old.revoke();
                self.repo
                    .save(&old)
                    .await
                    .map_err(|e| SessionError::Internal(e.to_string()))?;
            }
        }

        // MED-12 監査対応: UUID v4（122bit エントロピー）から OsRng を使った
        // 256bit（32byte）セキュアランダムトークンに変更する。
        // URL_SAFE_NO_PAD エンコードにより URL に安全な文字列として保存・送信できる。
        let mut token_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut token_bytes);
        let session_token =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&token_bytes);

        let now = Utc::now();
        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: input.user_id.clone(),
            device_id: input.device_id.clone(),
            device_name: input.device_name.clone(),
            device_type: input.device_type.clone(),
            user_agent: input.user_agent.clone(),
            ip_address: input.ip_address.clone(),
            token: session_token,
            expires_at: now + Duration::seconds(ttl),
            created_at: now,
            last_accessed_at: Some(now),
            revoked: false,
            metadata: input.metadata.clone().unwrap_or_default(),
        };

        self.repo
            .save(&session)
            .await
            .map_err(|e| SessionError::Internal(e.to_string()))?;
        self.event_publisher
            .publish_session_created(&session)
            .await
            .map_err(|e| SessionError::Internal(e.to_string()))?;

        // 監査ログ・セッション一覧用のメタデータを保存する
        if let Ok(user_uuid) = Uuid::parse_str(&session.user_id) {
            if let Ok(session_uuid) = Uuid::parse_str(&session.id) {
                let meta_input = SaveSessionMetadataInput {
                    session_id: session_uuid,
                    user_id: user_uuid,
                    device_id: Some(input.device_id.clone()),
                    device_name: input.device_name.clone(),
                    device_type: input.device_type.clone(),
                    ip_address: input.ip_address.clone(),
                    user_agent: input.user_agent.clone(),
                    expires_at: session.expires_at,
                };
                // メタデータ保存は補助的な処理のため、失敗してもセッション処理は継続する。
                // ただしサイレント無視は監査ログの欠落を検知できないため、警告ログを記録する。
                if let Err(e) = self.metadata_repo.save_metadata(&meta_input).await {
                    tracing::warn!(
                        error = %e,
                        "セッションメタデータの保存に失敗しました。監査ログが欠落する可能性があります"
                    );
                }
            }
        }

        Ok(CreateSessionOutput { session })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::adapter::repository::session_metadata_postgres::NoopSessionMetadataRepository;
    use crate::domain::repository::session_repository::MockSessionRepository;
    use crate::infrastructure::kafka_producer::MockSessionEventPublisher;

    #[tokio::test]
    async fn success() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| Ok(vec![]));
        mock.expect_save().returning(|_| Ok(()));
        let mut mock_publisher = MockSessionEventPublisher::new();
        mock_publisher
            .expect_publish_session_created()
            .returning(|_| Ok(()));

        let uc = CreateSessionUseCase::new(
            Arc::new(mock),
            Arc::new(NoopSessionMetadataRepository),
            Arc::new(mock_publisher),
            3600,
            86400,
        );
        let input = CreateSessionInput {
            user_id: "user-1".to_string(),
            device_id: "device-1".to_string(),
            device_name: Some("my device".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            ttl_seconds: Some(7200),
            max_devices: None,
            metadata: Some(HashMap::from([("ip".to_string(), "127.0.0.1".to_string())])),
        };
        let result = uc.execute(&input).await.unwrap();
        assert_eq!(result.session.user_id, "user-1");
        assert!(!result.session.id.is_empty());
        assert!(!result.session.token.is_empty());
        assert!(!result.session.revoked);
        assert_eq!(result.session.metadata.get("ip").unwrap(), "127.0.0.1");
    }

    #[tokio::test]
    async fn default_ttl() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| Ok(vec![]));
        mock.expect_save().returning(|_| Ok(()));

        let uc = CreateSessionUseCase::new(
            Arc::new(mock),
            Arc::new(NoopSessionMetadataRepository),
            Arc::new(crate::infrastructure::kafka_producer::NoopSessionEventPublisher),
            3600,
            86400,
        );
        let input = CreateSessionInput {
            user_id: "user-2".to_string(),
            device_id: "device-2".to_string(),
            device_name: None,
            device_type: None,
            user_agent: None,
            ip_address: None,
            ttl_seconds: None,
            max_devices: None,
            metadata: None,
        };
        let result = uc.execute(&input).await.unwrap();
        assert_eq!(result.session.user_id, "user-2");
    }

    #[tokio::test]
    async fn invalid_ttl() {
        let mock = MockSessionRepository::new();
        let uc = CreateSessionUseCase::new(
            Arc::new(mock),
            Arc::new(NoopSessionMetadataRepository),
            Arc::new(crate::infrastructure::kafka_producer::NoopSessionEventPublisher),
            3600,
            86400,
        );
        let input = CreateSessionInput {
            user_id: "user-3".to_string(),
            device_id: "device-3".to_string(),
            device_name: None,
            device_type: None,
            user_agent: None,
            ip_address: None,
            ttl_seconds: Some(100000),
            max_devices: None,
            metadata: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(SessionError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn repo_error() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| Ok(vec![]));
        mock.expect_save()
            .returning(|_| Err(SessionError::Internal("db error".to_string())));

        let uc = CreateSessionUseCase::new(
            Arc::new(mock),
            Arc::new(NoopSessionMetadataRepository),
            Arc::new(crate::infrastructure::kafka_producer::NoopSessionEventPublisher),
            3600,
            86400,
        );
        let input = CreateSessionInput {
            user_id: "user-4".to_string(),
            device_id: "device-4".to_string(),
            device_name: None,
            device_type: None,
            user_agent: None,
            ip_address: None,
            ttl_seconds: Some(3600),
            max_devices: None,
            metadata: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(result, Err(SessionError::Internal(_))));
    }

    #[tokio::test]
    async fn metadata_save_error_does_not_fail_usecase() {
        // メタデータリポジトリがエラーを返してもユースケースが成功することを確認する
        use crate::adapter::repository::session_metadata_postgres::MockSessionMetadataRepository;

        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| Ok(vec![]));
        mock.expect_save().returning(|_| Ok(()));
        let mut mock_publisher = MockSessionEventPublisher::new();
        mock_publisher
            .expect_publish_session_created()
            .returning(|_| Ok(()));

        // メタデータ保存が常にエラーを返すモックを設定する
        let mut mock_meta = MockSessionMetadataRepository::new();
        mock_meta
            .expect_save_metadata()
            .returning(|_| Err(anyhow::anyhow!("db connection error")));

        let uc = CreateSessionUseCase::new(
            Arc::new(mock),
            Arc::new(mock_meta),
            Arc::new(mock_publisher),
            3600,
            86400,
        );
        let input = CreateSessionInput {
            // user_id は UUID 形式でなければメタデータ保存パスに入らないため UUID を使用する
            user_id: "00000000-0000-0000-0000-000000000001".to_string(),
            device_id: "device-meta-err".to_string(),
            device_name: Some("test device".to_string()),
            device_type: Some("desktop".to_string()),
            user_agent: None,
            ip_address: None,
            ttl_seconds: Some(3600),
            max_devices: None,
            metadata: None,
        };
        // メタデータ保存失敗でもユースケース全体は成功する
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn too_many_sessions() {
        let mut mock = MockSessionRepository::new();
        mock.expect_find_by_user_id().returning(|_| {
            let sessions: Vec<Session> = (0..10)
                .map(|i| Session {
                    id: format!("sess-{}", i),
                    user_id: "user-busy".to_string(),
                    device_id: format!("device-{}", i),
                    device_name: None,
                    device_type: None,
                    user_agent: None,
                    ip_address: None,
                    token: format!("tok-{}", i),
                    expires_at: Utc::now() + Duration::hours(1),
                    created_at: Utc::now(),
                    last_accessed_at: Some(Utc::now()),
                    revoked: false,
                    metadata: HashMap::new(),
                })
                .collect();
            Ok(sessions)
        });
        mock.expect_save().returning(|_| Ok(()));

        let uc = CreateSessionUseCase::new(
            Arc::new(mock),
            Arc::new(NoopSessionMetadataRepository),
            Arc::new(crate::infrastructure::kafka_producer::NoopSessionEventPublisher),
            3600,
            86400,
        );
        let input = CreateSessionInput {
            user_id: "user-busy".to_string(),
            device_id: "device-new".to_string(),
            device_name: None,
            device_type: None,
            user_agent: None,
            ip_address: None,
            ttl_seconds: Some(3600),
            max_devices: None,
            metadata: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());
    }
}
