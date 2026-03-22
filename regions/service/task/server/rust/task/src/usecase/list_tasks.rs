// タスク一覧ユースケース。
use crate::domain::entity::task::{Task, TaskFilter};
use crate::domain::repository::task_repository::TaskRepository;
use std::sync::Arc;

pub struct ListTasksUseCase {
    task_repo: Arc<dyn TaskRepository>,
}

impl ListTasksUseCase {
    pub fn new(task_repo: Arc<dyn TaskRepository>) -> Self {
        Self { task_repo }
    }

    // タスク一覧取得の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, filter: &TaskFilter) -> anyhow::Result<(Vec<Task>, i64)> {
        let tasks = self.task_repo.find_all(filter).await?;
        let count = self.task_repo.count(filter).await?;
        Ok((tasks, count))
    }
}
