use crate::domain::entity::master_category::{
    CreateMasterCategory, MasterCategory, UpdateMasterCategory,
};
use crate::domain::repository::category_repository::CategoryRepository;
use crate::domain::service::category_service::CategoryService;
use crate::usecase::event_publisher::DomainMasterEventPublisher;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ManageCategoriesUseCase {
    category_repo: Arc<dyn CategoryRepository>,
    event_publisher: Arc<dyn DomainMasterEventPublisher>,
}

impl ManageCategoriesUseCase {
    pub fn new(
        category_repo: Arc<dyn CategoryRepository>,
        event_publisher: Arc<dyn DomainMasterEventPublisher>,
    ) -> Self {
        Self {
            category_repo,
            event_publisher,
        }
    }

    pub async fn list_categories(&self, active_only: bool) -> anyhow::Result<Vec<MasterCategory>> {
        self.category_repo.find_all(active_only).await
    }

    pub async fn get_category(&self, code: &str) -> anyhow::Result<Option<MasterCategory>> {
        self.category_repo.find_by_code(code).await
    }

    pub async fn get_category_by_id(&self, id: Uuid) -> anyhow::Result<Option<MasterCategory>> {
        self.category_repo.find_by_id(id).await
    }

    pub async fn create_category(
        &self,
        input: &CreateMasterCategory,
        created_by: &str,
    ) -> anyhow::Result<MasterCategory> {
        CategoryService::validate_code(&input.code)?;
        if let Some(ref schema) = input.validation_schema {
            CategoryService::validate_schema(schema)?;
        }
        if let Some(_existing) = self.category_repo.find_by_code(&input.code).await? {
            anyhow::bail!("Duplicate code: category '{}' already exists", input.code);
        }
        let category = self.category_repo.create(input, created_by).await?;
        self.publish_category_event("created", created_by, None, Some(&category))
            .await;
        Ok(category)
    }

    pub async fn update_category(
        &self,
        code: &str,
        updated_by: &str,
        input: &UpdateMasterCategory,
    ) -> anyhow::Result<MasterCategory> {
        if let Some(ref schema) = input.validation_schema {
            CategoryService::validate_schema(schema)?;
        }
        let existing = self
            .category_repo
            .find_by_code(code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", code))?;
        let updated = self.category_repo.update(code, input).await?;
        self.publish_category_event("updated", updated_by, Some(&existing), Some(&updated))
            .await;
        Ok(updated)
    }

    pub async fn update_category_by_id(
        &self,
        id: Uuid,
        updated_by: &str,
        input: &UpdateMasterCategory,
    ) -> anyhow::Result<MasterCategory> {
        let category = self
            .category_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", id))?;
        self.update_category(&category.code, updated_by, input).await
    }

    pub async fn delete_category(&self, code: &str, deleted_by: &str) -> anyhow::Result<()> {
        let existing = self
            .category_repo
            .find_by_code(code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", code))?;
        self.category_repo.delete(code).await?;
        self.publish_category_event("deleted", deleted_by, Some(&existing), None)
            .await;
        Ok(())
    }

    pub async fn delete_category_by_id(
        &self,
        id: Uuid,
        deleted_by: &str,
    ) -> anyhow::Result<()> {
        let category = self
            .category_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", id))?;
        self.delete_category(&category.code, deleted_by).await
    }

    async fn publish_category_event(
        &self,
        operation: &str,
        changed_by: &str,
        before: Option<&MasterCategory>,
        after: Option<&MasterCategory>,
    ) {
        let category = after.or(before);
        let Some(category) = category else {
            return;
        };

        let event_type = format!("category.{}", operation);
        let event = serde_json::json!({
            "event_id": Uuid::new_v4().to_string(),
            "event_type": event_type,
            "category_code": category.code,
            "operation": operation.to_uppercase(),
            "before": before,
            "after": after,
            "changed_by": changed_by,
            "trace_id": Uuid::new_v4().to_string(),
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Err(err) = self.event_publisher.publish_category_changed(&event).await {
            tracing::warn!(
                error = %err,
                operation,
                category_code = %category.code,
                "failed to publish category changed event"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::category_repository::MockCategoryRepository;
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

    #[tokio::test]
    async fn test_create_category_success() {
        let mut mock_repo = MockCategoryRepository::new();
        let mut mock_publisher = MockDomainMasterEventPublisher::new();

        let expected = sample_category();
        let expected_clone = expected.clone();

        mock_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(|_| Ok(None));

        mock_repo
            .expect_create()
            .times(1)
            .returning(move |_, _| Ok(expected_clone.clone()));

        mock_publisher
            .expect_publish_category_changed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = ManageCategoriesUseCase::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
        );

        let input = CreateMasterCategory {
            code: "CURRENCY".to_string(),
            display_name: "Currency".to_string(),
            description: None,
            validation_schema: None,
            is_active: Some(true),
            sort_order: Some(1),
        };

        let result = uc.create_category(&input, "admin").await;
        assert!(result.is_ok());
        let cat = result.unwrap();
        assert_eq!(cat.code, "CURRENCY");
    }

    #[tokio::test]
    async fn test_create_category_duplicate_code() {
        let mut mock_repo = MockCategoryRepository::new();
        let mock_publisher = MockDomainMasterEventPublisher::new();

        let existing = sample_category();

        mock_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(existing.clone())));

        let uc = ManageCategoriesUseCase::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
        );

        let input = CreateMasterCategory {
            code: "CURRENCY".to_string(),
            display_name: "Currency".to_string(),
            description: None,
            validation_schema: None,
            is_active: None,
            sort_order: None,
        };

        let result = uc.create_category(&input, "admin").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate code"));
    }

    #[tokio::test]
    async fn test_delete_category_success() {
        let mut mock_repo = MockCategoryRepository::new();
        let mut mock_publisher = MockDomainMasterEventPublisher::new();

        let existing = sample_category();
        let existing_clone = existing.clone();

        mock_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(existing_clone.clone())));

        mock_repo
            .expect_delete()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(|_| Ok(()));

        mock_publisher
            .expect_publish_category_changed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = ManageCategoriesUseCase::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
        );

        let result = uc.delete_category("CURRENCY", "admin").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_category_not_found() {
        let mut mock_repo = MockCategoryRepository::new();
        let mock_publisher = MockDomainMasterEventPublisher::new();

        mock_repo
            .expect_find_by_code()
            .with(eq("NONEXIST"))
            .times(1)
            .returning(|_| Ok(None));

        let uc = ManageCategoriesUseCase::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
        );

        let result = uc.delete_category("NONEXIST", "admin").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_list_categories() {
        let mut mock_repo = MockCategoryRepository::new();
        let mock_publisher = MockDomainMasterEventPublisher::new();

        let cat1 = sample_category();
        let categories = vec![cat1];

        mock_repo
            .expect_find_all()
            .with(eq(true))
            .times(1)
            .returning(move |_| Ok(categories.clone()));

        let uc = ManageCategoriesUseCase::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
        );

        let result = uc.list_categories(true).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_category() {
        let mut mock_repo = MockCategoryRepository::new();
        let mock_publisher = MockDomainMasterEventPublisher::new();

        let cat = sample_category();
        let cat_clone = cat.clone();

        mock_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(cat_clone.clone())));

        let uc = ManageCategoriesUseCase::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
        );

        let result = uc.get_category("CURRENCY").await;
        assert!(result.is_ok());
        let cat = result.unwrap();
        assert!(cat.is_some());
        assert_eq!(cat.unwrap().code, "CURRENCY");
    }

    #[tokio::test]
    async fn test_update_category_success() {
        let mut mock_repo = MockCategoryRepository::new();
        let mut mock_publisher = MockDomainMasterEventPublisher::new();

        let existing = sample_category();
        let existing_clone = existing.clone();
        let mut updated = existing.clone();
        updated.display_name = "Updated Currency".to_string();
        let updated_clone = updated.clone();

        mock_repo
            .expect_find_by_code()
            .with(eq("CURRENCY"))
            .times(1)
            .returning(move |_| Ok(Some(existing_clone.clone())));

        mock_repo
            .expect_update()
            .with(eq("CURRENCY"), always())
            .times(1)
            .returning(move |_, _| Ok(updated_clone.clone()));

        mock_publisher
            .expect_publish_category_changed()
            .times(1)
            .returning(|_| Ok(()));

        let uc = ManageCategoriesUseCase::new(
            Arc::new(mock_repo),
            Arc::new(mock_publisher),
        );

        let input = UpdateMasterCategory {
            display_name: Some("Updated Currency".to_string()),
            description: None,
            validation_schema: None,
            is_active: None,
            sort_order: None,
        };

        let result = uc.update_category("CURRENCY", "admin", &input).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().display_name, "Updated Currency");
    }
}
