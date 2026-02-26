//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の PolicyService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の PolicyGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::policy::v1::{
    policy_service_server::PolicyService, EvaluatePolicyRequest as ProtoEvaluatePolicyRequest,
    EvaluatePolicyResponse as ProtoEvaluatePolicyResponse,
    GetPolicyRequest as ProtoGetPolicyRequest, GetPolicyResponse as ProtoGetPolicyResponse,
    Policy as ProtoPolicy,
};

use super::policy_grpc::{
    EvaluatePolicyRequest, GetPolicyRequest, GrpcError, PolicyGrpcService,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
            GrpcError::Unimplemented(msg) => Status::unimplemented(msg),
        }
    }
}

// --- PolicyService tonic ラッパー ---

/// PolicyServiceTonic は tonic の PolicyService として PolicyGrpcService をラップする。
pub struct PolicyServiceTonic {
    inner: Arc<PolicyGrpcService>,
}

impl PolicyServiceTonic {
    pub fn new(inner: Arc<PolicyGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl PolicyService for PolicyServiceTonic {
    async fn evaluate_policy(
        &self,
        request: Request<ProtoEvaluatePolicyRequest>,
    ) -> Result<Response<ProtoEvaluatePolicyResponse>, Status> {
        let inner = request.into_inner();
        let req = EvaluatePolicyRequest {
            package_path: inner.package_path,
            input_json: inner.input_json,
        };
        let resp = self
            .inner
            .evaluate_policy(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoEvaluatePolicyResponse {
            allowed: resp.allowed,
            package_path: resp.package_path,
            decision_id: resp.decision_id,
            cached: resp.cached,
        }))
    }

    async fn get_policy(
        &self,
        request: Request<ProtoGetPolicyRequest>,
    ) -> Result<Response<ProtoGetPolicyResponse>, Status> {
        let inner = request.into_inner();
        let req = GetPolicyRequest { id: inner.id };
        let resp = self
            .inner
            .get_policy(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_policy = ProtoPolicy {
            id: resp.id,
            name: resp.name,
            description: resp.description,
            package_path: resp.package_path,
            rego_content: resp.rego_content,
            bundle_id: String::new(),
            enabled: resp.enabled,
            version: resp.version,
            created_at: None,
            updated_at: None,
        };

        Ok(Response::new(ProtoGetPolicyResponse {
            policy: Some(proto_policy),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::policy::Policy;
    use crate::domain::repository::policy_repository::MockPolicyRepository;
    use crate::usecase::evaluate_policy::EvaluatePolicyUseCase;
    use crate::usecase::get_policy::GetPolicyUseCase;

    fn make_tonic_service(mock: MockPolicyRepository) -> PolicyServiceTonic {
        let repo = Arc::new(mock);
        let grpc_svc = Arc::new(PolicyGrpcService::new(
            Arc::new(EvaluatePolicyUseCase::new(repo.clone(), None)),
            Arc::new(GetPolicyUseCase::new(repo)),
        ));
        PolicyServiceTonic::new(grpc_svc)
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
        let err = GrpcError::InvalidArgument("invalid policy id".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("invalid policy id"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("database error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("database error"));
    }

    #[test]
    fn test_grpc_error_unimplemented_to_status() {
        let err = GrpcError::Unimplemented("not yet implemented".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Unimplemented);
        assert!(status.message().contains("not yet implemented"));
    }

    #[tokio::test]
    async fn test_policy_service_tonic_get_policy() {
        let mut mock = MockPolicyRepository::new();
        let policy = Policy::new(
            "test-policy".to_string(),
            "Test policy".to_string(),
            "package test\ndefault allow = true".to_string(),
        );
        let policy_id = policy.id;
        let return_policy = policy.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == policy_id)
            .returning(move |_| Ok(Some(return_policy.clone())));

        let tonic_svc = make_tonic_service(mock);
        let req = Request::new(ProtoGetPolicyRequest {
            id: policy_id.to_string(),
        });
        let resp = tonic_svc.get_policy(req).await.unwrap();
        let inner = resp.into_inner();

        let policy_resp = inner.policy.unwrap();
        assert_eq!(policy_resp.id, policy_id.to_string());
        assert_eq!(policy_resp.name, "test-policy");
        assert!(policy_resp.enabled);
    }

    #[tokio::test]
    async fn test_policy_service_tonic_evaluate_policy_no_opa_no_policy_id() {
        let mock = MockPolicyRepository::new();
        let tonic_svc = make_tonic_service(mock);

        let req = Request::new(ProtoEvaluatePolicyRequest {
            package_path: "k1s0.system.tenant".to_string(),
            input_json: b"{}".to_vec(),
        });
        let result = tonic_svc.evaluate_policy(req).await;

        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Internal);
    }
}
