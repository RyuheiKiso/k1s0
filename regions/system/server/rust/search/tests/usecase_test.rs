#![allow(clippy::unwrap_used)]
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;

use k1s0_search_server::domain::entity::search_index::{
    PaginationResult, SearchDocument, SearchIndex, SearchQuery, SearchResult,
};
use k1s0_search_server::domain::repository::SearchRepository;
use k1s0_search_server::infrastructure::kafka_producer::{
    DocumentIndexedEvent, SearchEventPublisher,
};
use k1s0_search_server::usecase::create_index::{
    CreateIndexError, CreateIndexInput, CreateIndexUseCase,
};
use k1s0_search_server::usecase::delete_document::{
    DeleteDocumentError, DeleteDocumentInput, DeleteDocumentUseCase,
};
use k1s0_search_server::usecase::index_document::{
    IndexDocumentError, IndexDocumentInput, IndexDocumentUseCase,
};
use k1s0_search_server::usecase::list_indices::{ListIndicesError, ListIndicesUseCase};
use k1s0_search_server::usecase::search::{SearchError, SearchInput, SearchUseCase};

// ---------------------------------------------------------------------------
// Stub: In-memory SearchRepository
// ---------------------------------------------------------------------------

struct StubSearchRepository {
    indices: RwLock<Vec<SearchIndex>>,
    documents: RwLock<Vec<SearchDocument>>,
    force_error: Option<String>,
}

impl StubSearchRepository {
    fn new() -> Self {
        Self {
            indices: RwLock::new(Vec::new()),
            documents: RwLock::new(Vec::new()),
            force_error: None,
        }
    }

    fn with_indices(indices: Vec<SearchIndex>) -> Self {
        Self {
            indices: RwLock::new(indices),
            documents: RwLock::new(Vec::new()),
            force_error: None,
        }
    }

    fn with_indices_and_docs(indices: Vec<SearchIndex>, docs: Vec<SearchDocument>) -> Self {
        Self {
            indices: RwLock::new(indices),
            documents: RwLock::new(docs),
            force_error: None,
        }
    }

    fn with_error(msg: &str) -> Self {
        Self {
            indices: RwLock::new(Vec::new()),
            documents: RwLock::new(Vec::new()),
            force_error: Some(msg.to_string()),
        }
    }
}

#[async_trait]
impl SearchRepository for StubSearchRepository {
    async fn create_index(&self, index: &SearchIndex) -> anyhow::Result<()> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        self.indices.write().await.push(index.clone());
        Ok(())
    }

    async fn find_index(&self, name: &str) -> anyhow::Result<Option<SearchIndex>> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let indices = self.indices.read().await;
        Ok(indices.iter().find(|i| i.name == name).cloned())
    }

    async fn index_document(&self, doc: &SearchDocument) -> anyhow::Result<()> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let mut docs = self.documents.write().await;
        // Upsert: replace if same id + index_name
        docs.retain(|d| !(d.id == doc.id && d.index_name == doc.index_name));
        docs.push(doc.clone());
        Ok(())
    }

    async fn search(&self, query: &SearchQuery) -> anyhow::Result<SearchResult> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let docs = self.documents.read().await;
        let mut hits: Vec<SearchDocument> = docs
            .iter()
            .filter(|d| d.index_name == query.index_name)
            .filter(|d| {
                if query.query.is_empty() {
                    return true;
                }
                let content_str = d.content.to_string().to_lowercase();
                content_str.contains(&query.query.to_lowercase())
            })
            .filter(|d| {
                // Apply filters: all filter key-value pairs must match in content
                for (key, value) in &query.filters {
                    if let Some(field_val) = d.content.get(key) {
                        let field_str = field_val.as_str().unwrap_or("");
                        if field_str != value {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Assign simple scores
        for (i, hit) in hits.iter_mut().enumerate() {
            hit.score = 1.0 / (i as f32 + 1.0);
        }

        let total = hits.len() as u64;
        let from = query.from as usize;
        let size = query.size as usize;
        let page = if size > 0 { from / size + 1 } else { 1 };
        let paged_hits: Vec<SearchDocument> = hits.into_iter().skip(from).take(size).collect();
        let has_next = (from + size) < total as usize;

        // Build facets from requested facet fields
        let mut facets: HashMap<String, HashMap<String, u64>> = HashMap::new();
        for facet_field in &query.facets {
            let mut counts: HashMap<String, u64> = HashMap::new();
            for d in docs.iter().filter(|d| d.index_name == query.index_name) {
                if let Some(val) = d.content.get(facet_field) {
                    let val_str = val.as_str().unwrap_or("unknown").to_string();
                    *counts.entry(val_str).or_insert(0) += 1;
                }
            }
            if !counts.is_empty() {
                facets.insert(facet_field.clone(), counts);
            }
        }

        Ok(SearchResult {
            total,
            hits: paged_hits,
            facets,
            pagination: PaginationResult {
                total_count: total,
                page: page as u32,
                page_size: size as u32,
                has_next,
            },
        })
    }

    async fn delete_document(&self, index_name: &str, doc_id: &str) -> anyhow::Result<bool> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        let mut docs = self.documents.write().await;
        let len_before = docs.len();
        docs.retain(|d| !(d.index_name == index_name && d.id == doc_id));
        Ok(docs.len() < len_before)
    }

    async fn list_indices(&self) -> anyhow::Result<Vec<SearchIndex>> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        Ok(self.indices.read().await.clone())
    }
}

// ---------------------------------------------------------------------------
// Stub: In-memory SearchEventPublisher
// ---------------------------------------------------------------------------

struct StubSearchEventPublisher {
    events: RwLock<Vec<String>>,
    force_error: Option<String>,
}

impl StubSearchEventPublisher {
    fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            force_error: None,
        }
    }

    fn with_error(msg: &str) -> Self {
        Self {
            events: RwLock::new(Vec::new()),
            force_error: Some(msg.to_string()),
        }
    }
}

#[async_trait]
impl SearchEventPublisher for StubSearchEventPublisher {
    async fn publish_document_indexed(&self, event: &DocumentIndexedEvent) -> anyhow::Result<()> {
        if let Some(ref msg) = self.force_error {
            return Err(anyhow::anyhow!("{}", msg));
        }
        self.events
            .write()
            .await
            .push(format!("{}:{}", event.index_name, event.document_id));
        Ok(())
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_index(name: &str) -> SearchIndex {
    SearchIndex::new(
        name.to_string(),
        serde_json::json!({"fields": ["title", "body"]}),
    )
}

fn make_document(id: &str, index_name: &str, content: serde_json::Value) -> SearchDocument {
    SearchDocument {
        id: id.to_string(),
        index_name: index_name.to_string(),
        content,
        score: 0.0,
        indexed_at: Utc::now(),
    }
}

// ===========================================================================
// CreateIndex tests
// ===========================================================================

#[tokio::test]
async fn create_index_success() {
    let repo = Arc::new(StubSearchRepository::new());
    let uc = CreateIndexUseCase::new(repo.clone());

    let input = CreateIndexInput {
        name: "products".to_string(),
        mapping: serde_json::json!({"fields": ["name", "description", "price"]}),
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.name, "products");
    assert_eq!(result.mapping["fields"][0], "name");

    // Verify persisted
    let stored = repo.indices.read().await;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].name, "products");
}

#[tokio::test]
async fn create_index_already_exists() {
    let existing = make_index("products");
    let repo = Arc::new(StubSearchRepository::with_indices(vec![existing]));
    let uc = CreateIndexUseCase::new(repo);

    let input = CreateIndexInput {
        name: "products".to_string(),
        mapping: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreateIndexError::AlreadyExists(name) => assert_eq!(name, "products"),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_index_repo_error() {
    let repo = Arc::new(StubSearchRepository::with_error("connection refused"));
    let uc = CreateIndexUseCase::new(repo);

    let input = CreateIndexInput {
        name: "products".to_string(),
        mapping: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        CreateIndexError::Internal(msg) => assert!(msg.contains("connection refused")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn create_index_with_complex_mapping() {
    let repo = Arc::new(StubSearchRepository::new());
    let uc = CreateIndexUseCase::new(repo);

    let mapping = serde_json::json!({
        "properties": {
            "title": {"type": "text", "analyzer": "standard"},
            "price": {"type": "float"},
            "tags": {"type": "keyword"},
            "created_at": {"type": "date"}
        }
    });
    let input = CreateIndexInput {
        name: "advanced-products".to_string(),
        mapping: mapping.clone(),
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.name, "advanced-products");
    assert_eq!(result.mapping, mapping);
}

#[tokio::test]
async fn create_multiple_indices() {
    let repo = Arc::new(StubSearchRepository::new());
    let uc = CreateIndexUseCase::new(repo.clone());

    let names = vec!["products", "users", "orders"];
    for name in &names {
        let input = CreateIndexInput {
            name: name.to_string(),
            mapping: serde_json::json!({}),
        };
        uc.execute(&input).await.unwrap();
    }

    let stored = repo.indices.read().await;
    assert_eq!(stored.len(), 3);
}

// ===========================================================================
// IndexDocument tests
// ===========================================================================

#[tokio::test]
async fn index_document_success() {
    let index = make_index("products");
    let repo = Arc::new(StubSearchRepository::with_indices(vec![index]));
    let publisher = Arc::new(StubSearchEventPublisher::new());
    let uc = IndexDocumentUseCase::new(repo.clone(), publisher.clone());

    let input = IndexDocumentInput {
        id: "doc-1".to_string(),
        index_name: "products".to_string(),
        content: serde_json::json!({"name": "Widget", "price": 9.99}),
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.id, "doc-1");
    assert_eq!(result.index_name, "products");
    assert_eq!(result.content["name"], "Widget");
    assert_eq!(result.score, 0.0); // default score on indexing

    // Verify persisted
    let stored = repo.documents.read().await;
    assert_eq!(stored.len(), 1);

    // Verify event published
    let events = publisher.events.read().await;
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], "products:doc-1");
}

#[tokio::test]
async fn index_document_index_not_found() {
    let repo = Arc::new(StubSearchRepository::new()); // no indices
    let publisher = Arc::new(StubSearchEventPublisher::new());
    let uc = IndexDocumentUseCase::new(repo, publisher);

    let input = IndexDocumentInput {
        id: "doc-1".to_string(),
        index_name: "nonexistent".to_string(),
        content: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        IndexDocumentError::IndexNotFound(name) => assert_eq!(name, "nonexistent"),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn index_document_repo_error() {
    let repo = Arc::new(StubSearchRepository::with_error("write failed"));
    let publisher = Arc::new(StubSearchEventPublisher::new());
    let uc = IndexDocumentUseCase::new(repo, publisher);

    let input = IndexDocumentInput {
        id: "doc-1".to_string(),
        index_name: "products".to_string(),
        content: serde_json::json!({}),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        IndexDocumentError::Internal(msg) => assert!(msg.contains("write failed")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn index_document_publisher_error() {
    let index = make_index("products");
    let repo = Arc::new(StubSearchRepository::with_indices(vec![index]));
    let publisher = Arc::new(StubSearchEventPublisher::with_error("broker down"));
    let uc = IndexDocumentUseCase::new(repo, publisher);

    let input = IndexDocumentInput {
        id: "doc-1".to_string(),
        index_name: "products".to_string(),
        content: serde_json::json!({"name": "Widget"}),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        IndexDocumentError::Internal(msg) => assert!(msg.contains("broker down")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn index_multiple_documents() {
    let index = make_index("products");
    let repo = Arc::new(StubSearchRepository::with_indices(vec![index]));
    let publisher = Arc::new(StubSearchEventPublisher::new());
    let uc = IndexDocumentUseCase::new(repo.clone(), publisher);

    for i in 1..=5 {
        let input = IndexDocumentInput {
            id: format!("doc-{}", i),
            index_name: "products".to_string(),
            content: serde_json::json!({"name": format!("Product {}", i)}),
        };
        uc.execute(&input).await.unwrap();
    }

    let stored = repo.documents.read().await;
    assert_eq!(stored.len(), 5);
}

#[tokio::test]
async fn index_document_upsert_same_id() {
    let index = make_index("products");
    let repo = Arc::new(StubSearchRepository::with_indices(vec![index]));
    let publisher = Arc::new(StubSearchEventPublisher::new());
    let uc = IndexDocumentUseCase::new(repo.clone(), publisher);

    // Index the same doc twice with different content
    let input1 = IndexDocumentInput {
        id: "doc-1".to_string(),
        index_name: "products".to_string(),
        content: serde_json::json!({"name": "Widget v1"}),
    };
    uc.execute(&input1).await.unwrap();

    let input2 = IndexDocumentInput {
        id: "doc-1".to_string(),
        index_name: "products".to_string(),
        content: serde_json::json!({"name": "Widget v2"}),
    };
    uc.execute(&input2).await.unwrap();

    let stored = repo.documents.read().await;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].content["name"], "Widget v2");
}

// ===========================================================================
// Search tests
// ===========================================================================

#[tokio::test]
async fn search_success_with_hits() {
    let index = make_index("products");
    let docs = vec![
        make_document(
            "doc-1",
            "products",
            serde_json::json!({"name": "Red Widget", "category": "tools"}),
        ),
        make_document(
            "doc-2",
            "products",
            serde_json::json!({"name": "Blue Gadget", "category": "electronics"}),
        ),
        make_document(
            "doc-3",
            "products",
            serde_json::json!({"name": "Green Widget", "category": "tools"}),
        ),
    ];
    let repo = Arc::new(StubSearchRepository::with_indices_and_docs(
        vec![index],
        docs,
    ));
    let uc = SearchUseCase::new(repo);

    let input = SearchInput {
        index_name: "products".to_string(),
        query: "Widget".to_string(),
        from: 0,
        size: 10,
        filters: HashMap::new(),
        facets: vec![],
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.total, 2);
    assert_eq!(result.hits.len(), 2);
    assert!(result
        .hits
        .iter()
        .all(|h| h.content["name"].as_str().unwrap().contains("Widget")));
}

#[tokio::test]
async fn search_empty_query_returns_all() {
    let index = make_index("products");
    let docs = vec![
        make_document("doc-1", "products", serde_json::json!({"name": "A"})),
        make_document("doc-2", "products", serde_json::json!({"name": "B"})),
    ];
    let repo = Arc::new(StubSearchRepository::with_indices_and_docs(
        vec![index],
        docs,
    ));
    let uc = SearchUseCase::new(repo);

    let input = SearchInput {
        index_name: "products".to_string(),
        query: "".to_string(),
        from: 0,
        size: 10,
        filters: HashMap::new(),
        facets: vec![],
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.total, 2);
}

#[tokio::test]
async fn search_no_hits() {
    let index = make_index("products");
    let docs = vec![make_document(
        "doc-1",
        "products",
        serde_json::json!({"name": "Widget"}),
    )];
    let repo = Arc::new(StubSearchRepository::with_indices_and_docs(
        vec![index],
        docs,
    ));
    let uc = SearchUseCase::new(repo);

    let input = SearchInput {
        index_name: "products".to_string(),
        query: "Nonexistent".to_string(),
        from: 0,
        size: 10,
        filters: HashMap::new(),
        facets: vec![],
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.total, 0);
    assert!(result.hits.is_empty());
}

#[tokio::test]
async fn search_index_not_found() {
    let repo = Arc::new(StubSearchRepository::new());
    let uc = SearchUseCase::new(repo);

    let input = SearchInput {
        index_name: "nonexistent".to_string(),
        query: "test".to_string(),
        from: 0,
        size: 10,
        filters: HashMap::new(),
        facets: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        SearchError::IndexNotFound(name) => assert_eq!(name, "nonexistent"),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn search_repo_error() {
    let repo = Arc::new(StubSearchRepository::with_error("search engine down"));
    let uc = SearchUseCase::new(repo);

    let input = SearchInput {
        index_name: "products".to_string(),
        query: "test".to_string(),
        from: 0,
        size: 10,
        filters: HashMap::new(),
        facets: vec![],
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        SearchError::Internal(msg) => assert!(msg.contains("search engine down")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn search_with_pagination() {
    let index = make_index("products");
    let mut docs = Vec::new();
    for i in 1..=10 {
        docs.push(make_document(
            &format!("doc-{}", i),
            "products",
            serde_json::json!({"name": format!("Product {}", i)}),
        ));
    }
    let repo = Arc::new(StubSearchRepository::with_indices_and_docs(
        vec![index],
        docs,
    ));
    let uc = SearchUseCase::new(repo);

    // First page
    let input = SearchInput {
        index_name: "products".to_string(),
        query: "".to_string(),
        from: 0,
        size: 3,
        filters: HashMap::new(),
        facets: vec![],
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.total, 10);
    assert_eq!(result.hits.len(), 3);
    assert!(result.pagination.has_next);
    assert_eq!(result.pagination.page, 1);

    // Second page
    let input2 = SearchInput {
        index_name: "products".to_string(),
        query: "".to_string(),
        from: 3,
        size: 3,
        filters: HashMap::new(),
        facets: vec![],
    };
    let result2 = uc.execute(&input2).await.unwrap();

    assert_eq!(result2.hits.len(), 3);
    assert!(result2.pagination.has_next);

    // Last page
    let input3 = SearchInput {
        index_name: "products".to_string(),
        query: "".to_string(),
        from: 9,
        size: 3,
        filters: HashMap::new(),
        facets: vec![],
    };
    let result3 = uc.execute(&input3).await.unwrap();

    assert_eq!(result3.hits.len(), 1);
    assert!(!result3.pagination.has_next);
}

#[tokio::test]
async fn search_with_filters() {
    let index = make_index("products");
    let docs = vec![
        make_document(
            "doc-1",
            "products",
            serde_json::json!({"name": "Widget", "category": "tools"}),
        ),
        make_document(
            "doc-2",
            "products",
            serde_json::json!({"name": "Gadget", "category": "electronics"}),
        ),
        make_document(
            "doc-3",
            "products",
            serde_json::json!({"name": "Tool", "category": "tools"}),
        ),
    ];
    let repo = Arc::new(StubSearchRepository::with_indices_and_docs(
        vec![index],
        docs,
    ));
    let uc = SearchUseCase::new(repo);

    let mut filters = HashMap::new();
    filters.insert("category".to_string(), "tools".to_string());

    let input = SearchInput {
        index_name: "products".to_string(),
        query: "".to_string(),
        from: 0,
        size: 10,
        filters,
        facets: vec![],
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.total, 2);
    assert!(result.hits.iter().all(|h| h.content["category"] == "tools"));
}

#[tokio::test]
async fn search_with_facets() {
    let index = make_index("products");
    let docs = vec![
        make_document(
            "doc-1",
            "products",
            serde_json::json!({"name": "A", "category": "tools"}),
        ),
        make_document(
            "doc-2",
            "products",
            serde_json::json!({"name": "B", "category": "electronics"}),
        ),
        make_document(
            "doc-3",
            "products",
            serde_json::json!({"name": "C", "category": "tools"}),
        ),
        make_document(
            "doc-4",
            "products",
            serde_json::json!({"name": "D", "category": "tools"}),
        ),
    ];
    let repo = Arc::new(StubSearchRepository::with_indices_and_docs(
        vec![index],
        docs,
    ));
    let uc = SearchUseCase::new(repo);

    let input = SearchInput {
        index_name: "products".to_string(),
        query: "".to_string(),
        from: 0,
        size: 10,
        filters: HashMap::new(),
        facets: vec!["category".to_string()],
    };
    let result = uc.execute(&input).await.unwrap();

    assert!(result.facets.contains_key("category"));
    let cat_facets = &result.facets["category"];
    assert_eq!(cat_facets["tools"], 3);
    assert_eq!(cat_facets["electronics"], 1);
}

#[tokio::test]
async fn search_case_insensitive() {
    let index = make_index("products");
    let docs = vec![
        make_document("doc-1", "products", serde_json::json!({"name": "WIDGET"})),
        make_document("doc-2", "products", serde_json::json!({"name": "widget"})),
        make_document("doc-3", "products", serde_json::json!({"name": "Widget"})),
    ];
    let repo = Arc::new(StubSearchRepository::with_indices_and_docs(
        vec![index],
        docs,
    ));
    let uc = SearchUseCase::new(repo);

    let input = SearchInput {
        index_name: "products".to_string(),
        query: "widget".to_string(),
        from: 0,
        size: 10,
        filters: HashMap::new(),
        facets: vec![],
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.total, 3);
}

#[tokio::test]
async fn search_only_in_specified_index() {
    let index1 = make_index("products");
    let index2 = make_index("users");
    let docs = vec![
        make_document("doc-1", "products", serde_json::json!({"name": "Widget"})),
        make_document("doc-2", "users", serde_json::json!({"name": "Widget User"})),
    ];
    let repo = Arc::new(StubSearchRepository::with_indices_and_docs(
        vec![index1, index2],
        docs,
    ));
    let uc = SearchUseCase::new(repo);

    let input = SearchInput {
        index_name: "products".to_string(),
        query: "Widget".to_string(),
        from: 0,
        size: 10,
        filters: HashMap::new(),
        facets: vec![],
    };
    let result = uc.execute(&input).await.unwrap();

    assert_eq!(result.total, 1);
    assert_eq!(result.hits[0].index_name, "products");
}

// ===========================================================================
// DeleteDocument tests
// ===========================================================================

#[tokio::test]
async fn delete_document_success() {
    let index = make_index("products");
    let doc = make_document("doc-1", "products", serde_json::json!({"name": "Widget"}));
    let repo = Arc::new(StubSearchRepository::with_indices_and_docs(
        vec![index],
        vec![doc],
    ));
    let uc = DeleteDocumentUseCase::new(repo.clone());

    let input = DeleteDocumentInput {
        index_name: "products".to_string(),
        doc_id: "doc-1".to_string(),
    };
    let result = uc.execute(&input).await.unwrap();
    assert!(result);

    // Verify removed
    let stored = repo.documents.read().await;
    assert!(stored.is_empty());
}

#[tokio::test]
async fn delete_document_not_found() {
    let repo = Arc::new(StubSearchRepository::new());
    let uc = DeleteDocumentUseCase::new(repo);

    let input = DeleteDocumentInput {
        index_name: "products".to_string(),
        doc_id: "nonexistent".to_string(),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        DeleteDocumentError::NotFound(index, id) => {
            assert_eq!(index, "products");
            assert_eq!(id, "nonexistent");
        }
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn delete_document_repo_error() {
    let repo = Arc::new(StubSearchRepository::with_error("disk full"));
    let uc = DeleteDocumentUseCase::new(repo);

    let input = DeleteDocumentInput {
        index_name: "products".to_string(),
        doc_id: "doc-1".to_string(),
    };
    let err = uc.execute(&input).await.unwrap_err();

    match err {
        DeleteDocumentError::Internal(msg) => assert!(msg.contains("disk full")),
        e => panic!("unexpected error: {:?}", e),
    }
}

#[tokio::test]
async fn delete_document_only_removes_target() {
    let index = make_index("products");
    let docs = vec![
        make_document("doc-1", "products", serde_json::json!({"name": "Widget A"})),
        make_document("doc-2", "products", serde_json::json!({"name": "Widget B"})),
    ];
    let repo = Arc::new(StubSearchRepository::with_indices_and_docs(
        vec![index],
        docs,
    ));
    let uc = DeleteDocumentUseCase::new(repo.clone());

    let input = DeleteDocumentInput {
        index_name: "products".to_string(),
        doc_id: "doc-1".to_string(),
    };
    uc.execute(&input).await.unwrap();

    let stored = repo.documents.read().await;
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].id, "doc-2");
}

// ===========================================================================
// ListIndices tests
// ===========================================================================

#[tokio::test]
async fn list_indices_empty() {
    let repo = Arc::new(StubSearchRepository::new());
    let uc = ListIndicesUseCase::new(repo);

    let result = uc.execute().await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn list_indices_with_results() {
    let indices = vec![
        make_index("products"),
        make_index("users"),
        make_index("orders"),
    ];
    let repo = Arc::new(StubSearchRepository::with_indices(indices));
    let uc = ListIndicesUseCase::new(repo);

    let result = uc.execute().await.unwrap();
    assert_eq!(result.len(), 3);

    let names: Vec<&str> = result.iter().map(|i| i.name.as_str()).collect();
    assert!(names.contains(&"products"));
    assert!(names.contains(&"users"));
    assert!(names.contains(&"orders"));
}

#[tokio::test]
async fn list_indices_repo_error() {
    let repo = Arc::new(StubSearchRepository::with_error("db error"));
    let uc = ListIndicesUseCase::new(repo);

    let err = uc.execute().await.unwrap_err();

    match err {
        ListIndicesError::Internal(msg) => assert!(msg.contains("db error")),
    }
}

// ===========================================================================
// End-to-end workflow: create index -> index docs -> search -> delete -> verify
// ===========================================================================

#[tokio::test]
async fn search_crud_workflow() {
    let repo = Arc::new(StubSearchRepository::new());
    let publisher = Arc::new(StubSearchEventPublisher::new());

    // 1. Create index
    let create_uc = CreateIndexUseCase::new(repo.clone());
    let create_input = CreateIndexInput {
        name: "products".to_string(),
        mapping: serde_json::json!({"fields": ["name", "category"]}),
    };
    let created_index = create_uc.execute(&create_input).await.unwrap();
    assert_eq!(created_index.name, "products");

    // 2. Index documents
    let index_uc = IndexDocumentUseCase::new(repo.clone(), publisher.clone());
    let docs_data = vec![
        (
            "doc-1",
            serde_json::json!({"name": "Red Widget", "category": "tools"}),
        ),
        (
            "doc-2",
            serde_json::json!({"name": "Blue Gadget", "category": "electronics"}),
        ),
        (
            "doc-3",
            serde_json::json!({"name": "Green Widget", "category": "tools"}),
        ),
    ];
    for (id, content) in &docs_data {
        let input = IndexDocumentInput {
            id: id.to_string(),
            index_name: "products".to_string(),
            content: content.clone(),
        };
        index_uc.execute(&input).await.unwrap();
    }

    // 3. Search with query
    let search_uc = SearchUseCase::new(repo.clone());
    let search_input = SearchInput {
        index_name: "products".to_string(),
        query: "Widget".to_string(),
        from: 0,
        size: 10,
        filters: HashMap::new(),
        facets: vec!["category".to_string()],
    };
    let search_result = search_uc.execute(&search_input).await.unwrap();
    assert_eq!(search_result.total, 2);
    assert!(search_result.facets.contains_key("category"));

    // 4. Search with filter
    let mut filters = HashMap::new();
    filters.insert("category".to_string(), "tools".to_string());
    let filter_input = SearchInput {
        index_name: "products".to_string(),
        query: "".to_string(),
        from: 0,
        size: 10,
        filters,
        facets: vec![],
    };
    let filter_result = search_uc.execute(&filter_input).await.unwrap();
    assert_eq!(filter_result.total, 2);

    // 5. Delete a document
    let delete_uc = DeleteDocumentUseCase::new(repo.clone());
    let delete_input = DeleteDocumentInput {
        index_name: "products".to_string(),
        doc_id: "doc-1".to_string(),
    };
    delete_uc.execute(&delete_input).await.unwrap();

    // 6. Verify deletion via search
    let search_after = search_uc.execute(&search_input).await.unwrap();
    assert_eq!(search_after.total, 1);

    // 7. List indices
    let list_uc = ListIndicesUseCase::new(repo.clone());
    let indices = list_uc.execute().await.unwrap();
    assert_eq!(indices.len(), 1);
    assert_eq!(indices[0].name, "products");

    // 8. Verify events published
    let events = publisher.events.read().await;
    assert_eq!(events.len(), 3); // 3 documents indexed
}

#[tokio::test]
async fn search_cross_index_isolation() {
    let repo = Arc::new(StubSearchRepository::new());
    let publisher = Arc::new(StubSearchEventPublisher::new());

    // Create two indices
    let create_uc = CreateIndexUseCase::new(repo.clone());
    create_uc
        .execute(&CreateIndexInput {
            name: "products".to_string(),
            mapping: serde_json::json!({}),
        })
        .await
        .unwrap();
    create_uc
        .execute(&CreateIndexInput {
            name: "users".to_string(),
            mapping: serde_json::json!({}),
        })
        .await
        .unwrap();

    // Index docs in different indices
    let index_uc = IndexDocumentUseCase::new(repo.clone(), publisher);
    index_uc
        .execute(&IndexDocumentInput {
            id: "p-1".to_string(),
            index_name: "products".to_string(),
            content: serde_json::json!({"name": "Widget"}),
        })
        .await
        .unwrap();
    index_uc
        .execute(&IndexDocumentInput {
            id: "u-1".to_string(),
            index_name: "users".to_string(),
            content: serde_json::json!({"name": "Widget Fan"}),
        })
        .await
        .unwrap();

    // Search in products only
    let search_uc = SearchUseCase::new(repo.clone());
    let result = search_uc
        .execute(&SearchInput {
            index_name: "products".to_string(),
            query: "Widget".to_string(),
            from: 0,
            size: 10,
            filters: HashMap::new(),
            facets: vec![],
        })
        .await
        .unwrap();

    assert_eq!(result.total, 1);
    assert_eq!(result.hits[0].id, "p-1");

    // Delete from products doesn't affect users
    let delete_uc = DeleteDocumentUseCase::new(repo.clone());
    delete_uc
        .execute(&DeleteDocumentInput {
            index_name: "products".to_string(),
            doc_id: "p-1".to_string(),
        })
        .await
        .unwrap();

    let user_result = search_uc
        .execute(&SearchInput {
            index_name: "users".to_string(),
            query: "Widget".to_string(),
            from: 0,
            size: 10,
            filters: HashMap::new(),
            facets: vec![],
        })
        .await
        .unwrap();
    assert_eq!(user_result.total, 1);
    assert_eq!(user_result.hits[0].id, "u-1");
}
