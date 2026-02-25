# k1s0-graphql-client ライブラリ設計

## 概要

GraphQL クライアントライブラリ。GraphQL クエリ・ミューテーションを `execute` / `executeMutation` メソッドで実行し、型安全なレスポンスデシリアライゼーション・エラーハンドリングを提供する。全 Tier のサービスから共通利用する。

**配置先**: `regions/system/library/rust/graphql-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `GraphQlClient` | トレイト | クエリ・ミューテーション実行インターフェース（execute・executeMutation） |
| `InMemoryGraphQlClient` | 構造体 | テスト用インメモリ実装（レスポンス登録→実行） |
| `GraphQlQuery` | 構造体 | クエリ文字列・変数（任意）・オペレーション名（任意） |
| `GraphQlResponse<T>` | 構造体 | data（任意）・errors（任意） |
| `GraphQlError` | 構造体 | message・locations（任意）・path（任意） |
| `ErrorLocation` | 構造体 | line・column |
| `ClientError` | enum | `RequestError`・`DeserializationError`・`GraphQlError`・`NotFound` |

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

**Cargo.toml への追加行**:

```toml
k1s0-graphql-client = { path = "../../system/library/rust/graphql-client" }
```

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
```

## Go 実装

**配置先**: `regions/system/library/go/graphql-client/`

```
graphql-client/
├── graphql_client.go
├── graphql_client_test.go
└── go.mod
```

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
}

type InMemoryGraphQlClient struct{ /* ... */ }
func NewInMemoryGraphQlClient() *InMemoryGraphQlClient
func (c *InMemoryGraphQlClient) SetResponse(operationName string, response any)
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/graphql-client/`

```
graphql-client/
├── package.json        # "@k1s0/graphql-client", "type":"module"
├── tsconfig.json
├── src/
│   ├── types.ts        # GraphQlQuery, GraphQlError, GraphQlResponse
│   ├── client.ts       # GraphQlClient, InMemoryGraphQlClient
│   └── index.ts
└── __tests__/
    └── client.test.ts
```

**主要 API**:

```typescript
export interface GraphQlQuery {
  query: string;
  variables?: Record<string, unknown>;
  operationName?: string;
}

export interface GraphQlError {
  message: string;
  locations?: { line: number; column: number }[];
  path?: (string | number)[];
}

export interface GraphQlResponse<T = unknown> {
  data?: T;
  errors?: GraphQlError[];
}

export interface GraphQlClient {
  execute<T = unknown>(query: GraphQlQuery): Promise<GraphQlResponse<T>>;
  executeMutation<T = unknown>(mutation: GraphQlQuery): Promise<GraphQlResponse<T>>;
}

export class InMemoryGraphQlClient implements GraphQlClient {
  setResponse(operationName: string, response: unknown): void;
  async execute<T = unknown>(query: GraphQlQuery): Promise<GraphQlResponse<T>>;
  async executeMutation<T = unknown>(mutation: GraphQlQuery): Promise<GraphQlResponse<T>>;
}
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/graphql-client/`

```
graphql-client/
├── src/
│   └── K1s0.GraphQlClient/
│       ├── K1s0.GraphQlClient.csproj
│       ├── IGraphQlClient.cs
│       ├── InMemoryGraphQlClient.cs
│       └── GraphQlQuery.cs
├── tests/
│   └── K1s0.GraphQlClient.Tests/
│       ├── K1s0.GraphQlClient.Tests.csproj
│       └── GraphQlClientTests.cs
```

**名前空間**: `K1s0.GraphQlClient`

**主要 API**:

```csharp
namespace K1s0.GraphQlClient;

public record GraphQlQuery(
    string Query,
    Dictionary<string, object>? Variables = null,
    string? OperationName = null);

public record GraphQlError(
    string Message,
    IReadOnlyList<ErrorLocation>? Locations = null,
    IReadOnlyList<object>? Path = null);

public record ErrorLocation(int Line, int Column);

public record GraphQlResponse<T>(T? Data, IReadOnlyList<GraphQlError>? Errors = null)
{
    public bool HasErrors => Errors is { Count: > 0 };
}

public interface IGraphQlClient
{
    Task<GraphQlResponse<T>> ExecuteAsync<T>(GraphQlQuery query, CancellationToken cancellationToken = default);
    Task<GraphQlResponse<T>> ExecuteMutationAsync<T>(GraphQlQuery mutation, CancellationToken cancellationToken = default);
}

public sealed class InMemoryGraphQlClient : IGraphQlClient
{
    // テスト用実装
}
```

**カバレッジ目標**: 90%以上

---

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

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-library-authlib設計](system-library-authlib設計.md) — JWT 認証ライブラリ
- [system-library-pagination設計](system-library-pagination設計.md) — ページネーションライブラリ
