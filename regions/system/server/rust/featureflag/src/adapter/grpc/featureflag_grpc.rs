use std::sync::Arc;

use crate::usecase::create_flag::{CreateFlagError, CreateFlagInput, CreateFlagUseCase};
use crate::usecase::evaluate_flag::{EvaluateFlagError, EvaluateFlagInput, EvaluateFlagUseCase};
use crate::usecase::get_flag::{GetFlagError, GetFlagUseCase};
use crate::usecase::update_flag::{UpdateFlagError, UpdateFlagInput, UpdateFlagUseCase};

use crate::domain::entity::evaluation::EvaluationContext;
use crate::domain::entity::feature_flag::FlagVariant;

// --- gRPC Request/Response Types ---

#[derive(Debug, Clone)]
pub struct EvaluateFlagRequest {
    pub flag_key: String,
    pub user_id: String,
    pub tenant_id: String,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EvaluateFlagResponse {
    pub flag_key: String,
    pub enabled: bool,
    pub variant: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct GetFlagRequest {
    pub flag_key: String,
}

#[derive(Debug, Clone)]
pub struct GetFlagResponse {
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<PbFlagVariant>,
}

#[derive(Debug, Clone)]
pub struct PbFlagVariant {
    pub name: String,
    pub value: String,
    pub weight: i32,
}

#[derive(Debug, Clone)]
pub struct CreateFlagRequest {
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<PbFlagVariant>,
}

#[derive(Debug, Clone)]
pub struct CreateFlagResponse {
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateFlagRequest {
    pub flag_key: String,
    pub enabled: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateFlagResponse {
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
}

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- FeatureFlagGrpcService ---

pub struct FeatureFlagGrpcService {
    evaluate_flag_uc: Arc<EvaluateFlagUseCase>,
    get_flag_uc: Arc<GetFlagUseCase>,
    create_flag_uc: Arc<CreateFlagUseCase>,
    update_flag_uc: Arc<UpdateFlagUseCase>,
}

impl FeatureFlagGrpcService {
    pub fn new(
        evaluate_flag_uc: Arc<EvaluateFlagUseCase>,
        get_flag_uc: Arc<GetFlagUseCase>,
        create_flag_uc: Arc<CreateFlagUseCase>,
        update_flag_uc: Arc<UpdateFlagUseCase>,
    ) -> Self {
        Self {
            evaluate_flag_uc,
            get_flag_uc,
            create_flag_uc,
            update_flag_uc,
        }
    }

    pub async fn evaluate_flag(
        &self,
        req: EvaluateFlagRequest,
    ) -> Result<EvaluateFlagResponse, GrpcError> {
        let input = EvaluateFlagInput {
            flag_key: req.flag_key,
            context: EvaluationContext {
                user_id: if req.user_id.is_empty() {
                    None
                } else {
                    Some(req.user_id)
                },
                tenant_id: if req.tenant_id.is_empty() {
                    None
                } else {
                    Some(req.tenant_id)
                },
                attributes: req.attributes,
            },
        };

        match self.evaluate_flag_uc.execute(&input).await {
            Ok(result) => Ok(EvaluateFlagResponse {
                flag_key: result.flag_key,
                enabled: result.enabled,
                variant: result.variant.unwrap_or_default(),
                reason: result.reason,
            }),
            Err(EvaluateFlagError::FlagNotFound(key)) => {
                Err(GrpcError::NotFound(format!("flag not found: {}", key)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn get_flag(&self, req: GetFlagRequest) -> Result<GetFlagResponse, GrpcError> {
        match self.get_flag_uc.execute(&req.flag_key).await {
            Ok(flag) => Ok(GetFlagResponse {
                flag_key: flag.flag_key,
                description: flag.description,
                enabled: flag.enabled,
                variants: flag
                    .variants
                    .iter()
                    .map(|v| PbFlagVariant {
                        name: v.name.clone(),
                        value: v.value.clone(),
                        weight: v.weight,
                    })
                    .collect(),
            }),
            Err(GetFlagError::NotFound(key)) => {
                Err(GrpcError::NotFound(format!("flag not found: {}", key)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn create_flag(
        &self,
        req: CreateFlagRequest,
    ) -> Result<CreateFlagResponse, GrpcError> {
        let input = CreateFlagInput {
            flag_key: req.flag_key,
            description: req.description,
            enabled: req.enabled,
            variants: req
                .variants
                .into_iter()
                .map(|v| FlagVariant {
                    name: v.name,
                    value: v.value,
                    weight: v.weight,
                })
                .collect(),
        };

        match self.create_flag_uc.execute(&input).await {
            Ok(flag) => Ok(CreateFlagResponse {
                flag_key: flag.flag_key,
                description: flag.description,
                enabled: flag.enabled,
            }),
            Err(CreateFlagError::AlreadyExists(key)) => {
                Err(GrpcError::AlreadyExists(format!("flag already exists: {}", key)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn update_flag(
        &self,
        req: UpdateFlagRequest,
    ) -> Result<UpdateFlagResponse, GrpcError> {
        let input = UpdateFlagInput {
            flag_key: req.flag_key,
            enabled: req.enabled,
            description: req.description,
        };

        match self.update_flag_uc.execute(&input).await {
            Ok(flag) => Ok(UpdateFlagResponse {
                flag_key: flag.flag_key,
                description: flag.description,
                enabled: flag.enabled,
            }),
            Err(UpdateFlagError::NotFound(key)) => {
                Err(GrpcError::NotFound(format!("flag not found: {}", key)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::feature_flag::FeatureFlag;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use std::collections::HashMap;

    fn make_service(mock: MockFeatureFlagRepository) -> FeatureFlagGrpcService {
        let repo = Arc::new(mock);
        FeatureFlagGrpcService::new(
            Arc::new(EvaluateFlagUseCase::new(repo.clone())),
            Arc::new(GetFlagUseCase::new(repo.clone())),
            Arc::new(CreateFlagUseCase::new(repo.clone())),
            Arc::new(UpdateFlagUseCase::new(repo)),
        )
    }

    #[tokio::test]
    async fn test_evaluate_flag_success() {
        let mut mock = MockFeatureFlagRepository::new();
        let mut flag = FeatureFlag::new("dark-mode".to_string(), "Dark mode".to_string(), true);
        flag.variants.push(crate::domain::entity::feature_flag::FlagVariant {
            name: "on".to_string(),
            value: "true".to_string(),
            weight: 100,
        });
        let return_flag = flag.clone();

        mock.expect_find_by_key()
            .withf(|key| key == "dark-mode")
            .returning(move |_| Ok(return_flag.clone()));

        let svc = make_service(mock);
        let req = EvaluateFlagRequest {
            flag_key: "dark-mode".to_string(),
            user_id: "user-1".to_string(),
            tenant_id: String::new(),
            attributes: HashMap::new(),
        };
        let resp = svc.evaluate_flag(req).await.unwrap();

        assert!(resp.enabled);
        assert_eq!(resp.variant, "on");
        assert_eq!(resp.flag_key, "dark-mode");
    }

    #[tokio::test]
    async fn test_evaluate_flag_not_found() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_find_by_key()
            .returning(|_| Err(anyhow::anyhow!("flag not found")));

        let svc = make_service(mock);
        let req = EvaluateFlagRequest {
            flag_key: "nonexistent".to_string(),
            user_id: String::new(),
            tenant_id: String::new(),
            attributes: HashMap::new(),
        };
        let result = svc.evaluate_flag(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("not found")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_flag_success() {
        let mut mock = MockFeatureFlagRepository::new();
        let flag = FeatureFlag::new("beta".to_string(), "Beta feature".to_string(), true);
        let return_flag = flag.clone();

        mock.expect_find_by_key()
            .withf(|key| key == "beta")
            .returning(move |_| Ok(return_flag.clone()));

        let svc = make_service(mock);
        let req = GetFlagRequest {
            flag_key: "beta".to_string(),
        };
        let resp = svc.get_flag(req).await.unwrap();

        assert_eq!(resp.flag_key, "beta");
        assert!(resp.enabled);
    }

    #[tokio::test]
    async fn test_create_flag_success() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_exists_by_key()
            .withf(|key| key == "new-flag")
            .returning(|_| Ok(false));
        mock.expect_create().returning(|_| Ok(()));

        let svc = make_service(mock);
        let req = CreateFlagRequest {
            flag_key: "new-flag".to_string(),
            description: "New flag".to_string(),
            enabled: true,
            variants: vec![],
        };
        let resp = svc.create_flag(req).await.unwrap();

        assert_eq!(resp.flag_key, "new-flag");
        assert!(resp.enabled);
    }

    #[tokio::test]
    async fn test_create_flag_already_exists() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_exists_by_key()
            .returning(|_| Ok(true));

        let svc = make_service(mock);
        let req = CreateFlagRequest {
            flag_key: "existing".to_string(),
            description: "Existing".to_string(),
            enabled: true,
            variants: vec![],
        };
        let result = svc.create_flag(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::AlreadyExists(msg) => assert!(msg.contains("already exists")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_flag_success() {
        let mut mock = MockFeatureFlagRepository::new();
        let flag = FeatureFlag::new("dark-mode".to_string(), "Dark mode".to_string(), true);
        let return_flag = flag.clone();

        mock.expect_find_by_key()
            .withf(|key| key == "dark-mode")
            .returning(move |_| Ok(return_flag.clone()));
        mock.expect_update().returning(|_| Ok(()));

        let svc = make_service(mock);
        let req = UpdateFlagRequest {
            flag_key: "dark-mode".to_string(),
            enabled: Some(false),
            description: None,
        };
        let resp = svc.update_flag(req).await.unwrap();

        assert_eq!(resp.flag_key, "dark-mode");
        assert!(!resp.enabled);
    }
}
