use std::sync::Arc;

use crate::usecase::evaluate_policy::{EvaluatePolicyError, EvaluatePolicyInput, EvaluatePolicyUseCase};
use crate::usecase::get_policy::{GetPolicyError, GetPolicyUseCase};

// --- gRPC Request/Response Types ---

#[derive(Debug, Clone)]
pub struct EvaluatePolicyRequest {
    pub package_path: String,
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
    pub id: String,
    pub name: String,
    pub description: String,
    pub package_path: String,
    pub rego_content: String,
    pub enabled: bool,
    pub version: u32,
}

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),

    #[error("unimplemented: {0}")]
    Unimplemented(String),
}

// --- PolicyGrpcService ---

pub struct PolicyGrpcService {
    evaluate_policy_uc: Arc<EvaluatePolicyUseCase>,
    get_policy_uc: Arc<GetPolicyUseCase>,
}

impl PolicyGrpcService {
    pub fn new(
        evaluate_policy_uc: Arc<EvaluatePolicyUseCase>,
        get_policy_uc: Arc<GetPolicyUseCase>,
    ) -> Self {
        Self {
            evaluate_policy_uc,
            get_policy_uc,
        }
    }

    pub async fn evaluate_policy(
        &self,
        req: EvaluatePolicyRequest,
    ) -> Result<EvaluatePolicyResponse, GrpcError> {
        let input_json: serde_json::Value = if req.input_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(&req.input_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid input_json: {}", e)))?
        };

        // EvaluatePolicyUseCase requires a policy_id (UUID).
        // The proto uses package_path as the lookup key.
        // Since there is no find_by_package_path in the repository,
        // we return Unimplemented until a proper OPA integration is added.
        let _ = input_json;
        let _ = &self.evaluate_policy_uc;
        Err(GrpcError::Unimplemented(
            "evaluate_policy requires OPA integration; not yet implemented".to_string(),
        ))
    }

    pub async fn get_policy(
        &self,
        req: GetPolicyRequest,
    ) -> Result<GetPolicyResponse, GrpcError> {
        let id = req
            .id
            .parse::<uuid::Uuid>()
            .map_err(|_| GrpcError::InvalidArgument(format!("invalid policy id: {}", req.id)))?;

        match self.get_policy_uc.execute(&id).await {
            Ok(Some(policy)) => Ok(GetPolicyResponse {
                id: policy.id.to_string(),
                name: policy.name,
                description: policy.description,
                package_path: String::new(),
                rego_content: policy.rego_content,
                enabled: policy.enabled,
                version: policy.version,
            }),
            Ok(None) => Err(GrpcError::NotFound(format!("policy not found: {}", req.id))),
            Err(GetPolicyError::Internal(e)) => Err(GrpcError::Internal(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::policy::Policy;
    use crate::domain::repository::policy_repository::MockPolicyRepository;

    fn make_service(mock: MockPolicyRepository) -> PolicyGrpcService {
        let repo = Arc::new(mock);
        PolicyGrpcService::new(
            Arc::new(EvaluatePolicyUseCase::new(repo.clone())),
            Arc::new(GetPolicyUseCase::new(repo)),
        )
    }

    #[tokio::test]
    async fn test_get_policy_success() {
        let mut mock = MockPolicyRepository::new();
        let policy = Policy::new(
            "rbac-policy".to_string(),
            "RBAC policy".to_string(),
            "package authz\ndefault allow = true".to_string(),
        );
        let policy_id = policy.id;
        let return_policy = policy.clone();

        mock.expect_find_by_id()
            .withf(move |id| *id == policy_id)
            .returning(move |_| Ok(Some(return_policy.clone())));

        let svc = make_service(mock);
        let req = GetPolicyRequest {
            id: policy_id.to_string(),
        };
        let resp = svc.get_policy(req).await.unwrap();

        assert_eq!(resp.id, policy_id.to_string());
        assert_eq!(resp.name, "rbac-policy");
        assert!(resp.enabled);
    }

    #[tokio::test]
    async fn test_get_policy_not_found() {
        let mut mock = MockPolicyRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let svc = make_service(mock);
        let req = GetPolicyRequest {
            id: uuid::Uuid::new_v4().to_string(),
        };
        let result = svc.get_policy(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("not found")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_policy_invalid_id() {
        let mock = MockPolicyRepository::new();
        let svc = make_service(mock);
        let req = GetPolicyRequest {
            id: "not-a-uuid".to_string(),
        };
        let result = svc.get_policy(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::InvalidArgument(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_evaluate_policy_unimplemented() {
        let mock = MockPolicyRepository::new();
        let svc = make_service(mock);
        let req = EvaluatePolicyRequest {
            package_path: "k1s0.system.tenant".to_string(),
            input_json: b"{}".to_vec(),
        };
        let result = svc.evaluate_policy(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::Unimplemented(_) => {}
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
