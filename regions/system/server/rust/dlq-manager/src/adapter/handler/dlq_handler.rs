use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::DlqError;
use super::AppState;

// --- Request / Response DTOs ---

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    #[serde(default = "default_page")]
    pub page: i32,
    #[serde(default = "default_page_size")]
    pub page_size: i32,
}

fn default_page() -> i32 {
    1
}

fn default_page_size() -> i32 {
    20
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DlqMessageResponse {
    pub id: String,
    pub original_topic: String,
    pub error_message: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub payload: serde_json::Value,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_retry_at: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ListMessagesResponse {
    pub messages: Vec<DlqMessageResponse>,
    pub pagination: PaginationResponse,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginationResponse {
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RetryMessageResponse {
    pub id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RetryAllResponse {
    pub retried: i64,
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DeleteMessageResponse {
    pub success: bool,
    pub message: String,
}

// --- Helper ---

fn to_message_response(msg: &crate::domain::entity::DlqMessage) -> DlqMessageResponse {
    DlqMessageResponse {
        id: msg.id.to_string(),
        original_topic: msg.original_topic.clone(),
        error_message: msg.error_message.clone(),
        retry_count: msg.retry_count,
        max_retries: msg.max_retries,
        payload: msg.payload.clone(),
        status: msg.status.to_string(),
        created_at: msg.created_at.to_rfc3339(),
        updated_at: msg.updated_at.to_rfc3339(),
        last_retry_at: msg.last_retry_at.map(|t| t.to_rfc3339()),
    }
}

// --- Handlers ---

#[utoipa::path(get, path = "/healthz", responses((status = 200, description = "Health check OK")))]
pub async fn healthz() -> &'static str {
    "ok"
}

#[utoipa::path(get, path = "/readyz", responses((status = 200, description = "Ready")))]
pub async fn readyz() -> &'static str {
    "ok"
}

#[utoipa::path(get, path = "/metrics", responses((status = 200, description = "Prometheus metrics")))]
pub async fn metrics(State(state): State<AppState>) -> String {
    state.metrics.gather_metrics()
}

#[utoipa::path(
    get,
    path = "/api/v1/dlq/{topic}",
    params(
        ("topic" = String, Path, description = "DLQ topic name"),
        ("page" = Option<i32>, Query, description = "Page number"),
        ("page_size" = Option<i32>, Query, description = "Page size"),
    ),
    responses(
        (status = 200, description = "Message list", body = ListMessagesResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_messages(
    State(state): State<AppState>,
    Path(topic): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<ListMessagesResponse>, DlqError> {
    let (messages, total) = state
        .list_messages_uc
        .execute(&topic, query.page, query.page_size)
        .await
        .map_err(|e| DlqError::Internal(e.to_string()))?;

    let message_responses: Vec<DlqMessageResponse> =
        messages.iter().map(to_message_response).collect();

    let has_next = (query.page as i64 * query.page_size as i64) < total;

    Ok(Json(ListMessagesResponse {
        messages: message_responses,
        pagination: PaginationResponse {
            total_count: total,
            page: query.page,
            page_size: query.page_size,
            has_next,
        },
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/dlq/messages/{id}",
    params(("id" = String, Path, description = "Message ID")),
    responses(
        (status = 200, description = "Message found", body = DlqMessageResponse),
        (status = 404, description = "Message not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DlqMessageResponse>, DlqError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DlqError::Validation(format!("invalid message id: {}", id)))?;

    let message = state.get_message_uc.execute(uuid).await.map_err(|e| {
        if e.to_string().contains("not found") {
            DlqError::NotFound(e.to_string())
        } else {
            DlqError::Internal(e.to_string())
        }
    })?;

    Ok(Json(to_message_response(&message)))
}

#[utoipa::path(
    post,
    path = "/api/v1/dlq/messages/{id}/retry",
    params(("id" = String, Path, description = "Message ID")),
    responses(
        (status = 200, description = "Retry initiated", body = RetryMessageResponse),
        (status = 404, description = "Message not found"),
        (status = 409, description = "Message not retryable"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn retry_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<RetryMessageResponse>, DlqError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DlqError::Validation(format!("invalid message id: {}", id)))?;

    let message = state.retry_message_uc.execute(uuid).await.map_err(|e| {
        if e.to_string().contains("not found") {
            DlqError::NotFound(e.to_string())
        } else if e.to_string().contains("not retryable") {
            DlqError::Conflict(e.to_string())
        } else {
            DlqError::Internal(e.to_string())
        }
    })?;

    Ok(Json(RetryMessageResponse {
        id: message.id.to_string(),
        status: message.status.to_string(),
        message: "message retry initiated".to_string(),
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/dlq/messages/{id}",
    params(("id" = String, Path, description = "Message ID")),
    responses(
        (status = 200, description = "Message deleted", body = DeleteMessageResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteMessageResponse>, DlqError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| DlqError::Validation(format!("invalid message id: {}", id)))?;

    state
        .delete_message_uc
        .execute(uuid)
        .await
        .map_err(|e| DlqError::Internal(e.to_string()))?;

    Ok(Json(DeleteMessageResponse {
        success: true,
        message: format!("message {} deleted", id),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/dlq/{topic}/retry-all",
    params(("topic" = String, Path, description = "DLQ topic name")),
    responses(
        (status = 200, description = "All messages retried", body = RetryAllResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn retry_all(
    State(state): State<AppState>,
    Path(topic): Path<String>,
) -> Result<Json<RetryAllResponse>, DlqError> {
    let retried = state
        .retry_all_uc
        .execute(&topic)
        .await
        .map_err(|e| DlqError::Internal(e.to_string()))?;

    Ok(Json(RetryAllResponse {
        retried,
        message: format!("{} messages retried in topic {}", retried, topic),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::handler;
    use crate::domain::entity::DlqMessage;
    use crate::domain::repository::dlq_message_repository::MockDlqMessageRepository;
    use crate::usecase::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    fn make_test_state(mock: MockDlqMessageRepository) -> AppState {
        let repo: Arc<dyn crate::domain::repository::DlqMessageRepository> = Arc::new(mock);
        AppState {
            list_messages_uc: Arc::new(ListMessagesUseCase::new(repo.clone())),
            get_message_uc: Arc::new(GetMessageUseCase::new(repo.clone())),
            retry_message_uc: Arc::new(RetryMessageUseCase::new(repo.clone(), None)),
            delete_message_uc: Arc::new(DeleteMessageUseCase::new(repo.clone())),
            retry_all_uc: Arc::new(RetryAllUseCase::new(repo, None)),
            metrics: Arc::new(k1s0_telemetry::metrics::Metrics::new("k1s0-dlq-manager")),
            auth_state: None,
        }
    }

    #[tokio::test]
    async fn test_healthz() {
        let mock = MockDlqMessageRepository::new();
        let app = handler::router(make_test_state(mock));

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
    async fn test_readyz() {
        let mock = MockDlqMessageRepository::new();
        let app = handler::router(make_test_state(mock));

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
    async fn test_list_messages() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_topic()
            .returning(|_, _, _| Ok((vec![], 0)));

        let app = handler::router(make_test_state(mock));

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
    }

    #[tokio::test]
    async fn test_get_message_not_found() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let app = handler::router(make_test_state(mock));
        let id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/v1/dlq/messages/{}", id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_message_found() {
        let msg = DlqMessage::new(
            "orders.events.v1".to_string(),
            "failed".to_string(),
            serde_json::json!({}),
            3,
        );
        let msg_id = msg.id;
        let msg_clone = msg.clone();

        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(msg_clone.clone())));

        let app = handler::router(make_test_state(mock));

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
    }

    #[tokio::test]
    async fn test_get_message_invalid_id() {
        let mock = MockDlqMessageRepository::new();
        let app = handler::router(make_test_state(mock));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/dlq/messages/not-a-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_delete_message() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_delete().returning(|_| Ok(()));

        let app = handler::router(make_test_state(mock));
        let id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/v1/dlq/messages/{}", id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_retry_message_not_found() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let app = handler::router(make_test_state(mock));
        let id = Uuid::new_v4();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/dlq/messages/{}/retry", id))
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_retry_all() {
        let mut mock = MockDlqMessageRepository::new();
        mock.expect_find_by_topic()
            .returning(|_, _, _| Ok((vec![], 0)));

        let app = handler::router(make_test_state(mock));

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/dlq/orders.dlq.v1/retry-all")
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_message_success() {
        let msg_id = Uuid::new_v4();

        let mut mock = MockDlqMessageRepository::new();
        mock.expect_delete().returning(|_| Ok(()));

        let app = handler::router(make_test_state(mock));

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
}
