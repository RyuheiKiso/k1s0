use crate::domain::entity::master_item::{CreateMasterItem, MasterItem, UpdateMasterItem};
use crate::domain::repository::category_repository::CategoryRepository;
use crate::domain::repository::item_repository::ItemRepository;
use crate::domain::repository::version_repository::VersionRepository;
use crate::domain::service::item_domain_service::ItemDomainService;
use crate::domain::service::validation_service::ValidationService;
use crate::usecase::event_publisher::DomainMasterEventPublisher;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ManageItemsUseCase {
    category_repo: Arc<dyn CategoryRepository>,
    item_repo: Arc<dyn ItemRepository>,
    version_repo: Arc<dyn VersionRepository>,
    event_publisher: Arc<dyn DomainMasterEventPublisher>,
}

impl ManageItemsUseCase {
    pub fn new(
        category_repo: Arc<dyn CategoryRepository>,
        item_repo: Arc<dyn ItemRepository>,
        version_repo: Arc<dyn VersionRepository>,
        event_publisher: Arc<dyn DomainMasterEventPublisher>,
    ) -> Self {
        Self {
            category_repo,
            item_repo,
            version_repo,
            event_publisher,
        }
    }

    pub async fn list_items(
        &self,
        category_code: &str,
        active_only: bool,
    ) -> anyhow::Result<Vec<MasterItem>> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;
        self.item_repo
            .find_by_category(category.id, active_only)
            .await
    }

    pub async fn list_items_by_category_id(
        &self,
        category_id: Uuid,
        active_only: bool,
    ) -> anyhow::Result<Vec<MasterItem>> {
        self.category_repo
            .find_by_id(category_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_id))?;
        self.item_repo.find_by_category(category_id, active_only).await
    }

    pub async fn get_item(
        &self,
        category_code: &str,
        item_code: &str,
    ) -> anyhow::Result<Option<MasterItem>> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;
        self.item_repo
            .find_by_category_and_code(category.id, item_code)
            .await
    }

    pub async fn get_item_by_id(&self, item_id: Uuid) -> anyhow::Result<Option<MasterItem>> {
        self.item_repo.find_by_id(item_id).await
    }

    pub async fn create_item(
        &self,
        category_code: &str,
        input: &CreateMasterItem,
        created_by: &str,
    ) -> anyhow::Result<MasterItem> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;

        if let Some(_existing) = self
            .item_repo
            .find_by_category_and_code(category.id, &input.code)
            .await?
        {
            anyhow::bail!(
                "Duplicate code: item '{}' already exists in category '{}'",
                input.code,
                category_code
            );
        }

        ValidationService::validate_item_attributes(&category, &input.attributes)?;

        if let Some(parent_item_id) = input.parent_item_id {
            // 新規アイテムはまだIDが無いため仮UUIDで循環チェックを行う。
            // 実質的には parent_item_id の親チェーンの整合性を検証する。
            let temp_id = Uuid::new_v4();
            ItemDomainService::check_circular_parent(&self.item_repo, temp_id, parent_item_id)
                .await?;
        }

        let item = self.item_repo.create(category.id, input, created_by).await?;

        self.version_repo
            .create(
                item.id,
                1,
                None,
                Some(serde_json::to_value(&item)?),
                created_by,
                Some("Initial creation"),
            )
            .await?;

        self.publish_item_event("created", created_by, category_code, None, Some(&item))
            .await;
        Ok(item)
    }

    pub async fn create_item_by_category_id(
        &self,
        category_id: Uuid,
        input: &CreateMasterItem,
        created_by: &str,
    ) -> anyhow::Result<MasterItem> {
        let category = self
            .category_repo
            .find_by_id(category_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_id))?;
        self.create_item(&category.code, input, created_by).await
    }

    pub async fn update_item(
        &self,
        category_code: &str,
        item_code: &str,
        input: &UpdateMasterItem,
        actor: &str,
    ) -> anyhow::Result<MasterItem> {
        let category = self
            .category_repo
            .find_by_code(category_code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", category_code))?;

        let existing = self
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

        if let Some(parent_item_id) = input.parent_item_id {
            ItemDomainService::check_circular_parent(&self.item_repo, existing.id, parent_item_id)
                .await?;
        }

        if let Some(ref attrs) = input.attributes {
            ValidationService::validate_item_attributes(&category, &Some(attrs.clone()))?;
        }

        let before_data = serde_json::to_value(&existing)?;
        let updated = self.item_repo.update(existing.id, input).await?;
        let after_data = serde_json::to_value(&updated)?;

        let version_number = self
            .version_repo
            .get_latest_version_number(existing.id)
            .await?
            + 1;
        self.version_repo
            .create(
                existing.id,
                version_number,
                Some(before_data),
                Some(after_data),
                actor,
                None,
            )
            .await?;

        self.publish_item_event("updated", actor, category_code, Some(&existing), Some(&updated))
            .await;
        Ok(updated)
    }

    pub async fn update_item_by_id(
        &self,
        item_id: Uuid,
        input: &UpdateMasterItem,
        actor: &str,
    ) -> anyhow::Result<MasterItem> {
        let existing = self
            .item_repo
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item '{}' not found", item_id))?;
        let category = self
            .category_repo
            .find_by_id(existing.category_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", existing.category_id))?;
        self.update_item(&category.code, &existing.code, input, actor)
            .await
    }

    pub async fn delete_item(
        &self,
        category_code: &str,
        item_code: &str,
        actor: &str,
    ) -> anyhow::Result<()> {
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
        self.item_repo.delete(item.id).await?;
        self.publish_item_event("deleted", actor, category_code, Some(&item), None)
            .await;
        Ok(())
    }

    pub async fn delete_item_by_id(&self, item_id: Uuid, actor: &str) -> anyhow::Result<()> {
        let item = self
            .item_repo
            .find_by_id(item_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Item '{}' not found", item_id))?;
        let category = self
            .category_repo
            .find_by_id(item.category_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", item.category_id))?;
        self.item_repo.delete(item.id).await?;
        self.publish_item_event("deleted", actor, &category.code, Some(&item), None)
            .await;
        Ok(())
    }

    async fn publish_item_event(
        &self,
        operation: &str,
        changed_by: &str,
        category_code: &str,
        before: Option<&MasterItem>,
        after: Option<&MasterItem>,
    ) {
        let item = after.or(before);
        let Some(item) = item else {
            return;
        };

        let event_type = format!("item.{}", operation);
        let event = serde_json::json!({
            "event_id": Uuid::new_v4().to_string(),
            "event_type": event_type,
            "category_code": category_code,
            "item_code": item.code,
            "operation": operation.to_uppercase(),
            "before": before,
            "after": after,
            "changed_by": changed_by,
            "trace_id": Uuid::new_v4().to_string(),
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Err(err) = self.event_publisher.publish_item_changed(&event).await {
            tracing::warn!(
                error = %err,
                operation,
                item_code = %item.code,
                "failed to publish item changed event"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::master_category::MasterCategory;
    use crate::domain::entity::master_item_version::MasterItemVersion;
    use crate::domain::repository::category_repository::MockCategoryRepository;
    use crate::domain::repository::item_repository::MockItemRepository;
    use crate::domain::repository::version_repository::MockVersionRepository;
    use crate::usecase::event_publisher::MockDomainMasterEventPublisher;
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
            attributes: Some(serde_json::json!({"symbol": "¥"})),
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

    #[allow(dead_code)]
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

    #[tokio::test]
    async fn test_create_item_success() {
        let category = sample_category();
        let category_id = category.id;
        let item = sample_item(category_id);
        let item_clone = item.clone();
        let category_clone = category.clone();

        let mut mock_cat_repo = MockCategoryRepository::new();
        let mut mock_item_repo = MockItemRepository::new();
        let mut mock_ver_repo = MockVersionRepository::new();
        let mut mock_publisher = MockDomainMasterEventPublisher::new();

        mock_cat_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(category_clone.clone())));

        mock_item_repo
            .expect_find_by_category_and_code()
            .with(eq(category_id), eq("JPY"))
            .times(1)
            .returning(|_, _| Ok(None));

        mock_item_repo
            .expect_create()
            .times(1)
            .returning(move |_, _, _| Ok(item_clone.clone()));

        mock_ver_repo
            .expect_create()
            .times(1)
            .returning(move |item_id, _, _, _, _, _| {
                Ok(MasterItemVersion {
                    id: Uuid::new_v4(),
                    item_id,
                    version_number: 1,
                    before_data: None,
                    after_data: None,
                    changed_by: "admin".to_string(),
                    change_reason: Some("Initial creation".to_string()),
                    created_at: Utc::now(),
                })
            });

        mock_publisher
            .expect_publish_item_changed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = ManageItemsUseCase::new(
            Arc::new(mock_cat_repo),
            Arc::new(mock_item_repo),
            Arc::new(mock_ver_repo),
            Arc::new(mock_publisher),
        );

        let input = CreateMasterItem {
            code: "JPY".to_string(),
            display_name: "Japanese Yen".to_string(),
            description: None,
            attributes: Some(serde_json::json!({"symbol": "¥"})),
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: Some(true),
            sort_order: Some(1),
        };

        let result = uc.create_item("CURRENCY", &input, "admin").await;
        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.code, "JPY");
    }

    #[tokio::test]
    async fn test_create_item_duplicate_code() {
        let category = sample_category();
        let category_id = category.id;
        let existing = sample_item(category_id);
        let category_clone = category.clone();
        let existing_clone = existing.clone();

        let mut mock_cat_repo = MockCategoryRepository::new();
        let mut mock_item_repo = MockItemRepository::new();
        let mock_ver_repo = MockVersionRepository::new();
        let mock_publisher = MockDomainMasterEventPublisher::new();

        mock_cat_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(category_clone.clone())));

        mock_item_repo
            .expect_find_by_category_and_code()
            .with(eq(category_id), eq("JPY"))
            .times(1)
            .returning(move |_, _| Ok(Some(existing_clone.clone())));

        let uc = ManageItemsUseCase::new(
            Arc::new(mock_cat_repo),
            Arc::new(mock_item_repo),
            Arc::new(mock_ver_repo),
            Arc::new(mock_publisher),
        );

        let input = CreateMasterItem {
            code: "JPY".to_string(),
            display_name: "Japanese Yen".to_string(),
            description: None,
            attributes: None,
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: None,
            sort_order: None,
        };

        let result = uc.create_item("CURRENCY", &input, "admin").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate code"));
    }

    #[tokio::test]
    async fn test_create_item_with_validation_failure() {
        let mut category = sample_category();
        category.validation_schema = Some(serde_json::json!({
            "required": ["symbol"],
            "properties": {
                "symbol": { "type": "string", "maxLength": 3 }
            }
        }));
        let category_id = category.id;
        let category_clone = category.clone();

        let mut mock_cat_repo = MockCategoryRepository::new();
        let mut mock_item_repo = MockItemRepository::new();
        let mock_ver_repo = MockVersionRepository::new();
        let mock_publisher = MockDomainMasterEventPublisher::new();

        mock_cat_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(category_clone.clone())));

        mock_item_repo
            .expect_find_by_category_and_code()
            .with(eq(category_id), eq("JPY"))
            .times(1)
            .returning(|_, _| Ok(None));

        let uc = ManageItemsUseCase::new(
            Arc::new(mock_cat_repo),
            Arc::new(mock_item_repo),
            Arc::new(mock_ver_repo),
            Arc::new(mock_publisher),
        );

        // Missing required "symbol" field
        let input = CreateMasterItem {
            code: "JPY".to_string(),
            display_name: "Japanese Yen".to_string(),
            description: None,
            attributes: Some(serde_json::json!({"name": "Yen"})),
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: None,
            sort_order: None,
        };

        let result = uc.create_item("CURRENCY", &input, "admin").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("required field"));
    }

    #[tokio::test]
    async fn test_update_item_success() {
        let category = sample_category();
        let category_id = category.id;
        let existing = sample_item(category_id);
        let existing_id = existing.id;
        let mut updated = existing.clone();
        updated.display_name = "Updated Yen".to_string();
        let category_clone = category.clone();
        let existing_clone = existing.clone();
        let updated_clone = updated.clone();

        let mut mock_cat_repo = MockCategoryRepository::new();
        let mut mock_item_repo = MockItemRepository::new();
        let mut mock_ver_repo = MockVersionRepository::new();
        let mut mock_publisher = MockDomainMasterEventPublisher::new();

        mock_cat_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(category_clone.clone())));

        mock_item_repo
            .expect_find_by_category_and_code()
            .with(eq(category_id), eq("JPY"))
            .times(1)
            .returning(move |_, _| Ok(Some(existing_clone.clone())));

        mock_item_repo
            .expect_update()
            .with(eq(existing_id), always())
            .times(1)
            .returning(move |_, _| Ok(updated_clone.clone()));

        mock_ver_repo
            .expect_get_latest_version_number()
            .with(eq(existing_id))
            .times(1)
            .returning(|_| Ok(1));

        mock_ver_repo
            .expect_create()
            .times(1)
            .returning(move |item_id, ver, _, _, _, _| {
                Ok(MasterItemVersion {
                    id: Uuid::new_v4(),
                    item_id,
                    version_number: ver,
                    before_data: None,
                    after_data: None,
                    changed_by: "admin".to_string(),
                    change_reason: None,
                    created_at: Utc::now(),
                })
            });

        mock_publisher
            .expect_publish_item_changed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = ManageItemsUseCase::new(
            Arc::new(mock_cat_repo),
            Arc::new(mock_item_repo),
            Arc::new(mock_ver_repo),
            Arc::new(mock_publisher),
        );

        let input = UpdateMasterItem {
            display_name: Some("Updated Yen".to_string()),
            description: None,
            attributes: None,
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: None,
            sort_order: None,
        };

        let result = uc.update_item("CURRENCY", "JPY", &input, "admin").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().display_name, "Updated Yen");
    }

    #[tokio::test]
    async fn test_delete_item_success() {
        let category = sample_category();
        let category_id = category.id;
        let item = sample_item(category_id);
        let item_id = item.id;
        let category_clone = category.clone();
        let item_clone = item.clone();

        let mut mock_cat_repo = MockCategoryRepository::new();
        let mut mock_item_repo = MockItemRepository::new();
        let mock_ver_repo = MockVersionRepository::new();
        let mut mock_publisher = MockDomainMasterEventPublisher::new();

        mock_cat_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(category_clone.clone())));

        mock_item_repo
            .expect_find_by_category_and_code()
            .with(eq(category_id), eq("JPY"))
            .times(1)
            .returning(move |_, _| Ok(Some(item_clone.clone())));

        mock_item_repo
            .expect_delete()
            .with(eq(item_id))
            .times(1)
            .returning(|_| Ok(()));

        mock_publisher
            .expect_publish_item_changed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = ManageItemsUseCase::new(
            Arc::new(mock_cat_repo),
            Arc::new(mock_item_repo),
            Arc::new(mock_ver_repo),
            Arc::new(mock_publisher),
        );

        let result = uc.delete_item("CURRENCY", "JPY", "admin").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_item_not_found() {
        let category = sample_category();
        let category_id = category.id;
        let category_clone = category.clone();

        let mut mock_cat_repo = MockCategoryRepository::new();
        let mut mock_item_repo = MockItemRepository::new();
        let mock_ver_repo = MockVersionRepository::new();
        let mock_publisher = MockDomainMasterEventPublisher::new();

        mock_cat_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(category_clone.clone())));

        mock_item_repo
            .expect_find_by_category_and_code()
            .with(eq(category_id), eq("NONEXIST"))
            .times(1)
            .returning(|_, _| Ok(None));

        let uc = ManageItemsUseCase::new(
            Arc::new(mock_cat_repo),
            Arc::new(mock_item_repo),
            Arc::new(mock_ver_repo),
            Arc::new(mock_publisher),
        );

        let result = uc.delete_item("CURRENCY", "NONEXIST", "admin").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_list_items() {
        let category = sample_category();
        let category_id = category.id;
        let item = sample_item(category_id);
        let items = vec![item];
        let category_clone = category.clone();
        let items_clone = items.clone();

        let mut mock_cat_repo = MockCategoryRepository::new();
        let mut mock_item_repo = MockItemRepository::new();
        let mock_ver_repo = MockVersionRepository::new();
        let mock_publisher = MockDomainMasterEventPublisher::new();

        mock_cat_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(category_clone.clone())));

        mock_item_repo
            .expect_find_by_category()
            .with(eq(category_id), eq(false))
            .times(1)
            .returning(move |_, _| Ok(items_clone.clone()));

        let uc = ManageItemsUseCase::new(
            Arc::new(mock_cat_repo),
            Arc::new(mock_item_repo),
            Arc::new(mock_ver_repo),
            Arc::new(mock_publisher),
        );

        let result = uc.list_items("CURRENCY", false).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_create_item_category_not_found() {
        let mut mock_cat_repo = MockCategoryRepository::new();
        let mock_item_repo = MockItemRepository::new();
        let mock_ver_repo = MockVersionRepository::new();
        let mock_publisher = MockDomainMasterEventPublisher::new();

        mock_cat_repo
            .expect_find_by_code()
            .with(eq("NONEXIST"))
            .times(1)
            .returning(|_| Ok(None));

        let uc = ManageItemsUseCase::new(
            Arc::new(mock_cat_repo),
            Arc::new(mock_item_repo),
            Arc::new(mock_ver_repo),
            Arc::new(mock_publisher),
        );

        let input = CreateMasterItem {
            code: "JPY".to_string(),
            display_name: "Japanese Yen".to_string(),
            description: None,
            attributes: None,
            parent_item_id: None,
            effective_from: None,
            effective_until: None,
            is_active: None,
            sort_order: None,
        };

        let result = uc.create_item("NONEXIST", &input, "admin").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
