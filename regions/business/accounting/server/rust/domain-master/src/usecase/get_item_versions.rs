use crate::domain::entity::master_item_version::MasterItemVersion;
use crate::domain::repository::category_repository::CategoryRepository;
use crate::domain::repository::item_repository::ItemRepository;
use crate::domain::repository::version_repository::VersionRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetItemVersionsUseCase {
    category_repo: Arc<dyn CategoryRepository>,
    item_repo: Arc<dyn ItemRepository>,
    version_repo: Arc<dyn VersionRepository>,
}

impl GetItemVersionsUseCase {
    pub fn new(
        category_repo: Arc<dyn CategoryRepository>,
        item_repo: Arc<dyn ItemRepository>,
        version_repo: Arc<dyn VersionRepository>,
    ) -> Self {
        Self {
            category_repo,
            item_repo,
            version_repo,
        }
    }

    pub async fn list_versions(
        &self,
        category_code: &str,
        item_code: &str,
    ) -> anyhow::Result<Vec<MasterItemVersion>> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;

        let item = self
            .item_repo
            .find_by_category_and_code(category.id, item_code)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Item '{}' not found in category '{}'",
                    item_code,
                    category_code
                )
            })?;

        self.version_repo.find_by_item(item.id).await
    }

    pub async fn list_versions_by_item_id(
        &self,
        item_id: Uuid,
    ) -> anyhow::Result<Vec<MasterItemVersion>> {
        self.item_repo
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item '{}' not found", item_id))?;
        self.version_repo.find_by_item(item_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::master_category::MasterCategory;
    use crate::domain::entity::master_item::MasterItem;
    use crate::domain::repository::category_repository::MockCategoryRepository;
    use crate::domain::repository::item_repository::MockItemRepository;
    use crate::domain::repository::version_repository::MockVersionRepository;
    use chrono::Utc;
    use mockall::predicate::*;

    fn sample_category() -> MasterCategory {
        MasterCategory {
            id: Uuid::new_v4(),
            code: "CURRENCY".to_string(),
            display_name: "Currency".to_string(),
            description: None,
            validation_schema: None,
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_item(category_id: Uuid) -> MasterItem {
        MasterItem {
            id: Uuid::new_v4(),
            category_id,
            code: "JPY".to_string(),
            display_name: "Japanese Yen".to_string(),
            description: None,
            attributes: None,
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn sample_version(item_id: Uuid) -> MasterItemVersion {
        MasterItemVersion {
            id: Uuid::new_v4(),
            item_id,
            version_number: 1,
            before_data: None,
            after_data: Some(serde_json::json!({"code": "JPY"})),
            changed_by: "admin".to_string(),
            change_reason: Some("Initial creation".to_string()),
            created_at: Utc::now(),
        }
    }

    fn build_usecase(
        category_repo: MockCategoryRepository,
        item_repo: MockItemRepository,
        version_repo: MockVersionRepository,
    ) -> GetItemVersionsUseCase {
        GetItemVersionsUseCase::new(
            Arc::new(category_repo),
            Arc::new(item_repo),
            Arc::new(version_repo),
        )
    }

    #[tokio::test]
    async fn test_list_versions_success() {
        let mut category_repo = MockCategoryRepository::new();
        let mut item_repo = MockItemRepository::new();
        let mut version_repo = MockVersionRepository::new();

        let category = sample_category();
        let category_id = category.id;
        let category_clone = category.clone();

        let item = sample_item(category_id);
        let item_id = item.id;
        let item_clone = item.clone();

        let version = sample_version(item_id);
        let versions = vec![version.clone()];

        category_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(category_clone.clone())));

        item_repo
            .expect_find_by_category_and_code()
            .with(eq(category_id), eq("JPY"))
            .times(1)
            .returning(move |_, _| Ok(Some(item_clone.clone())));

        version_repo
            .expect_find_by_item()
            .with(eq(item_id))
            .times(1)
            .returning(move |_| Ok(versions.clone()));

        let uc = build_usecase(category_repo, item_repo, version_repo);
        let result = uc.list_versions("CURRENCY", "JPY").await;
        assert!(result.is_ok());
        let versions = result.unwrap();
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].version_number, 1);
    }

    #[tokio::test]
    async fn test_category_not_found() {
        let mut category_repo = MockCategoryRepository::new();
        let item_repo = MockItemRepository::new();
        let version_repo = MockVersionRepository::new();

        category_repo
            .expect_find_by_code()
            .with(eq("NONEXIST"))
            .times(1)
            .returning(|_| Ok(None));

        let uc = build_usecase(category_repo, item_repo, version_repo);
        let result = uc.list_versions("NONEXIST", "JPY").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_item_not_found() {
        let mut category_repo = MockCategoryRepository::new();
        let mut item_repo = MockItemRepository::new();
        let version_repo = MockVersionRepository::new();

        let category = sample_category();
        let category_id = category.id;
        let category_clone = category.clone();

        category_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(category_clone.clone())));

        item_repo
            .expect_find_by_category_and_code()
            .with(eq(category_id), eq("NONEXIST"))
            .times(1)
            .returning(|_, _| Ok(None));

        let uc = build_usecase(category_repo, item_repo, version_repo);
        let result = uc.list_versions("CURRENCY", "NONEXIST").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_list_versions_by_item_id() {
        let category_repo = MockCategoryRepository::new();
        let mut item_repo = MockItemRepository::new();
        let mut version_repo = MockVersionRepository::new();

        let item_id = Uuid::new_v4();
        let item = MasterItem {
            id: item_id,
            category_id: Uuid::new_v4(),
            code: "JPY".to_string(),
            display_name: "Japanese Yen".to_string(),
            description: None,
            attributes: None,
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: true,
            sort_order: 1,
            created_by: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let item_clone = item.clone();

        let v1 = sample_version(item_id);
        let mut v2 = sample_version(item_id);
        v2.version_number = 2;
        let versions = vec![v1, v2];

        item_repo
            .expect_find_by_id()
            .with(eq(item_id))
            .times(1)
            .returning(move |_| Ok(Some(item_clone.clone())));

        version_repo
            .expect_find_by_item()
            .with(eq(item_id))
            .times(1)
            .returning(move |_| Ok(versions.clone()));

        let uc = build_usecase(category_repo, item_repo, version_repo);
        let result = uc.list_versions_by_item_id(item_id).await;
        assert!(result.is_ok());
        let versions = result.unwrap();
        assert_eq!(versions.len(), 2);
    }
}
