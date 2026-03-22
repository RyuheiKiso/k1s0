// アクティビティ承認ユースケース（Submitted → Approved）。
use crate::domain::entity::activity::{Activity, ActivityStatus};
use crate::domain::repository::activity_repository::ActivityRepository;
use std::sync::Arc;
use uuid::Uuid;

pub struct ApproveActivityUseCase {
    repo: Arc<dyn ActivityRepository>,
}

impl ApproveActivityUseCase {
    pub fn new(repo: Arc<dyn ActivityRepository>) -> Self {
        Self { repo }
    }

    // アクティビティ承認の全処理をトレースするためにスパンを自動生成する
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, id: Uuid, approver_id: &str) -> anyhow::Result<Activity> {
        let activity = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Activity '{}' not found", id))?;
        activity.transition_to(ActivityStatus::Approved)?;
        self.repo.update_status(id, "approved", Some(approver_id.to_string())).await
    }
}
