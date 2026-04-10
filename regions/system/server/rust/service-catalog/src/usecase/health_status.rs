use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::health::HealthStatus;
use crate::domain::repository::HealthRepository;

/// `HealthStatusError` はヘルスステータスに関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum HealthStatusError {
    #[error("internal error: {0}")]
    Internal(String),
}

/// `HealthStatusUseCase` はサービスヘルスステータスのユースケース。
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::health::HealthState;
    use crate::domain::repository::health_repository::MockHealthRepository;
    use chrono::Utc;

    fn sample_health(service_id: Uuid) -> HealthStatus {
        HealthStatus {
            service_id,
            status: HealthState::Healthy,
            message: None,
            response_time_ms: Some(42),
            checked_at: Utc::now(),
        }
    }

    /// ヘルスステータスが存在する場合は Some を返す
    #[tokio::test]
    async fn get_returns_some() {
        let id = Uuid::new_v4();
        let mut mock = MockHealthRepository::new();
        mock.expect_get_latest()
            .returning(move |i| Ok(Some(sample_health(i))));

        let uc = HealthStatusUseCase::new(Arc::new(mock));
        let result = uc.get(id).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, HealthState::Healthy);
    }

    /// ヘルスステータスが存在しない場合は None を返す
    #[tokio::test]
    async fn get_returns_none() {
        let mut mock = MockHealthRepository::new();
        mock.expect_get_latest().returning(|_| Ok(None));

        let uc = HealthStatusUseCase::new(Arc::new(mock));
        let result = uc.get(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    /// report はヘルスステータスを upsert する
    #[tokio::test]
    async fn report_success() {
        let mut mock = MockHealthRepository::new();
        mock.expect_upsert().returning(|_| Ok(()));

        let uc = HealthStatusUseCase::new(Arc::new(mock));
        let health = sample_health(Uuid::new_v4());
        let result = uc.report(&health).await;
        assert!(result.is_ok());
    }

    /// list_all は全サービスのヘルスステータスを返す
    #[tokio::test]
    async fn list_all_returns_all() {
        let mut mock = MockHealthRepository::new();
        mock.expect_list_all_latest().returning(|| {
            Ok(vec![
                sample_health(Uuid::new_v4()),
                sample_health(Uuid::new_v4()),
            ])
        });

        let uc = HealthStatusUseCase::new(Arc::new(mock));
        let result = uc.list_all().await.unwrap();
        assert_eq!(result.len(), 2);
    }
}
