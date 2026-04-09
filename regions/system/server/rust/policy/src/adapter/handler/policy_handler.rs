use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use k1s0_auth::Claims;

use super::AppState;
use crate::usecase::create_policy::CreatePolicyInput;
use crate::usecase::evaluate_policy::EvaluatePolicyInput;
use crate::usecase::list_policies::ListPoliciesInput;
use crate::usecase::update_policy::UpdatePolicyInput;

/// CRIT-005 対応: Option<Extension<Claims>> からテナント ID を抽出するヘルパー関数。
/// Claims が存在しない場合（認証なし環境）はデフォルト値 "system" を返す。
// Option<&T> の方が &Option<T> よりも慣用的（Clippy: ref_option）
fn extract_tenant_id(claims: Option<&Extension<Claims>>) -> String {
    claims
        .map_or_else(|| "system".to_string(), |ext| ext.tenant_id().to_string())
}

/// GET /api/v1/policies
pub async fn list_policies(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Query(params): Query<ListPoliciesParams>,
) -> impl IntoResponse {
    // CRIT-005 対応: JWT Claims からテナント ID を取得してユースケースに渡す。
    let tenant_id = extract_tenant_id(claims.as_ref());

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);
    let enabled_only = params.enabled_only.unwrap_or(false);
    let bundle_id = if let Some(bundle_id) = params.bundle_id {
        if let Ok(id) = Uuid::parse_str(&bundle_id) {
            Some(id)
        } else {
            let err = ErrorResponse::with_details(
                "SYS_POLICY_INVALID_BUNDLE_ID",
                "invalid bundle_id format",
                vec![ErrorDetail {
                    field: "bundle_id".to_string(),
                    message: "must be a valid UUID".to_string(),
                }],
            );
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    } else {
        None
    };

    let input = ListPoliciesInput {
        page,
        page_size,
        bundle_id,
        enabled_only,
        tenant_id,
    };

    match state.list_policies_uc.execute(&input).await {
        Ok(output) => {
            let items: Vec<PolicyResponse> = output
                .policies
                .into_iter()
                .map(PolicyResponse::from)
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "policies": items,
                    "pagination": {
                        "total_count": output.total_count,
                        "page": output.page,
                        "page_size": output.page_size,
                        "has_next": output.has_next
                    }
                })),
            )
                .into_response()
        }
        Err(crate::usecase::list_policies::ListPoliciesError::Internal(msg)) => {
            // H-022 監査対応: 内部エラー詳細をレスポンスに含めない（ログには記録する）
            tracing::error!("policy_handler internal error: {msg}");
            let err = ErrorResponse::new("SYS_POLICY_INTERNAL_ERROR", "Internal server error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/policies/:id
pub async fn get_policy(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // CRIT-005 対応: JWT Claims からテナント ID を取得してユースケースに渡す。
    let tenant_id = extract_tenant_id(claims.as_ref());

    match state.get_policy_uc.execute(&id, &tenant_id).await {
        Ok(Some(policy)) => {
            let resp = PolicyResponse::from(policy);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Ok(None) => {
            let err =
                ErrorResponse::new("SYS_POLICY_NOT_FOUND", &format!("policy not found: {id}"));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            // H-022 監査対応: 内部エラー詳細をレスポンスに含めない
            tracing::error!("policy_handler internal error: {e:?}");
            let err = ErrorResponse::new("SYS_POLICY_INTERNAL_ERROR", "Internal server error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/policies
pub async fn create_policy(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(req): Json<CreatePolicyRequest>,
) -> impl IntoResponse {
    // CRIT-005 対応: JWT Claims からテナント ID を取得してユースケースに渡す。
    let tenant_id = extract_tenant_id(claims.as_ref());

    let CreatePolicyRequest {
        name,
        description,
        rego_content,
        package_path,
        bundle_id: bundle_id_raw,
    } = req;

    let bundle_id = match bundle_id_raw {
        Some(bundle_id) => {
            if let Ok(id) = Uuid::parse_str(&bundle_id) {
                Some(id)
            } else {
                let err = ErrorResponse::with_details(
                    "SYS_POLICY_INVALID_BUNDLE_ID",
                    "invalid bundle_id format",
                    vec![ErrorDetail {
                        field: "bundle_id".to_string(),
                        message: "must be a valid UUID".to_string(),
                    }],
                );
                return (StatusCode::BAD_REQUEST, Json(err)).into_response();
            }
        }
        None => None,
    };

    let input = CreatePolicyInput {
        name,
        description,
        rego_content,
        package_path,
        bundle_id,
        tenant_id,
    };

    match state.create_policy_uc.execute(&input).await {
        Ok(policy) => {
            let resp = PolicyResponse::from(policy);
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(crate::usecase::create_policy::CreatePolicyError::AlreadyExists(name)) => {
            let err = ErrorResponse::new(
                "SYS_POLICY_ALREADY_EXISTS",
                &format!("policy already exists: {name}"),
            );
            (StatusCode::CONFLICT, Json(err)).into_response()
        }
        Err(crate::usecase::create_policy::CreatePolicyError::Validation(msg)) => {
            let err = ErrorResponse::new("SYS_POLICY_VALIDATION_ERROR", &msg);
            (StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
        Err(crate::usecase::create_policy::CreatePolicyError::Internal(msg)) => {
            // H-022 監査対応: 内部エラー詳細をレスポンスに含めない（ログには記録する）
            tracing::error!("policy_handler internal error: {msg}");
            let err = ErrorResponse::new("SYS_POLICY_INTERNAL_ERROR", "Internal server error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// PUT /api/v1/policies/:id
pub async fn update_policy(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePolicyRequest>,
) -> impl IntoResponse {
    // CRIT-005 対応: JWT Claims からテナント ID を取得してユースケースに渡す。
    let tenant_id = extract_tenant_id(claims.as_ref());

    let input = UpdatePolicyInput {
        id,
        description: req.description,
        rego_content: req.rego_content,
        enabled: req.enabled,
        tenant_id,
    };

    match state.update_policy_uc.execute(&input).await {
        Ok(policy) => {
            let resp = PolicyResponse::from(policy);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => {
            // TODO(future-work): 型付きエラー enum への移行が望ましい（現状は文字列マッチングで代替）。
            // 優先度: LOW。domain::error::PolicyError enum を定義し、use_case から伝播させること。
            // 対応時は全ハンドラの Err(e) パターンを統一的に修正し、ADR を作成すること。
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_POLICY_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                // H-022 監査対応: 内部エラー詳細をレスポンスに含めない
                tracing::error!("policy_handler update internal error: {msg}");
                let err = ErrorResponse::new("SYS_POLICY_INTERNAL_ERROR", "Internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// DELETE /api/v1/policies/:id
pub async fn delete_policy(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    use crate::usecase::delete_policy::DeletePolicyError;

    // CRIT-005 対応: JWT Claims からテナント ID を取得してユースケースに渡す。
    let tenant_id = extract_tenant_id(claims.as_ref());

    match state.delete_policy_uc.execute(&id, &tenant_id).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("policy {} deleted", id)
            })),
        )
            .into_response(),
        Err(DeletePolicyError::NotFound(_)) => {
            let err =
                ErrorResponse::new("SYS_POLICY_NOT_FOUND", &format!("policy not found: {id}"));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(DeletePolicyError::Internal(msg)) => {
            // H-022 監査対応: 内部エラー詳細をレスポンスに含めない（ログには記録する）
            tracing::error!("policy_handler internal error: {msg}");
            let err = ErrorResponse::new("SYS_POLICY_INTERNAL_ERROR", "Internal server error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/policies/:id/evaluate
pub async fn evaluate_policy(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<Uuid>,
    Json(req): Json<EvaluatePolicyRequest>,
) -> impl IntoResponse {
    // CRIT-005 対応: JWT Claims からテナント ID を取得してユースケースに渡す。
    let tenant_id = extract_tenant_id(claims.as_ref());

    let input = EvaluatePolicyInput {
        policy_id: Some(id),
        package_path: String::new(),
        input: req.input,
        tenant_id,
    };

    match state.evaluate_policy_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "allowed": output.allowed,
                "package_path": output.package_path,
                "decision_id": output.decision_id,
                "cached": output.cached
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_POLICY_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                // H-022 監査対応: 内部エラー詳細をレスポンスに含めない
                tracing::error!("policy_handler evaluate internal error: {msg}");
                let err = ErrorResponse::new("SYS_POLICY_INTERNAL_ERROR", "Internal server error");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// GET /api/v1/bundles
pub async fn list_bundles(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
) -> impl IntoResponse {
    // CRIT-005 対応: JWT Claims からテナント ID を取得してユースケースに渡す。
    let tenant_id = extract_tenant_id(claims.as_ref());

    match state.list_bundles_uc.execute(&tenant_id).await {
        Ok(bundles) => {
            let items: Vec<BundleResponse> =
                bundles.into_iter().map(BundleResponse::from).collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({ "bundles": items })),
            )
                .into_response()
        }
        Err(e) => {
            // H-022 監査対応: 内部エラー詳細をレスポンスに含めない
            tracing::error!("policy_handler internal error: {e:?}");
            let err = ErrorResponse::new("SYS_POLICY_INTERNAL_ERROR", "Internal server error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/bundles/:id
pub async fn get_bundle(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    // CRIT-005 対応: JWT Claims からテナント ID を取得してユースケースに渡す。
    let tenant_id = extract_tenant_id(claims.as_ref());

    match state.get_bundle_uc.execute(&id, &tenant_id).await {
        Ok(bundle) => {
            let resp = BundleResponse::from(bundle);
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(crate::usecase::get_bundle::GetBundleError::NotFound(_)) => {
            let err =
                ErrorResponse::new("SYS_POLICY_NOT_FOUND", &format!("bundle not found: {id}"));
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(crate::usecase::get_bundle::GetBundleError::Internal(msg)) => {
            // H-022 監査対応: 内部エラー詳細をレスポンスに含めない（ログには記録する）
            tracing::error!("policy_handler internal error: {msg}");
            let err = ErrorResponse::new("SYS_POLICY_INTERNAL_ERROR", "Internal server error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/bundles
pub async fn create_bundle(
    State(state): State<AppState>,
    claims: Option<Extension<Claims>>,
    Json(req): Json<CreateBundleRequest>,
) -> impl IntoResponse {
    use crate::usecase::create_bundle::CreateBundleInput;

    // CRIT-005 対応: JWT Claims からテナント ID を取得してユースケースに渡す。
    let tenant_id = extract_tenant_id(claims.as_ref());

    let policy_ids: Result<Vec<Uuid>, _> =
        req.policy_ids.iter().map(|s| Uuid::parse_str(s)).collect();

    // let-else: UUIDパースエラーの場合は400を返す
    let Ok(policy_ids) = policy_ids else {
        let err = ErrorResponse::new("SYS_POLICY_INVALID_ID", "invalid policy_id format");
        return (StatusCode::BAD_REQUEST, Json(err)).into_response();
    };

    let input = CreateBundleInput {
        name: req.name,
        description: req.description,
        enabled: req.enabled,
        policy_ids,
        tenant_id,
    };

    match state.create_bundle_uc.execute(&input).await {
        Ok(bundle) => {
            let resp = BundleResponse::from(bundle);
            (StatusCode::CREATED, Json(resp)).into_response()
        }
        Err(e) => {
            // H-022 監査対応: 内部エラー詳細をレスポンスに含めない
            tracing::error!("policy_handler internal error: {e:?}");
            let err = ErrorResponse::new("SYS_POLICY_INTERNAL_ERROR", "Internal server error");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct ListPoliciesParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub bundle_id: Option<String>,
    pub enabled_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: String,
    pub rego_content: String,
    #[serde(default)]
    pub package_path: String,
    pub bundle_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePolicyRequest {
    pub description: Option<String>,
    pub rego_content: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct EvaluatePolicyRequest {
    pub input: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateBundleRequest {
    pub name: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub policy_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BundleResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub policy_count: usize,
    pub policy_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::entity::policy_bundle::PolicyBundle> for BundleResponse {
    fn from(b: crate::domain::entity::policy_bundle::PolicyBundle) -> Self {
        Self {
            id: b.id.to_string(),
            name: b.name,
            description: b.description,
            enabled: b.enabled,
            policy_count: b.policy_ids.len(),
            policy_ids: b
                .policy_ids
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            created_at: b.created_at.to_rfc3339(),
            updated_at: b.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PolicyResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rego_content: String,
    pub package_path: String,
    pub bundle_id: Option<String>,
    pub version: u32,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::entity::policy::Policy> for PolicyResponse {
    fn from(p: crate::domain::entity::policy::Policy) -> Self {
        Self {
            id: p.id.to_string(),
            name: p.name,
            description: p.description,
            rego_content: p.rego_content,
            package_path: p.package_path,
            bundle_id: p.bundle_id.map(|id| id.to_string()),
            version: p.version,
            enabled: p.enabled,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub field: String,
    pub message: String,
}

impl ErrorResponse {
    #[must_use]
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                details: vec![],
            },
        }
    }

    #[must_use]
    pub fn with_details(code: &str, message: &str, details: Vec<ErrorDetail>) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
                details,
            },
        }
    }
}
