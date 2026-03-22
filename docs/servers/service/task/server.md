# service-task-server 設計

service tier のタスク管理サーバー設計を定義する。タスクの作成・照会・ステータス管理を REST API で提供し、タスクイベントを Kafka に非同期配信する。
Rust で実装する。

## 概要

### RBAC対応表

| ロール名 | リソース/アクション |
|---------|-----------------|
| task:read | タスク一覧取得・単体取得 |
| task:write | タスク作成・更新・ステータス遷移・キャンセル |

Tier: `Tier::Service`。JWKS ベースの JWT 認証と、`require_permission(Tier::Service, "task", action)` による権限チェックを行う。

service tier のタスク管理サーバーは以下の機能を提供する。

| 機能 | 説明 |
| --- | --- |
| タスク作成 API | プロジェクト ID・担当者・優先度・期限を指定してタスクを作成する |
| タスク一覧取得 API | プロジェクト ID・ステータス・担当者・優先度によるフィルタリング付きの一覧を取得する |
| タスク詳細取得 API | タスク ID を指定してタスクとチェックリストを取得する |
| ステータス遷移 API | タスクステータスのステートマシンに従った遷移を行う（Open→InProgress→Review→Done/Cancelled） |
| チェックリスト管理 API | タスクに紐づくチェックリストアイテムの作成・更新・削除を行う |
| タスクイベント配信 | Kafka トピックへのタスク作成・更新・キャンセルイベントの非同期配信（Outbox pattern） |

### 技術スタック

> 共通技術スタックは [テンプレート仕様-サーバー.md](../../templates/server/サーバー.md#共通技術スタック) を参照。

| コンポーネント | Rust |
| --- | --- |
| Kafka クライアント | rdkafka v0.37 |
| バリデーション | validator v0.18 |

### 配置パス

配置: `regions/service/task/server/rust/task/`（[Tier別配置パス参照](../../templates/server/サーバー.md#tier-別配置パス)）

---

## API 定義

### REST API エンドポイント

全エンドポイントは [API設計.md](../../architecture/api/API設計.md) D-007 の統一エラーレスポンスに従う。エラーコードのプレフィックスは `SVC_TASK_` とする。

| Method | Path | Description | 認可 |
| --- | --- | --- | --- |
| POST | `/api/v1/tasks` | タスク作成 | `task:write` |
| GET | `/api/v1/tasks` | タスク一覧取得（フィルター付き） | `task:read` |
| GET | `/api/v1/tasks/{task_id}` | タスク詳細取得（チェックリスト含む） | `task:read` |
| PUT | `/api/v1/tasks/{task_id}` | タスク更新（タイトル・説明・担当者・優先度・期限） | `task:write` |
| PUT | `/api/v1/tasks/{task_id}/status` | タスクステータス遷移 | `task:write` |
| POST | `/api/v1/tasks/{task_id}/checklist` | チェックリストアイテム追加 | `task:write` |
| PUT | `/api/v1/tasks/{task_id}/checklist/{item_id}` | チェックリストアイテム更新 | `task:write` |
| DELETE | `/api/v1/tasks/{task_id}/checklist/{item_id}` | チェックリストアイテム削除 | `task:write` |
| GET | `/healthz` | ヘルスチェック | 不要（公開） |
| GET | `/readyz` | レディネスチェック | 不要（公開） |
| GET | `/metrics` | Prometheus メトリクス | 不要（公開） |

#### POST /api/v1/tasks

タスクを作成する。初期ステータスは `open`。

**リクエスト**

```json
{
  "project_id": "PROJECT-001",
  "title": "ログイン機能の実装",
  "description": "JWT ベースのログイン機能を実装する",
  "assignee_id": "user-uuid",
  "priority": "high",
  "due_date": "2026-04-30"
}
```

| フィールド | 型 | 必須 | 説明 |
| --- | --- | --- | --- |
| `project_id` | string | Yes | 所属プロジェクト ID |
| `title` | string | Yes | タスクタイトル（1文字以上） |
| `description` | string | No | タスク説明 |
| `assignee_id` | string | No | 担当者 ID |
| `priority` | string | Yes | 優先度（`low`, `medium`, `high`, `critical`） |
| `due_date` | string | No | 期限日（ISO 8601 日付形式） |

**レスポンス（201 Created）**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "project_id": "PROJECT-001",
  "title": "ログイン機能の実装",
  "description": "JWT ベースのログイン機能を実装する",
  "status": "open",
  "assignee_id": "user-uuid",
  "priority": "high",
  "due_date": "2026-04-30",
  "checklist_items": [],
  "created_by": "admin@example.com",
  "version": 1,
  "created_at": "2026-03-22T00:00:00+00:00",
  "updated_at": "2026-03-22T00:00:00+00:00"
}
```

#### PUT /api/v1/tasks/{task_id}/status

タスクステータスを遷移させる。ステートマシンのルールに従い、不正な遷移は拒否される。

**リクエスト**

```json
{
  "status": "in_progress"
}
```

**レスポンス（400 Bad Request — 不正なステータス遷移）**

```json
{
  "error": {
    "code": "SVC_TASK_INVALID_STATUS_TRANSITION",
    "message": "invalid status transition: 'done' -> 'open'",
    "request_id": "req_abc123def456",
    "details": []
  }
}
```

#### GET /healthz

**レスポンス（200 OK）**

```json
{
  "status": "ok"
}
```

#### GET /readyz

PostgreSQL への接続を確認する。

**レスポンス（200 OK）**

```json
{
  "status": "ready",
  "checks": {
    "database": "ok"
  }
}
```

---

## タスクステータス ステートマシン

```
              ┌──────────────────────────────────┐
              │                                  │
              ▼                                  │
┌──────┐   ┌─────────────┐   ┌────────┐   ┌──────┐
│ open  │──>│  in_progress │──>│ review │──>│ done │
└──┬───┘   └──────┬───────┘   └───┬────┘   └──────┘
   │               │               │
   │               │               │
   ▼               ▼               ▼
┌─────────────────────────────────────┐
│             cancelled                │
└─────────────────────────────────────┘
```

| 遷移元 | 遷移先 |
| --- | --- |
| open | in_progress, cancelled |
| in_progress | review, cancelled |
| review | done, in_progress, cancelled |

`done` と `cancelled` は終端ステータスであり、他のステータスへ遷移できない。

---

## タスク優先度

| 優先度 | 説明 |
| --- | --- |
| `low` | 低優先度（通常業務範囲） |
| `medium` | 中優先度（標準的なタスク） |
| `high` | 高優先度（重要度の高いタスク） |
| `critical` | 最高優先度（緊急対応が必要） |

---

## エラーコード

| エラーコード | HTTP Status | 説明 |
| --- | --- | --- |
| `SVC_TASK_NOT_FOUND` | 404 | 指定されたタスクが見つからない |
| `SVC_TASK_VALIDATION_FAILED` | 400 | リクエストのバリデーションエラー |
| `SVC_TASK_INVALID_STATUS_TRANSITION` | 400 | 不正なステータス遷移 |
| `SVC_TASK_VERSION_CONFLICT` | 409 | 楽観的ロックによるバージョン競合 |
| `SVC_TASK_CHECKLIST_ITEM_NOT_FOUND` | 404 | チェックリストアイテムが見つからない |
| `SVC_TASK_INTERNAL_ERROR` | 500 | 内部サーバーエラー |

---

## Kafka イベント

タスクのライフサイクルイベントを Kafka トピックに非同期配信する。Outbox pattern を採用し、タスク操作と同一トランザクションで outbox_events テーブルにイベントを書き込む。

| トピック | イベント | トリガー |
| --- | --- | --- |
| `k1s0.service.task.created.v1` | task.created | タスク作成時 |
| `k1s0.service.task.updated.v1` | task.updated | タスク更新・ステータス遷移時（cancelled 以外） |
| `k1s0.service.task.cancelled.v1` | task.cancelled | ステータスが cancelled に遷移時 |

### イベントペイロード例

**task.created**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440000",
    "event_type": "task.created",
    "source": "task-server",
    "timestamp": 1742601600000,
    "trace_id": "",
    "correlation_id": "660e8400-e29b-41d4-a716-446655440111",
    "schema_version": 1
  },
  "task_id": "660e8400-e29b-41d4-a716-446655440111",
  "project_id": "PROJECT-001",
  "title": "ログイン機能の実装",
  "status": "open",
  "priority": "high",
  "assignee_id": "user-uuid"
}
```

**task.updated**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440001",
    "event_type": "task.updated",
    "source": "task-server",
    "timestamp": 1742601600000,
    "trace_id": "",
    "correlation_id": "660e8400-e29b-41d4-a716-446655440111",
    "schema_version": 1
  },
  "task_id": "660e8400-e29b-41d4-a716-446655440111",
  "user_id": "admin@example.com",
  "status": "in_progress"
}
```

**task.cancelled**

```json
{
  "metadata": {
    "event_id": "550e8400-e29b-41d4-a716-446655440002",
    "event_type": "task.cancelled",
    "source": "task-server",
    "timestamp": 1742601600000,
    "trace_id": "",
    "correlation_id": "660e8400-e29b-41d4-a716-446655440111",
    "schema_version": 1
  },
  "task_id": "660e8400-e29b-41d4-a716-446655440111",
  "user_id": "admin@example.com",
  "status": "cancelled",
  "reason": "status changed to cancelled"
}
```

---

## 設定フィールド

### server

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | `0.0.0.0` | バインドアドレス |
| `port` | int | `8080` | REST API ポート（コンテナ内） |
| `grpc_port` | int | `50051` | gRPC ポート（コンテナ内） |

### database

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `host` | string | - | PostgreSQL ホスト |
| `port` | int | `5432` | PostgreSQL ポート |
| `name` | string | - | データベース名 |
| `schema` | string | `task_service` | スキーマ名 |
| `user` | string | - | 接続ユーザー |
| `password` | string | `""` | パスワード |
| `ssl_mode` | string | `disable` | SSL モード |
| `max_connections` | int | `25` | 最大接続数 |

### kafka

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `brokers` | string[] | Kafka ブローカーアドレス一覧 |
| `task_created_topic` | string | タスク作成イベントのトピック名 |
| `task_updated_topic` | string | タスク更新イベントのトピック名 |
| `task_cancelled_topic` | string | タスクキャンセルイベントのトピック名 |

### auth

| フィールド | 型 | デフォルト | 説明 |
| --- | --- | --- | --- |
| `jwks_url` | string | - | JWKS エンドポイント URL |
| `issuer` | string | - | JWT issuer |
| `audience` | string | - | JWT audience |
| `jwks_cache_ttl_secs` | int | `300` | JWKS キャッシュ TTL（秒） |

---

## アーキテクチャ

### クリーンアーキテクチャ レイヤー

[テンプレート仕様-サーバー.md](../../templates/server/サーバー.md) の4レイヤー構成に従う。

| レイヤー | パッケージ / モジュール | 責務 |
| --- | --- | --- |
| domain/entity | `Task`, `TaskChecklistItem`, `TaskStatus`, `TaskPriority`, `CreateTask`, `TaskFilter` | エンティティ・値オブジェクト定義 |
| domain/error | `TaskError` | ドメイン固有エラー型（`thiserror` ベース） |
| domain/repository | `TaskRepository` | リポジトリトレイト |
| domain/service | `TaskDomainService` | ドメインサービス（バリデーション・ステータス遷移検証） |
| usecase | `CreateTaskUseCase`, `GetTaskUseCase`, `UpdateTaskStatusUseCase`, `ListTasksUseCase`, `ManageChecklistUseCase` | ユースケース |
| usecase | `TaskEventPublisher` | イベント発行トレイト |
| adapter/handler | REST ハンドラー + ルーティング | プロトコル変換 |
| adapter/presenter | `TaskDetailResponse`, `TaskListResponse`, `TaskSummaryResponse` | ドメインモデル → API レスポンス変換 |
| adapter/middleware | `auth_middleware`, `require_permission` | JWT 認証・RBAC ミドルウェア |
| infrastructure/database | `TaskPostgresRepository` | PostgreSQL リポジトリ実装 |
| infrastructure/config | Config ローダー | config.yaml の読み込みとバリデーション |
| infrastructure/kafka | `TaskKafkaProducer` | Kafka プロデューサー（タスクイベント配信） |

### ドメインモデル

#### Task

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | タスクの一意識別子 |
| `project_id` | string | 所属プロジェクト ID |
| `title` | string | タスクタイトル |
| `description` | string? | タスク説明 |
| `status` | TaskStatus | タスクステータス（`open`, `in_progress`, `review`, `done`, `cancelled`） |
| `assignee_id` | string? | 担当者 ID |
| `priority` | TaskPriority | 優先度（`low`, `medium`, `high`, `critical`） |
| `due_date` | date? | 期限日 |
| `created_by` | string | 作成者 |
| `updated_by` | string? | 最終更新者 |
| `version` | i32 | バージョン番号（楽観的排他制御用） |
| `created_at` | timestamp | 作成日時 |
| `updated_at` | timestamp | 更新日時 |

#### TaskChecklistItem

| フィールド | 型 | 説明 |
| --- | --- | --- |
| `id` | UUID | チェックリストアイテムの一意識別子 |
| `task_id` | UUID | 親タスクの ID（FK） |
| `title` | string | アイテムタイトル |
| `is_completed` | bool | 完了フラグ |
| `sort_order` | i32 | 表示順 |
| `created_at` | timestamp | 作成日時 |
| `updated_at` | timestamp | 更新日時 |

---

## 詳細設計ドキュメント

実装・データベースの詳細は以下の分割ドキュメントを参照。

- [implementation.md](implementation.md) -- Rust 実装詳細（Cargo.toml・ドメイン・リポジトリ・ユースケース・ハンドラー）
- [database.md](database.md) -- データベーススキーマ・マイグレーション・ER 図

---

## クライアント実装

| プラットフォーム | 配置パス | 技術スタック |
|----------------|---------|-------------|
| React | `regions/service/task/client/react/task/` | TanStack Query + Router, Zod, Axios |
| Flutter | `regions/service/task/client/flutter/task/` | Riverpod, go_router, Dio |

両クライアントとも BFF 経由で本サーバーの REST API を呼び出す。

## ObservabilityConfig（log/trace/metrics）

本サーバーの observability 設定は共通仕様を採用する。log / trace / metrics の構造と推奨値は [共通実装](../_common/implementation.md) の「ObservabilityConfig（log/trace/metrics）」を参照。
