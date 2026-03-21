//! テーブル定義の管理操作を担当する usecase。
//!
//! テーブル定義の作成・読み取り・更新・削除と JSON スキーマ生成を提供する。

use crate::domain::entity::table_definition::{
    CreateTableDefinition, TableDefinition, UpdateTableDefinition,
};
use crate::domain::error::MasterMaintenanceError;
use crate::domain::repository::column_definition_repository::ColumnDefinitionRepository;
use crate::domain::repository::table_definition_repository::TableDefinitionRepository;
use crate::domain::service::metadata_service::SchemaGeneratorService;
use crate::domain::value_object::domain_filter::DomainFilter;
use crate::infrastructure::schema::SchemaManager;
use std::sync::Arc;
use uuid::Uuid;

/// テーブル定義の管理操作を提供する usecase 構造体。
pub struct ManageTableDefinitionsUseCase {
    table_repo: Arc<dyn TableDefinitionRepository>,
    column_repo: Arc<dyn ColumnDefinitionRepository>,
    schema_manager: Arc<dyn SchemaManager>,
}

impl ManageTableDefinitionsUseCase {
    /// 依存リポジトリとスキーママネージャーを注入して usecase を構築する。
    pub fn new(
        table_repo: Arc<dyn TableDefinitionRepository>,
        column_repo: Arc<dyn ColumnDefinitionRepository>,
        schema_manager: Arc<dyn SchemaManager>,
    ) -> Self {
        Self {
            table_repo,
            column_repo,
            schema_manager,
        }
    }

    /// テーブル一覧を取得する。カテゴリや active フラグ、ドメインスコープでフィルタできる。
    pub async fn list_tables(
        &self,
        category: Option<&str>,
        active_only: bool,
        domain_filter: &DomainFilter,
    ) -> anyhow::Result<Vec<TableDefinition>> {
        self.table_repo
            .find_all(category, active_only, domain_filter)
            .await
    }

    /// テーブル名でテーブル定義を取得する。見つからない場合は None を返す。
    pub async fn get_table(
        &self,
        name: &str,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<Option<TableDefinition>> {
        self.table_repo.find_by_name(name, domain_scope).await
    }

    /// テーブル ID でテーブル定義を取得する。見つからない場合は None を返す。
    pub async fn get_table_by_id(&self, id: Uuid) -> anyhow::Result<Option<TableDefinition>> {
        self.table_repo.find_by_id(id).await
    }

    /// テーブルを新規作成する。物理スキーマも合わせて作成する。
    pub async fn create_table(
        &self,
        input: &CreateTableDefinition,
        created_by: &str,
    ) -> anyhow::Result<TableDefinition> {
        self.schema_manager.create_table(input).await?;
        self.table_repo.create(input, created_by).await
    }

    /// テーブル定義を更新する。
    pub async fn update_table(
        &self,
        name: &str,
        input: &UpdateTableDefinition,
        domain_scope: Option<&str>,
    ) -> anyhow::Result<TableDefinition> {
        self.table_repo.update(name, input, domain_scope).await
    }

    /// テーブルを削除する。テーブル定義が見つからない場合は TableNotFound を返す。
    pub async fn delete_table(
        &self,
        name: &str,
        domain_scope: Option<&str>,
    ) -> Result<(), MasterMaintenanceError> {
        // テーブル定義を取得し、見つからない場合は TableNotFound を返す
        let table = self
            .table_repo
            .find_by_name(name, domain_scope)
            .await
            .map_err(MasterMaintenanceError::from)?
            .ok_or_else(|| MasterMaintenanceError::TableNotFound(name.to_string()))?;
        self.schema_manager
            .delete_table(&table)
            .await
            .map_err(MasterMaintenanceError::from)?;
        self.table_repo
            .delete(name, domain_scope)
            .await
            .map_err(MasterMaintenanceError::from)
    }

    /// テーブルの JSON スキーマを生成して返す。テーブルが見つからない場合は TableNotFound を返す。
    pub async fn get_table_schema(
        &self,
        name: &str,
        domain_scope: Option<&str>,
    ) -> Result<serde_json::Value, MasterMaintenanceError> {
        // テーブル定義を取得し、見つからない場合は TableNotFound を返す
        let table = self
            .table_repo
            .find_by_name(name, domain_scope)
            .await
            .map_err(MasterMaintenanceError::from)?
            .ok_or_else(|| MasterMaintenanceError::TableNotFound(name.to_string()))?;
        let columns = self
            .column_repo
            .find_by_table_id(table.id)
            .await
            .map_err(MasterMaintenanceError::from)?;
        Ok(SchemaGeneratorService::generate_json_schema(
            &table, &columns,
        ))
    }

    /// ドメインスコープの一覧と各ドメインのテーブル数を返す。
    pub async fn list_domains(&self) -> anyhow::Result<Vec<(String, i64)>> {
        self.table_repo.find_domains().await
    }
}
