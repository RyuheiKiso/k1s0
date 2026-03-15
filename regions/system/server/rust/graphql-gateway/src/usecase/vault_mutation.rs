use crate::domain::model::{DeleteSecretPayload, RotateSecretPayload, SetSecretPayload, UserError};
use crate::infrastructure::grpc::VaultGrpcClient;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::instrument;

pub struct VaultMutationResolver {
    client: Arc<VaultGrpcClient>,
}

impl VaultMutationResolver {
    pub fn new(client: Arc<VaultGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self, data), fields(service = "graphql-gateway"))]
    pub async fn set_secret(&self, path: &str, data: HashMap<String, String>) -> SetSecretPayload {
        match self.client.set_secret(path, data).await {
            Ok((p, version, _created_at)) => SetSecretPayload {
                path: Some(p),
                version: Some(version),
                errors: vec![],
            },
            Err(e) => SetSecretPayload {
                path: None,
                version: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self, data), fields(service = "graphql-gateway"))]
    pub async fn rotate_secret(
        &self,
        path: &str,
        data: HashMap<String, String>,
    ) -> RotateSecretPayload {
        match self.client.rotate_secret(path, data).await {
            Ok((p, new_version, rotated)) => RotateSecretPayload {
                path: Some(p),
                new_version: Some(new_version),
                rotated,
                errors: vec![],
            },
            Err(e) => RotateSecretPayload {
                path: None,
                new_version: None,
                rotated: false,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_secret(&self, path: &str, versions: Vec<i64>) -> DeleteSecretPayload {
        match self.client.delete_secret(path, versions).await {
            Ok(success) => DeleteSecretPayload {
                success,
                errors: vec![],
            },
            Err(e) => DeleteSecretPayload {
                success: false,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }
}
