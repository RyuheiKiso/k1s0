/// dlq-manager integration tests
/// インメモリリポジトリを使って REST API のエンドツーエンド動作を検証する。
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use k1s0_dlq_manager::adapter::handler;
use k1s0_dlq_manager::adapter::handler::AppState;
use k1s0_dlq_manager::domain::entity::DlqMessage;
use k1s0_dlq_manager::domain::repository::DlqMessageRepository;
use k1s0_dlq_manager::infrastructure::kafka::producer::DlqEventPublisher;
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

// ---------------------------------------------------------------------------
// SpyPublisher: Kafka 発行をスパイするテスト用実装
// ---------------------------------------------------------------------------

/// 発行されたトピックを記録するスパイ Publisher。
struct SpyPublisher {
    /// 発行されたトピック名の記録
    published_topics: Arc<Mutex<Vec<String>>>,
    /// true の場合、publish を失敗させる
    should_fail: bool,
}

impl SpyPublisher {
    /// 成功する SpyPublisher と、記録参照用の Arc を返す。
    fn new_success() -> (Self, Arc<Mutex<Vec<String>>>) {
        let topics = Arc::new(Mutex::new(Vec::new()));
        let spy = Self {
            published_topics: topics.clone(),
            should_fail: false,
        };
        (spy, topics)
    }

    /// 失敗する SpyPublisher を返す。
    fn new_failing() -> Self {
        Self {
            published_topics: Arc::new(Mutex::new(Vec::new())),
            should_fail: true,
        }
    }
}

#[async_trait]
impl DlqEventPublisher for SpyPublisher {
    async fn publish_to_topic(
        &self,
        topic: &str,
        _payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        if self.should_fail {
            anyhow::bail!("broker unavailable");
        }
        self.published_topics
            .lock()
            .unwrap()
            .push(topic.to_string());
        Ok(())
    }
}

/// Publisher を注入した AppState を構築するヘルパー。
fn make_app_state_with_publisher(
    repo: Arc<dyn DlqMessageRepository>,
    publisher: Arc<dyn DlqEventPublisher>,
) -> AppState {
    AppState {
        list_messages_uc: Arc::new(ListMessagesUseCase::new(repo.clone())),
        get_message_uc: Arc::new(GetMessageUseCase::new(repo.clone())),
        retry_message_uc: Arc::new(RetryMessageUseCase::new(
            repo.clone(),
            Some(publisher.clone()),
        )),
        delete_message_uc: Arc::new(DeleteMessageUseCase::new(repo.clone())),
        retry_all_uc: Arc::new(RetryAllUseCase::new(repo, Some(publisher))),
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

// ---------------------------------------------------------------------------
// Kafka Publisher 連携テスト
// ---------------------------------------------------------------------------

/// リトライ時に Publisher が元トピックへ発行されること。
#[tokio::test]
async fn test_retry_with_publisher_calls_publish_to_original_topic() {
    let msg = DlqMessage::new(
        "orders.events.v1".to_string(),
        "failed".to_string(),
        serde_json::json!({"order_id": "123"}),
        3,
    );
    let msg_id = msg.id;

    let (spy, published_topics) = SpyPublisher::new_success();
    let repo = Arc::new(InMemoryDlqRepo::with_message(msg));
    let app = handler::router(make_app_state_with_publisher(repo, Arc::new(spy)));

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
    // 発行成功 → RESOLVED
    assert_eq!(json["status"], "RESOLVED");

    // Publisher が元トピックへ発行されたことを確認
    let topics = published_topics.lock().unwrap();
    assert_eq!(topics.len(), 1);
    assert_eq!(topics[0], "orders.events.v1");
}

/// Publisher が失敗した場合、メッセージは RETRYING 状態を保つこと。
#[tokio::test]
async fn test_retry_with_failing_publisher_keeps_retrying_status() {
    let msg = DlqMessage::new(
        "payments.events.v1".to_string(),
        "timeout".to_string(),
        serde_json::json!({"amount": 500}),
        3,
    );
    let msg_id = msg.id;

    let spy = SpyPublisher::new_failing();
    let repo = Arc::new(InMemoryDlqRepo::with_message(msg));
    let app = handler::router(make_app_state_with_publisher(repo, Arc::new(spy)));

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

    // Publisher が失敗してもハンドラは 200 OK を返す
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // 発行失敗 → RETRYING のまま
    assert_eq!(json["status"], "RETRYING");
}

/// retry-all で Publisher が全 PENDING メッセージを発行し、RESOLVED になること。
#[tokio::test]
async fn test_retry_all_with_successful_publisher_resolves_all_messages() {
    let msg1 = DlqMessage::new(
        "orders.events.v1".to_string(),
        "failed".to_string(),
        serde_json::json!({}),
        3,
    );
    let msg2 = DlqMessage::new(
        "orders.events.v1".to_string(),
        "timeout".to_string(),
        serde_json::json!({}),
        3,
    );

    let (spy, published_topics) = SpyPublisher::new_success();
    let repo = Arc::new(InMemoryDlqRepo::new());
    repo.create(&msg1).await.unwrap();
    repo.create(&msg2).await.unwrap();

    let app = handler::router(make_app_state_with_publisher(repo, Arc::new(spy)));

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
    // 2 件のメッセージがリトライされた
    assert_eq!(json["retried"], 2);

    // 2 件のトピックへの発行が記録されていること
    let topics = published_topics.lock().unwrap();
    assert_eq!(topics.len(), 2);
    assert!(topics.iter().all(|t| t == "orders.events.v1"));
}

/// max_retries に達したメッセージは CONFLICT エラーになること。
#[tokio::test]
async fn test_retry_exhausted_message_returns_conflict() {
    // max_retries = 1、すでに 1 回リトライ済み → 不可
    let mut msg = DlqMessage::new(
        "orders.events.v1".to_string(),
        "failed".to_string(),
        serde_json::json!({}),
        1,
    );
    msg.mark_retrying(); // retry_count = 1 = max_retries → not retryable
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

    // リトライ不可 → 409 CONFLICT
    assert_eq!(response.status(), StatusCode::CONFLICT);
}
