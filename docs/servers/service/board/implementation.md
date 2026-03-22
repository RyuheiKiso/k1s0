# service-board-server 実装設計

service-board-server の Rust 実装構成を定義する。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## Rust 実装 (regions/service/board/server/rust/board/)

### ディレクトリ構成

```
regions/service/board/server/rust/board/
├── src/
│   ├── main.rs                          # エントリポイント
│   ├── lib.rs                           # ライブラリルート（MIGRATOR 定義）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── error.rs                     # BoardError（thiserror ベース）
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── board_column.rs          # BoardColumn エンティティ（WIP 制限チェックロジック含む）
│   │   └── repository/
│   │       ├── mod.rs
│   │       └── board_column_repository.rs  # BoardColumnRepository トレイト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── increment_column.rs          # IncrementColumnUseCase（WIP 制限チェック）
│   │   ├── decrement_column.rs          # DecrementColumnUseCase
│   │   ├── get_board_column.rs          # GetBoardColumnUseCase
│   │   ├── list_board_columns.rs        # ListBoardColumnsUseCase
│   │   ├── update_wip_limit.rs          # UpdateWipLimitUseCase
│   │   └── event_publisher.rs          # BoardEventPublisher トレイト + NoopBoardEventPublisher
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs                   # AppState, router(), actor_from_claims()
│   │   │   ├── board_handler.rs         # ボード REST ハンドラー
│   │   │   └── health.rs               # ヘルスチェックハンドラー
│   │   ├── presenter/
│   │   │   ├── mod.rs
│   │   │   └── board_presenter.rs       # BoardColumnResponse, BoardColumnListResponse
│   │   └── middleware/
│   │       ├── mod.rs
│   │       └── auth.rs                  # JWT 認証ミドルウェア（k1s0-server-common 経由）
│   └── infrastructure/
│       ├── mod.rs
│       ├── config.rs                    # Config ローダー・バリデーション
│       ├── database/
│       │   ├── mod.rs
│       │   └── board_column_repository.rs  # BoardColumnPostgresRepository（sqlx 実装）
│       └── kafka/
│           ├── mod.rs
│           └── board_producer.rs        # BoardKafkaProducer（rdkafka 実装）
├── config/
│   └── default.yaml                     # デフォルト設定ファイル
├── Cargo.toml
└── Cargo.lock
```

### Cargo.toml

```toml
[package]
name = "k1s0-board-server"
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

### BoardColumn

`project_id` + `status_code` の組み合わせで管理される。WIP 制限チェックロジックをエンティティに持つ。

```rust
pub struct BoardColumn {
    pub id: Uuid,
    pub project_id: String,
    pub status_code: String,
    pub task_count: i32,
    pub wip_limit: i32,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BoardColumn {
    /// WIP 制限チェックを行う。
    /// wip_limit = 0 は無制限。wip_limit > 0 かつ task_count >= wip_limit でエラー。
    pub fn check_wip_limit(&self) -> Result<(), BoardError> {
        if self.wip_limit > 0 && self.task_count >= self.wip_limit {
            Err(BoardError::WipLimitExceeded {
                project_id: self.project_id.clone(),
                status_code: self.status_code.clone(),
                current: self.task_count,
                limit: self.wip_limit,
            })
        } else {
            Ok(())
        }
    }
}
```

### BoardError

```rust
#[derive(Debug, thiserror::Error)]
pub enum BoardError {
    #[error("Board column not found: project={0}, status={1}")]
    NotFound(String, String),                              // → 404

    #[error("WIP limit exceeded: project={project_id}, status={status_code}, current={current}, limit={limit}")]
    WipLimitExceeded {                                     // → 409
        project_id: String,
        status_code: String,
        current: i32,
        limit: i32,
    },

    #[error("validation failed: {0}")]
    ValidationFailed(String),                              // → 400

    #[error("version conflict for board column")]
    VersionConflict,                                       // → 409

    #[error("internal error: {0}")]
    Internal(String),                                      // → 500
}
```

---

## リポジトリトレイト実装（Rust）

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BoardColumnRepository: Send + Sync {
    async fn find_by_project_and_status(
        &self, project_id: &str, status_code: &str,
    ) -> anyhow::Result<Option<BoardColumn>>;
    async fn find_all_by_project(
        &self, project_id: &str,
    ) -> anyhow::Result<Vec<BoardColumn>>;
    async fn upsert(&self, column: &BoardColumn) -> anyhow::Result<BoardColumn>;
    async fn increment(
        &self, project_id: &str, status_code: &str, expected_version: i32,
    ) -> anyhow::Result<BoardColumn>;
    async fn decrement(
        &self, project_id: &str, status_code: &str, expected_version: i32,
    ) -> anyhow::Result<BoardColumn>;
    async fn update_wip_limit(
        &self, project_id: &str, status_code: &str, wip_limit: i32, expected_version: i32,
    ) -> anyhow::Result<BoardColumn>;
}
```

---

## ユースケース実装（Rust）

### IncrementColumnUseCase

```rust
pub struct IncrementColumnUseCase {
    column_repo: Arc<dyn BoardColumnRepository>,
    event_publisher: Arc<dyn BoardEventPublisher>,
}

impl IncrementColumnUseCase {
    pub async fn execute(
        &self, project_id: &str, status_code: &str, actor: &str,
    ) -> anyhow::Result<BoardColumn> {
        // 1. カラムを取得（存在しなければ新規作成: task_count=0, wip_limit=0）
        let column = self.column_repo
            .find_by_project_and_status(project_id, status_code)
            .await?
            .unwrap_or_else(|| BoardColumn::new(project_id, status_code));

        // 2. WIP 制限チェック（wip_limit > 0 && task_count >= wip_limit でエラー）
        column.check_wip_limit()?;

        // 3. 楽観的ロック付き increment（version フィールド）
        let updated = self.column_repo
            .increment(project_id, status_code, column.version)
            .await?;

        // 4. board.column_updated イベント発行（Outbox pattern）
        self.event_publisher.publish_column_updated(&updated, actor).await;

        Ok(updated)
    }
}
```

### DecrementColumnUseCase

decrement 操作は WIP 制限チェックを行わない（常に許可）。task_count が 0 未満にはならないよう DB の CHECK 制約で保護する。

### UpdateWipLimitUseCase

```rust
impl UpdateWipLimitUseCase {
    pub async fn execute(
        &self, project_id: &str, status_code: &str, wip_limit: i32, actor: &str,
    ) -> anyhow::Result<BoardColumn> {
        // 1. カラムを取得
        let column = self.column_repo
            .find_by_project_and_status(project_id, status_code)
            .await?
            .ok_or_else(|| BoardError::NotFound(project_id.to_string(), status_code.to_string()))?;

        // 2. WIP 制限を更新（楽観的ロック）
        let updated = self.column_repo
            .update_wip_limit(project_id, status_code, wip_limit, column.version)
            .await?;

        // 3. board.column_updated イベント発行（Outbox pattern）
        self.event_publisher.publish_column_updated(&updated, actor).await;

        Ok(updated)
    }
}
```

### BoardEventPublisher トレイト

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait BoardEventPublisher: Send + Sync {
    async fn publish_column_updated(
        &self, column: &BoardColumn, actor: &str,
    ) -> anyhow::Result<()>;
}
```

`NoopBoardEventPublisher` は Kafka 未設定時のフォールバック実装。

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
        .route("/api/v1/boards/increment", post(increment_column))
        .route("/api/v1/boards/decrement", post(decrement_column))
        .route("/api/v1/boards/{project_id}/columns", get(list_columns))
        .route(
            "/api/v1/boards/{project_id}/columns/{status_code}",
            get(get_column),
        )
        .route(
            "/api/v1/boards/{project_id}/columns/{status_code}/wip-limit",
            put(update_wip_limit),
        )
        .route_layer(make_method_rbac_middleware("board"))
        .layer(from_fn_with_state(auth_state.clone(), auth_middleware));

    public_routes.merge(api_routes)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
```

### Infrastructure 実装（Rust）

#### BoardColumnPostgresRepository

- `increment`: `UPDATE board_columns SET task_count = task_count + 1, version = version + 1 WHERE project_id = $1 AND status_code = $2 AND version = $3`。バージョン不一致時はエラー
- `decrement`: `UPDATE board_columns SET task_count = GREATEST(task_count - 1, 0), version = version + 1 WHERE ...`。task_count が 0 を下回らないよう `GREATEST` を使用
- `upsert`: `INSERT ... ON CONFLICT (project_id, status_code) DO UPDATE SET ...`

---

## テスト

### 単体テスト

| テスト対象 | ファイル | 内容 |
| --- | --- | --- |
| BoardColumn.check_wip_limit | `domain/entity/board_column.rs` | wip_limit=0（無制限）、wip_limit超過、wip_limit未満 |
| IncrementColumnUseCase | `usecase/increment_column.rs` | success, WIP 制限超過エラー |
| DecrementColumnUseCase | `usecase/decrement_column.rs` | success |
| UpdateWipLimitUseCase | `usecase/update_wip_limit.rs` | success, not found |
| BoardEventPublisher | `usecase/event_publisher.rs` | noop publisher |

### 実 DB 統合テスト

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/test_db \
  cargo test --all -- --include-ignored
```

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [database.md](database.md) -- データベーススキーマ・マイグレーション
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート・クリーンアーキテクチャ
