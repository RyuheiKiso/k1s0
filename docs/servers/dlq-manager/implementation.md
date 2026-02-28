# system-dlq-manager-server 実装設計

> **ガイド**: 設計背景・実装例は [implementation.guide.md](./implementation.guide.md) を参照。

system-dlq-manager-server（DLQ メッセージ管理サーバー）の Rust 実装詳細を定義する。概要・API 定義・アーキテクチャは [system-dlq-manager-server.md](server.md) を参照。

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

> 共通依存は [Rust共通実装.md](../_common/Rust共通実装.md#共通cargo依存) を参照。サービス固有の追加依存:

```toml
async-trait = "0.1"
k1s0-dlq = { path = "../../../library/rust/dlq" }
k1s0-telemetry = { path = "../../../library/rust/telemetry" }

[dev-dependencies]
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

| ステータス | 説明 | 終端 |
|-----------|-----|------|
| `Pending` | Kafka から取り込まれた初期状態 | No |
| `Retrying` | 再処理が開始された状態 | No |
| `Resolved` | 再処理が成功し、元トピックに再発行済み | Yes |
| `Dead` | 処理不能状態 | Yes |

> 実装コードは [implementation.guide.md](./implementation.guide.md#ドメインモデル実装コード) を参照。

---

## リポジトリトレイト

### DlqMessageRepository

| メソッド | 説明 |
|---------|-----|
| `find_by_id(id)` | ID で DLQ メッセージを検索する |
| `find_by_topic(topic, page, page_size)` | トピック別にページネーション付きで一覧取得する。総件数も返す |
| `create(message)` | DLQ メッセージを新規作成する |
| `update(message)` | DLQ メッセージを更新する |
| `delete(id)` | DLQ メッセージを削除する |
| `count_by_topic(topic)` | トピック別のメッセージ件数を取得する |

> トレイト定義コードは [implementation.guide.md](./implementation.guide.md#リポジトリトレイト実装コード) を参照。

---

## ユースケース

| ユースケース | 責務 |
|------------|------|
| `ListMessagesUseCase` | トピック別に DLQ メッセージ一覧をページネーション付きで取得する |
| `GetMessageUseCase` | ID で DLQ メッセージ詳細を取得する。見つからなければエラー |
| `RetryMessageUseCase` | メッセージの再処理を行う。Kafka プロデューサーがある場合は元トピックに再発行し RESOLVED に遷移する。リトライ不可の場合はエラー |
| `DeleteMessageUseCase` | DLQ メッセージを削除する |
| `RetryAllUseCase` | トピック内の全リトライ可能メッセージを 100 件ずつ取得し、順次再処理する。成功件数を返す |

> 処理フロー詳細は [implementation.guide.md](./implementation.guide.md#ユースケース処理フロー) を参照。

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

| バリアント | HTTP Status | エラーコード |
|-----------|-------------|------------|
| `NotFound` | 404 | `SYS_DLQ_NOT_FOUND` |
| `Validation` | 400 | `SYS_DLQ_VALIDATION_ERROR` |
| `Conflict` | 409 | `SYS_DLQ_CONFLICT` |
| `Internal` | 500 | `SYS_DLQ_INTERNAL_ERROR` |

---

## インフラストラクチャ

### Config

| 設定ブロック | 主要フィールド | 説明 |
|------------|-------------|-----|
| `app` | `name`, `version`(default: "0.1.0"), `environment`(default: "dev") | アプリケーション識別情報 |
| `server` | `host`(default: "0.0.0.0"), `port`(default: 8080) | HTTP サーバー |
| `database` | `host`, `port`, `name`, `user`, `password`, `ssl_mode`, `max_open_conns` | PostgreSQL 接続（Optional） |
| `kafka` | `brokers`, `consumer_group`, `security_protocol`, `dlq_topic_pattern` | Kafka 接続（Optional） |

### DlqKafkaConsumer

| メソッド | 説明 |
|---------|-----|
| `new(config, repo)` | Kafka コンシューマーを作成し、`dlq_topic_pattern` を購読する |
| `run()` | メッセージ取り込みループを開始する。受信メッセージから DlqMessage を作成し、リポジトリに永続化する |

コンシューマー設定:
- `auto.offset.reset`: `earliest`
- `enable.auto.commit`: `true`

### DlqEventPublisher / DlqKafkaProducer

| メソッド | 説明 |
|---------|-----|
| `new(config)` | rdkafka `FutureProducer` を作成する。acks=all, message.timeout.ms=5000 |
| `publish_to_topic(topic, payload)` | JSON シリアライズしたペイロードを指定トピックに発行する。UUID v4 をキーとして使用 |

### DlqPostgresRepository

SQL クエリ:
- `find_by_id`: `SELECT ... FROM dlq.messages WHERE id = $1`
- `find_by_topic`: `SELECT COUNT(*) ...` + `SELECT ... FROM dlq.messages WHERE original_topic = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3`
- `create`: `INSERT INTO dlq.messages (...) VALUES (...)`
- `update`: `UPDATE dlq.messages SET ... WHERE id = $1`
- `delete`: `DELETE FROM dlq.messages WHERE id = $1`
- `count_by_topic`: `SELECT COUNT(*) FROM dlq.messages WHERE original_topic = $1`

### データベース設計

マイグレーションファイルは `regions/system/database/dlq-db/migrations/` に格納する。データベースは `dlq` スキーマに配置する。

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

**CHECK 制約:** `status` IN (`'PENDING'`, `'RETRYING'`, `'RESOLVED'`, `'DEAD'`)

**インデックス:**
- `idx_dlq_messages_original_topic` -- original_topic
- `idx_dlq_messages_status` -- status
- `idx_dlq_messages_created_at` -- created_at

> Bootstrap手順・実装コードは [implementation.guide.md](./implementation.guide.md#bootstrap-手順) を参照。

---

## テスト構成

### ユニットテスト

| テスト対象 | テスト数 | 内容 |
|----------|--------|------|
| `domain/entity/dlq_message` | 13 | 新規作成、ステータス遷移、リトライ判定、Display/from_str、UUID 一意性 |
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

`tests/` ディレクトリに配置。InMemory リポジトリを使用した REST API の統合テスト。

| テストファイル | 要件 | 内容 |
|-------------|------|------|
| `integration_test.rs` | InMemory | REST API の統合テスト（15 テストケース） |

---

## 関連ドキュメント

- [system-dlq-manager-server.md](server.md) -- 概要・API 定義・アーキテクチャ
- [system-library-dlq-client.md](../../libraries/messaging/dlq-client.md) -- DLQ クライアントライブラリ設計
- [system-library-概要.md](../../libraries/_common/概要.md) -- ライブラリ一覧・テスト方針
- [メッセージング設計.md](../../architecture/messaging/メッセージング設計.md) -- DLQ パターンの基本方針
- [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) -- サーバーテンプレート仕様
- [config.md](../../cli/config/config設計.md) -- config.yaml スキーマと環境別管理
- [コーディング規約.md](../../architecture/conventions/コーディング規約.md) -- Linter・Formatter・命名規則
