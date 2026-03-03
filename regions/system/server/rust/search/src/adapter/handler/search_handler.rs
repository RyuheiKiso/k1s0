use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::usecase::delete_document::{DeleteDocumentError, DeleteDocumentInput};
use crate::usecase::index_document::{IndexDocumentError, IndexDocumentInput};
use crate::usecase::search::{SearchError, SearchInput};
use crate::usecase::{
    CreateIndexUseCase, DeleteDocumentUseCase, IndexDocumentUseCase, ListIndicesUseCase,
    SearchUseCase,
};

use crate::adapter::middleware::auth::SearchAuthState;

#[derive(Clone)]
pub struct AppState {
    pub search_uc: Arc<SearchUseCase>,
    pub index_document_uc: Arc<IndexDocumentUseCase>,
    pub delete_document_uc: Arc<DeleteDocumentUseCase>,
    pub create_index_uc: Arc<CreateIndexUseCase>,
    pub list_indices_uc: Arc<ListIndicesUseCase>,
    pub metrics: Arc<k1s0_telemetry::metrics::Metrics>,
    pub auth_state: Option<SearchAuthState>,
}

impl AppState {
    pub fn with_auth(mut self, auth_state: SearchAuthState) -> Self {
        self.auth_state = Some(auth_state);
        self
    }
}

// --- Request / Response DTOs ---

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub index_name: String,
    pub query: String,
    #[serde(default)]
    pub from: u32,
    #[serde(default = "default_size")]
    pub size: u32,
    #[serde(default)]
    pub filters: HashMap<String, String>,
    #[serde(default)]
    pub facets: Vec<String>,
}

fn default_size() -> u32 {
    10
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
    pub facets: HashMap<String, HashMap<String, u64>>,
    pub hits: Vec<HitResponse>,
}

#[derive(Debug, Serialize)]
pub struct HitResponse {
    pub id: String,
    pub score: f32,
    pub document_json: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CreateIndexRequest {
    pub name: String,
    #[serde(default)]
    pub mapping: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct IndexDocumentRequest {
    pub id: String,
    pub index_name: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct IndexDocumentResponse {
    pub id: String,
    pub index_name: String,
    pub indexed_at: String,
}

// --- Handlers ---

/// POST /api/v1/search - Execute a search query
pub async fn search(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let input = SearchInput {
        index_name: req.index_name,
        query: req.query,
        from: req.from,
        size: req.size,
        filters: req.filters,
        facets: req.facets,
    };

    match state.search_uc.execute(&input).await {
        Ok(result) => {
            let page_size = req.size.max(1);
            let page = (req.from / page_size) + 1;
            let has_next = result.total > (req.from as u64 + result.hits.len() as u64);
            let resp = SearchResponse {
                total_count: result.total,
                page,
                page_size,
                has_next,
                facets: result.facets,
                hits: result
                    .hits
                    .into_iter()
                    .map(|h| HitResponse {
                        id: h.id,
                        score: h.score,
                        document_json: h.content,
                    })
                    .collect(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(SearchError::IndexNotFound(name)) => error_response(
            StatusCode::NOT_FOUND,
            "SYS_SEARCH_INDEX_NOT_FOUND",
            format!("index not found: {}", name),
        )
        .into_response(),
        Err(SearchError::Internal(msg)) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYS_SEARCH_INTERNAL_ERROR",
            msg,
        )
        .into_response(),
    }
}

/// POST /api/v1/search/index - Index a document
pub async fn index_document(
    State(state): State<AppState>,
    Json(req): Json<IndexDocumentRequest>,
) -> impl IntoResponse {
    let input = IndexDocumentInput {
        id: req.id,
        index_name: req.index_name,
        content: req.content,
    };

    match state.index_document_uc.execute(&input).await {
        Ok(doc) => {
            let resp = IndexDocumentResponse {
                id: doc.id,
                index_name: doc.index_name,
                indexed_at: doc.indexed_at.to_rfc3339(),
            };
            (
                StatusCode::CREATED,
                Json(serde_json::to_value(resp).unwrap()),
            )
                .into_response()
        }
        Err(IndexDocumentError::IndexNotFound(name)) => error_response(
            StatusCode::NOT_FOUND,
            "SYS_SEARCH_INDEX_NOT_FOUND",
            format!("index not found: {}", name),
        )
        .into_response(),
        Err(IndexDocumentError::Internal(msg)) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYS_SEARCH_INTERNAL_ERROR",
            msg,
        )
        .into_response(),
    }
}

/// POST /api/v1/search/indices - Create a search index
pub async fn create_index(
    State(state): State<AppState>,
    Json(req): Json<CreateIndexRequest>,
) -> impl IntoResponse {
    use crate::usecase::create_index::CreateIndexInput;

    let input = CreateIndexInput {
        name: req.name,
        mapping: req.mapping,
    };

    match state.create_index_uc.execute(&input).await {
        Ok(index) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "id": index.id.to_string(),
                "name": index.name,
                "mapping": index.mapping,
                "created_at": index.created_at.to_rfc3339()
            })),
        )
            .into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already exists") {
                error_response(
                    StatusCode::CONFLICT,
                    "SYS_SEARCH_INDEX_ALREADY_EXISTS",
                    msg,
                )
                .into_response()
            } else {
                error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "SYS_SEARCH_INTERNAL_ERROR",
                    msg,
                )
                .into_response()
            }
        }
    }
}

/// GET /api/v1/search/indices - List all search indices
pub async fn list_indices(State(state): State<AppState>) -> impl IntoResponse {
    match state.list_indices_uc.execute().await {
        Ok(indices) => {
            let items: Vec<serde_json::Value> = indices
                .into_iter()
                .map(|idx| {
                    serde_json::json!({
                        "id": idx.id.to_string(),
                        "name": idx.name,
                        "mapping": idx.mapping,
                        "created_at": idx.created_at.to_rfc3339()
                    })
                })
                .collect();
            (
                StatusCode::OK,
                Json(serde_json::json!({"indices": items})),
            )
                .into_response()
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYS_SEARCH_INTERNAL_ERROR",
            e.to_string(),
        )
        .into_response(),
    }
}

/// DELETE /api/v1/search/index/:index_name/:doc_id - Delete a document from an index
pub async fn delete_document_from_index(
    State(state): State<AppState>,
    Path((index_name, doc_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let input = DeleteDocumentInput {
        index_name,
        doc_id,
    };

    match state.delete_document_uc.execute(&input).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(DeleteDocumentError::NotFound(_, id)) => error_response(
            StatusCode::NOT_FOUND,
            "SYS_SEARCH_DOCUMENT_NOT_FOUND",
            format!("document not found: {}", id),
        )
        .into_response(),
        Err(DeleteDocumentError::Internal(msg)) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SYS_SEARCH_INTERNAL_ERROR",
            msg,
        )
        .into_response(),
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub request_id: String,
    pub details: Vec<String>,
}

fn error_response(
    status: StatusCode,
    code: &str,
    message: impl Into<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message: message.into(),
                request_id: uuid::Uuid::new_v4().to_string(),
                details: vec![],
            },
        }),
    )
}

