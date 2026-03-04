use std::sync::Arc;

use crate::domain::entity::evaluation::EvaluationContext;
use crate::domain::entity::feature_flag::{FlagRule, FlagVariant};
use crate::usecase::create_flag::{CreateFlagError, CreateFlagInput, CreateFlagUseCase};
use crate::usecase::delete_flag::{DeleteFlagError, DeleteFlagUseCase};
use crate::usecase::evaluate_flag::{EvaluateFlagError, EvaluateFlagInput, EvaluateFlagUseCase};
use crate::usecase::get_flag::{GetFlagError, GetFlagUseCase};
use crate::usecase::list_flags::{ListFlagsError, ListFlagsUseCase};
use crate::usecase::update_flag::{UpdateFlagError, UpdateFlagInput, UpdateFlagUseCase};

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
    pub variant: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct GetFlagRequest {
    pub flag_key: String,
}

#[derive(Debug, Clone)]
pub struct GetFlagResponse {
    pub id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<PbFlagVariant>,
    pub rules: Vec<PbFlagRule>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct PbFlagVariant {
    pub name: String,
    pub value: String,
    pub weight: i32,
}

#[derive(Debug, Clone)]
pub struct PbFlagRule {
    pub attribute: String,
    pub operator: String,
    pub value: String,
    pub variant: String,
}

#[derive(Debug, Clone)]
pub struct ListFlagsRequest {}

#[derive(Debug, Clone)]
pub struct ListFlagsResponse {
    pub flags: Vec<GetFlagResponse>,
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
    pub id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<PbFlagVariant>,
    pub rules: Vec<PbFlagRule>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct UpdateFlagRequest {
    pub flag_key: String,
    pub enabled: Option<bool>,
    pub description: Option<String>,
    pub variants: Vec<PbFlagVariant>,
    pub rules: Vec<PbFlagRule>,
}

#[derive(Debug, Clone)]
pub struct UpdateFlagResponse {
    pub id: String,
    pub flag_key: String,
    pub description: String,
    pub enabled: bool,
    pub variants: Vec<PbFlagVariant>,
    pub rules: Vec<PbFlagRule>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct DeleteFlagRequest {
    pub flag_key: String,
}

#[derive(Debug, Clone)]
pub struct DeleteFlagResponse {
    pub success: bool,
    pub message: String,
}

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

pub struct FeatureFlagGrpcService {
    list_flags_uc: Arc<ListFlagsUseCase>,
    evaluate_flag_uc: Arc<EvaluateFlagUseCase>,
    get_flag_uc: Arc<GetFlagUseCase>,
    create_flag_uc: Arc<CreateFlagUseCase>,
    update_flag_uc: Arc<UpdateFlagUseCase>,
    delete_flag_uc: Arc<DeleteFlagUseCase>,
}

impl FeatureFlagGrpcService {
    pub fn new(
        list_flags_uc: Arc<ListFlagsUseCase>,
        evaluate_flag_uc: Arc<EvaluateFlagUseCase>,
        get_flag_uc: Arc<GetFlagUseCase>,
        create_flag_uc: Arc<CreateFlagUseCase>,
        update_flag_uc: Arc<UpdateFlagUseCase>,
        delete_flag_uc: Arc<DeleteFlagUseCase>,
    ) -> Self {
        Self {
            list_flags_uc,
            evaluate_flag_uc,
            get_flag_uc,
            create_flag_uc,
            update_flag_uc,
            delete_flag_uc,
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
                variant: result.variant,
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
                id: flag.id.to_string(),
                flag_key: flag.flag_key,
                description: flag.description,
                enabled: flag.enabled,
                variants: to_pb_variants(&flag.variants),
                rules: to_pb_rules(&flag.rules),
                created_at: flag.created_at,
                updated_at: flag.updated_at,
            }),
            Err(GetFlagError::NotFound(key)) => {
                Err(GrpcError::NotFound(format!("flag not found: {}", key)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn list_flags(
        &self,
        _req: ListFlagsRequest,
    ) -> Result<ListFlagsResponse, GrpcError> {
        let flags = self.list_flags_uc.execute().await.map_err(|e| match e {
            ListFlagsError::Internal(msg) => GrpcError::Internal(msg),
        })?;

        Ok(ListFlagsResponse {
            flags: flags
                .into_iter()
                .map(|flag| GetFlagResponse {
                    id: flag.id.to_string(),
                    flag_key: flag.flag_key,
                    description: flag.description,
                    enabled: flag.enabled,
                    variants: to_pb_variants(&flag.variants),
                    rules: to_pb_rules(&flag.rules),
                    created_at: flag.created_at,
                    updated_at: flag.updated_at,
                })
                .collect(),
        })
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
                id: flag.id.to_string(),
                flag_key: flag.flag_key,
                description: flag.description,
                enabled: flag.enabled,
                variants: to_pb_variants(&flag.variants),
                rules: to_pb_rules(&flag.rules),
                created_at: flag.created_at,
                updated_at: flag.updated_at,
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
            variants: if req.variants.is_empty() {
                None
            } else {
                Some(
                    req.variants
                        .into_iter()
                        .map(|v| FlagVariant {
                            name: v.name,
                            value: v.value,
                            weight: v.weight,
                        })
                        .collect(),
                )
            },
            rules: if req.rules.is_empty() {
                None
            } else {
                Some(
                    req.rules
                        .into_iter()
                        .map(|r| FlagRule {
                            attribute: r.attribute,
                            operator: r.operator,
                            value: r.value,
                            variant: r.variant,
                        })
                        .collect(),
                )
            },
        };

        match self.update_flag_uc.execute(&input).await {
            Ok(flag) => Ok(UpdateFlagResponse {
                id: flag.id.to_string(),
                flag_key: flag.flag_key,
                description: flag.description,
                enabled: flag.enabled,
                variants: to_pb_variants(&flag.variants),
                rules: to_pb_rules(&flag.rules),
                created_at: flag.created_at,
                updated_at: flag.updated_at,
            }),
            Err(UpdateFlagError::NotFound(key)) => {
                Err(GrpcError::NotFound(format!("flag not found: {}", key)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn delete_flag(
        &self,
        req: DeleteFlagRequest,
    ) -> Result<DeleteFlagResponse, GrpcError> {
        let flag = self.get_flag_uc.execute(&req.flag_key).await.map_err(|e| match e {
            GetFlagError::NotFound(key) => GrpcError::NotFound(format!("flag not found: {}", key)),
            _ => GrpcError::Internal(e.to_string()),
        })?;

        self.delete_flag_uc
            .execute(&flag.id)
            .await
            .map_err(|e| match e {
                DeleteFlagError::NotFound(id) => GrpcError::NotFound(format!("flag not found: {}", id)),
                DeleteFlagError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(DeleteFlagResponse {
            success: true,
            message: format!("flag {} deleted", req.flag_key),
        })
    }
}

fn to_pb_variants(variants: &[FlagVariant]) -> Vec<PbFlagVariant> {
    variants
        .iter()
        .map(|v| PbFlagVariant {
            name: v.name.clone(),
            value: v.value.clone(),
            weight: v.weight,
        })
        .collect()
}

fn to_pb_rules(rules: &[FlagRule]) -> Vec<PbFlagRule> {
    rules
        .iter()
        .map(|r| PbFlagRule {
            attribute: r.attribute.clone(),
            operator: r.operator.clone(),
            value: r.value.clone(),
            variant: r.variant.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::flag_audit_log::FlagAuditLog;
    use crate::domain::entity::feature_flag::FeatureFlag;
    use crate::domain::repository::FlagAuditLogRepository;
    use crate::domain::repository::flag_repository::MockFeatureFlagRepository;
    use crate::infrastructure::kafka_producer::NoopFlagEventPublisher;
    use std::collections::HashMap;

    struct NoopAuditRepo;

    #[async_trait::async_trait]
    impl FlagAuditLogRepository for NoopAuditRepo {
        async fn create(&self, _log: &FlagAuditLog) -> anyhow::Result<()> {
            Ok(())
        }

        async fn list_by_flag_id(
            &self,
            _flag_id: &uuid::Uuid,
            _limit: i64,
            _offset: i64,
        ) -> anyhow::Result<Vec<FlagAuditLog>> {
            Ok(vec![])
        }
    }

    fn make_service(mock: MockFeatureFlagRepository) -> FeatureFlagGrpcService {
        let repo = Arc::new(mock);
        let audit_repo = Arc::new(NoopAuditRepo);
        FeatureFlagGrpcService::new(
            Arc::new(ListFlagsUseCase::new(repo.clone())),
            Arc::new(EvaluateFlagUseCase::new(repo.clone())),
            Arc::new(GetFlagUseCase::new(repo.clone())),
            Arc::new(CreateFlagUseCase::new(
                repo.clone(),
                Arc::new(NoopFlagEventPublisher),
                audit_repo.clone(),
            )),
            Arc::new(UpdateFlagUseCase::new(
                repo.clone(),
                Arc::new(NoopFlagEventPublisher),
                audit_repo.clone(),
            )),
            Arc::new(DeleteFlagUseCase::new(
                repo,
                Arc::new(NoopFlagEventPublisher),
                audit_repo,
            )),
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
        assert_eq!(resp.variant, Some("on".to_string()));
        assert_eq!(resp.flag_key, "dark-mode");
    }

    #[tokio::test]
    async fn test_list_flags_success() {
        let mut mock = MockFeatureFlagRepository::new();
        mock.expect_find_all().returning(|| Ok(vec![]));

        let svc = make_service(mock);
        let resp = svc.list_flags(ListFlagsRequest {}).await.unwrap();
        assert!(resp.flags.is_empty());
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
            variants: vec![],
            rules: vec![],
        };
        let resp = svc.update_flag(req).await.unwrap();

        assert_eq!(resp.flag_key, "dark-mode");
        assert!(!resp.enabled);
    }
}
