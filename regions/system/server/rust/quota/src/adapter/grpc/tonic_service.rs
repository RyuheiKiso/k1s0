//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の QuotaService トレイトを実装する。
//! 各メソッドで proto 型 <-> ドメイン型の変換を行い、QuotaGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::quota::v1::{
    quota_service_server::QuotaService,
    CreateQuotaPolicyRequest as ProtoCreateQuotaPolicyRequest,
    CreateQuotaPolicyResponse as ProtoCreateQuotaPolicyResponse,
    DeleteQuotaPolicyRequest as ProtoDeleteQuotaPolicyRequest,
    DeleteQuotaPolicyResponse as ProtoDeleteQuotaPolicyResponse,
    GetQuotaPolicyRequest as ProtoGetQuotaPolicyRequest,
    GetQuotaPolicyResponse as ProtoGetQuotaPolicyResponse,
    GetQuotaUsageRequest as ProtoGetQuotaUsageRequest,
    GetQuotaUsageResponse as ProtoGetQuotaUsageResponse,
    IncrementQuotaUsageRequest as ProtoIncrementQuotaUsageRequest,
    IncrementQuotaUsageResponse as ProtoIncrementQuotaUsageResponse,
    ListQuotaPoliciesRequest as ProtoListQuotaPoliciesRequest,
    ListQuotaPoliciesResponse as ProtoListQuotaPoliciesResponse,
    QuotaPolicy as ProtoQuotaPolicy, QuotaUsage as ProtoQuotaUsage,
    UpdateQuotaPolicyRequest as ProtoUpdateQuotaPolicyRequest,
    UpdateQuotaPolicyResponse as ProtoUpdateQuotaPolicyResponse,
};

use crate::domain::entity::quota::{QuotaPolicy, QuotaUsage};

use super::quota_grpc::{
    CreatePolicyRequest, GrpcError, ListPoliciesRequest, QuotaGrpcService, UpdatePolicyRequest,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- ドメイン型 -> Proto 型 変換ヘルパー ---

fn policy_to_proto(p: &QuotaPolicy) -> ProtoQuotaPolicy {
    ProtoQuotaPolicy {
        id: p.id.clone(),
        name: p.name.clone(),
        subject_type: p.subject_type.as_str().to_string(),
        subject_id: p.subject_id.clone(),
        limit: p.limit,
        period: p.period.as_str().to_string(),
        enabled: p.enabled,
        alert_threshold_percent: p.alert_threshold_percent.map(|v| v as u32),
    }
}

fn usage_to_proto(u: &QuotaUsage) -> ProtoQuotaUsage {
    ProtoQuotaUsage {
        quota_id: u.quota_id.clone(),
        subject_type: u.subject_type.as_str().to_string(),
        subject_id: u.subject_id.clone(),
        period: u.period.as_str().to_string(),
        limit: u.limit,
        used: u.used,
        remaining: u.remaining,
        usage_percent: u.usage_percent,
        exceeded: u.exceeded,
    }
}

// --- QuotaService tonic ラッパー ---

/// QuotaServiceTonic は tonic の QuotaService として QuotaGrpcService をラップする。
pub struct QuotaServiceTonic {
    inner: Arc<QuotaGrpcService>,
}

impl QuotaServiceTonic {
    pub fn new(inner: Arc<QuotaGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl QuotaService for QuotaServiceTonic {
    async fn create_quota_policy(
        &self,
        request: Request<ProtoCreateQuotaPolicyRequest>,
    ) -> Result<Response<ProtoCreateQuotaPolicyResponse>, Status> {
        let inner = request.into_inner();
        let req = CreatePolicyRequest {
            name: inner.name,
            subject_type: inner.subject_type,
            subject_id: inner.subject_id,
            limit: inner.limit,
            period: inner.period,
            enabled: inner.enabled,
            alert_threshold_percent: inner.alert_threshold_percent.map(|v| v as u8),
        };
        let policy = self
            .inner
            .create_policy(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCreateQuotaPolicyResponse {
            policy: Some(policy_to_proto(&policy)),
        }))
    }

    async fn get_quota_policy(
        &self,
        request: Request<ProtoGetQuotaPolicyRequest>,
    ) -> Result<Response<ProtoGetQuotaPolicyResponse>, Status> {
        let inner = request.into_inner();
        let policy = self
            .inner
            .get_policy(&inner.id)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetQuotaPolicyResponse {
            policy: Some(policy_to_proto(&policy)),
        }))
    }

    async fn list_quota_policies(
        &self,
        request: Request<ProtoListQuotaPoliciesRequest>,
    ) -> Result<Response<ProtoListQuotaPoliciesResponse>, Status> {
        let inner = request.into_inner();
        let req = ListPoliciesRequest {
            page: inner.page,
            page_size: inner.page_size,
        };
        let result = self
            .inner
            .list_policies(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_policies = result.policies.iter().map(policy_to_proto).collect();

        Ok(Response::new(ProtoListQuotaPoliciesResponse {
            policies: proto_policies,
            total: result.total,
        }))
    }

    async fn update_quota_policy(
        &self,
        request: Request<ProtoUpdateQuotaPolicyRequest>,
    ) -> Result<Response<ProtoUpdateQuotaPolicyResponse>, Status> {
        let inner = request.into_inner();
        let req = UpdatePolicyRequest {
            id: inner.id,
            enabled: inner.enabled,
            limit: inner.limit,
        };
        let policy = self
            .inner
            .update_policy(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoUpdateQuotaPolicyResponse {
            policy: Some(policy_to_proto(&policy)),
        }))
    }

    async fn delete_quota_policy(
        &self,
        request: Request<ProtoDeleteQuotaPolicyRequest>,
    ) -> Result<Response<ProtoDeleteQuotaPolicyResponse>, Status> {
        let inner = request.into_inner();
        let id = inner.id.clone();
        self.inner
            .delete_policy(&id)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoDeleteQuotaPolicyResponse {
            id,
            deleted: true,
        }))
    }

    async fn get_quota_usage(
        &self,
        request: Request<ProtoGetQuotaUsageRequest>,
    ) -> Result<Response<ProtoGetQuotaUsageResponse>, Status> {
        let inner = request.into_inner();
        let usage = self
            .inner
            .get_usage(&inner.quota_id)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoGetQuotaUsageResponse {
            usage: Some(usage_to_proto(&usage)),
        }))
    }

    async fn increment_quota_usage(
        &self,
        request: Request<ProtoIncrementQuotaUsageRequest>,
    ) -> Result<Response<ProtoIncrementQuotaUsageResponse>, Status> {
        let inner = request.into_inner();
        let result = self
            .inner
            .increment_usage(inner.quota_id, inner.amount)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoIncrementQuotaUsageResponse {
            quota_id: result.quota_id,
            used: result.used,
            remaining: result.remaining,
            usage_percent: result.usage_percent,
            exceeded: result.exceeded,
            allowed: result.allowed,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::quota::{Period, SubjectType};
    use crate::domain::repository::quota_repository::{
        MockQuotaPolicyRepository, MockQuotaUsageRepository,
    };
    use crate::usecase::{
        CreateQuotaPolicyUseCase, DeleteQuotaPolicyUseCase, GetQuotaPolicyUseCase,
        GetQuotaUsageUseCase, IncrementQuotaUsageUseCase, ListQuotaPoliciesUseCase,
        UpdateQuotaPolicyUseCase,
    };

    fn sample_policy() -> QuotaPolicy {
        QuotaPolicy::new(
            "test-policy".to_string(),
            SubjectType::Tenant,
            "tenant-1".to_string(),
            1000,
            Period::Daily,
            true,
            None,
        )
    }

    fn make_tonic_service(
        policy_mock: MockQuotaPolicyRepository,
        usage_mock: MockQuotaUsageRepository,
    ) -> QuotaServiceTonic {
        let policy_repo = Arc::new(policy_mock);
        let usage_repo = Arc::new(usage_mock);
        let grpc_svc = Arc::new(QuotaGrpcService::new(
            Arc::new(CreateQuotaPolicyUseCase::new(policy_repo.clone())),
            Arc::new(GetQuotaPolicyUseCase::new(policy_repo.clone())),
            Arc::new(ListQuotaPoliciesUseCase::new(policy_repo.clone())),
            Arc::new(UpdateQuotaPolicyUseCase::new(policy_repo.clone())),
            Arc::new(DeleteQuotaPolicyUseCase::new(policy_repo.clone())),
            Arc::new(GetQuotaUsageUseCase::new(policy_repo.clone(), usage_repo.clone())),
            Arc::new(IncrementQuotaUsageUseCase::new(policy_repo, usage_repo)),
        ));
        QuotaServiceTonic::new(grpc_svc)
    }

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("policy not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
        assert!(status.message().contains("policy not found"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("invalid input".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("invalid input"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("database error"));
    }

    #[test]
    fn test_policy_to_proto_conversion() {
        let policy = sample_policy();
        let proto = policy_to_proto(&policy);
        assert_eq!(proto.id, policy.id);
        assert_eq!(proto.name, "test-policy");
        assert_eq!(proto.subject_type, "tenant");
        assert_eq!(proto.subject_id, "tenant-1");
        assert_eq!(proto.limit, 1000);
        assert_eq!(proto.period, "daily");
        assert!(proto.enabled);
        assert_eq!(proto.alert_threshold_percent, None);
    }

    #[tokio::test]
    async fn test_create_quota_policy() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();
        policy_mock.expect_create().returning(|_| Ok(()));

        let tonic_svc = make_tonic_service(policy_mock, usage_mock);
        let req = Request::new(ProtoCreateQuotaPolicyRequest {
            name: "test-policy".to_string(),
            subject_type: "tenant".to_string(),
            subject_id: "tenant-1".to_string(),
            limit: 1000,
            period: "daily".to_string(),
            enabled: true,
            alert_threshold_percent: None,
        });
        let resp = tonic_svc.create_quota_policy(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.policy.is_some());
        let p = inner.policy.unwrap();
        assert_eq!(p.name, "test-policy");
        assert_eq!(p.subject_type, "tenant");
        assert_eq!(p.limit, 1000);
    }

    #[tokio::test]
    async fn test_get_quota_policy_not_found() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();
        policy_mock.expect_find_by_id().returning(|_| Ok(None));

        let tonic_svc = make_tonic_service(policy_mock, usage_mock);
        let req = Request::new(ProtoGetQuotaPolicyRequest {
            id: "nonexistent".to_string(),
        });
        let result = tonic_svc.get_quota_policy(req).await;
        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn test_get_quota_policy_success() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();
        let policy = sample_policy();
        let policy_id = policy.id.clone();
        let return_policy = policy.clone();
        policy_mock
            .expect_find_by_id()
            .withf(move |id| id == policy_id)
            .returning(move |_| Ok(Some(return_policy.clone())));

        let tonic_svc = make_tonic_service(policy_mock, usage_mock);
        let req = Request::new(ProtoGetQuotaPolicyRequest {
            id: policy.id.clone(),
        });
        let resp = tonic_svc.get_quota_policy(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.policy.is_some());
        assert_eq!(inner.policy.unwrap().name, "test-policy");
    }

    #[tokio::test]
    async fn test_list_quota_policies() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();
        let policies = vec![sample_policy()];
        let return_policies = policies.clone();
        policy_mock
            .expect_find_all()
            .returning(move |_, _| Ok((return_policies.clone(), 1)));

        let tonic_svc = make_tonic_service(policy_mock, usage_mock);
        let req = Request::new(ProtoListQuotaPoliciesRequest {
            page: 1,
            page_size: 20,
        });
        let resp = tonic_svc.list_quota_policies(req).await.unwrap();
        let inner = resp.into_inner();
        assert_eq!(inner.policies.len(), 1);
        assert_eq!(inner.total, 1);
    }

    #[tokio::test]
    async fn test_delete_quota_policy_success() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();
        policy_mock
            .expect_delete()
            .withf(|id| id == "quota-1")
            .returning(|_| Ok(true));

        let tonic_svc = make_tonic_service(policy_mock, usage_mock);
        let req = Request::new(ProtoDeleteQuotaPolicyRequest {
            id: "quota-1".to_string(),
        });
        let resp = tonic_svc.delete_quota_policy(req).await.unwrap();
        let inner = resp.into_inner();
        assert_eq!(inner.id, "quota-1");
        assert!(inner.deleted);
    }

    #[tokio::test]
    async fn test_delete_quota_policy_not_found() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let usage_mock = MockQuotaUsageRepository::new();
        policy_mock
            .expect_delete()
            .returning(|_| Ok(false));

        let tonic_svc = make_tonic_service(policy_mock, usage_mock);
        let req = Request::new(ProtoDeleteQuotaPolicyRequest {
            id: "nonexistent".to_string(),
        });
        let result = tonic_svc.delete_quota_policy(req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn test_get_quota_usage_success() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let mut usage_mock = MockQuotaUsageRepository::new();
        let policy = sample_policy();
        let policy_id = policy.id.clone();
        let return_policy = policy.clone();
        policy_mock
            .expect_find_by_id()
            .withf(move |id| id == policy_id)
            .returning(move |_| Ok(Some(return_policy.clone())));
        usage_mock.expect_get_usage().returning(|_| Ok(Some(500)));

        let tonic_svc = make_tonic_service(policy_mock, usage_mock);
        let req = Request::new(ProtoGetQuotaUsageRequest {
            quota_id: policy.id.clone(),
        });
        let resp = tonic_svc.get_quota_usage(req).await.unwrap();
        let inner = resp.into_inner();
        assert!(inner.usage.is_some());
        let usage = inner.usage.unwrap();
        assert_eq!(usage.used, 500);
        assert_eq!(usage.remaining, 500);
        assert!(!usage.exceeded);
    }

    #[tokio::test]
    async fn test_increment_quota_usage_success() {
        let mut policy_mock = MockQuotaPolicyRepository::new();
        let mut usage_mock = MockQuotaUsageRepository::new();
        let policy = sample_policy();
        let policy_id = policy.id.clone();
        let return_policy = policy.clone();
        policy_mock
            .expect_find_by_id()
            .withf(move |id| id == policy_id)
            .returning(move |_| Ok(Some(return_policy.clone())));
        usage_mock.expect_increment().returning(|_, _| Ok(100));

        let tonic_svc = make_tonic_service(policy_mock, usage_mock);
        let req = Request::new(ProtoIncrementQuotaUsageRequest {
            quota_id: policy.id.clone(),
            amount: 1,
        });
        let resp = tonic_svc.increment_quota_usage(req).await.unwrap();
        let inner = resp.into_inner();
        assert_eq!(inner.used, 100);
        assert_eq!(inner.remaining, 900);
        assert!(!inner.exceeded);
        assert!(inner.allowed);
    }
}
