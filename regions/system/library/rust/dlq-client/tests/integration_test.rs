use k1s0_dlq_client::{DlqClient, DlqStatus};
use wiremock::matchers::{method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_dlq_message(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "original_topic": "orders.v1",
        "error_message": "processing failed",
        "retry_count": 1,
        "max_retries": 3,
        "payload": {"order_id": "123"},
        "status": "PENDING",
        "created_at": "2024-01-01T00:00:00Z",
        "last_retry_at": null
    })
}

#[tokio::test]
async fn test_list_messages_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/dlq/.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "messages": [],
            "total": 0,
            "page": 1
        })))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.list_messages("orders.v1", 1, 20).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.messages.len(), 0);
    assert_eq!(resp.total, 0);
    assert_eq!(resp.page, 1);
}

#[tokio::test]
async fn test_list_messages_with_data() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path_regex(r"/api/v1/dlq/.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "messages": [make_dlq_message("msg-001"), make_dlq_message("msg-002")],
            "total": 2,
            "page": 1
        })))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.list_messages("orders.v1", 1, 20).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.messages.len(), 2);
    assert_eq!(resp.messages[0].id, "msg-001");
    assert_eq!(resp.messages[1].id, "msg-002");
    assert_eq!(resp.total, 2);
}

#[tokio::test]
async fn test_list_messages_pagination() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/dlq/orders.v1"))
        .and(query_param("page", "2"))
        .and(query_param("page_size", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "messages": [make_dlq_message("msg-011")],
            "total": 15,
            "page": 2
        })))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.list_messages("orders.v1", 2, 10).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.page, 2);
    assert_eq!(resp.total, 15);
    assert_eq!(resp.messages.len(), 1);
}

#[tokio::test]
async fn test_get_message_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/dlq/messages/msg-100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(make_dlq_message("msg-100")))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.get_message("msg-100").await;
    assert!(result.is_ok());
    let msg = result.unwrap();
    assert_eq!(msg.id, "msg-100");
    assert_eq!(msg.original_topic, "orders.v1");
    assert_eq!(msg.status, DlqStatus::Pending);
}

#[tokio::test]
async fn test_get_message_not_found_returns_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/dlq/messages/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.get_message("nonexistent").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("404"));
}

#[tokio::test]
async fn test_get_message_server_error_returns_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/dlq/messages/msg-500"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.get_message("msg-500").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("500"));
}

#[tokio::test]
async fn test_retry_message_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/dlq/messages/msg-200/retry"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "msg-200",
            "status": "RETRYING"
        })))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.retry_message("msg-200").await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.message_id, "msg-200");
    assert_eq!(resp.status, DlqStatus::Retrying);
}

#[tokio::test]
async fn test_retry_message_conflict_returns_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/dlq/messages/msg-conflict/retry"))
        .respond_with(ResponseTemplate::new(409).set_body_string("already retrying"))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.retry_message("msg-conflict").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("409"));
}

#[tokio::test]
async fn test_retry_message_not_found_returns_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/dlq/messages/msg-missing/retry"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.retry_message("msg-missing").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("404"));
}

#[tokio::test]
async fn test_delete_message_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/dlq/messages/msg-del"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.delete_message("msg-del").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_message_not_found_returns_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/api/v1/dlq/messages/msg-gone"))
        .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.delete_message("msg-gone").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("404"));
}

#[tokio::test]
async fn test_retry_all_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/dlq/orders.v1/retry-all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "retried_count": 5
        })))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.retry_all("orders.v1").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_retry_all_server_error_returns_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/dlq/orders.v1/retry-all"))
        .respond_with(ResponseTemplate::new(500).set_body_string("service unavailable"))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.retry_all("orders.v1").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("500"));
}

#[tokio::test]
async fn test_list_messages_empty_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/dlq/empty-topic"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "messages": [],
            "total": 0,
            "page": 1
        })))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.list_messages("empty-topic", 1, 20).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.total, 0);
    assert!(resp.messages.is_empty());
}

#[tokio::test]
async fn test_list_messages_multiple_statuses() {
    let mock_server = MockServer::start().await;

    let pending_msg = serde_json::json!({
        "id": "msg-p1",
        "original_topic": "orders.v1",
        "error_message": "timeout",
        "retry_count": 0,
        "max_retries": 3,
        "payload": {"order_id": "456"},
        "status": "PENDING",
        "created_at": "2024-01-01T00:00:00Z",
        "last_retry_at": null
    });

    let retrying_msg = serde_json::json!({
        "id": "msg-r1",
        "original_topic": "orders.v1",
        "error_message": "connection refused",
        "retry_count": 2,
        "max_retries": 3,
        "payload": {"order_id": "789"},
        "status": "RETRYING",
        "created_at": "2024-01-01T01:00:00Z",
        "last_retry_at": "2024-01-01T02:00:00Z"
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/dlq/orders.v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "messages": [pending_msg, retrying_msg],
            "total": 2,
            "page": 1
        })))
        .mount(&mock_server)
        .await;

    let client = DlqClient::new(&mock_server.uri());
    let result = client.list_messages("orders.v1", 1, 20).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.messages.len(), 2);
    assert_eq!(resp.messages[0].status, DlqStatus::Pending);
    assert_eq!(resp.messages[1].status, DlqStatus::Retrying);
    assert!(resp.messages[1].last_retry_at.is_some());
}
