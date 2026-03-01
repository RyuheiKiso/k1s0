# k1s0-graphql-client ライブラリ設計

## 概要

GraphQL クライアントライブラリ。GraphQL クエリ・ミューテーションを `execute` / `executeMutation` メソッドで実行し、型安全なレスポンスデシリアライゼーション・エラーハンドリングを提供する。全 Tier のサービスから共通利用する。

**配置先**: `regions/system/library/rust/graphql-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `GraphQlClient` | トレイト | クエリ・ミューテーション・サブスクリプション実行インターフェース（execute・executeMutation・subscribe） |
| `InMemoryGraphQlClient` | 構造体 | テスト用インメモリ実装（レスポンス登録→実行） |
| `GraphQlQuery` | 構造体 | クエリ文字列・変数（任意）・オペレーション名（任意） |
| `GraphQlResponse<T>` | 構造体 | data（任意）・errors（任意） |
| `GraphQlError` | 構造体 | message・locations（任意）・path（任意） |
| `ErrorLocation` | 構造体 | line・column |
| `ClientError` | enum | `RequestError`・`DeserializationError`・`GraphQlError`・`NotFound` |

### 命名規則の言語別対応

ドキュメント中のメソッド名は Rust/snake_case で統一表記しているが、各言語の慣習に従う。

| ドキュメント表記（Rust/snake_case） | Go（PascalCase） | TypeScript（camelCase） | Dart（camelCase） |
|-------------------------------------|-----------------|------------------------|------------------|
| `execute` | `Execute` | `execute` | `execute` |
| `execute_mutation` | `ExecuteMutation` | `executeMutation` | `executeMutation` |
| `subscribe` | `Subscribe` | `subscribe` | `subscribe` |
| `set_response` | `SetResponse` | `setResponse` | `setResponse` |
| `set_subscription_events` | `SetSubscriptionEvents` | `setSubscriptionEvents` | `setSubscriptionEvents` |

> **Rust の命名**: Rust の `InMemoryGraphQlClient` では `register_response()` / `register_subscription_events()` を使用する（他3言語の `set_*` 系とは異なる）。

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-graphql-client"
version = "0.1.0"
edition = "2021"

[features]
mock = ["mockall"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync"] }
mockall = { version = "0.13", optional = true }
```

**依存追加**: `k1s0-graphql-client = { path = "../../system/library/rust/graphql-client" }`（[追加方法参照](../_common/共通実装パターン.md#cargo依存追加)）

**モジュール構成**:

```
graphql-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # GraphQlClient トレイト・InMemoryGraphQlClient
│   ├── query.rs        # GraphQlQuery・GraphQlResponse・GraphQlError・ErrorLocation
│   └── error.rs        # ClientError
└── Cargo.toml
```

**データモデル**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQlQuery {
    pub query: String,
    pub variables: Option<serde_json::Value>,
    pub operation_name: Option<String>,
}

impl GraphQlQuery {
    pub fn new(query: impl Into<String>) -> Self;
    pub fn variables(mut self, variables: serde_json::Value) -> Self;
    pub fn operation_name(mut self, name: impl Into<String>) -> Self;
}
```

> **`GraphQlQuery.variables` の言語別型**:
> | 言語 | 型 |
> |------|-----|
> | Rust | `Option<serde_json::Value>` |
> | Go | `map[string]any`（`omitempty` JSON タグ付き） |
> | TypeScript | `Record<string, unknown>?` |
> | Dart | `Map<String, dynamic>?` |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQlError {
    pub message: String,
    pub locations: Option<Vec<ErrorLocation>>,
    pub path: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    pub line: u32,
    pub column: u32,
}
```

**トレイト**:

```rust
#[async_trait]
pub trait GraphQlClient: Send + Sync {
    async fn execute<T: DeserializeOwned + Send>(
        &self,
        query: GraphQlQuery,
    ) -> Result<GraphQlResponse<T>, ClientError>;

    async fn execute_mutation<T: DeserializeOwned + Send>(
        &self,
        mutation: GraphQlQuery,
    ) -> Result<GraphQlResponse<T>, ClientError>;

    async fn subscribe<T: DeserializeOwned + Send>(
        &self,
        subscription: GraphQlQuery,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<GraphQlResponse<T>, ClientError>> + Send>>, ClientError>;
}
```

**エラー型**:

```rust
pub enum ClientError {
    RequestError(String),
    DeserializationError(String),
    GraphQlError(String),
    NotFound(String),
}
```

**使用例**:

```rust
use k1s0_graphql_client::{GraphQlClient, GraphQlQuery, InMemoryGraphQlClient};

let client = InMemoryGraphQlClient::new();
client.register_response(
    "{ users { id } }",
    serde_json::json!({"users": [{"id": "1"}]}),
).await;

let query = GraphQlQuery::new("{ users { id } }");
let response: GraphQlResponse<serde_json::Value> = client.execute(query).await?;
assert!(response.data.is_some());

// ミューテーションの実行
let mutation = GraphQlQuery::new("mutation { createUser }")
    .operation_name("CreateUser");
let result: GraphQlResponse<serde_json::Value> = client.execute_mutation(mutation).await?;

client.register_subscription_events(
    "OnUserCreated",
    vec![
        serde_json::json!({"userCreated": {"id": "1", "name": "Alice"}}),
        serde_json::json!({"userCreated": {"id": "2", "name": "Bob"}}),
    ],
).await;
let subscription = GraphQlQuery::new("subscription { userCreated { id name } }")
    .operation_name("OnUserCreated");
let mut stream = client.subscribe::<serde_json::Value>(subscription).await.unwrap();
while let Some(event) = stream.next().await {
    println!("{:?}", event);
}
```

**本番用クライアントの初期化例**:

```rust
use k1s0_graphql_client::GraphQlHttpClient;
use std::collections::HashMap;

// 本番用 HTTP クライアント（endpoint と headers を設定）
let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), "Bearer <token>".to_string());

let client = GraphQlHttpClient::new(
    "https://api.example.com/graphql",
    headers,
);
```

## Go 実装

**配置先**: `regions/system/library/go/graphql-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要インターフェース**:

```go
type GraphQlQuery struct {
    Query         string         `json:"query"`
    Variables     map[string]any `json:"variables,omitempty"`
    OperationName string         `json:"operationName,omitempty"`
}

type GraphQlError struct {
    Message   string          `json:"message"`
    Locations []ErrorLocation `json:"locations,omitempty"`
    Path      []any           `json:"path,omitempty"`
}

type ErrorLocation struct {
    Line   int `json:"line"`
    Column int `json:"column"`
}

type GraphQlResponse[T any] struct {
    Data   *T             `json:"data,omitempty"`
    Errors []GraphQlError `json:"errors,omitempty"`
}

type GraphQlClient interface {
    Execute(ctx context.Context, query GraphQlQuery, result any) (*GraphQlResponse[any], error)
    ExecuteMutation(ctx context.Context, mutation GraphQlQuery, result any) (*GraphQlResponse[any], error)
    Subscribe(ctx context.Context, subscription GraphQlQuery) (<-chan *GraphQlResponse[any], error)
}

type InMemoryGraphQlClient struct{ /* ... */ }
func NewInMemoryGraphQlClient() *InMemoryGraphQlClient
func (c *InMemoryGraphQlClient) SetResponse(operationName string, response any)
func (c *InMemoryGraphQlClient) SetSubscriptionEvents(operationName string, events []any)
func (c *InMemoryGraphQlClient) Subscribe(ctx context.Context, subscription GraphQlQuery) (<-chan *GraphQlResponse[any], error)
```

> **言語別注記 - `GraphQlQuery` の初期化**: Go は Rust のようなビルダーメソッド（`new()`, `variables()`, `operation_name()`）を持たず、struct リテラルを使用して直接フィールドを代入する。
>
> ```go
> // Go での初期化例（struct リテラル）
> query := GraphQlQuery{
>     Query:         "{ users { id } }",
>     Variables:     map[string]any{"limit": 10},
>     OperationName: "GetUsers",
> }
> ```

**本番用クライアント（HTTP）の初期化**:

```go
// 本番用 HTTP クライアント（endpoint と headers を設定）
// 実装例: GraphQlHttpClient など別構造体で提供
client := NewGraphQlHttpClient(
    "https://api.example.com/graphql",
    map[string]string{"Authorization": "Bearer <token>"},
)
```

> **`GraphQlResponse.Errors` の nil 表現**: Go の `Errors []GraphQlError` はゼロ値が `nil` スライスであり、これがオプショナルを表す慣用表現。`nil` スライス = エラーなし。

## TypeScript 実装

**配置先**: `regions/system/library/typescript/graphql-client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**主要 API**:

```typescript
export interface GraphQlQuery {
  query: string;
  variables?: Record<string, unknown>;
  operationName?: string;
}

// ErrorLocation 型（line/column は GraphQL エラー位置情報）
export interface ErrorLocation {
  line: number;
  column: number;
}

export interface GraphQlError {
  message: string;
  locations?: ErrorLocation[];
  path?: (string | number)[];
}

export interface GraphQlResponse<T = unknown> {
  data?: T;           // オプショナル（GraphQL 仕様上、エラー時は省略可能）
  errors?: GraphQlError[];
}

export interface GraphQlClient {
  execute<T = unknown>(query: GraphQlQuery): Promise<GraphQlResponse<T>>;
  executeMutation<T = unknown>(mutation: GraphQlQuery): Promise<GraphQlResponse<T>>;
  subscribe<T = unknown>(subscription: GraphQlQuery): AsyncIterable<GraphQlResponse<T>>;
}

export class InMemoryGraphQlClient implements GraphQlClient {
  setResponse(operationName: string, response: unknown): void;
  setSubscriptionEvents(operationName: string, events: unknown[]): void;
  async execute<T = unknown>(query: GraphQlQuery): Promise<GraphQlResponse<T>>;
  async executeMutation<T = unknown>(mutation: GraphQlQuery): Promise<GraphQlResponse<T>>;
  async *subscribe<T = unknown>(subscription: GraphQlQuery): AsyncIterable<GraphQlResponse<T>>;
}

// 本番用 HTTP クライアントの初期化
export class GraphQlHttpClient implements GraphQlClient {
  constructor(endpoint: string, headers?: Record<string, string>);
  // ...（GraphQlClient インターフェースのメソッドを実装）
}
```

**本番用クライアントの初期化例**:

```typescript
import { GraphQlHttpClient } from 'k1s0-graphql-client';

const client = new GraphQlHttpClient(
  'https://api.example.com/graphql',
  { Authorization: 'Bearer <token>' },
);
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/graphql_client/`（[定型構成参照](../_common/共通実装パターン.md#定型ディレクトリ構成)）

**モジュール構成**:

```
graphql-client/
├── lib/
│   ├── graphql_client.dart       # 公開 API（再エクスポート）
│   └── src/
│       ├── graphql_client.dart   # GraphQlClient abstract class・InMemoryGraphQlClient
│       └── graphql_query.dart    # GraphQlQuery・GraphQlResponse・GraphQlError・ErrorLocation
└── pubspec.yaml
```

**データモデル**:

```dart
class GraphQlQuery {
  final String query;
  final Map<String, dynamic>? variables;
  final String? operationName;

  const GraphQlQuery({
    required String query,
    Map<String, dynamic>? variables,
    String? operationName,
  });
}

class GraphQlError {
  final String message;
  final List<ErrorLocation>? locations;
  final List<dynamic>? path;

  const GraphQlError({
    required String message,
    List<ErrorLocation>? locations,
    List<dynamic>? path,
  });
}

class ErrorLocation {
  final int line;
  final int column;

  const ErrorLocation(int line, int column);
}

class GraphQlResponse<T> {
  final T? data;
  final List<GraphQlError>? errors;

  const GraphQlResponse({T? data, List<GraphQlError>? errors});

  bool get hasErrors => errors != null && errors!.isNotEmpty;
}
```

**クライアントインターフェース**:

```dart
abstract class GraphQlClient {
  // Dart はジェネリクスの実行時型消去の都合上、fromJson 引数でデシリアライザを明示的に渡す設計
  // （Go/Rust/TypeScript では fromJson 引数は不要）
  Future<GraphQlResponse<T>> execute<T>(
    GraphQlQuery query,
    T Function(Map<String, dynamic>) fromJson,
  );

  Future<GraphQlResponse<T>> executeMutation<T>(
    GraphQlQuery mutation,
    T Function(Map<String, dynamic>) fromJson,
  );

  Stream<GraphQlResponse<T>> subscribe<T>(
    GraphQlQuery subscription,
    T Function(Map<String, dynamic>) fromJson,
  );
}

class InMemoryGraphQlClient implements GraphQlClient {
  // response 型は Map<String, dynamic> に限定（他言語の any/unknown より厳格な型安全設計）
  void setResponse(String operationName, Map<String, dynamic> response);
  void setSubscriptionEvents(String operationName, List<Map<String, dynamic>> events);

  @override
  Future<GraphQlResponse<T>> execute<T>(
    GraphQlQuery query,
    T Function(Map<String, dynamic>) fromJson,
  );

  @override
  Future<GraphQlResponse<T>> executeMutation<T>(
    GraphQlQuery mutation,
    T Function(Map<String, dynamic>) fromJson,
  );

  @override
  Stream<GraphQlResponse<T>> subscribe<T>(
    GraphQlQuery subscription,
    T Function(Map<String, dynamic>) fromJson,
  );
}
```

> **他言語との設計差異**: Dart の `execute` / `executeMutation` は `fromJson` 引数を必須とする。これは Dart のジェネリクスが実行時に型情報を消去するため、`T` への自動デシリアライズが不可能であることに起因する。Go/Rust/TypeScript では実行時リフレクション・トレイト境界・型推論によって `fromJson` 引数が不要。

> **`setResponse` の型**: Dart の `setResponse` は `response` 引数を `Map<String, dynamic>` として受け取る（Go/TypeScript の `any`/`unknown` より型が限定される）。これは Dart の型安全性に合わせた意図的な設計。

### Dart の `fromJson` パターンについて

Dart のジェネリクスは**実行時型消去**（type erasure）の影響を受けるため、`T` の実際の型情報がランタイムに失われる。このため、`execute<User>(query)` のように型パラメータを指定しても、`Map<String, dynamic>` を `User` に自動変換することができない。

他言語との比較:

| 言語 | `fromJson` 引数の要否 | 理由 |
|------|----------------------|------|
| Rust | 不要 | `DeserializeOwned` トレイト境界でコンパイル時にデシリアライザが確定 |
| Go | 不要 | `json.Unmarshal` + `any` 型で実行時に柔軟にデシリアライズ |
| TypeScript | 不要 | 型情報はコンパイル時のみ（実行時は JS）、JSON.parse で動的解析 |
| Dart | **必要** | ジェネリクス型情報が実行時に消去されるため、明示的なデシリアライザが必須 |

`fromJson` 引数の典型的な渡し方:

```dart
// モデルクラスに静的 fromJson を定義しておく慣例
class User {
  final String id;
  final String name;
  User({required this.id, required this.name});
  factory User.fromJson(Map<String, dynamic> json) =>
      User(id: json['id'] as String, name: json['name'] as String);
}

// execute に渡す
final response = await client.execute<User>(query, User.fromJson);
// または lambda で渡す
final response = await client.execute<User>(query, (json) => User.fromJson(json));
```

**本番用クライアントの初期化例**:

```dart
import 'package:k1s0_graphql_client/graphql_client.dart';

// 本番用 HTTP クライアント（endpoint と headers を設定）
final client = GraphQlHttpClient(
  endpoint: 'https://api.example.com/graphql',
  headers: {'Authorization': 'Bearer <token>'},
);
```

**使用例**:

```dart
import 'package:k1s0_graphql_client/graphql_client.dart';

final client = InMemoryGraphQlClient();
client.setResponse('GetUsers', {
  'data': {
    'users': [{'id': '1', 'name': 'Alice'}],
  },
});

final query = GraphQlQuery(query: '{ users { id name } }', operationName: 'GetUsers');
final response = await client.execute(
  query,
  (json) => User.fromJson(json),
);
assert(response.data != null);
assert(!response.hasErrors);

// ミューテーションの実行
final mutation = GraphQlQuery(
  query: 'mutation { createUser }',
  operationName: 'CreateUser',
);
final result = await client.executeMutation(
  mutation,
  (json) => User.fromJson(json),
);
```

**カバレッジ目標**: 90%以上

## テスト戦略

### ユニットテスト（Rust）

```rust
#[test]
fn test_query_builder() {
    let q = GraphQlQuery::new("{ users { id name } }")
        .variables(serde_json::json!({"limit": 10}))
        .operation_name("GetUsers");

    assert_eq!(q.query, "{ users { id name } }");
    assert!(q.variables.is_some());
    assert_eq!(q.operation_name.unwrap(), "GetUsers");
}

#[tokio::test]
async fn test_inmemory_execute() {
    let client = InMemoryGraphQlClient::new();
    client.register_response(
        "{ users { id } }",
        serde_json::json!({"users": [{"id": "1"}]}),
    ).await;

    let query = GraphQlQuery::new("{ users { id } }");
    let resp: GraphQlResponse<serde_json::Value> = client.execute(query).await.unwrap();
    assert!(resp.data.is_some());
}

#[tokio::test]
async fn test_inmemory_mutation() {
    let client = InMemoryGraphQlClient::new();
    client.register_response(
        "mutation { createUser }",
        serde_json::json!({"id": "new-1"}),
    ).await;

    let mutation = GraphQlQuery::new("mutation { createUser }");
    let resp: GraphQlResponse<serde_json::Value> = client.execute_mutation(mutation).await.unwrap();
    assert!(resp.data.is_some());
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](../_common/概要.md) — ライブラリ一覧・テスト方針
- [system-library-authlib設計](../auth-security/authlib.md) — JWT 認証ライブラリ
- [system-library-pagination設計](../data/pagination.md) — ページネーションライブラリ
