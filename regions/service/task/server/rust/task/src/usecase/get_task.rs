// タスク取得ユースケース。
use crate::domain::entity::task::{Task, TaskChecklistItem};
use crate::domain::repository::task_repository::TaskRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetTaskUseCase {
    task_repo: Arc<dyn TaskRepository>,
}

impl GetTaskUseCase {
    pub fn new(task_repo: Arc<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    /// タスクを ID で取得する
    // タスク取得の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, id: Uuid) -> anyhow::Result<Option<Task>> {
        self.task_repo.find_by_id(id).await
    }

    /// チェックリストを取得する
    // チェックリスト取得の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn get_checklist(&self, task_id: Uuid) -> anyhow::Result<Vec<TaskChecklistItem>> {
        self.task_repo.find_checklist(task_id).await
    }
}
