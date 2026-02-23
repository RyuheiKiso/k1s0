use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http_body_util::BodyExt;

use crate::record::IdempotencyRecord;
use crate::store::IdempotencyStore;
use crate::{IdempotencyError, IdempotencyStatus};

/// Idempotency-Key ヘッダー名
pub const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";

/// IdempotencyLayer の設定
#[derive(Clone)]
pub struct IdempotencyConfig {
    /// TTL（秒）。None の場合は無期限。
    pub ttl_secs: Option<i64>,
    /// ヘッダー名（デフォルト: "idempotency-key"）
    pub header_name: String,
}

impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self {
            ttl_secs: Some(86400), // 24時間
            header_name: IDEMPOTENCY_KEY_HEADER.to_string(),
        }
    }
}

/// axum middleware State に渡すストア + 設定（dyn dispatch）
#[derive(Clone)]
pub struct IdempotencyState {
    pub store: Arc<dyn IdempotencyStore>,
    pub config: IdempotencyConfig,
}

impl IdempotencyState {
    pub fn new(store: Arc<dyn IdempotencyStore>) -> Self {
        Self {
            store,
            config: IdempotencyConfig::default(),
        }
    }

    pub fn with_config(store: Arc<dyn IdempotencyStore>, config: IdempotencyConfig) -> Self {
        Self { store, config }
    }
}

/// axum ミドルウェア関数
///
/// `axum::middleware::from_fn_with_state` で利用する。
///
/// ```ignore
/// use axum::{Router, middleware};
/// use k1s0_idempotency::{idempotency_middleware, IdempotencyState, InMemoryIdempotencyStore};
///
/// let state = IdempotencyState::new(Arc::new(InMemoryIdempotencyStore::new()));
/// let app = Router::new()
///     .route("/create", post(handler))
///     .layer(middleware::from_fn_with_state(state, idempotency_middleware));
/// ```
pub async fn idempotency_middleware(
    State(state): State<IdempotencyState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let idempotency_key = req
        .headers()
        .get(&state.config.header_name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let key = match idempotency_key {
        Some(k) => k,
        None => return next.run(req).await,
    };

    // 既存レコードを確認
    match state.store.get(&key).await {
        Ok(Some(record)) => match record.status {
            IdempotencyStatus::Completed => {
                let status_code = record
                    .response_status
                    .and_then(|s| StatusCode::from_u16(s).ok())
                    .unwrap_or(StatusCode::OK);
                let body_str = record.response_body.unwrap_or_default();
                return (
                    status_code,
                    [("x-idempotent-replayed", "true")],
                    body_str,
                )
                    .into_response();
            }
            IdempotencyStatus::Pending => {
                return (StatusCode::CONFLICT, "リクエストは現在処理中です").into_response();
            }
            IdempotencyStatus::Failed => {
                let _ = state.store.delete(&key).await;
            }
        },
        Ok(None) => {}
        Err(e) => {
            tracing::warn!("idempotency store get error: {}", e);
        }
    }

    // 新規レコードを挿入
    let record = IdempotencyRecord::new(key.clone(), state.config.ttl_secs);
    match state.store.insert(record).await {
        Ok(()) => {}
        Err(IdempotencyError::Duplicate { .. }) => {
            return (StatusCode::CONFLICT, "リクエストは現在処理中です").into_response();
        }
        Err(e) => {
            tracing::warn!("idempotency store insert error: {}", e);
        }
    }

    // ハンドラー実行
    let response = next.run(req).await;

    // レスポンスを読み取ってストアに保存
    let (parts, body) = response.into_parts();
    let body_bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            let _ = state
                .store
                .update(&key, IdempotencyStatus::Failed, None, None)
                .await;
            return Response::from_parts(parts, Body::empty());
        }
    };
    let resp_body_str = String::from_utf8_lossy(&body_bytes).to_string();
    let status_code = parts.status.as_u16();

    if parts.status.is_success() {
        let _ = state
            .store
            .update(
                &key,
                IdempotencyStatus::Completed,
                Some(resp_body_str),
                Some(status_code),
            )
            .await;
    } else {
        let _ = state
            .store
            .update(
                &key,
                IdempotencyStatus::Failed,
                Some(resp_body_str),
                Some(status_code),
            )
            .await;
    }

    Response::from_parts(parts, Body::from(body_bytes))
}
