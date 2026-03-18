use crate::domain::entity::inventory_item::InventoryFilter;
use crate::domain::entity::inventory_item::InventoryItem as DomainInventoryItem;
use crate::proto::k1s0::service::inventory::v1::inventory_service_server::InventoryService;
use crate::proto::k1s0::service::inventory::v1::{
    GetInventoryRequest, GetInventoryResponse, InventoryItem, ListInventoryRequest,
    ListInventoryResponse, ReleaseStockRequest, ReleaseStockResponse, ReserveStockRequest,
    ReserveStockResponse, UpdateStockRequest, UpdateStockResponse,
};
use crate::proto::k1s0::system::common::v1::PaginationResult;
use crate::usecase;
use chrono::{DateTime, Utc};
use prost_types::Timestamp;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct InventoryGrpcService {
    pub reserve_stock_uc: Arc<usecase::reserve_stock::ReserveStockUseCase>,
    pub release_stock_uc: Arc<usecase::release_stock::ReleaseStockUseCase>,
    pub get_inventory_uc: Arc<usecase::get_inventory::GetInventoryUseCase>,
    pub list_inventory_uc: Arc<usecase::list_inventory::ListInventoryUseCase>,
    pub update_stock_uc: Arc<usecase::update_stock::UpdateStockUseCase>,
}

impl InventoryGrpcService {
    pub fn new(
        reserve_stock_uc: Arc<usecase::reserve_stock::ReserveStockUseCase>,
        release_stock_uc: Arc<usecase::release_stock::ReleaseStockUseCase>,
        get_inventory_uc: Arc<usecase::get_inventory::GetInventoryUseCase>,
        list_inventory_uc: Arc<usecase::list_inventory::ListInventoryUseCase>,
        update_stock_uc: Arc<usecase::update_stock::UpdateStockUseCase>,
    ) -> Self {
        Self {
            reserve_stock_uc,
            release_stock_uc,
            get_inventory_uc,
            list_inventory_uc,
            update_stock_uc,
        }
    }
}

#[tonic::async_trait]
impl InventoryService for InventoryGrpcService {
    async fn reserve_stock(
        &self,
        request: Request<ReserveStockRequest>,
    ) -> Result<Response<ReserveStockResponse>, Status> {
        let req = request.into_inner();
        let item = self
            .reserve_stock_uc
            .execute(
                &req.product_id,
                &req.warehouse_id,
                req.quantity,
                &req.order_id,
            )
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(ReserveStockResponse {
            item: Some(proto_inventory_item(item)),
        }))
    }

    async fn release_stock(
        &self,
        request: Request<ReleaseStockRequest>,
    ) -> Result<Response<ReleaseStockResponse>, Status> {
        let req = request.into_inner();
        let item = self
            .release_stock_uc
            .execute(
                &req.product_id,
                &req.warehouse_id,
                req.quantity,
                &req.order_id,
                &req.reason,
            )
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(ReleaseStockResponse {
            item: Some(proto_inventory_item(item)),
        }))
    }

    async fn get_inventory(
        &self,
        request: Request<GetInventoryRequest>,
    ) -> Result<Response<GetInventoryResponse>, Status> {
        let inventory_id = parse_uuid(&request.get_ref().inventory_id, "inventory_id")?;
        let item = self
            .get_inventory_uc
            .execute(inventory_id)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(GetInventoryResponse {
            item: Some(proto_inventory_item(item)),
        }))
    }

    async fn list_inventory(
        &self,
        request: Request<ListInventoryRequest>,
    ) -> Result<Response<ListInventoryResponse>, Status> {
        let req = request.into_inner();
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1).max(1);
        let page_size = req
            .pagination
            .as_ref()
            .map(|p| p.page_size)
            .unwrap_or(20)
            .clamp(1, 100);
        let offset = ((page - 1) as i64) * (page_size as i64);

        let filter = InventoryFilter {
            product_id: req.product_id,
            warehouse_id: req.warehouse_id,
            limit: Some(page_size as i64),
            offset: Some(offset),
        };
        let (items, total_count) = self
            .list_inventory_uc
            .execute(&filter)
            .await
            .map_err(map_anyhow_to_status)?;

        let has_next = (page as i64 * page_size as i64) < total_count;
        Ok(Response::new(ListInventoryResponse {
            items: items.into_iter().map(proto_inventory_item).collect(),
            total_count,
            pagination: Some(PaginationResult {
                total_count: total_count as i64,
                page,
                page_size,
                has_next,
            }),
        }))
    }

    async fn update_stock(
        &self,
        request: Request<UpdateStockRequest>,
    ) -> Result<Response<UpdateStockResponse>, Status> {
        let req = request.into_inner();
        let inventory_id = parse_uuid(&req.inventory_id, "inventory_id")?;
        let item = self
            .update_stock_uc
            .execute(inventory_id, req.qty_available, req.expected_version)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(UpdateStockResponse {
            item: Some(proto_inventory_item(item)),
        }))
    }
}

#[allow(clippy::result_large_err)]
fn parse_uuid(raw: &str, field_name: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(raw)
        .map_err(|_| Status::invalid_argument(format!("invalid {}: '{}'", field_name, raw)))
}

fn proto_inventory_item(item: DomainInventoryItem) -> InventoryItem {
    InventoryItem {
        id: item.id.to_string(),
        product_id: item.product_id,
        warehouse_id: item.warehouse_id,
        qty_available: item.qty_available,
        qty_reserved: item.qty_reserved,
        version: item.version,
        created_at: Some(datetime_to_timestamp(item.created_at)),
        updated_at: Some(datetime_to_timestamp(item.updated_at)),
    }
}

fn datetime_to_timestamp(value: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: value.timestamp(),
        nanos: value.timestamp_subsec_nanos() as i32,
    }
}

fn map_anyhow_to_status(err: anyhow::Error) -> Status {
    let msg = err.to_string();
    let lower = msg.to_ascii_lowercase();

    if lower.contains("not found") {
        return Status::not_found(msg);
    }
    if lower.contains("insufficient stock") || lower.contains("insufficient reserved") {
        return Status::failed_precondition(msg);
    }
    if lower.contains("validation") {
        return Status::invalid_argument(msg);
    }
    if lower.contains("version conflict") {
        return Status::aborted(msg);
    }

    Status::internal(msg)
}
