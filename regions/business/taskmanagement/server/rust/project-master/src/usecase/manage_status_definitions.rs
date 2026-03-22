// ステータス定義管理ユースケース。
// CRUD 操作とバージョン記録、イベント発行を行う。
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::status_definition::{
    CreateStatusDefinition, StatusDefinition, StatusDefinitionFilter, UpdateStatusDefinition,
};
use crate::domain::repository::status_definition_repository::StatusDefinitionRepository;
use crate::domain::service::validation_service::ValidationService;
use crate::usecase::event_publisher::{
    ProjectMasterEventPublisher, StatusDefinitionChangedEvent,
};

pub struct ManageStatusDefinitionsUseCase {
    repo: Arc<dyn StatusDefinitionRepository>,
    publisher: Arc<dyn ProjectMasterEventPublisher>,
}

impl ManageStatusDefinitionsUseCase {
    pub fn new(
        repo: Arc<dyn StatusDefinitionRepository>,
        publisher: Arc<dyn ProjectMasterEventPublisher>,
    ) -> Self {
        Self { repo, publisher }
    }

    pub async fn list(
        &self,
        filter: &StatusDefinitionFilter,
    ) -> anyhow::Result<(Vec<StatusDefinition>, i64)> {
        let items = self.repo.find_all(filter).await?;
        let total = self.repo.count(filter).await?;
        Ok((items, total))
    }

    pub async fn get(&self, id: Uuid) -> anyhow::Result<Option<StatusDefinition>> {
        self.repo.find_by_id(id).await
    }

    pub async fn create(
        &self,
        input: &CreateStatusDefinition,
        created_by: &str,
    ) -> anyhow::Result<StatusDefinition> {
        ValidationService::validate_create_status_definition(input)?;
        let status = self.repo.create(input, created_by).await?;
        let event = StatusDefinitionChangedEvent {
            status_definition_id: status.id.to_string(),
            project_type_id: status.project_type_id.to_string(),
            code: status.code.clone(),
            change_type: "created".to_string(),
            version_number: 1,
        };
        self.publisher
            .publish_status_definition_changed(&event)
            .await?;
        Ok(status)
    }

    pub async fn update(
        &self,
        id: Uuid,
        input: &UpdateStatusDefinition,
        updated_by: &str,
    ) -> anyhow::Result<StatusDefinition> {
        let status = self.repo.update(id, input, updated_by).await?;
        let event = StatusDefinitionChangedEvent {
            status_definition_id: status.id.to_string(),
            project_type_id: status.project_type_id.to_string(),
            code: status.code.clone(),
            change_type: "updated".to_string(),
            version_number: 0,
        };
        self.publisher
            .publish_status_definition_changed(&event)
            .await?;
        Ok(status)
    }

    pub async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        let status = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("status definition not found: {}", id))?;
        self.repo.delete(id).await?;
        let event = StatusDefinitionChangedEvent {
            status_definition_id: status.id.to_string(),
            project_type_id: status.project_type_id.to_string(),
            code: status.code.clone(),
            change_type: "deleted".to_string(),
            version_number: 0,
        };
        self.publisher
            .publish_status_definition_changed(&event)
            .await?;
        Ok(())
    }
}
