use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::health::HealthStatus;
use crate::domain::repository::HealthRepository;

/// HealthStatusError はヘルスステータスに関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum HealthStatusError {
    #[error("internal error: {0}")]
    Internal(String),
}

/// HealthStatusUseCase はサービスヘルスステータスのユースケース。
pub struct HealthStatusUseCase {
    health_repo: Arc<dyn HealthRepository>,
}

impl HealthStatusUseCase {
    pub fn new(health_repo: Arc<dyn HealthRepository>) -> Self {
        Self { health_repo }
    }

    /// 指定サービスの最新ヘルスステータスを取得する。
    pub async fn get(&self, service_id: Uuid) -> Result<Option<HealthStatus>, HealthStatusError> {
        self.health_repo
            .get_latest(service_id)
            .await
            .map_err(|e| HealthStatusError::Internal(e.to_string()))
    }

    /// ヘルスステータスを報告（upsert）する。
    pub async fn report(&self, health: &HealthStatus) -> Result<(), HealthStatusError> {
        self.health_repo
            .upsert(health)
            .await
            .map_err(|e| HealthStatusError::Internal(e.to_string()))
    }

    /// 全サービスの最新ヘルスステータス一覧を取得する。
    #[allow(dead_code)]
    pub async fn list_all(&self) -> Result<Vec<HealthStatus>, HealthStatusError> {
        self.health_repo
            .list_all_latest()
            .await
            .map_err(|e| HealthStatusError::Internal(e.to_string()))
    }
}
