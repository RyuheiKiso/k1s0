use std::sync::Arc;
use serde_json::Value;
use uuid::Uuid;
use crate::domain::entity::display_config::{DisplayConfig, CreateDisplayConfig};
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::repository::display_config_repository::DisplayConfigRepository;

pub struct ManageDisplayConfigsUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    display_config_repo: Arc<dyn DisplayConfigRepository>,
}

impl ManageDisplayConfigsUseCase {
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        display_config_repo: Arc<dyn DisplayConfigRepository>,
    ) -> Self {
        Self {
            table_repo,
            display_config_repo,
        }
    }

    pub async fn list_display_configs(
        &self,
        table_name: &str,
    ) -> anyhow::Result<Vec<DisplayConfig>> {
        let table = self
            .table_repo
            .find_by_name(table_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;

        self.display_config_repo.find_by_table_id(table.id).await
    }

    pub async fn get_display_config(&self, id: Uuid) -> anyhow::Result<Option<DisplayConfig>> {
        self.display_config_repo.find_by_id(id).await
    }

    pub async fn create_display_config(
        &self,
        table_name: &str,
        input: &Value,
        created_by: &str,
    ) -> anyhow::Result<DisplayConfig> {
        let table = self
            .table_repo
            .find_by_name(table_name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Table '{}' not found", table_name))?;

        let create_input: CreateDisplayConfig = serde_json::from_value(input.clone())?;

        let config = DisplayConfig {
            id: Uuid::new_v4(),
            table_id: table.id,
            config_type: create_input.config_type,
            config_json: create_input.config_json,
            is_default: create_input.is_default.unwrap_or(false),
            created_by: created_by.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.display_config_repo.create(&config).await
    }

    pub async fn update_display_config(
        &self,
        id: Uuid,
        input: &Value,
    ) -> anyhow::Result<DisplayConfig> {
        let mut config = self
            .display_config_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Display config not found"))?;

        if let Some(config_type) = input.get("config_type").and_then(|v| v.as_str()) {
            config.config_type = config_type.to_string();
        }
        if let Some(config_json) = input.get("config_json") {
            config.config_json = config_json.clone();
        }
        if let Some(is_default) = input.get("is_default").and_then(|v| v.as_bool()) {
            config.is_default = is_default;
        }
        config.updated_at = chrono::Utc::now();

        self.display_config_repo.update(id, &config).await
    }

    pub async fn delete_display_config(&self, id: Uuid) -> anyhow::Result<()> {
        self.display_config_repo.delete(id).await
    }
}
