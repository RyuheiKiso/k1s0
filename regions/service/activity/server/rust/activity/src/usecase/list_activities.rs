// アクティビティ一覧ユースケース。
use crate::domain::entity::activity::{Activity, ActivityFilter};
use crate::domain::repository::activity_repository::ActivityRepository;
use std::sync::Arc;

pub struct ListActivitiesUseCase {
    repo: Arc<dyn ActivityRepository>,
}

impl ListActivitiesUseCase {
    pub fn new(repo: Arc<dyn ActivityRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, filter: &ActivityFilter) -> anyhow::Result<(Vec<Activity>, i64)> {
        let items = self.repo.find_all(filter).await?;
        let count = self.repo.count(filter).await?;
        Ok((items, count))
    }
}
