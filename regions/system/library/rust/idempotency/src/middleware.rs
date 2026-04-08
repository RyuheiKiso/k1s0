use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http_body_util::BodyExt;

use crate::record::IdempotencyRecord;
use crate::store::IdempotencyStore;
use crate::IdempotencyError;

/// Idempotency-Key header name.
pub const IDEMPOTENCY_KEY_HEADER: &str = "idempotency-key";

/// Middleware configuration.
#[derive(Clone)]
pub struct IdempotencyConfig {
    /// TTL in seconds. `None` means no expiration.
    pub ttl_secs: Option<i64>,
    /// Header name used to carry the idempotency key.
    pub header_name: String,
}

impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self {
            ttl_secs: Some(86_400), // 24h
            header_name: IDEMPOTENCY_KEY_HEADER.to_string(),
        }
    }
}

impl IdempotencyConfig {
    #[must_use] 
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl_secs = Some(ttl.as_secs() as i64);
        self
    }
}

/// Shared middleware state.
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

/// Struct-based middleware entrypoint.
#[derive(Clone)]
pub struct IdempotencyMiddleware {
    state: IdempotencyState,
}

impl IdempotencyMiddleware {
    pub fn new(store: Arc<dyn IdempotencyStore>, config: IdempotencyConfig) -> Self {
        Self {
            state: IdempotencyState::with_config(store, config),
        }
    }

    #[must_use] 
    pub fn state(&self) -> IdempotencyState {
        self.state.clone()
    }
}

async fn process_request(state: &IdempotencyState, req: Request<Body>, next: Next) -> Response {
    let idempotency_key = req
        .headers()
        .get(&state.config.header_name)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);

    let key = match idempotency_key {
        Some(k) => k,
        None => return next.run(req).await,
    };

    // Replay if already completed, reject while pending, retry if failed.
    match state.store.get(&key).await {
        Ok(Some(record)) => match record.status {
            crate::IdempotencyStatus::Completed => {
                let status_code = record
                    .response_status
                    .and_then(|s| StatusCode::from_u16(s).ok())
                    .unwrap_or(StatusCode::OK);
                return (
                    status_code,
                    [("x-idempotent-replayed", "true")],
                    record.response_body.unwrap_or_default(),
                )
                    .into_response();
            }
            crate::IdempotencyStatus::Pending => {
                return (StatusCode::CONFLICT, "request is already being processed")
                    .into_response();
            }
            crate::IdempotencyStatus::Failed => {
                let _ = state.store.delete(&key).await;
            }
        },
        Ok(None) => {}
        Err(e) => {
            tracing::warn!("idempotency store get error: {e}");
        }
    }

    let record = IdempotencyRecord::new(key.clone(), state.config.ttl_secs);
    match state.store.set(record).await {
        Ok(()) => {}
        Err(IdempotencyError::Duplicate { .. }) => {
            return (StatusCode::CONFLICT, "request is already being processed").into_response();
        }
        Err(e) => {
            tracing::warn!("idempotency store set error: {e}");
        }
    }

    let response = next.run(req).await;
    let (parts, body) = response.into_parts();
    let body_bytes = if let Ok(collected) = body.collect().await { collected.to_bytes() } else {
        let _ = state.store.mark_failed(&key, None, None).await;
        return Response::from_parts(parts, Body::empty());
    };

    let response_body = String::from_utf8_lossy(&body_bytes).to_string();
    let response_status = Some(parts.status.as_u16());

    if parts.status.is_success() {
        let _ = state
            .store
            .mark_completed(&key, Some(response_body), response_status)
            .await;
    } else {
        let _ = state
            .store
            .mark_failed(&key, Some(response_body), response_status)
            .await;
    }

    Response::from_parts(parts, Body::from(body_bytes))
}

/// Backward-compatible middleware function.
pub async fn idempotency_middleware(
    State(state): State<IdempotencyState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    process_request(&state, req, next).await
}
