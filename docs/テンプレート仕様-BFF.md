# テンプレート仕様 — BFF (Backend for Frontend)

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で BFF（Backend for Frontend）を選択した際に生成される **全ファイルのスケルトンコード** を定義する。対象言語は **Go (gqlgen)** と **Rust (async-graphql)** の2つ。

BFF はフロントエンドクライアントに特化した API ゲートウェイパターンであり、上流の各マイクロサービスを集約し、GraphQL エンドポイントとしてフロントエンドに提供する。

### BFF 設計原則

- **DB / Kafka / Redis を使用しない** — BFF はデータストアを持たず、上流サーバーへの HTTP/gRPC プロキシのみを行う
- **GraphQL + HTTP 専任** — クライアントとの通信は GraphQL、上流サービスとの通信は HTTP（REST）/ gRPC クライアント
- **テンプレート変数**: `service_name` に `-bff` サフィックスを付与（例: `order-bff`）

### 生成条件

| 条件 | 値 |
|---|---|
| Tier | `service` |
| API 方式 | `graphql` を含む |
| BFF 言語 | CLI 対話フローで Go / Rust を選択 |

CLI の対話フローでは `step_detail_server()` 内で「GraphQL BFF を生成しますか？」と確認し、「はい」の場合に BFF 言語（Go / Rust）を選択する。`GenerateConfig.detail.bff_language` に `Some(Language::Go)` または `Some(Language::Rust)` が設定される。

---

## 配置パス

BFF は親サーバーのディレクトリ内に `bff/` サブディレクトリとして生成される。

```
regions/service/{service_name}/server/{server_lang}/bff/
```

例: `regions/service/order/server/go/bff/`

> **注記**: `{server_lang}` はサーバー本体の言語ディレクトリ。BFF 言語の選択は BFF 内部のスケルトンコード生成に影響するが、配置パスはサーバー本体の言語に従う。

---

## Go BFF テンプレート

テンプレートファイルは `CLI/templates/bff/go/` に配置する。

### ファイル構成

```
bff/go/
├── cmd/main.go
├── go.mod
├── internal/
│   ├── handler/
│   │   ├── graphql_resolver.go
│   │   └── graphql_resolver_test.go
│   └── client/
│       └── upstream.go
├── api/graphql/
│   ├── schema.graphql
│   └── gqlgen.yml
├── config/config.yaml
├── Dockerfile
└── README.md
```

| ファイル | テンプレート | 説明 |
|---|---|---|
| `cmd/main.go` | `cmd/main.go.tera` | エントリポイント。gqlgen server + healthz |
| `go.mod` | `go.mod.tera` | gqlgen, gqlparser 依存 |
| `internal/handler/graphql_resolver.go` | `internal/handler/graphql_resolver.go.tera` | gqlgen resolver (Query/Mutation, upstream HTTP 呼出) |
| `internal/handler/graphql_resolver_test.go` | `internal/handler/graphql_resolver_test.go.tera` | resolver テスト |
| `internal/client/upstream.go` | `internal/client/upstream.go.tera` | HTTP/gRPC upstream client |
| `api/graphql/schema.graphql` | `api/graphql/schema.graphql.tera` | Query/Mutation/型定義 |
| `api/graphql/gqlgen.yml` | `api/graphql/gqlgen.yml.tera` | gqlgen 設定 |
| `config/config.yaml` | `config/config.yaml.tera` | upstream.http_url, upstream.grpc_address |
| `Dockerfile` | `Dockerfile.tera` | マルチステージビルド |
| `README.md` | `README.md.tera` | BFF 説明 |

### cmd/main.go

`cmd/main.go.tera` — エントリポイント。gqlgen サーバー + ヘルスチェックエンドポイント。

```go
package main

import (
    "log"
    "net/http"

    "{{ go_module }}/internal/handler"
)

func main() {
    resolver := handler.NewResolver()
    srv := handler.NewGraphQLServer(resolver)

    log.Printf("{{ service_name }}-bff starting on :8080")
    log.Fatal(http.ListenAndServe(":8080", srv))
}
```

### go.mod

`go.mod.tera` — gqlgen + gqlparser 依存のみ。DB / Kafka / Redis 依存は含まない。

```go
module {{ go_module }}

go 1.22

require (
    github.com/99designs/gqlgen v0.17.45
    github.com/vektah/gqlparser/v2 v2.5.11
)
```

### internal/handler/graphql_resolver.go

`internal/handler/graphql_resolver.go.tera` — gqlgen リゾルバー。upstream サービスへの HTTP コールを実装する。

```go
package handler

import (
    "context"
    "net/http"
)

type Resolver struct{}

func NewResolver() *Resolver {
    return &Resolver{}
}

func NewGraphQLServer(resolver *Resolver) http.Handler {
    // gqlgen server setup
    mux := http.NewServeMux()
    mux.HandleFunc("/query", func(w http.ResponseWriter, r *http.Request) {
        w.WriteHeader(http.StatusOK)
    })
    mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
        w.WriteHeader(http.StatusOK)
    })
    return mux
}
```

### internal/handler/graphql_resolver_test.go

`internal/handler/graphql_resolver_test.go.tera` — resolver のユニットテスト。upstream サービスのモックを使用して GraphQL リゾルバーの動作を検証する。

```go
package handler

import (
    "net/http"
    "net/http/httptest"
    "testing"
)

func TestNewResolver(t *testing.T) {
    resolver := NewResolver()
    if resolver == nil {
        t.Fatal("NewResolver() returned nil")
    }
}

func TestGraphQLServerHealthz(t *testing.T) {
    resolver := NewResolver()
    srv := NewGraphQLServer(resolver)

    req := httptest.NewRequest(http.MethodGet, "/healthz", nil)
    rec := httptest.NewRecorder()

    srv.ServeHTTP(rec, req)

    if rec.Code != http.StatusOK {
        t.Errorf("healthz returned status %d, want %d", rec.Code, http.StatusOK)
    }
}

func TestGraphQLServerQuery(t *testing.T) {
    resolver := NewResolver()
    srv := NewGraphQLServer(resolver)

    req := httptest.NewRequest(http.MethodPost, "/query", nil)
    rec := httptest.NewRecorder()

    srv.ServeHTTP(rec, req)

    if rec.Code != http.StatusOK {
        t.Errorf("/query returned status %d, want %d", rec.Code, http.StatusOK)
    }
}
```

### internal/client/upstream.go

`internal/client/upstream.go.tera` — HTTP/gRPC upstream client。上流サービスへのリクエストを抽象化する。

```go
package client

import (
    "context"
    "encoding/json"
    "fmt"
    "io"
    "net/http"
    "time"
)

// UpstreamClient は上流サービスへの HTTP クライアント。
type UpstreamClient struct {
    httpClient *http.Client
    baseURL    string
}

// NewUpstreamClient は UpstreamClient を生成する。
func NewUpstreamClient(baseURL string) *UpstreamClient {
    return &UpstreamClient{
        httpClient: &http.Client{
            Timeout: 30 * time.Second,
        },
        baseURL: baseURL,
    }
}

// Get は上流サービスに GET リクエストを送信する。
func (c *UpstreamClient) Get(ctx context.Context, path string, result interface{}) error {
    url := fmt.Sprintf("%s%s", c.baseURL, path)
    req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
    if err != nil {
        return fmt.Errorf("failed to create request: %w", err)
    }

    resp, err := c.httpClient.Do(req)
    if err != nil {
        return fmt.Errorf("upstream request failed: %w", err)
    }
    defer resp.Body.Close()

    if resp.StatusCode >= 400 {
        body, _ := io.ReadAll(resp.Body)
        return fmt.Errorf("upstream returned %d: %s", resp.StatusCode, string(body))
    }

    if err := json.NewDecoder(resp.Body).Decode(result); err != nil {
        return fmt.Errorf("failed to decode response: %w", err)
    }
    return nil
}

// Post は上流サービスに POST リクエストを送信する。
func (c *UpstreamClient) Post(ctx context.Context, path string, body io.Reader, result interface{}) error {
    url := fmt.Sprintf("%s%s", c.baseURL, path)
    req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, body)
    if err != nil {
        return fmt.Errorf("failed to create request: %w", err)
    }
    req.Header.Set("Content-Type", "application/json")

    resp, err := c.httpClient.Do(req)
    if err != nil {
        return fmt.Errorf("upstream request failed: %w", err)
    }
    defer resp.Body.Close()

    if resp.StatusCode >= 400 {
        respBody, _ := io.ReadAll(resp.Body)
        return fmt.Errorf("upstream returned %d: %s", resp.StatusCode, string(respBody))
    }

    if result != nil {
        if err := json.NewDecoder(resp.Body).Decode(result); err != nil {
            return fmt.Errorf("failed to decode response: %w", err)
        }
    }
    return nil
}
```

### api/graphql/schema.graphql

`api/graphql/schema.graphql.tera` — GraphQL スキーマ定義。Query / Mutation / 型定義のサンプル。

```graphql
type Query {
    health: Boolean!
}
```

上記は初期スケルトン。実装時には以下のようなスキーマへ拡張する:

```graphql
type Query {
    """ヘルスチェック"""
    health: Boolean!

    """エンティティ一覧取得"""
    items: [Item!]!

    """ID 指定取得"""
    item(id: ID!): Item
}

type Mutation {
    """エンティティ新規作成"""
    createItem(input: CreateItemInput!): Item!

    """エンティティ更新"""
    updateItem(id: ID!, input: UpdateItemInput!): Item!

    """エンティティ削除"""
    deleteItem(id: ID!): Boolean!
}

type Item {
    id: ID!
    name: String!
    description: String
    status: String!
    createdAt: String!
    updatedAt: String!
}

input CreateItemInput {
    name: String!
    description: String
}

input UpdateItemInput {
    name: String
    description: String
    status: String
}
```

### api/graphql/gqlgen.yml

`api/graphql/gqlgen.yml.tera` — gqlgen コード生成設定。

```yaml
schema:
  - api/graphql/schema.graphql
exec:
  filename: internal/handler/generated.go
  package: handler
model:
  filename: internal/handler/models_gen.go
  package: handler
```

### config/config.yaml

`config/config.yaml.tera` — BFF 固有の設定。upstream 接続情報を含む。

```yaml
server:
  port: 8080
  name: {{ service_name }}-bff
upstream:
  http_url: http://{{ service_name }}:8080
  grpc_address: {{ service_name }}:50051
```

| フィールド | 説明 |
|---|---|
| `server.port` | BFF の待ち受けポート |
| `server.name` | BFF のサービス名 |
| `upstream.http_url` | 上流サービスの HTTP エンドポイント |
| `upstream.grpc_address` | 上流サービスの gRPC アドレス |

### Dockerfile

`Dockerfile.tera` — マルチステージビルド。distroless ランタイム。

```dockerfile
# === Build Stage ===
FROM golang:1.22-bookworm AS builder

WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -trimpath -ldflags="-s -w" -o /app/server ./cmd/main.go

# === Runtime Stage ===
FROM gcr.io/distroless/static-debian12

COPY --from=builder /app/server /server
COPY --from=builder /app/config /config

EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/server"]
```

### README.md

`README.md.tera` — BFF の説明・セットアップ手順。

```markdown
# {{ service_name }}-bff

{{ service_name_pascal }} BFF（Backend for Frontend）。GraphQL エンドポイントを提供し、上流マイクロサービスへのリクエストを集約する。

## セットアップ

```bash
# 依存インストール
go mod download

# gqlgen コード生成
go run github.com/99designs/gqlgen generate

# 開発サーバー起動
go run ./cmd/main.go

# テスト実行
go test ./...
```

## API

- **GraphQL エンドポイント**: `POST /query`
- **ヘルスチェック**: `GET /healthz`

## 設計方針

- DB / Kafka / Redis を使用しない（データストアレス）
- 上流サービスへの HTTP/gRPC プロキシのみ
- GraphQL スキーマファーストアプローチ（gqlgen）
```

---

## Rust BFF テンプレート

テンプレートファイルは `CLI/templates/bff/rust/` に配置する。

### ファイル構成

```
bff/rust/
├── src/
│   ├── main.rs
│   ├── handler/
│   │   ├── mod.rs
│   │   └── graphql.rs
│   └── client/
│       ├── mod.rs
│       └── upstream.rs
├── tests/
│   └── integration_test.rs
├── Cargo.toml
├── config/config.yaml
├── Dockerfile
└── README.md
```

| ファイル | テンプレート | 説明 |
|---|---|---|
| `src/main.rs` | `src/main.rs.tera` | axum + async-graphql server（現状は actix、axum 移行予定） |
| `src/handler/mod.rs` | `src/handler/mod.rs.tera` | handler モジュール定義 |
| `src/handler/graphql.rs` | `src/handler/graphql.rs.tera` | async-graphql Schema、QueryRoot/MutationRoot |
| `src/client/mod.rs` | `src/client/mod.rs.tera` | client モジュール定義 |
| `src/client/upstream.rs` | `src/client/upstream.rs.tera` | reqwest upstream client |
| `tests/integration_test.rs` | `tests/integration_test.rs.tera` | 統合テスト |
| `Cargo.toml` | `Cargo.toml.tera` | async-graphql, reqwest 依存 |
| `config/config.yaml` | `config/config.yaml.tera` | upstream 拡張 |
| `Dockerfile` | `Dockerfile.tera` | マルチステージビルド |
| `README.md` | `README.md.tera` | BFF 説明 |

### src/main.rs

`src/main.rs.tera` — エントリポイント。現状は actix-web を使用しているが、axum への移行を予定している。

```rust
use actix_web::{web, App, HttpServer, HttpResponse};

mod handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("{{ service_name }}-bff starting on :8080");
    HttpServer::new(|| {
        App::new()
            .route("/healthz", web::get().to(HttpResponse::Ok))
            .configure(handler::graphql::configure)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

### src/handler/mod.rs

`src/handler/mod.rs.tera` — handler モジュール定義。

```rust
pub mod graphql;
```

### src/handler/graphql.rs

`src/handler/graphql.rs.tera` — async-graphql Schema、QueryRoot/MutationRoot の定義。

```rust
use actix_web::{web, HttpResponse};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/query", web::post().to(graphql_handler));
}

async fn graphql_handler() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"data": null}))
}
```

実装時には async-graphql の Schema / QueryRoot / MutationRoot を定義し、upstream client 経由で上流サービスを呼び出す:

```rust
use async_graphql::{Object, Schema, EmptySubscription};

pub type BffSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn health(&self) -> bool {
        true
    }

    /// エンティティ一覧取得（upstream HTTP 呼出）
    async fn items(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Vec<Item>> {
        let client = ctx.data::<crate::client::upstream::UpstreamClient>()?;
        let items = client.get_items().await?;
        Ok(items)
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// エンティティ新規作成
    async fn create_item(&self, ctx: &async_graphql::Context<'_>, input: CreateItemInput) -> async_graphql::Result<Item> {
        let client = ctx.data::<crate::client::upstream::UpstreamClient>()?;
        let item = client.create_item(&input).await?;
        Ok(item)
    }
}
```

### src/client/mod.rs

`src/client/mod.rs.tera` — client モジュール定義。

```rust
pub mod upstream;
```

### src/client/upstream.rs

`src/client/upstream.rs.tera` — reqwest を使用した upstream HTTP/gRPC クライアント。

```rust
use std::time::Duration;

/// UpstreamClient は上流サービスへの HTTP クライアント。
pub struct UpstreamClient {
    client: reqwest::Client,
    base_url: String,
}

impl UpstreamClient {
    /// UpstreamClient を生成する。
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
        }
    }

    /// GET リクエストを上流サービスに送信する。
    pub async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.get(&url).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("upstream returned {}: {}", status, body);
        }
        let result = resp.json::<T>().await?;
        Ok(result)
    }

    /// POST リクエストを上流サービスに送信する。
    pub async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> anyhow::Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.client.post(&url).json(body).send().await?;
        let status = resp.status();
        if !status.is_success() {
            let resp_body = resp.text().await.unwrap_or_default();
            anyhow::bail!("upstream returned {}: {}", status, resp_body);
        }
        let result = resp.json::<T>().await?;
        Ok(result)
    }
}
```

### tests/integration_test.rs

`tests/integration_test.rs.tera` — BFF の統合テスト。

```rust
//! {{ service_name }}-bff 統合テスト

#[cfg(test)]
mod tests {
    #[actix_web::test]
    async fn test_healthz() {
        use actix_web::{test, web, App, HttpResponse};

        let app = test::init_service(
            App::new()
                .route("/healthz", web::get().to(HttpResponse::Ok))
        ).await;

        let req = test::TestRequest::get().uri("/healthz").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_query_endpoint() {
        use actix_web::{test, App};

        let app = test::init_service(
            App::new()
                .configure({{ service_name_snake }}_bff::handler::graphql::configure)
        ).await;

        let req = test::TestRequest::post().uri("/query").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
```

### Cargo.toml

`Cargo.toml.tera` — async-graphql + reqwest 依存。DB / Kafka / Redis 依存は含まない。

```toml
[package]
name = "{{ service_name }}-bff"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-graphql = "7"
async-graphql-actix-web = "7"
reqwest = { version = "0.12", features = ["json"] }
anyhow = "1"
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
actix-rt = "2"
```

### config/config.yaml

`config/config.yaml.tera` — upstream 拡張設定。

```yaml
server:
  port: 8080
  name: {{ service_name }}-bff
upstream:
  http_url: http://{{ service_name }}:8080
  grpc_address: {{ service_name }}:50051
```

### Dockerfile

`Dockerfile.tera` — マルチステージビルド。distroless ランタイム。

```dockerfile
# === Build Stage ===
FROM rust:1.82-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY . .
RUN cargo build --release

# === Runtime Stage ===
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/{{ service_name }}-bff /server
COPY --from=builder /app/config /config

EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/server"]
```

### README.md

`README.md.tera` — BFF の説明・セットアップ手順。

```markdown
# {{ service_name }}-bff

{{ service_name_pascal }} BFF（Backend for Frontend）。async-graphql による GraphQL エンドポイントを提供し、上流マイクロサービスへのリクエストを集約する。

## セットアップ

```bash
# ビルド
cargo build

# 開発サーバー起動
cargo run

# テスト実行
cargo test
```

## API

- **GraphQL エンドポイント**: `POST /query`
- **ヘルスチェック**: `GET /healthz`

## 設計方針

- DB / Kafka / Redis を使用しない（データストアレス）
- 上流サービスへの HTTP/gRPC プロキシのみ
- async-graphql によるスキーマファーストアプローチ
```

---

## GraphQL スキーマ仕様

### Query 型

| フィールド | 型 | 説明 |
|---|---|---|
| `health` | `Boolean!` | BFF ヘルスチェック |
| `items` | `[Item!]!` | エンティティ一覧取得（upstream 呼出） |
| `item(id: ID!)` | `Item` | ID 指定取得（upstream 呼出） |

### Mutation 型

| フィールド | 型 | 説明 |
|---|---|---|
| `createItem(input: CreateItemInput!)` | `Item!` | エンティティ新規作成 |
| `updateItem(id: ID!, input: UpdateItemInput!)` | `Item!` | エンティティ更新 |
| `deleteItem(id: ID!)` | `Boolean!` | エンティティ削除 |

### 型定義

| 型名 | フィールド |
|---|---|
| `Item` | `id: ID!`, `name: String!`, `description: String`, `status: String!`, `createdAt: String!`, `updatedAt: String!` |
| `CreateItemInput` | `name: String!`, `description: String` |
| `UpdateItemInput` | `name: String`, `description: String`, `status: String` |

---

## Resolver 実装パターン

### Go (gqlgen) — resolver サンプル

```go
func (r *queryResolver) Items(ctx context.Context) ([]*model.Item, error) {
    client := r.upstreamClient
    var items []*model.Item
    if err := client.Get(ctx, "/api/v1/items", &items); err != nil {
        return nil, fmt.Errorf("failed to fetch items: %w", err)
    }
    return items, nil
}

func (r *mutationResolver) CreateItem(ctx context.Context, input model.CreateItemInput) (*model.Item, error) {
    client := r.upstreamClient
    body, _ := json.Marshal(input)
    var item model.Item
    if err := client.Post(ctx, "/api/v1/items", bytes.NewReader(body), &item); err != nil {
        return nil, fmt.Errorf("failed to create item: %w", err)
    }
    return &item, nil
}
```

### Rust (async-graphql) — resolver サンプル

```rust
#[Object]
impl QueryRoot {
    async fn items(&self, ctx: &Context<'_>) -> Result<Vec<Item>> {
        let client = ctx.data::<UpstreamClient>()?;
        let items: Vec<Item> = client.get("/api/v1/items").await?;
        Ok(items)
    }

    async fn item(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Item>> {
        let client = ctx.data::<UpstreamClient>()?;
        let item: Option<Item> = client.get(&format!("/api/v1/items/{}", id)).await?;
        Ok(item)
    }
}

#[Object]
impl MutationRoot {
    async fn create_item(&self, ctx: &Context<'_>, input: CreateItemInput) -> Result<Item> {
        let client = ctx.data::<UpstreamClient>()?;
        let item: Item = client.post("/api/v1/items", &input).await?;
        Ok(item)
    }
}
```

---

## Upstream 接続仕様

### HTTP REST クライアント

BFF は上流サービスへの通信に HTTP REST クライアントを使用する。

| 項目 | Go | Rust |
|---|---|---|
| HTTP クライアント | `net/http` 標準ライブラリ | `reqwest` |
| タイムアウト | 30 秒 | 30 秒 |
| リトライ | なし（初期スケルトン） | なし（初期スケルトン） |
| エラーハンドリング | HTTP 4xx/5xx を `error` として返す | HTTP 4xx/5xx を `anyhow::Error` として返す |

### gRPC クライアント（拡張）

上流サービスが gRPC を提供する場合、BFF は gRPC クライアントを使用して接続する。

| 項目 | Go | Rust |
|---|---|---|
| gRPC クライアント | `google.golang.org/grpc` | `tonic` |
| 接続方式 | `grpc.Dial(address)` | `tonic::transport::Channel` |

gRPC クライアントは config.yaml の `upstream.grpc_address` で接続先を指定する。

---

## config.yaml 拡張仕様

BFF の config.yaml は通常のサーバーテンプレートとは異なり、DB / Kafka / Redis セクションを含まない。代わりに `upstream` セクションで上流サービスの接続情報を管理する。

### 設定項目

| フィールド | 型 | 必須 | デフォルト | 説明 |
|---|---|---|---|---|
| `server.port` | integer | Yes | `8080` | BFF の待ち受けポート |
| `server.name` | string | Yes | `{service_name}-bff` | BFF のサービス名 |
| `upstream.http_url` | string | Yes | `http://{service_name}:8080` | 上流サービスの HTTP URL |
| `upstream.grpc_address` | string | No | `{service_name}:50051` | 上流サービスの gRPC アドレス |

### Vault / シークレット

BFF は DB / Kafka / Redis を使用しないため、Vault シークレットは不要。Helm Chart 生成時に `vault.secrets` は空配列となる。

---

## テストテンプレート仕様

### Go BFF テスト

| テストファイル | 対象 | テスト内容 |
|---|---|---|
| `internal/handler/graphql_resolver_test.go` | resolver | NewResolver の生成、/healthz レスポンス、/query レスポンス |

テストパターン:
- `TestNewResolver` — resolver インスタンスの生成検証
- `TestGraphQLServerHealthz` — ヘルスチェックエンドポイントの応答検証
- `TestGraphQLServerQuery` — GraphQL エンドポイントの応答検証

### Rust BFF テスト

| テストファイル | 対象 | テスト内容 |
|---|---|---|
| `tests/integration_test.rs` | BFF 全体 | /healthz レスポンス、/query レスポンス |

テストパターン:
- `test_healthz` — ヘルスチェックエンドポイントの応答検証
- `test_query_endpoint` — GraphQL エンドポイントの応答検証

---

## GraphQL Subscription の拡張

BFF テンプレートはデフォルトで Query / Mutation のみを提供し、Subscription 型は含まない。サーバー側で GraphQL Subscription（リアルタイムイベント配信）を実装している場合、BFF がそれを中継するには以下の拡張が必要となる。

### 拡張手順

1. **schema.graphql に Subscription 型を追加する**
2. **BFF の main に WebSocket transport を追加する**（Go: `github.com/gorilla/websocket`、Rust: `axum::extract::ws`）
3. **upstream サーバーの WebSocket エンドポイントに接続する** client 実装を追加する

> **サーバーテンプレートとの連携**: サーバー側の GraphQL Subscription テンプレート（Subscription 型定義、WebSocket transport 設定）は [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) の「GraphQL Subscription テンプレート」セクションで定義されている。BFF が Subscription を中継する場合は、サーバー側のスキーマ定義と整合性を保つ必要がある。

### Go BFF の Subscription 拡張例

```go
// schema.graphql に追加
type Subscription {
    itemChanged: Item!
}

// cmd/main.go に WebSocket transport を追加
import "github.com/gorilla/websocket"

srv.AddTransport(transport.Websocket{
    Upgrader: websocket.Upgrader{
        CheckOrigin: func(r *http.Request) bool { return true },
    },
})
```

### Rust BFF の Subscription 拡張例

```rust
// graphql.rs に SubscriptionRoot を追加
pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn item_changed(&self) -> impl Stream<Item = Item> {
        // upstream WebSocket 接続からイベントを中継
        futures_util::stream::empty()
    }
}

// main.rs に WebSocket route を追加
.route("/ws", get(GraphQLSubscription::new(schema.clone())))
```

---

## 関連ドキュメント

- [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) --- サーバーテンプレート（BFF の親サーバー、GraphQL Subscription 含む）
- [API設計](API設計.md) --- D-011 GraphQL 設計、BFF パターン
- [認証認可設計](認証認可設計.md) --- D-013 BFF + HttpOnly Cookie 認証
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) --- 変数置換・条件分岐の仕様
- [テンプレート仕様-Helm](テンプレート仕様-Helm.md) --- BFF 用 Helm Chart
- [CLIフロー](CLIフロー.md) --- BFF 生成の対話フロー
- [テンプレート仕様-レンダリングテスト](テンプレート仕様-レンダリングテスト.md) --- BFF スナップショットテスト
