use std::sync::Arc;

use crate::domain::entity::workflow_definition::WorkflowDefinition;
use crate::domain::entity::workflow_step::WorkflowStep;
use crate::domain::repository::WorkflowDefinitionRepository;

#[derive(Debug, Clone)]
pub struct UpdateWorkflowInput {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
    pub steps: Option<Vec<WorkflowStep>>,
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateWorkflowError {
    #[error("workflow not found: {0}")]
    NotFound(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub struct UpdateWorkflowUseCase {
    repo: Arc<dyn WorkflowDefinitionRepository>,
}

impl UpdateWorkflowUseCase {
    pub fn new(repo: Arc<dyn WorkflowDefinitionRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(
        &self,
        input: &UpdateWorkflowInput,
    ) -> Result<WorkflowDefinition, UpdateWorkflowError> {
        let mut definition = self
            .repo
            .find_by_id(&input.id)
            .await
            .map_err(|e| UpdateWorkflowError::Internal(e.to_string()))?
            .ok_or_else(|| UpdateWorkflowError::NotFound(input.id.clone()))?;

        if let Some(ref name) = input.name {
            definition.name = name.clone();
        }
        if let Some(ref description) = input.description {
            definition.description = description.clone();
        }
        if let Some(enabled) = input.enabled {
            definition.enabled = enabled;
        }
        if let Some(ref steps) = input.steps {
            definition.steps = steps.clone();
        }
        definition.version += 1;
        definition.updated_at = chrono::Utc::now();

        self.repo
            .update(&definition)
            .await
            .map_err(|e| UpdateWorkflowError::Internal(e.to_string()))?;

        Ok(definition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::workflow_definition_repository::MockWorkflowDefinitionRepository;

    fn existing_def() -> WorkflowDefinition {
        WorkflowDefinition::new(
            "wf_001".to_string(),
            "original".to_string(),
            "original desc".to_string(),
            true,
            vec![],
        )
    }

    #[tokio::test]
    async fn success() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        let def = existing_def();
        mock.expect_find_by_id()
            .returning(move |_| Ok(Some(def.clone())));
        mock.expect_update().returning(|_| Ok(()));

        let uc = UpdateWorkflowUseCase::new(Arc::new(mock));
        let input = UpdateWorkflowInput {
            id: "wf_001".to_string(),
            name: Some("updated".to_string()),
            description: None,
            enabled: Some(false),
            steps: None,
        };
        let result = uc.execute(&input).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.name, "updated");
        assert!(!updated.enabled);
        assert_eq!(updated.version, 2);
    }

    #[tokio::test]
    async fn not_found() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_by_id().returning(|_| Ok(None));

        let uc = UpdateWorkflowUseCase::new(Arc::new(mock));
        let input = UpdateWorkflowInput {
            id: "wf_missing".to_string(),
            name: None,
            description: None,
            enabled: None,
            steps: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            UpdateWorkflowError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn internal_error() {
        let mut mock = MockWorkflowDefinitionRepository::new();
        mock.expect_find_by_id()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = UpdateWorkflowUseCase::new(Arc::new(mock));
        let input = UpdateWorkflowInput {
            id: "wf_001".to_string(),
            name: None,
            description: None,
            enabled: None,
            steps: None,
        };
        let result = uc.execute(&input).await;
        assert!(matches!(
            result.unwrap_err(),
            UpdateWorkflowError::Internal(_)
        ));
    }
}
