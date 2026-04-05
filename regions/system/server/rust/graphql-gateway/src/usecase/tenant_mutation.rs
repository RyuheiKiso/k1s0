use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::{CreateTenantPayload, UpdateTenantPayload, UserError};
use crate::infrastructure::grpc::TenantGrpcClient;

pub struct TenantMutationResolver {
    client: Arc<TenantGrpcClient>,
}

impl TenantMutationResolver {
    pub fn new(client: Arc<TenantGrpcClient>) -> Self {
        Self { client }
    }

    /// テナント作成ユースケース。
    /// C-003 監査対応: display_name, owner_id, plan を追加引数として受け取り、gRPC クライアントに渡す。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_tenant(
        &self,
        name: &str,
        display_name: &str,
        owner_id: &str,
        plan: &str,
    ) -> CreateTenantPayload {
        match self.client.create_tenant(name, display_name, owner_id, plan).await {
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

    /// CRIT-007 対応: UpdateTenantInput を proto UpdateTenantRequest に整合させる
    /// display_name と plan を直接渡す（status は suspend/activate 専用ミューテーションで変更する）
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_tenant(
        &self,
        id: &str,
        display_name: Option<&str>,
        plan: Option<&str>,
    ) -> UpdateTenantPayload {
        match self.client.update_tenant(id, display_name, plan).await {
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
