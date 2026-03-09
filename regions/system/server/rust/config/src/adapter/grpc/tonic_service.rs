use std::collections::HashMap;
use std::sync::Arc;

use tonic::{Request, Response, Status};

use super::config_grpc::{ConfigGrpcService, GrpcError};
use super::watch_stream::WatchConfigRequest;
use crate::proto::k1s0::system::config::v1 as pb;

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Aborted(msg) => Status::aborted(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

pub struct ConfigServiceTonic {
    inner: Arc<ConfigGrpcService>,
}

impl ConfigServiceTonic {
    pub fn new(inner: Arc<ConfigGrpcService>) -> Self {
        Self { inner }
    }
}

#[tonic::async_trait]
impl pb::config_service_server::ConfigService for ConfigServiceTonic {
    async fn get_config(
        &self,
        request: Request<pb::GetConfigRequest>,
    ) -> Result<Response<pb::GetConfigResponse>, Status> {
        let resp = self
            .inner
            .get_config(request.into_inner())
            .await
            .map_err(Status::from)?;
        Ok(Response::new(resp))
    }

    async fn list_configs(
        &self,
        request: Request<pb::ListConfigsRequest>,
    ) -> Result<Response<pb::ListConfigsResponse>, Status> {
        let resp = self
            .inner
            .list_configs(request.into_inner())
            .await
            .map_err(Status::from)?;
        Ok(Response::new(resp))
    }

    async fn update_config(
        &self,
        request: Request<pb::UpdateConfigRequest>,
    ) -> Result<Response<pb::UpdateConfigResponse>, Status> {
        let resp = self
            .inner
            .update_config(request.into_inner())
            .await
            .map_err(Status::from)?;
        Ok(Response::new(resp))
    }

    async fn delete_config(
        &self,
        request: Request<pb::DeleteConfigRequest>,
    ) -> Result<Response<pb::DeleteConfigResponse>, Status> {
        let resp = self
            .inner
            .delete_config(request.into_inner())
            .await
            .map_err(Status::from)?;
        Ok(Response::new(resp))
    }

    async fn get_service_config(
        &self,
        request: Request<pb::GetServiceConfigRequest>,
    ) -> Result<Response<pb::GetServiceConfigResponse>, Status> {
        let resp = self
            .inner
            .get_service_config(request.into_inner())
            .await
            .map_err(Status::from)?;
        Ok(Response::new(resp))
    }

    async fn get_config_schema(
        &self,
        request: Request<pb::GetConfigSchemaRequest>,
    ) -> Result<Response<pb::GetConfigSchemaResponse>, Status> {
        let resp = self
            .inner
            .get_config_schema(request.into_inner())
            .await
            .map_err(Status::from)?;
        Ok(Response::new(resp))
    }

    async fn upsert_config_schema(
        &self,
        request: Request<pb::UpsertConfigSchemaRequest>,
    ) -> Result<Response<pb::UpsertConfigSchemaResponse>, Status> {
        let resp = self
            .inner
            .upsert_config_schema(request.into_inner())
            .await
            .map_err(Status::from)?;
        Ok(Response::new(resp))
    }

    async fn list_config_schemas(
        &self,
        _request: Request<pb::ListConfigSchemasRequest>,
    ) -> Result<Response<pb::ListConfigSchemasResponse>, Status> {
        let resp = self
            .inner
            .list_config_schemas()
            .await
            .map_err(Status::from)?;
        Ok(Response::new(resp))
    }

    type WatchConfigStream =
        tokio_stream::wrappers::ReceiverStream<Result<pb::WatchConfigResponse, Status>>;

    async fn watch_config(
        &self,
        request: Request<pb::WatchConfigRequest>,
    ) -> Result<Response<Self::WatchConfigStream>, Status> {
        let req = request.into_inner();
        let mut handler = self
            .inner
            .watch_config(WatchConfigRequest {
                namespaces: req.namespaces,
            })
            .map_err(Status::from)?;

        let (tx, rx) = tokio::sync::mpsc::channel(128);
        tokio::spawn(async move {
            let mut previous_values: HashMap<(String, String), (Vec<u8>, i32)> = HashMap::new();
            while let Some(notif) = handler.next().await {
                let map_key = (notif.namespace.clone(), notif.key.clone());
                let new_value = notif.value_json.into_bytes();
                let previous = previous_values.get(&map_key).cloned();
                let old_value = previous
                    .as_ref()
                    .map(|(value, _)| value.clone())
                    .unwrap_or_default();
                let old_version = previous.as_ref().map(|(_, version)| *version).unwrap_or(0);
                let new_is_deleted = serde_json::from_slice::<serde_json::Value>(&new_value)
                    .map(|v| v.is_null())
                    .unwrap_or(false);
                let change_type = if new_is_deleted {
                    "DELETED"
                } else if previous.is_none() && notif.version <= 1 {
                    "CREATED"
                } else {
                    "UPDATED"
                };

                if new_is_deleted {
                    previous_values.remove(&map_key);
                } else {
                    previous_values.insert(map_key, (new_value.clone(), notif.version));
                }

                let resp = pb::WatchConfigResponse {
                    namespace: notif.namespace,
                    key: notif.key,
                    old_value,
                    new_value,
                    old_version,
                    new_version: notif.version,
                    changed_by: notif.updated_by,
                    change_type: change_type.to_string(),
                    changed_at: None,
                };
                if tx.send(Ok(resp)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }
}
