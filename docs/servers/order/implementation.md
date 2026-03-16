# service-order-server 実装設計

service-order-server の Rust 実装構成を定義する。概要・API 定義・アーキテクチャは [service-order-server.md](server.md) を参照。

---

## Rust 実装 (regions/service/order/server/rust/order/)

### ディレクトリ構成

```
regions/service/order/server/rust/order/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── lib.rs                           # ライブラリルート（MIGRATOR 定義）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── error.rs                     # OrderError（thiserror ベース）
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── order.rs                 # Order, OrderItem, OrderStatus, CreateOrder, OrderFilter
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   └── order_repository.rs      # OrderRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── order_service.rs         # OrderDomainService（バリデーション・ステータス遷移・合計計算）
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_order.rs              # CreateOrderUseCase
│   │   ├── get_order.rs                 # GetOrderUseCase
│   │   ├── update_order_status.rs       # UpdateOrderStatusUseCase
│   │   ├── list_orders.rs              # ListOrdersUseCase
│   │   └── event_publisher.rs          # OrderEventPublisher トレイト + NoopOrderEventPublisher
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs                   # AppState, router(), actor_from_claims()
│   │   │   ├── order_handler.rs         # 注文 REST ハンドラー
│   │   │   └── health.rs               # ヘルスチェックハンドラー
│   │   ├── presenter/
│   │   │   ├── mod.rs
│   │   │   └── order_presenter.rs       # OrderDetailResponse, OrderListResponse, OrderSummaryResponse
│   │   └── middleware/
│   │       ├── mod.rs
│   │       └── auth.rs                  # JWT 認証ミドルウェア（k1s0-server-common 経由）
│   └── infrastructure/
│       ├── mod.rs
│       ├── config.rs                    # Config ローダー・バリデーション
│       ├── database/
│       │   ├── mod.rs
│       │   └── order_repository.rs      # OrderPostgresRepository（sqlx 実装）
│       └── kafka/
│           ├── mod.rs
│           └── order_producer.rs        # OrderKafkaProducer（rdkafka 実装）
├── config/
│   └── default.yaml                     # デフォルト設定ファイル
├── Cargo.toml
└── Cargo.lock
```

### Cargo.toml

> 共通依存は [Rust共通実装.md](../_common/Rust共通実装.md#共通cargo依存) を参照。サービス固有の追加依存:

```toml
[package]
name = "k1s0-order-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = { version = "0.8", features = ["macros", "multipart"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# DB
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate"] }

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
async-trait = "0.1"

# Logging / Tracing
tracing = "0.1"

# Validation
validator = { version = "0.18", features = ["derive"] }

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

# k1s0 internal libraries
k1s0-telemetry = { path = "../../../../../system/library/rust/telemetry", features = ["full"] }
k1s0-auth = { path = "../../../../../system/library/rust/auth" }
k1s0-server-common = { path = "../../../../../system/library/rust/server-common", features = ["axum"] }

[features]
db-tests = []

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
tower = { version = "0.5", features = ["util"] }
axum-test = "17"
```

---

## lib.rs

マイグレーションファイルのパスを `MIGRATOR` として公開する。マイグレーション SQL は `regions/service/order/database/postgres/migrations/` に配置する。

```rust
pub mod adapter;
pub mod domain;
pub mod infrastructure;
pub mod usecase;

pub static MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../../database/postgres/migrations");
```

---

## src/main.rs 起動シーケンスと DI

> 起動シーケンスは [Rust共通実装.md](../_common/Rust共通実装.md#共通mainrs) を参照。以下はサービス固有の DI:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Configuration — CONFIG_PATH 環境変数またはデフォルト
    let cfg = Config::load(&config_path)?;

    // 2. Telemetry — k1s0-telemetry による初期化
    k1s0_telemetry::init_telemetry(&telemetry_cfg)?;

    // 3. Database — sqlx PgPool + マイグレーション自動適用
    let db_pool = connect_database(db_cfg).await?;
    MIGRATOR.run(&db_pool).await?;

    // 4. Metrics — k1s0-telemetry Prometheus メトリクス
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("order"));

    // 5. Repository — OrderPostgresRepository
    let order_repo: Arc<dyn OrderRepository> =
        Arc::new(OrderPostgresRepository::new(db_pool.clone()));

    // 6. Kafka Producer — 接続失敗時は NoopOrderEventPublisher にフォールバック
    let event_publisher: Arc<dyn OrderEventPublisher> = match cfg.kafka {
        Some(ref kafka_cfg) => match OrderKafkaProducer::new(kafka_cfg) {
            Ok(producer) => Arc::new(producer),
            Err(_) => Arc::new(NoopOrderEventPublisher),
        },
        None => Arc::new(NoopOrderEventPublisher),
    };

    // 7. Use Cases
    let create_order_uc = Arc::new(CreateOrderUseCase::new(
        order_repo.clone(), event_publisher.clone(),
    ));
    let get_order_uc = Arc::new(GetOrderUseCase::new(order_repo.clone()));
    let update_order_status_uc = Arc::new(UpdateOrderStatusUseCase::new(
        order_repo.clone(), event_publisher.clone(),
    ));
    let list_orders_uc = Arc::new(ListOrdersUseCase::new(order_repo.clone()));

    // 8. Auth — JWKS ベース JWT 検証（auth 設定がない場合は認証なし）
    let auth_state = cfg.auth.as_ref().map(|auth_cfg| AuthState {
        verifier: Arc::new(JwksVerifier::new(...)),
    });

    // 9. AppState + Router
    let state = AppState {
        create_order_uc, get_order_uc, update_order_status_uc, list_orders_uc,
        metrics, auth_state,
    };
    let app = handler::router(state);

    // 10. REST server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
}
```

---

## ドメインモデル実装（Rust）

### OrderStatus

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

impl OrderStatus {
    /// ステータス遷移が有効かどうかを検証する。
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Pending, Self::Confirmed)
                | (Self::Pending, Self::Cancelled)
                | (Self::Confirmed, Self::Processing)
                | (Self::Confirmed, Self::Cancelled)
                | (Self::Processing, Self::Shipped)
                | (Self::Processing, Self::Cancelled)
                | (Self::Shipped, Self::Delivered)
        )
    }
}
```

### Order / OrderItem

```rust
pub struct Order {
    pub id: Uuid,
    pub customer_id: String,
    pub status: OrderStatus,
    pub total_amount: i64,
    pub currency: String,
    pub notes: Option<String>,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: i64,
    pub subtotal: i64,
    pub created_at: DateTime<Utc>,
}
```

### OrderError

型安全なエラー分類で HTTP ステータスコードを決定する。`ServiceError` への `From` 実装により、ハンドラーでのエラー変換を簡潔にする。

```rust
#[derive(Debug, thiserror::Error)]
pub enum OrderError {
    #[error("Order '{0}' not found")]
    NotFound(String),                                     // → 404

    #[error("invalid status transition: '{from}' -> '{to}'")]
    InvalidStatusTransition { from: String, to: String }, // → 400

    #[error("validation failed: {0}")]
    ValidationFailed(String),                             // → 400

    #[error("version conflict for order '{0}'")]
    VersionConflict(String),                              // → 409

    #[error("internal error: {0}")]
    Internal(String),                                     // → 500
}
```

---

## リポジトリトレイト実装（Rust）

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Order>>;
    async fn find_items_by_order_id(&self, order_id: Uuid) -> anyhow::Result<Vec<OrderItem>>;
    async fn find_all(&self, filter: &OrderFilter) -> anyhow::Result<Vec<Order>>;
    async fn count(&self, filter: &OrderFilter) -> anyhow::Result<i64>;
    async fn create(&self, input: &CreateOrder, created_by: &str)
        -> anyhow::Result<(Order, Vec<OrderItem>)>;
    async fn update_status(
        &self, id: Uuid, status: &OrderStatus, updated_by: &str, expected_version: i32,
    ) -> anyhow::Result<Order>;
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
}
```

`mockall::automock` により、テスト時にモックリポジトリ `MockOrderRepository` が自動生成される。

---

## ドメインサービス実装（Rust）

`OrderDomainService` は純粋なドメインロジックを提供する。副作用（DB・Kafka）を持たず、ユースケースから呼び出される。

```rust
pub struct OrderDomainService;

impl OrderDomainService {
    /// 注文作成入力を検証する。
    pub fn validate_create_order(input: &CreateOrder) -> Result<(), OrderError> { ... }

    /// ステータス遷移を検証する。
    pub fn validate_status_transition(
        current: &OrderStatus, next: &OrderStatus,
    ) -> Result<(), OrderError> { ... }

    /// 注文明細から合計金額を計算する。
    pub fn calculate_total(items: &[CreateOrderItem]) -> i64 { ... }
}
```

バリデーションルール:
- `customer_id` が空でないこと
- `currency` が空でないこと
- `items` が 1 件以上あること
- 各明細の `product_id`, `product_name` が空でないこと
- 各明細の `quantity` が 1 以上であること
- 各明細の `unit_price` が 0 以上であること

---

## ユースケース実装（Rust）

### CreateOrderUseCase

```rust
pub struct CreateOrderUseCase {
    order_repo: Arc<dyn OrderRepository>,
    event_publisher: Arc<dyn OrderEventPublisher>,
}

impl CreateOrderUseCase {
    pub async fn execute(
        &self, input: &CreateOrder, created_by: &str,
    ) -> anyhow::Result<(Order, Vec<OrderItem>)> {
        // 1. ドメインバリデーション
        OrderDomainService::validate_create_order(input)?;
        // 2. 永続化（トランザクション内で orders + order_items を INSERT）
        let (order, items) = self.order_repo.create(input, created_by).await?;
        // 3. order.created イベント発行（失敗してもエラーにしない）
        self.publish_created_event(&order, &items).await;
        Ok((order, items))
    }
}
```

### UpdateOrderStatusUseCase

```rust
pub struct UpdateOrderStatusUseCase {
    order_repo: Arc<dyn OrderRepository>,
    event_publisher: Arc<dyn OrderEventPublisher>,
}

impl UpdateOrderStatusUseCase {
    pub async fn execute(
        &self, order_id: Uuid, new_status: &OrderStatus, actor: &str,
    ) -> anyhow::Result<Order> {
        // 1. 既存注文を取得
        let existing = self.order_repo.find_by_id(order_id).await?
            .ok_or_else(|| OrderError::NotFound(order_id.to_string()))?;
        // 2. ステータス遷移バリデーション
        OrderDomainService::validate_status_transition(&existing.status, new_status)?;
        // 3. 楽観的ロック付き更新
        let updated = self.order_repo
            .update_status(order_id, new_status, actor, existing.version)
            .await?;
        // 4. cancelled → order.cancelled, それ以外 → order.updated イベント発行
        if *new_status == OrderStatus::Cancelled {
            self.publish_cancelled_event(&updated, actor).await;
        } else {
            self.publish_updated_event(&updated, actor).await;
        }
        Ok(updated)
    }
}
```

### OrderEventPublisher トレイト

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait OrderEventPublisher: Send + Sync {
    async fn publish_order_created(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_order_updated(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_order_cancelled(&self, event: &Value) -> anyhow::Result<()>;
}
```

`NoopOrderEventPublisher` は Kafka 未設定時のフォールバック実装（全メソッドが `Ok(())` を返す）。

---

## REST ハンドラー実装（Rust）

### ルーティング

```rust
pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics_handler));

    let api_routes = if let Some(ref auth_state) = state.auth_state {
        // 認証有効時: HTTP メソッドに基づいて read/write 権限を自動判定
        // NOTE: 同一パスの GET/POST/PUT/DELETE は1つの .route() に統合すること。
        // 別ルーターに分割して merge すると Axum がルートを上書きし 404 になる。
        let api = Router::new()
            .route("/api/v1/orders", get(list_orders).post(create_order))
            .route(
                "/api/v1/orders/{order_id}",
                get(get_order),
            )
            .route(
                "/api/v1/orders/{order_id}/status",
                put(update_order_status),
            )
            .route_layer(make_method_rbac_middleware("order"));

        api
            .layer(from_fn_with_state(auth_state.clone(), auth_middleware))
    } else {
        // 認証無効時: 全エンドポイントを認証なしで公開
        // ...
    };

    public_routes.merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
```

### エラーハンドリング

`order_handler.rs` の `map_order_error` 関数が `anyhow::Error` を `ServiceError` に変換する。`OrderError` にダウンキャスト可能な場合は型安全に変換し、それ以外は `SVC_ORDER_INTERNAL_ERROR` として扱う。

### actor_from_claims

JWT Claims からアクター（操作者）を特定する優先順位:
1. `preferred_username`
2. `email`
3. `sub`
4. フォールバック: `"system"`

---

## Presenter 実装（Rust）

ドメインエンティティを API レスポンス形式に変換する。

| Presenter | 用途 | 主要フィールド |
| --- | --- | --- |
| `OrderDetailResponse` | 単体取得・作成・更新レスポンス | 全フィールド + items |
| `OrderListResponse` | 一覧取得レスポンス | orders (サマリ) + total |
| `OrderSummaryResponse` | 一覧内の各注文サマリ | id, customer_id, status, total_amount, currency, created_at, updated_at |
| `OrderItemResponse` | 明細レスポンス | id, product_id, product_name, quantity, unit_price, subtotal |

---

## Infrastructure 実装（Rust）

### OrderPostgresRepository

- `create`: トランザクション内で `orders` と `order_items` を INSERT。合計金額は `OrderDomainService::calculate_total` で計算
- `update_status`: 楽観的ロック付き UPDATE（`WHERE id = $1 AND version = $5`）。バージョン不一致時はエラー
- `find_all`: `customer_id` と `status` による任意フィルタ、`LIMIT/OFFSET` ページネーション
- `delete`: トランザクション内で `order_items` → `orders` の順に物理削除

### OrderKafkaProducer

- `rdkafka::FutureProducer` を使用
- `acks=all`、`message.timeout.ms=5000`
- メッセージキー: イベントの `order_id` フィールド
- トピックは config.yaml の `kafka.order_created_topic` / `order_updated_topic` / `order_cancelled_topic` で設定

### Config ローダー

- `serde_yaml` で YAML を読み込み、`Config::validate()` でバリデーション
- `database` と `kafka` は `Option` 型（`kafka` 未設定でも起動可能）
- `auth` も `Option` 型（開発環境では認証なしで動作可能）
- 環境変数 `CONFIG_PATH` でファイルパスを指定可能（デフォルト: `config/default.yaml`）
- 環境変数 `DATABASE_URL` で DB 接続文字列を直接指定可能

---

## テスト

### 単体テスト

| テスト対象 | ファイル | テスト数 | 内容 |
| --- | --- | --- | --- |
| OrderStatus | `domain/entity/order.rs` | 5 | roundtrip, invalid parse, valid/invalid transitions, subtotal calculation |
| OrderDomainService | `domain/service/order_service.rs` | 7 | create validation (success, empty customer, no items, zero quantity), calculate_total, status transition (valid/invalid) |
| CreateOrderUseCase | `usecase/create_order.rs` | 2 | success, validation failure |
| UpdateOrderStatusUseCase | `usecase/update_order_status.rs` | 3 | success, invalid transition, cancel |
| OrderEventPublisher | `usecase/event_publisher.rs` | 3 | noop publisher (created, updated, cancelled) |
| OrderPresenter | `adapter/presenter/order_presenter.rs` | 3 | detail response, list response, summary response |

テストでは `MockOrderRepository` と `MockOrderEventPublisher`（mockall 生成）を使用し、外部依存なしで実行可能。

### インテグレーションテスト

`db-tests` feature フラグで DB テストを有効化:

```bash
cargo test --features db-tests
```

---

## 関連ドキュメント

- [service-order-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [service-order-database.md](database.md) -- データベーススキーマ・マイグレーション
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- Linter・Formatter・命名規則
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通 Cargo 依存・build.rs・config.yaml
