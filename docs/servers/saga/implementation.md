# system-saga-server 実装設計

system-saga-server（Sagaオーケストレーションサーバー）の Rust 実装詳細を定義する。概要・API 定義・アーキテクチャは [system-saga-server.md](server.md) を参照。

---

## Rust 実装 (regions/system/server/rust/saga/)

### ディレクトリ構成

```
regions/system/server/rust/saga/
├── src/
│   ├── main.rs                              # エントリポイント + InMemorySagaRepository
│   ├── lib.rs                               # ライブラリクレート（pub mod 4モジュール）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── saga_state.rs                # SagaState / SagaStatus
│   │   │   ├── saga_step_log.rs             # SagaStepLog / StepAction / StepStatus
│   │   │   └── workflow.rs                  # WorkflowDefinition / WorkflowStep / RetryConfig
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── saga_repository.rs           # SagaRepository トレイト + SagaListParams
│   │       └── workflow_repository.rs       # WorkflowRepository トレイト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── start_saga.rs                    # Saga 開始
│   │   ├── execute_saga.rs                  # Saga 実行エンジン（核心ロジック）
│   │   ├── get_saga.rs                      # Saga 詳細取得
│   │   ├── list_sagas.rs                    # Saga 一覧取得
│   │   ├── cancel_saga.rs                   # Saga キャンセル
│   │   ├── register_workflow.rs             # ワークフロー登録
│   │   ├── list_workflows.rs                # ワークフロー一覧
│   │   └── recover_sagas.rs                 # 起動時リカバリ
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs                       # AppState / router() / ErrorResponse / ErrorBody
│   │   │   ├── saga_handler.rs              # REST ハンドラー（DTO + エンドポイント）
│   │   │   └── error.rs                     # SagaError（NotFound / Validation / Conflict / Internal）
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── saga_grpc.rs                 # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── saga_postgres.rs             # PostgreSQL リポジトリ実装
│   │       └── workflow_in_memory.rs        # InMemory ワークフローリポジトリ
│   └── infrastructure/
│       ├── mod.rs
│       ├── config.rs                        # Config / AppConfig / ServerConfig / SagaConfig / ServiceEndpoint
│       ├── database.rs                      # DatabaseConfig（接続URL構築）
│       ├── kafka_producer.rs                # SagaEventPublisher トレイト / KafkaProducer 実装
│       ├── grpc_caller.rs                   # GrpcStepCaller トレイト / ServiceRegistry / TonicGrpcCaller
│       └── workflow_loader.rs               # WorkflowLoader（YAML ファイルローダー）
├── config/
│   └── config.yaml                          # 本番設定
├── workflows/
│   └── order-fulfillment.yaml               # サンプルワークフロー定義
├── tests/
│   ├── integration_test.rs                  # REST API 統合テスト
│   ├── workflow_engine_test.rs              # ワークフローエンジンテスト
│   ├── postgres_repository_test.rs          # PostgreSQL リポジトリテスト（#[ignore]）
│   └── kafka_integration_test.rs            # Kafka 統合テスト（#[ignore]）
├── build.rs                                 # tonic-build（proto codegen）
├── Cargo.toml
└── Cargo.lock
```

### Cargo.toml

> 共通依存は [Rust共通実装.md](../_common/Rust共通実装.md#共通cargo依存) を参照。サービス固有の追加依存:

```toml
async-trait = "0.1"

# Auth library
k1s0-auth = { path = "../../../library/rust/auth" }

# Telemetry library
k1s0-telemetry = { path = "../../../library/rust/telemetry" }

[dev-dependencies]
axum-test = "16"
tower = { version = "0.5", features = ["util"] }
tempfile = "3"
```

### 主要依存ライブラリ

| ライブラリ | バージョン | 用途 |
|---------|----------|-----|
| axum | 0.7 | REST HTTP フレームワーク |
| tokio | 1 | 非同期ランタイム（full features） |
| tonic | 0.12 | gRPC フレームワーク |
| prost | 0.13 | Protobuf シリアライゼーション |
| sqlx | 0.8 | PostgreSQL 非同期ドライバー |
| rdkafka | 0.36 | Kafka プロデューサー / コンシューマー |
| serde / serde_json | 1 | JSON シリアライゼーション |
| serde_yaml | 0.9 | YAML 解析（ワークフロー定義・設定） |
| uuid | 1 | UUID v4 生成 |
| chrono | 0.4 | 日時処理 |
| anyhow | 1 | エラーハンドリング |
| thiserror | 2 | エラー型定義 |
| async-trait | 0.1 | 非同期トレイト |
| tracing | 0.1 | 構造化ログ |
| k1s0-auth | path | 認証ライブラリ（JWT 検証） |
| k1s0-telemetry | path | テレメトリライブラリ（OTel / メトリクス） |

### build.rs

> build.rs パターンは [Rust共通実装.md](../_common/Rust共通実装.md#共通buildrs) を参照。proto パス: `api/proto/k1s0/system/saga/v1/saga.proto`

saga-server では proto ファイル未存在時や protoc 未インストール時にスキップする条件付きビルドを採用。`out_dir("src/proto")` で生成先を明示指定する。

---

## ドメインモデル

### SagaState エンティティ

| フィールド | 型 | 説明 |
|---------|---|-----|
| `saga_id` | `Uuid` | Saga の一意識別子（v4 自動生成） |
| `workflow_name` | `String` | 実行するワークフロー名 |
| `current_step` | `i32` | 現在のステップインデックス（0 始まり） |
| `status` | `SagaStatus` | Saga ステータス |
| `payload` | `serde_json::Value` | 各ステップに渡す JSON ペイロード |
| `correlation_id` | `Option<String>` | 業務相関 ID（トレーサビリティ用） |
| `initiated_by` | `Option<String>` | 呼び出し元サービス名 |
| `error_message` | `Option<String>` | エラーメッセージ |
| `created_at` | `DateTime<Utc>` | 作成日時 |
| `updated_at` | `DateTime<Utc>` | 更新日時 |

**メソッド:**

| メソッド | 説明 |
|---------|-----|
| `new(workflow_name, payload, correlation_id, initiated_by)` | 初期状態（STARTED, current_step=0）で作成 |
| `advance_step()` | current_step を +1 し status を RUNNING に遷移 |
| `complete()` | status を COMPLETED に遷移、error_message をクリア |
| `start_compensation(error)` | status を COMPENSATING に遷移、error_message を設定 |
| `fail(error)` | status を FAILED に遷移（終端状態） |
| `cancel()` | status を CANCELLED に遷移（終端状態） |
| `is_terminal()` | COMPLETED / FAILED / CANCELLED かどうかを返す |

**実装コード:**

```rust
// src/domain/entity/saga_state.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaState {
    pub saga_id: Uuid,
    pub workflow_name: String,
    pub current_step: i32,
    pub status: SagaStatus,
    pub payload: serde_json::Value,
    pub correlation_id: Option<String>,
    pub initiated_by: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### SagaStatus 列挙型

| ステータス | 説明 | 終端 |
|-----------|-----|------|
| `Started` | Saga 作成直後の初期状態 | No |
| `Running` | ステップ実行中 | No |
| `Completed` | 全ステップ正常完了 | Yes |
| `Compensating` | ステップ失敗により補償処理実行中 | No |
| `Failed` | 補償処理完了後の失敗状態 | Yes |
| `Cancelled` | ユーザーキャンセル | Yes |

**実装コード:**

```rust
// src/domain/entity/saga_state.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SagaStatus {
    Started,
    Running,
    Completed,
    Compensating,
    Failed,
    Cancelled,
}
```

`Display` トレイトで SCREAMING_SNAKE_CASE 文字列に変換する。`from_str_value` で文字列からの逆変換を提供する。

### SagaStepLog エンティティ

| フィールド | 型 | 説明 |
|---------|---|-----|
| `id` | `Uuid` | ログの一意識別子 |
| `saga_id` | `Uuid` | 所属する Saga ID |
| `step_index` | `i32` | ステップインデックス |
| `step_name` | `String` | ステップ名 |
| `action` | `StepAction` | 実行アクション（EXECUTE / COMPENSATE） |
| `status` | `StepStatus` | 実行結果（SUCCESS / FAILED / TIMEOUT / SKIPPED） |
| `request_payload` | `Option<serde_json::Value>` | リクエストペイロード |
| `response_payload` | `Option<serde_json::Value>` | レスポンスペイロード |
| `error_message` | `Option<String>` | エラーメッセージ |
| `started_at` | `DateTime<Utc>` | 開始日時 |
| `completed_at` | `Option<DateTime<Utc>>` | 完了日時 |

**メソッド:**

| メソッド | 説明 |
|---------|-----|
| `new_execute(saga_id, step_index, step_name, request_payload)` | 実行ログを作成（初期 status=FAILED） |
| `new_compensate(saga_id, step_index, step_name, request_payload)` | 補償ログを作成 |
| `mark_success(response)` | status=SUCCESS、response_payload / completed_at を設定 |
| `mark_failed(error)` | status=FAILED、error_message / completed_at を設定 |
| `mark_timeout()` | status=TIMEOUT、error_message="step timed out" を設定 |

**実装コード:**

```rust
// src/domain/entity/saga_step_log.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStepLog {
    pub id: Uuid,
    pub saga_id: Uuid,
    pub step_index: i32,
    pub step_name: String,
    pub action: StepAction,
    pub status: StepStatus,
    pub request_payload: Option<serde_json::Value>,
    pub response_payload: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

**StepAction 列挙型:**

| 値 | 説明 |
|---|-----|
| `Execute` | 通常のステップ実行 |
| `Compensate` | 補償処理の実行 |

**StepStatus 列挙型:**

| 値 | 説明 |
|---|-----|
| `Success` | ステップ成功 |
| `Failed` | ステップ失敗 |
| `Timeout` | タイムアウト |
| `Skipped` | スキップ（補償メソッド未定義時等） |

### WorkflowDefinition / WorkflowStep

**WorkflowDefinition メソッド:**

| メソッド | 説明 |
|---------|-----|
| `from_yaml(content)` | YAML 文字列からワークフロー定義を解析・検証 |
| `validate()` | name 非空・steps 1件以上・各 step の name/service/method 非空を検証 |
| `timeout_duration(step_idx)` | 指定ステップのタイムアウト `Duration` を返す |

**RetryConfig メソッド:**

| メソッド | 説明 |
|---------|-----|
| `delay_for_attempt(attempt)` | `initial_interval_ms * 2^attempt` でバックオフ遅延を計算 |

**実装コード:**

```rust
// src/domain/entity/workflow.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub service: String,
    pub method: String,
    pub compensate: Option<String>,
    pub timeout_secs: u64,       // デフォルト: 30
    pub retry: Option<RetryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,       // デフォルト: 3
    pub backoff: String,         // デフォルト: "exponential"
    pub initial_interval_ms: u64, // デフォルト: 1000
}
```

---

## リポジトリトレイト

### SagaRepository

`SagaListParams` はフィルタリング用のパラメータ構造体:

| フィールド | 型 | 説明 |
|---------|---|-----|
| `workflow_name` | `Option<String>` | ワークフロー名フィルタ |
| `status` | `Option<SagaStatus>` | ステータスフィルタ |
| `correlation_id` | `Option<String>` | 相関 ID フィルタ |
| `page` | `i32` | ページ番号 |
| `page_size` | `i32` | 1 ページあたりの件数 |

| メソッド | 説明 |
|---------|-----|
| `create(state)` | SagaState を新規作成する |
| `update_with_step_log(state, log)` | Saga 状態とステップログを原子的に更新する |
| `update_status(saga_id, status, error_message)` | ステータスのみ更新する |
| `find_by_id(saga_id)` | ID で SagaState を取得する |
| `find_step_logs(saga_id)` | Saga に紐づくステップログを取得する |
| `list(params)` | フィルタ・ページネーション付き一覧取得 |
| `find_incomplete()` | 未完了 Saga を検索する（リカバリ用） |

**トレイト定義コード:**

```rust
// src/domain/repository/saga_repository.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SagaRepository: Send + Sync {
    async fn create(&self, state: &SagaState) -> anyhow::Result<()>;
    async fn update_with_step_log(&self, state: &SagaState, log: &SagaStepLog) -> anyhow::Result<()>;
    async fn update_status(&self, saga_id: Uuid, status: &SagaStatus, error_message: Option<String>) -> anyhow::Result<()>;
    async fn find_by_id(&self, saga_id: Uuid) -> anyhow::Result<Option<SagaState>>;
    async fn find_step_logs(&self, saga_id: Uuid) -> anyhow::Result<Vec<SagaStepLog>>;
    async fn list(&self, params: &SagaListParams) -> anyhow::Result<(Vec<SagaState>, i64)>;
    async fn find_incomplete(&self) -> anyhow::Result<Vec<SagaState>>;
}
```

### WorkflowRepository

| メソッド | 説明 |
|---------|-----|
| `register(workflow)` | ワークフロー定義を登録する |
| `get(name)` | 名前でワークフローを取得する |
| `list()` | 全ワークフロー一覧を取得する |

**トレイト定義コード:**

```rust
// src/domain/repository/workflow_repository.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    async fn register(&self, workflow: WorkflowDefinition) -> anyhow::Result<()>;
    async fn get(&self, name: &str) -> anyhow::Result<Option<WorkflowDefinition>>;
    async fn list(&self) -> anyhow::Result<Vec<WorkflowDefinition>>;
}
```

---

## ユースケース

| ユースケース | 責務 |
|------------|------|
| `StartSagaUseCase` | ワークフローを検索し、SagaState を作成して `tokio::spawn` で `ExecuteSagaUseCase` をバックグラウンド実行する |
| `ExecuteSagaUseCase` | Saga 実行エンジン。ステップ順に gRPC 呼び出しを行い、失敗時は補償処理を逆順実行する。Kafka イベントを発行する |
| `GetSagaUseCase` | saga_id で SagaState とステップログを取得する |
| `ListSagasUseCase` | SagaListParams に基づいてページネーション付き一覧を取得する |
| `CancelSagaUseCase` | 終端状態でない Saga をキャンセルする。終端状態の場合はエラーを返す |
| `RegisterWorkflowUseCase` | YAML 文字列をパースして WorkflowRepository に登録する |
| `ListWorkflowsUseCase` | 登録済みワークフロー定義の一覧を取得する |
| `RecoverSagasUseCase` | 起動時に STARTED / RUNNING / COMPENSATING 状態の Saga を検索し、`ExecuteSagaUseCase` で自動再開する |

---

## REST API エンドポイント

| Method | Path | ハンドラー関数 | 説明 |
|--------|------|-------------|-----|
| GET | `/healthz` | `healthz` | ヘルスチェック |
| GET | `/readyz` | `readyz` | レディネスチェック |
| GET | `/metrics` | `metrics` | Prometheus メトリクス |
| POST | `/api/v1/sagas` | `start_saga` | Saga 開始（201 Created） |
| GET | `/api/v1/sagas` | `list_sagas` | Saga 一覧取得（ページネーション付き） |
| GET | `/api/v1/sagas/:saga_id` | `get_saga` | Saga 詳細取得（ステップログ含む） |
| POST | `/api/v1/sagas/:saga_id/cancel` | `cancel_saga` | Saga キャンセル |
| POST | `/api/v1/workflows` | `register_workflow` | ワークフロー登録（201 Created） |
| GET | `/api/v1/workflows` | `list_workflows` | ワークフロー一覧取得 |

### REST ハンドラー DTO

**リクエスト:**

| DTO | フィールド | 用途 |
|-----|---------|-----|
| `StartSagaRequest` | `workflow_name`, `payload`, `correlation_id`, `initiated_by` | Saga 開始 |
| `ListSagasQuery` | `workflow_name`, `status`, `correlation_id`, `page`, `page_size` | Saga 一覧フィルタ |
| `RegisterWorkflowRequest` | `workflow_yaml` | ワークフロー登録（YAML 文字列） |

**レスポンス:**

| DTO | フィールド | 用途 |
|-----|---------|-----|
| `StartSagaResponse` | `saga_id`, `status` | Saga 開始結果 |
| `SagaDetailResponse` | `saga`, `step_logs` | Saga 詳細（ステップログ付き） |
| `SagaResponse` | `saga_id`, `workflow_name`, `current_step`, `status`, `payload`, `correlation_id`, `initiated_by`, `error_message`, `created_at`, `updated_at` | Saga 状態 |
| `StepLogResponse` | `id`, `step_index`, `step_name`, `action`, `status`, `request_payload`, `response_payload`, `error_message`, `started_at`, `completed_at` | ステップログ |
| `ListSagasResponse` | `sagas`, `pagination` | Saga 一覧 |
| `PaginationResponse` | `total_count`, `page`, `page_size`, `has_next` | ページネーション |
| `RegisterWorkflowResponse` | `name`, `step_count` | ワークフロー登録結果 |
| `ListWorkflowsResponse` | `workflows` | ワークフロー一覧 |
| `CancelSagaResponse` | `success`, `message` | キャンセル結果 |

### SagaError

| バリアント | HTTP Status | エラーコード |
|-----------|-------------|------------|
| `NotFound` | 404 | `SYS_SAGA_NOT_FOUND` |
| `Validation` | 400 | `SYS_SAGA_VALIDATION_ERROR` |
| `Conflict` | 409 | `SYS_SAGA_CONFLICT` |
| `Internal` | 500 | `SYS_SAGA_INTERNAL_ERROR` |

**実装コード:**

```rust
// src/adapter/handler/error.rs
#[derive(Debug, thiserror::Error)]
pub enum SagaError {
    #[error("saga not found: {0}")]
    NotFound(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal error: {0}")]
    Internal(String),
}
```

`IntoResponse` トレイトを実装し、`ErrorResponse` 構造体（`code`, `message`, `request_id`, `details`）で統一エラーレスポンスを返す。

---

## gRPC サービス

`build.rs` で `api/proto/k1s0/system/saga/v1/saga.proto` を tonic-build でコンパイルする。proto ファイルが存在しない場合やprotoc が利用不可の場合はスキップし、手動型定義で代替する。

- `adapter/grpc/saga_grpc.rs` -- `SagaGrpcService` 構造体がユースケースを受け取り gRPC リクエストを処理する
- `adapter/grpc/tonic_service.rs` -- `SagaServiceTonic` ラッパーで tonic サーバーに登録する

gRPC ポートは 50051 を使用する（REST の 8080 と並行起動）。

---

## インフラストラクチャ

### Config

| 設定ブロック | 主要フィールド | 説明 |
|------------|-------------|-----|
| `app` | `name`, `version`, `environment` | アプリケーション識別情報 |
| `server` | `host`(default: "0.0.0.0"), `port`(default: 8080) | HTTP サーバー |
| `database` | `host`, `port`, `name`, `user`, `password`, `ssl_mode`, `max_open_conns` | PostgreSQL 接続（Optional） |
| `kafka` | `brokers`, `consumer_group`, `security_protocol`, `sasl`, `topics` | Kafka 接続（Optional） |
| `services` | `{service-name: {host, port}}` | gRPC サービスエンドポイント |
| `saga` | `max_concurrent`(default: 100), `workflow_dir`(default: "workflows") | Saga 固有設定 |

**実装コード:**

```rust
// src/infrastructure/config.rs
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub server: ServerConfig,
    pub database: Option<DatabaseConfig>,      // オプショナル（DB 未設定時は InMemory）
    pub kafka: Option<KafkaConfig>,            // オプショナル（Kafka 未設定時はイベント非発行）
    pub services: HashMap<String, ServiceEndpoint>,  // gRPC サービスレジストリ
    pub saga: SagaConfig,                      // Saga 固有設定
}
```

### config.yaml サービス固有セクション例

> 共通セクション（app/server/database/kafka）は [Rust共通実装.md](../_common/Rust共通実装.md#共通configyaml) を参照。

```yaml
services:
  inventory-service:
    host: "inventory.k1s0-business.svc.cluster.local"
    port: 50051
  payment-service:
    host: "payment.k1s0-business.svc.cluster.local"
    port: 50051
  shipping-service:
    host: "shipping.k1s0-business.svc.cluster.local"
    port: 50051

saga:
  max_concurrent: 100
  workflow_dir: "workflows"
```

### DatabaseConfig

```rust
// src/infrastructure/database.rs
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,          // default: 5432
    pub name: String,
    pub user: String,
    pub password: String,
    pub ssl_mode: String,   // default: "disable"
    pub max_open_conns: u32, // default: 25
    pub max_idle_conns: u32, // default: 5
    pub conn_max_lifetime: String, // default: "5m"
}
```

`connection_url()` メソッドで `postgres://user:password@host:port/name?sslmode=ssl_mode` 形式の接続 URL を構築する。

### WorkflowLoader

| メソッド | 説明 |
|---------|-----|
| `new(workflow_dir)` | ワークフローディレクトリを指定してローダーを作成 |
| `load_all()` | ディレクトリ内の全 `.yaml` / `.yml` ファイルを読み込み、`WorkflowDefinition` リストを返す。ディレクトリ未存在時は空リストを返す（エラーにしない）。無効な YAML はスキップしてログ出力する |
| `load_file(path)` | 指定ファイルを読み込み、`WorkflowDefinition` を返す |

```rust
// src/infrastructure/workflow_loader.rs
pub struct WorkflowLoader {
    workflow_dir: PathBuf,
}
```

### ServiceRegistry

```rust
// src/infrastructure/grpc_caller.rs
pub struct ServiceRegistry {
    services: HashMap<String, ServiceEndpoint>,
}
```

`config.yaml` の `services` セクションからサービス名→エンドポイント（`http://host:port`）のマッピングを提供する。`resolve(service_name)` で名前解決を行う。

### TonicGrpcCaller（GrpcStepCaller 実装）

```rust
// src/infrastructure/grpc_caller.rs
pub struct TonicGrpcCaller {
    registry: Arc<ServiceRegistry>,
    channels: RwLock<HashMap<String, Channel>>,
}
```

- `ServiceRegistry` から取得したエンドポイントに対して tonic の gRPC チャネルを作成
- ワークフローステップの `method` フィールド（`ServiceName.MethodName` 形式）を `build_grpc_path` で `/ServiceName/MethodName` に変換
- チャネルは `RwLock<HashMap<String, Channel>>` で接続プールとして管理

### KafkaProducer（SagaEventPublisher 実装）

| メソッド | 説明 |
|---------|-----|
| `new(config)` | rdkafka `FutureProducer` を作成。SASL 認証設定にも対応 |
| `publish_saga_event(saga_id, event_type, payload)` | イベントを JSON シリアライズして Kafka トピックに発行。saga_id をキーとして使用 |
| `close()` | プロデューサーをフラッシュして終了 |

```rust
// src/infrastructure/kafka_producer.rs
pub struct KafkaProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
}
```

Kafka イベント一覧:

| イベント | 発行タイミング |
|---------|-------------|
| `SAGA_RUNNING` | Saga 実行開始時 |
| `SAGA_COMPLETED` | 全ステップ正常完了時 |
| `SAGA_COMPENSATING` | 補償処理開始時 |
| `SAGA_FAILED` | 補償処理完了（Saga 失敗確定）時 |

---

## Bootstrap 手順

`main.rs` の起動シーケンス:

```
1.  k1s0-telemetry 初期化（service_name="k1s0-saga-server", tier="system"）
2.  config.yaml ロード（CONFIG_PATH 環境変数 or デフォルト "config/config.yaml"）
3.  PostgreSQL 接続プール作成（database セクション or DATABASE_URL 環境変数、未設定時はスキップ）
4.  SagaRepository 構築（Postgres 接続可 → SagaPostgresRepository / 不可 → InMemorySagaRepository）
5.  WorkflowLoader で workflows/ ディレクトリから全 YAML をロード
6.  InMemoryWorkflowRepository にワークフロー定義を一括登録
7.  ServiceRegistry + TonicGrpcCaller 構築（config.yaml の services セクションから）
8.  KafkaProducer 構築（kafka セクション設定時のみ、失敗しても警告で続行）
9.  ユースケース群を Arc でラップして構築
10. RecoverSagasUseCase 実行（STARTED / RUNNING / COMPENSATING 状態の Saga を自動リカバリ）
11. AppState 構築（REST ハンドラー用）
12. SagaGrpcService 構築（gRPC ハンドラー用）
13. REST サーバー（axum, port 8080）+ gRPC サーバー（tonic, port 50051）を tokio::select! で並行起動
```

---

## テスト構成

### ユニットテスト

各モジュール内の `#[cfg(test)]` ブロックで実装。mockall を使用してリポジトリ・gRPC caller をモック化する。

| テスト対象 | テスト数 | 内容 |
|----------|--------|------|
| `domain/entity/saga_state` | 10 | 状態遷移、ステータス変換、終端判定 |
| `domain/entity/saga_step_log` | 7 | ログ作成、成功/失敗/タイムアウトマーク、Display |
| `domain/entity/workflow` | 6 | YAML 解析、バリデーション、タイムアウト、バックオフ計算 |
| `infrastructure/config` | 2 | 設定デシリアライズ、デフォルト値 |
| `infrastructure/database` | 2 | 接続 URL 生成、設定デシリアライズ |
| `infrastructure/kafka_producer` | 5 | KafkaConfig 解析、イベント発行、エラーハンドリング、モック |
| `infrastructure/grpc_caller` | 5 | サービスレジストリ解決、未登録サービスエラー、gRPC パス構築、モック |
| `infrastructure/workflow_loader` | 10 | ファイルロード、ディレクトリ走査、拡張子フィルタ、無効 YAML スキップ、存在しないディレクトリ |

### インテグレーションテスト

`tests/` ディレクトリに配置。外部依存を要するテストは `#[ignore]` でマークし、CI で選択的に実行する。

| テストファイル | 要件 | 内容 |
|-------------|------|------|
| `integration_test.rs` | InMemory | REST API の統合テスト（axum-test 使用） |
| `workflow_engine_test.rs` | モック | ワークフロー実行パスの検証 |
| `postgres_repository_test.rs` | PostgreSQL | DB 操作の検証（`#[ignore]`） |
| `kafka_integration_test.rs` | Kafka | イベント発行の検証（`#[ignore]`） |

### 統合テスト補償フローテストケース

| テスト名 | 内容 |
|---------|------|
| `test_get_compensating_saga_returns_compensating_status` | COMPENSATING 状態の Saga を取得すると status=COMPENSATING が返る |
| `test_get_failed_saga_returns_error_message` | FAILED 状態の Saga には error_message が含まれる |
| `test_get_saga_step_logs_include_compensate_action` | 補償後のステップログに EXECUTE と COMPENSATE の両アクションが記録される |

---

## 特記事項

- **RecoverSagasUseCase**: 起動時に `find_incomplete()` で STARTED / RUNNING / COMPENSATING 状態の Saga を自動検出し、`ExecuteSagaUseCase` で再開する。リカバリされた件数をログに出力する
- **YAMLワークフローローダー**: `WorkflowLoader` が `workflows/` ディレクトリから `.yaml` / `.yml` ファイルを読み込み、`InMemoryWorkflowRepository` に登録する。無効な YAML ファイルはスキップして他のファイルのロードを継続する
- **gRPCステップ実行レジストリ**: `ServiceRegistry` が `config.yaml` の `services` セクションからサービス名→エンドポイントの静的マッピングを提供する。`TonicGrpcCaller` がチャネルプーリング付きで動的 gRPC 呼び出しを行う
- **InMemoryリポジトリ**: `DATABASE_URL` 未設定時のdev/test用に `main.rs` に `InMemorySagaRepository` を実装済み。`RwLock<Vec<SagaState>>` と `RwLock<Vec<SagaStepLog>>` で状態を管理する
- **Kafka オプショナル**: Kafka 未設定時やプロデューサー作成失敗時もサーバーは起動する（イベントは発行されない）

---

## 関連ドキュメント

- [system-saga-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-server-implementation.md](../_common/implementation.md) -- auth-server 実装設計（同一パターン）
- [system-saga-database.md](database.md) -- Saga データベーススキーマ
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- Saga パターンの基本方針
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート仕様
- [config.md](../../cli/config/config設計.md) -- config.yaml スキーマと環境別管理
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- Linter・Formatter・命名規則
