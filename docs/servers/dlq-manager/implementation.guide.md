# system-dlq-manager-server 実装設計ガイド

> **仕様**: テーブル定義・APIスキーマは [implementation.md](./implementation.md) を参照。

---

## ドメインモデル実装コード

### DlqMessage エンティティ

```rust
// src/domain/entity/dlq_message.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqMessage {
    pub id: Uuid,
    pub original_topic: String,
    pub error_message: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub payload: serde_json::Value,
    pub status: DlqStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_retry_at: Option<DateTime<Utc>>,
}
```

### DlqStatus 列挙型

```rust
// src/domain/entity/dlq_message.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DlqStatus {
    Pending,
    Retrying,
    Resolved,
    Dead,
}
```

`Display` トレイトで SCREAMING_SNAKE_CASE 文字列に変換する。`from_str_value` で文字列からの逆変換を提供する。

---

## リポジトリトレイト実装コード

### DlqMessageRepository

```rust
// src/domain/repository/dlq_message_repository.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DlqMessageRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<DlqMessage>>;
    async fn find_by_topic(&self, topic: &str, page: i32, page_size: i32) -> anyhow::Result<(Vec<DlqMessage>, i64)>;
    async fn create(&self, message: &DlqMessage) -> anyhow::Result<()>;
    async fn update(&self, message: &DlqMessage) -> anyhow::Result<()>;
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
    async fn count_by_topic(&self, topic: &str) -> anyhow::Result<i64>;
}
```

---

## DlqError 実装コード

```rust
// src/adapter/handler/error.rs
#[derive(Debug, thiserror::Error)]
pub enum DlqError {
    #[error("dlq message not found: {0}")]
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
    pub database: Option<DatabaseConfig>,    // オプショナル（DB 未設定時は InMemory）
    pub kafka: Option<KafkaConfig>,          // オプショナル（Kafka 未設定時はイベント非発行）
}
```

### DatabaseConfig

```rust
// src/infrastructure/database.rs
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,           // default: 5432
    pub name: String,
    pub user: String,
    pub password: String,    // default: ""
    pub ssl_mode: String,    // default: "disable"
    pub max_open_conns: u32, // default: 25
    pub max_idle_conns: u32, // default: 5
    pub conn_max_lifetime: String, // default: "5m"
}
```

`connection_url()` メソッドで `postgres://user:password@host:port/name?sslmode=ssl_mode` 形式の接続 URL を構築する。

### KafkaConfig

```rust
// src/infrastructure/kafka/mod.rs
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub brokers: Vec<String>,
    pub consumer_group: String,        // default: ""
    pub security_protocol: String,     // default: "PLAINTEXT"
    pub dlq_topic_pattern: String,     // default: "*.dlq.v1"
}
```

### DlqKafkaConsumer

```rust
// src/infrastructure/kafka/consumer.rs
pub struct DlqKafkaConsumer {
    _consumer: rdkafka::consumer::StreamConsumer,
    repo: Arc<dyn DlqMessageRepository>,
}
```

### DlqEventPublisher / DlqKafkaProducer

```rust
// src/infrastructure/kafka/producer.rs
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait DlqEventPublisher: Send + Sync {
    async fn publish_to_topic(&self, topic: &str, payload: &serde_json::Value) -> anyhow::Result<()>;
}

pub struct DlqKafkaProducer {
    producer: rdkafka::producer::FutureProducer,
}
```

### DlqPostgresRepository

```rust
// src/infrastructure/persistence/dlq_postgres.rs
pub struct DlqPostgresRepository {
    pool: PgPool,
}
```

PostgreSQL の `dlq.messages` テーブルに対する CRUD を提供する。`DlqMessageRow` 構造体で `sqlx::FromRow` を使い行マッピングを行い、`TryFrom<DlqMessageRow>` で `DlqMessage` に変換する。

---

## ユースケース処理フロー

### RetryMessageUseCase

```
1. find_by_id でメッセージを取得（未存在時エラー）
2. is_retryable() を検証（不可時エラー）
3. mark_retrying() でステータス遷移
4. Kafka publisher がある場合:
   a. publish_to_topic で元トピックに再発行
   b. 成功 → mark_resolved()
   c. 失敗 → RETRYING のまま（ログ出力）
5. Kafka publisher がない場合:
   a. mark_resolved()（Kafka なしでも処理完了扱い）
6. update でリポジトリを更新
7. 更新後のメッセージを返却
```

### RetryAllUseCase

```
1. page=1, page_size=100 で開始
2. find_by_topic でメッセージを取得
3. 空なら終了
4. 各メッセージに対して:
   a. is_retryable() が false ならスキップ
   b. mark_retrying()
   c. Kafka publisher がある場合は再発行（成功→mark_resolved()、失敗→RETRYING維持）
   d. Kafka publisher がない場合は mark_resolved()
   e. update でリポジトリを更新
   f. retried カウントをインクリメント
5. page をインクリメントして 2. に戻る
6. 合計 retried 件数を返却
```

---

## Bootstrap 手順

`main.rs` の起動シーケンス:

```
1.  k1s0-telemetry 初期化（service_name="k1s0-dlq-manager", tier="system"）
2.  config.yaml ロード（CONFIG_PATH 環境変数 or デフォルト "config/config.yaml"）
3.  アプリケーション情報ログ出力（app_name, version, environment）
4.  PostgreSQL 接続プール作成（database セクション or DATABASE_URL 環境変数、未設定時はスキップ）
5.  DlqMessageRepository 構築（Postgres 接続可 → DlqPostgresRepository / 不可 → InMemoryDlqMessageRepository）
6.  DlqKafkaProducer 構築（kafka セクション設定時のみ、失敗しても警告で続行）
7.  DlqKafkaConsumer 構築 + バックグラウンドタスクで起動（kafka セクション設定時のみ）
8.  ユースケース群を Arc でラップして構築
    - ListMessagesUseCase
    - GetMessageUseCase
    - RetryMessageUseCase（publisher を注入）
    - DeleteMessageUseCase
    - RetryAllUseCase（publisher を注入）
9.  AppState 構築（ユースケース群 + Metrics）
10. REST ルーター構築（handler::router）
11. REST サーバー（axum, port 8080）を起動
```

---

## 特記事項

- **InMemory リポジトリ**: `DATABASE_URL` 未設定時の dev/test 用に `main.rs` に `InMemoryDlqMessageRepository` を実装済み。`RwLock<Vec<DlqMessage>>` で状態を管理する
- **Kafka オプショナル**: Kafka 未設定時やプロデューサー作成失敗時もサーバーは起動する。再処理時は Kafka 再発行をスキップし RESOLVED に遷移する
- **Kafka コンシューマーオプショナル**: Kafka 未設定時やコンシューマー作成失敗時は DLQ メッセージの自動取り込みが無効になる（ログで警告出力）
- **REST のみ**: Saga サーバーと異なり gRPC サービスは提供しない。REST API のみで動作する
- **ルーティング順序**: `/api/v1/dlq/messages/:id` を `/api/v1/dlq/:topic` より先に定義し、`messages` が `:topic` パラメータとして誤マッチしないようにする
