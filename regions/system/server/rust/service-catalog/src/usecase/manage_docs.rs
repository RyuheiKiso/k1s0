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
