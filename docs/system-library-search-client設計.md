# k1s0-search-client ライブラリ設計

## 概要

system-search-server（ポート 8094）へのドキュメント検索クライアントライブラリ。ドキュメントのインデックス登録・更新・削除・全文検索・ファセット検索・フィルタ検索・インデックス管理（作成・削除・マッピング取得）・バルクインデックス操作を統一インターフェースで提供する。全 Tier のサービスから共通利用し、検索機能を持つあらゆるドメインサービスの基盤となる。

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

**Cargo.toml への追加行**:

```toml
k1s0-search-client = { path = "../../system/library/rust/search-client" }
# gRPC 経由を有効化する場合:
k1s0-search-client = { path = "../../system/library/rust/search-client", features = ["grpc"] }
```

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

**使用例**:

```rust
use k1s0_search_client::{
    Filter, GrpcSearchClient, IndexDocument, IndexMapping, SearchClient, SearchQuery,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize)]
struct Product {
    id: String,
    name: String,
    price: u64,
    category: String,
}

// クライアントの構築
let client = GrpcSearchClient::new("http://search-server:8080").await?;

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
    .filter(Filter::range("price", 1000, 5000))
    .facet("category")
    .page(0)
    .size(20);

let search_result = client.search::<Product>("products", query).await?;
tracing::info!(
    total = search_result.total,
    took_ms = search_result.took_ms,
    "検索完了"
);
for hit in &search_result.hits {
    println!("{}: {}", hit.id, hit.name);
}

// ドキュメントの削除
client.delete_document("products", "prod-001").await?;
```

## Go 実装

**配置先**: `regions/system/library/go/search-client/`

```
search-client/
├── search_client.go
├── grpc_client.go
├── query.go
├── document.go
├── search_client_test.go
├── go.mod
└── go.sum
```

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

type BulkResult struct {
    SuccessCount int
    FailedCount  int
    Failures     []BulkFailure
}

type GrpcSearchClient struct{ /* ... */ }

func NewGrpcSearchClient(addr string) (*GrpcSearchClient, error)
func (c *GrpcSearchClient) IndexDocument(ctx context.Context, index string, doc IndexDocument) (IndexResult, error)
func (c *GrpcSearchClient) BulkIndex(ctx context.Context, index string, docs []IndexDocument) (BulkResult, error)
func (c *GrpcSearchClient) Search(ctx context.Context, index string, query SearchQuery) (SearchResult, error)
func (c *GrpcSearchClient) DeleteDocument(ctx context.Context, index, id string) error
func (c *GrpcSearchClient) CreateIndex(ctx context.Context, name string, mapping IndexMapping) error
```

**使用例**:

```go
client, err := NewGrpcSearchClient("search-server:8080")
if err != nil {
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

**配置先**: `regions/system/library/typescript/search-client/`

```
search-client/
├── package.json        # "@k1s0/search-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # SearchClient, GrpcSearchClient, SearchQuery, SearchResult, IndexDocument, IndexResult, BulkResult, IndexMapping, Filter, FacetBucket, SearchError
└── __tests__/
    └── search-client.test.ts
```

**主要 API**:

```typescript
export interface Filter {
  field: string;
  operator: 'eq' | 'lt' | 'gt' | 'range' | 'in';
  value: unknown;
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

export interface BulkResult {
  successCount: number;
  failedCount: number;
  failures: Array<{ id: string; error: string }>;
}

export interface IndexMapping {
  fields: Record<string, { type: string; indexed?: boolean }>;
}

export interface SearchClient {
  indexDocument(index: string, doc: IndexDocument): Promise<IndexResult>;
  bulkIndex(index: string, docs: IndexDocument[]): Promise<BulkResult>;
  search<T = Record<string, unknown>>(index: string, query: SearchQuery): Promise<SearchResult<T>>;
  deleteDocument(index: string, id: string): Promise<void>;
  createIndex(name: string, mapping: IndexMapping): Promise<void>;
}

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

**配置先**: `regions/system/library/dart/search_client/`

```
search_client/
├── pubspec.yaml        # k1s0_search_client
├── analysis_options.yaml
├── lib/
│   ├── search_client.dart
│   └── src/
│       ├── client.dart         # SearchClient abstract, GrpcSearchClient
│       ├── query.dart          # SearchQuery, Filter, FacetBucket
│       ├── document.dart       # IndexDocument, IndexResult, BulkResult, IndexMapping
│       └── error.dart          # SearchError
└── test/
    └── search_client_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  grpc: ^4.0.0
  protobuf: ^3.1.0
```

**使用例**:

```dart
import 'package:k1s0_search_client/search_client.dart';

final client = GrpcSearchClient('search-server:8080');

// ドキュメントのインデックス登録
final doc = IndexDocument(
  id: 'prod-001',
  fields: {'name': 'Rust プログラミング入門', 'price': 3800, 'category': 'books'},
);
final result = await client.indexDocument('products', doc);
print('インデックス済み: ${result.id} v${result.version}');

// 検索
final query = SearchQuery(
  query: 'Rust プログラミング',
  filters: [Filter(field: 'category', operator: 'eq', value: 'books')],
  facets: ['category'],
  page: 0,
  size: 20,
);
final searchResult = await client.search('products', query);
print('総件数: ${searchResult.total}, 処理時間: ${searchResult.tookMs}ms');
```

**カバレッジ目標**: 90%以上

## Python 実装

**配置先**: `regions/system/library/python/search_client/`

### パッケージ構造

```
search_client/
├── pyproject.toml
├── src/
│   └── k1s0_search_client/
│       ├── __init__.py           # 公開 API（再エクスポート）
│       ├── client.py             # SearchClient ABC・GrpcSearchClient
│       ├── query.py              # SearchQuery dataclass・Filter・FacetBucket
│       ├── document.py           # IndexDocument・IndexResult・BulkResult・IndexMapping dataclass
│       ├── exceptions.py         # SearchError
│       └── py.typed
└── tests/
    ├── test_search_client.py
    └── test_bulk_index.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `SearchClient` | ABC | 検索・インデックス操作抽象基底クラス |
| `GrpcSearchClient` | class | gRPC 経由の search-server 接続実装 |
| `SearchQuery` | dataclass | クエリ・フィルター・ファセット・ページネーション |
| `SearchResult` | dataclass | ヒット一覧・総件数・ファセット集計・処理時間 |
| `IndexDocument` | dataclass | ドキュメント ID・フィールドマップ |
| `IndexResult` | dataclass | インデックス済みドキュメント ID・バージョン |
| `BulkResult` | dataclass | 成功件数・失敗件数・失敗詳細リスト |
| `Filter` | dataclass | フィールド・演算子・値 |
| `SearchError` | Exception | 検索エラー基底クラス |

### 使用例

```python
import asyncio
from k1s0_search_client import (
    Filter,
    GrpcSearchClient,
    IndexDocument,
    SearchQuery,
)

client = GrpcSearchClient(server_url="http://search-server:8080")

# ドキュメントのインデックス登録
doc = IndexDocument(
    id="prod-001",
    fields={"name": "Rust プログラミング入門", "price": 3800, "category": "books"},
)
result = await client.index_document("products", doc)
print(f"インデックス済み: {result.id} v{result.version}")

# バルクインデックス
docs = [
    IndexDocument(id="prod-002", fields={"name": "Go 言語仕様", "price": 4200, "category": "books"}),
    IndexDocument(id="prod-003", fields={"name": "TypeScript 実践", "price": 3500, "category": "books"}),
]
bulk_result = await client.bulk_index("products", docs)
print(f"成功: {bulk_result.success_count}, 失敗: {bulk_result.failed_count}")

# 全文検索
query = SearchQuery(
    query="Rust プログラミング",
    filters=[Filter(field="category", operator="eq", value="books")],
    facets=["category"],
    page=0,
    size=20,
)
search_result = await client.search("products", query)
print(f"総件数: {search_result.total}, 処理時間: {search_result.took_ms}ms")
for hit in search_result.hits:
    print(hit)
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| grpcio | >=1.70 | gRPC クライアント |
| grpcio-tools | >=1.70 | Protobuf コード生成 |
| pydantic | >=2.10 | リクエスト・レスポンスバリデーション |

### テスト方針

- テストフレームワーク: pytest
- リント/フォーマット: ruff
- モック: unittest.mock / pytest-mock
- カバレッジ目標: 90%以上
- 実行: `pytest` / `ruff check .`

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
        async fn search<T: serde::de::DeserializeOwned + Send + 'static>(&self, index: &str, query: SearchQuery) -> Result<SearchResult<T>, SearchError>;
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

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-search-server設計](system-search-server設計.md) — 検索サーバー設計
- [system-library-pagination設計](system-library-pagination設計.md) — ページネーションライブラリ（SearchResult のページング）
- [system-library-eventstore設計](system-library-eventstore設計.md) — イベント永続化（ドメインイベントから検索インデックス更新）
- [system-library-kafka設計](system-library-kafka設計.md) — Kafka コンシューマー（インデックス更新イベント）
