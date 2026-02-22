# system-dlq-manager-server 実装設計

system-dlq-manager-server（DLQ メッセージ管理サーバー）の Rust 実装詳細を定義する。概要・API 定義・アーキテクチャは [system-dlq-manager-server設計.md](system-dlq-manager-server設計.md) を参照。

---

## Rust 実装 (regions/system/server/rust/dlq-manager/)

### ディレクトリ構成

```
regions/system/server/rust/dlq-manager/
├── src/
│   ├── main.rs                              # エントリポイント + InMemoryDlqMessageRepository
│   ├── lib.rs                               # ライブラリクレート（pub mod 4モジュール）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── dlq_message.rs               # DlqMessage / DlqStatus
│   │   └── repository/
│   │       ├── mod.rs
│   │       └── dlq_message_repository.rs    # DlqMessageRepository トレイト
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── list_messages.rs                 # トピック別メッセージ一覧取得
│   │   ├── get_message.rs                   # メッセージ詳細取得
│   │   ├── retry_message.rs                 # メッセージ再処理
│   │   ├── delete_message.rs                # メッセージ削除
│   │   └── retry_all.rs                     # トピック内全メッセージ一括再処理
│   ├── adapter/
│   │   ├── mod.rs
│   │   └── handler/
│   │       ├── mod.rs                       # AppState / router() / ErrorResponse / ErrorBody
│   │       ├── dlq_handler.rs               # REST ハンドラー（DTO + エンドポイント）
│   │       └── error.rs                     # DlqError（NotFound / Validation / Conflict / Internal）
│   └── infrastructure/
│       ├── mod.rs
│       ├── config.rs                        # Config / AppConfig / ServerConfig
│       ├── database.rs                      # DatabaseConfig（接続URL構築）
│       ├── kafka/
│       │   ├── mod.rs                       # KafkaConfig
│       │   ├── consumer.rs                  # DlqKafkaConsumer（DLQトピック購読）
│       │   └── producer.rs                  # DlqEventPublisher トレイト / DlqKafkaProducer
│       └── persistence/
│           ├── mod.rs
│           └── dlq_postgres.rs              # DlqPostgresRepository（PostgreSQL実装）
├── config/
│   ├── config.yaml                          # 本番設定
│   ├── config.dev.yaml                      # 開発設定
│   ├── config.staging.yaml                  # ステージング設定
│   └── config.prod.yaml                     # 本番設定（環境別）
├── tests/
│   └── integration_test.rs                  # REST API 統合テスト
└── Cargo.toml
```

### Cargo.toml

```toml
[package]
name = "k1s0-dlq-manager"
version = "0.1.0"
edition = "2021"

[lib]
name = "k1s0_dlq_manager"
path = "src/lib.rs"

[[bin]]
name = "k1s0-dlq-manager"
path = "src/main.rs"

[dependencies]
axum = { version = "0.7", features = ["macros"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["trace", "cors"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1"
thiserror = "2"
async-trait = "0.1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
rdkafka = { version = "0.36", features = ["cmake-build"] }
k1s0-dlq = { path = "../../../library/rust/dlq" }
k1s0-telemetry = { path = "../../../library/rust/telemetry" }

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
axum-test = "16"
tower = { version = "0.5", features = ["util"] }
```

### 主要依存ライブラリ

| ライブラリ | バージョン | 用途 |
|---------|----------|-----|
| axum | 0.7 | REST HTTP フレームワーク |
| tokio | 1 | 非同期ランタイム（full features） |
| sqlx | 0.8 | PostgreSQL 非同期ドライバー |
| rdkafka | 0.36 | Kafka プロデューサー / コンシューマー |
| serde / serde_json | 1 | JSON シリアライゼーション |
| serde_yaml | 0.9 | YAML 解析（設定ファイル） |
| uuid | 1 | UUID v4 生成 |
| chrono | 0.4 | 日時処理 |
| anyhow | 1 | エラーハンドリング |
| thiserror | 2 | エラー型定義 |
| async-trait | 0.1 | 非同期トレイト |
| tracing | 0.1 | 構造化ログ |
| k1s0-dlq | path | DLQ クライアントライブラリ |
| k1s0-telemetry | path | テレメトリライブラリ（OTel / メトリクス） |

---

## ドメインモデル

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

| フィールド | 型 | 説明 |
|---------|---|-----|
| `id` | `Uuid` | DLQ メッセージの一意識別子（v4 自動生成） |
| `original_topic` | `String` | 元のトピック名 |
| `error_message` | `String` | 処理失敗時のエラーメッセージ |
| `retry_count` | `i32` | 現在のリトライ回数 |
| `max_retries` | `i32` | 最大リトライ回数（デフォルト 3） |
| `payload` | `serde_json::Value` | メッセージペイロード |
| `status` | `DlqStatus` | メッセージステータス |
| `created_at` | `DateTime<Utc>` | 作成日時 |
| `updated_at` | `DateTime<Utc>` | 更新日時 |
| `last_retry_at` | `Option<DateTime<Utc>>` | 最終リトライ日時 |

**メソッド:**

| メソッド | 説明 |
|---------|-----|
| `new(original_topic, error_message, payload, max_retries)` | 初期状態（PENDING, retry_count=0）で作成 |
| `mark_retrying()` | status を RETRYING に遷移、retry_count を +1、last_retry_at を設定 |
| `mark_resolved()` | status を RESOLVED に遷移 |
| `mark_dead()` | status を DEAD に遷移 |
| `is_retryable()` | PENDING/RETRYING かつ retry_count < max_retries かどうかを返す |

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

| ステータス | 説明 | 終端 |
|-----------|-----|------|
| `Pending` | Kafka から取り込まれた初期状態 | No |
| `Retrying` | 再処理が開始された状態 | No |
| `Resolved` | 再処理が成功し、元トピックに再発行済み | Yes |
| `Dead` | 処理不能状態 | Yes |

`Display` トレイトで SCREAMING_SNAKE_CASE 文字列に変換する。`from_str_value` で文字列からの逆変換を提供する。

---

## リポジトリトレイト

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

| メソッド | 説明 |
|---------|-----|
| `find_by_id(id)` | ID で DLQ メッセージを検索する |
| `find_by_topic(topic, page, page_size)` | トピック別にページネーション付きで一覧取得する。総件数も返す |
| `create(message)` | DLQ メッセージを新規作成する |
| `update(message)` | DLQ メッセージを更新する |
| `delete(id)` | DLQ メッセージを削除する |
| `count_by_topic(topic)` | トピック別のメッセージ件数を取得する |

---

## ユースケース

| ユースケース | 責務 |
|------------|------|
| `ListMessagesUseCase` | トピック別に DLQ メッセージ一覧をページネーション付きで取得する |
| `GetMessageUseCase` | ID で DLQ メッセージ詳細を取得する。見つからなければエラー |
| `RetryMessageUseCase` | メッセージの再処理を行う。Kafka プロデューサーがある場合は元トピックに再発行し RESOLVED に遷移する。リトライ不可の場合はエラー |
| `DeleteMessageUseCase` | DLQ メッセージを削除する |
| `RetryAllUseCase` | トピック内の全リトライ可能メッセージを 100 件ずつ取得し、順次再処理する。成功件数を返す |

### RetryMessageUseCase の処理フロー

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

### RetryAllUseCase の処理フロー

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

## REST API エンドポイント

| Method | Path | ハンドラー関数 | 説明 |
|--------|------|-------------|-----|
| GET | `/healthz` | `healthz` | ヘルスチェック |
| GET | `/readyz` | `readyz` | レディネスチェック |
| GET | `/metrics` | `metrics` | Prometheus メトリクス |
| GET | `/api/v1/dlq/:topic` | `list_messages` | トピック別メッセージ一覧（ページネーション付き） |
| GET | `/api/v1/dlq/messages/:id` | `get_message` | メッセージ詳細取得 |
| POST | `/api/v1/dlq/messages/:id/retry` | `retry_message` | メッセージ再処理 |
| DELETE | `/api/v1/dlq/messages/:id` | `delete_message` | メッセージ削除 |
| POST | `/api/v1/dlq/:topic/retry-all` | `retry_all` | トピック内全メッセージ一括再処理 |

### ルーティング設計

`/api/v1/dlq/messages/:id` を `/api/v1/dlq/:topic` より先に定義し、パスパラメータの競合を回避する。

### REST ハンドラー DTO

**リクエスト:**

| DTO | フィールド | 用途 |
|-----|---------|-----|
| `ListMessagesQuery` | `page`(default:1), `page_size`(default:20) | メッセージ一覧フィルタ |

**レスポンス:**

| DTO | フィールド | 用途 |
|-----|---------|-----|
| `DlqMessageResponse` | `id`, `original_topic`, `error_message`, `retry_count`, `max_retries`, `payload`, `status`, `created_at`, `updated_at`, `last_retry_at` | メッセージ詳細 |
| `ListMessagesResponse` | `messages`, `pagination` | メッセージ一覧 |
| `PaginationResponse` | `total_count`, `page`, `page_size`, `has_next` | ページネーション |
| `RetryMessageResponse` | `id`, `status`, `message` | リトライ結果 |
| `RetryAllResponse` | `retried`, `message` | 一括リトライ結果 |
| `DeleteMessageResponse` | `success`, `message` | 削除結果 |

### DlqError

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

| バリアント | HTTP Status | エラーコード |
|-----------|-------------|------------|
| `NotFound` | 404 | `SYS_DLQ_NOT_FOUND` |
| `Validation` | 400 | `SYS_DLQ_VALIDATION_ERROR` |
| `Conflict` | 409 | `SYS_DLQ_CONFLICT` |
| `Internal` | 500 | `SYS_DLQ_INTERNAL_ERROR` |

`IntoResponse` トレイトを実装し、`ErrorResponse` 構造体（`code`, `message`, `request_id`, `details`）で統一エラーレスポンスを返す。

---

## インフラストラクチャ

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

| 設定ブロック | 主要フィールド | 説明 |
|------------|-------------|-----|
| `app` | `name`, `version`(default: "0.1.0"), `environment`(default: "dev") | アプリケーション識別情報 |
| `server` | `host`(default: "0.0.0.0"), `port`(default: 8080) | HTTP サーバー |
| `database` | `host`, `port`, `name`, `user`, `password`, `ssl_mode`, `max_open_conns` | PostgreSQL 接続（Optional） |
| `kafka` | `brokers`, `consumer_group`, `security_protocol`, `dlq_topic_pattern` | Kafka 接続（Optional） |

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

| メソッド | 説明 |
|---------|-----|
| `new(config, repo)` | Kafka コンシューマーを作成し、`dlq_topic_pattern` を購読する |
| `run()` | メッセージ取り込みループを開始する。受信メッセージから DlqMessage を作成し、リポジトリに永続化する |

コンシューマー設定:
- `auto.offset.reset`: `earliest`
- `enable.auto.commit`: `true`

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

| メソッド | 説明 |
|---------|-----|
| `new(config)` | rdkafka `FutureProducer` を作成する。acks=all, message.timeout.ms=5000 |
| `publish_to_topic(topic, payload)` | JSON シリアライズしたペイロードを指定トピックに発行する。UUID v4 をキーとして使用 |

### DlqPostgresRepository

```rust
// src/infrastructure/persistence/dlq_postgres.rs
pub struct DlqPostgresRepository {
    pool: PgPool,
}
```

PostgreSQL の `dlq.messages` テーブルに対する CRUD を提供する。`DlqMessageRow` 構造体で `sqlx::FromRow` を使い行マッピングを行い、`TryFrom<DlqMessageRow>` で `DlqMessage` に変換する。

SQL クエリ:
- `find_by_id`: `SELECT ... FROM dlq.messages WHERE id = $1`
- `find_by_topic`: `SELECT COUNT(*) ...` + `SELECT ... FROM dlq.messages WHERE original_topic = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3`
- `create`: `INSERT INTO dlq.messages (...) VALUES (...)`
- `update`: `UPDATE dlq.messages SET ... WHERE id = $1`
- `delete`: `DELETE FROM dlq.messages WHERE id = $1`
- `count_by_topic`: `SELECT COUNT(*) FROM dlq.messages WHERE original_topic = $1`

---

## データベース設計

### マイグレーション

マイグレーションファイルは `regions/system/database/dlq-db/migrations/` に格納する。

### スキーマ

データベースは `dlq` スキーマに配置する。

#### dlq_messages テーブル

| カラム | 型 | 制約 | 説明 |
| --- | --- | --- | --- |
| `id` | UUID | PK, DEFAULT gen_random_uuid() | メッセージ ID |
| `original_topic` | VARCHAR(255) | NOT NULL | 元のトピック名 |
| `error_message` | TEXT | NOT NULL | 処理失敗時のエラーメッセージ |
| `retry_count` | INT | NOT NULL, DEFAULT 0 | 現在のリトライ回数 |
| `max_retries` | INT | NOT NULL, DEFAULT 3 | 最大リトライ回数 |
| `payload` | JSONB | - | メッセージペイロード |
| `status` | VARCHAR(50) | NOT NULL, DEFAULT 'PENDING', CHECK制約 | ステータス |
| `created_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 作成日時 |
| `updated_at` | TIMESTAMPTZ | NOT NULL, DEFAULT NOW() | 更新日時（トリガー自動更新） |
| `last_retry_at` | TIMESTAMPTZ | - | 最終リトライ日時 |

**CHECK 制約:**
- `status` IN (`'PENDING'`, `'RETRYING'`, `'RESOLVED'`, `'DEAD'`)

**インデックス:**
- `idx_dlq_messages_original_topic` -- original_topic
- `idx_dlq_messages_status` -- status
- `idx_dlq_messages_created_at` -- created_at

### 自動 updated_at 更新トリガー

```sql
CREATE OR REPLACE FUNCTION dlq.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_dlq_messages_updated_at
    BEFORE UPDATE ON dlq.dlq_messages
    FOR EACH ROW
    EXECUTE FUNCTION dlq.update_updated_at_column();
```

### マイグレーション SQL

```sql
-- 001_create_schema.sql
CREATE SCHEMA IF NOT EXISTS dlq;

-- 002_create_dlq_messages.sql
CREATE TABLE dlq.dlq_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    original_topic VARCHAR(255) NOT NULL,
    error_message TEXT NOT NULL,
    retry_count INT NOT NULL DEFAULT 0,
    max_retries INT NOT NULL DEFAULT 3,
    payload JSONB,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING'
        CHECK (status IN ('PENDING', 'RETRYING', 'RESOLVED', 'DEAD')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_retry_at TIMESTAMPTZ
);

CREATE INDEX idx_dlq_messages_original_topic ON dlq.dlq_messages (original_topic);
CREATE INDEX idx_dlq_messages_status ON dlq.dlq_messages (status);
CREATE INDEX idx_dlq_messages_created_at ON dlq.dlq_messages (created_at);

-- 003_create_updated_at_trigger.sql
CREATE OR REPLACE FUNCTION dlq.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_dlq_messages_updated_at
    BEFORE UPDATE ON dlq.dlq_messages
    FOR EACH ROW
    EXECUTE FUNCTION dlq.update_updated_at_column();
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

---

## テスト構成

### ユニットテスト

各モジュール内の `#[cfg(test)]` ブロックで実装。mockall を使用してリポジトリ・Kafka パブリッシャーをモック化する。

| テスト対象 | テスト数 | 内容 |
|----------|--------|------|
| `domain/entity/dlq_message` | 13 | 新規作成、ステータス遷移（mark_retrying/resolved/dead）、リトライ判定（is_retryable）、Display/from_str、UUID 一意性 |
| `infrastructure/config` | 4 | 設定デシリアライズ、デフォルト値、DB 設定あり、Kafka 設定あり |
| `infrastructure/database` | 2 | 接続 URL 生成、設定デシリアライズ |
| `infrastructure/kafka/mod` | 2 | KafkaConfig デシリアライズ、デフォルト値 |
| `infrastructure/kafka/producer` | 2 | MockDlqEventPublisher 正常系、エラー系 |
| `usecase/list_messages` | 4 | 空一覧、結果あり、ページネーション、リポジトリエラー伝播 |
| `usecase/get_message` | 2 | 取得成功、未存在エラー |
| `usecase/retry_message` | 4 | 正常リトライ（publisher なし）、未存在エラー、リトライ不可（DEAD）、リトライ上限超過 |
| `usecase/delete_message` | 2 | 正常削除、エラー |
| `usecase/retry_all` | 3 | 空トピック、メッセージあり、非リトライ対象スキップ |
| `adapter/handler/dlq_handler` | 10 | healthz、readyz、一覧取得、詳細取得（成功/404/400）、削除（2件）、リトライ 404、一括リトライ |
| **合計** | **48** | |

### インテグレーションテスト

`tests/` ディレクトリに配置。InMemory リポジトリを使用した REST API の E2E テスト。

| テストファイル | 要件 | 内容 |
|-------------|------|------|
| `integration_test.rs` | InMemory | REST API の E2E テスト（11 テストケース） |

テスト一覧:

| テスト名 | 内容 |
|---------|------|
| `test_healthz_returns_ok` | ヘルスチェック正常応答 |
| `test_readyz_returns_ok` | レディネスチェック正常応答 |
| `test_list_messages_empty_topic` | 空トピックで total_count=0 |
| `test_list_messages_returns_stored_message` | 格納済みメッセージの取得 |
| `test_get_message_returns_404_when_not_found` | 未存在メッセージで 404 |
| `test_get_message_returns_message` | メッセージ詳細の正常取得 |
| `test_get_message_returns_400_for_invalid_id` | 不正 UUID で 400 |
| `test_retry_message_returns_404_when_not_found` | 未存在メッセージのリトライで 404 |
| `test_retry_message_resolves_pending_message` | PENDING メッセージが RESOLVED になる |
| `test_delete_message_returns_ok` | メッセージ削除成功 |
| `test_retry_all_returns_retried_count` | 一括リトライで retried 件数を返す |

---

## 関連ドキュメント

- [system-dlq-manager-server設計.md](system-dlq-manager-server設計.md) -- 概要・API 定義・アーキテクチャ
- [system-library-dlq-client設計.md](system-library-dlq-client設計.md) -- DLQ クライアントライブラリ設計
- [system-library-概要.md](system-library-概要.md) -- ライブラリ一覧・テスト方針
- [メッセージング設計.md](メッセージング設計.md) -- DLQ パターンの基本方針
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート仕様
- [config設計.md](config設計.md) -- config.yaml スキーマと環境別管理
- [コーディング規約.md](コーディング規約.md) -- Linter・Formatter・命名規則
