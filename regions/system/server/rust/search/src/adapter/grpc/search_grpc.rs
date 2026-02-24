use std::sync::Arc;

use crate::usecase::delete_document::{DeleteDocumentError, DeleteDocumentUseCase};
use crate::usecase::index_document::{IndexDocumentError, IndexDocumentInput, IndexDocumentUseCase};
use crate::usecase::search::{SearchError, SearchInput, SearchUseCase};

// --- gRPC Request/Response Types ---

#[derive(Debug, Clone)]
pub struct IndexDocumentRequest {
    pub index: String,
    pub document_id: String,
    pub document_json: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct IndexDocumentResponse {
    pub document_id: String,
    pub index: String,
    pub result: String,
}

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub index: String,
    pub query: String,
    pub filters_json: Vec<u8>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub id: String,
    pub score: f32,
    pub document_json: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DeleteDocumentRequest {
    pub index: String,
    pub document_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteDocumentResponse {
    pub success: bool,
    pub message: String,
}

// --- gRPC Error ---

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),
}

// --- SearchGrpcService ---

pub struct SearchGrpcService {
    index_document_uc: Arc<IndexDocumentUseCase>,
    search_uc: Arc<SearchUseCase>,
    delete_document_uc: Arc<DeleteDocumentUseCase>,
}

impl SearchGrpcService {
    pub fn new(
        index_document_uc: Arc<IndexDocumentUseCase>,
        search_uc: Arc<SearchUseCase>,
        delete_document_uc: Arc<DeleteDocumentUseCase>,
    ) -> Self {
        Self {
            index_document_uc,
            search_uc,
            delete_document_uc,
        }
    }

    pub async fn index_document(
        &self,
        req: IndexDocumentRequest,
    ) -> Result<IndexDocumentResponse, GrpcError> {
        let content: serde_json::Value = if req.document_json.is_empty() {
            serde_json::Value::Object(Default::default())
        } else {
            serde_json::from_slice(&req.document_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid document_json: {}", e)))?
        };

        let input = IndexDocumentInput {
            id: req.document_id.clone(),
            index_name: req.index.clone(),
            content,
        };

        match self.index_document_uc.execute(&input).await {
            Ok(doc) => Ok(IndexDocumentResponse {
                document_id: doc.id,
                index: doc.index_name,
                result: "created".to_string(),
            }),
            Err(IndexDocumentError::IndexNotFound(name)) => {
                Err(GrpcError::NotFound(format!("index not found: {}", name)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn search(&self, req: SearchRequest) -> Result<SearchResponse, GrpcError> {
        let page = if req.page == 0 { 1 } else { req.page };
        let page_size = if req.page_size == 0 { 10 } else { req.page_size };
        let from = (page - 1) * page_size;

        let input = SearchInput {
            index_name: req.index.clone(),
            query: req.query,
            from,
            size: page_size,
        };

        match self.search_uc.execute(&input).await {
            Ok(result) => {
                let has_next = result.total > (from as u64 + result.hits.len() as u64);
                let hits = result
                    .hits
                    .into_iter()
                    .map(|doc| {
                        let document_json = serde_json::to_vec(&doc.content).unwrap_or_default();
                        SearchHit {
                            id: doc.id,
                            score: 1.0,
                            document_json,
                        }
                    })
                    .collect();

                Ok(SearchResponse {
                    hits,
                    total_count: result.total,
                    page,
                    page_size,
                    has_next,
                })
            }
            Err(SearchError::IndexNotFound(name)) => {
                Err(GrpcError::NotFound(format!("index not found: {}", name)))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    pub async fn delete_document(
        &self,
        req: DeleteDocumentRequest,
    ) -> Result<DeleteDocumentResponse, GrpcError> {
        let input = crate::usecase::delete_document::DeleteDocumentInput {
            index_name: req.index.clone(),
            doc_id: req.document_id.clone(),
        };

        match self.delete_document_uc.execute(&input).await {
            Ok(_) => Ok(DeleteDocumentResponse {
                success: true,
                message: format!("document {} deleted from index {}", req.document_id, req.index),
            }),
            Err(DeleteDocumentError::NotFound(index, id)) => {
                Err(GrpcError::NotFound(format!(
                    "document not found: {}/{}",
                    index, id
                )))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::search_index::{SearchDocument, SearchIndex, SearchResult};
    use crate::domain::repository::search_repository::MockSearchRepository;

    fn make_service(mock: MockSearchRepository) -> SearchGrpcService {
        let repo = Arc::new(mock);
        SearchGrpcService::new(
            Arc::new(IndexDocumentUseCase::new(repo.clone())),
            Arc::new(SearchUseCase::new(repo.clone())),
            Arc::new(DeleteDocumentUseCase::new(repo)),
        )
    }

    #[tokio::test]
    async fn test_index_document_success() {
        let mut mock = MockSearchRepository::new();
        let index = SearchIndex::new("products".to_string(), serde_json::json!({}));
        let return_index = index.clone();

        mock.expect_find_index()
            .withf(|name| name == "products")
            .returning(move |_| Ok(Some(return_index.clone())));
        mock.expect_index_document().returning(|_| Ok(()));

        let svc = make_service(mock);
        let req = IndexDocumentRequest {
            index: "products".to_string(),
            document_id: "doc-1".to_string(),
            document_json: serde_json::to_vec(&serde_json::json!({"name": "Widget"})).unwrap(),
        };
        let resp = svc.index_document(req).await.unwrap();

        assert_eq!(resp.document_id, "doc-1");
        assert_eq!(resp.index, "products");
        assert_eq!(resp.result, "created");
    }

    #[tokio::test]
    async fn test_index_document_index_not_found() {
        let mut mock = MockSearchRepository::new();
        mock.expect_find_index().returning(|_| Ok(None));

        let svc = make_service(mock);
        let req = IndexDocumentRequest {
            index: "nonexistent".to_string(),
            document_id: "doc-1".to_string(),
            document_json: vec![],
        };
        let result = svc.index_document(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("index not found")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_search_success() {
        let mut mock = MockSearchRepository::new();
        let index = SearchIndex::new("products".to_string(), serde_json::json!({}));
        let return_index = index.clone();

        mock.expect_find_index()
            .withf(|name| name == "products")
            .returning(move |_| Ok(Some(return_index.clone())));

        mock.expect_search().returning(|_| {
            Ok(SearchResult {
                total: 1,
                hits: vec![SearchDocument {
                    id: "doc-1".to_string(),
                    index_name: "products".to_string(),
                    content: serde_json::json!({"name": "Widget"}),
                    indexed_at: chrono::Utc::now(),
                }],
            })
        });

        let svc = make_service(mock);
        let req = SearchRequest {
            index: "products".to_string(),
            query: "Widget".to_string(),
            filters_json: vec![],
            page: 1,
            page_size: 10,
        };
        let resp = svc.search(req).await.unwrap();

        assert_eq!(resp.total_count, 1);
        assert_eq!(resp.hits.len(), 1);
        assert_eq!(resp.hits[0].id, "doc-1");
        assert!(!resp.has_next);
    }

    #[tokio::test]
    async fn test_delete_document_success() {
        let mut mock = MockSearchRepository::new();
        mock.expect_delete_document()
            .withf(|index, id| index == "products" && id == "doc-1")
            .returning(|_, _| Ok(true));

        let svc = make_service(mock);
        let req = DeleteDocumentRequest {
            index: "products".to_string(),
            document_id: "doc-1".to_string(),
        };
        let resp = svc.delete_document(req).await.unwrap();

        assert!(resp.success);
        assert!(resp.message.contains("doc-1"));
    }

    #[tokio::test]
    async fn test_delete_document_not_found() {
        let mut mock = MockSearchRepository::new();
        mock.expect_delete_document().returning(|_, _| Ok(false));

        let svc = make_service(mock);
        let req = DeleteDocumentRequest {
            index: "products".to_string(),
            document_id: "missing".to_string(),
        };
        let result = svc.delete_document(req).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            GrpcError::NotFound(msg) => assert!(msg.contains("document not found")),
            e => unreachable!("unexpected error: {:?}", e),
        }
    }
}
