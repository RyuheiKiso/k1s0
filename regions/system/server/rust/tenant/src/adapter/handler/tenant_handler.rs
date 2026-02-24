use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::usecase::{
    CreateTenantError, CreateTenantInput, CreateTenantUseCase, GetTenantError, GetTenantUseCase,
    ListTenantsError, ListTenantsUseCase,
};

#[derive(Clone)]
pub struct AppState {
    pub create_tenant_uc: Arc<CreateTenantUseCase>,
    pub get_tenant_uc: Arc<GetTenantUseCase>,
    pub list_tenants_uc: Arc<ListTenantsUseCase>,
}

// --- Request / Response DTOs ---

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub display_name: String,
    pub plan: String,
    pub owner_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TenantResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub plan: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListTenantsQuery {
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

#[derive(Debug, Serialize)]
pub struct ListTenantsResponse {
    pub tenants: Vec<TenantResponse>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

// --- Handlers ---

pub async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn readyz() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ready"}))
}

pub async fn list_tenants(
    State(state): State<AppState>,
    Query(query): Query<ListTenantsQuery>,
) -> impl IntoResponse {
    match state.list_tenants_uc.execute(query.page, query.page_size).await {
        Ok((tenants, total)) => {
            let resp = ListTenantsResponse {
                tenants: tenants
                    .into_iter()
                    .map(|t| TenantResponse {
                        id: t.id.to_string(),
                        name: t.name,
                        display_name: t.display_name,
                        status: t.status.as_str().to_string(),
                        plan: t.plan,
                        created_at: t.created_at.to_rfc3339(),
                    })
                    .collect(),
                total_count: total,
                page: query.page,
                page_size: query.page_size,
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(ListTenantsError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

pub async fn get_tenant(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let tenant_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("invalid tenant id: {}", id)})),
            )
                .into_response()
        }
    };

    match state.get_tenant_uc.execute(tenant_id).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan,
                created_at: t.created_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(GetTenantError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("tenant not found: {}", id)})),
        )
            .into_response(),
        Err(GetTenantError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

pub async fn create_tenant(
    State(state): State<AppState>,
    Json(req): Json<CreateTenantRequest>,
) -> impl IntoResponse {
    let owner_id = req
        .owner_id
        .and_then(|s| Uuid::parse_str(&s).ok());

    let input = CreateTenantInput {
        name: req.name,
        display_name: req.display_name,
        plan: req.plan,
        owner_id,
    };

    match state.create_tenant_uc.execute(input).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan,
                created_at: t.created_at.to_rfc3339(),
            };
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).unwrap()),
            )
                .into_response()
        }
        Err(CreateTenantError::NameConflict(name)) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": format!("tenant name already exists: {}", name)})),
        )
            .into_response(),
        Err(CreateTenantError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// PUT /api/v1/tenants/:id - Update tenant (stub for now, no UpdateTenantUseCase exists)
pub async fn update_tenant(
    Path(id): Path<String>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": format!("update tenant {} not yet implemented", id)})),
    )
}

/// DELETE /api/v1/tenants/:id - Delete tenant (stub for now, no DeleteTenantUseCase exists)
pub async fn delete_tenant(
    Path(id): Path<String>,
) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({"error": format!("delete tenant {} not yet implemented", id)})),
    )
}
