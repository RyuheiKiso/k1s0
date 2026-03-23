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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::project_type_repository::MockProjectTypeRepository;
    use crate::usecase::event_publisher::MockProjectMasterEventPublisher;
    use chrono::Utc;
    use mockall::predicate::*;

    // テスト用プロジェクトタイプのサンプルデータを生成するヘルパー関数
    fn sample_project_type() -> ProjectType {
        let now = Utc::now();
        ProjectType {
            id: Uuid::new_v4(),
            code: "SOFTWARE".to_string(),
            display_name: "ソフトウェア開発".to_string(),
            description: None,
            default_workflow: None,
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    // 一覧取得の正常系：リポジトリから複数件を正しく返すことを確認する
    // 前提: リポジトリが2件のプロジェクトタイプを返す
    // 期待: (items, total) のタプルが返され、件数が一致する
    #[tokio::test]
    async fn test_list_success() {
        let mut mock_repo = MockProjectTypeRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let pt1 = sample_project_type();
        let pt2 = sample_project_type();
        let items = vec![pt1, pt2];
        let items_clone = items.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok(items_clone.clone()));
        mock_repo
            .expect_count()
            .times(1)
            .returning(|_| Ok(2));
        let uc = ManageProjectTypesUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let filter = ProjectTypeFilter::default();
        let result = uc.list(&filter).await;
        assert!(result.is_ok());
        let (list, total) = result.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(total, 2);
    }

    // ID取得の正常系：存在するIDで Some(ProjectType) を返すことを確認する
    // 前提: リポジトリが指定IDに対応するプロジェクトタイプを返す
    // 期待: Some(ProjectType) が返される
    #[tokio::test]
    async fn test_get_found() {
        let mut mock_repo = MockProjectTypeRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let pt = sample_project_type();
        let pt_clone = pt.clone();
        let id = pt.id;
        mock_repo
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Ok(Some(pt_clone.clone())));
        let uc = ManageProjectTypesUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.get(id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // ID取得の正常系：存在しないIDで None を返すことを確認する
    // 前提: リポジトリが None を返す
    // 期待: Ok(None) が返される
    #[tokio::test]
    async fn test_get_not_found() {
        let mut mock_repo = MockProjectTypeRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let id = Uuid::new_v4();
        mock_repo
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Ok(None));
        let uc = ManageProjectTypesUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.get(id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // 作成の正常系：バリデーション通過後にリポジトリが呼ばれイベントが発行されることを確認する
    // 前提: 有効な CreateProjectType を渡す
    // 期待: Ok(ProjectType) が返され、create と publish が各1回呼ばれる
    #[tokio::test]
    async fn test_create_success() {
        let mut mock_repo = MockProjectTypeRepository::new();
        let mut mock_pub = MockProjectMasterEventPublisher::new();
        let pt = sample_project_type();
        let pt_clone = pt.clone();
        mock_repo
            .expect_create()
            .times(1)
            .returning(move |_, _| Ok(pt_clone.clone()));
        mock_pub
            .expect_publish_project_type_changed()
            .times(1)
            .returning(|_| Ok(()));
        let uc = ManageProjectTypesUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = CreateProjectType {
            code: "SOFTWARE".to_string(),
            display_name: "ソフトウェア開発".to_string(),
            description: None,
            default_workflow: None,
            is_active: Some(true),
            sort_order: Some(1),
        };
        let result = uc.create(&input, "admin").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().code, "SOFTWARE");
    }

    // 作成の異常系（バリデーションエラー）：codeが空の場合にリポジトリが呼ばれないことを確認する
    // 前提: code が空文字列の CreateProjectType を渡す
    // 期待: Err が返され、create と publish は呼ばれない
    #[tokio::test]
    async fn test_create_validation_error_empty_code() {
        let mock_repo = MockProjectTypeRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let uc = ManageProjectTypesUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = CreateProjectType {
            code: "".to_string(),
            display_name: "ソフトウェア開発".to_string(),
            description: None,
            default_workflow: None,
            is_active: None,
            sort_order: None,
        };
        let result = uc.create(&input, "admin").await;
        assert!(result.is_err());
    }

    // 作成の異常系（バリデーションエラー）：display_nameが空の場合にリポジトリが呼ばれないことを確認する
    // 前提: display_name が空文字列の CreateProjectType を渡す
    // 期待: Err が返される
    #[tokio::test]
    async fn test_create_validation_error_empty_display_name() {
        let mock_repo = MockProjectTypeRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let uc = ManageProjectTypesUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = CreateProjectType {
            code: "SOFTWARE".to_string(),
            display_name: "".to_string(),
            description: None,
            default_workflow: None,
            is_active: None,
            sort_order: None,
        };
        let result = uc.create(&input, "admin").await;
        assert!(result.is_err());
    }

    // 作成の異常系（リポジトリエラー）：DBエラーが伝播することを確認する
    // 前提: リポジトリが Err を返す
    // 期待: ユースケースが Err を返す
    #[tokio::test]
    async fn test_create_repository_error() {
        let mut mock_repo = MockProjectTypeRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        mock_repo
            .expect_create()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("DB connection error")));
        let uc = ManageProjectTypesUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = CreateProjectType {
            code: "SOFTWARE".to_string(),
            display_name: "ソフトウェア開発".to_string(),
            description: None,
            default_workflow: None,
            is_active: None,
            sort_order: None,
        };
        let result = uc.create(&input, "admin").await;
        assert!(result.is_err());
    }

    // 更新の正常系：リポジトリが呼ばれイベントが発行されることを確認する
    // 前提: 有効なIDと UpdateProjectType を渡す
    // 期待: Ok(ProjectType) が返され、update と publish が各1回呼ばれる
    #[tokio::test]
    async fn test_update_success() {
        let mut mock_repo = MockProjectTypeRepository::new();
        let mut mock_pub = MockProjectMasterEventPublisher::new();
        let pt = sample_project_type();
        let pt_clone = pt.clone();
        let id = pt.id;
        mock_repo
            .expect_update()
            .with(eq(id), always(), always())
            .times(1)
            .returning(move |_, _, _| Ok(pt_clone.clone()));
        mock_pub
            .expect_publish_project_type_changed()
            .times(1)
            .returning(|_| Ok(()));
        let uc = ManageProjectTypesUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = UpdateProjectType {
            display_name: Some("更新後表示名".to_string()),
            description: None,
            default_workflow: None,
            is_active: None,
            sort_order: None,
        };
        let result = uc.update(id, &input, "admin").await;
        assert!(result.is_ok());
    }

    // 削除の正常系：事前に存在確認してから削除しイベントを発行することを確認する
    // 前提: 指定IDのプロジェクトタイプが存在する
    // 期待: Ok(()) が返され、find_by_id・delete・publish が各1回呼ばれる
    #[tokio::test]
    async fn test_delete_success() {
        let mut mock_repo = MockProjectTypeRepository::new();
        let mut mock_pub = MockProjectMasterEventPublisher::new();
        let pt = sample_project_type();
        let pt_clone = pt.clone();
        let id = pt.id;
        mock_repo
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Ok(Some(pt_clone.clone())));
        mock_repo
            .expect_delete()
            .with(eq(id))
            .times(1)
            .returning(|_| Ok(()));
        mock_pub
            .expect_publish_project_type_changed()
            .times(1)
            .returning(|_| Ok(()));
        let uc = ManageProjectTypesUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.delete(id).await;
        assert!(result.is_ok());
    }

    // 削除の異常系（NotFound）：存在しないIDの場合にエラーになることを確認する
    // 前提: リポジトリが None を返す
    // 期待: Err が返され、delete は呼ばれない
    #[tokio::test]
    async fn test_delete_not_found() {
        let mut mock_repo = MockProjectTypeRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let id = Uuid::new_v4();
        mock_repo
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Ok(None));
        let uc = ManageProjectTypesUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.delete(id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
