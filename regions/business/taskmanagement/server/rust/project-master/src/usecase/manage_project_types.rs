// プロジェクトタイプ管理ユースケース。
// CRUD 操作とイベント発行を行う。
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::project_type::{
    CreateProjectType, ProjectType, ProjectTypeFilter, UpdateProjectType,
};
use crate::domain::repository::project_type_repository::ProjectTypeRepository;
use crate::domain::service::validation_service::ValidationService;
use crate::usecase::event_publisher::{ProjectMasterEventPublisher, ProjectTypeChangedEvent};

pub struct ManageProjectTypesUseCase {
    repo: Arc<dyn ProjectTypeRepository>,
    publisher: Arc<dyn ProjectMasterEventPublisher>,
}

impl ManageProjectTypesUseCase {
    pub fn new(
        repo: Arc<dyn ProjectTypeRepository>,
        publisher: Arc<dyn ProjectMasterEventPublisher>,
    ) -> Self {
        Self { repo, publisher }
    }

    /// プロジェクトタイプ一覧を取得する
    pub async fn list(
        &self,
        filter: &ProjectTypeFilter,
    ) -> anyhow::Result<(Vec<ProjectType>, i64)> {
        let items = self.repo.find_all(filter).await?;
        let total = self.repo.count(filter).await?;
        Ok((items, total))
    }

    /// プロジェクトタイプを ID で取得する
    pub async fn get(&self, id: Uuid) -> anyhow::Result<Option<ProjectType>> {
        self.repo.find_by_id(id).await
    }

    /// プロジェクトタイプを作成する
    pub async fn create(
        &self,
        input: &CreateProjectType,
        created_by: &str,
    ) -> anyhow::Result<ProjectType> {
        ValidationService::validate_create_project_type(input)?;
        let project_type = self.repo.create(input, created_by).await?;
        let event = ProjectTypeChangedEvent {
            project_type_id: project_type.id.to_string(),
            code: project_type.code.clone(),
            change_type: "created".to_string(),
        };
        self.publisher.publish_project_type_changed(&event).await?;
        Ok(project_type)
    }

    /// プロジェクトタイプを更新する
    pub async fn update(
        &self,
        id: Uuid,
        input: &UpdateProjectType,
        updated_by: &str,
    ) -> anyhow::Result<ProjectType> {
        let project_type = self.repo.update(id, input, updated_by).await?;
        let event = ProjectTypeChangedEvent {
            project_type_id: project_type.id.to_string(),
            code: project_type.code.clone(),
            change_type: "updated".to_string(),
        };
        self.publisher.publish_project_type_changed(&event).await?;
        Ok(project_type)
    }

    /// プロジェクトタイプを削除する
    pub async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        // 削除前に取得してイベント用情報を保持する
        let project_type = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("project type not found: {}", id))?;
        self.repo.delete(id).await?;
        let event = ProjectTypeChangedEvent {
            project_type_id: project_type.id.to_string(),
            code: project_type.code.clone(),
            change_type: "deleted".to_string(),
        };
        self.publisher.publish_project_type_changed(&event).await?;
        Ok(())
    }
}
