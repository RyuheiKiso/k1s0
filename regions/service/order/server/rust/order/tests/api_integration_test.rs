//! Order サービス API 統合テスト。
//!
//! in-memory リポジトリを使用して handler レイヤーの統合テストを行う。

use axum::body::Body;
use axum::http::{Request, StatusCode};
use k1s0_order_server::adapter::handler::{self, AppState};
use k1s0_order_server::domain::entity::order::{
    CreateOrder, Order, OrderFilter, OrderItem, OrderStatus,
};
use k1s0_order_server::domain::entity::outbox::OutboxEvent;
use k1s0_order_server::domain::repository::order_repository::OrderRepository;
use k1s0_order_server::usecase;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

/// In-memory リポジトリ実装（統合テスト用）。
#[derive(Default)]
struct InMemoryOrderRepository {
    orders: RwLock<Vec<Order>>,
    items: RwLock<Vec<OrderItem>>,
}

#[async_trait::async_trait]
impl OrderRepository for InMemoryOrderRepository {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Order>> {
        let orders = self.orders.read().await;
        Ok(orders.iter().find(|o| o.id == id).cloned())
    }

    async fn find_items_by_order_id(&self, order_id: Uuid) -> anyhow::Result<Vec<OrderItem>> {
        let items = self.items.read().await;
        Ok(items.iter().filter(|i| i.order_id == order_id).cloned().collect())
    }

    async fn find_all(&self, _filter: &OrderFilter) -> anyhow::Result<Vec<Order>> {
        let orders = self.orders.read().await;
        Ok(orders.clone())
    }

    async fn count(&self, _filter: &OrderFilter) -> anyhow::Result<i64> {
        let orders = self.orders.read().await;
        Ok(orders.len() as i64)
    }

    async fn create(
        &self,
        input: &CreateOrder,
        created_by: &str,
    ) -> anyhow::Result<(Order, Vec<OrderItem>)> {
        let now = chrono::Utc::now();
        let order_id = Uuid::new_v4();
        let total: i64 = input.items.iter().map(|i| i.quantity as i64 * i.unit_price).sum();
        let order = Order {
            id: order_id,
            customer_id: input.customer_id.clone(),
            status: OrderStatus::Pending,
            total_amount: total,
            currency: input.currency.clone(),
            notes: input.notes.clone(),
            created_by: created_by.to_string(),
            updated_by: None,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        let order_items: Vec<OrderItem> = input
            .items
            .iter()
            .map(|i| OrderItem {
                id: Uuid::new_v4(),
                order_id,
                product_id: i.product_id.clone(),
                product_name: i.product_name.clone(),
                quantity: i.quantity,
                unit_price: i.unit_price,
                subtotal: i.quantity as i64 * i.unit_price,
                created_at: now,
            })
            .collect();

        self.orders.write().await.push(order.clone());
        self.items.write().await.extend(order_items.clone());
        Ok((order, order_items))
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: &OrderStatus,
        updated_by: &str,
        expected_version: i32,
    ) -> anyhow::Result<Order> {
        let mut orders = self.orders.write().await;
        let order = orders
            .iter_mut()
            .find(|o| o.id == id && o.version == expected_version)
            .ok_or_else(|| anyhow::anyhow!("Order '{}' not found or version conflict", id))?;
        order.status = status.clone();
        order.updated_by = Some(updated_by.to_string());
        order.version += 1;
        order.updated_at = chrono::Utc::now();
        Ok(order.clone())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        let mut orders = self.orders.write().await;
        let idx = orders
            .iter()
            .position(|o| o.id == id)
            .ok_or_else(|| anyhow::anyhow!("Order '{}' not found", id))?;
        orders.remove(idx);
        Ok(())
    }

    async fn insert_outbox_event(
        &self,
        _aggregate_type: &str,
        _aggregate_id: &str,
        _event_type: &str,
        _payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn fetch_unpublished_events(&self, _limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
        Ok(vec![])
    }

    async fn mark_event_published(&self, _event_id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

fn build_app() -> axum::Router {
    let repo: Arc<dyn OrderRepository> = Arc::new(InMemoryOrderRepository::default());
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("order-test"));

    let state = AppState {
        create_order_uc: Arc::new(usecase::create_order::CreateOrderUseCase::new(repo.clone())),
        get_order_uc: Arc::new(usecase::get_order::GetOrderUseCase::new(repo.clone())),
        update_order_status_uc: Arc::new(
            usecase::update_order_status::UpdateOrderStatusUseCase::new(repo.clone()),
        ),
        list_orders_uc: Arc::new(usecase::list_orders::ListOrdersUseCase::new(repo)),
        metrics,
        auth_state: None,
    };

    handler::router(state)
}

#[tokio::test]
async fn test_healthz() {
    let app = build_app();
    let response = app
        .oneshot(Request::builder().uri("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_and_get_order() {
    let app = build_app();

    // Create order
    let body = serde_json::json!({
        "customer_id": "CUST-001",
        "currency": "JPY",
        "items": [{
            "product_id": "PROD-001",
            "product_name": "Widget",
            "quantity": 2,
            "unit_price": 1000
        }]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/orders")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(created["customer_id"], "CUST-001");
    assert_eq!(created["status"], "pending");
    assert_eq!(created["total_amount"], 2000);
    assert_eq!(created["version"], 1);
    assert!(created["items"].as_array().unwrap().len() == 1);

    let order_id = created["id"].as_str().unwrap();

    // Get order
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/orders/{}", order_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_order_not_found() {
    let app = build_app();
    let fake_id = Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/orders/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(err["error"]["code"], "SVC_ORDER_NOT_FOUND");
    assert!(!err["error"]["request_id"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn test_invalid_uuid_returns_400() {
    let app = build_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/orders/not-a-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(err["error"]["code"], "SVC_ORDER_VALIDATION_FAILED");
    assert!(!err["error"]["details"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_create_order_validation_error() {
    let app = build_app();
    let body = serde_json::json!({
        "customer_id": "",
        "currency": "JPY",
        "items": []
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/orders")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(err["error"]["code"], "SVC_ORDER_VALIDATION_FAILED");
}

#[tokio::test]
async fn test_list_orders() {
    let app = build_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/orders")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(list["total"], 0);
    assert!(list["orders"].as_array().unwrap().is_empty());
}

/// ステータス更新テスト: pending → confirmed（正常遷移）
#[tokio::test]
async fn test_update_order_status_success() {
    let app = build_app();

    // まず注文を作成
    let body = serde_json::json!({
        "customer_id": "CUST-002",
        "currency": "JPY",
        "items": [{
            "product_id": "PROD-010",
            "product_name": "Gadget",
            "quantity": 1,
            "unit_price": 5000
        }]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/orders")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let order_id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "pending");

    // pending → confirmed に遷移
    let update_body = serde_json::json!({ "status": "confirmed" });
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/orders/{}/status", order_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(updated["status"], "confirmed");
    assert_eq!(updated["version"], 2);
}

/// ステータス更新テスト: pending → shipped（不正遷移）
#[tokio::test]
async fn test_update_order_status_invalid_transition() {
    let app = build_app();

    // まず注文を作成
    let body = serde_json::json!({
        "customer_id": "CUST-003",
        "currency": "USD",
        "items": [{
            "product_id": "PROD-020",
            "product_name": "Thingamajig",
            "quantity": 2,
            "unit_price": 3000
        }]
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/orders")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let order_id = created["id"].as_str().unwrap();

    // pending → shipped（不正遷移）
    let update_body = serde_json::json!({ "status": "shipped" });
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/orders/{}/status", order_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(err["error"]["code"], "SVC_ORDER_INVALID_STATUS_TRANSITION");
}
