# service-activity-server 実装設計

service-activity-server の Rust 実装構成を定義する。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## Rust 実装 (regions/service/activity/server/rust/activity/)

### ディレクトリ構成

```
regions/service/activity/server/rust/activity/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── lib.rs                           # ライブラリルート（MIGRATOR 定義）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── error.rs                     # ActivityError（thiserror ベース）
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── activity.rs              # Activity, ActivityStatus, ActivityType, CreateActivity, ActivityFilter
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   └── activity_repository.rs   # ActivityRepository トレイト（find_by_idempotency_key 含む）
│   │   └── service/
│   │       ├── mod.rs
│   │       └── activity_service.rs      # ActivityDomainService（バリデーション・ステータス遷移）
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_activity.rs           # CreateActivityUseCase（冪等性チェック）
│   │   ├── get_activity.rs              # GetActivityUseCase
│   │   ├── list_activities.rs           # ListActivitiesUseCase
│   │   ├── submit_activity.rs           # SubmitActivityUseCase（active → submitted）
│   │   ├── approve_activity.rs          # ApproveActivityUseCase（submitted → approved）
│   │   ├── reject_activity.rs           # RejectActivityUseCase（submitted → rejected）
│   │   └── event_publisher.rs           # ActivityEventPublisher トレイト + NoopActivityEventPublisher
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs                   # AppState, router(), actor_from_claims()
│   │   │   ├── activity_handler.rs      # アクティビティ REST ハンドラー
│   │   │   └── health.rs               # ヘルスチェックハンドラー
│   │   ├── presenter/
│   │   │   ├── mod.rs
│   │   │   └── activity_presenter.rs    # ActivityDetailResponse, ActivityListResponse
│   │   └── middleware/
│   │       ├── mod.rs
│   │       └── auth.rs                  # JWT 認証ミドルウェア（k1s0-server-common 経由）
│   └── infrastructure/
│       ├── mod.rs
│       ├── config.rs                    # Config ローダー・バリデーション
│       ├── database/
│       │   ├── mod.rs
│       │   └── activity_repository.rs   # ActivityPostgresRepository（sqlx 実装）
│       └── kafka/
│           ├── mod.rs
│           └── activity_producer.rs     # ActivityKafkaProducer（rdkafka 実装）
├── config/
│   └── default.yaml                     # デフォルト設定ファイル
├── Cargo.toml
└── Cargo.lock
```

### Cargo.toml

```toml
[package]
name = "k1s0-activity-server"
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
rdkafka = { version = "0.37", features = ["cmake-build"] }

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

```rust
pub mod adapter;
pub mod domain;
pub mod infrastructure;
pub mod usecase;

pub static MIGRATOR: sqlx::migrate::Migrator =
    sqlx::migrate!("../../../database/postgres/migrations");
```

---

## ドメインモデル実装（Rust）

### ActivityStatus

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityStatus {
    Active,
    Submitted,
    Approved,
    Rejected,
}

impl ActivityStatus {
    /// ステータス遷移が有効かどうかを検証する。
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Active, Self::Submitted)
                | (Self::Submitted, Self::Approved)
                | (Self::Submitted, Self::Rejected)
        )
    }
}
```

`Approved` と `Rejected` は終端ステータス（これ以上遷移できない）。

### ActivityType

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Comment,
    TimeEntry,
    StatusChange,
    Assignment,
}
```

### Activity

```rust
pub struct Activity {
    pub id: Uuid,
    pub task_id: String,
    pub actor_id: String,
    pub activity_type: ActivityType,
    pub content: Option<String>,
    pub duration_minutes: Option<i32>,
    pub status: ActivityStatus,
    pub metadata: Option<serde_json::Value>,
    pub idempotency_key: Option<String>,
    pub tenant_id: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### ActivityError

```rust
#[derive(Debug, thiserror::Error)]
pub enum ActivityError {
    #[error("Activity '{0}' not found")]
    NotFound(String),                                              // → 404

    #[error("invalid status transition: '{from}' -> '{to}'")]
    InvalidStatusTransition { from: String, to: String },         // → 400

    #[error("validation failed: {0}")]
    ValidationFailed(String),                                      // → 400

    #[error("duplicate idempotency key: '{0}'")]
    DuplicateIdempotencyKey(String),                              // → 200（既存レコード返却）

    #[error("internal error: {0}")]
    Internal(String),                                              // → 500
}
```

---

## リポジトリトレイト実装（Rust）

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ActivityRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Activity>>;
    async fn find_by_idempotency_key(
        &self, key: &str,
    ) -> anyhow::Result<Option<Activity>>;
    async fn find_all(&self, filter: &ActivityFilter) -> anyhow::Result<Vec<Activity>>;
    async fn count(&self, filter: &ActivityFilter) -> anyhow::Result<i64>;
    async fn create(&self, input: &CreateActivity) -> anyhow::Result<Activity>;
    async fn update_status(
        &self, id: Uuid, status: &ActivityStatus, expected_version: i32,
    ) -> anyhow::Result<Activity>;
}
```

---

## ユースケース実装（Rust）

### CreateActivityUseCase（冪等性チェック付き）

```rust
impl CreateActivityUseCase {
    pub async fn execute(
        &self, input: &CreateActivity,
    ) -> anyhow::Result<(Activity, bool)> {
        // 1. idempotency_key が指定されている場合は既存レコードを検索
        if let Some(ref key) = input.idempotency_key {
            if let Some(existing) = self.repo.find_by_idempotency_key(key).await? {
                // 既存レコードを返す（bool=true で重複を通知）
                return Ok((existing, true));
            }
        }
        // 2. ドメインバリデーション
        ActivityDomainService::validate_create_activity(input)?;
        // 3. 永続化（同一トランザクションで activities + outbox_events を INSERT）
        let activity = self.repo.create(input).await?;
        // 4. activity.created イベント発行（失敗してもエラーにしない）
        self.publish_created_event(&activity).await;
        Ok((activity, false))
    }
}
```

ハンドラー層では `bool=true` の場合に HTTP 200 + `SVC_ACTIVITY_DUPLICATE_IDEMPOTENCY_KEY` を返す。

### ApproveActivityUseCase

```rust
impl ApproveActivityUseCase {
    pub async fn execute(
        &self, activity_id: Uuid, actor: &str,
    ) -> anyhow::Result<Activity> {
        // 1. 対象アクティビティを取得
        let existing = self.repo.find_by_id(activity_id).await?
            .ok_or_else(|| ActivityError::NotFound(activity_id.to_string()))?;
        // 2. ステータス遷移バリデーション（submitted → approved のみ許可）
        ActivityDomainService::validate_status_transition(
            &existing.status, &ActivityStatus::Approved,
        )?;
        // 3. 楽観的ロック付き更新
        let updated = self.repo
            .update_status(activity_id, &ActivityStatus::Approved, existing.version)
            .await?;
        // 4. activity.approved イベント発行（Outbox）
        self.publish_approved_event(&updated, actor).await;
        Ok(updated)
    }
}
```

### ActivityEventPublisher トレイト

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ActivityEventPublisher: Send + Sync {
    async fn publish_activity_created(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_activity_approved(&self, event: &Value) -> anyhow::Result<()>;
}
```

`NoopActivityEventPublisher` は Kafka 未設定時のフォールバック実装（全メソッドが `Ok(())` を返す）。

---

## REST ハンドラー実装（Rust）

### ルーティング

```rust
pub fn router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/healthz", get(health::healthz))
        .route("/readyz", get(health::readyz))
        .route("/metrics", get(health::metrics_handler));

    let api_routes = Router::new()
        .route("/api/v1/activities", get(list_activities).post(create_activity))
        .route("/api/v1/activities/{activity_id}", get(get_activity))
        .route("/api/v1/activities/{activity_id}/submit", post(submit_activity))
        .route("/api/v1/activities/{activity_id}/approve", post(approve_activity))
        .route("/api/v1/activities/{activity_id}/reject", post(reject_activity))
        .route_layer(make_method_rbac_middleware("activity"))
        .layer(from_fn_with_state(auth_state.clone(), auth_middleware));

    public_routes.merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
```

### Infrastructure 実装（Rust）

#### ActivityPostgresRepository

- `create`: トランザクション内で `activities` テーブルに INSERT し、同時に `outbox_events` にイベントを書き込む（Outbox pattern）
- 冪等性: `idempotency_key` の UNIQUE 制約を利用。INSERT 前に `find_by_idempotency_key` でチェック
- `update_status`: 楽観的ロック付き UPDATE（`WHERE id = $1 AND version = $expected`）。バージョン不一致時はエラー
- `find_all`: `task_id`, `actor_id`, `activity_type`, `status` による任意フィルタ、`LIMIT/OFFSET` ページネーション

#### ActivityKafkaProducer

- `rdkafka::FutureProducer` を使用
- `acks=all`、`message.timeout.ms=5000`
- メッセージキー: イベントの `activity_id` フィールド

#### Outbox パターン実装状況と k1s0-outbox 移行計画（SUP-06）

**現在の実装**:
`ActivityPostgresRepository::create` がトランザクション内で `activities` テーブルへの INSERT と同時に `outbox_events` テーブルへ直接書き込んでいる。Outbox イベントのポーリング・Kafka 送信はリポジトリ実装内の独自ロジックで行っている。

**移行 TODO**:
`k1s0-outbox::OutboxEventPoller` への移行が TODO として残っている。移行することで以下が保証される。

- Transactional Outbox パターンの完全な整合性保証（Kafka との結果整合性）
- 冪等性制御と Kafka 配信の統合管理
- ポーリング間隔・リトライ戦略の一元管理

移行が完了するまでは、現在の `outbox_events` への直接書き込み方式を維持する。

---

## テスト

### 単体テスト

| テスト対象 | ファイル | 内容 |
| --- | --- | --- |
| ActivityStatus | `domain/entity/activity.rs` | roundtrip, invalid parse, valid/invalid transitions |
| ActivityDomainService | `domain/service/activity_service.rs` | create validation, status transition (valid/invalid) |
| CreateActivityUseCase | `usecase/create_activity.rs` | 新規作成・冪等性キー重複・バリデーション失敗 |
| SubmitActivityUseCase | `usecase/submit_activity.rs` | 成功・不正遷移（approved から再 submit 等） |
| ApproveActivityUseCase | `usecase/approve_activity.rs` | 成功・不正遷移 |
| RejectActivityUseCase | `usecase/reject_activity.rs` | 成功・不正遷移 |
| ActivityEventPublisher | `usecase/event_publisher.rs` | noop publisher (created, approved) |
| ActivityPresenter | `adapter/presenter/activity_presenter.rs` | detail response, list response |

### 実 DB 統合テスト

`tests/integration_db_test.rs` に `#[ignore]` 属性付きの統合テストを配置する。

```bash
# ローカルで実行する場合（DATABASE_URL 要設定）
DATABASE_URL=postgres://postgres:postgres@localhost:5432/test_db \
  cargo test --all -- --include-ignored
```

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [database.md](database.md) -- データベーススキーマ・マイグレーション
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
