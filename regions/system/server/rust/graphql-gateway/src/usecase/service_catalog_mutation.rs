// service-catalog ミューテーションリゾルバ。
// service-catalog は REST のみ提供するため ServiceCatalogHttpClient を使用する。

use std::collections::HashMap;
use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::{
    DeleteServicePayload, RegisterServicePayload, UpdateServicePayload, UserError,
};
use crate::infrastructure::http::ServiceCatalogHttpClient;

pub struct ServiceCatalogMutationResolver {
    client: Arc<ServiceCatalogHttpClient>,
}

impl ServiceCatalogMutationResolver {
    #[must_use] 
    pub fn new(client: Arc<ServiceCatalogHttpClient>) -> Self {
        Self { client }
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn register_service(
        &self,
        name: &str,
        display_name: &str,
        description: &str,
        tier: &str,
        version: &str,
        base_url: &str,
        grpc_endpoint: Option<&str>,
        health_url: &str,
        metadata: HashMap<String, String>,
    ) -> RegisterServicePayload {
        match self
            .client
            .register_service(
                name,
                display_name,
                description,
                tier,
                version,
                base_url,
                grpc_endpoint,
                health_url,
                metadata,
            )
            .await
        {
            Ok(service) => RegisterServicePayload {
                service: Some(service),
                errors: vec![],
            },
            Err(e) => RegisterServicePayload {
                service: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_service(
        &self,
        service_id: &str,
        display_name: Option<&str>,
        description: Option<&str>,
        version: Option<&str>,
        base_url: Option<&str>,
        grpc_endpoint: Option<&str>,
        health_url: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> UpdateServicePayload {
        match self
            .client
            .update_service(
                service_id,
                display_name,
                description,
                version,
                base_url,
                grpc_endpoint,
                health_url,
                metadata,
            )
            .await
        {
            Ok(service) => UpdateServicePayload {
                service: Some(service),
                errors: vec![],
            },
            Err(e) => UpdateServicePayload {
                service: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn delete_service(&self, service_id: &str) -> DeleteServicePayload {
        match self.client.delete_service(service_id).await {
            Ok(success) => DeleteServicePayload {
                success,
                errors: vec![],
            },
            Err(e) => DeleteServicePayload {
                success: false,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            },
        }
    }
}
