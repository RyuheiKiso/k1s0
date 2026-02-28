# system-event-store-server 設計

> **ガイド**: 設計背景・実装例は [server.guide.md](./server.guide.md) を参照。

system tier の CQRS パターン向けイベントソーシングサーバー。k1s0-eventstore ライブラリをサービス化し、Append-only イベントストリームの REST/gRPC API を提供する。Rust で実装する。

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

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

### 配置パス

配置: `regions/system/server/rust/event-store/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## 設計方針

[認証認可設計.md](../../architecture/auth/認証認可設計.md) の RBAC モデルに基づき、以下の方針で実装する。

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

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SYS_EVSTORE_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/events` | イベント追記（`stream_id` はリクエストボディで指定） | `sys_operator` 以上 |
| GET | `/api/v1/events` | 全イベント一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/events/:stream_id` | ストリーム別イベント一覧取得 | `sys_auditor` 以上 |
| GET | `/api/v1/streams` | ストリーム一覧取得 | `sys_auditor` 以上 |
| POST | `/api/v1/streams/:stream_id/snapshot` | スナップショット作成 | `sys_operator` 以上 |
| GET | `/api/v1/streams/:stream_id/snapshot` | 最新スナップショット取得 | `sys_auditor` 以上 |
| DELETE | `/api/v1/streams/:stream_id` | ストリーム削除（監査・テスト用途に限定） | `sys_admin` のみ |
| GET | `/healthz` | ヘルスチェック | 不要 |
| GET | `/readyz` | レディネスチェック | 不要 |

#### POST /api/v1/events

イベントをストリームに追記する。`expected_version` を指定することで楽観的ロックを実現する。`expected_version` が `-1` の場合はストリームが存在しないことを期待する（新規ストリーム作成）。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `events` | array | Yes | 追記するイベントの配列 |
| `events[].event_type` | string | Yes | ドメインイベント種別 |
| `events[].payload` | object | Yes | イベントペイロード |
| `events[].metadata` | object | No | メタデータ（actor_id, correlation_id, causation_id） |
| `expected_version` | int | Yes | 楽観的ロック用バージョン（-1 = 新規ストリーム） |

> JSON 例は [server.guide.md](./server.guide.md#post-apiv1events) を参照。

#### GET /api/v1/events/:stream_id

ストリームのイベント一覧をページネーション付きで取得する。

**クエリパラメータ**

| パラメータ | 型 | 必須 | デフォルト | 説明 |
| --- | --- | --- | --- | --- |
| `from_version` | int | No | 1 | 取得開始バージョン（含む） |
| `to_version` | int | No | - | 取得終了バージョン（含む）。省略時は最新まで |
| `event_type` | string | No | - | イベント種別でフィルタ |
| `page` | int | No | 1 | ページ番号 |
| `page_size` | int | No | 50 | 1 ページあたりの件数（最大 200） |

> JSON 例は [server.guide.md](./server.guide.md#get-apiv1eventsstream_id) を参照。

#### GET /api/v1/events

全ストリームのイベントを一覧取得する。ページネーション付き。

> JSON 例は [server.guide.md](./server.guide.md#get-apiv1events) を参照。

#### GET /api/v1/streams

登録済みストリームの一覧を取得する。

> JSON 例は [server.guide.md](./server.guide.md#get-apiv1streams) を参照。

#### POST /api/v1/streams/:stream_id/snapshot

集約の現在状態をスナップショットとして保存する。

**リクエストフィールド**

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `snapshot_version` | int | Yes | 対応するイベントのバージョン番号 |
| `aggregate_type` | string | Yes | 集約の種別 |
| `state` | object | Yes | 集約の状態（JSON） |

> JSON 例は [server.guide.md](./server.guide.md#post-apiv1streamsstream_idsnapshot) を参照。

#### GET /api/v1/streams/:stream_id/snapshot

ストリームの最新スナップショットを取得する。スナップショットが存在しない場合は `404` を返す。

> JSON 例は [server.guide.md](./server.guide.md#get-apiv1streamsstream_idsnapshot) を参照。

#### DELETE /api/v1/streams/:stream_id

ストリームとそれに紐づく全イベント・スナップショットを削除する。`sys_admin` のみ実行可能。イベントソーシングでは原則として使用しない（監査・テスト用途に限定する）。

> JSON 例は [server.guide.md](./server.guide.md#delete-apiv1streamsstream_id) を参照。

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

| 設定項目 | 値 |
| --- | --- |
| トピック | `k1s0.system.eventstore.event.published.v1` |
| キー | stream_id |
| パーティション戦略 | stream_id によるハッシュ分散（ストリーム内の順序保証） |
| 転送保証 | at-least-once（Kafka オフセットは正常 ACK 後にコミット） |

> Kafka メッセージフォーマットは [server.guide.md](./server.guide.md#kafka-メッセージフォーマット) を参照。

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の 4 レイヤー構成に従う。

| レイヤー | モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `EventStream`, `StoredEvent`, `Snapshot` | エンティティ定義 |
| domain/repository | `EventStreamRepository`, `EventRepository`, `SnapshotRepository` | リポジトリトレイト |
| domain/service | `EventStoreDomainService` | バージョン競合判定・イベント検証ロジック |
| usecase | `AppendEventsUsecase`, `ReadEventsUsecase`, `ReadEventBySequenceUsecase`, `CreateSnapshotUsecase`, `GetLatestSnapshotUsecase`, `DeleteStreamUsecase` | ユースケース |
| adapter/handler | REST ハンドラー（axum）, gRPC ハンドラー（tonic） | プロトコル変換 |
| infrastructure/config | Config ローダー | config.yaml の読み込み |
| infrastructure/persistence | `EventStreamPostgresRepository`, `EventPostgresRepository`, `SnapshotPostgresRepository` | PostgreSQL リポジトリ実装（append-only） |
| infrastructure/messaging | `EventPublishedKafkaProducer` | Kafka プロデューサー（イベント転送） |

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
                    │  ├──────────────────────────────────────────┤   │
                    │  │ gRPC Handler (event_store_grpc.rs)       │   │
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
    │  domain/entity  │              │ domain/repository          │   │
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
                    │             infrastructure 層  │
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

| テーブル | 用途 | 特記事項 |
| --- | --- | --- |
| `event_store.event_streams` | ストリーム管理 | PK: `id (TEXT)`, current_version で楽観的ロック |
| `event_store.events` | イベント保存 | Append-only（UPDATE/DELETE 禁止）、UNIQUE(stream_id, version)、sequence は IDENTITY |
| `event_store.snapshots` | スナップショット | PK: `id (TEXT)`, INDEX(stream_id, snapshot_version DESC) |

> DDL 全文は [server.guide.md](./server.guide.md#db-スキーマ-ddl) を参照。

---

## デプロイ

### Vault シークレットパス

| シークレット | パス |
| --- | --- |
| DB パスワード | `secret/data/k1s0/system/event-store/database` |
| Kafka SASL | `secret/data/k1s0/system/kafka/sasl` |

> Helm values・config.yaml の例は [server.guide.md](./server.guide.md#設定ファイル例) を参照。

---

## 詳細設計ドキュメント

- [system-event-store-server-implementation.md](../_common/implementation.md) -- 実装設計の詳細
- [system-event-store-server-deploy.md](../_common/deploy.md) -- デプロイ設計の詳細

---

## 関連ドキュメント

> 共通関連ドキュメントは [deploy.md](../_common/deploy.md#共通関連ドキュメント) を参照。

- [RBAC設計.md](../../architecture/auth/RBAC設計.md) -- RBAC ロールモデル
- [system-server.md](../auth/server.md) -- system tier サーバー一覧
- [system-server-implementation.md](../_common/implementation.md) -- system tier 実装設計
