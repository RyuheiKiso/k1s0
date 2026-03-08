use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::entity::policy::Policy;
use crate::domain::entity::policy_bundle::PolicyBundle;
use crate::usecase::create_bundle::{CreateBundleError, CreateBundleInput, CreateBundleUseCase};
use crate::usecase::create_policy::{CreatePolicyError, CreatePolicyInput, CreatePolicyUseCase};
use crate::usecase::delete_policy::{DeletePolicyError, DeletePolicyUseCase};
use crate::usecase::evaluate_policy::{
    EvaluatePolicyError, EvaluatePolicyInput, EvaluatePolicyUseCase,
};
use crate::usecase::get_bundle::{GetBundleError, GetBundleUseCase};
use crate::usecase::get_policy::{GetPolicyError, GetPolicyUseCase};
use crate::usecase::list_bundles::{ListBundlesError, ListBundlesUseCase};
use crate::usecase::list_policies::{ListPoliciesError, ListPoliciesInput, ListPoliciesUseCase};
use crate::usecase::update_policy::{UpdatePolicyError, UpdatePolicyInput, UpdatePolicyUseCase};

#[derive(Debug, Clone)]
pub struct PolicyData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub package_path: String,
    pub bundle_id: Option<String>,
    pub rego_content: String,
    pub enabled: bool,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PolicyBundleData {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub policy_count: u32,
    pub policy_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct EvaluatePolicyRequest {
    pub policy_id: String,
    pub input_json: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct EvaluatePolicyResponse {
    pub allowed: bool,
    pub package_path: String,
    pub decision_id: String,
    pub cached: bool,
}

#[derive(Debug, Clone)]
pub struct GetPolicyRequest {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct GetPolicyResponse {
    pub policy: PolicyData,
}

#[derive(Debug, Clone)]
pub struct ListPoliciesRequest {
    pub page: i32,
    pub page_size: i32,
    pub bundle_id: Option<String>,
    pub enabled_only: bool,
}

#[derive(Debug, Clone)]
pub struct ListPoliciesResponse {
    pub policies: Vec<PolicyData>,
    pub total_count: u64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: String,
    pub rego_content: String,
    pub package_path: String,
    pub bundle_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreatePolicyResponse {
    pub policy: PolicyData,
}

#[derive(Debug, Clone)]
pub struct UpdatePolicyRequest {
    pub id: String,
    pub description: Option<String>,
    pub rego_content: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct UpdatePolicyResponse {
    pub policy: PolicyData,
}

#[derive(Debug, Clone)]
pub struct DeletePolicyRequest {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct DeletePolicyResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CreateBundleRequest {
    pub name: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub policy_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CreateBundleResponse {
    pub bundle: PolicyBundleData,
}

#[derive(Debug, Clone)]
pub struct ListBundlesRequest;

#[derive(Debug, Clone)]
pub struct ListBundlesResponse {
    pub bundles: Vec<PolicyBundleData>,
}

#[derive(Debug, Clone)]
pub struct GetBundleRequest {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct GetBundleResponse {
    pub bundle: PolicyBundleData,
}

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("internal: {0}")]
    Internal(String),

    #[error("unimplemented: {0}")]
    Unimplemented(String),
}

pub struct PolicyGrpcService {
    create_policy_uc: Arc<CreatePolicyUseCase>,
    get_policy_uc: Arc<GetPolicyUseCase>,
    update_policy_uc: Arc<UpdatePolicyUseCase>,
    delete_policy_uc: Arc<DeletePolicyUseCase>,
    list_policies_uc: Arc<ListPoliciesUseCase>,
    evaluate_policy_uc: Arc<EvaluatePolicyUseCase>,
    create_bundle_uc: Arc<CreateBundleUseCase>,
    get_bundle_uc: Arc<GetBundleUseCase>,
    list_bundles_uc: Arc<ListBundlesUseCase>,
}

impl PolicyGrpcService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        create_policy_uc: Arc<CreatePolicyUseCase>,
        get_policy_uc: Arc<GetPolicyUseCase>,
        update_policy_uc: Arc<UpdatePolicyUseCase>,
        delete_policy_uc: Arc<DeletePolicyUseCase>,
        list_policies_uc: Arc<ListPoliciesUseCase>,
        evaluate_policy_uc: Arc<EvaluatePolicyUseCase>,
        create_bundle_uc: Arc<CreateBundleUseCase>,
        get_bundle_uc: Arc<GetBundleUseCase>,
        list_bundles_uc: Arc<ListBundlesUseCase>,
    ) -> Self {
        Self {
            create_policy_uc,
            get_policy_uc,
            update_policy_uc,
            delete_policy_uc,
            list_policies_uc,
            evaluate_policy_uc,
            create_bundle_uc,
            get_bundle_uc,
            list_bundles_uc,
        }
    }

    pub async fn evaluate_policy(
        &self,
        req: EvaluatePolicyRequest,
    ) -> Result<EvaluatePolicyResponse, GrpcError> {
        let policy_id = Uuid::parse_str(&req.policy_id).map_err(|_| {
            GrpcError::InvalidArgument(format!("invalid policy id: {}", req.policy_id))
        })?;

        let policy = self
            .get_policy_uc
            .execute(&policy_id)
            .await
            .map_err(|e| match e {
                GetPolicyError::Internal(msg) => GrpcError::Internal(msg),
            })?
            .ok_or_else(|| GrpcError::NotFound(format!("policy not found: {}", req.policy_id)))?;

        let package_path = policy.package_path.clone();

        let input_json: serde_json::Value = if req.input_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(&req.input_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid input_json: {}", e)))?
        };

        let uc_input = EvaluatePolicyInput {
            policy_id: Some(policy_id),
            package_path: package_path.clone(),
            input: input_json,
        };

        let output = self
            .evaluate_policy_uc
            .execute(&uc_input)
            .await
            .map_err(|e| match e {
                EvaluatePolicyError::NotFound(id) => {
                    GrpcError::NotFound(format!("policy not found: {}", id))
                }
                EvaluatePolicyError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(EvaluatePolicyResponse {
            allowed: output.allowed,
            package_path,
            decision_id: output.decision_id,
            cached: output.cached,
        })
    }

    pub async fn get_policy(&self, req: GetPolicyRequest) -> Result<GetPolicyResponse, GrpcError> {
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid policy id: {}", req.id)))?;

        let policy = self
            .get_policy_uc
            .execute(&id)
            .await
            .map_err(|e| match e {
                GetPolicyError::Internal(msg) => GrpcError::Internal(msg),
            })?
            .ok_or_else(|| GrpcError::NotFound(format!("policy not found: {}", req.id)))?;

        Ok(GetPolicyResponse {
            policy: to_policy_data(policy),
        })
    }

    pub async fn list_policies(
        &self,
        req: ListPoliciesRequest,
    ) -> Result<ListPoliciesResponse, GrpcError> {
        let page = if req.page <= 0 { 1 } else { req.page as u32 };
        let page_size = if req.page_size <= 0 {
            20
        } else {
            req.page_size as u32
        };

        let output = self
            .list_policies_uc
            .execute(&ListPoliciesInput {
                page,
                page_size,
                bundle_id: match req.bundle_id.as_deref() {
                    Some(bundle_id) => Some(Uuid::parse_str(bundle_id).map_err(|_| {
                        GrpcError::InvalidArgument(format!("invalid bundle_id: {}", bundle_id))
                    })?),
                    None => None,
                },
                enabled_only: req.enabled_only,
            })
            .await
            .map_err(|e| match e {
                ListPoliciesError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ListPoliciesResponse {
            policies: output.policies.into_iter().map(to_policy_data).collect(),
            total_count: output.total_count,
            page: output.page as i32,
            page_size: output.page_size as i32,
            has_next: output.has_next,
        })
    }

    pub async fn create_policy(
        &self,
        req: CreatePolicyRequest,
    ) -> Result<CreatePolicyResponse, GrpcError> {
        let bundle_id = match req.bundle_id.as_deref() {
            Some(bundle_id) => Some(Uuid::parse_str(bundle_id).map_err(|_| {
                GrpcError::InvalidArgument(format!("invalid bundle_id: {}", bundle_id))
            })?),
            None => None,
        };

        let created = self
            .create_policy_uc
            .execute(&CreatePolicyInput {
                name: req.name,
                description: req.description,
                rego_content: req.rego_content,
                package_path: req.package_path,
                bundle_id,
            })
            .await
            .map_err(|e| match e {
                CreatePolicyError::AlreadyExists(name) => {
                    GrpcError::AlreadyExists(format!("policy already exists: {}", name))
                }
                CreatePolicyError::Validation(msg) => GrpcError::InvalidArgument(msg),
                CreatePolicyError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(CreatePolicyResponse {
            policy: to_policy_data(created),
        })
    }

    pub async fn update_policy(
        &self,
        req: UpdatePolicyRequest,
    ) -> Result<UpdatePolicyResponse, GrpcError> {
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid policy id: {}", req.id)))?;

        let updated = self
            .update_policy_uc
            .execute(&UpdatePolicyInput {
                id,
                description: req.description,
                rego_content: req.rego_content,
                enabled: req.enabled,
            })
            .await
            .map_err(|e| match e {
                UpdatePolicyError::NotFound(id) => {
                    GrpcError::NotFound(format!("policy not found: {}", id))
                }
                UpdatePolicyError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(UpdatePolicyResponse {
            policy: to_policy_data(updated),
        })
    }

    pub async fn delete_policy(
        &self,
        req: DeletePolicyRequest,
    ) -> Result<DeletePolicyResponse, GrpcError> {
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid policy id: {}", req.id)))?;

        self.delete_policy_uc
            .execute(&id)
            .await
            .map_err(|e| match e {
                DeletePolicyError::NotFound(id) => {
                    GrpcError::NotFound(format!("policy not found: {}", id))
                }
                DeletePolicyError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(DeletePolicyResponse {
            success: true,
            message: format!("policy {} deleted", req.id),
        })
    }

    pub async fn create_bundle(
        &self,
        req: CreateBundleRequest,
    ) -> Result<CreateBundleResponse, GrpcError> {
        let policy_ids: Result<Vec<Uuid>, _> = req
            .policy_ids
            .iter()
            .map(|id| Uuid::parse_str(id))
            .collect();
        let policy_ids = policy_ids
            .map_err(|_| GrpcError::InvalidArgument("invalid policy_ids format".to_string()))?;

        let bundle = self
            .create_bundle_uc
            .execute(&CreateBundleInput {
                name: req.name,
                description: req.description,
                enabled: req.enabled,
                policy_ids,
            })
            .await
            .map_err(|e| match e {
                CreateBundleError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(CreateBundleResponse {
            bundle: to_bundle_data(bundle),
        })
    }

    pub async fn list_bundles(
        &self,
        _req: ListBundlesRequest,
    ) -> Result<ListBundlesResponse, GrpcError> {
        let bundles = self.list_bundles_uc.execute().await.map_err(|e| match e {
            ListBundlesError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        Ok(ListBundlesResponse {
            bundles: bundles.into_iter().map(to_bundle_data).collect(),
        })
    }

    pub async fn get_bundle(&self, req: GetBundleRequest) -> Result<GetBundleResponse, GrpcError> {
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid bundle id: {}", req.id)))?;

        let bundle = self.get_bundle_uc.execute(&id).await.map_err(|e| match e {
            GetBundleError::NotFound(id) => {
                GrpcError::NotFound(format!("bundle not found: {}", id))
            }
            GetBundleError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        Ok(GetBundleResponse {
            bundle: to_bundle_data(bundle),
        })
    }
}

fn to_policy_data(policy: Policy) -> PolicyData {
    PolicyData {
        id: policy.id.to_string(),
        name: policy.name,
        description: policy.description,
        package_path: policy.package_path,
        bundle_id: policy.bundle_id.map(|id| id.to_string()),
        rego_content: policy.rego_content,
        enabled: policy.enabled,
        version: policy.version,
        created_at: policy.created_at,
        updated_at: policy.updated_at,
    }
}

fn to_bundle_data(bundle: PolicyBundle) -> PolicyBundleData {
    PolicyBundleData {
        id: bundle.id.to_string(),
        name: bundle.name,
        description: bundle.description,
        enabled: bundle.enabled,
        policy_count: bundle.policy_ids.len() as u32,
        policy_ids: bundle
            .policy_ids
            .into_iter()
            .map(|id| id.to_string())
            .collect(),
        created_at: bundle.created_at,
        updated_at: bundle.updated_at,
    }
}
