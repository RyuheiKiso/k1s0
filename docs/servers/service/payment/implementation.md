# service-payment-server 実装設計

service-payment-server の Rust 実装構成を定義する。概要・API 定義・アーキテクチャは [service-payment-server.md](server.md) を参照。

---

## Rust 実装 (regions/service/payment/server/rust/payment/)

### ディレクトリ構成

```
regions/service/payment/server/rust/payment/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── lib.rs                           # ライブラリルート（MIGRATOR 定義）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── payment.rs              # Payment, PaymentStatus, InitiatePayment, PaymentFilter
│   │   │   └── outbox.rs               # OutboxEvent
│   │   └── repository/
│   │       ├── mod.rs
│   │       └── payment_repository.rs    # PaymentRepository トレイト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── initiate_payment.rs          # InitiatePaymentUseCase
│   │   ├── get_payment.rs               # GetPaymentUseCase
│   │   ├── list_payments.rs             # ListPaymentsUseCase
│   │   ├── complete_payment.rs          # CompletePaymentUseCase
│   │   ├── fail_payment.rs              # FailPaymentUseCase
│   │   └── refund_payment.rs            # RefundPaymentUseCase
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs                   # AppState, router()
│   │   │   ├── payment_handler.rs       # 決済 REST ハンドラー
│   │   │   └── health.rs               # ヘルスチェックハンドラー
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   └── payment_grpc.rs          # gRPC ハンドラー
│   │   └── middleware/
│   │       ├── mod.rs
│   │       ├── auth.rs                  # JWT 認証ミドルウェア
│   │       └── rbac.rs                  # RBAC ミドルウェア
│   └── infrastructure/
│       ├── mod.rs
│       ├── config.rs                    # Config ローダー
│       ├── startup.rs                   # DI + サーバー起動
│       ├── payment_postgres.rs          # PaymentPostgresRepository
│       ├── kafka_producer.rs            # PaymentKafkaProducer
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
name = "k1s0-payment-server"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.8", features = ["macros"] }
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
axum-test = "17"
```

---

## ドメインモデル実装（Rust）

### PaymentStatus

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Initiated,
    Completed,
    Failed,
    Refunded,
}

impl PaymentStatus {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Initiated, Self::Completed)
                | (Self::Initiated, Self::Failed)
                | (Self::Completed, Self::Refunded)
        )
    }
}
```

### Payment

```rust
pub struct Payment {
    pub id: Uuid,
    pub order_id: String,
    pub customer_id: String,
    pub amount: i64,
    pub currency: String,
    pub status: PaymentStatus,
    pub payment_method: Option<String>,
    pub transaction_id: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
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
pub trait PaymentRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Payment>>;
    async fn find_all(&self, filter: &PaymentFilter) -> anyhow::Result<Vec<Payment>>;
    async fn count(&self, filter: &PaymentFilter) -> anyhow::Result<i64>;
    async fn create(&self, input: &InitiatePayment) -> anyhow::Result<Payment>;
    async fn complete(
        &self, id: Uuid, transaction_id: &str, expected_version: i32,
    ) -> anyhow::Result<Payment>;
    async fn fail(
        &self, id: Uuid, error_code: &str, error_message: &str, expected_version: i32,
    ) -> anyhow::Result<Payment>;
    async fn refund(&self, id: Uuid, expected_version: i32) -> anyhow::Result<Payment>;
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

### InitiatePaymentUseCase

```rust
pub struct InitiatePaymentUseCase {
    repo: Arc<dyn PaymentRepository>,
}

impl InitiatePaymentUseCase {
    pub async fn execute(&self, input: &InitiatePayment) -> anyhow::Result<Payment> {
        // 1. バリデーション（amount > 0, order_id/customer_id 非空）
        // 2. 決済レコード作成（status: initiated）
        // 3. Outbox イベント記録（payment.initiated）
        // 4. 作成した決済を返却
    }
}
```

### CompletePaymentUseCase

決済完了。`initiated` → `completed` の遷移のみ許可。`transaction_id` を記録。楽観的ロック付き。

### FailPaymentUseCase

決済失敗。`initiated` → `failed` の遷移のみ許可。`error_code` / `error_message` を記録。

### RefundPaymentUseCase

決済返金。`completed` → `refunded` の遷移のみ許可。

---

## テスト

### 単体テスト

テストでは `MockPaymentRepository`（mockall 生成）を使用し、外部依存なしで実行可能。

| テスト対象 | 内容 |
| --- | --- |
| PaymentStatus | ステータス遷移の有効/無効パターン |
| InitiatePaymentUseCase | 正常開始、バリデーションエラー |
| CompletePaymentUseCase | 正常完了、不正な遷移（failed → completed） |
| FailPaymentUseCase | 正常失敗、不正な遷移（completed → failed） |
| RefundPaymentUseCase | 正常返金、不正な遷移（initiated → refunded） |

---

## Doc Sync (2026-03-21)

### gRPC 全ハンドラー 認証チェック追加 [技術品質監査 Critical 2-1]

**背景・問題**

`payment_grpc.rs` の全 6 ハンドラーで認証チェックが未実装であった。
Claims を取得せずに処理を続行していたため、未認証リクエストを受け入れる可能性があった。

また `payment_handler.rs` の REST ハンドラーでは `let _actor` と変数名にアンダースコアを付けており、
actor 情報をログ出力に使用していなかった。

**対応内容**

**gRPC ハンドラー（`payment_grpc.rs`）:**

全 6 ハンドラー（`initiate_payment`, `get_payment`, `list_payments`, `complete_payment`, `fail_payment`, `refund_payment`）に認証チェックを追加。

```rust
// 書き込み系ハンドラー（initiate / complete / fail / refund）
let claims: &Claims = request
    .extensions()
    .get()
    .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
let _actor = actor_from_claims(Some(claims));

// 読み取り系ハンドラー（get / list）
request
    .extensions()
    .get::<Claims>()
    .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
```

**REST ハンドラー（`payment_handler.rs`）:**

全 6 ハンドラーで認証不備を修正した：

- `get_payment`・`list_payments`：`Claims` パラメーター自体が欠落していたため追加した
- 全 6 ハンドラー：`Option<Extension<Claims>>` に対して `.ok_or_else(|| ServiceError::unauthorized(...))?` を適用し、
  未認証時に確実に 401 を返すよう変更した

```rust
// write 系
let claims = claims
    .ok_or_else(|| ServiceError::unauthorized("PAYMENT", "authentication required"))?;
let actor = actor_from_claims(Some(&claims.0));
tracing::info!(actor = %actor, "initiate_payment invoked");

// read 系（get_payment / list_payments）
claims.ok_or_else(|| ServiceError::unauthorized("PAYMENT", "authentication required"))?;
```

**影響範囲**

- `src/adapter/grpc/payment_grpc.rs`（全 6 ハンドラー）
- `src/adapter/handler/payment_handler.rs`（全 6 ハンドラー）

---

## 関連ドキュメント

- [service-payment-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [service-payment-database.md](database.md) -- データベーススキーマ・マイグレーション
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通 Cargo 依存・config.yaml
