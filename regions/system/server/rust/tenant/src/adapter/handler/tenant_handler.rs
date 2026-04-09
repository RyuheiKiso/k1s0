use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use k1s0_server_common::error as codes;
use k1s0_server_common::ErrorResponse;

use crate::adapter::middleware::auth::AuthState;
use crate::domain::entity::Plan;
use crate::usecase::{
    ActivateTenantError, ActivateTenantUseCase, AddMemberError, AddMemberInput, AddMemberUseCase,
    CreateTenantError, CreateTenantInput, CreateTenantUseCase, DeleteTenantError,
    DeleteTenantUseCase, GetTenantError, GetTenantUseCase, ListMembersError, ListMembersUseCase,
    ListTenantsError, ListTenantsUseCase, RemoveMemberError, RemoveMemberUseCase,
    SuspendTenantError, SuspendTenantUseCase, UpdateMemberRoleError, UpdateMemberRoleInput,
    UpdateMemberRoleUseCase, UpdateTenantError, UpdateTenantInput, UpdateTenantUseCase,
};

// テナントAPIの共通エラーレスポンスヘルパー（ErrorResponseを直接Jsonで返す）
fn not_found_response(msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new(codes::tenant::not_found(), msg)),
    )
}

fn member_not_found_response(msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new(codes::tenant::member_not_found(), msg)),
    )
}

fn bad_request_response(
    code: k1s0_server_common::ErrorCode,
    msg: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(code, msg)))
}

fn conflict_response(
    code: k1s0_server_common::ErrorCode,
    msg: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (StatusCode::CONFLICT, Json(ErrorResponse::new(code, msg)))
}

fn internal_response(msg: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(codes::tenant::internal_error(), msg)),
    )
}

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
    pub update_member_role_uc: Arc<UpdateMemberRoleUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<AuthState>,
    pub db_pool: Option<Arc<sqlx::PgPool>>,
    pub kafka_brokers: Option<Vec<String>>,
    pub keycloak_health_url: Option<String>,
    pub http_client: reqwest::Client,
}

impl AppState {
    #[must_use]
    pub fn with_auth(mut self, auth_state: AuthState) -> Self {
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
    #[serde(alias = "owner_user_id")]
    pub owner_id: String,
}

#[derive(Debug, Serialize)]
pub struct TenantResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub plan: String,
    pub owner_id: Option<String>,
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
pub struct UpdateMemberRoleRequest {
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

/// テナント一覧レスポンス。
/// LOW-10 確認: ページネーション形式は他サービスと統一されている。
/// auth サーバーの Pagination `構造体（total_count`, page, `page_size`, `has_next）と同一形式`。
/// TODO(future-work): cursor `ベースのページネーション（after_cursor` フィールド等）への移行を検討すること。
///   大規模データ取得時に OFFSET ベースではパフォーマンス劣化が発生するため、
///   keyset ページネーション化が推奨される（vault の `list_audit_logs` と同様の課題）。
///   優先度: LOW。対応時は DB クエリ・API レスポンス・クライアント側の変更が必要となるため ADR を作成すること。
#[derive(Debug, Serialize)]
pub struct ListTenantsResponse {
    pub tenants: Vec<TenantResponse>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

// --- Handlers ---

pub async fn healthz(State(state): State<AppState>) -> impl IntoResponse {
    // INFRA-03 監査対応: DB 接続確認を追加し、DB 障害時は 503 を返す
    if let Some(pool) = &state.db_pool {
        match sqlx::query("SELECT 1").execute(pool.as_ref()).await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!(error = %e, "DB ヘルスチェックに失敗しました");
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({"status": "error", "service": "tenant", "detail": "database unavailable"})),
                )
                    .into_response();
            }
        }
    }
    Json(serde_json::json!({"status": "ok", "service": "tenant"})).into_response()
}

pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let mut checks = serde_json::Map::new();
    let mut is_ready = true;

    if let Some(pool) = &state.db_pool {
        // MED-006 監査対応: SELECT 1 ではなくスキーマ固有テーブルを参照して
        // マイグレーション未実行でも ready を返す誤検知を防ぐ（ADR-0068 準拠）
        match sqlx::query("SELECT 1 FROM tenant.tenants LIMIT 0")
            .execute(pool.as_ref())
            .await
        {
            Ok(_) => {
                checks.insert("database".to_string(), serde_json::json!("ok"));
            }
            Err(e) => {
                // M-001 監査対応: readyz に内部エラー詳細（DB接続文字列等）を露出しない
                tracing::warn!(error = %e, "readyz database check failed");
                checks.insert("database".to_string(), serde_json::json!("error"));
                is_ready = false;
            }
        }
    } else {
        checks.insert("database".to_string(), serde_json::json!("not_configured"));
    }

    if let Some(brokers) = &state.kafka_brokers {
        use rdkafka::config::ClientConfig;
        use rdkafka::consumer::{BaseConsumer, Consumer};

        let consumer = ClientConfig::new()
            .set("bootstrap.servers", brokers.join(","))
            .set("group.id", "tenant-readyz-check")
            .set("enable.auto.commit", "false")
            .create::<BaseConsumer>();

        match consumer {
            Ok(c) => match c.fetch_metadata(None, std::time::Duration::from_secs(2)) {
                Ok(_) => {
                    checks.insert("kafka".to_string(), serde_json::json!("ok"));
                }
                Err(e) => {
                    // M-001 監査対応: readyz に内部エラー詳細（Kafka接続文字列等）を露出しない
                    tracing::warn!(error = %e, "readyz kafka metadata check failed");
                    checks.insert("kafka".to_string(), serde_json::json!("error"));
                    is_ready = false;
                }
            },
            Err(e) => {
                // M-001 監査対応: readyz に内部エラー詳細（Kafka設定等）を露出しない
                tracing::warn!(error = %e, "readyz kafka consumer creation failed");
                checks.insert("kafka".to_string(), serde_json::json!("error"));
                is_ready = false;
            }
        }
    } else {
        checks.insert("kafka".to_string(), serde_json::json!("not_configured"));
    }

    if let Some(url) = &state.keycloak_health_url {
        match state.http_client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                checks.insert("keycloak".to_string(), serde_json::json!("ok"));
            }
            Ok(resp) => {
                checks.insert(
                    "keycloak".to_string(),
                    serde_json::json!(format!("status {}", resp.status())),
                );
                is_ready = false;
            }
            Err(e) => {
                // M-001 監査対応: readyz に内部エラー詳細（Keycloak URL等）を露出しない
                tracing::warn!(error = %e, "readyz keycloak health check failed");
                checks.insert("keycloak".to_string(), serde_json::json!("error"));
                is_ready = false;
            }
        }
    } else {
        checks.insert("keycloak".to_string(), serde_json::json!("not_configured"));
    }

    // ADR-0068 準拠: status は healthy / unhealthy の2値を使用する（MED-006 監査対応）
    let status = if is_ready { "healthy" } else { "unhealthy" };
    let code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        code,
        Json(serde_json::json!({
            "status": status,
            "checks": checks,
        })),
    )
}

pub async fn list_tenants(
    State(state): State<AppState>,
    Query(query): Query<ListTenantsQuery>,
) -> impl IntoResponse {
    match state
        .list_tenants_uc
        .execute(query.page, query.page_size)
        .await
    {
        Ok((tenants, total)) => {
            let has_next = i64::from(query.page) * i64::from(query.page_size) < total;
            let resp = ListTenantsResponse {
                tenants: tenants
                    .into_iter()
                    .map(|t| TenantResponse {
                        id: t.id.to_string(),
                        name: t.name,
                        display_name: t.display_name,
                        status: t.status.as_str().to_string(),
                        plan: t.plan.as_str().to_string(),
                        owner_id: t.owner_id,
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
                has_next,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(ListTenantsError::Internal(msg)) => {
            (StatusCode::INTERNAL_SERVER_ERROR, internal_response(msg)).into_response()
        }
    }
}

pub async fn get_tenant(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // UUIDパース失敗時は即座に400エラーを返す
    let Ok(tenant_id) = Uuid::parse_str(&id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid tenant id: {id}"),
        )
        .into_response();
    };

    match state.get_tenant_uc.execute(tenant_id).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan.as_str().to_string(),
                owner_id: t.owner_id,
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(GetTenantError::NotFound(_)) => {
            not_found_response(format!("tenant not found: {id}")).into_response()
        }
        Err(GetTenantError::Internal(msg)) => internal_response(msg).into_response(),
    }
}

pub async fn create_tenant(
    State(state): State<AppState>,
    Json(req): Json<CreateTenantRequest>,
) -> impl IntoResponse {
    // オーナーIDのUUIDパース失敗時は400エラーを返す
    let Ok(owner_uuid) = Uuid::parse_str(&req.owner_id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid owner id: {}", req.owner_id),
        )
        .into_response();
    };
    let owner_id = Some(owner_uuid);

    // プランの文字列パース失敗時は400エラーを返す
    let Ok(plan) = req.plan.parse::<Plan>() else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid plan: {}", req.plan),
        )
        .into_response();
    };

    let input = CreateTenantInput {
        name: req.name,
        display_name: req.display_name,
        plan,
        owner_id,
    };

    match state.create_tenant_uc.execute(input).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan.as_str().to_string(),
                owner_id: t.owner_id,
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            };
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(CreateTenantError::NameConflict(name)) => conflict_response(
            codes::tenant::name_conflict(),
            format!("tenant name already exists: {name}"),
        )
        .into_response(),
        Err(CreateTenantError::Internal(msg)) => internal_response(msg).into_response(),
    }
}

/// PUT /api/v1/tenants/:id
pub async fn update_tenant(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTenantRequest>,
) -> impl IntoResponse {
    // UUIDパース失敗時は即座に400エラーを返す
    let Ok(tenant_id) = Uuid::parse_str(&id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid tenant id: {id}"),
        )
        .into_response();
    };

    // プランの文字列パース失敗時は400エラーを返す
    let Ok(plan) = req.plan.parse::<Plan>() else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid plan: {}", req.plan),
        )
        .into_response();
    };

    let input = UpdateTenantInput {
        id: tenant_id,
        display_name: req.display_name,
        plan,
    };

    match state.update_tenant_uc.execute(input).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan.as_str().to_string(),
                owner_id: t.owner_id,
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(UpdateTenantError::NotFound(_)) => {
            not_found_response(format!("tenant not found: {id}")).into_response()
        }
        Err(UpdateTenantError::InvalidStatus(msg)) => {
            bad_request_response(codes::tenant::invalid_status(), msg).into_response()
        }
        Err(UpdateTenantError::Internal(msg)) => internal_response(msg).into_response(),
    }
}

/// DELETE /api/v1/tenants/:id
pub async fn delete_tenant(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // UUIDパース失敗時は即座に400エラーを返す
    let Ok(tenant_id) = Uuid::parse_str(&id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid tenant id: {id}"),
        )
        .into_response();
    };

    match state.delete_tenant_uc.execute(tenant_id).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan.as_str().to_string(),
                owner_id: t.owner_id,
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(DeleteTenantError::NotFound(_)) => {
            not_found_response(format!("tenant not found: {id}")).into_response()
        }
        Err(DeleteTenantError::InvalidStatus(msg)) => {
            bad_request_response(codes::tenant::invalid_status(), msg).into_response()
        }
        Err(DeleteTenantError::Internal(msg)) => internal_response(msg).into_response(),
    }
}

/// POST /api/v1/tenants/:id/suspend
pub async fn suspend_tenant(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // UUIDパース失敗時は即座に400エラーを返す
    let Ok(tenant_id) = Uuid::parse_str(&id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid tenant id: {id}"),
        )
        .into_response();
    };

    match state.suspend_tenant_uc.execute(tenant_id).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan.as_str().to_string(),
                owner_id: t.owner_id,
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(SuspendTenantError::NotFound(_)) => {
            not_found_response(format!("tenant not found: {id}")).into_response()
        }
        Err(SuspendTenantError::InvalidStatus(msg)) => {
            bad_request_response(codes::tenant::invalid_status(), msg).into_response()
        }
        Err(SuspendTenantError::Internal(msg)) => internal_response(msg).into_response(),
    }
}

/// POST /api/v1/tenants/:id/activate
pub async fn activate_tenant(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // UUIDパース失敗時は即座に400エラーを返す
    let Ok(tenant_id) = Uuid::parse_str(&id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid tenant id: {id}"),
        )
        .into_response();
    };

    match state.activate_tenant_uc.execute(tenant_id).await {
        Ok(t) => {
            let resp = TenantResponse {
                id: t.id.to_string(),
                name: t.name,
                display_name: t.display_name,
                status: t.status.as_str().to_string(),
                plan: t.plan.as_str().to_string(),
                owner_id: t.owner_id,
                settings: t.settings,
                keycloak_realm: t.keycloak_realm,
                db_schema: t.db_schema,
                created_at: t.created_at.to_rfc3339(),
                updated_at: t.updated_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(ActivateTenantError::NotFound(_)) => {
            not_found_response(format!("tenant not found: {id}")).into_response()
        }
        Err(ActivateTenantError::InvalidStatus(msg)) => {
            bad_request_response(codes::tenant::invalid_status(), msg).into_response()
        }
        Err(ActivateTenantError::Internal(msg)) => internal_response(msg).into_response(),
    }
}

/// GET /api/v1/tenants/:id/members
pub async fn list_members(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // UUIDパース失敗時は即座に400エラーを返す
    let Ok(tenant_id) = Uuid::parse_str(&id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid tenant id: {id}"),
        )
        .into_response();
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
        Err(ListMembersError::NotFound(_)) => {
            not_found_response(format!("tenant not found: {id}")).into_response()
        }
        Err(ListMembersError::Internal(msg)) => internal_response(msg).into_response(),
    }
}

/// POST /api/v1/tenants/:id/members
pub async fn add_member(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> impl IntoResponse {
    // UUIDパース失敗時は即座に400エラーを返す
    let Ok(tenant_id) = Uuid::parse_str(&id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid tenant id: {id}"),
        )
        .into_response();
    };

    // ユーザーIDのUUIDパース失敗時は400エラーを返す
    let Ok(user_id) = Uuid::parse_str(&req.user_id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid user id: {}", req.user_id),
        )
        .into_response();
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
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(AddMemberError::AlreadyMember) => {
            conflict_response(codes::tenant::member_conflict(), "member already exists")
                .into_response()
        }
        Err(AddMemberError::Internal(msg)) => internal_response(msg).into_response(),
    }
}

/// DELETE /`api/v1/tenants/:tenant_id/members/:user_id`
pub async fn remove_member(
    State(state): State<AppState>,
    Path((tenant_id, user_id)): Path<(String, String)>,
) -> impl IntoResponse {
    // UUIDパース失敗時は即座に400エラーを返す
    let Ok(tenant_uuid) = Uuid::parse_str(&tenant_id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid tenant id: {tenant_id}"),
        )
        .into_response();
    };

    // ユーザーIDのUUIDパース失敗時は400エラーを返す
    let Ok(user_uuid) = Uuid::parse_str(&user_id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid user id: {user_id}"),
        )
        .into_response();
    };

    match state.remove_member_uc.execute(tenant_uuid, user_uuid).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(RemoveMemberError::NotFound) => {
            member_not_found_response("member not found").into_response()
        }
        Err(RemoveMemberError::Internal(msg)) => internal_response(msg).into_response(),
    }
}

/// PUT /`api/v1/tenants/:tenant_id/members/:user_id`
pub async fn update_member_role(
    State(state): State<AppState>,
    Path((tenant_id, user_id)): Path<(String, String)>,
    Json(req): Json<UpdateMemberRoleRequest>,
) -> impl IntoResponse {
    // UUIDパース失敗時は即座に400エラーを返す
    let Ok(tenant_uuid) = Uuid::parse_str(&tenant_id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid tenant id: {tenant_id}"),
        )
        .into_response();
    };

    // ユーザーIDのUUIDパース失敗時は400エラーを返す
    let Ok(user_uuid) = Uuid::parse_str(&user_id) else {
        return bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid user id: {user_id}"),
        )
        .into_response();
    };

    let input = UpdateMemberRoleInput {
        tenant_id: tenant_uuid,
        user_id: user_uuid,
        role: req.role,
    };

    match state.update_member_role_uc.execute(input).await {
        Ok(member) => {
            let resp = MemberResponse {
                id: member.id.to_string(),
                tenant_id: member.tenant_id.to_string(),
                user_id: member.user_id.to_string(),
                role: member.role,
                joined_at: member.joined_at.to_rfc3339(),
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(UpdateMemberRoleError::NotFound) => {
            member_not_found_response("member not found").into_response()
        }
        Err(UpdateMemberRoleError::TenantNotFound) => {
            not_found_response("tenant not found").into_response()
        }
        Err(UpdateMemberRoleError::InvalidRole(role)) => bad_request_response(
            codes::tenant::validation_error(),
            format!("invalid role: {role}"),
        )
        .into_response(),
        Err(UpdateMemberRoleError::Internal(msg)) => internal_response(msg).into_response(),
    }
}
