use crate::domain::entity::order::{
    CreateOrder, CreateOrderItem, Order as DomainOrder, OrderFilter, OrderItem as DomainOrderItem,
    OrderStatus,
};
use crate::proto::k1s0::service::order::v1::order_service_server::OrderService;
use crate::proto::k1s0::service::order::v1::{
    CreateOrderRequest, CreateOrderResponse, GetOrderRequest, GetOrderResponse, ListOrdersRequest,
    ListOrdersResponse, Order, OrderItem, UpdateOrderStatusRequest, UpdateOrderStatusResponse,
};
use crate::proto::k1s0::system::common::v1::PaginationResult;
use crate::usecase;
use chrono::{DateTime, Utc};
use k1s0_auth::actor_from_claims;
// カスタム Timestamp 型（k1s0.system.common.v1.Timestamp）を使用
use crate::proto::k1s0::system::common::v1::Timestamp;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct OrderGrpcService {
    pub create_order_uc: Arc<usecase::create_order::CreateOrderUseCase>,
    pub get_order_uc: Arc<usecase::get_order::GetOrderUseCase>,
    pub list_orders_uc: Arc<usecase::list_orders::ListOrdersUseCase>,
    pub update_order_status_uc: Arc<usecase::update_order_status::UpdateOrderStatusUseCase>,
}

impl OrderGrpcService {
    pub fn new(
        create_order_uc: Arc<usecase::create_order::CreateOrderUseCase>,
        get_order_uc: Arc<usecase::get_order::GetOrderUseCase>,
        list_orders_uc: Arc<usecase::list_orders::ListOrdersUseCase>,
        update_order_status_uc: Arc<usecase::update_order_status::UpdateOrderStatusUseCase>,
    ) -> Self {
        Self {
            create_order_uc,
            get_order_uc,
            list_orders_uc,
            update_order_status_uc,
        }
    }
}

#[tonic::async_trait]
impl OrderService for OrderGrpcService {
    async fn create_order(
        &self,
        request: Request<CreateOrderRequest>,
    ) -> Result<Response<CreateOrderResponse>, Status> {
        let actor = actor_from_claims(request.extensions().get());
        let req = request.into_inner();
        let input = CreateOrder {
            customer_id: req.customer_id,
            currency: req.currency,
            notes: req.notes,
            items: req
                .items
                .into_iter()
                .map(|item| CreateOrderItem {
                    product_id: item.product_id,
                    product_name: item.product_name,
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                })
                .collect(),
        };
        let (order, items) = self
            .create_order_uc
            .execute(&input, &actor)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(CreateOrderResponse {
            order: Some(proto_order(order, items)),
        }))
    }

    async fn get_order(
        &self,
        request: Request<GetOrderRequest>,
    ) -> Result<Response<GetOrderResponse>, Status> {
        let order_id = parse_uuid(&request.get_ref().order_id, "order_id")?;
        let (order, items) = self
            .get_order_uc
            .execute(order_id)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(GetOrderResponse {
            order: Some(proto_order(order, items)),
        }))
    }

    async fn list_orders(
        &self,
        request: Request<ListOrdersRequest>,
    ) -> Result<Response<ListOrdersResponse>, Status> {
        let req = request.into_inner();
        let status = req
            .status
            .as_deref()
            .map(|s| s.parse::<OrderStatus>())
            .transpose()
            .map_err(Status::invalid_argument)?;
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1).max(1);
        let page_size = req
            .pagination
            .as_ref()
            .map(|p| p.page_size)
            .unwrap_or(20)
            .clamp(1, 100);
        let offset = ((page - 1) as i64) * (page_size as i64);

        let filter = OrderFilter {
            customer_id: req.customer_id,
            status,
            limit: Some(page_size as i64),
            offset: Some(offset),
        };
        let (orders, total_count) = self
            .list_orders_uc
            .execute(&filter)
            .await
            .map_err(map_anyhow_to_status)?;

        let has_next = (page as i64 * page_size as i64) < total_count;
        Ok(Response::new(ListOrdersResponse {
            orders: orders.into_iter().map(|o| proto_order(o, vec![])).collect(),
            pagination: Some(PaginationResult {
                total_count: total_count as i64,
                page,
                page_size,
                has_next,
            }),
        }))
    }

    async fn update_order_status(
        &self,
        request: Request<UpdateOrderStatusRequest>,
    ) -> Result<Response<UpdateOrderStatusResponse>, Status> {
        let actor = actor_from_claims(request.extensions().get());
        let req = request.into_inner();
        let order_id = parse_uuid(&req.order_id, "order_id")?;
        let new_status: OrderStatus = req
            .status
            .parse()
            .map_err(|e: String| Status::invalid_argument(e))?;

        let order = self
            .update_order_status_uc
            .execute(order_id, &new_status, &actor)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(UpdateOrderStatusResponse {
            order: Some(proto_order(order, vec![])),
        }))
    }
}

#[allow(clippy::result_large_err)]
fn parse_uuid(raw: &str, field_name: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(raw)
        .map_err(|_| Status::invalid_argument(format!("invalid {}: '{}'", field_name, raw)))
}

fn proto_order(order: DomainOrder, items: Vec<DomainOrderItem>) -> Order {
    Order {
        id: order.id.to_string(),
        customer_id: order.customer_id,
        status: order.status.as_str().to_string(),
        total_amount: order.total_amount,
        currency: order.currency,
        notes: order.notes,
        created_by: order.created_by,
        updated_by: order.updated_by,
        version: order.version,
        items: items.into_iter().map(proto_order_item).collect(),
        created_at: Some(datetime_to_timestamp(order.created_at)),
        updated_at: Some(datetime_to_timestamp(order.updated_at)),
    }
}

fn proto_order_item(item: DomainOrderItem) -> OrderItem {
    OrderItem {
        id: item.id.to_string(),
        order_id: item.order_id.to_string(),
        product_id: item.product_id,
        product_name: item.product_name,
        quantity: item.quantity,
        unit_price: item.unit_price,
        subtotal: item.subtotal,
        created_at: Some(datetime_to_timestamp(item.created_at)),
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
    if lower.contains("invalid status transition") {
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
