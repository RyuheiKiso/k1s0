# system-graphql-gateway 実装設計

system-graphql-gateway の Rust 実装詳細を定義する。概要・API 定義・アーキテクチャは [system-graphql-gateway設計.md](system-graphql-gateway設計.md) を参照。

---

## Rust 実装 (regions/system/server/rust/graphql-gateway/)

### ディレクトリ構成

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

### Cargo.toml

既存の `Cargo.toml` に以下の依存関係を追加する。

```toml
[package]
name = "k1s0-graphql-gateway-server"
version = "0.1.0"
edition = "2021"

[lib]
name = "k1s0_graphql_gateway_server"
path = "src/lib.rs"

[[bin]]
name = "k1s0-graphql-gateway-server"
path = "src/main.rs"

[dependencies]
# Web フレームワーク
axum = { version = "0.7", features = ["macros", "ws"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }

# gRPC
tonic = "0.12"
prost = "0.13"
prost-types = "0.13"

# GraphQL
async-graphql = { version = "7", features = ["dataloader"] }
async-graphql-axum = "7"

# JWT 検証
jsonwebtoken = "9"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# シリアライゼーション
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# OpenTelemetry
opentelemetry = "0.27"
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.27", features = ["tonic"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
tracing-opentelemetry = "0.28"

# ユーティリティ
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
async-trait = "0.1"

# メトリクス
prometheus = "0.13"

# 内部ライブラリ
k1s0-telemetry = { path = "../../../library/rust/telemetry", features = ["full"] }

[build-dependencies]
tonic-build = "0.12"

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
axum-test = "16"
```

### build.rs

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &[
                "api/proto/k1s0/system/tenant/v1/tenant.proto",
                "api/proto/k1s0/system/featureflag/v1/featureflag.proto",
                "api/proto/k1s0/system/config/v1/config.proto",
            ],
            &["api/proto/", "../../../../../../api/proto/"],
        )?;
    Ok(())
}
```

### src/main.rs

```rust
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::signal;
use tracing::info;

mod adapter;
mod domain;
mod infra;
mod usecase;

use adapter::graphql_handler;
use infra::auth::JwksVerifier;
use infra::config::Config;
use infra::grpc::{ConfigGrpcClient, FeatureFlagGrpcClient, TenantGrpcClient};
use usecase::{
    ConfigQueryResolver, FeatureFlagQueryResolver, SubscriptionResolver,
    TenantMutationResolver, TenantQueryResolver,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- Config ---
    let cfg = Config::load("config/config.yaml")?;
    cfg.validate()?;

    // --- Logger ---
    infra::config::init_logger(&cfg.app.environment);

    // --- OpenTelemetry ---
    let _tracer = infra::config::init_tracer(&cfg.app.name)?;

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

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    info!("graphql-gateway starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("graphql-gateway exited");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("shutdown signal received");
}
```

---

## ドメインモデル（Rust）

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
    pub nodes: Vec<Tenant>,
    pub total_count: i32,
    pub has_next: bool,
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

## ユースケース（Rust）

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
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<TenantConnection> {
        self.client.list_tenants(page, page_size).await
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
    ) -> anyhow::Result<Tenant> {
        self.client.create_tenant(name, owner_user_id).await
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_tenant(
        &self,
        id: &str,
        name: Option<&str>,
        status: Option<&str>,
    ) -> anyhow::Result<Tenant> {
        self.client.update_tenant(id, name, status).await
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

## アダプター（Rust）

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
    State(schema): State<AppSchema>,
    Extension(claims): Extension<Claims>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let request = req.into_inner().data(claims);
    schema.execute(request).await.into()
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

## インフラ（Rust）

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
    /// gRPC エンドポイント（例: "http://tenant-server.k1s0-system.svc.cluster.local:9090"）
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

pub fn init_logger(environment: &str) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    if environment == "production" || environment == "staging" {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
    }
}

pub fn init_tracer(service_name: &str) -> anyhow::Result<opentelemetry_sdk::trace::Tracer> {
    use opentelemetry::global;
    use opentelemetry_otlp::WithExportConfig;

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_owned());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config().with_resource(
                opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                    "service.name",
                    service_name.to_owned(),
                )]),
            ),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    global::set_tracer_provider(tracer.provider().unwrap());
    Ok(tracer)
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
/// 公開鍵は内部にキャッシュし、TTL 経過後に再取得する。
pub struct JwksVerifier {
    jwks_url: String,
    http_client: Client,
    cache: Arc<RwLock<Option<CachedJwks>>>,
    cache_ttl: Duration,
}

struct CachedJwks {
    keys: Vec<Jwk>,
    fetched_at: Instant,
}

#[derive(Debug, Clone, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: Option<String>,
    kty: String,
    alg: Option<String>,
    n: Option<String>,
    e: Option<String>,
}

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

        let n = jwk.n.as_deref().ok_or_else(|| anyhow::anyhow!("JWK missing 'n'"))?;
        let e = jwk.e.as_deref().ok_or_else(|| anyhow::anyhow!("JWK missing 'e'"))?;

        let decoding_key = DecodingKey::from_rsa_components(n, e)
            .map_err(|e| anyhow::anyhow!("invalid RSA key: {}", e))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| anyhow::anyhow!("JWT verification failed: {}", e))?;

        Ok(token_data.claims)
    }

    async fn get_jwks(&self) -> anyhow::Result<Vec<Jwk>> {
        // キャッシュが有効であれば返す
        {
            let cache = self.cache.read().await;
            if let Some(ref c) = *cache {
                if c.fetched_at.elapsed() < self.cache_ttl {
                    debug!("JWKS cache hit");
                    return Ok(c.keys.clone());
                }
            }
        }

        // キャッシュ期限切れ: 再取得
        debug!("fetching JWKS from {}", self.jwks_url);
        let resp: JwksResponse = self
            .http_client
            .get(&self.jwks_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut cache = self.cache.write().await;
        *cache = Some(CachedJwks {
            keys: resp.keys.clone(),
            fetched_at: Instant::now(),
        });

        Ok(resp.keys)
    }
}
```

### src/infra/grpc/tenant_client.rs

```rust
use std::time::Duration;

use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::{Tenant, TenantConnection, TenantStatus};
use crate::infra::config::BackendConfig;

pub mod proto {
    tonic::include_proto!("k1s0.system.tenant.v1");
}

use proto::tenant_service_client::TenantServiceClient;

pub struct TenantGrpcClient {
    client: TenantServiceClient<Channel>,
}

impl TenantGrpcClient {
    pub async fn connect(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect()
            .await?;
        Ok(Self {
            client: TenantServiceClient::new(channel),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_tenant(&self, tenant_id: &str) -> anyhow::Result<Option<Tenant>> {
        let request = tonic::Request::new(proto::GetTenantRequest {
            tenant_id: tenant_id.to_owned(),
        });

        match self.client.clone().get_tenant(request).await {
            Ok(resp) => {
                let t = resp.into_inner().tenant?;
                Ok(Some(Tenant {
                    id: t.id,
                    name: t.name,
                    status: TenantStatus::from(t.status),
                    created_at: t.created_at.map(|ts| ts.seconds.to_string()).unwrap_or_default(),
                    updated_at: String::new(),
                }))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("TenantService.GetTenant failed: {}", e)),
        }
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn list_tenants(
        &self,
        page: i32,
        page_size: i32,
    ) -> anyhow::Result<TenantConnection> {
        let request = tonic::Request::new(proto::ListTenantsRequest {
            pagination: Some(proto::super::common::v1::Pagination { page, page_size }),
        });

        let resp = self
            .client
            .clone()
            .list_tenants(request)
            .await
            .map_err(|e| anyhow::anyhow!("TenantService.ListTenants failed: {}", e))?
            .into_inner();

        let nodes = resp
            .tenants
            .into_iter()
            .map(|t| Tenant {
                id: t.id,
                name: t.name,
                status: TenantStatus::from(t.status),
                created_at: t.created_at.map(|ts| ts.seconds.to_string()).unwrap_or_default(),
                updated_at: String::new(),
            })
            .collect();

        Ok(TenantConnection {
            nodes,
            total_count: resp.pagination.map(|p| p.total_count).unwrap_or(0),
            has_next: resp.pagination.map(|p| p.has_next).unwrap_or(false),
        })
    }

    pub async fn list_tenants_by_ids(
        &self,
        _ids: &[String],
    ) -> anyhow::Result<Vec<Tenant>> {
        // DataLoader 向け: 複数 ID をまとめて取得（ListTenants + クライアント側フィルタ）
        Ok(vec![])
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn create_tenant(
        &self,
        name: &str,
        owner_user_id: &str,
    ) -> anyhow::Result<Tenant> {
        let request = tonic::Request::new(proto::CreateTenantRequest {
            name: name.to_owned(),
            display_name: name.to_owned(),
            owner_user_id: owner_user_id.to_owned(),
            plan: "standard".to_owned(),
        });

        let t = self
            .client
            .clone()
            .create_tenant(request)
            .await
            .map_err(|e| anyhow::anyhow!("TenantService.CreateTenant failed: {}", e))?
            .into_inner()
            .tenant
            .ok_or_else(|| anyhow::anyhow!("empty tenant in response"))?;

        Ok(Tenant {
            id: t.id,
            name: t.name,
            status: TenantStatus::from(t.status),
            created_at: t.created_at.map(|ts| ts.seconds.to_string()).unwrap_or_default(),
            updated_at: String::new(),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn update_tenant(
        &self,
        _id: &str,
        _name: Option<&str>,
        _status: Option<&str>,
    ) -> anyhow::Result<Tenant> {
        // TenantService に UpdateTenant RPC が追加された時点で実装
        anyhow::bail!("UpdateTenant not yet implemented in TenantService");
    }
}
```

### src/infra/grpc/config_client.rs

```rust
use std::time::Duration;

use async_graphql::futures_util::Stream;
use tonic::transport::Channel;
use tracing::instrument;

use crate::domain::model::ConfigEntry;
use crate::infra::config::BackendConfig;

pub mod proto {
    tonic::include_proto!("k1s0.system.config.v1");
}

use proto::config_service_client::ConfigServiceClient;

pub struct ConfigGrpcClient {
    client: ConfigServiceClient<Channel>,
}

impl ConfigGrpcClient {
    pub async fn connect(cfg: &BackendConfig) -> anyhow::Result<Self> {
        let channel = Channel::from_shared(cfg.address.clone())?
            .timeout(Duration::from_millis(cfg.timeout_ms))
            .connect()
            .await?;
        Ok(Self {
            client: ConfigServiceClient::new(channel),
        })
    }

    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn get_config(
        &self,
        namespace: &str,
        key: &str,
    ) -> anyhow::Result<Option<ConfigEntry>> {
        let request = tonic::Request::new(proto::GetConfigRequest {
            namespace: namespace.to_owned(),
            key: key.to_owned(),
        });

        match self.client.clone().get_config(request).await {
            Ok(resp) => {
                let entry = resp.into_inner().entry?;
                let value_str = String::from_utf8(entry.value).unwrap_or_default();
                Ok(Some(ConfigEntry {
                    key: format!("{}/{}", entry.namespace, entry.key),
                    value: value_str,
                    updated_at: entry
                        .updated_at
                        .map(|ts| ts.seconds.to_string())
                        .unwrap_or_default(),
                }))
            }
            Err(status) if status.code() == tonic::Code::NotFound => Ok(None),
            Err(e) => Err(anyhow::anyhow!("ConfigService.GetConfig failed: {}", e)),
        }
    }

    /// WatchConfig Server-Side Streaming を購読し、変更イベントを ConfigEntry として返す。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_config(
        &self,
        namespaces: Vec<String>,
    ) -> impl Stream<Item = ConfigEntry> {
        let request = tonic::Request::new(proto::WatchConfigRequest { namespaces });

        let stream = self
            .client
            .clone()
            .watch_config(request)
            .await
            .expect("WatchConfig stream failed")
            .into_inner();

        async_graphql::futures_util::stream::unfold(stream, |mut stream| async move {
            match stream.message().await {
                Ok(Some(resp)) => {
                    let value_str = String::from_utf8(resp.new_value).unwrap_or_default();
                    let entry = ConfigEntry {
                        key: format!("{}/{}", resp.namespace, resp.key),
                        value: value_str,
                        updated_at: resp
                            .changed_at
                            .map(|ts| ts.seconds.to_string())
                            .unwrap_or_default(),
                    };
                    Some((entry, stream))
                }
                _ => None,
            }
        })
    }
}
```

---

## config.yaml

```yaml
# config/config.yaml
app:
  name: "graphql-gateway"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080

graphql:
  introspection: false
  playground: false
  max_depth: 10
  max_complexity: 1000

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local/jwks"

backends:
  tenant:
    address: "http://tenant-server.k1s0-system.svc.cluster.local:9090"
    timeout_ms: 3000
  featureflag:
    address: "http://featureflag-server.k1s0-system.svc.cluster.local:9090"
    timeout_ms: 3000
  config:
    address: "http://config-server.k1s0-system.svc.cluster.local:9090"
    timeout_ms: 3000

observability:
  log:
    level: "info"
    format: "json"
  trace:
    enabled: true
    endpoint: "jaeger.observability.svc.cluster.local:4317"
    sample_rate: 1.0
  metrics:
    enabled: true
    path: "/metrics"
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

## テスト構成（graphql-gateway）

### レイヤー別テスト

| レイヤー | テスト種別 | 手法 |
| --- | --- | --- |
| domain/model | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/graphql_handler | 統合テスト（HTTP） | `axum-test` + `tokio::test` |
| adapter/middleware | 単体テスト | `tokio::test` + モック JWT |
| infra/auth | 単体テスト | `tokio::test` + `wiremock` |
| infra/grpc | 統合テスト | `tonic` mock + `tokio::test` |

### ユースケーステスト例

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

### GraphQL ハンドラーテスト例

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

- [system-graphql-gateway設計.md](system-graphql-gateway設計.md) -- 概要・API 定義・アーキテクチャ
- [system-graphql-gateway-デプロイ設計.md](system-graphql-gateway-デプロイ設計.md) -- Dockerfile・Helm values・デプロイ設計
- [proto設計.md](proto設計.md) -- ConfigService / TenantService / FeatureFlagService proto 定義
- [認証認可設計.md](認証認可設計.md) -- JWT Claims 構造・RBAC ロール定義
- [GraphQL設計.md](GraphQL設計.md) -- GraphQL 設計ガイドライン
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート仕様
- [コーディング規約.md](コーディング規約.md) -- コーディング規約
