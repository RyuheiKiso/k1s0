use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::usecase::delete_document::{DeleteDocumentError, DeleteDocumentInput};
use crate::usecase::index_document::{IndexDocumentError, IndexDocumentInput};
use crate::usecase::search::{SearchError, SearchInput};
use crate::usecase::{DeleteDocumentUseCase, IndexDocumentUseCase, SearchUseCase};

#[derive(Clone)]
pub struct AppState {
    pub search_uc: Arc<SearchUseCase>,
    pub index_document_uc: Arc<IndexDocumentUseCase>,
    pub delete_document_uc: Arc<DeleteDocumentUseCase>,
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
}

fn default_size() -> u32 {
    10
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub total: u64,
    pub hits: Vec<HitResponse>,
}

#[derive(Debug, Serialize)]
pub struct HitResponse {
    pub id: String,
    pub index_name: String,
    pub content: serde_json::Value,
    pub indexed_at: String,
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
    };

    match state.search_uc.execute(&input).await {
        Ok(result) => {
            let resp = SearchResponse {
                total: result.total,
                hits: result
                    .hits
                    .into_iter()
                    .map(|h| HitResponse {
                        id: h.id,
                        index_name: h.index_name,
                        content: h.content,
                        indexed_at: h.indexed_at.to_rfc3339(),
                    })
                    .collect(),
            };
            (StatusCode::OK, Json(serde_json::to_value(resp).unwrap())).into_response()
        }
        Err(SearchError::IndexNotFound(name)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("index not found: {}", name)})),
        )
            .into_response(),
        Err(SearchError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
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
        Err(IndexDocumentError::IndexNotFound(name)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("index not found: {}", name)})),
        )
            .into_response(),
        Err(IndexDocumentError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
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
        Err(DeleteDocumentError::NotFound(_, id)) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("document not found: {}", id)})),
        )
            .into_response(),
        Err(DeleteDocumentError::Internal(msg)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
    }
}
