use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::common::v1::PaginationResult as ProtoPaginationResult;
use crate::proto::k1s0::system::search::v1::{
    search_service_server::SearchService, CreateIndexRequest as ProtoCreateIndexRequest,
    CreateIndexResponse as ProtoCreateIndexResponse,
    DeleteDocumentRequest as ProtoDeleteDocumentRequest,
    DeleteDocumentResponse as ProtoDeleteDocumentResponse, FacetCounts as ProtoFacetCounts,
    IndexDocumentRequest as ProtoIndexDocumentRequest,
    IndexDocumentResponse as ProtoIndexDocumentResponse,
    ListIndicesRequest as ProtoListIndicesRequest, ListIndicesResponse as ProtoListIndicesResponse,
    SearchHit as ProtoSearchHit, SearchIndex as ProtoSearchIndex,
    SearchRequest as ProtoSearchRequest, SearchResponse as ProtoSearchResponse,
};

use super::search_grpc::{
    CreateIndexRequest, DeleteDocumentRequest, GrpcError, IndexDocumentRequest, ListIndicesRequest,
    SearchGrpcService, SearchRequest,
};

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

pub struct SearchServiceTonic {
    inner: Arc<SearchGrpcService>,
}

impl SearchServiceTonic {
    pub fn new(inner: Arc<SearchGrpcService>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl SearchService for SearchServiceTonic {
    async fn create_index(
        &self,
        request: Request<ProtoCreateIndexRequest>,
    ) -> Result<Response<ProtoCreateIndexResponse>, Status> {
        let inner = request.into_inner();
        let req = CreateIndexRequest {
            name: inner.name,
            mapping_json: inner.mapping_json,
        };
        let resp = self
            .inner
            .create_index(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoCreateIndexResponse {
            index: Some(ProtoSearchIndex {
                id: resp.index.id,
                name: resp.index.name,
                mapping_json: resp.index.mapping_json,
                created_at: resp.index.created_at,
            }),
        }))
    }

    async fn list_indices(
        &self,
        _request: Request<ProtoListIndicesRequest>,
    ) -> Result<Response<ProtoListIndicesResponse>, Status> {
        let resp = self
            .inner
            .list_indices(ListIndicesRequest {})
            .await
            .map_err(Into::<Status>::into)?;

        let indices = resp
            .indices
            .into_iter()
            .map(|index| ProtoSearchIndex {
                id: index.id,
                name: index.name,
                mapping_json: index.mapping_json,
                created_at: index.created_at,
            })
            .collect();

        Ok(Response::new(ProtoListIndicesResponse { indices }))
    }

    async fn index_document(
        &self,
        request: Request<ProtoIndexDocumentRequest>,
    ) -> Result<Response<ProtoIndexDocumentResponse>, Status> {
        let inner = request.into_inner();
        let req = IndexDocumentRequest {
            index: inner.index,
            document_id: inner.document_id,
            document_json: inner.document_json,
        };
        let resp = self
            .inner
            .index_document(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoIndexDocumentResponse {
            document_id: resp.document_id,
            index: resp.index,
            result: resp.result,
        }))
    }

    async fn search(
        &self,
        request: Request<ProtoSearchRequest>,
    ) -> Result<Response<ProtoSearchResponse>, Status> {
        let inner = request.into_inner();
        let req = SearchRequest {
            index: inner.index,
            query: inner.query,
            filters_json: inner.filters_json,
            from: inner.from,
            size: inner.size,
            facets: inner.facets,
        };
        let resp = self.inner.search(req).await.map_err(Into::<Status>::into)?;

        let proto_hits = resp
            .hits
            .into_iter()
            .map(|hit| ProtoSearchHit {
                id: hit.id,
                score: hit.score,
                document_json: hit.document_json,
            })
            .collect();

        Ok(Response::new(ProtoSearchResponse {
            hits: proto_hits,
            pagination: Some(ProtoPaginationResult {
                total_count: resp.total_count as i64,
                page: resp.page as i32,
                page_size: resp.page_size as i32,
                has_next: resp.has_next,
            }),
            facets: resp
                .facets
                .into_iter()
                .map(|(name, buckets)| (name, ProtoFacetCounts { buckets }))
                .collect(),
        }))
    }

    async fn delete_document(
        &self,
        request: Request<ProtoDeleteDocumentRequest>,
    ) -> Result<Response<ProtoDeleteDocumentResponse>, Status> {
        let inner = request.into_inner();
        let req = DeleteDocumentRequest {
            index: inner.index,
            document_id: inner.document_id,
        };
        let resp = self
            .inner
            .delete_document(req)
            .await
            .map_err(Into::<Status>::into)?;

        Ok(Response::new(ProtoDeleteDocumentResponse {
            success: resp.success,
            message: resp.message,
        }))
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
    use crate::usecase::create_index::CreateIndexUseCase;
    use crate::usecase::delete_document::DeleteDocumentUseCase;
    use crate::usecase::index_document::IndexDocumentUseCase;
    use crate::usecase::list_indices::ListIndicesUseCase;
    use crate::usecase::search::SearchUseCase;

    fn make_tonic_service(mock: MockSearchRepository) -> SearchServiceTonic {
        let repo = Arc::new(mock);
        let grpc_svc = Arc::new(SearchGrpcService::new(
            Arc::new(CreateIndexUseCase::new(repo.clone())),
            Arc::new(ListIndicesUseCase::new(repo.clone())),
            Arc::new(IndexDocumentUseCase::new(
                repo.clone(),
                Arc::new(NoopSearchEventPublisher),
            )),
            Arc::new(SearchUseCase::new(repo.clone())),
            Arc::new(DeleteDocumentUseCase::new(repo)),
        ));
        SearchServiceTonic::new(grpc_svc)
    }

    #[test]
    fn test_grpc_error_not_found_to_status() {
        let err = GrpcError::NotFound("index not found".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("invalid document_json".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("search engine error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    #[tokio::test]
    async fn test_create_index() {
        let mut mock = MockSearchRepository::new();
        mock.expect_find_index()
            .withf(|name| name == "products")
            .returning(|_| Ok(None));
        mock.expect_create_index().returning(|_| Ok(()));

        let tonic_svc = make_tonic_service(mock);
        let req = Request::new(ProtoCreateIndexRequest {
            name: "products".to_string(),
            mapping_json: serde_json::to_vec(&serde_json::json!({"fields":["name"]})).unwrap(),
        });
        let resp = tonic_svc.create_index(req).await.unwrap();
        let inner = resp.into_inner();

        assert!(inner.index.is_some());
        assert_eq!(inner.index.unwrap().name, "products");
    }

    #[tokio::test]
    async fn test_list_indices() {
        let mut mock = MockSearchRepository::new();
        let index = SearchIndex::new("products".to_string(), serde_json::json!({}));
        let return_index = index.clone();

        mock.expect_list_indices()
            .returning(move || Ok(vec![return_index.clone()]));

        let tonic_svc = make_tonic_service(mock);
        let resp = tonic_svc
            .list_indices(Request::new(ProtoListIndicesRequest {}))
            .await
            .unwrap();
        let inner = resp.into_inner();

        assert_eq!(inner.indices.len(), 1);
        assert_eq!(inner.indices[0].name, "products");
    }

    #[tokio::test]
    async fn test_index_document() {
        let mut mock = MockSearchRepository::new();
        let index = SearchIndex::new("products".to_string(), serde_json::json!({}));
        let return_index = index.clone();

        mock.expect_find_index()
            .withf(|name| name == "products")
            .returning(move |_| Ok(Some(return_index.clone())));
        mock.expect_index_document().returning(|_| Ok(()));

        let tonic_svc = make_tonic_service(mock);
        let req = Request::new(ProtoIndexDocumentRequest {
            index: "products".to_string(),
            document_id: "doc-1".to_string(),
            document_json: serde_json::to_vec(&serde_json::json!({"name":"Widget"})).unwrap(),
        });
        let resp = tonic_svc.index_document(req).await.unwrap();
        let inner = resp.into_inner();

        assert_eq!(inner.document_id, "doc-1");
        assert_eq!(inner.index, "products");
        assert_eq!(inner.result, "created");
    }

    #[tokio::test]
    async fn test_search() {
        let mut mock = MockSearchRepository::new();
        let index = SearchIndex::new("products".to_string(), serde_json::json!({}));
        let return_index = index.clone();

        mock.expect_find_index()
            .withf(|name| name == "products")
            .returning(move |_| Ok(Some(return_index.clone())));
        mock.expect_search().returning(|_| {
            let total = 1u64;
            Ok(SearchResult {
                total,
                hits: vec![SearchDocument {
                    id: "doc-1".to_string(),
                    index_name: "products".to_string(),
                    content: serde_json::json!({"name":"Widget"}),
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

        let tonic_svc = make_tonic_service(mock);
        let req = Request::new(ProtoSearchRequest {
            index: "products".to_string(),
            query: "Widget".to_string(),
            filters_json: vec![],
            from: 0,
            size: 10,
            facets: vec![],
        });
        let resp = tonic_svc.search(req).await.unwrap();
        let inner = resp.into_inner();

        let pagination = inner.pagination.expect("pagination");
        assert_eq!(pagination.total_count, 1);
        assert_eq!(inner.hits.len(), 1);
        assert_eq!(inner.hits[0].id, "doc-1");
    }

    #[tokio::test]
    async fn test_delete_document() {
        let mut mock = MockSearchRepository::new();
        mock.expect_delete_document()
            .withf(|index, id| index == "products" && id == "doc-1")
            .returning(|_, _| Ok(true));

        let tonic_svc = make_tonic_service(mock);
        let req = Request::new(ProtoDeleteDocumentRequest {
            index: "products".to_string(),
            document_id: "doc-1".to_string(),
        });
        let resp = tonic_svc.delete_document(req).await.unwrap();
        let inner = resp.into_inner();

        assert!(inner.success);
        assert!(inner.message.contains("doc-1"));
    }
}
