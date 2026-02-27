use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::adapter::middleware::auth::TenantAuthState;
use crate::usecase::{
    ActivateTenantError, ActivateTenantUseCase, AddMemberError, AddMemberInput, AddMemberUseCase,
    CreateTenantError, CreateTenantInput, CreateTenantUseCase, DeleteTenantError,
    DeleteTenantUseCase, GetTenantError, GetTenantUseCase, ListMembersError, ListMembersUseCase,
    ListTenantsError, ListTenantsUseCase, RemoveMemberError, RemoveMemberUseCase,
    SuspendTenantError, SuspendTenantUseCase, UpdateTenantError, UpdateTenantInput,
    UpdateTenantUseCase,
};

#[derive(Clone)]
pub struct AppState {
    pub create_tenant_uc: Arc<CreateTenantUseCase>,
    pub get_tenant_uc: Arc<GetTenantUseCase>,
    pub list_tenants_uc: Arc<ListTenantsUseCase>,
    pub update_tenant_uc: Arc<UpdateTenantUseCase>,
    pub delete_tenant_uc: Arc<DeleteTenantUseCase>,
    pub suspend_tenant_uc: Arc<SuspendTenantUseCase>,
    pub activate_tenant_uc: Arc<ActivateTenantUseCase>,
    pub list_members_uc: Arc<ListMembersUseCase>,
    pub add_member_uc: Arc<AddMemberUseCase>,
    pub remove_member_uc: Arc<RemoveMemberUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<TenantAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: TenantAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
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
    pub settings: serde_json::Value,
    pub keycloak_realm: Option<String>,
    pub db_schema: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    pub display_name: String,
    pub plan: String,
}

#[derive(Debug, Serialize)]
pub struct MemberResponse {
    pub id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub role: String,
    pub joined_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub user_id: String,
    pub role: String,
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
                        settings: t.settings,
                        keycloak_realm: t.keycloak_realm,
                        db_schema: t.db_schema,
                        created_at: t.created_at.to_rfc3339(),
                        updated_at: t.updated_at.to_rfc3339(),
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
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
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
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
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

/// PUT /api/v1/tenants/:id
pub async fn update_tenant(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTenantRequest>,
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

    let input = UpdateTenantInput {
        id: tenant_id,
        display_name: req.display_name,
        plan: req.plan,
    };

    match state.update_tenant_uc.execute(input).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan,
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(UpdateTenantError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("tenant not found: {}", id)})),
        )
            .into_response(),
        Err(UpdateTenantError::InvalidStatus(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
        Err(UpdateTenantError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/tenants/:id
pub async fn delete_tenant(
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

    match state.delete_tenant_uc.execute(tenant_id).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan,
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(DeleteTenantError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("tenant not found: {}", id)})),
        )
            .into_response(),
        Err(DeleteTenantError::InvalidStatus(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
        Err(DeleteTenantError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// POST /api/v1/tenants/:id/suspend
pub async fn suspend_tenant(
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

    match state.suspend_tenant_uc.execute(tenant_id).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan,
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(SuspendTenantError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("tenant not found: {}", id)})),
        )
            .into_response(),
        Err(SuspendTenantError::InvalidStatus(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
        Err(SuspendTenantError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// POST /api/v1/tenants/:id/activate
pub async fn activate_tenant(
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

    match state.activate_tenant_uc.execute(tenant_id).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan,
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(ActivateTenantError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("tenant not found: {}", id)})),
        )
            .into_response(),
        Err(ActivateTenantError::InvalidStatus(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
        Err(ActivateTenantError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// GET /api/v1/tenants/:id/members
pub async fn list_members(
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

    match state.list_members_uc.execute(tenant_id).await {
        Ok(members) => {
            let resp: Vec<MemberResponse> = members
                .into_iter()
                .map(|m| MemberResponse {
                    id: m.id.to_string(),
                    tenant_id: m.tenant_id.to_string(),
                    user_id: m.user_id.to_string(),
                    role: m.role,
                    joined_at: m.joined_at.to_rfc3339(),
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({"members": resp}))).into_response()
        }
        Err(ListMembersError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("tenant not found: {}", id)})),
        )
            .into_response(),
        Err(ListMembersError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// POST /api/v1/tenants/:id/members
pub async fn add_member(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AddMemberRequest>,
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

    let user_id = match Uuid::parse_str(&req.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("invalid user id: {}", req.user_id)})),
            )
                .into_response()
        }
    };

    let input = AddMemberInput {
        tenant_id,
        user_id,
        role: req.role,
    };

    match state.add_member_uc.execute(input).await {
        Ok(member) => {
            let resp = MemberResponse {
                id: member.id.to_string(),
                tenant_id: member.tenant_id.to_string(),
                user_id: member.user_id.to_string(),
                role: member.role,
                joined_at: member.joined_at.to_rfc3339(),
            };
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).unwrap()),
            )
                .into_response()
        }
        Err(AddMemberError::AlreadyMember) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": "member already exists"})),
        )
            .into_response(),
        Err(AddMemberError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}

/// DELETE /api/v1/tenants/:tenant_id/members/:user_id
pub async fn remove_member(
    State(state): State<AppState>,
    Path((tenant_id, user_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let tenant_uuid = match Uuid::parse_str(&tenant_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("invalid tenant id: {}", tenant_id)})),
            )
                .into_response()
        }
    };

    let user_uuid = match Uuid::parse_str(&user_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": format!("invalid user id: {}", user_id)})),
            )
                .into_response()
        }
    };

    match state.remove_member_uc.execute(tenant_uuid, user_uuid).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(RemoveMemberError::NotFound) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "member not found"})),
        )
            .into_response(),
        Err(RemoveMemberError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}
