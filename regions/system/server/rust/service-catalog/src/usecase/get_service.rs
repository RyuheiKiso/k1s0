use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::service::Service;
use crate::domain::repository::ServiceRepository;

/// `GetServiceError` はサービス取得に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum GetServiceError {
    #[error("service not found: {0}")]
    NotFound(Uuid),

    #[error("internal error: {0}")]
    Internal(String),
}

/// `GetServiceUseCase` はサービス情報取得ユースケース。
pub struct GetServiceUseCase {
    service_repo: Arc<dyn ServiceRepository>,
}

impl GetServiceUseCase {
    pub fn new(service_repo: Arc<dyn ServiceRepository>) -> Self {
        Self { service_repo }
    }

    // CRIT-004 監査対応: RLS テナント分離のため tenant_id を受け取りリポジトリに渡す。
    pub async fn execute(&self, tenant_id: &str, id: Uuid) -> Result<Service, GetServiceError> {
        match self.service_repo.find_by_id(tenant_id, id).await {
            Ok(Some(service)) => Ok(service),
            Ok(None) => Err(GetServiceError::NotFound(id)),
            Err(e) => Err(GetServiceError::Internal(e.to_string())),
        }
    }
}

// テストコードでは unwrap() を許可する（unwrap_used = "deny" はプロダクションコード向け）
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::service::{ServiceLifecycle, ServiceTier};
    use crate::domain::repository::service_repository::MockServiceRepository;
    use chrono::Utc;

    /// テスト用 Service ヘルパー
    fn make_service(id: Uuid) -> Service {
        Service {
            id,
            name: "my-service".to_string(),
            description: None,
            team_id: Uuid::new_v4(),
            tier: ServiceTier::Standard,
            lifecycle: ServiceLifecycle::Production,
            repository_url: None,
            api_endpoint: None,
            healthcheck_url: None,
            tags: vec![],
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// サービスが見つかった場合は Service を返す
    #[tokio::test]
    async fn test_get_service_success() {
        let id = Uuid::new_v4();
        let svc = make_service(id);
        let svc_clone = svc.clone();
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id()
            .returning(move |_, _| Ok(Some(svc_clone.clone())));

        let uc = GetServiceUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-1", id).await.unwrap();
        assert_eq!(result.id, id);
    }

    /// 存在しないサービスは NotFound エラーを返す
    #[tokio::test]
    async fn test_get_service_not_found() {
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id().returning(|_, _| Ok(None));

        let uc = GetServiceUseCase::new(Arc::new(mock));
        let id = Uuid::new_v4();
        let result = uc.execute("tenant-1", id).await;
        assert!(matches!(result, Err(GetServiceError::NotFound(_))));
    }

    /// リポジトリエラーは Internal エラーに変換される
    #[tokio::test]
    async fn test_get_service_internal_error() {
        let mut mock = MockServiceRepository::new();
        mock.expect_find_by_id()
            .returning(|_, _| Err(anyhow::anyhow!("db error")));

        let uc = GetServiceUseCase::new(Arc::new(mock));
        let result = uc.execute("tenant-1", Uuid::new_v4()).await;
        assert!(matches!(result, Err(GetServiceError::Internal(_))));
    }
}
