use crate::domain::model::{
    CreateSessionPayload, RefreshSessionPayload, RevokeAllSessionsPayload, RevokeSessionPayload,
    UserError,
};
use crate::infrastructure::grpc::SessionGrpcClient;
use std::sync::Arc;
use tracing::instrument;

pub struct SessionMutationResolver {
    client: Arc<SessionGrpcClient>,
}

impl SessionMutationResolver {
    pub fn new(client: Arc<SessionGrpcClient>) -> Self {
        Self { client }
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_session(
        &self,
        user_id: &str,
        device_id: &str,
        device_name: Option<&str>,
        device_type: Option<&str>,
        user_agent: Option<&str>,
        ip_address: Option<&str>,
        ttl_seconds: Option<i32>,
    ) -> CreateSessionPayload {
        match self
            .client
            .create_session(
                user_id,
                device_id,
                device_name,
                device_type,
                user_agent,
                ip_address,
                ttl_seconds,
            )
            .await
        {
            Ok((session, token)) => CreateSessionPayload {
                session: Some(session),
                token: Some(token),
                errors: vec![],
            },
            Err(e) => CreateSessionPayload {
                session: None,
                token: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn refresh_session(
        &self,
        session_id: &str,
        ttl_seconds: Option<i32>,
    ) -> RefreshSessionPayload {
        match self.client.refresh_session(session_id, ttl_seconds).await {
            Ok((session, token)) => RefreshSessionPayload {
                session: Some(session),
                token: Some(token),
                errors: vec![],
            },
            Err(e) => RefreshSessionPayload {
                session: None,
                token: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn revoke_session(&self, session_id: &str) -> RevokeSessionPayload {
        match self.client.revoke_session(session_id).await {
            Ok(success) => RevokeSessionPayload {
                success,
                errors: vec![],
            },
            Err(e) => RevokeSessionPayload {
                success: false,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn revoke_all_sessions(&self, user_id: &str) -> RevokeAllSessionsPayload {
        match self.client.revoke_all_sessions(user_id).await {
            Ok(count) => RevokeAllSessionsPayload {
                revoked_count: count as i32,
                errors: vec![],
            },
            Err(e) => RevokeAllSessionsPayload {
                revoked_count: 0,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }
}
