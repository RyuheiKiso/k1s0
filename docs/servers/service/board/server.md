# service-board-server 設計

service tier のボード管理サーバー設計を定義する。Kanban ボードのカラム管理・WIP（Work In Progress）制限を REST/gRPC API で提供し、カラム更新イベントを Kafka に非同期配信する。
Rust で実装する。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| board:read | ボードカラム一覧取得・単体取得 |
| board:write | タスク数増減・WIP制限更新 |

Tier: `Tier::Service`。JWKS ベースの JWT 認証と、`require_permission(Tier::Service, "board", action)` による権限チェックを行う。

service tier のボード管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| タスク数増加 API | カラムのタスク数を +1 する（タスク移動時・WIP制限チェック付き） |
| タスク数減少 API | カラムのタスク数を -1 する（タスク完了・移動時） |
| カラム取得 API | 特定プロジェクトの特定ステータスのカラムを取得する |
| カラム一覧取得 API | プロジェクトの全カラムを取得する |
| WIP制限更新 API | カラムの WIP 制限値を更新する |
| カラム更新イベント配信 | Kafka トピックへのカラム更新イベントの非同期配信（Outbox pattern） |

### WIP制限ルール

- `wip_limit = 0` の場合は制限なし（無制限）
- `wip_limit > 0` かつ `task_count >= wip_limit` の場合は `WipLimitExceeded` エラー（409 Conflict）
- increment 操作時のみチェック（decrement は常に許可）

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| Kafka クライアント | rdkafka v0.37 |
| バリデーション | validator v0.18 |

### 配置パス

配置: `regions/service/board/server/rust/board/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SVC_BOARD_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/boards/increment` | カラムのタスク数 +1 | `board:write` |
| POST | `/api/v1/boards/decrement` | カラムのタスク数 -1 | `board:write` |
| GET | `/api/v1/boards/{project_id}/columns` | カラム一覧取得 | `board:read` |
| GET | `/api/v1/boards/{project_id}/columns/{status_code}` | カラム単体取得 | `board:read` |
| PUT | `/api/v1/boards/{project_id}/columns/{status_code}/wip-limit` | WIP制限更新 | `board:write` |
| GET | `/healthz` | ヘルスチェック | 不要（公開） |
| GET | `/readyz` | レディネスチェック | 不要（公開） |
| GET | `/metrics` | Prometheus メトリクス | 不要（公開） |

---

## エラーコード

| エラーコード | HTTP Status | 説明 |
| --- | --- | --- |
| `SVC_BOARD_NOT_FOUND` | 404 | 指定されたカラムが見つからない |
| `SVC_BOARD_WIP_LIMIT_EXCEEDED` | 409 | WIP制限超過（カラムが満杯） |
| `SVC_BOARD_VALIDATION_FAILED` | 400 | リクエストのバリデーションエラー |
| `SVC_BOARD_VERSION_CONFLICT` | 409 | 楽観的ロックによるバージョン競合 |
| `SVC_BOARD_INTERNAL_ERROR` | 500 | 内部サーバーエラー |

---

## Kafka イベント

ボードカラム更新イベントを Outbox pattern で Kafka トピックに非同期配信する。

| トピック | イベント | トリガー |
| --- | --- | --- |
| `k1s0.service.board.column_updated.v1` | board.column_updated | タスク数変更・WIP制限更新時 |

---

## 設定フィールド

### server

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | `0.0.0.0` | バインドアドレス |
| `port` | int | `8320` | REST API ポート |
| `grpc_port` | int | `9320` | gRPC ポート |

### database

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `schema` | string | `board_service` | スキーマ名 |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

| レイヤー | 主要モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `BoardColumn` | エンティティ（WIP制限チェックロジック含む） |
| domain/repository | `BoardColumnRepository` | リポジトリトレイト |
| domain/error | `BoardError` (WipLimitExceeded, etc.) | ドメインエラー型 |
| usecase | `IncrementColumnUseCase`, `DecrementColumnUseCase`, `GetBoardColumnUseCase`, `ListBoardColumnsUseCase`, `UpdateWipLimitUseCase` | ユースケース |
| usecase | `BoardEventPublisher` | イベント発行トレイト |
| adapter/handler | REST ハンドラー + gRPC サービス | プロトコル変換 |
| infrastructure/persistence | `BoardColumnPostgresRepository` | PostgreSQL + Outbox |
| infrastructure/messaging | `BoardKafkaProducer` | Kafka プロデューサー |

---

## クライアント実装

| プラットフォーム | 配置パス | 技術スタック |
|----------------|---------|-------------|
| React | `regions/service/board/client/react/board/` | TanStack Query + Router, Zod, Axios |
| Flutter | `regions/service/board/client/flutter/board/` | Riverpod, go_router, Dio |

## 詳細設計ドキュメント

- [service-board-server-implementation.md](implementation.md) -- Rust 実装詳細
- [service-board-database.md](database.md) -- データベーススキーマ・マイグレーション
