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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::repository::status_definition_repository::MockStatusDefinitionRepository;
    use crate::usecase::event_publisher::MockProjectMasterEventPublisher;
    use chrono::Utc;
    use mockall::predicate::*;

    // テスト用ステータス定義のサンプルデータを生成するヘルパー関数
    fn sample_status_definition() -> StatusDefinition {
        let now = Utc::now();
        StatusDefinition {
            id: Uuid::new_v4(),
            project_type_id: Uuid::new_v4(),
            code: "OPEN".to_string(),
            display_name: "オープン".to_string(),
            description: None,
            color: Some("#00FF00".to_string()),
            allowed_transitions: None,
            is_initial: true,
            is_terminal: false,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    // 一覧取得の正常系：リポジトリから複数件を正しく返すことを確認する
    // 前提: リポジトリが2件のステータス定義を返す
    // 期待: (items, total) のタプルが返され、件数が一致する
    #[tokio::test]
    async fn test_list_success() {
        let mut mock_repo = MockStatusDefinitionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let sd1 = sample_status_definition();
        let sd2 = sample_status_definition();
        let items = vec![sd1, sd2];
        let items_clone = items.clone();
        mock_repo
            .expect_find_all()
            .times(1)
            .returning(move |_| Ok(items_clone.clone()));
        mock_repo
            .expect_count()
            .times(1)
            .returning(|_| Ok(2));
        let uc = ManageStatusDefinitionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let filter = StatusDefinitionFilter::default();
        let result = uc.list(&filter).await;
        assert!(result.is_ok());
        let (list, total) = result.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(total, 2);
    }

    // ID取得の正常系：存在するIDで Some を返すことを確認する
    // 前提: リポジトリが指定IDに対応するステータス定義を返す
    // 期待: Ok(Some(StatusDefinition)) が返される
    #[tokio::test]
    async fn test_get_found() {
        let mut mock_repo = MockStatusDefinitionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let sd = sample_status_definition();
        let sd_clone = sd.clone();
        let id = sd.id;
        mock_repo
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Ok(Some(sd_clone.clone())));
        let uc = ManageStatusDefinitionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.get(id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // ID取得の正常系：存在しないIDで None を返すことを確認する
    // 前提: リポジトリが None を返す
    // 期待: Ok(None) が返される
    #[tokio::test]
    async fn test_get_not_found() {
        let mut mock_repo = MockStatusDefinitionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let id = Uuid::new_v4();
        mock_repo
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Ok(None));
        let uc = ManageStatusDefinitionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.get(id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // 作成の正常系：バリデーション通過後にリポジトリが呼ばれイベントが発行されることを確認する
    // 前提: 有効な CreateStatusDefinition を渡す
    // 期待: Ok(StatusDefinition) が返され、create と publish が各1回呼ばれる
    #[tokio::test]
    async fn test_create_success() {
        let mut mock_repo = MockStatusDefinitionRepository::new();
        let mut mock_pub = MockProjectMasterEventPublisher::new();
        let sd = sample_status_definition();
        let sd_clone = sd.clone();
        mock_repo
            .expect_create()
            .times(1)
            .returning(move |_, _| Ok(sd_clone.clone()));
        mock_pub
            .expect_publish_status_definition_changed()
            .times(1)
            .returning(|_| Ok(()));
        let uc = ManageStatusDefinitionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = CreateStatusDefinition {
            project_type_id: Uuid::new_v4(),
            code: "OPEN".to_string(),
            display_name: "オープン".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: Some(true),
            is_terminal: Some(false),
            sort_order: Some(1),
        };
        let result = uc.create(&input, "admin").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().code, "OPEN");
    }

    // 作成の異常系（バリデーションエラー）：codeが空の場合にリポジトリが呼ばれないことを確認する
    // 前提: code が空文字列の CreateStatusDefinition を渡す
    // 期待: Err が返され、create は呼ばれない
    #[tokio::test]
    async fn test_create_validation_error_empty_code() {
        let mock_repo = MockStatusDefinitionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let uc = ManageStatusDefinitionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = CreateStatusDefinition {
            project_type_id: Uuid::new_v4(),
            code: "".to_string(),
            display_name: "オープン".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: None,
            is_terminal: None,
            sort_order: None,
        };
        let result = uc.create(&input, "admin").await;
        assert!(result.is_err());
    }

    // 作成の異常系（バリデーションエラー）：display_nameが空の場合にリポジトリが呼ばれないことを確認する
    // 前提: display_name が空文字列の CreateStatusDefinition を渡す
    // 期待: Err が返される
    #[tokio::test]
    async fn test_create_validation_error_empty_display_name() {
        let mock_repo = MockStatusDefinitionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let uc = ManageStatusDefinitionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = CreateStatusDefinition {
            project_type_id: Uuid::new_v4(),
            code: "OPEN".to_string(),
            display_name: "".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: None,
            is_terminal: None,
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
        let mut mock_repo = MockStatusDefinitionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        mock_repo
            .expect_create()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("DB connection error")));
        let uc = ManageStatusDefinitionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = CreateStatusDefinition {
            project_type_id: Uuid::new_v4(),
            code: "OPEN".to_string(),
            display_name: "オープン".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: None,
            is_terminal: None,
            sort_order: None,
        };
        let result = uc.create(&input, "admin").await;
        assert!(result.is_err());
    }

    // 更新の正常系：リポジトリが呼ばれイベントが発行されることを確認する
    // 前提: 有効なIDと UpdateStatusDefinition を渡す
    // 期待: Ok(StatusDefinition) が返され、update と publish が各1回呼ばれる
    #[tokio::test]
    async fn test_update_success() {
        let mut mock_repo = MockStatusDefinitionRepository::new();
        let mut mock_pub = MockProjectMasterEventPublisher::new();
        let sd = sample_status_definition();
        let sd_clone = sd.clone();
        let id = sd.id;
        mock_repo
            .expect_update()
            .with(eq(id), always(), always())
            .times(1)
            .returning(move |_, _, _| Ok(sd_clone.clone()));
        mock_pub
            .expect_publish_status_definition_changed()
            .times(1)
            .returning(|_| Ok(()));
        let uc = ManageStatusDefinitionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = UpdateStatusDefinition {
            display_name: Some("更新後表示名".to_string()),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: None,
            is_terminal: None,
            sort_order: None,
            change_reason: Some("仕様変更".to_string()),
        };
        let result = uc.update(id, &input, "admin").await;
        assert!(result.is_ok());
    }

    // 削除の正常系：事前に存在確認してから削除しイベントを発行することを確認する
    // 前提: 指定IDのステータス定義が存在する
    // 期待: Ok(()) が返され、find_by_id・delete・publish が各1回呼ばれる
    #[tokio::test]
    async fn test_delete_success() {
        let mut mock_repo = MockStatusDefinitionRepository::new();
        let mut mock_pub = MockProjectMasterEventPublisher::new();
        let sd = sample_status_definition();
        let sd_clone = sd.clone();
        let id = sd.id;
        mock_repo
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(move |_| Ok(Some(sd_clone.clone())));
        mock_repo
            .expect_delete()
            .with(eq(id))
            .times(1)
            .returning(|_| Ok(()));
        mock_pub
            .expect_publish_status_definition_changed()
            .times(1)
            .returning(|_| Ok(()));
        let uc = ManageStatusDefinitionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.delete(id).await;
        assert!(result.is_ok());
    }

    // 削除の異常系（NotFound）：存在しないIDの場合にエラーになることを確認する
    // 前提: リポジトリが None を返す
    // 期待: Err が返され、delete は呼ばれない
    #[tokio::test]
    async fn test_delete_not_found() {
        let mut mock_repo = MockStatusDefinitionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let id = Uuid::new_v4();
        mock_repo
            .expect_find_by_id()
            .with(eq(id))
            .times(1)
            .returning(|_| Ok(None));
        let uc = ManageStatusDefinitionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.delete(id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
