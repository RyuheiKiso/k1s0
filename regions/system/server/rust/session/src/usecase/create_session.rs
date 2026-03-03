use std::collections::HashMap;
use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

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
    event_publisher: Arc<dyn SessionEventPublisher>,
    default_ttl: i64,
    max_ttl: i64,
}

impl CreateSessionUseCase {
    pub fn new(
        repo: Arc<dyn SessionRepository>,
        event_publisher: Arc<dyn SessionEventPublisher>,
        default_ttl: i64,
        max_ttl: i64,
    ) -> Self {
        Self {
            repo,
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

        let now = Utc::now();
        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: input.user_id.clone(),
            device_id: input.device_id.clone(),
            device_name: input.device_name.clone(),
            device_type: input.device_type.clone(),
            user_agent: input.user_agent.clone(),
            ip_address: input.ip_address.clone(),
            token: Uuid::new_v4().to_string(),
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

        Ok(CreateSessionOutput { session })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let uc = CreateSessionUseCase::new(Arc::new(mock), Arc::new(mock_publisher), 3600, 86400);
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
