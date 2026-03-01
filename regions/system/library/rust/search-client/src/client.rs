use async_trait::async_trait;

use crate::document::{BulkResult, IndexDocument, IndexMapping, IndexResult};
use crate::error::SearchError;
use crate::query::{SearchQuery, SearchResult};

#[async_trait]
#[cfg_attr(feature = "mock", mockall::automock)]
pub trait SearchClient: Send + Sync {
    async fn index_document(
        &self,
        index: &str,
        doc: IndexDocument,
    ) -> Result<IndexResult, SearchError>;

    async fn bulk_index(
        &self,
        index: &str,
        docs: Vec<IndexDocument>,
    ) -> Result<BulkResult, SearchError>;

    async fn search(
        &self,
        index: &str,
        query: SearchQuery,
    ) -> Result<SearchResult<serde_json::Value>, SearchError>;

    async fn delete_document(&self, index: &str, id: &str) -> Result<(), SearchError>;

    async fn create_index(&self, name: &str, mapping: IndexMapping) -> Result<(), SearchError>;
}

use std::collections::HashMap;

pub struct InMemorySearchClient {
    documents: tokio::sync::Mutex<HashMap<String, Vec<IndexDocument>>>,
}

impl InMemorySearchClient {
    pub fn new() -> Self {
        Self {
            documents: tokio::sync::Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemorySearchClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SearchClient for InMemorySearchClient {
    async fn index_document(
        &self,
        index: &str,
        doc: IndexDocument,
    ) -> Result<IndexResult, SearchError> {
        let mut docs = self.documents.lock().await;
        let entry = docs.entry(index.to_string()).or_default();
        let version = entry.len() as i64 + 1;
        let id = doc.id.clone();
        entry.push(doc);
        Ok(IndexResult { id, version })
    }

    async fn bulk_index(
        &self,
        index: &str,
        docs: Vec<IndexDocument>,
    ) -> Result<BulkResult, SearchError> {
        let mut store = self.documents.lock().await;
        let entry = store.entry(index.to_string()).or_default();
        let count = docs.len();
        entry.extend(docs);
        Ok(BulkResult {
            success_count: count,
            failed_count: 0,
            failures: Vec::new(),
        })
    }

    async fn search(
        &self,
        index: &str,
        query: SearchQuery,
    ) -> Result<SearchResult<serde_json::Value>, SearchError> {
        let docs = self.documents.lock().await;
        let entry = docs
            .get(index)
            .ok_or_else(|| SearchError::IndexNotFound(index.to_string()))?;
        let hits: Vec<serde_json::Value> = entry
            .iter()
            .filter(|doc| {
                if query.query.is_empty() {
                    return true;
                }
                doc.fields.values().any(|v| {
                    v.as_str()
                        .map(|s| s.contains(&query.query))
                        .unwrap_or(false)
                })
            })
            .skip(query.page as usize * query.size as usize)
            .take(query.size as usize)
            .map(|doc| serde_json::to_value(doc).unwrap())
            .collect();
        let total = hits.len() as u64;
        let mut facets = HashMap::new();
        for facet_field in &query.facets {
            facets.insert(
                facet_field.clone(),
                vec![crate::query::FacetBucket {
                    value: "default".to_string(),
                    count: total,
                }],
            );
        }
        Ok(SearchResult {
            hits,
            total,
            facets,
            took_ms: 1,
        })
    }

    async fn delete_document(&self, index: &str, id: &str) -> Result<(), SearchError> {
        let mut docs = self.documents.lock().await;
        if let Some(entry) = docs.get_mut(index) {
            entry.retain(|doc| doc.id != id);
        }
        Ok(())
    }

    async fn create_index(&self, name: &str, _mapping: IndexMapping) -> Result<(), SearchError> {
        let mut docs = self.documents.lock().await;
        docs.entry(name.to_string()).or_default();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::BulkFailure;

    #[tokio::test]
    async fn test_index_and_search() {
        let client = InMemorySearchClient::new();
        let mapping = IndexMapping::new()
            .field("name", "text")
            .field("price", "integer");
        client.create_index("products", mapping).await.unwrap();

        let doc = IndexDocument::new("prod-001")
            .field("name", serde_json::json!("Rust Programming"))
            .field("price", serde_json::json!(3800));
        let result = client.index_document("products", doc).await.unwrap();
        assert_eq!(result.id, "prod-001");
        assert_eq!(result.version, 1);

        let query = SearchQuery::new("Rust").facet("name").page(0).size(10);
        let search_result = client.search("products", query).await.unwrap();
        assert_eq!(search_result.total, 1);
        assert_eq!(search_result.took_ms, 1);
    }

    #[tokio::test]
    async fn test_bulk_index() {
        let client = InMemorySearchClient::new();
        client
            .create_index("items", IndexMapping::new())
            .await
            .unwrap();

        let docs = vec![
            IndexDocument::new("i-1").field("name", serde_json::json!("Item 1")),
            IndexDocument::new("i-2").field("name", serde_json::json!("Item 2")),
        ];
        let result = client.bulk_index("items", docs).await.unwrap();
        assert_eq!(result.success_count, 2);
        assert_eq!(result.failed_count, 0);
        assert!(result.failures.is_empty());
    }

    #[tokio::test]
    async fn test_delete_document() {
        let client = InMemorySearchClient::new();
        client
            .create_index("products", IndexMapping::new())
            .await
            .unwrap();
        let doc = IndexDocument::new("prod-001").field("name", serde_json::json!("Test"));
        client.index_document("products", doc).await.unwrap();

        client
            .delete_document("products", "prod-001")
            .await
            .unwrap();

        let query = SearchQuery::new("").page(0).size(10);
        let result = client.search("products", query).await.unwrap();
        assert_eq!(result.total, 0);
    }

    #[tokio::test]
    async fn test_search_index_not_found() {
        let client = InMemorySearchClient::new();
        let query = SearchQuery::new("test");
        let result = client.search("nonexistent", query).await;
        assert!(matches!(result, Err(SearchError::IndexNotFound(_))));
    }

    #[tokio::test]
    async fn test_search_error_variants() {
        let err = SearchError::IndexNotFound("missing".to_string());
        assert!(matches!(err, SearchError::IndexNotFound(_)));

        let err = SearchError::InvalidQuery("bad query".to_string());
        assert!(matches!(err, SearchError::InvalidQuery(_)));

        let err = SearchError::ServerError("internal".to_string());
        assert!(matches!(err, SearchError::ServerError(_)));

        let err = SearchError::Timeout;
        assert!(matches!(err, SearchError::Timeout));
    }

    #[test]
    fn test_search_query_builder() {
        use crate::query::Filter;
        let query = SearchQuery::new("Rust")
            .filter(Filter::eq("category", "books"))
            .filter(Filter::range("price", 1000, 5000))
            .facet("category")
            .page(0)
            .size(10);

        assert_eq!(query.query, "Rust");
        assert_eq!(query.filters.len(), 2);
        assert_eq!(query.facets, vec!["category"]);
        assert_eq!(query.size, 10);
        assert_eq!(query.page, 0);
    }

    #[test]
    fn test_index_document_builder() {
        let doc = IndexDocument::new("prod-001")
            .field("name", serde_json::json!("test"))
            .field("price", serde_json::json!(100));

        assert_eq!(doc.id, "prod-001");
        assert_eq!(doc.fields.len(), 2);
    }

    #[test]
    fn test_index_mapping_builder() {
        let mapping = IndexMapping::new()
            .field("name", "text")
            .field("price", "integer")
            .field("category", "keyword");

        assert_eq!(mapping.fields.len(), 3);
        assert_eq!(mapping.fields["name"].field_type, "text");
        assert!(mapping.fields["name"].indexed);
    }

    #[test]
    fn test_filter_constructors() {
        use crate::query::Filter;
        let eq = Filter::eq("status", "active");
        assert_eq!(eq.operator, "eq");
        assert_eq!(eq.field, "status");

        let lt = Filter::lt("price", 100);
        assert_eq!(lt.operator, "lt");

        let gt = Filter::gt("price", 50);
        assert_eq!(gt.operator, "gt");

        let range = Filter::range("price", 10, 100);
        assert_eq!(range.operator, "range");
        assert!(range.value_to.is_some());
    }

    #[test]
    fn test_bulk_failure() {
        let failure = BulkFailure {
            id: "doc-1".to_string(),
            error: "mapping error".to_string(),
        };
        assert_eq!(failure.id, "doc-1");
        assert_eq!(failure.error, "mapping error");
    }
}
