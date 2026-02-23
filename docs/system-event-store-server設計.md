# system-event-store-server 設計

system tier の CQRS パターン向けイベントソーシングサーバー設計を定義する。k1s0-eventstore ライブラリをサービス化し、Append-only イベントストリームの REST/gRPC API を提供する。スナップショット管理・イベントリプレイをサポートし、永続化されたイベントを Kafka トピック `k1s0.system.eventstore.event.published.v1` へ転送する。Rust での実装を定義する。

## 概要

system tier のイベントソーシングサーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| イベント追記 | Append-only イベントストリームへのイベント追記（楽観的ロック付き） |
| イベント取得 | ストリーム単位・シーケンス番号指定・バージョン範囲でのイベント取得 |
| スナップショット管理 | 集約状態のスナップショット作成・最新スナップショット取得 |
| イベントリプレイ | プロジェクション更新のためのイベント順次再生 |
| Kafka 転送 | 追記されたイベントを Kafka `k1s0.system.eventstore.event.published.v1` へ非同期転送 |

### 技術スタック

| コンポーネント | Rust |
| --- | --- |
| HTTP フレームワーク | axum + tokio |
| gRPC | tonic v0.12 |
| DB アクセス | sqlx v0.8（PostgreSQL append-only） |
| Kafka | rdkafka (rust-rdkafka) |
| OTel | opentelemetry v0.27 |
| 設定管理 | serde_yaml |
| シリアライゼーション | serde + serde_json |
| 非同期ランタイム | tokio 1 (full) |

### 配置パス

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の Tier 別配置パスに従い、以下に配置する。

| 言語 | パス |
| --- | --- |
| Rust | `regions/system/server/rust/event-store/` |

---

## 設計方針

[認証認可設計.md](認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

| 項目 | 設計 |
| --- | --- |
| 実装言語 | Rust |
| ストレージ | PostgreSQL の `event_store` スキーマ（event_streams, events, snapshots テーブル）。events テーブルは追記のみ（UPDATE/DELETE 禁止） |
| 楽観的ロック | イベント追記時に `expected_version` を指定し、競合時は `SYS_EVSTORE_VERSION_CONFLICT` を返す |
| Kafka 転送 | イベント追記後、バックグラウンドタスクが Kafka へ非同期転送（at-least-once 保証） |
| スナップショット | スナップショット取得後、そのバージョン以降のイベントのみ適用することで状態再構築を高速化 |
| 認可 | 参照は `sys_auditor`、追記・スナップショット作成は `sys_operator`、ストリーム削除は `sys_admin` |
| ポート | ホスト側 8099（内部 8080）、gRPC 9090 |

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_EVSTORE_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/streams/:stream_id/events` | イベント追記 | `sys_operator` 以上 |
| GET | `/api/v1/streams/:stream_id/events` | イベント一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/streams/:stream_id/events/:sequence` | 特定イベント取得 | `sys_auditor` 以上 |
| POST | `/api/v1/streams/:stream_id/snapshots` | スナップショット作成 | `sys_operator` 以上 |
| GET | `/api/v1/streams/:stream_id/snapshots/latest` | 最新スナップショット取得 | `sys_auditor` 以上 |
| DELETE | `/api/v1/streams/:stream_id` | ストリーム削除 | `sys_admin` のみ |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |
| GET | `/metrics` | Prometheus メトリクス | 不要 |

#### POST /api/v1/streams/:stream_id/events

イベントをストリームに追記する。`expected_version` を指定することで楽観的ロックを実現する。`expected_version` が `-1` の場合はストリームが存在しないことを期待する（新規ストリーム作成）。

**リクエスト**

```json
{
  "events": [
    {
      "event_type": "OrderPlaced",
      "payload": {
        "order_id": "order-001",
        "tenant_id": "tenant-abc",
        "items": [
          {"product_id": "prod-001", "quantity": 2, "unit_price": 1500}
        ],
        "total_amount": 3000
      },
      "metadata": {
        "actor_id": "user-001",
        "correlation_id": "corr_01JABCDEF1234567890",
        "causation_id": null
      }
    }
  ],
  "expected_version": 0
}
```

**レスポンス（201 Created）**

```json
{
  "stream_id": "order-order-001",
  "events": [
    {
      "stream_id": "order-order-001",
      "sequence": 1,
      "event_type": "OrderPlaced",
      "version": 1,
      "payload": {
        "order_id": "order-001",
        "tenant_id": "tenant-abc",
        "items": [
          {"product_id": "prod-001", "quantity": 2, "unit_price": 1500}
        ],
        "total_amount": 3000
      },
      "metadata": {
        "actor_id": "user-001",
        "correlation_id": "corr_01JABCDEF1234567890",
        "causation_id": null
      },
      "occurred_at": "2026-02-23T10:00:00.000+00:00",
      "stored_at": "2026-02-23T10:00:00.012+00:00"
    }
  ],
  "current_version": 1
}
```

**レスポンス（409 Conflict）**

```json
{
  "error": {
    "code": "SYS_EVSTORE_VERSION_CONFLICT",
    "message": "version conflict for stream order-order-001: expected 0, actual 3",
    "request_id": "req_abc123def456",
    "details": [
      {"field": "expected_version", "message": "0"},
      {"field": "actual_version", "message": "3"}
    ]
  }
}
```

#### GET /api/v1/streams/:stream_id/events

ストリームのイベント一覧をページネーション付きで取得する。`from_version` / `to_version` でバージョン範囲を絞り込める。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `from_version` | int | No | 1 | 取得開始バージョン（含む） |
| `to_version` | int | No | - | 取得終了バージョン（含む）。省略時は最新まで |
| `event_type` | string | No | - | イベント種別でフィルタ |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 50 | 1 ページあたりの件数（最大 200） |

**レスポンス（200 OK）**

```json
{
  "stream_id": "order-order-001",
  "events": [
    {
      "stream_id": "order-order-001",
      "sequence": 1,
      "event_type": "OrderPlaced",
      "version": 1,
      "payload": {
        "order_id": "order-001",
        "tenant_id": "tenant-abc",
        "total_amount": 3000
      },
      "metadata": {
        "actor_id": "user-001",
        "correlation_id": "corr_01JABCDEF1234567890",
        "causation_id": null
      },
      "occurred_at": "2026-02-23T10:00:00.000+00:00",
      "stored_at": "2026-02-23T10:00:00.012+00:00"
    },
    {
      "stream_id": "order-order-001",
      "sequence": 2,
      "event_type": "OrderShipped",
      "version": 2,
      "payload": {
        "order_id": "order-001",
        "tracking_number": "TRK-12345"
      },
      "metadata": {
        "actor_id": "user-002",
        "correlation_id": "corr_02JABCDEF1234567890",
        "causation_id": "corr_01JABCDEF1234567890"
      },
      "occurred_at": "2026-02-23T14:00:00.000+00:00",
      "stored_at": "2026-02-23T14:00:00.008+00:00"
    }
  ],
  "current_version": 2,
  "pagination": {
    "total_count": 2,
    "page": 1,
    "page_size": 50,
    "has_next": false
  }
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_EVSTORE_STREAM_NOT_FOUND",
    "message": "stream not found: order-order-999",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /api/v1/streams/:stream_id/events/:sequence

シーケンス番号で特定のイベントを取得する。

**レスポンス（200 OK）**

```json
{
  "stream_id": "order-order-001",
  "sequence": 1,
  "event_type": "OrderPlaced",
  "version": 1,
  "payload": {
    "order_id": "order-001",
    "tenant_id": "tenant-abc",
    "total_amount": 3000
  },
  "metadata": {
    "actor_id": "user-001",
    "correlation_id": "corr_01JABCDEF1234567890",
    "causation_id": null
  },
  "occurred_at": "2026-02-23T10:00:00.000+00:00",
  "stored_at": "2026-02-23T10:00:00.012+00:00"
}
```

#### POST /api/v1/streams/:stream_id/snapshots

集約の現在状態をスナップショットとして保存する。`snapshot_version` には状態が対応するイベントのバージョンを指定する。

**リクエスト**

```json
{
  "snapshot_version": 2,
  "aggregate_type": "Order",
  "state": {
    "order_id": "order-001",
    "status": "shipped",
    "tenant_id": "tenant-abc",
    "total_amount": 3000,
    "tracking_number": "TRK-12345"
  }
}
```

**レスポンス（201 Created）**

```json
{
  "id": "snap_01JABCDEF1234567890",
  "stream_id": "order-order-001",
  "snapshot_version": 2,
  "aggregate_type": "Order",
  "created_at": "2026-02-23T15:00:00.000+00:00"
}
```

#### GET /api/v1/streams/:stream_id/snapshots/latest

ストリームの最新スナップショットを取得する。スナップショットが存在しない場合は `404` を返す。

**レスポンス（200 OK）**

```json
{
  "id": "snap_01JABCDEF1234567890",
  "stream_id": "order-order-001",
  "snapshot_version": 2,
  "aggregate_type": "Order",
  "state": {
    "order_id": "order-001",
    "status": "shipped",
    "tenant_id": "tenant-abc",
    "total_amount": 3000,
    "tracking_number": "TRK-12345"
  },
  "created_at": "2026-02-23T15:00:00.000+00:00"
}
```

**レスポンス（404 Not Found）**

```json
{
  "error": {
    "code": "SYS_EVSTORE_SNAPSHOT_NOT_FOUND",
    "message": "no snapshot found for stream: order-order-001",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### DELETE /api/v1/streams/:stream_id

ストリームとそれに紐づく全イベント・スナップショットを削除する。`sys_admin` のみ実行可能。イベントソーシングでは原則として使用しない（監査・テスト用途に限定する）。

**レスポンス（200 OK）**

```json
{
  "success": true,
  "message": "stream order-order-001 and all related data deleted"
}
```

### エラーコード

| コード | HTTP Status | 説明 |
| --- | --- | --- |
| `SYS_EVSTORE_STREAM_NOT_FOUND` | 404 | 指定されたストリームが見つからない |
| `SYS_EVSTORE_EVENT_NOT_FOUND` | 404 | 指定されたシーケンス番号のイベントが見つからない |
| `SYS_EVSTORE_SNAPSHOT_NOT_FOUND` | 404 | スナップショットが存在しない |
| `SYS_EVSTORE_VERSION_CONFLICT` | 409 | 楽観的ロックの競合（expected_version が一致しない） |
| `SYS_EVSTORE_STREAM_ALREADY_EXISTS` | 409 | expected_version=-1 を指定したが既にストリームが存在する |
| `SYS_EVSTORE_VALIDATION_ERROR` | 400 | リクエストのバリデーションエラー |
| `SYS_EVSTORE_INTERNAL_ERROR` | 500 | 内部エラー |

### gRPC サービス定義

```protobuf
syntax = "proto3";
package k1s0.system.eventstore.v1;

service EventStoreService {
  rpc AppendEvents(AppendEventsRequest) returns (AppendEventsResponse);
  rpc ReadEvents(ReadEventsRequest) returns (ReadEventsResponse);
  rpc ReadEventBySequence(ReadEventBySequenceRequest) returns (ReadEventBySequenceResponse);
  rpc CreateSnapshot(CreateSnapshotRequest) returns (CreateSnapshotResponse);
  rpc GetLatestSnapshot(GetLatestSnapshotRequest) returns (GetLatestSnapshotResponse);
}

message AppendEventsRequest {
  string stream_id = 1;
  repeated EventData events = 2;
  int64 expected_version = 3;
}

message AppendEventsResponse {
  string stream_id = 1;
  repeated StoredEvent events = 2;
  int64 current_version = 3;
}

message ReadEventsRequest {
  string stream_id = 1;
  int64 from_version = 2;
  optional int64 to_version = 3;
  uint32 page = 4;
  uint32 page_size = 5;
}

message ReadEventsResponse {
  string stream_id = 1;
  repeated StoredEvent events = 2;
  int64 current_version = 3;
  uint64 total_count = 4;
  bool has_next = 5;
}

message ReadEventBySequenceRequest {
  string stream_id = 1;
  uint64 sequence = 2;
}

message ReadEventBySequenceResponse {
  StoredEvent event = 1;
}

message CreateSnapshotRequest {
  string stream_id = 1;
  int64 snapshot_version = 2;
  string aggregate_type = 3;
  bytes state_json = 4;
}

message CreateSnapshotResponse {
  string id = 1;
  string stream_id = 2;
  int64 snapshot_version = 3;
  string created_at = 4;
}

message GetLatestSnapshotRequest {
  string stream_id = 1;
}

message GetLatestSnapshotResponse {
  Snapshot snapshot = 1;
}

message EventData {
  string event_type = 1;
  bytes payload_json = 2;
  EventMetadata metadata = 3;
}

message StoredEvent {
  string stream_id = 1;
  uint64 sequence = 2;
  string event_type = 3;
  int64 version = 4;
  bytes payload_json = 5;
  EventMetadata metadata = 6;
  string occurred_at = 7;
  string stored_at = 8;
}

message EventMetadata {
  optional string actor_id = 1;
  optional string correlation_id = 2;
  optional string causation_id = 3;
}

message Snapshot {
  string id = 1;
  string stream_id = 2;
  int64 snapshot_version = 3;
  string aggregate_type = 4;
  bytes state_json = 5;
  string created_at = 6;
}
```

---

## Kafka メッセージング設計

### イベント公開トピック

イベント追記後、バックグラウンドタスクが `k1s0.system.eventstore.event.published.v1` トピックへ非同期転送する。プロジェクション更新やリードモデル構築のためのダウンストリームサービスがこのトピックを Consumer する。

**メッセージフォーマット**

```json
{
  "event_type": "EVENT_PUBLISHED",
  "stream_id": "order-order-001",
  "sequence": 1,
  "domain_event_type": "OrderPlaced",
  "version": 1,
  "payload": {
    "order_id": "order-001",
    "tenant_id": "tenant-abc",
    "total_amount": 3000
  },
  "metadata": {
    "actor_id": "user-001",
    "correlation_id": "corr_01JABCDEF1234567890",
    "causation_id": null
  },
  "occurred_at": "2026-02-23T10:00:00.000+00:00",
  "stored_at": "2026-02-23T10:00:00.012+00:00"
}
```

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.eventstore.event.published.v1` |
| キー | stream_id |
| パーティション戦略 | stream_id によるハッシュ分散（ストリーム内の順序保証） |
| 転送保証 | at-least-once（Kafka オフセットは正常 ACK 後にコミット） |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) の 4 レイヤー構成に従う。

```
domain（エンティティ・リポジトリインターフェース・ドメインサービス）
  ^
usecase（ビジネスロジック）
  ^
adapter（REST ハンドラー・gRPC ハンドラー）
  ^
infra（DB接続・Kafka Producer・設定ローダー）
```

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/model | `EventStream`, `StoredEvent`, `Snapshot` | エンティティ定義 |
| domain/repository | `EventStreamRepository`, `EventRepository`, `SnapshotRepository` | リポジトリトレイト |
| domain/service | `EventStoreDomainService` | バージョン競合判定・イベント検証ロジック |
| usecase | `AppendEventsUsecase`, `ReadEventsUsecase`, `ReadEventBySequenceUsecase`, `CreateSnapshotUsecase`, `GetLatestSnapshotUsecase`, `DeleteStreamUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infra/config | Config ローダー | config.yaml の読み込み |
| infra/persistence | `EventStreamPostgresRepository`, `EventPostgresRepository`, `SnapshotPostgresRepository` | PostgreSQL リポジトリ実装（append-only） |
| infra/messaging | `EventPublishedKafkaProducer` | Kafka プロデューサー（イベント転送） |

### ドメインモデル

#### EventStream

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | ストリーム ID（例: `order-order-001`） |
| `aggregate_type` | String | 集約の種別（例: `Order`） |
| `current_version` | i64 | 現在のバージョン番号 |
| `created_at` | DateTime\<Utc\> | 作成日時 |
| `updated_at` | DateTime\<Utc\> | 最終更新日時（最後のイベント追記日時） |

#### StoredEvent

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `stream_id` | String | 所属ストリーム ID |
| `sequence` | u64 | グローバルシーケンス番号（全ストリーム横断の連番） |
| `event_type` | String | ドメインイベント種別（例: `OrderPlaced`） |
| `version` | i64 | ストリーム内のバージョン番号（1始まり） |
| `payload` | serde_json::Value | イベントペイロード（任意の JSON） |
| `metadata` | EventMetadata | メタデータ（actor_id, correlation_id, causation_id） |
| `occurred_at` | DateTime\<Utc\> | イベント発生日時 |
| `stored_at` | DateTime\<Utc\> | DB への保存日時 |

#### Snapshot

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | String | スナップショット ID |
| `stream_id` | String | 対象ストリーム ID |
| `snapshot_version` | i64 | 対応するイベントのバージョン番号 |
| `aggregate_type` | String | 集約の種別 |
| `state` | serde_json::Value | 集約の状態（任意の JSON） |
| `created_at` | DateTime\<Utc\> | 作成日時 |

### 依存関係図

```
                    ┌─────────────────────────────────────────────────┐
                    │                    adapter 層                    │
                    │  ┌──────────────────────────────────────────┐   │
                    │  │ REST Handler (event_store_handler.rs)    │   │
                    │  │  healthz / readyz / metrics              │   │
                    │  │  append_events / read_events /           │   │
                    │  │  read_event_by_sequence /                │   │
                    │  │  create_snapshot / get_latest_snapshot / │   │
                    │  │  delete_stream                           │   │
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (event_store_grpc.rs)       │   │
                    │  │  AppendEvents / ReadEvents /             │   │
                    │  │  ReadEventBySequence /                   │   │
                    │  │  CreateSnapshot / GetLatestSnapshot      │   │
                    │  └──────────────────────┬───────────────────┘   │
                    └─────────────────────────┼───────────────────────┘
                                              │
                    ┌─────────────────────────▼───────────────────────┐
                    │                   usecase 層                    │
                    │  AppendEventsUsecase /                          │
                    │  ReadEventsUsecase /                            │
                    │  ReadEventBySequenceUsecase /                   │
                    │  CreateSnapshotUsecase /                        │
                    │  GetLatestSnapshotUsecase /                     │
                    │  DeleteStreamUsecase                            │
                    └─────────────────────────┬───────────────────────┘
                                              │
              ┌───────────────────────────────┼───────────────────────┐
              │                               │                       │
    ┌─────────▼──────┐              ┌─────────▼──────────────────┐   │
    │  domain/model   │              │ domain/repository          │   │
    │  EventStream,   │              │ EventStreamRepository      │   │
    │  StoredEvent,   │              │ EventRepository            │   │
    │  Snapshot       │              │ SnapshotRepository         │   │
    └────────────────┘              │ (trait)                    │   │
              │                     └──────────┬─────────────────┘   │
              │  ┌────────────────┐            │                     │
              └──▶ domain/service │            │                     │
                 │ EventStore    │            │                     │
                 │ DomainService │            │                     │
                 └────────────────┘            │                     │
                    ┌──────────────────────────┼─────────────────────┘
                    │                  infra 層  │
                    │  ┌──────────────┐  ┌─────▼──────────────────┐  │
                    │  │ Kafka        │  │ EventStream/Event/      │  │
                    │  │ Producer     │  │ Snapshot Postgres       │  │
                    │  │ (published)  │  │ Repository (x3)         │  │
                    │  └──────────────┘  └────────────────────────┘  │
                    │  ┌──────────────┐                              │
                    │  │ Config       │                              │
                    │  │ Loader       │                              │
                    │  └──────────────┘                              │
                    └────────────────────────────────────────────────┘
```

### DB スキーマ設計

```sql
-- event_store スキーマ
CREATE SCHEMA IF NOT EXISTS event_store;

-- ストリームテーブル
CREATE TABLE event_store.event_streams (
    id              TEXT        NOT NULL PRIMARY KEY,
    aggregate_type  TEXT        NOT NULL,
    current_version BIGINT      NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- イベントテーブル（Append-only: UPDATE/DELETE 禁止）
CREATE TABLE event_store.events (
    sequence        BIGINT      NOT NULL GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    stream_id       TEXT        NOT NULL REFERENCES event_store.event_streams(id),
    version         BIGINT      NOT NULL,
    event_type      TEXT        NOT NULL,
    payload         JSONB       NOT NULL,
    metadata        JSONB       NOT NULL DEFAULT '{}',
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    stored_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (stream_id, version)
);

CREATE INDEX idx_events_stream_id ON event_store.events (stream_id, version);

-- スナップショットテーブル
CREATE TABLE event_store.snapshots (
    id                TEXT        NOT NULL PRIMARY KEY,
    stream_id         TEXT        NOT NULL REFERENCES event_store.event_streams(id),
    snapshot_version  BIGINT      NOT NULL,
    aggregate_type    TEXT        NOT NULL,
    state             JSONB       NOT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_snapshots_stream_id ON event_store.snapshots (stream_id, snapshot_version DESC);
```

---

## 設定ファイル

### config.yaml（本番）

```yaml
app:
  name: "event-store"
  version: "0.1.0"
  environment: "production"

server:
  host: "0.0.0.0"
  port: 8080
  grpc_port: 9090

database:
  url: "postgresql://app:@postgres.k1s0-system.svc.cluster.local:5432/k1s0_system"
  schema: "event_store"
  max_connections: 20
  min_connections: 5
  connect_timeout_seconds: 5

kafka:
  brokers:
    - "kafka-0.messaging.svc.cluster.local:9092"
  security_protocol: "PLAINTEXT"
  topic_published: "k1s0.system.eventstore.event.published.v1"
  producer_acks: "all"
  producer_retries: 3

auth:
  jwks_url: "http://auth-server.k1s0-system.svc.cluster.local:8080/.well-known/jwks.json"

event_store:
  max_events_per_append: 100
  max_page_size: 200
```

---

## デプロイ

### Helm values

[helm設計.md](helm設計.md) のサーバー用 Helm Chart を使用する。event-store 固有の values は以下の通り。

```yaml
# values-event-store.yaml（infra/helm/services/system/event-store/values.yaml）
image:
  registry: harbor.internal.example.com
  repository: k1s0-system/event-store
  tag: ""

replicaCount: 2

container:
  port: 8080
  grpcPort: 9090

service:
  type: ClusterIP
  port: 80
  grpcPort: 9090

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 8
  targetCPUUtilizationPercentage: 70

kafka:
  enabled: true
  brokers: []

vault:
  enabled: true
  role: "system"
  secrets:
    - path: "secret/data/k1s0/system/event-store/database"
      key: "password"
      mountPath: "/vault/secrets/database-password"
```

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/event-store/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

---

## 詳細設計ドキュメント

- [system-event-store-server-実装設計.md](system-event-store-server-実装設計.md) -- 実装設計の詳細
- [system-event-store-server-デプロイ設計.md](system-event-store-server-デプロイ設計.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

- [RBAC設計.md](RBAC設計.md) -- RBAC ロールモデル
- [認証認可設計.md](認証認可設計.md) -- RBAC 認可モデル
- [API設計.md](API設計.md) -- REST API 設計ガイドライン
- [メッセージング設計.md](メッセージング設計.md) -- Kafka メッセージング設計
- [可観測性設計.md](可観測性設計.md) -- メトリクス・トレース設計
- [テンプレート仕様-サーバー.md](テンプレート仕様-サーバー.md) -- サーバーテンプレート仕様
- [コーディング規約.md](コーディング規約.md) -- コーディング規約
- [system-server設計.md](system-server設計.md) -- system tier サーバー一覧
- [system-server-実装設計.md](system-server-実装設計.md) -- system tier 実装設計
- [tier-architecture.md](tier-architecture.md) -- Tier アーキテクチャ
- [helm設計.md](helm設計.md) -- Helm Chart・Vault Agent Injector
- [system-library-概要.md](system-library-概要.md) -- ライブラリ一覧（eventstore ライブラリ）
