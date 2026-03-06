use crate::domain::entity::master_category::{
    CreateMasterCategory, MasterCategory, UpdateMasterCategory,
};
use crate::domain::repository::category_repository::CategoryRepository;
use std::sync::Arc;

pub struct ManageCategoriesUseCase {
    category_repo: Arc<dyn CategoryRepository>,
}

impl ManageCategoriesUseCase {
    pub fn new(category_repo: Arc<dyn CategoryRepository>) -> Self {
        Self { category_repo }
    }

    pub async fn list_categories(
        &self,
        active_only: bool,
    ) -> anyhow::Result<Vec<MasterCategory>> {
        self.category_repo.find_all(active_only).await
    }

    pub async fn get_category(&self, code: &str) -> anyhow::Result<Option<MasterCategory>> {
        self.category_repo.find_by_code(code).await
    }

    pub async fn create_category(
        &self,
        input: &CreateMasterCategory,
        created_by: &str,
    ) -> anyhow::Result<MasterCategory> {
        if let Some(_existing) = self.category_repo.find_by_code(&input.code).await? {
            anyhow::bail!("Duplicate code: category '{}' already exists", input.code);
        }
        self.category_repo.create(input, created_by).await
    }

    pub async fn update_category(
        &self,
        code: &str,
        input: &UpdateMasterCategory,
    ) -> anyhow::Result<MasterCategory> {
        self.category_repo
            .find_by_code(code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", code))?;
        self.category_repo.update(code, input).await
    }

    pub async fn delete_category(&self, code: &str) -> anyhow::Result<()> {
        self.category_repo
            .find_by_code(code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Category '{}' not found", code))?;
        self.category_repo.delete(code).await
    }
}
