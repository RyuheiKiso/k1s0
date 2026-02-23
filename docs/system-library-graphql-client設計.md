# k1s0-graphql-client ライブラリ設計

## 概要

system-graphql-gateway（ポート 8095）への GraphQL クライアントライブラリ。GraphQL クエリ・ミューテーション・サブスクリプション（WebSocket 経由）を統一インターフェースで提供する。自動型生成・レスポンスのデシリアライゼーション・エラーハンドリングを組み込み、全 Tier のサービスから共通利用する。

**配置先**: `regions/system/library/rust/graphql-client/`

## 公開 API

| 型・トレイト | 種別 | 説明 |
|-------------|------|------|
| `GraphQlClient` | トレイト | クエリ・ミューテーション・サブスクリプション実行インターフェース |
| `HttpGraphQlClient` | 構造体 | HTTP POST 経由の GraphQL 実行実装 |
| `GraphQlQuery` | 構造体 | クエリ文字列・変数マップ・オペレーション名 |
| `GraphQlResponse<T>` | 構造体 | データ・エラー一覧・エクステンション |
| `GraphQlError` | 構造体 | メッセージ・ロケーション・パス・エクステンション |
| `SubscriptionClient` | 構造体 | WebSocket 経由のサブスクリプション管理（接続・再接続・メッセージ受信） |
| `ClientError` | enum | `Network`・`Parse`・`GraphQl`・`Timeout`・`Unauthorized` |

## Rust 実装

**Cargo.toml**:

```toml
[package]
name = "k1s0-graphql-client"
version = "0.1.0"
edition = "2021"

[features]
subscription = ["tokio-tungstenite"]

[dependencies]
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
reqwest = { version = "0.12", features = ["json"] }
tokio-tungstenite = { version = "0.24", optional = true, features = ["native-tls"] }

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
mockall = "0.13"
wiremock = "0.6"
```

**Cargo.toml への追加行**:

```toml
k1s0-graphql-client = { path = "../../system/library/rust/graphql-client" }
# WebSocket サブスクリプションを有効化する場合:
k1s0-graphql-client = { path = "../../system/library/rust/graphql-client", features = ["subscription"] }
```

**モジュール構成**:

```
graphql-client/
├── src/
│   ├── lib.rs          # 公開 API（再エクスポート）
│   ├── client.rs       # GraphQlClient トレイト・HttpGraphQlClient
│   ├── query.rs        # GraphQlQuery・GraphQlResponse・GraphQlError
│   ├── subscription.rs # SubscriptionClient（WebSocket）
│   └── error.rs        # ClientError
└── Cargo.toml
```

**使用例**:

```rust
use k1s0_graphql_client::{GraphQlClient, GraphQlQuery, HttpGraphQlClient};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct UsersQuery {
    users: Vec<User>,
}

// クライアントの構築
let client = HttpGraphQlClient::new("http://graphql-gateway:8080/graphql")
    .with_auth_token("Bearer <token>");

// クエリの実行
let query = GraphQlQuery::new(r#"
    query GetUsers($limit: Int!) {
        users(limit: $limit) {
            id
            name
            email
        }
    }
"#)
.variable("limit", serde_json::json!(10))
.operation_name("GetUsers");

let response = client.query::<UsersQuery>(query).await?;
for user in &response.data.users {
    tracing::info!(id = %user.id, name = %user.name, "ユーザー取得");
}

// ミューテーションの実行
#[derive(Debug, Serialize)]
struct CreateUserInput {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUserMutation {
    create_user: User,
}

let mutation = GraphQlQuery::new(r#"
    mutation CreateUser($input: CreateUserInput!) {
        create_user(input: $input) {
            id
            name
            email
        }
    }
"#)
.variable("input", serde_json::to_value(CreateUserInput {
    name: "田中 太郎".to_string(),
    email: "tanaka@example.com".to_string(),
})?);

let result = client.mutate::<CreateUserMutation>(mutation).await?;
tracing::info!(id = %result.data.create_user.id, "ユーザー作成完了");
```

## Go 実装

**配置先**: `regions/system/library/go/graphql-client/`

```
graphql-client/
├── graphql_client.go      # GraphQlClient インターフェース・HttpGraphQlClient
├── query.go               # GraphQlQuery・GraphQlResponse・GraphQlError
├── subscription.go        # SubscriptionClient
├── graphql_client_test.go
├── go.mod
└── go.sum
```

**依存関係**: `github.com/stretchr/testify v1.10.0`

**主要インターフェース**:

```go
type GraphQlClient interface {
    Query(ctx context.Context, query GraphQlQuery, result interface{}) error
    Mutate(ctx context.Context, query GraphQlQuery, result interface{}) error
}

type GraphQlQuery struct {
    Query         string
    Variables     map[string]interface{}
    OperationName string
}

type Location struct {
    Line   int
    Column int
}

type GraphQlError struct {
    Message    string
    Locations  []Location
    Path       []interface{}
    Extensions map[string]interface{}
}

type HttpGraphQlClient struct{ /* ... */ }

func NewHttpGraphQlClient(endpoint string) *HttpGraphQlClient
func (c *HttpGraphQlClient) WithAuthToken(token string) *HttpGraphQlClient
func (c *HttpGraphQlClient) Query(ctx context.Context, query GraphQlQuery, result interface{}) error
func (c *HttpGraphQlClient) Mutate(ctx context.Context, query GraphQlQuery, result interface{}) error
```

**使用例**:

```go
client := NewHttpGraphQlClient("http://graphql-gateway:8080/graphql").
    WithAuthToken("Bearer <token>")

query := GraphQlQuery{
    Query: `query GetUsers($limit: Int!) { users(limit: $limit) { id name } }`,
    Variables: map[string]interface{}{"limit": 10},
    OperationName: "GetUsers",
}

var result struct {
    Users []struct {
        ID   string `json:"id"`
        Name string `json:"name"`
    } `json:"users"`
}

if err := client.Query(ctx, query, &result); err != nil {
    return err
}
fmt.Printf("ユーザー数: %d\n", len(result.Users))
```

## TypeScript 実装

**配置先**: `regions/system/library/typescript/graphql-client/`

```
graphql-client/
├── package.json        # "@k1s0/graphql-client", "type":"module"
├── tsconfig.json
├── vitest.config.ts
├── src/
│   └── index.ts        # GraphQlClient, HttpGraphQlClient, GraphQlQuery, GraphQlResponse, GraphQlError, SubscriptionClient, ClientError
└── __tests__/
    └── graphql-client.test.ts
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
  locations?: Array<{ line: number; column: number }>;
  path?: string[];
  extensions?: Record<string, unknown>;
}

export interface GraphQlResponse<T> {
  data: T;
  errors?: GraphQlError[];
  extensions?: Record<string, unknown>;
}

export interface GraphQlClient {
  query<T>(query: GraphQlQuery): Promise<GraphQlResponse<T>>;
  mutate<T>(query: GraphQlQuery): Promise<GraphQlResponse<T>>;
}

export class HttpGraphQlClient implements GraphQlClient {
  constructor(endpoint: string, options?: { authToken?: string; timeoutMs?: number });
  query<T>(query: GraphQlQuery): Promise<GraphQlResponse<T>>;
  mutate<T>(query: GraphQlQuery): Promise<GraphQlResponse<T>>;
}

export class SubscriptionClient {
  constructor(endpoint: string, options?: { authToken?: string });
  subscribe<T>(query: GraphQlQuery, onData: (data: T) => void, onError?: (error: ClientError) => void): () => void;
  disconnect(): void;
}

export class ClientError extends Error {
  constructor(message: string, public readonly code: 'NETWORK' | 'PARSE' | 'GRAPHQL' | 'TIMEOUT' | 'UNAUTHORIZED', public readonly graphQlErrors?: GraphQlError[]);
}
```

**カバレッジ目標**: 90%以上

## Dart 実装

**配置先**: `regions/system/library/dart/graphql_client/`

```
graphql_client/
├── pubspec.yaml        # k1s0_graphql_client
├── analysis_options.yaml
├── lib/
│   ├── graphql_client.dart
│   └── src/
│       ├── client.dart         # GraphQlClient abstract, HttpGraphQlClient
│       ├── query.dart          # GraphQlQuery, GraphQlResponse, GraphQlError
│       ├── subscription.dart   # SubscriptionClient
│       └── error.dart          # ClientError
└── test/
    └── graphql_client_test.dart
```

**pubspec.yaml 主要依存**:

```yaml
dependencies:
  http: ^1.2.0
  web_socket_channel: ^3.0.0
```

**使用例**:

```dart
import 'package:k1s0_graphql_client/graphql_client.dart';

final client = HttpGraphQlClient(
  endpoint: 'http://graphql-gateway:8080/graphql',
  authToken: 'Bearer <token>',
);

// クエリの実行
final query = GraphQlQuery(
  query: '''
    query GetUsers(\$limit: Int!) {
      users(limit: \$limit) { id name email }
    }
  ''',
  variables: {'limit': 10},
  operationName: 'GetUsers',
);

final response = await client.query(query);
final users = response.data['users'] as List;
print('ユーザー数: ${users.length}');
```

**カバレッジ目標**: 90%以上

## C# 実装

**配置先**: `regions/system/library/csharp/graphql-client/`

```
graphql-client/
├── src/
│   ├── GraphQlClient.csproj
│   ├── IGraphQlClient.cs        # クエリ・ミューテーション実行インターフェース
│   ├── HttpGraphQlClient.cs     # HTTP実装
│   ├── GraphQlQuery.cs          # GraphQlQuery・GraphQlResponse・GraphQlError
│   ├── SubscriptionClient.cs    # WebSocketサブスクリプション
│   └── ClientException.cs      # 公開例外型
├── tests/
│   ├── GraphQlClient.Tests.csproj
│   ├── Unit/
│   │   └── GraphQlQueryTests.cs
│   └── Integration/
│       └── HttpGraphQlClientTests.cs
├── .editorconfig
└── README.md
```

**NuGet 依存関係**:

| パッケージ | 用途 |
|-----------|------|
| System.Net.Http.Json | JSON HTTP リクエスト |
| System.Net.WebSockets.Client | WebSocket サブスクリプション |

**名前空間**: `K1s0.System.GraphQlClient`

**主要クラス・インターフェース**:

| 型 | 種別 | 説明 |
|---|------|------|
| `IGraphQlClient` | interface | クエリ・ミューテーション実行の抽象インターフェース |
| `HttpGraphQlClient` | class | HTTP POST 経由の GraphQL 実行実装 |
| `GraphQlQuery` | record | クエリ文字列・変数マップ・オペレーション名 |
| `GraphQlResponse<T>` | record | データ・エラー一覧 |
| `GraphQlError` | record | メッセージ・ロケーション・パス |
| `SubscriptionClient` | class | WebSocket サブスクリプション管理 |
| `ClientException` | class | 公開例外型 |

**主要 API**:

```csharp
namespace K1s0.System.GraphQlClient;

public record GraphQlQuery(
    string Query,
    IReadOnlyDictionary<string, object>? Variables = null,
    string? OperationName = null);

public record GraphQlError(
    string Message,
    IReadOnlyList<Location>? Locations = null,
    IReadOnlyList<string>? Path = null);

public record GraphQlResponse<T>(T Data, IReadOnlyList<GraphQlError>? Errors = null);

public interface IGraphQlClient : IAsyncDisposable
{
    Task<GraphQlResponse<T>> QueryAsync<T>(GraphQlQuery query, CancellationToken ct = default);
    Task<GraphQlResponse<T>> MutateAsync<T>(GraphQlQuery query, CancellationToken ct = default);
}

public sealed class HttpGraphQlClient : IGraphQlClient
{
    public HttpGraphQlClient(string endpoint, string? authToken = null);
    public Task<GraphQlResponse<T>> QueryAsync<T>(GraphQlQuery query, CancellationToken ct = default);
    public Task<GraphQlResponse<T>> MutateAsync<T>(GraphQlQuery query, CancellationToken ct = default);
    public ValueTask DisposeAsync();
}
```

**カバレッジ目標**: 90%以上

---

## Swift

### パッケージ構成
- ターゲット: `K1s0GraphQlClient`
- Swift 6.0 / swift-tools-version: 6.0
- プラットフォーム: macOS 14+, iOS 17+

### 主要な公開 API

```swift
public struct GraphQlQuery: Sendable {
    public let query: String
    public let variables: [String: any Sendable]
    public let operationName: String?
    public init(query: String, variables: [String: any Sendable] = [:], operationName: String? = nil)
}

public struct GraphQlError: Sendable {
    public let message: String
    public let path: [String]?
}

public struct GraphQlResponse<T: Decodable & Sendable>: Sendable {
    public let data: T
    public let errors: [GraphQlError]?
}

public protocol GraphQlClient: Sendable {
    func query<T: Decodable & Sendable>(_ query: GraphQlQuery) async throws -> GraphQlResponse<T>
    func mutate<T: Decodable & Sendable>(_ query: GraphQlQuery) async throws -> GraphQlResponse<T>
}

public actor HttpGraphQlClient: GraphQlClient {
    public init(endpoint: URL, authToken: String? = nil)
    public func query<T: Decodable & Sendable>(_ query: GraphQlQuery) async throws -> GraphQlResponse<T>
    public func mutate<T: Decodable & Sendable>(_ query: GraphQlQuery) async throws -> GraphQlResponse<T>
}
```

### エラー型

```swift
public enum ClientError: Error, Sendable {
    case network(underlying: Error)
    case parse(underlying: Error)
    case graphQl(errors: [GraphQlError])
    case timeout
    case unauthorized
}
```

### テスト
- Swift Testing フレームワーク（@Suite, @Test, #expect）
- カバレッジ目標: 80%以上

---

## Python 実装

**配置先**: `regions/system/library/python/graphql_client/`

### パッケージ構造

```
graphql_client/
├── pyproject.toml
├── src/
│   └── k1s0_graphql_client/
│       ├── __init__.py       # 公開 API（再エクスポート）
│       ├── client.py         # GraphQlClient ABC・HttpGraphQlClient
│       ├── query.py          # GraphQlQuery・GraphQlResponse・GraphQlError dataclass
│       ├── subscription.py   # SubscriptionClient
│       ├── exceptions.py     # ClientError
│       └── py.typed
└── tests/
    ├── test_graphql_client.py
    └── test_subscription.py
```

### 主要クラス・インターフェース

| 型 | 種別 | 説明 |
|---|------|------|
| `GraphQlClient` | ABC | クエリ・ミューテーション実行抽象基底クラス |
| `HttpGraphQlClient` | class | HTTP POST 経由の GraphQL 実行実装 |
| `GraphQlQuery` | dataclass | クエリ文字列・変数マップ・オペレーション名 |
| `GraphQlResponse` | dataclass | データ・エラー一覧・エクステンション |
| `GraphQlError` | dataclass | メッセージ・ロケーション・パス |
| `SubscriptionClient` | class | WebSocket サブスクリプション管理 |
| `ClientError` | Exception | 基底エラークラス |

### 使用例

```python
from k1s0_graphql_client import GraphQlQuery, HttpGraphQlClient

client = HttpGraphQlClient(
    endpoint="http://graphql-gateway:8080/graphql",
    auth_token="Bearer <token>",
)

query = GraphQlQuery(
    query="""
        query GetUsers($limit: Int!) {
            users(limit: $limit) { id name email }
        }
    """,
    variables={"limit": 10},
    operation_name="GetUsers",
)

response = await client.query(query)
for user in response.data["users"]:
    print(f"{user['id']}: {user['name']}")
```

### 依存ライブラリ

| パッケージ | バージョン | 用途 |
|-----------|-----------|------|
| httpx | >=0.27 | 非同期 HTTP クライアント |
| websockets | >=13.0 | WebSocket サブスクリプション |
| pydantic | >=2.10 | レスポンスバリデーション |

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
    fn test_graphql_query_builder() {
        let query = GraphQlQuery::new("query { users { id } }")
            .variable("limit", serde_json::json!(10))
            .operation_name("GetUsers");

        assert_eq!(query.operation_name, Some("GetUsers".to_string()));
        assert_eq!(query.variables.len(), 1);
    }

    #[test]
    fn test_client_error_types() {
        let err = ClientError::Unauthorized;
        assert!(matches!(err, ClientError::Unauthorized));
    }
}
```

### 統合テスト

- `wiremock` で GraphQL エンドポイントをモック。クエリ・ミューテーション・エラーレスポンスの各パターンをカバー。
- GraphQL エラー（`errors` フィールドあり）のレスポンスで `ClientError::GraphQl` が返ることを確認。
- 401 レスポンスで `ClientError::Unauthorized` が返ることを確認。
- タイムアウト設定を超過した場合に `ClientError::Timeout` が返ることを確認。

### モックテスト

```rust
use mockall::mock;

mock! {
    pub TestGraphQlClient {}
    #[async_trait]
    impl GraphQlClient for TestGraphQlClient {
        async fn query<T: serde::de::DeserializeOwned + Send + 'static>(&self, query: GraphQlQuery) -> Result<GraphQlResponse<T>, ClientError>;
        async fn mutate<T: serde::de::DeserializeOwned + Send + 'static>(&self, query: GraphQlQuery) -> Result<GraphQlResponse<T>, ClientError>;
    }
}

#[tokio::test]
async fn test_user_service_queries_graphql_gateway() {
    let mut mock = MockTestGraphQlClient::new();
    mock.expect_query::<UsersQuery>()
        .once()
        .returning(|_| Ok(GraphQlResponse {
            data: UsersQuery { users: vec![] },
            errors: None,
            extensions: None,
        }));

    let service = UserService::new(Arc::new(mock));
    service.list_users(10).await.unwrap();
}
```

**カバレッジ目標**: 90%以上

---

## 関連ドキュメント

- [system-library-概要](system-library-概要.md) — ライブラリ一覧・テスト方針
- [system-graphql-gateway設計](system-graphql-gateway設計.md) — GraphQL ゲートウェイ設計
- [system-library-websocket設計](system-library-websocket設計.md) — WebSocket ライブラリ（サブスクリプション接続管理）
- [system-library-authlib設計](system-library-authlib設計.md) — JWT 認証ライブラリ
- [system-library-pagination設計](system-library-pagination設計.md) — ページネーションライブラリ
- [GraphQL設計.md](GraphQL設計.md) — GraphQL 設計ガイドライン
