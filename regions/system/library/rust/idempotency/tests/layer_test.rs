use std::sync::Arc;

use axum::body::Body;
use axum::middleware;
use axum::routing::post;
use axum::Router;
use http::Request;
use http_body_util::BodyExt;
use k1s0_idempotency::{
    idempotency_middleware, IdempotencyState, IdempotencyStore, InMemoryIdempotencyStore,
    IDEMPOTENCY_KEY_HEADER,
};
use tower::ServiceExt;

fn app(store: Arc<InMemoryIdempotencyStore>) -> Router {
    let state = IdempotencyState::new(store as Arc<dyn IdempotencyStore>);
    Router::new()
        .route("/create", post(|| async { "created" }))
        .layer(middleware::from_fn_with_state(
            state,
            idempotency_middleware,
        ))
}

async fn send_request(app: Router, key: Option<&str>) -> (u16, String) {
    let mut builder = Request::builder().method("POST").uri("/create");
    if let Some(k) = key {
        builder = builder.header(IDEMPOTENCY_KEY_HEADER, k);
    }
    let req = builder.body(Body::empty()).unwrap();
    let response = app.oneshot(req).await.unwrap();
    let status = response.status().as_u16();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body).to_string();
    (status, body_str)
}

#[tokio::test]
async fn test_no_idempotency_key_passes_through() {
    let store = Arc::new(InMemoryIdempotencyStore::new());
    let (status, body) = send_request(app(store), None).await;
    assert_eq!(status, 200);
    assert_eq!(body, "created");
}

#[tokio::test]
async fn test_first_request_with_key_passes_through() {
    let store = Arc::new(InMemoryIdempotencyStore::new());
    let (status, body) = send_request(app(store), Some("key-1")).await;
    assert_eq!(status, 200);
    assert_eq!(body, "created");
}

#[tokio::test]
async fn test_duplicate_request_returns_cached_response() {
    let store = Arc::new(InMemoryIdempotencyStore::new());

    let (status1, body1) = send_request(app(store.clone()), Some("dup-1")).await;
    assert_eq!(status1, 200);
    assert_eq!(body1, "created");

    let (status2, body2) = send_request(app(store.clone()), Some("dup-1")).await;
    assert_eq!(status2, 200);
    assert_eq!(body2, "created");
}

#[tokio::test]
async fn test_duplicate_response_has_replayed_header() {
    let store = Arc::new(InMemoryIdempotencyStore::new());

    // 最初のリクエスト
    let req1 = Request::builder()
        .method("POST")
        .uri("/create")
        .header(IDEMPOTENCY_KEY_HEADER, "hdr-test")
        .body(Body::empty())
        .unwrap();
    let resp1 = app(store.clone()).oneshot(req1).await.unwrap();
    assert!(resp1.headers().get("x-idempotent-replayed").is_none());

    // 2回目はリプレイヘッダーあり
    let req2 = Request::builder()
        .method("POST")
        .uri("/create")
        .header(IDEMPOTENCY_KEY_HEADER, "hdr-test")
        .body(Body::empty())
        .unwrap();
    let resp2 = app(store.clone()).oneshot(req2).await.unwrap();
    assert_eq!(
        resp2.headers().get("x-idempotent-replayed").unwrap(),
        "true"
    );
}

#[tokio::test]
async fn test_different_keys_are_independent() {
    let store = Arc::new(InMemoryIdempotencyStore::new());

    let (s1, _) = send_request(app(store.clone()), Some("key-a")).await;
    let (s2, _) = send_request(app(store.clone()), Some("key-b")).await;
    assert_eq!(s1, 200);
    assert_eq!(s2, 200);
}

#[tokio::test]
async fn test_store_records_completed_status() {
    let store = Arc::new(InMemoryIdempotencyStore::new());

    send_request(app(store.clone()), Some("check-store")).await;

    let record = store.get("check-store").await.unwrap().unwrap();
    assert_eq!(
        record.status,
        k1s0_idempotency::IdempotencyStatus::Completed
    );
    assert_eq!(record.response_status, Some(200));
    assert!(record.response_body.is_some());
}

#[tokio::test]
async fn test_failed_handler_records_failed_status() {
    let store = Arc::new(InMemoryIdempotencyStore::new());
    let state = IdempotencyState::new(store.clone() as Arc<dyn IdempotencyStore>);

    let app = Router::new()
        .route(
            "/fail",
            post(|| async { (http::StatusCode::INTERNAL_SERVER_ERROR, "oops") }),
        )
        .layer(middleware::from_fn_with_state(
            state,
            idempotency_middleware,
        ));

    let req = Request::builder()
        .method("POST")
        .uri("/fail")
        .header(IDEMPOTENCY_KEY_HEADER, "fail-key")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status().as_u16(), 500);

    let record = store.get("fail-key").await.unwrap().unwrap();
    assert_eq!(record.status, k1s0_idempotency::IdempotencyStatus::Failed);
    assert_eq!(record.response_status, Some(500));
}

#[tokio::test]
async fn test_failed_request_allows_retry() {
    let store = Arc::new(InMemoryIdempotencyStore::new());

    // 失敗するハンドラー
    let fail_state = IdempotencyState::new(store.clone() as Arc<dyn IdempotencyStore>);
    let fail_app = Router::new()
        .route(
            "/endpoint",
            post(|| async { (http::StatusCode::INTERNAL_SERVER_ERROR, "error") }),
        )
        .layer(middleware::from_fn_with_state(
            fail_state,
            idempotency_middleware,
        ));

    let req = Request::builder()
        .method("POST")
        .uri("/endpoint")
        .header(IDEMPOTENCY_KEY_HEADER, "retry-key")
        .body(Body::empty())
        .unwrap();
    let resp = fail_app.oneshot(req).await.unwrap();
    assert_eq!(resp.status().as_u16(), 500);

    // 同じキーで再実行 → 前回 Failed なので再実行可能
    let ok_state = IdempotencyState::new(store.clone() as Arc<dyn IdempotencyStore>);
    let ok_app = Router::new()
        .route("/endpoint", post(|| async { "ok" }))
        .layer(middleware::from_fn_with_state(
            ok_state,
            idempotency_middleware,
        ));

    let req2 = Request::builder()
        .method("POST")
        .uri("/endpoint")
        .header(IDEMPOTENCY_KEY_HEADER, "retry-key")
        .body(Body::empty())
        .unwrap();
    let resp2 = ok_app.oneshot(req2).await.unwrap();
    assert_eq!(resp2.status().as_u16(), 200);
}
