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
│   │   ├── port.rs                # ポートトレイト（TenantPort, FeatureFlagPort, ConfigPort）
│   │   ├── model/
│   │   │   ├── mod.rs
│   │   │   ├── tenant.rs          # Tenant, TenantStatus, TenantConnection
│   │   │   ├── feature_flag.rs    # FeatureFlag
│   │   │   ├── config_entry.rs    # ConfigEntry
│   │   │   ├── auth.rs             # User, Role, PermissionCheck, AuditLog, AuditLogConnection
│   │   │   ├── session.rs          # Session, SessionStatus
│   │   │   ├── vault.rs            # SecretMetadata, VaultAuditLogEntry
│   │   │   ├── scheduler.rs        # Job, JobExecution
│   │   │   ├── notification.rs     # NotificationLog, NotificationChannel, NotificationTemplate
│   │   │   ├── workflow.rs         # WorkflowDefinition, WorkflowStep, WorkflowInstance, WorkflowTask
│   │   │   ├── payload.rs          # Mutation payload types (all services)
│   │   │   └── graphql_context.rs # GraphqlContext (user_id, roles, request_id, loaders)
│   │   └── loader/
│   │       └── mod.rs             # DataLoader trait 実装（TenantLoader, FeatureFlagLoader, ConfigLoader）
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── tenant_query.rs        # TenantQueryResolver
│   │   ├── feature_flag_query.rs  # FeatureFlagQueryResolver
│   │   ├── config_query.rs        # ConfigQueryResolver
│   │   ├── tenant_mutation.rs     # TenantMutationResolver
│   │   ├── auth_query.rs           # AuthQueryResolver
│   │   ├── auth_mutation.rs        # AuthMutationResolver
│   │   ├── session_query.rs        # SessionQueryResolver
│   │   ├── session_mutation.rs     # SessionMutationResolver
│   │   ├── vault_query.rs          # VaultQueryResolver
│   │   ├── vault_mutation.rs       # VaultMutationResolver
│   │   ├── scheduler_query.rs      # SchedulerQueryResolver
│   │   ├── scheduler_mutation.rs   # SchedulerMutationResolver
│   │   ├── notification_query.rs   # NotificationQueryResolver
│   │   ├── notification_mutation.rs # NotificationMutationResolver
│   │   ├── workflow_query.rs       # WorkflowQueryResolver
│   │   ├── workflow_mutation.rs    # WorkflowMutationResolver
│   │   └── subscription.rs        # SubscriptionResolver
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── graphql_handler.rs     # POST/GET /graphql, /graphql/ws
│   │   └── middleware/
│   │       ├── mod.rs
│   │       └── auth_middleware.rs  # JWT 検証 axum layer
│   └── infrastructure/
│       ├── mod.rs
│       ├── config/
│       │   └── mod.rs             # Config struct
│       ├── grpc/
│       │   ├── mod.rs
│       │   ├── tenant_client.rs   # TenantGrpcClient
│       │   ├── feature_flag_client.rs
│       │   ├── config_client.rs
│       │   ├── auth_client.rs      # AuthGrpcClient (AuthService + AuditService)
│       │   ├── session_client.rs   # SessionGrpcClient
│       │   ├── vault_client.rs     # VaultGrpcClient
│       │   ├── scheduler_client.rs # SchedulerGrpcClient
│       │   ├── notification_client.rs # NotificationGrpcClient
│       │   └── workflow_client.rs  # WorkflowGrpcClient
│       ├── http/
│       │   ├── mod.rs
│       │   └── service_catalog_client.rs  # ServiceCatalogHttpClient (reqwest/REST)
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

> 共通依存は [Rust共通実装.md](../../_common/Rust共通実装.md#共通cargo依存) を参照。

| クレート | バージョン | 用途 |
| --- | --- | --- |
| `axum` | 0.8 | HTTP フレームワーク（`macros`, `ws` feature） |
| `async-graphql` | 7 | GraphQL サーバー（`dataloader` feature） |
| `async-graphql-axum` | 7 | axum 統合 |
| `jsonwebtoken` | 9 | JWT 検証 |
| `reqwest` | 0.12 | JWKS 取得用 HTTP クライアント（`json`, `rustls-tls` feature） |
| `async-trait` | 0.1 | 非同期トレイト |
| `k1s0-telemetry` | path | テレメトリ（`full` feature） |
| `axum-test` | 16 | テスト（dev-dependency） |

### build.rs

gRPC クライアント側のため `build_server(false)` / `build_client(true)`。proto パス: `tenant.proto`, `featureflag.proto`, `config.proto`, `navigation.proto`, `auth.proto`, `session.proto`, `vault.proto`, `scheduler.proto`, `notification.proto`, `workflow.proto`。

> **注記（監査 C-4 対応）**: `service_catalog.proto` は削除済み。service-catalog サーバーは HTTP/axum で実装されており gRPC を提供しないため、`infrastructure/http/service_catalog_client.rs` に reqwest ベースの REST クライアントを用意している（[ADR-0039](../../../architecture/adr/0039-service-catalog-rest-client.md)）。

---

## テスト構成

| レイヤー | テスト種別 | 手法 |
| --- | --- | --- |
| domain/model | 単体テスト | `#[cfg(test)]` + `assert!` |
| usecase | 単体テスト（モック） | `mockall` |
| adapter/graphql_handler | 統合テスト（HTTP） | `axum-test` + `tokio::test` |
| adapter/middleware | 単体テスト | `tokio::test` + モック JWT |
| infrastructure/auth | 単体テスト | `tokio::test` + `wiremock` |
| infrastructure/grpc | 統合テスト | `tonic` mock + `tokio::test` |

---

## Cargo.toml

> 共通依存は [Rust共通実装.md](../../_common/Rust共通実装.md#共通cargo依存) を参照。サービス固有の追加依存:

```toml
# axum に WebSocket サポートを追加
axum = { version = "0.8", features = ["macros", "ws"] }

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
axum-test = "17"
```

---

## src/main.rs

> 起動シーケンスは [Rust共通実装.md](../../_common/Rust共通実装.md#共通mainrs) を参照。graphql-gateway は DB/Kafka を使用せず、REST サーバーのみ起動する。以下はサービス固有の DI:

```rust
    // --- gRPC クライアント ---
    let tenant_client = Arc::new(TenantGrpcClient::connect(&cfg.backends.tenant).await?);
    let feature_flag_client = Arc::new(FeatureFlagGrpcClient::connect(&cfg.backends.featureflag).await?);
    let config_client = Arc::new(ConfigGrpcClient::connect(&cfg.backends.config).await?);
    let navigation_client = Arc::new(NavigationGrpcClient::connect(&cfg.backends.navigation).await?);
    // service-catalog は REST のみ提供するため ServiceCatalogHttpClient を使用する（ADR-0039）
    let service_catalog_client = Arc::new(ServiceCatalogHttpClient::new(&cfg.backends.service_catalog)?);
    let auth_client = Arc::new(AuthGrpcClient::connect(&cfg.backends.auth).await?);
    let session_client = Arc::new(SessionGrpcClient::connect(&cfg.backends.session).await?);
    let vault_client = Arc::new(VaultGrpcClient::connect(&cfg.backends.vault).await?);
    let scheduler_client = Arc::new(SchedulerGrpcClient::connect(&cfg.backends.scheduler).await?);
    let notification_client = Arc::new(NotificationGrpcClient::connect(&cfg.backends.notification).await?);
    let workflow_client = Arc::new(WorkflowGrpcClient::connect(&cfg.backends.workflow).await?);

    // --- JWT 検証 ---
    let jwks_verifier = Arc::new(JwksVerifier::new(cfg.auth.jwks_url.clone()));

    // --- GatewayClients ---
    let clients = GatewayClients {
        tenant: tenant_client.clone(),
        feature_flag: feature_flag_client.clone(),
        config: config_client.clone(),
        navigation: navigation_client.clone(),
        service_catalog: service_catalog_client.clone(),
        auth: auth_client.clone(),
        session: session_client.clone(),
        vault: vault_client.clone(),
        scheduler: scheduler_client.clone(),
        notification: notification_client.clone(),
        workflow: workflow_client.clone(),
    };

    // --- GatewayResolvers ---
    let resolvers = GatewayResolvers { /* ... all 20 resolvers ... */ };

    // --- Router ---
    let app = graphql_handler::router(
        jwks_verifier,
        clients,
        resolvers,
        cfg.graphql.clone(),
        metrics.clone(),
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

### src/domain/port.rs

クリーンアーキテクチャの依存性逆転の原則（DIP）に基づき、ドメイン層がインフラストラクチャ層に直接依存しないよう、ポートトレイトを定義する。
各 gRPC クライアントはこのトレイトを実装することで、domain 層からはトレイトオブジェクト経由でのみアクセスされる。

```rust
use crate::domain::model::{ConfigEntry, FeatureFlag, Tenant};

/// テナントサービスへのアクセスを抽象化するポートトレイト
#[async_trait::async_trait]
pub trait TenantPort: Send + Sync {
    async fn list_tenants_by_ids(&self, ids: &[String]) -> anyhow::Result<Vec<Tenant>>;
}

/// フィーチャーフラグサービスへのアクセスを抽象化するポートトレイト
#[async_trait::async_trait]
pub trait FeatureFlagPort: Send + Sync {
    async fn list_flags_by_keys(&self, keys: &[String]) -> anyhow::Result<Vec<FeatureFlag>>;
}

/// コンフィグサービスへのアクセスを抽象化するポートトレイト
#[async_trait::async_trait]
pub trait ConfigPort: Send + Sync {
    async fn list_configs_by_keys(&self, keys: &[String]) -> anyhow::Result<Vec<ConfigEntry>>;
}
```

### src/domain/model/graphql_context.rs

> **注記（監査 C-05 対応）**: 旧実装では `use crate::infrastructure::grpc::{...}` により domain 層が infrastructure 層に直接依存していた（クリーンアーキテクチャ違反）。ポートトレイト（`domain::port`）を導入して依存を逆転させた。

```rust
use std::sync::Arc;

use async_graphql::dataloader::DataLoader;

// ポートトレイトのみに依存し、インフラストラクチャ層への直接依存を排除
use crate::domain::port::{ConfigPort, FeatureFlagPort, TenantPort};

pub struct GraphqlContext {
    pub user_id: String,
    pub roles: Vec<String>,
    pub request_id: String,
    pub tenant_loader: Arc<DataLoader<TenantLoader>>,
    pub flag_loader: Arc<DataLoader<FeatureFlagLoader>>,
    pub config_loader: Arc<DataLoader<ConfigLoader>>,
}

// client フィールドは具象型ではなくトレイトオブジェクトを保持
pub struct TenantLoader {
    pub client: Arc<dyn TenantPort>,
}

pub struct FeatureFlagLoader {
    pub client: Arc<dyn FeatureFlagPort>,
}

pub struct ConfigLoader {
    pub client: Arc<dyn ConfigPort>,
}
```

### src/domain/loader/mod.rs

```rust
use async_graphql::dataloader::Loader;
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::model::graphql_context::{ConfigLoader, FeatureFlagLoader, TenantLoader};
use crate::domain::model::{ConfigEntry, FeatureFlag, Tenant};

impl Loader<String> for TenantLoader {
    type Value = Tenant;
    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let tenants = self
            .client
            .list_tenants_by_ids(keys)
            .await
            .map_err(Arc::new)?;
        Ok(tenants.into_iter().map(|t| (t.id.clone(), t)).collect())
    }
}

impl Loader<String> for FeatureFlagLoader {
    type Value = FeatureFlag;
    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let flags = self
            .client
            .list_flags_by_keys(keys)
            .await
            .map_err(Arc::new)?;
        Ok(flags.into_iter().map(|f| (f.key.clone(), f)).collect())
    }
}

impl Loader<String> for ConfigLoader {
    type Value = ConfigEntry;
    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[String],
    ) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let mut result = HashMap::new();
        for key in keys {
            let parts: Vec<&str> = key.splitn(2, '/').collect();
            if parts.len() != 2 {
                continue;
            }
            match self.client.get_config(parts[0], parts[1]).await {
                Ok(Some(entry)) => {
                    result.insert(key.clone(), entry);
                }
                Ok(None) => {}
                Err(e) => return Err(Arc::new(e)),
            }
        }
        Ok(result)
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
use crate::infrastructure::grpc::TenantGrpcClient;

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
    ) -> anyhow::Result<TenantConnection> {
        self.client.list_tenants(first, after).await
    }
}
```

### src/usecase/feature_flag_query.rs

```rust
use std::sync::Arc;

use tracing::instrument;

use crate::domain::model::FeatureFlag;
use crate::infrastructure::grpc::FeatureFlagGrpcClient;

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
use crate::infrastructure::grpc::ConfigGrpcClient;

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
use crate::infrastructure::grpc::TenantGrpcClient;

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

use crate::domain::model::{ConfigEntry, FeatureFlag, Tenant};
use crate::infrastructure::grpc::{ConfigGrpcClient, FeatureFlagGrpcClient, TenantGrpcClient};

pub struct SubscriptionResolver {
    config_client: Arc<ConfigGrpcClient>,
    tenant_client: Arc<TenantGrpcClient>,
    feature_flag_client: Arc<FeatureFlagGrpcClient>,
}

impl SubscriptionResolver {
    pub fn new(
        config_client: Arc<ConfigGrpcClient>,
        tenant_client: Arc<TenantGrpcClient>,
        feature_flag_client: Arc<FeatureFlagGrpcClient>,
    ) -> Self {
        Self {
            config_client,
            tenant_client,
            feature_flag_client,
        }
    }

    /// WatchConfig ストリームを返す。設定変更が発生するたびに ConfigEntry を配信する。
    #[instrument(skip(self), fields(service = "graphql-gateway"))]
    pub async fn watch_config(
        &self,
        namespaces: Vec<String>,
    ) -> impl Stream<Item = ConfigEntry> {
        self.config_client.watch_config(namespaces).await
    }

    pub fn watch_tenant_updated(
        &self,
        tenant_id: String,
    ) -> impl Stream<Item = Tenant> {
        // 現在は 5 秒ポーリングで監視
        unimplemented!()
    }

    pub fn watch_feature_flag_changed(
        &self,
        key: String,
    ) -> impl Stream<Item = FeatureFlag> {
        // 現在は 5 秒ポーリングで監視
        unimplemented!()
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
use crate::infrastructure::config::GraphQLConfig;
use crate::infrastructure::grpc::{ConfigGrpcClient, FeatureFlagGrpcClient, TenantGrpcClient};
use crate::usecase::{
    ConfigQueryResolver, FeatureFlagQueryResolver, SubscriptionResolver,
    TenantMutationResolver, TenantQueryResolver,
};

pub type AppSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

#[derive(Clone)]
pub struct AppState {
    pub schema: AppSchema,
    pub query_timeout: std::time::Duration,
    pub tenant_client: Arc<TenantGrpcClient>,
    pub feature_flag_client: Arc<FeatureFlagGrpcClient>,
    pub config_client: Arc<ConfigGrpcClient>,
    pub navigation_client: Arc<NavigationGrpcClient>,
    // service-catalog は REST クライアント（ADR-0039）
    pub service_catalog_client: Arc<ServiceCatalogHttpClient>,
    pub auth_client: Arc<AuthGrpcClient>,
    pub session_client: Arc<SessionGrpcClient>,
    pub vault_client: Arc<VaultGrpcClient>,
    pub scheduler_client: Arc<SchedulerGrpcClient>,
    pub notification_client: Arc<NotificationGrpcClient>,
    pub workflow_client: Arc<WorkflowGrpcClient>,
}

pub fn router(
    jwks_verifier: Arc<crate::infrastructure::auth::JwksVerifier>,
    clients: GatewayClients,
    resolvers: GatewayResolvers,
    graphql_cfg: GraphQLConfig,
    metrics: Arc<k1s0_telemetry::metrics::Metrics>,
) -> Router {
    let schema = Schema::build(
        QueryRoot {
            tenant_query: tenant_query.clone(),
            feature_flag_query: feature_flag_query.clone(),
            config_query: config_query.clone(),
        },
        MutationRoot {
            tenant_mutation: tenant_mutation.clone(),
            feature_flag_client: feature_flag_client.clone(),
        },
        SubscriptionRoot {
            subscription: subscription.clone(),
        },
    )
    .limit_depth(graphql_cfg.max_depth as usize)
    .limit_complexity(graphql_cfg.max_complexity as usize)
    .finish();

    let state = AppState {
        schema: schema.clone(),
        query_timeout: std::time::Duration::from_secs(graphql_cfg.query_timeout_seconds as u64),
        tenant_client,
        feature_flag_client,
        config_client,
    };

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
        .with_state(state);

    // 開発環境のみ Playground を有効化
    if graphql_cfg.playground {
        router = router.route("/graphql", get(graphql_playground));
    }

    router
}

async fn graphql_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    req: GraphQLRequest,
) -> impl IntoResponse {
    let request = req.into_inner().data(claims);
    match tokio::time::timeout(state.query_timeout, state.schema.execute(request)).await {
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
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>, // JWT は auth_layer で検証済み
    ws: axum::extract::WebSocketUpgrade,
) -> impl IntoResponse {
    GraphQLSubscription::new(state.schema).on_upgrade(ws)
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

async fn readyz(State(state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    let tenant = state.tenant_client.healthz().await;
    let feature_flag = state.feature_flag_client.healthz().await;
    let config = state.config_client.healthz().await;
    let navigation = state.navigation_client.healthz().await;
    let service_catalog = state.service_catalog_client.healthz().await;
    let auth = state.auth_client.healthz().await;
    let session = state.session_client.healthz().await;
    let vault = state.vault_client.healthz().await;
    let scheduler = state.scheduler_client.healthz().await;
    let notification = state.notification_client.healthz().await;
    let workflow = state.workflow_client.healthz().await;

    let checks = serde_json::json!({
        "tenant": if tenant.is_ok() { "ok" } else { "error" },
        "featureflag": if feature_flag.is_ok() { "ok" } else { "error" },
        "config": if config.is_ok() { "ok" } else { "error" },
        "navigation": if navigation.is_ok() { "ok" } else { "error" },
        "service_catalog": if service_catalog.is_ok() { "ok" } else { "error" },
        "auth": if auth.is_ok() { "ok" } else { "error" },
        "session": if session.is_ok() { "ok" } else { "error" },
        "vault": if vault.is_ok() { "ok" } else { "error" },
        "scheduler": if scheduler.is_ok() { "ok" } else { "error" },
        "notification": if notification.is_ok() { "ok" } else { "error" },
        "workflow": if workflow.is_ok() { "ok" } else { "error" }
    });

    let all_ok = tenant.is_ok()
        && feature_flag.is_ok()
        && config.is_ok()
        && navigation.is_ok()
        && service_catalog.is_ok()
        && auth.is_ok()
        && session.is_ok()
        && vault.is_ok()
        && scheduler.is_ok()
        && notification.is_ok()
        && workflow.is_ok();

    if all_ok {
        (StatusCode::OK, Json(serde_json::json!({"status": "ready", "checks": checks})))
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"status": "not_ready", "checks": checks})),
        )
    }
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
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::infrastructure::auth::JwksVerifier;

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

#[derive(Clone)]
pub struct AuthLayer {
    verifier: Arc<JwksVerifier>,
}

pub fn auth_layer(verifier: Arc<JwksVerifier>) -> AuthLayer {
    AuthLayer { verifier }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            verifier: self.verifier.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    verifier: Arc<JwksVerifier>,
}

impl<S> Service<Request> for AuthService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let verifier = self.verifier.clone();
        let mut inner = self.inner.clone();
        Box::pin(async move {
            match verify_jwt(verifier, req).await {
                Ok(req) => inner.call(req).await,
                Err(response) => Ok(response),
            }
        })
    }
}

async fn verify_jwt(verifier: Arc<JwksVerifier>, mut req: Request) -> Result<Request, Response> {
    let token = extract_bearer_token(req.headers());

    let token = match token {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "missing Authorization header"
                    }
                })),
            )
                .into_response());
        }
    };

    let claims = match verifier.verify_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": {
                        "code": "UNAUTHORIZED",
                        "message": "invalid or expired JWT token"
                    }
                })),
            )
                .into_response());
        }
    };

    req.extensions_mut().insert(claims);
    Ok(req)
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

### src/infrastructure/config/mod.rs

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
    pub navigation: BackendConfig,
    pub service_catalog: BackendConfig,
    pub auth: BackendConfig,
    pub session: BackendConfig,
    pub vault: BackendConfig,
    pub scheduler: BackendConfig,
    pub notification: BackendConfig,
    pub workflow: BackendConfig,
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
        Ok(())
    }
}
```

> `ConfigLoader` で解決できない場合（`GraphqlContext` がない等）は `ConfigQueryResolver` へのフォールバックで単体取得を行う。

### src/infrastructure/auth/jwks.rs

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

### src/infrastructure/grpc/tenant_client.rs

gRPC クライアント実装（TenantService、FeatureFlagService、ConfigService）の詳細は guide から統合済み。各クライアントは `tonic::transport::Channel` で接続し、Domain モデルに変換する。

---

## 設定ファイル例

> 共通セクション（app/server/observability）は [Rust共通実装.md](../../_common/Rust共通実装.md#共通configyaml) を参照。graphql-gateway は database/kafka を使用しない。サービス固有セクション:

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
  navigation:
    address: "http://navigation-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  # service-catalog は REST エンドポイントを使用する（ADR-0039）
  service_catalog:
    address: "http://service-catalog-server.k1s0-system.svc.cluster.local:8080"
    timeout_ms: 3000
  auth:
    address: "http://auth-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  session:
    address: "http://session-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  vault:
    address: "http://vault-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  scheduler:
    address: "http://scheduler-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  notification:
    address: "http://notification-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000
  workflow:
    address: "http://workflow-server.k1s0-system.svc.cluster.local:50051"
    timeout_ms: 3000

observability:
  log:
    level: "info"
    format: "json"
  trace:
    enabled: true
    endpoint: "http://otel-collector.observability.svc.cluster.local:4317"
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

## Doc Sync (2026-03-04)

### 実装方針の反映
- `GraphqlContext` に `config_loader` を含める。
- `ConfigLoader` は `GetConfig` 呼び出し時に tenant 文脈を含めて問い合わせる。
- Loader trait は native async を採用し、エラー型は `Arc<anyhow::Error>` を使用する。
- Auth middleware は Tower `Layer` / `Service` パターンで統一する。
- `/readyz` は全 11 バックエンド（tenant / featureflag / config / navigation / service_catalog / auth / session / vault / scheduler / notification / workflow）の疎通を確認する。
- Subscription は WebSocket 経由で配信し、tenant / featureflag / config の全 3 サブスクリプションで gRPC Server-Side Streaming（`WatchTenant` / `WatchFeatureFlag` / `WatchConfig`）を使用したイベント駆動方式を採用する。
- WebSocket 接続時に JWT 検証を実施する。
- `list_tenants` では `last` / `before` を受け付けない。
- `create_tenant` は bare payload ではなく GraphQL payload オブジェクトを返す。
- メトリクス実装は `k1s0_telemetry` に統一する。

### ファイル整理
- `regions/system/server/rust/graphql-gateway/src/handler/schema.rs` は stale ファイルとして削除済み。

## Doc Sync (2026-03-21)

### gRPC クライアント connect_lazy 化 [技術品質監査 Critical 1-2]

**背景・問題**

全 11 個の gRPC クライアント（`auth_client.rs` 等）が起動時に `Channel::connect().await?` を呼び出していた。
これにより、いずれかのバックエンドサービスが起動していない場合、GraphQL Gateway 自体の起動が失敗する問題があった。
マイクロサービスアーキテクチャにおいては、起動順序の依存が運用上の障害になる。

**対応内容**

全 11 クライアントの接続方式を `connect_lazy()` に変更した。

- 変更前: `async fn connect(cfg)` → `Channel::from_shared(...).connect().await?`
- 変更後: `fn new(cfg)` → `Channel::from_shared(...).connect_lazy()`

`connect_lazy()` は実際の RPC 呼び出し時に初めて接続を確立するため、起動時のバックエンド依存が排除される。
`startup.rs` 側では `XxxClient::connect(&cfg).await?` を `XxxClient::new(&cfg)?` に変更し、
`init_clients()` 関数を `async fn` から同期 `fn` に変更した。

**影響範囲**

- `src/infrastructure/grpc/` 配下の全クライアントファイル（11ファイル）
- `src/infrastructure/startup.rs`（クライアント初期化箇所）

**設計上の注意**

バックエンドが実際に到達不能の場合はRPC呼び出し時にエラーが発生する。
`/readyz` エンドポイントは引き続き全バックエンドの疎通確認を行うため、
ヘルスチェック経由で到達不能バックエンドを検出可能。

### RBAC ロジック共通化 [P2-24]

**背景・問題**

`graphql_handler.rs` にローカルな `Tier` 列挙型と `check_permission` 関数が定義されており、
他サーバーで同様のロジックを実装する際に重複が生じる問題があった。

**対応内容**

`graphql_handler.rs` からローカルの `Tier` 列挙型と `check_permission` 関数を削除し、
`k1s0_server_common::middleware::rbac::{Tier, check_permission}` を使用するように変更した。

- `Cargo.toml` の `k1s0-server-common` 依存に `auth` feature を追加
- `graphql_handler.rs` の `use` 宣言を `k1s0_server_common::middleware::rbac` に更新

**影響範囲**

- `src/adapter/graphql_handler.rs`
- `Cargo.toml`（`k1s0-server-common` の `auth` feature 有効化）

### gRPC リトライポリシー [P2-25]

**背景・問題**

gRPC 呼び出しで `UNAVAILABLE` / `DEADLINE_EXCEEDED` が発生した場合、即座にエラーを返していた。
一時的なネットワーク障害やポッド再起動時の呼び出し失敗を自動リカバリする仕組みが必要だった。

**対応内容**

`src/infrastructure/grpc_retry.rs` を新規追加した。
`with_retry(operation_name, max_attempts, closure)` ヘルパー関数を実装し、
`tonic::Code::Unavailable` および `tonic::Code::DeadlineExceeded` に対して指数バックオフ付きリトライを行う。
`auth_client.rs` の `get_user` メソッドにて `with_retry` を適用済み。

**影響範囲**

- `src/infrastructure/grpc_retry.rs`（新規追加）
- `src/infrastructure/grpc/auth_client.rs`（`get_user` へのリトライ適用）
- `src/infrastructure/mod.rs`（`grpc_retry` モジュール公開）

**ディレクトリ構成への反映**

```
│       └── grpc_retry.rs              # with_retry ヘルパー（指数バックオフ）
```

### Subscription ストリームエラー伝播 [P2-26]

**背景・問題**

`config_client.rs` の `watch_config` が返すストリームのアイテム型が `ConfigEntry` であり、
ストリーム切断時のエラーがサブスクライバーに伝播されずサイレントに終了していた。

**対応内容**

`config_client.rs` の `watch_config` ストリームアイテム型を
`ConfigEntry` から `Result<ConfigEntry, tonic::Status>` に変更した。
ストリーム切断時のエラーが `Err(tonic::Status)` としてサブスクライバーに通知される。

あわせて `graphql_handler.rs` の `config_changed` サブスクリプションリゾルバーで、
`Err(status)` を `async_graphql::Error` に変換して返す処理を追加した。

**影響範囲**

- `src/infrastructure/grpc/config_client.rs`（ストリーム型変更）
- `src/usecase/subscription.rs`（`watch_config` 戻り値型変更: `Stream<Item = ConfigEntry>` → `Stream<Item = Result<ConfigEntry, tonic::Status>>`）
- `src/adapter/graphql_handler.rs`（`config_changed` でのエラー変換処理追加）
