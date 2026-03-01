# system-graphql-gateway 実装設計

概要・API 定義・アーキテクチャは [system-graphql-gateway設計.md](server.md) を参照。

---

## ディレクトリ構成

```
regions/system/server/rust/graphql-gateway/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── model/
│   │   │   ├── mod.rs
│   │   │   ├── tenant.rs          # Tenant, TenantStatus, TenantConnection
│   │   │   ├── feature_flag.rs    # FeatureFlag
│   │   │   ├── config_entry.rs    # ConfigEntry
│   │   │   └── graphql_context.rs # GraphqlContext (user_id, roles, request_id)
│   │   └── loader/
│   │       └── mod.rs             # DataLoader trait 定義
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── tenant_query.rs        # TenantQueryResolver
│   │   ├── feature_flag_query.rs  # FeatureFlagQueryResolver
│   │   ├── config_query.rs        # ConfigQueryResolver
│   │   ├── tenant_mutation.rs     # TenantMutationResolver
│   │   └── subscription.rs        # SubscriptionResolver
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── graphql_handler.rs     # POST/GET /graphql, /graphql/ws
│   │   └── middleware/
│   │       ├── mod.rs
│   │       └── auth_middleware.rs  # JWT 検証 axum layer
│   └── infra/
│       ├── mod.rs
│       ├── config/
│       │   └── mod.rs             # Config struct
│       ├── grpc/
│       │   ├── mod.rs
│       │   ├── tenant_client.rs   # TenantGrpcClient
│       │   ├── feature_flag_client.rs
│       │   └── config_client.rs
│       └── auth/
│           └── jwks.rs            # JWKS 取得・JWT 検証
├── api/
│   └── graphql/
│       └── schema.graphql
├── config/
│   ├── config.yaml
│   ├── config.dev.yaml
│   ├── config.staging.yaml
│   └── config.prod.yaml
└── build.rs
```

---

## 依存クレート

> 共通依存は [Rust共通実装.md](../_common/Rust共通実装.md#共通cargo依存) を参照。

| クレート | バージョン | 用途 |
| --- | --- | --- |
| `axum` | 0.7 | HTTP フレームワーク（`macros`, `ws` feature） |
| `async-graphql` | 7 | GraphQL サーバー（`dataloader` feature） |
| `async-graphql-axum` | 7 | axum 統合 |
| `jsonwebtoken` | 9 | JWT 検証 |
| `reqwest` | 0.12 | JWKS 取得用 HTTP クライアント（`json`, `rustls-tls` feature） |
| `async-trait` | 0.1 | 非同期トレイト |
| `k1s0-telemetry` | path | テレメトリ（`full` feature） |
| `axum-test` | 16 | テスト（dev-dependency） |

### build.rs

gRPC クライアント側のため `build_server(false)` / `build_client(true)`。proto パス: `tenant.proto`, `featureflag.proto`, `config.proto`。

---

## テスト構成

| レイヤー | テスト種別 | 手法 |
| --- | --- | --- |
| domain/model | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/graphql_handler | 統合テスト（HTTP） | `axum-test` + `tokio::test` |
| adapter/middleware | 単体テスト | `tokio::test` + モック JWT |
| infra/auth | 単体テスト | `tokio::test` + `wiremock` |
| infra/grpc | 統合テスト | `tonic` mock + `tokio::test` |

---

## Cargo.toml

> 共通依存は [Rust共通実装.md](../_common/Rust共通実装.md#共通cargo依存) を参照。サービス固有の追加依存:

```toml
# axum に WebSocket サポートを追加
axum = { version = "0.7", features = ["macros", "ws"] }

# GraphQL
async-graphql = { version = "7", features = ["dataloader"] }
async-graphql-axum = "7"

# JWT 検証
jsonwebtoken = "9"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# 非同期トレイト
async-trait = "0.1"

# 内部ライブラリ
k1s0-telemetry = { path = "../../../library/rust/telemetry", features = ["full"] }

[dev-dependencies]
axum-test = "16"
```

---

## src/main.rs

> 起動シーケンスは [Rust共通実装.md](../_common/Rust共通実装.md#共通mainrs) を参照。graphql-gateway は DB/Kafka を使用せず、REST サーバーのみ起動する。以下はサービス固有の DI:

```rust
    // --- gRPC クライアント ---
    let tenant_client = Arc::new(
        TenantGrpcClient::connect(&cfg.backends.tenant).await?,
    );
    let feature_flag_client = Arc::new(
        FeatureFlagGrpcClient::connect(&cfg.backends.featureflag).await?,
    );
    let config_client = Arc::new(
        ConfigGrpcClient::connect(&cfg.backends.config).await?,
    );

    // --- JWT 検証 ---
    let jwks_verifier = Arc::new(
        JwksVerifier::new(cfg.auth.jwks_url.clone()),
    );

    // --- Resolver DI ---
    let tenant_query = Arc::new(TenantQueryResolver::new(tenant_client.clone()));
    let feature_flag_query = Arc::new(FeatureFlagQueryResolver::new(feature_flag_client.clone()));
    let config_query = Arc::new(ConfigQueryResolver::new(config_client.clone()));
    let tenant_mutation = Arc::new(TenantMutationResolver::new(tenant_client.clone()));
    let subscription = Arc::new(SubscriptionResolver::new(config_client.clone()));

    // --- Router ---
    let app = graphql_handler::router(
        jwks_verifier,
        tenant_query,
        feature_flag_query,
        config_query,
        tenant_mutation,
        subscription,
        cfg.graphql.clone(),
    );
```

---

## ドメインモデル実装

### src/domain/model/tenant.rs

```rust
use async_graphql::{Enum, Object, SimpleObject};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, SimpleObject)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub status: TenantStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum TenantStatus {
    Active,
    Suspended,
    Deleted,
}

impl From<String> for TenantStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "ACTIVE"    => TenantStatus::Active,
            "SUSPENDED" => TenantStatus::Suspended,
            "DELETED"   => TenantStatus::Deleted,
            _           => TenantStatus::Active,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TenantConnection {
    pub edges: Vec<TenantEdge>,
    pub page_info: PageInfo,
    pub total_count: i32,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct TenantEdge {
    pub node: Tenant,
    pub cursor: String,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
}
```

### src/domain/model/feature_flag.rs

```rust
use async_graphql::SimpleObject;

#[derive(Debug, Clone, SimpleObject)]
pub struct FeatureFlag {
    pub key: String,
    pub name: String,
    pub enabled: bool,
    pub rollout_percentage: i32,
    pub target_environments: Vec<String>,
}
```

### src/domain/model/config_entry.rs

```rust
use async_graphql::SimpleObject;

#[derive(Debug, Clone, SimpleObject)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}
```

### src/domain/model/payload.rs

```rust
use async_graphql::SimpleObject;

use crate::domain::model::{FeatureFlag, Tenant};

#[derive(Debug, Clone, SimpleObject)]
pub struct CreateTenantPayload {
    pub tenant: Option<Tenant>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UpdateTenantPayload {
    pub tenant: Option<Tenant>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct SetFeatureFlagPayload {
    pub feature_flag: Option<FeatureFlag>,
    pub errors: Vec<UserError>,
}

#[derive(Debug, Clone, SimpleObject)]
pub struct UserError {
    pub field: Option<Vec<String>>,
    pub message: String,
}
```

### src/domain/model/graphql_context.rs

```rust
use async_graphql::dataloader::DataLoader;

use crate::infra::grpc::{ConfigGrpcClient, FeatureFlagGrpcClient, TenantGrpcClient};

/// GraphQL リクエストコンテキスト。JWT から抽出した認証情報と DataLoader を保持する。
pub struct GraphqlContext {
    /// JWT sub クレームから取得したユーザー ID
    pub user_id: String,
    /// JWT realm_access.roles から取得したロールリスト
    pub roles: Vec<String>,
    /// リクエスト追跡 ID（X-Request-Id ヘッダーまたは UUID 自動生成）
    pub request_id: String,
    /// テナントバッチローダー
    pub tenant_loader: DataLoader<TenantLoader>,
    /// フィーチャーフラグバッチローダー
    pub flag_loader: DataLoader<FeatureFlagLoader>,
}

pub struct TenantLoader {
    pub client: std::sync::Arc<TenantGrpcClient>,
}

pub struct FeatureFlagLoader {
    pub client: std::sync::Arc<FeatureFlagGrpcClient>,
}

pub struct ConfigLoader {
    pub client: std::sync::Arc<ConfigGrpcClient>,
}
```

### src/domain/loader/mod.rs

```rust
use async_graphql::dataloader::Loader;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::model::{FeatureFlag, Tenant};
use crate::domain::model::graphql_context::{FeatureFlagLoader, TenantLoader};

/// TenantLoader は ID リストを受け取り、TenantService.BatchGetTenants を呼び出してバッチ取得する。
#[async_trait]
impl Loader<String> for TenantLoader {
    type Value = Tenant;
    type Error = anyhow::Error;

    async fn load(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, Self::Value>, Self::Error> {
        // TenantService.ListTenants を呼び出して ID フィルタリング
        let tenants = self.client.list_tenants_by_ids(keys).await?;
        Ok(tenants.into_iter().map(|t| (t.id.clone(), t)).collect())
    }
}

/// FeatureFlagLoader はフラグキーリストを受け取り、FeatureFlagService.ListFlags を呼び出してバッチ取得する。
#[async_trait]
impl Loader<String> for FeatureFlagLoader {
    type Value = FeatureFlag;
    type Error = anyhow::Error;

    async fn load(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let flags = self.client.list_flags_by_keys(keys).await?;
        Ok(flags.into_iter().map(|f| (f.key.clone(), f)).collect())
    }
}
```

---

## ユースケース実装

### src/usecase/tenant_query.rs

```rust
use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::{Tenant, TenantConnection};
use crate::infra::grpc::TenantGrpcClient;

pub struct TenantQueryResolver {
    client: Arc<TenantGrpcClient>,
}

impl TenantQueryResolver {
    pub fn new(client: Arc<TenantGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_tenant(&self, id: &str) -> anyhow::Result<Option<Tenant>> {
        self.client.get_tenant(id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_tenants(
        &self,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> anyhow::Result<TenantConnection> {
        self.client.list_tenants(first, after, last, before).await
    }
}
```

### src/usecase/feature_flag_query.rs

```rust
use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::FeatureFlag;
use crate::infra::grpc::FeatureFlagGrpcClient;

pub struct FeatureFlagQueryResolver {
    client: Arc<FeatureFlagGrpcClient>,
}

impl FeatureFlagQueryResolver {
    pub fn new(client: Arc<FeatureFlagGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_feature_flag(&self, key: &str) -> anyhow::Result<Option<FeatureFlag>> {
        self.client.get_flag(key).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_feature_flags(
        &self,
        environment: Option<&str>,
    ) -> anyhow::Result<Vec<FeatureFlag>> {
        self.client.list_flags(environment).await
    }
}
```

### src/usecase/config_query.rs

```rust
use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::ConfigEntry;
use crate::infra::grpc::ConfigGrpcClient;

pub struct ConfigQueryResolver {
    client: Arc<ConfigGrpcClient>,
}

impl ConfigQueryResolver {
    pub fn new(client: Arc<ConfigGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_config(&self, key: &str) -> anyhow::Result<Option<ConfigEntry>> {
        // key は "namespace/key" 形式を想定
        let parts: Vec<&str> = key.splitn(2, '/').collect();
        if parts.len() != 2 {
            return Ok(None);
        }
        self.client.get_config(parts[0], parts[1]).await
    }
}
```

### src/usecase/tenant_mutation.rs

```rust
use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::Tenant;
use crate::infra::grpc::TenantGrpcClient;

pub struct TenantMutationResolver {
    client: Arc<TenantGrpcClient>,
}

impl TenantMutationResolver {
    pub fn new(client: Arc<TenantGrpcClient>) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_tenant(
        &self,
        name: &str,
        owner_user_id: &str,
    ) -> anyhow::Result<CreateTenantPayload> {
        match self.client.create_tenant(name, owner_user_id).await {
            Ok(tenant) => Ok(CreateTenantPayload {
                tenant: Some(tenant),
                errors: vec![],
            }),
            Err(e) => Ok(CreateTenantPayload {
                tenant: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            }),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_tenant(
        &self,
        id: &str,
        name: Option<&str>,
        status: Option<&str>,
    ) -> anyhow::Result<UpdateTenantPayload> {
        match self.client.update_tenant(id, name, status).await {
            Ok(tenant) => Ok(UpdateTenantPayload {
                tenant: Some(tenant),
                errors: vec![],
            }),
            Err(e) => Ok(UpdateTenantPayload {
                tenant: None,
                errors: vec![UserError {
                    field: None,
                    message: e.to_string(),
                }],
            }),
        }
    }
}
```

### src/usecase/subscription.rs

```rust
use std::sync::Arc;

use async_graphql::futures_util::Stream;
use tracing::instrument;

use crate::domain::model::ConfigEntry;
use crate::infra::grpc::ConfigGrpcClient;

pub struct SubscriptionResolver {
    config_client: Arc<ConfigGrpcClient>,
}

impl SubscriptionResolver {
    pub fn new(config_client: Arc<ConfigGrpcClient>) -> Self {
        Self { config_client }
    }

    /// WatchConfig ストリームを返す。設定変更が発生するたびに ConfigEntry を配信する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_config(
        &self,
        namespaces: Vec<String>,
    ) -> impl Stream<Item = ConfigEntry> {
        self.config_client.watch_config(namespaces).await
    }
}
```

---

## アダプター実装

### src/adapter/graphql_handler.rs

```rust
use std::sync::Arc;

use async_graphql::{
    dataloader::DataLoader, EmptySubscription, MergedObject, Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Json, Router,
};

use crate::adapter::middleware::auth_middleware::{auth_layer, Claims};
use crate::domain::model::graphql_context::{FeatureFlagLoader, GraphqlContext, TenantLoader};
use crate::infra::config::GraphQLConfig;
use crate::usecase::{
    ConfigQueryResolver, FeatureFlagQueryResolver, SubscriptionResolver,
    TenantMutationResolver, TenantQueryResolver,
};

pub type AppSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn router(
    jwks_verifier: Arc<crate::infra::auth::JwksVerifier>,
    tenant_query: Arc<TenantQueryResolver>,
    feature_flag_query: Arc<FeatureFlagQueryResolver>,
    config_query: Arc<ConfigQueryResolver>,
    tenant_mutation: Arc<TenantMutationResolver>,
    subscription: Arc<SubscriptionResolver>,
    graphql_cfg: GraphQLConfig,
) -> Router {
    let schema = Schema::build(
        QueryRoot {
            tenant_query: tenant_query.clone(),
            feature_flag_query: feature_flag_query.clone(),
            config_query: config_query.clone(),
        },
        MutationRoot {
            tenant_mutation: tenant_mutation.clone(),
        },
        SubscriptionRoot {
            subscription: subscription.clone(),
        },
    )
    .limit_depth(graphql_cfg.max_depth as usize)
    .limit_complexity(graphql_cfg.max_complexity as usize)
    .finish();

    let query_timeout = std::time::Duration::from_secs(graphql_cfg.query_timeout_seconds as u64);

    let mut router = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics_handler))
        .route(
            "/graphql",
            post(graphql_handler).layer(auth_layer(jwks_verifier.clone())),
        )
        .route(
            "/graphql/ws",
            get(graphql_ws_handler).layer(auth_layer(jwks_verifier)),
        )
        .with_state(schema.clone());

    // 開発環境のみ Playground を有効化
    if graphql_cfg.playground {
        router = router.route("/graphql", get(graphql_playground));
    }

    router
}

async fn graphql_handler(
    State((schema, timeout)): State<(AppSchema, std::time::Duration)>,
    Extension(claims): Extension<Claims>,
    req: GraphQLRequest,
) -> impl IntoResponse {
    let request = req.into_inner().data(claims);
    match tokio::time::timeout(timeout, schema.execute(request)).await {
        Ok(resp) => GraphQLResponse::from(resp).into_response(),
        Err(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "data": null,
                "errors": [{
                    "message": "query execution timed out",
                    "extensions": { "code": "TIMEOUT" }
                }]
            })),
        )
            .into_response(),
    }
}

async fn graphql_ws_handler(
    State(schema): State<AppSchema>,
    ws: axum::extract::WebSocketUpgrade,
) -> impl IntoResponse {
    GraphQLSubscription::new(schema).on_upgrade(ws)
}

async fn graphql_playground() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql")
            .subscription_endpoint("/graphql/ws"),
    ))
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn readyz() -> Json<serde_json::Value> {
    // バックエンド gRPC 疎通確認は実装時に追加
    Json(serde_json::json!({"status": "ready"}))
}

async fn metrics_handler() -> impl IntoResponse {
    use prometheus::{Encoder, TextEncoder};
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    (StatusCode::OK, buffer)
}
```

### src/adapter/middleware/auth_middleware.rs

```rust
use std::sync::Arc;

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tower::Layer;

use crate::infra::auth::JwksVerifier;

/// JWT Claims（async-graphql の Extension として GraphQL Context に注入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub preferred_username: Option<String>,
    pub email: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub exp: i64,
}

impl Claims {
    pub fn roles(&self) -> Vec<String> {
        self.realm_access
            .as_ref()
            .map(|r| r.roles.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

/// auth_layer は Authorization ヘッダーの Bearer JWT を検証する axum ミドルウェアレイヤーを返す。
pub fn auth_layer(
    verifier: Arc<JwksVerifier>,
) -> axum::middleware::FromFnLayer<
    impl Fn(Request, Next) -> impl std::future::Future<Output = Response> + Send + Clone,
    (),
    (Arc<JwksVerifier>,),
> {
    let verifier = verifier.clone();
    axum::middleware::from_fn(move |req: Request, next: Next| {
        let verifier = verifier.clone();
        async move { verify_jwt(verifier, req, next).await }
    })
}

async fn verify_jwt(
    verifier: Arc<JwksVerifier>,
    mut req: Request,
    next: Next,
) -> Response {
    let token = extract_bearer_token(req.headers());

    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "missing Authorization header"
                    }
                })),
            )
                .into_response();
        }
    };

    let claims = match verifier.verify_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "invalid or expired JWT token"
                    }
                })),
            )
                .into_response();
        }
    };

    req.extensions_mut().insert(claims);
    next.run(req).await
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.to_owned())
}
```

---

## インフラ実装

### src/infra/config/mod.rs

```rust
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub graphql: GraphQLConfig,
    pub auth: AuthConfig,
    pub backends: BackendsConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GraphQLConfig {
    /// スキーマイントロスペクションを有効化するか（development のみ true 推奨）
    pub introspection: bool,
    /// GraphQL Playground を有効化するか（development のみ true 推奨）
    pub playground: bool,
    /// クエリネスト深度の上限
    pub max_depth: u32,
    /// クエリ複雑度の上限
    pub max_complexity: u32,
    /// クエリ実行タイムアウト（秒）
    pub query_timeout_seconds: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// JWKS エンドポイント URL
    pub jwks_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackendsConfig {
    pub tenant: BackendConfig,
    pub featureflag: BackendConfig,
    pub config: BackendConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfig {
    /// gRPC エンドポイント（例: "http://tenant-server.k1s0-system.svc.cluster.local:50051"）
    pub address: String,
    /// リクエストタイムアウト（ミリ秒）
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    pub log: LogConfig,
    pub trace: TraceConfig,
    pub metrics: MetricsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TraceConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub sample_rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub path: String,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| path.to_owned());
        let content = fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("failed to read config file {}: {}", path, e))?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.app.name.is_empty() {
            anyhow::bail!("app.name is required");
        }
        if self.server.port == 0 {
            anyhow::bail!("server.port must be > 0");
        }
        if self.auth.jwks_url.is_empty() {
            anyhow::bail!("auth.jwks_url is required");
        }
        if self.backends.tenant.address.is_empty() {
            anyhow::bail!("backends.tenant.address is required");
        }
        Ok(())
    }
}
```

### src/infra/auth/jwks.rs

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::adapter::middleware::auth_middleware::Claims;

/// JwksVerifier は JWKS エンドポイントから公開鍵を取得し、JWT の署名を検証する。
pub struct JwksVerifier {
    jwks_url: String,
    http_client: Client,
    cache: Arc<RwLock<Option<CachedJwks>>>,
    cache_ttl: Duration,
}

// ... (省略: CachedJwks, JwksResponse, Jwk 構造体は上記 guide 内容参照)

impl JwksVerifier {
    pub fn new(jwks_url: String) -> Self {
        Self {
            jwks_url,
            http_client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            cache: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::from_secs(600), // 10 分
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn verify_token(&self, token: &str) -> anyhow::Result<Claims> {
        let keys = self.get_jwks().await?;
        let header = decode_header(token)
            .map_err(|e| anyhow::anyhow!("invalid JWT header: {}", e))?;

        // kid でマッチする鍵を選択。kid が無い場合は最初の RSA 鍵を使用
        let jwk = match &header.kid {
            Some(kid) => keys.iter().find(|k| k.kid.as_deref() == Some(kid)),
            None => keys.iter().find(|k| k.kty == "RSA"),
        }
        .ok_or_else(|| anyhow::anyhow!("no matching JWK found"))?;

        // RSA 公開鍵で JWT を検証
        // ...（verify_token の実装詳細は JwksVerifier 内部）
        todo!()
    }
}
```

### src/infra/grpc/tenant_client.rs

gRPC クライアント実装（TenantService、FeatureFlagService、ConfigService）の詳細は guide から統合済み。各クライアントは `tonic::transport::Channel` で接続し、Domain モデルに変換する。

---

## 設定ファイル例

> 共通セクション（app/server/observability）は [Rust共通実装.md](../_common/Rust共通実装.md#共通configyaml) を参照。graphql-gateway は database/kafka を使用しない。サービス固有セクション:

```yaml
graphql:
  introspection: false
  playground: false
  max_depth: 10
  max_complexity: 1000
  query_timeout_seconds: 30

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local/jwks"

backends:
  tenant:
        address: "http://tenant-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  featureflag:
        address: "http://featureflag-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  config:
        address: "http://config-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
```

```yaml
# config/config.dev.yaml
app:
  environment: "development"

graphql:
  introspection: true
  playground: true

observability:
  log:
    level: "debug"
    format: "text"
  trace:
    sample_rate: 1.0
```

---

## テスト例

### ユースケーステスト

```rust
// src/usecase/tenant_query.rs
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::eq;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_tenant_found() {
        let mut mock_client = MockTenantGrpcClient::new();
        mock_client
            .expect_get_tenant()
            .with(eq("tenant-123"))
            .returning(|_| {
                Ok(Some(Tenant {
                    id: "tenant-123".to_owned(),
                    name: "テスト株式会社".to_owned(),
                    status: TenantStatus::Active,
                    created_at: "2026-01-01T00:00:00".to_owned(),
                    updated_at: String::new(),
                }))
            });

        let resolver = TenantQueryResolver::new(Arc::new(mock_client));
        let result = resolver.get_tenant("tenant-123").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "テスト株式会社");
    }

    #[tokio::test]
    async fn test_get_tenant_not_found() {
        let mut mock_client = MockTenantGrpcClient::new();
        mock_client
            .expect_get_tenant()
            .with(eq("unknown"))
            .returning(|_| Ok(None));

        let resolver = TenantQueryResolver::new(Arc::new(mock_client));
        let result = resolver.get_tenant("unknown").await.unwrap();
        assert!(result.is_none());
    }
}
```

### GraphQL ハンドラーテスト

```rust
// tests/graphql_handler_test.rs
#[cfg(test)]
mod tests {
    use axum_test::TestServer;
    use serde_json::json;

    #[tokio::test]
    async fn test_graphql_query_without_jwt_returns_401() {
        let app = build_test_app();
        let server = TestServer::new(app).unwrap();

        let response = server
            .post("/graphql")
            .json(&json!({
                "query": "query { tenant(id: \"test\") { id name } }"
            }))
            .await;

        assert_eq!(response.status_code(), 401);
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = build_test_app();
        let server = TestServer::new(app).unwrap();

        let response = server.get("/healthz").await;

        assert_eq!(response.status_code(), 200);
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "ok");
    }
}
```

---

## 関連ドキュメント

- [system-graphql-gateway設計.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-graphql-gateway-deploy.md](deploy.md) -- Dockerfile・Helm values・デプロイ設計
- [proto設計.md](../../architecture/api/proto設計.md) -- ConfigService / TenantService / FeatureFlagService proto 定義
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- JWT Claims 構造・RBAC ロール定義
- [GraphQL設計.md](../../architecture/api/GraphQL設計.md) -- GraphQL 設計ガイドライン
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート仕様
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- コーディング規約
