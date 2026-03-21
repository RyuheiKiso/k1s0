use std::sync::Arc;

use uuid::Uuid;

use crate::domain::entity::service_doc::ServiceDoc;
use crate::domain::repository::DocRepository;

/// ManageDocsError はドキュメント管理に関するエラーを表す。
#[derive(Debug, thiserror::Error)]
pub enum ManageDocsError {
    #[error("internal error: {0}")]
    Internal(String),
}

/// ManageDocsUseCase はサービスドキュメント管理ユースケース。
pub struct ManageDocsUseCase {
    doc_repo: Arc<dyn DocRepository>,
}

impl ManageDocsUseCase {
    pub fn new(doc_repo: Arc<dyn DocRepository>) -> Self {
        Self { doc_repo }
    }

    /// 指定サービスのドキュメント一覧を取得する。
    pub async fn list(&self, service_id: Uuid) -> Result<Vec<ServiceDoc>, ManageDocsError> {
        self.doc_repo
            .list_by_service(service_id)
            .await
            .map_err(|e| ManageDocsError::Internal(e.to_string()))
    }

    /// 指定サービスのドキュメントを一括設定する。
    pub async fn set(
        &self,
        service_id: Uuid,
        docs: Vec<ServiceDoc>,
    ) -> Result<(), ManageDocsError> {
        self.doc_repo
            .set_docs(service_id, docs)
            .await
            .map_err(|e| ManageDocsError::Internal(e.to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::service_doc::DocType;
    use crate::domain::repository::doc_repository::MockDocRepository;
    use chrono::Utc;

    fn sample_doc(service_id: Uuid) -> ServiceDoc {
        ServiceDoc {
            id: Uuid::new_v4(),
            service_id,
            title: "Runbook".to_string(),
            url: "https://wiki.example.com/runbook".to_string(),
            doc_type: DocType::Runbook,
            created_at: Utc::now(),
        }
    }

    /// list はドキュメント一覧を返す
    #[tokio::test]
    async fn list_returns_docs() {
        let service_id = Uuid::new_v4();
        let mut mock = MockDocRepository::new();
        mock.expect_list_by_service()
            .withf(move |i| *i == service_id)
            .returning(move |i| Ok(vec![sample_doc(i)]));

        let uc = ManageDocsUseCase::new(Arc::new(mock));
        let result = uc.list(service_id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].doc_type, DocType::Runbook);
    }

    /// list がエラーを返す場合は Internal エラーになる
    #[tokio::test]
    async fn list_internal_error() {
        let mut mock = MockDocRepository::new();
        mock.expect_list_by_service()
            .returning(|_| Err(anyhow::anyhow!("db error")));

        let uc = ManageDocsUseCase::new(Arc::new(mock));
        let result = uc.list(Uuid::new_v4()).await;
        assert!(matches!(result, Err(ManageDocsError::Internal(_))));
    }

    /// set はドキュメントを一括設定する
    #[tokio::test]
    async fn set_success() {
        let service_id = Uuid::new_v4();
        let mut mock = MockDocRepository::new();
        mock.expect_set_docs().returning(|_, _| Ok(()));

        let uc = ManageDocsUseCase::new(Arc::new(mock));
        let result = uc.set(service_id, vec![sample_doc(service_id)]).await;
        assert!(result.is_ok());
    }
}
