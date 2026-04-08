use async_trait::async_trait;

use crate::domain::entity::api_registration::{ApiSchema, ApiSchemaVersion};

// テナント分離のため全メソッドに tenant_id を追加する。
// RLS の set_config はリポジトリ実装内で呼び出す。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ApiSchemaRepository: Send + Sync {
    // 指定スキーマ名をテナントスコープで検索する
    async fn find_by_name(&self, tenant_id: &str, name: &str) -> anyhow::Result<Option<ApiSchema>>;
    // テナントスコープでスキーマ一覧を取得する
    async fn find_all(
        &self,
        tenant_id: &str,
        schema_type: Option<String>,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchema>, u64)>;
    // テナントスコープでスキーマを作成する
    async fn create(&self, tenant_id: &str, schema: &ApiSchema) -> anyhow::Result<()>;
    // テナントスコープでスキーマを更新する
    async fn update(&self, tenant_id: &str, schema: &ApiSchema) -> anyhow::Result<()>;
}

// バージョン操作もテナント分離が必要なため tenant_id を追加する。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ApiSchemaVersionRepository: Send + Sync {
    // テナントスコープでスキーマ名とバージョンで検索する
    async fn find_by_name_and_version(
        &self,
        tenant_id: &str,
        name: &str,
        version: u32,
    ) -> anyhow::Result<Option<ApiSchemaVersion>>;
    // テナントスコープで最新バージョンを取得する
    async fn find_latest_by_name(&self, tenant_id: &str, name: &str) -> anyhow::Result<Option<ApiSchemaVersion>>;
    // テナントスコープでバージョン一覧を取得する
    async fn find_all_by_name(
        &self,
        tenant_id: &str,
        name: &str,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ApiSchemaVersion>, u64)>;
    // テナントスコープでバージョンを作成する
    async fn create(&self, tenant_id: &str, version: &ApiSchemaVersion) -> anyhow::Result<()>;
    // テナントスコープでバージョンを削除する
    async fn delete(&self, tenant_id: &str, name: &str, version: u32) -> anyhow::Result<bool>;
    // テナントスコープでスキーマ名のバージョン数を取得する
    async fn count_by_name(&self, tenant_id: &str, name: &str) -> anyhow::Result<u64>;
}
