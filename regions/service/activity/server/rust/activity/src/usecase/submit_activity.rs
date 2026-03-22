// アクティビティ提出ユースケース（Active → Submitted）。
use crate::domain::entity::activity::{Activity, ActivityStatus};
use crate::domain::repository::activity_repository::ActivityRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct SubmitActivityUseCase {
    repo: Arc<dyn ActivityRepository>,
}

impl SubmitActivityUseCase {
    pub fn new(repo: Arc<dyn ActivityRepository>) -> Self {
        Self { repo }
    }

    // アクティビティ提出の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, id: Uuid, actor_id: &str) -> anyhow::Result<Activity> {
        let activity = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Activity '{}' not found", id))?;
        activity.transition_to(ActivityStatus::Submitted)?;
        self.repo.update_status(id, "submitted", Some(actor_id.to_string())).await
    }
}
