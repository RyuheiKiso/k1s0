# service-inventory-server 実装設計

service-inventory-server の Rust 実装構成を定義する。概要・API 定義・アーキテクチャは [service-inventory-server.md](server.md) を参照。

---

## Rust 実装 (regions/service/inventory/server/rust/inventory/)

### ディレクトリ構成

```
regions/service/inventory/server/rust/inventory/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── lib.rs                           # ライブラリルート（MIGRATOR 定義）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── inventory.rs             # InventoryItem, InventoryFilter
│   │   └── repository/
│   │       ├── mod.rs
│   │       └── inventory_repository.rs  # InventoryRepository トレイト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── reserve_stock.rs             # ReserveStockUseCase
│   │   ├── release_stock.rs             # ReleaseStockUseCase
│   │   ├── get_inventory.rs             # GetInventoryUseCase
│   │   ├── list_inventory.rs            # ListInventoryUseCase
│   │   └── update_stock.rs              # UpdateStockUseCase
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs                   # AppState, router()
│   │   │   ├── inventory_handler.rs     # 在庫 REST ハンドラー
│   │   │   └── health.rs               # ヘルスチェックハンドラー
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   └── inventory_grpc.rs        # gRPC ハンドラー
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                  # JWT 認証ミドルウェア
│   │       └── rbac.rs                  # RBAC ミドルウェア
│   └── infrastructure/
│       ├── mod.rs
│       ├── config.rs                    # Config ローダー
│       ├── startup.rs                   # DI + サーバー起動
│       ├── inventory_postgres.rs        # InventoryPostgresRepository
│       ├── kafka_producer.rs            # InventoryKafkaProducer
│       └── outbox_poller.rs             # OutboxPoller
├── config/
│   └── default.yaml                     # デフォルト設定ファイル
├── Cargo.toml
└── Cargo.lock
```

### Cargo.toml

> 共通依存は [Rust共通実装.md](../_common/Rust共通実装.md#共通cargo依存) を参照。サービス固有の追加依存:

```toml
[package]
name = "k1s0-inventory-server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }
tonic = "0.12"
prost = "0.13"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
async-trait = "0.1"
tracing = "0.1"
rdkafka = { version = "0.36", features = ["cmake-build"] }
k1s0-telemetry = { path = "../../../../../system/library/rust/telemetry", features = ["full"] }
k1s0-auth = { path = "../../../../../system/library/rust/auth" }
k1s0-server-common = { path = "../../../../../system/library/rust/server-common", features = ["axum"] }

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
tower = { version = "0.5", features = ["util"] }
axum-test = "16"
```

---

## ドメインモデル実装（Rust）

### InventoryItem

```rust
pub struct InventoryItem {
    pub id: Uuid,
    pub product_id: String,
    pub warehouse_id: String,
    pub qty_available: i32,
    pub qty_reserved: i32,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

---

## リポジトリトレイト実装（Rust）

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<InventoryItem>>;
    async fn find_by_product_and_warehouse(
        &self, product_id: &str, warehouse_id: &str,
    ) -> anyhow::Result<Option<InventoryItem>>;
    async fn find_all(&self, filter: &InventoryFilter) -> anyhow::Result<Vec<InventoryItem>>;
    async fn count(&self, filter: &InventoryFilter) -> anyhow::Result<i64>;
    async fn reserve(
        &self, product_id: &str, warehouse_id: &str, quantity: i32,
    ) -> anyhow::Result<InventoryItem>;
    async fn release(
        &self, product_id: &str, warehouse_id: &str, quantity: i32,
    ) -> anyhow::Result<InventoryItem>;
    async fn update_stock(
        &self, id: Uuid, qty_available: i32, expected_version: i32,
    ) -> anyhow::Result<InventoryItem>;
    async fn insert_outbox_event(
        &self, aggregate_type: &str, aggregate_id: &str,
        event_type: &str, payload: &serde_json::Value,
    ) -> anyhow::Result<()>;
    async fn fetch_unpublished_events(&self, limit: i64) -> anyhow::Result<Vec<OutboxEvent>>;
    async fn mark_event_published(&self, event_id: Uuid) -> anyhow::Result<()>;
}
```

---

## ユースケース実装（Rust）

### ReserveStockUseCase

```rust
pub struct ReserveStockUseCase {
    repo: Arc<dyn InventoryRepository>,
}

impl ReserveStockUseCase {
    pub async fn execute(&self, input: &ReserveStockInput) -> anyhow::Result<InventoryItem> {
        // 1. バリデーション（quantity > 0）
        // 2. 在庫予約（repo.reserve）— DB の CHECK 制約で在庫不足を検知
        // 3. Outbox イベント記録（inventory.reserved）
        // 4. 更新後の在庫を返却
    }
}
```

### ReleaseStockUseCase

予約済み在庫の解放。`repo.release` で `qty_reserved` から `qty_available` へ移動。

### UpdateStockUseCase

楽観的ロック付きの在庫数量直接更新。`expected_version` 不一致時はエラー。

---

## テスト

### 単体テスト

テストでは `MockInventoryRepository`（mockall 生成）を使用し、外部依存なしで実行可能。

| テスト対象 | 内容 |
| --- | --- |
| ReserveStockUseCase | 正常予約、在庫不足エラー、バリデーションエラー |
| ReleaseStockUseCase | 正常解放、在庫アイテム未存在 |
| UpdateStockUseCase | 正常更新、バージョン競合 |

---

## 関連ドキュメント

- [service-inventory-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [service-inventory-database.md](database.md) -- データベーススキーマ・マイグレーション
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通 Cargo 依存・config.yaml
