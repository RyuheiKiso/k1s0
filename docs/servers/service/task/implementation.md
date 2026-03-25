# service-task-server 実装設計

service-task-server の Rust 実装構成を定義する。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## Rust 実装 (regions/service/task/server/rust/task/)

### ディレクトリ構成

```
regions/service/task/server/rust/task/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── lib.rs                           # ライブラリルート（MIGRATOR 定義）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── error.rs                     # TaskError（thiserror ベース）
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── task.rs                  # Task, TaskChecklistItem, TaskStatus, TaskPriority, CreateTask, TaskFilter
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   └── task_repository.rs       # TaskRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       └── task_service.rs          # TaskDomainService（バリデーション・ステータス遷移）
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_task.rs               # CreateTaskUseCase
│   │   ├── get_task.rs                  # GetTaskUseCase
│   │   ├── update_task_status.rs        # UpdateTaskStatusUseCase
│   │   ├── list_tasks.rs                # ListTasksUseCase
│   │   ├── manage_checklist.rs          # ManageChecklistUseCase
│   │   └── event_publisher.rs          # TaskEventPublisher トレイト + NoopTaskEventPublisher
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs                   # AppState, router(), actor_from_claims()
│   │   │   ├── task_handler.rs          # タスク REST ハンドラー
│   │   │   └── health.rs               # ヘルスチェックハンドラー
│   │   ├── presenter/
│   │   │   ├── mod.rs
│   │   │   └── task_presenter.rs        # TaskDetailResponse, TaskListResponse, TaskSummaryResponse
│   │   └── middleware/
│   │       ├── mod.rs
│   │       └── auth.rs                  # JWT 認証ミドルウェア（k1s0-server-common 経由）
│   └── infrastructure/
│       ├── mod.rs
│       ├── config.rs                    # Config ローダー・バリデーション
│       ├── database/
│       │   ├── mod.rs
│       │   └── task_repository.rs       # TaskPostgresRepository（sqlx 実装）
│       └── kafka/
│           ├── mod.rs
│           └── task_producer.rs         # TaskKafkaProducer（rdkafka 実装）
├── config/
│   └── default.yaml                     # デフォルト設定ファイル
├── Cargo.toml
└── Cargo.lock
```

### Cargo.toml

<!-- workspace 依存管理方式に更新し、gRPC 関連依存（tonic・prost・prost-types・k1s0-proto-build・k1s0-correlation・k1s0-outbox）を追加。k1s0-server-common の features を実際の値に修正。D-3/D-5 対応。 -->
```toml
[package]
name = "k1s0-task-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework（workspace 依存管理）
axum = { workspace = true, features = ["macros", "multipart"] }
tokio = { workspace = true, features = ["full"] }
tower = { workspace = true }
tower-http = { workspace = true, features = ["trace", "cors"] }

# gRPC（tonic/prost による gRPC サーバー実装）
tonic = { workspace = true }
prost = { workspace = true }
prost-types = { workspace = true }

# Serialization
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
serde_yaml = { workspace = true }

# DB
sqlx = { workspace = true, features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json", "migrate"] }

# Utilities
uuid = { workspace = true, features = ["v4", "serde"] }
chrono = { workspace = true, features = ["serde"] }
anyhow = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }

# Logging / Tracing
tracing = { workspace = true }

# Validation
validator = { workspace = true, features = ["derive"] }

# Kafka
rdkafka = { workspace = true, features = ["cmake-build"] }

# k1s0 internal libraries
k1s0-telemetry = { path = "../../../../../system/library/rust/telemetry", features = ["full"] }
k1s0-auth = { path = "../../../../../system/library/rust/auth" }
# k1s0-server-common: auth（JWT 検証）・grpc-auth（gRPC 認証）・shutdown（グレースフルシャットダウン）を有効化
k1s0-server-common = { path = "../../../../../system/library/rust/server-common", features = ["auth", "grpc-auth", "shutdown"] }
# proto 生成コード（gRPC クライアント/サーバースタブ）
k1s0-proto-build = { path = "../../../../../system/library/rust/proto-build" }
# 分散トレーシング用相関 ID
k1s0-correlation = { path = "../../../../../system/library/rust/correlation" }
# Outbox パターン（タスクイベントのトランザクション内書き込み）
k1s0-outbox = { path = "../../../../../system/library/rust/outbox" }

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

## src/main.rs 起動シーケンスと DI

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Configuration — CONFIG_PATH 環境変数またはデフォルト
    let cfg = Config::load(&config_path)?;

    // 2. Telemetry — k1s0-telemetry による初期化
    k1s0_telemetry::init_telemetry(&telemetry_cfg)
        .map_err(|e| anyhow::anyhow!("テレメトリ初期化に失敗しました: {}", e))?;

    // 3. Database — sqlx PgPool + advisory lock + マイグレーション自動適用
    let db_pool = connect_database(db_cfg).await?;
    MIGRATOR.run(&db_pool).await?;

    // 4. Metrics — k1s0-telemetry Prometheus メトリクス
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("task"));

    // 5. Repository — TaskPostgresRepository
    let task_repo: Arc<dyn TaskRepository> =
        Arc::new(TaskPostgresRepository::new(db_pool.clone()));

    // 6. Kafka Producer — 接続失敗時は NoopTaskEventPublisher にフォールバック
    let event_publisher: Arc<dyn TaskEventPublisher> = match cfg.kafka {
        Some(ref kafka_cfg) => match TaskKafkaProducer::new(kafka_cfg) {
            Ok(producer) => Arc::new(producer),
            Err(_) => Arc::new(NoopTaskEventPublisher),
        },
        None => Arc::new(NoopTaskEventPublisher),
    };

    // 7. Use Cases
    let create_task_uc = Arc::new(CreateTaskUseCase::new(
        task_repo.clone(), event_publisher.clone(),
    ));
    let get_task_uc = Arc::new(GetTaskUseCase::new(task_repo.clone()));
    let update_task_status_uc = Arc::new(UpdateTaskStatusUseCase::new(
        task_repo.clone(), event_publisher.clone(),
    ));
    let list_tasks_uc = Arc::new(ListTasksUseCase::new(task_repo.clone()));
    let manage_checklist_uc = Arc::new(ManageChecklistUseCase::new(task_repo.clone()));

    // 8. Auth — JWKS ベース JWT 検証
    let auth_state = cfg.auth.as_ref().map(|auth_cfg| AuthState {
        verifier: Arc::new(JwksVerifier::new(...).expect("JWKS verifier 初期化失敗")),
    });

    // 9. AppState + Router
    let state = AppState {
        create_task_uc, get_task_uc, update_task_status_uc, list_tasks_uc,
        manage_checklist_uc, metrics, auth_state,
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

### TaskStatus

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Open,
    InProgress,
    Review,
    Done,
    Cancelled,
}

impl TaskStatus {
    /// ステータス遷移が有効かどうかを検証する。
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Open, Self::InProgress)
                | (Self::Open, Self::Cancelled)
                | (Self::InProgress, Self::Review)
                | (Self::InProgress, Self::Cancelled)
                | (Self::Review, Self::Done)
                | (Self::Review, Self::InProgress)
                | (Self::Review, Self::Cancelled)
        )
    }
}
```

### TaskPriority

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}
```

### Task / TaskChecklistItem

<!-- task.rs の実装と一致するよう project_id を Uuid 型、due_date を Option<DateTime<Utc>> に修正し、reporter_id と labels フィールドを追加。D-3 対応。 -->
```rust
pub struct Task {
    pub id: Uuid,
    // project_id は UUID 型（Uuid 型に修正）
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub assignee_id: Option<String>,
    // タスクを報告したユーザーの ID（proto の reporter_id フィールドに対応）
    pub reporter_id: Option<String>,
    pub priority: TaskPriority,
    // due_date は日時型（Option<NaiveDate> から Option<DateTime<Utc>> に修正）
    pub due_date: Option<DateTime<Utc>>,
    // タスクに付与されたラベル一覧（proto の labels フィールドに対応）
    pub labels: Vec<String>,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct TaskChecklistItem {
    pub id: Uuid,
    pub task_id: Uuid,
    pub title: String,
    pub is_completed: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### TaskError

```rust
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Task '{0}' not found")]
    NotFound(String),                                      // → 404

    #[error("invalid status transition: '{from}' -> '{to}'")]
    InvalidStatusTransition { from: String, to: String },  // → 400

    #[error("validation failed: {0}")]
    ValidationFailed(String),                              // → 400

    #[error("version conflict for task '{0}'")]
    VersionConflict(String),                               // → 409

    #[error("checklist item '{0}' not found")]
    ChecklistItemNotFound(String),                         // → 404

    #[error("internal error: {0}")]
    Internal(String),                                      // → 500
}
```

---

## リポジトリトレイト実装（Rust）

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Task>>;
    async fn find_checklist_by_task_id(&self, task_id: Uuid) -> anyhow::Result<Vec<TaskChecklistItem>>;
    async fn find_all(&self, filter: &TaskFilter) -> anyhow::Result<Vec<Task>>;
    async fn count(&self, filter: &TaskFilter) -> anyhow::Result<i64>;
    async fn create(&self, input: &CreateTask, created_by: &str) -> anyhow::Result<Task>;
    async fn update_status(
        &self, id: Uuid, status: &TaskStatus, updated_by: &str, expected_version: i32,
    ) -> anyhow::Result<Task>;
    async fn create_checklist_item(&self, item: &TaskChecklistItem) -> anyhow::Result<TaskChecklistItem>;
    async fn update_checklist_item(&self, item: &TaskChecklistItem) -> anyhow::Result<TaskChecklistItem>;
    async fn delete_checklist_item(&self, id: Uuid) -> anyhow::Result<()>;
}
```

`mockall::automock` により、テスト時にモックリポジトリ `MockTaskRepository` が自動生成される。

---

## ドメインサービス実装（Rust）

`TaskDomainService` は純粋なドメインロジックを提供する。副作用（DB・Kafka）を持たない。

```rust
pub struct TaskDomainService;

impl TaskDomainService {
    /// タスク作成入力を検証する。
    pub fn validate_create_task(input: &CreateTask) -> Result<(), TaskError> { ... }

    /// ステータス遷移を検証する。
    pub fn validate_status_transition(
        current: &TaskStatus, next: &TaskStatus,
    ) -> Result<(), TaskError> { ... }
}
```

バリデーションルール:
- `project_id` が空でないこと
- `title` が1文字以上であること
- `priority` が有効な値であること

---

## ユースケース実装（Rust）

### CreateTaskUseCase

```rust
pub struct CreateTaskUseCase {
    task_repo: Arc<dyn TaskRepository>,
    event_publisher: Arc<dyn TaskEventPublisher>,
}

impl CreateTaskUseCase {
    pub async fn execute(
        &self, input: &CreateTask, created_by: &str,
    ) -> anyhow::Result<Task> {
        // 1. ドメインバリデーション
        TaskDomainService::validate_create_task(input)?;
        // 2. 永続化（同一トランザクションで tasks + outbox_events を INSERT）
        let task = self.task_repo.create(input, created_by).await?;
        // 3. task.created イベント発行（失敗してもエラーにしない）
        self.publish_created_event(&task).await;
        Ok(task)
    }
}
```

### UpdateTaskStatusUseCase

```rust
impl UpdateTaskStatusUseCase {
    pub async fn execute(
        &self, task_id: Uuid, new_status: &TaskStatus, actor: &str,
    ) -> anyhow::Result<Task> {
        // 1. 既存タスクを取得
        let existing = self.task_repo.find_by_id(task_id).await?
            .ok_or_else(|| TaskError::NotFound(task_id.to_string()))?;
        // 2. ステータス遷移バリデーション
        TaskDomainService::validate_status_transition(&existing.status, new_status)?;
        // 3. 楽観的ロック付き更新（version フィールド）
        let updated = self.task_repo
            .update_status(task_id, new_status, actor, existing.version)
            .await?;
        // 4. cancelled → task.cancelled, それ以外 → task.updated イベント発行（Outbox）
        if *new_status == TaskStatus::Cancelled {
            self.publish_cancelled_event(&updated, actor).await;
        } else {
            self.publish_updated_event(&updated, actor).await;
        }
        Ok(updated)
    }
}
```

### TaskEventPublisher トレイト

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TaskEventPublisher: Send + Sync {
    async fn publish_task_created(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_task_updated(&self, event: &Value) -> anyhow::Result<()>;
    async fn publish_task_cancelled(&self, event: &Value) -> anyhow::Result<()>;
}
```

`NoopTaskEventPublisher` は Kafka 未設定時のフォールバック実装（全メソッドが `Ok(())` を返す）。

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
        .route("/api/v1/tasks", get(list_tasks).post(create_task))
        .route("/api/v1/tasks/{task_id}", get(get_task).put(update_task))
        .route("/api/v1/tasks/{task_id}/status", put(update_task_status))
        .route("/api/v1/tasks/{task_id}/checklist", post(add_checklist_item))
        .route(
            "/api/v1/tasks/{task_id}/checklist/{item_id}",
            put(update_checklist_item).delete(delete_checklist_item),
        )
        .route_layer(make_method_rbac_middleware("task"))
        .layer(from_fn_with_state(auth_state.clone(), auth_middleware));

    public_routes.merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
```

### Infrastructure 実装（Rust）

#### TaskPostgresRepository

- `create`: トランザクション内で `tasks` テーブルに INSERT し、同時に `outbox_events` にイベントを書き込む（Outbox pattern）
- `update_status`: 楽観的ロック付き UPDATE（`WHERE id = $1 AND version = $expected`）。バージョン不一致時はエラー
- `find_all`: `project_id`, `status`, `assignee_id`, `priority` による任意フィルタ、`LIMIT/OFFSET` ページネーション

#### TaskKafkaProducer

- `rdkafka::FutureProducer` を使用
- `acks=all`、`message.timeout.ms=5000`
- メッセージキー: イベントの `task_id` フィールド

#### Outbox パターン実装状況と k1s0-outbox 移行計画（SUP-06）

**現在の実装**:
`TaskPostgresRepository::create` がトランザクション内で `tasks` テーブルへの INSERT と同時に `outbox_events` テーブルへ直接書き込んでいる。Outbox イベントのポーリング・Kafka 送信はリポジトリ実装内の独自ロジックで行っている。

**移行 TODO**:
`Cargo.toml` に `k1s0-outbox` への依存が追加済みである。`k1s0-outbox::OutboxEventPoller` へ移行することで以下が保証される。

- Transactional Outbox パターンの完全な整合性保証
- Kafka への At-Least-Once 配信保証
- ポーリング間隔・リトライ戦略の一元管理

移行が完了するまでは、現在の `outbox_events` への直接書き込み方式を維持する。

---

## テスト

### 単体テスト

| テスト対象 | ファイル | 内容 |
| --- | --- | --- |
| TaskStatus | `domain/entity/task.rs` | roundtrip, invalid parse, valid/invalid transitions |
| TaskDomainService | `domain/service/task_service.rs` | create validation (success, empty title), status transition (valid/invalid) |
| CreateTaskUseCase | `usecase/create_task.rs` | success, validation failure |
| UpdateTaskStatusUseCase | `usecase/update_task_status.rs` | success, invalid transition, cancel |
| TaskEventPublisher | `usecase/event_publisher.rs` | noop publisher (created, updated, cancelled) |
| TaskPresenter | `adapter/presenter/task_presenter.rs` | detail response, list response |

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
