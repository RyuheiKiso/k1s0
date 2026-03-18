use std::sync::Arc;

use crate::adapter::presentation::ConfigEditorSchemaDto;
use crate::domain::entity::config_entry::ConfigEntry;
use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};
use crate::proto::k1s0::system::config::v1 as pb;
use crate::usecase::delete_config::{DeleteConfigError, DeleteConfigUseCase};
use crate::usecase::get_config::{GetConfigError, GetConfigUseCase};
use crate::usecase::get_config_schema::{GetConfigSchemaError, GetConfigSchemaUseCase};
use crate::usecase::get_service_config::{GetServiceConfigError, GetServiceConfigUseCase};
use crate::usecase::list_config_schemas::{ListConfigSchemasError, ListConfigSchemasUseCase};
use crate::usecase::list_configs::{ListConfigsError, ListConfigsParams, ListConfigsUseCase};
use crate::usecase::update_config::{UpdateConfigError, UpdateConfigInput, UpdateConfigUseCase};
use crate::usecase::upsert_config_schema::{UpsertConfigSchemaInput, UpsertConfigSchemaUseCase};
use crate::usecase::watch_config::WatchConfigUseCase;

use super::watch_stream::{WatchConfigRequest, WatchConfigStreamHandler};

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("aborted: {0}")]
    Aborted(String),

    #[error("internal: {0}")]
    Internal(String),
}

pub struct ConfigGrpcService {
    get_config_uc: Arc<GetConfigUseCase>,
    list_configs_uc: Arc<ListConfigsUseCase>,
    get_service_config_uc: Arc<GetServiceConfigUseCase>,
    update_config_uc: Arc<UpdateConfigUseCase>,
    delete_config_uc: Arc<DeleteConfigUseCase>,
    get_config_schema_uc: Option<Arc<GetConfigSchemaUseCase>>,
    upsert_config_schema_uc: Option<Arc<UpsertConfigSchemaUseCase>>,
    list_config_schemas_uc: Option<Arc<ListConfigSchemasUseCase>>,
    watch_uc: Option<Arc<WatchConfigUseCase>>,
}

impl ConfigGrpcService {
    pub fn new_with_watch(
        get_config_uc: Arc<GetConfigUseCase>,
        list_configs_uc: Arc<ListConfigsUseCase>,
        get_service_config_uc: Arc<GetServiceConfigUseCase>,
        update_config_uc: Arc<UpdateConfigUseCase>,
        delete_config_uc: Arc<DeleteConfigUseCase>,
        watch_uc: Arc<WatchConfigUseCase>,
    ) -> Self {
        Self {
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
            update_config_uc,
            delete_config_uc,
            get_config_schema_uc: None,
            upsert_config_schema_uc: None,
            list_config_schemas_uc: None,
            watch_uc: Some(watch_uc),
        }
    }

    pub fn with_schema_usecases(
        mut self,
        get_config_schema_uc: Arc<GetConfigSchemaUseCase>,
        upsert_config_schema_uc: Arc<UpsertConfigSchemaUseCase>,
        list_config_schemas_uc: Arc<ListConfigSchemasUseCase>,
    ) -> Self {
        self.get_config_schema_uc = Some(get_config_schema_uc);
        self.upsert_config_schema_uc = Some(upsert_config_schema_uc);
        self.list_config_schemas_uc = Some(list_config_schemas_uc);
        self
    }

    pub async fn get_config(
        &self,
        req: pb::GetConfigRequest,
    ) -> Result<pb::GetConfigResponse, GrpcError> {
        if req.namespace.is_empty() || req.key.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "namespace and key are required".to_string(),
            ));
        }

        match self.get_config_uc.execute(&req.namespace, &req.key).await {
            Ok(entry) => Ok(pb::GetConfigResponse {
                entry: Some(domain_config_to_pb(&entry)),
            }),
            Err(GetConfigError::NotFound(ns, key)) => Err(GrpcError::NotFound(format!(
                "config not found: {}/{}",
                ns, key
            ))),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn list_configs(
        &self,
        req: pb::ListConfigsRequest,
    ) -> Result<pb::ListConfigsResponse, GrpcError> {
        if req.namespace.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "namespace is required".to_string(),
            ));
        }

        let params = ListConfigsParams {
            page: req.pagination.as_ref().map(|p| p.page).unwrap_or(1),
            page_size: req.pagination.as_ref().map(|p| p.page_size).unwrap_or(20),
            search: if req.search.is_empty() {
                None
            } else {
                Some(req.search)
            },
        };

        match self.list_configs_uc.execute(&req.namespace, &params).await {
            Ok(result) => Ok(pb::ListConfigsResponse {
                entries: result.entries.iter().map(domain_config_to_pb).collect(),
                pagination: Some(ProtoPaginationResult {
                    total_count: result.pagination.total_count as i64,
                    page: result.pagination.page,
                    page_size: result.pagination.page_size,
                    has_next: result.pagination.has_next,
                }),
            }),
            Err(ListConfigsError::Validation(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_service_config(
        &self,
        req: pb::GetServiceConfigRequest,
    ) -> Result<pb::GetServiceConfigResponse, GrpcError> {
        if req.service_name.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "service_name is required".to_string(),
            ));
        }

        match self.get_service_config_uc.execute(&req.service_name).await {
            Ok(result) => Ok(pb::GetServiceConfigResponse {
                entries: result
                    .entries
                    .into_iter()
                    .map(|e| pb::ServiceConfigEntry {
                        namespace: e.namespace,
                        key: e.key,
                        value: e.value.to_string(),
                        version: e.version,
                    })
                    .collect(),
            }),
            Err(GetServiceConfigError::NotFound(name)) => {
                Err(GrpcError::NotFound(format!("service not found: {}", name)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn update_config(
        &self,
        req: pb::UpdateConfigRequest,
    ) -> Result<pb::UpdateConfigResponse, GrpcError> {
        if req.namespace.is_empty() || req.key.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "namespace and key are required".to_string(),
            ));
        }

        let value_json = String::from_utf8(req.value)
            .map_err(|e| GrpcError::InvalidArgument(format!("invalid value bytes: {}", e)))?;
        let value: serde_json::Value = serde_json::from_str(&value_json)
            .map_err(|e| GrpcError::InvalidArgument(format!("invalid value_json: {}", e)))?;

        let input = UpdateConfigInput {
            namespace: req.namespace,
            key: req.key,
            value,
            version: req.version,
            description: if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
            updated_by: req.updated_by,
        };

        match self.update_config_uc.execute(&input).await {
            Ok(entry) => Ok(pb::UpdateConfigResponse {
                entry: Some(domain_config_to_pb(&entry)),
            }),
            Err(UpdateConfigError::NotFound(ns, key)) => Err(GrpcError::NotFound(format!(
                "config not found: {}/{}",
                ns, key
            ))),
            Err(UpdateConfigError::Validation(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(UpdateConfigError::SchemaValidation(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(UpdateConfigError::VersionConflict { expected, current }) => {
                Err(GrpcError::Aborted(format!(
                    "version conflict: expected={}, current={}",
                    expected, current
                )))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn delete_config(
        &self,
        req: pb::DeleteConfigRequest,
    ) -> Result<pb::DeleteConfigResponse, GrpcError> {
        if req.namespace.is_empty() || req.key.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "namespace and key are required".to_string(),
            ));
        }

        let deleted_by = if req.deleted_by.is_empty() {
            "grpc-user"
        } else {
            req.deleted_by.as_str()
        };

        match self
            .delete_config_uc
            .execute(&req.namespace, &req.key, deleted_by)
            .await
        {
            Ok(()) => Ok(pb::DeleteConfigResponse { success: true }),
            Err(DeleteConfigError::NotFound(ns, key)) => Err(GrpcError::NotFound(format!(
                "config not found: {}/{}",
                ns, key
            ))),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_config_schema(
        &self,
        req: pb::GetConfigSchemaRequest,
    ) -> Result<pb::GetConfigSchemaResponse, GrpcError> {
        let uc = self.get_config_schema_uc.as_ref().ok_or_else(|| {
            GrpcError::Internal("get_config_schema usecase is not configured".to_string())
        })?;
        if req.service_name.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "service_name is required".to_string(),
            ));
        }

        match uc.execute(&req.service_name).await {
            Ok(schema) => Ok(pb::GetConfigSchemaResponse {
                schema: Some(domain_schema_to_pb(&schema)),
            }),
            Err(GetConfigSchemaError::NotFound(service_name)) => Err(GrpcError::NotFound(format!(
                "config schema not found: {}",
                service_name
            ))),
            Err(GetConfigSchemaError::Internal(msg)) => Err(GrpcError::Internal(msg)),
        }
    }

    pub async fn upsert_config_schema(
        &self,
        req: pb::UpsertConfigSchemaRequest,
    ) -> Result<pb::UpsertConfigSchemaResponse, GrpcError> {
        let uc = self.upsert_config_schema_uc.as_ref().ok_or_else(|| {
            GrpcError::Internal("upsert_config_schema usecase is not configured".to_string())
        })?;
        let Some(schema) = req.schema else {
            return Err(GrpcError::InvalidArgument("schema is required".to_string()));
        };
        if schema.service_name.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "schema.service_name is required".to_string(),
            ));
        }
        if schema.namespace_prefix.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "schema.namespace_prefix is required".to_string(),
            ));
        }

        let schema_json = pb_schema_to_json(&schema);
        let input = UpsertConfigSchemaInput {
            service_name: schema.service_name,
            namespace_prefix: schema.namespace_prefix,
            schema_json,
            updated_by: if req.updated_by.is_empty() {
                "grpc-user".to_string()
            } else {
                req.updated_by
            },
        };

        let updated = uc
            .execute(&input)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;
        Ok(pb::UpsertConfigSchemaResponse {
            schema: Some(domain_schema_to_pb(&updated)),
        })
    }

    pub async fn list_config_schemas(&self) -> Result<pb::ListConfigSchemasResponse, GrpcError> {
        let uc = self.list_config_schemas_uc.as_ref().ok_or_else(|| {
            GrpcError::Internal("list_config_schemas usecase is not configured".to_string())
        })?;

        match uc.execute().await {
            Ok(schemas) => Ok(pb::ListConfigSchemasResponse {
                schemas: schemas.iter().map(domain_schema_to_pb).collect(),
            }),
            Err(ListConfigSchemasError::Internal(msg)) => Err(GrpcError::Internal(msg)),
        }
    }

    pub fn watch_config(
        &self,
        req: WatchConfigRequest,
    ) -> Result<WatchConfigStreamHandler, GrpcError> {
        match &self.watch_uc {
            Some(watch_uc) => {
                let receiver = watch_uc.subscribe();
                let namespace_filters: Vec<String> = req
                    .namespaces
                    .into_iter()
                    .filter(|ns| !ns.is_empty())
                    .collect();
                Ok(WatchConfigStreamHandler::new(receiver, namespace_filters))
            }
            None => Err(GrpcError::Internal(
                "watch_config is not enabled on this server".to_string(),
            )),
        }
    }
}

fn domain_config_to_pb(e: &ConfigEntry) -> pb::ConfigEntry {
    pb::ConfigEntry {
        id: e.id.to_string(),
        namespace: e.namespace.clone(),
        key: e.key.clone(),
        value: e.value_json.to_string().into_bytes(),
        version: e.version,
        description: e.description.clone(),
        created_by: e.created_by.clone(),
        updated_by: e.updated_by.clone(),
        created_at: Some(ProtoTimestamp {
            seconds: e.created_at.timestamp(),
            nanos: e.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(ProtoTimestamp {
            seconds: e.updated_at.timestamp(),
            nanos: e.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

fn domain_schema_to_pb(
    schema: &crate::domain::entity::config_schema::ConfigSchema,
) -> pb::ConfigEditorSchema {
    ConfigEditorSchemaDto::try_from(schema)
        .map(|dto| dto.to_pb())
        .unwrap_or_else(|_| pb::ConfigEditorSchema {
            service_name: schema.service_name.clone(),
            namespace_prefix: schema.namespace_prefix.clone(),
            categories: vec![],
            updated_at: Some(ProtoTimestamp {
                seconds: schema.updated_at.timestamp(),
                nanos: schema.updated_at.timestamp_subsec_nanos() as i32,
            }),
        })
}

fn pb_schema_to_json(schema: &pb::ConfigEditorSchema) -> serde_json::Value {
    ConfigEditorSchemaDto::from_pb(schema)
        .map(|dto| dto.into_schema_json())
        .unwrap_or_else(|_| serde_json::json!({ "categories": [] }))
}
