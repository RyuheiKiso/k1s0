use crate::domain::entity::inventory_item::InventoryFilter;
use crate::domain::entity::inventory_item::InventoryItem as DomainInventoryItem;
use crate::proto::k1s0::service::inventory::v1::inventory_service_server::InventoryService;
use k1s0_auth::Claims;
use crate::proto::k1s0::service::inventory::v1::{
    GetInventoryRequest, GetInventoryResponse, InventoryItem, ListInventoryRequest,
    ListInventoryResponse, ReleaseStockRequest, ReleaseStockResponse, ReserveStockRequest,
    ReserveStockResponse, UpdateStockRequest, UpdateStockResponse,
};
use crate::proto::k1s0::system::common::v1::PaginationResult;
use crate::usecase;
use chrono::{DateTime, Utc};
// カスタム Timestamp 型（k1s0.system.common.v1.Timestamp）を使用
use crate::proto::k1s0::system::common::v1::Timestamp;
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
        // Claims が存在しない（未認証）場合は Unauthenticated を返す（P0-2 対応）。
        request
            .extensions()
            .get::<Claims>()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
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
        // Claims が存在しない（未認証）場合は Unauthenticated を返す（P0-2 対応）。
        request
            .extensions()
            .get::<Claims>()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
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
        // read 操作も認証必須（gRPC と REST の認証強度を統一する）。
        request
            .extensions()
            .get::<Claims>()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
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
        // read 操作も認証必須（gRPC と REST の認証強度を統一する）。
        request
            .extensions()
            .get::<Claims>()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
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
            pagination: Some(PaginationResult {
                total_count,
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
        // Claims が存在しない（未認証）場合は Unauthenticated を返す（P0-2 対応）。
        request
            .extensions()
            .get::<Claims>()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
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

/// anyhow::Error をドメインエラー型で型ベースに tonic::Status へ変換する。
/// ダウンキャストに失敗した場合は internal エラーとする。
fn map_anyhow_to_status(err: anyhow::Error) -> Status {
    use crate::domain::error::InventoryError;

    match err.downcast::<InventoryError>() {
        Ok(domain_err) => {
            let msg = domain_err.to_string();
            match domain_err {
                InventoryError::NotFound(_) => Status::not_found(msg),
                InventoryError::InsufficientStock { .. } => Status::failed_precondition(msg),
                InventoryError::InsufficientReserved { .. } => Status::failed_precondition(msg),
                InventoryError::ValidationFailed(_) => Status::invalid_argument(msg),
                InventoryError::VersionConflict(_) => Status::aborted(msg),
                InventoryError::Internal(_) => Status::internal(msg),
            }
        }
        Err(err) => Status::internal(err.to_string()),
    }
}
