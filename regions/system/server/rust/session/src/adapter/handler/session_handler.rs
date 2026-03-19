use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use axum::Json;

use k1s0_server_common::error as codes;
use k1s0_server_common::ErrorResponse;

use crate::adapter::middleware::auth::AuthState;
use crate::adapter::repository::session_metadata_postgres::SessionMetadataRepository;
use crate::domain::entity::session::Session;
use crate::error::SessionError;
use crate::infrastructure::kafka_producer::SessionEventPublisher;
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
    pub metadata_repo: Arc<dyn SessionMetadataRepository>,
    pub event_publisher: Arc<dyn SessionEventPublisher>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<AuthState>,
    /// Redis が構成されているかを示すフラグ。health エンドポイントで使用。
    pub redis_configured: bool,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: AuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

// セッションエラーをHTTPレスポンスに変換するヘルパー（ErrorResponseを直接Jsonで返す）
fn error_response(err: SessionError) -> (StatusCode, Json<ErrorResponse>) {
    let (status, code, message) = match &err {
        SessionError::NotFound(_) => (
            StatusCode::NOT_FOUND,
            codes::session::not_found(),
            err.to_string(),
        ),
        SessionError::Expired(_) => (StatusCode::GONE, codes::session::expired(), err.to_string()),
        SessionError::Revoked(_) | SessionError::AlreadyRevoked(_) => (
            StatusCode::CONFLICT,
            codes::session::already_revoked(),
            err.to_string(),
        ),
        SessionError::InvalidInput(_) => (
            StatusCode::BAD_REQUEST,
            codes::session::validation_error(),
            err.to_string(),
        ),
        SessionError::TooManySessions(_) => (
            StatusCode::TOO_MANY_REQUESTS,
            codes::session::max_devices_exceeded(),
            err.to_string(),
        ),
        SessionError::Internal(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            codes::session::internal_error(),
            err.to_string(),
        ),
    };
    (status, Json(ErrorResponse::new(code, message)))
}

// 認可エラーレスポンスヘルパー
fn forbidden_response(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse::new(codes::session::forbidden(), message)),
    )
}

pub async fn create_session(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Json(input): Json<CreateSessionHttpRequest>,
) -> impl IntoResponse {
    if let Some(Extension(claims)) = claims {
        if claims.sub != input.user_id {
            return forbidden_response("cannot create session for another user").into_response();
        }
    }

    let uc_input = CreateSessionInput {
        user_id: input.user_id,
        device_id: input.device_id,
        device_name: input.device_name,
        device_type: input.device_type,
        user_agent: input.user_agent,
        ip_address: input.ip_address,
        ttl_seconds: input.ttl_seconds.map(i64::from),
        max_devices: input.max_devices,
        metadata: input.metadata,
    };

    match state.create_uc.execute(&uc_input).await {
        Ok(output) => (
            StatusCode::CREATED,
            Json(SessionHttpResponse::from_session(output.session)),
        )
            .into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateSessionHttpRequest {
    pub user_id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub ttl_seconds: Option<u32>,
    pub max_devices: Option<u32>,
    pub metadata: Option<HashMap<String, String>>,
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let input = GetSessionInput {
        id: Some(session_id),
        token: None,
    };
    match state.get_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(SessionHttpResponse::from_session(output.session)),
        )
            .into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

pub async fn refresh_session(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(session_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(Extension(claims)) = claims {
        let lookup = GetSessionInput {
            id: Some(session_id.clone()),
            token: None,
        };
        match state.get_uc.execute(&lookup).await {
            Ok(output) => {
                if output.session.user_id != claims.sub {
                    return forbidden_response("cannot refresh another user's session")
                        .into_response();
                }
            }
            Err(e) => return error_response(e).into_response(),
        }
    }

    let ttl_seconds = body
        .get("ttl_seconds")
        .and_then(|v| v.as_i64())
        .unwrap_or(3600);
    let input = RefreshSessionInput {
        id: session_id,
        ttl_seconds,
    };
    match state.refresh_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(SessionHttpResponse::from_session(output.session)),
        )
            .into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

pub async fn revoke_session(
    State(state): State<AppState>,
    claims: Option<Extension<k1s0_auth::Claims>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    if let Some(Extension(claims)) = claims {
        let lookup = GetSessionInput {
            id: Some(session_id.clone()),
            token: None,
        };
        match state.get_uc.execute(&lookup).await {
            Ok(output) => {
                if output.session.user_id != claims.sub {
                    return forbidden_response("cannot revoke another user's session")
                        .into_response();
                }
            }
            Err(e) => return error_response(e).into_response(),
        }
    }

    let input = RevokeSessionInput { id: session_id };
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
        Ok(output) => {
            let total_count = output.sessions.len() as u32;
            let mapped = ListSessionsHttpResponse {
                sessions: output
                    .sessions
                    .into_iter()
                    .map(SessionHttpResponse::from_session)
                    .collect(),
                total_count,
            };
            (StatusCode::OK, Json(mapped)).into_response()
        }
        Err(e) => error_response(e).into_response(),
    }
}

pub async fn revoke_all_sessions(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let input = RevokeAllSessionsInput { user_id };
    match state.revoke_all_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "revoked_count": output.count
            })),
        )
            .into_response(),
        Err(e) => error_response(e).into_response(),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionHttpResponse {
    pub session_id: String,
    pub user_id: String,
    pub device_id: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub metadata: HashMap<String, String>,
}

impl SessionHttpResponse {
    fn from_session(session: Session) -> Self {
        Self {
            session_id: session.id,
            user_id: session.user_id,
            device_id: session.device_id,
            device_name: session.device_name,
            device_type: session.device_type,
            user_agent: session.user_agent,
            ip_address: session.ip_address,
            token: session.token,
            expires_at: session.expires_at,
            created_at: session.created_at,
            last_accessed_at: session.last_accessed_at,
            status: if session.revoked {
                "revoked".to_string()
            } else {
                "active".to_string()
            },
            metadata: session.metadata,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ListSessionsHttpResponse {
    pub sessions: Vec<SessionHttpResponse>,
    pub total_count: u32,
}
