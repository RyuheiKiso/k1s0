use crate::domain::model::Session;
use crate::infrastructure::grpc::SessionGrpcClient;
use std::sync::Arc;
use tracing::instrument;

pub struct SessionQueryResolver {
    client: Arc<SessionGrpcClient>,
}

impl SessionQueryResolver {
    pub fn new(client: Arc<SessionGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_session(&self, session_id: &str) -> anyhow::Result<Option<Session>> {
        self.client.get_session(session_id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_user_sessions(&self, user_id: &str) -> anyhow::Result<Vec<Session>> {
        self.client.list_user_sessions(user_id).await
    }
}
