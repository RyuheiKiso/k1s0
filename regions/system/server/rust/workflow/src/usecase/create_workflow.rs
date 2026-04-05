use std::sync::Arc;

use crate::domain::entity::workflow_definition::WorkflowDefinition;
use crate::domain::entity::workflow_step::WorkflowStep;
use crate::domain::repository::WorkflowDefinitionRepository;

// RUST-CRIT-001 対応: テナント分離のため tenant_id フィールドを追加する
#[derive(Debug, Clone)]
pub struct CreateWorkflowInput {
    pub tenant_id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateWorkflowError {
    #[error("workflow already exists: {0}")]
    AlreadyExists(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct CreateWorkflowUseCase {
    repo: Arc<dyn WorkflowDefinitionRepository>,
}

impl CreateWorkflowUseCase {
    pub fn new(repo: Arc<dyn WorkflowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &CreateWorkflowInput,
    ) -> Result<WorkflowDefinition, CreateWorkflowError> {
        if input.name.is_empty() {
            return Err(CreateWorkflowError::Validation(
                "name is required".to_string(),
            ));
        }

        if input.steps.is_empty() {
            return Err(CreateWorkflowError::Validation(
                "at least one step is required".to_string(),
            ));
        }

        // テナント分離: tenant_id を渡してRLSによるフィルタリングを有効化する
        let existing = self
            .repo
            .find_by_name(&input.tenant_id, &input.name)
            .await
            .map_err(|e| CreateWorkflowError::Internal(e.to_string()))?;
        if existing.is_some() {
            return Err(CreateWorkflowError::AlreadyExists(input.name.clone()));
        }

        let id = format!("wf_{}", uuid::Uuid::new_v4().simple());
        let definition = WorkflowDefinition::new(
            id,
            input.name.clone(),
            input.description.clone(),
            input.enabled,
            input.steps.clone(),
        );

        self.repo
            .create(&input.tenant_id, &definition)
            .await
            .map_err(|e| CreateWorkflowError::Internal(e.to_string()))?;

        Ok(definition)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;

    fn sample_steps() -> Vec<WorkflowStep> {
        vec![WorkflowStep::new(
            "step-1".to_string(),
            "Approval".to_string(),
            "human_task".to_string(),
            Some("manager".to_string()),
            Some(48),
            Some("end".to_string()),
            Some("end".to_string()),
        )]
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_by_name().returning(|_, _| Ok(None));
        mock.expect_create().returning(|_, _| Ok(()));

        let uc = CreateWorkflowUseCase::new(Arc::new(mock));
        let input = CreateWorkflowInput {
            tenant_id: "test-tenant".to_string(),
            name: "purchase-approval".to_string(),
            description: "Purchase flow".to_string(),
            enabled: true,
            steps: sample_steps(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let def = result.unwrap();
        assert_eq!(def.name, "purchase-approval");
        assert_eq!(def.version, 1);
        assert!(def.enabled);
    }

    #[tokio::test]
    async fn already_exists() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_by_name().returning(|_, _| {
            Ok(Some(WorkflowDefinition::new(
                "wf_existing".to_string(),
                "purchase-approval".to_string(),
                "".to_string(),
                true,
                vec![],
            )))
        });

        let uc = CreateWorkflowUseCase::new(Arc::new(mock));
        let input = CreateWorkflowInput {
            tenant_id: "test-tenant".to_string(),
            name: "purchase-approval".to_string(),
            description: "".to_string(),
            enabled: true,
            steps: sample_steps(),
        };
        let result = uc.execute(&input).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateWorkflowError::AlreadyExists(_)
        ));
    }

    #[tokio::test]
    async fn validation_empty_name() {
        let mock = MockWorkflowDefinitionRepository::new();
        let uc = CreateWorkflowUseCase::new(Arc::new(mock));
        let input = CreateWorkflowInput {
            tenant_id: "test-tenant".to_string(),
            name: "".to_string(),
            description: "".to_string(),
            enabled: true,
            steps: sample_steps(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            CreateWorkflowError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_by_name()
            .returning(|_, _| Err(anyhow::anyhow!("db error")));

        let uc = CreateWorkflowUseCase::new(Arc::new(mock));
        let input = CreateWorkflowInput {
            tenant_id: "test-tenant".to_string(),
            name: "test".to_string(),
            description: "".to_string(),
            enabled: true,
            steps: sample_steps(),
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            CreateWorkflowError::Internal(_)
        ));
    }
}
