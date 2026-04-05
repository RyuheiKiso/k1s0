use crate::domain::model::auth::{AuditEventType, AuditResult};
use crate::domain::model::{AuditLogConnection, PermissionCheck, Role, User};
use crate::infrastructure::grpc::AuthGrpcClient;
use std::sync::Arc;
use tracing::instrument;

pub struct AuthQueryResolver {
    client: Arc<AuthGrpcClient>,
}

impl AuthQueryResolver {
    pub fn new(client: Arc<AuthGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_user(&self, user_id: &str) -> anyhow::Result<Option<User>> {
        self.client.get_user(user_id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_users(
        &self,
        first: Option<i32>,
        after: Option<i32>,
        search: Option<&str>,
        enabled: Option<bool>,
    ) -> anyhow::Result<Vec<User>> {
        let page_size = first.unwrap_or(20);
        let page = after.map(|a| a + 1).unwrap_or(1);
        self.client
            .list_users(Some(page_size), Some(page), search, enabled)
            .await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_user_roles(&self, user_id: &str) -> anyhow::Result<Vec<Role>> {
        self.client.get_user_roles(user_id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn check_permission(
        &self,
        user_id: Option<&str>,
        permission: &str,
        resource: &str,
        roles: &[String],
    ) -> anyhow::Result<PermissionCheck> {
        self.client
            .check_permission(user_id, permission, resource, roles)
            .await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn search_audit_logs(
        &self,
        first: Option<i32>,
        after: Option<i32>,
        user_id: Option<&str>,
        event_type: Option<AuditEventType>,
        result: Option<AuditResult>,
    ) -> anyhow::Result<AuditLogConnection> {
        let page_size = first.unwrap_or(20);
        let page = after.map(|a| a + 1).unwrap_or(1);
        let (logs, total_count, has_next) = self
            .client
            .search_audit_logs(Some(page_size), Some(page), user_id, event_type, result)
            .await?;
        Ok(AuditLogConnection {
            logs,
            total_count,
            has_next,
        })
    }
}
