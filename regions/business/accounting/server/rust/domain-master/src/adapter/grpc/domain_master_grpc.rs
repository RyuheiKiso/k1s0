use crate::domain::entity::master_category::{
    CreateMasterCategory, MasterCategory as DomainMasterCategory, UpdateMasterCategory,
};
use crate::domain::entity::master_item::{
    CreateMasterItem, MasterItem as DomainMasterItem, UpdateMasterItem,
};
use crate::domain::entity::master_item_version::MasterItemVersion as DomainMasterItemVersion;
use crate::domain::entity::tenant_master_extension::{
    TenantMasterExtension as DomainTenantMasterExtension,
    TenantMergedItem as DomainTenantMergedItem, UpsertTenantMasterExtension,
};
use crate::proto::k1s0::business::accounting::domainmaster::v1::domain_master_service_server::DomainMasterService;
use crate::proto::k1s0::business::accounting::domainmaster::v1::{
    CreateCategoryRequest, CreateCategoryResponse, CreateItemRequest, CreateItemResponse,
    DeleteCategoryRequest, DeleteCategoryResponse, DeleteItemRequest, DeleteItemResponse,
    DeleteTenantExtensionRequest, DeleteTenantExtensionResponse, GetCategoryRequest,
    GetCategoryResponse, GetItemRequest, GetItemResponse, GetTenantExtensionRequest,
    GetTenantExtensionResponse, ListCategoriesRequest, ListCategoriesResponse,
    ListItemVersionsRequest, ListItemVersionsResponse, ListItemsRequest, ListItemsResponse,
    ListTenantItemsRequest, ListTenantItemsResponse, MasterCategory, MasterItem, MasterItemVersion,
    TenantMasterExtension, TenantMergedItem, UpdateCategoryRequest, UpdateCategoryResponse,
    UpdateItemRequest, UpdateItemResponse, UpsertTenantExtensionRequest,
    UpsertTenantExtensionResponse,
};
use crate::proto::k1s0::system::common::v1::{Pagination, PaginationResult};
use crate::usecase;
use chrono::{DateTime, Utc};
use k1s0_auth::actor_from_claims;
use prost_types::{value::Kind, ListValue, Struct, Value};
// カスタム Timestamp 型（k1s0.system.common.v1.Timestamp）を使用
use crate::proto::k1s0::system::common::v1::Timestamp;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct DomainMasterGrpcService {
    pub manage_categories_uc: Arc<usecase::manage_categories::ManageCategoriesUseCase>,
    pub manage_items_uc: Arc<usecase::manage_items::ManageItemsUseCase>,
    pub get_item_versions_uc: Arc<usecase::get_item_versions::GetItemVersionsUseCase>,
    pub manage_tenant_extensions_uc:
        Arc<usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase>,
}

impl DomainMasterGrpcService {
    pub fn new(
        manage_categories_uc: Arc<usecase::manage_categories::ManageCategoriesUseCase>,
        manage_items_uc: Arc<usecase::manage_items::ManageItemsUseCase>,
        get_item_versions_uc: Arc<usecase::get_item_versions::GetItemVersionsUseCase>,
        manage_tenant_extensions_uc: Arc<
            usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase,
        >,
    ) -> Self {
        Self {
            manage_categories_uc,
            manage_items_uc,
            get_item_versions_uc,
            manage_tenant_extensions_uc,
        }
    }
}

#[tonic::async_trait]
impl DomainMasterService for DomainMasterGrpcService {
    async fn list_categories(
        &self,
        request: Request<ListCategoriesRequest>,
    ) -> Result<Response<ListCategoriesResponse>, Status> {
        let req = request.into_inner();
        let categories = self
            .manage_categories_uc
            .list_categories(req.active_only)
            .await
            .map_err(map_anyhow_to_status)?;
        let (categories, pagination) = paginate(categories, req.pagination);

        Ok(Response::new(ListCategoriesResponse {
            categories: categories.into_iter().map(proto_category).collect(),
            pagination: Some(pagination),
        }))
    }

    async fn get_category(
        &self,
        request: Request<GetCategoryRequest>,
    ) -> Result<Response<GetCategoryResponse>, Status> {
        let category_id = parse_uuid(&request.get_ref().category_id, "category_id")?;
        let category = self
            .manage_categories_uc
            .get_category_by_id(category_id)
            .await
            .map_err(map_anyhow_to_status)?
            .ok_or_else(|| Status::not_found(format!("Category '{}' not found", category_id)))?;

        Ok(Response::new(GetCategoryResponse {
            category: Some(proto_category(category)),
        }))
    }

    async fn create_category(
        &self,
        request: Request<CreateCategoryRequest>,
    ) -> Result<Response<CreateCategoryResponse>, Status> {
        let actor = actor_from_claims(request.extensions().get());
        let req = request.into_inner();
        let input = CreateMasterCategory {
            code: req.code,
            display_name: req.display_name,
            description: req.description,
            validation_schema: req.validation_schema.map(struct_to_json),
            is_active: req.is_active,
            sort_order: req.sort_order,
        };
        let category = self
            .manage_categories_uc
            .create_category(&input, &actor)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(CreateCategoryResponse {
            category: Some(proto_category(category)),
        }))
    }

    async fn update_category(
        &self,
        request: Request<UpdateCategoryRequest>,
    ) -> Result<Response<UpdateCategoryResponse>, Status> {
        let actor = actor_from_claims(request.extensions().get());
        let req = request.into_inner();
        let category_id = parse_uuid(&req.category_id, "category_id")?;
        let input = UpdateMasterCategory {
            display_name: req.display_name,
            description: req.description,
            validation_schema: req.validation_schema.map(struct_to_json),
            is_active: req.is_active,
            sort_order: req.sort_order,
        };
        let category = self
            .manage_categories_uc
            .update_category_by_id(category_id, &actor, &input)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(UpdateCategoryResponse {
            category: Some(proto_category(category)),
        }))
    }

    async fn delete_category(
        &self,
        request: Request<DeleteCategoryRequest>,
    ) -> Result<Response<DeleteCategoryResponse>, Status> {
        let actor = actor_from_claims(request.extensions().get());
        let category_id = parse_uuid(&request.get_ref().category_id, "category_id")?;
        self.manage_categories_uc
            .delete_category_by_id(category_id, &actor)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(DeleteCategoryResponse { success: true }))
    }

    async fn list_items(
        &self,
        request: Request<ListItemsRequest>,
    ) -> Result<Response<ListItemsResponse>, Status> {
        let req = request.into_inner();
        let category_id = parse_uuid(&req.category_id, "category_id")?;
        let items = self
            .manage_items_uc
            .list_items_by_category_id(category_id, req.active_only)
            .await
            .map_err(map_anyhow_to_status)?;
        let (items, pagination) = paginate(items, req.pagination);

        Ok(Response::new(ListItemsResponse {
            items: items.into_iter().map(proto_item).collect(),
            pagination: Some(pagination),
        }))
    }

    async fn get_item(
        &self,
        request: Request<GetItemRequest>,
    ) -> Result<Response<GetItemResponse>, Status> {
        let item_id = parse_uuid(&request.get_ref().item_id, "item_id")?;
        let item = self
            .manage_items_uc
            .get_item_by_id(item_id)
            .await
            .map_err(map_anyhow_to_status)?
            .ok_or_else(|| Status::not_found(format!("Item '{}' not found", item_id)))?;

        Ok(Response::new(GetItemResponse {
            item: Some(proto_item(item)),
        }))
    }

    async fn create_item(
        &self,
        request: Request<CreateItemRequest>,
    ) -> Result<Response<CreateItemResponse>, Status> {
        let actor = actor_from_claims(request.extensions().get());
        let req = request.into_inner();
        let category_id = parse_uuid(&req.category_id, "category_id")?;
        let input = CreateMasterItem {
            code: req.code,
            display_name: req.display_name,
            description: req.description,
            attributes: req.attributes.map(struct_to_json),
            parent_item_id: req
                .parent_item_id
                .as_deref()
                .map(|value| parse_uuid(value, "parent_item_id"))
                .transpose()?,
            effective_from: req.effective_from.and_then(timestamp_to_datetime),
            effective_until: req.effective_until.and_then(timestamp_to_datetime),
            is_active: req.is_active,
            sort_order: req.sort_order,
        };
        let item = self
            .manage_items_uc
            .create_item_by_category_id(category_id, &input, &actor)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(CreateItemResponse {
            item: Some(proto_item(item)),
        }))
    }

    async fn update_item(
        &self,
        request: Request<UpdateItemRequest>,
    ) -> Result<Response<UpdateItemResponse>, Status> {
        let actor = actor_from_claims(request.extensions().get());
        let req = request.into_inner();
        let item_id = parse_uuid(&req.item_id, "item_id")?;
        let input = UpdateMasterItem {
            display_name: req.display_name,
            description: req.description,
            attributes: req.attributes.map(struct_to_json),
            parent_item_id: req
                .parent_item_id
                .as_deref()
                .map(|value| parse_uuid(value, "parent_item_id"))
                .transpose()?,
            effective_from: req.effective_from.and_then(timestamp_to_datetime),
            effective_until: req.effective_until.and_then(timestamp_to_datetime),
            is_active: req.is_active,
            sort_order: req.sort_order,
        };
        let item = self
            .manage_items_uc
            .update_item_by_id(item_id, &input, &actor)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(UpdateItemResponse {
            item: Some(proto_item(item)),
        }))
    }

    async fn delete_item(
        &self,
        request: Request<DeleteItemRequest>,
    ) -> Result<Response<DeleteItemResponse>, Status> {
        let actor = actor_from_claims(request.extensions().get());
        let item_id = parse_uuid(&request.get_ref().item_id, "item_id")?;
        self.manage_items_uc
            .delete_item_by_id(item_id, &actor)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(DeleteItemResponse { success: true }))
    }

    async fn list_item_versions(
        &self,
        request: Request<ListItemVersionsRequest>,
    ) -> Result<Response<ListItemVersionsResponse>, Status> {
        let item_id = parse_uuid(&request.get_ref().item_id, "item_id")?;
        let versions = self
            .get_item_versions_uc
            .list_versions_by_item_id(item_id)
            .await
            .map_err(map_anyhow_to_status)?;
        let (versions, pagination) = paginate(versions, request.into_inner().pagination);

        Ok(Response::new(ListItemVersionsResponse {
            versions: versions.into_iter().map(proto_item_version).collect(),
            pagination: Some(pagination),
        }))
    }

    async fn get_tenant_extension(
        &self,
        request: Request<GetTenantExtensionRequest>,
    ) -> Result<Response<GetTenantExtensionResponse>, Status> {
        let req = request.into_inner();
        let item_id = parse_uuid(&req.item_id, "item_id")?;
        let extension = self
            .manage_tenant_extensions_uc
            .get_extension(&req.tenant_id, item_id)
            .await
            .map_err(map_anyhow_to_status)?
            .ok_or_else(|| {
                Status::not_found(format!(
                    "Tenant extension not found for tenant '{}' and item '{}'",
                    req.tenant_id, item_id
                ))
            })?;

        Ok(Response::new(GetTenantExtensionResponse {
            extension: Some(proto_tenant_extension(extension)),
        }))
    }

    async fn upsert_tenant_extension(
        &self,
        request: Request<UpsertTenantExtensionRequest>,
    ) -> Result<Response<UpsertTenantExtensionResponse>, Status> {
        let actor = actor_from_claims(request.extensions().get());
        let req = request.into_inner();
        let item_id = parse_uuid(&req.item_id, "item_id")?;
        let input = UpsertTenantMasterExtension {
            display_name_override: req.display_name_override,
            attributes_override: req.attributes_override.map(struct_to_json),
            is_enabled: req.is_enabled,
        };
        let extension = self
            .manage_tenant_extensions_uc
            .upsert_extension(&req.tenant_id, item_id, &input, &actor)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(UpsertTenantExtensionResponse {
            extension: Some(proto_tenant_extension(extension)),
        }))
    }

    async fn delete_tenant_extension(
        &self,
        request: Request<DeleteTenantExtensionRequest>,
    ) -> Result<Response<DeleteTenantExtensionResponse>, Status> {
        let actor = actor_from_claims(request.extensions().get());
        let req = request.into_inner();
        let item_id = parse_uuid(&req.item_id, "item_id")?;
        self.manage_tenant_extensions_uc
            .delete_extension(&req.tenant_id, item_id, &actor)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(DeleteTenantExtensionResponse {
            success: true,
        }))
    }

    async fn list_tenant_items(
        &self,
        request: Request<ListTenantItemsRequest>,
    ) -> Result<Response<ListTenantItemsResponse>, Status> {
        let req = request.into_inner();
        let category_id = parse_uuid(&req.category_id, "category_id")?;
        let items = self
            .manage_tenant_extensions_uc
            .list_tenant_items_by_category_id(&req.tenant_id, category_id, req.active_only)
            .await
            .map_err(map_anyhow_to_status)?;
        let (items, pagination) = paginate(items, req.pagination);

        Ok(Response::new(ListTenantItemsResponse {
            items: items.into_iter().map(proto_tenant_merged_item).collect(),
            pagination: Some(pagination),
        }))
    }
}

#[allow(clippy::result_large_err)]
fn parse_uuid(raw: &str, field_name: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(raw)
        .map_err(|_| Status::invalid_argument(format!("invalid {}: '{}'", field_name, raw)))
}

fn pagination_result(total_count: usize, page: i32, page_size: i32) -> PaginationResult {
    PaginationResult {
        total_count: total_count as i64,
        page,
        page_size,
        has_next: (page as usize * page_size as usize) < total_count,
    }
}

fn paginate<T>(items: Vec<T>, pagination: Option<Pagination>) -> (Vec<T>, PaginationResult) {
    let total_count = items.len();
    let page = pagination.as_ref().map(|p| p.page).unwrap_or(1).max(1);
    let page_size = pagination
        .as_ref()
        .map(|p| p.page_size)
        .unwrap_or(total_count.max(1) as i32)
        .clamp(1, 100);
    let start = ((page - 1) as usize).saturating_mul(page_size as usize);
    let page_items = items
        .into_iter()
        .skip(start)
        .take(page_size as usize)
        .collect();

    (page_items, pagination_result(total_count, page, page_size))
}

fn proto_category(category: DomainMasterCategory) -> MasterCategory {
    MasterCategory {
        id: category.id.to_string(),
        code: category.code,
        display_name: category.display_name,
        description: category.description.unwrap_or_default(),
        validation_schema: category.validation_schema.map(json_to_struct),
        is_active: category.is_active,
        sort_order: category.sort_order,
        created_by: category.created_by,
        created_at: Some(datetime_to_timestamp(category.created_at)),
        updated_at: Some(datetime_to_timestamp(category.updated_at)),
    }
}

fn proto_item(item: DomainMasterItem) -> MasterItem {
    MasterItem {
        id: item.id.to_string(),
        category_id: item.category_id.to_string(),
        code: item.code,
        display_name: item.display_name,
        description: item.description.unwrap_or_default(),
        attributes: item.attributes.map(json_to_struct),
        parent_item_id: item.parent_item_id.map(|value| value.to_string()),
        effective_from: item.effective_from.map(datetime_to_timestamp),
        effective_until: item.effective_until.map(datetime_to_timestamp),
        is_active: item.is_active,
        sort_order: item.sort_order,
        created_by: item.created_by,
        created_at: Some(datetime_to_timestamp(item.created_at)),
        updated_at: Some(datetime_to_timestamp(item.updated_at)),
    }
}

fn proto_item_version(version: DomainMasterItemVersion) -> MasterItemVersion {
    MasterItemVersion {
        id: version.id.to_string(),
        item_id: version.item_id.to_string(),
        version_number: version.version_number,
        before_data: version.before_data.map(json_to_struct),
        after_data: version.after_data.map(json_to_struct),
        changed_by: version.changed_by,
        change_reason: version.change_reason.unwrap_or_default(),
        created_at: Some(datetime_to_timestamp(version.created_at)),
    }
}

fn proto_tenant_extension(extension: DomainTenantMasterExtension) -> TenantMasterExtension {
    TenantMasterExtension {
        id: extension.id.to_string(),
        tenant_id: extension.tenant_id,
        item_id: extension.item_id.to_string(),
        display_name_override: extension.display_name_override,
        attributes_override: extension.attributes_override.map(json_to_struct),
        is_enabled: extension.is_enabled,
        created_at: Some(datetime_to_timestamp(extension.created_at)),
        updated_at: Some(datetime_to_timestamp(extension.updated_at)),
    }
}

fn proto_tenant_merged_item(item: DomainTenantMergedItem) -> TenantMergedItem {
    TenantMergedItem {
        base_item: Some(proto_item(item.base_item)),
        extension: item.extension.map(proto_tenant_extension),
        effective_display_name: item.effective_display_name,
        effective_attributes: item.effective_attributes.map(json_to_struct),
    }
}

fn datetime_to_timestamp(value: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: value.timestamp(),
        nanos: value.timestamp_subsec_nanos() as i32,
    }
}

fn timestamp_to_datetime(value: Timestamp) -> Option<DateTime<Utc>> {
    DateTime::<Utc>::from_timestamp(value.seconds, value.nanos as u32)
}

fn json_to_struct(value: serde_json::Value) -> Struct {
    match json_to_prost_value(value).kind {
        Some(Kind::StructValue(value)) => value,
        _ => Struct::default(),
    }
}

fn struct_to_json(value: Struct) -> serde_json::Value {
    prost_value_to_json(Value {
        kind: Some(Kind::StructValue(value)),
    })
}

fn json_to_prost_value(value: serde_json::Value) -> Value {
    let kind = match value {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(value) => Kind::BoolValue(value),
        serde_json::Value::Number(value) => Kind::NumberValue(value.as_f64().unwrap_or_default()),
        serde_json::Value::String(value) => Kind::StringValue(value),
        serde_json::Value::Array(values) => Kind::ListValue(ListValue {
            values: values.into_iter().map(json_to_prost_value).collect(),
        }),
        serde_json::Value::Object(values) => Kind::StructValue(Struct {
            fields: values
                .into_iter()
                .map(|(key, value)| (key, json_to_prost_value(value)))
                .collect(),
        }),
    };
    Value { kind: Some(kind) }
}

fn prost_value_to_json(value: Value) -> serde_json::Value {
    match value.kind {
        None | Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::BoolValue(value)) => serde_json::Value::Bool(value),
        Some(Kind::NumberValue(value)) => serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Some(Kind::StringValue(value)) => serde_json::Value::String(value),
        Some(Kind::StructValue(value)) => serde_json::Value::Object(
            value
                .fields
                .into_iter()
                .map(|(key, value)| (key, prost_value_to_json(value)))
                .collect(),
        ),
        Some(Kind::ListValue(value)) => {
            serde_json::Value::Array(value.values.into_iter().map(prost_value_to_json).collect())
        }
    }
}

fn map_anyhow_to_status(err: anyhow::Error) -> Status {
    let msg = err.to_string();
    let lower = msg.to_ascii_lowercase();

    if lower.contains("not found") {
        return Status::not_found(msg);
    }
    if lower.contains("duplicate code") || lower.contains("already exists") {
        return Status::already_exists(msg);
    }
    if lower.contains("validation error") || lower.contains("invalid ") {
        return Status::invalid_argument(msg);
    }

    Status::internal(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn paginate_respects_requested_page_and_size() {
        let items = vec![1, 2, 3, 4, 5];
        let (page_items, pagination) = paginate(
            items,
            Some(Pagination {
                page: 2,
                page_size: 2,
            }),
        );

        assert_eq!(page_items, vec![3, 4]);
        assert_eq!(pagination.total_count, 5);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.page_size, 2);
        assert!(pagination.has_next);
    }

    #[test]
    fn paginate_clamps_invalid_values() {
        let items = vec![1, 2, 3];
        let (page_items, pagination) = paginate(
            items,
            Some(Pagination {
                page: 0,
                page_size: 0,
            }),
        );

        assert_eq!(page_items, vec![1]);
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.page_size, 1);
    }

    #[test]
    fn struct_json_roundtrip_preserves_values() {
        let original = json!({
            "name": "category",
            "active": true,
            "sort_order": 3,
            "tags": ["a", "b"]
        });

        let prost_struct = json_to_struct(original.clone());
        let restored = struct_to_json(prost_struct);

        assert_eq!(restored["name"], original["name"]);
        assert_eq!(restored["active"], original["active"]);
        assert_eq!(restored["tags"], original["tags"]);
        assert_eq!(restored["sort_order"].as_f64(), Some(3.0));
    }
}
