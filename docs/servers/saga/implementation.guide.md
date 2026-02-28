# system-saga-server 実装設計ガイド

> **仕様**: テーブル定義・APIスキーマは [implementation.md](./implementation.md) を参照。

---

## ドメインモデル実装コード

### SagaState エンティティ

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

### WorkflowDefinition / WorkflowStep

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

## リポジトリトレイト実装コード

### SagaRepository

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

## SagaError 実装コード

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

## インフラストラクチャ実装コード

### Config

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

```rust
// src/infrastructure/kafka_producer.rs
pub struct KafkaProducer {
    producer: rdkafka::producer::FutureProducer,
    topic: String,
}
```

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

## 特記事項

- **RecoverSagasUseCase**: 起動時に `find_incomplete()` で STARTED / RUNNING / COMPENSATING 状態の Saga を自動検出し、`ExecuteSagaUseCase` で再開する。リカバリされた件数をログに出力する
- **YAMLワークフローローダー**: `WorkflowLoader` が `workflows/` ディレクトリから `.yaml` / `.yml` ファイルを読み込み、`InMemoryWorkflowRepository` に登録する。無効な YAML ファイルはスキップして他のファイルのロードを継続する
- **gRPCステップ実行レジストリ**: `ServiceRegistry` が `config.yaml` の `services` セクションからサービス名→エンドポイントの静的マッピングを提供する。`TonicGrpcCaller` がチャネルプーリング付きで動的 gRPC 呼び出しを行う
- **InMemoryリポジトリ**: `DATABASE_URL` 未設定時のdev/test用に `main.rs` に `InMemorySagaRepository` を実装済み。`RwLock<Vec<SagaState>>` と `RwLock<Vec<SagaStepLog>>` で状態を管理する
- **Kafka オプショナル**: Kafka 未設定時やプロデューサー作成失敗時もサーバーは起動する（イベントは発行されない）

---

## 統合テスト補償フローテストケース

| テスト名 | 内容 |
|---------|------|
| `test_get_compensating_saga_returns_compensating_status` | COMPENSATING 状態の Saga を取得すると status=COMPENSATING が返る |
| `test_get_failed_saga_returns_error_message` | FAILED 状態の Saga には error_message が含まれる |
| `test_get_saga_step_logs_include_compensate_action` | 補償後のステップログに EXECUTE と COMPENSATE の両アクションが記録される |
