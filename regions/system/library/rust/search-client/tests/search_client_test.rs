use k1s0_search_client::{
    Filter, IndexDocument, IndexMapping, SearchClient, SearchError, SearchQuery,
};

// InMemorySearchClient is behind feature "test-utils"
use k1s0_search_client::InMemorySearchClient;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

async fn setup_index(client: &InMemorySearchClient, index: &str) {
    let mapping = IndexMapping::new()
        .field("name", "text")
        .field("price", "integer")
        .field("category", "keyword");
    client.create_index(index, mapping).await.unwrap();
}

fn make_doc(id: &str, name: &str, price: i64) -> IndexDocument {
    IndexDocument::new(id)
        .field("name", serde_json::json!(name))
        .field("price", serde_json::json!(price))
}

// ===========================================================================
// index_document
// ===========================================================================

// 1件のドキュメントをインデックスできることを確認する。
#[tokio::test]
async fn index_single_document() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "products").await;

    let result = client
        .index_document("products", make_doc("p1", "Rust Book", 3800))
        .await
        .unwrap();
    assert_eq!(result.id, "p1");
    assert_eq!(result.version, 1);
    assert_eq!(client.document_count("products").await, 1);
}

// 複数ドキュメントをインデックスするたびにバージョンが増加することを確認する。
#[tokio::test]
async fn index_multiple_documents_increments_version() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "products").await;

    let r1 = client
        .index_document("products", make_doc("p1", "A", 100))
        .await
        .unwrap();
    let r2 = client
        .index_document("products", make_doc("p2", "B", 200))
        .await
        .unwrap();
    assert_eq!(r1.version, 1);
    assert_eq!(r2.version, 2);
}

// ===========================================================================
// search
// ===========================================================================

// 空のクエリで検索するとすべてのドキュメントが返されることを確認する。
#[tokio::test]
async fn search_empty_query_returns_all() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "idx").await;
    client
        .index_document("idx", make_doc("1", "Alpha", 10))
        .await
        .unwrap();
    client
        .index_document("idx", make_doc("2", "Beta", 20))
        .await
        .unwrap();

    let result = client
        .search("idx", SearchQuery::new("").page(0).size(10))
        .await
        .unwrap();
    assert_eq!(result.total, 2);
}

// クエリテキストに一致するドキュメントのみが検索結果に返されることを確認する。
#[tokio::test]
async fn search_filters_by_query_text() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "idx").await;
    client
        .index_document("idx", make_doc("1", "Rust Programming", 3800))
        .await
        .unwrap();
    client
        .index_document("idx", make_doc("2", "Go Programming", 3200))
        .await
        .unwrap();

    let result = client
        .search("idx", SearchQuery::new("Rust").page(0).size(10))
        .await
        .unwrap();
    assert_eq!(result.total, 1);
}

// ファセット付きで検索するとファセット情報が結果に含まれることを確認する。
#[tokio::test]
async fn search_with_facets() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "idx").await;
    client
        .index_document("idx", make_doc("1", "Item", 100))
        .await
        .unwrap();

    let query = SearchQuery::new("").facet("category").page(0).size(10);
    let result = client.search("idx", query).await.unwrap();
    assert!(result.facets.contains_key("category"));
}

// 存在しないインデックスを検索すると IndexNotFound エラーが返されることを確認する。
#[tokio::test]
async fn search_nonexistent_index_returns_error() {
    let client = InMemorySearchClient::new();
    let result = client.search("missing", SearchQuery::new("test")).await;
    assert!(matches!(result, Err(SearchError::IndexNotFound(_))));
}

// ページネーションで指定したサイズ分だけ結果が返されることを確認する。
#[tokio::test]
async fn search_pagination() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "idx").await;
    for i in 0..5 {
        client
            .index_document("idx", make_doc(&format!("d{i}"), &format!("Doc {i}"), i))
            .await
            .unwrap();
    }

    let page0 = client
        .search("idx", SearchQuery::new("").page(0).size(2))
        .await
        .unwrap();
    assert_eq!(page0.hits.len(), 2);
}

// ===========================================================================
// delete_document
// ===========================================================================

// ドキュメント削除後にインデックスから除去されることを確認する。
#[tokio::test]
async fn delete_removes_document() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "idx").await;
    client
        .index_document("idx", make_doc("1", "Test", 100))
        .await
        .unwrap();
    assert_eq!(client.document_count("idx").await, 1);

    client.delete_document("idx", "1").await.unwrap();
    assert_eq!(client.document_count("idx").await, 0);
}

// 存在しないドキュメントの削除が無操作で正常終了することを確認する。
#[tokio::test]
async fn delete_nonexistent_doc_is_noop() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "idx").await;
    // Should not error even if doc doesn't exist
    client.delete_document("idx", "ghost").await.unwrap();
}

// 存在しないインデックスへの削除が無操作で正常終了することを確認する。
#[tokio::test]
async fn delete_from_nonexistent_index_is_noop() {
    let client = InMemorySearchClient::new();
    client.delete_document("no-index", "ghost").await.unwrap();
}

// ===========================================================================
// bulk_index
// ===========================================================================

// 一括インデックスですべてのドキュメントが正常に登録されることを確認する。
#[tokio::test]
async fn bulk_index_inserts_all() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "idx").await;

    let docs = vec![
        make_doc("1", "A", 10),
        make_doc("2", "B", 20),
        make_doc("3", "C", 30),
    ];
    let result = client.bulk_index("idx", docs).await.unwrap();
    assert_eq!(result.success_count, 3);
    assert_eq!(result.failed_count, 0);
    assert!(result.failures.is_empty());
    assert_eq!(client.document_count("idx").await, 3);
}

// 空のドキュメントリストで一括インデックスを実行しても正常終了することを確認する。
#[tokio::test]
async fn bulk_index_empty_vec() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "idx").await;

    let result = client.bulk_index("idx", vec![]).await.unwrap();
    assert_eq!(result.success_count, 0);
}

// ===========================================================================
// create_index
// ===========================================================================

// インデックス作成後に空の検索結果が返されることを確認する。
#[tokio::test]
async fn create_index_makes_searchable() {
    let client = InMemorySearchClient::new();
    setup_index(&client, "new-idx").await;

    let result = client
        .search("new-idx", SearchQuery::new("").page(0).size(10))
        .await
        .unwrap();
    assert_eq!(result.total, 0);
}

// ===========================================================================
// query builder
// ===========================================================================

// SearchQuery のデフォルト値が正しく設定されることを確認する。
#[test]
fn search_query_builder_defaults() {
    let q = SearchQuery::new("test");
    assert_eq!(q.query, "test");
    assert!(q.filters.is_empty());
    assert!(q.facets.is_empty());
    assert_eq!(q.page, 0);
    assert_eq!(q.size, 20);
}

// フィルター・ファセット・ページネーションを指定して SearchQuery が正しく構築されることを確認する。
#[test]
fn search_query_builder_with_filters() {
    let q = SearchQuery::new("shoes")
        .filter(Filter::eq("brand", "Nike"))
        .filter(Filter::range("price", 50, 200))
        .filter(Filter::gt("rating", 4))
        .filter(Filter::lt("weight", 500))
        .facet("brand")
        .facet("color")
        .page(2)
        .size(25);

    assert_eq!(q.filters.len(), 4);
    assert_eq!(q.facets, vec!["brand", "color"]);
    assert_eq!(q.page, 2);
    assert_eq!(q.size, 25);
}

// ===========================================================================
// IndexDocument builder
// ===========================================================================

// IndexDocument ビルダーでIDとフィールドが正しく設定されることを確認する。
#[test]
fn index_document_builder() {
    let doc = IndexDocument::new("id-1")
        .field("title", serde_json::json!("Hello"))
        .field("count", serde_json::json!(42));
    assert_eq!(doc.id, "id-1");
    assert_eq!(doc.fields.len(), 2);
    assert_eq!(doc.fields["title"], serde_json::json!("Hello"));
}

// ===========================================================================
// IndexMapping builder
// ===========================================================================

// IndexMapping ビルダーでフィールドが正しく登録されることを確認する。
#[test]
fn index_mapping_builder() {
    let m = IndexMapping::new()
        .field("title", "text")
        .field("count", "integer");
    assert_eq!(m.fields.len(), 2);
    assert_eq!(m.fields["title"].field_type, "text");
    assert!(m.fields["title"].indexed);
}

// IndexMapping のデフォルト値でフィールドが空であることを確認する。
#[test]
fn index_mapping_default() {
    let m = IndexMapping::default();
    assert!(m.fields.is_empty());
}

// ===========================================================================
// Filter constructors
// ===========================================================================

// Filter::eq で等値フィルターが正しく構築されることを確認する。
#[test]
fn filter_eq() {
    let f = Filter::eq("status", "active");
    assert_eq!(f.field, "status");
    assert_eq!(f.operator, "eq");
    assert_eq!(f.value, serde_json::json!("active"));
    assert!(f.value_to.is_none());
}

// Filter::lt で未満フィルターが正しく構築されることを確認する。
#[test]
fn filter_lt() {
    let f = Filter::lt("age", 30);
    assert_eq!(f.operator, "lt");
}

// Filter::gt で超過フィルターが正しく構築されることを確認する。
#[test]
fn filter_gt() {
    let f = Filter::gt("score", 80);
    assert_eq!(f.operator, "gt");
}

// Filter::range で範囲フィルターが正しく構築されることを確認する。
#[test]
fn filter_range() {
    let f = Filter::range("price", 10, 100);
    assert_eq!(f.operator, "range");
    assert_eq!(f.value, serde_json::json!(10));
    assert_eq!(f.value_to, Some(serde_json::json!(100)));
}

// ===========================================================================
// error variant coverage
// ===========================================================================

// IndexNotFound エラーの Display 出力にインデックス名が含まれることを確認する。
#[test]
fn error_display_index_not_found() {
    let e = SearchError::IndexNotFound("test".to_string());
    assert!(format!("{e}").contains("test"));
}

// InvalidQuery エラーの Display 出力に詳細が含まれることを確認する。
#[test]
fn error_display_invalid_query() {
    let e = SearchError::InvalidQuery("bad".to_string());
    assert!(format!("{e}").contains("bad"));
}

// ServerError エラーの Display 出力に詳細が含まれることを確認する。
#[test]
fn error_display_server_error() {
    let e = SearchError::ServerError("oops".to_string());
    assert!(format!("{e}").contains("oops"));
}

// Timeout エラーの Display 出力が空でないことを確認する。
#[test]
fn error_display_timeout() {
    let e = SearchError::Timeout;
    assert!(!format!("{e}").is_empty());
}

// ===========================================================================
// BulkFailure
// ===========================================================================

// BulkFailure のフィールドが正しく設定されることを確認する。
#[test]
fn bulk_failure_fields() {
    use k1s0_search_client::BulkFailure;
    let f = BulkFailure {
        id: "x".to_string(),
        error: "err".to_string(),
    };
    assert_eq!(f.id, "x");
    assert_eq!(f.error, "err");
}
