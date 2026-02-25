//! Quota gRPC サービス実装（ドメイン層ラッパー）。

use std::sync::Arc;

use crate::usecase::{
    CreateQuotaPolicyUseCase, DeleteQuotaPolicyUseCase, GetQuotaPolicyUseCase,
    GetQuotaUsageUseCase, IncrementQuotaUsageUseCase, ListQuotaPoliciesUseCase,
    UpdateQuotaPolicyUseCase,
};
use crate::usecase::create_quota_policy::{CreateQuotaPolicyError, CreateQuotaPolicyInput};
use crate::usecase::delete_quota_policy::DeleteQuotaPolicyError;
use crate::usecase::get_quota_policy::GetQuotaPolicyError;
use crate::usecase::get_quota_usage::GetQuotaUsageError;
use crate::usecase::increment_quota_usage::{IncrementQuotaUsageError, IncrementQuotaUsageInput};
use crate::usecase::list_quota_policies::ListQuotaPoliciesInput;
use crate::usecase::update_quota_policy::{UpdateQuotaPolicyError, UpdateQuotaPolicyInput};
use crate::domain::entity::quota::{QuotaPolicy, QuotaUsage, IncrementResult};

/// GrpcError は gRPC レイヤーのエラー型。
#[derive(Debug)]
pub enum GrpcError {
    NotFound(String),
    InvalidArgument(String),
    Internal(String),
}

impl From<CreateQuotaPolicyError> for GrpcError {
    fn from(e: CreateQuotaPolicyError) -> Self {
        match e {
            CreateQuotaPolicyError::Validation(msg) => GrpcError::InvalidArgument(msg),
            CreateQuotaPolicyError::Internal(msg) => GrpcError::Internal(msg),
        }
    }
}

impl From<GetQuotaPolicyError> for GrpcError {
    fn from(e: GetQuotaPolicyError) -> Self {
        match e {
            GetQuotaPolicyError::NotFound(msg) => GrpcError::NotFound(msg),
            GetQuotaPolicyError::Internal(msg) => GrpcError::Internal(msg),
        }
    }
}

impl From<UpdateQuotaPolicyError> for GrpcError {
    fn from(e: UpdateQuotaPolicyError) -> Self {
        match e {
            UpdateQuotaPolicyError::NotFound(msg) => GrpcError::NotFound(msg),
            UpdateQuotaPolicyError::Validation(msg) => GrpcError::InvalidArgument(msg),
            UpdateQuotaPolicyError::Internal(msg) => GrpcError::Internal(msg),
        }
    }
}

impl From<DeleteQuotaPolicyError> for GrpcError {
    fn from(e: DeleteQuotaPolicyError) -> Self {
        match e {
            DeleteQuotaPolicyError::NotFound(msg) => GrpcError::NotFound(msg),
            DeleteQuotaPolicyError::Internal(msg) => GrpcError::Internal(msg),
        }
    }
}

impl From<GetQuotaUsageError> for GrpcError {
    fn from(e: GetQuotaUsageError) -> Self {
        match e {
            GetQuotaUsageError::NotFound(msg) => GrpcError::NotFound(msg),
            GetQuotaUsageError::Internal(msg) => GrpcError::Internal(msg),
        }
    }
}

impl From<IncrementQuotaUsageError> for GrpcError {
    fn from(e: IncrementQuotaUsageError) -> Self {
        match e {
            IncrementQuotaUsageError::NotFound(msg) => GrpcError::NotFound(msg),
            IncrementQuotaUsageError::Exceeded {
                quota_id,
                used,
                limit,
                ..
            } => GrpcError::InvalidArgument(format!(
                "quota exceeded for {}: {}/{}",
                quota_id, used, limit
            )),
            IncrementQuotaUsageError::Internal(msg) => GrpcError::Internal(msg),
        }
    }
}

/// CreatePolicyRequest は CreateQuotaPolicy gRPC リクエストの内部表現。
pub struct CreatePolicyRequest {
    pub name: String,
    pub subject_type: String,
    pub subject_id: String,
    pub limit: u64,
    pub period: String,
    pub enabled: bool,
    pub alert_threshold_percent: Option<u8>,
}

/// UpdatePolicyRequest は UpdateQuotaPolicy gRPC リクエストの内部表現（部分更新）。
pub struct UpdatePolicyRequest {
    pub id: String,
    pub enabled: Option<bool>,
    pub limit: Option<u64>,
}

/// ListPoliciesRequest は ListQuotaPolicies gRPC リクエストの内部表現。
pub struct ListPoliciesRequest {
    pub page: u32,
    pub page_size: u32,
}

/// ListPoliciesResult は ListQuotaPolicies の結果。
pub struct ListPoliciesResult {
    pub policies: Vec<QuotaPolicy>,
    pub total: u64,
}

/// QuotaGrpcService はクォータ gRPC サービスのビジネスロジック層。
pub struct QuotaGrpcService {
    pub create_policy_uc: Arc<CreateQuotaPolicyUseCase>,
    pub get_policy_uc: Arc<GetQuotaPolicyUseCase>,
    pub list_policies_uc: Arc<ListQuotaPoliciesUseCase>,
    pub update_policy_uc: Arc<UpdateQuotaPolicyUseCase>,
    pub delete_policy_uc: Arc<DeleteQuotaPolicyUseCase>,
    pub get_usage_uc: Arc<GetQuotaUsageUseCase>,
    pub increment_usage_uc: Arc<IncrementQuotaUsageUseCase>,
}

impl QuotaGrpcService {
    pub fn new(
        create_policy_uc: Arc<CreateQuotaPolicyUseCase>,
        get_policy_uc: Arc<GetQuotaPolicyUseCase>,
        list_policies_uc: Arc<ListQuotaPoliciesUseCase>,
        update_policy_uc: Arc<UpdateQuotaPolicyUseCase>,
        delete_policy_uc: Arc<DeleteQuotaPolicyUseCase>,
        get_usage_uc: Arc<GetQuotaUsageUseCase>,
        increment_usage_uc: Arc<IncrementQuotaUsageUseCase>,
    ) -> Self {
        Self {
            create_policy_uc,
            get_policy_uc,
            list_policies_uc,
            update_policy_uc,
            delete_policy_uc,
            get_usage_uc,
            increment_usage_uc,
        }
    }

    pub async fn create_policy(
        &self,
        req: CreatePolicyRequest,
    ) -> Result<QuotaPolicy, GrpcError> {
        let input = CreateQuotaPolicyInput {
            name: req.name,
            subject_type: req.subject_type,
            subject_id: req.subject_id,
            limit: req.limit,
            period: req.period,
            enabled: req.enabled,
            alert_threshold_percent: req.alert_threshold_percent,
        };
        self.create_policy_uc
            .execute(&input)
            .await
            .map_err(GrpcError::from)
    }

    pub async fn get_policy(&self, id: &str) -> Result<QuotaPolicy, GrpcError> {
        self.get_policy_uc
            .execute(id)
            .await
            .map_err(GrpcError::from)
    }

    pub async fn list_policies(
        &self,
        req: ListPoliciesRequest,
    ) -> Result<ListPoliciesResult, GrpcError> {
        let input = ListQuotaPoliciesInput {
            page: req.page,
            page_size: req.page_size,
        };
        let output = self
            .list_policies_uc
            .execute(&input)
            .await
            .map_err(|e| GrpcError::Internal(e.to_string()))?;
        Ok(ListPoliciesResult {
            policies: output.quotas,
            total: output.total_count,
        })
    }

    pub async fn update_policy(
        &self,
        req: UpdatePolicyRequest,
    ) -> Result<QuotaPolicy, GrpcError> {
        // 部分更新: まず現在のポリシーを取得してから更新
        let current = self.get_policy_uc
            .execute(&req.id)
            .await
            .map_err(GrpcError::from)?;

        let new_enabled = req.enabled.unwrap_or(current.enabled);
        let new_limit = req.limit.unwrap_or(current.limit);

        let input = UpdateQuotaPolicyInput {
            id: req.id,
            name: current.name,
            subject_type: current.subject_type.as_str().to_string(),
            subject_id: current.subject_id,
            limit: new_limit,
            period: current.period.as_str().to_string(),
            enabled: new_enabled,
            alert_threshold_percent: current.alert_threshold_percent,
        };
        self.update_policy_uc
            .execute(&input)
            .await
            .map_err(GrpcError::from)
    }

    pub async fn delete_policy(&self, id: &str) -> Result<(), GrpcError> {
        self.delete_policy_uc
            .execute(id)
            .await
            .map_err(GrpcError::from)
    }

    pub async fn get_usage(&self, quota_id: &str) -> Result<QuotaUsage, GrpcError> {
        self.get_usage_uc
            .execute(quota_id)
            .await
            .map_err(GrpcError::from)
    }

    pub async fn increment_usage(
        &self,
        quota_id: String,
        amount: u64,
    ) -> Result<IncrementResult, GrpcError> {
        let input = IncrementQuotaUsageInput { quota_id, amount };
        self.increment_usage_uc
            .execute(&input)
            .await
            .map_err(GrpcError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_error_variants() {
        let e1 = GrpcError::NotFound("test".to_string());
        let e2 = GrpcError::InvalidArgument("test".to_string());
        let e3 = GrpcError::Internal("test".to_string());
        assert!(matches!(e1, GrpcError::NotFound(_)));
        assert!(matches!(e2, GrpcError::InvalidArgument(_)));
        assert!(matches!(e3, GrpcError::Internal(_)));
    }

    #[test]
    fn test_create_policy_error_conversion() {
        let e = CreateQuotaPolicyError::Validation("bad input".to_string());
        let grpc: GrpcError = e.into();
        assert!(matches!(grpc, GrpcError::InvalidArgument(_)));

        let e = CreateQuotaPolicyError::Internal("db error".to_string());
        let grpc: GrpcError = e.into();
        assert!(matches!(grpc, GrpcError::Internal(_)));
    }

    #[test]
    fn test_get_policy_error_conversion() {
        let e = GetQuotaPolicyError::NotFound("id".to_string());
        let grpc: GrpcError = e.into();
        assert!(matches!(grpc, GrpcError::NotFound(_)));
    }

    #[test]
    fn test_delete_policy_error_conversion() {
        let e = DeleteQuotaPolicyError::NotFound("id".to_string());
        let grpc: GrpcError = e.into();
        assert!(matches!(grpc, GrpcError::NotFound(_)));
    }

    #[test]
    fn test_get_usage_error_conversion() {
        let e = GetQuotaUsageError::NotFound("id".to_string());
        let grpc: GrpcError = e.into();
        assert!(matches!(grpc, GrpcError::NotFound(_)));
    }

    #[test]
    fn test_increment_usage_exceeded_conversion() {
        let e = IncrementQuotaUsageError::Exceeded {
            quota_id: "q1".to_string(),
            subject_id: "s1".to_string(),
            used: 101,
            limit: 100,
            period: "daily".to_string(),
        };
        let grpc: GrpcError = e.into();
        assert!(matches!(grpc, GrpcError::InvalidArgument(_)));
    }
}
