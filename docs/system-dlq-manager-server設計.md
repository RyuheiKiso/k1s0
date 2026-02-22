# system-dlq-manager-server 設計

system tier の DLQ Manager サーバー設計を定義する。Kafka のデッドレタートピック（`*.dlq.v1`）に送られた処理失敗メッセージを集約管理し、REST API 経由でメッセージの一覧取得・詳細取得・再処理・削除・一括再処理を提供する。
Rust での実装を定義する。

## 概要

system tier の DLQ Manager は以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| DLQ メッセージ一覧取得 | トピック別に DLQ メッセージをページネーション付きで一覧取得する |
| DLQ メッセージ詳細取得 | ID 指定で個別の DLQ メッセージ詳細を取得する |
| DLQ メッセージ再処理 | 個別メッセージを元トピックに再発行し、ステータスを RESOLVED に遷移する |
| DLQ メッセージ削除 | 不要なメッセージを削除する |
| 一括再処理 | トピック内の全リトライ可能メッセージを一括で再処理する |
| DLQ メッセージ自動取り込み | Kafka コンシューマーが `*.dlq.v1` パターンのトピックを購読し、新規 DLQ メッセージを自動で DB に永続化する |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| DB アクセス | sqlx v0.8 |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | k1s0-telemetry |
| 設定管理 | serde_yaml |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/dlq-manager/` |

---

## 設計方針

[メッセージング設計.md](メッセージング設計.md) の DLQ パターンに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| メッセージ取り込み | Kafka コンシューマーが `*.dlq.v1` パターンを購読し、バックグラウンドで自動取り込み |
| 再処理方式 | 元トピックへの Kafka 再発行（プロデューサー経由） |
| リトライ上限 | メッセージ毎に `max_retries`（デフォルト 3）で制御 |
| 状態管理 | PostgreSQL に永続化（DLQ スキーマ） |
| ステータス遷移 | PENDING -> RETRYING -> RESOLVED / DEAD |
| Kafka オプショナル | Kafka 未設定時も REST API は動作する（再処理時は Kafka 再発行をスキップ） |
| DB オプショナル | DB 未設定時はインメモリリポジトリで動作（開発・テスト用） |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_DLQ_` とする。

| Method | Path | Description |
| --- | --- | --- |
| GET | `/api/v1/dlq/:topic` | トピック別 DLQ メッセージ一覧取得（ページネーション付き） |
| GET | `/api/v1/dlq/messages/:id` | DLQ メッセージ詳細取得 |
| POST | `/api/v1/dlq/messages/:id/retry` | DLQ メッセージ再処理 |
| DELETE | `/api/v1/dlq/messages/:id` | DLQ メッセージ削除 |
| POST | `/api/v1/dlq/:topic/retry-all` | トピック内全メッセージ一括再処理 |
| GET | `/healthz` | ヘルスチェック |
| GET | `/readyz` | レディネスチェック |
| GET | `/metrics` | Prometheus メトリクス |

#### GET /api/v1/dlq/:topic

トピック別に DLQ メッセージ一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 20 | 1 ページあたりの件数 |

**レスポンス（200 OK）**

```json
{
  "messages": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "original_topic": "orders.events.v1",
      "error_message": "processing failed",
      "retry_count": 0,
      "max_retries": 3,
      "payload": {"order_id": "123"},
      "status": "PENDING",
      "created_at": "2026-02-20T10:30:00.000+00:00",
      "updated_at": "2026-02-20T10:30:00.000+00:00",
      "last_retry_at": null
    }
  ],
  "pagination": {
    "total_count": 150,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}
```

#### GET /api/v1/dlq/messages/:id

ID 指定で DLQ メッセージの詳細を取得する。

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "original_topic": "orders.events.v1",
  "error_message": "processing failed",
  "retry_count": 1,
  "max_retries": 3,
  "payload": {"order_id": "123"},
  "status": "RETRYING",
  "created_at": "2026-02-20T10:30:00.000+00:00",
  "updated_at": "2026-02-20T10:31:00.000+00:00",
  "last_retry_at": "2026-02-20T10:31:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_DLQ_NOT_FOUND",
    "message": "dlq message not found: invalid-uuid",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

**レスポンス（400 Bad Request）**

```json
{
  "error": {
    "code": "SYS_DLQ_VALIDATION_ERROR",
    "message": "invalid message id: not-a-uuid",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### POST /api/v1/dlq/messages/:id/retry

DLQ メッセージを再処理する。Kafka プロデューサーが設定されている場合は元トピックにメッセージを再発行し、ステータスを RESOLVED に遷移する。リトライ不可能なメッセージ（DEAD / RESOLVED、またはリトライ上限到達）に対しては 409 Conflict を返す。

**レスポンス（200 OK）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "RESOLVED",
  "message": "message retry initiated"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_DLQ_NOT_FOUND",
    "message": "dlq message not found: 550e8400-e29b-41d4-a716-446655440000",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_DLQ_CONFLICT",
    "message": "message is not retryable: status=DEAD, retry_count=3/3",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### DELETE /api/v1/dlq/messages/:id

DLQ メッセージを削除する。

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "message 550e8400-e29b-41d4-a716-446655440000 deleted"
}
```

#### POST /api/v1/dlq/:topic/retry-all

指定トピック内の全リトライ可能メッセージを一括再処理する。DEAD / RESOLVED ステータスやリトライ上限到達メッセージはスキップされる。ページネーション（100 件ずつ）でメッセージを取得し、順次処理する。

**レスポンス（200 OK）**

```json
{
  "retried": 15,
  "message": "15 messages retried in topic orders.dlq.v1"
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_DLQ_NOT_FOUND` | 404 | 指定された DLQ メッセージが見つからない |
| `SYS_DLQ_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー（不正な UUID 等） |
| `SYS_DLQ_CONFLICT` | 409 | リトライ不可能なメッセージに対する再処理要求 |
| `SYS_DLQ_INTERNAL_ERROR` | 500 | 内部エラー |

---

## DLQ メッセージ状態遷移

### ステータス一覧

| ステータス | 説明 |
| --- | --- |
| `PENDING` | Kafka から取り込まれた初期状態 |
| `RETRYING` | 再処理が開始された状態 |
| `RESOLVED` | 再処理が成功し、元トピックに再発行済み（終端状態） |
| `DEAD` | リトライ上限に達した処理不能状態（終端状態） |

### 状態遷移図

```
  PENDING ──▶ RETRYING ──▶ RESOLVED (終端)
    │              │
    │              │ Kafka再発行失敗
    │              ▼
    │          RETRYING (retry_count++, 再試行可能)
    │
    └─────────────▶ DEAD (終端: mark_dead() による手動遷移)
```

### リトライ判定ロジック

メッセージが `is_retryable()` を満たす条件:

1. ステータスが `PENDING` または `RETRYING` であること
2. `retry_count < max_retries` であること

上記を満たさない場合、再処理リクエストは 409 Conflict で拒否される。

---

## Kafka コンシューマー設計

### DLQ トピック自動取り込み

DLQ Manager は Kafka コンシューマーをバックグラウンドタスクとして起動し、`*.dlq.v1` パターンに一致するトピックを自動購読する。

| 設定項目 | 値 |
| --- | --- |
| トピックパターン | `*.dlq.v1`（config.yaml の `kafka.dlq_topic_pattern`） |
| コンシューマーグループ | `dlq-manager.default` |
| auto.offset.reset | `earliest` |
| enable.auto.commit | `true` |

### メッセージ取り込みフロー

```
1. Kafka コンシューマーがメッセージを受信
2. ペイロードを JSON にデシリアライズ（失敗時は null）
3. Kafka ヘッダーの "error" キーからエラーメッセージを取得（未設定時は "unknown error"）
4. DlqMessage エンティティを作成（status=PENDING, max_retries=3）
5. リポジトリ経由で DB に永続化
6. ログ出力（message_id, topic）
```

### 再発行（プロデューサー）

再処理時にメッセージを元トピックに再発行する Kafka プロデューサー。

| 設定項目 | 値 |
| --- | --- |
| acks | `all` |
| message.timeout.ms | `5000` |
| キー | UUID v4（新規生成） |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の 4 レイヤー構成に従う。

```
domain（エンティティ・リポジトリインターフェース）
  ^
usecase（ビジネスロジック）
  ^
adapter（ハンドラー・プレゼンター）
  ^
infra（DB接続・Kafka Consumer/Producer・設定ローダー）
```

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `DlqMessage`, `DlqStatus` | エンティティ定義・状態遷移 |
| domain/repository | `DlqMessageRepository` | リポジトリトレイト |
| usecase | `ListMessagesUseCase`, `GetMessageUseCase`, `RetryMessageUseCase`, `DeleteMessageUseCase`, `RetryAllUseCase` | ユースケース |
| adapter/handler | REST ハンドラー | プロトコル変換（axum） |
| infra/config | Config ローダー | config.yaml の読み込み |
| infra/database | DatabaseConfig | DB 接続設定 |
| infra/kafka/consumer | `DlqKafkaConsumer` | DLQ トピック購読・メッセージ取り込み |
| infra/kafka/producer | `DlqEventPublisher`, `DlqKafkaProducer` | 元トピックへの再発行 |
| infra/persistence | `DlqPostgresRepository` | PostgreSQL リポジトリ実装 |

### ドメインモデル

#### DlqMessage

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | DLQ メッセージの一意識別子 |
| `original_topic` | String | 元のトピック名 |
| `error_message` | String | 処理失敗時のエラーメッセージ |
| `retry_count` | i32 | 現在のリトライ回数 |
| `max_retries` | i32 | 最大リトライ回数（デフォルト 3） |
| `payload` | JSON | メッセージペイロード |
| `status` | DlqStatus | メッセージステータス（PENDING / RETRYING / RESOLVED / DEAD） |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 更新日時 |
| `last_retry_at` | Option\<DateTime\<Utc\>\> | 最終リトライ日時 |

**メソッド:**
- `new()` -- 初期状態（PENDING, retry_count=0）で作成
- `mark_retrying()` -- ステータスを RETRYING に遷移、retry_count をインクリメント、last_retry_at を更新
- `mark_resolved()` -- ステータスを RESOLVED に遷移
- `mark_dead()` -- ステータスを DEAD に遷移
- `is_retryable()` -- リトライ可能かどうか（PENDING/RETRYING かつ retry_count < max_retries）

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (dlq_handler.rs)            │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  list_messages / get_message             │   │
                    │  │  retry_message / delete_message          │   │
                    │  │  retry_all                               │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  ListMessagesUseCase / GetMessageUseCase /      │
                    │  RetryMessageUseCase / DeleteMessageUseCase /   │
                    │  RetryAllUseCase                                │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/entity  │              │ domain/repository          │   │
    │  DlqMessage,    │              │ DlqMessageRepository       │   │
    │  DlqStatus      │              │ (trait)                    │   │
    └────────────────┘              └──────────┬─────────────────┘   │
                                               │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │                  infra 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ DlqPostgresRepository  │  │
                    │  │ Consumer +   │  │ InMemoryDlqRepository  │  │
                    │  │ Producer     │  │ (dev/test)             │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐  ┌────────────────────────┐  │
                    │  │ Config       │  │ Database               │  │
                    │  │ Loader       │  │ Config                 │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    └────────────────────────────────────────────────┘
```

---

## テスト方針

### ユニットテスト

各モジュール内の `#[cfg(test)]` ブロックで実装。mockall を使用してリポジトリ・Kafka パブリッシャーをモック化。

| テスト対象 | テスト数 | 内容 |
| --- | --- | --- |
| domain/entity/dlq_message | 13 | ステータス遷移、リトライ判定、Display/from_str、UUID 生成 |
| infrastructure/config | 4 | 設定デシリアライズ、デフォルト値、DB/Kafka 設定 |
| infrastructure/database | 2 | 接続 URL 生成、設定デシリアライズ |
| infrastructure/kafka/mod | 2 | KafkaConfig デシリアライズ、デフォルト値 |
| infrastructure/kafka/producer | 2 | MockDlqEventPublisher 正常/エラー |
| usecase/list_messages | 4 | 空一覧、結果あり、ページネーション、リポジトリエラー |
| usecase/get_message | 2 | 取得成功、未存在 |
| usecase/retry_message | 4 | 正常リトライ、未存在、リトライ不可、上限超過 |
| usecase/delete_message | 2 | 正常削除、エラー |
| usecase/retry_all | 3 | 空トピック、メッセージあり、非リトライ対象スキップ |
| adapter/handler/dlq_handler | 10 | healthz/readyz、一覧、詳細取得（成功/404/400）、削除、リトライ、一括リトライ |
| **合計** | **48** | |

### 統合テスト

`tests/` ディレクトリに配置。インメモリリポジトリを使用した REST API のエンドツーエンド動作を検証する。

| テストファイル | 要件 | 内容 |
| --- | --- | --- |
| `integration_test.rs` | InMemory | REST API の E2E テスト（12 テストケース） |

統合テスト一覧:
- `test_healthz_returns_ok` -- ヘルスチェック正常
- `test_readyz_returns_ok` -- レディネスチェック正常
- `test_list_messages_empty_topic` -- 空トピックの一覧取得
- `test_list_messages_returns_stored_message` -- メッセージありの一覧取得
- `test_get_message_returns_404_when_not_found` -- 未存在メッセージ取得時 404
- `test_get_message_returns_message` -- メッセージ取得成功
- `test_get_message_returns_400_for_invalid_id` -- 不正 UUID 時 400
- `test_retry_message_returns_404_when_not_found` -- 未存在メッセージリトライ時 404
- `test_retry_message_resolves_pending_message` -- PENDING メッセージのリトライ成功
- `test_delete_message_returns_ok` -- メッセージ削除成功
- `test_retry_all_returns_retried_count` -- 一括リトライの件数返却

---

## 設定ファイル

### config.yaml（本番）

```yaml
app:
  name: "dlq-manager"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080

database:
  host: "postgres.k1s0-system.svc.cluster.local"
  port: 5432
  name: "k1s0_dlq"
  user: "app"
  password: ""
  ssl_mode: "disable"
  max_open_conns: 25
  max_idle_conns: 5
  conn_max_lifetime: "5m"

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  consumer_group: "dlq-manager.default"
  security_protocol: "PLAINTEXT"
  dlq_topic_pattern: "*.dlq.v1"
```

---

## デプロイ

### Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。dlq-manager 固有の values は以下の通り。

```yaml
# values-dlq-manager.yaml（infra/helm/services/system/dlq-manager/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/dlq-manager
  tag: ""

replicaCount: 2

container:
  port: 8080
  grpcPort: null    # HTTP only（gRPC なし）

service:
  type: ClusterIP
  port: 80
  grpcPort: null

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 5
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/dlq-manager/database"
      key: "password"
      mountPath: "/vault/secrets/db-password"
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/dlq-manager/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 関連ドキュメント

- [メッセージング設計.md](メッセージング設計.md) -- DLQ パターンの基本方針・採用基準
- [system-dlq-manager-server-実装設計.md](system-dlq-manager-server-実装設計.md) -- 実装設計の詳細
- [system-library-dlq-client設計.md](system-library-dlq-client設計.md) -- DLQ クライアントライブラリ設計
- [system-library-概要.md](system-library-概要.md) -- ライブラリ一覧
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート仕様
- [API設計.md](API設計.md) -- REST API 設計ガイドライン
- [REST-API設計.md](REST-API設計.md) -- D-007 統一エラーレスポンス
- [可観測性設計.md](可観測性設計.md) -- メトリクス・トレース設計
- [config設計.md](config設計.md) -- config.yaml スキーマ
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
