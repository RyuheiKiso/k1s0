use std::collections::HashMap;
use std::sync::Arc;

use crate::usecase::create_index::{CreateIndexError, CreateIndexInput, CreateIndexUseCase};
use crate::usecase::delete_document::{DeleteDocumentError, DeleteDocumentUseCase};
use crate::usecase::index_document::{
    IndexDocumentError, IndexDocumentInput, IndexDocumentUseCase,
};
use crate::usecase::list_indices::{ListIndicesError, ListIndicesUseCase};
use crate::usecase::search::{SearchError, SearchInput, SearchUseCase};

#[derive(Debug, Clone)]
pub struct SearchIndex {
    pub id: String,
    pub name: String,
    pub mapping_json: Vec<u8>,
    pub created_at: String,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct CreateIndexRequest {
    pub name: String,
    pub mapping_json: Vec<u8>,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct CreateIndexResponse {
    pub index: SearchIndex,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct ListIndicesRequest {
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct ListIndicesResponse {
    pub indices: Vec<SearchIndex>,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct IndexDocumentRequest {
    pub index: String,
    pub document_id: String,
    pub document_json: Vec<u8>,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct IndexDocumentResponse {
    pub document_id: String,
    pub index: String,
    pub result: String,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub index: String,
    pub query: String,
    pub filters_json: Vec<u8>,
    pub from: u32,
    pub size: u32,
    pub facets: Vec<String>,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
    pub facets: HashMap<String, HashMap<String, u64>>,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub id: String,
    pub score: f32,
    pub document_json: Vec<u8>,
}

/// CRIT-005 対応: テナント ID フィールドを追加したリクエスト型。
#[derive(Debug, Clone)]
pub struct DeleteDocumentRequest {
    pub index: String,
    pub document_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone)]
pub struct DeleteDocumentResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GrpcError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("internal: {0}")]
    Internal(String),
}

// ユースケースフィールドの命名規則として _uc サフィックスを使用する（アーキテクチャ上の意図的な設計）
#[allow(clippy::struct_field_names)]
pub struct SearchGrpcService {
    create_index_uc: Arc<CreateIndexUseCase>,
    list_indices_uc: Arc<ListIndicesUseCase>,
    index_document_uc: Arc<IndexDocumentUseCase>,
    search_uc: Arc<SearchUseCase>,
    delete_document_uc: Arc<DeleteDocumentUseCase>,
}

impl SearchGrpcService {
    #[must_use]
    pub fn new(
        create_index_uc: Arc<CreateIndexUseCase>,
        list_indices_uc: Arc<ListIndicesUseCase>,
        index_document_uc: Arc<IndexDocumentUseCase>,
        search_uc: Arc<SearchUseCase>,
        delete_document_uc: Arc<DeleteDocumentUseCase>,
    ) -> Self {
        Self {
            create_index_uc,
            list_indices_uc,
            index_document_uc,
            search_uc,
            delete_document_uc,
        }
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらインデックスを作成する。
    pub async fn create_index(
        &self,
        req: CreateIndexRequest,
    ) -> Result<CreateIndexResponse, GrpcError> {
        if req.name.trim().is_empty() {
            return Err(GrpcError::InvalidArgument("name is required".to_string()));
        }

        let mapping = if req.mapping_json.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_slice(&req.mapping_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid mapping_json: {e}")))?
        };

        let index = self
            .create_index_uc
            .execute(&CreateIndexInput {
                name: req.name,
                mapping,
                tenant_id: req.tenant_id,
            })
            .await
            .map_err(|e| match e {
                CreateIndexError::AlreadyExists(name) => {
                    GrpcError::AlreadyExists(format!("index already exists: {name}"))
                }
                CreateIndexError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(CreateIndexResponse {
            index: SearchIndex {
                id: index.id.to_string(),
                name: index.name,
                mapping_json: serde_json::to_vec(&index.mapping).unwrap_or_default(),
                created_at: index.created_at.to_rfc3339(),
            },
        })
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらインデックス一覧を取得する。
    pub async fn list_indices(
        &self,
        req: ListIndicesRequest,
    ) -> Result<ListIndicesResponse, GrpcError> {
        let indices = self
            .list_indices_uc
            .execute(&req.tenant_id)
            .await
            .map_err(|e| match e {
                ListIndicesError::Internal(msg) => GrpcError::Internal(msg),
            })?;

        Ok(ListIndicesResponse {
            indices: indices
                .into_iter()
                .map(|index| SearchIndex {
                    id: index.id.to_string(),
                    name: index.name,
                    mapping_json: serde_json::to_vec(&index.mapping).unwrap_or_default(),
                    created_at: index.created_at.to_rfc3339(),
                })
                .collect(),
        })
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらドキュメントを登録する。
    pub async fn index_document(
        &self,
        req: IndexDocumentRequest,
    ) -> Result<IndexDocumentResponse, GrpcError> {
        let content: serde_json::Value = if req.document_json.is_empty() {
            serde_json::Value::Object(serde_json::Map::default())
        } else {
            serde_json::from_slice(&req.document_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid document_json: {e}")))?
        };

        let input = IndexDocumentInput {
            id: req.document_id.clone(),
            index_name: req.index.clone(),
            content,
            tenant_id: req.tenant_id,
        };

        match self.index_document_uc.execute(&input).await {
            Ok(doc) => Ok(IndexDocumentResponse {
                document_id: doc.id,
                index: doc.index_name,
                result: "created".to_string(),
            }),
            Err(IndexDocumentError::IndexNotFound(name)) => {
                Err(GrpcError::NotFound(format!("index not found: {name}")))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながら検索を実行する。
    pub async fn search(&self, req: SearchRequest) -> Result<SearchResponse, GrpcError> {
        let page_size = if req.size == 0 { 10 } else { req.size };
        let from = req.from;
        let filters = if req.filters_json.is_empty() {
            std::collections::HashMap::new()
        } else {
            serde_json::from_slice(&req.filters_json)
                .map_err(|e| GrpcError::InvalidArgument(format!("invalid filters_json: {e}")))?
        };

        let input = SearchInput {
            index_name: req.index.clone(),
            query: req.query,
            from,
            size: page_size,
            filters,
            facets: req.facets,
            tenant_id: req.tenant_id,
        };

        match self.search_uc.execute(&input).await {
            Ok(result) => {
                let hits = result
                    .hits
                    .into_iter()
                    .map(|doc| {
                        let document_json = serde_json::to_vec(&doc.content).unwrap_or_default();
                        SearchHit {
                            id: doc.id,
                            score: doc.score,
                            document_json,
                        }
                    })
                    .collect();

                Ok(SearchResponse {
                    hits,
                    total_count: result.pagination.total_count,
                    page: result.pagination.page,
                    page_size: result.pagination.page_size,
                    has_next: result.pagination.has_next,
                    facets: result.facets,
                })
            }
            Err(SearchError::IndexNotFound(name)) => {
                Err(GrpcError::NotFound(format!("index not found: {name}")))
            }
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }

    /// CRIT-005 対応: `tenant_id` を使って RLS でテナント分離しながらドキュメントを削除する。
    pub async fn delete_document(
        &self,
        req: DeleteDocumentRequest,
    ) -> Result<DeleteDocumentResponse, GrpcError> {
        let input = crate::usecase::delete_document::DeleteDocumentInput {
            index_name: req.index.clone(),
            doc_id: req.document_id.clone(),
            tenant_id: req.tenant_id,
        };

        match self.delete_document_uc.execute(&input).await {
            Ok(_) => Ok(DeleteDocumentResponse {
                success: true,
                message: format!(
                    "document {} deleted from index {}",
                    req.document_id, req.index
                ),
            }),
            Err(DeleteDocumentError::NotFound(index, id)) => Err(GrpcError::NotFound(format!(
                "document not found: {index}/{id}"
            ))),
            Err(e) => Err(GrpcError::Internal(e.to_string())),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::domain::entity::search_index::{
        PaginationResult, SearchDocument, SearchIndex, SearchResult,
    };
    use crate::domain::repository::search_repository::MockSearchRepository;
    use crate::infrastructure::kafka_producer::NoopSearchEventPublisher;

    fn make_service(mock: MockSearchRepository) -> SearchGrpcService {
        let repo = Arc::new(mock);
        SearchGrpcService::new(
            Arc::new(CreateIndexUseCase::new(repo.clone())),
            Arc::new(ListIndicesUseCase::new(repo.clone())),
            Arc::new(IndexDocumentUseCase::new(
                repo.clone(),
                Arc::new(NoopSearchEventPublisher),
            )),
            Arc::new(SearchUseCase::new(repo.clone())),
            Arc::new(DeleteDocumentUseCase::new(repo)),
        )
    }

    #[tokio::test]
    async fn test_list_indices_success() {
        let mut mock = MockSearchRepository::new();
        mock.expect_list_indices().returning(|_| Ok(vec![]));

        let svc = make_service(mock);
        let resp = svc
            .list_indices(ListIndicesRequest {
                tenant_id: "tenant-a".to_string(),
            })
            .await
            .unwrap();
        assert!(resp.indices.is_empty());
    }

    #[tokio::test]
    async fn test_index_document_success() {
        let mut mock = MockSearchRepository::new();
        // テスト用のダミーインデックス（テナント IDは "tenant-a" を使用する）
        let index = SearchIndex::new(
            "products".to_string(),
            serde_json::json!({}),
            "tenant-a".to_string(),
        );
        let return_index = index.clone();

        mock.expect_find_index()
            .withf(|name, _tenant_id| name == "products")
            .returning(move |_, _| Ok(Some(return_index.clone())));
        mock.expect_index_document().returning(|_, _| Ok(()));

        let svc = make_service(mock);
        let req = IndexDocumentRequest {
            index: "products".to_string(),
            document_id: "doc-1".to_string(),
            document_json: serde_json::to_vec(&serde_json::json!({"name": "Widget"})).unwrap(),
            tenant_id: "tenant-a".to_string(),
        };
        let resp = svc.index_document(req).await.unwrap();

        assert_eq!(resp.document_id, "doc-1");
        assert_eq!(resp.index, "products");
        assert_eq!(resp.result, "created");
    }

    #[tokio::test]
    async fn test_search_success() {
        let mut mock = MockSearchRepository::new();
        // テスト用のダミーインデックス（テナント IDは "tenant-a" を使用する）
        let index = SearchIndex::new(
            "products".to_string(),
            serde_json::json!({}),
            "tenant-a".to_string(),
        );
        let return_index = index.clone();

        mock.expect_find_index()
            .withf(|name, _tenant_id| name == "products")
            .returning(move |_, _| Ok(Some(return_index.clone())));

        mock.expect_search().returning(|_, _| {
            let total = 1u64;
            Ok(SearchResult {
                total,
                hits: vec![SearchDocument {
                    id: "doc-1".to_string(),
                    index_name: "products".to_string(),
                    content: serde_json::json!({"name": "Widget"}),
                    score: 1.0,
                    indexed_at: chrono::Utc::now(),
                }],
                facets: std::collections::HashMap::new(),
                pagination: PaginationResult {
                    total_count: total,
                    page: 1,
                    page_size: 10,
                    has_next: false,
                },
            })
        });

        let svc = make_service(mock);
        let req = SearchRequest {
            index: "products".to_string(),
            query: "Widget".to_string(),
            filters_json: vec![],
            from: 0,
            size: 10,
            facets: vec![],
            tenant_id: "tenant-a".to_string(),
        };
        let resp = svc.search(req).await.unwrap();

        assert_eq!(resp.total_count, 1);
        assert_eq!(resp.hits.len(), 1);
        assert_eq!(resp.hits[0].id, "doc-1");
        assert!(!resp.has_next);
    }
}
