// プロジェクトタイプリポジトリ trait。
// 永続化層の実装に依存せず、ドメイン層からデータアクセスを抽象化する。
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entity::project_type::{
    CreateProjectType, ProjectType, ProjectTypeFilter, UpdateProjectType,
};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ProjectTypeRepository: Send + Sync {
    /// ID でプロジェクトタイプを取得する
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<ProjectType>>;
    /// コードでプロジェクトタイプを取得する
    async fn find_by_code(&self, code: &str) -> anyhow::Result<Option<ProjectType>>;
    /// 一覧取得
    async fn find_all(&self, filter: &ProjectTypeFilter) -> anyhow::Result<Vec<ProjectType>>;
    /// 件数取得
    async fn count(&self, filter: &ProjectTypeFilter) -> anyhow::Result<i64>;
    /// プロジェクトタイプを作成する
    async fn create(
        &self,
        input: &CreateProjectType,
        created_by: &str,
    ) -> anyhow::Result<ProjectType>;
    /// プロジェクトタイプを更新する
    async fn update(
        &self,
        id: Uuid,
        input: &UpdateProjectType,
        updated_by: &str,
    ) -> anyhow::Result<ProjectType>;
    /// プロジェクトタイプを削除する
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
}
