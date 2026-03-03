use std::sync::Arc;

use crate::domain::entity::config_entry::ConfigEntry;
use crate::usecase::delete_config::{DeleteConfigError, DeleteConfigUseCase};
use crate::usecase::get_config::{GetConfigError, GetConfigUseCase};
use crate::usecase::get_config_schema::{GetConfigSchemaError, GetConfigSchemaUseCase};
use crate::usecase::get_service_config::{GetServiceConfigError, GetServiceConfigUseCase};
use crate::usecase::list_configs::{ListConfigsError, ListConfigsParams, ListConfigsUseCase};
use crate::usecase::upsert_config_schema::{UpsertConfigSchemaInput, UpsertConfigSchemaUseCase};
use crate::usecase::update_config::{UpdateConfigError, UpdateConfigInput, UpdateConfigUseCase};
use crate::usecase::watch_config::ConfigChangeEvent;

use super::watch_stream::{WatchConfigRequest, WatchConfigStreamHandler};

// proto 生成コードが未生成のため、proto 定義に準じた型を手動定義する。
// tonic build 後に生成コードの型に置き換える。

// --- gRPC Request/Response Types ---

#[derive(Debug, Clone)]
pub struct GetConfigRequest {
    pub namespace: String,
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct GetConfigResponse {
    pub entry: Option<PbConfigEntry>,
}

#[derive(Debug, Clone)]
pub struct PbConfigEntry {
    pub id: String,
    pub namespace: String,
    pub key: String,
    pub value_json: String,
    pub version: i32,
    pub description: String,
    pub created_by: String,
    pub updated_by: String,
    pub created_at: Option<PbTimestamp>,
    pub updated_at: Option<PbTimestamp>,
}

#[derive(Debug, Clone)]
pub struct PbTimestamp {
    pub seconds: i64,
    pub nanos: i32,
}

#[derive(Debug, Clone)]
pub struct ListConfigsRequest {
    pub namespace: String,
    pub pagination: Option<PbPagination>,
    pub search: String,
}

#[derive(Debug, Clone)]
pub struct PbPagination {
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone)]
pub struct PbPaginationResult {
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct ListConfigsResponse {
    pub entries: Vec<PbConfigEntry>,
    pub pagination: Option<PbPaginationResult>,
}

#[derive(Debug, Clone)]
pub struct GetServiceConfigRequest {
    pub service_name: String,
}

#[derive(Debug, Clone)]
pub struct PbServiceConfigEntry {
    pub namespace: String,
    pub key: String,
    pub value_json: String,
}

#[derive(Debug, Clone)]
pub struct GetServiceConfigResponse {
    pub service_name: String,
    pub entries: Vec<PbServiceConfigEntry>,
}

#[derive(Debug, Clone)]
pub struct UpdateConfigRequest {
    pub namespace: String,
    pub key: String,
    pub value_json: String,
    pub version: i32,
    pub description: String,
    pub updated_by: String,
}

#[derive(Debug, Clone)]
pub struct UpdateConfigResponse {
    pub entry: Option<PbConfigEntry>,
}

#[derive(Debug, Clone)]
pub struct DeleteConfigRequest {
    pub namespace: String,
    pub key: String,
    pub deleted_by: String,
}

#[derive(Debug, Clone)]
pub struct DeleteConfigResponse {
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct GetConfigSchemaRequest {
    pub service_name: String,
}

#[derive(Debug, Clone)]
pub struct GetConfigSchemaResponse {
    pub schema: Option<PbConfigEditorSchema>,
}

#[derive(Debug, Clone)]
pub struct UpsertConfigSchemaRequest {
    pub schema: Option<PbConfigEditorSchema>,
    pub updated_by: String,
}

#[derive(Debug, Clone)]
pub struct UpsertConfigSchemaResponse {
    pub schema: Option<PbConfigEditorSchema>,
}

#[derive(Debug, Clone)]
pub struct PbConfigEditorSchema {
    pub service: String,
    pub namespace_prefix: String,
    pub categories: Vec<PbConfigCategorySchema>,
    pub updated_at: Option<PbTimestamp>,
}

#[derive(Debug, Clone)]
pub struct PbConfigCategorySchema {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub namespaces: Vec<String>,
    pub fields: Vec<PbConfigFieldSchema>,
}

#[derive(Debug, Clone)]
pub struct PbConfigFieldSchema {
    pub key: String,
    pub label: String,
    pub description: String,
    pub field_type: i32,
    pub min: i64,
    pub max: i64,
    pub options: Vec<String>,
    pub pattern: String,
    pub unit: String,
    pub default_value: Vec<u8>,
}

// --- gRPC Error ---

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

// --- ConfigGrpcService ---

pub struct ConfigGrpcService {
    get_config_uc: Arc<GetConfigUseCase>,
    list_configs_uc: Arc<ListConfigsUseCase>,
    get_service_config_uc: Arc<GetServiceConfigUseCase>,
    update_config_uc: Arc<UpdateConfigUseCase>,
    delete_config_uc: Arc<DeleteConfigUseCase>,
    get_config_schema_uc: Option<Arc<GetConfigSchemaUseCase>>,
    upsert_config_schema_uc: Option<Arc<UpsertConfigSchemaUseCase>>,
    /// broadcast channel sender for config-watch streaming.
    /// None の場合は watch_config() 呼び出し時に Unavailable エラーを返す。
    watch_sender: Option<tokio::sync::broadcast::Sender<ConfigChangeEvent>>,
}

impl ConfigGrpcService {
    /// 既存互換のコンストラクタ（watch 機能なし）。
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

    /// watch 機能付きのコンストラクタ。
    /// `watch_sender` は `WatchConfigUseCase::new()` が返す Sender を渡す。
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

    /// 設定値取得。
    pub async fn get_config(&self, req: GetConfigRequest) -> Result<GetConfigResponse, GrpcError> {
        if req.namespace.is_empty() || req.key.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "namespace and key are required".to_string(),
            ));
        }

        match self.get_config_uc.execute(&req.namespace, &req.key).await {
            Ok(entry) => Ok(GetConfigResponse {
                entry: Some(domain_config_to_pb(&entry)),
            }),
            Err(GetConfigError::NotFound(ns, key)) => Err(GrpcError::NotFound(format!(
                "config not found: {}/{}",
                ns, key
            ))),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// 設定値一覧取得。
    pub async fn list_configs(
        &self,
        req: ListConfigsRequest,
    ) -> Result<ListConfigsResponse, GrpcError> {
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
            Ok(result) => {
                let pb_entries: Vec<PbConfigEntry> =
                    result.entries.iter().map(domain_config_to_pb).collect();

                Ok(ListConfigsResponse {
                    entries: pb_entries,
                    pagination: Some(PbPaginationResult {
                        total_count: result.pagination.total_count,
                        page: result.pagination.page,
                        page_size: result.pagination.page_size,
                        has_next: result.pagination.has_next,
                    }),
                })
            }
            Err(ListConfigsError::Validation(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// サービス設定取得。
    pub async fn get_service_config(
        &self,
        req: GetServiceConfigRequest,
    ) -> Result<GetServiceConfigResponse, GrpcError> {
        if req.service_name.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "service_name is required".to_string(),
            ));
        }

        match self.get_service_config_uc.execute(&req.service_name).await {
            Ok(result) => {
                let pb_entries: Vec<PbServiceConfigEntry> = result
                    .entries
                    .iter()
                    .map(|e| PbServiceConfigEntry {
                        namespace: e.namespace.clone(),
                        key: e.key.clone(),
                        value_json: e.value.to_string(),
                    })
                    .collect();

                Ok(GetServiceConfigResponse {
                    service_name: result.service_name,
                    entries: pb_entries,
                })
            }
            Err(GetServiceConfigError::NotFound(name)) => {
                Err(GrpcError::NotFound(format!("service not found: {}", name)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// 設定値更新。
    pub async fn update_config(
        &self,
        req: UpdateConfigRequest,
    ) -> Result<UpdateConfigResponse, GrpcError> {
        if req.namespace.is_empty() || req.key.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "namespace and key are required".to_string(),
            ));
        }

        let value: serde_json::Value = serde_json::from_str(&req.value_json)
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
            Ok(entry) => Ok(UpdateConfigResponse {
                entry: Some(domain_config_to_pb(&entry)),
            }),
            Err(UpdateConfigError::NotFound(ns, key)) => Err(GrpcError::NotFound(format!(
                "config not found: {}/{}",
                ns, key
            ))),
            Err(UpdateConfigError::Validation(msg)) => Err(GrpcError::InvalidArgument(msg)),
            Err(UpdateConfigError::SchemaValidation(msg)) => {
                Err(GrpcError::InvalidArgument(msg))
            }
            Err(UpdateConfigError::VersionConflict { expected, current }) => {
                Err(GrpcError::Aborted(format!(
                    "version conflict: expected={}, current={}",
                    expected, current
                )))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// 設定値削除。
    pub async fn delete_config(
        &self,
        req: DeleteConfigRequest,
    ) -> Result<DeleteConfigResponse, GrpcError> {
        if req.namespace.is_empty() || req.key.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "namespace and key are required".to_string(),
            ));
        }

        let deleted_by = if req.deleted_by.is_empty() {
            "grpc-user"
        } else {
            &req.deleted_by
        };

        match self
            .delete_config_uc
            .execute(&req.namespace, &req.key, deleted_by)
            .await
        {
            Ok(()) => Ok(DeleteConfigResponse { success: true }),
            Err(DeleteConfigError::NotFound(ns, key)) => Err(GrpcError::NotFound(format!(
                "config not found: {}/{}",
                ns, key
            ))),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_config_schema(
        &self,
        req: GetConfigSchemaRequest,
    ) -> Result<GetConfigSchemaResponse, GrpcError> {
        let uc = self.get_config_schema_uc.as_ref().ok_or_else(|| {
            GrpcError::Internal("get_config_schema usecase is not configured".to_string())
        })?;
        if req.service_name.is_empty() {
            return Err(GrpcError::InvalidArgument(
                "service_name is required".to_string(),
            ));
        }

        match uc.execute(&req.service_name).await {
            Ok(schema) => Ok(GetConfigSchemaResponse {
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
        req: UpsertConfigSchemaRequest,
    ) -> Result<UpsertConfigSchemaResponse, GrpcError> {
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
        Ok(UpsertConfigSchemaResponse {
            schema: Some(domain_schema_to_pb(&updated)),
        })
    }

    /// 設定変更監視ストリームハンドラを生成する。
    ///
    /// watch 機能が有効（watch_sender が Some）の場合は `WatchConfigStreamHandler` を返す。
    /// 無効の場合は `GrpcError::Internal` を返す。
    ///
    /// gRPC ストリーミングが利用可能になった後は、このメソッドで得たハンドラを
    /// `WatchConfigStreamHandler::next()` でポーリングしてレスポンスストリームに送出する。
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

// --- 変換ヘルパー ---

fn domain_config_to_pb(e: &ConfigEntry) -> PbConfigEntry {
    PbConfigEntry {
        id: e.id.to_string(),
        namespace: e.namespace.clone(),
        key: e.key.clone(),
        value_json: e.value_json.to_string(),
        version: e.version,
        description: e.description.clone().unwrap_or_default(),
        created_by: e.created_by.clone(),
        updated_by: e.updated_by.clone(),
        created_at: Some(PbTimestamp {
            seconds: e.created_at.timestamp(),
            nanos: e.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(PbTimestamp {
            seconds: e.updated_at.timestamp(),
            nanos: e.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

fn domain_schema_to_pb(
    schema: &crate::domain::entity::config_schema::ConfigSchema,
) -> PbConfigEditorSchema {
    let categories = schema
        .schema_json
        .get("categories")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .map(|cat| PbConfigCategorySchema {
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
                        .map(|values| {
                            values
                                .iter()
                                .filter_map(|s| s.as_str().map(ToString::to_string))
                                .collect()
                        })
                        .unwrap_or_default(),
                    fields: cat
                        .get("fields")
                        .and_then(|v| v.as_array())
                        .map(|fields| fields.iter().map(json_field_to_pb).collect())
                        .unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();

    PbConfigEditorSchema {
        service: schema.service_name.clone(),
        namespace_prefix: schema.namespace_prefix.clone(),
        categories,
        updated_at: Some(PbTimestamp {
            seconds: schema.updated_at.timestamp(),
            nanos: schema.updated_at.timestamp_subsec_nanos() as i32,
        }),
    }
}

fn json_field_to_pb(field: &serde_json::Value) -> PbConfigFieldSchema {
    let default_value = field
        .get("default_value")
        .map(|v| serde_json::to_vec(v).unwrap_or_default())
        .unwrap_or_default();
    PbConfigFieldSchema {
        key: field
            .get("key")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        label: field
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        description: field
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        field_type: field
            .get("type")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32,
        min: field.get("min").and_then(|v| v.as_i64()).unwrap_or(0),
        max: field.get("max").and_then(|v| v.as_i64()).unwrap_or(0),
        options: field
            .get("options")
            .and_then(|v| v.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|s| s.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_default(),
        pattern: field
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        unit: field
            .get("unit")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        default_value,
    }
}

fn pb_schema_to_json(schema: &PbConfigEditorSchema) -> serde_json::Value {
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
                        "type": field.field_type,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::config_entry::{
        ConfigListResult, Pagination, ServiceConfigEntry, ServiceConfigResult,
    };
    use crate::domain::repository::config_repository::MockConfigRepository;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_test_entry() -> ConfigEntry {
        ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(25),
            version: 3,
            description: Some("DB max connections".to_string()),
            created_by: "admin@example.com".to_string(),
            updated_by: "admin@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_config_service(mock: MockConfigRepository) -> ConfigGrpcService {
        let repo = Arc::new(mock);
        let get_config_uc = Arc::new(GetConfigUseCase::new(repo.clone()));
        let list_configs_uc = Arc::new(ListConfigsUseCase::new(repo.clone()));
        let get_service_config_uc = Arc::new(GetServiceConfigUseCase::new(repo.clone()));
        let update_config_uc = Arc::new(UpdateConfigUseCase::new(repo.clone()));
        let delete_config_uc = Arc::new(DeleteConfigUseCase::new(repo));

        ConfigGrpcService::new(
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
            update_config_uc,
            delete_config_uc,
        )
    }

    #[tokio::test]
    async fn test_get_config_success() {
        let mut mock = MockConfigRepository::new();
        let entry = make_test_entry();
        let expected_id = entry.id;

        mock.expect_find_by_namespace_and_key()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(move |_, _| Ok(Some(entry.clone())));

        let svc = make_config_service(mock);

        let req = GetConfigRequest {
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
        };
        let resp = svc.get_config(req).await.unwrap();
        let pb_entry = resp.entry.unwrap();

        assert_eq!(pb_entry.id, expected_id.to_string());
        assert_eq!(pb_entry.namespace, "system.auth.database");
        assert_eq!(pb_entry.key, "max_connections");
        assert_eq!(pb_entry.value_json, "25");
        assert_eq!(pb_entry.version, 3);
    }

    #[tokio::test]
    async fn test_get_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(None));

        let svc = make_config_service(mock);

        let req = GetConfigRequest {
            namespace: "nonexistent".to_string(),
            key: "missing".to_string(),
        };
        let result = svc.get_config(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("not found")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_config_invalid_argument() {
        let mock = MockConfigRepository::new();
        let svc = make_config_service(mock);

        let req = GetConfigRequest {
            namespace: "".to_string(),
            key: "max_connections".to_string(),
        };
        let result = svc.get_config(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::InvalidArgument(msg) => {
                assert!(msg.contains("namespace and key are required"))
            }
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_list_configs_success() {
        let mut mock = MockConfigRepository::new();
        mock.expect_list_by_namespace()
            .withf(|ns, page, page_size, _| {
                ns == "system.auth.database" && *page == 1 && *page_size == 20
            })
            .returning(|_, page, page_size, _| {
                Ok(ConfigListResult {
                    entries: vec![
                        ConfigEntry {
                            id: Uuid::new_v4(),
                            namespace: "system.auth.database".to_string(),
                            key: "max_connections".to_string(),
                            value_json: serde_json::json!(25),
                            version: 1,
                            description: None,
                            created_by: "admin@example.com".to_string(),
                            updated_by: "admin@example.com".to_string(),
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        },
                        ConfigEntry {
                            id: Uuid::new_v4(),
                            namespace: "system.auth.database".to_string(),
                            key: "ssl_mode".to_string(),
                            value_json: serde_json::json!("require"),
                            version: 1,
                            description: None,
                            created_by: "admin@example.com".to_string(),
                            updated_by: "admin@example.com".to_string(),
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        },
                    ],
                    pagination: Pagination {
                        total_count: 2,
                        page,
                        page_size,
                        has_next: false,
                    },
                })
            });

        let svc = make_config_service(mock);

        let req = ListConfigsRequest {
            namespace: "system.auth.database".to_string(),
            pagination: None,
            search: String::new(),
        };
        let resp = svc.list_configs(req).await.unwrap();

        assert_eq!(resp.entries.len(), 2);
        assert_eq!(resp.entries[0].key, "max_connections");
        assert_eq!(resp.entries[1].key, "ssl_mode");

        let pagination = resp.pagination.unwrap();
        assert_eq!(pagination.total_count, 2);
        assert!(!pagination.has_next);
    }

    #[tokio::test]
    async fn test_list_configs_with_pagination() {
        let mut mock = MockConfigRepository::new();
        mock.expect_list_by_namespace()
            .withf(|_, page, page_size, _| *page == 2 && *page_size == 10)
            .returning(|_, page, page_size, _| {
                Ok(ConfigListResult {
                    entries: vec![ConfigEntry {
                        id: Uuid::new_v4(),
                        namespace: "system.auth.database".to_string(),
                        key: "timeout".to_string(),
                        value_json: serde_json::json!(30),
                        version: 1,
                        description: None,
                        created_by: "admin@example.com".to_string(),
                        updated_by: "admin@example.com".to_string(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    }],
                    pagination: Pagination {
                        total_count: 25,
                        page,
                        page_size,
                        has_next: true,
                    },
                })
            });

        let svc = make_config_service(mock);

        let req = ListConfigsRequest {
            namespace: "system.auth.database".to_string(),
            pagination: Some(PbPagination {
                page: 2,
                page_size: 10,
            }),
            search: String::new(),
        };
        let resp = svc.list_configs(req).await.unwrap();

        assert_eq!(resp.entries.len(), 1);
        let pagination = resp.pagination.unwrap();
        assert_eq!(pagination.total_count, 25);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.page_size, 10);
        assert!(pagination.has_next);
    }

    #[tokio::test]
    async fn test_get_service_config_success() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name()
            .withf(|name| name == "auth-server")
            .returning(|_| {
                Ok(ServiceConfigResult {
                    service_name: "auth-server".to_string(),
                    entries: vec![
                        ServiceConfigEntry {
                            namespace: "system.auth.database".to_string(),
                            key: "max_connections".to_string(),
                            value: serde_json::json!(25),
                        },
                        ServiceConfigEntry {
                            namespace: "system.auth.jwt".to_string(),
                            key: "issuer".to_string(),
                            value: serde_json::json!("https://auth.example.com"),
                        },
                    ],
                })
            });

        let svc = make_config_service(mock);

        let req = GetServiceConfigRequest {
            service_name: "auth-server".to_string(),
        };
        let resp = svc.get_service_config(req).await.unwrap();

        assert_eq!(resp.service_name, "auth-server");
        assert_eq!(resp.entries.len(), 2);
        assert_eq!(resp.entries[0].namespace, "system.auth.database");
        assert_eq!(resp.entries[1].key, "issuer");
    }

    #[tokio::test]
    async fn test_get_service_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_service_name()
            .returning(|_| Err(anyhow::anyhow!("service not found")));

        let svc = make_config_service(mock);

        let req = GetServiceConfigRequest {
            service_name: "nonexistent-service".to_string(),
        };
        let result = svc.get_service_config(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("not found")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_config_success() {
        let mut mock = MockConfigRepository::new();
        let updated_entry = ConfigEntry {
            id: Uuid::new_v4(),
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(50),
            version: 4,
            description: Some("increased".to_string()),
            created_by: "admin@example.com".to_string(),
            updated_by: "operator@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let expected_version = updated_entry.version;

        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(Some(make_test_entry())));
        mock.expect_update()
            .withf(|ns, key, _, ver, _, _| {
                ns == "system.auth.database" && key == "max_connections" && *ver == 3
            })
            .returning(move |_, _, _, _, _, _| Ok(updated_entry.clone()));
        mock.expect_record_change_log().returning(|_| Ok(()));

        let svc = make_config_service(mock);

        let req = UpdateConfigRequest {
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: "50".to_string(),
            version: 3,
            description: "increased".to_string(),
            updated_by: "operator@example.com".to_string(),
        };
        let resp = svc.update_config(req).await.unwrap();
        let pb_entry = resp.entry.unwrap();

        assert_eq!(pb_entry.value_json, "50");
        assert_eq!(pb_entry.version, expected_version);
        assert_eq!(pb_entry.updated_by, "operator@example.com");
    }

    #[tokio::test]
    async fn test_update_config_invalid_argument() {
        let mock = MockConfigRepository::new();
        let svc = make_config_service(mock);

        let req = UpdateConfigRequest {
            namespace: "".to_string(),
            key: "max_connections".to_string(),
            value_json: "50".to_string(),
            version: 3,
            description: String::new(),
            updated_by: "operator@example.com".to_string(),
        };
        let result = svc.update_config(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::InvalidArgument(msg) => {
                assert!(msg.contains("namespace and key are required"))
            }
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_delete_config_success() {
        let mut mock = MockConfigRepository::new();
        let entry = make_test_entry();
        mock.expect_find_by_namespace_and_key()
            .returning(move |_, _| Ok(Some(entry.clone())));
        mock.expect_delete()
            .withf(|ns, key| ns == "system.auth.database" && key == "max_connections")
            .returning(|_, _| Ok(true));
        mock.expect_record_change_log().returning(|_| Ok(()));

        let svc = make_config_service(mock);

        let req = DeleteConfigRequest {
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            deleted_by: "admin@example.com".to_string(),
        };
        let resp = svc.delete_config(req).await.unwrap();

        assert!(resp.success);
    }

    #[tokio::test]
    async fn test_delete_config_not_found() {
        let mut mock = MockConfigRepository::new();
        mock.expect_find_by_namespace_and_key()
            .returning(|_, _| Ok(None));
        mock.expect_delete().returning(|_, _| Ok(false));

        let svc = make_config_service(mock);

        let req = DeleteConfigRequest {
            namespace: "nonexistent".to_string(),
            key: "missing".to_string(),
            deleted_by: "admin@example.com".to_string(),
        };
        let result = svc.delete_config(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("not found")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    // --- watch_config テスト ---

    fn make_config_service_with_watch(
        mock: MockConfigRepository,
    ) -> (
        ConfigGrpcService,
        tokio::sync::broadcast::Sender<ConfigChangeEvent>,
    ) {
        use crate::usecase::watch_config::WatchConfigUseCase;
        let repo = Arc::new(mock);
        let get_config_uc = Arc::new(GetConfigUseCase::new(repo.clone()));
        let list_configs_uc = Arc::new(ListConfigsUseCase::new(repo.clone()));
        let get_service_config_uc = Arc::new(GetServiceConfigUseCase::new(repo.clone()));
        let update_config_uc = Arc::new(UpdateConfigUseCase::new(repo.clone()));
        let delete_config_uc = Arc::new(DeleteConfigUseCase::new(repo));
        let (_watch_uc, tx) = WatchConfigUseCase::new();

        let svc = ConfigGrpcService::new_with_watch(
            get_config_uc,
            list_configs_uc,
            get_service_config_uc,
            update_config_uc,
            delete_config_uc,
            tx.clone(),
        );
        (svc, tx)
    }

    #[tokio::test]
    async fn test_watch_config_not_enabled_returns_error() {
        // watch_sender なしの ConfigGrpcService で watch_config() は Internal エラー
        let mock = MockConfigRepository::new();
        let svc = make_config_service(mock);

        let req = WatchConfigRequest {
            namespaces: vec!["system.auth".to_string()],
        };
        let result = svc.watch_config(req);

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::Internal(msg) => assert!(msg.contains("not enabled")),
            e => unreachable!("unexpected error in test: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_watch_config_returns_handler_when_enabled() {
        let mock = MockConfigRepository::new();
        let (svc, _tx) = make_config_service_with_watch(mock);

        let req = WatchConfigRequest {
            namespaces: vec!["system.auth".to_string()],
        };
        let result = svc.watch_config(req);

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_watch_config_handler_receives_notification() {
        let mock = MockConfigRepository::new();
        let (svc, tx) = make_config_service_with_watch(mock);

        let req = WatchConfigRequest {
            namespaces: vec!["system.auth".to_string()],
        };
        let mut handler = svc.watch_config(req).unwrap();

        // 別タスクからイベントを送信
        let event = ConfigChangeEvent {
            namespace: "system.auth.database".to_string(),
            key: "max_connections".to_string(),
            value_json: serde_json::json!(100),
            updated_by: "admin".to_string(),
            version: 5,
        };
        let _ = tx.send(event);

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.namespace, "system.auth.database");
        assert_eq!(notif.key, "max_connections");
        assert_eq!(notif.version, 5);
    }

    #[tokio::test]
    async fn test_watch_config_empty_namespace_filter_receives_all() {
        let mock = MockConfigRepository::new();
        let (svc, tx) = make_config_service_with_watch(mock);

        let req = WatchConfigRequest {
            namespaces: vec![], // 空 = フィルタなし
        };
        let mut handler = svc.watch_config(req).unwrap();

        let event = ConfigChangeEvent {
            namespace: "business.billing".to_string(),
            key: "invoice_limit".to_string(),
            value_json: serde_json::json!(500),
            updated_by: "billing-svc".to_string(),
            version: 2,
        };
        let _ = tx.send(event);

        let notif = handler.next().await.unwrap();
        assert_eq!(notif.namespace, "business.billing");
        assert_eq!(notif.version, 2);
    }
}
