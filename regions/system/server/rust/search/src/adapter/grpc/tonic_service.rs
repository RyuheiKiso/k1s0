//! tonic gRPC サービス実装。
//!
//! proto 生成コード (`src/proto/`) の SearchService トレイトを実装する。
//! 各メソッドで proto 型 <-> 手動型の変換を行い、既存の SearchGrpcService に委譲する。

use std::sync::Arc;

use tonic::{Request, Response, Status};

use crate::proto::k1s0::system::search::v1::{
    search_service_server::SearchService, DeleteDocumentRequest as ProtoDeleteDocumentRequest,
    DeleteDocumentResponse as ProtoDeleteDocumentResponse,
    IndexDocumentRequest as ProtoIndexDocumentRequest,
    IndexDocumentResponse as ProtoIndexDocumentResponse, SearchHit as ProtoSearchHit,
    SearchRequest as ProtoSearchRequest, SearchResponse as ProtoSearchResponse,
};

use super::search_grpc::{
    DeleteDocumentRequest, GrpcError, IndexDocumentRequest, SearchGrpcService, SearchRequest,
};

// --- GrpcError -> tonic::Status 変換 ---

impl From<GrpcError> for Status {
    fn from(e: GrpcError) -> Self {
        match e {
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
        }
    }
}

// --- SearchService tonic ラッパー ---

/// SearchServiceTonic は tonic の SearchService として SearchGrpcService をラップする。
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
            page: inner.page,
            page_size: inner.page_size,
        };
        let resp = self
            .inner
            .search(req)
            .await
            .map_err(Into::<Status>::into)?;

        let proto_hits = resp
            .hits
            .into_iter()
            .map(|h| ProtoSearchHit {
                id: h.id,
                score: h.score,
                document_json: h.document_json,
            })
            .collect();

        Ok(Response::new(ProtoSearchResponse {
            hits: proto_hits,
            total_count: resp.total_count,
            page: resp.page,
            page_size: resp.page_size,
            has_next: resp.has_next,
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
mod tests {
    use super::*;
    use crate::domain::entity::search_index::{SearchDocument, SearchIndex, SearchResult};
    use crate::domain::repository::search_repository::MockSearchRepository;
    use crate::usecase::delete_document::DeleteDocumentUseCase;
    use crate::usecase::index_document::IndexDocumentUseCase;
    use crate::usecase::search::SearchUseCase;

    fn make_tonic_service(mock: MockSearchRepository) -> SearchServiceTonic {
        let repo = Arc::new(mock);
        let grpc_svc = Arc::new(SearchGrpcService::new(
            Arc::new(IndexDocumentUseCase::new(repo.clone())),
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
        assert!(status.message().contains("index not found"));
    }

    #[test]
    fn test_grpc_error_invalid_argument_to_status() {
        let err = GrpcError::InvalidArgument("invalid document_json".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert!(status.message().contains("invalid document_json"));
    }

    #[test]
    fn test_grpc_error_internal_to_status() {
        let err = GrpcError::Internal("search engine error".to_string());
        let status: Status = err.into();
        assert_eq!(status.code(), tonic::Code::Internal);
        assert!(status.message().contains("search engine error"));
    }

    #[tokio::test]
    async fn test_search_service_tonic_index_document() {
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
            document_json: serde_json::to_vec(&serde_json::json!({"name": "Widget"})).unwrap(),
        });
        let resp = tonic_svc.index_document(req).await.unwrap();
        let inner = resp.into_inner();

        assert_eq!(inner.document_id, "doc-1");
        assert_eq!(inner.index, "products");
        assert_eq!(inner.result, "created");
    }

    #[tokio::test]
    async fn test_search_service_tonic_search() {
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

        let tonic_svc = make_tonic_service(mock);
        let req = Request::new(ProtoSearchRequest {
            index: "products".to_string(),
            query: "Widget".to_string(),
            filters_json: vec![],
            page: 1,
            page_size: 10,
        });
        let resp = tonic_svc.search(req).await.unwrap();
        let inner = resp.into_inner();

        assert_eq!(inner.total_count, 1);
        assert_eq!(inner.hits.len(), 1);
        assert_eq!(inner.hits[0].id, "doc-1");
    }

    #[tokio::test]
    async fn test_search_service_tonic_delete_document() {
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
