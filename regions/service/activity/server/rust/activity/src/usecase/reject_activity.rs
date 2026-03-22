// アクティビティ却下ユースケース（Submitted → Rejected）。
use crate::domain::entity::activity::{Activity, ActivityStatus};
use crate::domain::repository::activity_repository::ActivityRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct RejectActivityUseCase {
    repo: Arc<dyn ActivityRepository>,
}

impl RejectActivityUseCase {
    pub fn new(repo: Arc<dyn ActivityRepository>) -> Self {
        Self { repo }
    }

    // アクティビティ却下の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, id: Uuid, rejector_id: &str, _reason: &str) -> anyhow::Result<Activity> {
        let activity = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Activity '{}' not found", id))?;
        activity.transition_to(ActivityStatus::Rejected)?;
        self.repo.update_status(id, "rejected", Some(rejector_id.to_string())).await
    }
}
