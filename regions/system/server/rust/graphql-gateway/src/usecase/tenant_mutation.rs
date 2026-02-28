use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::{CreateTenantPayload, UpdateTenantPayload, UserError};
use crate::infra::grpc::TenantGrpcClient;

pub struct TenantMutationResolver {
    client: Arc<TenantGrpcClient>,
}

impl TenantMutationResolver {
    pub fn new(client: Arc<TenantGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_tenant(
        &self,
        name: &str,
        owner_user_id: &str,
    ) -> CreateTenantPayload {
        match self.client.create_tenant(name, owner_user_id).await {
            Ok(tenant) => CreateTenantPayload {
                tenant: Some(tenant),
                errors: vec![],
            },
            Err(e) => CreateTenantPayload {
                tenant: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_tenant(
        &self,
        id: &str,
        name: Option<&str>,
        status: Option<&str>,
    ) -> UpdateTenantPayload {
        match self.client.update_tenant(id, name, status).await {
            Ok(tenant) => UpdateTenantPayload {
                tenant: Some(tenant),
                errors: vec![],
            },
            Err(e) => UpdateTenantPayload {
                tenant: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }
}
