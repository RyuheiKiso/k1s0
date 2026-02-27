use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::SessionError;
use crate::adapter::middleware::auth::SessionAuthState;
use crate::usecase::create_session::{CreateSessionInput, CreateSessionUseCase};
use crate::usecase::get_session::{GetSessionInput, GetSessionUseCase};
use crate::usecase::list_user_sessions::{ListUserSessionsInput, ListUserSessionsUseCase};
use crate::usecase::refresh_session::{RefreshSessionInput, RefreshSessionUseCase};
use crate::usecase::revoke_all_sessions::{RevokeAllSessionsInput, RevokeAllSessionsUseCase};
use crate::usecase::revoke_session::{RevokeSessionInput, RevokeSessionUseCase};

#[derive(Clone)]
pub struct AppState {
    pub create_uc: Arc<CreateSessionUseCase>,
    pub get_uc: Arc<GetSessionUseCase>,
    pub refresh_uc: Arc<RefreshSessionUseCase>,
    pub revoke_uc: Arc<RevokeSessionUseCase>,
    pub list_uc: Arc<ListUserSessionsUseCase>,
    pub revoke_all_uc: Arc<RevokeAllSessionsUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<SessionAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: SessionAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

fn error_response(err: SessionError) -> (StatusCode, Json<serde_json::Value>) {
    let (status, message) = match &err {
        SessionError::NotFound(_) => (StatusCode::NOT_FOUND, err.to_string()),
        SessionError::Expired(_) => (StatusCode::GONE, err.to_string()),
        SessionError::Revoked(_) => (StatusCode::CONFLICT, err.to_string()),
        SessionError::InvalidInput(_) => (StatusCode::BAD_REQUEST, err.to_string()),
        SessionError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    };
    (status, Json(serde_json::json!({"error": message})))
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(input): Json<CreateSessionInput>,
) -> impl IntoResponse {
    match state.create_uc.execute(&input).await {
        Ok(output) => (StatusCode::CREATED, Json(serde_json::to_value(output).unwrap())).into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = GetSessionInput {
        id: Some(id),
        token: None,
    };
    match state.get_uc.execute(&input).await {
        Ok(output) => (StatusCode::OK, Json(serde_json::to_value(output).unwrap())).into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

pub async fn refresh_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let ttl_seconds = body
        .get("ttl_seconds")
        .and_then(|v| v.as_i64())
        .unwrap_or(3600);
    let input = RefreshSessionInput { id, ttl_seconds };
    match state.refresh_uc.execute(&input).await {
        Ok(output) => (StatusCode::OK, Json(serde_json::to_value(output).unwrap())).into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

pub async fn revoke_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let input = RevokeSessionInput { id };
    match state.revoke_uc.execute(&input).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

pub async fn list_user_sessions(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let input = ListUserSessionsInput { user_id };
    match state.list_uc.execute(&input).await {
        Ok(output) => (StatusCode::OK, Json(serde_json::to_value(output).unwrap())).into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

pub async fn revoke_all_sessions(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let input = RevokeAllSessionsInput { user_id };
    match state.revoke_all_uc.execute(&input).await {
        Ok(output) => (StatusCode::OK, Json(serde_json::to_value(output).unwrap())).into_response(),
        Err(e) => error_response(e).into_response(),
    }
}
