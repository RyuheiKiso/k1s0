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

#[derive(Debug, Clone)]
#[allow(dead_code)]
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

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct EvaluatePolicyRequest {
    pub policy_id: String,
    pub input_json: Vec<u8>,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct EvaluatePolicyResponse {
    pub allowed: bool,
    pub package_path: String,
    pub decision_id: String,
    pub cached: bool,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct GetPolicyRequest {
    pub id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct GetPolicyResponse {
    pub policy: PolicyData,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct ListPoliciesRequest {
    pub page: i32,
    pub page_size: i32,
    pub bundle_id: Option<String>,
    pub enabled_only: bool,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct ListPoliciesResponse {
    pub policies: Vec<PolicyData>,
    pub total_count: u64,
    pub page: i32,
    pub page_size: i32,
    pub has_next: bool,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: String,
    pub rego_content: String,
    pub package_path: String,
    pub bundle_id: Option<String>,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct CreatePolicyResponse {
    pub policy: PolicyData,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct UpdatePolicyRequest {
    pub id: String,
    pub description: Option<String>,
    pub rego_content: Option<String>,
    pub enabled: Option<bool>,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct UpdatePolicyResponse {
    pub policy: PolicyData,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct DeletePolicyRequest {
    pub id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct DeletePolicyResponse {
    pub success: bool,
    pub message: String,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct CreateBundleRequest {
    pub name: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub policy_ids: Vec<String>,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct CreateBundleResponse {
    pub bundle: PolicyBundleData,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct ListBundlesRequest {
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct ListBundlesResponse {
    pub bundles: Vec<PolicyBundleData>,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct GetBundleRequest {
    pub id: String,
    pub tenant_id: String,
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

// ユースケースフィールドの命名規則として _uc サフィックスを使用する（アーキテクチャ上の意図的な設計）
#[allow(clippy::struct_field_names)]
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
    #[must_use]
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

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらポリシー評価を実行する。
    pub async fn evaluate_policy(
        &self,
        req: EvaluatePolicyRequest,
    ) -> Result<EvaluatePolicyResponse, GrpcError> {
        let policy_id = Uuid::parse_str(&req.policy_id).map_err(|_| {
            GrpcError::InvalidArgument(format!("invalid policy id: {}", req.policy_id))
        })?;

        let policy = self
            .get_policy_uc
            .execute(&policy_id, &req.tenant_id)
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
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid input_json: {e}")))?
        };

        let uc_input = EvaluatePolicyInput {
            policy_id: Some(policy_id),
            package_path: package_path.clone(),
            input: input_json,
            tenant_id: req.tenant_id,
        };

        let output = self
            .evaluate_policy_uc
            .execute(&uc_input)
            .await
            .map_err(|e| match e {
                EvaluatePolicyError::NotFound(id) => {
                    GrpcError::NotFound(format!("policy not found: {id}"))
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

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらポリシーを取得する。
    pub async fn get_policy(&self, req: GetPolicyRequest) -> Result<GetPolicyResponse, GrpcError> {
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid policy id: {}", req.id)))?;

        let policy = self
            .get_policy_uc
            .execute(&id, &req.tenant_id)
            .await
            .map_err(|e| match e {
                GetPolicyError::Internal(msg) => GrpcError::Internal(msg),
            })?
            .ok_or_else(|| GrpcError::NotFound(format!("policy not found: {}", req.id)))?;

        Ok(GetPolicyResponse {
            policy: to_policy_data(policy),
        })
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらポリシー一覧を取得する。
    pub async fn list_policies(
        &self,
        req: ListPoliciesRequest,
    ) -> Result<ListPoliciesResponse, GrpcError> {
        // LOW-008: 安全な型変換（負の場合はデフォルト値を使用、プロトコルの不変条件）
        let page = if req.page <= 0 { 1 } else { u32::try_from(req.page).unwrap_or(0) };
        let page_size = if req.page_size <= 0 {
            20
        } else {
            u32::try_from(req.page_size).unwrap_or(0)
        };

        let output = self
            .list_policies_uc
            .execute(&ListPoliciesInput {
                page,
                page_size,
                bundle_id: match req.bundle_id.as_deref() {
                    Some(bundle_id) => Some(Uuid::parse_str(bundle_id).map_err(|_| {
                        GrpcError::InvalidArgument(format!("invalid bundle_id: {bundle_id}"))
                    })?),
                    None => None,
                },
                enabled_only: req.enabled_only,
                tenant_id: req.tenant_id,
            })
            .await
            .map_err(|e| match e {
                ListPoliciesError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ListPoliciesResponse {
            policies: output.policies.into_iter().map(to_policy_data).collect(),
            total_count: output.total_count,
            // LOW-008: 安全な型変換（page/page_size は正の値でありi32範囲内）
            page: i32::try_from(output.page).unwrap_or(i32::MAX),
            page_size: i32::try_from(output.page_size).unwrap_or(i32::MAX),
            has_next: output.has_next,
        })
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらポリシーを作成する。
    pub async fn create_policy(
        &self,
        req: CreatePolicyRequest,
    ) -> Result<CreatePolicyResponse, GrpcError> {
        let bundle_id = match req.bundle_id.as_deref() {
            Some(bundle_id) => Some(Uuid::parse_str(bundle_id).map_err(|_| {
                GrpcError::InvalidArgument(format!("invalid bundle_id: {bundle_id}"))
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
                tenant_id: req.tenant_id,
            })
            .await
            .map_err(|e| match e {
                CreatePolicyError::AlreadyExists(name) => {
                    GrpcError::AlreadyExists(format!("policy already exists: {name}"))
                }
                CreatePolicyError::Validation(msg) => GrpcError::InvalidArgument(msg),
                CreatePolicyError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(CreatePolicyResponse {
            policy: to_policy_data(created),
        })
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらポリシーを更新する。
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
                tenant_id: req.tenant_id,
            })
            .await
            .map_err(|e| match e {
                UpdatePolicyError::NotFound(id) => {
                    GrpcError::NotFound(format!("policy not found: {id}"))
                }
                UpdatePolicyError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(UpdatePolicyResponse {
            policy: to_policy_data(updated),
        })
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらポリシーを削除する。
    pub async fn delete_policy(
        &self,
        req: DeletePolicyRequest,
    ) -> Result<DeletePolicyResponse, GrpcError> {
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid policy id: {}", req.id)))?;

        self.delete_policy_uc
            .execute(&id, &req.tenant_id)
            .await
            .map_err(|e| match e {
                DeletePolicyError::NotFound(id) => {
                    GrpcError::NotFound(format!("policy not found: {id}"))
                }
                DeletePolicyError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(DeletePolicyResponse {
            success: true,
            message: format!("policy {} deleted", req.id),
        })
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらバンドルを作成する。
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
                tenant_id: req.tenant_id,
            })
            .await
            .map_err(|e| match e {
                CreateBundleError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(CreateBundleResponse {
            bundle: to_bundle_data(bundle),
        })
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらバンドル一覧を取得する。
    pub async fn list_bundles(
        &self,
        req: ListBundlesRequest,
    ) -> Result<ListBundlesResponse, GrpcError> {
        let bundles = self
            .list_bundles_uc
            .execute(&req.tenant_id)
            .await
            .map_err(|e| match e {
                ListBundlesError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ListBundlesResponse {
            bundles: bundles.into_iter().map(to_bundle_data).collect(),
        })
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらバンドルを取得する。
    pub async fn get_bundle(&self, req: GetBundleRequest) -> Result<GetBundleResponse, GrpcError> {
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid bundle id: {}", req.id)))?;

        let bundle = self
            .get_bundle_uc
            .execute(&id, &req.tenant_id)
            .await
            .map_err(|e| match e {
                GetBundleError::NotFound(id) => {
                    GrpcError::NotFound(format!("bundle not found: {id}"))
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
        // LOW-008: 安全な型変換（ポリシー数は u32 範囲内が前提）
        policy_count: u32::try_from(bundle.policy_ids.len()).unwrap_or(u32::MAX),
        policy_ids: bundle
            .policy_ids
            .into_iter()
            .map(|id| id.to_string())
            .collect(),
        created_at: bundle.created_at,
        updated_at: bundle.updated_at,
    }
}
