# k1s0-search-client ライブラリ設計

## 概要

system-search-server（ポート 8094）へのドキュメント検索クライアントライブラリ。ドキュメントのインデックス登録・更新・削除・全文検索・ファセット検索・フィルタ検索・インデックス管理（作成・削除・マッピング取得）・バルクインデックス操作を統一インターフェースで提供する。全 Tier のサービスから共通利用し、検索機能を持つあらゆるドメインサービスの基盤となる。

> **ポート注記**: ポート `8094` は Docker Compose 環境でのホスト側ポートである。本番環境では Kubernetes Service 経由（`search-server:8080`）で接続する。

**配置先**: `regions/system/library/rust/search-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `SearchClient` | トレイト | 検索・インデックス操作インターフェース |
| `GrpcSearchClient` | 構造体 | gRPC 経由の search-server 接続実装 |
| `SearchQuery` | 構造体 | クエリ文字列・フィルター・ファセット・ページネーション |
| `SearchResult<T>` | 構造体 | ヒット件数・ヒット一覧・ファセット集計・処理時間 |
| `IndexDocument` | 構造体 | ドキュメント ID・フィールドマップ |
| `IndexResult` | 構造体 | インデックス済みドキュメント ID・バージョン |
| `BulkResult` | 構造体 | 成功件数・失敗件数・失敗詳細リスト |
| `IndexMapping` | 構造体 | フィールド定義マップ（フィールド名・型・インデックス設定）|
| `Filter` | 構造体 | フィールド・演算子・値 |
| `FacetBucket` | 構造体 | バケット値・ドキュメント件数 |
| `SearchError` | enum | `IndexNotFound`・`InvalidQuery`・`ServerError`・`Timeout` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-search-client"
version = "0.1.0"
edition = "2021"

[features]
grpc = ["tonic"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
tonic = { version = "0.12", optional = true }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
```

**依存追加**: `k1s0-search-client = { path = "../../system/library/rust/search-client" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
search-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）・使用例ドキュメント
│   ├── client.rs       # SearchClient トレイト
│   ├── grpc.rs         # GrpcSearchClient
│   ├── query.rs        # SearchQuery・Filter・FacetBucket
│   ├── document.rs     # IndexDocument・IndexResult・BulkResult・IndexMapping
│   └── error.rs        # SearchError
└── Cargo.toml
```

**主要 API**:

```rust
// --- client.rs ---

#[async_trait]
pub trait SearchClient: Send + Sync {
    async fn index_document(&self, index: &str, doc: IndexDocument) -> Result<IndexResult, SearchError>;
    async fn bulk_index(&self, index: &str, docs: Vec<IndexDocument>) -> Result<BulkResult, SearchError>;
    async fn search(&self, index: &str, query: SearchQuery) -> Result<SearchResult<serde_json::Value>, SearchError>;
    async fn delete_document(&self, index: &str, id: &str) -> Result<(), SearchError>;
    async fn create_index(&self, name: &str, mapping: IndexMapping) -> Result<(), SearchError>;
}

// GrpcSearchClient（gRPC実装、feature = "grpc" 有効時）
// pub struct GrpcSearchClient { /* ... */ }
// impl GrpcSearchClient {
//     pub async fn new(addr: &str) -> Result<GrpcSearchClient, SearchError>
// }

// --- document.rs ---

pub struct IndexDocument {
    pub id: String,
    pub fields: HashMap<String, serde_json::Value>,
}
impl IndexDocument {
    pub fn new(id: impl Into<String>) -> Self;
    pub fn field(mut self, name: impl Into<String>, value: serde_json::Value) -> Self;
}

pub struct IndexResult {
    pub id: String,
    pub version: i64,
}

pub struct BulkFailure {
    pub id: String,
    pub error: String,
}

pub struct BulkResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub failures: Vec<BulkFailure>,
}

pub struct FieldMapping {
    pub field_type: String,
    pub indexed: bool,
}

pub struct IndexMapping {
    pub fields: HashMap<String, FieldMapping>,
}
impl IndexMapping {
    pub fn new() -> Self;
    pub fn field(mut self, name: impl Into<String>, field_type: impl Into<String>) -> Self;
}

// --- query.rs ---

pub struct Filter {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
    pub value_to: Option<serde_json::Value>,
}
impl Filter {
    pub fn eq(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self;
    pub fn lt(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self;
    pub fn gt(field: impl Into<String>, value: impl Into<serde_json::Value>) -> Self;
    pub fn range(field: impl Into<String>, from: impl Into<serde_json::Value>, to: impl Into<serde_json::Value>) -> Self;
}

pub struct FacetBucket {
    pub value: String,
    pub count: u64,
}

pub struct SearchQuery {
    pub query: String,
    pub filters: Vec<Filter>,
    pub facets: Vec<String>,
    pub page: u32,
    pub size: u32,
}
impl SearchQuery {
    pub fn new(query: impl Into<String>) -> Self;
    pub fn filter(mut self, filter: Filter) -> Self;
    pub fn facet(mut self, facet: impl Into<String>) -> Self;
    pub fn page(mut self, page: u32) -> Self;
    pub fn size(mut self, size: u32) -> Self;
}

pub struct SearchResult<T> {
    pub hits: Vec<T>,
    pub total: u64,
    pub facets: HashMap<String, Vec<FacetBucket>>,
    pub took_ms: u64,
}

// --- error.rs ---

#[derive(Debug, thiserror::Error)]
pub enum SearchError {
    #[error("インデックスが見つかりません: {0}")]
    IndexNotFound(String),
    #[error("無効なクエリ: {0}")]
    InvalidQuery(String),
    #[error("サーバーエラー: {0}")]
    ServerError(String),
    #[error("タイムアウト")]
    Timeout,
}
```

**使用例**:

```rust
use k1s0_search_client::{
    Filter, IndexDocument, IndexMapping, MockSearchClient, SearchClient, SearchQuery,
};
use serde_json::json;

// MockSearchClient でテスト（feature = "mock"）
// または独自実装の SearchClient トレイト実装を使用

// インデックスの作成
let mapping = IndexMapping::new()
    .field("name", "text")
    .field("price", "integer")
    .field("category", "keyword");

client.create_index("products", mapping).await?;

// ドキュメントのインデックス登録
let doc = IndexDocument::new("prod-001")
    .field("name", json!("Rust プログラミング入門"))
    .field("price", json!(3800))
    .field("category", json!("books"));

let result = client.index_document("products", doc).await?;
tracing::info!(id = %result.id, version = result.version, "インデックス登録完了");

// バルクインデックス（高スループット）
let docs = vec![
    IndexDocument::new("prod-002").field("name", json!("Go 言語仕様")).field("price", json!(4200)).field("category", json!("books")),
    IndexDocument::new("prod-003").field("name", json!("TypeScript 実践")).field("price", json!(3500)).field("category", json!("books")),
];
let bulk_result = client.bulk_index("products", docs).await?;
tracing::info!(
    success = bulk_result.success_count,
    failed = bulk_result.failed_count,
    "バルクインデックス完了"
);

// 全文検索（ファセット・フィルター付き）
let query = SearchQuery::new("Rust プログラミング")
    .filter(Filter::eq("category", "books"))
    .filter(Filter::range("price", json!(1000), json!(5000)))
    .facet("category")
    .page(0)
    .size(20);

// search は SearchResult<serde_json::Value> を返す
let search_result = client.search("products", query).await?;
tracing::info!(
    total = search_result.total,
    took_ms = search_result.took_ms,
    "検索完了"
);
for hit in &search_result.hits {
    println!("{}", hit["id"]);
}

// ドキュメントの削除
client.delete_document("products", "prod-001").await?;
```

## Go 実装

**配置先**: `regions/system/library/go/search-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**依存関係**: `google.golang.org/grpc v1.70`, `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type SearchClient interface {
    IndexDocument(ctx context.Context, index string, doc IndexDocument) (IndexResult, error)
    BulkIndex(ctx context.Context, index string, docs []IndexDocument) (BulkResult, error)
    Search(ctx context.Context, index string, query SearchQuery) (SearchResult, error)
    DeleteDocument(ctx context.Context, index, id string) error
    CreateIndex(ctx context.Context, name string, mapping IndexMapping) error
}

type SearchQuery struct {
    Query   string
    Filters []Filter
    Facets  []string
    Page    uint32
    Size    uint32
}

type Filter struct {
    Field    string
    Operator string // "eq", "lt", "gt", "range"
    Value    interface{}
}

type SearchResult struct {
    Hits    []map[string]interface{}
    Total   uint64
    Facets  map[string][]FacetBucket
    TookMs  uint64
}

type FacetBucket struct {
    Value string
    Count uint64
}

type IndexDocument struct {
    ID     string
    Fields map[string]interface{}
}

type IndexResult struct {
    ID      string
    Version int64
}

type BulkFailure struct {
    ID    string
    Error string
}

type BulkResult struct {
    SuccessCount int
    FailedCount  int
    Failures     []BulkFailure
}

type FieldMapping struct {
    FieldType string
    Indexed   bool
}

type IndexMapping struct {
    Fields map[string]FieldMapping
}

// IndexMappingビルダー
func NewIndexMapping() IndexMapping
func (m IndexMapping) WithField(name, fieldType string) IndexMapping

// InMemorySearchClient（テスト用インメモリ実装）
type InMemorySearchClient struct{ /* ... */ }

func NewInMemorySearchClient() *InMemorySearchClient
func (c *InMemorySearchClient) CreateIndex(ctx context.Context, name string, mapping IndexMapping) error
func (c *InMemorySearchClient) IndexDocument(ctx context.Context, index string, doc IndexDocument) (IndexResult, error)
func (c *InMemorySearchClient) BulkIndex(ctx context.Context, index string, docs []IndexDocument) (BulkResult, error)
func (c *InMemorySearchClient) Search(ctx context.Context, index string, query SearchQuery) (SearchResult, error)
func (c *InMemorySearchClient) DeleteDocument(ctx context.Context, index, id string) error
func (c *InMemorySearchClient) DocumentCount(index string) int

// GrpcSearchClient（gRPC実装、将来実装予定）
type GrpcSearchClient struct{ /* ... */ }

func NewGrpcSearchClient(addr string) (*GrpcSearchClient, error)
func (c *GrpcSearchClient) IndexDocument(ctx context.Context, index string, doc IndexDocument) (IndexResult, error)
func (c *GrpcSearchClient) BulkIndex(ctx context.Context, index string, docs []IndexDocument) (BulkResult, error)
func (c *GrpcSearchClient) Search(ctx context.Context, index string, query SearchQuery) (SearchResult, error)
func (c *GrpcSearchClient) DeleteDocument(ctx context.Context, index, id string) error
func (c *GrpcSearchClient) CreateIndex(ctx context.Context, name string, mapping IndexMapping) error
func (c *GrpcSearchClient) Close()
```

**使用例**:

```go
// InMemorySearchClient（テスト・開発環境）
client := NewInMemorySearchClient()

mapping := NewIndexMapping().
    WithField("name", "text").
    WithField("price", "integer").
    WithField("category", "keyword")

if err := client.CreateIndex(ctx, "products", mapping); err != nil {
    log.Fatal(err)
}

result, err := client.Search(ctx, "products", SearchQuery{
    Query:   "Rust プログラミング",
    Filters: []Filter{{Field: "category", Operator: "eq", Value: "books"}},
    Facets:  []string{"category"},
    Page:    0,
    Size:    20,
})
if err != nil {
    return err
}
fmt.Printf("総件数: %d, 処理時間: %dms\n", result.Total, result.TookMs)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/search-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface Filter {
  field: string;
  operator: 'eq' | 'lt' | 'gt' | 'range' | 'in';
  value: unknown;
  valueTo?: unknown;
}

export interface FacetBucket {
  value: string;
  count: number;
}

export interface SearchQuery {
  query: string;
  filters?: Filter[];
  facets?: string[];
  page?: number;
  size?: number;
}

export interface SearchResult<T = Record<string, unknown>> {
  hits: T[];
  total: number;
  facets: Record<string, FacetBucket[]>;
  tookMs: number;
}

export interface IndexDocument {
  id: string;
  fields: Record<string, unknown>;
}

export interface IndexResult {
  id: string;
  version: number;
}

export interface BulkFailure {
  id: string;
  error: string;
}

export interface BulkResult {
  successCount: number;
  failedCount: number;
  failures: BulkFailure[];
}

export interface FieldMapping {
  type: string;
  indexed?: boolean;
}

export interface IndexMapping {
  fields: Record<string, FieldMapping>;
}

export interface SearchClient {
  indexDocument(index: string, doc: IndexDocument): Promise<IndexResult>;
  bulkIndex(index: string, docs: IndexDocument[]): Promise<BulkResult>;
  search<T = Record<string, unknown>>(index: string, query: SearchQuery): Promise<SearchResult<T>>;
  deleteDocument(index: string, id: string): Promise<void>;
  createIndex(name: string, mapping: IndexMapping): Promise<void>;
}

// InMemorySearchClient（テスト・開発環境用インメモリ実装）
export class InMemorySearchClient implements SearchClient {
  createIndex(name: string, mapping: IndexMapping): Promise<void>;
  indexDocument(index: string, doc: IndexDocument): Promise<IndexResult>;
  bulkIndex(index: string, docs: IndexDocument[]): Promise<BulkResult>;
  search<T = Record<string, unknown>>(index: string, query: SearchQuery): Promise<SearchResult<T>>;
  deleteDocument(index: string, id: string): Promise<void>;
  documentCount(index: string): number;
}

// GrpcSearchClient（gRPC実装、将来実装予定）
export class GrpcSearchClient implements SearchClient {
  constructor(serverUrl: string);
  indexDocument(index: string, doc: IndexDocument): Promise<IndexResult>;
  bulkIndex(index: string, docs: IndexDocument[]): Promise<BulkResult>;
  search<T = Record<string, unknown>>(index: string, query: SearchQuery): Promise<SearchResult<T>>;
  deleteDocument(index: string, id: string): Promise<void>;
  createIndex(name: string, mapping: IndexMapping): Promise<void>;
  close(): Promise<void>;
}

export class SearchError extends Error {
  constructor(
    message: string,
    public readonly code: 'INDEX_NOT_FOUND' | 'INVALID_QUERY' | 'SERVER_ERROR' | 'TIMEOUT'
  );
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/search_client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**pubspec.yaml 主要依存**:

```yaml
dependencies: {}
```

**モジュール構成**:

```
search_client/
├── lib/
│   ├── search_client.dart      # 公開 API（再エクスポート）
│   └── src/
│       ├── client.dart         # SearchClient 抽象クラス・InMemorySearchClient
│       ├── document.dart       # IndexDocument・IndexResult・BulkResult・BulkFailure・FieldMapping・IndexMapping
│       ├── query.dart          # SearchQuery・Filter・FacetBucket・SearchResult
│       └── error.dart          # SearchErrorCode・SearchError
└── pubspec.yaml
```

**主要 API**:

```dart
// --- document.dart ---

class IndexDocument {
  final String id;
  final Map<String, dynamic> fields;
  const IndexDocument({required String id, required Map<String, dynamic> fields});
}

class IndexResult {
  final String id;
  final int version;
  const IndexResult({required String id, required int version});
}

class BulkFailure {
  final String id;
  final String error;
  const BulkFailure({required String id, required String error});
}

class BulkResult {
  final int successCount;
  final int failedCount;
  final List<BulkFailure> failures;
  const BulkResult({
    required int successCount,
    required int failedCount,
    required List<BulkFailure> failures,
  });
}

class FieldMapping {
  final String fieldType;
  final bool indexed;
  const FieldMapping({required String fieldType, bool indexed = true});
}

class IndexMapping {
  final Map<String, FieldMapping> fields;
  IndexMapping({Map<String, FieldMapping>? fields});
  IndexMapping withField(String name, String fieldType);
}

// --- query.dart ---

class Filter {
  final String field;
  final String operator;
  final dynamic value;
  final dynamic valueTo;
  const Filter({required String field, required String operator, required dynamic value, dynamic valueTo});

  factory Filter.eq(String field, dynamic value);
  factory Filter.lt(String field, dynamic value);
  factory Filter.gt(String field, dynamic value);
  factory Filter.range(String field, dynamic from, dynamic to);
}

class FacetBucket {
  final String value;
  final int count;
  const FacetBucket({required String value, required int count});
}

class SearchQuery {
  final String query;
  final List<Filter> filters;
  final List<String> facets;
  final int page;
  final int size;
  const SearchQuery({
    required String query,
    List<Filter> filters = const [],
    List<String> facets = const [],
    int page = 0,
    int size = 20,
  });
}

class SearchResult<T> {
  final List<T> hits;
  final int total;
  final Map<String, List<FacetBucket>> facets;
  final int tookMs;
  const SearchResult({
    required List<T> hits,
    required int total,
    required Map<String, List<FacetBucket>> facets,
    required int tookMs,
  });
}

// --- error.dart ---

enum SearchErrorCode {
  indexNotFound,
  invalidQuery,
  serverError,
  timeout,
}

class SearchError implements Exception {
  final String message;
  final SearchErrorCode code;
  const SearchError(String message, SearchErrorCode code);
  String toString();
}

// --- client.dart ---

abstract class SearchClient {
  Future<IndexResult> indexDocument(String index, IndexDocument doc);
  Future<BulkResult> bulkIndex(String index, List<IndexDocument> docs);
  Future<SearchResult<Map<String, dynamic>>> search(String index, SearchQuery query);
  Future<void> deleteDocument(String index, String id);
  Future<void> createIndex(String name, IndexMapping mapping);
}

// InMemorySearchClient（テスト・開発環境用インメモリ実装）
class InMemorySearchClient implements SearchClient {
  Future<void> createIndex(String name, IndexMapping mapping);
  Future<IndexResult> indexDocument(String index, IndexDocument doc);
  Future<BulkResult> bulkIndex(String index, List<IndexDocument> docs);
  Future<SearchResult<Map<String, dynamic>>> search(String index, SearchQuery query);
  Future<void> deleteDocument(String index, String id);
  int documentCount(String index);
}

// GrpcSearchClient（gRPC実装）
class GrpcSearchClient implements SearchClient {
  GrpcSearchClient(String serverUrl);
  Future<IndexResult> indexDocument(String index, IndexDocument doc);
  Future<BulkResult> bulkIndex(String index, List<IndexDocument> docs);
  Future<SearchResult<Map<String, dynamic>>> search(String index, SearchQuery query);
  Future<void> deleteDocument(String index, String id);
  Future<void> createIndex(String name, IndexMapping mapping);
  Future<void> close();
}
```

**使用例**:

```dart
import 'package:k1s0_search_client/search_client.dart';

// InMemorySearchClient（テスト・開発環境）
final client = InMemorySearchClient();

// インデックスの作成
final mapping = IndexMapping()
    .withField('name', 'text')
    .withField('price', 'integer')
    .withField('category', 'keyword');
await client.createIndex('products', mapping);

// ドキュメントのインデックス登録
final doc = IndexDocument(
  id: 'prod-001',
  fields: {'name': 'Rust プログラミング入門', 'price': 3800, 'category': 'books'},
);
final result = await client.indexDocument('products', doc);
print('インデックス済み: ${result.id} v${result.version}');

// バルクインデックス
final docs = [
  IndexDocument(id: 'prod-002', fields: {'name': 'Go 言語仕様', 'price': 4200, 'category': 'books'}),
  IndexDocument(id: 'prod-003', fields: {'name': 'TypeScript 実践', 'price': 3500, 'category': 'books'}),
];
final bulkResult = await client.bulkIndex('products', docs);
print('成功: ${bulkResult.successCount}, 失敗: ${bulkResult.failedCount}');

// 全文検索（フィルター・ファセット付き）
final query = SearchQuery(
  query: 'Rust プログラミング',
  filters: [
    Filter.eq('category', 'books'),
    Filter.range('price', 1000, 5000),
  ],
  facets: ['category'],
  page: 0,
  size: 20,
);
final searchResult = await client.search('products', query);
print('総件数: ${searchResult.total}, 処理時間: ${searchResult.tookMs}ms');

// ドキュメントの削除
await client.deleteDocument('products', 'prod-001');
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト（`#[cfg(test)]`）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("Rust")
            .filter(Filter::eq("category", "books"))
            .facet("category")
            .page(0)
            .size(10);

        assert_eq!(query.query, "Rust");
        assert_eq!(query.filters.len(), 1);
        assert_eq!(query.facets, vec!["category"]);
        assert_eq!(query.size, 10);
    }

    #[test]
    fn test_index_document_builder() {
        let doc = IndexDocument::new("prod-001")
            .field("name", json!("test"))
            .field("price", json!(100));

        assert_eq!(doc.id, "prod-001");
        assert_eq!(doc.fields.len(), 2);
    }

    #[test]
    fn test_search_error_index_not_found() {
        let err = SearchError::IndexNotFound("missing-index".to_string());
        assert!(matches!(err, SearchError::IndexNotFound(_)));
    }
}
```

### 統合テスト

- `testcontainers` で search-server コンテナを起動して実際の index/search フローを検証
- 全文検索・フィルター検索・ファセット集計の各パターンをカバー
- バルクインデックスで部分失敗（一部ドキュメントが不正）のシナリオを確認
- 存在しないインデックスで `IndexNotFound` エラーが返ることを確認

### モックテスト

```rust
use mockall::mock;

mock! {
    pub TestSearchClient {}
    #[async_trait]
    impl SearchClient for TestSearchClient {
        async fn index_document(&self, index: &str, doc: IndexDocument) -> Result<IndexResult, SearchError>;
        async fn bulk_index(&self, index: &str, docs: Vec<IndexDocument>) -> Result<BulkResult, SearchError>;
        async fn search(&self, index: &str, query: SearchQuery) -> Result<SearchResult<serde_json::Value>, SearchError>;
        async fn delete_document(&self, index: &str, id: &str) -> Result<(), SearchError>;
        async fn create_index(&self, name: &str, mapping: IndexMapping) -> Result<(), SearchError>;
    }
}

#[tokio::test]
async fn test_product_service_indexes_on_create() {
    let mut mock = MockTestSearchClient::new();
    mock.expect_index_document()
        .withf(|idx, _| idx == "products")
        .once()
        .returning(|_, _| Ok(IndexResult { id: "prod-001".to_string(), version: 1 }));

    let service = ProductService::new(Arc::new(mock));
    service.create_product(new_product_request()).await.unwrap();
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-search-server設計](../../servers/search/server.md) — 検索サーバー設計
- [system-library-pagination設計](../data/pagination.md) — ページネーションライブラリ（SearchResult のページング）
- [system-library-eventstore設計](../data/eventstore.md) — イベント永続化（ドメインイベントから検索インデックス更新）
- [system-library-kafka設計](../messaging/kafka.md) — Kafka コンシューマー（インデックス更新イベント）
