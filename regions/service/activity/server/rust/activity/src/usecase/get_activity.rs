// アクティビティ取得ユースケース。
use crate::domain::entity::activity::Activity;
use crate::domain::repository::activity_repository::ActivityRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct GetActivityUseCase {
    repo: Arc<dyn ActivityRepository>,
}

impl GetActivityUseCase {
    pub fn new(repo: Arc<dyn ActivityRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, id: Uuid) -> anyhow::Result<Option<Activity>> {
        self.repo.find_by_id(id).await
    }
}
