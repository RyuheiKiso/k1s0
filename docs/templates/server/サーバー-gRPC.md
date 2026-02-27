# テンプレート仕様 — サーバー — gRPC Health Check / GraphQL Subscription

本ドキュメントは、[テンプレート仕様-サーバー](サーバー.md) から分割された gRPC Health Check テンプレートおよび GraphQL Subscription テンプレートの詳細仕様である。

---

## gRPC Health Check テンプレート

`api_styles is containing("grpc")` の場合のみ生成される gRPC Health Checking Protocol 実装。[gRPC Health Checking Protocol](https://github.com/grpc/grpc/blob/master/doc/health-checking.md) に準拠し、Kubernetes の gRPC ネイティブプローブに対応する。

> **Helm Chart との連携**: gRPC Health Check プローブの設定値（`probes.grpcHealthCheck.enabled` / `probes.grpcHealthCheck.port`）は [テンプレート仕様-Helm](../infrastructure/Helm.md) の values.yaml で定義済みである。`grpcHealthCheck.enabled: true` に設定することで、Kubernetes の liveness/readiness プローブが gRPC ネイティブプローブに切り替わる。

### src/adapter/handler/grpc_health.rs

`src/adapter/handler/grpc_health.rs.tera` — tonic-health crate を使用した gRPC Health Checking Protocol 実装。条件: `{% if api_styles is containing("grpc") %}`。

```rust
{% if api_styles is containing("grpc") %}
use tonic_health::server::health_reporter;

/// gRPC Health Checking Protocol のサービスを初期化する。
/// tonic-health を使用して grpc.health.v1.Health サービスを提供し、
/// Kubernetes の gRPC ネイティブプローブに対応する。
pub async fn create_health_service() -> (
    tonic_health::server::HealthReporter,
    tonic_health::server::HealthServer,
) {
    let (mut reporter, service) = health_reporter();

    // サービス名を指定してヘルスステータスを設定
    reporter
        .set_serving::<tonic::transport::Server>()
        .await;

    (reporter, service)
}
{% endif %}
```

---

## GraphQL Subscription テンプレート

`api_styles is containing("graphql")` の場合のみ生成される GraphQL Subscription サポート。WebSocket transport を使用してリアルタイム通信を実現する。

> **BFF との関連**: BFF テンプレート（[テンプレート仕様-BFF](../client/BFF.md)）はデフォルトで Query / Mutation のみを提供する。サーバーテンプレートの GraphQL Subscription は、サーバー側でリアルタイムイベントを配信する際に使用する。BFF が Subscription を中継する場合は、BFF の schema.graphql に Subscription 型を追加し、upstream サーバーの WebSocket エンドポイントに接続する拡張が必要となる。

### schema 定義への SubscriptionRoot 追加

`src/adapter/handler/graphql.rs.tera` — async-graphql の SubscriptionRoot を追加する。条件: `{% if api_styles is containing("graphql") %}`。

```rust
{% if api_styles is containing("graphql") %}
use async_graphql::{Context, Object, Schema, Subscription};
use futures_core::stream::Stream;
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

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    // TODO: Mutation リゾルバーを実装する
    async fn placeholder(&self) -> bool {
        true
    }
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// エンティティ変更通知
    async fn {{ service_name_snake }}_changed(
        &self,
    ) -> impl Stream<Item = {{ service_name_pascal }}Entity> {
        // TODO: 実際のイベントストリーム（Kafka consumer 等）に接続する
        futures_util::stream::empty()
    }
}

pub type {{ service_name_pascal }}Schema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn build_schema(uc: {{ service_name_pascal }}UseCase) -> {{ service_name_pascal }}Schema {
    Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .data(Arc::new(uc))
        .finish()
}
{% endif %}
```

### main.rs への WebSocket route 追加

`src/main.rs.tera` — axum の WebSocket サポートを追加する。`axum::extract::ws` を使用する。

```rust
{% if api_styles is containing("graphql") %}
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};

// GraphQL エンドポイント
let app = Router::new()
    .route("/query", post(graphql_handler))
    .route("/ws", get(GraphQLSubscription::new(schema.clone())));

async fn graphql_handler(
    schema: Extension<{{ service_name_pascal }}Schema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
{% endif %}
```

---

## 関連ドキュメント

- [テンプレート仕様-サーバー](サーバー.md) --- 概要・条件付き生成表・Tier別配置パス
- [テンプレート仕様-サーバー-Rust](サーバー-Rust.md) --- Rust テンプレート詳細
- [テンプレート仕様-サーバー-可観測性](サーバー-可観測性.md) --- 可観測性テンプレート
- [テンプレート仕様-サーバー-認証](サーバー-認証.md) --- 認証認可Middleware テンプレート
- [テンプレート仕様-Helm](../infrastructure/Helm.md) --- Helm Chart テンプレート（gRPC ヘルスチェック設定含む）
- [テンプレート仕様-BFF](../client/BFF.md) --- BFF テンプレート
- [API設計](../../api/gateway/API設計.md) --- REST / gRPC / GraphQL 設計
