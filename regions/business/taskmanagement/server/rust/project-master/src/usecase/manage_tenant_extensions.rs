// テナント拡張管理ユースケース。
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entity::tenant_project_extension::{
    TenantMergedStatus, TenantProjectExtension, UpsertTenantExtension,
};
use crate::domain::repository::tenant_extension_repository::TenantExtensionRepository;
use crate::usecase::event_publisher::{
    ProjectMasterEventPublisher, TenantExtensionChangedEvent,
};

pub struct ManageTenantExtensionsUseCase {
    repo: Arc<dyn TenantExtensionRepository>,
    publisher: Arc<dyn ProjectMasterEventPublisher>,
}

impl ManageTenantExtensionsUseCase {
    pub fn new(
        repo: Arc<dyn TenantExtensionRepository>,
        publisher: Arc<dyn ProjectMasterEventPublisher>,
    ) -> Self {
        Self { repo, publisher }
    }

    pub async fn get(
        &self,
        tenant_id: &str,
        status_definition_id: Uuid,
    ) -> anyhow::Result<Option<TenantProjectExtension>> {
        self.repo.find(tenant_id, status_definition_id).await
    }

    pub async fn list_merged(
        &self,
        tenant_id: &str,
        project_type_id: Uuid,
        active_only: bool,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<(Vec<TenantMergedStatus>, i64)> {
        let items = self
            .repo
            .list_merged(tenant_id, project_type_id, active_only, limit, offset)
            .await?;
        let total = self.repo.count_merged(tenant_id, project_type_id).await?;
        Ok((items, total))
    }

    pub async fn upsert(
        &self,
        input: &UpsertTenantExtension,
    ) -> anyhow::Result<TenantProjectExtension> {
        let ext = self.repo.upsert(input).await?;
        let event = TenantExtensionChangedEvent {
            tenant_id: ext.tenant_id.clone(),
            status_definition_id: ext.status_definition_id.to_string(),
            change_type: "upserted".to_string(),
        };
        self.publisher
            .publish_tenant_extension_changed(&event)
            .await?;
        Ok(ext)
    }

    pub async fn delete(
        &self,
        tenant_id: &str,
        status_definition_id: Uuid,
    ) -> anyhow::Result<()> {
        self.repo.delete(tenant_id, status_definition_id).await?;
        let event = TenantExtensionChangedEvent {
            tenant_id: tenant_id.to_string(),
            status_definition_id: status_definition_id.to_string(),
            change_type: "deleted".to_string(),
        };
        self.publisher
            .publish_tenant_extension_changed(&event)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::status_definition::StatusDefinition;
    use crate::domain::repository::tenant_extension_repository::MockTenantExtensionRepository;
    use crate::usecase::event_publisher::MockProjectMasterEventPublisher;
    use chrono::Utc;
    use mockall::predicate::*;

    // テスト用テナント拡張のサンプルデータを生成するヘルパー関数
    fn sample_extension(tenant_id: &str, sd_id: Uuid) -> TenantProjectExtension {
        let now = Utc::now();
        TenantProjectExtension {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.to_string(),
            status_definition_id: sd_id,
            display_name_override: Some("カスタム名".to_string()),
            attributes_override: None,
            is_enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    // テスト用マージステータスのサンプルデータを生成するヘルパー関数
    fn sample_merged_status() -> TenantMergedStatus {
        let now = Utc::now();
        let base = StatusDefinition {
            id: Uuid::new_v4(),
            project_type_id: Uuid::new_v4(),
            code: "OPEN".to_string(),
            display_name: "オープン".to_string(),
            description: None,
            color: None,
            allowed_transitions: None,
            is_initial: true,
            is_terminal: false,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: now,
            updated_at: now,
        };
        TenantMergedStatus {
            base_status: base,
            extension: None,
            effective_display_name: "オープン".to_string(),
            effective_attributes: None,
        }
    }

    // テナント拡張取得の正常系：存在するエントリで Some を返すことを確認する
    // 前提: リポジトリが対応するテナント拡張を返す
    // 期待: Ok(Some(TenantProjectExtension)) が返される
    #[tokio::test]
    async fn test_get_found() {
        let mut mock_repo = MockTenantExtensionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let sd_id = Uuid::new_v4();
        let ext = sample_extension("tenant-001", sd_id);
        let ext_clone = ext.clone();
        mock_repo
            .expect_find()
            .with(eq("tenant-001"), eq(sd_id))
            .times(1)
            .returning(move |_, _| Ok(Some(ext_clone.clone())));
        let uc = ManageTenantExtensionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.get("tenant-001", sd_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    // テナント拡張取得の正常系：存在しないエントリで None を返すことを確認する
    // 前提: リポジトリが None を返す
    // 期待: Ok(None) が返される
    #[tokio::test]
    async fn test_get_not_found() {
        let mut mock_repo = MockTenantExtensionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let sd_id = Uuid::new_v4();
        mock_repo
            .expect_find()
            .times(1)
            .returning(|_, _| Ok(None));
        let uc = ManageTenantExtensionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.get("tenant-999", sd_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // マージ一覧取得の正常系：リポジトリから複数件を正しく返すことを確認する
    // 前提: リポジトリが2件のマージステータスを返す
    // 期待: (items, total) のタプルが返され、件数が一致する
    #[tokio::test]
    async fn test_list_merged_success() {
        let mut mock_repo = MockTenantExtensionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let pt_id = Uuid::new_v4();
        let items = vec![sample_merged_status(), sample_merged_status()];
        let items_clone = items.clone();
        mock_repo
            .expect_list_merged()
            .times(1)
            .returning(move |_, _, _, _, _| Ok(items_clone.clone()));
        mock_repo
            .expect_count_merged()
            .times(1)
            .returning(|_, _| Ok(2));
        let uc = ManageTenantExtensionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.list_merged("tenant-001", pt_id, true, 10, 0).await;
        assert!(result.is_ok());
        let (list, total) = result.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(total, 2);
    }

    // アップサート（upsert）の正常系：リポジトリが呼ばれイベントが発行されることを確認する
    // 前提: 有効な UpsertTenantExtension を渡す
    // 期待: Ok(TenantProjectExtension) が返され、upsert と publish が各1回呼ばれる
    #[tokio::test]
    async fn test_upsert_success() {
        let mut mock_repo = MockTenantExtensionRepository::new();
        let mut mock_pub = MockProjectMasterEventPublisher::new();
        let sd_id = Uuid::new_v4();
        let ext = sample_extension("tenant-001", sd_id);
        let ext_clone = ext.clone();
        mock_repo
            .expect_upsert()
            .times(1)
            .returning(move |_| Ok(ext_clone.clone()));
        mock_pub
            .expect_publish_tenant_extension_changed()
            .times(1)
            .returning(|_| Ok(()));
        let uc = ManageTenantExtensionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = UpsertTenantExtension {
            tenant_id: "tenant-001".to_string(),
            status_definition_id: sd_id,
            display_name_override: Some("カスタム名".to_string()),
            attributes_override: None,
            is_enabled: Some(true),
        };
        let result = uc.upsert(&input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().tenant_id, "tenant-001");
    }

    // アップサートの異常系（リポジトリエラー）：DBエラーが伝播することを確認する
    // 前提: リポジトリが Err を返す
    // 期待: ユースケースが Err を返す
    #[tokio::test]
    async fn test_upsert_repository_error() {
        let mut mock_repo = MockTenantExtensionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        mock_repo
            .expect_upsert()
            .times(1)
            .returning(|_| Err(anyhow::anyhow!("DB connection error")));
        let uc = ManageTenantExtensionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let input = UpsertTenantExtension {
            tenant_id: "tenant-001".to_string(),
            status_definition_id: Uuid::new_v4(),
            display_name_override: None,
            attributes_override: None,
            is_enabled: None,
        };
        let result = uc.upsert(&input).await;
        assert!(result.is_err());
    }

    // 削除の正常系：リポジトリが呼ばれイベントが発行されることを確認する
    // 前提: テナントIDとステータス定義IDを渡す
    // 期待: Ok(()) が返され、delete と publish が各1回呼ばれる
    #[tokio::test]
    async fn test_delete_success() {
        let mut mock_repo = MockTenantExtensionRepository::new();
        let mut mock_pub = MockProjectMasterEventPublisher::new();
        let sd_id = Uuid::new_v4();
        mock_repo
            .expect_delete()
            .with(eq("tenant-001"), eq(sd_id))
            .times(1)
            .returning(|_, _| Ok(()));
        mock_pub
            .expect_publish_tenant_extension_changed()
            .times(1)
            .returning(|_| Ok(()));
        let uc = ManageTenantExtensionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.delete("tenant-001", sd_id).await;
        assert!(result.is_ok());
    }

    // 削除の異常系（リポジトリエラー）：DBエラーが伝播することを確認する
    // 前提: リポジトリが Err を返す
    // 期待: ユースケースが Err を返し、publish は呼ばれない
    #[tokio::test]
    async fn test_delete_repository_error() {
        let mut mock_repo = MockTenantExtensionRepository::new();
        let mock_pub = MockProjectMasterEventPublisher::new();
        let sd_id = Uuid::new_v4();
        mock_repo
            .expect_delete()
            .times(1)
            .returning(|_, _| Err(anyhow::anyhow!("DB error")));
        let uc = ManageTenantExtensionsUseCase::new(Arc::new(mock_repo), Arc::new(mock_pub));
        let result = uc.delete("tenant-001", sd_id).await;
        assert!(result.is_err());
    }
}
