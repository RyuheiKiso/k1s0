use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{AppState, ErrorResponse};
#[allow(unused_imports)]
use crate::domain::entity::service::Service;
use crate::domain::entity::service::{ServiceLifecycle, ServiceTier};
use crate::domain::repository::service_repository::ServiceListFilters;
use crate::usecase::register_service::RegisterServiceInput;
use crate::usecase::update_service::UpdateServiceInput;

/// サービス一覧取得のクエリパラメータ。
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListServicesQuery {
    pub team_id: Option<Uuid>,
    pub tier: Option<String>,
    pub lifecycle: Option<String>,
    pub tag: Option<String>,
}

#[utoipa::path(
    get,
    path = "/healthz",
    responses(
        (status = 200, description = "Health check OK"),
    )
)]
pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

#[utoipa::path(
    get,
    path = "/readyz",
    responses(
        (status = 200, description = "Ready"),
        (status = 503, description = "Not ready"),
    )
)]
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let mut db_status = "skipped";
    let mut overall_ok = true;

    if let Some(ref pool) = state.db_pool {
        match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => db_status = "ok",
            Err(_) => {
                db_status = "error";
                overall_ok = false;
            }
        }
    }

    let status_code = if overall_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status_code,
        Json(serde_json::json!({
            "status": if overall_ok { "ready" } else { "not ready" },
            "checks": {
                "database": db_status
            }
        })),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics"),
    )
)]
pub async fn metrics_endpoint(State(state): State<AppState>) -> impl IntoResponse {
    let body = state.metrics.gather_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/services",
    params(
        ("team_id" = Option<Uuid>, Query, description = "Filter by team ID"),
        ("tier" = Option<String>, Query, description = "Filter by tier"),
        ("lifecycle" = Option<String>, Query, description = "Filter by lifecycle"),
        ("tag" = Option<String>, Query, description = "Filter by tag"),
    ),
    responses(
        (status = 200, description = "Service list", body = Vec<Service>),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_services(
    State(state): State<AppState>,
    Query(params): Query<ListServicesQuery>,
) -> impl IntoResponse {
    let tier = params.tier.and_then(|t| t.parse::<ServiceTier>().ok());
    let lifecycle = params
        .lifecycle
        .and_then(|l| l.parse::<ServiceLifecycle>().ok());

    let filters = ServiceListFilters {
        team_id: params.team_id,
        tier,
        lifecycle,
        tag: params.tag,
    };

    match state.list_services_uc.execute(filters).await {
        Ok(services) => (StatusCode::OK, Json(services)).into_response(),
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/services/{id}",
    params(("id" = Uuid, Path, description = "Service ID")),
    responses(
        (status = 200, description = "Service found", body = Service),
        (status = 404, description = "Service not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_service(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    match state.get_service_uc.execute(id).await {
        Ok(service) => (StatusCode::OK, Json(service)).into_response(),
        Err(crate::usecase::get_service::GetServiceError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_SCAT_001", "The specified service was not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/services",
    request_body = RegisterServiceInput,
    responses(
        (status = 201, description = "Service registered", body = Service),
        (status = 400, description = "Invalid input"),
        (status = 404, description = "Team not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn register_service(
    State(state): State<AppState>,
    Json(input): Json<RegisterServiceInput>,
) -> impl IntoResponse {
    match state.register_service_uc.execute(input).await {
        Ok(service) => (StatusCode::CREATED, Json(service)).into_response(),
        Err(crate::usecase::register_service::RegisterServiceError::TeamNotFound(_)) => {
            let err = ErrorResponse::new("SYS_SCAT_001", "The specified team was not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::register_service::RegisterServiceError::InvalidInput(msg)) => {
            let err = ErrorResponse::new("SYS_SCAT_002", msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/services/{id}",
    params(("id" = Uuid, Path, description = "Service ID")),
    request_body = UpdateServiceInput,
    responses(
        (status = 200, description = "Service updated", body = Service),
        (status = 400, description = "Invalid input"),
        (status = 404, description = "Service not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_service(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateServiceInput>,
) -> impl IntoResponse {
    match state.update_service_uc.execute(id, input).await {
        Ok(service) => (StatusCode::OK, Json(service)).into_response(),
        Err(crate::usecase::update_service::UpdateServiceError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_SCAT_001", "The specified service was not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::update_service::UpdateServiceError::InvalidInput(msg)) => {
            let err = ErrorResponse::new("SYS_SCAT_002", msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/services/{id}",
    params(("id" = Uuid, Path, description = "Service ID")),
    responses(
        (status = 204, description = "Service deleted"),
        (status = 404, description = "Service not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_service(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.delete_service_uc.execute(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(crate::usecase::delete_service::DeleteServiceError::NotFound(_)) => {
            let err = ErrorResponse::new("SYS_SCAT_001", "The specified service was not found");
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_SCAT_005", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
