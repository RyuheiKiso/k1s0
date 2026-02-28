# テンプレート仕様 — サーバー — Rust (axum + tokio)

本ドキュメントは、[テンプレート仕様-サーバー](サーバー.md) から分割された Rust (axum + tokio) テンプレートの詳細仕様である。

---

## Redis クライアント

`has_redis = true` の場合に生成される Redis キャッシュクライアント。`redis_session`（BFF用セッション管理）とは独立した汎用キャッシュクライアントである。

### `src/infra/redis_client.rs`

テンプレート: `CLI/templates/server/rust/src/infra/redis_client.rs.tera`

redis クレートのラッパーとして以下のメソッドを提供する:
- `RedisClient::new(cfg)` — クライアント初期化
- `get(key)` — キャッシュ取得
- `set(key, value, ttl_secs)` — TTL付きキャッシュ保存
- `delete(key)` — キャッシュ削除
- `ping()` — ヘルスチェック

## config/config.yaml

`config/config.yaml.tera` — [config.md](../../cli/config/config設計.md) 準拠のアプリケーション設定ファイル。

```yaml
app:
  name: "{{ service_name }}"
  version: "0.1.0"
  environment: "development"

server:
  port: "8080"
  read_timeout_sec: 30
  write_timeout_sec: 30

{% if has_database %}
database:
  host: "localhost"
{% if database_type == "postgresql" %}
  port: 5432
{% elif database_type == "mysql" %}
  port: 3306
{% endif %}
  user: "{{ service_name_snake }}"
  password: ""
  name: "{{ service_name_snake }}"
{% if database_type == "postgresql" %}
  ssl_mode: "disable"
{% endif %}
  pool:
    max_open: 25
    max_idle: 5
    max_lifetime_sec: 300
{% endif %}

{% if has_kafka %}
kafka:
  brokers:
    - "localhost:9092"
  schema_registry: "http://localhost:8081"
{% endif %}

{% if has_redis %}
redis:
  addr: "localhost:6379"
  password: ""
  db: 0
{% endif %}

observability:
  trace_endpoint: "localhost:4317"
  metric_endpoint: "localhost:4317"
  log_level: "info"
```

## api/openapi/openapi.yaml

`api/openapi/openapi.yaml.tera` — {% if api_styles is containing("rest") %} に該当。OpenAPI 3.0 定義。[API設計.md](../../architecture/api/API設計.md) D-123 utoipa に準拠する。

```yaml
{% if api_styles is containing("rest") %}
openapi: "3.0.3"
info:
  title: "{{ service_name_pascal }} API"
  version: "1.0.0"
  description: "{{ service_name }} の REST API 定義"
servers:
  - url: "http://localhost:8080"
    description: "ローカル開発環境"
paths:
  /api/v1/{{ service_name }}:
    get:
      summary: "一覧取得"
      operationId: "list{{ service_name_pascal }}"
      tags:
        - "{{ service_name }}"
      responses:
        "200":
          description: "成功"
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: "#/components/schemas/{{ service_name_pascal }}"
    post:
      summary: "新規作成"
      operationId: "create{{ service_name_pascal }}"
      tags:
        - "{{ service_name }}"
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/Create{{ service_name_pascal }}Request"
      responses:
        "201":
          description: "作成成功"
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/{{ service_name_pascal }}"
        "400":
          description: "バリデーションエラー"
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ErrorResponse"
  /api/v1/{{ service_name }}/{id}:
    get:
      summary: "ID 指定取得"
      operationId: "get{{ service_name_pascal }}"
      tags:
        - "{{ service_name }}"
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
      responses:
        "200":
          description: "成功"
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/{{ service_name_pascal }}"
        "404":
          description: "リソースが見つからない"
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/ErrorResponse"
components:
  schemas:
    {{ service_name_pascal }}:
      type: object
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string
        description:
          type: string
        status:
          type: string
        created_at:
          type: string
          format: date-time
        updated_at:
          type: string
          format: date-time
    Create{{ service_name_pascal }}Request:
      type: object
      required:
        - name
      properties:
        name:
          type: string
          maxLength: 255
        description:
          type: string
    ErrorResponse:
      type: object
      properties:
        code:
          type: string
        message:
          type: string
{% endif %}
```

## api/proto/service.proto

`api/proto/service.proto.tera` — {% if api_styles is containing("grpc") %} に該当。[API設計.md](../../architecture/api/API設計.md) D-009 gRPC 定義パターンに従う。

```proto
{% if api_styles is containing("grpc") %}
syntax = "proto3";

package {{ service_name_snake }}.v1;

// {{ service_name_pascal }}Service は {{ service_name }} の gRPC サービス定義。
service {{ service_name_pascal }}Service {
  rpc Get{{ service_name_pascal }} (Get{{ service_name_pascal }}Request) returns (Get{{ service_name_pascal }}Response);
  rpc List{{ service_name_pascal }} (List{{ service_name_pascal }}Request) returns (List{{ service_name_pascal }}Response);
  rpc Create{{ service_name_pascal }} (Create{{ service_name_pascal }}Request) returns (Create{{ service_name_pascal }}Response);
}

message Get{{ service_name_pascal }}Request {
  string id = 1;
}

message Get{{ service_name_pascal }}Response {
  string id = 1;
  string name = 2;
  string description = 3;
  string status = 4;
  string created_at = 5;
  string updated_at = 6;
}

message List{{ service_name_pascal }}Request {}

message List{{ service_name_pascal }}Response {
  repeated Get{{ service_name_pascal }}Response items = 1;
}

message Create{{ service_name_pascal }}Request {
  string name = 1;
  string description = 2;
}

message Create{{ service_name_pascal }}Response {
  string id = 1;
}
{% endif %}
```

## Cargo.toml

`Cargo.toml.tera` — クレート定義と依存関係。

```toml
[package]
name = "{{ rust_crate }}"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
opentelemetry = "0.24"
opentelemetry-otlp = "0.17"
opentelemetry_sdk = "0.24"
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }
thiserror = "2"
{% if api_styles is containing("rest") %}
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "8", features = ["axum"] }
{% endif %}
{% if api_styles is containing("grpc") %}
tonic = "0.12"
prost = "0.13"
{% endif %}
{% if api_styles is containing("graphql") %}
async-graphql = "7"
async-graphql-axum = "7"
{% endif %}
{% if has_database %}
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "{{ database_type }}"] }
{% endif %}
{% if has_kafka %}
rdkafka = { version = "0.36", features = ["cmake-build"] }
{% endif %}
{% if has_redis %}
redis = { version = "0.27", features = ["tokio-comp"] }
{% endif %}

[dev-dependencies]
mockall = "0.13"
tokio = { version = "1", features = ["test-util", "macros"] }

{% if api_styles is containing("grpc") %}
[build-dependencies]
tonic-build = "0.12"
{% endif %}
```

## src/main.rs

`src/main.rs.tera` — エントリポイント。axum Router、tower ミドルウェア、graceful shutdown。

```rust
use axum::{routing::get, Json, Router};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod adapter;
mod domain;
mod infra;
mod usecase;

use infra::config::Config;

#[tokio::main]
async fn main() {
    // --- Logger ---
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // --- Config ---
    let config = Config::load("config/config.yaml")
        .expect("Failed to load config");

{% if has_database %}
    // --- Database ---
    let pool = infra::persistence::create_pool(&config.database)
        .await
        .expect("Failed to connect to database");
{% endif %}

{% if has_kafka %}
    // --- Kafka ---
    let producer = infra::messaging::Producer::new(&config.kafka);
{% endif %}

    // --- DI: Repository → UseCase → Handler ---
{% if has_database %}
    let repo = infra::persistence::Repository::new(pool.clone());
{% endif %}
    let uc = usecase::{{ service_name_pascal }}UseCase::new(
{% if has_database %}
        repo,
{% endif %}
    );
    let handler = adapter::handler::AppHandler::new(uc);

    // --- Router ---
    let app = Router::new()
        .route("/healthz", get(|| async { Json(json!({"status": "ok"})) }))
        .route("/readyz", get({
{% if has_database %}
            let pool = pool.clone();
{% endif %}
            move || async move {
{% if has_database %}
                match sqlx::query("SELECT 1").execute(&pool).await {
                    Ok(_) => Json(json!({"status": "ready"})),
                    Err(_) => Json(json!({"status": "not ready"})),
                }
{% else %}
                Json(json!({"status": "ready"}))
{% endif %}
            }
        }))
        .merge(handler.routes())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // --- Server ---
    let addr: SocketAddr = format!("0.0.0.0:{}", config.server.port)
        .parse()
        .expect("Invalid address");
    tracing::info!("server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    // --- Graceful Shutdown ---
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received");
}
```

## src/domain/mod.rs

`src/domain/mod.rs.tera` — ドメインモジュール宣言。

```rust
pub mod model;
pub mod repository;
```

## src/domain/model.rs

`src/domain/model.rs.tera` — エンティティ定義。

```rust
use serde::{Deserialize, Serialize};

/// {{ service_name_pascal }}Entity はドメインエンティティを表す。
#[derive(Debug, Clone, Serialize, Deserialize)]
{% if has_database %}
#[derive(sqlx::FromRow)]
{% endif %}
pub struct {{ service_name_pascal }}Entity {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
```

## src/domain/repository.rs

`src/domain/repository.rs.tera` — リポジトリ trait。

```rust
use async_trait::async_trait;

use super::model::{{ service_name_pascal }}Entity;

/// {{ service_name_pascal }}Repository はデータアクセスの抽象化 trait。
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait {{ service_name_pascal }}Repository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<{{ service_name_pascal }}Entity>>;
    async fn find_all(&self) -> anyhow::Result<Vec<{{ service_name_pascal }}Entity>>;
    async fn create(&self, entity: &{{ service_name_pascal }}Entity) -> anyhow::Result<()>;
    async fn update(&self, entity: &{{ service_name_pascal }}Entity) -> anyhow::Result<()>;
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
}
```

## src/usecase/mod.rs

`src/usecase/mod.rs.tera` — ユースケースモジュール。

```rust
pub mod service;

pub use service::{{ service_name_pascal }}UseCase;
```

## src/usecase/service.rs

`src/usecase/service.rs.tera` — ユースケース実装（DI パターン）。

```rust
use std::sync::Arc;

use crate::domain::model::{{ service_name_pascal }}Entity;
{% if has_database %}
use crate::domain::repository::{{ service_name_pascal }}Repository;
{% endif %}

/// {{ service_name_pascal }}UseCase はビジネスロジックを提供する。
pub struct {{ service_name_pascal }}UseCase {
{% if has_database %}
    repo: Arc<dyn {{ service_name_pascal }}Repository>,
{% endif %}
}

impl {{ service_name_pascal }}UseCase {
    pub fn new(
{% if has_database %}
        repo: impl {{ service_name_pascal }}Repository + 'static,
{% endif %}
    ) -> Self {
        Self {
{% if has_database %}
            repo: Arc::new(repo),
{% endif %}
        }
    }

    pub async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<{{ service_name_pascal }}Entity>> {
{% if has_database %}
        self.repo.find_by_id(id).await
{% else %}
        // TODO: 実装
        Ok(None)
{% endif %}
    }

    pub async fn get_all(&self) -> anyhow::Result<Vec<{{ service_name_pascal }}Entity>> {
{% if has_database %}
        self.repo.find_all().await
{% else %}
        // TODO: 実装
        Ok(vec![])
{% endif %}
    }

    pub async fn create(&self, entity: &{{ service_name_pascal }}Entity) -> anyhow::Result<()> {
{% if has_database %}
        self.repo.create(entity).await
{% else %}
        // TODO: 実装
        Ok(())
{% endif %}
    }

    pub async fn update(&self, entity: &{{ service_name_pascal }}Entity) -> anyhow::Result<()> {
{% if has_database %}
        self.repo.update(entity).await
{% else %}
        // TODO: 実装
        Ok(())
{% endif %}
    }

    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
{% if has_database %}
        self.repo.delete(id).await
{% else %}
        // TODO: 実装
        Ok(())
{% endif %}
    }
}
```

## usecase_test のエラーケーステスト仕様（Rust）

`src/usecase/service.rs` 内の `#[cfg(test)]` モジュール — mockall を使用したエラーケーステスト。

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::Mock{{ service_name_pascal }}Repository;

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mut mock_repo = Mock{{ service_name_pascal }}Repository::new();
        mock_repo
            .expect_find_by_id()
            .with(mockall::predicate::eq("nonexistent"))
            .returning(|_| Ok(None));

        let uc = {{ service_name_pascal }}UseCase::new(mock_repo);
        let result = uc.get_by_id("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_create_duplicate() {
        let mut mock_repo = Mock{{ service_name_pascal }}Repository::new();
        mock_repo
            .expect_create()
            .returning(|_| Err(anyhow::anyhow!("duplicate entity")));

        let uc = {{ service_name_pascal }}UseCase::new(mock_repo);
        let entity = {{ service_name_pascal }}Entity {
            id: "dup-id".to_string(),
            name: "duplicate".to_string(),
            description: None,
            status: "active".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let result = uc.create(&entity).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_not_found() {
        let mut mock_repo = Mock{{ service_name_pascal }}Repository::new();
        mock_repo
            .expect_update()
            .returning(|_| Err(anyhow::anyhow!("entity not found")));

        let uc = {{ service_name_pascal }}UseCase::new(mock_repo);
        let entity = {{ service_name_pascal }}Entity {
            id: "nonexistent".to_string(),
            name: "updated".to_string(),
            description: None,
            status: "active".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let result = uc.update(&entity).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_not_found() {
        let mut mock_repo = Mock{{ service_name_pascal }}Repository::new();
        mock_repo
            .expect_delete()
            .with(mockall::predicate::eq("nonexistent"))
            .returning(|_| Err(anyhow::anyhow!("entity not found")));

        let uc = {{ service_name_pascal }}UseCase::new(mock_repo);
        let result = uc.delete("nonexistent").await;
        assert!(result.is_err());
    }
}
```

## src/adapter/mod.rs

`src/adapter/mod.rs.tera` — アダプターモジュール宣言。

```rust
pub mod handler;
```

## src/adapter/handler/ — REST (axum + utoipa)

`src/adapter/handler/rest.rs.tera` — {% if api_styles is containing("rest") %} に該当。

```rust
{% if api_styles is containing("rest") %}
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::usecase::{{ service_name_pascal }}UseCase;

/// AppHandler は REST ハンドラーを提供する。
pub struct AppHandler {
    uc: Arc<{{ service_name_pascal }}UseCase>,
}

impl AppHandler {
    pub fn new(uc: {{ service_name_pascal }}UseCase) -> Self {
        Self { uc: Arc::new(uc) }
    }

    pub fn routes(&self) -> Router {
        let uc = self.uc.clone();
        Router::new()
            .route("/api/v1/{{ service_name }}", get(list).post(create))
            .route("/api/v1/{{ service_name }}/:id", get(get_by_id))
            .with_state(uc)
    }
}

/// ErrorResponse は API設計.md D-007 準拠のエラーレスポンス。
#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
}

async fn list(
    State(uc): State<Arc<{{ service_name_pascal }}UseCase>>,
) -> Result<Json<Vec<crate::domain::model::{{ service_name_pascal }}Entity>>, StatusCode> {
    uc.get_all()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_by_id(
    State(uc): State<Arc<{{ service_name_pascal }}UseCase>>,
    Path(id): Path<String>,
) -> Result<Json<crate::domain::model::{{ service_name_pascal }}Entity>, StatusCode> {
    match uc.get_by_id(&id).await {
        Ok(Some(entity)) => Ok(Json(entity)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
struct CreateRequest {
    name: String,
    description: Option<String>,
}

async fn create(
    State(_uc): State<Arc<{{ service_name_pascal }}UseCase>>,
    Json(_req): Json<CreateRequest>,
) -> StatusCode {
    // TODO: req → entity 変換、uc.create 呼び出し
    StatusCode::CREATED
}
{% endif %}
```

## src/adapter/handler/ — gRPC (tonic)

`src/adapter/handler/grpc.rs.tera` — {% if api_styles is containing("grpc") %} に該当。

```rust
{% if api_styles is containing("grpc") %}
use tonic::{Request, Response, Status};
use std::sync::Arc;

use crate::usecase::{{ service_name_pascal }}UseCase;

pub mod proto {
    tonic::include_proto!("{{ service_name_snake }}.v1");
}

use proto::{{ service_name_snake }}_service_server::{{ service_name_pascal }}Service;
use proto::{
    Get{{ service_name_pascal }}Request, Get{{ service_name_pascal }}Response,
    List{{ service_name_pascal }}Request, List{{ service_name_pascal }}Response,
    Create{{ service_name_pascal }}Request, Create{{ service_name_pascal }}Response,
};

pub struct {{ service_name_pascal }}GrpcService {
    uc: Arc<{{ service_name_pascal }}UseCase>,
}

impl {{ service_name_pascal }}GrpcService {
    pub fn new(uc: {{ service_name_pascal }}UseCase) -> Self {
        Self { uc: Arc::new(uc) }
    }
}

#[tonic::async_trait]
impl {{ service_name_pascal }}Service for {{ service_name_pascal }}GrpcService {
    async fn get_{{ service_name_snake }}(
        &self,
        request: Request<Get{{ service_name_pascal }}Request>,
    ) -> Result<Response<Get{{ service_name_pascal }}Response>, Status> {
        let id = request.into_inner().id;
        match self.uc.get_by_id(&id).await {
            Ok(Some(entity)) => Ok(Response::new(Get{{ service_name_pascal }}Response {
                id: entity.id,
                name: entity.name,
                description: entity.description.unwrap_or_default(),
                status: entity.status,
                created_at: entity.created_at,
                updated_at: entity.updated_at,
            })),
            Ok(None) => Err(Status::not_found(format!("resource not found: {}", id))),
            Err(e) => Err(Status::internal(format!("internal error: {}", e))),
        }
    }

    async fn list_{{ service_name_snake }}(
        &self,
        _request: Request<List{{ service_name_pascal }}Request>,
    ) -> Result<Response<List{{ service_name_pascal }}Response>, Status> {
        match self.uc.get_all().await {
            Ok(entities) => {
                let items = entities.into_iter().map(|e| Get{{ service_name_pascal }}Response {
                    id: e.id,
                    name: e.name,
                    description: e.description.unwrap_or_default(),
                    status: e.status,
                    created_at: e.created_at,
                    updated_at: e.updated_at,
                }).collect();
                Ok(Response::new(List{{ service_name_pascal }}Response { items }))
            }
            Err(e) => Err(Status::internal(format!("internal error: {}", e))),
        }
    }

    async fn create_{{ service_name_snake }}(
        &self,
        _request: Request<Create{{ service_name_pascal }}Request>,
    ) -> Result<Response<Create{{ service_name_pascal }}Response>, Status> {
        // TODO: 実装
        Ok(Response::new(Create{{ service_name_pascal }}Response {
            id: "todo".to_string(),
        }))
    }
}
{% endif %}
```

## src/adapter/handler/ — GraphQL (async-graphql)

`src/adapter/handler/graphql.rs.tera` — {% if api_styles is containing("graphql") %} に該当。

```rust
{% if api_styles is containing("graphql") %}
use async_graphql::{Context, Object, Schema, EmptyMutation, EmptySubscription};
use std::sync::Arc;

use crate::domain::model::{{ service_name_pascal }}Entity;
use crate::usecase::{{ service_name_pascal }}UseCase;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn {{ service_name_snake }}(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> async_graphql::Result<Option<{{ service_name_pascal }}Entity>> {
        let uc = ctx.data::<Arc<{{ service_name_pascal }}UseCase>>()?;
        Ok(uc.get_by_id(&id).await?)
    }

    async fn {{ service_name_snake }}_list(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<{{ service_name_pascal }}Entity>> {
        let uc = ctx.data::<Arc<{{ service_name_pascal }}UseCase>>()?;
        Ok(uc.get_all().await?)
    }
}

pub type {{ service_name_pascal }}Schema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_schema(uc: {{ service_name_pascal }}UseCase) -> {{ service_name_pascal }}Schema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(Arc::new(uc))
        .finish()
}
{% endif %}
```

## src/adapter/handler/mod.rs

`src/adapter/handler/mod.rs.tera` — ハンドラーモジュール宣言。

```rust
{% if api_styles is containing("rest") %}
mod rest;
pub use rest::AppHandler;
{% elif api_styles is containing("grpc") %}
mod grpc;
pub use grpc::{{ service_name_pascal }}GrpcService as AppHandler;
{% elif api_styles is containing("graphql") %}
mod graphql;
pub use graphql::{build_schema, {{ service_name_pascal }}Schema, QueryRoot};
{% endif %}
```

## src/infra/mod.rs

`src/infra/mod.rs.tera` — インフラモジュール宣言。

```rust
pub mod config;
{% if has_database %}
pub mod persistence;
{% endif %}
{% if has_kafka %}
pub mod messaging;
{% endif %}
```

## src/infra/persistence.rs

`src/infra/persistence.rs.tera` — {% if has_database %} に該当。sqlx 接続プール。

```rust
{% if has_database %}
use sqlx::{{ database_type }}::{{ database_type | title }}Pool;
use sqlx::{{ database_type }}::{{ database_type | title }}PoolOptions;
use async_trait::async_trait;
use std::time::Duration;

use crate::domain::model::{{ service_name_pascal }}Entity;
use crate::domain::repository::{{ service_name_pascal }}Repository;
use crate::infra::config::DatabaseConfig;

pub type DbPool = {{ database_type | title }}Pool;

pub async fn create_pool(cfg: &DatabaseConfig) -> anyhow::Result<DbPool> {
    let pool = {{ database_type | title }}PoolOptions::new()
        .max_connections(cfg.pool.max_open)
        .idle_timeout(Duration::from_secs(cfg.pool.max_lifetime_sec))
        .connect(&cfg.connection_string())
        .await?;
    Ok(pool)
}

pub struct Repository {
    pool: DbPool,
}

impl Repository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl {{ service_name_pascal }}Repository for Repository {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<{{ service_name_pascal }}Entity>> {
        let entity = sqlx::query_as::<_, {{ service_name_pascal }}Entity>(
            "SELECT * FROM examples WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(entity)
    }

    async fn find_all(&self) -> anyhow::Result<Vec<{{ service_name_pascal }}Entity>> {
        let entities = sqlx::query_as::<_, {{ service_name_pascal }}Entity>(
            "SELECT * FROM examples ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(entities)
    }

    async fn create(&self, entity: &{{ service_name_pascal }}Entity) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO examples (id, name, description, status) VALUES ($1, $2, $3, $4)"
        )
        .bind(&entity.id)
        .bind(&entity.name)
        .bind(&entity.description)
        .bind(&entity.status)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, entity: &{{ service_name_pascal }}Entity) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE examples SET name = $1, description = $2, status = $3 WHERE id = $4"
        )
        .bind(&entity.name)
        .bind(&entity.description)
        .bind(&entity.status)
        .bind(&entity.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM examples WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
{% endif %}
```

## src/infra/messaging.rs

`src/infra/messaging.rs.tera` — {% if has_kafka %} に該当。[メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) D-119 準拠。

```rust
{% if has_kafka %}
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::consumer::{StreamConsumer, Consumer as _};
use rdkafka::Message;
use std::time::Duration;

use crate::infra::config::KafkaConfig;

/// Producer は Kafka プロデューサー。
/// トピック命名規則: k1s0.{tier}.{domain}.{event-type}.{version}
pub struct Producer {
    producer: FutureProducer,
}

impl Producer {
    pub fn new(cfg: &KafkaConfig) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", cfg.brokers.join(","))
            .create()
            .expect("Failed to create Kafka producer");
        Self { producer }
    }

    pub async fn publish(&self, topic: &str, key: &str, payload: &[u8]) -> anyhow::Result<()> {
        self.producer
            .send(
                FutureRecord::to(topic)
                    .key(key)
                    .payload(payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| anyhow::anyhow!("Failed to publish: {}", e))?;
        Ok(())
    }
}

/// Consumer は Kafka コンシューマー。
/// コンシューマーグループ命名規則: {service-name}.{purpose}
pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

impl KafkaConsumer {
    pub fn new(cfg: &KafkaConfig, group_id: &str, topics: &[&str]) -> Self {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", cfg.brokers.join(","))
            .set("group.id", group_id)
            .set("auto.offset.reset", "earliest")
            .create()
            .expect("Failed to create Kafka consumer");
        consumer.subscribe(topics).expect("Failed to subscribe");
        Self { consumer }
    }
}
{% endif %}
```

## src/infra/config.rs

`src/infra/config.rs.tera` — [config.md](../../cli/config/config設計.md) 準拠の設定ローダー。

```rust
use serde::Deserialize;
use std::fs;

/// Config はアプリケーション設定の全体構造。
#[derive(Debug, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
{% if has_database %}
    pub database: DatabaseConfig,
{% endif %}
{% if has_kafka %}
    pub kafka: KafkaConfig,
{% endif %}
{% if has_redis %}
    pub redis: RedisConfig,
{% endif %}
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: String,
    pub read_timeout_sec: u64,
    pub write_timeout_sec: u64,
}

{% if has_database %}
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub name: String,
{% if database_type == "postgresql" %}
    pub ssl_mode: String,
{% endif %}
    pub pool: PoolConfig,
}

#[derive(Debug, Deserialize)]
pub struct PoolConfig {
    pub max_open: u32,
    pub max_idle: u32,
    pub max_lifetime_sec: u64,
}

impl DatabaseConfig {
    pub fn connection_string(&self) -> String {
{% if database_type == "postgresql" %}
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, self.ssl_mode
        )
{% elif database_type == "mysql" %}
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.name
        )
{% elif database_type == "sqlite" %}
        self.name.clone()
{% endif %}
    }
}
{% endif %}

{% if has_kafka %}
#[derive(Debug, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub schema_registry: String,
}
{% endif %}

{% if has_redis %}
#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub addr: String,
    pub password: String,
    pub db: u32,
}
{% endif %}

#[derive(Debug, Deserialize)]
pub struct ObservabilityConfig {
    pub trace_endpoint: String,
    pub metric_endpoint: String,
    pub log_level: String,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
```

## Dockerfile

`Dockerfile.tera` — [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) 準拠。

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

COPY --from=builder /app/target/release/{{ rust_crate }} /app/server
COPY --from=builder /app/config /config

EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/app/server"]
```

---

## GraphQL スキーマ定義テンプレート

GraphQL（`api_styles is containing("graphql")`）選択時のスキーマ定義。API設計.md D-011 / D-124 に準拠する。

Rust は async-graphql のマクロベース（コードファースト）方式を採用するため、スキーマファイルは生成しない。スキーマ定義は `src/adapter/handler/graphql.rs.tera` 内のマクロアトリビュートで記述する（API設計.md D-124 参照）。

---

## buf 設定テンプレート

gRPC（`api_styles is containing("grpc")`）選択時に生成される buf 設定ファイル。API設計.md D-009 に準拠し、proto ファイルの lint・破壊的変更検出を制御する。コード生成は `tonic-build` を使用して `build.rs` から行う。

### buf.yaml

`buf.yaml.tera` — lint・breaking change 検出設定。

```yaml
{% if api_styles is containing("grpc") %}
version: v2
modules:
  - path: api/proto
lint:
  use:
    - STANDARD
breaking:
  use:
    - FILE
{% endif %}
```

### build.rs

Rust は `tonic-build` を使用して `build.rs` から proto コードを生成する。buf は lint・breaking change 検出にのみ使用し、コード生成は行わない。

`build.rs.tera` — tonic-build によるコード生成。

```rust
{% if api_styles is containing("grpc") %}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(
            &["api/proto/k1s0/{{ tier }}/{{ service_name_snake }}/v1/{{ service_name_snake }}.proto"],
            &["api/proto"],
        )?;
    Ok(())
}
{% endif %}
```

> **補足**: Rust の Cargo.toml テンプレートでは `[build-dependencies]` に `tonic-build = "0.12"` が含まれる。

---

## サーバー README テンプレート

常に生成される README ファイル。プロジェクトの概要・セットアップ手順・ディレクトリ構成を記載する。

### README.md

`README.md.tera` — サーバーの README。

```markdown
# {{ service_name }}

{{ service_name_pascal }} サーバー。

## セットアップ

```bash
# ビルド
cargo build

# 開発サーバー起動
cargo run

# テスト実行
cargo test
```

## ディレクトリ構成

```
.
├── src/
│   ├── main.rs           # エントリポイント
│   ├── domain/           # ドメインモデル・リポジトリ trait
│   ├── usecase/          # ビジネスロジック
│   ├── adapter/handler/  # ハンドラー（REST / gRPC / GraphQL）
│   └── infra/            # DB・メッセージング・設定
├── config/               # 設定ファイル
├── tests/                # 統合テスト
├── Cargo.toml
├── Dockerfile
└── README.md
```

## API

{% if api_styles is containing("rest") %}
- **方式**: REST（axum + utoipa）
{% elif api_styles is containing("grpc") %}
- **方式**: gRPC（tonic）
{% elif api_styles is containing("graphql") %}
- **方式**: GraphQL（async-graphql）
{% endif %}

## 設定

`config/config.yaml` で接続先を管理する。環境別の上書きは `config/config.dev.yaml` を参照。
```

---

## サーバーテストファイルテンプレート

常に生成されるテストファイル。コーディング規約.md のテストツール選定に準拠する。

- **Rust**: mockall（モック自動生成）+ tokio test-util（async テスト）

### tests/integration_test.rs

`tests/integration_test.rs.tera` — サーバー統合テスト。mockall でリポジトリをモック化し、axum ハンドラーをテストする。

```rust
{% if api_styles is containing("rest") %}
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use {{ rust_crate }}::adapter::handler::rest::AppHandler;
use {{ rust_crate }}::usecase::{{ service_name_pascal }}UseCase;
{% if has_database %}
use {{ rust_crate }}::domain::repository::Mock{{ service_name_pascal }}Repository;
use {{ rust_crate }}::domain::model::{{ service_name_pascal }}Entity;
{% endif %}

#[tokio::test]
async fn test_list_returns_ok() {
{% if has_database %}
    let mut mock_repo = Mock{{ service_name_pascal }}Repository::new();
    mock_repo.expect_find_all()
        .returning(|| Ok(vec![]));
    let uc = {{ service_name_pascal }}UseCase::new(mock_repo);
{% else %}
    let uc = {{ service_name_pascal }}UseCase::new();
{% endif %}
    let handler = AppHandler::new(uc);
    let app = handler.routes();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/{{ service_name }}")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_by_id_not_found() {
{% if has_database %}
    let mut mock_repo = Mock{{ service_name_pascal }}Repository::new();
    mock_repo.expect_find_by_id()
        .returning(|_| Ok(None));
    let uc = {{ service_name_pascal }}UseCase::new(mock_repo);
{% else %}
    let uc = {{ service_name_pascal }}UseCase::new();
{% endif %}
    let handler = AppHandler::new(uc);
    let app = handler.routes();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/{{ service_name }}/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
{% endif %}
{% if api_styles is containing("grpc") %}
use {{ rust_crate }}::usecase::{{ service_name_pascal }}UseCase;

#[tokio::test]
async fn test_grpc_placeholder() {
    // TODO: tonic テストクライアントを使用した統合テストを実装する
    let _uc = {{ service_name_pascal }}UseCase::new(
{% if has_database %}
        {{ rust_crate }}::domain::repository::Mock{{ service_name_pascal }}Repository::new(),
{% endif %}
    );
    assert!(true);
}
{% endif %}
{% if api_styles is containing("graphql") %}
use {{ rust_crate }}::adapter::handler::graphql::build_schema;
use {{ rust_crate }}::usecase::{{ service_name_pascal }}UseCase;

#[tokio::test]
async fn test_graphql_schema_creation() {
    let uc = {{ service_name_pascal }}UseCase::new(
{% if has_database %}
        {{ rust_crate }}::domain::repository::Mock{{ service_name_pascal }}Repository::new(),
{% endif %}
    );
    let schema = build_schema(uc);
    let result = schema.execute("{ __typename }").await;
    assert!(result.errors.is_empty());
}
{% endif %}
```

---

## 既存ドキュメント参照マップ

各テンプレートファイルが準拠する既存ドキュメントの設計パターン。

| テンプレートファイル        | 参照ドキュメント                               | 該当セクション                                |
| --------------------------- | ---------------------------------------------- | --------------------------------------------- |
| config/config.yaml          | [config.md](../../cli/config/config設計.md)                 | YAML スキーマ定義                             |
| adapter/handler (REST)      | [API設計.md](../../architecture/api/API設計.md)                       | D-007 エラーレスポンス、D-008 バージョニング  |
| adapter/handler (gRPC)      | [API設計.md](../../architecture/api/API設計.md)                       | D-009 gRPC 定義パターン、D-010 バージョニング |
| adapter/handler (GraphQL)   | [API設計.md](../../architecture/api/API設計.md)                       | D-011 GraphQL 設計、D-124 実装技術選定        |
| api/openapi/                | [API設計.md](../../architecture/api/API設計.md)                       | D-123 oapi-codegen / utoipa                   |
| Dockerfile                  | [Dockerイメージ戦略.md](../../infrastructure/docker/Dockerイメージ戦略.md) | ベースイメージ・マルチステージビルド          |
| infra/messaging             | [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) | D-119 Kafka トピック、D-120 イベント駆動      |
| OTel 初期化                 | [可観測性設計.md](../../architecture/observability/可観測性設計.md)             | D-110 分散トレーシング                        |
| 構造化ログ (tracing)        | [可観測性設計.md](../../architecture/observability/可観測性設計.md)             | D-109 構造化ログ                              |
| domain/repository           | [コーディング規約.md](../../architecture/conventions/コーディング規約.md)     | モック生成（mockall）                         |
| buf.yaml                    | [API設計.md](../../architecture/api/API設計.md)                       | D-009 gRPC 定義、buf lint                    |
| build.rs（Rust gRPC）       | [API設計.md](../../architecture/api/API設計.md)                       | D-009 tonic-build によるコード生成           |
| テストファイル（*_test.rs） | [コーディング規約.md](../../architecture/conventions/コーディング規約.md)     | テストツール（mockall）                       |
| README.md                   | ---                                              | プロジェクト概要・セットアップ手順           |
| BFF テンプレート            | [API設計.md](../../architecture/api/API設計.md)                       | D-011 GraphQL 設計、BFF パターン             |

## BFF (Backend for Frontend) テンプレート

BFF テンプレートの詳細は [テンプレート仕様-BFF](../client/BFF.md) を参照。

BFF テンプレートは以下の条件で生成される:

| 条件 | 値 |
|---|---|
| Tier | `service` |
| API 方式 | `graphql` を含む |
| BFF 言語 | CLI 対話フローで Go / Rust を選択 |

### 生成ロジック

`execute.rs` の `execute_generate_at()` / `execute_generate_with_config()` において、以下の条件で BFF ディレクトリが追加生成される:

```rust
if config.kind == Kind::Server
    && config.tier == Tier::Service
    && config.detail.api_styles.contains(&ApiStyle::GraphQL)
{
    let bff_path = output_path.join("bff");
    fs::create_dir_all(&bff_path)?;
}
```

`bff_language` が `None` の場合でも `service + GraphQL` の条件を満たせば空の `bff/` ディレクトリが作成される（後方互換性維持）。`bff_language` が `Some(Language::Go)` または `Some(Language::Rust)` の場合は、対応する言語のスケルトンコードが生成される。

---

## TODO スタブ実装方針

### 概要

テンプレートから生成されるスケルトンコードには、未実装箇所を示す TODO スタブが含まれる。本セクションでは、スタブの記述方針と各言語のパターンを定義する。

### スケルトンコードの原則

1. **コンパイル可能であること** — 生成直後に `cargo build` が成功すること
2. **テスト可能であること** — 最低限のインターフェース実装が存在し、テストランナーが実行可能であること
3. **最小限の依存のみ** — 未使用の依存は含めない。必要な依存は `Cargo.toml` に記載済みであること

### Rust のスタブパターン

Rust では、コンパイル可能性を優先し、以下の段階的なスタブパターンを使用する。

| パターン | 用途 | コンパイル | 実行時 |
|---|---|---|---|
| `todo!()` | 未実装を明示（開発初期） | 可能 | パニック |
| `unimplemented!()` | 意図的に未実装（インターフェース実装時） | 可能 | パニック |
| 最小実装 | テスト可能なスタブ（テンプレートデフォルト） | 可能 | 正常動作 |

テンプレートでは **最小実装** パターンをデフォルトとする。これにより、生成直後にテストが通る状態を保証する。

```rust
pub async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<OrderEntity>> {
    // TODO: 実装
    Ok(None)
}

pub async fn get_all(&self) -> anyhow::Result<Vec<OrderEntity>> {
    // TODO: 実装
    Ok(vec![])
}

pub async fn create(&self, entity: &OrderEntity) -> anyhow::Result<()> {
    // TODO: 実装
    Ok(())
}
```

gRPC ハンドラーのスタブ:

```rust
async fn create_order(
    &self,
    _request: Request<CreateOrderRequest>,
) -> Result<Response<CreateOrderResponse>, Status> {
    // TODO: 実装
    Ok(Response::new(CreateOrderResponse {
        id: "todo".to_string(),
    }))
}
```

### TODO コメント規約

- すべての TODO スタブには `// TODO: ` プレフィックスを付与する
- grep / IDE で一括検索可能な統一フォーマットとする
- 具体的な実装内容を示す場合は `// TODO: req → entity 変換、uc.Create 呼び出し` のように記述する
- テストの TODO は `// TODO: gRPC クライアントを使用した統合テストを実装する` のように次のアクションを明示する

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [テンプレート仕様-サーバー](サーバー.md) --- 概要・条件付き生成表・Tier別配置パス
- [テンプレート仕様-サーバー-可観測性](サーバー-可観測性.md) --- 可観測性テンプレート
- [テンプレート仕様-サーバー-認証](サーバー-認証.md) --- 認証認可Middleware テンプレート
- [テンプレート仕様-サーバー-gRPC](サーバー-gRPC.md) --- gRPC Health Check・GraphQL Subscription
- [API設計](../../architecture/api/API設計.md) --- REST / gRPC / GraphQL 設計
- [config設計](../../cli/config/config設計.md) --- config.yaml スキーマと環境別管理
- [可観測性設計](../../architecture/observability/可観測性設計.md) --- 監視・トレーシング設計
- [認証認可設計](../../architecture/auth/認証認可設計.md) --- 認証・認可・シークレット管理
- [Dockerイメージ戦略](../../infrastructure/docker/Dockerイメージ戦略.md) --- Docker ビルド戦略
- [メッセージング設計](../../architecture/messaging/メッセージング設計.md) --- Kafka トピック・イベント駆動設計
