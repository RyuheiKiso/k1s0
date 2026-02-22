use k1s0_saga::{SagaClient, SagaStatus, StartSagaRequest};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_saga_state(saga_id: &str) -> serde_json::Value {
    serde_json::json!({
        "saga": {
            "saga_id": saga_id,
            "workflow_name": "order-fulfillment",
            "current_step": 0,
            "status": "STARTED",
            "payload": {},
            "correlation_id": null,
            "initiated_by": null,
            "error_message": null,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        }
    })
}

fn make_start_response(saga_id: &str) -> serde_json::Value {
    serde_json::json!({
        "saga_id": saga_id,
        "status": "STARTED"
    })
}

#[tokio::test]
async fn test_start_saga_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/sagas"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(make_start_response("550e8400-e29b-41d4-a716-446655440000")),
        )
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let req = StartSagaRequest {
        workflow_name: "order-fulfillment".to_string(),
        payload: serde_json::json!({"order_id": "ord-001"}),
        correlation_id: None,
        initiated_by: None,
    };
    let result = client.start_saga(&req).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.saga_id, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(resp.status, "STARTED");
}

#[tokio::test]
async fn test_start_saga_with_correlation_id() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/sagas"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(make_start_response("660e8400-e29b-41d4-a716-446655440001")),
        )
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let req = StartSagaRequest {
        workflow_name: "order-fulfillment".to_string(),
        payload: serde_json::json!({}),
        correlation_id: Some("corr-123".to_string()),
        initiated_by: None,
    };
    let result = client.start_saga(&req).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.saga_id, "660e8400-e29b-41d4-a716-446655440001");
}

#[tokio::test]
async fn test_start_saga_server_error_returns_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/sagas"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let req = StartSagaRequest {
        workflow_name: "order-fulfillment".to_string(),
        payload: serde_json::json!({}),
        correlation_id: None,
        initiated_by: None,
    };
    let result = client.start_saga(&req).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("500"));
}

#[tokio::test]
async fn test_start_saga_bad_request_returns_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/sagas"))
        .respond_with(
            ResponseTemplate::new(400).set_body_string("invalid workflow_name"),
        )
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let req = StartSagaRequest {
        workflow_name: String::new(),
        payload: serde_json::json!({}),
        correlation_id: None,
        initiated_by: None,
    };
    let result = client.start_saga(&req).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("400"));
}

#[tokio::test]
async fn test_get_saga_success() {
    let mock_server = MockServer::start().await;
    let saga_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/sagas/{}", saga_id)))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(make_saga_state(saga_id)),
        )
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let result = client.get_saga(saga_id).await;
    assert!(result.is_ok());
    let state = result.unwrap();
    assert_eq!(state.saga_id.to_string(), saga_id);
    assert_eq!(state.workflow_name, "order-fulfillment");
    assert_eq!(state.status, SagaStatus::Started);
    assert_eq!(state.current_step, 0);
}

#[tokio::test]
async fn test_get_saga_with_all_fields() {
    let mock_server = MockServer::start().await;
    let saga_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/sagas/{}", saga_id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "saga": {
                "saga_id": saga_id,
                "workflow_name": "order-fulfillment",
                "current_step": 2,
                "status": "RUNNING",
                "payload": {"order_id": "ord-001"},
                "correlation_id": "corr-456",
                "initiated_by": "user-admin",
                "error_message": null,
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T01:00:00Z"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let result = client.get_saga(saga_id).await;
    assert!(result.is_ok());
    let state = result.unwrap();
    assert_eq!(state.status, SagaStatus::Running);
    assert_eq!(state.current_step, 2);
    assert_eq!(state.correlation_id.as_deref(), Some("corr-456"));
    assert_eq!(state.initiated_by.as_deref(), Some("user-admin"));
}

#[tokio::test]
async fn test_get_saga_not_found_returns_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/sagas/nonexistent-id"))
        .respond_with(ResponseTemplate::new(404).set_body_string("saga not found"))
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let result = client.get_saga("nonexistent-id").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("404"));
}

#[tokio::test]
async fn test_get_saga_invalid_json_returns_error() {
    let mock_server = MockServer::start().await;
    let saga_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/sagas/{}", saga_id)))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("this is not valid json{{{"),
        )
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let result = client.get_saga(saga_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cancel_saga_success() {
    let mock_server = MockServer::start().await;
    let saga_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/sagas/{}/cancel", saga_id)))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let result = client.cancel_saga(saga_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cancel_saga_not_found_returns_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/sagas/missing-id/cancel"))
        .respond_with(ResponseTemplate::new(404).set_body_string("saga not found"))
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let result = client.cancel_saga("missing-id").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("404"));
}

#[tokio::test]
async fn test_cancel_saga_conflict_returns_error() {
    let mock_server = MockServer::start().await;
    let saga_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/sagas/{}/cancel", saga_id)))
        .respond_with(
            ResponseTemplate::new(409).set_body_string("saga already completed"),
        )
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let result = client.cancel_saga(saga_id).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("409"));
}

#[tokio::test]
async fn test_cancel_saga_server_error_returns_error() {
    let mock_server = MockServer::start().await;
    let saga_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("POST"))
        .and(path(format!("/api/v1/sagas/{}/cancel", saga_id)))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let result = client.cancel_saga(saga_id).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("500"));
}

#[tokio::test]
async fn test_client_endpoint_normalization() {
    let mock_server = MockServer::start().await;
    let saga_id = "550e8400-e29b-41d4-a716-446655440000";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/sagas/{}", saga_id)))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(make_saga_state(saga_id)),
        )
        .mount(&mock_server)
        .await;

    // trailing slash付きのURIでクライアントを作成
    let uri_with_slash = format!("{}/", mock_server.uri());
    let client = SagaClient::new(&uri_with_slash);
    let result = client.get_saga(saga_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_start_saga_with_initiated_by() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/v1/sagas"))
        .respond_with(
            ResponseTemplate::new(201)
                .set_body_json(make_start_response("770e8400-e29b-41d4-a716-446655440002")),
        )
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());
    let req = StartSagaRequest {
        workflow_name: "payment-processing".to_string(),
        payload: serde_json::json!({"amount": 100}),
        correlation_id: Some("corr-789".to_string()),
        initiated_by: Some("service-order".to_string()),
    };
    let result = client.start_saga(&req).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.saga_id, "770e8400-e29b-41d4-a716-446655440002");
}

#[tokio::test]
async fn test_concurrent_requests() {
    let mock_server = MockServer::start().await;
    let saga_id_1 = "550e8400-e29b-41d4-a716-446655440000";
    let saga_id_2 = "660e8400-e29b-41d4-a716-446655440001";

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/sagas/{}", saga_id_1)))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(make_saga_state(saga_id_1)),
        )
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/sagas/{}", saga_id_2)))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(make_saga_state(saga_id_2)),
        )
        .mount(&mock_server)
        .await;

    let client = SagaClient::new(&mock_server.uri());

    let (result1, result2) = tokio::join!(
        client.get_saga(saga_id_1),
        client.get_saga(saga_id_2),
    );

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert_eq!(result1.unwrap().saga_id.to_string(), saga_id_1);
    assert_eq!(result2.unwrap().saga_id.to_string(), saga_id_2);
}
