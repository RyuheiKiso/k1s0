use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AppState;
use crate::usecase::create_policy::CreatePolicyInput;
use crate::usecase::evaluate_policy::EvaluatePolicyInput;
use crate::usecase::update_policy::UpdatePolicyInput;

/// GET /api/v1/policies
pub async fn list_policies(State(state): State<AppState>) -> impl IntoResponse {
    match state.policy_repo.find_all().await {
        Ok(policies) => {
            let items: Vec<PolicyResponse> =
                policies.into_iter().map(PolicyResponse::from).collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({ "policies": items })),
            )
                .into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_POLICY_LIST_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// GET /api/v1/policies/:id
pub async fn get_policy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.get_policy_uc.execute(&id).await {
        Ok(Some(policy)) => {
            let resp = PolicyResponse::from(policy);
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Ok(None) => {
            let err = ErrorResponse::new(
                "SYS_POLICY_NOT_FOUND",
                &format!("policy not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_POLICY_GET_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/policies
pub async fn create_policy(
    State(state): State<AppState>,
    Json(req): Json<CreatePolicyRequest>,
) -> impl IntoResponse {
    let input = CreatePolicyInput {
        name: req.name,
        description: req.description,
        rego_content: req.rego_content,
    };

    match state.create_policy_uc.execute(&input).await {
        Ok(policy) => {
            let resp = PolicyResponse::from(policy);
            (StatusCode::CREATED, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") {
                let err = ErrorResponse::new("SYS_POLICY_ALREADY_EXISTS", &msg);
                (StatusCode::CONFLICT, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_POLICY_CREATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// PUT /api/v1/policies/:id
pub async fn update_policy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePolicyRequest>,
) -> impl IntoResponse {
    let input = UpdatePolicyInput {
        id,
        description: req.description,
        rego_content: req.rego_content,
        enabled: req.enabled,
    };

    match state.update_policy_uc.execute(&input).await {
        Ok(policy) => {
            let resp = PolicyResponse::from(policy);
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_POLICY_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_POLICY_UPDATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// DELETE /api/v1/policies/:id
pub async fn delete_policy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.policy_repo.delete(&id).await {
        Ok(true) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": format!("policy {} deleted", id)
            })),
        )
            .into_response(),
        Ok(false) => {
            let err = ErrorResponse::new(
                "SYS_POLICY_NOT_FOUND",
                &format!("policy not found: {}", id),
            );
            (StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_POLICY_DELETE_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/policies/:id/evaluate
pub async fn evaluate_policy(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<EvaluatePolicyRequest>,
) -> impl IntoResponse {
    let input = EvaluatePolicyInput {
        policy_id: id,
        input: req.input,
    };

    match state.evaluate_policy_uc.execute(&input).await {
        Ok(output) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "allowed": output.allowed,
                "reason": output.reason
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") {
                let err = ErrorResponse::new("SYS_POLICY_NOT_FOUND", &msg);
                (StatusCode::NOT_FOUND, Json(err)).into_response()
            } else {
                let err = ErrorResponse::new("SYS_POLICY_EVALUATE_FAILED", &msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
            }
        }
    }
}

/// GET /api/v1/bundles
pub async fn list_bundles(State(state): State<AppState>) -> impl IntoResponse {
    match state.list_bundles_uc.execute().await {
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
            let err = ErrorResponse::new("SYS_POLICY_BUNDLE_LIST_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

/// POST /api/v1/bundles
pub async fn create_bundle(
    State(state): State<AppState>,
    Json(req): Json<CreateBundleRequest>,
) -> impl IntoResponse {
    use crate::usecase::create_bundle::CreateBundleInput;

    let policy_ids: Result<Vec<Uuid>, _> = req
        .policy_ids
        .iter()
        .map(|s| Uuid::parse_str(s))
        .collect();

    let policy_ids = match policy_ids {
        Ok(ids) => ids,
        Err(_) => {
            let err = ErrorResponse::new("SYS_POLICY_INVALID_ID", "invalid policy_id format");
            return (StatusCode::BAD_REQUEST, Json(err)).into_response();
        }
    };

    let input = CreateBundleInput {
        name: req.name,
        policy_ids,
    };

    match state.create_bundle_uc.execute(&input).await {
        Ok(bundle) => {
            let resp = BundleResponse::from(bundle);
            (StatusCode::CREATED, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(e) => {
            let err = ErrorResponse::new("SYS_POLICY_BUNDLE_CREATE_FAILED", &e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// --- Request / Response types ---

#[derive(Debug, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: String,
    pub rego_content: String,
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
    pub policy_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BundleResponse {
    pub id: String,
    pub name: String,
    pub policy_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::domain::entity::policy_bundle::PolicyBundle> for BundleResponse {
    fn from(b: crate::domain::entity::policy_bundle::PolicyBundle) -> Self {
        Self {
            id: b.id.to_string(),
            name: b.name,
            policy_ids: b.policy_ids.iter().map(|id| id.to_string()).collect(),
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
}

impl ErrorResponse {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: code.to_string(),
                message: message.to_string(),
            },
        }
    }
}
