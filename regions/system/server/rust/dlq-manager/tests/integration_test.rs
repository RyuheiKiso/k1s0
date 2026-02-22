/// dlq-manager integration tests
/// インメモリリポジトリを使って REST API のエンドツーエンド動作を検証する。
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use k1s0_dlq_manager::adapter::handler;
use k1s0_dlq_manager::adapter::handler::AppState;
use k1s0_dlq_manager::domain::entity::DlqMessage;
use k1s0_dlq_manager::domain::repository::DlqMessageRepository;
use k1s0_dlq_manager::usecase::{
    DeleteMessageUseCase, GetMessageUseCase, ListMessagesUseCase, RetryAllUseCase,
    RetryMessageUseCase,
};
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

/// テスト用インメモリリポジトリ
struct InMemoryDlqRepo {
    messages: RwLock<Vec<DlqMessage>>,
}

impl InMemoryDlqRepo {
    fn new() -> Self {
        Self {
            messages: RwLock::new(Vec::new()),
        }
    }

    fn with_message(msg: DlqMessage) -> Self {
        Self {
            messages: RwLock::new(vec![msg]),
        }
    }
}

#[async_trait]
impl DlqMessageRepository for InMemoryDlqRepo {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<DlqMessage>> {
        let msgs = self.messages.read().await;
        Ok(msgs.iter().find(|m| m.id == id).cloned())
    }

    async fn find_by_topic(
        &self,
        topic: &str,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<(Vec<DlqMessage>, i64)> {
        let msgs = self.messages.read().await;
        let filtered: Vec<_> = msgs
            .iter()
            .filter(|m| m.original_topic.contains(topic) || topic.contains(&m.original_topic))
            .cloned()
            .collect();
        let total = filtered.len() as i64;
        let start = ((page - 1) * page_size) as usize;
        let page_items = filtered
            .into_iter()
            .skip(start)
            .take(page_size as usize)
            .collect();
        Ok((page_items, total))
    }

    async fn create(&self, message: &DlqMessage) -> anyhow::Result<()> {
        self.messages.write().await.push(message.clone());
        Ok(())
    }

    async fn update(&self, message: &DlqMessage) -> anyhow::Result<()> {
        let mut msgs = self.messages.write().await;
        if let Some(m) = msgs.iter_mut().find(|m| m.id == message.id) {
            *m = message.clone();
        }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        let mut msgs = self.messages.write().await;
        msgs.retain(|m| m.id != id);
        Ok(())
    }

    async fn count_by_topic(&self, topic: &str) -> anyhow::Result<i64> {
        let msgs = self.messages.read().await;
        let count = msgs
            .iter()
            .filter(|m| m.original_topic.contains(topic))
            .count() as i64;
        Ok(count)
    }
}

fn make_app_state(repo: Arc<dyn DlqMessageRepository>) -> AppState {
    AppState {
        list_messages_uc: Arc::new(ListMessagesUseCase::new(repo.clone())),
        get_message_uc: Arc::new(GetMessageUseCase::new(repo.clone())),
        retry_message_uc: Arc::new(RetryMessageUseCase::new(repo.clone(), None)),
        delete_message_uc: Arc::new(DeleteMessageUseCase::new(repo.clone())),
        retry_all_uc: Arc::new(RetryAllUseCase::new(repo, None)),
        metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("test")),
    }
}

#[tokio::test]
async fn test_healthz_returns_ok() {
    let repo = Arc::new(InMemoryDlqRepo::new());
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_readyz_returns_ok() {
    let repo = Arc::new(InMemoryDlqRepo::new());
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/readyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_messages_empty_topic() {
    let repo = Arc::new(InMemoryDlqRepo::new());
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/dlq/orders.dlq.v1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["pagination"]["total_count"], 0);
    assert_eq!(json["messages"], serde_json::json!([]));
}

#[tokio::test]
async fn test_list_messages_returns_stored_message() {
    let msg = DlqMessage::new(
        "orders.events.v1".to_string(),
        "processing failed".to_string(),
        serde_json::json!({"order_id": "123"}),
        3,
    );
    let repo = Arc::new(InMemoryDlqRepo::with_message(msg));
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/dlq/orders.events.v1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["pagination"]["total_count"], 1);
    assert_eq!(json["messages"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_get_message_returns_404_when_not_found() {
    let repo = Arc::new(InMemoryDlqRepo::new());
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/dlq/messages/{}", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_message_returns_message() {
    let msg = DlqMessage::new(
        "payments.events.v1".to_string(),
        "timeout".to_string(),
        serde_json::json!({"amount": 100}),
        5,
    );
    let msg_id = msg.id;
    let repo = Arc::new(InMemoryDlqRepo::with_message(msg));
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/dlq/messages/{}", msg_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["original_topic"], "payments.events.v1");
    assert_eq!(json["status"], "PENDING");
}

#[tokio::test]
async fn test_get_message_returns_400_for_invalid_id() {
    let repo = Arc::new(InMemoryDlqRepo::new());
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/dlq/messages/not-a-valid-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_retry_message_returns_404_when_not_found() {
    let repo = Arc::new(InMemoryDlqRepo::new());
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/dlq/messages/{}/retry", Uuid::new_v4()))
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_retry_message_resolves_pending_message() {
    let msg = DlqMessage::new(
        "orders.events.v1".to_string(),
        "failed".to_string(),
        serde_json::json!({}),
        3,
    );
    let msg_id = msg.id;
    let repo = Arc::new(InMemoryDlqRepo::with_message(msg));
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/dlq/messages/{}/retry", msg_id))
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // publisher なしなので RESOLVED になる
    assert_eq!(json["status"], "RESOLVED");
}

#[tokio::test]
async fn test_delete_message_returns_ok() {
    let msg = DlqMessage::new(
        "orders.events.v1".to_string(),
        "failed".to_string(),
        serde_json::json!({}),
        3,
    );
    let msg_id = msg.id;
    let repo = Arc::new(InMemoryDlqRepo::with_message(msg));
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/v1/dlq/messages/{}", msg_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_retry_all_returns_retried_count() {
    let msg = DlqMessage::new(
        "orders.events.v1".to_string(),
        "failed".to_string(),
        serde_json::json!({}),
        3,
    );
    let repo = Arc::new(InMemoryDlqRepo::with_message(msg));
    let app = handler::router(make_app_state(repo));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/dlq/orders.events.v1/retry-all")
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["retried"], 1);
}
