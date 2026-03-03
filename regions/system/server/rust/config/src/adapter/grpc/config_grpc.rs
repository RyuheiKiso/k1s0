use std::sync::Arc;

use crate::domain::entity::config_entry::ConfigEntry;
use crate::proto::k1s0::system::common::v1::{
    PaginationResult as ProtoPaginationResult, Timestamp as ProtoTimestamp,
};
use crate::proto::k1s0::system::config::v1 as pb;
use crate::usecase::delete_config::{DeleteConfigError, DeleteConfigUseCase};
use crate::usecase::get_config::{GetConfigError, GetConfigUseCase};
use crate::usecase::get_config_schema::{GetConfigSchemaError, GetConfigSchemaUseCase};
use crate::usecase::get_service_config::{GetServiceConfigError, GetServiceConfigUseCase};
use crate::usecase::list_configs::{ListConfigsError, ListConfigsParams, ListConfigsUseCase};
use crate::usecase::upsert_config_schema::{UpsertConfigSchemaInput, UpsertConfigSchemaUseCase};
use crate::usecase::update_config::{UpdateConfigError, UpdateConfigInput, UpdateConfigUseCase};
use crate::usecase::watch_config::ConfigChangeEvent;

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
    watch_sender: Option<tokio::sync::broadcast::Sender<ConfigChangeEvent>>,
}

impl ConfigGrpcService {
    pub fn new(
        get_config_uc: Arc<GetConfigUseCase>,
        list_configs_uc: Arc<ListConfigsUseCase>,
        get_service_config_uc: Arc<GetServiceConfigUseCase>,
        update_config_uc: Arc<UpdateConfigUseCase>,
        delete_config_uc: Arc<DeleteConfigUseCase>,
    ) -> Self {
        Self {
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
            update_config_uc,
            delete_config_uc,
            get_config_schema_uc: None,
            upsert_config_schema_uc: None,
            watch_sender: None,
        }
    }

    pub fn new_with_watch(
        get_config_uc: Arc<GetConfigUseCase>,
        list_configs_uc: Arc<ListConfigsUseCase>,
        get_service_config_uc: Arc<GetServiceConfigUseCase>,
        update_config_uc: Arc<UpdateConfigUseCase>,
        delete_config_uc: Arc<DeleteConfigUseCase>,
        watch_sender: tokio::sync::broadcast::Sender<ConfigChangeEvent>,
    ) -> Self {
        Self {
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
            update_config_uc,
            delete_config_uc,
            get_config_schema_uc: None,
            upsert_config_schema_uc: None,
            watch_sender: Some(watch_sender),
        }
    }

    pub fn with_schema_usecases(
        mut self,
        get_config_schema_uc: Arc<GetConfigSchemaUseCase>,
        upsert_config_schema_uc: Arc<UpsertConfigSchemaUseCase>,
    ) -> Self {
        self.get_config_schema_uc = Some(get_config_schema_uc);
        self.upsert_config_schema_uc = Some(upsert_config_schema_uc);
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
                    total_count: result.pagination.total_count as i32,
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
                configs: result
                    .entries
                    .into_iter()
                    .map(|e| (format!("{}.{}", e.namespace, e.key), e.value.to_string()))
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
            Err(GetConfigSchemaError::NotFound(service_name)) => {
                Err(GrpcError::NotFound(format!(
                    "config schema not found: {}",
                    service_name
                )))
            }
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
        if schema.service.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "schema.service is required".to_string(),
            ));
        }
        if schema.namespace_prefix.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "schema.namespace_prefix is required".to_string(),
            ));
        }

        let schema_json = pb_schema_to_json(&schema);
        let input = UpsertConfigSchemaInput {
            service_name: schema.service,
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

    pub fn watch_config(
        &self,
        req: WatchConfigRequest,
    ) -> Result<WatchConfigStreamHandler, GrpcError> {
        match &self.watch_sender {
            Some(sender) => {
                let receiver = sender.subscribe();
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

fn domain_schema_to_pb(schema: &crate::domain::entity::config_schema::ConfigSchema) -> pb::ConfigEditorSchema {
    let categories = schema
        .schema_json
        .get("categories")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|cat| pb::ConfigCategorySchema {
                    id: cat
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    label: cat
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    icon: cat
                        .get("icon")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    namespaces: cat
                        .get("namespaces")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(ToString::to_string))
                                .collect()
                        })
                        .unwrap_or_default(),
                    fields: cat
                        .get("fields")
                        .and_then(|v| v.as_array())
                        .map(|farr| {
                            farr.iter()
                                .map(|f| pb::ConfigFieldSchema {
                                    key: f
                                        .get("key")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    label: f
                                        .get("label")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    description: f
                                        .get("description")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    r#type: f
                                        .get("type")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0) as i32,
                                    min: f.get("min").and_then(|v| v.as_i64()).unwrap_or(0),
                                    max: f.get("max").and_then(|v| v.as_i64()).unwrap_or(0),
                                    options: f
                                        .get("options")
                                        .and_then(|v| v.as_array())
                                        .map(|opts| {
                                            opts.iter()
                                                .filter_map(|x| x.as_str().map(ToString::to_string))
                                                .collect()
                                        })
                                        .unwrap_or_default(),
                                    pattern: f
                                        .get("pattern")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    unit: f
                                        .get("unit")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or_default()
                                        .to_string(),
                                    default_value: f
                                        .get("default_value")
                                        .map(|v| serde_json::to_vec(v).unwrap_or_default())
                                        .unwrap_or_default(),
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();

    pb::ConfigEditorSchema {
        service: schema.service_name.clone(),
        namespace_prefix: schema.namespace_prefix.clone(),
        categories,
        updated_at: Some(ProtoTimestamp {
            seconds: schema.updated_at.timestamp(),
            nanos: schema.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

fn pb_schema_to_json(schema: &pb::ConfigEditorSchema) -> serde_json::Value {
    let categories: Vec<serde_json::Value> = schema
        .categories
        .iter()
        .map(|cat| {
            let fields: Vec<serde_json::Value> = cat
                .fields
                .iter()
                .map(|field| {
                    let default_value = serde_json::from_slice::<serde_json::Value>(
                        &field.default_value,
                    )
                    .unwrap_or(serde_json::Value::Null);
                    serde_json::json!({
                        "key": field.key,
                        "label": field.label,
                        "description": field.description,
                        "type": field.r#type,
                        "min": field.min,
                        "max": field.max,
                        "options": field.options,
                        "pattern": field.pattern,
                        "unit": field.unit,
                        "default_value": default_value
                    })
                })
                .collect();

            serde_json::json!({
                "id": cat.id,
                "label": cat.label,
                "icon": cat.icon,
                "namespaces": cat.namespaces,
                "fields": fields
            })
        })
        .collect();
    serde_json::json!({ "categories": categories })
}
